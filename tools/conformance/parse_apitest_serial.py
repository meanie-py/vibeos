#!/usr/bin/env python3
"""Parse an apitest sweep serial log into a per-subtest conformance table.

The user-mode sibling of parse_kmtest_serial.py. Floor semantics are shared
(ratchet.py); only the markers differ. Those markers were written against REAL
captured output, not an assumed format -- the first draft of the sweep itself
was wrong about how a wine/test.h binary behaves with no arguments, so nothing
here is guessed.

Input: a boot serial log from a run with the apitest payload. Markers, all from
tools/payloads/apitest/sweep.c:

    [APITEST-SWEEP] suite <suite.exe> subtests=N
    [APITEST-SWEEP] launch <suite.exe> <subtest>
    [APITEST-SWEEP] exit <suite.exe> <subtest> code=N
    [APITEST-SWEEP] TIMEOUT <suite.exe> <subtest> ...
    [APITEST-SWEEP] SKIP <suite.exe> <subtest>
    [APITEST-SWEEP] NOSUBTESTS <suite.exe> (exit N) first-line=...
    [APITEST-SWEEP] launch FAILED <suite.exe> ...
    [APITEST-SWEEP] done total=... launched=... subtests=...
    *** BUGCHECK: 0x...

and, piped from each child's stdout and prefixed by the sweep:

    [APITEST:<suite.exe> <subtest>] <file>:<line>: Test failed: <msg>
    [APITEST:<suite.exe> <subtest>] <name>: N tests executed
        (T marked as todo, F failures), S skipped.

THE IDENTITY IS "<suite.exe> <subtest>", not the suite. A suite bundles up to
19 subtests here and the sweep runs one process each, so suite-level rows would
throw away both the isolation and the attribution the harness exists to provide.

Unlike kmtest, wine/test.h prints a summary line per subtest carrying the
executed/failure/skipped counts directly, so the counts are read rather than
tallied. Failing assertions still supply the identities that form the floor.

STATUS: OK (summary seen + exit 0), FAIL-EXIT (summary + nonzero exit), NO-END
(launched, exited, no summary line), TIMEOUT, FAILSTART, SKIP, CRASH (launched,
then BUGCHECK with no exit).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# Statuses the sweep itself already resolved: it moved on, so the subtest was
# not in flight when a later BUGCHECK landed.
RESOLVED = frozenset({"SKIP", "TIMEOUT", "ENUM-TIMEOUT", "FAILSTART"})

SUITE = re.compile(r"\[APITEST-SWEEP\] suite (\S+) subtests=(\d+)")
# Emitted BEFORE the enumeration child runs, so a crash while enumerating has
# something to be attributed to. Without it the sweep is invisible between
# suites: a BUGCHECK there had no in-flight row at all, which stopped the
# characterisation driver dead ("sweep incomplete but no in-flight subtest").
ENUMERATE = re.compile(r"\[APITEST-SWEEP\] enumerate (\S+)")
LAUNCH = re.compile(r"\[APITEST-SWEEP\] launch (?!FAILED)(\S+) (\S+)")
FAILSTART = re.compile(r"\[APITEST-SWEEP\] launch FAILED (\S+)")
EXIT = re.compile(r"\[APITEST-SWEEP\] exit (\S+) (\S+) code=(-?\d+)")
# The ENUMERATION timeout has to be matched FIRST and separately, because the
# sweep writes it as "TIMEOUT <suite> during enumeration (60s)" -- prose, not a
# subtest. A bare `TIMEOUT (\S+) (\S+)` happily read that as the suite plus a
# subtest literally named "during", and the invented identity
# `msgina_apitest.exe during TIMEOUT` reached the banked table. That is worse
# than a miscount: the conformance floor IS a set of assertion identities, so a
# phantom one can never be satisfied by any future run.
# Accepts the newer unambiguous ENUM-TIMEOUT spelling too, since the archived
# serials still carry the old prose form.
ENUM_TIMEOUT = re.compile(
    r"\[APITEST-SWEEP\] (?:ENUM-TIMEOUT (\S+)|TIMEOUT (\S+) during enumeration)"
)
# Per-subtest: anchored on the trailing "(Ns) - terminated" the subtest form
# always carries and the enumeration form never does.
TIMEOUT = re.compile(r"\[APITEST-SWEEP\] TIMEOUT (\S+) (\S+) \(\d+s\) - terminated")
SKIP = re.compile(r"\[APITEST-SWEEP\] SKIP (\S+) (\S+)")
NOSUBTESTS = re.compile(r"\[APITEST-SWEEP\] NOSUBTESTS (\S+) \(exit (-?\d+)\)(.*)")
DONE = re.compile(r"\[APITEST-SWEEP\] done (.*)")
# Capture the ARGUMENT TUPLE, not just the stop code. 0x1E is
# KMODE_EXCEPTION_NOT_HANDLED -- a category, not a diagnosis -- and the first
# argument is the exception code that actually discriminates. This cost a bad
# call: 13 quarantine entries were un-quarantined together because all were
# recorded as "BUGCHECK 0x0000001E", on the theory that the probe-then-raw-access
# fixes (always 0xC0000005) would clear them. powrprof's was 0xC0000047 and was
# never a member. The tuple was in the serial the whole time; the parser threw it
# away, so the quarantine comments could not record it.
BUGCHECK = re.compile(r"\*\*\* BUGCHECK: (0x[0-9A-Fa-f]+(?:\s*\([^)]*\))?)")

# Child output, already prefixed by the sweep with its suite + subtest.
CHILD = re.compile(r"\[APITEST:(\S+) (\S+)\] (.*)")
# wine/test.h PLURALISES its counts, so a subtest with exactly one failing
# assertion prints "1 failure)" and one with a single case prints "1 test executed".
# Requiring the plural silently dropped every such summary: the row kept its
# initial NO-END status, its pass/fail counts were lost, and NO-END is in the
# ratchet's BAD set -- so 14 of round 1's 31 NO-END rows were real results scored
# as broken. (The other 17 are genuine: exit 0x80 with no summary line at all.)
SUMMARY = re.compile(
    r"^\S+: (\d+) tests? executed \((\d+) marked as todo, (\d+) failures?\), (\d+) skipped\."
)
ASSERT_FAIL = re.compile(r"^(\S+\.(?:c|cpp|h):\d+): Test failed:")

sys.path.insert(0, str(Path(__file__).resolve().parent))
from ratchet import BAD, compare, fmt, load_baseline  # noqa: E402
from ratchet import _call as _native_call  # noqa: E402
from ratchet import _rows as _native_rows  # noqa: E402


def parse(log_text: str):
    rows, *metadata = _native_call("apitest", text=log_text)
    metadata[0]["nosubtests"] = {
        name: tuple(value) for name, value in metadata[0]["nosubtests"].items()
    }
    return (_native_rows(rows), *metadata)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("serial", type=Path, help="boot serial log from an apitest run")
    ap.add_argument(
        "--baseline", type=Path, help="compare against this floor; exit 1 on regression"
    )
    ap.add_argument("--write-baseline", type=Path, help="store this run as the new floor")
    ap.add_argument(
        "--only",
        nargs="+",
        default=None,
        help="with --write-baseline, re-seed ONLY these rows and carry the rest "
        "forward untouched. A wholesale rewrite re-rolls every row from one boot, "
        "which freezes whatever that boot happened to show",
    )
    args = ap.parse_args()

    results, meta = parse(args.serial.read_text(errors="replace"))

    base = load_baseline(args.baseline) if args.baseline and args.baseline.is_file() else {}
    for r in results:
        print(fmt(r, flaky=base.get(r["name"], {}).get("flaky", ())))

    bad = sum(1 for r in results if r["status"] in BAD)
    print(
        f"\n# {len(results)} subtest(s) across {len(meta['suites'])} suite(s); "
        f"{bad} bad; sweep {'done' if meta['done'] else 'INCOMPLETE'}"
        + (f"; BUGCHECK {meta['bugcheck']}" if meta["bugcheck"] else "")
    )
    for suite, (code, first) in sorted(meta["nosubtests"].items()):
        print(f"# NOSUBTESTS {suite} exit={code} {first}")

    if args.write_baseline:
        count = _native_call(
            "write_cli_baseline",
            kind="apitest",
            path=str(args.write_baseline),
            results=results,
            base=base,
            only=args.only,
        )
        print(f"# wrote {args.write_baseline} ({count} row(s))")

    if args.baseline:
        regressions = compare(results, base)
        if regressions:
            print("\nRATCHET REGRESSIONS:")
            for x in regressions:
                print(f"  {x}")
            return 1
        print("\nratchet holds: no regressions vs baseline.")
    return 0


def _refmt(name: str, b: dict) -> str:
    return _native_call("refmt", name=name, base=b)


if __name__ == "__main__":
    raise SystemExit(main())
