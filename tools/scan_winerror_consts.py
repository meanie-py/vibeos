#!/usr/bin/env python3
"""
scan_winerror_consts.py — a Rust `const` named after a winerror.h code, with another value.

THE CLASS. Hand-written error-code tables drift: the name is right and the number is a
neighbour's, or invented. Nothing notices, because the wrong value is still a valid
HRESULT of the right facility. oleaut32's olesys.rs (found 2026-10-05) had four:
TYPE_E_CANTLOADLIBRARY carried TYPE_E_AMBIGUOUSNAME's value, so LoadTypeLib on a PE
with no typelib reported an ambiguous name, and SetFuncAndParamNames returned an
unknown code where the typelib tests expect TYPE_E_AMBIGUOUSNAME. secur32 had two
SEC_E_ codes off by a few.

Neither sibling sees this. scan_invented_constants judges only constants whose comment
admits a guess; scan_hresult_literals judges only BARE literals. Here the oracle is
exact, so no corpus scoping is needed: winerror.h names are long and unique, and of the
4742 consts that match one, the first run found exactly the six above.

WHAT IS FLAGGED: `const NAME: T = <int literal>` where winerror.h `#define`s NAME and
no definition of it there has that value (compared as 32 bits). A deliberate divergence
needs `// nolint: winerror-const (why)` on the line.

Usage:
    python3 tools/scan_winerror_consts.py [--lint] [paths...]
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
WINERROR = REPO / "sdk" / "include" / "psdk" / "winerror.h"
NOLINT = "nolint: winerror-const"
DEFINE_RE = re.compile(
    r"^\s*#define\s+(\w+)\s+(?:_HRESULT_TYPEDEF_|_NDIS_ERROR_TYPEDEF_)?\(?\s*"
    r"(0x[0-9A-Fa-f]+|\d+)L?\s*\)?\s*(?:$|//|/\*)",
    re.M,
)
CONST_RE = re.compile(
    r"\bconst\s+(\w+)\s*:\s*[\w:]+\s*=\s*\(?\s*(0x[0-9A-Fa-f_]+|\d[\d_]*)"
    r"(?:_?[ui](?:8|16|32|64|size))?\b"
)


def header_values(text: str) -> dict[str, set[int]]:
    values: dict[str, set[int]] = {}
    for m in DEFINE_RE.finditer(text):
        values.setdefault(m.group(1), set()).add(int(m.group(2), 0) & 0xFFFFFFFF)
    return values


def scan_text(text: str, codes: dict[str, set[int]]) -> list[tuple[int, str, int]]:
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        m = CONST_RE.search(line)
        if not m or m.group(1) not in codes or NOLINT in line:
            continue
        if line.lstrip().startswith("//"):
            continue
        value = int(m.group(2).replace("_", ""), 0) & 0xFFFFFFFF
        if value not in codes[m.group(1)]:
            out.append((n, m.group(1), value))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument("--lint", action="store_true", help="terse output; exit 1 on findings")
    args = ap.parse_args()

    codes = header_values(WINERROR.read_text(errors="replace"))
    files = []
    for p in args.paths or [RUST_ROOT]:
        files += [p] if p.is_file() else sorted(source_files(p))
    n = 0
    for f in files:
        if f.suffix != ".rs":
            continue
        for line, name, value in scan_text(f.read_text(errors="replace"), codes):
            rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
            want = ", ".join(f"{v:#010x}" for v in sorted(codes[name]))
            print(f"{rel}:{line}: {name} = {value:#010x}, winerror.h says {want}")
            n += 1
    if not args.lint:
        print(f"\n{n} finding(s)")
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main())
