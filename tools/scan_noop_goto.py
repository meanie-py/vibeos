#!/usr/bin/env python3
"""
scan_noop_goto.py — a C `goto` translated as a labeled block that only breaks out of itself.

    // C:  if (AdapterType == RamdiskAdapter) goto Quit;   ...controller parse...  Quit: ...
    'quit: {
        break 'quit;     // jumps to the end of THIS block -- i.e. nowhere
    }

`break 'x` leaves the block labeled 'x, so a block holding nothing but its own break is a
no-op and control falls through into exactly the code the C jumped over. setuplib's
ParseArcName did that for `ramdisk(0)`: the parse carried on looking for a controller and
every ramdisk ARC path failed to resolve (found by tools/payloads/modrun, 2026-10-07).

Not every hit is live: a translator that already restructured the flow into if/else
leaves the block behind as a vestige (diskpart's mountmgr.c had three). Either way the block
must go: delete a vestige, or move the label out so the block spans the code the C skips:

    'quit: {
        if ramdisk { break 'quit; }
        ...controller parse...
    }
    // Quit:

Usage:
    python3 tools/scan_noop_goto.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_noop_goto.py --lint
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
NOOP_RE = re.compile(r"'(\w+):\s*\{\s*break\s+'\1\s*;\s*\}")
# A fix's own comment often quotes the shape it removed; matching comment text flagged three
# already-fixed sites. Comments are blanked (newlines kept) before matching.
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)


def _code(text: str) -> str:
    return COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            for m in NOOP_RE.finditer(_code(text)):
                findings.append((f, text.count("\n", 0, m.start()) + 1, m.group(1)))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, label in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: noop-goto '{label}")
        else:
            print(f"{rel}:{lineno}: `'{label}: {{ break '{label}; }}` jumps nowhere (a C goto?)")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): delete the vestige, or span the block over the skipped code.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
