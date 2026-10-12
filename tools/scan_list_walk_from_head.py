#!/usr/bin/env python3
"""
scan_list_walk_from_head.py — a list walk whose cursor starts AT the head it stops at.

THE CLASS. wine/list.h's `LIST_FOR_EACH_ENTRY(cursor, head, T, field)` starts at
`head->next` and stops when the cursor's `field` is `head` again. A hand lowering that
starts the cursor at the head itself,

    e = (addr_of_mut!(RunningObjectTable) as *mut u8).sub(offset_of!(rot_entry, entry)) as _;
    while addr_of!((*e).entry) != addr_of_mut!(RunningObjectTable) { ... }

tests the stop condition before the first step and runs ZERO times: every lookup misses,
every count is 0, nothing is ever freed. rpcss's IrotIsRunning and IrotGetObject shipped
this way (2026-10-11): the running-object table never found a registered object. It
compiles and does not crash, so nothing but reading it catches it.

WHAT IS FLAGGED: an assignment whose right-hand side is a container-of (`offset_of!`,
`CONTAINING_RECORD!`, `LIST_ENTRY!`, `list_entry`) of a head `H` reached without a step
(no `.next`/`.prev`/`.Flink`/`.Blink`/`list_head(`), followed directly by a `while` whose
condition names both the cursor and `H`. A deliberate one carries
`// nolint: list-walk-from-head (why)` on the `while` line.

Findings are keyed by file and enclosing function, not line, so they survive edits.

Usage:
    python3 tools/scan_list_walk_from_head.py [--lint] [paths...]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
NOLINT = "nolint: list-walk-from-head"

_LEX = re.compile(
    r"/\*.*?\*/|//[^\n]*|b?r#*\"(?:.|\n)*?\"#*|b?\"(?:\\.|[^\"\\])*\"|b?'(?:\\.|[^\\'\n])'",
    re.S,
)
_ASSIGN = re.compile(
    r"(?:\blet\s+(?:mut\s+)?)?\b(?P<var>[A-Za-z_]\w*)\s*(?::[^=;{}]+)?(?<![=!<>])=(?!=)"
    r"(?P<rhs>[^;]*);"
)
_CONTAINER = re.compile(r"offset_of!|CONTAINING_RECORD!|LIST_ENTRY!|\blist_entry\b")
_HEAD = re.compile(
    r"addr_of(?:_mut)?!\(\s*(?P<a>[\w.()*]+?)\s*\)|&raw\s+(?:mut|const)\s+(?P<b>[\w.]+)"
    r"|&mut\s+(?P<c>[\w.]+)"
)
_STEP = re.compile(r"\.(?:next|prev|Flink|Blink)\b|\blist_(?:head|tail|next|prev)\s*\(")
_FN = re.compile(r"\bfn\s+([A-Za-z_]\w*)")


def _mask(text: str) -> str:
    def blank(m: re.Match) -> str:
        s = m.group(0)
        return re.sub(r"[^\n]", " ", s)

    return _LEX.sub(blank, text)


def _while_condition(masked: str, start: int) -> tuple[int, str] | None:
    """(index of `while`, its condition) when the next statement at `start` is a `while`."""
    m = re.compile(r"\s*while\b").match(masked, start)
    if not m:
        return None
    depth = 0
    for j in range(m.end(), len(masked)):
        c = masked[j]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif c == "{" and depth == 0:
            return m.end() - 5, masked[m.end() : j]
    return None


def scan_text(text: str) -> list[tuple[int, str, str]]:
    """(line, function, cursor) for each walk that starts at its own stop head."""
    masked = _mask(text)
    out = []
    for m in _ASSIGN.finditer(masked):
        rhs = m.group("rhs")
        if not _CONTAINER.search(rhs) or _STEP.search(rhs):
            continue
        heads = {next(g for g in h.groups() if g) for h in _HEAD.finditer(rhs)}
        if not heads:
            continue
        w = _while_condition(masked, m.end())
        if w is None:
            continue
        at, cond = w
        var = m.group("var")
        if not re.search(rf"\b{re.escape(var)}\b", cond):
            continue
        if not any(re.search(rf"(?<![\w.]){re.escape(h)}(?![\w(])", cond) for h in heads):
            continue
        line_no = text.count("\n", 0, at) + 1
        line = text.splitlines()[line_no - 1] if text else ""
        if NOLINT in line:
            continue
        fns = list(_FN.finditer(masked, 0, m.start()))
        out.append((line_no, fns[-1].group(1) if fns else "?", var))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument(
        "--lint", action="store_true", help="terse, line-free output; exit 1 on findings"
    )
    args = ap.parse_args()
    files: list[Path] = []
    for p in args.paths or [RUST_ROOT]:
        files += [p] if p.is_file() else sorted(source_files(p))
    findings = []
    for f in files:
        rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
        for line, fn, var in scan_text(f.read_text(errors="replace")):
            findings.append((str(rel), line, fn, var))
    for rel, line, fn, var in findings:
        if args.lint:
            print(f"{rel}: {fn}: list walk `{var}` starts at the head it stops at (runs 0 times)")
        else:
            print(f"{rel}:{line}: {fn}: `{var}` starts AT its stop head; start at head->next")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
