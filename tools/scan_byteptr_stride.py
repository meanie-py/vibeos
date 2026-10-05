#!/usr/bin/env python3
"""
scan_byteptr_stride.py — byte arithmetic done through a pointer TO a byte pointer.

THE CLASS. C writes a byte offset as `(PBYTE)base + off`. The translator sometimes
casts to `*mut PBYTE` instead -- a pointer to a PBYTE -- and then `.offset(off)`,
which steps in units of a POINTER (8 bytes on 64-bit). It compiles, the result is
cast straight back to whatever the code wanted, and nothing is wrong until an
offset is non-zero:

    src = (src_rec as *mut PBYTE).offset(field_offset)    // oleaut32 RecordCopy:
                                                          // every field past 0 read
                                                          // from 8x its address
    DestBits = ((*DestSurf).pvScan0 as *mut PBYTE)        // win32k 16bpp transparent
        .offset(left << 1).offset(top * lDelta)           // blit: wrote 8x past target

A `*mut PBYTE` that really is an array of byte pointers is rare enough that the
gate holds none; mark a genuine one `// nolint: byteptr-stride (why)`.

Usage:
    python3 tools/scan_byteptr_stride.py [--lint] [paths...]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
BYTE_PTR_TYPEDEFS = r"(?:P|LP)(?:BYTE|UCHAR)"
SHAPE_RE = re.compile(
    r"as\s+\*(?:mut|const)\s+(" + BYTE_PTR_TYPEDEFS + r")\s*\)\s*\.\s*"
    r"(offset|add|sub|wrapping_add|wrapping_sub|wrapping_offset)\s*\(",
    re.S,
)
NOLINT = "nolint: byteptr-stride"


def scan_text(text: str) -> list[tuple[int, str, str]]:
    lines = text.splitlines()
    out = []
    for m in SHAPE_RE.finditer(text):
        line = text.count("\n", 0, m.start()) + 1
        if NOLINT in lines[line - 1]:
            continue
        out.append((line, m.group(1), m.group(2)))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument("--lint", action="store_true", help="terse output; exit 1 on findings")
    args = ap.parse_args()

    files = []
    for p in args.paths or [RUST_ROOT]:
        files += [p] if p.is_file() else sorted(source_files(p))
    n = 0
    for f in files:
        for line, typedef, method in scan_text(f.read_text(errors="replace")):
            rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
            what = f"`as *mut {typedef}` then `.{method}(`"
            print(f"{rel}:{line}: {what} steps by pointer size, not bytes")
            n += 1
    if not args.lint:
        print(f"\n{n} finding(s)")
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main())
