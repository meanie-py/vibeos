#!/usr/bin/env python3
"""
scan_comment_swallowed_code.py — a statement that a bulk edit folded into a line comment.

    match Result { RESULT::NoMemory => { // DPRINT removed return STATUS_INSUFFICIENT_RESOURCES;
    } RESULT::NotHive => { /* Fail */ // DPRINT removed return STATUS_NOT_REGISTRY_FILE;

A tool that replaced each `DPRINT1(...)` with `// DPRINT removed` did it on lines where the
C's statements had been collapsed onto one line, so the comment ran to the end of the line
and took the `return` after it. Both arms then fell through: cmlib's HvLoadHive carried on
with no valid header for a file that was not a hive at all. It compiles -- an empty match
arm is fine -- and reads like a deliberate note.

The shape: code before a `//`, and a comment that says something was removed and then ends
in what is unmistakably a Rust statement (`return ...;`, `x = ...;`, a call `...);`).
Gated at zero.

Usage:
    python3 tools/scan_comment_swallowed_code.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_comment_swallowed_code.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')
SWALLOWED_RE = re.compile(
    r"\b(?:removed|stripped|elided|dropped)\b\s+"
    r"(?P<stmt>(?:return\b[^;]*|[A-Za-z_][\w.]*\s*=\s*[^=;][^;]*|[A-Za-z_][\w:]*\([^;]*\))\s*;)\s*$"
)


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            for i, line in enumerate(text.splitlines(), 1):
                code = STRING_RE.sub('""', line)
                at = code.find("//")
                if at <= 0 or not code[:at].strip():
                    continue  # a whole-line comment is prose (or quoted C), not swallowed code
                m = SWALLOWED_RE.search(code[at + 2 :])
                if m:
                    findings.append((f, i, " ".join(m.group("stmt").split())))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, stmt in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "comment-swallowed" if lint else "statement inside a trailing comment:"
        print(f"{rel}:{lineno}: {tag} {stmt}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): put the statement back on its own line.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
