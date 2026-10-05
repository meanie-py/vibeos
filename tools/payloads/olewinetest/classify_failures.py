#!/usr/bin/env python3
"""Split an olewinetest run's failures into the ones Wine shares and the real ones.

ReactOS builds Wine's tests for the "windows" platform, so a `todo_wine` check that
Wine itself fails counts as a hard failure here. Those are gaps against Windows that
upstream Wine has too; the REAL column is what our port gets wrong relative to Wine,
which is what to chase first. (2026-10-04: moniker's 127 failures were 124 todo_wine.)

A failure is todo_wine when the `ok()` line sits under a `todo_wine` statement in the
three lines above it, or inside a `todo_wine { ... }` block. `todo_wine_if(...)` is
conditional on the platform at runtime and is NOT counted as todo here.

    python3 tools/payloads/olewinetest/classify_failures.py <serial.log> [--detail N]
"""

from __future__ import annotations

import argparse
import collections
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
SUITE_DIRS = ("ole32", "oleaut32")
FAIL_RE = re.compile(
    r"\[APITEST:(\w+)_winetest\.exe [^\]]*\] ([a-z_0-9]+\.c):(\d+): Test failed: ([^\r\n]{0,100})"
)
TODO_RE = re.compile(r"\btodo_wine\b(?!_if)")


def todo_lines(lines: list[str]) -> set[int]:
    """1-based line numbers inside a `todo_wine { ... }` block."""
    inside: set[int] = set()
    depth = 0
    armed = False  # saw `todo_wine`, its statement or block has not started yet
    for n, line in enumerate(lines, 1):
        if depth > 0:
            inside.add(n)
            depth += line.count("{") - line.count("}")
            continue
        m = TODO_RE.search(line)
        rest = line[m.end() :] if m else line
        if m or armed:
            if rest.lstrip().startswith("{"):
                depth = rest.count("{") - rest.count("}")
                inside.add(n)
                armed = False
            else:
                armed = ";" not in rest
    return inside


class Source:
    def __init__(self) -> None:
        self.cache: dict[tuple[str, str], tuple[list[str], set[int]] | None] = {}

    def get(self, suite: str, name: str) -> tuple[list[str], set[int]] | None:
        key = (suite, name)
        if key not in self.cache:
            path = REPO / "modules" / "rostests" / "winetests" / suite / name
            if path.is_file():
                lines = path.read_text(errors="replace").split("\n")
                self.cache[key] = (lines, todo_lines(lines))
            else:
                self.cache[key] = None
        return self.cache[key]


def classify(text: str) -> dict[str, list]:
    src = Source()
    res: dict[str, list] = collections.defaultdict(lambda: [0, 0, collections.Counter()])
    for m in FAIL_RE.finditer(text):
        suite, name, n, msg = m.group(1), m.group(2), int(m.group(3)), m.group(4)
        got = src.get(suite, name)
        if got is None:
            continue
        lines, blocks = got
        before = " ".join(lines[max(0, n - 4) : n - 1])
        todo = n in blocks or bool(TODO_RE.search(before)) or bool(TODO_RE.search(lines[n - 1]))
        row = res[f"{suite}:{name}"]
        if todo:
            row[0] += 1
        else:
            row[1] += 1
            row[2][(n, msg[:90])] += 1
    return res


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("log", type=Path, help="a payload serial log (payload/<arch>-uefi_serial.log)")
    ap.add_argument("--detail", type=int, default=0, help="list up to N real failures per suite")
    args = ap.parse_args()
    res = classify(args.log.read_text(errors="replace"))
    for key, (todo, real, detail) in sorted(res.items(), key=lambda kv: -kv[1][1]):
        print(f"{key:28} todo_wine={todo:4} REAL={real:4}")
        for (n, msg), k in sorted(detail.items())[: args.detail]:
            print(f"     {k:3}x :{n} {msg}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
