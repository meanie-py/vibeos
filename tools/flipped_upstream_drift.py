#!/usr/bin/env python3
"""Flipped C files that upstream changed AFTER their Rust port was cut.

A cohab flip comments a `.c` out of CMake and lets Rust own its symbols; the C stays on
disk. The upstream sync keeps merging into that dead C, and nothing carries the change
into the Rust: ole32's ports were translated from the pre-wine-10.0 C, the wine-10.0 sync
landed on top, and the Rust never gained the objref moniker that ole32.spec, oleproxy and
ole32_winetest all name (2026-10-04). Smoke cannot see this; the C compiles to nothing.

METHOD (so a second count can be compared against this one): every comment line in a CMake
file that names an existing `.c`/`.cpp` (cohab_census's file walk) is a flip. Its flip
commit is the latest commit that touched a comment naming that file in that CMake file
(`git log -1 -G`), which, if the comment was edited since, is LATER than the real flip and
so undercounts. The drift is the non-merge commits in `flip..merge-base(HEAD, upstream)`
that touch the C: upstream commits merged after the flip. Our own edits to dead C are not
counted. A commit counted here may already be mirrored in the Rust by hand, so this is a
review queue, not a list of bugs.

USAGE
    python3 tools/flipped_upstream_drift.py [--upstream upstream/master] [--top N]
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cohab_census

REPO = cohab_census.REPO
PATH_RE = re.compile(r"^#\s*(\$\{[A-Z_]+\}/)?([\w./+-]+\.(?:c|cpp|cc))\b")


def git(*args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(REPO), *args], capture_output=True, text=True
    ).stdout.strip()


def flipped_sources() -> list[tuple[str, str]]:
    """(cmake file, C path) for every commented-out source that still exists."""
    out = []
    for cm in cohab_census.cmake_files():
        for line in cm.read_text(errors="replace").splitlines():
            m = PATH_RE.match(line.strip())
            if not m:
                continue
            c = (REPO / m.group(2)) if m.group(1) else (cm.parent / m.group(2))
            if c.is_file():
                out.append((str(cm.relative_to(REPO)), str(c.resolve().relative_to(REPO))))
    return sorted(set(out))


def drift(cmake: str, c: str, base: str) -> tuple[int, str, str] | None:
    name = re.escape(Path(c).name)
    flip = git("log", "-1", "--format=%H", "-G", rf"^\s*#.*{name}", "--", cmake)
    if not flip:
        return None
    n = git("rev-list", "--count", "--no-merges", f"{flip}..{base}", "--", c)
    if not n or int(n) == 0:
        return None
    return int(n), c, git("diff", "--shortstat", flip, base, "--", c)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--upstream", default="upstream/master")
    ap.add_argument("--top", type=int, default=40, help="directories to list (0 = all files)")
    a = ap.parse_args()
    base = git("merge-base", "HEAD", a.upstream)
    if not base:
        print(f"no merge-base with {a.upstream}; fetch it first", file=sys.stderr)
        return 2
    srcs = flipped_sources()
    with ThreadPoolExecutor(16) as ex:
        rows = [r for r in ex.map(lambda s: drift(*s, base), srcs) if r]
    print(
        f"{len(srcs)} flipped C sources; {len(rows)} with upstream commits merged after the flip "
        f"(upstream merge-base {base[:11]})"
    )
    if a.top == 0:
        for n, c, st in sorted(rows, key=lambda r: r[1]):
            print(f"{n:4d} commits  {c}  {st}")
        return 0
    by: dict[str, list[tuple[int, str, str]]] = {}
    for r in rows:
        by.setdefault("/".join(r[1].split("/")[:3]), []).append(r)

    def added(st: str) -> int:
        m = re.search(r"(\d+) insertion", st)
        return int(m.group(1)) if m else 0

    for d in sorted(by, key=lambda d: -sum(added(r[2]) for r in by[d]))[: a.top]:
        print(f"{len(by[d]):4d} files  +{sum(added(r[2]) for r in by[d]):6d} lines  {d}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
