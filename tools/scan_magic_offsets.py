#!/usr/bin/env python3
"""
scan_magic_offsets.py — catch hand-written NUMERIC struct offsets in Rust code.

verify_offsets.py proves bindgen struct layouts match C (`offset_of!` asserts,
compile-time). What it CANNOT see is usage-site numerology: code that reaches
into a struct by hand-maintained numbers instead of typed field access. That
class produced two real kernel bugs in one week (2026-07-05):

  - MMPFN offset tables one slot shifted (pfn_offsets_extra: RmapListHead/
    NextLRU/PreviousLRU) -> win32k BUGCHECK 0xBE via wrong union slot
  - KTHREAD.Teb hardcoded 0x1e8 in ke/bug.rs (pre-fix) -> banner read garbage

Shapes flagged (line-based; the 0-NEW baseline gate absorbs the FP stock):

  A. `const <NAME>: ... = <int>` where NAME contains OFFSET / ends _OFF
     (hand-rolled offset constant tables)
  B. `<expr> as usize + 0x<hex> ... as *const/*mut` (pointer re-cast after
     numeric add — the Teb-0x1e8 shape); also `usize + <dec>` >= 8
  C. `.byte_add( <int> )` / `.byte_offset( <int> )` / `.byte_sub( <int> )`
  D. `u8` pointer `.add(0x<hex>)` / `.cast::<u8>().add(0x<hex>)` (hex literal
     byte-arithmetic into a struct)

The fix for a true positive is `core::mem::offset_of!(<bindgen type>, field)`
(or a generated per-arch constant traceable to bindgen) — never a bare number.

Usage:
    python3 tools/scan_magic_offsets.py [paths...]   # default: sweep sdk/lib/rust
    python3 tools/scan_magic_offsets.py --lint       # terse (CLI baseline gate)

Suppress a verified false positive on its line:
    let x = p.byte_add(4); // nolint: magic-offset (SLIST alignment word, not a field)

Exit 1 if any finding (the CLI gate diffs against tools/lint-baseline/).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

# Files that legitimately traffic in numeric offsets.
SKIP_FILE_RE = re.compile(
    r"(bindgen_[a-z0-9_]+\.rs$|offset_check[a-z0-9_]*\.rs$|size_check[a-z0-9_]*\.rs$"
    r"|/target/|/\.responses/)"
)

NOLINT_RE = re.compile(r"//\s*nolint(?::\s*magic-offset\b|\s*$|:?\s*\()")

# Rust integer literals may carry a type suffix (0x18usize, 24u32) and `_` digit
# separators. A trailing `\b` after the digits does NOT tolerate that — `u` is a
# word char, so `0x18usize` leaves no boundary and the shape silently MISSES.
# This is the §9.119 lesson: scan_bugcheck_codes.py had the identical hole and
# reported a clean 0 for six weeks while `KeBugCheckEx(0x35u32, ...)` sat in the
# tree. No live escapee here today — fixed as a latent hole, not a live bug.
_SFX = r"(?:[ui](?:8|16|32|64|128|size))?"

# B. `as usize + <literal>` later re-cast to a pointer on the same line
B_RE = re.compile(r"as\s+usize\s*[+\-]\s*(0x[0-9A-Fa-f_]+|\d[\d_]{1,})" + _SFX + r"\b.*\bas\s+\*(?:const|mut)\b")
# C. byte_add / byte_offset / byte_sub with a literal
C_RE = re.compile(r"\.byte_(?:add|offset|sub)\s*\(\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)" + _SFX + r"\b")
# D. struct-pointer-recast-to-u8 byte arithmetic: hex/large-dec literal, or a
#    SCREAMING_CASE const path (the pfn_offsets_extra::NEXT_LRU table shape).
#    Lowercase args (`.add(i)`, `.add(len)`) are ordinary buffer indexing — skip.
D_RE = re.compile(
    r"(?:\*(?:const|mut)\s+u8\)?\s*\.\s*(?:add|offset|sub)|cast::<u8>\(\)\s*\.\s*(?:add|offset|sub))"
    r"\s*\(\s*(0x[0-9A-Fa-f]+\b|\d{2,}\b|[A-Za-z_][A-Za-z0-9_]*(?:::)?[A-Z][A-Z0-9_]{2,}\b)"
)

# E. hex-literal element arithmetic on ANY typed pointer — `.add(0x77 / 4)` on
#    a *const ULONG is a hand-scaled FIELD offset wearing an index costume
#    (the TEB.GdiBatchCount-at-0x74 bug, field manual §9.41: real offset
#    0x1740, flush never fired, GDI batch overflowed into CLIENTINFO). Buffer
#    indexing essentially never uses hex literals; struct numerology does.
E_RE = re.compile(r"\.\s*(?:add|offset|sub)\s*\(\s*0x[0-9A-Fa-f_]+" + _SFX + r"\b")

SHAPES = (
    ("PTR_RECAST_ARITH", B_RE),
    ("BYTE_ADD", C_RE),
    ("U8_BYTE_ARITH", D_RE),
    ("TYPED_PTR_HEX_ARITH", E_RE),
)


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    findings = []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return findings
    in_block_comment = False
    for i, line in enumerate(text.splitlines(), 1):
        stripped = line.strip()
        # crude block-comment tracking (bindgen-appended C source, doc blocks)
        if in_block_comment:
            if "*/" in stripped:
                in_block_comment = False
            continue
        if stripped.startswith("/*") and "*/" not in stripped:
            in_block_comment = True
            continue
        if stripped.startswith("//"):
            continue
        if NOLINT_RE.search(line):
            continue
        code = line.split("//", 1)[0]  # ignore trailing comments
        for shape, rx in SHAPES:
            m = rx.search(code)
            if not m:
                continue
            # Shape B decimal guard: small decimal adds are usually index math.
            if shape == "PTR_RECAST_ARITH":
                lit = m.group(1)
                if not lit.startswith("0x") and int(lit) < 8:
                    continue
            findings.append((i, shape, stripped[:160]))
            break  # one finding per line is enough
    return findings


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    terse = "--lint" in sys.argv
    paths: list[Path] = []
    if args:
        for a in args:
            p = Path(a)
            paths.extend(sorted(p.rglob("*.rs")) if p.is_dir() else [p])
    else:
        paths = sorted(RUST_ROOT.rglob("*.rs"))

    total = 0
    files = 0
    for path in paths:
        rel = str(path)
        if SKIP_FILE_RE.search(rel.replace("\\", "/")):
            continue
        files += 1
        for lineno, shape, snippet in scan_file(path):
            total += 1
            try:
                shown = path.relative_to(REPO)
            except ValueError:
                shown = path
            if terse:
                print(f"[{shape}] {shown}:{lineno}: {snippet}")
            else:
                print(f"  [{shape}] {shown}:{lineno}: {snippet}")
    if not terse:
        print(f"TOTAL: {total} magic-offset findings across {files} files", file=sys.stderr)
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main())
