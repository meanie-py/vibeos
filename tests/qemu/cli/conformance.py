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


def _run_dir(arch: str) -> Path:
    ts = time.strftime("%Y%m%d-%H%M%S")
    d = Path("/tmp/vibeos-cli") / f"{ts}-conformance-{arch}"
    d.mkdir(parents=True, exist_ok=True)
    return d


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
        if smp:
            # ANY --smp N (including 1) means the ntkrnlmp media: at N=1 that is
            # the whole experiment — "the MP kernel on one CPU" — and without
            # --smp-kernel it would silently sweep the UP kernel instead and
            # report a meaningless all-green.
            build_cmd.append("--smp-kernel")
        print(
            f"[conformance] building {base} with VIBEOS_PAYLOAD={SUITE}"
            f"{' --smp-kernel' if smp else ''} ..."
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
    results, done_line, bugcheck = parser.parse(log_path.read_text(errors="replace"))

    lines = [parser.fmt(r) for r in results]
    ok = sum(1 for r in results if r["status"] == "OK")
    bad = sum(1 for r in results if r["status"] in parser.BAD)
    results_path = run_dir / f"{SUITE}-results.txt"
    results_path.write_text("\n".join(lines) + "\n")

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
        "serial_log": str(log_path),
    }
    (run_dir / "summary.json").write_text(json.dumps(summary, indent=2))

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
