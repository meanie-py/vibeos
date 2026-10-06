#!/usr/bin/env python3
"""Split an olewinetest run's failures into the ones Wine shares and the real ones.

ReactOS builds Wine's tests for the "windows" platform, so a `todo_wine` check that
Wine itself fails counts as a hard failure here. Those are gaps against Windows that
upstream Wine has too; the REAL column is what our port gets wrong relative to Wine,
which is what to chase first. (2026-10-04: moniker's 127 failures were 124 todo_wine.)

A failure is todo_wine when its line is in the statement or `{ ... }` block that a
`todo_wine` governs. `todo_wine_if(cond)` is
reported in its own `todo_if` column: whether Wine fails it depends on `cond` at run
time (typelib's `sys != native_sys` is true for every win32 check on a 64-bit build,
which is 30 of its failures), so read the condition before chasing one.

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
TODO_IF_RE = re.compile(r"\btodo_wine_if\s*\(")


def todo_lines(lines: list[str], marker: re.Pattern[str] = TODO_RE) -> set[int]:
    """1-based line numbers of the statement or `{ ... }` block each `marker` governs.

    A marker ending in `(` opens a condition, which may run over several lines
    (ole2.c's `todo_wine_if(!(a || b` / `|| c))`); its statement starts after it.
    """
    inside: set[int] = set()
    depth = 0
    cond = 0  # open parens of a marker's condition
    state = ""  # "pending": marker seen, its statement not begun; "stmt": inside one
    for n, line in enumerate(lines, 1):
        if depth > 0:
            inside.add(n)
            depth += line.count("{") - line.count("}")
            continue
        if state == "stmt":
            inside.add(n)
            state = "" if ";" in line else "stmt"
            continue
        m = None if cond else marker.search(line)
        if m:
            rest, state = line[m.end() :], "pending"
            cond = 1 if m.group().endswith("(") else 0
        elif state == "pending":
            rest = line
        else:
            continue
        while cond and rest:
            ch, rest = rest[0], rest[1:]
            cond += {"(": 1, ")": -1}.get(ch, 0)
        body = rest.strip()
        if cond or not body:
            continue
        inside.add(n)
        if body.startswith("{"):
            depth, state = body.count("{") - body.count("}"), ""
        else:
            state = "" if ";" in body else "stmt"
    return inside


class Source:
    def __init__(self) -> None:
        self.cache: dict[tuple[str, str], tuple[list[str], set[int], set[int]] | None] = {}

    def get(self, suite: str, name: str) -> tuple[list[str], set[int], set[int]] | None:
        key = (suite, name)
        if key not in self.cache:
            path = REPO / "modules" / "rostests" / "winetests" / suite / name
            if path.is_file():
                lines = path.read_text(errors="replace").split("\n")
                self.cache[key] = (lines, todo_lines(lines), todo_lines(lines, TODO_IF_RE))
            else:
                self.cache[key] = None
        return self.cache[key]


def classify(text: str) -> dict[str, list]:
    src = Source()
    res: dict[str, list] = collections.defaultdict(lambda: [0, 0, 0, collections.Counter()])
    for m in FAIL_RE.finditer(text):
        suite, name, n, msg = m.group(1), m.group(2), int(m.group(3)), m.group(4)
        got = src.get(suite, name)
        if got is None:
            continue
        _, todo, todo_if = got
        row = res[f"{suite}:{name}"]
        if n in todo:
            row[0] += 1
        elif n in todo_if:
            row[1] += 1
        else:
            row[2] += 1
            row[3][(n, msg[:90])] += 1
    return res


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("log", type=Path, help="a payload serial log (payload/<arch>-uefi_serial.log)")
    ap.add_argument("--detail", type=int, default=0, help="list up to N real failures per suite")
    args = ap.parse_args()
    res = classify(args.log.read_text(errors="replace"))
    for key, (todo, todo_if, real, detail) in sorted(res.items(), key=lambda kv: -kv[1][2]):
        print(f"{key:28} todo_wine={todo:4} todo_if={todo_if:4} REAL={real:4}")
        for (n, msg), k in sorted(detail.items())[: args.detail]:
            print(f"     {k:3}x :{n} {msg}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
