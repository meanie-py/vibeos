#!/usr/bin/env python3
"""scan_match_arm_break.py — a C switch's `break;` that now leaves the LOOP.

THE INCIDENT
    mf's session_check_topology walks the topology's nodes with a `for` and a `switch` on
    the node type; the source-node case ends in `break;`, which in C leaves the switch and
    goes on to `IMFTopologyNode_Release(node)` and the next node. Translated to a `match`
    inside a `while`, the same `break;` leaves the WHILE: the first source node's
    reference leaked on every SetTopology, and Wine's mf suite counted eight of them.
    rustc accepts it; nothing about the arm looks wrong in isolation.

WHAT COUNTS AS A FINDING
    An unlabelled `break;` that is the LAST statement of a match-arm BLOCK (`PAT => { ..;
    break; }`, other statements before it) while some enclosing block is a loop. That is
    the exact shape `case X: ...; break;` takes. Not flagged:
      * `PAT => break,` -- an expression arm is written on purpose;
      * `loop { match x { .. => { ..; break; } } }` -- the once-through loop that stands in
        for a C switch, where leaving the loop IS leaving the switch;
      * a `break` inside a loop nested in the arm itself.
    Hand-written Rust can mean it; those go in the baseline with the reason in the triage.

    Findings go to stdout for the baseline comparison; the tally goes to stderr.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

_LOOP_HEAD = re.compile(r"(?:\bwhile\b[^{;]*|\bfor\b[^{;]*)$")
_TOKEN = re.compile(r"[{}]|\bbreak\s*;")
_CLOSE = re.compile(r"\s*\}")
_MATCH = re.compile(r"\s*match\b")


_NOISE = re.compile(
    r"//[^\n]*"  # line comment
    r"|/\*.*?\*/"  # block comment (not nested; enough for blanking braces)
    r'|b?"(?:\\.|[^"\\])*"'  # string / byte string
    r"|b?'(?:\\.|[^'\\\n])'",  # char / byte char (a lifetime has no closing quote)
    re.S,
)


def _blank(text: str) -> str:
    """Comments and string/char literals blanked to spaces; newlines and offsets kept."""

    def spaces(m: re.Match[str]) -> str:
        g = m.group(0)
        return " " * len(g) if "\n" not in g else re.sub(r"[^\n]", " ", g)

    return _NOISE.sub(spaces, text)


def scan_text(text: str) -> list[int]:
    """Line numbers of each `break;` that leaves a loop where the C left only a switch."""
    if "break" not in text or "=>" not in text:
        return []
    s = _blank(text)
    stack: list[tuple[str, int]] = []
    found: list[int] = []
    for m in _TOKEN.finditer(s):
        tok = m.group(0)
        if tok == "{":
            before = s[max(0, m.start() - 400) : m.start()].rstrip()
            if before.endswith("=>"):
                kind = "arm"
            elif before.endswith("loop"):
                kind = "loop-kw"
            elif _LOOP_HEAD.search(before[-200:]):
                kind = "loop"
            else:
                kind = "other"
            stack.append((kind, m.start()))
        elif tok == "}":
            if stack:
                stack.pop()
        elif stack and stack[-1][0] == "arm":
            if not _CLOSE.match(s, m.end()) or not s[stack[-1][1] + 1 : m.start()].strip():
                continue
            if not any(kind in ("loop", "loop-kw") for kind, _ in stack[:-1]):
                continue
            # The once-through `loop { match .. }` only: a `while`/`for` whose body opens
            # with a match is the mf shape itself.
            if (
                len(stack) >= 3
                and stack[-3][0] == "loop-kw"
                and _MATCH.match(s, stack[-3][1] + 1)
            ):
                continue
            found.append(s.count("\n", 0, m.start()) + 1)
    return found


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        action="append",
        default=None,
        help="directory to scan (repeatable); defaults to sdk/",
    )
    args = ap.parse_args()
    repo = Path(__file__).resolve().parent.parent
    findings = files = 0
    for root in [repo / r for r in (args.root or ["sdk"])]:
        for path in sorted(source_files(root)):
            files += 1
            for lineno in scan_text(path.read_text(encoding="utf-8", errors="replace")):
                print(f"{path.relative_to(repo)}:{lineno}: MATCH_ARM_BREAK")
                findings += 1
    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
