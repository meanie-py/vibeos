#!/usr/bin/env python3
"""The conformance RATCHET, shared by every sweep parser.

Extracted from parse_kmtest_serial.py when the apitest sweep arrived and wanted
exactly the same floor semantics. Nothing in here knows what a sweep looks like:
it takes result dicts (`name`, `status`, `pass`, `fail`, `skip`, `exit`,
`errors`, `asserts`) and a baseline file, and answers whether the floor held.
The sweep-specific half -- the serial markers -- stays in each parser.

THE FLOOR IS A SET OF ASSERTION IDENTITIES, NOT A SET OF COUNTS. Counts cannot
express what a run did: fix three assertions, break three others, and
pass/fail/skip do not move. Measured over 595 archived arm64 kmtest sweeps, 33
of the 37 count-signatures shared by more than one run cover runs whose FAILING
ASSERTIONS differ -- the worst is 10 runs with byte-identical counts differing
in 6 identities.
"""

from __future__ import annotations

from pathlib import Path


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


def truncated_identity(ident, known):
    """Is `ident` a floor identity with its head cut off by an interleaved print?

    The sweep's failure lines share the serial port with every other print, and
    one can land inside the file name (i386, verbatim):

        [+  118.919] Exam(drivers/storage/port/scsiport/ioctl.c:609) unknown ioctl code: 0x7C0C8
        [+  118.931] ple.c:20: Test failed: IRQL is 31, expected 2

    which reads as `ple.c:20`, a NEW identity, where the floor lists
    `Example.c:20`. A proper suffix of a known identity in the same test's row
    (so the same line number and the same end of the file name) is that
    identity; anything else is still new.
    """
    return any(k != ident and k.endswith(ident) for k in known)


# ENUM-TIMEOUT: the suite never enumerated its subtests, so NONE of them ran.
# It has to be BAD or a suite that reports nothing at all reads as clean.
BAD = {"CRASH", "TIMEOUT", "FAILSTART", "NO-END", "INCOMPLETE", "ENUM-TIMEOUT"}


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
                    if not truncated_identity(a, known):
                        regressions.append(f"{r['name']}: NEW failing assertion {a}")
    ran = {r["name"] for r in results}
    for name, b in base.items():
        if name not in ran:
            regressions.append(f"{name}: in baseline ({b['status']}) but never launched")
    return regressions
