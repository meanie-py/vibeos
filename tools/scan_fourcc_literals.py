#!/usr/bin/env python3
"""scan_fourcc_literals.py — a FOURCC folded to the wrong integer.

THE INCIDENT
    evr's `MFIsFormatYUV` is a C `switch` over D3DFORMAT FOURCCs written as
    `MAKEFOURCC('I','M','C','1')`. The macro is not sealed (it is an expression, not a
    constant), so the model folded each one to an integer itself and kept the macro as a
    comment:

        | 808530521 // MAKEFOURCC('I','M','C','1')
        | 842094158 // MAKEFOURCC('N','V','1','1')
        | 842094158 // MAKEFOURCC('N','V','1','2')

    Three of the nine values were wrong (IMC1 got Y210's number, NV11 got NV12's). rustc
    noticed only because two wrong values happened to COLLIDE with right ones and made
    arms unreachable; a wrong value that collides with nothing compiles silently and
    answers FALSE for a real YUV format.

WHAT COUNTS AS A FINDING
    An integer literal immediately followed by a `// MAKEFOURCC('a','b','c','d')` (or
    `mmioFOURCC`) comment whose value is not `a | b<<8 | c<<16 | d<<24`. The comment is the
    model's own statement of intent, so the check needs no C: it holds the number to the
    characters written beside it. Baseline is EMPTY.

    Findings go to stdout for the baseline comparison; the tally goes to stderr.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

_CH = r"'(\\?.)'"
FOURCC = re.compile(
    r"\b(0[xX][0-9a-fA-F_]+|\d[\d_]*)(?:[iu](?:8|16|32|64|size))?"
    r"[^\n/]*?//\s*(?:MAKEFOURCC|mmioFOURCC)\(\s*"
    + r"\s*,\s*".join([_CH] * 4)
    + r"\s*\)"
)
_ESC = {"\\'": "'", "\\\\": "\\", "\\0": "\0"}


def _ord(ch: str) -> int:
    return ord(_ESC.get(ch, ch))


def scan_text(text: str) -> list[tuple[int, str, int, int]]:
    """(line, fourcc, written, correct) for each mis-folded literal."""
    out = []
    for lineno, line in enumerate(text.splitlines(), 1):
        for m in FOURCC.finditer(line):
            written = int(m.group(1).replace("_", ""), 0)
            chars = [m.group(i) for i in range(2, 6)]
            correct = sum(_ord(c) << (8 * i) for i, c in enumerate(chars))
            if written != correct:
                out.append((lineno, "".join(chars), written, correct))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", action="append", default=None,
                    help="directory to scan (repeatable); defaults to sdk/")
    args = ap.parse_args()
    repo = Path(__file__).resolve().parent.parent
    findings = files = 0
    for root in [repo / r for r in (args.root or ["sdk"])]:
        for path in sorted(source_files(root)):
            files += 1
            for lineno, cc, written, correct in scan_text(
                path.read_text(encoding="utf-8", errors="replace")
            ):
                print(
                    f"{path.relative_to(repo)}:{lineno}: FOURCC_MISFOLDED '{cc}' written as "
                    f"{written}, is {correct} (0x{correct:08x})"
                )
                findings += 1
    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
