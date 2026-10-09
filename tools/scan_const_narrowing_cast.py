#!/usr/bin/env python3
"""
scan_const_narrowing_cast.py — a named constant cast to a type too narrow for its value.

    C:    SET_FLAG(srb->SrbFlags, SRB_CLASS_FLAGS_PERSISTANT);           // 0x20000000
    Rust: (*srb).SrbFlags |= (SRB_CLASS_FLAGS_PERSISTANT as USHORT) as u32;   // |= 0

rustc's overflow lint covers literals only; `CONST as u16` truncates silently. disk.sys's
flush SRB lost its persistent flag that way, so classpnp freed an SRB embedded in the
device extension and the first flush bugchecked 0xC2 (BAD_POOL_CALLER) -- setup's registry
page, found by the install smoke.

Values come from the integer `const` items in the same crate (bindgen islands included).
A name defined with two different values in one crate is skipped rather than guessed.
Signed targets accept the two's-complement spelling (0xFFFF as i16 is -1, on purpose in C).

Usage:
    python3 tools/scan_const_narrowing_cast.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_const_narrowing_cast.py --lint
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
NOLINT_RE = re.compile(r"nolint:\s*const-narrowing-cast\b")
CONST_RE = re.compile(
    r"\bconst\s+([A-Za-z_]\w*)\s*:\s*[\w:]+\s*=\s*\(?\s*(-?)\s*(0x[0-9a-fA-F_]+|\d[\d_]*)"
    r"(?:[iu](?:8|16|32|64|128|size))?\s*\)?\s*(?:as\s+[\w:]+\s*)?;"
)
NARROW = {
    "u8": (8, False),
    "UCHAR": (8, False),
    "BYTE": (8, False),
    "BOOLEAN": (8, False),
    "i8": (8, True),
    "CHAR": (8, True),
    "CCHAR": (8, True),
    "u16": (16, False),
    "USHORT": (16, False),
    "WORD": (16, False),
    "WCHAR": (16, False),
    "i16": (16, True),
    "SHORT": (16, True),
    "CSHORT": (16, True),
}
CAST_RE = re.compile(r"(?<![\w.:])([A-Z_][A-Z0-9_]{2,})\s+as\s+(" + "|".join(NARROW) + r")\b")


def _code(text: str) -> str:
    return COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def _crate_of(path: Path) -> Path:
    for parent in path.parents:
        if (parent / "Cargo.toml").is_file():
            return parent
    return path.parent


def _fits(value: int, bits: int, signed: bool) -> bool:
    """True when the cast keeps the value's meaning. All-ones survives any narrowing on
    purpose -- `-1 as u16` is WEOF, `0xFFFFFFFF as u8` is a 0xFF fill byte -- so a value
    whose kept bits are all set is the idiom, not a truncation."""
    mask = (1 << bits) - 1
    if value & mask == mask:
        return True
    if value < 0:
        return signed and value >= -(1 << (bits - 1))
    return value < (1 << bits)


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    files: list[Path] = []
    for root in paths:
        files += [root] if root.is_file() else [f for f in source_files(root) if f.suffix == ".rs"]
    values: dict[Path, dict[str, set[int]]] = defaultdict(lambda: defaultdict(set))
    texts: dict[Path, str] = {}
    for f in files:
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        code = _code(text)
        texts[f] = text
        crate = _crate_of(f)
        for m in CONST_RE.finditer(code):
            v = int(m.group(3).replace("_", ""), 0)
            values[crate][m.group(1)].add(-v if m.group(2) else v)
    findings = []
    for f, text in texts.items():
        known = values[_crate_of(f)]
        lines = text.splitlines()
        for m in CAST_RE.finditer(_code(text)):
            vals = known.get(m.group(1))
            if not vals or len(vals) != 1:
                continue
            value = next(iter(vals))
            bits, signed = NARROW[m.group(2)]
            if _fits(value, bits, signed):
                continue
            lineno = text.count("\n", 0, m.start()) + 1
            if NOLINT_RE.search(lines[lineno - 1]):
                continue
            findings.append((f, lineno, f"{m.group(1)} ({value:#x}) as {m.group(2)}"))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, what in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "const-narrowing-cast" if lint else "truncating cast of a constant:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): the value does not fit; drop the cast or fix the C.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
