#!/usr/bin/env python3
"""Census of commented-out source references across every CMake file.

WHY THIS EXISTS
---------------
A cohab flip is held in place by a COMMENT. `ntos.cmake:216` reads
`#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c`, and that single `#` is the only
thing keeping a 31KB C translation unit out of the link while `lpc-rust`'s
`reply.rs` owns its symbols. Delete the `#` and both definitions are live.

That is a partial cohab flip, and it is the failure that froze the guest on
2026-09-19: 58 live C++ symbols silently captured, the taskbar hung inside
`OnCreate`, `ProcessStartupItems` never ran, the conformance sweep launched
0 of 91 tests -- and SMOKE PASSED ON ALL THREE ARCHES, because the freeze is
after the `[WIN32K] shell ready` marker. Smoke cannot see this class of bug.

The upstream sync makes that reachable by accident. Upstream has touched 400
CMake files since our fork; 197 of ours carry a commented-out source. The
intersection is 80 files -- and a merge does not need a CONFLICT to disturb one.
A clean three-way auto-merge can drop or revive a comment line with no marker,
no resolution, and no human in the loop. Review scales to the handful that
conflict; only a census scales to all 80.

WHY THE PATTERN IS DELIBERATELY WIDE
------------------------------------
It matches any comment line mentioning a source-file name, which sweeps in
prose like `# replaced by Rust (sdk/lib/rust/mouhid-rust): mouhid.c`. That is
intentional and is not a bug to fix. This tool is a DIFFER, not a classifier:
stable prose is stable in the baseline, so a false positive costs exactly one
baseline line, while a false negative costs a frozen guest that smoke calls
green. When in doubt, widen it.

Line numbers are deliberately NOT part of the key -- a merge shifts every line
in the file and we would drown in noise. The key is (file, normalized text),
with a count so a duplicated line cannot hide.

USAGE
-----
    python3 tools/cohab_census.py --write     # snapshot -> tools/cohab-census.txt
    python3 tools/cohab_census.py             # findings on stdout; empty == unchanged

Findings baseline is EMPTY = strict. After a DELIBERATE flip (or un-flip),
re-run --write in the same commit that makes the change, so the snapshot always
records a decision somebody made on purpose.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CENSUS = REPO / "tools" / "cohab-census.txt"

# Any comment line naming something that looks like a source file.
SRC_IN_COMMENT = re.compile(
    r"^\s*#.*?[A-Za-z0-9_${}/.+-]*[A-Za-z0-9_+-]\.(?:c|cpp|cxx|cc|S|s|asm|inc)\b"
)

SKIP_DIRS = {".git", "build", "output", "target", "__pycache__"}


def cmake_files() -> list[Path]:
    out = []
    for p in REPO.rglob("*"):
        if p.name != "CMakeLists.txt" and p.suffix != ".cmake":
            continue
        if any(part in SKIP_DIRS or part.startswith("build-") for part in p.parts):
            continue
        if p.is_file():
            out.append(p)
    return sorted(out)


def census() -> Counter:
    c: Counter = Counter()
    for p in cmake_files():
        rel = p.relative_to(REPO).as_posix()
        try:
            text = p.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for line in text.splitlines():
            if SRC_IN_COMMENT.match(line):
                c[(rel, " ".join(line.split()))] += 1
    return c


def render(c: Counter) -> str:
    return "".join(f"{f}\t{n}\t{t}\n" for (f, t), n in sorted(c.items()))


def load(path: Path) -> Counter:
    c: Counter = Counter()
    if not path.is_file():
        return c
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        f, n, t = line.split("\t", 2)
        c[(f, t)] = int(n)
    return c


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="write the snapshot and exit")
    args = ap.parse_args()

    now = census()
    if args.write:
        CENSUS.write_text(render(now), encoding="utf-8")
        print(f"cohab_census: wrote {len(now)} entries to {CENSUS.relative_to(REPO)}")
        return 0

    was = load(CENSUS)
    if not CENSUS.is_file():
        print("[CENSUS-MISSING] tools/cohab-census.txt absent -- run --write to seed it")
        return 1

    findings = []
    removed = 0
    for key in sorted(set(was) | set(now)):
        f, t = key
        before, after = was.get(key, 0), now.get(key, 0)
        if before == after:
            continue
        if after < before:
            tag, removed = "CENSUS-REMOVED", removed + 1
        else:
            tag = "CENSUS-ADDED"
        findings.append(f"[{tag}] {f}: {before} -> {after}: {t}")
    for line in findings:
        print(line)

    if findings:
        # The two directions are NOT equally alarming, and saying so is the difference
        # between a gate people read and a gate people rubber-stamp.
        print()
        if removed:
            print(
                f"  {removed} REMOVED: a comment-out disappeared. That is the dangerous "
                "direction --\n"
                "  it can mean a flipped .c went back into the build beside its Rust twin, "
                "which\n"
                "  is the 2026-09-19 partial flip that froze the guest while smoke passed on "
                "all\n"
                "  three arches. Read each one before accepting it."
            )
        if len(findings) - removed:
            print(
                f"  {len(findings) - removed} ADDED: a new comment-out. Routine -- it is what "
                "flipping a\n  module to Rust looks like."
            )
        print()
        print("  If every line above is a change you MEANT to make (including one that")
        print("  arrived by merging another lane's flip), re-record the snapshot:")
        print("      python3 tools/cohab_census.py --write")
        print("  and commit tools/cohab-census.txt with that change.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
