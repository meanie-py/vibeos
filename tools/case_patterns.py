"""Bitwise case-pattern helpers retained for the translation audit.

The source scanner lives in tools/vibeos-lint/src/rules/case_bitor_pattern.rs.
"""

from __future__ import annotations

import re

NOLINT = "nolint: case-bitor-pattern"
COMMENT_RE = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')
CHAR_RE = re.compile(r"b?'(?:\\.|[^'\\])'")
IDENT = r"[A-Za-z_]\w*"
# `case A | B:` / `case (A|B|C):` -- only labels made of bare identifiers joined by `|`.
C_CASE_RE = re.compile(rf"\bcase\s+\(?\s*({IDENT}(?:\s*\|\s*{IDENT})+)\s*\)?\s*:(?!:)")
PATH = rf"{IDENT}(?:::{IDENT})*"
# An arm starts a line, or follows `{` / `,`; a guard's `x == A | B` follows `==`.
ARM_RE = re.compile(rf"(?:^|[{{,])[ \t]*({PATH}(?:\s*\|\s*{PATH})+)\s*(?:=>|\bif\b)", re.M)
MATCHES_RE = re.compile(
    rf"\bmatches!\s*\([^,()]*(?:\([^()]*\)[^,()]*)*,\s*({PATH}(?:\s*\|\s*{PATH})+)\s*[,)]"
)


def _blank(m: re.Match) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def _idents(expr: str) -> frozenset[str]:
    return frozenset(p.strip().rsplit("::", 1)[-1] for p in expr.split("|"))


def c_bitor_labels(text: str) -> set[frozenset[str]]:
    text = COMMENT_RE.sub(_blank, text)
    return {_idents(m.group(1)) for m in C_CASE_RE.finditer(text)}


def _patterns(text: str) -> tuple[re.Pattern, ...]:
    return (ARM_RE, MATCHES_RE) if "matches!" in text else (ARM_RE,)


def scan_text(text: str, labels: set[frozenset[str]]) -> list[tuple[int, str]]:
    # Most files hold no candidate at all: match the raw text first, and pay for the
    # comment/string blanking only where a candidate is in the label set.
    if not any(_idents(m.group(1)) in labels for rx in _patterns(text) for m in rx.finditer(text)):
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
