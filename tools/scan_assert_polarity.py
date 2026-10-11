#!/usr/bin/env python3
"""scan_assert_polarity.py — a translated ASSERT whose guard IS the asserted condition.

THE INCIDENT (2026-10-10)
    C's `ASSERT(x)` lowers to `if (!(x)) RtlAssert("x", ...)`. pcix's PciBuildHackTable
    ends its failure path with

        ASSERT(!NT_SUCCESS(Status));

    and the Rust translation wrote

        if !NT_SUCCESS(Status) {
            RtlAssert(b"!NT_SUCCESS(Status)\\0".as_ptr() as PVOID, ...);
        }

    The guard is the condition, not its negation, so the assert fires on every
    legitimate failure (each boot printed `*** ASSERT FAILED: !NT_SUCCESS(Status)`) and
    stays silent in the one case it exists to catch. Nothing else can see this: it
    compiles, the message text matches the C exactly, and scan_dropped_asserts counts it
    as present.

WHAT COUNTS AS A FINDING
    `if GUARD { RtlAssert(b"MSG\\0" ...` where GUARD and MSG are the same expression
    once Rust spelling is normalised to C (`(*p).f` -> `p->f`, `x.is_null()` ->
    `x == NULL`, `as T` casts dropped). A correct lowering is `if !(MSG)` or an
    equivalent inverted comparison, and is never token-identical to MSG.

Prints `path:line: [ASSERT_POLARITY] ...` per finding; the baseline is empty.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

if __package__:
    from .scan_rawdylib_linkname import iter_rs
else:
    from scan_rawdylib_linkname import iter_rs

SITE = re.compile(
    r'\bif\s+([^{};]+?)\s*\{\s*(?:[\w:]+::)?RtlAssert\(\s*b"((?:[^"\\]|\\.)*?)\\0"',
    re.S,
)
TOKEN = re.compile(r"!=|==|<=|>=|&&|\|\||!|<|>|\w+")


def tokens(expr: str) -> list[str]:
    expr = re.sub(r"\bas\s+[\w:]+", "", expr)
    expr = expr.replace("->", ".").replace("(*", "(").replace(".is_null()", " == NULL")
    return TOKEN.findall(expr)


def scan_text(text: str) -> list[tuple[int, str, str]]:
    if "RtlAssert" not in text:
        return []
    out = []
    for m in SITE.finditer(text):
        guard, msg = m.group(1), m.group(2)
        if tokens(guard) == tokens(msg):
            out.append((text.count("\n", 0, m.start()) + 1, " ".join(guard.split()), msg))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", action="append", help="roots to scan (default: sdk)")
    args = ap.parse_args()
    repo = Path(__file__).resolve().parent.parent
    files = findings = 0
    for root in [repo / r for r in (args.root or ["sdk"])]:
        for path in sorted(iter_rs(root)):
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            files += 1
            rel = path.relative_to(repo) if path.is_relative_to(repo) else path
            for lineno, guard, msg in scan_text(text):
                print(
                    f"{rel}:{lineno}: [ASSERT_POLARITY] `if {guard}` guards ASSERT({msg})"
                    " — the guard must be its negation"
                )
                findings += 1
    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
