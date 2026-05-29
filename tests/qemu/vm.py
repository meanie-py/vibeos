"""QEMU VM lifecycle manager for ReactOS test sessions."""

from __future__ import annotations

import atexit
import json
import os
import shutil
import signal
import subprocess
import tempfile
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
        if lpid and _pid_alive(lpid):
            continue  # live session (maybe another agent) — DO NOT TOUCH
        # Launcher is dead → this run crashed. Reap its (and only its) qemu.
        if qpid and _pid_alive(qpid) and _proc_is_qemu(qpid):
            try:
                os.killpg(os.getpgid(qpid), signal.SIGTERM)
                time.sleep(2)
                if _pid_alive(qpid):
                    os.killpg(os.getpgid(qpid), signal.SIGKILL)
                actions.append(f"reaped orphan qemu pid={qpid} (launcher {lpid} dead)")
            except (ProcessLookupError, OSError) as e:
                actions.append(f"qemu pid={qpid} reap failed: {e}")
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
    machine_args: list[str] = field(default_factory=lambda: [
        "-M", "virt", "-cpu", "cortex-a72",
        "-accel", "tcg,thread=multi",
        # virtio-mmio kbd+mouse — the devices arm64input.rs binds to
        # (scans virtio-mmio transports at 0x0a000000 + i*0x200 for the
        # virtio-input device id). Without these the poll thread polls
        # empty virtqueues and the input stack has no functional coverage.
        # `*-device` (not `*-pci`) is the virtio-mmio transport variant.
        "-device", "virtio-keyboard-device,id=arm64kbd",
        "-device", "virtio-mouse-device,id=arm64mouse",
        # QEMU `virt` defaults its virtio-mmio transports to legacy (v1).
        # arm64input's modern (v2) split-queue path is the well-exercised
        # one; force modern so smoke drives it. Modern virtio-mmio is what
        # real ARM64 virtio environments use — a representative, non-
        # weakening host-config choice (the legacy queue path remains a
        # tracked driver follow-up, not a smoke blocker).
        "-global", "virtio-mmio.force-legacy=false",
    ])
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
    required_milestones: list[str] = field(default_factory=lambda: [
        "[KERNEL] KiSystemStartup entered",
        "[PS] PspUserThreadStartup: returning",
        "[SMSS] _main entered",
        "[CSRSS] _main entered",
        "[WINLOGON] WinMain entered",
        "[EXPLORER] _tWinMain entered",
        "shell ready",
    ])

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img = QEMU_DIR / "boot.img"
        uefi_fw = QEMU_DIR / "QEMU_EFI.fd"
        efivars_raw = work_dir / "efivars.fd"
        efivars = work_dir / "efivars.qcow2"

        shutil.copy2(QEMU_DIR / "efivars.fd", efivars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2",
             str(efivars_raw), str(efivars)],
            check=True, capture_output=True,
        )
        efivars_raw.unlink()

        return [
            *self.machine_args,
            "-m", self.memory,
            "-drive", f"if=pflash,format=raw,file={uefi_fw},readonly=on",
            "-drive", f"if=pflash,format=qcow2,file={efivars}",
            "-drive", f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device", "ramfb",
            "-display", "none",
        ]


@dataclass
class I386Config(ArchConfig):
    arch: str = "i386"
    qemu_bin: str = "qemu-system-i386"
    machine_args: list[str] = field(default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"])
    memory: str = "512M"
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 360.0
    min_boot_bytes: int = 500
    desktop_settle_seconds: float = 5.0
    required_milestones: list[str] = field(default_factory=lambda: [
        "[SMSS] _main entered",
        "[CSRSS] _main entered",
        "[WINLOGON] WinMain entered",
        "[EXPLORER] _tWinMain entered",
        "shell ready",
    ])

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-i386" / "livecd.iso"
        return [
            *self.machine_args,
            "-m", self.memory,
            "-vga", "std",
            "-cdrom", str(iso),
            "-display", "none",
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
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2",
             str(ovmf_vars_raw), str(ovmf_vars)],
            check=True, capture_output=True,
        )
        ovmf_vars_raw.unlink()
        return [
            *self.machine_args,
            "-m", self.memory,
            "-drive", f"if=pflash,format=raw,file={ovmf_code},readonly=on",
            "-drive", f"if=pflash,format=qcow2,file={ovmf_vars}",
            "-drive", f"id=cd0,if=none,format=raw,file={iso},readonly=on",
            "-device", "ide-cd,drive=cd0,bootindex=0",
            "-vga", "std",
            "-display", "none",
            "-no-reboot",
        ]


@dataclass
class Amd64Config(ArchConfig):
    arch: str = "amd64"
    qemu_bin: str = "qemu-system-x86_64"
    machine_args: list[str] = field(default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"])
    memory: str = "512M"
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    boot_timeout: float = 360.0
    min_boot_bytes: int = 1000
    # amd64 explorer paints the taskbar 90+ seconds after the kernel-side
    # "shell ready" marker — slower than i386 because of a known KiUnwaitThread
    # CPU storm that competes with explorer's user-mode paint thread (the storm
    # doesn't gate the taskbar but does delay it). Without enough settle the
    # smoke screenshot saves a bare blue desktop; with 120s we capture the
    # rendered Start button + clock.
    desktop_settle_seconds: float = 120.0
    required_milestones: list[str] = field(default_factory=lambda: [
        "[SMSS] _main entered",
        "[CSRSS] _main entered",
        "[WINLOGON] WinMain entered",
        "[EXPLORER] _tWinMain entered",
        "shell ready",
    ])

    def qemu_args(self, work_dir: Path) -> list[str]:
        iso = PROJECT_DIR / "build-amd64" / "livecd.iso"
        return [
            *self.machine_args,
            "-m", self.memory,
            "-vga", "cirrus",
            "-cdrom", str(iso),
            "-display", "none",
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
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2",
             str(ovmf_vars_raw), str(ovmf_vars)],
            check=True, capture_output=True,
        )
        ovmf_vars_raw.unlink()
        return [
            *self.machine_args,
            "-m", self.memory,
            "-drive", f"if=pflash,format=raw,file={ovmf_code},readonly=on",
            "-drive", f"if=pflash,format=qcow2,file={ovmf_vars}",
            "-drive", f"id=cd0,if=none,format=raw,file={iso},readonly=on",
            "-device", "ide-cd,drive=cd0,bootindex=0",
            "-vga", "std",
            "-display", "none",
            "-no-reboot",
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
            check=True, capture_output=True,
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
            check=True, capture_output=True,
        )

        shutil.copy2(QEMU_DIR / "efivars.fd", efivars_raw)
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2",
             str(efivars_raw), str(efivars)],
            check=True, capture_output=True,
        )
        efivars_raw.unlink()

        return [
            *self.machine_args,
            "-m", self.memory,
            "-drive", f"if=pflash,format=raw,file={uefi_fw},readonly=on",
            "-drive", f"if=pflash,format=qcow2,file={efivars}",
            "-drive", f"file={boot_img},format=raw,if=virtio,readonly=on,file.locking=off",
            "-device", "ramfb",
            "-display", "none",
        ]


ARCH_CONFIGS: dict[str, type[ArchConfig]] = {
    "arm64": Arm64Config,
    "arm64-test": Arm64TestKernelConfig,
    "i386": I386Config,
    "i386-uefi": I386UefiConfig,
    "amd64": Amd64Config,
    "amd64-uefi": Amd64UefiConfig,
}


def get_arch_config(arch: str) -> ArchConfig:
    cls = ARCH_CONFIGS.get(arch)
    if cls is None:
        raise ValueError(f"Unknown architecture: {arch!r}. Choose from: {list(ARCH_CONFIGS)}")
    return cls()


class QemuVM:
    """Manages a single QEMU VM instance with QMP + serial control."""

    def __init__(self, config: ArchConfig, work_dir: Path) -> None:
        self.config = config
        self.work_dir = work_dir
        self.work_dir.mkdir(parents=True, exist_ok=True)

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

    def start(self) -> None:
        """Start QEMU process, connect QMP + serial."""
        subprocess.run(
            ["qemu-img", "create", "-f", "qcow2", str(self.scratch_path), "64M"],
            check=True, capture_output=True,
        )

        self.serial.listen(str(self.serial_path))

        cmd = [
            self.config.qemu_bin,
            "-qmp", f"unix:{self.qmp_path},server,nowait",
            "-chardev", f"socket,id=ser0,path={self.serial_path},server=off,reconnect=1",
            "-serial", "chardev:ser0",
            # Scratch disk for snapshots (writable qcow2 backend for savevm).
            # i386: if=none without a device — FreeLDR crashes on unpartitioned
            # BIOS-visible disks; QEMU savevm only needs a block backend.
            "-drive", f"file={self.scratch_path},format=qcow2,if=none,id=scratch",
            *(
                ["-device", "virtio-blk-device,drive=scratch" if self.config.arch.startswith("arm64")
                     else "virtio-blk-pci,drive=scratch"]
                if not self.config.arch.startswith("i386") else []
            ),
            "-monitor", "none",
            *(
                ["-debugcon", f"file:{self.debugcon_path}"]
                if not self.config.arch.startswith("arm64") else []
            ),
            *self.config.qemu_args(self.work_dir),
        ]

        print(f"[QemuVM] {' '.join(cmd)}")

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
        try:
            LOCK_DIR.mkdir(parents=True, exist_ok=True)
            self._lock_path = LOCK_DIR / f"{os.getpid()}-{self._proc.pid}.json"
            self._lock_path.write_text(json.dumps({
                "launcher_pid": os.getpid(),
                "qemu_pid": self._proc.pid,
                "arch": self.config.arch,
                "work_dir": str(self.work_dir),
                "started": time.time(),
                "cmd": cmd,
            }))
            atexit.register(lambda p=self._lock_path: p.unlink(missing_ok=True))
        except OSError:
            self._lock_path = None

        self.serial.accept(timeout=15.0)

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
        """Save a screenshot via QMP screendump."""
        path = Path(path)
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

    def stop(self) -> None:
        """Kill QEMU process, clean up sockets."""
        self.serial.close()
        self.qmp.close()

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
