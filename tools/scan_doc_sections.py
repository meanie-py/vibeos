#!/usr/bin/env python3
"""Duplicate-section detector for the field manual (and any doc with `### N.M` heads).

WHY THIS EXISTS. Every lane appends its `### 9.NNN` section at the tail of
`translation-debugging-field-manual.md`, so two lanes working the same day pick
the same number independently. That happened three times in three days
(§9.113, §9.118, then §9.119+§9.120 together). A section number is only safe
once merged.

Tail-vs-tail collisions announce themselves as a merge conflict, which is
survivable — you notice, you renumber. The dangerous case is the one that does
NOT conflict: if two lanes insert at different points, or one renumbers into a
number a third lane already used, git merges both cleanly and the file quietly
ends up with the same number twice. Every cross-reference to it is then
ambiguous, and nothing tells you.

So: fail loudly at the gate. Empty baseline = strict.

Usage:
    scan_doc_sections.py [--lint] [paths...]      (default: the field manual)
Exit 1 on any duplicate when --lint is passed.
"""
from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DEFAULT_DOCS = [REPO / "docs" / "engineering" / "translation-debugging-field-manual.md"]

# `### 9.114`, `### 9.115a`, `### 12.3` — number + optional letter suffix, plus any
# sub-levels. The sub-levels are load-bearing: `9.6c`, `9.6c.1` and `9.6c.2` are three
# DISTINCT sections, and a pattern that stops at the letter reports them as one
# triplicated number (the first version of this lint did exactly that).
HEAD_RE = re.compile(r"^#{2,4}\s+(\d+\.\d+[a-z]?(?:\.\d+)*)(?![\w.])")


def scan(path: Path) -> dict[str, list[int]]:
    seen: dict[str, list[int]] = defaultdict(list)
    for lineno, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
        m = HEAD_RE.match(line)
        if m:
            seen[m.group(1)].append(lineno)
    return seen


def main(argv: list[str]) -> int:
    lint = "--lint" in argv
    args = [a for a in argv if not a.startswith("--")]
    docs = [Path(a) for a in args] if args else DEFAULT_DOCS

    dupes = 0
    for doc in docs:
        if not doc.is_file():
            print(f"{doc}: MISSING", file=sys.stderr)
            return 1
        for num, lines in sorted(scan(doc).items()):
            if len(lines) > 1:
                dupes += 1
                rel = doc.relative_to(REPO) if doc.is_relative_to(REPO) else doc
                print(f"{rel}: DUPLICATE section {num} at lines {', '.join(map(str, lines))}")
                print(f"    Two lanes picked {num} independently. Renumber the LATER one to the "
                      f"next free number and fix its cross-references — surgically, not with a "
                      f"blanket sed (the other lane's refs point at ITS section).")

    if dupes:
        print(f"\nTotal: {dupes} duplicate section number(s).")
        return 1 if lint else 0
    print(f"scan_doc_sections: OK — no duplicate section numbers in {len(docs)} doc(s).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
