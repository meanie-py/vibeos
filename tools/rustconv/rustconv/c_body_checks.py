"""Deterministic checks a translated plain-C body must pass, fresh or replayed from cache.

They are cheap, and they exist so a CHEAP model's answer can be judged at acceptance time
instead of in the cargo grind. Measured on setuplib (2026-10-05): DeepSeek with thinking
disabled cost ~1/40th per small function but carried C control flow over literally --
`goto Quit;` and a bare `Quit:` label -- in 2 of 6 TUs, which rustc reports only as a parse
error that hides every other diagnostic in the crate.

Each check is narrow enough to be RIGHT when it fires (a wrong checker bills every retry):
`goto` is never Rust, and a label is flagged only when the C function itself declares that
label, so a struct field split across lines (`Buffer:` / value) cannot trip it.
"""

from __future__ import annotations

import re

from .placement_audit import placement_findings

_GOTO = re.compile(r"\bgoto\s+[A-Za-z_]\w*\s*;")
_C_LABEL = re.compile(r"(?m)^[ \t]*([A-Za-z_]\w*)[ \t]*:(?!:)")
_NOT_LABELS = frozenset({"default", "case", "public", "private", "protected"})
_RS_COMMENT_OR_STRING = re.compile(
    r"//[^\n]*|/\*.*?\*/|b?\"(?:\\.|[^\"\\])*\"|b?'(?:\\.|[^'\\\n])'", re.S
)


def control_flow_findings(c_body: str, rust: str) -> list[str]:
    code = _RS_COMMENT_OR_STRING.sub(" ", rust)
    out = []
    if _GOTO.search(code):
        out.append(
            "C `goto` survived into the Rust; restructure it (a labeled block with "
            "`break 'label`, or a loop) as the c2rust anchor does"
        )
    labels = {m.group(1) for m in _C_LABEL.finditer(c_body)} - _NOT_LABELS
    for label in sorted(labels):
        if re.search(rf"(?m)^[ \t]*{re.escape(label)}[ \t]*:[ \t]*$", code):
            out.append(f"C label `{label}:` survived into the Rust")
    return out


def body_findings(c_body: str, rust: str) -> list[str]:
    """Every deterministic finding for one plain-C body (control flow + constant placement)."""
    return control_flow_findings(c_body, rust) + [
        f.message().replace("\n", "; ") for f in placement_findings(c_body, rust)
    ]
