#!/usr/bin/env python3
"""scan_calloc_elem_type.py — an array allocated in CHARACTERS and used as wider elements.

THE INCIDENT
    wscript's `Host_Echo` keeps one BSTR per argument:

        C:    strs = calloc(argc, sizeof(*strs));                       // BSTR: a POINTER
        Rust: strs = calloc(argc, size_of::<OLECHAR>()) as *mut BSTR;   // two bytes

    `*strs` is a BSTR, which is an `OLECHAR *`, and the translation took the element's
    element. The array was a quarter of its size on 64-bit, every Echo of two or more
    arguments wrote past it, `WScript.Echo("hello", 42)` printed "hello " (the second
    pointer slot was overwritten), and the corrupted heap crashed the host a few calls
    later. rustc cannot see it: the cast is the C's own `void *` conversion. modrun's
    cscript suite found it, diffing the host against its pre-flip C.

WHAT COUNTS AS A FINDING
    `calloc(n, size_of::<T>()) as *mut U` (or `heap_calloc`) where T is a one- or two-byte
    character/scalar type and U is not: the element size cannot be U's. Sizing a
    flexible-array tail (`offset_of!(..) + n * size_of::<c_char>()`) is malloc, and is not
    read. Baseline is EMPTY.

    Findings go to stdout for the baseline comparison; the tally goes to stderr.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

NARROW = {
    "u8", "i8", "c_char", "c_uchar", "c_schar", "CHAR", "UCHAR", "BYTE", "TCHAR",
    "u16", "i16", "c_ushort", "c_short", "USHORT", "SHORT", "WORD", "WCHAR", "OLECHAR",
    "wchar_t",
}  # fmt: skip
CALLOC = re.compile(
    r"\b(?:heap_)?calloc\s*\(\s*(?:[^;]{1,160}?),\s*\(?\s*(?:(?:::)?core::mem::|mem::)?"
    r"size_of::<\s*([\w:]+)\s*>\s*\(\)\s*\)?\s*(?:as\s+[\w:]+\s*)?\)\s*as\s*\*mut\s+([\w:]+)"
)
COMMENT = re.compile(r"//[^\n]*")


def scan_text(text: str) -> list[tuple[int, str, str]]:
    """(line, sized as, used as) for each character-sized array used as wider elements."""
    code = COMMENT.sub("", text)
    out = []
    for m in CALLOC.finditer(code):
        sized, used = m.group(1).split("::")[-1], m.group(2).split("::")[-1]
        if sized in NARROW and used not in NARROW and used != "c_void":
            out.append((code.count("\n", 0, m.start()) + 1, sized, used))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        action="append",
        default=None,
        help="directory to scan (repeatable); defaults to sdk/",
    )
    args = ap.parse_args()
    repo = Path(__file__).resolve().parent.parent
    findings = files = 0
    for root in [repo / r for r in (args.root or ["sdk"])]:
        for path in sorted(source_files(root)):
            if path.suffix != ".rs":
                continue
            files += 1
            for lineno, sized, used in scan_text(
                path.read_text(encoding="utf-8", errors="replace")
            ):
                print(
                    f"{path.relative_to(repo)}:{lineno}: CALLOC_ELEM_TYPE sized as {sized}, "
                    f"used as *mut {used} (the C's sizeof(*p)?)"
                )
                findings += 1
    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
