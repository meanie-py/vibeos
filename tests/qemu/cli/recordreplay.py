"""QEMU record/replay ("rr") for vibeos boots — deterministic re-execution.

WHY THIS EXISTS
---------------
Our hardest bugs are heisenbugs: an arm64 MP boot faults at PC=0 roughly 1 in 60
boots, and every attempt to look at it changes it.  Soaking found the RATE and
nothing else in months.  QEMU's record/replay makes one *particular* boot — the
one that crashed — into an artifact you can run again, identically, as many
times as you like.  That turns "reproduce it again and hope" into "open the
recording".

Caveat worth knowing before you plan around this, CORRECTED: a replay CAN be
stopped, but only on an instruction-chunk boundary.  `rr replay --break-at N`
arms a QMP `replay-break` and the guest pauses; `rr breaks` prints the legal
targets.  What aborts QEMU is a stop PART WAY through a chunk, which is what
every gdb stop was -- stepi, a mid-run attach and a breakpoint hit all land
wherever the guest happens to be, so "the gdbstub is unusable" was the symptom
of the chunk rule rather than a separate defect.  gdb stepping is still
unavailable for that reason and reverse execution with it; a trace plus
`--break-at` is a capture you can stop at a chosen instruction, not a debugging
session.  What does still block replaying a whole boot is the WEDGE.  See
docs/engineering/qemu-record-replay.md.

The tool is the deliverable.  Catching any specific bug with it is a bonus.

HOW IT WORKS
------------
`-icount shift=N,rr=record,rrfile=T` makes the vCPU advance on a virtual clock
derived from instruction count, and logs every non-deterministic input (device
reads, interrupts, clock reads) to the trace file T.  `rr=replay` then feeds
those same inputs back in the same instruction-slots, so the guest re-executes
bit-for-bit.

Two things have to be true for that to hold, and both need us to REWRITE the
command line rather than just append flags:

1. Block I/O must be deterministic.  A real disk completes asynchronously, so
   the completion lands in a different instruction slot each run.  The
   `blkreplay` block filter parks completions and releases them at the recorded
   virtual time.  Every guest-visible drive must be wrapped.

2. The guest must start from byte-identical media.  Recording MUTATES media
   (efivars, the scratch disk).  So a trace owns private copies: mutable images
   live as qcow2 overlays inside the trace dir, with a pristine copy kept beside
   them that `prepare_replay()` restores before each replay.  A trace is thus
   self-contained and replayable any number of times.

`rrsnapshot=init` additionally writes a vmstate snapshot into the first
snapshot-capable image at record start and loads it at replay start, which pins
device state as well as disk content.

CONSTRAINTS (we FAIL rather than silently produce a non-deterministic trace)
---------------------------------------------------------------------------
* TCG only, single-threaded.  MTTCG has no global instruction order to record,
  and KVM has no instruction counter at all.
* One vCPU.  This is not as limiting as it sounds for us: `--smp 1` on the MP
  kernel is precisely the UP:=smp1 collapse experiment, and the arm64 PC=0
  fault reproduces there.
* No EXPLICIT network.  A caller's own -netdev/-nic/-net is refused. The machine's
  implicit default NIC is made explicit and recorded through filter-replay (see
  transform) — it used to be missed, and its unrecorded packets were the arm64 replay
  divergence.
* Native backend only.  The windows-host backend stages media into a Windows
  path and runs QEMU.exe; the trace dir would be half on each side.

See docs/engineering/qemu-record-replay.md.
"""

from __future__ import annotations

import bisect
import json
import os
import shutil
import struct
import subprocess
import time
from collections.abc import Iterator
from pathlib import Path
from typing import NamedTuple

# Default icount shift.  `shift=N` costs the virtual clock 2**N ns per
# instruction, which is the same as fixing the guest's apparent CPU at 2**-N
# GHz: shift=3 is a 125 MHz machine, shift=7 a 7.8 MHz one.
#
# MEASURED, not guessed.  shift=7 does not boot arm64: the guest's clock then
# advances ~6x faster than the host's, so the UEFI 5-minute watchdog expires
# while freeldr is still loading the ramdisk, and the boot loops
# (`Watchdog Timer resetting system`, five times, never reaching the kernel).
# shift=3 boots to the kernel and through the kmtest sweep.  The cost of going
# lower is real seconds: a firmware delay loop is denominated in virtual ns, so
# halving the shift doubles the instructions — and the wall clock — it burns.
#
# `auto` is NOT usable here at all: it calibrates against the host, so record
# and replay would run different machines.
DEFAULT_ICOUNT_SHIFT = 3

# Drive interfaces that are memory-mapped firmware, not block-layer devices the
# guest issues async requests to.  pflash reads are synchronous MemoryRegion
# accesses, so they are already deterministic given identical content — wrapping
# them in blkreplay is both unnecessary and rejected by QEMU (blkreplay is not a
# valid pflash backend).  We still copy the image into the trace so replay sees
# the same bytes.
_FIRMWARE_IFS = {"pflash"}

# Device models whose backend blkreplay cannot drive (see the refusal in
# transform() for the measurement). ATAPI only so far; virtio-blk is fine.
_ATAPI_MODELS = {"ide-cd", "scsi-cd"}


class RecordReplayError(RuntimeError):
    """A configuration that cannot produce a deterministic trace.

    Always raised BEFORE launching QEMU.  The alternative — launching anyway and
    getting a trace that diverges on replay — costs a full boot to discover and
    looks like a tool bug rather than a config error.
    """


# --------------------------------------------------------------------------
# drive-spec handling
# --------------------------------------------------------------------------


def parse_drive(spec: str) -> dict[str, str]:
    """Split a `-drive` argument into its key=value map.

    Bare tokens (QEMU allows a leading filename) become `file=`.  QEMU escapes a
    literal comma as `,,`; none of our media paths contain one, and a path that
    did would already be broken elsewhere in the harness, so we do not implement
    the escape — we would rather mis-split loudly than silently.
    """
    out: dict[str, str] = {}
    for tok in spec.split(","):
        if not tok:
            continue
        if "=" in tok:
            k, v = tok.split("=", 1)
            out[k] = v
        else:
            out["file"] = tok
    return out


def format_drive(d: dict[str, str]) -> str:
    return ",".join(f"{k}={v}" for k, v in d.items())


def _drive_id(d: dict[str, str], n: int) -> str:
    """The id a device's `drive=` reference points at.

    A drive declared `if=virtio` / `if=ide` has no explicit id: QEMU creates the
    device itself and nothing refers to it by name, so we are free to invent one
    (we need one, because blkreplay has to name its backing image).
    """
    return d.get("id") or f"rrdrive{n}"


# --------------------------------------------------------------------------
# media staging
# --------------------------------------------------------------------------


def _sha256(path: Path) -> str:
    import hashlib

    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def stage_image(src: Path, dst: Path, src_format: str) -> None:
    """Put a replayable copy of `src` at `dst` as qcow2.

    A raw image becomes a qcow2 OVERLAY backed by the original: no copy of a
    600MB ISO, and the original is never written (the overlay absorbs guest
    writes).  A qcow2 source is copied outright, because a qcow2 backing file
    would make the trace depend on a file the harness rewrites per run.

    qcow2 is not cosmetic: `rrsnapshot=` needs an image that can hold a vmstate
    snapshot, and raw cannot.
    """
    dst.parent.mkdir(parents=True, exist_ok=True)
    if dst.exists():
        dst.unlink()
    if src_format == "qcow2":
        shutil.copy2(src, dst)
        return
    subprocess.run(
        [
            "qemu-img",
            "create",
            "-f",
            "qcow2",
            "-F",
            src_format,
            "-b",
            str(src.resolve()),
            str(dst),
        ],
        check=True,
        capture_output=True,
    )


# --------------------------------------------------------------------------
# the command-line transform
# --------------------------------------------------------------------------


def transform(
    argv: list[str],
    *,
    mode: str,
    trace_dir: Path,
    icount_shift: int = DEFAULT_ICOUNT_SHIFT,
    stage: bool = True,
) -> tuple[list[str], dict]:
    """Rewrite a normal vibeos QEMU command line into a record or replay one.

    Returns `(argv, media)` where `media` maps each trace-relative image name to
    a record of the source it was staged from — path, format, whether it was
    copied outright or kept as an overlay, and (for an overlay) the source
    digest. The manifest keeps it so a replay can say *which* media it is
    replaying and notice when the tree has moved on under it.

    `stage=False` skips the qemu-img work (the images are already in the trace
    dir); replay uses it, and so do the unit tests.
    """
    if mode not in ("record", "replay"):
        raise RecordReplayError(f"mode must be 'record' or 'replay', got {mode!r}")

    trace_dir = Path(trace_dir)
    # drive ids referenced by an `ide-cd` device, collected as we walk so the
    # ATAPI check below can name the actual drive rather than guess.
    wrapped_by_device: set[str] = set()
    # name -> {source, format, copied, sha256}; see the write site for why the
    # digest is part of it.
    media: dict[str, dict[str, object]] = {}
    out: list[str] = []
    drive_n = 0

    i = 0
    while i < len(argv):
        arg = argv[i]

        if arg == "-accel":
            accel = argv[i + 1]
            if accel.startswith("kvm") or accel.startswith("whpx"):
                raise RecordReplayError(
                    f"record/replay needs TCG (instruction counting); got -accel {accel}. "
                    "Hardware virtualization has no instruction counter to anchor the "
                    "trace to. The vm.py hook demotes x86 to TCG before it gets here "
                    "(backend.rewrite_accel), so seeing this means a hand-built command "
                    "line — swap the accelerator yourself, and drop any paired "
                    "`-cpu host` with it."
                )
            # MTTCG interleaves vCPU threads with no recorded global order.  We
            # already refuse -smp >1, but thread=multi must go too: it is the
            # arm64 default and QEMU would otherwise reject the icount pairing.
            out += ["-accel", "tcg,thread=single"]
            i += 2
            continue

        if arg == "-smp":
            n = argv[i + 1].split(",")[0]
            if n not in ("1",):
                raise RecordReplayError(
                    f"record/replay is single-vCPU only; got -smp {n}. Use --smp 1 "
                    "(which on the MP kernel is the UP:=smp1 collapse case we care "
                    "about anyway)."
                )
            out += ["-smp", "1"]
            i += 2
            continue

        if arg in ("-netdev", "-nic", "-net"):
            raise RecordReplayError(
                f"record/replay cannot record network input without replay net filters "
                f"(got {arg} {argv[i + 1] if i + 1 < len(argv) else ''}). Record a boot "
                "config with no NIC."
            )

        if arg == "-icount":
            raise RecordReplayError(
                "command line already carries -icount; refusing to record on top of "
                "someone else's instruction-counting config"
            )

        if arg == "-drive":
            d = parse_drive(argv[i + 1])
            iface = d.get("if", "none")
            did = _drive_id(d, drive_n)
            drive_n += 1
            src = Path(d["file"])
            src_fmt = d.get("format", "raw")
            # Name the trace copy after the SOURCE, not after the drive id: an
            # `if=virtio` drive has no id of its own, so id-naming buys you a
            # media dir of rrdrive1/2/3 and no way to tell firmware from the
            # ramdisk six months later. Collisions get the id appended.
            stem = src.stem or did
            name = f"{stem}.qcow2"
            if name in media:
                name = f"{stem}-{did}.qcow2"
            dst = trace_dir / "media" / name
            if stage:
                stage_image(src, dst, src_fmt)
            # Record WHAT the source was, not just where: a raw source is kept
            # as a qcow2 OVERLAY (a 290MB ramdisk is not worth copying per
            # trace), so the trace depends on that file still holding the same
            # bytes — and `qemu/boot.img` is rewritten by every build. Without
            # the digest the trace rots silently and the replay just diverges.
            media[name] = {
                "source": str(src),
                "format": src_fmt,
                "copied": src_fmt == "qcow2",
                "sha256": _sha256(src) if (stage and src_fmt != "qcow2") else None,
            }

            if iface in _FIRMWARE_IFS:
                # Keep the pflash shape; only the backing file moves into the trace.
                fw = dict(d)
                fw["file"] = str(dst)
                fw["format"] = "qcow2"
                fw.pop("readonly", None)
                out += ["-drive", format_drive(fw)]
                i += 2
                continue

            # Guest block device: image backend + blkreplay filter in front of it.
            # The filter keeps the ORIGINAL id so every `-device ...,drive=<id>`
            # elsewhere on the command line keeps resolving, untouched.
            img = {
                "file": str(dst),
                "format": "qcow2",
                "if": "none",
                "id": f"{did}-img",
            }
            rr = {
                "driver": "blkreplay",
                "if": iface,
                "image": f"{did}-img",
                "id": did,
            }
            out += ["-drive", format_drive(img), "-drive", format_drive(rr)]
            i += 2
            continue

        if arg == "-device":
            spec = argv[i + 1] if i + 1 < len(argv) else ""
            model = spec.split(",", 1)[0]
            if model in _ATAPI_MODELS:
                for tok in spec.split(","):
                    if tok.startswith("drive="):
                        wrapped_by_device.add(tok.split("=", 1)[1])

        out.append(arg)
        i += 1

    # blkreplay cannot drive an ATAPI CD on QEMU 9.0.2. Measured by wrapping one
    # drive at a time on an amd64 boot: scratch behind virtio-blk-pci wrapped
    # gives 2050 serial lines in 100s (unwrapped: 1975), while cd0 behind ide-cd
    # wrapped gives ONE — the guest never leaves the OVMF screen clear. Neither
    # icount alone nor rrsnapshot is implicated; both are fine on their own.
    #
    # Refusing is the whole point. Leaving that one drive unwrapped would record
    # a trace whose boot media replays non-deterministically, which is the silent
    # failure every other refusal in this function exists to prevent — and here it
    # would be the BOOT media, so nothing downstream could be trusted.
    atapi = sorted(wrapped_by_device)
    if atapi:
        raise RecordReplayError(
            f"cannot record: {', '.join(atapi)} sits behind an ATAPI CD device "
            "(`ide-cd`), and QEMU 9.0.2's blkreplay filter stalls that path — the "
            "guest never leaves firmware. This is why amd64/i386 cannot be recorded "
            "yet: their boot media is an ISO on ide-cd. arm64 is unaffected (virtio "
            "throughout). See docs/engineering/qemu-record-replay.md."
        )

    if drive_n == 0:
        raise RecordReplayError(
            "no -drive on the command line — rrsnapshot needs a snapshot-capable "
            "image to pin device state into"
        )

    rrfile = trace_dir / "replay.bin"
    # NO `sleep=off`, in either mode. It is the obvious fix for how slow a replay
    # is, it would not have helped (the replaying QEMU is at 97% CPU — executing,
    # not idling), and it does not work on QEMU 9.0.2 regardless: recording with
    # it never leaves the UEFI firmware, and replaying with it aborts loading the
    # rrsnapshot —
    #   qemu-system-aarch64: migration/vmstate.c:140: vmstate_load_state:
    #   Assertion `first_elem || !n_elems || !size' failed.
    # A/B'd on one trace with nothing else changed. See the divergence log.
    x86 = is_x86_argv(argv)
    # Every arch's RTC ticks on QEMU_CLOCK_HOST by default, and every read of that
    # clock is a logged replay event. x86's MC146818 put its timers in the log as
    # host-clock checkpoints (five before the first instruction alone); clock=vm
    # made x86 recordings ~3x faster and ~5x smaller. arm64's PL031 also reads the
    # host clock from its vmstate pre_save, i.e. out of band whenever its state is
    # saved. The base date is still one recorded read.
    if "-rtc" not in argv:
        out += ["-rtc", "clock=vm"]
    # A machine given no network flag still gets one: QEMU adds its DEFAULT NIC (arm64
    # virt: virtio-net-pci; x86 pc: e1000) on user-mode networking. The refusal above
    # only sees explicit -netdev/-nic/-net, so that NIC walked past it, and its
    # packets — answered by slirp at host time and DMA'd into guest RAM — reached the
    # guest unrecorded. That was the arm64 kmtest divergence: the replayed serial is a
    # byte-exact prefix of the recording up to the netstack self-test probing
    # virtio-net, and the wedge always fell in that phase. `-nic user` with no model
    # builds the same default NIC through the same machine path (same model, slot and
    # MAC), and filter-replay puts its traffic in the log.
    #
    # filter-replay pins RECEIVED packets. virtio-net's transmit side was still loose:
    # its default `tx=bh` completes transmits from a plain bottom half, which runs
    # at host time and writes the used ring into guest RAM wherever the guest
    # happens to be. With receive recorded, the arm64 replay got through DHCP and
    # DNS and then wedged at the first ICMP echo. `tx=timer` runs the same flush on
    # a QEMU_CLOCK_VIRTUAL timer, which under icount is the instruction counter.
    # `-global` reaches the default NIC, which takes no device properties.
    out += [
        "-nic",
        "user,id=rrnet0",
        "-object",
        "filter-replay,id=rrnetreplay0,netdev=rrnet0",
        "-global",
        "virtio-net-pci.tx=timer",
    ]
    # Every virtio queue notify is the same shape of leak. With ioeventfd on (the
    # default, and NOT gated on KVM: virtio-mmio/virtio-pci enable it by flag under
    # TCG too), the guest's notify write only signals an eventfd, and the main loop
    # runs the device's queue handler later at host time — outside any instruction
    # the log can pin. Off, the handler runs synchronously inside the vCPU's MMIO
    # write, at a deterministic icount. Covers virtio-blk, -net and -input alike.
    out += ["-global", "virtio-mmio.ioeventfd=off", "-global", "virtio-pci.ioeventfd=off"]
    # x86 REPLAY does not load the `init` snapshot. Loading it re-arms a virtual-clock
    # timer due at time zero (a device post-load hook), which in replay may only run at
    # a recorded CHECKPOINT_CLOCK_VIRTUAL -- and the recording has none before its first
    # instruction. Main loop and vCPU then livelock (vCPU in icount_handle_deadline ->
    # qemu_clock_notify, forever) at instruction count 0. Recording still CREATES the
    # snapshot: prepare_replay reverts every disk to it, so the replay starts from the
    # same disks and the same power-on state the recording did.
    snap = "" if (x86 and mode == "replay") else ",rrsnapshot=init"
    out += [
        "-icount",
        f"shift={icount_shift},rr={mode},rrfile={rrfile}{snap}",
    ]
    return out, media


def is_x86_argv(argv: list[str]) -> bool:
    """Whether a QEMU command line is an x86 machine (by its binary name)."""
    return bool(argv) and ("x86_64" in argv[0] or "i386" in argv[0])


# --------------------------------------------------------------------------
# trace lifecycle
# --------------------------------------------------------------------------


def write_manifest(trace_dir: Path, *, arch: str, argv: list[str], media: dict, **extra) -> Path:
    """Pin everything a replay's fidelity depends on.

    A trace recorded against a different QEMU build, or against media from a
    different commit, will diverge — usually as a confusing mid-boot abort
    rather than an error.  Recording these here means `replay` can say so up
    front instead of leaving you to debug the wrong thing.
    """
    trace_dir.mkdir(parents=True, exist_ok=True)
    ver = subprocess.run(
        ["qemu-system-aarch64", "--version"], capture_output=True, text=True
    ).stdout.splitlines()
    manifest = {
        "arch": arch,
        "created": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "qemu_version": ver[0] if ver else "unknown",
        "icount_shift": DEFAULT_ICOUNT_SHIFT,
        "argv": argv,
        "media": media,
        **extra,
    }
    p = trace_dir / "manifest.json"
    p.write_text(json.dumps(manifest, indent=2) + "\n")
    return p


def trace_read_position(trace_dir: Path) -> int | None:
    """How far into `replay.bin` the replaying QEMU has read, or None if nobody
    has it open.

    This is the measurement that separates a replay that is merely SLOW from one
    that is WEDGED, and nothing else we have distinguishes them: both look like
    a quiet serial log and a busy CPU. A frozen position means QEMU is executing
    without consuming any further recorded event — it is not going to finish, and
    waiting longer is wasted.

    The value is the KERNEL file offset, so it advances in buffered chunks rather
    than byte by byte. That is fine for the only question asked of it — is it
    moving at all, over minutes — but do not read it as a byte-accurate cursor.

    Read from /proc, so Linux-only; returns None anywhere it cannot look, and
    callers treat that as "no information" rather than as a verdict.
    """
    rrfile = (Path(trace_dir) / "replay.bin").resolve()
    proc = Path("/proc")
    if not proc.is_dir():
        return None
    for pid_dir in proc.iterdir():
        if not pid_dir.name.isdigit():
            continue
        fd_dir = pid_dir / "fd"
        try:
            fds = list(fd_dir.iterdir())
        except OSError:
            continue  # not ours, or exited between listing and opening
        for fd in fds:
            try:
                # /proc already exposes the kernel's absolute target. Resolving
                # every target walks unrelated mounts (including Windows media)
                # and can stall this progress probe on their filesystem I/O.
                if fd.readlink() != rrfile:
                    continue
                for line in (pid_dir / "fdinfo" / fd.name).read_text().splitlines():
                    if line.startswith("pos:"):
                        return int(line.split()[1])
            except (OSError, ValueError):
                continue
    return None


def note_run(trace_dir: Path, *, mode: str, serial_log: str) -> Path:
    """Record where the CURRENT run's serial is going.

    Separate from the manifest on purpose — see the call site in vm.py. Rewritten
    by every record and every replay, so whoever reads it always gets the run
    that just happened.
    """
    p = Path(trace_dir) / "last-run.json"
    p.write_text(
        json.dumps({"mode": mode, "serial_log": serial_log, "when": time.time()}, indent=2) + "\n"
    )
    return p


def last_run(trace_dir: Path) -> dict:
    p = Path(trace_dir) / "last-run.json"
    if not p.is_file():
        raise RecordReplayError(
            f"{trace_dir} has no last-run.json — the launch hook did not run, so this "
            "boot was not actually recorded or replayed"
        )
    return json.loads(p.read_text())


def snapshot_pristine(trace_dir: Path) -> None:
    """Freeze the media as they stand at the END of the recording.

    AFTER, not before — and the distinction is the whole correctness of replay.
    `rrsnapshot=init` writes a vmstate snapshot INTO the qcow2 at record time,
    and replay loads it. Media frozen before the boot do not contain that
    snapshot, so restoring them would leave replay with nothing to load. The
    guest's own writes are not a problem: loading `init` reverts the disk to the
    recorded starting point, which is exactly what it is for.

    Without any freeze the FIRST replay would work (the record run left the
    images where the trace expects) and every one after it would inherit
    whatever the previous replay wrote outside the snapshot.
    """
    media = trace_dir / "media"
    pristine = trace_dir / "pristine"
    if pristine.exists():
        shutil.rmtree(pristine)
    shutil.copytree(media, pristine)


def prepare_replay(trace_dir: Path) -> None:
    """Restore pristine media. Idempotent; run before EVERY replay."""
    pristine = trace_dir / "pristine"
    media = trace_dir / "media"
    if not pristine.is_dir():
        raise RecordReplayError(
            f"{trace_dir} has no pristine/ — it was not produced by a completed "
            "recording, so there is nothing to replay from"
        )
    if media.exists():
        shutil.rmtree(media)
    shutil.copytree(pristine, media)
    argv = load_manifest(trace_dir).get("argv") if (trace_dir / "manifest.json").is_file() else None
    if is_x86_argv(argv or []):
        # x86 replays from power-on, not from the snapshot (see transform): put each
        # disk back to its state at record start, which the snapshot holds.
        for img in sorted(media.glob("*.qcow2")):
            cp = subprocess.run(
                ["qemu-img", "snapshot", "-a", "init", str(img)], capture_output=True, text=True
            )
            if cp.returncode != 0:
                raise RecordReplayError(
                    f"cannot revert {img.name} to its record-start state (`init`): "
                    f"{cp.stderr.strip()} -- the replay would start from post-recording disks"
                )


def load_manifest(trace_dir: Path) -> dict:
    p = Path(trace_dir) / "manifest.json"
    if not p.is_file():
        raise RecordReplayError(f"no manifest.json in {trace_dir} — not a vibeos rr trace")
    return json.loads(p.read_text())


# Marks a check_trace() line that means the replay CANNOT work, as opposed to
# might not. Kept as a marker inside the message so there is one place to look,
# rather than a parallel classification that can drift out of step with the text.
FATAL_MARKER = "[fatal]"


def check_trace(trace_dir: Path) -> list[str]:
    """Problems with a trace that may — or may not — replay faithfully.

    A line containing FATAL_MARKER means the replay is pointless, not merely
    suspect, and `rr replay` refuses on those instead of burning a long run: a
    replay of a trace whose overlay source had been rebuilt got as far as opening
    the gdbstub and then dropped the connection, having printed exactly that
    warning and carried on regardless. A warning nobody can act on mid-run is not
    a warning.
    """
    warnings: list[str] = []
    m = load_manifest(trace_dir)
    cur = subprocess.run(
        ["qemu-system-aarch64", "--version"], capture_output=True, text=True
    ).stdout.splitlines()
    if cur and m.get("qemu_version") and cur[0] != m["qemu_version"]:
        warnings.append(
            f"recorded under {m['qemu_version']!r}, replaying under {cur[0]!r} — "
            "device models differ between QEMU versions and the trace may diverge"
        )
    for name, info in (m.get("media") or {}).items():
        # Tolerate the pre-digest manifest shape (a bare source string).
        if isinstance(info, str):
            info = {"source": info, "copied": False, "sha256": None}
        src, sp = info.get("source", "?"), Path(info.get("source", ""))
        if info.get("copied"):
            continue  # a full copy inside the trace; the source cannot matter
        if not sp.exists():
            warnings.append(
                f"{FATAL_MARKER} {name} is an overlay over {src}, which is GONE — "
                "this trace cannot be replayed. Re-record."
            )
        elif info.get("sha256") and _sha256(sp) != info["sha256"]:
            warnings.append(
                f"{FATAL_MARKER} {name} is an overlay over {src}, which has CHANGED "
                "since the recording (rebuilt media) — the guest would read different "
                "bytes than were recorded. Re-record."
            )
    if not m.get("finalised") and not trace_is_provably_complete(trace_dir):
        # Two ways to be satisfied, because the marker only covers traces recorded
        # after it existed. An older trace whose log decodes end-to-end onto a
        # shutdown event is demonstrably whole, and refusing it would be refusing
        # a trace we can prove is fine — the marker's absence is missing evidence,
        # not evidence of truncation.
        warnings.append(
            f"{FATAL_MARKER} carries no `finalised` marker, and its log does not "
            "decode end-to-end onto a shutdown event, so there is no evidence the "
            "recording was allowed to finish. Traces made before `rr record` waited "
            "for the trace to stop growing are TRUNCATED, and replaying one aborts "
            'with "Missing character write event in the replay log". Re-record.'
        )
    rr = Path(trace_dir) / "replay.bin"
    if not rr.is_file() or rr.stat().st_size == 0:
        warnings.append("replay.bin is missing or empty — the recording did not complete")
    return warnings


# Option groups whose SHAPE a replay must match, beyond the drives. Paths and
# ports change legitimately between runs, so only the backend kind is compared.
_SHAPE_OPTS = ("-serial", "-qmp", "-monitor", "-chardev", "-display")


def _host_shape(argv: list[str]) -> list[str]:
    """The host-side device shape, with per-run detail stripped.

    Replay reach turns out to depend on backends the GUEST cannot observe.
    Measured, same trace, three configurations:

        serial=file:      no QMP   ->  1773 guest lines
        serial=file:      QMP      ->    23 guest lines
        serial=chardev    QMP      ->  1693 guest lines   (the harness; how it was recorded)

    So a replay that does not reproduce the recording's host-side shape can
    under-reach by two orders of magnitude and report nothing unusual. That is
    worse than a crash: it looks like a finding about the guest.
    """
    out, i = [], 0
    while i < len(argv):
        a = argv[i]
        if a in _SHAPE_OPTS:
            val = argv[i + 1] if i + 1 < len(argv) else ""
            kind = val.split(":", 1)[0].split(",", 1)[0]
            out.append(f"{a}={kind}")
            i += 2
            continue
        i += 1
    return out


def assert_same_shape(trace_dir: Path, argv: list[str]) -> None:
    """Refuse a replay whose device set differs from the recording's.

    The failure this prevents is nasty: QEMU starts, the guest boots a little
    way, and then aborts deep in the trace with a message about event ordering.
    That reads as "record/replay is flaky" when it actually means the boot
    command line grew or lost a device since the recording — an arch config
    changed, a VIBEOS_QEMU_* knob is set differently, the harness moved.
    """
    m = load_manifest(trace_dir)
    recorded = [a for a in m.get("argv", []) if a.startswith(("driver=blkreplay", "if="))]
    current = [a for a in argv if a.startswith(("driver=blkreplay", "if="))]
    if recorded != current:
        raise RecordReplayError(
            "replay device set does not match the recording:\n"
            f"  recorded: {recorded}\n"
            f"  now:      {current}\n"
            "Something changed the boot command line since the recording. Re-record."
        )

    rec_shape = _host_shape(m.get("argv", []))
    cur_shape = _host_shape(argv)
    if rec_shape != cur_shape:
        raise RecordReplayError(
            "replay HOST-SIDE shape does not match the recording:\n"
            f"  recorded: {rec_shape}\n"
            f"  now:      {cur_shape}\n"
            "These are backends the guest cannot see, and they still change how far "
            "a replay gets — measured, same trace: file-serial without QMP reached "
            "1773 guest lines, file-serial WITH QMP reached 23, and the recording's "
            "own chardev+QMP shape reached 1693. Replay through the same harness "
            "that recorded, rather than a hand-built command line."
        )


# --------------------------------------------------------------------------
# where a replay can legally be stopped
# --------------------------------------------------------------------------

# Fixed-size payloads for the v12 (REPLAY_VERSION 0xe0200c) event ids, keyed by
# id.  Only the LENGTH matters here — we are skipping, not interpreting — so the
# table carries no names and no semantics beyond "how many bytes follow".
#
# Ids absent from the table are zero-payload events.  EVENT_ASYNC (3..10) and
# EVENT_RANDOM (27) are variable-length and handled in the walk itself.
_V12_PAYLOAD = {
    0: 4,  # EVENT_INSTRUCTION: u32 instruction delta
    22: 8,  # EVENT_CHAR_WRITE: u32 res + u32 offset
    26: 4,  # EVENT_AUDIO_OUT
    28: 8,  # EVENT_CLOCK_HOST
    29: 8,  # EVENT_CLOCK_VIRTUAL_RT
}
_V12_ASYNC_IDS = frozenset(range(3, 11))

# v12 shutdown event ids: EVENT_SHUTDOWN through EVENT_SHUTDOWN___MAX. A log that
# ends on one of these ended because the machine stopped, not because the writer
# was killed.
_V12_SHUTDOWN_IDS = frozenset(range(11, 22))
_V12_RANDOM_ID = 27
# REPLAY_ASYNC_EVENT_BH carries a u64 step id; the other async kinds we have
# never seen in an arm64 trace, and guessing their width would silently
# desynchronise the whole walk, so they stop it instead.
_V12_ASYNC_PAYLOAD = {0: 8}
_SUPPORTED_REPLAY_VERSION = 0xE0200C


# Event names this decoder is SURE of, taken from the sizes it already relies on
# (see _V12_PAYLOAD / _V12_ASYNC_IDS above). Ids absent here are printed as
# "id <n>" rather than guessed at: inventing a plausible name for an event we
# only know the width of would put a label in a wedge report that reads as
# evidence and is not.
_EVENT_NAMES = {0: "EVENT_INSTRUCTION", 22: "EVENT_CHAR_WRITE", 27: "EVENT_RANDOM"}


def event_name(eid: int) -> str:
    if eid in _EVENT_NAMES:
        return _EVENT_NAMES[eid]
    if eid in _V12_ASYNC_IDS:
        return f"EVENT_ASYNC (id {eid})"
    if eid in _V12_SHUTDOWN_IDS:
        return f"EVENT_SHUTDOWN (id {eid})"
    if 30 <= eid <= 38:
        return f"EVENT_CHECKPOINT (id {eid})"
    return f"id {eid}"


class ReplayEvent(NamedTuple):
    """One decoded record: where it starts, what it is, and the icount it ends at."""

    offset: int
    eid: int
    icount: int

    @property
    def name(self) -> str:
        return event_name(self.eid)


def walk_events(trace_dir: Path) -> Iterator[ReplayEvent]:
    """Decode `replay.bin` into its events, in order.

    One walker, because every question we ask of a trace is a question about this
    same byte stream: which icounts are legal break targets, whether the log
    reaches a shutdown, and which event a wedged replay is sitting on. Three
    hand-rolled decoders would drift apart, and a decoder that drifts does not
    fail — it returns plausible numbers.

    Raises RecordReplayError rather than stopping early on anything it cannot
    size, for the reason given on `instruction_boundaries`: a short answer here
    is indistinguishable from a correct one.
    """
    path = Path(trace_dir) / "replay.bin"
    if not path.is_file():
        raise RecordReplayError(f"{path} does not exist — nothing was recorded here")
    total = 0
    with path.open("rb") as fh:
        head = fh.read(12)  # u32 version + u64 of header junk
        if len(head) < 12:
            raise RecordReplayError(f"{path} is truncated: {len(head)} bytes, no header")
        version = struct.unpack(">I", head[:4])[0]
        if version != _SUPPORTED_REPLAY_VERSION:
            raise RecordReplayError(
                f"{path} is replay version 0x{version:x}; this decoder only knows "
                f"0x{_SUPPORTED_REPLAY_VERSION:x}. Refusing to guess at event sizes — "
                "a mis-sized event desynchronises the walk and yields plausible "
                "nonsense rather than an error."
            )
        while True:
            at = fh.tell()
            b = fh.read(1)
            if not b:
                break
            eid = b[0]
            if eid == 0:
                raw = fh.read(4)
                if len(raw) < 4:
                    break  # trailing partial event: the recording was cut short
                total += struct.unpack(">I", raw)[0]
                yield ReplayEvent(at, eid, total)
                continue
            if eid in _V12_ASYNC_IDS:
                kind = fh.read(1)
                fh.read(1)  # checkpoint id, only used to cross-check
                if not kind:
                    break
                width = _V12_ASYNC_PAYLOAD.get(kind[0])
                if width is None:
                    raise RecordReplayError(
                        f"{path}: async event kind {kind[0]} at offset {fh.tell()} has "
                        "no known payload width, so the rest of the log cannot be "
                        "walked. Boundary list would be silently short."
                    )
                fh.read(width)
                yield ReplayEvent(at, eid, total)
                continue
            if eid == _V12_RANDOM_ID:
                fh.read(4)  # ret
                size_raw = fh.read(4)
                if len(size_raw) < 4:
                    break
                fh.read(struct.unpack(">I", size_raw)[0])
                yield ReplayEvent(at, eid, total)
                continue
            fh.read(_V12_PAYLOAD.get(eid, 0))
            yield ReplayEvent(at, eid, total)


def instruction_boundaries(trace_dir: Path) -> list[int]:
    """Every icount at which `replay.bin` records an instruction-chunk boundary.

    These are the only icounts a replay can be stopped at. A replay executes the
    guest in the chunks the recording wrote down — `EVENT_INSTRUCTION + N` means
    "run N more instructions before looking at the log again" — and asking it to
    stop PART WAY through a chunk kills it outright:

        qemu-system-aarch64: Playback shouldn't have to iowait
          (insn total 1024013292/750000 left, event 17661 is EVENT_INSTRUCTION)

    The two numbers in that message are the request and the remainder of the
    chunk it landed inside, which is how the rule was found: three `replay-break`
    attempts on the same trace, the two that survived sat exactly on a boundary
    in this list and the one that aborted sat 499,347 instructions into a
    1,249,347-instruction chunk.

    Chunk sizes are wildly uneven — in one arm64 boot, 1.07M chunks of 18
    instructions apiece during real execution and 1.25M-instruction chunks while
    the guest idles in WFI — so there is no arithmetic shortcut and no safe
    rounding rule. The set has to come from the trace, which is why this walks
    it. Cost is a few seconds for a 10MB log.

    Raises RecordReplayError on a log this decoder does not understand, rather
    than returning a short list: a truncated boundary set would silently make
    `snap_break_icount` recommend an icount that aborts the replay.
    """
    return [e.icount for e in walk_events(trace_dir) if e.eid == 0]


def events_around_offset(
    trace_dir: Path, offset: int, before: int = 5, after: int = 5
) -> tuple[list[ReplayEvent], list[ReplayEvent]]:
    """The events preceding and following a byte offset in `replay.bin`.

    For reading a WEDGE. `trace_read_position` gives the offset a stalled replay
    stopped reading at, and the question is which recorded event it could not
    get past.

    Returns (preceding, following) and deliberately does not name a single
    culprit, because the offset cannot support one: it is the kernel file
    position of a buffered read, so QEMU has consumed bytes into its own buffer
    that it has not yet acted on. The true stall event is therefore at or BEFORE
    the offset, by up to a buffer's worth — which makes `preceding` the evidence
    and `following` the context. A tool that printed "the replay died on this
    event" from this input would be reporting the handle's reading as the
    thing's, which is the mistake this docstring exists to prevent.
    """
    preceding: list[ReplayEvent] = []
    following: list[ReplayEvent] = []
    for ev in walk_events(trace_dir):
        if ev.offset < offset:
            preceding.append(ev)
            if len(preceding) > before:
                preceding.pop(0)
        else:
            following.append(ev)
            if len(following) >= after:
                break
    return preceding, following


def trace_is_provably_complete(trace_dir: Path) -> bool:
    """True if `replay.bin` decodes end-to-end AND finishes on a shutdown event.

    This is the evidence the `finalised` marker cannot supply. The marker records
    that the TOOL waited for the trace to stop growing; it says nothing about
    traces recorded before the marker existed, which is why its absence had to be
    treated as fatal — "nothing here can tell the two apart".

    Something can: the log itself. Walking it proves two things at once. That the
    walk consumes exactly the whole file means no event is cut in half, which is
    what truncation looks like. That the final event is a shutdown means the
    machine stopped of its own accord rather than the writer being killed
    mid-stream. A recording killed part way fails both — it ends inside an event,
    at a byte that is payload rather than an event id.

    Checking the last byte alone would be cheap and wrong: in a truncated log that
    byte is arbitrary, and ~4% of arbitrary bytes fall in the shutdown range. The
    walk is what makes this proof instead of a guess, so it is worth its seconds.

    The criterion is "the walk REACHES a shutdown event", not "the file ends on
    one". Replay stops at shutdown and never looks further, so bytes after it are
    irrelevant — and there are some: clean1 carries a trailing 0x27 past its
    EVENT_SHUTDOWN_HOST_QMP_RESET, which QEMU's own replay-dump.py never reads
    because it stops at the same place. Requiring EOF would fail a trace that is
    demonstrably whole.

    Returns False rather than raising on a log this decoder cannot read — an
    unknown version is a reason to withhold the upgrade, not to refuse the trace.
    """
    path = Path(trace_dir) / "replay.bin"
    if not path.is_file():
        return False
    try:
        with path.open("rb") as fh:
            head = fh.read(12)
            if len(head) < 12:
                return False
            if struct.unpack(">I", head[:4])[0] != _SUPPORTED_REPLAY_VERSION:
                return False
            while True:
                b = fh.read(1)
                if not b:
                    break
                eid = b[0]
                if eid in _V12_SHUTDOWN_IDS:
                    return True  # the machine stopped; replay stops here too
                if eid == 0:
                    if len(fh.read(4)) < 4:
                        return False
                elif eid in _V12_ASYNC_IDS:
                    k = fh.read(1)
                    if not k or len(fh.read(1)) < 1:
                        return False
                    width = _V12_ASYNC_PAYLOAD.get(k[0])
                    if width is None or len(fh.read(width)) < width:
                        return False
                elif eid == _V12_RANDOM_ID:
                    if len(fh.read(4)) < 4:
                        return False
                    raw = fh.read(4)
                    if len(raw) < 4:
                        return False
                    n = struct.unpack(">I", raw)[0]
                    if len(fh.read(n)) < n:
                        return False
                else:
                    want = _V12_PAYLOAD.get(eid, 0)
                    if len(fh.read(want)) < want:
                        return False
    except OSError:
        return False
    return False  # ran off the end without ever stopping: truncated


def snap_break_icount(trace_dir: Path, want: int) -> int:
    """The first legal break icount at or after `want`.

    Snapping FORWARD rather than to the nearest is deliberate: a caller asking to
    stop at N wants to have seen everything up to N, and a backward snap would
    silently stop short of it.

    Raises if `want` is past the end of the recording — there is no later
    boundary to offer, and a replay asked to break past its own end simply runs
    to completion, which looks identical to the break never having been set.
    """
    bounds = instruction_boundaries(trace_dir)
    if not bounds:
        raise RecordReplayError(
            f"{trace_dir} records no instruction chunks at all — there is nowhere to break"
        )
    i = bisect.bisect_left(bounds, want)
    if i == len(bounds):
        raise RecordReplayError(
            f"icount {want} is past the end of this recording (last boundary "
            f"{bounds[-1]}); a break there would never fire"
        )
    return bounds[i]


# --------------------------------------------------------------------------
# the vm.py hook
# --------------------------------------------------------------------------

ENV_VAR = "VIBEOS_QEMU_RR"
SHIFT_ENV_VAR = "VIBEOS_QEMU_RR_SHIFT"
BREAK_ENV_VAR = "VIBEOS_QEMU_RR_BREAK"


def env_shift() -> int:
    """icount shift for this run (VIBEOS_QEMU_RR_SHIFT, else DEFAULT_ICOUNT_SHIFT).

    This is a knob and not a constant because it is the one rr setting that can
    make a healthy guest fail. `shift=N` costs the virtual clock 2**N ns per
    instruction, so it fixes the guest's apparent CPU frequency at
    2**-N GHz — shift=7 is a 7.8 MHz machine. Firmware and bootloader timeouts
    are denominated in that same virtual time, so too LARGE a shift spends the
    whole watchdog budget on work that a real machine finishes in
    milliseconds. See the divergence log in docs/engineering/qemu-record-replay.md.
    """
    raw = os.environ.get(SHIFT_ENV_VAR, "").strip()
    if not raw:
        return DEFAULT_ICOUNT_SHIFT
    try:
        n = int(raw)
    except ValueError as exc:
        raise RecordReplayError(f"{SHIFT_ENV_VAR} must be an integer, got {raw!r}") from exc
    if not 0 <= n <= 10:
        raise RecordReplayError(f"{SHIFT_ENV_VAR} must be 0..10, got {n}")
    return n


def env_break() -> int | None:
    """Requested `replay-break` icount for this replay, or None.

    The value is what the CALLER asked for, not necessarily where the break will
    land: it is snapped forward to a legal boundary at the point the hook is
    installed, because only there is the trace directory known. See
    `snap_break_icount`.
    """
    raw = os.environ.get(BREAK_ENV_VAR, "").strip()
    if not raw:
        return None
    try:
        n = int(raw)
    except ValueError as exc:
        raise RecordReplayError(f"{BREAK_ENV_VAR} must be an integer, got {raw!r}") from exc
    if n < 0:
        raise RecordReplayError(f"{BREAK_ENV_VAR} must not be negative, got {n}")
    return n


def env_request() -> tuple[str, Path] | None:
    """Read `VIBEOS_QEMU_RR=<record|replay>:<trace-dir>`, or None.

    This env var is how the rr transform reaches vm.py without the boot harness
    growing an rr-shaped parameter on every code path that builds a VM.  The CLI
    verbs set it; nothing sets it by default.
    """
    raw = os.environ.get(ENV_VAR, "").strip()
    if not raw:
        return None
    if ":" not in raw:
        raise RecordReplayError(f"{ENV_VAR} must be '<record|replay>:<trace-dir>', got {raw!r}")
    mode, _, d = raw.partition(":")
    if mode not in ("record", "replay"):
        raise RecordReplayError(f"{ENV_VAR} mode must be record or replay, got {mode!r}")
    return mode, Path(d)
