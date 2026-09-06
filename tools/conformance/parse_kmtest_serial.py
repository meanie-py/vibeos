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


def fmt(r, flaky=()):
    """One baseline row, plus an indented `assert` line per failing assertion.

    Counts alone cannot express what a run did: fix three assertions, break three
    others, and pass/fail/skip are unchanged. Measured on 595 archived arm64
    sweeps, 33 of the 37 count-signatures shared by more than one run cover runs
    whose FAILING ASSERTIONS differ — the worst is 10 runs with byte-identical
    counts differing in 6 identities. So the identity set is the floor and the
    counts are commentary.

    `assert?` marks an identity known to come and go. A bistable row is not a
    regression when it appears and not an improvement when it does not; without
    the mark, every soak round would report one or the other forever.
    """
    line = f"{r['name']} {r['status']} pass={r['pass']} fail={r['fail']} skip={r['skip']}"
    if r["exit"] is not None:
        line += f" exit={r['exit']}"
    if r["errors"]:
        line += f" errors={r['errors']}"
    flaky = set(flaky)
    for a in sorted(r.get("asserts") or ()):
        line += f"\n    assert{'?' if a in flaky else ''} {a}"
    # A known-bistable identity keeps its row on a run where it did NOT fire —
    # dropping it would re-seed the flake away and hand the next run a false
    # regression.
    for a in sorted(flaky - set(r.get("asserts") or ())):
        line += f"\n    assert? {a}"
    return line


def load_baseline(path):
    base = {}
    entry = None
    for line in Path(path).read_text().splitlines():
        raw, line = line, line.strip()
        if not line or line.startswith("#"):
            continue
        if raw[:1].isspace() and line.split()[0] in ("assert", "assert?"):
            if entry is None:
                continue  # an assert line before any test row: ignore, not crash
            kind, ident = line.split(None, 1)
            entry.setdefault("asserts", set()).add(ident.strip())
            if kind == "assert?":
                entry.setdefault("flaky", set()).add(ident.strip())
            continue
        parts = line.split()
        entry = {"status": parts[1]}
        for kv in parts[2:]:
            k, v = kv.split("=")
            entry[k] = int(v)
        base[parts[0]] = entry
    return base


BAD = {"CRASH", "TIMEOUT", "FAILSTART", "NO-END", "INCOMPLETE"}


def compare(results, base):
    """Return list of regression strings (empty = ratchet holds)."""
    regressions = []
    for r in results:
        b = base.get(r["name"])
        if b is None:
            continue  # new test: informational, not a regression
        if r["status"] in BAD and b["status"] not in BAD:
            regressions.append(f"{r['name']}: {b['status']} -> {r['status']}")
            continue
        if r["status"] not in BAD and b["status"] not in BAD:
            # A dropped pass count is still count-gated, and has to be: kmtest
            # logs nothing per SUCCEEDING assertion (kmt_test.h disables that
            # branch), so successes have no identities to compare. A test that
            # quietly stops asserting shows up here and nowhere else.
            if r["pass"] < b.get("pass", 0):
                regressions.append(f"{r['name']}: pass {b['pass']} -> {r['pass']} (dropped)")
            known = b.get("asserts")
            if known is None:
                # Row not seeded with identities yet — gate it the old way rather
                # than silently not gating it at all. Seeding is per-arch and
                # incremental; see --write-baseline.
                if r["fail"] > b.get("fail", 0):
                    regressions.append(f"{r['name']}: fail {b['fail']} -> {r['fail']} (grew)")
            else:
                # The real gate: assertions that fail now and were not failing
                # before. Offsetting changes cannot hide here, and a fail COUNT
                # that grew without a new identity is a loop running longer, not
                # a regression.
                for a in sorted(set(r.get("asserts") or ()) - known):
                    regressions.append(f"{r['name']}: NEW failing assertion {a}")
    ran = {r["name"] for r in results}
    for name, b in base.items():
        if name not in ran:
            regressions.append(f"{name}: in baseline ({b['status']}) but never launched")
    return regressions


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
