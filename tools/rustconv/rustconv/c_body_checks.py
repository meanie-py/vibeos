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
import shutil
import subprocess

from .placement_audit import placement_findings

_GOTO = re.compile(r"\bgoto\s+[A-Za-z_]\w*\s*;")
_GOTO_SHIM = re.compile(r"\bgoto_\w+\s*!?\s*\(|\bfn\s+goto_\w+")
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
    # The dodge: told "no goto", a thinking-on reply for setuplib's FindSupportedSystemPartition
    # invented `goto_NoSystemPartition!()` (and elsewhere `fn goto_Quit()`) -- a shim named
    # after the C label, which no parse check sees and which rustc reports as a missing macro.
    shim = _GOTO_SHIM.search(code)
    if shim:
        out.append(
            f"C `goto` lowered to an invented shim `{shim.group(0).strip()}`; use a labeled "
            "block (`break 'label`) or a state-variable dispatch loop"
        )
    labels = {m.group(1) for m in _C_LABEL.finditer(c_body)} - _NOT_LABELS
    for label in sorted(labels):
        if re.search(rf"(?m)^[ \t]*{re.escape(label)}[ \t]*:[ \t]*$", code):
            out.append(f"C label `{label}:` survived into the Rust")
    return out


def parse_findings(rust: str) -> list[str]:
    """The body does not even PARSE as Rust -- judged by rustc's own parser (rustfmt).

    One unparseable body hides every name-resolution and type error in its crate: setuplib
    sat at "94 errors" with C syntax left in five bodies (`L"x" INF_ARCH`, `a->b`, an
    anonymous `struct { .. }` type) masking ~1,800. A body that does not parse was never a
    translation, so it is rejected at purchase like any other envelope damage. Not
    tree-sitter: it flags 19 of the tree's 3,403 shipping files that rustc accepts.
    """
    exe = shutil.which("rustfmt")
    if exe is None:
        raise RuntimeError(
            "rustfmt not on PATH: the Rust parse check cannot run (rustup component add rustfmt)"
        )
    proc = subprocess.run(
        [exe, "--edition", "2021", "--emit", "stdout"],
        input=rust,
        capture_output=True,
        text=True,
        timeout=60,
    )
    if proc.returncode == 0:
        return []
    first = next((ln for ln in proc.stderr.splitlines() if ln.startswith("error")), "")
    return [f"the Rust does not parse ({first or 'rustfmt rejected it'})"]


def body_findings(c_body: str, rust: str) -> list[str]:
    """Every deterministic finding for one plain-C body (parse, control flow, placement)."""
    return (
        parse_findings(rust)
        + control_flow_findings(c_body, rust)
        + [f.message().replace("\n", "; ") for f in placement_findings(c_body, rust)]
    )
