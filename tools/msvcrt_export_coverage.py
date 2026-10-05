#!/usr/bin/env python3
"""How much of msvcrt.dll's export table is Rust-backed, measured from the BINARIES.

WHY THIS IS A TOOL AND NOT A NUMBER IN A DOC
--------------------------------------------
`docs/engineering/rust-port-status.md` carries this figure in its summary table, and it
has been re-derived BY HAND three times (2026-07-14, 2026-07-26, 2026-08-07). Each time
it went stale within weeks — the last hand figure said 44% while the true value was 91%,
because eight waves landed in between and nobody re-ran the recipe that was written out
in prose beside it. A recipe you have to remember is a recipe that rots; this is the
command behind the rule.

WHAT IT MEASURES
----------------
For each export the DLL builds on the arch, which input of its REAL link line answers it:
`export_census.census()`, which reads the link line out of build.ninja and asks `llvm-nm`
which of those inputs defines the backing symbol, in resolution order. Rust-backed means a
Rust archive answered (msvcrt-crt-rust, or the pseh-rust / setjmp-rust user builds named
ahead of ntdll's import library).

Excluded from the denominator: spec2def stubs (no body anywhere), DLL forwards (another
module answers), and `?`-mangled C++ (a separate lane). Imported means another DLL's import
library answers the name; `memmove` is that on purpose (field manual §9.65: LLVM folds a
byte-copy loop back into a call to itself), anything else there is a misbinding to fix.

This tool used to resolve on its own: the spec without `-arch=` filtering (i386's x87 and
port-I/O surface counted as amd64 "still C") and only msvcrt-crt-rust's archive counted as
Rust. Once no C was linked at all (2026-10-04) its "still C: 83" was entirely those two
errors, so it now asks the census instead of being a second resolver.

FLEET ISOLATION: reads THIS worktree's build-<arch> only, and hard-fails if it is missing
or older than the Rust sources (the census's own checks).

Usage:
    python3 tools/msvcrt_export_coverage.py [--arch amd64] [--list]
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))
import export_census  # noqa: E402

#: Every CRT DLL that links msvcrt-crt-rust (dll/win32/msvcrt/msvcrt_rust_cohab.cmake).
#: msvcrt20/msvcrt40/crtdll are i386-only.
DLLS = ("msvcrt", "msvcrt20", "msvcrt40", "crtdll")

#: Imported names that are deliberate, with why.
DELIBERATE_IMPORTS = {"memmove": "deliberately NOT Rust — field manual §9.65"}


def classify(label: str) -> str:
    """One census bucket label -> rust / imported / excluded / C."""
    if label == "rust" or (label.startswith("archive: ") and "_rust" in label):
        return "rust"
    if label in ("stub", "forward"):
        return "excluded"
    if label in ("archive: libntdll.a", "archive: libkernel32.a"):
        return "imported"
    return "C"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", default="amd64", choices=["i386", "amd64", "arm64"])
    ap.add_argument("--list", action="store_true", help="print the names in each non-Rust group")
    ap.add_argument("--dll", default="msvcrt", choices=DLLS, help="which CRT DLL to measure")
    args = ap.parse_args()

    if not (REPO / f"build-{args.arch}/dll/win32/{args.dll}/{args.dll}.dll").is_file():
        sys.exit(
            f"FAIL: {args.dll} is not built for {args.arch} (msvcrt20/msvcrt40/crtdll are "
            f"i386-only).\nfix: ./vibeos-cli build --arch {args.arch}"
        )
    r = export_census.census(args.dll, args.arch)
    groups: dict[str, list[tuple[str, str]]] = {"rust": [], "imported": [], "excluded": [], "C": []}
    cxx = 0
    for label, names in r["buckets"].items():
        for name, member in names:
            if name.startswith("?"):
                cxx += 1
                continue
            groups[classify(label)].append((name, f"{label} {member}".strip()))
    if "UNRESOLVED" in r["buckets"]:
        sys.exit("FAIL: the census could not attribute every export — run tools/export_census.py")

    denom = len(groups["rust"]) + len(groups["imported"]) + len(groups["C"])
    if not denom:
        sys.exit("FAIL: 0 implemented exports — the spec parser is broken")
    print(f"{args.dll}.dll export coverage ({args.arch})")
    print(
        f"  {len(groups['rust'])} of {denom} implemented exports Rust-backed "
        f"— {100 * len(groups['rust']) // denom}%"
    )
    print(f"  excluded: {len(groups['excluded'])} stubs/forwards, {cxx} C++-mangled")
    misbound = [n for n, _ in groups["imported"] if n not in DELIBERATE_IMPORTS]
    for name, _ in sorted(groups["imported"]):
        why = DELIBERATE_IMPORTS.get(name, "MISBOUND: another DLL's import library answers it")
        print(f"  imported: {name} ({why})")
    print(f"  still C: {len(groups['C'])}")
    if args.list:
        for name, where in sorted(groups["C"]):
            print(f"    {name}  <- {where}")
    # Loud on purpose: either one is a binding the Rust does not own and nobody chose.
    return 1 if misbound or groups["C"] else 0


if __name__ == "__main__":
    sys.exit(main())
