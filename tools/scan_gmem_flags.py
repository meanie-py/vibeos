#!/usr/bin/env python3
"""
scan_gmem_flags.py — a HEAP_* flag (or any non-GMEM/LMEM bit) handed to Global/LocalAlloc.

THE CLASS. `GlobalAlloc`/`LocalAlloc` take GMEM_/LMEM_ flags; `HeapAlloc` takes HEAP_
flags. The translator sometimes swaps the families: oleaut32's create_stream asked for
`GlobalAlloc(HEAP_ZERO_MEMORY, n)` where the C says `GMEM_FIXED` (0). 0x8 is outside
GMEM_VALID_FLAGS (0x7F72), so GlobalAlloc refused it with ERROR_INVALID_PARAMETER and
every file-backed OleLoadPicturePath / OleLoadPictureFile failed. (HEAP_ZERO_MEMORY
does not even mean zero-init there; GMEM_ZEROINIT is 0x40.)

WHAT IS FLAGGED: the flags argument of Global/LocalAlloc (1st) or Global/LocalReAlloc
(3rd) when it names a HEAP_ constant, or is a bare literal with a bit outside 0x7F72.
Mark a genuine exception `// nolint: gmem-flags (why)`.

Usage:
    python3 tools/scan_gmem_flags.py [--lint] [paths...]
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
GMEM_VALID_FLAGS = 0x7F72
CALL_RE = re.compile(r"\b(?:Global|Local)(Alloc|ReAlloc)\s*\(")
LIT_RE = re.compile(r"(0x[0-9a-fA-F_]+|\d[\d_]*)(?:u16|u32|i32|usize)?(?:\s+as\s+\w+)?")
NOLINT = "nolint: gmem-flags"


def _args(text: str, open_idx: int) -> list[str] | None:
    """Split the call's top-level arguments, starting just after its `(`."""
    depth, cur, out = 0, [], []
    for i in range(open_idx, len(text)):
        c = text[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                out.append("".join(cur).strip())
                return out
            depth -= 1
        elif c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(c)
    return None


def _strip_comments(arg: str) -> str:
    return re.sub(r"/\*.*?\*/|//[^\n]*", "", arg, flags=re.S).strip()


def scan_text(text: str) -> list[tuple[int, str]]:
    lines = text.splitlines()
    out = []
    for m in CALL_RE.finditer(text):
        # `fn GlobalAlloc(` declarations and `extern` prototypes are not calls.
        if re.search(r"\bfn\s+$", text[max(0, m.start() - 8) : m.start()]):
            continue
        args = _args(text, m.end())
        idx = 0 if m.group(1) == "Alloc" else 2
        if not args or len(args) <= idx:
            continue
        flag = _strip_comments(args[idx])
        line = text.count("\n", 0, m.start()) + 1
        if NOLINT in lines[line - 1]:
            continue
        if "HEAP_" in flag:
            out.append((line, flag))
            continue
        lit = LIT_RE.fullmatch(flag)
        if lit and int(lit.group(1).replace("_", ""), 0) & ~GMEM_VALID_FLAGS & 0xFFFF:
            out.append((line, flag))
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
        for line, flag in scan_text(f.read_text(errors="replace")):
            rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
            print(f"{rel}:{line}: Global/LocalAlloc flags `{flag}` are not GMEM_/LMEM_ flags")
            n += 1
    if not args.lint:
        print(f"\n{n} finding(s)")
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main())
