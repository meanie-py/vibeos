"""Spans of C source that the BUILD compiles but never executes.

WHY this exists: fact/call sealing demands the model reproduce every sealed token
exactly once, and rejects the translation otherwise. That is the right contract for
live code -- it is what makes a dropped constant or a dropped call fail loudly instead
of silently (field manual, the SEAL entry). But it is the WRONG contract for code the
build never runs: the model correctly omits dead code, and the envelope then rejects a
correct translation and bills for a retry that will be rejected the same way.

Two flavours, both measured on the Tier 3 batch A run (2026-07-25), which lost 10 of 56
functions to exactly this:

1. **Preprocessor-disabled arms.** snmpapi's `DllMain` guards `case DLL_WINE_PREATTACH:
   return FALSE;` behind `#ifndef __REACTOS__`. Both values were sealed and demanded;
   both are unreachable in every vibeos build.

2. **PSEH except-blocks under DUMMY_PSEH.** All 9 wlanapi RPC clients wrap the stub call
   in `RpcTryExcept { ... } RpcExcept(EXCEPTION_EXECUTE_HANDLER) { ... } RpcEndExcept`.
   `RpcExcept` is `_SEH2_EXCEPT`, which under `-D_USE_DUMMY_PSEH=1` (present in this
   tree's real compile flags for every such module) expands to
   `if (_SEH2_Volatile0 || (0 && (...)))`, and `sdk/lib/pseh/dummy.c` defines
   `int _SEH2_Volatile0 = 0;` and never assigns it. The handler body is compiled and
   never entered -- so the C itself does not catch the RPC exception today, and a Rust
   translation that omits the handler matches the shipped C exactly.

The result is advisory, not a licence to drop things: a token in a dead span becomes
OPTIONAL (permitted, not demanded). The "invented token" check is untouched, so the
model still cannot fabricate a value. Omissions are surfaced to the caller rather than
swallowed, because a silently-relaxed guarantee is the failure mode this whole
mechanism exists to prevent.

Deliberately NOT done here: changing the prompt contract to stop asking for these
tokens. The contract text is hashed into the response-cache key, so rewording it
invalidates every banked sealed translation and re-charges for work already paid for.
Relaxing the audit alone leaves the cache intact.
"""

from __future__ import annotations

import re

# `#if` family openers. `#else`/`#elif` do not open a region -- the whole conditional,
# every arm of it, is treated as one dead span, because deciding WHICH arm is live needs
# the real macro state and this module deliberately stays lexical (see module docstring).
_IF_RE = re.compile(r"^[ \t]*#[ \t]*(if|ifdef|ifndef)\b", re.MULTILINE)
_ENDIF_RE = re.compile(r"^[ \t]*#[ \t]*endif\b", re.MULTILINE)

# The except-clause openers that DUMMY_PSEH compiles to an always-false `if`.
# `RpcExcept` is psdk/rpc.h's alias for `_SEH2_EXCEPT`; `__except` is pseh2.h's.
_EXCEPT_RE = re.compile(r"\b(?:RpcExcept|_SEH2_EXCEPT|__except)\b[ \t]*(?=\()")


def _line_span(text: str, pos: int) -> tuple[int, int]:
    start = text.rfind("\n", 0, pos) + 1
    end = text.find("\n", pos)
    return start, len(text) if end < 0 else end


def preprocessor_conditional_spans(text: str) -> list[tuple[int, int]]:
    """Spans covering every `#if`/`#ifdef`/`#ifndef` ... `#endif` region.

    Nested conditionals collapse into the outermost span. An unterminated `#if`
    (truncated function body) runs to end-of-text rather than being dropped, so a
    token in it stays optional instead of becoming spuriously required.
    """
    events = [(m.start(), 1) for m in _IF_RE.finditer(text)]
    events += [(m.start(), -1) for m in _ENDIF_RE.finditer(text)]
    events.sort()

    spans: list[tuple[int, int]] = []
    depth = 0
    opened_at = 0
    for pos, delta in events:
        if delta == 1:
            if depth == 0:
                opened_at = _line_span(text, pos)[0]
            depth += 1
        else:
            if depth == 0:
                continue  # stray #endif; nothing open
            depth -= 1
            if depth == 0:
                spans.append((opened_at, _line_span(text, pos)[1]))
    if depth > 0:
        spans.append((opened_at, len(text)))
    return spans


def _matching_brace(text: str, opening: int) -> int | None:
    depth = 0
    for pos in range(opening, len(text)):
        char = text[pos]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return pos + 1
    return None


def dummy_pseh_except_spans(text: str, masked: str | None = None) -> list[tuple[int, int]]:
    """Spans covering each `RpcExcept(...) { ... }` / `__except (...) { ... }` body.

    `masked` is the comment/literal-blanked view used to locate the openers, so an
    `__except` mentioned in a comment does not open a span. Offsets are shared with
    `text`, which is what the sealer indexes into.
    """
    source = masked if masked is not None else text
    spans: list[tuple[int, int]] = []
    for match in _EXCEPT_RE.finditer(source):
        # Skip the parenthesised filter expression, then take the block after it.
        paren = source.find("(", match.end() - 1)
        if paren < 0:
            continue
        depth = 0
        close = None
        for pos in range(paren, len(source)):
            if source[pos] == "(":
                depth += 1
            elif source[pos] == ")":
                depth -= 1
                if depth == 0:
                    close = pos
                    break
        if close is None:
            continue
        brace = source.find("{", close)
        if brace < 0 or source[close + 1 : brace].strip():
            continue  # not a braced block (e.g. `__except(...) statement;`)
        end = _matching_brace(source, brace)
        if end is None:
            continue
        spans.append((match.start(), end))
    return spans


def dead_region_spans(text: str, masked: str | None = None) -> list[tuple[int, int]]:
    """Every span of `text` the build compiles but does not execute."""
    spans = preprocessor_conditional_spans(text) + dummy_pseh_except_spans(text, masked)
    return sorted(spans)


def in_dead_region(spans: list[tuple[int, int]], start: int, end: int) -> bool:
    """True when [start, end) lies wholly inside one of `spans`."""
    return any(a <= start and end <= b for a, b in spans)
