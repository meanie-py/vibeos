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
#: Every CRT DLL built from msvcrt_static that links the Rust cohab
#: (dll/win32/msvcrt/msvcrt_rust_cohab.cmake). msvcrt20/msvcrt40/crtdll are i386-only.
DLLS = ("msvcrt", "msvcrt20", "msvcrt40", "crtdll")
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


def defined(path: Path, i386: bool = False) -> set[str]:
    """Global symbols an archive or object DEFINES, undecorated of `@N` -- and, on
    i386, of the one leading underscore every C symbol carries there. Without that
    the i386 figure read ~0% (`_strlen` never matched `strlen`); nobody saw it because
    the tool had only ever been run for its amd64 default."""
    out = subprocess.run(
        [str(NM), "--defined-only", str(path)], capture_output=True, text=True
    ).stdout
    syms = set()
    for line in out.splitlines():
        parts = line.split()
        if len(parts) >= 3 and parts[1] in ("T", "D", "B", "R"):
            name = parts[2].split("@")[0]
            if i386 and name.startswith("_"):
                name = name[1:]
            syms.add(name)
    return syms


def parse_spec(spec: Path) -> tuple[list[tuple[str, str]], int, int, int]:
    impls: list[tuple[str, str]] = []
    stubs = forwards = cxx = 0
    for raw in spec.read_text().splitlines():
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
    ap.add_argument(
        "--dll",
        default="msvcrt",
        choices=DLLS,
        help="which CRT DLL's export table to measure (all link the same Rust archive)",
    )
    args = ap.parse_args()

    build = REPO / f"build-{args.arch}/dll/win32/msvcrt"
    dll_build = REPO / f"build-{args.arch}/dll/win32/{args.dll}"
    if not (dll_build / "CMakeFiles" / f"{args.dll}.dir").is_dir():
        sys.exit(
            f"FAIL: {args.dll} is not built for {args.arch} (msvcrt20/msvcrt40/crtdll are "
            f"i386-only).\nfix: ./vibeos-cli build --arch {args.arch}"
        )
    archive = build / "libmsvcrt_crt_cargo.a"
    if not archive.is_file():
        sys.exit(
            f"FAIL: {archive.relative_to(REPO)} not built — a coverage figure with no "
            f"archive behind it is a guess.\nfix: ./vibeos-cli build --arch {args.arch}"
        )
    # Staleness is the failure this refuses to produce quietly: an archive older than the
    # Rust it is supposed to contain reports the PREVIOUS wave's coverage as this one's.
    # EVERY crate compiled into the archive, not just msvcrt-crt-rust: the string family
    # lives in crt-portable-rust, and a watch on one crate reported a stale archive fresh.
    sys.path.insert(0, str(REPO / "tools"))
    import verify_ucrtbase_cohab as vuc
    from source_walk import source_files

    crates = vuc._closure("msvcrt-crt-cargo")
    newest = max(
        (p.stat().st_mtime for c in crates for p in source_files(vuc.WS / c / "src")),
        default=0.0,
    )
    if newest > archive.stat().st_mtime:
        sys.exit(
            f"FAIL: {archive.relative_to(REPO)} is OLDER than msvcrt-crt-rust's sources — "
            f"the figure would be the previous wave's.\nfix: ./vibeos-cli build --arch {args.arch}"
        )

    i386 = args.arch == "i386"
    rust = defined(archive, i386)
    # Every object comes from the DLL's REAL link line, never a directory glob: ninja
    # leaves objects behind when a source moves. The `...` entry points used to be C
    # trampolines in a separate object library (msvcrt_rust_shims) and were counted from
    # there; since f1658ab724d they are va_entry! thunks inside the Rust archive itself,
    # so `rust` above already holds them.
    import ucrt_tier2_refset as refset

    link = refset.link_line(REPO / f"build-{args.arch}", args.dll).split()
    objs = [REPO / f"build-{args.arch}" / t for t in link if t.endswith(".obj")]

    # A DLL's OWN module objects (crtdll's dllmain.c, msvcrt20.c) come first on its link
    # line and win first-match over the archive, so a name they define is answered by
    # that C whatever the archive also holds. The spec-generated stubs are excluded:
    # they back only `-stub` entries, which are not counted.
    own: set[str] = set()
    for obj in objs:
        if f"{args.dll}.dir" in str(obj) and not obj.name.endswith("_stubs.c.obj"):
            own |= defined(obj, i386)

    spec = REPO / f"dll/win32/{args.dll}/{args.dll}.spec"
    impls, stubs, forwards, cxx = parse_spec(spec)
    if not impls:
        sys.exit(f"FAIL: parsed 0 implemented exports out of {spec.name} — the parser is broken")

    def is_rust(e: str, i: str) -> bool:
        return (i in rust or e in rust) and not (i in own or e in own)

    backed = [e for e, i in impls if is_rust(e, i)]
    missing = sorted(e for e, i in impls if not is_rust(e, i))

    print(f"{args.dll}.dll export coverage ({args.arch})")
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
