#!/usr/bin/env python3
"""Parse a kmtest sweep serial log into a per-test conformance results table.

conformance-ratchet quest M3 (docs/engineering/conformance-ratchet-quest.md).

Input: a smoke live serial log (qemu/<arch>_smoke_live.log) from a boot with the
kmtest payload. The markers it consumes (all emitted by tools/payloads/kmtest/sweep.c
and the VIBEOS_KMTEST_SERIAL tee in kmtest.c):

    [KMTEST-SWEEP] launch <Test>
    [KMTEST:BEGIN <Test>]
    [KMTEST:ERROR] <msg>: error 0x<code>
    [KMTEST:END <Test> pass=N fail=N skip=N]
    [KMTEST-SWEEP] exit <Test> code=N
    [KMTEST-SWEEP] TIMEOUT <Test>
    [KMTEST-SWEEP] launch FAILED <Test> ...
    [KMTEST-SWEEP] done total=...
    *** BUGCHECK: 0x...

plus, from kmt_test.h's KmtVOk, one line per FAILING assertion:

    <file>.c:<line>: Test failed: <message>

Successes emit nothing (`if (0/*KmtReportSuccess*/)`), which is why the pass count
is still gated as a count below — there are no per-success identities to compare.

Output (stdout): one line per launched test —
    <Test> <STATUS> pass=N fail=N skip=N [exit=N] [errors=N]
STATUS: OK (END seen + exit 0), FAIL-EXIT (END + nonzero exit), NO-END (launched,
process exited, no END marker), TIMEOUT, FAILSTART, CRASH (launched, then BUGCHECK
with no exit). A trailing summary line reports totals + whether the sweep finished
(done marker) or died (BUGCHECK).

This is the read side of the ratchet: `--baseline <file>` compares against a stored
baseline (same format) and exits 1 on any regression. `--write-baseline <file>` stores
the current results as the new floor.

THE FLOOR IS A SET OF ASSERTION IDENTITIES, NOT A SET OF COUNTS. A baseline row may
carry indented continuation lines naming the assertions expected to fail:

    IoDeviceObject OK pass=185 fail=2 skip=0
        assert  IoDeviceObject_drv.c:198
        assert? IoDeviceObject_drv.c:206

`assert` is expected to fail; `assert?` is known-bistable, so it is a regression
neither when it fires nor when it does not, and a re-seed carries it forward rather
than banking whichever face one boot showed.

Counts cannot express what a run did: fix three assertions, break three others, and
pass/fail/skip do not move. Measured over 595 archived arm64 sweeps, 33 of the 37
count-signatures shared by more than one run cover runs whose FAILING ASSERTIONS
differ — the worst is 10 runs with byte-identical counts differing in 6 identities.
A count-gated ratchet accepts all of them as the same result.

So the gate is: a new status in BAD, a DROPPED pass count (counts, necessarily —
successes have no identities), or an assertion failing that the floor does not list.
A fail count that grew without a new identity is a loop running longer, not a
regression. A row with no `assert` lines has not been seeded yet and keeps the old
fail-count gate, so seeding can proceed per-arch without a flag day.
"""

import argparse
import re
import sys
from pathlib import Path

LAUNCH = re.compile(r"\[KMTEST-SWEEP\] launch (?!FAILED)(\S+)")
FAILSTART = re.compile(r"\[KMTEST-SWEEP\] launch FAILED (\S+)")
END = re.compile(r"\[KMTEST:END (\S+) pass=(\d+) fail=(\d+) skip=(\d+)\]")
ERROR = re.compile(r"\[KMTEST:ERROR\]")
EXIT = re.compile(r"\[KMTEST-SWEEP\] exit (\S+) code=(\d+)")
TIMEOUT = re.compile(r"\[KMTEST-SWEEP\] TIMEOUT (\S+)")
DONE = re.compile(r"\[KMTEST-SWEEP\] done (.*)")
BUGCHECK = re.compile(r"\*\*\* BUGCHECK: (0x[0-9A-Fa-f]+)")
# kmt_test.h's KmtVOk emits one of these per FAILING assertion (successes are not
# logged — `if (0/*KmtReportSuccess*/)`). `<file>:<line>` is the assertion's identity;
# the message carries run-varying values (addresses, NTSTATUS) and is NOT part of it.
ASSERT_FAIL = re.compile(r"^(\S+\.c:\d+): Test failed:")

# The floor semantics live in ratchet.py, shared with the apitest parser; only
# the markers above are kmtest-specific. The sys.path insert is load-bearing:
# tests/qemu/cli/conformance.py imports THIS file via spec_from_file_location,
# which does not put its directory on sys.path the way running it as a script
# does -- so a bare `from ratchet import` works from the shell and raises
# ModuleNotFoundError from the CLI, which is the worse half of the pair to find
# out about at runtime.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from ratchet import BAD, compare, fmt, load_baseline  # noqa: E402


def parse(log_text):
    """Return (results: list[dict], done_line: str|None, bugcheck: str|None)."""
    results = []
    current = None  # the test whose launch we've seen but not exit/timeout
    done_line = None
    bugcheck = None
    for line in log_text.splitlines():
        m = LAUNCH.search(line)
        if m:
            current = {
                "name": m.group(1),
                "status": None,
                "pass": 0,
                "fail": 0,
                "skip": 0,
                "exit": None,
                "errors": 0,
                "end_seen": False,
                "asserts": set(),
            }
            results.append(current)
            continue
        m = FAILSTART.search(line)
        if m:
            results.append(
                {
                    "name": m.group(1),
                    "status": "FAILSTART",
                    "pass": 0,
                    "fail": 0,
                    "skip": 0,
                    "exit": None,
                    "errors": 0,
                    "end_seen": False,
                    "asserts": set(),
                }
            )
            current = None
            continue
        m = END.search(line)
        if m and current and m.group(1) == current["name"]:
            current["pass"], current["fail"], current["skip"] = (
                int(m.group(2)),
                int(m.group(3)),
                int(m.group(4)),
            )
            current["end_seen"] = True
            continue
        m = ASSERT_FAIL.match(line.strip())
        if m and current:
            # Attributed to the launched test, not to the next END: a test that
            # CRASHes or TIMEOUTs never emits END, and attributing on END alone
            # would bleed its failures into whichever test ends next. That
            # misattribution is real — a 595-run arm64 soak shows ExPools.c lines
            # surfacing under ExSequencedList exactly when the run ahead died.
            current["asserts"].add(m.group(1))
            continue
        if current and ERROR.search(line):
            current["errors"] += 1
            continue
        m = EXIT.search(line)
        if m and current and m.group(1) == current["name"]:
            current["exit"] = int(m.group(2))
            if current["end_seen"]:
                current["status"] = "OK" if current["exit"] == 0 else "FAIL-EXIT"
            else:
                current["status"] = "NO-END"
            current = None
            continue
        m = TIMEOUT.search(line)
        if m and current and m.group(1) == current["name"]:
            current["status"] = "TIMEOUT"
            current = None
            continue
        m = DONE.search(line)
        if m:
            done_line = m.group(1)
            # The first complete sweep is authoritative. On arm64 the sweep exe
            # can re-autostart after finishing (the guest keeps draining), and a
            # second, partial re-run would otherwise clobber the early tests'
            # clean results with INCOMPLETE/None. A mid-run BUGCHECK emits no DONE,
            # so CRASH handling above is unaffected; single-run x86 has nothing
            # after DONE, so this is a no-op there.
            break
        m = BUGCHECK.search(line)
        if m and bugcheck is None:
            bugcheck = m.group(1)
            if current and current["status"] is None:
                current["status"] = "CRASH"
                current = None
    if current and current["status"] is None:
        current["status"] = "INCOMPLETE"  # log truncated mid-test, no bugcheck seen

    # Collapse duplicate test names into one entry each. A test can be launched
    # more than once within a sweep when a transient (e.g. the netstack-TDI
    # self-test) makes an early attempt not complete and the runner re-reaches
    # it; the incomplete first attempt (None/INCOMPLETE) must not mask the clean
    # later pass. Keep the most-complete result per name, preserving first-seen
    # order. A genuinely flaky test (real CRASH on some runs) is caught later by
    # the design's >=3-run stability check + flaky.txt, not hidden here.
    RANK = {
        "OK": 6,
        "FAIL-EXIT": 5,
        "NO-END": 4,
        "TIMEOUT": 3,
        "CRASH": 3,
        "FAILSTART": 2,
        "INCOMPLETE": 1,
        None: 0,
    }
    best, order = {}, []
    for r in results:
        n = r["name"]
        if n not in best:
            best[n] = r
            order.append(n)
        elif RANK.get(r["status"], 0) > RANK.get(best[n]["status"], 0):
            best[n] = r
    results = [best[n] for n in order]

    return results, done_line, bugcheck


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("log", help="serial log path")
    ap.add_argument("--baseline", help="compare against this baseline; exit 1 on regression")
    ap.add_argument("--write-baseline", help="write results as new baseline file")
    args = ap.parse_args()

    text = Path(args.log).read_text(errors="replace")
    results, done_line, bugcheck = parse(text)

    lines = [fmt(r) for r in results]
    print("\n".join(lines))
    ok = sum(1 for r in results if r["status"] == "OK")
    bad = sum(1 for r in results if r["status"] in BAD)
    print(
        f"# summary: launched={len(results)} ok={ok} bad={bad} "
        f"sweep={'done: ' + done_line if done_line else 'DID NOT FINISH'}"
        f"{' bugcheck=' + bugcheck if bugcheck else ''}"
    )

    if args.write_baseline:
        Path(args.write_baseline).write_text(
            "# kmtest conformance baseline — parse_kmtest_serial.py format\n"
            + "\n".join(lines)
            + "\n"
        )
        print(f"# baseline written: {args.write_baseline}")

    if args.baseline:
        regressions = compare(results, load_baseline(args.baseline))
        if regressions:
            print("# RATCHET REGRESSIONS:")
            for reg in regressions:
                print(f"#   {reg}")
            sys.exit(1)
        print("# ratchet holds: no regressions vs baseline")


if __name__ == "__main__":
    main()
