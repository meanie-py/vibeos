"""QEMU VM lifecycle manager for ReactOS test sessions."""

from __future__ import annotations

import atexit
import fcntl
import json
import os
import shutil
import signal
import socket
import socketserver
import subprocess
import tempfile
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path

from .qmp_client import QmpClient
from .serial_client import SerialClient

# Project root: os-playground/
PROJECT_DIR = Path(__file__).resolve().parent.parent.parent  # vibeos/ repo root
QEMU_DIR = PROJECT_DIR / "qemu"

# --------------------------------------------------------------------------- #
# QEMU lock files — SAFE, SCOPED orphan reaping.
#
# Multiple agents/worktrees run smoke VMs in parallel. A blanket
# `pkill -f qemu-system` would murder another agent's live VM. Instead every
# QEMU we launch drops a lock file recording OUR launcher pid + the qemu pid.
# Reaping only kills a QEMU whose *launcher process is dead* (a crashed/timed-
# out run = a true orphan). A QEMU whose launcher is still alive belongs to a
# running session (possibly another agent) and is NEVER touched.
# --------------------------------------------------------------------------- #
LOCK_DIR = Path("/tmp/vibeos-qemu-locks")
_CLK = os.sysconf("SC_CLK_TCK")

# Hard wall-clock ceiling on any single QEMU we launch. The longest legitimate
# run is a cold-cache arm64 TCG boot (~5 min) plus a 120s desktop settle — well
# under this. The cap exists to guarantee a wedged guest, a hung test, or a
# deadlocked QMP can NEVER hold a KVM slot / GBs of RAM indefinitely. Enforced
# three ways for defence in depth:
#   1. an in-process threading.Timer that kills the VM the instant it fires;
#   2. a "deadline" field in each lock file, so ANY reaper invocation (every
#      smoke session start, every qemu/start.sh) culls a VM past its deadline
#      even if the owning launcher is still alive but wedged;
#   3. (qemu/start.sh) a detached watchdog subshell for toolkit-launched VMs.
MAX_VM_LIFETIME_SECONDS = 30 * 60

# Predictable, per-arch live serial log. Agents `tail -f` this to watch boot
# progress in real time (a cold arm64 TCG boot is silent for long stretches —
# the live log is the difference between "making progress" and "wedged").
# qemu/*.log is gitignored.
LIVE_LOG_DIR = QEMU_DIR

# Global concurrency cap: at most this many QEMUs running at once across ALL
# worktrees/sessions. Mirrors qemu/lib/common.sh:QEMU_MAX_CONCURRENT. Enforced
# with a shared pool of flock'd slot files — a launcher that can't get a slot
# QUEUES rather than piling another VM onto an already-saturated host. The time
# spent queued is excluded from the 30-min lifetime cap: we acquire the slot
# BEFORE recording the VM's start/deadline, so queue wait never shortens the
# VM's allowed runtime.
QEMU_MAX_CONCURRENT = int(os.environ.get("VIBEOS_QEMU_MAX_CONCURRENT", "10"))
QEMU_QUEUE_TIMEOUT = float(os.environ.get("VIBEOS_QEMU_QUEUE_TIMEOUT", "1800"))
SLOT_DIR = LOCK_DIR / "slots"


def per_arch_cap(arch: str) -> int:
    """Same-arch concurrency sub-cap, layered under the global pool. Mirrors
    qemu/lib/common.sh:qemu_per_arch_cap. The global cap bounds total host load;
    this serializes SAME-ARCH boots, which contend on /dev/kvm + the shared IDE
    path the global count is blind to (two i386/amd64 KVM guests starve each
    other into false HANG_TIMEOUTs). KVM arches default to 1 (strict serialize);
    arm64 (TCG, no accelerator contention) defaults to 0 = uncapped."""
    base = arch.replace("-uefi", "").replace("-test", "")
    if base == "i386":
        return int(
            os.environ.get("VIBEOS_QEMU_CAP_I386", os.environ.get("VIBEOS_QEMU_PER_ARCH_CAP", "1"))
        )
    if base == "amd64":
        return int(
            os.environ.get("VIBEOS_QEMU_CAP_AMD64", os.environ.get("VIBEOS_QEMU_PER_ARCH_CAP", "1"))
        )
    if base == "arm64":
        return int(os.environ.get("VIBEOS_QEMU_CAP_ARM64", "0"))
    return 0


def process_cpu_pct(pid: int, window: float = 3.0) -> float:
    """Percent of ONE core the qemu process (all threads) burns over
    `window`s. Under -accel kvm a HLT-idle guest blocks the vCPU thread
    (~0-6%); a spinning/starved guest pegs ~90%+. This is the signal that
    catches "boots green but pathological" regressions (e.g. the serial-
    backpressure starvation) that every other smoke check misses.
    """

    def jiffies() -> int:
        total = 0
        for t in Path(f"/proc/{pid}/task").iterdir():
            try:
                p = (t / "stat").read_text().rsplit(")", 1)[1].split()
                total += int(p[11]) + int(p[12])
            except (OSError, IndexError):
                pass
        return total

    j0 = jiffies()
    time.sleep(window)
    j1 = jiffies()
    return round((j1 - j0) / _CLK / window * 100.0, 1)


def _pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True  # exists, just not ours to signal
    return True


def _proc_is_qemu(pid: int) -> bool:
    try:
        return "qemu" in Path(f"/proc/{pid}/comm").read_text()
    except OSError:
        return False


def reap_orphan_qemus() -> list[str]:
    """Kill ONLY QEMUs whose launcher process is gone. Returns action log.

    Safe to call from any session: a lock whose launcher pid is still alive
    is left strictly untouched (it may be another agent's live VM).
    """
    actions: list[str] = []
    if not LOCK_DIR.is_dir():
        return actions
    for lf in LOCK_DIR.glob("*.json"):
        try:
            info = json.loads(lf.read_text())
        except (OSError, ValueError):
            lf.unlink(missing_ok=True)
            continue
        lpid, qpid = info.get("launcher_pid"), info.get("qemu_pid")
        deadline = info.get("deadline")
        launcher_alive = bool(lpid) and _pid_alive(lpid)
        # A VM past its 30-min hard cap is reaped REGARDLESS of launcher
        # liveness — the cap is an absolute ceiling, not an orphan check. A
        # live launcher only protects a VM that's still within its lifetime.
        expired = bool(deadline) and time.time() >= deadline
        if launcher_alive and not expired:
            continue  # live session (maybe another agent), within cap — DO NOT TOUCH
        if qpid and _pid_alive(qpid) and _proc_is_qemu(qpid):
            reason = "exceeded 30-min lifetime cap" if expired else f"launcher {lpid} dead"
            try:
                os.killpg(os.getpgid(qpid), signal.SIGTERM)
                time.sleep(2)
                if _pid_alive(qpid):
                    os.killpg(os.getpgid(qpid), signal.SIGKILL)
                actions.append(f"reaped qemu pid={qpid} ({reason})")
            except (ProcessLookupError, OSError) as e:
                actions.append(f"qemu pid={qpid} reap failed: {e}")
                continue  # leave the lock; don't claim it's gone if kill failed
        lf.unlink(missing_ok=True)
    return actions


@dataclass
class ArchConfig:
    arch: str
    qemu_bin: str
    machine_args: list[str]
    memory: str
    boot_marker: str
    boot_timeout: float = 120.0
    min_boot_bytes: int = 1000
    qmp_timeout: float = 10.0
    # How long to wait after boot_marker for the desktop to come up.
    # The boot marker proves the kernel booted; this extra time lets
    # user-mode processes (SMSS, csrss, winlogon, explorer) finish.
    desktop_settle_seconds: float = 0.0
    # Milestones expected to appear in the boot output (substring match).
    # Validated by test_milestones after wait_for_boot completes.
    # Order is informational — we report which milestones were reached / missed.
    required_milestones: list[str] = field(default_factory=list)

    def qemu_args(self, work_dir: Path) -> list[str]:
        """Build full QEMU command line. Subclasses override via callbacks."""
        raise NotImplementedError


@dataclass
class Arm64Config(ArchConfig):
    arch: str = "arm64"
    qemu_bin: str = "qemu-system-aarch64"
    machine_args: list[str] = field(
        default_factory=lambda: [
            "-M",
            "virt",
            "-cpu",
            "cortex-a72",
            "-accel",
            "tcg,thread=multi",
            # virtio-mmio kbd+mouse — the devices arm64input.rs binds to
            # (scans virtio-mmio transports at 0x0a000000 + i*0x200 for the
            # virtio-input device id). Without these the poll thread polls
            # empty virtqueues and the input stack has no functional coverage.
            # `*-device` (not `*-pci`) is the virtio-mmio transport variant.
            "-device",
            "virtio-keyboard-device,id=arm64kbd",
            "-device",
            "virtio-mouse-device,id=arm64mouse",
            # QEMU `virt` defaults its virtio-mmio transports to legacy (v1).
            # arm64input's modern (v2) split-queue path is the well-exercised
            # one; force modern so smoke drives it. Modern virtio-mmio is what
            # real ARM64 virtio environments use — a representative, non-
            # weakening host-config choice (the legacy queue path remains a
            # tracked driver follow-up, not a smoke blocker).
            "-global",
            "virtio-mmio.force-legacy=false",
        ]
    )
    memory: str = "1G"
    # Wait for SMSS user-mode dispatch to be set up. This proves the kernel
    # got to user-mode entry — currently the highest reliably-reached point
    # on ARM64. SMSS doesn't currently reach its _main (LdrInitializeThunk
    # appears to fail silently). Idle detection alone is too weak — every
    # silent kernel passes.
    # On arm64 we emulate via TCG (no KVM in WSL2 for aarch64), so user-mode
    # boot is much slower than on i386/amd64. Wait for explorer.exe to enter
    # _tWinMain — that proves the entire user-mode chain (smss → csrss →
    # winlogon → explorer) is up. Boot takes ~5 min cold-cache TCG.
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 540.0
    min_boot_bytes: int = 5000
    # QMP can be slow to respond when the kernel is in tight context-switch
    # loops on TCG — give it more headroom than i386/amd64.
    qmp_timeout: float = 60.0
    # Brief settle after _tWinMain to let the taskbar/desktop draw before
    # screenshot/snapshot tests run.
    desktop_settle_seconds: float = 15.0
    # Boot milestones for test_milestones. "IoInitSystem returning TRUE" is
    # silenced on arm64 (iomgr.rs::io_trace is a no-op for aarch64) so we
    # don't gate on it; the surrounding [SMSS]/[CSRSS] entries already prove
    # IO subsystem init completed.
    required_milestones: list[str] = field(
        default_factory=lambda: [
            "[KERNEL] KiSystemStartup entered",
            "[PS] PspUserThreadStartup: returning",
            "[SMSS] _main entered",
            "[CSRSS] _main entered",
            "[WINLOGON] WinMain entered",
            "[EXPLORER] _tWinMain entered",
            "shell ready",
        ]
    )

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img = QEMU_DIR / "boot.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        shutil.copy2(QEMU_DIR / "efivars.fd", efivars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2", str(efivars_raw), str(efivars)],
            check=True,
            capture_output=True,
        )
        efivars_raw.unlink()

        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-drive",
            f"if=pflash,format=raw,file={uefi_fw},readonly=on",
            "-drive",
            f"if=pflash,format=qcow2,file={efivars}",
            "-drive",
            f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device",
            "ramfb",
            "-display",
            "none",
        ]


@dataclass
class I386Config(ArchConfig):
    arch: str = "i386"
    qemu_bin: str = "qemu-system-i386"
    machine_args: list[str] = field(
        default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"]
    )
    memory: str = "512M"
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 360.0
    min_boot_bytes: int = 500
    # See Amd64Config: marker fires before the taskbar paints. i386 is at least
    # as fast as amd64 (taskbar measured up by +15.5s on amd64), so 30s of margin
    # captures the rendered shell. 5s was too short — it screenshotted the bare
    # blue desktop before explorer built the tray.
    desktop_settle_seconds: float = 30.0
    required_milestones: list[str] = field(
        default_factory=lambda: [
            "[SMSS] _main entered",
            "[CSRSS] _main entered",
            "[WINLOGON] WinMain entered",
            "[EXPLORER] _tWinMain entered",
            "shell ready",
        ]
    )

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-i386" / "livecd.iso"
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-vga",
            "std",
            "-cdrom",
            str(iso),
            "-display",
            "none",
            "-no-reboot",
        ]


@dataclass
class I386UefiConfig(I386Config):
    """i386 booted via UEFI (OVMF32) from the livecd ISO's EFI El Torito entry.

    The uefildr.efi (Rust EfiEntry) is the loader; WinLdrSetProcessorContext
    performs the PAE→non-PAE transition the firmware requires. OVMF only
    auto-boots removable media when the CD carries an explicit bootindex, so
    the ISO is attached as an ide-cd device with bootindex=0 (a bare -cdrom
    drops to the UEFI shell instead).
    """

    arch: str = "i386-uefi"

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-i386" / "livecd.iso"
        ovmf_code = QEMU_DIR / "OVMF32_CODE.fd"
        ovmf_vars_src = QEMU_DIR / "OVMF32_VARS.fd"
        # The writable VARS pflash must be qcow2: QMP savevm refuses to
        # snapshot a writable raw block device ("does not support snapshots").
        # Mirror the arm64 efivars handling — raw → qcow2 convert.
        ovmf_vars_raw = work_dir / "ovmf32_vars.fd"
        ovmf_vars = work_dir / "ovmf32_vars.qcow2"
        shutil.copy2(ovmf_vars_src, ovmf_vars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2", str(ovmf_vars_raw), str(ovmf_vars)],
            check=True,
            capture_output=True,
        )
        ovmf_vars_raw.unlink()
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-drive",
            f"if=pflash,format=raw,file={ovmf_code},readonly=on",
            "-drive",
            f"if=pflash,format=qcow2,file={ovmf_vars}",
            "-drive",
            f"id=cd0,if=none,format=raw,file={iso},readonly=on",
            "-device",
            "ide-cd,drive=cd0,bootindex=0",
            "-vga",
            "std",
            "-display",
            "none",
            "-no-reboot",
        ]


@dataclass
class Amd64Config(ArchConfig):
    arch: str = "amd64"
    qemu_bin: str = "qemu-system-x86_64"
    machine_args: list[str] = field(
        default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"]
    )
    memory: str = "512M"
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 360.0
    min_boot_bytes: int = 1000
    # The boot marker (`_tWinMain entered`) fires the instant explorer starts;
    # the taskbar/Start button/tray paint a few seconds later. Measured 2026-06-05
    # (post gmxWorldToDeviceDefault GDI fix): the taskbar is up and stable by
    # +15.5s after the marker. 30s gives ~2x margin so the canonical screenshot
    # always captures the rendered shell, not a bare blue desktop. (A known
    # intermittent KiUnwaitThread CPU storm can delay it further on some boots;
    # the storm never gates the taskbar, and the smoke verdict is the serial
    # marker, not the screenshot, so an occasional early shot is only cosmetic.)
    desktop_settle_seconds: float = 30.0
    required_milestones: list[str] = field(
        default_factory=lambda: [
            "[SMSS] _main entered",
            "[CSRSS] _main entered",
            "[WINLOGON] WinMain entered",
            "[EXPLORER] _tWinMain entered",
            "shell ready",
        ]
    )

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-amd64" / "livecd.iso"
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-vga",
            "cirrus",
            "-cdrom",
            str(iso),
            "-display",
            "none",
            "-no-reboot",
        ]


@dataclass
class Amd64UefiConfig(Amd64Config):
    """amd64 booted via UEFI (OVMF) from the livecd ISO's EFI El Torito entry.

    Mirrors I386UefiConfig: i440fx (-M pc) + 64-bit OVMF, the ISO attached as an
    ide-cd with bootindex=0 (the CD is reachable over IDE, so uniata serves it
    and no AHCI driver is needed — q35 would put it on SATA). The uefildr.efi
    (Rust EfiEntry) loader boots the same kernel; the GOP framebuffer (arm64vid
    + framebuf) and synthetic-RSDP ACPI path make the desktop render and PS/2
    input work, same as i386-uefi.
    """

    arch: str = "amd64-uefi"

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-amd64" / "livecd.iso"
        ovmf_code = Path("/usr/share/OVMF/OVMF_CODE_4M.fd")
        ovmf_vars_src = Path("/usr/share/OVMF/OVMF_VARS_4M.fd")
        # Writable VARS pflash must be qcow2 for QMP savevm (raw is rejected).
        ovmf_vars_raw = work_dir / "ovmf_vars.fd"
        ovmf_vars = work_dir / "ovmf_vars.qcow2"
        shutil.copy2(ovmf_vars_src, ovmf_vars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2", str(ovmf_vars_raw), str(ovmf_vars)],
            check=True,
            capture_output=True,
        )
        ovmf_vars_raw.unlink()
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-drive",
            f"if=pflash,format=raw,file={ovmf_code},readonly=on",
            "-drive",
            f"if=pflash,format=qcow2,file={ovmf_vars}",
            "-drive",
            f"id=cd0,if=none,format=raw,file={iso},readonly=on",
            "-device",
            "ide-cd,drive=cd0,bootindex=0",
            "-vga",
            "std",
            "-display",
            "none",
            "-no-reboot",
        ]


def _start_echo_server() -> socketserver.BaseServer:
    """A tiny threaded TCP echo server bound to 0.0.0.0:<ephemeral>. Backs the slirp
    guestfwd on the windows-host backend: the guest's outbound connection (guest ->
    virtio TX -> slirp guestfwd -> chardev -> here) gets its bytes echoed straight back,
    replacing the `-cmd:cat` line echo that can't run on the Windows QEMU host. Bound to
    the wildcard so the Windows QEMU's chardev reaches it via WSL2 localhost forwarding.
    Daemon threads — never blocks interpreter exit; caller shuts it down in stop()."""

    class _Echo(socketserver.BaseRequestHandler):
        def handle(self) -> None:
            while True:
                data = self.request.recv(4096)
                if not data:
                    return
                self.request.sendall(data)

    srv = socketserver.ThreadingTCPServer(("0.0.0.0", 0), _Echo)
    srv.daemon_threads = True
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


def _free_tcp_port() -> int:
    """Pick a currently-free localhost TCP port for a slirp hostfwd rule.

    Bind-to-0 then release: there's a tiny reuse race, but QEMU rebinding it
    moments later in practice never collides, and it keeps parallel net VMs
    (multiple agents/worktrees) from fighting over a fixed port.
    """
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


@dataclass
class Amd64UefiNetConfig(Amd64UefiConfig):
    """amd64-uefi + a virtio-net-pci NIC on a slirp user netdev.

    The net-enabled variant for the smoltcp netstack lane (M0 of
    docs/engineering/smoltcp-netstack-design.md). Deliberately NOT the default
    smoke config: boot smoke keeps its meaning (no NIC), net is its own signal
    (tests/qemu/test_net.py).

    slirp defaults give the guest a ready-made mini-internet:
      network 10.0.2.0/24, gateway+host 10.0.2.2, DNS forwarder 10.0.2.3,
      built-in DHCP server (leases from 10.0.2.15).
    Host->guest reachability goes through hostfwd only (slirp has no inbound
    ICMP), so the TCP echo check connects to 127.0.0.1:echo_hostfwd_port,
    forwarded to guest port ECHO_GUEST_PORT.
    """

    arch: str = "amd64-uefi-net"
    # Guest-side TCP echo port the netstack self-test listens on (M2+).
    ECHO_GUEST_PORT: int = 7777
    # Host port forwarded to the guest echo port; allocated in qemu_args().
    echo_hostfwd_port: int = 0
    # slirp guestfwd: the guest reaches an OUTBOUND echo at this virtual
    # address:port, which slirp pipes through `cat` on the host (a built-in
    # line echo — no host-listener startup race). This is the path an outbound
    # TCP client (and later winsock-via-AFD) uses; the guest's serial markers
    # prove connect/send/recv.
    GUESTFWD_ADDR: str = "10.0.2.100"
    GUESTFWD_PORT: int = 9999

    def qemu_args(self, work_dir: Path) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        return [
            *super().qemu_args(work_dir),
            # Modern-only virtio (PCI\VEN_1AF4&DEV_1041, virtio 1.x): the
            # netstack backend (virtio-drivers) speaks the modern interface,
            # and forcing it here keeps the device contract unambiguous —
            # same reasoning as arm64's virtio-mmio.force-legacy=false.
            "-device",
            "virtio-net-pci,disable-legacy=on,netdev=net0,id=vnet0",
            "-netdev",
            f"user,id=net0,hostfwd=tcp:127.0.0.1:{self.echo_hostfwd_port}"
            f"-:{self.ECHO_GUEST_PORT}"
            f",guestfwd=tcp:{self.GUESTFWD_ADDR}:{self.GUESTFWD_PORT}-cmd:cat",
        ]


@dataclass
class Arm64TestKernelConfig(Arm64Config):
    """ARM64 config with /TESTKERNEL appended to freeldr.ini boot options."""

    arch: str = "arm64-test"
    boot_marker: str = "TESTKERNEL READY"
    boot_timeout: float = 180.0
    desktop_settle_seconds: float = 0.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img_src = QEMU_DIR / "boot.img"
        boot_img = work_dir / "boot-test.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        shutil.copy2(boot_img_src, boot_img)

        ini_path = work_dir / "freeldr.ini"
        subprocess.run(
            ["mcopy", "-o", "-i", str(boot_img), "::/freeldr.ini", str(ini_path)],
            check=True,
            capture_output=True,
        )
        ini_text = ini_path.read_text()
        if "/TESTKERNEL" not in ini_text:
            ini_text = ini_text.replace(
                "/SOS /RDPATH=",
                "/SOS /TESTKERNEL /RDPATH=",
            )
            ini_path.write_text(ini_text)
        subprocess.run(
            ["mcopy", "-o", "-i", str(boot_img), str(ini_path), "::/freeldr.ini"],
            check=True,
            capture_output=True,
        )

        shutil.copy2(QEMU_DIR / "efivars.fd", efivars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2", str(efivars_raw), str(efivars)],
            check=True,
            capture_output=True,
        )
        efivars_raw.unlink()

        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-drive",
            f"if=pflash,format=raw,file={uefi_fw},readonly=on",
            "-drive",
            f"if=pflash,format=qcow2,file={efivars}",
            "-drive",
            f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device",
            "ramfb",
            "-display",
            "none",
        ]


# x86 is UEFI-only after the BIOS prune. I386Config / Amd64Config remain only as
# the (non-selectable) base classes that hold shared defaults for the -uefi
# variants; their legacy BIOS -cdrom qemu_args are no longer reachable.
ARCH_CONFIGS: dict[str, type[ArchConfig]] = {
    "arm64": Arm64Config,
    "arm64-test": Arm64TestKernelConfig,
    "i386-uefi": I386UefiConfig,
    "amd64-uefi": Amd64UefiConfig,
    "amd64-uefi-net": Amd64UefiNetConfig,
}


def get_arch_config(arch: str) -> ArchConfig:
    cls = ARCH_CONFIGS.get(arch)
    if cls is None:
        raise ValueError(f"Unknown architecture: {arch!r}. Choose from: {list(ARCH_CONFIGS)}")
    return cls()


class QemuVM:
    """Manages a single QEMU VM instance with QMP + serial control."""

    def __init__(
        self,
        config: ArchConfig,
        work_dir: Path,
        *,
        backend: str = "native",
        windows: dict | None = None,
    ) -> None:
        self.config = config
        self.work_dir = work_dir
        self.work_dir.mkdir(parents=True, exist_ok=True)

        # Launch backend: "native" (local qemu + Unix sockets + KVM/TCG — unchanged) or
        # "windows-host" (Windows QEMU.exe via interop + TCP transport + staged media — see
        # backend.py / docs/engineering/windows-host-qemu-spike.md). windows = the
        # qemu.windows.* config subtree.
        self._backend = backend
        self._win = windows or {}
        self._win_uid = ""  # unique -name tag for uid-scoped kill (windows backend)

        self._sock_dir = Path(tempfile.mkdtemp(prefix="ros-"))
        atexit.register(lambda d=self._sock_dir: shutil.rmtree(d, ignore_errors=True))

        self.qmp_path = self._sock_dir / "qmp.sock"
        self.serial_path = self._sock_dir / "ser.sock"
        self.scratch_path = work_dir / "scratch.qcow2"
        self.debugcon_path = work_dir / "debugcon.bin"

        self.qmp = QmpClient()
        self.serial = SerialClient()
        self._proc: subprocess.Popen | None = None
        self._boot_output: str = ""
        self._watchdog: threading.Timer | None = None
        self._slot_fd: int | None = None
        self._slot_index: int | None = None
        self._arch_slot_fd: int | None = None  # per-arch sub-cap lock (KVM arches)
        # windows-host only: harness echo server backing the slirp guestfwd when
        # `-cmd:cat` can't run (Windows host has no `cat`). None on native / non-net.
        self._echo_server: socketserver.BaseServer | None = None

    @property
    def net_host(self) -> str:
        """Host address a TEST should dial to reach a slirp `hostfwd`. Native: loopback.
        windows-host: the WSL default-gateway IP — a Windows-side `0.0.0.0` hostfwd is
        reachable from WSL only via the gateway, not `127.0.0.1` (WSL's loopback is the
        WSL VM's own). See backend.gateway_ip / rewrite_netdev_for_windows."""
        if self._backend == "windows-host":
            from . import backend as be

            return be.gateway_ip()
        return "127.0.0.1"

    def _scratch_device_args(self) -> list[str]:
        """Scratch disk + (non-i386) its virtio device. i386: bare if=none (FreeLDR
        crashes on an unpartitioned BIOS-visible disk; savevm only needs a backend)."""
        return [
            "-drive",
            f"file={self.scratch_path},format=qcow2,if=none,id=scratch",
            *(
                [
                    "-device",
                    "virtio-blk-device,drive=scratch"
                    if self.config.arch.startswith("arm64")
                    else "virtio-blk-pci,drive=scratch",
                ]
                if not self.config.arch.startswith("i386")
                else []
            ),
        ]

    def _set_live_tee(self) -> None:
        """Tee serial to a predictable per-arch live log (`tail -f`). Truncated per session."""
        self.live_log_path = LIVE_LOG_DIR / f"{self.config.arch}_smoke_live.log"
        # ASCII-only header: the tee file is latin-1 (byte-faithful to the serial stream).
        self.serial.set_tee(
            str(self.live_log_path),
            header=f"# vibeos smoke live serial -- {self.config.arch} -- "
            f"pid(launcher)={os.getpid()} -- started {time.ctime()}\n"
            f"# tail -f {self.live_log_path}\n",
        )

    def _build_native_cmd(self) -> list[str]:
        """Native backend: local qemu, Unix-socket QMP/serial, KVM/TCG. Unchanged."""
        subprocess.run(
            ["qemu-img", "create", "-f", "qcow2", str(self.scratch_path), "64M"],
            check=True,
            capture_output=True,
        )
        self.serial.listen(str(self.serial_path))
        self._set_live_tee()
        return [
            self.config.qemu_bin,
            "-qmp",
            f"unix:{self.qmp_path},server,nowait",
            "-chardev",
            f"socket,id=ser0,path={self.serial_path},server=off,reconnect=1",
            "-serial",
            "chardev:ser0",
            *self._scratch_device_args(),
            "-monitor",
            "none",
            *(
                ["-debugcon", f"file:{self.debugcon_path}"]
                if not self.config.arch.startswith("arm64")
                else []
            ),
            *self.config.qemu_args(self.work_dir),
        ]

    def _build_windows_cmd(self) -> list[str]:
        """windows-host backend: Windows QEMU.exe via interop, media staged to a
        Windows-visible dir, QMP + serial over TCP (we listen on 127.0.0.1; the Windows
        QEMU connects back via WSL2 localhost forwarding). See backend.py."""
        from . import backend as be

        if self.config.arch.startswith("arm64"):
            raise RuntimeError("windows-host backend supports x86 only (arm64 stays native TCG)")

        accel = self._win.get("accel", "tcg")
        staging = Path(self._win.get("staging_dir", "/mnt/c/temp/vibeos-qemu"))
        shutil.rmtree(staging, ignore_errors=True)
        staging.mkdir(parents=True, exist_ok=True)

        # Scratch (created WSL-side under /mnt, read by Windows QEMU via its Windows path).
        self.scratch_path = staging / "scratch.qcow2"
        subprocess.run(
            ["qemu-img", "create", "-f", "qcow2", str(self.scratch_path), "64M"],
            check=True,
            capture_output=True,
        )
        # debugcon written by Windows QEMU to a staged path; read back via /mnt.
        self.debugcon_path = staging / "debugcon.bin"

        # We listen (WSL); the Windows QEMU connects to host_addr:<port>. CRITICAL: bind
        # 0.0.0.0, NOT 127.0.0.1 — WSL2's localhost relay only forwards Windows->WSL for
        # services bound to the wildcard; a 127.0.0.1-bound WSL listener is invisible to
        # Windows (verified: 127.0.0.1 reachable from Windows iff WSL binds 0.0.0.0; the
        # WSL-VM IP is not reachable at all under NAT). QEMU still dials 127.0.0.1.
        host = self._win.get("host_addr", "127.0.0.1")
        ser_port = self.serial.listen_tcp("0.0.0.0", 0)
        qmp_port = self.qmp.listen_tcp("0.0.0.0", 0)
        self._set_live_tee()

        self._win_uid = f"vibeos-{os.getpid()}-{ser_port}"
        win_bin = be.windows_qemu_bin(self.config.qemu_bin, self._win["qemu_dir"], accel)

        arch_args = be.rewrite_accel(
            be.stage_media_paths(self.config.qemu_args(self.work_dir), staging), accel
        )
        # Net config (amd64-uefi-net): adapt the slirp fwds across the WSL<->Windows
        # boundary — hostfwd binds 0.0.0.0 (dialed via the gateway IP, see net_host) and
        # the `-cmd:cat` guestfwd echo is redirected to a chardev backed by a harness echo
        # server (Windows has no `cat`). No-op for non-net configs.
        chardev_id = "gf0"
        arch_args, needs_echo = be.rewrite_netdev_for_windows(arch_args, chardev_id)
        echo_chardev: list[str] = []
        if needs_echo:
            self._echo_server = _start_echo_server()
            echo_port = self._echo_server.server_address[1]
            echo_chardev = [
                "-chardev",
                f"socket,id={chardev_id},host={host},port={echo_port},"
                "server=off,reconnect-ms=1000",
            ]
        return [
            win_bin,
            "-name",
            self._win_uid,
            # reconnect-ms (not the older `reconnect=N`): recent QEMU (the Windows 11.x
            # build) dropped the seconds-form param. We listen, QEMU dials back in.
            "-chardev",
            f"socket,id=qmp0,host={host},port={qmp_port},server=off,reconnect-ms=1000",
            "-mon",
            "chardev=qmp0,mode=control",
            "-chardev",
            f"socket,id=ser0,host={host},port={ser_port},server=off,reconnect-ms=1000",
            "-serial",
            "chardev:ser0",
            *be.stage_media_paths(self._scratch_device_args(), staging),
            "-monitor",
            "none",
            "-debugcon",
            f"file:{be.win_path(self.debugcon_path)}",
            *echo_chardev,
            *arch_args,
        ]

    def start(self) -> None:
        """Start QEMU process, connect QMP + serial (backend-aware)."""
        if self._backend == "windows-host":
            cmd = self._build_windows_cmd()
        else:
            cmd = self._build_native_cmd()

        print(f"[QemuVM] {' '.join(cmd)}")

        # Acquire a global concurrency slot BEFORE launching — and before we
        # record the start/deadline below, so the time spent queued here is
        # excluded from the 30-min lifetime cap. Held for this pytest process's
        # lifetime; released in stop()/atexit.
        self._acquire_slot()

        self._proc = subprocess.Popen(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            preexec_fn=os.setpgrp,
        )

        # Drop a scoped lock so the reaper can safely kill THIS qemu iff our
        # launcher (this pytest process) dies. reap_orphan_qemus() is auto-
        # wired in conftest.py at session start. qemu/reap.sh covers both
        # this lock dir and the kernel-debug-toolkit's manifest.json files.
        started = time.time()
        try:
            LOCK_DIR.mkdir(parents=True, exist_ok=True)
            self._lock_path = LOCK_DIR / f"{os.getpid()}-{self._proc.pid}.json"
            self._lock_path.write_text(
                json.dumps(
                    {
                        "launcher_pid": os.getpid(),
                        "qemu_pid": self._proc.pid,
                        "arch": self.config.arch,
                        "work_dir": str(self.work_dir),
                        "started": started,
                        # Integer epoch seconds: shell consumers (reap.sh) compare this
                        # with `[ "$(date +%s)" -ge "$dl" ]`, which errors on a float and
                        # would silently treat the VM as never-expired. Sub-second
                        # precision is meaningless for a 30-min cap anyway.
                        "deadline": int(started + MAX_VM_LIFETIME_SECONDS),
                        "slot": self._slot_index,
                        # Absolute so qemu/ps.sh can point at it from ANY worktree.
                        "live_log": str(self.live_log_path),
                        "cmd": cmd,
                    }
                )
            )
            atexit.register(lambda p=self._lock_path: p.unlink(missing_ok=True))
        except OSError:
            self._lock_path = None

        # In-process hard-cap watchdog: kills this VM the instant the 30-min
        # ceiling is hit, regardless of what the test is doing. Belt to the
        # lock-file "deadline" suspenders (which other reapers enforce if this
        # process is itself wedged). Daemon timer — never blocks interpreter exit.
        self._watchdog = threading.Timer(MAX_VM_LIFETIME_SECONDS, self._watchdog_kill)
        self._watchdog.daemon = True
        self._watchdog.start()

        self.serial.accept(timeout=15.0)

        if self._backend == "windows-host":
            # We listen; the Windows QEMU connects back. accept() blocks for the
            # connection then completes the QMP handshake.
            self._connect_with_retry(
                lambda: self.qmp.accept(timeout=self.config.qmp_timeout),
                "QMP",
            )
        else:
            self._wait_for_socket(self.qmp_path, timeout=10.0)
            self._connect_with_retry(
                lambda: self.qmp.connect(str(self.qmp_path), timeout=self.config.qmp_timeout),
                "QMP",
            )

    def wait_for_boot(self) -> str:
        """Wait for boot to complete. Returns all boot output.

        Uses boot_marker if set, otherwise waits for serial output to go idle.
        After detection, waits desktop_settle_seconds for user-mode to finish.
        """
        try:
            if self.config.boot_marker:
                self._boot_output = self.serial.read_until(
                    self.config.boot_marker,
                    timeout=self.config.boot_timeout,
                )
            else:
                self._boot_output = self.serial.read_until_idle(
                    idle_seconds=5.0,
                    timeout=self.config.boot_timeout,
                    min_bytes=self.config.min_boot_bytes,
                )
        except TimeoutError:
            # Save whatever serial output was captured before the timeout
            self._boot_output = self.serial.partial_output
            raise

        # Wait for desktop to come up (user-mode processes need time after
        # the kernel finishes loading boot drivers).
        settle = self.config.desktop_settle_seconds
        if settle > 0:
            print(f"[QemuVM] Waiting {settle:.0f}s for desktop to settle...")
            # Keep draining serial during settle (some arches produce late output)
            try:
                extra = self.serial.read_until_idle(
                    idle_seconds=settle,
                    timeout=settle + 10,
                    min_bytes=0,
                )
                self._boot_output += extra
            except TimeoutError:
                pass  # Settle time elapsed — that's fine

        return self._boot_output

    def save_snapshot(self, name: str = "ready") -> None:
        """Save VM state via QMP/HMP savevm."""
        result = self.qmp.hmp(f"savevm {name}")
        if "Error" in result:
            raise RuntimeError(f"savevm failed: {result}")
        time.sleep(0.5)

    def restore_snapshot(self, name: str = "ready") -> None:
        """Restore VM state via QMP/HMP loadvm."""
        result = self.qmp.hmp(f"loadvm {name}")
        if "Error" in result:
            raise RuntimeError(f"loadvm failed: {result}")
        time.sleep(0.5)
        self.serial.read_available(timeout=0.2)

    def serial_output(self, timeout: float = 1.0) -> str:
        """Read available serial output since last read."""
        return self.serial.read_available(timeout=timeout)

    def screendump(self, path: str | Path) -> Path:
        """Save a screenshot via QMP screendump.

        windows-host: the Windows QEMU writes to a Windows path, so screendump to a staged
        path then copy it back to the requested (Linux) location, transparently to callers."""
        path = Path(path)
        if self._backend == "windows-host":
            from . import backend as be

            staging = Path(self._win.get("staging_dir", "/mnt/c/temp/vibeos-qemu"))
            staged = staging / f"screen-{path.name}"
            result = self.qmp.execute("screendump", filename=be.win_path(staged))
            time.sleep(0.5)
            if not staged.exists():
                raise RuntimeError(f"screendump failed: {result}")
            shutil.copy2(staged, path)
            return path
        result = self.qmp.execute("screendump", filename=str(path))
        time.sleep(0.5)
        if not path.exists():
            raise RuntimeError(f"screendump failed: {result}")
        return path

    def read_debugcon(self) -> bytes:
        """Read raw debugcon (bochs debug port 0xE9) output."""
        if self.debugcon_path.exists():
            return self.debugcon_path.read_bytes()
        return b""

    def debugcon_report(self) -> str | None:
        """Decode debugcon output into a triage report."""
        raw = self.read_debugcon()
        if not raw:
            return None
        try:
            from .debugcon import debugcon_report as _report

            return _report(raw, self.config.arch)
        except Exception:
            return f"Debugcon: {len(raw)} bytes (catalog unavailable)"

    def _acquire_slot(self) -> None:
        """Take one of QEMU_MAX_CONCURRENT global slots, queuing until free.

        A slot is an flock on a shared slot-NN file (same pool qemu/start.sh
        uses). Held on self._slot_fd for this process's lifetime; the kernel
        closes the fd if pytest dies, freeing the slot (the orphan QEMU is then
        reaped separately). Raises TimeoutError if no slot frees within the
        queue timeout.
        """
        SLOT_DIR.mkdir(parents=True, exist_ok=True)
        deadline = time.monotonic() + QEMU_QUEUE_TIMEOUT
        announced = False
        while True:
            for i in range(QEMU_MAX_CONCURRENT):
                fd = os.open(
                    str(SLOT_DIR / f"slot-{i:02d}"), os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o644
                )
                try:
                    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self._slot_fd = fd
                    self._slot_index = i
                    if announced:
                        print(f"[QemuVM] acquired slot {i} after queuing")
                    self._acquire_arch_slot()  # layer per-arch sub-cap under it
                    return
                except OSError:
                    os.close(fd)
            if time.monotonic() >= deadline:
                raise TimeoutError(
                    f"no free QEMU slot after {QEMU_QUEUE_TIMEOUT:.0f}s "
                    f"(concurrency cap={QEMU_MAX_CONCURRENT})"
                )
            if not announced:
                print(
                    f"[QemuVM] all {QEMU_MAX_CONCURRENT} QEMU slots busy; "
                    f"queuing (queue time excluded from "
                    f"{MAX_VM_LIFETIME_SECONDS // 60}-min lifetime cap)..."
                )
                announced = True
            time.sleep(2)

    def _acquire_arch_slot(self) -> None:
        """Serialize same-arch boots under the global slot (see per_arch_cap).
        Held on self._arch_slot_fd for this process's lifetime alongside the
        global slot; cross-arch boots don't contend so they stay parallel. On
        timeout we release the global slot we already hold so it doesn't leak."""
        # windows-host runs TCG on the Windows host (no nested KVM), which we measured to
        # NOT contend under same-arch concurrency — the per-arch serialize cap is the
        # nested-KVM workaround it makes obsolete, so skip it (like arm64 TCG). getattr
        # default keeps slot-only partial QemuVM constructions (tests) working.
        if getattr(self, "_backend", "native") == "windows-host":
            return
        cap = per_arch_cap(self.config.arch)
        if cap <= 0:
            return  # uncapped (arm64 TCG): no per-arch lock
        arch = self.config.arch
        deadline = time.monotonic() + QEMU_QUEUE_TIMEOUT
        announced = False
        while True:
            for i in range(cap):
                fd = os.open(
                    str(SLOT_DIR / f"arch-{arch}-{i:02d}"),
                    os.O_WRONLY | os.O_CREAT | os.O_APPEND,
                    0o644,
                )
                try:
                    fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self._arch_slot_fd = fd
                    if announced:
                        print(f"[QemuVM] acquired same-arch ({arch}) slot {i} after queuing")
                    return
                except OSError:
                    os.close(fd)
            if time.monotonic() >= deadline:
                self._release_slot()  # don't leak the global slot we hold
                raise TimeoutError(
                    f"no free same-arch ({arch}) slot after {QEMU_QUEUE_TIMEOUT:.0f}s "
                    f"(per-arch cap={cap})"
                )
            if not announced:
                print(
                    f"[QemuVM] same-arch ({arch}) boot in progress; queuing "
                    f"same-arch (cap={cap}, cross-arch stays parallel)..."
                )
                announced = True
            time.sleep(2)

    def _release_slot(self) -> None:
        """Release the concurrency slot(s) (idempotent): the per-arch sub-cap
        lock and the global slot."""
        for attr in ("_arch_slot_fd", "_slot_fd"):
            fd = getattr(self, attr)
            if fd is not None:
                try:
                    os.close(fd)
                except OSError:
                    pass
                setattr(self, attr, None)

    def _watchdog_kill(self) -> None:
        """Fired by the 30-min hard-cap timer. Kill the VM's process group.

        Independent of the normal stop() path — the VM has overrun any
        legitimate lifetime, so we go straight to a pgroup TERM→KILL. Safe if
        the proc is already gone (stop() ran first). The serial drainer is left
        to close() so the live log keeps the final bytes.
        """
        proc = self._proc
        if proc is None:
            return
        print(
            f"[QemuVM] WATCHDOG: {self.config.arch} VM exceeded "
            f"{MAX_VM_LIFETIME_SECONDS // 60}-min hard cap — killing pid={proc.pid}"
        )
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
            time.sleep(3)
            if proc.poll() is None:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        except (ProcessLookupError, OSError):
            pass

    def stop(self) -> None:
        """Kill QEMU process, clean up sockets."""
        if self._watchdog is not None:
            self._watchdog.cancel()
            self._watchdog = None
        self.serial.close()
        self.qmp.close()

        # Tear down the guestfwd echo server (windows-host net config only).
        if self._echo_server is not None:
            self._echo_server.shutdown()
            self._echo_server.server_close()
            self._echo_server = None

        # windows-host: the Windows QEMU.exe runs behind WSL interop, so the Linux
        # pgroup kill below won't reach it — kill it precisely by our unique -name tag.
        if self._backend == "windows-host" and self._win_uid:
            from . import backend as be

            be.kill_by_uid(self._win_uid)

        if self._proc is not None:
            try:
                os.killpg(os.getpgid(self._proc.pid), signal.SIGTERM)
                self._proc.wait(timeout=5)
            except (ProcessLookupError, subprocess.TimeoutExpired):
                try:
                    os.killpg(os.getpgid(self._proc.pid), signal.SIGKILL)
                    self._proc.wait(timeout=2)
                except (ProcessLookupError, subprocess.TimeoutExpired):
                    pass
            self._proc = None

        lock = getattr(self, "_lock_path", None)
        if lock is not None:
            lock.unlink(missing_ok=True)

        # Release the concurrency slot only after QEMU is actually dead, so the
        # slot count reflects running VMs (not ones still shutting down).
        self._release_slot()

        shutil.rmtree(self._sock_dir, ignore_errors=True)

    def _connect_with_retry(self, connect_fn, name: str, timeout: float = 10.0) -> None:
        """Retry a connect function until it succeeds or QEMU dies."""
        deadline = time.monotonic() + timeout
        last_err = None
        while time.monotonic() < deadline:
            if self._proc and self._proc.poll() is not None:
                self._raise_start_error(f"QEMU exited with code {self._proc.returncode}")
            try:
                connect_fn()
                return
            except (ConnectionRefusedError, ConnectionError, OSError) as e:
                last_err = e
                time.sleep(0.2)
        self._raise_start_error(f"Failed to connect to {name}: {last_err}")

    def _wait_for_socket(self, path: Path, timeout: float = 10.0) -> None:
        """Wait for a Unix socket file to appear."""
        deadline = time.monotonic() + timeout
        while not path.exists():
            if time.monotonic() > deadline:
                self._raise_start_error(f"Socket {path} did not appear within {timeout}s")
            if self._proc and self._proc.poll() is not None:
                self._raise_start_error(f"QEMU exited with code {self._proc.returncode}")
            time.sleep(0.1)

    def _raise_start_error(self, msg: str) -> None:
        """Raise RuntimeError with QEMU stderr for diagnostics."""
        stderr = ""
        if self._proc and self._proc.stderr:
            try:
                stderr = self._proc.stderr.read().decode("utf-8", errors="replace")
            except Exception:
                pass
        raise RuntimeError(f"{msg}\nQEMU stderr:\n{stderr}")
