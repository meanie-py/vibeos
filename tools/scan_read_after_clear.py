#!/usr/bin/env python3
"""
scan_read_after_clear.py — reading a location right after atomically exchanging it with zero.

    OldPte.Long = InterlockedExchangePte(PointerPte, 0);
    ...
    *WasDirty = (((*PointerPte).Long >> 6) & 1) as BOOLEAN;   // always 0: it was just cleared

The C reads the RETURNED old value (`OldPte.u.Hard.Dirty`); the translation read the PTE it
had just zeroed. i386/amd64 `MmDeleteVirtualMappingEx` therefore reported every page clean,
so nothing written through a file mapping ever reached the file: setup's cab extraction left
984 zero-filled files on the installed disk. It compiles, and reads plausibly.

Flagged: `InterlockedExchange*(P, 0 | null)` followed within a few lines by a read through
`(*P).`. Read the exchange's return value instead. Gated at zero.

Usage:
    python3 tools/scan_read_after_clear.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_read_after_clear.py --lint
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
MARKER = "ORIGINAL C SOURCE"
CLEAR_RE = re.compile(
    r"(?:\w+::)*InterlockedExchange(?:Pte|Pointer|64|SizeT)?\s*\(\s*(\w+)\s*"
    r"(?:as\s+[\w:*\s]+?)?,\s*(?:0|(?:core::)?(?:ptr::)?null_mut\(\))\s*\)"
)
WINDOW = 900  # chars after the exchange


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
            cut = text.find(MARKER)
            if cut >= 0:
                text = text[:cut]
            code = re.sub(
                r"//[^\n]*|/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S
            )
            for m in CLEAR_RE.finditer(code):
                var = m.group(1)
                tail = code[m.end() : m.end() + WINDOW]
                end = re.search(r"\bfn\s+\w+", tail)  # stay inside this function
                if end:
                    tail = tail[: end.start()]
                r = re.search(rf"\(\*{re.escape(var)}\)\s*\.\s*\w+", tail)
                if r:
                    line = code.count("\n", 0, m.end() + r.start()) + 1
                    findings.append(
                        (f, line, f"{var} cleared, then {' '.join(r.group(0).split())}")
                    )
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
        tag = "read-after-clear" if lint else "read of a location just exchanged with zero:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): read the exchange's returned old value.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
