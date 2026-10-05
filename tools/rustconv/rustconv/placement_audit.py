"""Catch sealed constants that came back in each other's places.

The fact seal (fact_seal.py) proves the translated body holds the C's constants as a
closed SET. A set is satisfied by a permutation: `a = 10; b = 20;` with the two tokens
swapped restores to `a = 20; b = 10;` and passes. msvcrt's `MSVCRT_exp` shipped
`math_error(0, ...)` where the C says `math_error(_DOMAIN, ...)` that way, and the
argument-fact check (argument_facts.py) only covers a sealed call with one call site.

This check works on the RESTORED body against the C, by spelling, so it needs no
placeholder history and can audit bodies already in the tree. Each constant occurrence
gets a context: the identifiers of its statement (split at `;`, `{`, `}`), minus
keywords and minus the constants themselves. Translation keeps names (fields, locals,
callees), so a correctly placed constant shares at least one of them with SOME C
occurrence of the same spelling.

A Rust occurrence of `A` is MISPLACED when it shares nothing with any C context of
`A` but does share something with a C context of a different constant `B`. One such
occurrence is weak evidence (Rust hoists temporaries, splits `if` from its body). It
becomes a finding only when the misplacements form a cycle: `A` sits where the C puts
`B` AND `B` sits where the C puts `A` (or a longer ring). A wrong checker bills every
retry and can never pass, so the rule is built to stay silent rather than guess:
an empty context, or a C spelling the Rust does not use, decides nothing.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from .fact_seal import seal_facts

_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_RUST_INT = re.compile(
    r"(?<![A-Za-z0-9_.])(0[xX][0-9A-Fa-f_]+|0[oO][0-7_]+|0[bB][01_]+|[0-9][0-9_]*)"
    r"(?:_?(?:[iu](?:8|16|32|64|128|size)))?(?![A-Za-z0-9_.])"
)
_C_INT = re.compile(r"^(0[xX][0-9A-Fa-f]+|0[oO]?[0-7]+|[0-9]+)[uUlL]*$")
_RS_COMMENT_OR_STRING = re.compile(
    r"//[^\n]*|/\*.*?\*/|b?\"(?:\\.|[^\"\\])*\"|b?'(?:\\.|[^'\\\n])'", re.S
)

_KEYWORDS = frozenset(
    [
        "if",
        "else",
        "while",
        "for",
        "do",
        "switch",
        "case",
        "default",
        "break",
        "continue",
        "return",
        "goto",
        "sizeof",
        "struct",
        "union",
        "enum",
        "typedef",
        "const",
        "volatile",
        "static",
        "extern",
        "register",
        "auto",
        "signed",
        "unsigned",
        "void",
        "char",
        "short",
        "int",
        "long",
        "float",
        "double",
        "let",
        "mut",
        "fn",
        "match",
        "loop",
        "as",
        "in",
        "unsafe",
        "pub",
        "use",
        "mod",
        "impl",
        "self",
        "Self",
        "true",
        "false",
        "ref",
        "move",
        "where",
        "type",
        "crate",
        "super",
        "dyn",
    ]
)


def _int_value(spelling: str) -> int | None:
    m = _C_INT.match(spelling)
    if not m:
        return None
    s = m.group(1)
    if s[:2] in ("0x", "0X"):
        return int(s, 16)
    if s[:2] in ("0o", "0O"):
        return int(s[2:], 8)
    if len(s) > 1 and s[0] == "0":
        return int(s, 8)
    return int(s)


def _rust_int_value(lit: str) -> int:
    s = lit.replace("_", "")
    if s[:2] in ("0x", "0X"):
        return int(s, 16)
    if s[:2] in ("0o", "0O"):
        return int(s[2:], 8)
    if s[:2] in ("0b", "0B"):
        return int(s[2:], 2)
    return int(s)


# Constants Rust SYNTAX introduces where the C has none. `if (x & F)` becomes
# `(x & F) != 0`, `i++` becomes `i += 1`, `p++` becomes `p.add(1)`, an
# uninitialised local becomes `let x = 0`. They are not the C's constants, so they
# cannot be misplaced ones; counting them made `0 <-> 1` "rings" out of every loop.
_RUST_SYNTAX_ZERO = re.compile(r"(?:[!=]=|\blet\s+(?:mut\s+)?\w+\s*(?::[^=;]+)?=)\s*\(?\s*$")
_RUST_SYNTAX_ONE = re.compile(
    r"(?:[+-]=\s*|\.(?:add|sub|offset|wrapping_add|wrapping_sub|checked_add|checked_sub|"
    r"overflowing_add|saturating_add|saturating_sub)\(\s*)$"
)
# `[u8; 4]` — an array LENGTH in a type, not a value placed in a statement.
_ARRAY_LEN = re.compile(r";\s*$")
# A symbol in TYPE position: `size_of::<X>`, `*mut X`, `x: X`, `as X`, `Vec<X>`.
_TYPE_BEFORE = re.compile(r"(?:::<|<|\*mut\s+|\*const\s+|:\s*|\bas\s+)$")


def _rust_syntax(rust: str, start: int, end: int, value: int | None) -> bool:
    before = rust[max(0, start - 80) : start]
    line = before.rsplit("\n", 1)[-1]
    if value == 0 and _RUST_SYNTAX_ZERO.search(line):
        return True
    if value == 1 and _RUST_SYNTAX_ONE.search(line):
        return True
    if (
        value is not None
        and _ARRAY_LEN.search(line)
        and rust[end : end + 8].lstrip().startswith("]")
    ):
        return True
    return bool(
        value is None
        and (_TYPE_BEFORE.search(line) or rust[end : end + 4].lstrip().startswith(">"))
    )


def _blank(m: re.Match[str]) -> str:
    return re.sub(r"[^\n]", " ", m.group(0))


def _statements(text: str) -> list[tuple[int, int]]:
    spans, start = [], 0
    for i, ch in enumerate(text):
        if ch in ";{}":
            spans.append((start, i))
            start = i + 1
    spans.append((start, len(text)))
    return spans


_CMP = re.compile(r"(?:==|!=|<=|>=|<|>)$")
_IDENT_TAIL = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*(?:\[[^\]]*\]\s*)*$")
_AS_CAST = re.compile(r"\s+as\s+[A-Za-z_][A-Za-z0-9_:<>*\s]*$")
_LET_TYPE = re.compile(r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:[^=]*$")


def _norm(name: str) -> str:
    """`PxePfn`, `pxe_pfn` and `__PxePfn` name one thing across a translation."""
    return name.replace("_", "").lower()


def _operand(expr: str) -> str | None:
    """The identifier an operator's left side ends in, past casts and indexing."""
    expr = _AS_CAST.sub("", expr.rstrip().rstrip("(").rstrip())
    t = _IDENT_TAIL.search(expr)
    if t and t.group(1) not in _KEYWORDS and not t.group(1).startswith("__RUSTCONV_"):
        return _norm(t.group(1))
    return None


def _first_ident(text: str) -> str:
    for w in _IDENT.findall(text):
        if w not in _KEYWORDS and not w.startswith("__RUSTCONV_"):
            return _norm(w)
    return ""


def _guard(text: str, stmt_start: int, head: str) -> str:
    """The condition guarding a `return`: in the same statement (C's `if (c) return`)
    or the `if c {` block it opens (either language)."""
    m = re.search(r"\bif\b(.*)$", head)
    if m:
        return _first_ident(m.group(1))
    prev = text[max(0, stmt_start - 200) : stmt_start]
    m = re.search(r"\bif\b([^;{}]*)\{?\s*$", prev)
    return _first_ident(m.group(1)) if m else ""


def _slot(text: str, at: int, end: int) -> tuple[str, ...] | None:
    """Where a constant sits. None when there is nothing to compare by."""
    depth, commas, i = 0, 0, at - 1
    while i >= 0:
        ch = text[i]
        if ch in ";{}":
            break
        if ch in ")]":
            depth += 1
        elif ch == "[":
            if depth == 0:
                return None  # an index or an array literal
            depth -= 1
        elif ch == "(":
            if depth == 0:
                m = _IDENT_TAIL.search(text[max(0, i - 80) : i].rstrip().rstrip("!"))
                if m and m.group(1) not in _KEYWORDS:
                    return ("arg", _norm(m.group(1)).lstrip("_"), str(commas))
                commas = 0  # a grouping paren: keep looking outward
            else:
                depth -= 1
        elif ch == "," and depth == 0:
            commas += 1
        i -= 1
    stmt_start = i + 1
    head = text[stmt_start:at]
    after = text[end : end + 120]
    # `case X:` (C) and `X =>` / `X | Y =>` (Rust): keyed by what the arm does first.
    if re.search(r"\bcase\s*$", head):
        m = re.match(r"\s*:", after)
        if m:
            return ("case", _first_ident(re.sub(r"\bcase\b[^:]*:", " ", after[m.end() :])))
    m = re.match(r"\s*(?:\|\s*[A-Za-z0-9_:]+\s*)*=>", after)
    if m:
        return ("case", _first_ident(after[m.end() :]))
    if re.search(r"\breturn\s*$", head):
        return ("ret", _guard(text, stmt_start, head[: head.rfind("return")]))
    # assignment target: the last `=` that is not part of a comparison or `=>`
    for m in reversed(list(re.finditer(r"(?<![=!<>+\-*/%&|^])([+\-*/%&|^]?)=(?![=>])", head))):
        lhs = head[: m.start()].rstrip()
        let = _LET_TYPE.search(lhs)
        name = _norm(let.group(1)) if let else _operand(lhs)
        if name:
            return ("set", name)
        break
    before = head.rstrip()
    for op in ("==", "!=", "<=", ">=", "<", ">", "&", "|", "^"):
        if before.endswith(op) and not before.endswith(op * 2):
            kind = "cmp" if op[0] in "=!<>" else "bit"
            name = _operand(before[: -len(op)])
            return (kind, name) if name else None
    m = re.match(r"\s*(==|!=|<=|>=|<|>)\s*([A-Za-z_][A-Za-z0-9_]*)", after)
    if m and m.group(2) not in _KEYWORDS:
        return ("cmp", _norm(m.group(2)))
    return None


@dataclass(frozen=True)
class Misplacement:
    spelling: str  # the constant as the C spells it
    took_place_of: str  # the constant whose C context the Rust occurrence matches
    rust_statement: str


@dataclass(frozen=True)
class PlacementFinding:
    ring: tuple[str, ...]  # constants that traded places, in cycle order
    evidence: tuple[Misplacement, ...]

    def message(self) -> str:
        ring = " -> ".join((*self.ring, self.ring[0]))
        lines = [f"sealed constants traded places: {ring}"]
        for m in self.evidence:
            lines.append(
                f"  {m.spelling} sits where the C has {m.took_place_of}: {m.rust_statement}"
            )
        return "\n".join(lines)


def placement_findings(c_body: str, rust_body: str) -> list[PlacementFinding]:
    """Permutations of the C's constants in `rust_body`; empty when none is evidenced."""
    sealed = seal_facts(c_body, dialect="c")
    if len(sealed.facts) < 2:
        return []
    # C side: the slots each spelling occupies, read off the SEALED text so a
    # neighbouring constant never stands in for a callee or an operand.
    c_text = sealed.text
    c_slots: dict[str, set[tuple[str, ...]]] = {}
    for fact in sealed.facts:
        if fact.dead:
            continue
        at = c_text.find(fact.placeholder)
        slot = _slot(c_text, at, at + len(fact.placeholder))
        c_slots.setdefault(fact.original, set())
        if slot:
            c_slots[fact.original].add(slot)
    owner: dict[tuple[str, ...], set[str]] = {}
    for spelling, slots in c_slots.items():
        for slot in slots:
            owner.setdefault(slot, set()).add(spelling)

    # Rust side: find each spelling. Integers match by VALUE (`0x10` == `16u32`).
    rust = _RS_COMMENT_OR_STRING.sub(_blank, rust_body)
    r_spans = _statements(rust)
    by_value: dict[int, list[str]] = {}
    for sp in c_slots:
        v = _int_value(sp)
        if v is not None:
            by_value.setdefault(v, []).append(sp)
    occurrences: list[tuple[str, int, int]] = []  # (spelling, start, end)
    for m in _RUST_INT.finditer(rust):
        try:
            v = _rust_int_value(m.group(1))
        except ValueError:
            continue
        names = by_value.get(v, [])
        if len(names) == 1 and not _rust_syntax(rust, m.start(), m.end(), v):
            occurrences.append((names[0], m.start(), m.end()))  # one C spelling per value
    for sp in c_slots:
        if _int_value(sp) is not None:
            continue
        for m in re.finditer(rf"\b{re.escape(sp)}\b", rust):
            if not _rust_syntax(rust, m.start(), m.end(), None):
                occurrences.append((sp, m.start(), m.end()))

    edges: dict[str, dict[str, Misplacement]] = {}
    for spelling, start, end in occurrences:
        slot = _slot(rust, start, end)
        if slot is None or spelling in owner.get(slot, ()):
            continue
        rivals = {r for r in owner.get(slot, ()) if not _same_value(r, spelling)}
        if not rivals:
            continue  # the C puts nothing here: decide nothing
        stmt, _ = _statement_text(rust_body, r_spans, start)
        for rival in sorted(rivals):
            edges.setdefault(spelling, {}).setdefault(rival, Misplacement(spelling, rival, stmt))
    return _rings(edges)


def _same_value(a: str, b: str) -> bool:
    va, vb = _int_value(a), _int_value(b)
    return va is not None and va == vb


def _statement_text(body: str, spans: list[tuple[int, int]], at: int) -> tuple[str, str]:
    for a, b in spans:
        if a <= at <= b:
            return " ".join(body[a:b].split())[:160], ""
    return "", ""


def _rings(edges: dict[str, dict[str, Misplacement]]) -> list[PlacementFinding]:
    """Cycles in the misplacement graph (strongly connected components of size >= 2)."""
    index: dict[str, int] = {}
    low: dict[str, int] = {}
    stack: list[str] = []
    on: set[str] = set()
    comps: list[list[str]] = []
    counter = [0]

    def visit(v: str) -> None:
        index[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        on.add(v)
        for w in edges.get(v, {}):
            if w not in index:
                visit(w)
                low[v] = min(low[v], low[w])
            elif w in on:
                low[v] = min(low[v], index[w])
        if low[v] == index[v]:
            comp = []
            while True:
                w = stack.pop()
                on.discard(w)
                comp.append(w)
                if w == v:
                    break
            if len(comp) > 1:
                comps.append(comp)

    for v in sorted(edges):
        if v not in index:
            visit(v)
    out = []
    for comp in comps:
        members = set(comp)
        ring = tuple(sorted(comp))
        evidence = tuple(
            m for s in ring for t, m in sorted(edges.get(s, {}).items()) if t in members
        )
        out.append(PlacementFinding(ring, evidence))
    return out
