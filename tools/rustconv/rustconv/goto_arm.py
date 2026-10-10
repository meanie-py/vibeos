"""A `case` arm's trailing `goto` dropped, so the Rust arm falls into the code after the `match`.

pciidex's PnP dispatcher ends its START_DEVICE arm `goto CompleteIrp;`, and the code after
the `switch` forwards the IRP down the stack. The Rust arm simply ended, so it fell into that
forward path: the IRP went down a second time and the start status was discarded. Nothing
fails at boot. The early-exit `goto`s inside `if`s in the same arm survived; only the
unconditional last one was lost.

Decidable from the two texts: the C names the `case` label, and sealing keeps that name in the
Rust pattern. A C arm whose last statement is `goto L;` must map to a Rust arm whose last
statement leaves it (`return`, `break`, `continue`, a panic), unless `L:` is the very next
statement after the `switch`, where falling out IS the jump.
"""

from __future__ import annotations

import re
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Protocol

from .logical_not import _fn_bodies
from .loop_step import _match_brace, _strip, _top_statements


class _Decls(Protocol):
    @property
    def goto_arms(self) -> tuple[tuple[str, str], ...]: ...  # (case label, goto target)


@dataclass(frozen=True)
class Fallthrough:
    fn: str
    case: str
    label: str
    line: int


_SWITCH = re.compile(r"\bswitch\s*\(")
_CASE = re.compile(r"\bcase\s+([A-Za-z_]\w*)\s*:|\bdefault\s*:")
_GOTO_TAIL = re.compile(r"\bgoto\s+([A-Za-z_]\w*)\s*;\s*$")
_DIVERGE = re.compile(
    r"^(?:return\b|break\b|continue\b|panic!|unreachable!|UNREACHABLE!|"
    r"(?:core::|::core::)?hint::unreachable_unchecked|(?:core::|::core::|std::)?process::abort)"
)


def _strip_c(body: str) -> str:
    return re.sub(
        r"/\*.*?\*/|//[^\n]*|\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^'\\])'", " ", body, flags=re.S
    )


def c_goto_arms(body_text: str) -> tuple[tuple[str, str], ...]:
    """`(case, label)` for each `case` arm ending in an unconditional `goto label;` that does not
    jump to the statement right after its `switch`."""
    code = _strip_c(body_text)
    found: list[tuple[str, str]] = []
    for m in _SWITCH.finditer(code):
        open_at = code.find("{", m.end())
        if open_at < 0:
            continue
        close = _match_brace(code, open_at)
        if close < 0:
            continue
        after = code[close + 1 :].lstrip()
        body = code[open_at + 1 : close]
        # Arm boundaries: the switch's own case labels, at brace depth 0 of its body.
        marks: list[tuple[int, int, str | None]] = []
        depth = 0
        k = 0
        while k < len(body):
            c = body[k]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
            elif depth == 0:
                if body.startswith("switch", k) and (k == 0 or not body[k - 1].isalnum()):
                    # a nested switch with no braced arm: skip its body
                    o = body.find("{", k)
                    e = _match_brace(body, o) if o >= 0 else -1
                    if e > 0:
                        k = e + 1
                        continue
                cm = _CASE.match(body, k)
                if cm and (k == 0 or not (body[k - 1].isalnum() or body[k - 1] == "_")):
                    marks.append((k, cm.end(), cm.group(1)))
                    k = cm.end()
                    continue
            k += 1
        for i, (_, end, case) in enumerate(marks):
            if case is None:
                continue
            stop = marks[i + 1][0] if i + 1 < len(marks) else len(body)
            arm = body[end:stop].strip()
            # `case A: { ...; goto L; }` -- peel braces that wrap the whole arm.
            while arm.startswith("{") and _match_brace(arm, 0) == len(arm) - 1:
                arm = arm[1:-1].strip()
            g = _GOTO_TAIL.search(arm)
            if not g:
                continue
            # The `goto` must be the arm's own last statement, not an `if (x) goto L;`.
            before = arm[: g.start()].rstrip()
            if before.endswith(")") or re.search(r"\belse\s*$", before):
                continue
            if re.match(rf"{re.escape(g.group(1))}\s*:", after):
                continue
            found.append((case, g.group(1)))
    return tuple(found)


def _arm_block(s: str, lo: int, hi: int, case: str) -> list[tuple[int, int]]:
    """Body spans (`{`..`}`) of every Rust match arm between lo and hi whose pattern names case."""
    spans = []
    for m in re.finditer(rf"(?m)^[ \t]*([^\n=]*\b{re.escape(case)}\b[^\n=]*?)=>\s*\{{", s[lo:hi]):
        pat = m.group(1)
        if pat.lstrip().startswith(("if ", "let ", "//")) or "(" in pat.split("|")[0][:1]:
            continue
        open_at = lo + m.end() - 1
        close = _match_brace(s, open_at)
        if 0 < close <= hi:
            spans.append((open_at, close))
    return spans


def _diverges(block: str) -> bool:
    """Whether a `{ .. }` body's last statement leaves it. A (labelled) block recurses -- atapi
    keeps the jump as `'CompleteIrp: { ..; return Status; }` -- and an `if`/`else` chain
    diverges when every branch does."""
    stmts: list[str] = []
    for st in _top_statements(block):
        # `_top_statements` ends a statement at every top-level `}`, so `else` starts a new one.
        if stmts and re.match(r"else\b", st):
            stmts[-1] += " " + st
        else:
            stmts.append(st)
    if not stmts:
        return False
    last = stmts[-1].lstrip()
    if _DIVERGE.match(last):
        return True
    last = re.sub(r"^'\w+\s*:\s*", "", last)
    if last.startswith(("{", "unsafe {", "unsafe{")):
        o = last.find("{")
        return _match_brace(last, o) == len(last) - 1 and _diverges(last[o + 1 : -1])
    if last.startswith("if "):
        branches, k, tail_else = [], 0, False
        while (o := last.find("{", k)) >= 0:
            e = _match_brace(last, o)
            if e < 0:
                return False
            tail_else = re.search(r"\belse\s*$", last[k:o]) is not None
            branches.append(last[o + 1 : e])
            k = e + 1
        # Without a final bare `else`, the fall-through path is the one that does not diverge.
        return tail_else and all(_diverges(b) for b in branches)
    return False


def _sets_jump_flag(block: str, rest: str, label: str) -> bool:
    """The other faithful lowering of `goto L;`: the arm's last statement raises a flag and the
    code after the `match` runs L's block under `if flag { .. }`. servman's MainWndProc ends
    WM_COMMAND with `goto_default = true;` for `goto HandleDefaultMessage;`, and the
    DefWindowProc call sits in `if goto_default { .. }` after the match. Only a flag whose
    NAME says it is that jump (`goto` in it, or the label itself), and which the rest of the
    function actually tests -- an arbitrary `done = true;` proves nothing."""
    stmts = _top_statements(block)
    if not stmts:
        return False
    m = re.fullmatch(r"\s*([A-Za-z_]\w*)\s*=\s*true\s*;?\s*", stmts[-1])
    if not m:
        return False
    flag = m.group(1)
    if "goto" not in flag.lower() and label.lower() not in flag.lower():
        return False
    return re.search(rf"\bif\s+{re.escape(flag)}\s*\{{", rest) is not None


def scan(rust: str, decls: Mapping[str, _Decls]) -> list[Fallthrough]:
    s = _strip(rust)
    found: list[Fallthrough] = []
    # Spans from the raw text: `_strip` blanks the `"C"` of `extern "C"`. Offsets agree.
    for name, lo, hi in _fn_bodies(rust):
        d = decls.get(name)
        if d is None or not d.goto_arms:
            continue
        for case, label in d.goto_arms:
            for open_at, close in _arm_block(s, lo, hi, case):
                if _diverges(s[open_at + 1 : close]):
                    continue
                if _sets_jump_flag(s[open_at + 1 : close], s[close:hi], label):
                    continue
                found.append(Fallthrough(name, case, label, s.count("\n", 0, open_at) + 1))
    return found
