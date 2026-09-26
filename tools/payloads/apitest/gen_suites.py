#!/usr/bin/env python3
"""Derive the apitest sweep's suite list from the binaries a build actually produced.

    python3 tools/payloads/apitest/gen_suites.py --arch amd64

Writes tools/payloads/apitest/apitest-suites-<arch>.txt (gitignored), which
payload.yaml stages in-guest as %SystemRoot%\\bin\\apitest-suites.txt.

WHY THIS IS DERIVED AND THE KMTEST LIST IS NOT
----------------------------------------------
The kmtest sweep reads a COMMITTED list, and upstream's 1385-line ExPushLock
kmtest sat in the build, registered in the driver's testlist, never launched
once, for the length of a whole merge -- while the sweep reported
`launched=91 ok=91 bad=0`, which reads as a clean run. Nothing reconciled the
committed list against the tests that exist. So this list is generated from the
staged binaries every time, and a suite can only go missing by not building.

Quarantines still need to be expressible, so a committed
`quarantine-<arch>.txt` (one name per line, '#' comments) is subtracted -- but
subtracting from a derived set is loud: a quarantine naming a suite that no
longer exists is reported, rather than silently doing nothing.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]


# Two naming conventions, both under apitests/: most suites are <mod>_apitest.exe,
# but the statically-linked RTL suite is rtl_unittest.exe ("so it won't run in
# rosautotest", per its CMakeLists). Match both rather than silently miss one.
SUITE_GLOBS = ("*_apitest.exe", "*_unittest.exe")


def discover(build_dir: Path) -> list[str]:
    """Every apitest binary the build produced, by basename, sorted."""
    root = build_dir / "modules" / "rostests" / "apitests"
    if not root.is_dir():
        return []
    found: set[str] = set()
    for pat in SUITE_GLOBS:
        found.update(p.name for p in root.rglob(pat))
    return sorted(found)


def read_quarantine(arch: str) -> tuple[set[str], list[str], Path]:
    """Returns (whole suites, "<suite> <subtest>" entries, path).

    A bare `foo_apitest.exe` line drops the whole suite. A
    `foo_apitest.exe SomeSubtest` line drops only that subtest, which is the
    granularity that actually matters: advapi32's RtlEncryptMemory crash-loops
    services.exe and would otherwise cost the other 18 advapi32 subtests.
    """
    path = HERE / f"quarantine-{arch}.txt"
    suites: set[str] = set()
    subtests: list[str] = []
    if path.is_file():
        for line in path.read_text(encoding="utf-8").splitlines():
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            if " " in line:
                subtests.append(" ".join(line.split()))
            else:
                suites.add(line)
    return suites, subtests, path


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", required=True, choices=("i386", "amd64", "arm64"))
    ap.add_argument(
        "--build-dir",
        type=Path,
        default=None,
        help="defaults to <repo>/build-<arch>",
    )
    ap.add_argument(
        "--only",
        action="append",
        metavar="SUITE.exe",
        help="restrict the sweep to these suites (repeatable). For reproducing one "
        "suite's crash without the other 58 running first: a full sweep is ~3 hours "
        "and mixes every suite's residue into one serial, which is no way to ask "
        "whether a wedge is self-contained. Quarantines still apply, and a name that "
        "matches nothing is an error rather than an empty run.",
    )
    ap.add_argument(
        "--ignore-quarantine",
        action="store_true",
        help="do not subtract the quarantine — neither the suites nor the subtest "
        "skips. This is how you REPRODUCE a quarantined wedge: the entry that "
        "records it is also what suppresses it, so a plain --only run of a "
        "quarantined suite announces SKIP and cannot fail. Hand-editing the "
        "quarantine to get around that is how you forget to put it back, so it "
        "is a flag. Like --only, the generated lists are marked as a reproduction "
        "set and must not seed a baseline.",
    )
    args = ap.parse_args(argv)

    build_dir = args.build_dir or (REPO / f"build-{args.arch}")
    found = discover(build_dir)
    if not found:
        print(
            f"gen_suites: no apitest binaries under {build_dir}/modules/rostests/apitests.\n"
            f"  fix: ./vibeos-cli build --arch {args.arch} --rostests --keep-going",
            file=sys.stderr,
        )
        return 1

    # The quarantine staleness checks below must see EVERYTHING the build produced,
    # not the --only subset: filtering first made them report 9 healthy quarantine
    # entries as naming suites "this build did not produce". That warning exists to
    # catch typos, so a filtered run that cries wolf destroys its only job.
    all_found = list(found)

    if args.only:
        want = set(args.only)
        missing = sorted(want - set(all_found))
        if missing:
            print(
                f"gen_suites: --only names {len(missing)} suite(s) this build did not "
                f"produce: {', '.join(missing)}",
                file=sys.stderr,
            )
            return 1
        found = [s for s in found if s in want]

    quarantined, skipped, qpath = read_quarantine(args.arch)
    # --ignore-quarantine disables the SUBTRACTION ONLY. The staleness checks below
    # keep reading the real file: clearing these up front (the first draft did)
    # silently disabled the typo check too -- the same "filter applied upstream of the
    # checks that describe the build" mistake --only's first draft made, and the
    # comment there claimed the opposite of what the code did until a test said so.
    armed = set() if args.ignore_quarantine else quarantined
    armed_skips = [] if args.ignore_quarantine else skipped
    stale = sorted(quarantined - set(all_found))
    if stale:
        # Loud, per the module docstring: a quarantine that matches nothing is
        # either a typo or a suite that was renamed away under it.
        print(
            f"gen_suites: {qpath.name} names {len(stale)} suite(s) this build did not "
            f"produce: {', '.join(stale)}",
            file=sys.stderr,
        )

    # The skip file is staged beside the suite list and read by the sweep.
    skip_out = HERE / f"apitest-skip-{args.arch}.txt"
    skip_lines = [
        f"# apitest subtest skips — {args.arch}. GENERATED from {qpath.name}.",
        "# '<suite.exe> <subtest>' per line; the sweep announces each as SKIP.",
        *(
            ["# QUARANTINE IGNORED (--ignore-quarantine): deliberately EMPTY."]
            if args.ignore_quarantine
            else []
        ),
        *armed_skips,
    ]
    skip_out.write_text("\n".join(skip_lines) + "\n", encoding="utf-8")

    stale_subtest_suites = sorted({e.split(" ", 1)[0] for e in skipped} - set(all_found))
    if stale_subtest_suites:
        print(
            f"gen_suites: {qpath.name} has subtest skips for suite(s) this build did "
            f"not produce: {', '.join(stale_subtest_suites)}",
            file=sys.stderr,
        )

    out = HERE / f"apitest-suites-{args.arch}.txt"
    lines = [
        f"# apitest conformance sweep list — {args.arch}. GENERATED by",
        "# tools/payloads/apitest/gen_suites.py from the binaries in",
        f"# {build_dir.name}/modules/rostests/apitests. Do not edit; to drop a suite,",
        f"# add it to {qpath.name} instead.",
    ]
    if args.only:
        lines += [
            "# RESTRICTED by --only to: " + ", ".join(sorted(args.only)),
            "# This is a REPRODUCTION list, not the full sweep — do not bank a baseline",
            "# from a run using it; the coverage would silently be one suite's worth.",
        ]
    if args.ignore_quarantine:
        lines += [
            f"# QUARANTINE IGNORED (--ignore-quarantine): {qpath.name} was not",
            "# subtracted, so known-wedging suites and subtests are ARMED. This is a",
            "# REPRODUCTION list — do not bank a baseline from a run using it.",
        ]
    if armed:
        lines.append(f"# {len(armed & set(found))} suite(s) quarantined via {qpath.name}.")
    lines += [s for s in found if s not in armed]
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")

    kept = len([s for s in found if s not in armed])
    # `built` counts what the BUILD produced, not what --only kept, or a restricted
    # run reports "1 suite(s) (1 built)" and reads like the build lost 62 of them.
    restricted = f", RESTRICTED to {len(found)} by --only" if args.only else ""
    if args.ignore_quarantine:
        restricted += ", QUARANTINE IGNORED"
    # One f-string for both shapes: the --only-specific `filtered` clause was
    # duplicated into two branches, and `restricted` only reached one of them, so
    # --ignore-quarantine on its own printed a summary that did not mention it.
    filtered = f"{len(all_found) - len(found)} filtered, " if args.only else ""
    print(
        f"gen_suites: {out.name} — {kept} suite(s) ({len(all_found)} built, "
        f"{filtered}{len(found) - kept} quarantined{restricted}), "
        f"{len(armed_skips)} subtest skip(s)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
