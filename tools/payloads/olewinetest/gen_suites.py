#!/usr/bin/env python3
"""Generate the olewinetest sweep list: one `<suite>.exe <subtest>` line per Wine test.

Subtests come from each suite's testlist.c (`{ "name", func_name }`), the same
table the binary dispatches on. --only narrows to SUITE[:Subtest]; --skip drops
SUITE:Subtest (a known crasher that would blind every test after it).

    python3 tools/payloads/olewinetest/gen_suites.py --arch amd64 [--only oleaut32:vartype]

Other payloads reuse it for their own suite sets: --suites picks the suites,
--register the DLLs registered before they run, --out-dir where the lists go
(mfwinetest's build_hint is the example).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[2]
SUITES = ("oleaut32", "ole32")
# Register first (see payload.yaml): the sweep runs `<bin>\\<exe> <rest>`.
# windowscodecs: OleLoadPicture decodes through WIC (CLSID_WICImagingFactory).
# urlmon / shell32: ole32's compobj and marshal suites CoCreateInstance
# CLSID_InternetZoneManager and CLSID_ShellDesktop. An installed system has them
# registered by setup; the live CD does not, and the tests release the NULL
# that REGDB_E_CLASSNOTREG leaves behind.
REGISTER = ("ole32", "oleaut32", "windowscodecs", "urlmon", "shell32")
START_RE = re.compile(r"^START_TEST\((\w+)\)", re.M)
ENTRY_RE = re.compile(r'^\s*\{\s*"([A-Za-z0-9_]+)"\s*,\s*func_\w+\s*\}', re.M)


def subtests(suite: str) -> list[str]:
    src = REPO / "modules" / "rostests" / "winetests" / suite
    if (src / "testlist.c").is_file():
        return ENTRY_RE.findall((src / "testlist.c").read_text())
    # A single-file suite built STANDALONE (rtworkq) has no testlist.c: its one
    # subtest is its START_TEST.
    return sorted(m for c in src.glob("*.c") for m in START_RE.findall(c.read_text()))


def parse_pairs(values: list[str] | None) -> list[tuple[str, str]]:
    out = []
    for v in values or []:
        suite, _, sub = v.partition(":")
        out.append((suite.removesuffix("_winetest.exe").removesuffix("_winetest"), sub))
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", required=True, choices=("i386", "amd64", "arm64"))
    ap.add_argument("--only", action="append", metavar="SUITE[:Subtest]")
    ap.add_argument("--skip", action="append", metavar="SUITE:Subtest")
    ap.add_argument(
        "--global-flag",
        action="append",
        metavar="SUITE=HEX",
        help="set the Image File Execution Options GlobalFlag for SUITE_winetest.exe before "
        "the suites run (staged reg.exe), e.g. oleaut32=0x02000000 for page heap: a heap "
        "corruption then faults AT the bad store instead of at a later free",
    )
    ap.add_argument(
        "--repeat",
        action="append",
        metavar="SUITE:Subtest=N",
        help="run that subtest N times in a row in ONE boot, e.g. mf:mf=3 for a flake that "
        "only shows in a later run of the same process tree (smoke --repeat boots afresh)",
    )
    ap.add_argument(
        "--suites", help="comma-separated suites (default: %(default)s)", default=",".join(SUITES)
    )
    ap.add_argument(
        "--register",
        help="comma-separated DLLs to regsvr32 first (default: %(default)s)",
        default=",".join(REGISTER),
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=HERE,
        help="where winetest-suites-<arch>.txt goes (default: this payload)",
    )
    args = ap.parse_args(argv)
    suites = tuple(s for s in args.suites.split(",") if s)
    register = tuple(s for s in args.register.split(",") if s)
    tag = args.out_dir.name

    build = REPO / f"build-{args.arch}" / "modules" / "rostests" / "winetests"
    missing = [s for s in suites if not (build / s / f"{s}_winetest.exe").is_file()]
    if missing:
        print(
            f"gen_suites: {', '.join(missing)} not built under {build}.\n"
            f"  fix: ./vibeos-cli build --arch {args.arch} --rostests --keep-going",
            file=sys.stderr,
        )
        return 1

    table = {s: subtests(s) for s in suites}
    only = parse_pairs(args.only)
    for suite, sub in only + parse_pairs(args.skip):
        if suite not in table or (sub and sub not in table[suite]):
            print(f"gen_suites: no such suite/subtest {suite}:{sub}", file=sys.stderr)
            return 1
    skip = {(s, t) for s, t in parse_pairs(args.skip)}
    repeat: dict[tuple[str, str], int] = {}
    for spec in args.repeat or []:
        pair, _, count = spec.partition("=")
        suite, sub = parse_pairs([pair])[0]
        if suite not in table or sub not in table[suite] or not count.isdigit() or int(count) < 1:
            print(f"gen_suites: bad --repeat {spec!r} (want SUITE:Subtest=N)", file=sys.stderr)
            return 1
        repeat[(suite, sub)] = int(count)

    lines = [
        f"# {tag} sweep list - {args.arch}. GENERATED by",
        "# tools/payloads/olewinetest/gen_suites.py from each suite's testlist.c.",
    ]
    if only:
        lines.append("# RESTRICTED by --only to: " + ", ".join(args.only))
    if repeat:
        lines.append("# REPEATED by --repeat: " + ", ".join(args.repeat))
    ifeo = "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options"
    for gf in args.global_flag or []:
        suite, _, flags = gf.partition("=")
        suite = suite.removesuffix("_winetest.exe").removesuffix("_winetest")
        if suite not in suites or not flags:
            print(f"gen_suites: bad --global-flag {gf!r} (want SUITE=HEX)", file=sys.stderr)
            return 1
        lines.append(
            f'reg.exe add "{ifeo}\\{suite}_winetest.exe" /v GlobalFlag /t REG_DWORD /d {flags} /f'
        )
        lines.append("# GlobalFlag " + flags + " on " + suite + ": a DEBUG run, not a baseline")
    lines += [f"regsvr32.exe /s {dll}.dll" for dll in register]
    n = 0
    for suite, subs in table.items():
        for sub in subs:
            if only and not any(s == suite and (not t or t == sub) for s, t in only):
                continue
            if (suite, sub) in skip:
                continue
            times = repeat.get((suite, sub), 1)
            lines += [f"{suite}_winetest.exe {sub}"] * times
            n += times
    (args.out_dir / f"winetest-suites-{args.arch}.txt").write_text("\n".join(lines) + "\n")
    (args.out_dir / f"winetest-skip-{args.arch}.txt").write_text(
        "# none: skips are applied when the list is generated\n"
    )
    print(f"gen_suites: {n} subtest(s) -> winetest-suites-{args.arch}.txt")
    return 0


if __name__ == "__main__":
    sys.exit(main())
