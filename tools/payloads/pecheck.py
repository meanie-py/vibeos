#!/usr/bin/env python3
"""Check a payload .exe's PE headers and import table -- in place of `objdump -p | grep`.

Why not the pipe: every payload build.sh runs `set -euo pipefail`, and `grep -q` exits on
its FIRST match. objdump, still writing, then dies of SIGPIPE, so the pipeline's status is
141 -- a coin flip on how far objdump got. Both uses were broken by it:

  * `objdump -p x | grep -qi ucrtbase.dll || fail`  failed a GOOD exe at random (it
    killed a ucrt Tier 2 sweep at shard 1 of 8 on 2026-09-26, shard 0 having passed);
  * `objdump -p x | grep -qi api-ms-win-crt && fail || true`  PASSED a bad exe at random:
    the match is exactly what makes grep exit early, so the failure branch never ran.

Here the import table is read directly. Every failure is a nonzero exit with a message.

  pecheck.py EXE --tag NAME [--gui] [--require-dll DLL]... [--forbid-dll SUBSTR]...
                 [--graded-from DLL --graded-decls SRC.c [--refs-header generated-refs.h]]

--graded-from is ucrtdiff's check: every `__cdecl ref_<name>(` declared in SRC.c must be
imported from DLL by <name> -- a row whose "ours" side got inlined or constant-folded would
otherwise grade the compiler instead of our DLL. A name the refs header marks
`#define UCRTDIFF_REF_HAS_<name> 0` is compiled out, and so exempt -- and so is every
declaration inside an `#if UCRTDIFF_REF_HAS_<name>` block for such a name, since that
block is compiled out too (a family's helpers, e.g. the ref_fopen its stream runners use).
"""

from __future__ import annotations

import argparse
import re
import struct
import sys
from pathlib import Path


class PEError(Exception):
    pass


def _guard_value(expr: str, off_refs: set[str]) -> bool | None:
    """An `#if`/`#elif` expression over REF guards alone, evaluated; None for anything else.

    `UCRTDIFF_REF_HAS_<off name>` is 0 and every other REF_HAS / REF_CALLABLE is taken
    as 1 -- the live direction, so a guard can only ever EXEMPT a declaration when the
    names that switch it off are known to be off. A compound guard (`HAS_a && HAS_b`)
    used to read as "some other #if", so a block switched off by one takeover was
    still counted as compiled in and its declarations demanded imports.
    """
    py = re.sub(r"UCRTDIFF_REF_HAS_(\w+)", lambda m: "0" if m.group(1) in off_refs else "1", expr)
    py = re.sub(r"UCRTDIFF_REF_CALLABLE_\w+", "1", py)
    py = re.sub(r"/\*.*?\*/", " ", py)
    py = py.replace("&&", " and ").replace("||", " or ")
    py = re.sub(r"!(?!=)", " not ", py)
    if not re.fullmatch(r"[\s01()]*(?:(?:and|or|not)[\s01()]*)*", py):
        return None
    try:
        # The fullmatch above admits only 0, 1, and/or/not and parentheses.
        return bool(eval(py, {"__builtins__": {}}, {}))
    except SyntaxError:
        return None


def compiled_in(src: str, off_refs: set[str]) -> str:
    """`src` without the lines an `#if` over `UCRTDIFF_REF_HAS_<name>` guards removes.

    Only guards made entirely of REF_HAS/REF_CALLABLE terms are evaluated; every other
    `#if` is taken as live, which is the conservative direction -- a declaration is
    only ever exempted by a guard we can read.
    """
    kept = []
    # (this branch live, is a REF guard, some earlier branch of this chain was live)
    stack: list[tuple[bool, bool, bool]] = []
    # A guard continued with `\` is one directive (the spawn block's four-way `||`).
    joined = re.sub(r"\\\n", " ", src)
    # Object-like macros whose bodies are themselves REF guard expressions
    # (`#define LW_ANY (UCRTDIFF_REF_HAS__write || ...)`), expanded into the `#if`s
    # that use them. Unexpanded, `#if LW_ANY` read as "some other #if" and so as
    # live, and a whole compiled-out block demanded imports for what it declares.
    guard_macros: dict[str, str] = {}
    for m in re.finditer(r"^\s*#\s*define\s+(\w+)\s+(.+)$", joined, re.M):
        if "UCRTDIFF_REF_" in m.group(2) and _guard_value(m.group(2), off_refs) is not None:
            guard_macros[m.group(1)] = f"({m.group(2)})"

    def expand(expr: str) -> str:
        return re.sub(r"\b(\w+)\b", lambda t: guard_macros.get(t.group(1), t.group(1)), expr)

    for line in joined.splitlines():
        d = line.strip()
        m = re.match(r"#\s*if\s+(.*)$", d)
        if m and not re.match(r"#\s*if(n?def)\b", d):
            v = _guard_value(expand(m.group(1)), off_refs)
            stack.append((True, False, True) if v is None else (v, True, v))
            continue
        if re.match(r"#\s*if(n?def)?\b", d):
            stack.append((True, False, True))
            continue
        m = re.match(r"#\s*elif\s+(.*)$", d)
        if m and stack:
            _, guard, taken = stack[-1]
            v = _guard_value(expand(m.group(1)), off_refs) if guard else None
            if v is None:
                stack[-1] = (True, False, True)
            else:
                stack[-1] = (v and not taken, True, taken or v)
            continue
        if re.match(r"#\s*else\b", d) and stack:
            live, guard, taken = stack[-1]
            stack[-1] = (not taken if guard else live, guard, True)
            continue
        if re.match(r"#\s*endif\b", d) and stack:
            stack.pop()
            continue
        if all(live for live, _, _ in stack):
            kept.append(line)
            # `#if X` / `#define LW_ANY 1` / `#else` / `#define LW_ANY 0`: the value
            # the LIVE branch gives a flag macro, for the `#if LW_ANY` that follows.
            dm = re.match(r"#\s*define\s+(\w+)\s+([01])\s*$", d)
            # Never the REF guards themselves: the harness's `#ifndef X / #define X 1`
            # fallbacks sit in `#ifndef` branches this pass keeps, and recording them
            # overrode the generated header's 0s.
            if dm and not dm.group(1).startswith("UCRTDIFF_REF_"):
                guard_macros[dm.group(1)] = dm.group(2)
    return "\n".join(kept)


def parse(path: Path) -> tuple[int, dict[str, list[str]]]:
    """(subsystem, {dll name (as written): [imported function names]})."""
    data = path.read_bytes()
    if data[:2] != b"MZ":
        raise PEError("not an MZ image")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe : pe + 4] != b"PE\0\0":
        raise PEError("no PE signature")
    nsect, optsize = (
        struct.unpack_from("<H", data, pe + 6)[0],
        struct.unpack_from("<H", data, pe + 20)[0],
    )
    opt = pe + 24
    magic = struct.unpack_from("<H", data, opt)[0]
    if magic not in (0x10B, 0x20B):
        raise PEError(f"unknown optional-header magic {magic:#x}")
    wide = magic == 0x20B
    subsystem = struct.unpack_from("<H", data, opt + 68)[0]
    ddir = opt + (112 if wide else 96)
    imp_rva = struct.unpack_from("<I", data, ddir + 8)[0]  # data directory 1: imports
    sections = []
    sh = opt + optsize
    for i in range(nsect):
        vsize, va, rawsize, rawptr = struct.unpack_from("<IIII", data, sh + 40 * i + 8)
        sections.append((va, max(vsize, rawsize), rawptr))

    def off(rva: int) -> int:
        for va, size, raw in sections:
            if va <= rva < va + size:
                return raw + (rva - va)
        raise PEError(f"RVA {rva:#x} is in no section")

    def cstr(rva: int) -> str:
        o = off(rva)
        return data[o : data.index(b"\0", o)].decode("latin-1")

    imports: dict[str, list[str]] = {}
    if imp_rva:
        d = off(imp_rva)
        while True:
            oft, _, _, name_rva, ft = struct.unpack_from("<IIIII", data, d)
            if not (oft or name_rva or ft):
                break
            names: list[str] = []
            t = off(oft or ft)
            step, fmt = (8, "<Q") if wide else (4, "<I")
            ordinal_bit = 1 << (63 if wide else 31)
            while True:
                v = struct.unpack_from(fmt, data, t)[0]
                if not v:
                    break
                names.append(f"#{v & 0xFFFF}" if v & ordinal_bit else cstr((v & 0x7FFFFFFF) + 2))
                t += step
            imports[cstr(name_rva)] = names
            d += 20
    return subsystem, imports


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("exe", type=Path)
    ap.add_argument("--tag", required=True, help="payload name, for the failure message")
    ap.add_argument("--gui", action="store_true", help="require IMAGE_SUBSYSTEM_WINDOWS_GUI")
    ap.add_argument("--require-dll", action="append", default=[])
    ap.add_argument("--forbid-dll", action="append", default=[], help="case-insensitive substring")
    ap.add_argument("--graded-from")
    ap.add_argument("--graded-decls", type=Path)
    ap.add_argument("--refs-header", type=Path)
    a = ap.parse_args(argv)

    def fail(msg: str) -> int:
        print(f"[{a.tag}] FAIL: {a.exe.name}: {msg}")
        return 1

    try:
        subsystem, imports = parse(a.exe)
    except (OSError, PEError, struct.error, ValueError) as e:
        return fail(f"cannot read its PE headers: {e}")
    dlls = {d.lower(): d for d in imports}
    if a.gui and subsystem != 2:
        # A console exe launched from explorer's startup walk gets no CSRSS console and
        # dies without a word on serial (kmtest's lesson).
        return fail(f"subsystem={subsystem} (want GUI, 2)")
    for want in a.require_dll:
        if want.lower() not in dlls:
            return fail(f"does not import {want} (imports: {', '.join(sorted(imports))})")
    for bad in a.forbid_dll:
        hit = [d for d in imports if bad.lower() in d.lower()]
        if hit:
            return fail(f"imports {', '.join(hit)} (forbidden: {bad})")
    if a.graded_from:
        if a.graded_decls is None:
            return fail("--graded-from needs --graded-decls")
        have = set(imports.get(dlls.get(a.graded_from.lower(), ""), []))
        off_refs: set[str] = set()
        if a.refs_header is not None and a.refs_header.exists():
            off_refs = set(
                re.findall(r"^#define UCRTDIFF_REF_HAS_(\w+) 0$", a.refs_header.read_text(), re.M)
            )
        src = compiled_in(a.graded_decls.read_text(errors="replace"), off_refs)
        wanted = sorted(set(re.findall(r"__cdecl\s+\*?ref_([A-Za-z_][A-Za-z0-9_]*)\s*\(", src)))
        missing = [n for n in wanted if n not in off_refs and n not in have]
        if missing:
            print(f"[{a.tag}]   those rows would grade the compiler, not our DLL")
            return fail(f"graded but NOT imported from {a.graded_from}: {' '.join(missing)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
