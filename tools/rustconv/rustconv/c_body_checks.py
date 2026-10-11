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
import sys
from pathlib import Path

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
    if _c_dowhile_with_continue(c_body):
        skipped = _rust_skipped_conditions(rust)
        if skipped:
            out.append(
                f"a `continue` skips the do-while condition in {skipped} loop(s): C's "
                "`continue` runs the condition; use `loop { 'body: { .. break 'body; .. } "
                "if !(c) { break; } }`"
            )
    out += _bitor_case_patterns(c_body, rust)
    out += invented_flag_assignments(c_body, code)
    labels = {m.group(1) for m in _C_LABEL.finditer(c_body)} - _NOT_LABELS
    for label in sorted(labels):
        if re.search(rf"(?m)^[ \t]*{re.escape(label)}[ \t]*:[ \t]*$", code):
            out.append(f"C label `{label}:` survived into the Rust")
    return out


_C_COMMENT_OR_STRING = re.compile(
    r"//[^\n]*|/\*.*?\*/|L?\"(?:\\.|[^\"\\])*\"|L?'(?:\\.|[^'\\])'", re.S
)
_C_LOOP_OPEN = re.compile(r"\b(?:do\s*\{|for\s*\(|while\s*\()")


def _c_loop_body_open(text: str, m: re.Match[str]) -> int:
    """Index of the `{` opening the loop body `m` starts, or -1 (a braceless body)."""
    if m.group(0).startswith("do"):
        return m.end() - 1
    depth, i = 0, m.end() - 1
    while i < len(text):
        depth += {"(": 1, ")": -1}.get(text[i], 0)
        if depth == 0:
            break
        i += 1
    rest = text[i + 1 :]
    return i + 1 + (len(rest) - len(rest.lstrip())) if rest.lstrip().startswith("{") else -1


def _c_brace_end(text: str, open_idx: int) -> int:
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return -1


def _c_dowhile_with_continue(c_body: str) -> bool:
    """A C `do { .. } while` whose OWN `continue` (not a nested loop's) jumps to its condition."""
    text = _C_COMMENT_OR_STRING.sub(" ", c_body)
    for m in re.finditer(r"\bdo\s*\{", text):
        open_idx = m.end() - 1
        close = _c_brace_end(text, open_idx)
        if close < 0 or not re.match(r"\s*while\b", text[close + 1 :]):
            continue
        body = list(text[open_idx + 1 : close])
        inner = "".join(body)
        for n in _C_LOOP_OPEN.finditer(inner):
            k = _c_loop_body_open(inner, n)
            e = _c_brace_end(inner, k) if k >= 0 else -1
            for q in range(k, e + 1 if e >= 0 else k):
                body[q] = " "
        if re.search(r"\bcontinue\s*;", "".join(body)):
            return True
    return False


def _bitor_case_patterns(c_body: str, rust: str) -> list[str]:
    """C `case A|B:` arms the Rust wrote as an or-pattern (tools/scan_case_bitor_pattern.py)."""
    if not re.search(r"\bcase\s+\(?\s*\w+\s*\|", c_body):
        return []
    repo = str(Path(__file__).resolve().parents[3])
    if repo not in sys.path:
        sys.path.insert(0, repo)
    from tools.scan_case_bitor_pattern import c_bitor_labels, scan_text

    return [
        f"`{alts}` is an or-pattern matching EITHER constant, but the C `case` is their "
        "bitwise OR (one value); write `x if x == A | B =>`"
        for _line, alts in scan_text(rust, c_bitor_labels(c_body))
    ]


def _rust_skipped_conditions(rust: str) -> int:
    """Loops the do-while lint (tools/scan_dowhile_continue.py) would flag, by its own code."""
    repo = str(Path(__file__).resolve().parents[3])
    if repo not in sys.path:
        sys.path.insert(0, repo)
    from tools.scan_dowhile_continue import scan_text

    return len(scan_text(rust))


#: What a C assignment may spell for the values 0 and 1.
_C_VALUE_SPELLINGS = {
    "0": {
        "FALSE",
        "NULL",
        "STATUS_SUCCESS",
        "ERROR_SUCCESS",
        "S_OK",
        "NO_ERROR",
        "UNICODE_NULL",
        "ANSI_NULL",
        "'\\0'",
        "L'\\0'",
    },
    "1": {"TRUE"},
}
_RS_FLAG_ASSIGN = re.compile(
    r"(?m)(?P<lead>(?:^|[;{}])[^\n;{}]*?)(?<![\w.*\])])(?P<name>[A-Za-z_]\w*)\s*=(?!=)\s*\(?\s*"
    r"(?P<val>[01])(?:[iu](?:8|16|32|64|size))?\s*\)?\s*(?:as\s+[\w:]+\s*)*;"
)
_C_ASSIGN = re.compile(r"(?<![\w.>=!<*\])])([A-Za-z_]\w*)\s*=(?!=)\s*")


def _c_assignments(c_code: str):
    """``(name, rhs)`` for each C assignment, the RHS ending where the EXPRESSION does.

    Not at the next `;`: `for (i = 0, p = es->text; ...)` gave `i` the value
    `0, p = es->text`, and `if ((page = h / w) < 1) page = 1;` swallowed the second
    assignment into the first one's RHS -- three correct comctl32 bodies (EDIT_EM_ReplaceSel,
    LISTBOX_GetMaxTopIndex, LISTBOX_UpdatePage) were refused as "invented" values the C
    spells plainly. The RHS stops at `;`, a top-level comma, or an unbalanced `)`/`]`/`}`;
    every assignment is found independently, nested ones included.
    """
    for m in _C_ASSIGN.finditer(c_code):
        depth, end = 0, m.end()
        while end < len(c_code):
            ch = c_code[end]
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                if depth == 0:
                    break
                depth -= 1
            elif ch == ";" or (ch == "," and depth == 0):
                break
            end += 1
        yield m.group(1), c_code[m.end() : end]


def invented_flag_assignments(c_body: str, rust: str) -> list[str]:
    """`Status = 0;` where the C function never gives Status a value that is 0 (or 1).

    bootsup.c's InstallBootManagerAndBootEntries assigns Status only from calls and two
    failure codes; the translation's early return on GetFileSystemName() failure added
    `Status = 0; // keep as in C` and so reported SUCCESS for a failed mount. No seal sees it:
    small literals are not sealed, and nothing was dropped. The check fires only where the C
    does assign the variable (so its values are known) and never with a spelling of that
    value; a `let` initialiser is exempt (repair gives an uninitialised C local a zero).
    """
    c_code = _C_COMMENT_OR_STRING.sub(" ", c_body)
    c_values: dict[str, set[str]] = {}
    for first, raw_rhs in _c_assignments(c_code):
        # A chained assignment gives every name in it the final value: usetup's
        # `Pos = Length = 0;` was split into two statements by a correct translation, and
        # `Length = 0` was refused on every sample because only `Pos` was recorded.
        names = [first]
        rhs = raw_rhs.strip()
        while chained := re.match(r"([A-Za-z_]\w*)\s*=(?!=)\s*(.+)$", rhs, re.S):
            names.append(chained.group(1))
            rhs = chained.group(2).strip()
        rhs = re.sub(r"^\(\s*\w+\s*\)\s*", "", rhs).strip("() ")
        for name in names:
            c_values.setdefault(name, set()).add(rhs)
    out = []
    for m in _RS_FLAG_ASSIGN.finditer(rust):
        if re.search(r"\blet\b", m.group("lead")):
            continue
        name, val = m.group("name"), m.group("val")
        rhs = c_values.get(name)
        if not rhs:
            continue
        literal = {re.sub(r"(?i)^(?:0x)?0*([01])[ul]*$", r"\1", r) for r in rhs}
        if rhs & _C_VALUE_SPELLINGS[val] or val in literal:
            continue
        out.append(
            f"`{name} = {val}` is not in the C: it assigns {name} only "
            f"{', '.join(sorted(rhs))[:120]} -- an invented success/flag value"
        )
    return sorted(set(out))


_C_PTR_TYPEDEF_DECL = re.compile(
    r"(?<![\w*])((?:P|LP)[A-Z][A-Z0-9_]*)\s+(?!\*)([A-Za-z_]\w*)\s*(?=[;=,)])"
)


def pointer_typedef_indirection_findings(c_body: str, rust: str) -> list[str]:
    """A local or parameter the C declares as a pointer TYPEDEF (`PFILEPATHS_W x`), which the
    Rust declares one level deeper (`x: *mut PFILEPATHS_W`).

    usetup's FileCopyCallback and four pages did exactly this: `Param1 as *mut PFILEPATHS_W`
    and then `(*(*x)).Target` -- the struct's first field read as the struct's address, so
    every progress message (and every keypress, through PINPUT_RECORD) read garbage. It
    compiles; only the C's own declaration says it is wrong. A C `PFOO *x` is left alone
    (setuplib's `PCM_INT13_DRIVE_PARAMETER *Int13Drives` really is two levels).
    """
    c_code = _C_COMMENT_OR_STRING.sub(" ", c_body)
    c_types: dict[str, set[str]] = {}
    for m in _C_PTR_TYPEDEF_DECL.finditer(c_code):
        c_types.setdefault(m.group(2), set()).add(m.group(1))
    out = []
    for m in re.finditer(
        r"\b(?:let\s+(?:mut\s+)?)?([A-Za-z_]\w*)\s*:\s*\*(?:mut|const)\s+((?:P|LP)[A-Z][A-Z0-9_]*)\b",
        rust,
    ):
        name, ty = m.group(1), m.group(2)
        if ty in c_types.get(name, set()):
            out.append(
                f"`{name}: *{'mut' if '*mut' in m.group(0) else 'const'} {ty}` adds a level of "
                f"indirection: the C declares `{ty} {name}`, and {ty} is already a pointer"
            )
    return sorted(set(out))


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
        + pointer_typedef_indirection_findings(c_body, rust)
        + [f.message().replace("\n", "; ") for f in placement_findings(c_body, rust)]
    )
