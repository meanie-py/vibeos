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
BUGCHECK = re.compile(r"\*\*\* BUGCHECK: (0x[0-9A-Fa-f]+)")

# Child output, already prefixed by the sweep with its suite + subtest.
CHILD = re.compile(r"\[APITEST:(\S+) (\S+)\] (.*)")
SUMMARY = re.compile(
    r"^\S+: (\d+) tests executed \((\d+) marked as todo, (\d+) failures\), (\d+) skipped\."
)
ASSERT_FAIL = re.compile(r"^(\S+\.(?:c|cpp|h):\d+): Test failed:")

sys.path.insert(0, str(Path(__file__).resolve().parent))
from ratchet import BAD, compare, fmt, load_baseline  # noqa: E402


def parse(log_text: str):
    """Return (results, meta). results is one dict per launched subtest."""
    results: dict[str, dict] = {}
    order: list[str] = []
    meta = {"done": None, "bugcheck": None, "suites": {}, "nosubtests": {}}

    def row(name: str) -> dict:
        if name not in results:
            results[name] = {
                "name": name,
                "status": "NO-END",
                "pass": 0,
                "fail": 0,
                "skip": 0,
                "todo": 0,
                "exit": None,
                "errors": 0,
                "asserts": set(),
            }
            order.append(name)
        return results[name]

    def drop(name: str) -> None:
        """Forget a row entirely (used for enumeration rows that succeeded)."""
        if name in results:
            del results[name]
            order.remove(name)

    for line in log_text.splitlines():
        m = ENUMERATE.search(line)
        if m:
            # In flight until the suite enumerates, reports NOSUBTESTS, or times
            # out. Deliberately NOT kept once it resolves: a per-suite row that
            # always says OK would add ~60 identities to the floor that assert
            # nothing about the OS.
            row(f"{m.group(1)} <enumeration>")
            continue

        m = SUITE.search(line)
        if m:
            meta["suites"][m.group(1)] = int(m.group(2))
            drop(f"{m.group(1)} <enumeration>")
            continue

        m = NOSUBTESTS.search(line)
        if m:
            meta["nosubtests"][m.group(1)] = (int(m.group(2)), m.group(3).strip())
            drop(f"{m.group(1)} <enumeration>")
            continue

        m = SKIP.search(line)
        if m:
            row(f"{m.group(1)} {m.group(2)}")["status"] = "SKIP"
            continue

        m = LAUNCH.search(line)
        if m:
            row(f"{m.group(1)} {m.group(2)}")
            continue

        m = FAILSTART.search(line)
        if m:
            row(m.group(1))["status"] = "FAILSTART"
            continue

        m = ENUM_TIMEOUT.search(line)
        if m:
            # Suite-level: nothing was enumerated, so there is no subtest to
            # name. Recorded against the suite alone so it still shows up as
            # bad, without inventing an identity.
            suite = m.group(1) or m.group(2)
            row(f"{suite} <enumeration>")["status"] = "ENUM-TIMEOUT"
            continue

        m = TIMEOUT.search(line)
        if m:
            row(f"{m.group(1)} {m.group(2)}")["status"] = "TIMEOUT"
            continue

        m = EXIT.search(line)
        if m:
            r = row(f"{m.group(1)} {m.group(2)}")
            r["exit"] = int(m.group(3))
            if r["status"] == "NO-END":
                # exit seen but no summary line: the child died or printed
                # nothing parseable. Leave NO-END; it is in BAD and gates.
                pass
            else:
                r["status"] = "OK" if r["exit"] == 0 else "FAIL-EXIT"
            continue

        m = CHILD.search(line)
        if m:
            r = row(f"{m.group(1)} {m.group(2)}")
            body = m.group(3)
            s = SUMMARY.match(body)
            if s:
                executed, todo, failures, skipped = (int(x) for x in s.groups())
                r["fail"] = failures
                r["todo"] = todo
                r["skip"] = skipped
                # wine counts EXECUTED, not passed; pass is the remainder. A
                # `todo` is neither: it is an expected failure the framework
                # already discounts.
                r["pass"] = max(executed - failures, 0)
                # A summary line means the subtest reached its own end. The exit
                # code decides OK vs FAIL-EXIT, and may arrive after this line.
                r["status"] = "OK" if (r["exit"] in (None, 0)) else "FAIL-EXIT"
                continue
            a = ASSERT_FAIL.match(body)
            if a:
                r["asserts"].add(a.group(1))
            continue

        m = DONE.search(line)
        if m:
            meta["done"] = m.group(1)
            continue

        m = BUGCHECK.search(line)
        if m:
            meta["bugcheck"] = m.group(1)
            # Whatever was mid-flight when the machine died owns the crash.
            # The test is "the sweep never saw it exit", NOT "it looked bad":
            # a subtest can print its own summary line and THEN take the kernel
            # down on teardown, and reading that as OK is a false green -- the
            # first draft of this parser scored `afd_apitest send` OK on the
            # very boot whose BUGCHECK it caused.
            # ...but only a row still IN FLIGHT can own it. A status the sweep
            # already resolved -- it skipped it, timed it out and terminated it,
            # or failed to enumerate it and moved to the next suite -- means the
            # sweep was past that subtest when the machine died. Overwriting
            # those is the same "blame whatever was last" error this harness has
            # made at every other layer: it handed a BUGCHECK to msgina's
            # enumeration timeout, a suite that never ran a single subtest.
            # If nothing is in flight the crash stays unattributed, which is the
            # honest answer.
            for name in reversed(order):
                if results[name]["exit"] is None and results[name]["status"] not in RESOLVED:
                    results[name]["status"] = "CRASH"
                    break

    return [results[n] for n in order], meta


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
        keep = dict(base)
        for r in results:
            if args.only and r["name"] not in args.only:
                continue
            keep[r["name"]] = r
        rows = []
        for name in sorted(keep):
            r = keep[name]
            rows.append(fmt(r) if isinstance(r.get("asserts"), set) else _refmt(name, r))
        args.write_baseline.write_text(
            "# apitest conformance baseline — parse_apitest_serial.py format\n"
            + "\n".join(rows)
            + "\n"
        )
        print(f"# wrote {args.write_baseline} ({len(keep)} row(s))")

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
    """Re-emit a baseline row we did not re-run, exactly as it was loaded."""
    line = (
        f"{name} {b['status']} pass={b.get('pass', 0)} "
        f"fail={b.get('fail', 0)} skip={b.get('skip', 0)}"
    )
    if "exit" in b:
        line += f" exit={b['exit']}"
    if b.get("errors"):
        line += f" errors={b['errors']}"
    flaky = b.get("flaky", set())
    for a in sorted(b.get("asserts") or ()):
        line += f"\n    assert{'?' if a in flaky else ''} {a}"
    return line


if __name__ == "__main__":
    raise SystemExit(main())
