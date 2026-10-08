#!/usr/bin/env python3
"""Probe QUARANTINED kmtests to find the ones a later fix already un-broke.

WHY THIS EXISTS. Quarantine entries are written on the day a test crashes and
then never revisited, because re-testing them by hand is tedious and a crasher
truncates the whole sweep. So fixes land, and the tests those fixes repaired stay
quarantined for weeks. `eb43b45e35b` (arm64 RtlStack un-quarantined itself once
the `save_next` unwind decoder was fixed) was found by accident; this tool looks
on purpose.

THE CONSTRAINT that shapes the design: the in-guest sweep tees results to serial
and the harness drains until `[KMTEST-SWEEP] done` **or a BUGCHECK**. A test that
bugchecks the kernel therefore ends the run, and every test after it in the list
produces no verdict at all. Enabling all 21 quarantined tests in one boot teaches
you about exactly one of them.

A ruinous test spoils the boot in one of TWO ways, and they need opposite
handling:

  * it CRASHES — the boot ends, the tests after it get no verdict, and the probe
    re-queues them. Silence is the honest outcome here.
  * it WEDGES IN-KERNEL (TIMEOUT) — the boot does NOT end. Draining continues,
    but the wedged process never releases kmtest_drv's registered result buffer,
    so every later test cascade-fails SET_RESULTBUFFER 0x5 and records a NO-END
    verdict about itself that is really a fact about the wedge.

The second is the dangerous one, because it produces confident-looking verdicts
instead of silence. `split_at_wedge` voids them.

THE STRATEGY. Run a probe list containing ONLY the candidates, ordered
most-likely-fixed first. One boot yields verdicts for everything up to the first
crasher. Drop that crasher, re-run. Worst case is one boot per crasher; best case
is one boot total. Candidates known to HANG (rather than crash) are worth putting
last or excluding — a hang burns the full drain budget instead of failing fast.

The committed per-arch list is restored on every exit path, including Ctrl-C.
`git status` is the check: this tool must never leave the tree dirty.

Usage:
    conformance_probe.py --arch arm64 --candidates SeInheritance,ExFastMutex,...
    conformance_probe.py --arch i386 --candidates-from probe-i386.txt
    conformance_probe.py --arch amd64 --auto      # every '#'-commented test
"""

from __future__ import annotations

import argparse
import fcntl
import json
import re
import signal
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LIST = REPO / "tools" / "payloads" / "kmtest" / "kmtest-tests-{arch}.txt"

HEADER = """\
# kmtest sweep list — TEMPORARY PROBE LIST written by tools/conformance_probe.py.
# If you are reading this in a committed file, the probe was interrupted without
# restoring; `git checkout` this path is NOT the fix — see the tool's docstring
# and restore from git history.
"""

# Verdicts that POISON every verdict recorded after them in the same boot.
#
# A CRASH ends the boot outright, so the tests after it simply get no verdict and
# the probe already re-queues them. A TIMEOUT is the nastier case: it is an
# IN-KERNEL WEDGE, the boot keeps draining, and the wedged process never releases
# kmtest_drv's registered result buffer — so every LATER test cascade-fails
# SET_RESULTBUFFER 0x5 and lands a NO-END verdict that says nothing about that
# test. Those are contamination, not evidence.
POISONING = {"TIMEOUT"}


def split_at_wedge(got: list[str], verdicts: dict[str, str]) -> tuple[list[str], list[str]]:
    """Split a boot's verdicts into (trustworthy, contaminated).

    `got` is the tests that received a verdict, in list order. Everything up to
    and including the first POISONING verdict is real; everything after it was
    produced by a kernel already wedged by that test, and is not evidence about
    the tests it names. With no wedge, all of `got` is trustworthy.
    """
    wedge_at = next((i for i, t in enumerate(got) if verdicts[t] in POISONING), None)
    if wedge_at is None:
        return got, []
    return got[: wedge_at + 1], got[wedge_at + 1 :]


def quarantined(arch: str) -> list[str]:
    """Test names that are commented out in the committed per-arch list."""
    out = []
    for line in LIST.with_name(LIST.name.format(arch=arch)).read_text().splitlines():
        m = re.match(r"^#([A-Za-z][A-Za-z0-9_]*)\s*$", line)
        if m:
            out.append(m.group(1))
    return out


def run_sweep(arch: str) -> tuple[dict[str, str], str]:
    """Boot the probe list. Returns {test: status} and the raw verdict string."""
    p = subprocess.run(
        [str(REPO / "vibeos-cli"), "conformance", "--arch", arch],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    blob = (p.stdout or "") + (p.stderr or "")
    m = re.search(r"summary:\s+(\S+summary\.json)", blob)
    if not m:
        # The common cause is a precondition, not a crash: a build dir without
        # ENABLE_ROSTESTS_KMTESTS stages no kmtest binaries. `conformance` says so
        # on stdout and exits — surface ITS message rather than a traceback.
        hint = "\n".join(
            ln for ln in blob.splitlines() if "ROSTESTS" in ln or ln.strip().startswith("cd build-")
        )
        raise RuntimeError(
            "conformance produced no summary.json.\n"
            + (hint or blob[-1500:])
            + "\n(probe made no progress; the committed list is restored)"
        )
    summary = json.loads(Path(m.group(1)).read_text())
    results_path = Path(summary["results_file"])
    verdicts: dict[str, str] = {}
    for line in results_path.read_text().splitlines():
        if line.startswith("#") or not line.strip():
            continue
        parts = line.split()
        if len(parts) >= 2:
            verdicts[parts[0]] = parts[1]
    return verdicts, str(summary.get("sweep") or summary.get("verdict") or "?")


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--arch", required=True, choices=["i386", "amd64", "arm64"])
    ap.add_argument("--candidates", help="comma-separated test names, in probe order")
    ap.add_argument("--candidates-from", type=Path, help="file, one test name per line")
    ap.add_argument("--auto", action="store_true", help="probe every quarantined test")
    ap.add_argument("--max-boots", type=int, default=8)
    args = ap.parse_args(argv)

    # One probe per worktree at a time, whatever the arch. Every probe boot rebuilds
    # the media, and the build's freshness guard counts ALL THREE sweep lists as build
    # source: a second probe rewriting its own arch's list makes the first one's next
    # build fail BUILD_INCOMPLETE ("build source changed since build"). Three parallel
    # --auto probes (2026-10-08) cost arm64 its whole run that way.
    git_dir = subprocess.run(
        ["git", "rev-parse", "--absolute-git-dir"],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    # Held for the life of the process, so no context manager.
    lock = open(Path(git_dir) / "conformance-probe.lock", "w")  # noqa: SIM115
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        sys.exit(
            "[probe] another conformance_probe is running in this worktree; probes "
            "cannot share one (each rewrites a sweep list the other's build checks). "
            "Run the arches one after another."
        )

    path = LIST.with_name(LIST.name.format(arch=args.arch))
    original = path.read_text()

    if args.candidates:
        cands = [c.strip() for c in args.candidates.split(",") if c.strip()]
    elif args.candidates_from:
        cands = [
            ln.strip()
            for ln in args.candidates_from.read_text().splitlines()
            if ln.strip() and not ln.startswith("#")
        ]
    elif args.auto:
        cands = quarantined(args.arch)
    else:
        ap.error("one of --candidates / --candidates-from / --auto is required")

    def restore(*_a):
        path.write_text(original)
        print(f"\n[probe] restored {path.relative_to(REPO)}")

    signal.signal(signal.SIGINT, lambda *a: (restore(), sys.exit(130)))
    signal.signal(signal.SIGTERM, lambda *a: (restore(), sys.exit(143)))

    passed: dict[str, str] = {}
    crashers: list[str] = []
    try:
        remaining = list(cands)
        for boot in range(1, args.max_boots + 1):
            if not remaining:
                break
            path.write_text(HEADER + "\n".join(remaining) + "\n")
            print(f"[probe] boot {boot}: {len(remaining)} candidate(s) -> {remaining}")
            verdicts, sweep = run_sweep(args.arch)
            got = [t for t in remaining if t in verdicts]
            missing = [t for t in remaining if t not in verdicts]

            # Drop everything recorded AFTER an in-kernel wedge (see POISONING).
            # Without this the probe banks the wedge's cascade as real results:
            # amd64 round 1 booted SeTokenFiltering (TIMEOUT) ahead of four other
            # candidates, which then reported NO-END purely because the wedged
            # process still held the result buffer — and all four were summarised
            # as validly re-tested. `missing` is empty in that boot, so the
            # `if not missing` early-out below would have accepted the lot.
            got, contaminated = split_at_wedge(got, verdicts)

            for t in got:
                passed[t] = verdicts[t]
            print(f"[probe]   sweep={sweep}  verdicts={len(got)}  no-verdict={len(missing)}")
            for t in got:
                print(f"[probe]     {t:32s} {verdicts[t]}")

            if contaminated:
                wedged = got[-1]
                crashers.append(wedged)
                print(f"[probe]   WEDGED BY {wedged} ({verdicts[wedged]}) — it holds the result")
                print(f"[probe]   buffer, so the {len(contaminated)} verdict(s) after it are")
                print("[probe]   CONTAMINATION, not evidence; still CANDIDATES")
                keep = set(contaminated) | set(missing)
                remaining = [t for t in remaining if t in keep]
                continue

            if not missing:
                remaining = []
                break
            # WHICH test ended the run? Two different cases, and conflating them
            # drops an innocent test with zero evidence against it:
            #
            #  (a) the last test that DID get a verdict crashed the box. The parser
            #      records it as CRASH/INCOMPLETE, and the next test never ran. The
            #      culprit is that test, not the first one without a verdict.
            #  (b) a test died so hard it produced no parseable verdict at all. Then
            #      the first candidate with no verdict really is the culprit.
            #
            # The first version of this only implemented (b), so on a boot logging
            # "SeInheritance OK / ZwAllocateVirtualMemory CRASH" it blamed and dropped
            # ExFastMutex — which had never executed. Four arm64 tests were recorded
            # as crashers on that basis and none of them had run.
            last_bad = next(
                (t for t in reversed(got) if verdicts.get(t) in ("CRASH", "INCOMPLETE")), None
            )
            if last_bad is not None:
                crashers.append(last_bad)
                print(f"[probe]   ENDED BY {last_bad} ({verdicts[last_bad]}) — it has a verdict;")
                print(f"[probe]   {len(missing)} test(s) after it never ran; still CANDIDATES")
                remaining = missing
            else:
                crashers.append(missing[0])
                print(f"[probe]   TRUNCATED at {missing[0]} (no verdict at all) — dropping it")
                remaining = missing[1:]
    finally:
        restore()

    print("\n==== PROBE SUMMARY ====")
    print(f"arch: {args.arch}")
    ok = [t for t, s in passed.items() if s == "OK"]
    notok = [(t, s) for t, s in passed.items() if s != "OK"]
    print(f"\nNOW PASSING ({len(ok)}) -- candidates to UN-QUARANTINE:")
    for t in ok:
        print(f"  {t}")
    print(f"\nRAN but not OK ({len(notok)}):")
    for t, s in notok:
        print(f"  {t:32s} {s}")
    print(f"\nSTILL TRUNCATE THE SWEEP ({len(crashers)}) -- keep quarantined:")
    for t in crashers:
        print(f"  {t}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
