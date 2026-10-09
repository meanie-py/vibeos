#!/usr/bin/env python3
"""
scan_byte_step_one.py — `(p as *mut u8).add(1)` where the C said `(PUCHAR)(p + 1)`.

C's `(T *)(Header + 1)` steps past one WHOLE `*Header` -- the idiom for "the data that
follows this header". Translated cast-first, it steps one BYTE:

    C:    Buffer = (PUCHAR)(CabinetContext->PCABHeader + 1);      // +36
    Rust: Buffer = ((*CabinetContext).PCABHeader as *mut u8).add(1);   // +1

usetup's cabinet reader then read its folder table from inside the header and faulted
extracting the first file, which is how the install smoke found it. The same shape was in
four more places: usetup walking from one CFFILE to the next, setuplib's NTOS_INSTALLATION
writing its path strings over its own ListEntry, and the recycle bin locating its records.

rustc cannot see it (both are valid pointer arithmetic), and neither can this scanner,
fully: `(p as *const u8).add(1)` is exactly right when `p` already points at bytes or
when the code means "the second byte". So every site is listed, the reviewed-correct ones
are the baseline, and a NEW one is a site to read against its C. Fix: `p.add(1) as *mut u8`
(or the named field, when the struct ends in a flexible array).

Usage:
    python3 tools/scan_byte_step_one.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_byte_step_one.py --lint
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
BYTE_TYPES = r"(?:u8|i8|c_char|core::ffi::c_char|UCHAR|BYTE|CHAR)"
# `(<expr> as *mut u8).add(1)`. <expr> may hold a deref group like `(*Search).File` but
# no call, so a call's argument list (`strstr(buf, b".\0".as_ptr() as *const i8).offset(1)`)
# is not read as the cast, and neither is a call's own paren (`strlen(p as *const i8)
# .wrapping_add(1)` is a length plus one).
STEP_RE = re.compile(
    r"(?<![\w!])\(\s*((?:[^()]|\(\*?[\w.]+\))+?)\s+as\s+\*(?:mut|const)\s+"
    + BYTE_TYPES
    + r"\s*\)\s*"
    r"\.(?:add|offset|wrapping_add|wrapping_offset)\(\s*1(?:\s+as\s+\w+)?\s*\)"
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
            for m in STEP_RE.finditer(_code(text)):
                expr = " ".join(m.group(1).split())
                findings.append((f, text.count("\n", 0, m.start()) + 1, expr))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, expr in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: byte-step-one {expr}")
        else:
            print(f"{rel}:{lineno}: ({expr} as *u8).add(1) steps ONE BYTE -- the C's (p + 1)?")
    if findings and not lint:
        print(f"\n{len(findings)} site(s): read each against its C; a header wants p.add(1).")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
