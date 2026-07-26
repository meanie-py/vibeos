"""QEMU VM lifecycle manager for ReactOS test sessions."""

from __future__ import annotations

import atexit
import fcntl
import json
import os
import shlex
import shutil
import signal
import socket
import socketserver
import subprocess
import tempfile
import threading
import time
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

from .cli import hostlock
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
#
# 10 → 16 (2026-07-25): this is the OUTER bound, and the real smoke throttle is the
# hostlock `bootcpu` pool, which is now capacity-derived (8 on this 16C/32T box, was a flat
# 4). At 10 an 8-boot fleet left only 2 slots for everything else — a `debug start`, a
# `boot --until`, a farm worker — so interactive debugging queued behind other lanes' gates.
# 16 uniprocessor TCG guests are ~16-21 busy host threads and ~16 GB, which this box has;
# anything that actually needs protecting from oversubscription is protected by `bootcpu`.
QEMU_MAX_CONCURRENT = int(os.environ.get("VIBEOS_QEMU_MAX_CONCURRENT", "16"))
QEMU_QUEUE_TIMEOUT = float(os.environ.get("VIBEOS_QEMU_QUEUE_TIMEOUT", "1800"))
SLOT_DIR = LOCK_DIR / "slots"

# windows-host hostfwd has a pick-then-bind TOCTOU: we pick a Windows-free port,
# QEMU binds it a moment later, and a concurrent process can steal it in between →
# QEMU exits immediately with "Could not set up host forwarding rule" (before any
# guest code). Relaunch with a freshly-picked port a few times to ride out the race.
HOSTFWD_BIND_ERROR = "Could not set up host forwarding rule"
# QEMU's message when the gdbstub TCP bind loses the Windows-stack port race.
GDB_BIND_ERROR = "Could not open gdbserver"
# Any early-exit stderr matching one of these = a pick-then-bind TOCTOU on the
# Windows stack → relaunch with a freshly-picked port (see _launch_qemu).
BIND_RACE_ERRORS = (HOSTFWD_BIND_ERROR, GDB_BIND_ERROR)
HOSTFWD_LAUNCH_RETRIES = int(os.environ.get("VIBEOS_HOSTFWD_LAUNCH_RETRIES", "4"))

# freeldr's boot-entry selector, as it appears verbatim inside the livecd ISO.
# The x86 media has always carried BOTH flavors — [LiveCD_SMP] (/KERNEL=ntkrnlmp.exe
# /HAL=halmp.dll) ships in every livecd.ini and all four binaries are staged — but
# freeldr chooses from this one line and we set TimeOut=0, so no menu ever appears
# to choose at. There is no boot-time channel to tell freeldr otherwise: -append
# applies only to a direct `-kernel` Linux boot, not a UEFI CD boot of freeldr.
_INI_DEFAULT_OS = b"DefaultOS="
_INI_SMP_ENTRY = b"LiveCD_SMP"


def smp_selected_iso(iso: Path, work_dir: Path) -> Path:
    """An ISO whose freeldr default entry is the MP pair, WITHOUT rebuilding media.

    ISO9660 stores each file's bytes once, contiguously, with no per-file checksum,
    and freeldr.ini is plain text — so flipping the default is an equal-length
    in-place write of one line into a copy of the image. The alternative was a cmake
    reconfigure + mkisofs repack (8m30s measured on amd64) or shipping a second
    221 MB media; this is a 0.15s copy and a 22-byte poke.

    Padding: `DefaultOS=LiveCD_SMP` is 20 bytes and the shipped
    `DefaultOS=LiveCD_Debug` is 22, so the remainder becomes `\\r\\n` (a blank line
    the INI parser ignores) plus `;` comment filler for any excess. A 1-byte
    remainder cannot form a clean line terminator, so it raises rather than emitting
    a lone CR.

    Every failure path raises. Booting UP media on an `--smp` run is precisely the
    false-green `_smp_check` exists to catch, and a silent fallback here would
    manufacture exactly that."""
    data = bytearray(iso.read_bytes())
    hits = [i for i in range(len(data)) if data.startswith(_INI_DEFAULT_OS, i)]
    if len(hits) != 1:
        raise RuntimeError(
            f"{iso}: expected exactly one {_INI_DEFAULT_OS.decode()} in the image, found "
            f"{len(hits)} — freeldr.ini is not where this expects it; rebuild with "
            "`build --smp-kernel` instead of patching"
        )
    start = hits[0]
    end = start
    while end < len(data) and data[end] not in (0x0D, 0x0A):
        end += 1
    old = bytes(data[start:end])
    if old == _INI_DEFAULT_OS + _INI_SMP_ENTRY:
        return iso  # media already defaults to the MP entry (built --smp-kernel)
    if b"[" + _INI_SMP_ENTRY + b"]" not in data:
        raise RuntimeError(
            f"{iso}: no [{_INI_SMP_ENTRY.decode()}] section — this media predates the "
            "halmp.dll twin and cannot boot the MP pair; rebuild it"
        )
    new = _INI_DEFAULT_OS + _INI_SMP_ENTRY
    pad = len(old) - len(new)
    if pad < 0:
        raise RuntimeError(f"{iso}: {old!r} is shorter than {new!r}; cannot patch in place")
    if pad == 1:
        raise RuntimeError(f"{iso}: 1-byte remainder after {new!r} cannot terminate the line")
    if pad:
        new += b"\r\n" + b";" * (pad - 2)
    assert len(new) == len(old)
    data[start:end] = new
    out = work_dir / iso.name
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(data)
    return out


def _maybe_smp_iso(iso: Path, work_dir: Path) -> Path:
    """`smp_selected_iso` when this run asked for SMP, else the media untouched."""
    return smp_selected_iso(iso, work_dir) if os.environ.get("VIBEOS_QEMU_SMP", "").strip() else iso


def per_arch_cap(arch: str) -> int:
    """Same-arch concurrency sub-cap, layered under the global pool. Mirrors
    qemu/lib/common.sh:qemu_per_arch_cap. The global cap bounds total host load;
    this serializes SAME-ARCH boots, which contend on /dev/kvm + the shared IDE
    path the global count is blind to (two i386/amd64 KVM guests starve each
    other into false HANG_TIMEOUTs). KVM arches default to 1 (strict serialize);
    arm64 (TCG, no accelerator contention) defaults to 0 = uncapped.

    NOTE: this cap only applies to backends that actually share an accelerator —
    native+KVM and windows-host+WHPX. The DEFAULT windows-host TCG backend runs
    each guest off-host with no shared accelerator/IDE, so _acquire_arch_slot()
    early-returns BEFORE consulting this cap (vm.py, see its docstring) and
    same-arch boots run in parallel. This function is still the source of the cap
    value for the backends that need it."""
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
    # Early-hang watchdog: if the boot emits serial, then goes SILENT this many
    # seconds without reaching the marker, declare a wedge and fail fast instead
    # of burning the full boot_timeout. 0 = disabled (wait the whole timeout).
    # Set WELL above the longest legitimate inter-line gap on a healthy boot so a
    # slow-but-progressing boot is never false-aborted (a healthy boot reaches the
    # marker long before this anyway); the win is on genuine wedges that go quiet
    # for minutes (e.g. the nettest/LpcExit hangs sit silent ~500s under the 540s
    # arm64 timeout — this catches them in a fraction of that).
    hang_silence_seconds: float = 0.0
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
    # boot is slower than i386/amd64. Wait for explorer.exe to enter _tWinMain —
    # that proves the entire user-mode chain (smss → csrss → winlogon → explorer)
    # is up. Post the 2026-07 arm64 MiPteToX self-map fix (which made teardown
    # ~10× faster — the old "~5 min TCG" lore was mostly that bug), a healthy
    # boot reaches the marker in ~36–40s even under parallel-gate contention.
    boot_marker: str = "[EXPLORER] _tWinMain entered"
    # 180s = ~4.5× the observed ~40s worst case — ample margin for a slow-but-
    # progressing boot, while a genuine wedge now fails in ≤180s not 540s. (Was
    # 540s, sized for the pre-fix ~5min boots.) A rare contention-slow boot that
    # trips this is classified HANG_TIMEOUT=slow and retried by the gate.
    boot_timeout: float = 180.0
    min_boot_bytes: int = 5000
    # QMP can be slow to respond when the kernel is in tight context-switch
    # loops on TCG — give it more headroom than i386/amd64.
    qmp_timeout: float = 60.0
    # Brief settle after _tWinMain to let the taskbar/desktop draw before
    # screenshot/snapshot tests run.
    desktop_settle_seconds: float = 15.0
    # A healthy boot reaches the marker in ~40s with continuous output, so 120s
    # of pure silence with no marker = a real wedge (3× the whole healthy boot,
    # well above any legitimate inter-line gap). Catches a wedge in ~2min instead
    # of ~4min. (Was 240s, sized against the pre-fix ~520s silent hangs.)
    hang_silence_seconds: float = 120.0
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
    # See Amd64Config: marker fires before the taskbar paints. 5s was too short
    # (bare blue desktop). 30s was sized on "taskbar up by +15.5s" lore, but a
    # 2026-07-05 timestamped boot measured taskbar child controls created at
    # marker+0.9s, and the +30s screenshot STILL shows an unpainted tray (the
    # known render-completeness issue) — the extra 15s bought nothing. 15s keeps
    # 3x the old too-short value and a real post-marker health window while
    # cutting a pure fixed wait from every green boot.
    desktop_settle_seconds: float = 15.0
    # Healthy i386 boot reaches the marker in ~60s; 120s of pure silence with no
    # marker = wedged. Above the ~30s storage/EventLog gap so that flake is not
    # false-tripped, well under the 360s timeout (saves ~4min per wedge).
    hang_silence_seconds: float = 120.0
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
        iso = _maybe_smp_iso(PROJECT_DIR / "build-i386" / "livecd.iso", work_dir)
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
        iso = _maybe_smp_iso(PROJECT_DIR / "build-i386" / "livecd.iso", work_dir)
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
    # the taskbar/Start button/tray paint a few seconds later. 2026-06-05 lore
    # said "taskbar up by +15.5s" and sized this 30s for 2x margin. A 2026-07-05
    # timestamped boot measured taskbar child controls created at marker+0.9s,
    # and the +30s screenshot still shows an unpainted tray (render-completeness
    # issue, tracked separately) — the second 15s buys nothing. 15s preserves
    # the post-marker health window (BugCheck/panic-during-settle detection)
    # while halving a fixed wait paid by every green boot, twice per amd64
    # smoke (plain + net-config boots).
    desktop_settle_seconds: float = 15.0
    # Healthy amd64 boot reaches the marker in ~60s; 120s of pure silence with no
    # marker = wedged. Above the ~30s storage/EventLog gap (so that KNOWN_FLAKE is
    # not false-tripped), well under the 360s timeout (saves ~4min per wedge).
    hang_silence_seconds: float = 120.0
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
        iso = _maybe_smp_iso(PROJECT_DIR / "build-amd64" / "livecd.iso", work_dir)
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
        iso = _maybe_smp_iso(PROJECT_DIR / "build-amd64" / "livecd.iso", work_dir)
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


# When set (by the windows-host backend, around the config's qemu_args() call),
# _free_tcp_port() delegates to this so hostfwd ports are picked on the Windows
# TCP stack — where QEMU.exe actually binds them. None on the native backend.
_windows_port_picker: Callable[[], int] | None = None


def _free_tcp_port() -> int:
    """Pick a currently-free TCP port for a slirp hostfwd rule.

    Native: bind-to-0 on the WSL stack then release (tiny reuse race, but QEMU
    rebinding it moments later in practice never collides; keeps parallel net VMs
    across agents/worktrees from fighting over a fixed port).

    Windows-host: the hostfwd binds `0.0.0.0:<port>` on the *Windows* stack, a
    different stack from WSL under NAT — so a WSL-free port can be unbindable on
    Windows (→ `Could not set up host forwarding rule`). When `_windows_port_picker`
    is set, delegate to it to pick a Windows-side-free port instead.
    """
    if _windows_port_picker is not None:
        return _windows_port_picker()
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
    # Short settle: every post-boot net assertion goes through
    # test_net._wait_for_marker, which polls the LIVE log with its own
    # timeout — nothing reads late markers out of the static _boot_output
    # capture (boot milestones all fire before the marker). The inherited
    # 15s idle-settle was pure dead time on this config (measured: guest
    # self-tests done by +13.4s, last boot output +15.6s, settle then idled
    # to +30.6s). 2s keeps a minimal post-marker health window.
    desktop_settle_seconds: float = 2.0
    # Guest-side TCP echo port the netstack self-test listens on (M2+).
    ECHO_GUEST_PORT: int = 7777
    # Host port forwarded to the guest echo port; allocated in qemu_args().
    echo_hostfwd_port: int = 0
    # Guest-side TCP port netstack's M8 *kernel TDI server* self-test listens on
    # (tdisrvtest.rs) — distinct from the engine's direct-smoltcp echo (:7777) so
    # the TDI listen/accept dispatch is what's exercised. Host port allocated in
    # qemu_args(); test_tdi_server_listen_accept dials it.
    TDI_SRV_GUEST_PORT: int = 8888
    tdi_srv_hostfwd_port: int = 0
    # Guest port a payload app (tools/payloads/*, e.g. httpd) listens on;
    # forwarded so test_payload's tcp_hammer can reach it from the host.
    PAYLOAD_GUEST_PORT: int = 8080
    payload_hostfwd_port: int = 0
    # slirp guestfwd: the guest reaches an OUTBOUND echo at this virtual
    # address:port, which slirp pipes through `cat` on the host (a built-in
    # line echo — no host-listener startup race). This is the path an outbound
    # TCP client (and later winsock-via-AFD) uses; the guest's serial markers
    # prove connect/send/recv.
    GUESTFWD_ADDR: str = "10.0.2.100"
    GUESTFWD_PORT: int = 9999

    def qemu_args(self, work_dir: Path) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        self.tdi_srv_hostfwd_port = _free_tcp_port()
        self.payload_hostfwd_port = _free_tcp_port()
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
            f",hostfwd=tcp:127.0.0.1:{self.tdi_srv_hostfwd_port}"
            f"-:{self.TDI_SRV_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.payload_hostfwd_port}"
            f"-:{self.PAYLOAD_GUEST_PORT}"
            f",guestfwd=tcp:{self.GUESTFWD_ADDR}:{self.GUESTFWD_PORT}-cmd:cat",
        ]


@dataclass
class I386UefiNetConfig(I386UefiConfig):
    """i386-uefi + a virtio-net-pci NIC on a slirp user netdev.

    The i386 sibling of Amd64UefiNetConfig — same slirp mini-internet, hostfwd /
    guestfwd contract, and modern-only virtio device. Used to validate M10's
    interrupt-driven RX on i386 (the x86 HAL HalGetInterruptVector + IOAPIC path
    is shared with amd64). Not a default smoke config: net is its own signal.
    """

    arch: str = "i386-uefi-net"
    # See Amd64UefiNetConfig: marker tests poll the live log; idle-settle is dead time.
    desktop_settle_seconds: float = 2.0
    ECHO_GUEST_PORT: int = 7777
    echo_hostfwd_port: int = 0
    TDI_SRV_GUEST_PORT: int = 8888
    tdi_srv_hostfwd_port: int = 0
    PAYLOAD_GUEST_PORT: int = 8080
    payload_hostfwd_port: int = 0
    GUESTFWD_ADDR: str = "10.0.2.100"
    GUESTFWD_PORT: int = 9999

    def qemu_args(self, work_dir: Path) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        self.tdi_srv_hostfwd_port = _free_tcp_port()
        self.payload_hostfwd_port = _free_tcp_port()
        return [
            *super().qemu_args(work_dir),
            "-device",
            "virtio-net-pci,disable-legacy=on,netdev=net0,id=vnet0",
            "-netdev",
            f"user,id=net0,hostfwd=tcp:127.0.0.1:{self.echo_hostfwd_port}"
            f"-:{self.ECHO_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.tdi_srv_hostfwd_port}"
            f"-:{self.TDI_SRV_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.payload_hostfwd_port}"
            f"-:{self.PAYLOAD_GUEST_PORT}"
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


@dataclass
class Arm64NetConfig(Arm64Config):
    """arm64 virt + a virtio-net-pci NIC on a slirp user netdev (M7 of
    docs/engineering/smoltcp-netstack-design.md — the arm64 net data path).

    arm64's existing virtio devices (kbd/mouse) are virtio-MMIO; the NIC is
    virtio-net-PCI, attached to the `virt` machine's Generic PCIe (GPEX) host
    bridge. netstack's PCI access is portable (HalGetBusDataByOffset over ECAM,
    newly implemented in the arm64 HAL), so the same device.rs/pci.rs path that
    works on amd64 enumerates here. Same slirp mini-internet + hostfwd/guestfwd
    contract as Amd64UefiNetConfig — see that class for the slirp topology.

    Deliberately NOT the default arm64 smoke config: boot smoke keeps its
    meaning (no NIC), net is its own signal (tests/qemu/test_net.py).
    """

    arch: str = "arm64-net"
    ECHO_GUEST_PORT: int = 7777
    echo_hostfwd_port: int = 0
    # M8 kernel TDI server self-test (tdisrvtest.rs) — see Amd64UefiNetConfig.
    TDI_SRV_GUEST_PORT: int = 8888
    tdi_srv_hostfwd_port: int = 0
    PAYLOAD_GUEST_PORT: int = 8080
    payload_hostfwd_port: int = 0
    GUESTFWD_ADDR: str = "10.0.2.100"
    GUESTFWD_PORT: int = 9999
    # Modest extra settle so the netstack engine markers that DO complete on
    # arm64 (dhcp/ping) reliably land in the captured output. NOTE: the deeper
    # self-tests (dns, outbound TCP) currently do NOT complete on arm64 — the
    # netstack worker stalls after `ping gateway OK` (an arm64-specific worker
    # bug, NOT a timing/settle issue: a 150s settle did not change it). Tracked
    # as an M7 follow-up; the core data path (ARP/DHCP/ICMP/TDI) is proven.
    desktop_settle_seconds: float = 30.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        self.tdi_srv_hostfwd_port = _free_tcp_port()
        self.payload_hostfwd_port = _free_tcp_port()
        return [
            *super().qemu_args(work_dir),
            # Modern-only virtio (PCI\VEN_1AF4&DEV_1041), same contract as the
            # amd64 net config; attaches to the virt GPEX PCIe root bus.
            "-device",
            "virtio-net-pci,disable-legacy=on,netdev=net0,id=vnet0",
            "-netdev",
            f"user,id=net0,hostfwd=tcp:127.0.0.1:{self.echo_hostfwd_port}"
            f"-:{self.ECHO_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.tdi_srv_hostfwd_port}"
            f"-:{self.TDI_SRV_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.payload_hostfwd_port}"
            f"-:{self.PAYLOAD_GUEST_PORT}"
            f",guestfwd=tcp:{self.GUESTFWD_ADDR}:{self.GUESTFWD_PORT}-cmd:cat",
        ]


# x86 is UEFI-only after the BIOS prune. I386Config / Amd64Config remain only as
# the (non-selectable) base classes that hold shared defaults for the -uefi
# variants; their legacy BIOS -cdrom qemu_args are no longer reachable.
ARCH_CONFIGS: dict[str, type[ArchConfig]] = {
    "arm64": Arm64Config,
    "arm64-test": Arm64TestKernelConfig,
    "arm64-net": Arm64NetConfig,
    "i386-uefi": I386UefiConfig,
    "i386-uefi-net": I386UefiNetConfig,
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
        gdb: bool = False,
        gdb_wait: bool = False,
    ) -> None:
        self.config = config
        self.work_dir = work_dir
        self.work_dir.mkdir(parents=True, exist_ok=True)

        # Optional gdbstub. On windows-host QEMU.exe is the SERVER (gdb is the client),
        # so — unlike QMP/serial, where the harness listens and QEMU dials back — the
        # stub binds 0.0.0.0:<port> on the WINDOWS TCP stack and a WSL gdb connects to
        # it via gateway_ip():<port> (transport-identical to a slirp hostfwd; see
        # gdb_host/gdb_port + _build_windows_cmd). gdb_wait adds -S (halt at reset, for
        # boot-from-zero debugging). Under the windows-host TCG backend software
        # breakpoints fire reliably — the reason this path exists (they DON'T under
        # native KVM; see qemu/start.sh). gdb_port is filled in at launch.
        self._gdb = gdb
        self._gdb_wait = gdb_wait
        self.gdb_port: int | None = None

        # Launch backend: "native" (local qemu + Unix sockets + KVM/TCG — unchanged) or
        # "windows-host" (Windows QEMU.exe via interop + TCP transport + staged media — see
        # backend.py / docs/engineering/windows-host-qemu-spike.md). windows = the
        # qemu.windows.* config subtree.
        self._backend = backend
        self._win = windows or {}
        self._win_uid = ""  # unique -name tag for uid-scoped kill (windows backend)
        self._win_staging: Path | None = None  # per-run Windows staging subdir (windows backend)

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
        self._cpu_lease_fd: int | None = None  # host CPU-broker lease (TCG boots only)
        # windows-host only: effective accelerator after the per-arch resolve (set in
        # _build_windows_cmd). "tcg" until then; gates whether _acquire_arch_slot serializes
        # same-arch boots (whpx contends like KVM, tcg doesn't).
        self._win_accel: str = "tcg"
        # windows-host only: harness echo server backing the slirp guestfwd when
        # `-cmd:cat` can't run (Windows host has no `cat`). None on native / non-net.
        self._echo_server: socketserver.BaseServer | None = None
        # stderr captured if QEMU exited early during launch (consumed once from the
        # PIPE), so the diagnostic path can still report it — see _launch_qemu.
        self._early_stderr: str = ""

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

    @property
    def gdb_host(self) -> str:
        """Address a WSL gdb dials to reach the gdbstub. Same reasoning as net_host
        (the stub is server-side, so it's the inbound/hostfwd direction): windows-host
        binds 0.0.0.0 on the Windows stack, reached from WSL via the gateway IP; native
        binds WSL loopback."""
        if self._backend == "windows-host":
            from . import backend as be

            return be.gateway_ip()
        return "127.0.0.1"

    def _gdb_args(self, win_stack: bool) -> list[str]:
        """The `-gdb`/`-S` flags when a stub is requested, else []. Picks the port on
        the stack QEMU binds it on: the Windows stack for windows-host (QEMU.exe is the
        gdb SERVER, so it binds locally there), the WSL stack for native. Sets
        self.gdb_port for gdb_host/callers to read."""
        if not self._gdb:
            return []
        if win_stack:
            from . import backend as be

            self.gdb_port = be.windows_free_tcp_port()
            bind = f"tcp:0.0.0.0:{self.gdb_port}"  # 0.0.0.0 so WSL reaches it via gateway
        else:
            self.gdb_port = _free_tcp_port()
            bind = f"tcp:127.0.0.1:{self.gdb_port}"
        return ["-gdb", bind, *(["-S"] if self._gdb_wait else [])]

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

    def _tripwire_args(self) -> list[str]:
        """vibeos tripwire channel: VIBEOS_TRIPWIRE=<cfg> injects per-boot kernel
        debug knobs (e.g. SPECIALPOOL=Gla5) via QEMU fw_cfg — no media rebuild.
        The kernel reads opt/org.vibeos.tripwire in MmArmInitSystem Phase 0
        (ntoskrnl-rust src/tripwire.rs). Space/semicolon-separated KEY=VAL tokens;
        commas are forbidden (QEMU's -fw_cfg parser owns the comma)."""
        cfg = os.environ.get("VIBEOS_TRIPWIRE", "").strip()
        if not cfg:
            return []
        if "," in cfg:
            raise ValueError(
                f"VIBEOS_TRIPWIRE must not contain commas (got {cfg!r}); "
                "use spaces or semicolons between KEY=VAL tokens"
            )
        return ["-fw_cfg", f"name=opt/org.vibeos.tripwire,string={cfg}"]

    def _smp_args(self) -> list[str]:
        """VIBEOS_QEMU_SMP=<N> boots the guest with N virtual CPUs (-smp N).
        Opt-in only (SMP bring-up — docs/engineering/smp-bringup-quest.md);
        default configs stay UP. All backends exercise real concurrency
        (TCG multi-vCPU is MTTCG — real host threads)."""
        n = os.environ.get("VIBEOS_QEMU_SMP", "").strip()
        if not n:
            return []
        if not n.isdigit() or int(n) < 1:
            raise ValueError(f"VIBEOS_QEMU_SMP must be a positive integer (got {n!r})")
        return ["-smp", n]

    def _extra_qemu_args(self) -> list[str]:
        """VIBEOS_QEMU_EXTRA_ARGS=<raw qemu args> appends arbitrary devices/options to
        the boot (shlex-split). Intended for device-bring-up rigs (usb-kbd, usb-tablet,
        usb-ehci, ...) without editing arch configs. CAVEAT (windows-host backend): file
        paths inside these args are NOT staged/rewritten to Windows paths — reference
        only harness-managed backends (e.g. drive=scratch) or non-file devices, or use
        VIBEOS_QEMU_USB_DISK for a staged USB disk."""
        raw = os.environ.get("VIBEOS_QEMU_EXTRA_ARGS", "").strip()
        if not raw:
            return []
        return shlex.split(raw)

    def _usb_disk_args(self) -> list[str]:
        """VIBEOS_QEMU_USB_DISK=1 attaches an EHCI controller + usb-storage device backed
        by a fresh 64M qcow2 next to the scratch disk (usbstor bring-up rig). The drive
        path lives beside scratch_path so the windows-host backend stages it like any
        other media (route through be.stage_media_paths there, same as scratch)."""
        if os.environ.get("VIBEOS_QEMU_USB_DISK", "").strip() not in ("1", "true", "yes"):
            return []
        usb_disk = self.scratch_path.parent / "usbdisk.qcow2"
        if not usb_disk.exists():
            subprocess.run(
                ["qemu-img", "create", "-f", "qcow2", str(usb_disk), "64M"],
                check=True,
                capture_output=True,
            )
        return [
            "-device",
            "usb-ehci,id=xehci",
            "-drive",
            f"file={usb_disk},format=qcow2,if=none,id=usbstor0",
            "-device",
            "usb-storage,bus=xehci.0,drive=usbstor0",
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
            *self._usb_disk_args(),
            *self._tripwire_args(),
            *self._smp_args(),
            *self._extra_qemu_args(),
            "-monitor",
            "none",
            *self._gdb_args(win_stack=False),
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

        is_arm64 = self.config.arch.startswith("arm64")
        # Per-arch accel resolve: x86 honors the configured value (default whpx), arm64 is
        # forced to tcg (whpx is x86-only). Only probe WHPX when it could change the outcome
        # (x86 + configured whpx) so arm64/tcg runs pay no PowerShell cost; a definite
        # "feature Disabled" degrades x86 to tcg, an inconclusive probe trusts config.
        configured = self._win.get("accel", "tcg")
        whpx_ok = be.whpx_available() if (configured == "whpx" and not is_arm64) else None
        accel = be.resolve_accel(self.config.arch, configured, whpx_ok)
        self._win_accel = accel  # read by _acquire_arch_slot (whpx contends like KVM)
        if accel == "whpx":
            # WHPX's kernel phase is fast (hardware virt), but the FIRMWARE phase is
            # slow AND serial-silent: OVMF reads the livecd via ATAPI PIO, and every
            # 2-byte port read is a VM exit — the uefildr file-load stretch runs
            # 3-5 min with zero serial output (progress is visible only on debugcon).
            # The x86 defaults (120s silence watchdog / 360s boot timeout) are tuned
            # for TCG's fast port I/O and false-abort a healthy whpx boot, so scale
            # both. See docs/engineering/windows-host-qemu-whpx-spike.md.
            self.config.hang_silence_seconds = max(self.config.hang_silence_seconds, 420.0)
            self.config.boot_timeout = max(self.config.boot_timeout, 900.0)
        # Per-run staging SUBDIR under the (shared) Windows staging root. Concurrent same-arch
        # windows-host smokes across worktrees must not stomp each other's media — the per-arch
        # KVM slot does NOT serialize windows-host (it runs off-host and early-returns the slot),
        # so without a per-run dir two amd64 runs collide on livecd.iso/debugcon.bin (false
        # LAUNCH_FAIL / false net failures). Mirrors the native per-run /tmp/vibeos-cli/<ts>/boot.
        staging_root = Path(self._win.get("staging_dir", "/mnt/c/temp/vibeos-qemu"))
        run_tag = f"{os.getpid()}-{self.work_dir.parent.name}-{self.work_dir.name}"
        staging = staging_root / run_tag
        self._win_staging = staging
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

        # Pick hostfwd ports on the WINDOWS stack (where QEMU.exe binds them), not
        # WSL's — see _free_tcp_port / be.windows_free_tcp_port. The hook is read by
        # _free_tcp_port inside the net configs' qemu_args(); scoped tightly so the
        # native path and non-net configs are untouched.
        global _windows_port_picker
        _windows_port_picker = be.windows_free_tcp_port
        try:
            raw_arch_args = self.config.qemu_args(self.work_dir)
        finally:
            _windows_port_picker = None
        staged_arch_args = be.stage_media_paths(raw_arch_args, staging)
        # arm64 already boots TCG with a real -cpu MODEL (cortex-a72) and
        # -accel tcg,thread=multi — exactly what we want on the Windows host.
        # rewrite_accel is an x86 concept (swap KVM->whpx/tcg and drop the
        # KVM-only `-cpu host`); on arm64 it would DROP the required -cpu model,
        # so leave arm64's machine args untouched.
        arch_args = (
            staged_arch_args
            if is_arm64
            else be.rewrite_accel(staged_arch_args, accel, self.config.arch)
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
                f"socket,id={chardev_id},host={host},port={echo_port},server=off,reconnect-ms=1000",
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
            *be.stage_media_paths(self._usb_disk_args(), staging),
            *self._tripwire_args(),
            *self._smp_args(),
            *self._extra_qemu_args(),
            "-monitor",
            "none",
            # debugcon = x86 ISA port 0xE9, absent on the arm64 `virt` machine
            # (mirror the native path's arm64 skip in _build_native_cmd).
            *([] if is_arm64 else ["-debugcon", f"file:{be.win_path(self.debugcon_path)}"]),
            *self._gdb_args(win_stack=True),
            *echo_chardev,
            *arch_args,
        ]

    def _launch_qemu(self) -> list[str]:
        """Build the qemu cmd and Popen it into self._proc; return the launched cmd.
        On the windows-host backend a slirp hostfwd OR the gdbstub binds a Windows-stack
        port with a pick-then-bind TOCTOU; ride it out by relaunching with a freshly-
        picked port up to HOSTFWD_LAUNCH_RETRIES times. Native / non-hostfwd / non-gdb
        configs launch exactly once (no early-exit poll, so their healthy path pays no
        latency)."""
        attempts = HOSTFWD_LAUNCH_RETRIES if self._backend == "windows-host" else 1
        last_stderr = ""
        for attempt in range(attempts):
            if self._backend == "windows-host":
                cmd = self._build_windows_cmd()
            else:
                cmd = self._build_native_cmd()
            print(f"[QemuVM] {' '.join(cmd)}")
            proc = subprocess.Popen(
                cmd,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                preexec_fn=os.setpgrp,
            )
            # A Windows-stack bind that can lose the port race: a hostfwd, or the gdbstub.
            win_bind = self._backend == "windows-host" and (
                any("hostfwd=" in a for a in cmd) or self._gdb
            )
            if not win_bind:
                self._proc = proc
                return cmd
            early = self._early_exit_stderr(proc)
            if early is None:
                self._proc = proc  # still running at the deadline → healthy launch
                return cmd
            if any(e in early for e in BIND_RACE_ERRORS):
                # Lost the port race — discard this attempt and relaunch with a fresh
                # port (the next _build_windows_cmd re-picks + re-binds).
                self._discard_failed_launch(proc)
                last_stderr = early
                print(
                    f"[QemuVM] Windows-port bind race (attempt {attempt + 1}/{attempts}), "
                    "retrying with a fresh port"
                )
                continue
            # Exited early for some OTHER reason — not the flake. Stash stderr for the
            # diagnostic path and let the normal start sequence surface the dead proc.
            self._early_stderr = early
            self._proc = proc
            return cmd
        raise RuntimeError(
            f"QEMU hostfwd-bind failed after {attempts} attempts (Windows port TOCTOU).\n"
            f"Last QEMU stderr:\n{last_stderr}"
        )

    def _early_exit_stderr(self, proc: subprocess.Popen, budget: float = 2.0) -> str | None:
        """Poll up to `budget`s for an early QEMU exit. If it exits, return its full
        stderr (consumed once). If still running at the deadline, return None (healthy).
        Reads stderr only after exit, so a healthy long-running QEMU's pipe is never
        drained here (same as the pre-existing behavior)."""
        deadline = time.monotonic() + budget
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                if proc.stderr:
                    try:
                        return proc.stderr.read().decode("utf-8", errors="replace")
                    except OSError:
                        return ""
                return ""
            time.sleep(0.05)
        return None

    def _discard_failed_launch(self, proc: subprocess.Popen) -> None:
        """Tear down a hostfwd-raced launch attempt: reap the dead qemu, stop the echo
        server, and replace the serial/qmp clients with fresh instances — SerialClient
        can't be reused after close() (its drainer won't restart once _closed is set).
        The next _build_windows_cmd re-binds new listeners + re-stages media."""
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        if self._echo_server is not None:
            try:
                self._echo_server.shutdown()
                self._echo_server.server_close()
            except Exception:
                pass
            self._echo_server = None
        self.serial.close()
        self.qmp.close()
        self.qmp = QmpClient()
        self.serial = SerialClient()

    def start(self) -> None:
        """Start QEMU process, connect QMP + serial (backend-aware)."""
        # Acquire a global concurrency slot BEFORE launching — and before we
        # record the start/deadline below, so the time spent queued here is
        # excluded from the 30-min lifetime cap. Held for this pytest process's
        # lifetime; released in stop()/atexit. Acquired once, ahead of any
        # hostfwd-bind relaunch attempts.
        self._acquire_slot()

        # Host CPU broker: a TCG boot is CPU-bound emulation, so take one lease from the
        # writer-takes-all pool BEFORE launch — a concurrent cold build grabs the whole pool
        # (exclusive), so this blocks (QEMU not yet spawned → no wasted CPU) until the build
        # frees it, instead of starving into a false HANG_TIMEOUT. Like the slot, the wait
        # happens before `started` is recorded so it's excluded from the lifetime cap.
        self._acquire_cpu_lease()

        cmd = self._launch_qemu()

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

            staging = self._win_staging or Path(
                self._win.get("staging_dir", "/mnt/c/temp/vibeos-qemu")
            )
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
        # windows-host on TCG (all arm64, and x86 when whpx is off) runs off-host with no
        # shared accelerator, so it does NOT contend under same-arch concurrency — the
        # per-arch serialize cap is the nested-KVM workaround it obsoletes, so skip it.
        # WHPX is the exception: it's REAL hardware virt on the Windows host, so same-arch
        # x86 whpx guests contend for it like KVM → fall through and serialize those. getattr
        # defaults keep slot-only partial QemuVM constructions (tests) working.
        if (
            getattr(self, "_backend", "native") == "windows-host"
            and getattr(self, "_win_accel", "tcg") != "whpx"
        ):
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

    def _boot_is_tcg(self) -> bool:
        """Whether this boot runs under CPU-bound TCG emulation → it should take a host
        CPU-broker lease so a concurrent cold build (writer-takes-all) can't starve it into
        a false HANG_TIMEOUT. KVM/WHPX boots are hardware-accelerated → no lease.

        - native arm64: always TCG (no aarch64 KVM in WSL2)
        - native x86: KVM when /dev/kvm exists (the pc/q35 configs hardcode `-accel kvm`),
          else QEMU itself degrades to TCG → treat as CPU-heavy
        - windows-host: TCG unless WHPX resolved on (x86 only). Resolved during launch and
          cached on `_win_accel` ('tcg' until then), so a pre-launch call conservatively
          treats it as TCG — correct today (WHPX shelved → all windows-host boots are TCG)
          and only ever over-takes one of K permits if WHPX returns."""
        if getattr(self, "_backend", "native") == "windows-host":
            return getattr(self, "_win_accel", "tcg") != "whpx"
        if self.config.arch.startswith("arm64"):
            return True
        return not Path("/dev/kvm").exists()

    def _acquire_cpu_lease(self) -> None:
        """Take one host CPU-broker lease for a TCG boot (no-op when the broker is off or the
        boot is hardware-accelerated). Held on `_cpu_lease_fd` for the VM's lifetime, released
        in stop(). On timeout (a stuck holder, not real contention) warn + proceed UNLOCKED —
        never wedge a boot forever. Called before launch so a queued boot doesn't burn CPU
        while it waits behind an exclusive build."""
        if not hostlock.enabled() or not self._boot_is_tcg():
            return
        # Route by scheduling domain: a windows-host boot runs as qemu.exe on the WINDOWS
        # side, so it draws from the SEPARATE `bootcpu` pool and is NOT excluded by a WSL
        # build's exclusive `cpuheavy` grant (the build leaves core headroom via its -j cap).
        # A native TCG boot (arm64, or x86-degraded) runs in WSL and shares cores with the
        # build, so it stays on `cpuheavy` where a cold build still excludes it.
        if getattr(self, "_backend", "native") == "windows-host":
            self._cpu_lease_fd = hostlock.acquire_boot_lease(
                f"boot:{self.config.arch}", work_dir=str(self.work_dir)
            )
        else:
            self._cpu_lease_fd = hostlock.acquire_cpu_lease(
                f"boot:{self.config.arch}", work_dir=str(self.work_dir)
            )
        if self._cpu_lease_fd is None:
            print(
                f"[QemuVM] WARNING: host CPU lease timeout (boot:{self.config.arch}) — "
                "proceeding UNLOCKED (a holder looks stuck; check `vibeos-cli host-status`)"
            )

    def _release_slot(self) -> None:
        """Release the concurrency slot(s) (idempotent): the per-arch sub-cap
        lock, the global slot, and the host CPU-broker lease."""
        hostlock.release(self._cpu_lease_fd)
        self._cpu_lease_fd = None
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

        # Remove this run's per-run Windows staging subdir (only ours — never the shared root,
        # which other concurrent runs are using). Bounded retry: on DrvFs the Windows QEMU
        # releases the staged file handles asynchronously after the kill above, so an immediate
        # rmtree can no-op and leak the (~400MB) dir; retry until it's gone.
        if self._win_staging is not None:
            for _ in range(5):
                shutil.rmtree(self._win_staging, ignore_errors=True)
                if not self._win_staging.exists():
                    break
                time.sleep(0.3)
            self._win_staging = None

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
        # Fall back to stderr the launch-time early-exit poll already consumed from
        # the PIPE (read() above would return empty in that case).
        stderr = stderr or self._early_stderr
        raise RuntimeError(f"{msg}\nQEMU stderr:\n{stderr}")
