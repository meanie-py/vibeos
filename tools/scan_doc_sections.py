#!/usr/bin/env python3
"""Section-numbering guard for the field manual (and any doc with `### N.M` heads).

WHY THIS EXISTS. The field manual used to be append-at-EOF with a global `### 9.NNN`
counter, and every lane picked its number independently. Two things followed, both
expensive:

  1. **Merge conflicts on every single merge.** Two lanes appending different text at
     the same end-of-file position always conflict. It is a positional collision, not a
     semantic one, so it carried no information — pure tax.
  2. **Number collisions.** A section number is only safe once merged, and lanes don't
     merge before writing. That happened five times in one week (9.113, 9.118,
     9.119+9.120, 9.122 twice), plus five historical pairs that merged CLEANLY and sat
     in the tree with the same number twice (9.27/9.36/9.41/9.43/9.44), making every
     cross-reference to them ambiguous with nothing to announce it.

**The counter is retired** (2026-07-25). Entries are now placed in the topical section
their symptom belongs to and identified by their heading; the surviving `§9.N` tokens are
frozen historical IDs kept inline so pre-existing cross-references still resolve by grep.

So this script enforces two things:

  * **RETIRED COUNTER** (field manual only): a `### N.M` *heading* means somebody minted
    a new number, which re-imports both problems above. Report it and fail.
  * **DUPLICATES** (any doc): the same number used twice. Still checked, because other
    docs legitimately use numbered sections, and because a duplicate is the failure mode
    that does NOT announce itself.

Usage:
    scan_doc_sections.py [--lint] [paths...]      (default: the field manual)
Exit 1 under --lint on any finding.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
FIELD_MANUAL = REPO / "docs" / "engineering" / "translation-debugging-field-manual.md"
DEFAULT_DOCS = [FIELD_MANUAL]

# `### 9.114`, `### 9.115a`, `### 12.3` — number + optional letter suffix, plus any
# sub-levels. The sub-levels are load-bearing: `9.6c`, `9.6c.1` and `9.6c.2` are three
# DISTINCT sections, and a pattern that stops at the letter reports them as one
# triplicated number (the first version of this lint did exactly that).
HEAD_RE = re.compile(r"^#{2,4}\s+(\d+\.\d+[a-z]?(?:\.\d+)*)(?![\w.])")


def scan(path: Path) -> dict[str, list[int]]:
    seen: dict[str, list[int]] = defaultdict(list)
    fenced = False
    for lineno, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
        # A fenced block can contain a line that looks like a heading (the manual quotes
        # `git status` output containing `## master...origin/master`). Don't count those.
        if line.lstrip().startswith("```"):
            fenced = not fenced
            continue
        if fenced:
            continue
        m = HEAD_RE.match(line)
        if m:
            seen[m.group(1)].append(lineno)
    return seen


def main(argv: list[str]) -> int:
    lint = "--lint" in argv
    args = [a for a in argv if not a.startswith("--")]
    docs = [Path(a) for a in args] if args else DEFAULT_DOCS

    findings = 0
    for doc in docs:
        if not doc.is_file():
            print(f"{doc}: MISSING", file=sys.stderr)
            return 1
        rel = doc.relative_to(REPO) if doc.is_relative_to(REPO) else doc
        numbered = scan(doc)

        # (1) Retired counter — field manual only. Other docs may number their sections.
        if doc.resolve() == FIELD_MANUAL.resolve():
            for num, lines in sorted(numbered.items()):
                for line in lines:
                    findings += 1
                    print(f"{rel}:{line}: NEW NUMBERED SECTION `{num}` — the counter is retired")
            if numbered:
                print(
                    "    Don't mint a section number. Put the entry in the topical section its\n"
                    "    symptom belongs to and title it by the symptom; a heading is its own id.\n"
                    "    Sequential numbering caused five same-number collisions in one week and\n"
                    "    made every lane's merge conflict at end-of-file. Existing `§9.N` tokens\n"
                    "    are frozen inline tags for old cross-references, not a series to extend."
                )

        # (2) Duplicates — any doc.
        for num, lines in sorted(numbered.items()):
            if len(lines) > 1:
                findings += 1
                print(f"{rel}: DUPLICATE section {num} at lines {', '.join(map(str, lines))}")
                print(
                    f"    Two lanes picked {num} independently. A duplicate merges CLEANLY, so "
                    f"nothing else will tell you: every cross-reference to {num} is now "
                    f"ambiguous. Merge the two entries, or retitle the later one by its symptom."
                )

    if findings:
        print(f"\nTotal: {findings} section-numbering finding(s).")
        return 1 if lint else 0
    # Baseline is EMPTY, and `vibeos-cli check` diffs STDOUT against it — so a clean run must
    # print nothing there, or the success line itself reads as a new finding. Confirm on stderr
    # so an interactive run still gets an acknowledgement.
    print(
        f"scan_doc_sections: OK — no minted or duplicate section numbers in {len(docs)} doc(s).",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
