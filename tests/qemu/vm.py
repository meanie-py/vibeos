"""QEMU VM lifecycle manager for ReactOS test sessions."""

from __future__ import annotations

import atexit
import contextlib
import fcntl
import json
import os
import re
import shlex
import shutil
import signal
import socket
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

from .cli import hostlock
from .portlease import launch_lock
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
# Overridable via VIBEOS_QEMU_MAX_LIFETIME, the same env var qemu/lib/common.sh
# has always honoured (its comment says it mirrors THIS constant, and for a long
# time it silently did not: the shell path was tunable and the pytest path was
# not). A long conformance sweep legitimately outlives 30 minutes, and without
# this the cap is a second, lower, invisible bound underneath the payload's own
# launch_drain_secs -- which is exactly how the apitest sweep kept getting
# truncated at ~30 min after its drain budget was fixed. Raise it deliberately,
# per-run, for sweeps; never as a global default.
MAX_VM_LIFETIME_SECONDS = int(os.environ.get("VIBEOS_QEMU_MAX_LIFETIME", 30 * 60))

# Gap between the two vCPU-state samples save_hang_state() takes. Long enough
# that a live box is guaranteed several clock ticks and context switches (arm64
# TCG runs the scheduler at 100 Hz), short enough to add nothing meaningful to a
# run that has already drained its whole timeout.
HANG_RESAMPLE_SECONDS = 5.0

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
# The mirror-image flake on the OTHER side of the same boundary: every `server=off`
# chardev (serial, qmp, each guestfwd echo) has the Windows QEMU DIAL a listener we
# just bound in WSL, and WSL2's localhost relay intermittently has not published the
# port yet -- QEMU exits before any guest code with
#   -chardev socket,id=gf2,...,server=off: Failed to connect to '127.0.0.1:58627'
# We bind every one of those listeners ourselves moments earlier, so a connect refusal
# is never a misconfiguration; a relaunch on a fresh port is the whole fix. This cost
# a whole smoke run its net suite (15 errors, all one dead launch) while the three
# boots it was grading passed.
CHARDEV_CONNECT_ERROR = "Failed to connect to '"
# Any early-exit stderr matching one of these = a transient Windows-stack port
# problem → relaunch with a freshly-picked port (see _launch_qemu).
BIND_RACE_ERRORS = (HOSTFWD_BIND_ERROR, GDB_BIND_ERROR, CHARDEV_CONNECT_ERROR)
HOSTFWD_LAUNCH_RETRIES = int(os.environ.get("VIBEOS_HOSTFWD_LAUNCH_RETRIES", "4"))

# The usbstor rig's stick (VIBEOS_QEMU_USB_DISK=1): an MBR disk, one FAT16
# partition at 1 MiB labelled USB_STICK_LABEL, holding USB_STICK_FILE.
USB_STICK_LABEL = "VIBESTICK"
USB_STICK_FILE = "VIBEOS.TXT"
USB_STICK_TEXT = b"hello from the usb stick\r\n"
_USB_STICK_MIB = 64


def make_usb_stick_image(path: Path) -> None:
    """Build the usbstor rig's stick image at `path`, as qcow2 (host sfdisk + mkfs.fat + mcopy)."""
    part_offset = 1 << 20
    part_bytes = (_USB_STICK_MIB << 20) - part_offset
    with tempfile.TemporaryDirectory() as td:
        part = Path(td) / "part.img"
        payload = Path(td) / USB_STICK_FILE
        payload.write_bytes(USB_STICK_TEXT)
        subprocess.run(
            ["mkfs.fat", "-F", "16", "-n", USB_STICK_LABEL, "-C", str(part), str(part_bytes >> 10)],
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["mcopy", "-i", str(part), str(payload), f"::/{USB_STICK_FILE}"],
            check=True,
            capture_output=True,
        )
        tmp = path.with_suffix(".tmp")
        with tmp.open("wb") as f:
            f.truncate(_USB_STICK_MIB << 20)
            f.seek(part_offset)
            f.write(part.read_bytes())
        subprocess.run(
            ["sfdisk", "--no-reread", "--no-tell-kernel", str(tmp)],
            input=b"label: dos\nstart=2048, type=06\n",
            check=True,
            capture_output=True,
        )
        # qcow2, not raw: the rigs snapshot the VM (savevm), and QEMU refuses
        # while any writable drive lacks snapshot support.
        subprocess.run(
            ["qemu-img", "convert", "-f", "raw", "-O", "qcow2", str(tmp), str(path)],
            check=True,
            capture_output=True,
        )
        tmp.unlink()


# freeldr's boot-entry selector, as it appears verbatim inside the livecd ISO.
# The x86 media has always carried BOTH flavors — [LiveCD_SMP] (/KERNEL=ntkrnlmp.exe
# /HAL=halmp.dll) ships in every livecd.ini and all four binaries are staged — but
# freeldr chooses from this one line and we set TimeOut=0, so no menu ever appears
# to choose at. There is no boot-time channel to tell freeldr otherwise: -append
# applies only to a direct `-kernel` Linux boot, not a UEFI CD boot of freeldr.
_INI_DEFAULT_OS = b"DefaultOS="
_INI_SMP_ENTRY = b"LiveCD_SMP"

# The shipped default entry -> the MP twin that boots ntkrnlmp.exe + halmp.dll,
# per media flavor. arm64's boot.img is a FAT image rather than ISO9660, but it
# stores freeldr.ini's bytes literally and contiguously just the same, and
# `DefaultOS=` (with the `=`) occurs exactly once in it — the bare `DefaultOS`
# token also appears inside uefildr.efi, which is why the needle keeps the `=`.
# So one poke serves all three arches and arm64 needs no mtools at boot time.
_MP_ENTRY_FOR = {
    b"LiveCD_Debug": _INI_SMP_ENTRY,  # i386/amd64 livecd.iso
    b"ReactOS_Arm64_UP": b"ReactOS_Arm64_MP",  # arm64 qemu/boot.img
}


def _write_sparse_image(path: Path, data: bytes | bytearray) -> None:
    """Own-copy media: retain every byte without allocating all-zero extents.

    No links, shared backing file, or dependency on the source after creation.
    Filesystems without sparse support still produce the exact same image.
    """
    block = 64 * 1024
    zeros = bytes(block)
    with path.open("wb") as output:
        for offset in range(0, len(data), block):
            chunk = data[offset : offset + block]
            if chunk == zeros[: len(chunk)]:
                output.seek(len(chunk), os.SEEK_CUR)
            else:
                output.write(chunk)
        output.truncate(len(data))


def smp_selected_iso(iso: Path, work_dir: Path) -> Path:
    """Boot media whose freeldr default entry is the MP pair, WITHOUT rebuilding it.

    ISO9660 stores each file's bytes once, contiguously, with no per-file checksum,
    and freeldr.ini is plain text — so flipping the default is an equal-length
    in-place write of one line into a copy of the image. The alternative was a cmake
    reconfigure + mkisofs repack (8m30s measured on amd64) or shipping a second
    221 MB media; this is a 0.15s copy and a 22-byte poke.

    Works on arm64's qemu/boot.img too, which is FAT rather than ISO9660 but stores
    the ini's bytes just as literally; its entries are named ReactOS_Arm64_UP/_MP at
    equal length precisely so this poke needs no padding there. Before that, arm64
    got its MP pair from a DIFFERENT BUILD (`mkramdisk.sh --smp` rewrote Options=
    and staged the binaries only then), so forgetting `--smp-kernel` produced
    identical-looking media that silently booted UP — the one false-green this
    whole mechanism exists to remove.

    Padding: `DefaultOS=LiveCD_SMP` is 20 bytes and the shipped
    `DefaultOS=LiveCD_Debug` is 22, so the remainder becomes `\\r\\n` (a blank line
    the INI parser ignores) plus `;` comment filler for any excess. A 1-byte
    remainder cannot form a clean line terminator, so it raises rather than emitting
    a lone CR.

    Every failure path raises. Booting UP media on an `--smp` run is precisely the
    false-green `_smp_check` exists to catch, and a silent fallback here would
    manufacture exactly that."""
    data = bytearray(iso.read_bytes())
    # Anchored to a line start, because freeldr only reads the key there and a
    # COMMENT is free to mention it: qemu/freeldr.ini's own note about keeping the
    # two entry names equal-length says "DefaultOS=" in prose, and an unanchored
    # scan counted that as a second selector and refused to patch anything.
    # str.find in a loop, not a per-offset startswith comprehension — this runs
    # over a 293 MB boot.img and the comprehension walked every byte in Python.
    hits = []
    i = data.find(_INI_DEFAULT_OS)
    while i != -1:
        if i == 0 or data[i - 1] == 0x0A:
            hits.append(i)
        i = data.find(_INI_DEFAULT_OS, i + 1)
    if len(hits) != 1:
        raise RuntimeError(
            f"{iso}: expected exactly one line-initial {_INI_DEFAULT_OS.decode()} in the "
            f"image, found {len(hits)} — freeldr.ini is not where this expects it; rebuild "
            "with `build --smp-kernel` instead of patching"
        )
    start = hits[0]
    end = start
    while end < len(data) and data[end] not in (0x0D, 0x0A):
        end += 1
    old = bytes(data[start:end])
    old_entry = old[len(_INI_DEFAULT_OS) :]
    if old_entry in _MP_ENTRY_FOR.values():
        return iso  # media already defaults to the MP entry (built --smp-kernel)
    # Unknown default -> assume x86-shaped media, so unfamiliar livecd.ini spellings
    # still get the LiveCD_SMP treatment (and still RAISE below if it isn't there)
    # rather than being handed back unpatched.
    mp_entry = _MP_ENTRY_FOR.get(old_entry, _INI_SMP_ENTRY)
    if b"[" + mp_entry + b"]" not in data:
        raise RuntimeError(
            f"{iso}: no [{mp_entry.decode()}] section — this media predates the "
            "halmp.dll twin and cannot boot the MP pair; rebuild it"
        )
    new = _INI_DEFAULT_OS + mp_entry
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
    _write_sparse_image(out, data)
    return out


def tripwire_args() -> list[str]:
    """vibeos tripwire channel: VIBEOS_TRIPWIRE=<cfg> injects per-boot kernel
    debug knobs (e.g. SPECIALPOOL=Gla5) via QEMU fw_cfg — no media rebuild.
    The kernel reads opt/org.vibeos.tripwire in MmArmInitSystem Phase 0
    (ntoskrnl-rust src/tripwire.rs). Space/semicolon-separated KEY=VAL tokens;
    commas are forbidden (QEMU's -fw_cfg parser owns the comma).

    Module-level so every launcher shares it: the gdb engine once assembled its
    own command line without this, so a debug boot silently ran WITHOUT the
    tripwire it was started under — the one boot where you most want it."""
    cfg = os.environ.get("VIBEOS_TRIPWIRE", "").strip()
    if not cfg:
        return []
    if "," in cfg:
        raise ValueError(
            f"VIBEOS_TRIPWIRE must not contain commas (got {cfg!r}); "
            "use spaces or semicolons between KEY=VAL tokens"
        )
    return ["-fw_cfg", f"name=opt/org.vibeos.tripwire,string={cfg}"]


def smp_count() -> int | None:
    """The vCPU count this run asked for (VIBEOS_QEMU_SMP), or None for a UP boot."""
    n = os.environ.get("VIBEOS_QEMU_SMP", "").strip()
    if not n:
        return None
    if not n.isdigit() or int(n) < 1:
        raise ValueError(f"VIBEOS_QEMU_SMP must be a positive integer (got {n!r})")
    return int(n)


def testkernel_boot_img(src: Path, work_dir: Path) -> Path:
    """The /TESTKERNEL variant of arm64 boot.img, built ONCE and shared read-only.

    This used to be a 280 MiB `copy2` plus an mcopy poke on every boot, i.e. on
    every construction of this config.

    Sizing it honestly, because the first version of this comment did not: the
    /TESTKERNEL config is driven by tests/qemu/test_kernel.py, whose `vm`
    fixture is scope="session", so its 28 tests share ONE boot and cost one
    copy per pytest session rather than 28. The waste shows up wherever
    qemu_args is called repeatedly instead -- `smoke --arch arm64-test
    --repeat N` pays it N times -- and it is pure waste in every case, since
    the transform is deterministic and every copy held identical bytes. (Do not
    confuse this with the conformance/kmtest sweep, which maps arm64 to plain
    Arm64Config and never went through this path at all.)

    Sharing is safe here, and it is not a new practice: `Arm64Config` already
    hands every parallel arm64 boot the SAME `qemu/boot.img`, and the drive is
    attached `readonly=on,file.locking=off`. The per-boot copy was the anomaly —
    the transform is deterministic (extract freeldr.ini, add /TESTKERNEL if
    absent, write it back), so every copy held identical bytes.

    The recorded hazard this must not repeat is SIGBUS on a shared inode, which
    happened when a tool rewrote a file IN PLACE while a sibling had it mapped.
    So this never writes the shared file in place: it builds a private temp copy
    and `os.replace`s it into position. Replace is atomic and gives the new
    content a NEW inode, so a boot that already opened the old one keeps reading
    valid bytes until it exits. Two sweeps racing both build and one replace
    wins; the bytes are identical either way.

    The cache key is the source's (mtime_ns, size), so a rebuild that re-stages
    boot.img produces a new key and the stale variant is ignored, then pruned.
    Unlinking a stale key is safe even if a boot still has it open — the inode
    outlives the name.
    """
    st = src.stat()
    key = f"{st.st_mtime_ns:x}-{st.st_size:x}"
    cached = src.parent / f"boot-test.{key}.img"

    if not cached.exists():
        staging = work_dir / "boot-test.staging.img"
        shutil.copy2(src, staging)
        ini_path = work_dir / "freeldr.ini"
        subprocess.run(
            ["mcopy", "-o", "-i", str(staging), "::/freeldr.ini", str(ini_path)],
            check=True,
            capture_output=True,
        )
        ini_text = ini_path.read_text()
        if "/TESTKERNEL" not in ini_text:
            ini_text = ini_text.replace("/SOS /RDPATH=", "/SOS /TESTKERNEL /RDPATH=")
            ini_path.write_text(ini_text)
        subprocess.run(
            ["mcopy", "-o", "-i", str(staging), str(ini_path), "::/freeldr.ini"],
            check=True,
            capture_output=True,
        )
        # Same filesystem as `cached`, so this is an atomic rename rather than a
        # copy — no window where a boot could open a half-written image.
        os.replace(staging, cached)

    for stale in src.parent.glob("boot-test.*.img"):
        if stale != cached:
            stale.unlink(missing_ok=True)

    return cached


def _maybe_smp_iso(iso: Path, work_dir: Path) -> Path:
    """`smp_selected_iso` when this run asked for SMP, else the media untouched."""
    return smp_selected_iso(iso, work_dir) if smp_count() else iso


def smp_boot_timeout(base: float, n: int | None) -> float:
    """Scale a UP-tuned boot_timeout for an `-smp N` boot.

    Under MTTCG every guest vCPU is a host thread doing its own translation and
    execution, so N vCPUs on an already-contended host stretch the wall clock to
    any given milestone roughly linearly — and the guest genuinely does more work
    too (AP bring-up, IPI round-trips, real spinlock contention). arm64's 180s is
    sized against a ~40s UP boot; at -smp 2 a healthy boot that had reached
    `explorer.exe created` and was still emitting SCM output tripped it anyway.

    This scales the CLOCK only. The boot marker is unchanged, and
    `hang_silence_seconds` is deliberately NOT scaled: the silence watchdog is
    what actually catches a wedge, so a genuine MP hang still fails in ~120s
    instead of burning the whole stretched budget.
    """
    return base if not n or n < 2 else base * n


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
    base = arch.split("-", 1)[0]
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


def _reap_kill(info: dict) -> tuple[bool, str]:
    """Stop an orphan the PRECISE way for its backend -- killvm.kill_one, the same path as
    `vibeos-cli kill`. The reaper used to `killpg` the recorded pid, which on the default
    windows-host backend is only the WSL interop shim: the real qemu.exe could survive
    it, and the reaper then deleted the lock, leaving a VM that `ps`, `kill` and the
    lifetime cap could no longer see. Returns (stopped, outcome line)."""
    from .cli import killvm

    outcome = killvm.kill_one(info)
    failed = outcome.startswith(("FAILED", "STILL ALIVE")) or "SIGKILL FAILED" in outcome
    return not failed, outcome


def _runs_dir() -> Path:
    """The kernel-debug toolkit's manifests (qemu/lib/common.sh: RUNS_DIR)."""
    return Path(os.environ.get("RUNS_DIR") or PROJECT_DIR / "qemu" / "runs")


def reap_orphan_qemus(*, dry_run: bool = False) -> list[str]:
    """Kill ONLY QEMUs whose launcher process is gone, or that are past their lifetime
    deadline. Returns the action log.

    Walks both trackers: the smoke harness's lock files and the kernel-debug toolkit's
    run manifests (qemu/start.sh). The second walk was `qemu/reap.sh`, where a jq that was
    not installed made every lock field read empty -- so every lock looked stale, and
    each new smoke purged every live VM's lock. It is here now, where a field is a dict
    lookup and "could not read" is an exception rather than an empty string.

    Safe to call from any session: a tracker whose owner pid is still alive and whose
    VM is within its deadline is left strictly untouched (it may be another agent's).
    """
    actions: list[str] = []
    now = time.time()
    if LOCK_DIR.is_dir():
        for lf in LOCK_DIR.glob("*.json"):
            try:
                info = json.loads(lf.read_text())
            except (OSError, ValueError):
                # Only a lock that has STAYED unparseable is garbage. One caught
                # mid-write by an older launcher (before locks were renamed into
                # place) belongs to a live VM, and deleting it unregisters that VM.
                try:
                    if not dry_run and now - lf.stat().st_mtime > 60:
                        lf.unlink(missing_ok=True)
                except OSError:
                    pass
                continue
            if not isinstance(info, dict):
                continue
            lpid, qpid = info.get("launcher_pid"), info.get("qemu_pid")
            deadline = info.get("deadline")
            launcher_alive = isinstance(lpid, int) and _pid_alive(lpid)
            # A VM past its hard cap is reaped REGARDLESS of launcher liveness -- the
            # cap is an absolute ceiling, not an orphan check. A live launcher only
            # protects a VM that is still within its lifetime.
            expired = isinstance(deadline, (int, float)) and now >= deadline
            if launcher_alive and not expired:
                continue  # live session (maybe another agent), within cap -- DO NOT TOUCH
            reason = "exceeded lifetime cap" if expired else f"launcher {lpid} dead"
            linux_side = isinstance(qpid, int) and _pid_alive(qpid) and _proc_is_qemu(qpid)
            # A windows-host VM is killed by its uid even when the Linux shim is gone:
            # the shim dying says nothing about qemu.exe on the other side.
            from .cli import killvm

            if linux_side or killvm.win_uid_of(info):
                if dry_run:
                    actions.append(f"DRY-RUN would reap qemu pid={qpid} ({reason})")
                    continue
                stopped, outcome = _reap_kill(info)
                if not stopped:
                    actions.append(f"FAILED to reap qemu pid={qpid} ({reason}): {outcome}")
                    continue  # keep the lock: never claim a VM is gone when it may not be
                actions.append(f"killed orphan qemu pid={qpid} ({reason}): {outcome}")
            if not dry_run:
                lf.unlink(missing_ok=True)
    runs = _runs_dir()
    if runs.is_dir():
        for manifest in sorted(runs.glob("*/manifest.json")):
            rid = manifest.parent.name
            try:
                info = json.loads(manifest.read_text())
            except (OSError, ValueError):
                continue  # unknown is not stale
            if not isinstance(info, dict) or info.get("stopped_at"):
                continue
            qpid = info.get("qemu_pid")
            if not (isinstance(qpid, int) and _pid_alive(qpid) and _proc_is_qemu(qpid)):
                continue
            opid = info.get("owner_pid") or info.get("parent_pid")
            deadline = info.get("deadline")
            if isinstance(deadline, (int, float)) and now >= deadline:
                reason = f"manifest {rid} exceeded lifetime cap"
            elif not isinstance(opid, int) or _pid_alive(opid):
                continue  # legacy manifest without an owner, or owner still here
            else:
                reason = f"manifest {rid} owner pid={opid} dead"
            if dry_run:
                actions.append(f"DRY-RUN would reap qemu pid={qpid} ({reason})")
                continue
            stopped, outcome = _reap_kill(info)
            verb = "killed orphan" if stopped else "FAILED to reap"
            actions.append(f"{verb} qemu pid={qpid} ({reason}): {outcome}")
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

    def prepare_args(self, work_dir: Path) -> list[str]:
        """Prepare media without selecting any released TCP ports."""
        return self.qemu_args(work_dir)

    def network_args(self) -> list[str]:
        """Select hostfwd ports only inside the launch handshake lock."""
        return []


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
        # Same boot-entry selection x86 gets: the ramdisk always carries both
        # kernel/HAL pairs and both freeldr entries, so an --smp run repoints
        # DefaultOS= in a COPY of boot.img instead of needing --smp-kernel media.
        boot_img = _maybe_smp_iso(QEMU_DIR / "boot.img", work_dir)
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
            *boot_media_device_args(iso),
            "-vga",
            "std",
            "-display",
            "none",
            "-no-reboot",
        ]


def boot_media_device_args(iso, ide_bus: str | None = None) -> list[str]:
    """How the UEFI boot ISO is attached — `ide-cd` unless told otherwise.

    `VIBEOS_QEMU_BOOT_TRANSPORT=usb-storage|virtio-blk` swaps the device model,
    which exists because two things need a NON-ATAPI boot device and neither
    could get one without hand-rolling a QEMU command line:

    * record/replay refuses `ide-cd`/`scsi-cd` outright — QEMU 9.0.2's
      `blkreplay` stalls the ATAPI path — so x86 rr is unreachable while the ISO
      can only be a CD.
    * freeldr's UEFI boot-path code picks its ARC name from the boot handle
      (`RemovableMedia && BlockSize == 2048` means cdrom, else rdisk+partition),
      and that second branch has never been exercised here. A probe found the
      boot going quiet right after "Switching stack..." over both virtio-blk and
      usb-storage, and reproducing that needed a one-off QEMU invocation that
      bypassed the staleness and media guards every other boot goes through.

    A remembered recipe is a missing mechanism, so this is the mechanism.
    """
    transport = os.environ.get("VIBEOS_QEMU_BOOT_TRANSPORT", "").strip()
    drive = ["-drive", f"id=cd0,if=none,format=raw,file={iso},readonly=on"]
    if not transport or transport == "ide-cd":
        # `ide_bus` pins the port (q35's AHCI holds one device per port).
        bus = f",bus={ide_bus}" if ide_bus else ""
        return [*drive, "-device", f"ide-cd,drive=cd0,bootindex=0{bus}"]
    if transport == "virtio-blk":
        return [*drive, "-device", "virtio-blk-pci,drive=cd0,bootindex=0"]
    if transport == "usb-storage":
        # xhci, because usb-storage on the default UHCI controller is 1.1-slow
        # and OVMF enumerates xhci fine.
        return [
            *drive,
            "-device",
            "qemu-xhci,id=xhci",
            "-device",
            "usb-storage,bus=xhci.0,drive=cd0,bootindex=0",
        ]
    raise ValueError(
        f"VIBEOS_QEMU_BOOT_TRANSPORT={transport!r} is not one of ide-cd, virtio-blk, usb-storage"
    )


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


# Real-hardware hazards for amd64-uefi (VIBEOS_QEMU_HOSTILE=all, or a comma list to
# bisect one at a time). Each strips something QEMU's default PC gives us for free
# and a 2018+ mini PC does not, so real-hardware bugs show up at smoke speed:
#   q35    ICH9 chipset: AHCI boot media, HPET, PCIe topology (not i440fx + IDE)
#   cpu    `-cpu max`: SMEP/SMAP/UMIP/x2APIC/XSAVE features TCG then ENFORCES
#   nops2  no i8042; input only through USB HID on xHCI
#   novga  no legacy VGA; GOP from ramfb only
#   mem    6 GiB, so RAM and the memory map straddle 4 GiB
#   serial no COM1: the console UART sits at an I/O port only an ACPI SPCR names,
#          as AMT serial-over-LAN, a BMC or a PCIe serial card does on a real PC
HOSTILE_HAZARDS = ("q35", "cpu", "nops2", "novga", "mem", "serial")
HOSTILE_UART = 0x5000
HOSTILE_UART_IRQ = 5


def console_uart_args(arch: str, chardev: str) -> list[str]:
    """The console UART on `chardev`: COM1, or under the `serial` hostile hazard a
    16550 at a non-legacy port that only the ACPI SPCR describes. Shared by smoke
    and the gdb launcher so a debug boot reproduces the boot it is debugging."""
    if arch == "amd64-uefi" and "serial" in hostile_hazards():
        return [
            "-device",
            f"isa-serial,chardev={chardev},iobase={HOSTILE_UART:#x},irq={HOSTILE_UART_IRQ}",
        ]
    return ["-serial", f"chardev:{chardev}"]


def make_spcr(path: Path, port: int, irq: int) -> None:
    """ACPI SPCR rev 2 for a 16550 at I/O `port`, 115200 8N1, VT100, not on PCI.
    The whole table, header included, as `-acpitable file=` expects."""
    body = bytearray(80)
    body[0:4] = b"SPCR"
    body[4:8] = (80).to_bytes(4, "little")
    body[8] = 2  # revision
    body[10:16] = b"VIBEOS"
    body[16:24] = b"HOSTILE "
    body[24:28] = (1).to_bytes(4, "little")
    body[28:32] = b"VIBE"
    body[32:36] = (1).to_bytes(4, "little")
    body[36] = 0  # interface: full 16550
    # Generic Address Structure: SystemIO, 8-bit registers, byte access.
    body[40:52] = bytes([1, 8, 0, 1]) + port.to_bytes(8, "little")
    body[52] = 0x01  # interrupt type: dual-8259
    body[53] = irq
    body[54:58] = irq.to_bytes(4, "little")  # GSIV
    body[58] = 7  # 115200 baud
    body[59] = 0  # no parity
    body[60] = 1  # one stop bit
    body[61] = 0  # no flow control
    body[62] = 0  # terminal type VT100
    body[64:66] = (0xFFFF).to_bytes(2, "little")  # PCI device: none
    body[66:68] = (0xFFFF).to_bytes(2, "little")  # PCI vendor: none
    body[9] = (-sum(body)) & 0xFF
    path.write_bytes(bytes(body))


def hostile_hazards() -> set[str]:
    raw = os.environ.get("VIBEOS_QEMU_HOSTILE", "").strip().lower()
    if not raw:
        return set()
    if raw == "all":
        return set(HOSTILE_HAZARDS)
    want = {h.strip() for h in raw.split(",") if h.strip()}
    unknown = want - set(HOSTILE_HAZARDS)
    if unknown:
        raise ValueError(
            f"VIBEOS_QEMU_HOSTILE: unknown hazard(s) {sorted(unknown)}; "
            f"choose from {', '.join(HOSTILE_HAZARDS)} or 'all'"
        )
    return want


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
        hazards = hostile_hazards()
        machine = list(self.machine_args)
        if "q35" in hazards or "nops2" in hazards:
            m = machine.index("-M") + 1
            machine[m] = ("q35" if "q35" in hazards else machine[m]) + (
                ",i8042=off" if "nops2" in hazards else ""
            )
        if "cpu" in hazards:
            # `max`, not `host`: rewrite_accel drops only the KVM passthrough model.
            machine[machine.index("-cpu") + 1] = "max"
        return [
            *machine,
            "-m",
            "6G" if "mem" in hazards else self.memory,
            "-drive",
            f"if=pflash,format=raw,file={ovmf_code},readonly=on",
            "-drive",
            f"if=pflash,format=qcow2,file={ovmf_vars}",
            # ICH9 AHCI ports hold ONE device each; the temp disk auto-takes port 0.
            *boot_media_device_args(iso, ide_bus="ide.2" if "q35" in hazards else None),
            *(["-vga", "none", "-device", "ramfb"] if "novga" in hazards else ["-vga", "std"]),
            *self._spcr_args(work_dir, hazards),
            "-display",
            "none",
            "-no-reboot",
        ]

    def _spcr_args(self, work_dir: Path, hazards: set[str]) -> list[str]:
        if "serial" not in hazards:
            return []
        spcr = work_dir / "spcr.bin"
        make_spcr(spcr, HOSTILE_UART, HOSTILE_UART_IRQ)
        return ["-acpitable", f"file={spcr}"]


def _start_echo_server(label: str = "gf") -> socketserver.ThreadingTCPServer:
    """A tiny threaded TCP echo server bound to 0.0.0.0:<ephemeral>. Backs ONE slirp
    guestfwd on the windows-host backend: the guest's outbound connection (guest ->
    virtio TX -> slirp guestfwd -> chardev -> here) gets its bytes echoed straight back,
    replacing the `-cmd:cat` line echo that can't run on the Windows QEMU host. Bound to
    the wildcard so the Windows QEMU's chardev reaches it via WSL2 localhost forwarding.
    Daemon threads — never blocks interpreter exit; caller shuts it down in stop().

    `VIBEOS_ECHO_PROBE=<path>` logs every connection and payload. That log is how the
    shared-chardev mix-up in GUESTFWD_ECHO_PORTS was finally pinned: three different
    guest probes arriving on ONE numbered connection is the whole diagnosis in three
    lines, and nothing else in the harness can see it — from the guest's side a stolen
    echo is indistinguishable from a transport that silently dropped the reply."""

    probe_path = os.environ.get("VIBEOS_ECHO_PROBE")
    seq = iter(range(1, 1 << 30))

    def log(msg: str) -> None:
        if not probe_path:
            return
        try:
            with open(probe_path, "a") as handle:
                handle.write(f"{time.time():.3f} [{label}] {msg}\n")
        except OSError:
            pass

    class _Echo(socketserver.BaseRequestHandler):
        def handle(self) -> None:
            conn = next(seq)
            log(f"conn#{conn} OPEN from {self.client_address}")
            total = 0
            while True:
                data = self.request.recv(4096)
                if not data:
                    log(f"conn#{conn} CLOSE after {total} bytes")
                    return
                total += len(data)
                log(f"conn#{conn} echo {len(data)}B {data[:48]!r}")
                self.request.sendall(data)

    srv = socketserver.ThreadingTCPServer(("0.0.0.0", 0), _Echo)
    srv.daemon_threads = True
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


# When set (by the windows-host backend, around the config's network_args() call),
# _free_tcp_port() delegates to this so hostfwd ports are picked on the Windows
# TCP stack — where QEMU.exe actually binds them. None on the native backend.
_windows_port_picker: Callable[[], int] | None = None
_selected_launch_ports: set[int] | None = None


def _unique_launch_port(picker: Callable[[], int]) -> int:
    for _ in range(32):
        port = picker()
        if _selected_launch_ports is None:
            return port
        if port not in _selected_launch_ports:
            _selected_launch_ports.add(port)
            return port
    raise RuntimeError("port allocator repeatedly returned a port already assigned to this QEMU")


TEMP_DISK_MB = 64
_TEMP_PART_START = 2048  # sectors; the conventional 1 MiB alignment


def make_temp_disk(out: Path) -> None:
    """MBR + one FAT16 partition with an empty \\TEMP, converted to qcow2.

    Built fresh per boot: it is scratch, and a disk that carried files from an
    earlier run would let a test pass on leftovers.
    """
    raw = out.with_suffix(".img")
    raw.unlink(missing_ok=True)
    with open(raw, "wb") as f:
        f.truncate(TEMP_DISK_MB * 1024 * 1024)
    # type 6 = FAT16 (>32M); bootable flag off -- nothing should ever boot this.
    subprocess.run(
        ["sfdisk", "--quiet", str(raw)],
        input=f"start={_TEMP_PART_START}, type=6\n",
        text=True,
        check=True,
        capture_output=True,
    )
    sectors = TEMP_DISK_MB * 2048 - _TEMP_PART_START
    subprocess.run(
        [
            "mkfs.fat",
            "-F",
            "16",
            "-n",
            "VIBEOSTEMP",
            "--offset",
            str(_TEMP_PART_START),
            str(raw),
            str(sectors // 2),
        ],
        check=True,
        capture_output=True,
    )
    subprocess.run(
        ["mmd", "-i", f"{raw}@@{_TEMP_PART_START * 512}", "::/TEMP"],
        check=True,
        capture_output=True,
    )
    out.unlink(missing_ok=True)
    subprocess.run(
        ["qemu-img", "convert", "-f", "raw", "-O", "qcow2", str(raw), str(out)],
        check=True,
        capture_output=True,
    )
    raw.unlink()


def temp_disk_args(arch: str, out_dir: Path) -> list[str]:
    """The \\TEMP disk policy, shared by every launcher (QemuVM, `gdb start`).

    One decision, because a launcher that reimplements it drifts: the gdb engine
    hand-mirrored the smoke command line, never learned this disk, and so could not
    reproduce the one crash that only happens with it attached.
    """
    if not arch.startswith(("i386", "amd64")):
        return []
    # Default ON: it is also the only hard disk any x86 smoke attaches, and the
    # disk path stayed broken for months without a single red because nothing
    # exercised it -- classpnp's ClassCompleteRequest spun on a never-initialized
    # lock (every boot with a disk hung), and on i386 fastfat's DriverEntry
    # returned into its caller's stack the first time a FAT volume loaded it.
    # `VIBEOS_QEMU_TEMP_DISK=0` opts out.
    if os.environ.get("VIBEOS_QEMU_TEMP_DISK", "1").strip() in ("0", "false", "no"):
        return []
    disk = out_dir / "tempdisk.qcow2"
    make_temp_disk(disk)
    return [
        "-drive",
        f"file={disk},format=qcow2,if=none,id=tempdisk",
        "-device",
        "ide-hd,drive=tempdisk",
    ]


def _acpitable_relative(args: list[str]) -> list[str]:
    """`-acpitable file=` splits its value on ':' to accept several files, so a
    staged Windows path is cut at its drive letter ("Could not open 'C'"). Name
    the file relative to the staging dir instead, which is the Windows QEMU's
    working directory."""
    out = list(args)
    for i in range(1, len(out)):
        if out[i - 1] == "-acpitable" and out[i].startswith("file="):
            out[i] = "file=" + re.split(r"[\\/]", out[i])[-1]
    return out


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
        return _unique_launch_port(_windows_port_picker)

    def native():
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            return listener.getsockname()[1]

    return _unique_launch_port(native)


# Every INDEPENDENT guest-side consumer of the outbound slirp echo gets its OWN
# virtual port. Sharing one endpoint looks harmless and is not: slirp delivers an
# echo reply to the FIRST socket matching (addr, port), so when two consumers are
# connected at once one of them silently receives the other's bytes and the loser
# just times out. `-cmd:cat` hides this on the native backend by forking a fresh
# `cat` per CONNECTION; the windows-host rewrite binds a whole forward to ONE
# chardev byte stream, where three consumers sharing :9999 decayed into a coin
# flip that reached ~90% gate failure and wedged the merge queue for a day. The
# losing test moves run to run, which is what made it read as a flake.
#
# Adding a consumer means adding a port HERE and in the guest — never reuse one.
GUESTFWD_ECHO_PORTS: dict[str, int] = {
    "netstack-outbound": 9999,  # netstack-rust engine.rs — direct-smoltcp client
    "tdi-selftest": 9998,  # netstack-rust tditest.rs — TDI roundtrip
    "usermode-winsock": 9997,  # base/applications/network/nettest/nettest.c
}


def _guestfwd_echo_opts(addr: str) -> str:
    """The `,guestfwd=...` netdev options — one per GUESTFWD_ECHO_PORTS entry."""
    return "".join(
        f",guestfwd=tcp:{addr}:{port}-cmd:cat" for port in sorted(GUESTFWD_ECHO_PORTS.values())
    )


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
    # slirp guestfwd: the guest reaches an OUTBOUND echo at this virtual address,
    # one port per consumer (GUESTFWD_ECHO_PORTS), which slirp pipes through `cat`
    # on the host (a built-in line echo — no host-listener startup race). This is
    # the path an outbound TCP client (and winsock-via-AFD) uses; the guest's
    # serial markers prove connect/send/recv.
    GUESTFWD_ADDR: str = "10.0.2.100"

    def qemu_args(self, work_dir: Path) -> list[str]:
        return [*self.prepare_args(work_dir), *self.network_args()]

    def prepare_args(self, work_dir: Path) -> list[str]:
        return super().qemu_args(work_dir)

    def network_args(self) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        self.tdi_srv_hostfwd_port = _free_tcp_port()
        self.payload_hostfwd_port = _free_tcp_port()
        return [
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
            f"-:{self.PAYLOAD_GUEST_PORT}" + _guestfwd_echo_opts(self.GUESTFWD_ADDR),
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

    def qemu_args(self, work_dir: Path) -> list[str]:
        return [*self.prepare_args(work_dir), *self.network_args()]

    def prepare_args(self, work_dir: Path) -> list[str]:
        return super().qemu_args(work_dir)

    def network_args(self) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        self.tdi_srv_hostfwd_port = _free_tcp_port()
        self.payload_hostfwd_port = _free_tcp_port()
        return [
            "-device",
            "virtio-net-pci,disable-legacy=on,netdev=net0,id=vnet0",
            "-netdev",
            f"user,id=net0,hostfwd=tcp:127.0.0.1:{self.echo_hostfwd_port}"
            f"-:{self.ECHO_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.tdi_srv_hostfwd_port}"
            f"-:{self.TDI_SRV_GUEST_PORT}"
            f",hostfwd=tcp:127.0.0.1:{self.payload_hostfwd_port}"
            f"-:{self.PAYLOAD_GUEST_PORT}" + _guestfwd_echo_opts(self.GUESTFWD_ADDR),
        ]


@dataclass
class Arm64TestKernelConfig(Arm64Config):
    """ARM64 config with /TESTKERNEL appended to freeldr.ini boot options."""

    arch: str = "arm64-test"
    boot_marker: str = "TESTKERNEL READY"
    boot_timeout: float = 180.0
    desktop_settle_seconds: float = 0.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        boot_img = testkernel_boot_img(QEMU_DIR / "boot.img", work_dir)
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
    # Modest extra settle so the netstack engine markers that DO complete on
    # arm64 (dhcp/ping) reliably land in the captured output. NOTE: the deeper
    # self-tests (dns, outbound TCP) currently do NOT complete on arm64 — the
    # netstack worker stalls after `ping gateway OK` (an arm64-specific worker
    # bug, NOT a timing/settle issue: a 150s settle did not change it). Tracked
    # as an M7 follow-up; the core data path (ARP/DHCP/ICMP/TDI) is proven.
    desktop_settle_seconds: float = 30.0

    def qemu_args(self, work_dir: Path) -> list[str]:
        return [*self.prepare_args(work_dir), *self.network_args()]

    def prepare_args(self, work_dir: Path) -> list[str]:
        return super().qemu_args(work_dir)

    def network_args(self) -> list[str]:
        self.echo_hostfwd_port = _free_tcp_port()
        self.tdi_srv_hostfwd_port = _free_tcp_port()
        self.payload_hostfwd_port = _free_tcp_port()
        return [
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
            f"-:{self.PAYLOAD_GUEST_PORT}" + _guestfwd_echo_opts(self.GUESTFWD_ADDR),
        ]


# x86 is UEFI-only after the BIOS prune. I386Config / Amd64Config remain only as
# the (non-selectable) base classes that hold shared defaults for the -uefi
# variants; their legacy BIOS -cdrom qemu_args are no longer reachable.
ARCH_CONFIGS: dict[str, Callable[[], ArchConfig]] = {
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


def launch_with_retry(
    make_vm: Callable[[], QemuVM],
    timings: dict[str, float],
    log: Callable[[str], None],
    label: str,
) -> tuple[QemuVM | None, int, Exception | None]:
    """Start a VM, retrying a host-side launch failure.

    EVERY gated boot path must launch through this. It lived in cli/smoke.py, so the
    pytest `vm` fixture -- which carries conformance, the payload suites and the net
    suite -- kept a bare `vm.start()`: 16 conformance rounds in two weeks died on
    "QEMU did not connect to serial socket within 15.0s", before a guest instruction
    ran, and were graded as LOST BOOTS beside real kernel failures.

    Returns (vm, attempts, last_exc); `last_exc` is None on success. A fresh VM
    is built per attempt because the port and serial socket path are chosen in
    QemuVM.__init__ -- retrying start() on the same instance would retry onto
    the same bad port and could never recover.
    """
    last_exc: Exception | None = None
    vm: QemuVM | None = None
    attempts = 0
    while attempts < LAUNCH_ATTEMPTS:
        attempts += 1
        vm = make_vm()
        started = time.monotonic()
        try:
            vm.start()
            timings["start_seconds"] = round(time.monotonic() - started, 3)
            timings.update(vm.launch_timings)
            if attempts > 1:
                # Loud on the way through: a retried launch is still an incident.
                timings["launch_attempts"] = attempts
                log(f"{label}: launched on attempt {attempts} (host-side flake)")
            return vm, attempts, None
        except Exception as e:
            last_exc = e
            first = str(e).splitlines()[0][:140]
            log(f"{label}: start failed (attempt {attempts}/{LAUNCH_ATTEMPTS}): {first}")
            with contextlib.suppress(Exception):
                vm.stop()
            if attempts < LAUNCH_ATTEMPTS:
                time.sleep(LAUNCH_RETRY_SECONDS)
    return vm, attempts, last_exc


# Host-side launch retries. Small on purpose: the failure mode this exists for
# resolves on the next attempt (a lost socket / port), so a third attempt buys
# almost nothing and a large number would turn a genuinely broken launch into a
# slow one. Overridable for tests.
LAUNCH_ATTEMPTS = 3
LAUNCH_RETRY_SECONDS = 2.0


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
        serial_log: Path | None = None,
    ) -> None:
        self.config = config
        # An -smp N boot is slower in wall-clock than the UP boot the timeout was
        # sized against; scale the clock, never the marker. See smp_boot_timeout().
        # Always recompute from the stashed UP value, never from the current one:
        # a caller that reuses one config across boots (a gate retry) would
        # otherwise multiply again on every VM and drift to an absurd budget.
        if not hasattr(config, "_up_boot_timeout"):
            config._up_boot_timeout = config.boot_timeout  # type: ignore[attr-defined]
        self.config.boot_timeout = smp_boot_timeout(
            config._up_boot_timeout,  # type: ignore[attr-defined]
            smp_count(),
        )
        self.work_dir = work_dir
        self.work_dir.mkdir(parents=True, exist_ok=True)
        # A fixed per-arch path lets concurrent pytest boots overwrite each
        # other's evidence. CLI subprocesses supply their own durable run root.
        serial_root = Path(os.environ.get("VIBEOS_ARTIFACT_DIR") or work_dir)
        serial_root.mkdir(parents=True, exist_ok=True)
        self.live_log_path = serial_log or serial_root / f"{config.arch}_serial.log"

        # A SECOND, durable home for hang diagnostics. Under the pytest `vm`
        # fixture work_dir is a tmp_path_factory dir, and pytest keeps only the
        # last 3 runs — so a capture written only there is deleted by unrelated
        # later runs, long before anyone reads the run dir it belonged to. That
        # is not hypothetical: the arm64 MP wedge was captured and pruned, and
        # the hunt then spent a round re-deriving "we have no PCs" from a run
        # dir holding only serial. Callers that shell out to pytest and own a
        # durable dir (vibeos-cli conformance, farm) point this at it.
        self.artifact_dir: Path | None = None
        _art = os.environ.get("VIBEOS_ARTIFACT_DIR", "").strip()
        if _art:
            try:
                self.artifact_dir = Path(_art)
                self.artifact_dir.mkdir(parents=True, exist_ok=True)
            except OSError:
                self.artifact_dir = None

        # Optional gdbstub. On windows-host QEMU.exe is the SERVER (gdb is the client),
        # so — unlike QMP/serial, where the harness listens and QEMU dials back — the
        # stub binds 0.0.0.0:<port> on the WINDOWS TCP stack and a WSL gdb connects to
        # it via gateway_ip():<port> (transport-identical to a slirp hostfwd; see
        # gdb_host/gdb_port + _build_windows_cmd). gdb_wait adds -S (halt at reset, for
        # boot-from-zero debugging). Under the windows-host TCG backend software
        # breakpoints fire reliably — the reason this path exists (they DON'T under
        # native KVM; see qemu/start.sh). gdb_port is filled in at launch.
        # VIBEOS_QEMU_GDB=1 (stub) / =wait (stub + -S) turns the stub on for boot
        # paths that have no --gdb of their own -- notably the pytest-driven
        # conformance sweep, which is the boot `rr replay --gdb` re-runs. It only
        # ever ADDS the stub; a caller that already asked for it keeps what it asked.
        _env_gdb = os.environ.get("VIBEOS_QEMU_GDB", "").strip().lower()
        self._gdb = gdb or _env_gdb in ("1", "true", "yes", "wait")
        self._gdb_wait = gdb_wait or _env_gdb == "wait"
        self.gdb_port: int | None = None

        # Launch backend: "native" (local qemu + Unix sockets + KVM/TCG — unchanged) or
        # "windows-host" (Windows QEMU.exe via interop + TCP transport + staged media — see
        # backend.py / docs/engineering/windows-host-qemu-whpx-spike.md). windows = the
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
        # Called once, with self.qmp, the moment the QMP handshake completes and
        # before anything reads serial. The one hook that lets a caller talk to
        # QEMU at a point it cannot otherwise reach: after the monitor is live,
        # but before the guest has run far enough to matter.
        #
        # It exists for record/replay, where `replay-break` must be armed before
        # the replay runs past the icount you wanted to stop at, and where
        # opening a SECOND QMP socket alongside this one is not an option:
        # replay reach is measurably sensitive to how many host-side backends are
        # attached (file-serial with no QMP reached 1773 guest lines where the
        # same trace with QMP reached 23 — see the host-shape entry in
        # docs/engineering/qemu-record-replay.md), so a second monitor would
        # perturb the very thing under observation.
        #
        # Deliberately NOT reset by _reset_clients(): it is caller policy, not
        # per-connection state, so it survives a restart and fires again on the
        # next start() — which is what a caller arming a break on every boot of
        # this VM means.
        #
        # None by default, so every existing boot path is byte-for-byte unchanged.
        self.on_qmp_ready: Callable[[QmpClient], None] | None = None
        self._proc: subprocess.Popen | None = None
        self._boot_output: str = ""
        # Where each vCPU was parked when a boot timed out; filled by
        # wait_for_boot, empty on every other outcome. See capture_cpu_state.
        self.hang_cpu_state: str = ""
        self._watchdog: threading.Timer | None = None
        self._slot_fd: int | None = None
        self._slot_index: int | None = None
        self._arch_slot_fd: int | None = None  # per-arch sub-cap lock (KVM arches)
        self._cpu_lease_fd: int | None = None  # host CPU-broker lease (TCG boots only)
        # Resolved in start() before admission, then reused for every launch attempt.
        self._win_accel: str = "tcg"
        self.launch_timings: dict[str, float] = {}
        # windows-host only: harness echo servers backing the slirp guestfwds when
        # `-cmd:cat` can't run (Windows host has no `cat`) — ONE PER GUESTFWD, never
        # shared (see GUESTFWD_ECHO_PORTS). Empty on native / non-net.
        self._echo_servers: list[socketserver.BaseServer] = []
        # Keep stderr on disk: diagnostics must survive teardown, and a full pipe
        # must never stall QEMU. The early tail survives bind-race retries.
        self._stderr_path: Path | None = None
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

            self.gdb_port = _unique_launch_port(be.windows_free_tcp_port)
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
        return tripwire_args()

    def _smp_args(self) -> list[str]:
        """VIBEOS_QEMU_SMP=<N> boots the guest with N virtual CPUs (-smp N).
        Opt-in only (SMP bring-up — docs/engineering/smp-bringup-quest.md);
        default configs stay UP. All backends exercise real concurrency
        (TCG multi-vCPU is MTTCG — real host threads)."""
        n = smp_count()
        if n is None:
            return []
        return ["-smp", str(n)]

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

    def _serial_args(self) -> list[str]:
        return console_uart_args(self.config.arch, "ser0")

    def _xhci_enabled(self) -> bool:
        if os.environ.get("VIBEOS_QEMU_XHCI", "").strip() in ("1", "true", "yes"):
            return True
        # Without an i8042 the only keyboard and mouse are USB ones.
        return self.config.arch == "amd64-uefi" and "nops2" in hostile_hazards()

    def _xhci_args(self) -> list[str]:
        """VIBEOS_QEMU_XHCI=1 attaches a qemu-xhci controller with a USB keyboard
        and mouse (usbxhci bring-up rig, docs/engineering/usbxhci-design.md).

        The controller is pinned to PCI slot 8. Left to auto-assignment it takes
        the first free slot, which shifts arm64's virtio-blk-pci boot disk off the
        `Pci(0x2,0x0)` path saved in efivars, and UEFI falls through to PXE."""
        if not self._xhci_enabled():
            return []
        # The sticks' -devices come after the controller that owns their bus;
        # only their -drives ride with the staged disk args (_usb_disk_args).
        # Two sticks, two paths: one behind a usb-hub (full speed, the hub path
        # the Pi's VIA hub uses), and one attached directly, which QEMU puts on
        # a USB3 root port at SuperSpeed because usb-storage can do SuperSpeed.
        stick = (
            [
                "-device",
                "usb-hub,bus=xhci.0,port=3,id=uhub",
                "-device",
                "usb-storage,bus=xhci.0,port=3.1,drive=usbstor0,id=ustick",
                "-device",
                "usb-storage,bus=xhci.0,drive=usbstor1,id=ustick_ss",
            ]
            if self._usb_disk_enabled()
            else []
        )
        return [
            "-device",
            "qemu-xhci,id=xhci,addr=0x8",
            "-device",
            "usb-kbd,bus=xhci.0,id=ukbd",
            "-device",
            "usb-mouse,bus=xhci.0,id=umouse",
            *stick,
        ]

    def _without_virtio_input(self, args: list[str]) -> list[str]:
        """Drop arm64's virtio keyboard/mouse when the xHCI rig is on.

        QEMU sends host input to the most recently ACTIVATED handler. usb-kbd
        activates when it is created; virtio-input activates later, when the
        guest driver starts it, and so would take every keystroke meant for USB.
        (`input-send-event`'s `device` argument cannot pick one: it names a
        display console, and QEMU aborts looking it up with -display none.)"""
        if not self._xhci_enabled():
            return args
        out: list[str] = []
        i = 0
        while i < len(args):
            if (
                args[i] == "-device"
                and i + 1 < len(args)
                and args[i + 1].startswith(("virtio-keyboard-device", "virtio-mouse-device"))
            ):
                i += 2
                continue
            out.append(args[i])
            i += 1
        return out

    def _usb_disk_enabled(self) -> bool:
        return os.environ.get("VIBEOS_QEMU_USB_DISK", "").strip() in ("1", "true", "yes")

    def _temp_disk_args(self) -> list[str]:
        """A writable FAT volume holding \\TEMP, for x86 boots that run winetests.

        The livecd's only volume is the CD, so `GetTempPath` is X:\\reactos\\TEMP --
        read-only -- and any conformance test that writes a scratch file fails on
        the harness rather than on the OS (user32:class's manifest is the measured
        case: 0xc0150010 on both sides of the WoW64 A/B). winlogon points the
        winetest children's TEMP here when it finds C:\\TEMP (winlogon.rs).

        IDE, not virtio: x86 has no virtio-blk driver, and uniata already serves
        the CD. Partitioned, not a superfloppy: FreeLDR crashes on an unpartitioned
        BIOS-visible disk (see _scratch_device_args), and partmgr expects a table.
        No bootindex, so OVMF never tries it ahead of the CD. qcow2 so savevm works.
        """
        return temp_disk_args(self.config.arch, self.scratch_path.parent)

    def _usb_disk_args(self) -> list[str]:
        """VIBEOS_QEMU_USB_DISK=1 attaches usb-storage sticks (usbstor bring-up rig):
        with VIBEOS_QEMU_XHCI=1 two on the xHCI controller, one behind a hub and one
        SuperSpeed (the devices themselves are added by _xhci_args), otherwise one
        on its own EHCI controller. The stick is a
        64M MBR disk with one FAT16 partition holding USB_STICK_FILE, so a guest
        that mounts it has something known to read back. The image lives beside
        scratch_path so the windows-host backend stages it like any other media."""
        if not self._usb_disk_enabled():
            return []
        usb_disk = self.scratch_path.parent / "usbdisk.qcow2"
        if not usb_disk.exists():
            make_usb_stick_image(usb_disk)
        drive = ["-drive", f"file={usb_disk},format=qcow2,if=none,id=usbstor0"]
        if self._xhci_enabled():
            ss_disk = self.scratch_path.parent / "usbdisk-ss.qcow2"
            if not ss_disk.exists():
                make_usb_stick_image(ss_disk)
            return [*drive, "-drive", f"file={ss_disk},format=qcow2,if=none,id=usbstor1"]
        return [
            "-device",
            "usb-ehci,id=xehci",
            *drive,
            "-device",
            "usb-storage,bus=xehci.0,drive=usbstor0",
        ]

    def _set_live_tee(self) -> None:
        """Tee to the caller's owned log, or the legacy per-arch live path."""
        # ASCII-only header: the tee file is latin-1 (byte-faithful to the serial stream).
        self.serial.set_tee(
            str(self.live_log_path),
            header=f"# vibeos smoke live serial -- {self.config.arch} -- "
            f"pid(launcher)={os.getpid()} -- started {time.ctime()}\n"
            f"# tail -f {self.live_log_path}\n",
        )

    def _arm_replay_break(self, rr, trace_dir) -> None:
        """Install the on_qmp_ready hook that stops this replay at an icount.

        A replay can only be stopped part-way by QMP `replay-break`, and only at
        an icount where the recording ends an instruction chunk — anywhere else
        aborts QEMU outright with `Playback shouldn't have to iowait`. That rule
        is measured, not assumed: predicted before the run and confirmed to the
        digit, both that a boundary pauses cleanly and that a mid-chunk target
        dies with the exact remainder computed from the trace.

        So the requested icount is SNAPPED forward to a legal boundary here,
        where the trace is in hand, and the snap is announced rather than done
        quietly — a caller who asked for a specific instruction deserves to know
        the guest will be a few instructions past it.

        Armed at QMP-ready rather than after boot because the replay is already
        running by then and may be past the target.
        """
        want = rr.env_break()
        if want is None:
            return
        target = rr.snap_break_icount(trace_dir, want)
        if target != want:
            print(
                f"[rr] break icount {want} is mid-chunk and would abort the replay; "
                f"using the next legal boundary {target} (+{target - want})",
                file=sys.stderr,
            )

        def _arm(qmp):
            qmp.execute("replay-break", icount=target)
            print(f"[rr] replay-break armed at icount {target}", file=sys.stderr)

        self.on_qmp_ready = _arm
        self._rr_break_icount = target

    def report_replay_break(self) -> str:
        """Say whether an armed `replay-break` actually fired.

        Worth a method because the alternative is inference, and the inference is
        unreliable: a replay paused at a break and a replay WEDGED present
        identically from outside — frozen serial log, QEMU busy. The obvious
        discriminator does not hold either; a stopped replay was measured sitting
        at 87% CPU in R state, so "a pause idles and a wedge spins" is false.

        QEMU does announce the stop, as a QMP STOP event. The harness's client
        used to discard events while scanning for command replies, which is why
        this could not be answered before.

        Called on the timeout path because that is where an armed break LANDS: the
        guest stops, serial goes quiet, and the harness waits out its boot timeout.
        A break that fired is the expected outcome there, not a failure.
        """
        target = getattr(self, "_rr_break_icount", None)
        if target is None:
            return ""
        if self.qmp.saw_event("STOP") is not None:
            return f"[rr] replay-break FIRED: guest stopped at icount {target}"
        return (
            f"[rr] replay-break at icount {target} did NOT fire (no QMP STOP seen). "
            "The replay stopped short of it, or wedged before reaching it — compare "
            "the trace read position against `rr breaks` to tell which."
        )

    def _apply_record_replay(self, cmd: list[str]) -> list[str]:
        """Rewrite the launch command for QEMU record/replay when asked.

        Opt-in via VIBEOS_QEMU_RR=<record|replay>:<trace-dir>, set by
        `vibeos-cli rr`. Nothing sets it by default, so every existing boot path
        is byte-for-byte unchanged when it is absent.

        The rewrite has to happen HERE rather than in the arch configs because it
        spans the whole command line: it wraps drives contributed by the arch
        config AND by the harness (scratch, usb), and it overrides accel/smp that
        the arch config chose. Doing it per-config would mean four copies that
        drift. See docs/engineering/qemu-record-replay.md.
        """
        from .cli import recordreplay as rr

        req = rr.env_request()
        if req is None:
            return cmd
        mode, trace_dir = req
        if self._backend != "native":
            raise rr.RecordReplayError(
                f"record/replay needs the native backend; this VM is on {self._backend!r}. "
                "The windows-host backend stages media into a Windows-visible directory "
                "and runs QEMU.exe, so the trace and its media would straddle two "
                "filesystems with two path namespaces."
            )
        # x86 boots KVM by default, which has no instruction counter, so rr would
        # refuse every amd64/i386 recording. Demote to TCG here rather than making
        # the operator find the knob: `rewrite_accel` is the tree's existing
        # mechanism for it and also drops the paired `-cpu host`, which is a KVM
        # host-passthrough concept that TCG cannot honour. NOT for arm64 — it
        # already runs TCG with a real `-cpu cortex-a72` model that rewrite_accel
        # would strip (same carve-out as _build_windows_cmd).
        if not self.config.arch.startswith("arm64"):
            from . import backend as be

            cmd = be.rewrite_accel(cmd, "tcg", self.config.arch)
        # stage on record (copy media into the trace); on replay the media are
        # already there, restored from pristine/ by the rr CLI verb.
        new_cmd, media = rr.transform(
            cmd,
            mode=mode,
            trace_dir=trace_dir,
            icount_shift=rr.env_shift(),
            stage=(mode == "record"),
        )
        if mode == "record":
            rr.write_manifest(
                trace_dir,
                arch=self.config.arch,
                argv=new_cmd,
                media=media,
                smp=os.environ.get("VIBEOS_QEMU_SMP", "1"),
            )
            # NB: pristine/ is frozen by the rr CLI verb once the recording
            # EXITS, not here — the vmstate snapshot rrsnapshot= writes only
            # exists after the run.
        else:
            rr.assert_same_shape(trace_dir, new_cmd)
            self._arm_replay_break(rr, trace_dir)
        # Give the boot watchdogs room. icount + blkreplay is several times
        # slower than plain TCG, and a REPLAY is slower still — it must wait out
        # each recorded I/O completion rather than take the host's. The x86
        # configs' 120s silence watchdog is tuned for TCG's pace and abuts a
        # healthy rr run: a replay of an arm64 smoke was aborted HANG_TIMEOUT
        # with `VM status: running` and the vCPU advancing through an EL0 user
        # PC — executing fine, just quiet for longer than the watchdog allows.
        # Same shape as the whpx scaling in _build_windows_cmd, and it applies
        # only when rr was explicitly asked for.
        self.config.boot_timeout = max(self.config.boot_timeout, 1800.0)
        if self.config.hang_silence_seconds:
            self.config.hang_silence_seconds = max(self.config.hang_silence_seconds, 600.0)
        # Where THIS run's serial lands, rewritten every run. The rr CLI verb
        # copies it into the trace afterwards so `rr verify` can diff a replay
        # against the recording. It must not live in the manifest: the manifest
        # is the RECORD's pin and a replay never rewrites it, so reading the
        # path from there hands a replay the record's own transcript and the
        # comparison trivially passes. (It did. The oracle reported VERIFIED on
        # a replay that had hung at a fifth of the recording's output.)
        rr.note_run(trace_dir, mode=mode, serial_log=str(self.live_log_path))
        print(f"[QemuVM] record/replay: {mode} -> {trace_dir}")
        return new_cmd

    def _prepare_staging(self) -> Path:
        """Register a PID-owned directory under the cleanup lock, then release it.

        Cleanup protects live Linux owners even before QEMU exists. Keep the
        shared lock for inspection/deletion and directory creation, never copies.
        """
        from .windows_staging import auto_cleanup_locked

        root = Path(self._win.get("staging_dir", "/mnt/c/temp/vibeos-qemu"))
        waiting = time.monotonic()
        with launch_lock():
            self.launch_timings["cleanup_lock_seconds"] = time.monotonic() - waiting
            started = time.monotonic()
            auto_cleanup_locked(root)
            staging = root / f"{os.getpid()}-{self.work_dir.parent.name}-{self.work_dir.name}"
            self._win_staging = staging
            shutil.rmtree(staging, ignore_errors=True)
            staging.mkdir(parents=True, exist_ok=True)
            self.launch_timings["cleanup_seconds"] = time.monotonic() - started
        return staging

    def _prepare_launch(self) -> None:
        """Prepare private disks and pin immutable media before port selection."""
        from . import backend as be

        windows = self._backend == "windows-host"
        staging = self._prepare_staging() if windows else self.work_dir
        if windows:
            self.scratch_path = staging / "scratch.qcow2"
            self.debugcon_path = staging / "debugcon.bin"
        subprocess.run(
            ["qemu-img", "create", "-f", "qcow2", str(self.scratch_path), "64M"],
            check=True,
            capture_output=True,
        )
        arch = self._without_virtio_input(self.config.prepare_args(self.work_dir))
        disks = [*self._scratch_device_args(), *self._usb_disk_args(), *self._temp_disk_args()]
        self._prepared_arch_args = (
            _acpitable_relative(be.stage_media_paths(arch, staging)) if windows else arch
        )
        self._prepared_disk_args = be.stage_media_paths(disks, staging) if windows else disks

    def _build_native_cmd(self) -> list[str]:
        """Attach prepared media and select ports under the handshake lock."""
        self.serial.listen(str(self.serial_path))
        self._set_live_tee()
        return [
            self.config.qemu_bin,
            "-qmp",
            f"unix:{self.qmp_path},server,nowait",
            "-chardev",
            f"socket,id=ser0,path={self.serial_path},server=off,reconnect=1",
            *self._serial_args(),
            *self._prepared_disk_args,
            *self._tripwire_args(),
            *self._smp_args(),
            *self._xhci_args(),
            *self._extra_qemu_args(),
            "-monitor",
            "none",
            *self._gdb_args(win_stack=False),
            *(
                ["-debugcon", f"file:{self.debugcon_path}"]
                if not self.config.arch.startswith("arm64")
                else []
            ),
            *self._prepared_arch_args,
            *self.config.network_args(),
        ]

    def _build_windows_cmd(self) -> list[str]:
        """windows-host backend: Windows QEMU.exe via interop, media staged to a
        Windows-visible dir, QMP + serial over TCP (we listen on 127.0.0.1; the Windows
        QEMU connects back via WSL2 localhost forwarding). See backend.py."""
        from . import backend as be

        is_arm64 = self.config.arch.startswith("arm64")
        accel = self._win_accel
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
        # _free_tcp_port inside the net configs' network_args(); scoped tightly so the
        # native path and non-net configs are untouched.
        global _windows_port_picker
        _windows_port_picker = be.windows_free_tcp_port
        try:
            network_args = self.config.network_args()
        finally:
            _windows_port_picker = None
        staged_arch_args = [*self._prepared_arch_args, *network_args]
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
        # each `-cmd:cat` guestfwd echo is redirected to its OWN chardev backed by its OWN
        # harness echo server (Windows has no `cat`). One per guestfwd is the whole point:
        # a chardev is one byte stream, so a shared one merges every consumer's echo
        # traffic — GUESTFWD_ECHO_PORTS has the full story. No-op for non-net configs.
        arch_args, chardev_ids = be.rewrite_netdev_for_windows(arch_args)
        echo_chardev: list[str] = []
        for chardev_id in chardev_ids:
            server = _start_echo_server(chardev_id)
            self._echo_servers.append(server)
            echo_port = server.server_address[1]
            echo_chardev += [
                "-chardev",
                f"socket,id={chardev_id},host={host},port={echo_port},server=off",
            ]
        return [
            win_bin,
            "-name",
            self._win_uid,
            # Listeners are ready before launch. Connect synchronously: reconnect
            # starts asynchronous QIO tasks even on the first connection, while
            # these single-session clients cannot recover a disconnected stream.
            "-chardev",
            f"socket,id=qmp0,host={host},port={qmp_port},server=off",
            "-mon",
            "chardev=qmp0,mode=control",
            "-chardev",
            f"socket,id=ser0,host={host},port={ser_port},server=off",
            *self._serial_args(),
            *self._prepared_disk_args,
            *self._tripwire_args(),
            *self._smp_args(),
            *self._xhci_args(),
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
        global _selected_launch_ports
        for attempt in range(attempts):
            _selected_launch_ports = set()
            try:
                if self._backend == "windows-host":
                    cmd = self._build_windows_cmd()
                else:
                    cmd = self._build_native_cmd()
                cmd = self._apply_record_replay(cmd)
            finally:
                _selected_launch_ports = None
            print(f"[QemuVM] {' '.join(cmd)}")
            self._stderr_path = self.live_log_path.with_name(f"{self.config.arch}_qemu.stderr.log")
            with self._stderr_path.open("wb") as stderr_log:
                proc = subprocess.Popen(
                    cmd,
                    stdout=subprocess.DEVNULL,
                    stderr=stderr_log,
                    preexec_fn=os.setpgrp,
                    # The Windows QEMU runs in its staging dir: `-acpitable` names its
                    # file relative to it (see _acpitable_relative). Every other path
                    # in the command is absolute, so nothing else depends on this.
                    cwd=getattr(self, "_win_staging", None)
                    if self._backend == "windows-host"
                    else None,
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
        """Poll for a launch error without consuming its durable stderr log."""
        deadline = time.monotonic() + budget
        while time.monotonic() < deadline:
            if proc.poll() is not None:
                return self._read_qemu_stderr()
            time.sleep(0.05)
        return None

    def _discard_failed_launch(self, proc: subprocess.Popen) -> None:
        """Tear down a hostfwd-raced launch attempt: reap the dead qemu, stop the echo
        server, and replace the serial/qmp clients with fresh instances — SerialClient
        can't be reused after close() (its drainer won't restart once _closed is set).
        The next _build_windows_cmd re-binds listeners and reuses untouched media."""
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
        for server in self._echo_servers:
            try:
                server.shutdown()
                server.server_close()
            except Exception:
                pass
        self._echo_servers = []
        self.serial.close()
        self.qmp.close()
        self.qmp = QmpClient()
        self.serial = SerialClient()

    def _resolve_windows_accel(self) -> None:
        """Resolve before admission so slot and CPU leases match the actual launch."""
        if self._backend != "windows-host":
            return
        from . import backend as be

        configured = self._win.get("accel", "tcg")
        probe = (
            be.whpx_available()
            if configured == "whpx" and not self.config.arch.startswith("arm64")
            else None
        )
        self._win_accel = be.resolve_accel(self.config.arch, configured, probe)

    def start(self) -> None:
        """Start QEMU process, connect QMP + serial (backend-aware)."""
        # Acquire a global concurrency slot BEFORE launching — and before we
        # record the start/deadline below, so the time spent queued here is
        # excluded from the 30-min lifetime cap. Held for this pytest process's
        # lifetime; released in stop()/atexit. Acquired once, ahead of any
        # hostfwd-bind relaunch attempts.
        started = time.monotonic()
        self._resolve_windows_accel()
        self.launch_timings = {"resolve_seconds": round(time.monotonic() - started, 3)}
        admitted = time.monotonic()
        self._acquire_slot()

        # Host CPU broker: a TCG boot is CPU-bound emulation, so take one lease from the
        # writer-takes-all pool BEFORE launch — a concurrent cold build grabs the whole pool
        # (exclusive), so this blocks (QEMU not yet spawned → no wasted CPU) until the build
        # frees it, instead of starving into a false HANG_TIMEOUT. Like the slot, the wait
        # happens before `started` is recorded so it's excluded from the lifetime cap.
        self._acquire_cpu_lease()
        self.launch_timings["admission_seconds"] = round(time.monotonic() - admitted, 3)

        # Media preparation can overlap other launches. PID-owned staging and
        # the media cache's own lock protect it from cleanup and concurrent fills.
        try:
            preparing = time.monotonic()
            self._prepare_launch()
            self.launch_timings["media_prepare_seconds"] = round(time.monotonic() - preparing, 3)
        except BaseException:
            self.stop()
            raise

        # Coordinate selection through the successful QMP handshake. Merely
        # surviving the early-exit poll does not prove hostfwd/GDB binds worked.
        # All builders enter, protecting the scoped Windows port-picker too.
        waiting = time.monotonic()
        try:
            with launch_lock():
                self.launch_timings["launch_lock_seconds"] = round(time.monotonic() - waiting, 3)
                try:
                    self._start_with_ports()
                except BaseException:
                    # Stop before releasing the port handshake on launch failure.
                    self.stop()
                    raise
        except BaseException:
            # A lock timeout also releases the boot/slot leases acquired above.
            self.stop()
            raise

    def _start_with_ports(self) -> None:
        preparing = time.monotonic()
        cmd = self._launch_qemu()
        proc = self._proc
        if proc is None:
            raise RuntimeError("QEMU launch returned without a process")
        self.launch_timings["launch_seconds"] = round(time.monotonic() - preparing, 3)
        self.launch_timings["prepare_seconds"] = round(
            self.launch_timings.get("media_prepare_seconds", 0)
            + self.launch_timings["launch_seconds"],
            3,
        )
        self._record_launch(cmd)
        connecting = time.monotonic()
        try:
            self._connect_launch(cmd, proc)
        finally:
            self.launch_timings["handshake_seconds"] = round(time.monotonic() - connecting, 3)
            self.launch_timings["launch_lock_held_seconds"] = round(time.monotonic() - preparing, 3)
            self._record_launch(cmd)

    def _record_launch(self, cmd: list[str]) -> None:
        (self.live_log_path.parent / f"{self.config.arch}_launch.json").write_text(
            json.dumps(
                {
                    "arch": self.config.arch,
                    "backend": self._backend,
                    "accel": self._win_accel
                    if self._backend == "windows-host"
                    else ("tcg" if self._boot_is_tcg() else "kvm"),
                    "argv": cmd,
                    "timings": self.launch_timings,
                },
                indent=2,
            )
            + "\n"
        )

    def _connect_launch(self, cmd: list[str], proc: subprocess.Popen) -> None:
        # Drop a scoped lock so the reaper can safely kill THIS qemu iff our
        # launcher (this pytest process) dies. reap_orphan_qemus() is auto-
        # wired in conftest.py at session start. qemu/reap.sh covers both
        # this lock dir and the kernel-debug-toolkit's manifest.json files.
        started = time.time()
        try:
            LOCK_DIR.mkdir(parents=True, exist_ok=True)
            self._lock_path = LOCK_DIR / f"{os.getpid()}-{proc.pid}.json"
            # Written to a temp name and RENAMED into place: write_text truncates
            # and then writes, and reap_orphan_qemus() -- run from any session --
            # deletes a lock it cannot parse. A reap landing in that window would
            # unregister a LIVE VM, leaving it invisible to `vibeos-cli ps`/`kill`
            # and outside the 30-min cap. rename is atomic on one filesystem.
            tmp_lock = self._lock_path.with_suffix(".json.tmp")
            tmp_lock.write_text(
                json.dumps(
                    {
                        "launcher_pid": os.getpid(),
                        "qemu_pid": proc.pid,
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
            os.replace(tmp_lock, self._lock_path)
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

        if self.on_qmp_ready is not None:
            # Deliberately NOT wrapped in try/except. A hook that swallowed its
            # own failure would leave the caller believing a `replay-break` was
            # armed when it was not, and the run would then look like "the break
            # never fired" — indistinguishable from a break landing at an illegal
            # icount, which is a real and separate failure we spent a night
            # learning to tell apart. Fail loud instead.
            self.on_qmp_ready(self.qmp)

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
        except ConnectionError as exc:
            self._boot_output = self.serial.partial_output
            # WSL's process relay may report the exit slightly after socket EOF.
            if self._proc:
                try:
                    self._proc.wait(timeout=0.25)
                except subprocess.TimeoutExpired:
                    pass
                code = self._proc.poll()
                state = f"exited with code {code}" if code is not None else "still running"
                self._raise_start_error(f"{exc}; QEMU {state}")
            raise
        except TimeoutError:
            # Save whatever serial output was captured before the timeout
            self._boot_output = self.serial.partial_output
            note = self.report_replay_break()
            if note:
                print(note, file=sys.stderr)
            # ...and, while the guest is still alive, where each vCPU is parked.
            # Done HERE rather than at the call sites because every consumer of a
            # hung boot reaches it through this one except: smoke's _one_boot, the
            # pytest `vm` fixture that conformance shells out to, and farm. Wiring
            # them individually would have left the conformance sweep — the very
            # place the arm64 --smp 1 stall shows up — with no capture at all.
            #
            # This covers a boot that never reaches its marker. It does NOT cover
            # a guest that boots fine and then wedges while a test drains serial
            # (the kmtest sweep) — that path never enters this except, so it must
            # call save_hang_state() itself.
            self.save_hang_state("boot-wait timeout")
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
                # Settle time elapsed -- that's fine, but what the guest printed during
                # it is still boot output. A chatty payload never lets serial go idle,
                # so this is the NORMAL path for one, and dropping the window here left
                # those bytes unconsumed for restore_snapshot's flush to throw away.
                self._absorb_pending()

        return self._boot_output

    def _absorb_pending(self) -> None:
        """Move every drained-but-unconsumed serial byte into `_boot_output`.

        WHY (2026-09-26): `test_payload_launched` waited out whole VM lifetimes for a
        marker that was sitting in its own serial log -- ucrtdiff's `[UCRTDIFF] done`,
        oldcrt's `[OLDCRT] done PASS`. A fast payload finishes inside the settle window;
        the settle timed out (the payload was talking) and kept none of it; the `ready`
        snapshot was then taken with the payload already done; and the first test's
        restore_snapshot flushed the unread bytes -- marker included -- and rewound the
        guest to a point after the payload had exited, so nothing ever printed it again.
        Output from before the snapshot belongs to the boot record, never to a flush."""
        self._boot_output += self.serial.read_available(timeout=0)

    def save_snapshot(self, name: str = "ready") -> None:
        """Save VM state via QMP/HMP savevm."""
        # Everything drained so far was printed BEFORE the snapshot: keep it (see
        # _absorb_pending). restore_snapshot flushes what is left, and a restored guest
        # never prints pre-snapshot output again.
        self._absorb_pending()
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
        output = self.serial.read_available(timeout=timeout)
        if not output and self._proc is not None and self._proc.poll() is not None:
            raise ConnectionError(f"QEMU exited with code {self._proc.returncode}")
        return output

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

    def capture_cpu_state(self) -> str:
        """Where every vCPU is parked, straight from the QEMU monitor.

        The only witness to a hang that prints nothing. A guest spinning in a
        kernel spinlock — or wedged with its clock dead — does not block QEMU's
        main loop, so the monitor still answers while the guest is stuck; that
        asymmetry is what makes this work at all. Until this existed, both
        backends ran with `-monitor none` and a wedged boot was killed without
        ever being asked where it stopped, so every silent hang had to be
        triaged from the serial tail alone.

        `info cpus` no longer carries a PC (QEMU 5.0 replaced query-cpus with
        query-cpus-fast, which dropped the field), so the PC comes from
        `info registers -a` — the `-a` is what makes it cover every vCPU
        instead of just the monitor's currently selected one.

        Never raises: a diagnostic that can itself fail the run is worse than
        no diagnostic, and this one runs on a path that is already failing.
        """
        chunks = []
        for cmd in ("info status", "info cpus", "info registers -a"):
            try:
                body = self.qmp.hmp(cmd).strip()
            except Exception as exc:  # QMP dead//timed out — say so, don't raise
                body = f"<unavailable: {exc!r}>"
            chunks.append(f"=== {cmd} ===\n{body}")
        # The instructions AT each PC. A register dump says where a vCPU is;
        # only the code there says what it is doing — a `wfi` is idle, a `b .`
        # is a deliberate park, and an unmapped or user page under a kernel PC
        # is a fault loop. `x /6i` reads through the guest's current
        # translation, so a PC the CPU cannot fetch comes back as an error
        # line, which is itself the answer.
        try:
            from .cli.triage import cpu_pcs

            pcs = cpu_pcs("\n".join(chunks))
        except Exception:
            pcs = []
        for idx, pc in pcs:
            cmd = f"x /6i 0x{pc}"
            try:
                body = self.qmp.hmp(cmd).strip()
            except Exception as exc:
                body = f"<unavailable: {exc!r}>"
            chunks.append(f"=== CPU#{idx} {cmd} ===\n{body}")
        # The memory under each stack pointer, read through the guest's OWN
        # translation. This is the decisive test for a wedge parked on a stack
        # access: if the monitor cannot read it, the page is GONE and the guest
        # is re-faulting on it forever; if the words come back, the page is
        # mapped and the fault is not a translation fault, so look at
        # permissions or at something other than paging. That fork cannot be
        # answered after teardown, and no in-guest probe can answer it at all —
        # the guest is the thing that is stuck.
        # 0xB8 bytes covers an arm64 KTRAP_FRAME through TfX0 (trap.S: TfSpsr
        # 0xA0, TfEsr 0xA4, TfSp 0xA8, TfX0 0xB0), so a frame-restore wedge
        # shows the exact words the restore sequence was reading.
        try:
            from .cli.triage import cpu_sps

            sps = cpu_sps("\n".join(chunks))
        except Exception:
            sps = []
        for idx, sp in sps:
            cmd = f"x /23gx 0x{sp}"
            try:
                body = self.qmp.hmp(cmd).strip()
            except Exception as exc:
                body = f"<unavailable: {exc!r}>"
            chunks.append(f"=== CPU#{idx} stack {cmd} ===\n{body}")
        return "\n\n".join(chunks) + "\n"

    def save_hang_state(self, phase: str) -> str:
        """Capture where every vCPU is parked and persist it where it survives.

        Call this from ANY path that has decided the guest is wedged, while the
        guest is still alive — the capture is only possible before teardown, and
        a wedge that prints nothing leaves no other evidence at all. `phase`
        names what gave up (boot wait, sweep drain, ...) so a bare file in a run
        dir still says which stall it belongs to.

        Writes to work_dir AND artifact_dir: see the artifact_dir comment in
        __init__ for why work_dir alone loses the file. Never raises — a
        diagnostic that can fail its own already-failing run is worse than none.
        """
        ok = True
        try:
            self.hang_cpu_state = self.capture_cpu_state()
        except Exception as exc:
            self.hang_cpu_state = f"<capture failed: {exc!r}>\n"
            ok = False
        # A SECOND sample, seconds later. One sample cannot tell a box that is
        # frozen (nothing runs at all — dead clock, no scheduling) from one that
        # is merely blocked (CPUs cycling through the idle loop, work waiting on
        # something that never arrives). Both park every vCPU in KiIdleLoop and
        # read identically in a single dump — and they are completely different
        # bugs, so the fork has to be settled here, while the guest is alive.
        # A drained-out sweep has already spent its budget; a few more seconds to
        # make the artifact self-answering is free by comparison.
        second = ""
        try:
            time.sleep(HANG_RESAMPLE_SECONDS)
            second = self.capture_cpu_state()
        except Exception as exc:
            second = f"<resample failed: {exc!r}>\n"
            ok = False
        if not ok:
            # Two failed samples parse to the same (empty) PC list, and calling
            # that "FROZEN" would be the instrument inventing its own evidence.
            verdict = "UNKNOWN — the monitor did not answer; there is no sample to compare"
        else:
            # triage owns the `info registers -a` parse; import it here rather
            # than at module scope because vm.py is imported by the cli package.
            try:
                from .cli.triage import cpu_pcs

                moved = cpu_pcs(second) != cpu_pcs(self.hang_cpu_state)
            except Exception:
                moved = second.strip() != self.hang_cpu_state.strip()
            verdict = (
                "EXECUTING — vCPU PCs moved between samples, so the box is running "
                "and blocked on something specific, not frozen"
                if moved
                else "FROZEN — every vCPU PC is identical across both samples; "
                "nothing was scheduled in between"
            )
            # A same-PC pair has a THIRD reading the two-sample test cannot
            # see: a tight fault loop — abort, handler, eret to the same PC,
            # repeat — never moves the PC either, yet nothing is frozen. On
            # arm64 the register dump carries the tell (kernel EL, user PC),
            # so name it here rather than let the file say FROZEN and send the
            # reader after a dead clock. The 2026-09-05 wedge sat behind that
            # word for five captures in a row.
            try:
                from .cli.triage import cpu_kernel_at_user_pc

                looped = cpu_kernel_at_user_pc(second) or cpu_kernel_at_user_pc(self.hang_cpu_state)
            except Exception:
                looped = []
            if looped:
                where = ", ".join(f"CPU#{i} pc=0x{pc}" for i, pc in looped)
                verdict = (
                    "FAULT-LOOP — a vCPU is at kernel EL with a USER-range PC "
                    f"({where}); the kernel cannot execute there, so it is "
                    "retrying the same instruction abort forever. Not a dead "
                    "clock, not blocked. Read the trap handler's retry decision "
                    "for this abort, not the timers.  [" + verdict + "]"
                )
        smp = smp_count()
        body = (
            f"=== hang capture: {phase} ===\n"
            f"arch={self.config.arch} smp={smp or 1} "
            f"boot_timeout={self.config.boot_timeout:.0f}s\n"
            f"verdict: {verdict}\n"
            f"(samples {HANG_RESAMPLE_SECONDS}s apart)\n\n"
            + self.hang_cpu_state
            + f"\n=== resample (+{HANG_RESAMPLE_SECONDS}s) ===\n"
            + second
        )
        for d in (self.work_dir, self.artifact_dir):
            if d is None:
                continue
            try:
                # mkdir here rather than trusting __init__: this runs minutes or
                # hours after launch, and a swallowed OSError from a dir that
                # went missing looks exactly like "the guest never wedged".
                d.mkdir(parents=True, exist_ok=True)
                (d / "hang-cpustate.txt").write_text(body)
            except OSError:
                pass
        return self.hang_cpu_state

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
        arch = self.config.arch.split("-", 1)[0]
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
        - windows-host: TCG unless WHPX resolved on (x86 only). start() resolves
          `_win_accel` before acquiring slots or CPU leases."""
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

        # Tear down the guestfwd echo servers (windows-host net config only).
        for server in self._echo_servers:
            server.shutdown()
            server.server_close()
        self._echo_servers = []

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

    def _read_qemu_stderr(self) -> str:
        """Read a bounded tail without waiting for a live QEMU to close a pipe."""
        path = getattr(self, "_stderr_path", None)
        if path is not None and path.exists():
            with path.open("rb") as stream:
                stream.seek(max(0, path.stat().st_size - 65536))
                return stream.read(65536).decode("utf-8", errors="replace")
        return getattr(self, "_early_stderr", "")

    def _raise_start_error(self, msg: str) -> None:
        """Raise with the QEMU stderr tail, including when QEMU is still alive."""
        stderr = self._read_qemu_stderr() or self._early_stderr
        raise RuntimeError(f"{msg}\nQEMU stderr:\n{stderr}")
