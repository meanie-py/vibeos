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

WHAT IT MEASURES, AND WHY EACH EXCLUSION IS THERE
-------------------------------------------------
The spec file is the export table. For each entry it asks: does the symbol the DLL will
actually bind come from the Rust archive (or from the C `va_start` shim TUs, which are
part of the flip — pinned-stable Rust cannot define a C-variadic function)?

Three kinds of entry are excluded, and each one was a real miscount in the first draft:

* `-stub` — `@ varargs -stub _cprintf_s(str)` puts `-stub` in the FLAGS, not the
  directive, so a parser that only checks `w[1] == "stub"` counts 20 unimplemented
  exports as implemented AND lists them as "still C". msvcrt has no body for these at
  all; they are not translation debt.
* DLL forwards — `@ cdecl _getdrives() kernel32.GetLogicalDrives` is resolved by the
  loader against another module. msvcrt contributes nothing, so it is neither Rust nor C.
* `?`-mangled C++ — a separate lane with its own denominator, as the status doc says.

The oracle is `llvm-nm` on the built archive and objects, never the Rust source: a body
that exists but is exported under the wrong name is the failure this whole family of
tools exists to catch (see tools/verify_cohab_coverage.py's header for the three that
shipped that way).

FLEET ISOLATION: reads THIS worktree's build-<arch> only, and hard-fails if it is
missing or older than the Rust sources. A coverage figure derived from a sibling lane's
build dir, or from an archive that predates the flip being measured, is worse than no
figure — it reads as evidence.

Usage:
    python3 tools/msvcrt_export_coverage.py [--arch amd64] [--list]
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
NM = REPO / ".build-tools/llvm-mingw/bin/llvm-nm"
SPEC = REPO / "dll/win32/msvcrt/msvcrt.spec"
RUST_SRC = REPO / "sdk/lib/rust/msvcrt-crt-rust/src"

#: Where the still-C remainder comes from, so the output says WHOSE lane each one is
#: rather than listing 106 names and leaving the reader to group them. Order matters:
#: first match wins, and the prefix rules are deliberately ahead of the name sets.
CATEGORIES: list[tuple[str, object]] = [
    ("crtmath: x87 _CI* intrinsics (asm)", lambda s: s.startswith("_CI")),
    ("crtmath: __libm_sse2_* (asm)", lambda s: s.startswith("__libm_sse2_")),
    (
        "i386 FDIV/FPU errata fixups (asm)",
        lambda s: s.startswith(("_adj_", "_safe_f")) or s == "_adjust_fdiv",
    ),
    ("_ftol family (asm)", lambda s: s.startswith("_ftol")),
    (
        "crtmath: float/bessel/misc libm",
        lambda s: (
            s
            in {
                "cosf",
                "expf",
                "fmodf",
                "log10f",
                "logf",
                "powf",
                "sinf",
                "_copysign",
                "_copysignf",
                "_j0",
                "_j1",
                "_jn",
                "_y0",
                "_y1",
                "_yn",
                "frexp",
                "ldiv",
                "_fpclass",
            }
        ),
    ),
    ("dbgrpt (_Crt*Report*)", lambda s: s.startswith("_Crt")),
    (
        "SEH / setjmp (asm + except_i386)",
        lambda s: (
            s
            in {
                "_EH_prolog",
                "__CxxFrameHandler3",
                "__CxxLongjmpUnwind",
                "_except_handler2",
                "_except_handler3",
                "_except_handler4_common",
                "_global_unwind2",
                "_local_unwind2",
                "_local_unwind4",
                "_longjmpex",
                "_seh_longjmp_unwind",
                "_seh_longjmp_unwind4",
                "_setjmp",
                "_setjmp3",
                "_setjmpex",
                "longjmp",
                "setjmp",
            }
        ),
    ),
    (
        "port I/O (i386/inout.s)",
        lambda s: s in {"_inp", "_inpd", "_inpw", "_outp", "_outpd", "_outpw"},
    ),
    # Not an omission: LLVM folds any byte-copy loop back into a call to the function
    # itself, so a hand-written Rust memmove is infinite self-recursion (field manual
    # §9.65). msvcrt's comes from the C/asm on purpose.
    ("memmove (deliberately NOT Rust — §9.65)", lambda s: s == "memmove"),
]


def defined(path: Path) -> set[str]:
    """Global symbols an archive or object DEFINES, undecorated of `@N`."""
    out = subprocess.run(
        [str(NM), "--defined-only", str(path)], capture_output=True, text=True
    ).stdout
    syms = set()
    for line in out.splitlines():
        parts = line.split()
        if len(parts) >= 3 and parts[1] in ("T", "D", "B", "R"):
            syms.add(parts[2].split("@")[0])
    return syms


def parse_spec() -> tuple[list[tuple[str, str]], int, int, int]:
    impls: list[tuple[str, str]] = []
    stubs = forwards = cxx = 0
    for raw in SPEC.read_text().splitlines():
        line = raw.split("#")[0].strip()
        if not line:
            continue
        words = line.split()
        if len(words) < 2:
            continue
        if words[0] != "@" and not words[0].isdigit():
            continue
        directive = words[1]
        flags = [w for w in words[2:] if w.startswith("-")]
        rest = [w for w in words[2:] if not w.startswith("-")]
        if not rest:
            continue
        export = rest[0].split("(")[0]
        if directive == "stub" or "-stub" in flags:
            stubs += 1
            continue
        if export.startswith("?"):
            cxx += 1
            continue
        impl = rest[-1].split("(")[0] if len(rest) > 1 else export
        if "." in impl:
            forwards += 1
            continue
        impls.append((export, impl))
    return impls, stubs, forwards, cxx


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--arch", default="amd64", choices=["i386", "amd64", "arm64"])
    ap.add_argument("--list", action="store_true", help="print the still-C names per category")
    args = ap.parse_args()

    build = REPO / f"build-{args.arch}/dll/win32/msvcrt"
    archive = build / "libmsvcrt_crt_cargo.a"
    if not archive.is_file():
        sys.exit(
            f"FAIL: {archive.relative_to(REPO)} not built — a coverage figure with no "
            f"archive behind it is a guess.\nfix: ./vibeos-cli build --arch {args.arch}"
        )
    # Staleness is the failure this refuses to produce quietly: an archive older than the
    # Rust it is supposed to contain reports the PREVIOUS wave's coverage as this one's.
    newest = max((p.stat().st_mtime for p in RUST_SRC.rglob("*.rs")), default=0.0)
    if newest > archive.stat().st_mtime:
        sys.exit(
            f"FAIL: {archive.relative_to(REPO)} is OLDER than msvcrt-crt-rust's sources — "
            f"the figure would be the previous wave's.\nfix: ./vibeos-cli build --arch {args.arch}"
        )

    rust = defined(archive)
    # The `...` entry points: C trampolines in the MODULE target over Rust `_v*` twins.
    # They are part of the flip — stable Rust cannot define a C-variadic function — so an
    # export they carry is Rust-backed even though the symbol lives in a .c.obj.
    shims = sorted((build / "CMakeFiles/msvcrt.dir").glob("rust_*shim.c.obj"))
    for obj in shims:
        rust |= defined(obj)

    impls, stubs, forwards, cxx = parse_spec()
    if not impls:
        sys.exit("FAIL: parsed 0 implemented exports out of msvcrt.spec — the parser is broken")
    backed = [e for e, i in impls if i in rust or e in rust]
    missing = sorted(e for e, i in impls if not (i in rust or e in rust))

    print(f"msvcrt.dll export coverage ({args.arch}, {len(shims)} varargs shim TUs)")
    print(
        f"  {len(backed)} of {len(impls)} implemented C exports Rust-backed "
        f"— {100 * len(backed) // len(impls)}%"
    )
    print(f"  excluded: {stubs} stubs, {forwards} DLL forwards, {cxx} C++-mangled")
    print(f"  still C: {len(missing)}")

    seen: set[str] = set()
    for label, pred in CATEGORIES:
        hits = [s for s in missing if pred(s) and s not in seen]
        seen |= set(hits)
        if hits:
            print(f"    {len(hits):3}  {label}")
            if args.list:
                print(f"         {' '.join(hits)}")
    rest = [s for s in missing if s not in seen]
    if rest:
        # Loud on purpose: an uncategorised name is either a new still-C export or a
        # category that has gone stale, and both want a human rather than a silent tail.
        print(f"    {len(rest):3}  UNCATEGORISED — new still-C exports? {' '.join(rest)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
