"""`vibeos-cli rr ...` — record a boot, then replay it deterministically.

    rr record --arch arm64 --payload kmtest     # boot once, keep the recording
    rr prune --keep 3 [--apply]                  # traces are big; nothing else reclaims them
    rr list                                      # what traces exist
    rr info <trace>                              # what it is, and will it still replay
    rr replay <trace> [--gdb]                    # run that exact boot again
    rr verify <trace>                            # replay and diff serial vs the record
    rr breaks <trace> [--near N]                 # icounts a replay can legally stop at
    rr record ... --sample 10 ; rr diverge <trace>   # first icount replay != record

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
import subprocess
import sys
import threading
import time
from pathlib import Path

from tools import qemu_infra_compat as native

from . import recordreplay as rr  # re-exported for the arg parser
from . import rrsample


def _native_call(operation, *, callbacks=None, **kwargs):
    try:
        if callbacks is not None:
            with native.DedicatedSession() as session:
                return session.call("rr_cli_" + operation, kwargs, callbacks=callbacks)
        return native.call("rr_cli_" + operation, kwargs)
    except native.NativeError as error:
        raise rr.RecordReplayError(str(error)) from error


# Traces are expensive (a full boot each) and worth keeping across reboots, so
# they do NOT live in /tmp.
DEFAULT_ROOT = Path(os.environ.get("VIBEOS_RR_ROOT", Path.home() / "vibeos-rr"))

# vibeos knobs that change the device set or the guest's behaviour. A replay
# must run with the same values or the recorded input stream no longer lines up
# with the devices asking for it, so we persist them with the trace.
_PINNED_ENV = (
    "VIBEOS_QEMU_SMP",
    "VIBEOS_QEMU_GIC",
    "VIBEOS_QEMU_SMMU",
    "VIBEOS_QEMU_USB_DISK",
    "VIBEOS_QEMU_NVME",
    "VIBEOS_QEMU_NVME_BRIDGE",
    "VIBEOS_QEMU_XHCI",
    "VIBEOS_QEMU_XHCI_MSI",
    "VIBEOS_QEMU_CPU",
    "VIBEOS_QEMU_HEADLESS",
    "VIBEOS_QEMU_BOOT_NVME",
    "VIBEOS_QEMU_EXTRA_ARGS",
    "VIBEOS_TRIPWIRE",
    "VIBEOS_UP_DEFAULT_BOOT",
    # x86 records over usb-storage: blkreplay refuses the ATAPI ide-cd the default boot
    # uses, and the recorded device set must be the replayed one.
    "VIBEOS_QEMU_BOOT_TRANSPORT",
)


def _root(args) -> Path:
    return Path(getattr(args, "root", None) or DEFAULT_ROOT)


def _resolve_trace(args) -> Path:
    return Path(_native_call("resolve_trace", trace_root=_root(args), raw=str(args.trace)))


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
    # --no-net: amd64's smoke runs a SECOND boot for its net suite. Under the rr env that
    # boot re-staged fresh overlays over the trace's media after the recording ended,
    # wiping the `init` snapshot (replay: "Snapshot 'init' does not exist"). Recording a
    # network is unsupported anyway (recordreplay.py), and arm64 has no net stage, which
    # is why no arm64 trace was ever hit.
    return [cli, "smoke", "--arch", arch, "--smp", str(smp), "--no-net"]


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
    if getattr(args, "sample", None):
        # Fingerprint the guest every N seconds so `rr diverge` has a recorded
        # half to compare against. See rrsample.py.
        env[rrsample.SAMPLE_ENV_VAR] = str(args.sample)
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
    if size == 0 and "argv" not in meta:
        # vm.py writes the argv the moment it launches QEMU; without it the boot
        # command refused before any VM existed (stale media, a --no-build
        # mismatch), and "QEMU was killed" would send the reader the wrong way.
        print(
            f"[rr] NO TRACE WRITTEN — the boot command exited rc={rc} without ever "
            "launching QEMU. Its own message above says why.",
            file=sys.stderr,
        )
        return 1
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
    """Refuse finalization unless the native monotonic wait observes a stable trace."""
    return _native_call("await_quiescence", trace_dir=trace, settle=settle, limit=limit)


def _keep_serial(trace: Path, name: str, *, expect_mode: str) -> Path | None:
    result = _native_call("keep_serial", trace_dir=trace, name=name, expect_mode=expect_mode)
    if result.get("warning"):
        print(result["warning"], file=sys.stderr)
    return Path(result["path"]) if result["path"] else None


def _guest_lines(p: Path) -> list[str]:
    return _native_call("guest_lines", path=p)


def _serial_divergence(a: list[str], b: list[str]) -> tuple[int, str] | None:
    result = _native_call("serial_divergence", record=a, replay=b)
    return tuple(result) if result is not None else None


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
    """Thread/DTO adapter for the native trace progress watcher."""

    def __init__(self, trace: Path, period: float = 20.0) -> None:
        self.trace, self.period = trace, period
        self.total = (trace / "replay.bin").stat().st_size
        self.max_pos, self.stalled_for = 0, 0.0
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self._error: BaseException | None = None

    def _run(self) -> None:
        try:
            result = _native_call(
                "watch_progress",
                trace_dir=self.trace,
                period=self.period,
                callbacks={"rr_cli_alive": lambda _: not self._stop.is_set()},
            )
            self.total, self.max_pos, self.stalled_for = (
                result["total"],
                result["max_pos"],
                result["stalled_for"],
            )
        except BaseException as error:
            self._error = error

    def __enter__(self) -> _TraceProgress:
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *exc) -> None:
        self._stop.set()
        if self._thread:
            self._thread.join(timeout=10)
            if self._thread.is_alive():
                raise rr.RecordReplayError("native trace progress watcher did not stop")
        if self._error and exc[0] is None:
            raise self._error

    def report(self) -> str | None:
        return _native_call(
            "progress_report",
            state=dict(total=self.total, max_pos=self.max_pos, stalled_for=self.stalled_for),
        )


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


def cmd_diverge(args, repo: Path) -> int:
    """Replay with a break at every recorded fingerprint; report the first mismatch.

    The wedge is where a divergence becomes VISIBLE. This finds where it HAPPENED,
    to the resolution the recording was sampled at. Needs `rr record --sample`.
    """
    trace = _resolve_trace(args)
    samples = trace / rrsample.SAMPLES_FILE
    if not samples.is_file() or not samples.read_text().strip():
        print(
            f"[rr] {trace.name} has no fingerprints ({rrsample.SAMPLES_FILE}); there is "
            "nothing recorded to compare a replay against. Re-record it with "
            "`rr record ... --sample 10`.",
            file=sys.stderr,
        )
        return 2
    verdict_path = trace / rrsample.VERDICT_FILE
    verdict_path.unlink(missing_ok=True)
    os.environ[rrsample.SAMPLE_ENV_VAR] = "check"
    args.gdb = False
    args.break_at = None
    rc = cmd_replay(args, repo)
    if not verdict_path.is_file():
        print(f"[rr] diverge: no verdict written (replay rc={rc})", file=sys.stderr)
        return 1
    v = json.loads(verdict_path.read_text())
    print(f"[rr] diverge: compared {v['compared']} of {v['samples']} fingerprints")
    for sk in v["skipped"]:
        print(f"[rr]   skipped icount {sk['icount']}: {sk['why']}")
    for ic in v.get("inconclusive", []):
        print(f"[rr]   inconclusive icount {ic['icount']}: {ic['why']} ({ic['mem_mib']})")
    print(f"[rr]   last match:     {v['last_match']}")
    mm = v.get("first_mismatch")
    if mm:
        print(f"[rr]   FIRST MISMATCH: icount {mm['icount']}")
        for line in mm.get("regs", [])[:40]:
            print(f"[rr]     {line}")
        mib = mm.get("mem_mib", [])
        if mib:
            print(f"[rr]     RAM differs in {len(mib)} MiB: {' '.join(mib[:32])}")
        for chunk in mm.get("mem_bytes", [])[:4]:
            if "error" in chunk:
                print(f"[rr]     {chunk['base']}: {chunk['error']}")
                continue
            print(f"[rr]     {chunk['base']}: {chunk['bytes_differing']} bytes differ")
            for run in chunk["runs"][:3]:
                print(f"[rr]       {run['addr']} +{run['len']}  (dump from {run['at']})")
                print(f"[rr]         record {run['record']}")
                print(f"[rr]         replay {run['replay']}")
        print(f"[rr]   full detail: {verdict_path}")
        return 1
    if v.get("error"):
        print(f"[rr]   ERROR: {v['error']}")
        for line in (v.get("wedge_regs") or "").splitlines()[:4]:
            print(f"[rr]     {line.strip()}")
        return 1
    print("[rr]   every compared fingerprint matched")
    return 0


def cmd_list(args, repo: Path) -> int:
    root = _root(args)
    rows = _native_call("inventory", trace_root=root)
    if not rows:
        print(f"no traces under {root}")
        return 0
    print(f"{'trace':<40} {'arch':<6} {'payload':<8} {'rc':<4} {'rr':<6} {'total':<7} created")
    for row in rows:
        m = row["manifest"]
        print(
            f"{row['name']:<40} {m.get('arch', '?'):<6} {m.get('payload', '?'):<8} "
            f"{m.get('boot_rc', '?')!s:<4} {row['bytes'] / 1e6:<6.0f} "
            f"{row['total'] / 1e6:<7.0f} {m.get('created', '?')}"
        )
    return 0


def cmd_prune(args, repo: Path) -> int:
    result = _native_call(
        "prune",
        trace_root=_root(args),
        keep=max(0, int(args.keep)),
        names=args.name or [],
        apply=args.apply,
    )
    if not result["doomed"]:
        print(f"{result['count']} trace(s), keeping {result['keep']} — nothing to prune")
        return 0
    for row in result["doomed"]:
        print(
            f"{'DELETE' if args.apply else 'would delete'} {row['name']:<34} "
            f"{row['total'] / 1e6:8.0f}MB"
        )
    verb = "freed" if args.apply else "would free"
    print(f"{verb} {result['bytes'] / 1e9:.2f}GB across {len(result['doomed'])} trace(s)")
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
    src = "--at"
    icount = getattr(args, "icount", None)
    if icount is not None:
        # Exact, unlike a read offset: the first event at or past the instruction count
        # QEMU reports for a stalled replay (`info replay` on a monitor).
        offset = next((ev.offset for ev in rr.walk_events(trace) if ev.icount >= icount), None)
        if offset is None:
            print(f"icount {icount} is past the last event in {trace}", file=sys.stderr)
            return 1
        src = f"--icount {icount}"
    if offset is None:
        offset = rr.trace_read_position(trace)
        if offset is None:
            print(
                f"nobody has {trace}/replay.bin open, so there is no read position to "
                "look at. Pass --at <offset> to inspect a position recorded earlier.",
                file=sys.stderr,
            )
            return 1
        src = "live reader"

    size = (trace / "replay.bin").stat().st_size
    before, after = rr.events_around_offset(trace, offset, args.count, args.count)
    pct = 100.0 * offset / size if size else 0.0
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
        "diverge": cmd_diverge,
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
