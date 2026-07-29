"""`vibeos-cli conformance` — the NT test-corpus ratchet front door (quest M4).

One verb: (optionally) build a kmtest-payload image, boot it, run the full
in-guest kmtest sweep, parse the serial log into per-test results, and compare
against the committed per-arch baseline
(`tools/conformance-baseline/<arch>/kmtest.txt`).

Report-only by default (the quest keeps it report-only until a baseline is
stable across >=3 runs); `--gate` makes a ratchet regression exit nonzero so it
can be wired as an opt-in gate stage. `--write-baseline` stores the current
results as the new floor.

The sweep mechanics live in the payload harness: media built with
`VIBEOS_PAYLOAD=kmtest` grafts `kmtest-sweep.exe` into the All Users Startup
folder + `kmtest-tests.txt` next to the binaries; the sweep runs one
`kmtest_.exe` per listed test and tees results to serial. `payload.yaml` sets
`launch_marker: "[KMTEST-SWEEP] done"` + `launch_drain_secs`, so
`test_payload.py::test_payload_launched` holds the boot open and drains serial
until the sweep completes (or BUGCHECKs). We then parse
`qemu/<pytest_arch>_smoke_live.log`.

Design: docs/engineering/conformance-ratchet-quest.md (M3/M4).
"""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import time
from pathlib import Path

SUITE = "kmtest"
# base arch -> pytest arch (x86 boot the UEFI configs; arm64 is itself).
_PYTEST_ARCH = {"i386": "i386-uefi", "amd64": "amd64-uefi"}


def _load_parser(repo: Path):
    """Import tools/conformance/parse_kmtest_serial.py as a module."""
    path = repo / "tools" / "conformance" / "parse_kmtest_serial.py"
    spec = importlib.util.spec_from_file_location("parse_kmtest_serial", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load parser at {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def baseline_coverage_refusal(results, base_map, *, force: bool) -> str | None:
    """Refuse a `--write-baseline` that would DELETE tests from the ratchet floor.

    A baseline is a floor, and `--write-baseline` replaces it wholesale with this
    run's rows. So a sweep that launched fewer tests than the floor — a boot flake,
    an early BUGCHECK, a drained-out timeout — silently removes the missing tests
    from the floor, permanently, with no signal anywhere. That is the ratchet form
    of "never weaken a test to make it pass", and catching it was a hand-check
    before every re-seed.

    Returns the refusal message, or None to proceed.
    """
    new_names = {r["name"] for r in results}
    dropped = sorted(n for n in base_map if n not in new_names)
    if not dropped or force:
        return None
    return (
        f"[conformance] --write-baseline REFUSED: this sweep launched "
        f"{len(results)} tests but the existing baseline has {len(base_map)}, and "
        f"writing it would DELETE {len(dropped)} test(s) from the ratchet floor:\n"
        + "".join(f"    {n}\n" for n in dropped)
        + "  A short sweep is usually a boot flake or an early BUGCHECK, not a real "
        "coverage change — re-run it.\n"
        "  If the removal is intended (a test was retired), pass --force-baseline."
    )


def baseline_lowering_note(results, base_map) -> str | None:
    """Say out loud which rows a `--write-baseline` is about to make worse.

    Unlike coverage loss this is NOT refused: absorbing a known regression into the
    floor is sometimes deliberate (i386 and arm64 both accepted ExHardError
    30/0 -> 24/6 on purpose). It just must never happen silently.
    """
    worse = [
        f"    {r['name']}: pass {b.get('pass')} -> {r.get('pass')}, "
        f"fail {b.get('fail')} -> {r.get('fail')}"
        for r in results
        if (b := base_map.get(r["name"]))
        and (r.get("fail", 0) > b.get("fail", 0) or r.get("pass", 0) < b.get("pass", 0))
    ]
    if not worse:
        return None
    return (
        "[conformance] NOTE: --write-baseline is LOWERING the floor for "
        f"{len(worse)} test(s) — accepting these as the new normal:\n" + "\n".join(worse)
    )


def _run_dir(arch: str) -> Path:
    ts = time.strftime("%Y%m%d-%H%M%S")
    d = Path("/tmp/vibeos-cli") / f"{ts}-conformance-{arch}"
    d.mkdir(parents=True, exist_ok=True)
    return d


# Durable soak ledger, alongside the flake farm's corpus so one triage pass can
# read both. /tmp/vibeos-cli run dirs rotate and a sweep truncates the previous
# same-arch serial log, so a multi-hour soak that keeps its findings only there
# loses them. The 2026-07-26 arm64 soak had to be partly RETRACTED because
# nobody could reconstruct, after the fact, which media each round ran on —
# hence `sha`/`dirty`/`smp`/`tripwire` in every record, not just the verdict.
FARM_DIR = Path.home() / "vibeos-flake-farm"
LEDGER = FARM_DIR / "conformance.jsonl"
_ROUNDS: list[dict] = []


def _head_sha(repo: Path) -> str:
    try:
        r = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            timeout=30,
        )
        return r.stdout.strip() if r.returncode == 0 else "?"
    except Exception:
        return "?"


def _append_round(rec: dict) -> None:
    _ROUNDS.append(rec)
    try:
        FARM_DIR.mkdir(parents=True, exist_ok=True)
        with LEDGER.open("a") as f:
            f.write(json.dumps(rec) + "\n")
    except Exception as e:  # a soak must never die on its own bookkeeping
        print(f"[conformance] ledger append failed: {e}", file=sys.stderr)


def _rostests_enabled(repo: Path, base_arch: str) -> bool | None:
    """True if the build dir will compile kmtests. modules/CMakeLists.txt gates
    them on `ENABLE_ROSTESTS OR ENABLE_ROSTESTS_KMTESTS`, and the fleet default
    is the KMTESTS-only path (ROSTESTS=OFF, KMTESTS=ON — kmtests without the full
    suite), so both keys must be OR'd, not first-match. None if no cache yet."""
    cache = repo / f"build-{base_arch}" / "CMakeCache.txt"
    if not cache.is_file():
        return None
    on, saw = False, False
    for line in cache.read_text(errors="replace").splitlines():
        for key in ("ENABLE_ROSTESTS:", "ENABLE_ROSTESTS_KMTESTS:"):
            if line.startswith(key):
                saw = True
                if line.strip().endswith("=ON"):
                    on = True
    return on if saw else False


def conformance(args, repo: Path) -> int:
    """`--repeat N` soak wrapper around one sweep.

    The arm64 flip blocker's five faces are all SWEEP faces — a bugcheck partway
    through, a sweep that launches nothing, a sweep that stalls — so `farm`,
    which only boots to desktop, cannot see any of them, and the soak that found
    them was hand-looped. Rounds after the first force `--no-build`: the media is
    fixed for the run, which is what makes rounds comparable.
    """
    repeat = int(getattr(args, "repeat", 0) or 0)
    if repeat <= 1:
        return _conformance_once(args, repo)

    worst = 0
    for i in range(repeat):
        if i:
            args.no_build = True
        print(f"\n=== conformance round {i + 1}/{repeat} ({args.arch}) ===", flush=True)
        try:
            rc = _conformance_once(args, repo)
        except KeyboardInterrupt:
            print(f"[conformance] interrupted after {i} round(s)", file=sys.stderr)
            break
        except Exception as e:  # one bad round must not end the soak
            print(f"[conformance] round {i + 1} raised {e!r} — continuing", file=sys.stderr)
            continue
        worst = max(worst, rc)

    # Roll-up. Deliberately counts the SHAPES that distinguish the faces
    # (bugcheck / launched=0 / stalled / clean) rather than just pass-fail: a
    # bare failure count is what made nine rounds read as one bug.
    print(f"\n=== soak roll-up: {len(_ROUNDS)} round(s) -> {LEDGER} ===")
    shapes: dict[str, int] = {}
    for r in _ROUNDS:
        shapes[r["shape"]] = shapes.get(r["shape"], 0) + 1
    for k, v in sorted(shapes.items(), key=lambda kv: -kv[1]):
        print(f"  {v:>3}  {k}")
    return worst


def _conformance_once(args, repo: Path) -> int:
    raw = args.arch
    base = raw.split("-")[0]  # i386 / amd64 / arm64 (baseline dir + build --arch)
    pytest_arch = _PYTEST_ARCH.get(raw, raw)  # amd64 -> amd64-uefi; passthrough if suffixed
    smp = args.smp or 0

    if smp == 1 and getattr(args, "write_baseline", False):
        # `--smp 1` ratchets against the UP baseline on purpose (below); letting it
        # WRITE that file would overwrite the UP oracle with MP results and quietly
        # destroy the only thing the collapse experiment is measured against.
        print(
            "[conformance] --smp 1 --write-baseline refused: the UP baseline must be "
            f"produced by the UP kernel.\n  seed it with `--arch {base} --write-baseline` "
            "(no --smp), or use --smp 2 for an MP-specific baseline.",
            file=sys.stderr,
        )
        return 2

    parser = _load_parser(repo)
    run_dir = _run_dir(base)
    # SMP runs ratchet against their own baseline (kmtest-smp<N>.txt): MP kernels
    # legitimately diverge from UP (tests branch on KmtIsMultiProcessorBuild,
    # skip MP-fatal sections, expect SYNCH_LEVEL=IPI-2), so comparing an -smp
    # sweep against the UP baseline reports phantom regressions.
    #
    # `--smp 1` is the EXCEPTION and the point of the UP→smp1 collapse experiment
    # (smp-bringup-quest.md "UP endgame"): MP kernel, ONE cpu, ratcheted against
    # the UP baseline. Those two must agree test-for-test before the MP kernel can
    # become the default, so the UP baseline is deliberately the right oracle here.
    baseline_name = f"{SUITE}-smp{smp}.txt" if smp > 1 else f"{SUITE}.txt"
    baseline_path = repo / "tools" / "conformance-baseline" / base / baseline_name

    # --- 1. build (default on) with the kmtest payload -----------------------
    if not args.no_build:
        rost = _rostests_enabled(repo, base)
        if rost is False:
            # Fail loud, no silent degradation: a payload build without the test
            # binaries would boot and then [KMTEST-SWEEP] launch FAILED. Point at
            # the one-time (cached) configure instead.
            print(
                f"[conformance] build-{base} has neither ENABLE_ROSTESTS nor "
                f"ENABLE_ROSTESTS_KMTESTS — the kmtest binaries won't be staged.\n"
                f"  fix (one-time, cmake-cached; KMTESTS-only is the fleet norm — full "
                f"ENABLE_ROSTESTS drags every apitest/winetest into the livecd graph):\n"
                f"    cd build-{base} && bash ../tools/canon-run.sh cmake "
                f"-DENABLE_ROSTESTS_KMTESTS=ON .\n"
                f"  then re-run this verb (or pass --no-build if media is already staged).",
                file=sys.stderr,
            )
            return 2
        env = dict(os.environ, VIBEOS_PAYLOAD=SUITE)
        build_cmd = [str(repo / "vibeos-cli"), "build", "--arch", base]
        # ANY --smp N (including 1) must sweep the ntkrnlmp media: at N=1 that is
        # the whole experiment — "the MP kernel on one CPU" — and sweeping the UP
        # kernel instead would report a meaningless all-green.
        #
        # On x86 that costs NOTHING now: the ISO already carries both kernels and
        # both HALs plus a [LiveCD_SMP] entry, and vm.smp_selected_iso patches
        # `DefaultOS=` inside a COPY of the media at boot time. --smp-kernel would
        # only rewrite that same line via a cmake cache var — i.e. a reconfigure
        # and an ISO re-roll to change 22 bytes we can change for free.
        #
        # arm64 still needs it: its boot.img is a hand-curated ramdisk that does
        # not carry halmp.dll/ntkrnlmp.exe at all unless mkramdisk.sh --smp stages
        # them, so there is nothing in the media to boot-select.
        needs_smp_media_build = smp and base == "arm64"
        if needs_smp_media_build:
            build_cmd.append("--smp-kernel")
        print(
            f"[conformance] building {base} with VIBEOS_PAYLOAD={SUITE}"
            f"{' --smp-kernel' if needs_smp_media_build else ''} ..."
        )
        bp = subprocess.run(build_cmd, cwd=str(repo), env=env)
        if bp.returncode != 0:
            print("[conformance] BUILD FAILED — aborting.", file=sys.stderr)
            return bp.returncode

    # --- 2. run the in-guest sweep -------------------------------------------
    # test_payload_launched drains serial up to launch_drain_secs waiting for
    # [KMTEST-SWEEP] done. Give the subprocess a margin over the drain budget.
    test_file = repo / "tests" / "qemu" / "test_payload.py"
    env = dict(os.environ)
    if smp:
        env["VIBEOS_QEMU_SMP"] = str(smp)
    sweep_cmd = [
        sys.executable,
        "-m",
        "pytest",
        str(test_file),
        "--arch",
        pytest_arch,
        "--payload",
        SUITE,
        "--skip-clean-build",  # we just built (or --no-build); don't nuke
        "--timeout",
        str(args.timeout),
        "-q",
        "-p",
        "no:cacheprovider",
        "-k",
        "launched",  # only the launch+drain test
    ]
    print(
        f"[conformance] {SUITE} sweep on {pytest_arch}"
        f"{f' -smp {smp}' if smp else ''} — draining up to {args.timeout}s ..."
    )
    t0 = time.time()
    sweep_rc: int | None = None
    try:
        sp = subprocess.run(sweep_cmd, cwd=str(repo), env=env, timeout=args.timeout + 300)
        sweep_rc = sp.returncode
    except subprocess.TimeoutExpired:
        print(
            f"[conformance] sweep subprocess exceeded {args.timeout + 300}s — "
            f"parsing whatever serial was captured.",
            file=sys.stderr,
        )
    elapsed = time.time() - t0

    # --- 3. parse the serial log ---------------------------------------------
    log_path = repo / "qemu" / f"{pytest_arch}_smoke_live.log"
    if not log_path.is_file():
        print(f"[conformance] no serial log at {log_path} — did the guest boot?", file=sys.stderr)
        return 2
    serial_text = log_path.read_text(errors="replace")
    results, done_line, bugcheck = parser.parse(serial_text)

    lines = [parser.fmt(r) for r in results]
    ok = sum(1 for r in results if r["status"] == "OK")
    bad = sum(1 for r in results if r["status"] in parser.BAD)
    results_path = run_dir / f"{SUITE}-results.txt"
    results_path.write_text("\n".join(lines) + "\n")
    # SAVE the serial into the run dir, and report the COPY. qemu/<arch>_smoke_live.log
    # is the LIVE per-arch file: the next sweep of the same arch truncates it, so a
    # summary.json pointing there hands you the *next* run's serial — or nothing.
    # This destroyed the evidence for an arm64 -smp 1 BUGCHECK 0x1E that a following
    # -smp 2 sweep overwrote seconds later, leaving only the stop code. The results
    # file alone is not enough: a mid-sweep crash needs the faulting context, and the
    # crash is exactly the case where you cannot just re-run and expect it back.
    # Found independently a second time on the conformance-ratchet lane: an
    # IoFilesystem BUGCHECK 0x19 printed a full [POOL] page dump + free ring, and a
    # later boot of that arch had overwritten every byte before anyone read the run
    # dir — the crash had to be re-collected with a fresh build+boot. Two lanes
    # paying for the same lost evidence is what makes this worth a comment.
    saved_serial = run_dir / f"{pytest_arch}_serial.log"
    saved_serial.write_text(serial_text)

    # An `--smp N` sweep that quietly swept the UP kernel is the false green this
    # whole experiment exists to avoid, and it is INVISIBLE in the results — a UP
    # sweep looks like a very good MP sweep. `smoke --smp N` asserts this
    # (_smp_check), but conformance boots via pytest and never went through that
    # path, so it had no such check at all. freeldr announces both, so this is an
    # assertion on evidence we already captured, not new instrumentation.
    if smp:
        from .smoke import _booted_hal, _booted_kernel

        booted_k, booted_h = _booted_kernel(serial_text), _booted_hal(serial_text)
        wrong = [
            f"{what}={got}"
            for what, got, want in (
                ("kernel", booted_k, "ntkrnlmp.exe"),
                ("HAL", booted_h, "halmp.dll"),
            )
            if got and got != want
        ]
        if wrong:
            print(
                f"[conformance] --smp {smp} did NOT get the MP pair: {', '.join(wrong)}.\n"
                f"  This sweep graded the UP kernel and its numbers mean nothing for the\n"
                f"  collapse experiment. Serial: {saved_serial}",
                file=sys.stderr,
            )
            return 2
        print(f"[conformance] MP pair confirmed on serial: {booted_k} + {booted_h}")

    # --- 4. compare against baseline -----------------------------------------
    regressions: list[str] = []
    new_passes: list[str] = []
    have_baseline = baseline_path.is_file()
    if have_baseline:
        base_map = parser.load_baseline(baseline_path)
        regressions = parser.compare(results, base_map)
        for r in results:
            b = base_map.get(r["name"])
            if r["status"] == "OK" and (b is None or b["status"] in parser.BAD):
                new_passes.append(r["name"])

    # --- 5. write baseline if asked ------------------------------------------
    if args.write_baseline:
        # A baseline is a RATCHET FLOOR, and --write-baseline replaces it wholesale
        # with this run's rows. So a sweep that launched fewer tests than the floor
        # — a boot flake, an early BUGCHECK, a drained-out timeout — silently
        # DELETES the missing tests from the floor, permanently, with no signal.
        # That is the ratchet form of "never weaken a test to make it pass", and it
        # is the exact hand-check that had to be done before every re-seed. Refuse
        # instead, and name the tests that would vanish.
        #
        # Only COVERAGE LOSS is refused. A row whose numbers got worse is a
        # judgement call the operator can legitimately accept (i386 and arm64 both
        # deliberately absorbed ExHardError 30/0 -> 24/6), so those are printed
        # loudly and allowed.
        if have_baseline:
            refusal = baseline_coverage_refusal(
                results, base_map, force=getattr(args, "force_baseline", False)
            )
            if refusal:
                print(refusal, file=sys.stderr)
                return 2
            note = baseline_lowering_note(results, base_map)
            if note:
                print(note)
        baseline_path.parent.mkdir(parents=True, exist_ok=True)
        smp_arg = f" --smp {smp}" if smp > 1 else ""
        baseline_path.write_text(
            "# kmtest conformance baseline — parse_kmtest_serial.py format\n"
            f"# arch={base}{f' smp={smp}' if smp > 1 else ''}  generated by"
            f" `vibeos-cli conformance --arch {base}{smp_arg} --write-baseline`\n"
            + "\n".join(lines)
            + "\n"
        )
        print(f"[conformance] baseline written: {baseline_path}")

    # --- 6. summary.json ------------------------------------------------------
    summary = {
        "suite": SUITE,
        "arch": base,
        "pytest_arch": pytest_arch,
        "smp": smp or 1,
        "launched": len(results),
        "ok": ok,
        "bad": bad,
        "sweep_done": done_line,
        "bugcheck": bugcheck,
        "sweep_rc": sweep_rc,
        "elapsed_s": round(elapsed, 1),
        "have_baseline": have_baseline,
        "regressions": regressions,
        "new_passes": new_passes,
        "results_file": str(results_path),
        "serial_log": str(saved_serial),
        "serial_log_live": str(log_path),  # truncated by the next same-arch sweep
    }
    (run_dir / "summary.json").write_text(json.dumps(summary, indent=2))

    # --- 6b. durable soak ledger ---------------------------------------------
    # `shape` is the face discriminator, computed from the fields that actually
    # separate them. Note launched==0 vs a stall vs a bugcheck all render the
    # same in a pass/fail view, which is precisely how five distinct bugs spent
    # months filed under one heading.
    serial_text = ""
    try:
        serial_text = Path(summary["serial_log"]).read_text(errors="replace")
    except Exception:
        pass
    if bugcheck:
        shape = f"A:bugcheck {bugcheck}"
    elif not results:
        shape = "B:no-launch"
    elif not done_line:
        shape = "stalled/DID-NOT-FINISH"
    elif regressions:
        shape = "clean-but-ratchet-regression"
    else:
        shape = "clean"
    try:
        from . import triage as _triage

        ev = _triage.wedge_evidence(serial_text)
    except Exception:
        ev = {"census_stuck": 0, "lockdep": 0, "cswitch": 0, "lines": []}
    rec = dict(summary)
    rec.update(
        {
            "ts": time.strftime("%Y%m%d-%H%M%S"),
            "shape": shape,
            "sha": _head_sha(repo),
            "tripwire": os.environ.get("VIBEOS_TRIPWIRE", ""),
            "run_dir": str(run_dir),
            "census_stuck": ev["census_stuck"],
            "lockdep_hits": ev["lockdep"],
            "cswitch_hits": ev["cswitch"],
            "census_lines": ev["lines"],
            # The widened census prints a wait-reason histogram when the parked
            # profile shifts; its presence is what turns face D from invisible
            # into named, so record whether it fired at all.
            "wait_profile": "[CENSUS] wait profile:" in serial_text,
            "rust_panics": serial_text.count("*** RUST PANIC"),
        }
    )
    _append_round(rec)

    # --- 7. report + exit code -----------------------------------------------
    verdict = "done" if done_line else "DID NOT FINISH"
    print(
        f"\nconformance {SUITE} {base}"
        f"{f' -smp {smp}' if smp else ''}  —  "
        f"launched={len(results)} ok={ok} bad={bad} sweep={verdict}"
        f"{' BUGCHECK=' + bugcheck if bugcheck else ''}  ({elapsed:.0f}s)"
    )
    print(f"  results:  {results_path}")
    print(f"  summary:  {run_dir / 'summary.json'}")
    if bugcheck or bad:
        print(f"  serial:   {saved_serial}")

    if not have_baseline:
        print(f"  no baseline at {baseline_path} — pass --write-baseline to seed it.")
        return 0
    if new_passes:
        print(f"  NEW PASSES ({len(new_passes)}): {', '.join(new_passes)}")
    if regressions:
        print(f"  RATCHET REGRESSIONS ({len(regressions)}):")
        for reg in regressions:
            print(f"    {reg}")
        if args.gate:
            print("  === CONFORMANCE GATE RED ===")
            return 1
        print("  (report-only; pass --gate to fail the command on regressions)")
        return 0
    print("  ratchet holds: no regressions vs baseline.")
    return 0
