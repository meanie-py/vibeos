"""`vibeos-cli rr ...` — record a boot, then replay it deterministically.

    rr record --arch arm64 --payload kmtest     # boot once, keep the recording
    rr prune --keep 3 [--apply]                  # traces are big; nothing else reclaims them
    rr list                                      # what traces exist
    rr info <trace>                              # what it is, and will it still replay
    rr replay <trace> [--gdb]                    # run that exact boot again
    rr verify <trace>                            # replay and diff serial vs the record
    rr breaks <trace> [--near N]                 # icounts a replay can legally stop at

A trace is a self-contained directory under $VIBEOS_RR_ROOT (default
~/vibeos-rr/): the instruction-stream log, private copies of every image the
guest touched, a pristine copy of those images to reset to, the exact argv, and
a manifest pinning the QEMU build and commit it came from.  Nothing in it points
into a build tree, so a trace outlives the tree that made it.

Replay deliberately goes back through the SAME boot harness the recording used
rather than re-running the stored argv: that way a replay produces an ordinary
artifact dir with serial, triage and screenshots, and `--gdb` is the harness's
existing gdbstub plumbing rather than a second implementation.  The per-run
sockets and work dir differ between record and replay and that is fine — they
are not guest-visible.  What must not differ is the media and the device set,
and `recordreplay.assert_same_shape` asserts exactly that.

See docs/engineering/qemu-record-replay.md for the determinism rules and the
divergence log.
"""

from __future__ import annotations

import bisect
import itertools
import json
import os
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

from . import recordreplay as rr  # re-exported for the arg parser

# Traces are expensive (a full boot each) and worth keeping across reboots, so
# they do NOT live in /tmp.
DEFAULT_ROOT = Path(os.environ.get("VIBEOS_RR_ROOT", Path.home() / "vibeos-rr"))

# vibeos knobs that change the device set or the guest's behaviour. A replay
# must run with the same values or the recorded input stream no longer lines up
# with the devices asking for it, so we persist them with the trace.
_PINNED_ENV = (
    "VIBEOS_QEMU_SMP",
    "VIBEOS_QEMU_USB_DISK",
    "VIBEOS_QEMU_XHCI",
    "VIBEOS_QEMU_EXTRA_ARGS",
    "VIBEOS_TRIPWIRE",
    "VIBEOS_UP_DEFAULT_BOOT",
)


def _root(args) -> Path:
    return Path(getattr(args, "root", None) or DEFAULT_ROOT)


def _resolve_trace(args) -> Path:
    """Accept a bare trace name or a path; `latest` picks the newest."""
    raw = str(args.trace)
    p = Path(raw)
    if p.is_dir():
        return p
    root = _root(args)
    if raw == "latest":
        traces = sorted(
            (d for d in root.glob("*") if (d / "manifest.json").is_file()),
            key=lambda d: d.stat().st_mtime,
        )
        if not traces:
            raise rr.RecordReplayError(f"no traces under {root}")
        return traces[-1]
    cand = root / raw
    if cand.is_dir():
        return cand
    raise rr.RecordReplayError(f"no such trace: {raw} (looked in {root})")


def _head_sha(repo: Path) -> str:
    out = subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo, capture_output=True, text=True)
    return out.stdout.strip() if out.returncode == 0 else "unknown"


def _boot_cmd(repo: Path, arch: str, payload: str, smp: int, build: bool, timeout: int):
    """The vibeos-cli invocation whose boot we are recording.

    A `conformance` or `smoke` boot — never a `gate` boot. Anything the gate adds
    is therefore outside what a trace captures, and adding it here would be a
    deliberate choice rather than something inherited: two traces are comparable
    only while what gets recorded is pinned. (The WoW64 ladder is the live
    example — it runs on arm64 gates but not here, because nothing in this
    function passes --ladder.) See docs/engineering/qemu-record-replay.md.
    """
    cli = str(repo / "vibeos-cli")
    if payload == "kmtest":
        cmd = [cli, "conformance", "--arch", arch, "--smp", str(smp), "--timeout", str(timeout)]
        if not build:
            cmd.append("--no-build")
        return cmd
    return [cli, "smoke", "--arch", arch, "--smp", str(smp)]


# --------------------------------------------------------------------------


def cmd_record(args, repo: Path) -> int:
    root = _root(args)
    name = args.name or f"{time.strftime('%Y%m%d-%H%M%S')}-{args.arch}-{args.payload}"
    trace = root / name
    if trace.exists():
        print(f"[rr] trace already exists: {trace}", file=sys.stderr)
        return 2
    trace.mkdir(parents=True)

    env = dict(os.environ)
    env[rr.ENV_VAR] = f"record:{trace}"
    env[rr.SHIFT_ENV_VAR] = str(args.shift)
    if args.shift != rr.DEFAULT_ICOUNT_SHIFT:
        # Measured on arm64, same boot, three shifts: only the default replays.
        #   shift 1  33,571,609 byte trace -> replay stalls at 4,096 (271 serial bytes)
        #   shift 3  10,666,166            -> reaches 2,342,912 (124,212, desktop bringup)
        #   shift 5   5,999,935            -> replay stalls at 4,096 (271 serial bytes)
        # 4,096 is the first read buffer: at 1 and 5 the replay never gets going.
        # The recording still succeeds, so the failure is invisible until someone
        # tries to replay -- which is exactly when the recording is expensive to
        # redo. Say it up front.
        print(
            f"[rr] WARNING: --shift {args.shift} is not the default "
            f"({rr.DEFAULT_ICOUNT_SHIFT}). Recording will work; REPLAY PROBABLY WILL "
            "NOT. Measured on arm64: shift 1 and shift 5 both stall at the first "
            "4,096-byte buffer having produced 271 bytes of guest serial, while "
            "shift 3 reaches desktop bringup. See "
            "docs/engineering/qemu-record-replay.md.",
            file=sys.stderr,
        )
    env["VIBEOS_QEMU_SMP"] = str(args.smp)
    # The trace has to come off the native stack; see recordreplay's module
    # docstring. Say so up front rather than letting the VM raise mid-launch.
    env["VIBEOS_QEMU_BACKEND"] = "native"

    cmd = _boot_cmd(repo, args.arch, args.payload, args.smp, args.build, args.timeout)
    print(f"[rr] recording -> {trace}")
    print(f"[rr] boot: {' '.join(cmd)}")
    t0 = time.time()
    rc = subprocess.run(cmd, cwd=str(repo), env=env).returncode
    elapsed = time.time() - t0

    # The boot's own verdict is NOT the trace's verdict: a boot that bugchecked
    # is the most valuable recording we can hold, and reporting it as a failure
    # would make the tool useless for exactly its purpose. Judge the trace by
    # whether it replays.
    # The boot command returning is not the recording ending; wait for the trace
    # to stop growing before finalising anything. See await_trace_quiescence.
    size = await_trace_quiescence(trace)
    if size:
        # Freeze the media now: they carry the `init` vmstate snapshot the
        # recording just wrote, which every replay loads. Before the boot they
        # would not have it. See recordreplay.snapshot_pristine.
        rr.snapshot_pristine(trace)
        _keep_serial(trace, "record-serial.log", expect_mode="record")
        finalised = True
    meta = {}
    mpath = trace / "manifest.json"
    finalised = locals().get("finalised", False)
    if mpath.is_file():
        meta = json.loads(mpath.read_text())
    meta.update(
        {
            "boot_cmd": cmd,
            "boot_rc": rc,
            "record_seconds": round(elapsed, 1),
            "payload": args.payload,
            "commit": _head_sha(repo),
            # Set only after await_trace_quiescence() confirmed the recording had
            # stopped growing. Traces made before that existed are truncated and
            # look identical from the outside — their replays abort with
            # "Missing character write event in the replay log". The absence of
            # this flag is the only way to recognise that generation.
            "finalised": finalised,
            "env": {k: os.environ.get(k, "") for k in _PINNED_ENV if os.environ.get(k)},
        }
    )
    meta["env"]["VIBEOS_QEMU_SMP"] = str(args.smp)
    mpath.write_text(json.dumps(meta, indent=2) + "\n")

    print(f"[rr] boot rc={rc} in {elapsed:.0f}s; trace {size / 1e6:.0f}MB at {trace}")
    if size == 0:
        print(
            "[rr] NO TRACE WRITTEN — the recording did not reach a clean shutdown.\n"
            "     QEMU flushes the rr log on exit, so a killed QEMU leaves nothing. "
            "     Let the boot end on its own (or via QMP quit), not via a timeout kill.",
            file=sys.stderr,
        )
        return 1
    print(f"[rr] replay it:  ./vibeos-cli rr replay {trace.name}")
    return 0


def await_trace_quiescence(trace: Path, *, settle: float = 6.0, limit: float = 420.0) -> int:
    """Block until the recording has actually finished, and return the final size.

    The boot command returning does NOT mean QEMU has exited. `vibeos-cli
    conformance` hands back as soon as its sweep is parsed, while the VM is still
    shutting down — measured on a kmtest recording: the command returned at
    05:28:38, the guest's serial finished at 05:29:23, and the trace finished at
    05:29:39. Finalising at the first of those three froze pristine/ mid-record,
    copied a 1,456-byte transcript for what became a 240,750-byte boot, and
    reported "trace 0MB" for what became 10.6MB.

    Every one of those is silent and each poisons something different: a
    mid-record pristine/ may not carry the `init` snapshot every replay loads, and
    a truncated record-serial.log is what `rr verify` diffs against — the same
    false-green class as reading the serial path from the manifest.

    Smoke never showed it because its harness waits for its own VM. kmtest is the
    payload this lane is actually chartered for, so this mattered.
    """
    rrbin = trace / "replay.bin"
    last, stable_since = -1, time.time()
    deadline = time.time() + limit
    while time.time() < deadline:
        size = rrbin.stat().st_size if rrbin.is_file() else 0
        if size != last:
            last, stable_since = size, time.time()
        elif time.time() - stable_since >= settle:
            return size
        time.sleep(2.0)
    print(
        f"[rr] WARNING: trace still growing after {limit:.0f}s; finalising anyway "
        f"at {last:,} bytes — it may be truncated.",
        file=sys.stderr,
    )
    return last


def _keep_serial(trace: Path, name: str, *, expect_mode: str) -> Path | None:
    """Copy the run that JUST happened into the trace under `name`.

    The path comes from last-run.json, which the launch hook rewrites on every
    run — not from the manifest, which is the recording's pin and which a replay
    never touches. Reading it from the manifest handed a replay the RECORD's own
    transcript, so `verify` compared a file with itself and reported VERIFIED on
    a replay that had hung at a fifth of the recording's output.

    `expect_mode` is the guard for the same class of mistake: if the last run
    through this trace was not the mode we think we just ran, we are about to
    grade the wrong boot.
    """
    try:
        run = rr.last_run(trace)
    except rr.RecordReplayError:
        return None
    if run.get("mode") != expect_mode:
        print(
            f"[rr] refusing to keep serial: last run through this trace was "
            f"{run.get('mode')!r}, expected {expect_mode!r}",
            file=sys.stderr,
        )
        return None
    src = Path(run.get("serial_log", ""))
    if not src.is_file():
        return None
    dst = trace / name
    shutil.copy2(src, dst)
    return dst


def _guest_lines(p: Path) -> list[str]:
    """The guest's own serial output, with the harness's banner dropped.

    The live log opens with a `# vibeos smoke live serial ... pid(launcher)=N ...
    started <time>` header written by the HARNESS, not the guest. It differs on
    every run by construction, so diffing it would make `verify` report a
    divergence on line 1 of every trace — a self-inflicted false positive that
    would discredit the one check that proves the tool works.
    """
    lines = p.read_text(errors="replace").splitlines()
    i = 0
    while i < len(lines) and (not lines[i].strip() or lines[i].startswith("#")):
        i += 1
    return lines[i:]


def _serial_divergence(a: list[str], b: list[str]) -> tuple[int, str] | None:
    """First line where two transcripts differ, or None if one is a prefix of the
    other. A prefix is not a divergence: a replay that was stopped early, or a
    record whose tail was still being flushed, both produce one."""
    for n, (x, y) in enumerate(zip(a, b, strict=False), 1):
        if x != y:
            return n, f"record: {x[:160]}\nreplay: {y[:160]}"
    return None


def cmd_verify(args, repo: Path) -> int:
    """Replay a trace and prove the guest re-executed identically.

    This is the tool's own oracle. Without it `rr replay` can only tell you that
    a VM ran, not that it ran the SAME boot — and a trace that quietly diverges
    is worse than no trace, because you debug the divergence thinking it is the
    bug.
    """
    trace = _resolve_trace(args)
    recorded = trace / "record-serial.log"
    if not recorded.is_file():
        print(
            f"[rr] {trace.name} has no record-serial.log — it predates `verify`, "
            "or the recording did not complete. Re-record.",
            file=sys.stderr,
        )
        return 2
    rc = cmd_replay(args, repo)
    replayed = _keep_serial(trace, "replay-serial.log", expect_mode="replay")
    if replayed is None:
        print("[rr] the replay produced no serial transcript", file=sys.stderr)
        return 2

    ra, rb = _guest_lines(recorded), _guest_lines(replayed)
    div = _serial_divergence(ra, rb)
    if div is None and len(ra) == len(rb):
        print(f"[rr] VERIFIED — {len(ra)} lines of guest serial, identical to the recording")
        return 0
    if div is None:
        # A prefix, not a contradiction — but WHICH side is short decides whether
        # this is a pass. A short REPLAY means it never got to the end and the
        # tail is unproven; a short RECORD only means the recording's own tail
        # was cut, which the replay then went past.
        print(
            f"[rr] PARTIAL — agreed for {min(len(ra), len(rb))} lines with no "
            f"divergence; record has {len(ra)}, replay {len(rb)}."
        )
        if len(rb) < len(ra):
            print("[rr] The REPLAY stopped short, so the rest of the boot is unproven.")
            return rc or 1
        return rc
    n, detail = div
    print(f"[rr] DIVERGED at serial line {n}:\n{detail}")
    print(
        "[rr] This is a FINDING about our determinism, not necessarily a tool bug. "
        f"Log it in docs/engineering/qemu-record-replay.md. Transcripts: {recorded} "
        f"and {replayed}"
    )
    return 1


class _TraceProgress:
    """Samples how far the replaying QEMU has read into replay.bin.

    Exists so a replay that stops can SAY WHY. Without it the operator gets a
    bare HANG_TIMEOUT and no way to tell "be patient" from "this trace will not
    replay" — and the two call for opposite responses. A frozen read position
    with the CPU pinned means QEMU is executing and consuming no further recorded
    events; more waiting buys nothing.
    """

    def __init__(self, trace: Path, period: float = 20.0) -> None:
        self.trace = trace
        self.period = period
        self.total = (trace / "replay.bin").stat().st_size
        self.max_pos = 0
        self.stalled_for = 0.0
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None

    def _run(self) -> None:
        last = -1
        while not self._stop.wait(self.period):
            pos = rr.trace_read_position(self.trace)
            if pos is None:
                continue
            self.max_pos = max(self.max_pos, pos)
            self.stalled_for = self.stalled_for + self.period if pos == last else 0.0
            last = pos

    def __enter__(self) -> _TraceProgress:
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *exc) -> None:
        self._stop.set()
        if self._thread:
            self._thread.join(timeout=2)

    def report(self) -> str | None:
        if not self.total or not self.max_pos:
            return None
        pct = 100.0 * self.max_pos / self.total
        if self.stalled_for >= 60.0:
            return (
                f"replay WEDGED: read {self.max_pos:,} of {self.total:,} trace bytes "
                f"({pct:.1f}%) and stopped consuming events for {self.stalled_for:.0f}s "
                "while QEMU kept running. This trace will not get further; waiting "
                "longer does not help. See docs/engineering/qemu-record-replay.md."
            )
        return f"replay consumed {self.max_pos:,} of {self.total:,} trace bytes ({pct:.1f}%)"


def cmd_replay(args, repo: Path) -> int:
    trace = _resolve_trace(args)
    m = rr.load_manifest(trace)
    problems = rr.check_trace(trace)
    fatal = [w for w in problems if rr.FATAL_MARKER in w]
    for w in problems:
        print(f"[rr] {'REFUSING' if w in fatal else 'WARNING'}: {w}", file=sys.stderr)
    if fatal:
        # Do not burn a replay that cannot work. Not hypothetical: a replay whose
        # media source had been rebuilt ran long enough to open the gdbstub before
        # dropping the connection, having printed this exact warning first.
        print("[rr] not replaying — re-record with `./vibeos-cli rr record`.", file=sys.stderr)
        return 2

    rr.prepare_replay(trace)

    env = dict(os.environ)
    env[rr.ENV_VAR] = f"replay:{trace}"
    # The shift is part of the machine the trace was recorded against, not a
    # preference — replaying at a different one feeds the recorded input stream
    # to a differently-clocked guest.
    env[rr.SHIFT_ENV_VAR] = str(m.get("icount_shift", rr.DEFAULT_ICOUNT_SHIFT))
    env["VIBEOS_QEMU_BACKEND"] = "native"
    for k, v in (m.get("env") or {}).items():
        env[k] = v
    if getattr(args, "break_at", None) is not None:
        # Snapped in vm.py, where the trace is in hand; announced there too.
        env[rr.BREAK_ENV_VAR] = str(args.break_at)

    if args.gdb:
        # -S as well: a replay you cannot stop before instruction 0 is a replay
        # you cannot reverse-continue from, which is the point of having it.
        env["VIBEOS_QEMU_GDB"] = "wait"
        print(
            "[rr] NOTE: do NOT stop a replay from the debugger. stepi, attaching\n"
            "     mid-run and even a breakpoint HIT all abort QEMU with\n"
            "     `Playback shouldn't have to iowait`. Stop it with QMP\n"
            "     `replay-break` at an icount instead — `--break-at <icount>` does\n"
            "     this for you, snapping to a legal instruction-chunk boundary —\n"
            "     THEN attach gdb to the already-paused guest. That works:\n"
            "     registers read, QEMU survives.",
            file=sys.stderr,
        )

    cmd = m.get("boot_cmd") or _boot_cmd(
        repo, m["arch"], m.get("payload", "kmtest"), 1, False, 6000
    )
    cmd = [c for c in cmd if c != "--build"]
    if "conformance" in cmd and "--no-build" not in cmd:
        cmd.append("--no-build")
    if "smoke" in cmd and "--allow-stale" not in cmd:
        # Not a weakening of the staleness gate: a replay boots the TRACE's media,
        # not the tree's, so "the tree has moved on since this build" is exactly
        # the situation a trace is supposed to survive. `rr info` is what checks
        # the media a replay actually depends on.
        cmd.append("--allow-stale")
    print(f"[rr] replaying {trace.name} (recorded {m.get('created')})")
    print(f"[rr] boot: {' '.join(cmd)}")
    with _TraceProgress(trace) as progress:
        rc = subprocess.run(cmd, cwd=str(repo), env=env).returncode
    print(f"[rr] replay rc={rc}")
    note = progress.report()
    if note:
        print(f"[rr] {note}")
    return rc


def cmd_list(args, repo: Path) -> int:
    root = _root(args)
    if not root.is_dir():
        print(f"no traces (root {root} does not exist)")
        return 0
    rows = []
    for d in sorted(root.iterdir()):
        mp = d / "manifest.json"
        if not mp.is_file():
            continue
        try:
            m = json.loads(mp.read_text())
        except Exception:
            continue
        rrbin = d / "replay.bin"
        size = rrbin.stat().st_size if rrbin.is_file() else 0
        total = sum(f.stat().st_size for f in d.rglob("*") if f.is_file())
        rows.append(
            (
                d.name,
                m.get("arch", "?"),
                m.get("payload", "?"),
                m.get("boot_rc", "?"),
                f"{size / 1e6:.0f}M",
                f"{total / 1e6:.0f}M",
                m.get("created", "?"),
            )
        )
    if not rows:
        print(f"no traces under {root}")
        return 0
    print(f"{'trace':<40} {'arch':<6} {'payload':<8} {'rc':<4} {'rr':<6} {'total':<7} created")
    for r in rows:
        print(f"{r[0]:<40} {r[1]:<6} {r[2]:<8} {r[3]!s:<4} {r[4]:<6} {r[5]:<7} {r[6]}")
    return 0


def cmd_prune(args, repo: Path) -> int:
    """Delete old traces, newest-first retention. DRY-RUN by default.

    Traces are big and nothing was reclaiming them: a kmtest capture is ~1.4GB
    (489MB of trace plus its media, twice, counting pristine/), and a handful of
    experiment traces had the disk at 89% within a night. `vibeos-cli prune`
    reclaims build and boot artifacts and does not know about ~/vibeos-rr at all,
    so this is the only thing that will.

    Dry-run by default and `--apply` to act, matching `vibeos-cli prune` — a
    trace costs a full boot to make, so deleting one should take the same
    deliberate second keystroke.
    """
    root = _root(args)
    if not root.is_dir():
        print(f"no traces (root {root} does not exist)")
        return 0
    traces = sorted(
        (d for d in root.iterdir() if (d / "manifest.json").is_file()),
        key=lambda d: d.stat().st_mtime,
        reverse=True,
    )
    keep = max(0, int(args.keep))
    doomed = traces[keep:]
    if args.name:
        doomed = [d for d in traces if d.name in set(args.name)]
    if not doomed:
        print(f"{len(traces)} trace(s), keeping {keep} — nothing to prune")
        return 0
    total = 0
    for d in doomed:
        size = sum(f.stat().st_size for f in d.rglob("*") if f.is_file())
        total += size
        print(f"{'DELETE' if args.apply else 'would delete'} {d.name:<34} {size / 1e6:8.0f}MB")
        if args.apply:
            shutil.rmtree(d, ignore_errors=True)
    verb = "freed" if args.apply else "would free"
    print(f"{verb} {total / 1e9:.2f}GB across {len(doomed)} trace(s)")
    if not args.apply:
        print("re-run with --apply to actually delete")
    return 0


def cmd_info(args, repo: Path) -> int:
    trace = _resolve_trace(args)
    m = rr.load_manifest(trace)
    print(f"trace:   {trace}")
    for k in (
        "arch",
        "payload",
        "created",
        "commit",
        "qemu_version",
        "icount_shift",
        "boot_rc",
        "record_seconds",
    ):
        if k in m:
            print(f"{k + ':':<16} {m[k]}")
    print(f"{'env:':<16} {m.get('env', {})}")
    print(f"{'media:':<16}")
    for name, info in (m.get("media") or {}).items():
        if isinstance(info, str):
            info = {"source": info, "copied": False}
        how = "copy of" if info.get("copied") else "overlay over"
        print(f"    {name:<24} {how} {info.get('source')}")
    warnings = rr.check_trace(trace)
    if warnings:
        print("warnings:")
        for w in warnings:
            print(f"    {w}")
    else:
        print("warnings:        none — should replay faithfully")
    return 0


def cmd_breaks(args, repo: Path) -> int:
    trace = _resolve_trace(args)
    bounds = rr.instruction_boundaries(trace)
    if not bounds:
        print(f"{trace} records no instruction chunks — there is nowhere to break")
        return 1

    if args.near is None:
        gaps = [b - a for a, b in itertools.pairwise(bounds)]
        gaps.sort()
        print(f"trace:   {trace}")
        print(f"{'boundaries:':<16} {len(bounds)}")
        print(f"{'icount range:':<16} {bounds[0]} .. {bounds[-1]}")
        if gaps:
            mid = gaps[len(gaps) // 2]
            print(f"{'chunk size:':<16} min {gaps[0]}  median {mid}  max {gaps[-1]}")
        print(
            "\nA `replay-break` must land on one of these. Anywhere else aborts the\n"
            "replay with 'Playback shouldn't have to iowait'. Use --near <icount> to\n"
            "pick one."
        )
        return 0

    i = bisect.bisect_left(bounds, args.near)
    exact = i < len(bounds) and bounds[i] == args.near
    lo = max(0, i - args.count)
    hi = min(len(bounds), i + args.count + 1)
    for b in bounds[lo:hi]:
        if b == args.near:
            mark = "   <- requested (legal)"
        elif not exact and i < len(bounds) and b == bounds[i]:
            mark = "   <- snaps here"
        else:
            mark = ""
        print(f"  {b}{mark}")
    if exact:
        print(f"\n{args.near} is a legal break icount.")
    elif i == len(bounds):
        print(
            f"\n{args.near} is past the end of the recording (last boundary "
            f"{bounds[-1]}) — a break there would never fire."
        )
        return 1
    else:
        prev = bounds[i - 1] if i else 0
        print(
            f"\n{args.near} is NOT legal: it sits {args.near - prev} instructions into a "
            f"{bounds[i] - prev}-instruction chunk, leaving {bounds[i] - args.near}. "
            f"Breaking there aborts the replay. Use {bounds[i]}."
        )
    return 0


def cmd_wedge(args, repo: Path) -> int:
    """Name the recorded events around where a stalled replay stopped reading.

    The wedge is the open rr limitation: a replay stops consuming the trace about
    two thirds through, at a near-deterministic offset, while burning CPU. The
    offset has been measured; which recorded event sits there has not, and that
    is the next thing to know.

    Reports a WINDOW, never a culprit. `trace_read_position` is the kernel file
    offset of a buffered read, so QEMU has pulled bytes it has not yet acted on:
    the stall is at or BEFORE the offset. Printing "the replay died here" from
    that would be the handle's reading dressed up as the thing's.
    """
    trace = _resolve_trace(args)
    offset = args.at
    live = False
    if offset is None:
        offset = rr.trace_read_position(trace)
        live = True
        if offset is None:
            print(
                f"nobody has {trace}/replay.bin open, so there is no read position to "
                "look at. Pass --at <offset> to inspect a position recorded earlier.",
                file=sys.stderr,
            )
            return 1

    size = (trace / "replay.bin").stat().st_size
    before, after = rr.events_around_offset(trace, offset, args.count, args.count)
    pct = 100.0 * offset / size if size else 0.0
    src = "live reader" if live else "--at"
    print(f"trace:  {trace}")
    print(f"offset: {offset} of {size} ({pct:.2f}%, from {src})")
    if not before and not after:
        print("\nno events decode around that offset")
        return 1
    print("\nBEFORE (the stall is at or before one of these):")
    for ev in before:
        print(f"  @{ev.offset:<10} icount {ev.icount:<14} {ev.name}")
    print("\nAFTER (context; not yet acted on):")
    for ev in after:
        print(f"  @{ev.offset:<10} icount {ev.icount:<14} {ev.name}")
    print(
        "\nThe offset is a BUFFERED kernel read position, so it runs ahead of what\n"
        "the replay has acted on. Treat BEFORE as the evidence."
    )
    return 0


def rr_cmd(args, repo: Path) -> int:
    verb = args.rr_verb
    fn = {
        "record": cmd_record,
        "replay": cmd_replay,
        "list": cmd_list,
        "info": cmd_info,
        "breaks": cmd_breaks,
        "wedge": cmd_wedge,
        "verify": cmd_verify,
        "prune": cmd_prune,
    }.get(verb)
    if fn is None:
        print(f"unknown rr verb: {verb}", file=sys.stderr)
        return 2
    try:
        return fn(args, repo)
    except rr.RecordReplayError as exc:
        print(f"[rr] {exc}", file=sys.stderr)
        return 2
