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

# The sweep's own markers are matched from `SWEEP] ` on: another print can land
# inside their head and carry it off, `[K(drivers/.../ioctl.c:609) unknown ...`
# then `MTEST-SWEEP] launch KeFloatPointState` on the next line (i386, verbatim),
# and a launch read as missing makes the test "in baseline but never launched".
# Test names are identifiers: a print glued straight onto one,
# `launch RtlIntSafe(drivers/...)`, must not become part of the name.
LAUNCH = re.compile(r"SWEEP\] launch (?!FAILED)([A-Za-z0-9_]+)")
FAILSTART = re.compile(r"SWEEP\] launch FAILED ([A-Za-z0-9_]+)")
# The test's own first line, from inside kmtest: the second witness of a launch
# when the sweep's marker was cut too deep to read (`[KMTEST-SWEEP] lau` + a
# kernel print, then `nch KeEvent`).
BEGIN = re.compile(r"KMTEST:BEGIN ([A-Za-z0-9_]+)\]")
# Likewise END from `KMTEST:END` on: its own `[` can be the byte carried off.
END = re.compile(r"KMTEST:END (\S+) pass=(\d+) fail=(\d+) skip=(\d+)\]")
ERROR = re.compile(r"\[KMTEST:ERROR\]")
EXIT = re.compile(r"SWEEP\] exit ([A-Za-z0-9_]+) code=(\d+)")
TIMEOUT = re.compile(r"SWEEP\] TIMEOUT ([A-Za-z0-9_]+)")
DONE = re.compile(r"SWEEP\] done (.*)")
BUGCHECK = re.compile(r"\*\*\* BUGCHECK: (0x[0-9A-Fa-f]+)")
# kmt_test.h's KmtVOk emits one of these per FAILING assertion (successes are not
# logged — `if (0/*KmtReportSuccess*/)`). `<file>:<line>` is the assertion's identity;
# the message carries run-varying values (addresses, NTSTATUS) and is NOT part of it.
#
# NOT anchored to the start of the line: a failure message that prints a raw WCHAR
# (NpfsFileInfo.c:56 prints 0xFFFF) can swallow its own newline, gluing the NEXT
# assertion onto the end of the line. Anchored, NpfsFileInfo.c:57 — which fails on
# every boot on every arch — was seen only when the byte happened not to eat the
# newline, and the 596-round soak marked it bistable. The file part must start at
# a non-identifier boundary: a hex address glued straight onto a name
# (`0x000000000012F300RtlStack.c:42`) has no recoverable boundary, so it is left
# unmatched rather than banked as a garbage identity.
#
# Kmtests are C++ too: upstream made ObTypes a .cpp (synced 2026-09-21), and a `.c`-only
# pattern silently dropped every one of its identities from the ratchet while its counts
# still parsed, so nothing looked wrong.
ASSERT_FAIL = re.compile(r"(?<![A-Za-z0-9_])([A-Za-z][A-Za-z0-9_]*\.c(?:pp)?:\d+): Test failed:")

# The floor semantics live in ratchet.py, shared with the apitest parser; only
# the markers above are kmtest-specific. The sys.path insert is load-bearing:
# tests/qemu/cli/conformance.py imports THIS file via spec_from_file_location,
# which does not put its directory on sys.path the way running it as a script
# does -- so a bare `from ratchet import` works from the shell and raises
# ModuleNotFoundError from the CLI, which is the worse half of the pair to find
# out about at runtime.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from ratchet import BAD, compare, fmt, load_baseline  # noqa: E402
from ratchet import _call as _native_call  # noqa: E402
from ratchet import _rows as _native_rows  # noqa: E402

END_PREFIX = "KMTEST:END "
# The conformance harness stamps every serial line `[+  128.268] `; a spliced
# tail is matched after it.
TIMESTAMP = re.compile(r"^\[\+\s*\d+(?:\.\d+)?\]\s?")


def splice_split_end_markers(lines):
    return _native_call("splice_end", lines=lines)


def parse(log_text):
    rows, *metadata = _native_call("kmtest", text=log_text)
    return (_native_rows(rows), *metadata)


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
        _native_call(
            "write_cli_baseline", kind="kmtest", path=str(args.write_baseline), results=results
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
