#!/usr/bin/env python3
"""
scan_case_bitor_pattern.py — a C `case A|B:` translated as a Rust or-pattern.

THE CLASS. In C, `case TVGN_CARET|TVSI_NOSINGLEEXPAND:` is ONE label whose value is
the bitwise OR of the two constants. Written the same way in a Rust `match` arm,

    TVGN_CARET | TVSI_NOSINGLEEXPAND => { ... }

it means the opposite: two alternatives, matching EITHER value and never the OR. It
compiles cleanly and nothing downstream objects. comctl32's TREEVIEW_SelectItem shipped
it (2026-10-09): the combined code fell through to the default arm, and a plain
TVGN_CARET took the no-single-expand path. The faithful translations are a guard
(`x if x == A | B =>`) or a named const holding the OR.

WHAT IS FLAGGED: a Rust match arm (or `matches!` pattern) whose or-alternatives, by
their last path segment, are exactly the identifier set of some C `case` label that
ORs those identifiers together. The C oracle is every tracked C/C++ file in the repo
plus any C under the scanned paths, so a Rust port needs no pairing with its source.
A Rust or-pattern that really is `case A: case B:` in its own C, while some OTHER C
file ORs the same pair, is the false-positive shape: mark it
`// nolint: case-bitor-pattern (why)` on the arm line.

Usage:
    python3 tools/scan_case_bitor_pattern.py [--lint] [paths...]
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
NOLINT = "nolint: case-bitor-pattern"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')
CHAR_RE = re.compile(r"b?'(?:\\.|[^'\\])'")
IDENT = r"[A-Za-z_]\w*"
# `case A | B:` / `case (A|B|C):` -- only labels made of bare identifiers joined by `|`.
C_CASE_RE = re.compile(
    rf"\bcase\s+\(?\s*({IDENT}(?:\s*\|\s*{IDENT})+)\s*\)?\s*:(?!:)"
)
PATH = rf"{IDENT}(?:::{IDENT})*"
# An arm starts a line, or follows `{` / `,`; a guard's `x == A | B` follows `==`.
ARM_RE = re.compile(rf"(?:^|[{{,])[ \t]*({PATH}(?:\s*\|\s*{PATH})+)\s*(?:=>|\bif\b)", re.M)
MATCHES_RE = re.compile(rf"\bmatches!\s*\([^,()]*(?:\([^()]*\)[^,()]*)*,\s*({PATH}(?:\s*\|\s*{PATH})+)\s*[,)]")


def _blank(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def _idents(expr: str) -> frozenset[str]:
    return frozenset(p.strip().rsplit("::", 1)[-1] for p in expr.split("|"))


def c_bitor_labels(text: str) -> set[frozenset[str]]:
    text = COMMENT_RE.sub(_blank, text)
    return {_idents(m.group(1)) for m in C_CASE_RE.finditer(text)}


def repo_c_labels() -> set[frozenset[str]]:
    files = subprocess.run(
        ["git", "-C", str(REPO), "grep", "-lE", r"\bcase\s+\(?\s*\w+\s*\|", "--",
         "*.c", "*.cpp", "*.h"],
        capture_output=True, text=True, check=False,
    )
    if files.returncode not in (0, 1):
        raise SystemExit(f"scan_case_bitor_pattern: git grep failed: {files.stderr.strip()}")
    out: set[frozenset[str]] = set()
    for rel in files.stdout.split():
        out |= c_bitor_labels((REPO / rel).read_text(errors="replace"))
    return out


def _patterns(text: str) -> tuple[re.Pattern, ...]:
    return (ARM_RE, MATCHES_RE) if "matches!" in text else (ARM_RE,)


def scan_text(text: str, labels: set[frozenset[str]]) -> list[tuple[int, str]]:
    # Most files hold no candidate at all: match the raw text first, and pay for the
    # comment/string blanking only where a candidate is in the label set.
    if not any(
        _idents(m.group(1)) in labels for rx in _patterns(text) for m in rx.finditer(text)
    ):
        return []
    code = CHAR_RE.sub(_blank, STRING_RE.sub(_blank, COMMENT_RE.sub(_blank, text)))
    lines = text.splitlines()
    hits = []
    for rx in _patterns(text):
        for m in rx.finditer(code):
            alts = _idents(m.group(1))
            if len(alts) < 2 or alts not in labels:
                continue
            line = code.count("\n", 0, m.start(1)) + 1
            if NOLINT in lines[line - 1]:
                continue
            hits.append((line, " | ".join(sorted(alts))))
    return sorted(hits)


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument("--lint", action="store_true", help="terse, line-free output; exit 1 on findings")
    args = ap.parse_args()

    labels = repo_c_labels()
    rs_files: list[Path] = []
    for p in args.paths or [RUST_ROOT]:
        if p.is_file():
            rs_files.append(p)
            continue
        rs_files += sorted(source_files(p))
        for pat in ("*.c", "*.cpp"):
            for c in source_files(p, pat):
                labels |= c_bitor_labels(c.read_text(errors="replace"))

    findings = []
    for f in rs_files:
        rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
        for line, alts in scan_text(f.read_text(errors="replace"), labels):
            findings.append((str(rel), line, alts))
    for rel, line, alts in findings:
        if args.lint:
            print(f"{rel}: or-pattern `{alts}` is a C `case` that ORs them into one value")
        else:
            print(f"{rel}:{line}: `{alts}` matches EITHER; the C label is their bitwise OR")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
