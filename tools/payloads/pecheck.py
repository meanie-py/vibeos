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
`#define UCRTDIFF_REF_HAS_<name> 0` is compiled out, and so exempt.
"""

from __future__ import annotations

import argparse
import re
import struct
import sys
from pathlib import Path


class PEError(Exception):
    pass


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
        src = a.graded_decls.read_text(errors="replace")
        wanted = sorted(set(re.findall(r"__cdecl\s+\*?ref_([A-Za-z_][A-Za-z0-9_]*)\s*\(", src)))
        off_refs: set[str] = set()
        if a.refs_header is not None and a.refs_header.exists():
            off_refs = set(
                re.findall(r"^#define UCRTDIFF_REF_HAS_(\w+) 0$", a.refs_header.read_text(), re.M)
            )
        missing = [n for n in wanted if n not in off_refs and n not in have]
        if missing:
            print(f"[{a.tag}]   those rows would grade the compiler, not our DLL")
            return fail(f"graded but NOT imported from {a.graded_from}: {' '.join(missing)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
