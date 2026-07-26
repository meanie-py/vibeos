#!/usr/bin/env python3
"""Emit prototypes for every function a msvcrt reference TU defines.

A reference TU is the GENUINE dll/win32/msvcrt/*.c compiled for the host with
its public symbols renamed to `ref_*`.  In the real build those functions are
prototyped by msvcrt's own <stdio.h>/<io.h>/<direct.h>; the host harness
suppresses those headers (they cannot share a TU with glibc's), so any function
file.c calls ABOVE its own definition has no declaration in scope.

C's implicit-declaration rule then invents `int()`.  That is not a compile
error by default and not a link error ever — it is a 64-bit return truncated to
32 bits (`_ftelli64_nolock`, `_wfdopen`, every `wchar_t*` returner) in code that
runs.  file.c has 65 such calls.  The harness compiles with
-Werror=implicit-function-declaration precisely so they cannot pass silently,
and this generator is what satisfies it.

Prototypes are derived from each function's OWN DEFINITION in the source, so
they cannot disagree with it: if a signature changes, the generated prototype
changes with it, and if this parser ever fails to see a definition the symbol is
reported rather than skipped.  They are emitted in the source's own spelling
(`_fread_nolock`, not `ref__fread_nolock`) and included AFTER build.rs's rename
wall, so the same `#define` that renames the definition renames the prototype.

Usage:
    python3 tools/gen_msvcrt_ref_protos.py file.c            # write the header
    python3 tools/gen_msvcrt_ref_protos.py file.c --check    # fail on drift
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
MSVCRT = REPO / "dll/win32/msvcrt"
NM = REPO / ".build-tools/llvm-mingw/bin/llvm-nm"
REFSTUB = REPO / "sdk/lib/rust/msvcrt-difftest/refstub"


# Where the built object for each translation unit lives.  THIS worktree only —
# resolving a sibling's build dir would prototype against another lane's commit
# (fleet isolation law; the verify_signatures incident).
def obj_for(stem: str) -> Path:
    return REPO / (f"build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/{stem}.c.obj")


def public_symbols(stem: str) -> list[str]:
    """The TU's externally-visible function symbols, from the built object."""
    obj = obj_for(stem)
    if not obj.exists():
        sys.exit(f"FAIL: {obj.relative_to(REPO)} not built.\nfix: ./vibeos-cli build --arch amd64")
    out = subprocess.run(
        [str(NM), "--defined-only", str(obj)], capture_output=True, text=True
    ).stdout
    syms = {
        p[2].split("@")[0]
        for p in (ln.split() for ln in out.splitlines())
        if len(p) == 3 and p[1] == "T"
    }
    return sorted(syms)


# `<return type> [CDECL|WINAPIV|…] <name>(<params>)` followed by `{`.
#
# The calling convention is OPTIONAL: msvcrt's internal helpers
# (msvcrt_init_io &c.) carry none, and the variadic entry points use WINAPIV
# rather than CDECL — missing that spelling is what left `_open` and the eight
# `_fprintf_*` shells unparsed on the first pass.  Anything still unmatched is
# REPORTED, never skipped: a silently-missing prototype is the exact bug this
# file exists to prevent.
DEF_RE = r"""
    ^                          # a definition starts a line
    (?P<ret>[A-Za-z_][^;{{}}()\n]*?)   # return type, with * and qualifiers
    \s*
    (?:\b(?P<cc>CDECL|WINAPIV|WINAPI|__cdecl)\s+)?
    (?<![A-Za-z0-9_])          # a real word boundary: without this, `_stat`
                               # matches the tail of `msvcrt_stat64_to_stat`
                               # and takes the prefix as a return type
    (?P<name>{name})\s*
    \((?P<params>[^;{{}}]*?)\)\s*
    \{{                        # the body — a definition, not a declaration
"""


def unrenamable(stem: str, symbols: list[str]) -> set[str]:
    """Symbols a `#define <n> ref_<n>` wall cannot rename.  Two causes, both real.

    1. **The symbol is also a struct tag.**  `_stat` names both a function and
       `struct _stat`, so the wall rewrites the TYPE as well; `struct ref__stat`
       does not exist and the TU stops compiling.  Loud, at least.

    2. **The TU `#undef`s it.**  file.c opens with 18 of them (`#undef _fstat`,
       `#undef _stat64`, …, plus `#undef __iob_func` further down) because in
       the real build those names are time_t-width macros from msvcrt's
       headers.  Each `#undef` silently DELETES our rename, and the function is
       then defined under its true name.  This one is dangerous in a way (1) is
       not: nothing fails at compile time.  The reference TU simply emits
       `_fstat64` instead of `ref__fstat64`, which either collides with the
       subject's definition or leaves the harness's `ref__fstat64` undefined —
       10 of file.c's 189 symbols leaked exactly this way before this function
       existed.

    Both get an asm label on the declaration instead: that renames the SYMBOL at
    the object level, touches no macro or tag namespace, and `#undef` cannot
    reach it.
    """
    src = (MSVCRT / f"{stem}.c").read_text(errors="replace")
    tags = set(re.findall(r"\bstruct\s+([A-Za-z_]\w*)", src))
    undefd = set(re.findall(r"^\s*#\s*undef\s+([A-Za-z_]\w*)", src, re.M))
    return {s for s in symbols if s in tags or s in undefd}


def prototypes(stem: str, symbols: list[str], colliding: set[str]) -> tuple[list[str], list[str]]:
    src = (MSVCRT / f"{stem}.c").read_text(errors="replace")
    found, missing = [], []
    for sym in symbols:
        m = re.search(DEF_RE.format(name=re.escape(sym)), src, re.M | re.S | re.X)
        if not m:
            missing.append(sym)
            continue
        ret = " ".join(m.group("ret").split())
        params = " ".join(m.group("params").split()) or "void"
        cc = f"{m.group('cc')} " if m.group("cc") else ""
        # A collider is NOT renamed by the wall (build.rs skips it), so carry
        # the rename here, on the symbol only.
        label = f' __asm__("ref_{sym}")' if sym in colliding else ""
        found.append(f"{ret} {cc}{sym}({params}){label};")
    return found, missing


def render_skiplist(stem: str, colliding: set[str]) -> str:
    """The names build.rs must NOT put in the `#define` rename wall."""
    return "\n".join(
        [
            f"# GENERATED by tools/gen_msvcrt_ref_protos.py from {stem}.c — DO NOT EDIT.",
            "#",
            "# Symbols a `#define <n> ref_<n>` rename wall cannot rename: either the",
            "# name is also a struct tag (the wall would rewrite the TYPE), or the TU",
            "# `#undef`s it (which silently DELETES the rename). See unrenamable().",
            f"# msvcrt_{stem}_protos.h carries their rename as an asm label instead;",
            "# build.rs must skip exactly these when emitting the wall.",
            *sorted(colliding),
            "",
        ]
    )


def render(stem: str, protos: list[str], missing: list[str]) -> str:
    out = [
        f"/* GENERATED by tools/gen_msvcrt_ref_protos.py from {stem}.c — DO NOT EDIT.",
        " *",
        f" * A prototype for every function {stem}.c defines, in the source's own",
        " * spelling, to be included AFTER build.rs's rename wall so each one is",
        " * renamed alongside the definition it describes.",
        " *",
        " * Without these, every call placed above its definition falls to C's",
        " * implicit-declaration rule and becomes `int()` — a 64-bit return silently",
        " * truncated to 32 bits, in code that compiles and links.  The harness's",
        " * -Werror=implicit-function-declaration is what makes that loud; this file",
        " * is what answers it.",
        " *",
        f" * Regenerate:  python3 tools/gen_msvcrt_ref_protos.py {stem}.c",
        " */",
        f"#ifndef MSVCRT_DIFFTEST_{stem.upper()}_PROTOS_H",
        f"#define MSVCRT_DIFFTEST_{stem.upper()}_PROTOS_H",
        "",
    ]
    out += protos
    if missing:
        out += [
            "",
            "/* Symbols whose definition this generator could not parse.  Listed,",
            " * not silently dropped — each is a prototype the reference TU is",
            " * missing, so if the build then complains about an implicit",
            " * declaration, the name will be here. */",
        ]
        out += [f"/*   UNPARSED: {m} */" for m in missing]
    out += ["", f"#endif /* MSVCRT_DIFFTEST_{stem.upper()}_PROTOS_H */", ""]
    return "\n".join(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("source", help="the msvcrt .c file, e.g. file.c")
    ap.add_argument(
        "--check",
        action="store_true",
        help="fail if the committed header differs from a fresh derivation",
    )
    args = ap.parse_args()

    stem = Path(args.source).stem
    if not (MSVCRT / f"{stem}.c").exists():
        sys.exit(f"FAIL: no such source: dll/win32/msvcrt/{stem}.c")

    syms = public_symbols(stem)
    colliding = unrenamable(stem, syms)
    protos, missing = prototypes(stem, syms, colliding)

    outputs = [
        (REFSTUB / f"msvcrt_{stem}_protos.h", render(stem, protos, missing)),
        (REFSTUB / f"msvcrt_{stem}_no_rename.txt", render_skiplist(stem, colliding)),
    ]

    if args.check:
        stale = [p for p, text in outputs if not p.exists() or p.read_text() != text]
        if stale:
            for p in stale:
                print(f"DRIFT: {p.relative_to(REPO)} is stale or missing")
            print(f"fix: python3 tools/gen_msvcrt_ref_protos.py {stem}.c")
            return 1
        print(f"ok: {len(protos)} prototypes match a fresh derivation")
        return 0

    for p, text in outputs:
        p.write_text(text)
    note = f", {len(missing)} UNPARSED" if missing else ""
    print(f"wrote {outputs[0][0].relative_to(REPO)} ({len(protos)} of {len(syms)} symbols{note})")
    print(
        f"wrote {outputs[1][0].relative_to(REPO)} "
        f"({len(colliding)} unrenamable symbols, renamed by asm label)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
