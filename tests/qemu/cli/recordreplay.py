"""QEMU record/replay ("rr") for vibeos boots — deterministic re-execution.

WHY THIS EXISTS
---------------
Our hardest bugs are heisenbugs: an arm64 MP boot faults at PC=0 roughly 1 in 60
boots, and every attempt to look at it changes it.  Soaking found the RATE and
nothing else in months.  QEMU's record/replay makes one *particular* boot — the
one that crashed — into an artifact you can run again, identically, as many
times as you like, with gdb attached and reverse-execution available.  That
turns "reproduce it again and hope" into "open the recording".

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
* No network.  Recording a slirp/tap backend needs replay net filters we do not
  have wired; arm64 conformance has no NIC, so we refuse instead of guessing.
* Native backend only.  The windows-host backend stages media into a Windows
  path and runs QEMU.exe; the trace dir would be half on each side.

See docs/engineering/qemu-record-replay.md.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from pathlib import Path

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

        out.append(arg)
        i += 1

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
    out += [
        "-icount",
        f"shift={icount_shift},rr={mode},rrfile={rrfile},rrsnapshot=init",
    ]
    return out, media


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


def load_manifest(trace_dir: Path) -> dict:
    p = Path(trace_dir) / "manifest.json"
    if not p.is_file():
        raise RecordReplayError(f"no manifest.json in {trace_dir} — not a vibeos rr trace")
    return json.loads(p.read_text())


def check_trace(trace_dir: Path) -> list[str]:
    """Warnings (not errors) about a trace that may not replay faithfully."""
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
                f"{name} is an overlay over {src}, which is GONE — this trace cannot be replayed"
            )
        elif info.get("sha256") and _sha256(sp) != info["sha256"]:
            warnings.append(
                f"{name} is an overlay over {src}, which has CHANGED since the "
                "recording (rebuilt media) — the replay will diverge"
            )
    rr = Path(trace_dir) / "replay.bin"
    if not rr.is_file() or rr.stat().st_size == 0:
        warnings.append("replay.bin is missing or empty — the recording did not complete")
    return warnings


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


# --------------------------------------------------------------------------
# the vm.py hook
# --------------------------------------------------------------------------

ENV_VAR = "VIBEOS_QEMU_RR"
SHIFT_ENV_VAR = "VIBEOS_QEMU_RR_SHIFT"


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
