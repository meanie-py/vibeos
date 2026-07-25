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
            for t in got:
                passed[t] = verdicts[t]
            print(f"[probe]   sweep={sweep}  verdicts={len(got)}  no-verdict={len(missing)}")
            for t in got:
                print(f"[probe]     {t:32s} {verdicts[t]}")
            if not missing:
                remaining = []
                break
            # First candidate with no verdict is the one that ended the run.
            crashers.append(missing[0])
            print(f"[probe]   TRUNCATED at {missing[0]} — dropping it and re-running")
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
