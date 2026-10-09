#!/usr/bin/env python3
"""
scan_ptr_typedef_cast_read.py — `(*(x as *mut PFOO).read()).Field` for the C's `((PFOO)x)->Field`.

`PFOO` is already a pointer. Casting to `*mut PFOO` and `.read()`ing loads a POINTER out
of whatever `x` points at, and the outer `*` then dereferences that -- one level deeper
than the C. The struct's first field becomes the address:

    C:    LocaleId = ((PGENENTRY)GetListEntryData(ListEntry))->Id;
    Rust: LocaleId = (*(GetListEntryData(ListEntry) as *mut PGENENTRY).read()).Id;

usetup's unattended language lookup then handed `_wcsicmp` the Id string's first eight
bytes, L"0000", as a pointer (rcx=0x0030003000300030) and the bootcd's smss.exe died:
BUGCHECK 0x71 at the start of every unattended install. `scan_ptr_typedef_indirection`
catches the same mistake in a DECLARATION (`x: *mut PFOO`); this is its cast form.

A slot that really holds a pointer reads it the same way, so a reviewed site takes
`nolint: ptr-typedef-cast-read` with its reason. Gated at zero.

Usage:
    python3 tools/scan_ptr_typedef_cast_read.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_ptr_typedef_cast_read.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
NOLINT_RE = re.compile(r"nolint:\s*ptr-typedef-cast-read\b")
# `(*(<expr> as *mut PFOO).read())`: the read pointer is dereferenced on the spot.
CAST_READ_RE = re.compile(
    r"\(\*\([^;\n]{1,200}?\s+as\s+\*(?:mut|const)\s+(L?P[A-Z][A-Za-z0-9_]*)\s*\)\s*\.read\(\)\s*\)"
)


def _code(text: str) -> str:
    return COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            lines = text.splitlines()
            for m in CAST_READ_RE.finditer(_code(text)):
                lineno = text.count("\n", 0, m.start()) + 1
                if NOLINT_RE.search(lines[lineno - 1]):
                    continue
                findings.append((f, lineno, m.group(1)))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, typedef in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: ptr-typedef-cast-read {typedef}")
        else:
            print(f"{rel}:{lineno}: `as *mut {typedef}).read()` then deref: one level past the C")
    if findings and not lint:
        print(
            f"\n{len(findings)} finding(s): cast to `{findings[0][2]}` itself, not a pointer to it."
        )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
