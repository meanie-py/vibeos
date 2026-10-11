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
from pathlib import Path

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


# Macros whose truth value is FIXED for every build in this tree, so a branch guarded on
# one of them is not "conditional" -- it is dead code, or live code, and we can say which.
# `__REACTOS__` is defined by the root CMakeLists for every TU, which makes the
# wine-derived `#ifndef __REACTOS__` idiom unambiguously dead.
KNOWN_DEFINED = frozenset({"__REACTOS__"})

_COND_OPEN_RE = re.compile(r"^[ \t]*#[ \t]*(ifdef|ifndef|if)\b(.*)$")
_COND_ANY_RE = re.compile(r"^[ \t]*#[ \t]*(ifdef|ifndef|if|elif|else|endif)\b(.*)$")

# One `MACRO <op> <int>` comparison. The literal may be decimal or hex, with an
# optional L/U suffix, because a version macro is routinely spelled `0x0601` or `100L`.
_INT_CMP_RE = re.compile(
    r"^\(*\s*([A-Za-z_]\w*)\s*(==|!=|<=|>=|<|>)\s*(0[xX][0-9a-fA-F]+|\d+)[uUlL]*\s*\)*$"
)


def pinned_int_macros(definitions: object) -> dict[str, int]:
    """Macros a module pins to an integer literal via `-DNAME=<int>`.

    `definitions` is the module's raw `-D` list. A `-D_MSVCR_VER=0` is every bit as
    decidable as a `#undef` -- it is on the compile command for every TU in the target --
    but for a long time nothing read the VALUE, only the name.
    """
    pinned: dict[str, int] = {}
    for d in definitions:  # type: ignore[attr-defined]
        if not d.startswith("-D") or "=" not in d:
            continue
        name, _, value = d[2:].partition("=")
        value = value.strip().rstrip("uUlL")
        try:
            pinned[name.strip()] = int(value, 0)
        except ValueError:
            continue
    return pinned


def eval_int_guard(expr: str, pinned: dict[str, int]) -> bool | None:
    """Evaluate a `#if` over pinned integer macros. True/False, or None when undecidable.

    Deliberately narrow: a chain of `&&` OR a chain of `||`, never a mix, because getting
    C's precedence subtly wrong here would mark LIVE code dead and send someone to delete
    a body the compiler is really building. A term is either `MACRO <op> <int>` against a
    pinned macro or a `defined(X)` we cannot settle; anything else is undecidable.

    Short-circuit still resolves the mixed-certainty cases that matter: one decidably-false
    conjunct kills the block however unknown its neighbours are. That is what settles
    handler4.c's `defined(__x86_64__) && _MSVCR_VER>=140` on all three arches -- we never
    learn whether __x86_64__ is defined, and we do not need to.
    """
    if "&&" in expr and "||" in expr:
        return None
    joiner = "&&" if "&&" in expr else ("||" if "||" in expr else None)
    terms = expr.split(joiner) if joiner else [expr]

    verdicts: list[bool | None] = []
    for term in terms:
        term = term.strip()
        m = _INT_CMP_RE.match(term)
        if m is not None and m.group(1) in pinned:
            lhs, op, rhs = pinned[m.group(1)], m.group(2), int(m.group(3), 0)
            verdicts.append(
                {
                    "==": lhs == rhs,
                    "!=": lhs != rhs,
                    "<=": lhs <= rhs,
                    ">=": lhs >= rhs,
                    "<": lhs < rhs,
                    ">": lhs > rhs,
                }[op]
            )
            continue
        # A `defined(X)` the tree fixes (KNOWN_DEFINED: every compile has -D__REACTOS__).
        # calc's winmain.c: `#if defined(__GNUC__) && !defined(__REACTOS__)` is false however
        # __GNUC__ stands, and its arm holds a second header for the `_tWinMain` body.
        d = re.fullmatch(r"(!)?\s*defined\s*\(?\s*(\w+)\s*\)?", term)
        if d is not None and d.group(2) in KNOWN_DEFINED:
            verdicts.append(d.group(1) is None)
            continue
        # A bare `DBG` / `!DBG` is C's truth test on the macro's value.
        b = re.fullmatch(r"(!)?\s*\(?\s*([A-Za-z_]\w*)\s*\)?", term)
        if b is not None and b.group(2) in pinned:
            verdicts.append((pinned[b.group(2)] != 0) != bool(b.group(1)))
            continue
        # A `defined(X)` term we cannot settle, or anything else: unknown.
        verdicts.append(None)

    if joiner == "&&" or joiner is None:
        if any(v is False for v in verdicts):
            return False
        return True if all(v is True for v in verdicts) else None
    if any(v is True for v in verdicts):
        return True
    return False if all(v is False for v in verdicts) else None


def cond_truth(
    line: str,
    pinned: dict[str, int] | None = None,
    local: frozenset[str] = frozenset(),
    off: frozenset[str] = frozenset(),
) -> bool | None:
    """``True``/``False`` when a preprocessor condition is FIXED for this tree, else ``None``.

    Deliberately narrow. Everything it cannot settle is ``None``, and every caller treats
    ``None`` as live -- over-reporting live code is the safe direction, mistaking live code
    for dead is not.

    `pinned` carries the module's `-DNAME=<int>` (see :func:`pinned_int_macros`). Without it
    only the tree-wide :data:`KNOWN_DEFINED` names decide anything, which is the correct
    default for a caller that does not know which CMake target owns the file.
    """
    m = _COND_OPEN_RE.match(line)
    if m is None:
        return None
    # A trailing comment is not part of the condition: `#if 0 // TODO` is still `#if 0`.
    rest = re.sub(r"/\*.*?\*/", " ", m.group(2)).split("//")[0].strip()
    kind = m.group(1)
    if kind in ("ifdef", "ifndef"):
        sym = rest.split()[0] if rest.split() else ""
        if sym in KNOWN_DEFINED or sym in local or (pinned and sym in pinned):
            return kind == "ifdef"
        if sym in off:
            return kind == "ifndef"
        return None
    if re.fullmatch(r"\(?\s*0+\s*\)?", rest):
        return False
    if re.fullmatch(r"\(?\s*[1-9]\d*\s*\)?", rest):
        return True
    m2 = re.fullmatch(r"(!)?\s*defined\s*\(?\s*(\w+)\s*\)?", rest)
    if m2 and (
        m2.group(2) in KNOWN_DEFINED or m2.group(2) in local or (pinned and m2.group(2) in pinned)
    ):
        return m2.group(1) is None
    if m2 and m2.group(2) in off:
        return m2.group(1) is not None
    return eval_int_guard(rest, pinned or {})


_LOCAL_DEFINE_RE = re.compile(r"^[ \t]*#[ \t]*(define|undef)[ \t]+([A-Za-z_]\w*)\b(?!\()")


def never_compiled_lines(
    text: str, pinned: dict[str, int] | None = None, off: frozenset[str] = frozenset()
) -> set[int]:
    """The 1-based line numbers this tree's compiler never sees.

    A function whose definition starts on one of these lines does not exist in any object
    we build, so translating it buys a body that nothing can ever call -- and one paid call
    per function is what that costs. `except_i386.c` alone carries seven: `_global_unwind2`,
    `_except_handler2/3`, `_local_unwind2`, `longjmp` and the two `__regs__setjmp*`, all
    inside `#ifndef __REACTOS__`, all confirmed absent from the object file.

    `pinned` extends that to the module's own `-DNAME=<int>`, and in msvcrt that is the
    bigger half: ~200 `#if _MSVCR_VER >= 80/100/110/120/140` directives across the tree are
    all decidably FALSE at the `-D_MSVCR_VER=0` the CMakeLists pins, and every body inside
    one has been on the bill.

    Only FIXED conditions (:func:`cond_truth`) mark anything dead. An `#if` this module
    cannot settle leaves its whole subtree live, and a dead outer arm keeps its inner
    conditionals dead regardless of what they say -- which is the one asymmetry worth
    stating, since a live-looking `#ifdef __REACTOS__` nested inside a dead
    `#ifndef __REACTOS__` is still dead.
    """
    dead: set[int] = set()
    # stack frames: (this arm is dead, the condition was decidable, some arm already taken)
    stack: list[tuple[bool, bool, bool]] = []
    # Names this FILE defines on a line every build compiles: `#define NDEBUG` above an
    # `#ifndef NDEBUG` debug dumper (setuplib osdetect.c's DumpNTOSInstalls) makes that body
    # dead, and c2rust -- which sees the preprocessed TU -- rightly has no anchor for it.
    local: set[str] = set()
    for n, line in enumerate(text.splitlines(), 1):
        d = _LOCAL_DEFINE_RE.match(line)
        if d and all(known and not dead_arm for dead_arm, known, _ in stack):
            if d.group(1) == "define":
                local.add(d.group(2))
            else:
                local.discard(d.group(2))
        m = _COND_ANY_RE.match(line)
        if m is None:
            if any(d for d, _, _ in stack):
                dead.add(n)
            continue
        directive = m.group(1)
        if directive in ("if", "ifdef", "ifndef"):
            truth = cond_truth(line, pinned, frozenset(local), off - local)
            stack.append((truth is False, truth is not None, truth is True))
        elif directive in ("else", "elif") and stack:
            _, known, taken = stack[-1]
            if directive == "else" and known:
                stack[-1] = (taken, True, True)
            else:
                # An `#elif` on a decidable chain is only reachable if no arm was taken,
                # and we cannot decide the `#elif` itself -- so call it live.
                stack[-1] = (False, False, taken)
        elif directive == "endif" and stack:
            stack.pop()
    return dead


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


# `#define NAME <int>` — the pinned-macro shape that lives in a HEADER rather than on the
# compile command. Same decidability as a `-DNAME=<int>`, different hiding place.
_HEADER_DEFINE_RE = re.compile(
    r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_]\w*)[ \t]+"
    r"\(?\s*(0[xX][0-9a-fA-F]+|\d+)\s*\)?[uUlL]*[ \t]*$",
    re.MULTILINE,
)


def sibling_header_int_macros(source: Path | str) -> dict[str, int]:
    """`#define NAME <int>` from the headers sitting beside `source`.

    `pinned_int_macros` reads the module's `-D` list, which is the right place for
    `_MSVCR_VER` and misses the equally-decidable case of a constant defined in the
    module's own header. shellmenu.h's `#define MERGE_FOLDERS 1` is the one that cost us:
    `GetMergedFolder` is written as `#if MERGE_FOLDERS ... #else ... #endif`, and with the
    macro unreadable the whole conditional was treated as one dead span -- which made the
    live arm's tokens OPTIONAL and let a body that translated the `#else` pass the seal.

    Deliberately shallow: the headers in the same directory, no include graph, no
    transitive closure. A macro this cannot see simply stays undecided, which is the
    status quo -- so a wrong answer here is the only real failure mode, and a
    directory-local `#define NAME <literal>` is about as unambiguous as C gets. A name
    defined twice with different values is dropped rather than guessed at.
    """
    path = Path(source)
    folder = path.parent
    if not folder.is_dir():
        return {}
    found: dict[str, int] = {}
    conflicting: set[str] = set()
    for header in sorted(folder.glob("*.h")):
        try:
            text = header.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for match in _HEADER_DEFINE_RE.finditer(text):
            name, literal = match.group(1), match.group(2)
            value = int(literal, 16) if literal[:2].lower() == "0x" else int(literal)
            if name in found and found[name] != value:
                conflicting.add(name)
            found[name] = value
    for name in conflicting:
        found.pop(name, None)
    return found


def conditional_arm_notes(text: str, pinned: dict[str, int] | None = None) -> list[str]:
    """Notes about preprocessor conditionals in a C function body, for the audit.

    A translated body is one arm of each conditional it spans, and NOTHING downstream
    checks it picked the right one: the seals treat the whole conditional as dead (so the
    live arm's tokens are merely optional), and the result compiles either way. The one
    time this bit us the wrong arm was chosen with the macro in plain sight, and the only
    thing that noticed was a rustc `unreachable statement` warning at link time, several
    steps after the money was spent.

    Report-only by design, and the strings say which arm is live when this module can tell
    -- a finding a reviewer can settle by reading two lines beats a rejection that bills a
    retry to re-learn the same thing.
    """

    def arm_truth(line: str) -> bool | None:
        """`cond_truth`, plus the bare `#if NAME` form.

        `eval_int_guard` only settles a `NAME <op> <int>` comparison, so `#if MERGE_FOLDERS`
        comes back undecided even with the macro in `pinned`. Handled HERE rather than in
        `cond_truth` on purpose: that one feeds `never_compiled_lines`, which decides what
        we refuse to translate at all, and widening it is a change to run behaviour rather
        than to a report.
        """
        settled = cond_truth(line, pinned)
        if settled is not None or not pinned:
            return settled
        m = _COND_OPEN_RE.match(line)
        if m is None or m.group(1) != "if":
            return None
        bare = m.group(2).split("//")[0].split("/*")[0].strip().strip("()").strip()
        return pinned[bare] != 0 if bare in pinned else None

    notes: list[str] = []
    dead = never_compiled_lines(text, pinned)
    for line in text.splitlines():
        m = _COND_OPEN_RE.match(line)
        if m is None:
            continue
        truth = arm_truth(line)
        directive = line.strip()
        if truth is None:
            notes.append(f"{directive} (undecidable here — check the body took the built arm)")
        else:
            arm = "the #if arm is LIVE" if truth else "the #else arm is LIVE"
            notes.append(f"{directive} — {arm}")
    if notes and dead:
        notes.append(f"{len(dead)} line(s) of this body are never compiled")
    return notes
