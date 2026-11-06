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
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools import qemu_infra_compat as native  # noqa: E402

TODO_RE = re.compile(r"\btodo_wine\b(?!_if)")
TODO_IF_RE = re.compile(r"\btodo_wine_if\s*\(")


def todo_lines(lines: list[str], marker: re.Pattern[str] = TODO_RE) -> set[int]:
    if marker.pattern not in (TODO_RE.pattern, TODO_IF_RE.pattern):
        raise ValueError("unsupported Wine todo marker")
    return set(
        native.call(
            "wine_failures_todo_lines",
            {"lines": lines, "conditional": marker.pattern == TODO_IF_RE.pattern},
            REPO,
        )
    )


def classify(text: str) -> dict[str, list]:
    rows = native.call("wine_failures_classify", {"text": text}, REPO)
    return {
        row["name"]: [
            row["todo"],
            row["todo_if"],
            row["real"],
            collections.Counter({(d["line"], d["message"]): d["count"] for d in row["detail"]}),
        ]
        for row in rows
    }


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("log", type=Path, help="a payload serial log")
    ap.add_argument("--detail", type=int, default=0)
    args = ap.parse_args()
    for key, (todo, todo_if, real, detail) in sorted(
        classify(args.log.read_text(errors="replace")).items(), key=lambda kv: -kv[1][2]
    ):
        print(f"{key:28} todo_wine={todo:4} todo_if={todo_if:4} REAL={real:4}")
        for (n, msg), k in sorted(detail.items())[: args.detail]:
            print(f"     {k:3}x :{n} {msg}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
