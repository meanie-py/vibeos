#!/usr/bin/env python3
"""verify_conv_macros — keep abi_enforce's calling-convention table honest.

WHY THIS EXISTS
---------------
rustconv derives a translated fn's calling convention from the RAW C text (it does not
run the preprocessor). So it must recognise every macro that *aliases* a convention:

    SOCKET
    WSAAPI                       <- #define WSAAPI FAR PASCAL   (winsock2.h:578)
    socket(int af, ...)          <- #define PASCAL WINAPI -> __stdcall

If `WSAAPI` is not in `abi_enforce._KW_TO_CONV`, `_derive_convention` does not report
"unknown" — it silently returns **cdecl**. On i386 that is a stack imbalance on every
call the OS makes through the fn pointer, and it compiles, links, and boots. This is the
same bug class that shipped `font_enum_proc`/`property_sheet_proc` as cdecl in oleaut32
(the `CALLBACK`/`PASCAL` gap, 2026-07) — the fix was a bigger table, and the *lesson* was
that the table needs a guard, because the headers keep having more of these than anyone
remembers. There are 59.

Aliases chain, so a single-level grep is not enough:
    WSPAPI -> WSAAPI -> FAR PASCAL -> WINAPI -> __stdcall
This resolves the transitive closure from the real headers and diffs it against the table.

USAGE
-----
    verify_conv_macros.py            # --check: exit 1 if the table is missing an alias
    verify_conv_macros.py --emit     # print the resolved table (to paste into abi_enforce)

Wired into `vibeos-cli check`. If it fails, a header grew a new convention alias: add it
to `abi_enforce._KW_TO_CONV` (with its file:line citation) rather than silencing this.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))

# What the compiler actually sees — the roots of every alias chain.
ROOTS: dict[str, str] = {
    "__stdcall": "system",
    "_stdcall": "system",
    "__cdecl": "cdecl",
    "_cdecl": "cdecl",
    "__fastcall": "fastcall",
    "_fastcall": "fastcall",
}

_DEF = re.compile(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+?)\s*$")
# RHS decoration that is not a convention: `FAR PASCAL`, `EXTERN_C HRESULT STDAPICALLTYPE`,
# `__declspec(dllimport) WINAPI`. Strip it so the convention token is reachable.
_NOISE = re.compile(
    r"\b(FAR|NEAR|EXTERN_C|extern|DECLSPEC_\w+)\b|__declspec\([^)]*\)|/\*.*?\*/|//.*$"
)

# Vendored third-party libs: we do NOT translate these, so their aliases are noise in
# the table. Excluded from --check so a zlib bump can't fail our gate; still shown by --emit.
#
# The membership question is answered by vendor.yaml, NOT by a list here. It used to be a
# hardcoded 6-tuple covering 5 of the 19 directories under sdk/include/reactos/libs/ (it
# missed mbedtls's 78 headers, libmpg123's 38, libtiff's 15, fullfat's 19) with one stale
# entry, `libs/libxml`, matching nothing at all.
#
# HONEST SCOPE: switching to the registry widened the excluded set from 5 trees to 23 and
# dropped ZERO aliases from the table — measured, not assumed. 5,749 `#define`s now sit
# inside registered trees and not one of them resolves to a calling convention. So this is
# a GUARD, not an active filter: its job is the day an mbedtls or libtiff bump introduces
# some `FOOAPI` alias, which would otherwise show up as MISSING and get "fixed" by adding a
# foreign library's macro to abi_enforce's table of conventions WE translate against.
# A local list of what is foreign drifts because nothing tells it when a library arrives;
# the registry has a container ratchet that does. See tools/vendored.py.
sys.path.insert(0, str(REPO / "tools"))
import vendored  # noqa: E402


def harvest(root: Path) -> dict[str, tuple[str, str]]:
    """alias -> (convention, "path:line"), transitively resolved."""
    defs: dict[str, tuple[list[str], str, bool]] = {}
    registry = vendored.load(root)
    for hdr_dir in (root / "sdk" / "include" / "psdk", root / "sdk" / "include" / "reactos"):
        if not hdr_dir.is_dir():
            continue
        for h in sorted(hdr_dir.rglob("*.h")):
            rel = h.relative_to(root).as_posix()
            is_vendored = registry.is_vendored(rel)
            try:
                lines = h.read_text(errors="replace").splitlines()
            except OSError:
                continue
            for i, line in enumerate(lines, 1):
                m = _DEF.match(line)
                if not m:
                    continue
                name, rhs = m.group(1), m.group(2)
                rhs = _NOISE.sub(" ", rhs).strip()
                toks = [t for t in re.split(r"\s+", rhs) if t]
                if not toks:
                    continue
                # First definition wins: headers re-#define under #ifdef guards, and the
                # first is the one the normal (non-exotic-arch) build sees.
                defs.setdefault(name, (toks, f"{rel}:{i}", is_vendored))

    def resolve(name: str, seen: frozenset[str] = frozenset()) -> str | None:
        if name in ROOTS:
            return ROOTS[name]
        if name in seen or name not in defs:
            return None
        for tok in defs[name][0]:
            got = resolve(tok, seen | {name})
            if got:
                return got
        return None

    out: dict[str, tuple[str, str]] = {}
    for name, (_toks, cite, from_vendored) in defs.items():
        conv = resolve(name)
        if conv and not from_vendored:
            out[name] = (conv, cite)
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--emit", action="store_true", help="print the resolved table instead of checking"
    )
    args = ap.parse_args(argv)

    found = harvest(REPO)

    if args.emit:
        by_conv: dict[str, list[tuple[str, str]]] = {}
        for name, (conv, cite) in sorted(found.items()):
            by_conv.setdefault(conv, []).append((name, cite))
        for conv in ("system", "cdecl", "fastcall"):
            print(f"    # --- {conv} ---")
            for name, cite in by_conv.get(conv, []):
                print(f'    "{name}": "{conv}",  # {cite}')
        return 0

    from rustconv.abi_enforce import _KW_TO_CONV

    missing = sorted(n for n in found if n not in _KW_TO_CONV)
    wrong = sorted(
        (n, _KW_TO_CONV[n], found[n][0])
        for n in found
        if n in _KW_TO_CONV and _KW_TO_CONV[n] != found[n][0]
    )

    for name, have, want in wrong:
        conv, cite = found[name][0], found[name][1]
        print(f"WRONG   {name}: table says {have!r}, headers say {want!r}  ({cite})")
    for name in missing:
        conv, cite = found[name]
        print(f"MISSING {name} -> {conv}  ({cite})")

    if missing or wrong:
        print(
            f"\nSUMMARY: {len(missing)} missing, {len(wrong)} wrong "
            f"(of {len(found)} convention aliases in the headers)\n"
            "An UNKNOWN alias is not a no-op: _derive_convention falls back to cdecl,\n"
            "silently. Add these to rustconv/abi_enforce.py::_KW_TO_CONV with citations."
        )
        return 1

    print(f"SUMMARY: 0 missing, 0 wrong ({len(found)} convention aliases, all known)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
