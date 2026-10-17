"""Bounded deterministic repair over a WHOLE-CRATE ``cargo check``.

Translating a C DLL/driver into a fresh ``<mod>-rust`` crate leaves a few hundred rustc errors
that cluster into a handful of *mechanical* roots.  Fixing those by hand (or by paying an LLM
per site) is waste: each one is decided by an authority that is already closed — the rustc
diagnostic's own ``expected``/``found`` types, a bindgen declaration, or Rust lexical structure.
This module is the reusable pass that closes exactly those roots and *nothing else*.

WHY it refuses to take rustc's own advice on numeric mismatches: rustc suggests
``.try_into().unwrap()``, which injects a PANIC where the original C truncated silently.  A
panic in a live service is a real bug and trips ``tools/scan_silent_failures.py``, so we emit a
faithful ``as <ty>`` instead.  Every suggestion in ``PANICKING_SUGGESTIONS`` is refused the same
way.

WHY the width gate is the most important thing here: an ``as u32`` inferred from an *i386* run
is WRONG on amd64/arm64 whenever the real parameter is a width-varying alias (``SIZE_T``,
``ULONG_PTR``, ``usize``).  A previous hand-rolled version of this pass baked i386 widths into
wkssvc allocator arguments.  So a cast whose target width equals the analysing target's POINTER
width is only applied when at least two targets independently propose the *identical* edit;
otherwise it is withheld and reported for a hand-written semantic alias.  Casts *to* ``usize``/
``isize`` are the alias and are always safe.

Structure (so tests never need a cargo run):
  * ``propose_edits`` is PURE — parsed rustc diagnostics + a file reader in, proposed edits out.
  * ``propose_flex_array_edits`` is PURE — bindgen declarations + source bytes in, edits out.
  * ``merge_plans`` decides cross-target agreement.
  * ``repair_crate`` is the only part that shells out, and it never leaves the tree worse: if a
    pass raises the error count it restores that pass's bytes and aborts loudly.

Every applied edit carries its rule id AND the authority sentence that justified it, so an
acceptance can be audited from the ledger without re-reading a compiler scratch tree.
"""

from __future__ import annotations

import functools
import json
import os
import re
import subprocess
from collections.abc import Callable, Collection, Mapping, Sequence
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any

from .c_integer_promotions import comparison_operands, promoted_type
from .config import RUST_TARGETS
from .fact_seal import rust_integer_spelling

# --- rules ------------------------------------------------------------------------------------

RULE_NUMERIC_CAST = "truncating_numeric_cast"
RULE_COMPARISON = "c_integer_comparison"
RULE_WIDTH_ALIAS = "width_alias_cast"
RULE_NO_STD_PATH = "no_std_path"
RULE_POINTER_CAST = "raw_pointer_cast"
RULE_FLEX_ARRAY = "flex_array_index"
RULE_PANICKING_CAST = "panicking_cast"
RULE_ANON_UNION = "anon_union_field"
RULE_C2RUST_HOP = "c2rust_anon_hop"
RULE_MATCH_SCRUTINEE = "match_scrutinee_cast"
RULE_MATCH_ARM_GUARD = "match_arm_c_conversion"
RULE_PACKED_FIELD_PTR = "packed_field_ptr"
RULE_CPP_BASE_FIELD = "cpp_base_field"
RULE_REFGUID_DEREF = "refguid_deref"
RULE_MACRO_PLACE_DEREF = "macro_place_deref"
RULE_INDEX_USIZE = "index_usize"
RULE_MACRO_CALL = "macro_call_bang"
RULE_SIZEOF_TYPE = "sizeof_type_name"
RULE_ARRAY_DECAY = "array_decay"
RULE_C_INT_SUFFIX = "c_integer_suffix"
RULE_NULL_FN_POINTER = "null_fn_pointer"
RULE_C_TRUTHINESS = "c_truthiness"
RULE_OPTION_FN_CALL = "option_fn_call"
RULE_UNINIT_LOCAL = "uninit_c_local"
RULE_MUT_BINDING = "c_mutable_binding"
RULE_INFER_WIDEN = "c_integer_widen"
RULE_OPTION_IS_NULL = "option_fn_is_null"
RULE_SHADOWED_CONST = "local_shadows_constant"
RULE_SIZEOF_CONST = "sizeof_constant"
RULE_FN_ITEM_CAST = "fn_item_cast"
RULE_VARIADIC_PROMOTION = "variadic_promotion"
RULE_LOCAL_STATIC_MUT = "c_local_static_mut"
RULE_POINTER_NOT_NOT = "pointer_double_negation"
RULE_ZEROED_DEFAULT = "zeroed_not_default"
RULE_ARROW_FIELD = "raw_pointer_field"
RULE_UNSIGNED_NEGATE = "unsigned_negative_literal"
RULE_OPTION_FN_AS_PTR = "option_fn_as_pointer"
RULE_SOME_FN_ITEM = "some_fn_item"
RULE_KEYWORD_FIELD = "bindgen_keyword_field"
RULE_LITERAL_MUT = "c_literal_to_mut_param"
RULE_BITFIELD = "bitfield_accessor"
RULE_CONST_OPERAND = "c_constant_operand"
RULE_C2RUST_ANON = "c2rust_anon_member"
RULE_CPP_REFERENCE_ARG = "cpp_reference_argument"
RULE_FLOAT_LITERAL = "c_float_literal"
RULE_OBJECT_DEREF = "object_deref"

#: Rules driven by rustc diagnostics. ``RULE_WIDTH_ALIAS`` is FIRST on purpose: it and
#: ``RULE_NUMERIC_CAST`` fire on the same E0308 int->int shape, and the alias is the better
#: answer wherever it is available, so it claims the span and the numeric rule stands down.
DIAGNOSTIC_RULES: tuple[str, ...] = (
    RULE_WIDTH_ALIAS,
    # Before the comparison and numeric-cast rules: a constant whose value fits needs no
    # promotion proof and no width vote.
    RULE_CONST_OPERAND,
    # Before the comparison and numeric-cast rules: a WIDENING has a portable answer
    # (`as _`), so it should not be withheld as a per-target width.
    RULE_INFER_WIDEN,
    RULE_COMPARISON,
    RULE_NUMERIC_CAST,
    RULE_MATCH_SCRUTINEE,
    RULE_MATCH_ARM_GUARD,
    RULE_NO_STD_PATH,
    # Before RULE_POINTER_CAST: both answer an E0308 whose EXPECTED side is a raw pointer, and
    # decay is the more specific shape (found side is an array, not a pointer). They cannot both
    # claim a span, but keeping the specific rule first makes that structural rather than lucky.
    RULE_ARRAY_DECAY,
    RULE_POINTER_CAST,
    RULE_ANON_UNION,
    RULE_PACKED_FIELD_PTR,
    RULE_CPP_BASE_FIELD,
    RULE_REFGUID_DEREF,
    RULE_MACRO_PLACE_DEREF,
    RULE_INDEX_USIZE,
    RULE_MACRO_CALL,
    RULE_SIZEOF_TYPE,
    RULE_C_INT_SUFFIX,
    RULE_NULL_FN_POINTER,
    RULE_C_TRUTHINESS,
    RULE_OPTION_FN_CALL,
    RULE_UNINIT_LOCAL,
    RULE_MUT_BINDING,
    RULE_OPTION_IS_NULL,
    RULE_SHADOWED_CONST,
    RULE_SIZEOF_CONST,
    RULE_FN_ITEM_CAST,
    RULE_VARIADIC_PROMOTION,
    RULE_LOCAL_STATIC_MUT,
    RULE_POINTER_NOT_NOT,
    RULE_ZEROED_DEFAULT,
    RULE_ARROW_FIELD,
    RULE_UNSIGNED_NEGATE,
    RULE_OPTION_FN_AS_PTR,
    RULE_SOME_FN_ITEM,
    RULE_KEYWORD_FIELD,
    RULE_C2RUST_HOP,
    RULE_LITERAL_MUT,
    RULE_BITFIELD,
    RULE_C2RUST_ANON,
    RULE_FLOAT_LITERAL,
    RULE_OBJECT_DEREF,
)
#: Rules driven by source + a generated declaration (rustc cannot see these: an out-of-bounds
#: index on a bindgen ``[T; 1]`` flexible-array member is a RUNTIME panic, and the
#: ``unconditional_panic`` lint that would catch a literal index does not even run under
#: ``cargo check`` (verified on rustc 1.92 — check emits metadata only).
SOURCE_RULES: tuple[str, ...] = (RULE_FLEX_ARRAY, RULE_PANICKING_CAST)
ALL_RULES: tuple[str, ...] = DIAGNOSTIC_RULES + SOURCE_RULES

#: rustc suggestions we refuse on sight: they convert a silent C truncation/failure into a
#: kernel/service panic.  Recorded here so the refusal is auditable, not folklore.
PANICKING_SUGGESTIONS: tuple[str, ...] = (
    ".try_into().unwrap()",
    ".unwrap()",
    ".expect(",
    "panic!",
    "unreachable!",
    "todo!",
    "unimplemented!",
)

#: Byte widths of Rust's fixed-width integer primitives.  ``usize``/``isize`` are deliberately
#: absent — they are the width-varying alias and are handled as always-safe cast targets.
INT_WIDTH: dict[str, int] = {
    "i8": 1,
    "u8": 1,
    "i16": 2,
    "u16": 2,
    "i32": 4,
    "u32": 4,
    "i64": 8,
    "u64": 8,
    "i128": 16,
    "u128": 16,
}
POINTER_SIZED: frozenset[str] = frozenset({"usize", "isize"})
INT_PRIMITIVES: frozenset[str] = frozenset(INT_WIDTH) | POINTER_SIZED

#: Pointer width per target.  Accepts both the cargo triple and our arch shorthand.
_POINTER_WIDTH: dict[str, int] = {
    "i386": 4,
    "i586": 4,
    "i686": 4,
    "amd64": 8,
    "x86_64": 8,
    "arm64": 8,
    "aarch64": 8,
}

#: ``std`` submodules that also exist in ``core`` — the closed authority for a std->core rewrite.
#: Anything outside this list (``collections``, ``vec``, ``string``, ``io``, ...) lives in
#: ``alloc``/``std`` only, so the rewrite would be a guess: those stay RED.
CORE_MODULES: frozenset[str] = frozenset(
    {
        "any",
        "arch",
        "array",
        "ascii",
        "borrow",
        "cell",
        "char",
        "clone",
        "cmp",
        "convert",
        "default",
        "f32",
        "f64",
        "ffi",
        "fmt",
        "future",
        "hash",
        "hint",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "iter",
        "marker",
        "mem",
        "num",
        "ops",
        "option",
        "panic",
        "pin",
        "primitive",
        "ptr",
        "result",
        "slice",
        "str",
        "sync",
        "task",
        "time",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
    }
)


class RepairError(Exception):
    """A repair could not prove its authority, or the environment is not what we require.

    Raised instead of degrading to a guess (AGENTS.md: fail loud, never silently degrade).
    """


# --- data -------------------------------------------------------------------------------------


@dataclass(frozen=True)
class Edit:
    """One byte-range substitution, with the authority that justifies it."""

    file_name: str
    byte_start: int
    byte_end: int
    replacement: str
    rule: str
    authority: str
    original: str = ""
    line: int = 0
    column: int = 0
    #: True when the edit bakes in a concrete width equal to the analysing target's pointer
    #: width, i.e. it is only trustworthy if another target agrees.
    requires_target_agreement: bool = False
    #: Targets that independently proposed this exact edit (filled in by ``merge_plans``).
    targets: tuple[str, ...] = ()

    @property
    def key(self) -> tuple[str, int, int, str]:
        return (self.file_name, self.byte_start, self.byte_end, self.rule)

    def to_dict(self) -> dict[str, Any]:
        return {
            "file": self.file_name,
            "line": self.line,
            "column": self.column,
            "bytes": [self.byte_start, self.byte_end],
            "rule": self.rule,
            "authority": self.authority,
            "before": self.original,
            "after": self.replacement,
            "targets": list(self.targets),
        }


@dataclass(frozen=True)
class Skipped:
    """A site a rule recognised but declined to touch, with the guard that stopped it."""

    rule: str
    reason: str
    file_name: str
    line: int
    detail: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "rule": self.rule,
            "reason": self.reason,
            "file": self.file_name,
            "line": self.line,
            "detail": self.detail,
        }


@dataclass(frozen=True)
class Red:
    """A diagnostic no rule claims: needs a human (or an LLM repair pass), never a guess."""

    code: str | None
    shape: str
    file_name: str
    line: int
    message: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "code": self.code,
            "shape": self.shape,
            "file": self.file_name,
            "line": self.line,
            "message": self.message,
        }


@dataclass(frozen=True)
class RepairPlan:
    """Everything one target's diagnostics imply: edits, declined sites, and the residual RED."""

    target: str
    edits: tuple[Edit, ...] = ()
    skipped: tuple[Skipped, ...] = ()
    red: tuple[Red, ...] = ()


@dataclass(frozen=True)
class Withheld:
    """An edit that had a rule but not enough authority across targets to be trusted."""

    rule: str
    reason: str
    file_name: str
    line: int
    detail: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "rule": self.rule,
            "reason": self.reason,
            "file": self.file_name,
            "line": self.line,
            "detail": self.detail,
        }


@dataclass(frozen=True)
class MergedPlan:
    analysed_targets: tuple[str, ...] = ()
    edits: tuple[Edit, ...] = ()
    withheld: tuple[Withheld, ...] = ()
    skipped: tuple[Skipped, ...] = ()
    red: tuple[Red, ...] = ()


@dataclass(frozen=True)
class FlexArrayDecls:
    """bindgen flexible-array members: field name -> element type, plus ambiguous names."""

    fields: Mapping[str, str] = field(default_factory=dict)
    ambiguous: frozenset[str] = frozenset()
    source: str = ""


@dataclass(frozen=True)
class CargoCheckRun:
    target: str
    diagnostics: tuple[dict[str, Any], ...]
    error_count: int
    returncode: int
    stderr_tail: str = ""


@dataclass
class RepairPass:
    index: int
    error_counts: dict[str, int]
    applied: list[Edit] = field(default_factory=list)
    withheld: list[Withheld] = field(default_factory=list)
    skipped: list[Skipped] = field(default_factory=list)
    red: list[Red] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "pass": self.index,
            "error_counts": dict(sorted(self.error_counts.items())),
            "applied": [item.to_dict() for item in self.applied],
            "withheld": [item.to_dict() for item in self.withheld],
            "skipped": [item.to_dict() for item in self.skipped],
            "red_count": len(self.red),
        }


@dataclass
class RepairSession:
    crate: str
    targets: tuple[str, ...]
    rules: tuple[str, ...]
    passes: list[RepairPass] = field(default_factory=list)
    status: str = "fixpoint"
    detail: str = ""
    final_red: list[Red] = field(default_factory=list)
    dry_run: bool = False

    @property
    def applied(self) -> list[Edit]:
        return [edit for item in self.passes for edit in item.applied]

    def to_dict(self) -> dict[str, Any]:
        red_shapes: dict[str, int] = {}
        for item in self.final_red:
            red_shapes[item.shape] = red_shapes.get(item.shape, 0) + 1
        return {
            "schema": 1,
            "crate": self.crate,
            "targets": list(self.targets),
            "rules": list(self.rules),
            "status": self.status,
            "detail": self.detail,
            "dry_run": self.dry_run,
            "applied_total": len(self.applied),
            "passes": [item.to_dict() for item in self.passes],
            "red_shapes": dict(sorted(red_shapes.items(), key=lambda kv: (-kv[1], kv[0]))),
            "red": [item.to_dict() for item in self.final_red],
        }


# --- lexical helpers (byte-exact; rustc spans are BYTE offsets) --------------------------------

_CHAR_LIT = re.compile(rb"'(?:\\.|[^\\'\n])'")
_SPACE = 0x20
_NEWLINE = 0x0A


@functools.lru_cache(maxsize=256)
def _mask_lexical(data: bytes) -> bytes:
    """Blank comments, string literals and char literals, preserving every byte offset.

    Byte-exact on purpose: rustc reports BYTE offsets, and translated sources routinely carry
    non-ASCII in comments, so any str round-trip would shift the offsets we splice at.

    Memoized on the content: some 30 rules mask the WHOLE file once per diagnostic, and this
    is a per-byte loop. On rapps (800 diagnostics, 100 KB units) one crate-repair ran over an
    hour in this function; the file only changes when an edit lands, which is a new key.
    """
    out = bytearray(data)
    index = 0
    size = len(data)
    while index < size:
        pair = data[index : index + 2]
        char = data[index : index + 1]
        if pair == b"//":
            end = data.find(b"\n", index + 2)
            end = size if end < 0 else end
            for pos in range(index, end):
                out[pos] = _SPACE
            index = end
        elif pair == b"/*":
            end = data.find(b"*/", index + 2)
            end = size if end < 0 else end + 2
            for pos in range(index, end):
                if out[pos] != _NEWLINE:
                    out[pos] = _SPACE
            index = end
        elif char == b'"':
            pos = index + 1
            while pos < size:
                if data[pos : pos + 1] == b"\\":
                    pos += 2
                    continue
                if data[pos : pos + 1] == b'"':
                    pos += 1
                    break
                pos += 1
            for blank in range(index, min(pos, size)):
                if out[blank] != _NEWLINE:
                    out[blank] = _SPACE
            index = pos
        elif char == b"'":
            # Char literal only. A lifetime (`'a`) must NOT start a masked region, so anything
            # that is not a complete char literal is left alone.
            match = _CHAR_LIT.match(data, index)
            if match is None:
                index += 1
            else:
                for pos in range(match.start(), match.end()):
                    out[pos] = _SPACE
                index = match.end()
        else:
            index += 1
    return bytes(out)


_OPEN = {ord("("): ord(")"), ord("["): ord("]"), ord("{"): ord("}")}
_CLOSE = {value: key for key, value in _OPEN.items()}


def _match_forward(masked: bytes, start: int) -> int | None:
    """Index just past the bracket that closes the one at ``start``."""
    opener = masked[start]
    if opener not in _OPEN:
        return None
    depth = 0
    for pos in range(start, len(masked)):
        byte = masked[pos]
        if byte in _OPEN:
            depth += 1
        elif byte in _CLOSE:
            depth -= 1
            if depth == 0:
                return pos + 1
    return None


def _balanced(masked_span: bytes) -> bool:
    depth = 0
    for byte in masked_span:
        if byte in _OPEN:
            depth += 1
        elif byte in _CLOSE:
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


def _next_code_byte(masked: bytes, pos: int) -> bytes:
    while pos < len(masked) and masked[pos : pos + 1].isspace():
        pos += 1
    return masked[pos : pos + 1]


def _prev_code_byte(masked: bytes, pos: int) -> bytes:
    """The last non-space byte strictly before ``pos`` (b"" at the start of the file)."""
    pos -= 1
    while pos >= 0 and masked[pos : pos + 1].isspace():
        pos -= 1
    return masked[pos : pos + 1] if pos >= 0 else b""


def _in_match_pattern(masked: bytes, end: int) -> bool:
    """True when the span ending at ``end`` sits in match-arm PATTERN position.

    Rust forbids expressions in patterns, so wrapping such a span in `(x) as u32` turns a
    compiling match into `error: expected a pattern, found an expression` -- and it fires on
    exactly the constants that most want repairing, because a `match` on a C status code is
    where narrow-vs-wide integer mismatches surface. Two lanes hit this independently:
    wlanapi's `WlanRpcStatusToWinError` lost all 5 of its arms, and msvcrt's `create_mbcinfo`
    lost its dispatch when bindgen typed one member of a four-macro `#define` block `u32`
    (its value is 0) and the other three `i32` (theirs are negative), splitting a family
    that is all `int` in C.

    The correct repair is to widen the SCRUTINEE and leave the patterns as names -- which
    also keeps them auditable under the catch-all-binding rule (field manual §9.112) -- or
    to fix the type where the constant is DECLARED. Either way, declining here is not a
    lost fix, it is the right shape.

    Scans forward for `=>` before any of `; { } ,`. The scrutinee itself stops at `{` and is
    therefore NOT reported -- casting it is legal and often the fix.

    A span inside brackets is resolved by stepping out of them first: `foo(A, b);` and
    `(A, 0) => v` are indistinguishable at the comma, and only differ in what follows the
    closing paren. `<`/`>` are deliberately NOT treated as brackets -- they are comparison
    operators far more often than generic delimiters in this position.
    """
    pos = _after_enclosing_group(masked, end)
    depth = 0
    while pos < len(masked):
        char = masked[pos : pos + 1]
        if char in (b"(", b"["):
            depth += 1
        elif char in (b")", b"]"):
            depth -= 1  # may go negative: we left a group that opened before the span
        elif depth <= 0:
            if char == b"=" and masked[pos + 1 : pos + 2] == b">":
                return True
            if char in (b";", b"{", b"}", b","):
                return False
        pos += 1
    return False


def _in_macro_definition(masked: bytes, start: int) -> bool:
    """True when ``start`` lies inside a ``macro_rules!`` body.

    rustc reports a type error arising inside a macro EXPANSION against the macro's
    DEFINITION span, not the call site. So a repair keyed on that span edits the macro body,
    which (a) is never the right fix -- the offending value came from the caller -- and (b)
    does not converge: the next pass sees the same error, wraps the body again, and the cast
    stack grows once per pass forever.

    That is not hypothetical. `msvcrt-crt-rust`'s `ARRAY_SIZE!` is `($a).len()`, and one
    caller passing it where an `i32` was wanted drove six passes of
    `(((((($a).len()) as i32) as usize) as i32) as usize) as i32` -- corrupting a macro used
    by every translated module in the crate, to fix one call site that was never touched.

    Declining is the whole fix: the same error re-surfaces at the caller on the next run
    once a human writes the cast there, which is where it belongs.
    """
    idx = masked.rfind(b"macro_rules!", 0, start)
    if idx < 0:
        return False
    # Bound the macro's BODY exactly, rather than counting every bracket after the keyword:
    # the arm list `($a:expr) => { ... }` is full of parens, and a caller further down the
    # file sits inside its own `fn ... {`, so a naive running depth reports "still inside"
    # for the whole rest of the file. Track the outer BRACE only, from its open to its match.
    open_at = masked.find(b"{", idx)
    if open_at < 0 or open_at > start:
        return False
    depth = 0
    for pos in range(open_at, len(masked)):
        byte = masked[pos : pos + 1]
        if byte == b"{":
            depth += 1
        elif byte == b"}":
            depth -= 1
            if depth == 0:
                return open_at < start < pos
    return open_at < start  # unterminated body: treat the tail as inside


def _after_enclosing_group(masked: bytes, end: int) -> int:
    """Byte just past the innermost bracket group that encloses ``end``, else ``end``."""
    depth = 0
    pos = end - 1
    while pos >= 0:
        char = masked[pos : pos + 1]
        if char in (b")", b"]"):
            depth += 1
        elif char in (b"(", b"["):
            if depth == 0:
                closer = _matching_closer(masked, pos)
                return end if closer is None else closer + 1
            depth -= 1
        elif char in (b";", b"{", b"}") and depth == 0:
            break
        pos -= 1
    return end


def _matching_closer(masked: bytes, opening: int) -> int | None:
    depth = 0
    for pos in range(opening, len(masked)):
        char = masked[pos : pos + 1]
        if char in (b"(", b"["):
            depth += 1
        elif char in (b")", b"]"):
            depth -= 1
            if depth == 0:
                return pos
    return None


# NOTE: no `^` — `Pattern.match(data, pos)` already anchors at pos, whereas `^` would demand the
# start of the string/line and silently never fire mid-expression.
_ASSIGN_AHEAD = re.compile(rb"\s*(?:[-+*/%^&|]|<<|>>)?=(?!=)")


def _is_assignment_target(masked: bytes, end: int) -> bool:
    """True when the place expression ending at ``end`` is being assigned to.

    Walks the trailing ``.field`` / ``[index]`` chain first, because in ``x.f[i].g = 1`` the
    assignment belongs to the whole place, not to the indexed sub-expression.
    """
    pos = end
    while True:
        probe = pos
        while probe < len(masked) and masked[probe : probe + 1].isspace():
            probe += 1
        byte = masked[probe : probe + 1]
        if byte == b".":
            match = re.compile(rb"\.\s*[A-Za-z_]\w*").match(masked, probe)
            if match is None:
                break
            pos = match.end()
            continue
        if byte == b"[":
            closing = _match_forward(masked, probe)
            if closing is None:
                break
            pos = closing
            continue
        break
    return _ASSIGN_AHEAD.match(masked, pos) is not None


def _unsafe_regions(masked: bytes) -> tuple[tuple[int, int], ...]:
    """Brace ranges that are inside ``unsafe fn`` / ``unsafe { }`` / ``unsafe extern``."""
    regions: list[tuple[int, int]] = []
    for match in re.finditer(rb"\bunsafe\b", masked):
        brace = masked.find(b"{", match.end())
        if brace < 0:
            continue
        gap = masked[match.end() : brace]
        if b";" in gap or b"}" in gap:
            continue  # e.g. `unsafe fn f();` — the next `{` belongs to something else
        end = _match_forward(masked, brace)
        if end is None:
            continue
        regions.append((brace, end))
    return tuple(regions)


def _in_regions(regions: Sequence[tuple[int, int]], pos: int) -> bool:
    return any(start <= pos < end for start, end in regions)


def _line_of(data: bytes, offset: int) -> int:
    return data.count(b"\n", 0, offset) + 1


# --- diagnostic parsing -----------------------------------------------------------------------

_SUMMARY = re.compile(
    r"^(?:aborting due to|Some errors have detailed|For more information about|could not compile)"
)
_LABEL_MISMATCH = re.compile(r"^expected `(?P<expected>[^`]+)`, found `(?P<found>[^`]+)`$")
_CONVERT_HINT = re.compile(
    r"^you can convert (?:an?) `(?P<found>\w+)` to (?:an?) `(?P<expected>\w+)`"
)
_RAW_POINTER = re.compile(r"^\*(?P<mutability>mut|const)\s+(?P<pointee>.+)$")
_VOID_POINTEE = re.compile(r"^(?:(?:core|std)::ffi::)?c_void$")


def _indirection(ty: str) -> tuple[int, str]:
    """(pointer depth, innermost pointee) of a rustc type label."""
    depth = 0
    while (m := _RAW_POINTER.match(ty.strip())) is not None:
        depth += 1
        ty = m.group("pointee")
    return depth, ty.strip()


def _indirection_differs(expected: str, found: str) -> bool:
    """True when a raw-pointer mismatch changes the number of derefs. `void *` converts to and
    from any object pointer in C, so a bare `*mut c_void` on either side never counts."""
    (de, be), (df, bf) = _indirection(expected), _indirection(found)
    if de == df:
        return False
    return not ((de == 1 and _VOID_POINTEE.match(be)) or (df == 1 and _VOID_POINTEE.match(bf)))


#: An array type as rustc spells it in a mismatch label: `[u16; 80]`, `[WCHAR; MAX_PATH]`.
#: The length is captured but deliberately unused — decay is length-independent, and matching
#: on it would only invent a reason to decline.
_ARRAY_TYPE = re.compile(r"^\[\s*(?P<elem>.+?)\s*;\s*(?P<count>[^\[\]]+?)\s*\]$")
#: E0605. The same array-decay bug in CAST position — C's ``(LPARAM)szBuffer`` — which rustc
#: reports with a different code and no ``expected/found`` label, so ``_mismatch_sites`` is blind
#: to it. On intl this shape was 39 sites per target against the argument shape's 52: skipping it
#: would have mechanised barely half the class.
_NON_PRIMITIVE_CAST = re.compile(
    r"^non-primitive cast: `(?P<found>\[[^`]+\])` as `(?P<target>[^`]+)`$"
)
_STD_UNRESOLVED = "use of unresolved module or unlinked crate `std`"


def parse_cargo_messages(stdout: str) -> list[dict[str, Any]]:
    """Extract rustc diagnostics from ``cargo --message-format=json`` stdout.

    Accepts both the cargo envelope (``{"reason": "compiler-message", "message": {...}}``) and
    bare ``rustc --error-format=json`` lines, so fixtures can be captured either way.
    """
    seen: set[tuple[Any, ...]] = set()
    diagnostics: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("reason") is not None:
            if payload.get("reason") != "compiler-message":
                continue
            diagnostic = payload.get("message")
        else:
            diagnostic = payload
        if not isinstance(diagnostic, dict) or "message" not in diagnostic:
            continue
        spans = diagnostic.get("spans") or []
        primary = next((span for span in spans if span.get("is_primary")), None)
        fingerprint = (
            diagnostic.get("level"),
            diagnostic.get("message"),
            (diagnostic.get("code") or {}).get("code"),
            primary.get("file_name") if primary else None,
            primary.get("byte_start") if primary else None,
            primary.get("byte_end") if primary else None,
        )
        if fingerprint in seen:
            continue  # cargo re-reports the same diagnostic once per compiled target
        seen.add(fingerprint)
        diagnostics.append(diagnostic)
    return diagnostics


def is_error_diagnostic(diagnostic: Mapping[str, Any]) -> bool:
    """True for a real error, excluding rustc's trailing summary lines."""
    if diagnostic.get("level") != "error":
        return False
    return not _SUMMARY.match(str(diagnostic.get("message") or ""))


def count_errors(diagnostics: Sequence[Mapping[str, Any]]) -> int:
    return sum(1 for diagnostic in diagnostics if is_error_diagnostic(diagnostic))


def count_halting_errors(diagnostics: Sequence[Mapping[str, Any]]) -> int:
    """Errors with NO error code: parse and macro-expansion failures.

    rustc stops before type checking once one exists, so every later error vanishes from
    the count. A pass that adds one makes the TOTAL fall, and "the count did not rise" then
    accepts it: setuplib went 546 -> 16 on a pass whose only fault was three
    `no rules expected .` expansions. These must not rise either.
    """
    return sum(
        1 for d in diagnostics if is_error_diagnostic(d) and not (d.get("code") or {}).get("code")
    )


def pointer_width(target: str) -> int:
    """Pointer width in bytes for a cargo triple or our arch shorthand. Loud on unknowns."""
    arch = target.split("-", 1)[0]
    try:
        return _POINTER_WIDTH[arch]
    except KeyError as error:
        raise RepairError(
            f"crate_repair: unknown target {target!r} — cannot decide whether a cast bakes in a "
            f"pointer width. Add {arch!r} to crate_repair._POINTER_WIDTH."
        ) from error


def resolve_target(name: str) -> str:
    """Map ``i386``/``amd64``/``arm64`` to the cargo triple; pass a triple through unchanged."""
    if name in RUST_TARGETS:
        return RUST_TARGETS[name]
    if "-" in name:
        return name
    raise RepairError(
        f"crate_repair: {name!r} is neither an arch shorthand ({', '.join(sorted(RUST_TARGETS))}) "
        "nor a cargo target triple."
    )


# --- pure analysis: diagnostic-driven rules ---------------------------------------------------

FileReader = Callable[[str], bytes | None]


@dataclass(frozen=True)
class _Ctx:
    target: str
    width: int
    #: ``fn name -> [declared param type, ...]``, harvested from THIS crate's own sources only.
    #: Empty disables ``RULE_WIDTH_ALIAS`` entirely rather than letting it guess.
    signatures: Mapping[str, Sequence[str]] = field(default_factory=dict)
    #: Type aliases this crate itself declares — the scope proof for a cast written in the
    #: caller using a name read off the callee.
    local_aliases: frozenset[str] = frozenset()
    #: Spans ``RULE_WIDTH_ALIAS`` has already answered, so ``RULE_NUMERIC_CAST`` stands down
    #: instead of proposing a second, width-baked edit for the same bytes.
    claimed: set[tuple[str, int, int]] = field(default_factory=set)
    #: ``(file, scrutinee start, end) -> [(arm start, arm end, arm text, scrutinee type)]`` for
    #: each mismatched arm that is a bare constant path: the guard fallback when arms disagree.
    match_arm_sites: dict[tuple[str, int, int], list[tuple[int, int, str, str]]] = field(
        default_factory=dict
    )
    #: ``(struct, flattened field) -> dotted path through bindgen's ``__bindgen_anon_N``.
    #: Derived from the crate's own type declarations; empty disables ``RULE_ANON_UNION``.
    anon_unions: Mapping[tuple[str, str], str] = field(default_factory=dict)
    #: ``field name -> {struct/union declaring a field of that name's TYPE}`` (aliases
    #: resolved): which type a place `a.b.Field` has, for the autoref branch of the rule.
    field_types: Mapping[str, frozenset[str]] = field(default_factory=dict)
    #: ``scrutinee span -> {type each arm wants}``, accumulated across diagnostics so
    #: ``propose_edits`` can drop a match whose arms do not agree. One ``match`` yields one
    #: diagnostic PER offending arm, and no single-diagnostic rule can see the others.
    match_wants: dict[tuple[str, int, int], set[str]] = field(default_factory=dict)
    #: ``NAME -> value`` for every integer constant THIS crate defines with one literal value
    #: on every arch. Empty disables ``RULE_CONST_OPERAND``.
    constants: Mapping[str, int] = field(default_factory=dict)
    #: The repaired crate's directory as diagnostics spell it (`atapi-rust/`). A span outside
    #: it -- a SHARED crate's macro body -- is never edited; its call site is. Empty: all own.
    own_prefix: str = ""
    #: ``alias -> {crate-relative file that can name it}`` for every alias THIS crate declares,
    #: from ``harvest_alias_scope``. A declared alias missing from a file's set is not in scope
    #: there. Names the crate never declares (glob-reachable from a -sys island) are absent.
    alias_scope: Mapping[str, frozenset[str]] = field(default_factory=dict)


def _is_own(ctx: _Ctx, file_name: str) -> bool:
    return not ctx.own_prefix or file_name.startswith(ctx.own_prefix)


def _retarget_foreign_macro(
    span: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> Mapping[str, Any]:
    """A type error rustc blames on a SHARED crate's macro body belongs at the invocation:
    the macro is right for its other callers, and this caller's use is what needs C's
    implicit conversion (atapi's `INQUIRYDATABUFFERSIZE - RTL_SIZEOF_THROUGH_FIELD!(..)`)."""
    if not span.get("expansion"):
        return span
    file_name = str(span.get("file_name") or "")
    if _is_own(ctx, file_name):
        return span
    data = load(file_name)
    if data is None or not _in_macro_definition(
        _mask_lexical(data), int(span.get("byte_start") or 0)
    ):
        return span
    return _outermost_call_site(span)


def _shape_of(diagnostic: Mapping[str, Any]) -> str:
    code = (diagnostic.get("code") or {}).get("code")
    for span in diagnostic.get("spans") or []:
        match = _LABEL_MISMATCH.match(str(span.get("label") or ""))
        if match is not None:
            expected = _normalise_type(match.group("expected"))
            found = _normalise_type(match.group("found"))
            return f"{code or 'E?'}:{found}->{expected}"
    return str(code or "uncoded")


def _normalise_type(name: str) -> str:
    name = name.strip()
    if name in INT_PRIMITIVES:
        return "int"
    pointer = _RAW_POINTER.match(name)
    if pointer is not None:
        return f"*{pointer.group('mutability')}"
    if name.startswith("&"):
        return "&ref"
    if name.startswith("fn") or '" fn' in name:
        return "fn"
    return "other"


def _mismatch_sites(diagnostic: Mapping[str, Any]) -> list[tuple[dict[str, Any], str, str]]:
    """Every span carrying an ``expected X, found Y`` label.

    Deliberately NOT "the primary span": in the ``arguments to this function are incorrect``
    shape the primary span is the CALLEE and the fixable value spans are secondary (verified
    against rustc 1.92).  Using labels finds both shapes and never invents a site.
    """
    sites = []
    seen: set[tuple[str, int, int]] = set()

    def record(span: Mapping[str, Any], expected: str, found: str) -> None:
        key = (
            str(span.get("file_name") or ""),
            int(span.get("byte_start") or 0),
            int(span.get("byte_end") or 0),
        )
        if key in seen:
            return
        seen.add(key)
        sites.append((dict(span), expected, found))

    for span in diagnostic.get("spans") or []:
        match = _LABEL_MISMATCH.match(str(span.get("label") or ""))
        if match is None:
            continue
        record(span, match.group("expected"), match.group("found"))
    # THE MULTI-ARGUMENT RENDERING. When a call has 2+ bad arguments rustc switches shape: the
    # message becomes "arguments to this function are incorrect", every top-level span loses its
    # label, and each mismatch moves into a CHILD note whose MESSAGE carries the types and whose
    # single span is the argument. Reading only top-level labels made every rule in this module
    # silently blind to those calls -- intl had 7 per target sitting in RED for that reason
    # alone, all of them ordinary two-argument `wcscpy(dst, src)`.
    for child in diagnostic.get("children") or []:
        match = _LABEL_MISMATCH.match(str(child.get("message") or ""))
        if match is None:
            continue
        spans = child.get("spans") or []
        if len(spans) != 1:
            # Zero spans is the companion "expected raw pointer ...\n found array ..." note,
            # which carries no location; several would be ambiguous. Neither is attributable.
            continue
        record(spans[0], match.group("expected"), match.group("found"))
    return sites


def _conversion_hints(diagnostic: Mapping[str, Any]) -> dict[int, str]:
    """Map ``insertion offset -> target type`` from rustc's own conversion help children.

    The child span is a ZERO-LENGTH insertion point at the end of the offending expression, so
    it is useful as corroboration keyed by offset — never as a span to wrap (wrapping it is what
    produced `() as u32` debris in the hand-rolled version of this pass).
    """
    hints: dict[int, str] = {}
    for child in diagnostic.get("children") or []:
        match = _CONVERT_HINT.match(str(child.get("message") or ""))
        if match is None:
            continue
        for span in child.get("spans") or []:
            start = span.get("byte_start")
            if start is not None and start == span.get("byte_end"):
                hints[int(start)] = match.group("expected")
    return hints


def _enclosing_call(masked: bytes, start: int) -> tuple[str | None, int]:
    """``(callee name, argument index)`` for the call whose argument begins at ``start``.

    Walks out to the nearest UNCLOSED ``(`` and then forward from it counting top-level commas.
    ``<``/``>`` deliberately do not nest: ``size_of::<T>()`` and ``a >= b`` both appear in real
    argument lists, and treating either as a bracket desynchronises the depth and silently
    shifts the index — which reads the WRONG parameter's type off the callee.
    """
    depth, index, open_paren = 0, start - 1, None
    while index >= 0:
        char = masked[index : index + 1]
        if char in b")]}":
            depth += 1
        elif char in b"([{":
            if depth == 0:
                open_paren = index if char == b"(" else None
                break
            depth -= 1
        elif char == b";" and depth == 0:
            break
        index -= 1
    if open_paren is None:
        return None, 0
    name_match = re.search(rb"(\w+)$", masked[:open_paren].rstrip())
    if name_match is None:
        return None, 0
    arg_index, depth = 0, 0
    for pos in range(open_paren + 1, start):
        char = masked[pos : pos + 1]
        if char in b"([{":
            depth += 1
        elif char in b")]}":
            depth -= 1
        elif char == b"," and depth == 0:
            arg_index += 1
    return name_match.group(1).decode("utf-8", "replace"), arg_index


#: A trailing ``as <int primitive>`` on an argument expression — the width the translation baked.
_RE_TRAILING_CAST = re.compile(r"\s+as\s+(?:" + "|".join(sorted(INT_PRIMITIVES)) + r")\s*$")


def _return_type_name(diagnostic: Mapping[str, Any], load: FileReader) -> str | None:
    """The type NAME a mismatch's "expected ... because of return type" label points at."""
    for span in diagnostic.get("spans") or []:
        if "because of return type" not in str(span.get("label") or ""):
            continue
        data = load(str(span.get("file_name") or ""))
        if data is None:
            return None
        text = data[int(span["byte_start"]) : int(span["byte_end"])].decode("utf-8", "replace")
        return text.strip() if re.fullmatch(r"\s*[A-Za-z_]\w*\s*", text) else None
    return None


def _width_alias_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """int-vs-int at an argument position -> cast to the callee's DECLARED TYPE NAME.

    THE MEASUREMENT THAT MOTIVATES IT. ``RULE_NUMERIC_CAST`` answers the same E0308 with the
    width rustc resolved on the target it ran on — ``as u32`` from an i386 run — and
    ``merge_plans`` then withholds it unless another target independently asks for the same
    width. On syssetup that almost never happens: of the 17 integer sites erroring on all three
    targets, exactly ONE had three-way agreement, because the PARAMETER TYPES themselves differ
    per arch (SIZE_T, WPARAM, UINT_PTR). Cross-target voting cannot resolve a class where the
    targets are *correctly* asking for different widths.

    What is portable is the NAME. ``HeapAlloc``'s third parameter is ``SIZE_T`` on every arch;
    the alias is what varies, not the spelling. So this rule reads the type off the callee's own
    declaration and writes ``as SIZE_T`` — one edit, right on all three, needing no vote.

    THE SCOPE TRAP, and why the guard is what it is. The cast is written in the CALLER's scope
    from a name read in the CALLEE's, so an alias that resolves differently in the two compiles
    and is wrong (``propose_panicking_cast_edits`` ate exactly this: rtl-misc's ``SIZE_T`` is
    ``usize``, ntkrnl-sys's amd64 bindgen declares the parameter ``u64``, and rustc holds those
    distinct at equal width). The guard here is not "is it a primitive" — that would reject the
    entire class — it is PROVENANCE: the callee must be declared in this crate's own sources,
    and the alias must be one this crate itself declares. Same crate, same ``types`` island,
    one meaning.
    """
    if not ctx.signatures:
        return [], [], False
    sites = [
        (span, expected, found)
        for span, expected, found in _mismatch_sites(diagnostic)
        if expected in INT_PRIMITIVES and found in INT_PRIMITIVES and expected != found
    ]
    if not sites:
        return [], [], False
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    for span, expected, found in sites:
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)

        def decline(reason: str, detail: str = "", line: int = line, name: str = file_name) -> None:
            skipped.append(Skipped(RULE_WIDTH_ALIAS, reason, name, line, detail))

        if end <= start:
            continue  # zero-length insertion hint; RULE_NUMERIC_CAST reports these
        data = load(file_name)
        if data is None:
            continue
        masked = _mask_lexical(data)
        if _next_code_byte(masked, end) == b"(":
            continue  # the callee's own span, not a value
        if _in_macro_definition(masked, start) or _in_match_pattern(masked, end):
            continue
        text = data[start:end].decode("utf-8", "replace")
        if not _balanced(masked[start:end]):
            continue
        returned = _return_type_name(diagnostic, load)
        if (
            returned is not None
            and masked[:start].rstrip().endswith(b"return")
            and _next_code_byte(masked, end) == b";"
        ):
            # `return (EXIT_SUCCESS) as i32;` in a fn declared `-> EXIT_CODE`: the function's
            # own signature is the authority, exactly as a callee's parameter is below.
            # diskpart's command table (31 E0308s) became 31 of these once the definitions
            # took the C prototype's return type.
            if returned in INT_PRIMITIVES or returned not in ctx.local_aliases:
                decline("return_alias_not_local", f"`{returned}`")
                continue
            inner = _RE_TRAILING_CAST.sub("", text).strip()
            if not _balanced(_mask_lexical(inner.encode())):
                inner = text.strip()
            ctx.claimed.add((file_name, start, end))
            edits.append(
                Edit(
                    file_name=file_name,
                    byte_start=start,
                    byte_end=end,
                    replacement=f"({inner}) as {returned}",
                    rule=RULE_WIDTH_ALIAS,
                    authority=(
                        f"the enclosing function returns `{returned}`, declared in this crate; "
                        f"the expression is `{found}`. Casting to the NAME is portable."
                    ),
                    original=text,
                    line=line,
                    column=int(span.get("column_start") or 0),
                    requires_target_agreement=False,
                    targets=(ctx.target,),
                )
            )
            continue
        if _prev_code_byte(masked, start) not in (b"(", b",") or _next_code_byte(
            masked, end
        ) not in (
            b")",
            b",",
        ):
            # The span must BE the whole argument, not a sub-expression of one. rustc blames the
            # narrowest disagreeing operand, so in
            #     SetAutoDaylight(SendDlgItemMessage(..) != (BST_UNCHECKED) as i64)
            # it labels `(BST_UNCHECKED) as i64` — which sits inside argument 0, not at it.
            # Casting THAT to SetAutoDaylight's `BOOL` is a well-formed edit that changes the
            # wrong expression; the real repair is around the whole comparison.
            decline("not_whole_argument", text)
            continue
        callee, arg_index = _enclosing_call(masked, start)
        if callee is None:
            continue  # not an argument — nothing to read a type name off
        params = ctx.signatures.get(callee)
        if params is None:
            decline("callee_not_local", f"`{callee}` is not declared in this crate's own sources")
            continue
        if arg_index >= len(params) or not params[arg_index]:
            decline(
                "callee_arity", f"{callee}() declares {len(params)} params; this is {arg_index}"
            )
            continue
        target = params[arg_index].strip()
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", target):
            continue  # a pointer/array/fn-ptr parameter — a different rule's business
        if target in INT_PRIMITIVES:
            # The declaration already names a primitive, so there is no portable alias to
            # prefer: RULE_NUMERIC_CAST's `as <expected>` is the same edit. Leave it to it.
            continue
        if target not in ctx.local_aliases:
            decline(
                "alias_not_local",
                f"`{target}` is not a type this crate declares, so its meaning in the caller's "
                "scope is not proven",
            )
            continue
        scope = ctx.alias_scope.get(target)
        here = file_name[len(ctx.own_prefix) :] if _is_own(ctx, file_name) else file_name
        if scope is not None and here not in scope:
            # The alias is real but private to another module: fastfat's flush.rs declares
            # `type FAT_FLUSH_TYPE = i32;` without `pub`, and three callers in other modules
            # got `as FAT_FLUSH_TYPE` (E0412). The pass loop kept them because the same pass
            # fixed more than it broke, so this has to be a guard, not a revert.
            decline("alias_not_in_scope", f"`{target}` is declared in {sorted(scope)[0]} only")
            continue
        if re.search(r"\bas\s+" + re.escape(target) + r"\s*$", text):
            # An existing cast to this spelling still errors: caller and callee resolve
            # different aliases. Repeating it cannot fix the mismatch (SysAudio SIZE_T).
            ctx.claimed.add((file_name, start, end))
            decline(
                "alias_scope_mismatch",
                f"`{text}` already casts to `{target}`; resolve the caller's import",
            )
            continue
        # Strip a width the translation already baked (`(x) as u64`) rather than stacking a
        # second cast on top of it — the value is unchanged and the source stays readable.
        inner = _RE_TRAILING_CAST.sub("", text)
        if inner != text and not _balanced(_mask_lexical(inner.encode())):
            inner = text
        inner = inner.strip()
        # Already a single parenthesised group? Then `(inner) as T` would only add `((..))`.
        wrapped = inner
        if not (
            inner.startswith("(")
            and inner.endswith(")")
            and _matching_closer(_mask_lexical(inner.encode()), 0) == len(inner) - 1
        ):
            wrapped = f"({inner})"
        ctx.claimed.add((file_name, start, end))
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"{wrapped} as {target}",
                rule=RULE_WIDTH_ALIAS,
                authority=(
                    f"{callee}() parameter {arg_index} is declared `{target}` in this crate's "
                    f"own sources, and `{target}` is declared here too; rustc resolved it to "
                    f"`{expected}` on {ctx.target} and the expression is `{found}`. Casting to "
                    "the NAME is portable across all three targets, so it needs no width vote."
                ),
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                requires_target_agreement=False,
                targets=(ctx.target,),
            )
        )
    return edits, skipped, bool(edits)


def _comparison_rule(diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx):
    edits = []
    skipped = []
    for span, expected, found in _mismatch_sites(diagnostic):
        if expected not in INT_PRIMITIVES or found not in INT_PRIMITIVES:
            continue
        file_name = str(span.get("file_name") or "")
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        data = load(file_name)
        if data is None or end <= start or (file_name, start, end) in ctx.claimed:
            continue
        operands = comparison_operands(data, start, end)
        if operands is None:
            continue
        # Prevent the assignment-cast rule from narrowing a comparison operand.
        ctx.claimed.add((file_name, start, end))
        target = promoted_type(expected, found)
        if operands[1] != (start, end) or target is None:
            skipped.append(
                Skipped(
                    RULE_COMPARISON,
                    "promotion_not_proven",
                    file_name,
                    int(span.get("line_start") or 0),
                    "comparison requires source integer-promotion review",
                )
            )
            continue
        for (lo, hi), original_type in zip(operands, (expected, found), strict=True):
            if original_type == target:
                continue
            text = data[lo:hi].decode()
            edits.append(
                Edit(
                    file_name=file_name,
                    byte_start=lo,
                    byte_end=hi,
                    replacement=f"({text}) as {target}",
                    original=text,
                    rule=RULE_COMPARISON,
                    authority=(
                        f"C integer promotions: compare {expected} and {found} as {target}; "
                        "never truncate the other operand to a byte (PortCls resource types)"
                    ),
                    line=int(span.get("line_start") or 0),
                    column=int(span.get("column_start") or 0),
                    requires_target_agreement=False,
                    targets=(ctx.target,),
                )
            )
    return edits, skipped, bool(edits)


def _numeric_cast_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    sites = [
        (span, expected, found)
        for span, expected, found in _mismatch_sites(diagnostic)
        if expected in INT_PRIMITIVES and found in INT_PRIMITIVES
    ]
    if not sites:
        return [], [], False
    hints = _conversion_hints(diagnostic)
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    for span, expected, found in sites:
        span = _retarget_foreign_macro(span, load, ctx)
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)

        def decline(reason: str, detail: str = "", line: int = line, name: str = file_name) -> None:
            skipped.append(Skipped(RULE_NUMERIC_CAST, reason, name, line, detail))

        if end <= start:
            # Guard: rustc's child suggestion spans are zero-length INSERTION points.
            decline("zero_length_span", f"{expected} at byte {start}")
            continue
        if (file_name, start, end) in ctx.claimed:
            # RULE_WIDTH_ALIAS already answered this span with the callee's declared type NAME,
            # which is portable. A second, width-baked edit for the same bytes would only
            # collide in select_non_overlapping.
            decline("claimed_by_width_alias", f"bytes {start}..{end}")
            continue
        hint = hints.get(end)
        if hint is not None and hint != expected:
            decline("hint_label_disagreement", f"label={expected} hint={hint}")
            continue
        data = load(file_name)
        if data is None:
            decline("unreadable_file")
            continue
        if comparison_operands(data, start, end) is not None:
            decline("comparison_requires_promotion", "enable c_integer_comparison or review source")
            continue
        masked = _mask_lexical(data)
        if _next_code_byte(masked, end) == b"(":
            # Guard: a span followed by `(` is a CALLEE name (the primary span of the
            # "arguments to this function are incorrect" shape), not a value to cast.
            decline("callee_span", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_macro_definition(masked, start):
            # Guard: rustc blames a macro's DEFINITION for a type error in its expansion, so
            # repairing that span edits the macro body -- wrong fix, and it re-fires every
            # pass because the caller never changes. See _in_macro_definition.
            decline("macro_definition", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_match_pattern(masked, end):
            # Guard: a cast is an expression, and Rust forbids expressions in patterns. The
            # type disagreement is real, but it has to be resolved where the constant is
            # DECLARED, or by widening the scrutinee -- never at the arm.
            decline("match_pattern", data[start:end].decode("utf-8", "replace"))
            continue
        text = data[start:end].decode("utf-8", "replace")
        if not _balanced(masked[start:end]):
            decline("unbalanced_span", text)
            continue
        if re.search(rf"\bas\s+{re.escape(expected)}\s*$", text):
            decline("already_cast", text)
            continue
        sensitive = INT_WIDTH.get(expected) == ctx.width
        authority = (
            f"rustc {(diagnostic.get('code') or {}).get('code') or 'E?'} label 'expected "
            f"`{expected}`, found `{found}`'; both are integer primitives, so a truncating "
            f"`as {expected}` reproduces the C implicit conversion (rustc's own "
            f"`.try_into().unwrap()` would inject a panic)"
        )
        if expected in POINTER_SIZED:
            authority += "; target type is the width-varying alias itself, so it is portable"
        elif sensitive:
            authority += (
                f"; `{expected}` is pointer-width on {ctx.target}, so the concrete width needs "
                "cross-target agreement"
            )
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({text}) as {expected}",
                rule=RULE_NUMERIC_CAST,
                authority=authority,
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                requires_target_agreement=sensitive,
                targets=(ctx.target,),
            )
        )
    return edits, skipped, True


_RE_FN_TO_VOID = re.compile(r"expected `\*(mut|const) c_void`, found fn item$")
_RE_ZERO_TO_PTR = re.compile(
    r"expected `\*(mut|const) [^`]+`, found `(?:usize|isize|i32|u32|integer)`$"
)


def _fn_or_zero_as_pointer(span: Mapping[str, Any], load: FileReader, ctx: _Ctx) -> Edit | None:
    """Two C pointer conversions with no Rust spelling of their own:

    - a FUNCTION where `void *` is wanted (atapi passes its DPC routine as an
      IoAcquireRemoveLock tag): `(f as *mut c_void)`, the function's address;
    - the literal `0` where a pointer is wanted (`if (Worker.Thread != 0)`): C's null
      pointer constant, `core::ptr::null_mut()` / `null()`. Only the bare literal.
    """
    if not span.get("is_primary"):
        return None
    label = str(span.get("label") or "")
    fn_m, zero_m = _RE_FN_TO_VOID.match(label), _RE_ZERO_TO_PTR.match(label)
    if fn_m is None and zero_m is None:
        return None
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or end <= start or _in_macro_definition(_mask_lexical(data), start):
        return None
    text = data[start:end].decode("utf-8", "replace").strip()
    if fn_m is not None:
        if not re.fullmatch(r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*", text):
            return None
        replacement = f"({text} as *{fn_m.group(1)} core::ffi::c_void)"
        why = "C converts a function to `void *` as its address"
    else:
        assert zero_m is not None
        if text not in ("0", "0usize", "0isize"):
            return None
        replacement = "core::ptr::null_mut()" if zero_m.group(1) == "mut" else "core::ptr::null()"
        why = "C's `0` in pointer context is the null pointer constant"
    return Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=end,
        replacement=replacement,
        rule=RULE_POINTER_CAST,
        authority=f"rustc: {label}; {why}",
        original=text,
        line=int(span.get("line_start") or 0),
        column=int(span.get("column_start") or 0),
        requires_target_agreement=False,
        targets=(ctx.target,),
    )


def _c_type_name(rust: str) -> str | None:
    """`*mut ntkrnl_sys::ddk::_SLIST_HEADER` -> `SLIST_HEADER`: the C tag, path and `_` gone."""
    m = re.fullmatch(r"(?:&(?:mut )?|\*(?:mut|const) )?((?:[A-Za-z_]\w*::)*)_?([A-Za-z]\w*)", rust)
    return m.group(2) if m else None


def _same_c_type_cast(
    span: Mapping[str, Any], expected: str, found: str, load: FileReader, ctx: _Ctx
) -> Edit | None:
    """A pointer to ONE C struct under two Rust identities: `.cast()`.

    A driver island re-emits a struct the ddk glob carries only opaquely (gen_driver_types'
    UNBLOCK: ntkrnl-sys's `_SCSI_REQUEST_BLOCK` is `{ _address: u8 }`), and ntkrnl-sys's
    hand-written `SLIST_HEADER` sits beside bindgen's `_SLIST_HEADER`. The address is the same
    object either way; only the nominal type differs. Both sides must be paths to the same C
    name, at least one of them from another crate. A value that is not a pointer stays loud.
    """
    want, have = _c_type_name(expected), _c_type_name(found)
    bare = r"^(?:&(?:mut )?|\*(?:mut|const) )"
    # Spelled differently but naming one C type: two Rust identities. (rustc's multi-argument
    # rendering drops the paths -- `*mut SLIST_HEADER` vs `&mut _SLIST_HEADER` -- so a path is
    # not required; the same spelling twice is not this shape at all.)
    if want is None or want != have or re.sub(bare, "", expected) == re.sub(bare, "", found):
        return None
    if expected.startswith("&") or (found.startswith("*const") and expected.startswith("*mut")):
        return None
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or end <= start:
        return None
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start) or not _balanced(masked[start:end]):
        return None
    if _next_code_byte(masked, end) == b"(":
        return None
    text = data[start:end].decode("utf-8", "replace").strip()
    ref = re.fullmatch(r"&\s*(mut\s+)?(.+)", text, re.S)
    if found.startswith("&"):
        if ref is None:
            return None
        text = f"&raw {'mut' if ref.group(1) else 'const'} {ref.group(2).strip()}"
    return Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=end,
        replacement=f"({text}).cast()",
        rule=RULE_POINTER_CAST,
        authority=f"`{expected}` and `{found}` are the same C type `{want}` under two Rust paths",
        original=data[start:end].decode("utf-8", "replace"),
        line=int(span.get("line_start") or 0),
        column=int(span.get("column_start") or 0),
        requires_target_agreement=False,
        targets=(ctx.target,),
    )


def _ref_to_void_pointer(
    span: Mapping[str, Any], expected: str, found: str, load: FileReader, ctx: _Ctx
) -> Edit | None:
    """`&mut X` where `*mut c_void` is wanted: C's implicit `T *` -> `void *` (atapi's
    `KeWaitForSingleObject(&Event, ..)`, PVOID Object). `(&raw mut X).cast()`: no reference
    is made, and the pointee type comes from the parameter. A shared `&X` goes only to
    `*const c_void`; to `*mut` it is a mutability question, not this one."""
    want = re.fullmatch(r"\*(mut|const) (?:core::ffi::|std::ffi::|libc::)?c_void", expected)
    if want is None or not found.startswith("&"):
        return None
    mutable = found.startswith("&mut ")
    if want.group(1) == "mut" and not mutable:
        return None
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or end <= start:
        return None
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start) or not _balanced(masked[start:end]):
        return None
    text = data[start:end].decode("utf-8", "replace").strip()
    m = re.fullmatch(r"&\s*(mut\s+)?(.+)", text, re.S)
    if m is None or bool(m.group(1)) != mutable:
        return None
    return Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=end,
        replacement=f"(&raw {'mut' if mutable else 'const'} {m.group(2).strip()}).cast()",
        rule=RULE_POINTER_CAST,
        authority=f"C converts any object pointer to `void *` implicitly; found `{found}`",
        original=text,
        line=int(span.get("line_start") or 0),
        column=int(span.get("column_start") or 0),
        requires_target_agreement=False,
        targets=(ctx.target,),
    )


# A place: a name or a DEREFERENCE -- `(*p)`, `(*CImageDx::_shared())` (a deref is a place
# whatever the pointer came from) -- then fields and indexes.
_PLACE = re.compile(
    r"(?:\(\*\s*[\w.:]+(?:\(\))?\s*\)"
    # `(*core::ptr::addr_of_mut!(rcImage))` and a parenthesised name `(rc)`: still places.
    r"|\(\*\s*(?:core::ptr::)?addr_of(?:_mut)?!\(\s*[\w.]+\s*\)\s*\)"
    r"|\(\s*[A-Za-z_]\w*\s*\)"
    r"|[A-Za-z_]\w*)(?:\s*\.\s*\w+|\[[^\[\]]+\])*"
)


def _cpp_reference_arg(
    span: Mapping[str, Any], expected: str, found: str, load: FileReader, ctx: _Ctx
) -> Edit | None:
    """A C++ REFERENCE parameter (`POINT& pt`, `const RECT& rc`), passed the object:
    `expected &mut tagPOINT, found tagPOINT` -> `&mut pt`; `&T`/`*const T` -> `&(expr)`
    (a const reference binds a temporary too); `*mut T` takes `&mut place` (Rust coerces
    the reference to the pointer). Plain C cannot produce this shape -- passing a struct
    where its pointer is wanted does not compile -- so only a reference reaches it. A
    `&mut` wants a PLACE; anything else (a call's result) stays red."""
    m = re.fullmatch(r"(&mut |&|\*mut |\*const )(.+)", expected)

    def same(a: str) -> str:  # `cpp_classes::CRect` and `CRect` are one type
        return a.strip().split("::")[-1]

    def matches(want: str, have: str) -> bool:
        if same(want) == same(have):
            return True
        # A generic parameter rustc has not inferred is `_` (`&mut Option<_>` for
        # CImageDx::get_fn<F>): a wildcard, matched against the found type.
        if "_" not in re.sub(r"\w_|_\w", "", want):
            return False
        pat = re.escape(re.sub(r"\s+", " ", want.strip())).replace(r"\_", "_")
        pat = re.sub(r"(?<![\w])_(?![\w])", ".+", pat)
        return re.fullmatch(pat, re.sub(r"\s+", " ", have.strip())) is not None

    if m is None:
        return None
    if m.group(1) in ("&mut ", "&") and m.group(2).strip() == "_":
        # A fully generic reference parameter (precomp.h's `Swap(T& a, T& b)`): any PLACE
        # binds, whatever its type -- including a pointer-typed HBITMAP.
        file_name = str(span.get("file_name") or "")
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        data = load(file_name)
        text = data[start:end].decode("utf-8", "replace").strip() if data else ""
        if not _PLACE.fullmatch(text):
            return None
        return Edit(
            file_name=file_name,
            byte_start=start,
            byte_end=end,
            replacement=f"{m.group(1)}{text}",
            rule=RULE_CPP_REFERENCE_ARG,
            authority=f"a generic C++ reference parameter binds the place; expected `{expected}`",
            original=text,
            line=int(span.get("line_start") or 0),
            column=int(span.get("column_start") or 0),
            requires_target_agreement=False,
            targets=(ctx.target,),
        )
    raw = re.fullmatch(r"\*(mut|const) (.+)", found.strip())
    if m.group(1) in ("&mut ", "&") and raw is not None and matches(m.group(2), raw.group(2)):
        # The model wrote C's `&place` as `addr_of_mut!(place)` for a C++ REFERENCE param
        # (`canvasWindow.GetImageRect(rcImage)`): the reference to that place is the arg.
        file_name = str(span.get("file_name") or "")
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        data = load(file_name)
        if data is None or end <= start:
            return None
        text = data[start:end].decode("utf-8", "replace").strip()
        a = re.fullmatch(r"(?:core::ptr::)?addr_of(_mut)?!\((.+)\)", text, re.S)
        if a is None or not _PLACE.fullmatch(a.group(2).strip()):
            return None
        if m.group(1) == "&mut " and a.group(1) is None:
            return None  # a const address where a mutable reference is wanted: not ours
        return Edit(
            file_name=file_name,
            byte_start=start,
            byte_end=end,
            replacement=f"{m.group(1)}{a.group(2).strip()}",
            rule=RULE_CPP_REFERENCE_ARG,
            authority=f"a C++ reference parameter binds the place itself; expected `{expected}`",
            original=text,
            line=int(span.get("line_start") or 0),
            column=int(span.get("column_start") or 0),
            requires_target_agreement=False,
            targets=(ctx.target,),
        )
    if not matches(m.group(2), found) or found.lstrip().startswith(("&", "*")):
        return None
    mutable = m.group(1) in ("&mut ", "*mut ")
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or end <= start:
        return None
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start) or not _balanced(masked[start:end]):
        return None
    text = data[start:end].decode("utf-8", "replace").strip()
    if mutable and not _PLACE.fullmatch(text):
        return None
    repl = f"&mut {text}" if mutable else (f"&{text}" if _PLACE.fullmatch(text) else f"&({text})")
    return Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=end,
        replacement=repl,
        rule=RULE_CPP_REFERENCE_ARG,
        authority=f"a C++ reference parameter binds the object itself; expected `{expected}`",
        original=text,
        line=int(span.get("line_start") or 0),
        column=int(span.get("column_start") or 0),
        requires_target_agreement=False,
        targets=(ctx.target,),
    )


def _pointer_cast_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    claimed = False
    for span in diagnostic.get("spans") or []:
        edit = _fn_or_zero_as_pointer(span, load, ctx)
        if edit is not None:
            edits.append(edit)
            claimed = True
    # Several bad arguments in one call are ONE "arguments to this function are incorrect"
    # error: each argument's "expected `*mut HWND__`, found `usize`" is a child NOTE whose
    # span is the argument, with no label of its own. sndrec32's `MessageBox(0, ..)` and
    # `CreateEvent(0, .., 0)` were 32 errors the rule never saw.
    for child in diagnostic.get("children") or []:
        message = str(child.get("message") or "")
        for span in child.get("spans") or []:
            edit = _fn_or_zero_as_pointer({**span, "label": message}, load, ctx)
            if edit is not None:
                edits.append(edit)
                claimed = True
    for span, expected, found in _mismatch_sites(diagnostic):
        twin = _same_c_type_cast(span, expected, found, load, ctx)
        if twin is not None:
            edits.append(twin)
            claimed = True
            continue
        void_edit = _ref_to_void_pointer(span, expected, found, load, ctx)
        if void_edit is not None:
            edits.append(void_edit)
            claimed = True
            continue
        ref_edit = _cpp_reference_arg(span, expected, found, load, ctx)
        if ref_edit is not None:
            edits.append(ref_edit)
            claimed = True
            continue
        want = _RAW_POINTER.match(expected)
        have = _RAW_POINTER.match(found)
        if want is None or have is None:
            continue
        claimed = True
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)

        def decline(reason: str, detail: str = "", line: int = line, name: str = file_name) -> None:
            skipped.append(Skipped(RULE_POINTER_CAST, reason, name, line, detail))

        if want.group("mutability") != have.group("mutability"):
            # A const<->mut flip is a real semantic escalation, not a shape repair. Stays RED.
            decline("pointer_mutability_flip", f"expected {expected}, found {found}")
            continue
        if _indirection_differs(expected, found):
            # `*mut *mut *mut c_void` where `*mut *mut c_void` is wanted is a missing or extra
            # deref, not a pointee rename, and `.cast()` would compile it. Only a bare void
            # pointer on either side is C's implicit conversion. Stays RED.
            decline("indirection_depth", f"expected {expected}, found {found}")
            continue
        if end <= start:
            decline("zero_length_span", expected)
            continue
        data = load(file_name)
        if data is None:
            decline("unreadable_file")
            continue
        masked = _mask_lexical(data)
        if _next_code_byte(masked, end) == b"(":
            decline("callee_span", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_macro_definition(masked, start):
            # Same reason as the numeric rule: a macro-expansion error is reported against
            # the DEFINITION, and repairing there edits a body shared by every caller.
            decline("macro_definition", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_match_pattern(masked, end):
            decline("match_pattern", data[start:end].decode("utf-8", "replace"))
            continue
        text = data[start:end].decode("utf-8", "replace")
        if not _balanced(masked[start:end]):
            decline("unbalanced_span", text)
            continue
        if text.rstrip().endswith(".cast()"):
            decline("already_cast", text)
            continue
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({text}).cast()",
                rule=RULE_POINTER_CAST,
                authority=(
                    f"rustc label 'expected `{expected}`, found `{found}`': both raw pointers of "
                    "the same mutability. `<*mut T>::cast::<U>()` INFERS U from the expected "
                    "type, so no type name is guessed and no provenance/mutability changes"
                ),
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    return edits, skipped, claimed


def _split_cast(masked: bytes, start: int, end: int) -> int | None:
    """Offset of the top-level ``as`` operator in the cast expression spanning ``[start, end)``.

    The LAST one at depth zero: ``a as usize as *mut T`` is two casts and the span rustc gives us
    ends at the outer one.  Depth-tracking on ``()[]{}`` keeps an ``as`` inside a nested call or
    index from being mistaken for the operator — ``f(x as u32) as LPARAM`` has two, and picking
    the inner one would splice the decay into the wrong expression.
    """
    found = None
    for match in re.finditer(rb"\bas\b", masked[start:end]):
        pos = start + match.start()
        depth = 0
        for index in range(start, pos):
            byte = masked[index : index + 1]
            if byte in b"([{":
                depth += 1
            elif byte in b")]}":
                depth -= 1
        if depth == 0:
            found = pos
    return found


def _array_decay_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """``[T; N]`` where a ``*T`` is wanted -> ``.as_ptr()`` / ``.as_mut_ptr()``.

    C's most basic idiom: ``WCHAR szBuf[80]`` passed to an ``LPWSTR`` parameter decays to a
    pointer at the call.  Rust has no implicit decay, so every such argument arrives as
    ``expected `*mut u16`, found `[u16; 80]``` and every module has paid for it by hand.

    THE GUARD IS ELEMENT-TYPE IDENTITY, and it is the whole safety argument.  ``[u16; N]`` ->
    ``*mut u8`` is NOT a decay, it is a REINTERPRET — the same shape as the wide-literal bug
    class that shipped truncated registry names in wkssvc and an unbindable pipe name in
    browser.  A rule that "helpfully" decayed across element types would silently bless exactly
    that.  Mismatched elements stay RED, including ``c_void``: the void-pointer conversion C
    performs for free is a second, separate step, and if it is genuinely wanted the next pass's
    ``RULE_POINTER_CAST`` adds the ``.cast()`` on evidence rather than this rule guessing it.

    NOTE the deliberate exception to this module's central discipline: the edit is emitted with
    ``requires_target_agreement=False``.  Every other cast rule bakes a width and therefore needs
    cross-target corroboration, but decay is width-INDEPENDENT — ``as_ptr`` names no type and
    resolves identically at 32 and 64 bits — so demanding agreement here would withhold correct
    edits at any site that happens to compile on one arch for an unrelated reason.
    """
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    claimed = False
    for span, expected, found in _mismatch_sites(diagnostic):
        want = _RAW_POINTER.match(expected)
        have = _ARRAY_TYPE.match(found)
        if want is None or have is None:
            continue
        claimed = True
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)

        def decline(reason: str, detail: str = "", line: int = line, name: str = file_name) -> None:
            skipped.append(Skipped(RULE_ARRAY_DECAY, reason, name, line, detail))

        pointee = want.group("pointee").strip()
        element = have.group("elem").strip()
        if pointee != element:
            decline(
                "element_type_mismatch",
                f"expected {expected}, found {found}: decay yields *{element}, not *{pointee}. "
                "This is a REINTERPRET, not an array decay — fix the declared type, or cast "
                "explicitly if the reinterpret is genuinely intended.",
            )
            continue
        if end <= start:
            decline("zero_length_span", expected)
            continue
        data = load(file_name)
        if data is None:
            decline("unreadable_file")
            continue
        masked = _mask_lexical(data)
        if _next_code_byte(masked, end) == b"(":
            decline("callee_span", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_macro_definition(masked, start):
            decline("macro_definition", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_match_pattern(masked, end):
            decline("match_pattern", data[start:end].decode("utf-8", "replace"))
            continue
        text = data[start:end].decode("utf-8", "replace")
        if not _balanced(masked[start:end]):
            decline("unbalanced_span", text)
            continue
        if _is_assignment_target(masked, end):
            # `buf = ...` wants the ARRAY, not a pointer to it; decaying the assignment target
            # would compile to a discarded temporary in some shapes and is never the repair.
            decline("assignment_target", text)
            continue
        stripped = text.rstrip()
        if stripped.endswith(".as_ptr()") or stripped.endswith(".as_mut_ptr()"):
            decline("already_decayed", text)
            continue
        method = "as_mut_ptr" if want.group("mutability") == "mut" else "as_ptr"
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({text}).{method}()",
                rule=RULE_ARRAY_DECAY,
                authority=(
                    f"rustc label 'expected `{expected}`, found `{found}`': the element type "
                    f"`{element}` is IDENTICAL to the pointee, so this is C array-to-pointer "
                    f"decay and `{method}()` is its exact Rust spelling. No type name is named "
                    "and no width is baked, so the edit is target-independent"
                ),
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    cast_edits, cast_skipped, cast_claimed = _array_decay_cast(diagnostic, load, ctx)
    return edits + cast_edits, skipped + cast_skipped, claimed or cast_claimed


def _array_decay_cast(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0605 ``[T; N] as X`` — the same decay wearing C's explicit cast, e.g. ``(LPARAM)szBuf``.

    Two accepted targets, and the difference in how they are proven matters:

    * ``X`` is a raw pointer, of ANY pointee. Note the deliberate asymmetry with the argument
      shape above, which demands element identity: here a cast is WRITTEN IN THE SOURCE, so
      ``(BYTE *)szBuffer`` is the author stating the reinterpret, and reproducing it is
      faithfulness rather than guesswork. An implicit argument conversion carries no such
      statement, and there a pointee mismatch is evidence the DECLARATION is wrong — which is
      why it stays red there and is honoured here. Either way ``as`` cannot mean anything but
      the address: an array is not pointer-sized, so no value interpretation exists.
    * ``X`` is an integer that is POINTER-WIDE on the analysing target. This is C's
      pointer-to-integer cast (``LPARAM``/``WPARAM``/``LRESULT``), and here the edit is emitted
      with ``requires_target_agreement=True`` even though ``as_ptr`` bakes no width — because the
      CLAIM being made is "this integer is a pointer-width alias", and that claim is exactly what
      independent agreement across a 32- and a 64-bit target proves. A genuine byte-reinterpret
      like ``[u8; 4] as u32`` resolves to 4 bytes everywhere, so it is pointer-wide on i386 ONLY,
      fails to earn agreement, and is withheld instead of silently becoming an address.

    The replacement keeps the target type EXACTLY as the source spells it. rustc's message
    reports the resolved width (``i32``), and writing that back would bake i386's width into a
    site whose source correctly says ``LPARAM`` — the mistake this module exists to prevent.
    """
    match = _NON_PRIMITIVE_CAST.match(str(diagnostic.get("message") or ""))
    if match is None:
        return [], [], False
    array = _ARRAY_TYPE.match(match.group("found"))
    if array is None:
        return [], [], False
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    element = array.group("elem").strip()
    resolved = match.group("target").strip()
    for span in diagnostic.get("spans") or []:
        if not span.get("is_primary"):
            continue
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)

        def decline(reason: str, detail: str = "", line: int = line, name: str = file_name) -> None:
            skipped.append(Skipped(RULE_ARRAY_DECAY, reason, name, line, detail))

        if end <= start:
            decline("zero_length_span", resolved)
            continue
        data = load(file_name)
        if data is None:
            decline("unreadable_file")
            continue
        masked = _mask_lexical(data)
        if _in_macro_definition(masked, start):
            decline("macro_definition", data[start:end].decode("utf-8", "replace"))
            continue
        text = data[start:end].decode("utf-8", "replace")
        if not _balanced(masked[start:end]):
            decline("unbalanced_span", text)
            continue
        operator = _split_cast(masked, start, end)
        if operator is None:
            decline("no_cast_operator", text)
            continue
        expression = data[start:operator].decode("utf-8", "replace").rstrip()
        written = data[operator + 2 : end].decode("utf-8", "replace").strip()
        if not expression or not written:
            decline("unsplittable_cast", text)
            continue
        pointer = _RAW_POINTER.match(resolved)
        if pointer is not None:
            method = "as_mut_ptr" if pointer.group("mutability") == "mut" else "as_ptr"
            sensitive = False
            same = pointer.group("pointee").strip() == element
            authority = (
                f"rustc E0605 on `{match.group('found')} as {resolved}`: an array cast to a raw "
                "pointer can only mean its ADDRESS, so this is C array decay followed by the "
                "cast the source already writes"
                + (
                    ""
                    if same
                    else f"; the pointee `{pointer.group('pointee').strip()}` differs from the "
                    f"element `{element}`, which is the reinterpret the SOURCE CAST states "
                    "explicitly (C's `(T *)arr`) — not an inference of ours"
                )
            )
        elif resolved in INT_PRIMITIVES and (
            resolved in POINTER_SIZED or INT_WIDTH.get(resolved) == ctx.width
        ):
            method = "as_ptr"
            sensitive = resolved not in POINTER_SIZED
            authority = (
                f"rustc E0605 on `{match.group('found')} as {resolved}`: `{resolved}` is "
                f"pointer-wide on {ctx.target}, so `{written}` is a pointer-to-integer cast of a "
                "decayed array. The source spelling of the target type is preserved verbatim"
            )
        else:
            decline(
                "cast_target_not_pointer_wide",
                f"`{text}`: `{resolved}` is neither a matching pointer nor pointer-wide on "
                f"{ctx.target}. Decaying here would silently cast an ADDRESS where the code may "
                "have meant the contents — fix it by hand.",
            )
            continue
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({expression}).{method}() as {written}",
                rule=RULE_ARRAY_DECAY,
                authority=authority,
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                requires_target_agreement=sensitive,
                targets=(ctx.target,),
            )
        )
    return edits, skipped, True


_C_SUFFIX_MSG = re.compile(r"^invalid suffix `([uUlL]+)` for number literal$")


def _c_int_suffix_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C integer suffix (`8192ULL`) that reached Rust: the same value, Rust-spelled.

    The fact seal restored the C spelling verbatim before it learned not to; bodies banked
    before that fix still carry it. The suffix's type request is the surrounding code's
    business, exactly as for an unsuffixed literal, so dropping it is the whole repair.
    """
    message = str(diagnostic.get("message") or "")
    if not _C_SUFFIX_MSG.match(message):
        return [], [], False
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    for span in diagnostic.get("spans") or []:
        if not span.get("is_primary"):
            continue
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)
        data = load(file_name)
        text = data[start:end].decode("utf-8", "replace") if data is not None else ""
        fixed = rust_integer_spelling(text)
        if data is None or fixed == text or not re.fullmatch(r"(?:0[xXoO])?[0-9A-Fa-f]+", fixed):
            skipped.append(
                Skipped(RULE_C_INT_SUFFIX, "not_a_c_integer_literal", file_name, line, text)
            )
            continue
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=fixed,
                rule=RULE_C_INT_SUFFIX,
                authority="rustc rejects the C suffix; the value is unchanged without it",
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    return edits, skipped, True


_NULL_EXPR = re.compile(
    r"^(?:(?:core::)?ptr::)?null(?:_mut)?(?:::<[^()]*>)?\(\)(?:\s+as\s+[^;,()]+)?$"
    r"|^0\s+as\s+\*(?:mut|const)\s+[^;,()]+$"
)


def _null_fn_pointer_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A NULL C function pointer where bindgen types the slot `Option<fn ...>`: that is `None`.

    Only the literal null spellings qualify (`core::ptr::null_mut()`, `ptr::null()`,
    `0 as *mut _`); any other pointer handed to an `Option<fn>` slot is a real type question
    and stays RED.
    """
    texts = json.dumps(diagnostic.get("children") or [])
    if "Option<" not in texts or "fn(" not in texts:
        return [], [], False
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    claimed = False
    for span, expected, found in _mismatch_sites(diagnostic):
        if not expected.startswith("Option<") or not _RAW_POINTER.match(found):
            continue
        claimed = True
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)
        data = load(file_name)
        text = data[start:end].decode("utf-8", "replace").strip() if data is not None else ""
        if data is None or not _NULL_EXPR.match(text):
            skipped.append(
                Skipped(RULE_NULL_FN_POINTER, "not_a_null_literal", file_name, line, text)
            )
            continue
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement="None",
                rule=RULE_NULL_FN_POINTER,
                authority="rustc: the slot is Option<fn>, and the argument is a null pointer",
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    return edits, skipped, claimed


#: C integer types whose Rust primitive is the same on every target, so `bool as T` bakes no
#: pointer width (BOOLEAN, UCHAR, USHORT, ULONG, LONG, BOOL).
_FIXED_C_INTS = frozenset({"u8", "i8", "u16", "i16", "u32", "i32"})


def _outermost_call_site(span: Mapping[str, Any]) -> dict[str, Any]:
    """Follow rustc's macro-expansion chain to the invocation in the user's source."""
    seen = 0
    while span.get("expansion") and seen < 16:
        span = span["expansion"].get("span") or span
        seen += 1
    return dict(span)


def _is_whole_macro_arm(masked: bytes, start: int, end: int) -> bool:
    """True when ``[start, end)`` is the ENTIRE body of the `=> { ... }` arm around it.

    Then the macro's value IS that expression, and a C conversion belongs to each use, not
    to the definition: cmd's `_istspace!` is `iswctype(..)`, an int, used as a condition at
    one site and as a number elsewhere -- `!= 0` in the body fixed one and broke the rest.
    """
    arrow = masked.rfind(b"=>", 0, start)
    if arrow < 0:
        return False
    open_at = masked.find(b"{", arrow)
    if open_at < 0 or open_at > start:
        return False
    depth = 0
    for pos in range(open_at, len(masked)):
        byte = masked[pos : pos + 1]
        if byte == b"{":
            depth += 1
        elif byte == b"}":
            depth -= 1
            if depth == 0:
                return masked[open_at + 1 : pos].strip() == masked[start:end].strip()
    return False


def _c_truthiness_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """C truthiness, both directions, as C defines it.

    An integer where Rust wants `bool` is `x != 0`, and C's `!x` on an integer is `x == 0` --
    in Rust `!x` on a `u8` is a BITWISE not that still has type `u8`, so `if !Foo()` is a
    type error rather than a silent bug, and rustc blames the whole `!Foo()` (verified on
    1.92). setuplib had 37 such sites, all `BOOLEAN`/`BOOL` results used as conditions. The
    reverse, a comparison stored into a `BOOLEAN`, is `(cond) as u8`: C's 0/1. Only to
    fixed-width C integers, so no target's pointer width is baked in.
    """
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    claimed = False
    sites = list(_mismatch_sites(diagnostic))
    # An unsuffixed literal is labelled WITHOUT backticks ("expected `bool`, found integer"),
    # which `_mismatch_sites` does not parse. The usual shape is the C idiom of comparing a
    # BOOLEAN-returning call with 0 where the Rust callee already returns `bool`
    # (`IsListEmpty(..) == 0`): rustc blames the literal, and `0` is `false`.
    for span in diagnostic.get("spans") or []:
        if str(span.get("label") or "") == "expected `bool`, found integer":
            sites.append((dict(span), "bool", "integer-literal"))
    for span, expected, found in sites:
        if found == "integer-literal":
            claimed = True
            file_name = str(span.get("file_name") or "")
            start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
            data = load(file_name)
            text = data[start:end].decode("utf-8", "replace").strip() if data else ""
            lit = re.fullmatch(r"(0[xX][0-9a-fA-F_]+|[0-9][0-9_]*)", text)
            if lit is None:
                continue
            value = (
                int(lit.group(1).replace("_", ""), 0)
                if text.startswith(("0x", "0X"))
                else int(lit.group(1).replace("_", ""))
            )
            edits.append(
                Edit(
                    file_name=file_name,
                    byte_start=start,
                    byte_end=end,
                    replacement="true" if value else "false",
                    rule=RULE_C_TRUTHINESS,
                    authority="rustc: an integer literal where `bool` is expected; C's 0 is false",
                    original=text,
                    line=int(span.get("line_start") or 0),
                    column=int(span.get("column_start") or 0),
                    targets=(ctx.target,),
                )
            )
            continue
        # "integer" is rustc's word for an unsuffixed literal (`while 1`, `x = 0` into a flag).
        to_bool = expected == "bool" and (found in INT_PRIMITIVES or found == "integer")
        from_bool = found == "bool" and expected in _FIXED_C_INTS
        # A POINTER as a condition is C's `p != NULL` (atapi.h's ASSERT_REQUEST: `(Request)
        # && ...`), and `!p` is `p == NULL`.
        ptr_to_bool = expected == "bool" and found.startswith(("*mut ", "*const "))
        if not (to_bool or from_bool or ptr_to_bool):
            continue
        # A value a MACRO produced is blamed inside the macro's definition (core's
        # `matches!` for `IsRecognizedPartition!`); the edit belongs at the outermost call --
        # but only when that definition is not ours to fix. A span inside one of OUR
        # `macro_rules!` (setuplib's IS_VALID_INSTALL_PATH_CHAR: `isalnum(c) || ..`) is an
        # error in the macro body; wrapping the call site cannot fix it, and the next pass
        # wrapped it again: `((((x) != 0)) != false) != false ..` six deep.
        in_own_macro = False
        # The blamed tokens may be the macro's ARGUMENT, sitting inside the invocation in the
        # user's own file (`assert!(x as INT)`): converting the call's `()` result fixes
        # nothing -- comctl32's listview grew `!= 0)) != false))` around 34 asserts, one more
        # layer every pass, until the pass regressed and was thrown away. Convert the argument.
        site = _outermost_call_site(span) if span.get("expansion") else span
        argument = (
            span.get("expansion")
            and span.get("file_name") == site.get("file_name")
            and int(site.get("byte_start") or 0) <= int(span.get("byte_start") or 0)
            and int(span.get("byte_end") or 0) <= int(site.get("byte_end") or 0)
            and (int(span.get("byte_start") or 0), int(span.get("byte_end") or 0))
            != (int(site.get("byte_start") or 0), int(site.get("byte_end") or 0))
        )
        if argument:
            span = dict(span, expansion=None)
        if span.get("expansion") and from_bool:
            # A condition STORED into an integer is that use's conversion, never the macro's:
            # atapi.h's IS_ATAPI is a condition at 14 sites and a BOOLEAN at one, and
            # rewriting its body to `as u8` broke the 14.
            span = _outermost_call_site(span)
        elif span.get("expansion"):
            own = load(str(span.get("file_name") or ""))
            own_masked = _mask_lexical(own) if own is not None else b""
            span_at = (int(span.get("byte_start") or 0), int(span.get("byte_end") or 0))
            in_own_macro = (
                own is not None
                and _is_own(ctx, str(span.get("file_name") or ""))
                and _in_macro_definition(own_masked, span_at[0])
                and not _is_whole_macro_arm(own_masked, *span_at)
            )
            if not in_own_macro:
                span = _outermost_call_site(span)
        claimed = True
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)
        data = load(file_name)
        if data is None or end <= start:
            continue
        masked = _mask_lexical(data)
        # Inside one of OUR macro bodies the conversion is written there (it is the macro's
        # C semantics, e.g. `isalnum($c)` -> `!= 0`), never at a call site; any other macro
        # definition is not ours to touch.
        if (_in_macro_definition(masked, start) and not in_own_macro) or not _balanced(
            masked[start:end]
        ):
            skipped.append(Skipped(RULE_C_TRUTHINESS, "macro_or_unbalanced", file_name, line))
            continue
        text = data[start:end].decode("utf-8", "replace")
        # Idempotent: never wrap a conversion this rule already wrote.
        # Per direction: a `(x != 0)` the to-bool direction wrote is done, but the same text is
        # exactly what a from-bool site holds (atapi.h's IS_ATAPI: `(f & ATAPI) != 0` into a
        # BOOLEAN), and skipping it there left it red.
        done = (
            r"\(.*\) as (?:u8|i8|u16|i16|u32|i32)"
            if from_bool
            else r"\(\(.*\) (?:!=|==) 0\)|\(!?\(.*\)\.is_null\(\)\)"
        )
        if re.fullmatch(done, text, re.S):
            continue
        if ptr_to_bool:
            if text.startswith("!") and not text.startswith("!="):
                replacement = f"(({text[1:].strip()}).is_null())"
            else:
                replacement = f"(!({text}).is_null())"
        elif from_bool:
            replacement = f"({text}) as {expected}"
        elif text.startswith("!") and not text.startswith("!="):
            inner = text[1:].strip()
            replacement = f"(({inner}) == 0)"
        else:
            replacement = f"(({text}) != 0)"
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=replacement,
                rule=RULE_C_TRUTHINESS,
                authority=(
                    f"rustc: expected `{expected}`, found `{found}`; C converts an integer to a "
                    "condition as `!= 0` (and `!x` as `== 0`), and a condition to 0/1"
                ),
                original=text,
                line=line,
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    return edits, skipped, claimed


def _option_fn_call_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """C's call through a function pointer, where bindgen types the pointer `Option<fn>`.

    rustc E0618 "expected function, found `Option<.. fn ..>`" blames the CALLEE alone (verified
    on 1.92, both `g(2)` and `(s.f)(1)`), so it becomes `(callee.unwrap_unchecked())`. NOT
    `.unwrap()`: `assert_faithful` refuses any panic, and rightly -- calling a NULL function
    pointer is undefined in C, and `unwrap_unchecked` on `None` is undefined in Rust, the same
    contract with no new failure path.
    setuplib: 35 sites, SpInfExports' INF hooks and the USETUP progress callbacks.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0618":
        return [], [], False
    message = str(diagnostic.get("message") or "")
    # rustc abbreviates a long type to `Option<...>`, so `fn(` is often NOT in the message;
    # calling an Option of anything but a fn is an error the unwrap_unchecked leaves visible.
    if "found `Option<" not in message:
        return [], [], False
    span = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or end <= start:
        return [], [], False
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start) or not _balanced(masked[start:end]):
        return [], [], False
    text = data[start:end].decode("utf-8", "replace")
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({text}.unwrap_unchecked())",
                rule=RULE_OPTION_FN_CALL,
                authority="rustc E0618: the callee is an Option<fn>; C calls the pointer directly",
                original=text,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


#: Rust keywords bindgen renames with a trailing `_` when they name a C field.
_BINDGEN_KEYWORDS = frozenset(
    {
        "Self",
        "self",
        "type",
        "match",
        "fn",
        "impl",
        "mod",
        "move",
        "mut",
        "ref",
        "return",
        "static",
        "struct",
        "trait",
        "use",
        "where",
        "loop",
        "crate",
        "super",
        "box",
        "in",
        "as",
        "async",
        "await",
        "dyn",
    }
)
_RE_NO_FIELD_NAME = re.compile(r"no field `(?P<field>\w+)` on type")
_RE_NO_FIELD_NAMED = re.compile(r"has no field named `(?P<field>\w+)`")


_RE_C2RUST_ANON = re.compile(r"c2rust_unnamed(?:_\d+)?")


def _float_literal_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """An integer LITERAL where rustc wants `f32`/`f64`: `let xDpi: f32 = 0;`, `M_PI / 180`.
    C converts the constant to the floating type; `0.0` / `180.0` is that value exactly. The
    label carries no backticks around `integer` (an unsuffixed literal), and the span may be
    a secondary one under an "arguments to this method are incorrect" primary."""
    edits: list[Edit] = []
    message = str(diagnostic.get("message") or "")
    floaty = re.search(
        r"cannot (?:add|subtract|multiply|divide) `(f32|f64)` by `\{integer\}`", message
    )
    for span in diagnostic.get("spans") or []:
        label = str(span.get("label") or "")
        want = re.fullmatch(r"expected `(f32|f64)`, found integer", label)
        if want is None and not (floaty and span.get("is_primary")):
            continue
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        if floaty:
            # The primary is the operator; the literal is the right operand after it.
            m = re.match(rb"\s*([0-9][0-9_]*)\b(?![.\w])", data[end:])
            if m is None:
                continue
            start, end = end + m.start(1), end + m.end(1)
        text = data[start:end].decode("utf-8", "replace").strip()
        if not re.fullmatch(r"[0-9][0-9_]*", text):
            continue
        edits.append(
            _simple_edit(
                RULE_FLOAT_LITERAL,
                file_name,
                data,
                start,
                end,
                f"{text}.0",
                "rustc: an integer literal where a float is expected; C converts the constant",
                span,
                ctx,
            )
        )
    return edits, [], bool(edits)


def _object_deref_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0614 on a generated C++ CLASS: `(*toolsModel).GetBgBrush()` where mspaint's
    `toolsModel` is a global OBJECT the model took for a pointer. rustc has proved the operand
    is the class itself, so the object without `*` is what the C++ named. Only for
    `cpp_classes::` types: a smart pointer's `*` is a real operation and stays red."""
    if (diagnostic.get("code") or {}).get("code") != "E0614":
        return [], [], False
    m = re.search(r"type `cpp_classes::\w+` cannot be dereferenced", str(diagnostic.get("message")))
    got = _primary_bytes(diagnostic, load) if m else None
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    text = data[start:end].decode("utf-8", "replace")
    star = re.fullmatch(r"\*\s*([A-Za-z_][\w.]*)", text)
    if star is None:
        return [], [], False
    note = "rustc E0614: the operand is the class object itself, not a pointer to it"
    return (
        [
            _simple_edit(
                RULE_OBJECT_DEREF, file_name, data, start, end, star.group(1), note, span, ctx
            )
        ],
        [],
        True,
    )


def _guid_eq_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0369 `riid == IID_IUnknown` on a GUID: C++ compares with guiddef.h's
    `operator==(REFGUID, REFGUID)`, which is `IsEqualGUID` -- 16 bytes, memcmp. rustc names
    both operands (secondary spans) around the operator (primary)."""
    if (diagnostic.get("code") or {}).get("code") != "E0369":
        return [], [], False
    m = re.search(
        r"binary operation `(==|!=)` cannot be applied to type `(?:_GUID|GUID|IID|CLSID)`",
        str(diagnostic.get("message") or ""),
    )
    if m is None:
        return [], [], False
    spans = diagnostic.get("spans") or []
    ops = sorted(
        (s for s in spans if not s.get("is_primary")), key=lambda s: int(s.get("byte_start") or 0)
    )
    if len(ops) != 2 or ops[0].get("file_name") != ops[1].get("file_name"):
        return [], [], False
    file_name = str(ops[0].get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    a0, a1 = int(ops[0]["byte_start"]), int(ops[0]["byte_end"])
    b0, b1 = int(ops[1]["byte_start"]), int(ops[1]["byte_end"])
    between = data[a1:b0].decode("utf-8", "replace").strip()
    if between != m.group(1):
        return [], [], False
    lhs = data[a0:a1].decode("utf-8", "replace")
    rhs = data[b0:b1].decode("utf-8", "replace")
    neg = "!" if m.group(1) == "!=" else ""
    note = "rustc E0369: C++ compares GUIDs with guiddef.h's operator==, i.e. IsEqualGUID"
    return (
        [
            _simple_edit(
                RULE_OBJECT_DEREF,
                file_name,
                data,
                a0,
                b1,
                f"{neg}IsEqualGUID!(&({lhs}), &({rhs}))",
                note,
                ops[0],
                ctx,
            )
        ],
        [],
        True,
    )


def _heap_ptr_conversion_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`SetWindowTextW((*h).wszBtnText)` where the field is a `CComHeapPtr<WCHAR>`: ATL's
    `operator T*()` converts it implicitly; in Rust that is the held pointer, `m_pData`."""
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    label = str(span.get("label") or "")
    if not re.fullmatch(r"expected `\*(?:const|mut) [\w:]+`, found `CHeapPtr<[^`]*>`", label):
        return [], [], False
    text = data[start:end].decode("utf-8", "replace")
    if not re.fullmatch(r"[\w.()*\[\]]+", text):
        return [], [], False
    simple = re.fullmatch(r"[\w.]+|\([^()]*\)(?:\.\w+)*", text) is not None
    repl = f"{text}.m_pData" if simple else f"({text}).m_pData"
    note = "rustc: a CHeapPtr where a pointer is expected; ATL converts with operator T*()"
    return (
        [_simple_edit(RULE_OBJECT_DEREF, file_name, data, start, end, repl, note, span, ctx)],
        [],
        True,
    )


def _atl_format_id_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`atl_format!(s, IDS_FILESIZE, ..)`: the format argument is an INTEGER, i.e. C++
    called `CString::Format(UINT nFormatID, ...)`, the string-table overload. That is
    atlclasses' `atl_format_id!`; rustc's `expected *const u16, found u32` proves which."""
    if (diagnostic.get("code") or {}).get("code") != "E0308":
        return [], [], False
    edits: list[Edit] = []
    for span, expected, found in _mismatch_sites(diagnostic):
        if expected.replace(" ", "") != "*constu16" or found not in INT_PRIMITIVES | {"integer"}:
            continue
        outer = span
        while isinstance(outer.get("expansion"), Mapping):
            outer = outer["expansion"].get("span") or {}
        if outer is not span:
            # atl_format! binds each argument through a `match`, so rustc reports the
            # mismatch at the macro's own `$fmt` use, inside atlclasses-rust. The site to
            # edit is the outermost invocation in the module: `atl_format!(s, IDS_X, ..)`.
            file_name = str(outer.get("file_name") or "")
            data = load(file_name)
            name_at = int(outer.get("byte_start") or 0)
            if data is None or not data[name_at:].startswith(b"atl_format!("):
                continue
            edits.append(
                _simple_edit(
                    RULE_OBJECT_DEREF,
                    file_name,
                    data,
                    name_at,
                    name_at + len(b"atl_format!"),
                    "atl_format_id!",
                    "the format argument is a string-table id: "
                    "CString::Format(UINT nFormatID, ...)",
                    dict(outer),
                    ctx,
                )
            )
            continue
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        start = int(span.get("byte_start") or 0)
        # Same length, so offsets hold: `_enclosing_call` names identifiers, not `name!`.
        masked = _mask_lexical(data).replace(b"atl_format!(", b"atl_format_(")
        callee, arg_index = _enclosing_call(masked, start)
        if callee != "atl_format_" or arg_index != 1:
            continue
        name_at = data.rfind(b"atl_format!", 0, start)
        if name_at < 0:
            continue
        edits.append(
            _simple_edit(
                RULE_OBJECT_DEREF,
                file_name,
                data,
                name_at,
                name_at + len(b"atl_format!"),
                "atl_format_id!",
                "the format argument is a string-table id: CString::Format(UINT nFormatID, ...)",
                span,
                ctx,
            )
        )
    return edits, [], bool(edits)


def _cstring_conversion_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """ATL CStringW's implicit conversions, written out: a CStringW where `*const u16` is
    wanted is `operator PCXSTR()` -> `.GetString()`; a `*const u16` where a CStringW is
    wanted (an assignment) is `operator=(PCXSTR)` -> `CStringW::from_ptr(..)`."""
    if (diagnostic.get("code") or {}).get("code") != "E0308":
        return [], [], False
    edits: list[Edit] = []
    for span, expected, found in _mismatch_sites(diagnostic):
        exp, fnd = expected.replace(" ", ""), found.replace(" ", "")
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        text = data[start:end].decode("utf-8", "replace").strip()
        if not text or not _balanced(_mask_lexical(text.encode())):
            continue
        if exp == "*constu16" and fnd.split("::")[-1] in ("CStringW", "&CStringW", "&mutCStringW"):
            rep = f"{text}.GetString()" if _PLACE.fullmatch(text) else f"({text}).GetString()"
            why = "CStringW's implicit operator PCXSTR"
        elif exp.split("::")[-1] == "CStringW" and fnd in ("*constu16", "*mutu16"):
            rep = f"CStringW::from_ptr({text})"
            why = "CStringW's operator=(PCXSTR) / converting constructor"
        else:
            continue
        edits.append(
            _simple_edit(RULE_OBJECT_DEREF, file_name, data, start, end, rep, why, span, ctx)
        )
    return edits, [], bool(edits)


def _ref_compound_assign_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0368 `strFilter += x` where `strFilter: &mut CStringW` (a C++ `CString&` parameter):
    the operator is the referent's, `*strFilter += x`."""
    if (diagnostic.get("code") or {}).get("code") != "E0368":
        return [], [], False
    if not re.search(r"cannot be applied to type `&mut ", str(diagnostic.get("message") or "")):
        return [], [], False
    edits: list[Edit] = []
    for span in diagnostic.get("spans") or []:
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None or span.get("is_primary"):
            continue
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        text = data[start:end].decode("utf-8", "replace")
        if not re.fullmatch(r"[A-Za-z_]\w*", text) or not re.match(rb"\s*[-+*/|&^]?=", data[end:]):
            continue
        note = "the compound assignment is the referent's operator (a C++ reference)"
        edits.append(
            _simple_edit(
                RULE_OBJECT_DEREF, file_name, data, start, end, f"*{text}", note, span, ctx
            )
        )
    return edits, [], bool(edits)


def _zero_constant_pointer_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A constant whose value is 0 where a pointer is wanted: CRegKey::Create's ATL default
    `LPTSTR lpszClass = REG_NONE` -- C++ takes an integral constant 0 as the null pointer."""
    if (diagnostic.get("code") or {}).get("code") != "E0308" or not ctx.constants:
        return [], [], False
    edits: list[Edit] = []
    for span, expected, found in _mismatch_sites(diagnostic):
        ptr = re.fullmatch(r"\*(mut|const) .+", expected.strip())
        if ptr is None or found not in INT_PRIMITIVES:
            continue
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        name = data[start:end].decode("utf-8", "replace").strip()
        if ctx.constants.get(name) != 0:
            continue
        null = "core::ptr::null_mut()" if ptr.group(1) == "mut" else "core::ptr::null()"
        note = f"`{name}` is the constant 0, C++'s null pointer constant where a pointer is wanted"
        edits.append(
            _simple_edit(RULE_OBJECT_DEREF, file_name, data, start, end, null, note, span, ctx)
        )
    return edits, [], bool(edits)


_FLOAT_OP = re.compile(
    r"cannot (add|subtract|multiply|divide) `[iu](?:8|16|32|64|size)` "
    r"(?:by|to|from) `(f32|f64)`"
)


def _int_float_arith_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`(*this_).newWidth / g_xDpi`, int / float: C converts the integer operand to the float
    type (the usual arithmetic conversions). The primary span is the operator; the integer is
    the operand before it."""
    m = _FLOAT_OP.search(str(diagnostic.get("message") or ""))
    got = _primary_bytes(diagnostic, load) if m else None
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    assert m is not None  # `got` is None without it
    if data[start:end].strip() not in (b"/", b"*", b"+", b"-"):
        return [], [], False
    op = start
    while op > 0 and data[op - 1 : op] in b" \t":
        op -= 1
    # The operand ends where the operator begins: the same backward chain scan as a receiver.
    left = _receiver_start(data, op) if op > 0 else None
    if left is None:
        return [], [], False
    text = data[left:op].decode("utf-8", "replace").strip()
    if not text or not _balanced(_mask_lexical(text.encode())):
        return [], [], False
    note = f"C converts the integer operand to `{m.group(2)}` (usual arithmetic conversions)"
    return (
        [
            _simple_edit(
                RULE_OBJECT_DEREF,
                file_name,
                data,
                left,
                op,
                f"({text} as {m.group(2)})",
                note,
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _merge_rules(diagnostic, load, ctx, *rules):
    edits, skipped, claimed = [], [], False
    for rule in rules:
        e, sk, c = rule(diagnostic, load, ctx)
        edits += e
        skipped += sk
        claimed = claimed or c
    return edits, skipped, claimed


def _raw_receiver_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0599 a method called on a RAW POINTER to a generated C++ class: C++'s `_shared()`
    returned `SHARED&`, the Rust returns `*mut SHARED`, and `.Release()` is SHARED's.
    Calling it through the pointee, `(*CImageDx::_shared()).Release()`, is the C++ call."""
    if (diagnostic.get("code") or {}).get("code") != "E0599":
        return [], [], False
    m = re.search(
        r"no method named `(\w+)` found for raw pointer `\*(?:mut|const) cpp_classes::\w+`",
        str(diagnostic.get("message") or ""),
    )
    got = _primary_bytes(diagnostic, load) if m else None
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    assert m is not None  # `got` is None without it
    if data[start:end].decode("utf-8", "replace") != m.group(1):
        return [], [], False
    dot = data.rfind(b".", 0, start)
    if dot < 0 or data[dot + 1 : start].strip():
        return [], [], False
    recv = _receiver_start(data, dot)
    if recv is None:
        return [], [], False
    text = data[recv:dot].decode("utf-8", "replace").strip()
    note = "rustc E0599: the method is the pointee class's; C++ called it on the reference"
    return (
        [
            _simple_edit(
                RULE_OBJECT_DEREF, file_name, data, recv, dot, f"(*{text})", note, span, ctx
            )
        ],
        [],
        True,
    )


def _hwnd_to_cwindow_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`m_ctl = GetDlgItem(id)` / `CWindow w = GetDlgItem(id)`: C++ converts the HWND with
    `CWindow(HWND)` / `operator=(HWND)`, both of which only set `m_hWnd`. cppapp generates
    that constructor as `CWindow__make_0`."""
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    if str(span.get("label") or "") != "expected `CWindow`, found `*mut HWND__`":
        return [], [], False
    text = data[start:end].decode("utf-8", "replace")
    note = "rustc: an HWND where a CWindow is expected; C++ converts with CWindow(HWND)"
    repl = f"CWindow__make_0({text})"
    return (
        [_simple_edit(RULE_OBJECT_DEREF, file_name, data, start, end, repl, note, span, ctx)],
        [],
        True,
    )


def _com_ptr_receiver_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0599 an interface method called on a `CComPtr<IFoo>`: C++'s `sp->SetTitle(..)` goes
    through `operator->` to the interface, whose methods (one per vtable slot, generated by
    rustconv.com_vtables) are on `IFoo`. CComPtr deliberately has no Deref to the interface,
    so the call is written out: `(*sp.p).SetTitle(..)`."""
    if (diagnostic.get("code") or {}).get("code") != "E0599":
        return [], [], False
    m = re.search(
        r"no method named `(\w+)` found for struct `(?:atl_rust::)?CComPtr<",
        str(diagnostic.get("message") or ""),
    )
    got = _primary_bytes(diagnostic, load) if m else None
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    assert m is not None  # `got` is None without it
    if data[start:end].decode("utf-8", "replace") != m.group(1):
        return [], [], False
    dot = data.rfind(b".", 0, start)
    if dot < 0 or data[dot + 1 : start].strip():
        return [], [], False
    recv = _receiver_start(data, dot)
    if recv is None:
        return [], [], False
    text = data[recv:dot].decode("utf-8", "replace").strip()
    note = "rustc E0599: CComPtr's operator-> yields the interface, whose method this is"
    return (
        [
            _simple_edit(
                RULE_OBJECT_DEREF, file_name, data, recv, dot, f"(*{text}.p)", note, span, ctx
            )
        ],
        [],
        True,
    )


def _c2rust_anon_turbofish(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`null_mut::<C2RustUnnamed_11>()`: c2rust's name for a typedef'd anonymous struct, copied
    from the anchor into a null pointer's turbofish (format's and autochk's `PTEXTOUTPUT
    output = NULL`). The binding's own type says what the pointer points at, so the turbofish
    goes and inference reads it from there."""
    span = _primary(diagnostic)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    if data is None or not re.fullmatch(rb"C2RustUnnamed(?:_\d+)?", data[start:end]):
        return [], [], False
    lo = data.rfind(b"::<", 0, start)
    if lo < 0 or data[lo + 3 : start].strip() or data[end:].lstrip()[:3] != b">()":
        return [], [], False
    if not re.search(rb"\b(?:null|null_mut)\s*$", data[max(0, lo - 12) : lo]):
        return [], [], False
    close = data.index(b">", end) + 1
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=lo,
                byte_end=close,
                replacement="",
                rule=RULE_C2RUST_ANON,
                authority="c2rust's name for an anonymous typedef'd struct; the binding's type "
                "is the pointee",
                original=data[lo:close].decode("utf-8", "replace"),
                line=int(span.get("line_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _c2rust_anon_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`.c2rust_unnamed` / `.c2rust_unnamed_0`: c2rust's name for an anonymous member -> gone.

    The model copies its c2rust anchor's spelling of C's anonymous struct/union members
    (ndis: `Chars.c2rust_unnamed.c2rust_unnamed_0.SendCompleteHandler`, `IoStatus.
    c2rust_unnamed.__bindgen_anon_1.Status`). Bindgen names them `__bindgen_anon_N`, with its
    own numbering, so the segment is DELETED, leaving C's flattened spelling -- which
    RULE_ANON_UNION then routes from the island's own declarations, or which compiles as is.
    """
    if (diagnostic.get("code") or {}).get("code") == "E0412":
        return _c2rust_anon_turbofish(diagnostic, load, ctx)
    if (diagnostic.get("code") or {}).get("code") != "E0609":
        return [], [], False
    m = _RE_NO_FIELD_NAME.search(str(diagnostic.get("message") or ""))
    if m is None or not _RE_C2RUST_ANON.fullmatch(m.group("field")):
        return [], [], False
    span = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or data[start:end].decode("utf-8", "replace") != m.group("field"):
        return [], [], False
    dot = start
    while dot > 0 and data[dot - 1 : dot] in (b" ", b"\t", b"\n", b"\r"):
        dot -= 1
    if data[dot - 1 : dot] != b".":
        return [], [], False
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=dot - 1,
                byte_end=end,
                replacement="",
                rule=RULE_C2RUST_ANON,
                authority="c2rust's name for an anonymous member; C injects the member's fields",
                original=data[dot - 1 : end].decode("utf-8", "replace"),
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _keyword_field_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C field named like a Rust keyword: `.Self` (or c2rust's `.Self_0`) -> bindgen's `.Self_`.

    atapi's ATAPORT_COMMON_EXTENSION has a `PDEVICE_OBJECT Self;`, which bindgen emits as
    `Self_`; the model spelled the C name (26 sites) or c2rust's (5). Only a keyword is renamed,
    and to bindgen's one spelling, so a field that really is missing stays an error.
    """
    # E0609 is a field ACCESS (`.type_0`); E0560 the same name in a struct LITERAL
    # (`reg_type_rels { type_0: REG_SZ, .. }`, reg.exe's type table).
    code = (diagnostic.get("code") or {}).get("code")
    if code not in ("E0609", "E0560"):
        return [], [], False
    msg = str(diagnostic.get("message") or "")
    m = _RE_NO_FIELD_NAME.search(msg) or _RE_NO_FIELD_NAMED.search(msg)
    if m is None:
        return [], [], False
    field_name = m.group("field")
    base = field_name[:-2] if field_name.endswith("_0") else field_name
    if base not in _BINDGEN_KEYWORDS:
        return [], [], False
    span = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or data[start:end].decode("utf-8", "replace") != field_name:
        return [], [], False
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"{base}_",
                rule=RULE_KEYWORD_FIELD,
                authority="bindgen renames a keyword-named C field with a trailing underscore",
                original=field_name,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


_RE_C2RUST_ANON = re.compile(r"c2rust_unnamed(?:_\d+)?")


def _c2rust_hop_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`(*pIC).c2rust_unnamed.hWnd` on a type with no such field -> `(*pIC).hWnd`.

    c2rust names an anonymous member `c2rust_unnamed[_N]` and an anchored translation copies
    it. Islands use that spelling only where bindgen_unions_to_real converted a union; an MS
    embedded base struct (imm32_undoc.h's `INPUTCONTEXTDX { INPUTCONTEXT; }`) is restored as
    `__bindgen_anon_1`, so imm32 carried 61 hops onto a field that does not exist. rustc names
    the type that lacks it; dropping the hop leaves the C's own spelling, which
    anon_union_field then routes on the next pass -- or which stays an error if C would not
    reach it either.
    """
    if (diagnostic.get("code") or {}).get("code") == "E0412":
        return _c2rust_anon_turbofish(diagnostic, load, ctx)
    if (diagnostic.get("code") or {}).get("code") != "E0609":
        return [], [], False
    m = _RE_NO_FIELD_NAME.search(str(diagnostic.get("message") or ""))
    if m is None or not _RE_C2RUST_ANON.fullmatch(m.group("field")):
        return [], [], False
    span = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if (
        data is None
        or start < 1
        or data[start - 1 : start] != b"."
        or data[end : end + 1] != b"."
        or data[start:end].decode("utf-8", "replace") != m.group("field")
    ):
        return [], [], False
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start - 1,
                byte_end=end,
                replacement="",
                rule=RULE_C2RUST_HOP,
                authority="rustc: the receiver type has no c2rust anonymous-member field",
                original=f".{m.group('field')}",
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


_RE_METHOD_NOT_FIELD = re.compile(r"attempted to take value of method `(?P<name>\w+)`")
_RE_ASSIGN_OP = re.compile(rb"\s*(?P<op>\|=|&=|\^=|\+=|-=|<<=|>>=|=(?!=))")


def _statement_end(data: bytes, start: int) -> int | None:
    """Offset of the `;` ending the statement that continues at `start` (depth 0)."""
    depth = 0
    for k in range(start, len(data)):
        c = data[k]
        if c in b"([{":
            depth += 1
        elif c in b")]}":
            depth -= 1
            if depth < 0:
                return None
        elif c == ord(";") and depth == 0:
            return k
    return None


def _bitfield_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C bitfield used as a field: bindgen emits it as a getter `F()` and a setter `set_F(v)`.

    atapi had 101 of these (`(*InquiryData).DeviceType = ...`, `Caps.Removable`). A read
    becomes the getter call; a plain or compound assignment statement becomes the setter, its
    value cast with `as _` to the setter's parameter type -- exactly what C's store into a
    bitfield does (the setter masks to the field width, as C truncates). Anything else (a
    reference to the field, an assignment used as an expression) is left red.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0615":
        return [], [], False
    m = _RE_METHOD_NOT_FIELD.search(str(diagnostic.get("message") or ""))
    span = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    if m is None or span is None:
        return [], [], False
    name = m.group("name")
    file_name = str(span.get("file_name") or "")
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    data = load(file_name)
    if data is None or data[start:end].decode("utf-8", "replace") != name:
        return [], [], False
    authority = "bindgen emits a C bitfield as a getter F() and a setter set_F()"
    line, column = int(span.get("line_start") or 0), int(span.get("column_start") or 0)
    op = _RE_ASSIGN_OP.match(data, end)
    before = data[:start].rstrip()
    if op is None or (start > 0 and data[start - 1 : start] != b"."):
        if data[end : end + 1] == b"(" or before.endswith((b"&", b"&mut", b"addr_of!(")):
            return [], [], False
        edit = Edit(
            file_name,
            start,
            end,
            f"{name}()",
            RULE_BITFIELD,
            authority,
            name,
            line,
            column,
            targets=(ctx.target,),
        )
        return [edit], [], True
    recv_start = _receiver_start(data, start - 1)
    stmt_end = _statement_end(data, op.end())
    # Only a whole statement: the receiver must begin the line (after indentation).
    if (
        recv_start is None
        or stmt_end is None
        or data[:recv_start].rstrip(b" \t")[-1:] not in (b"\n", b"{", b";", b"}", b"")
    ):
        return [], [], False
    recv = data[recv_start : start - 1].decode("utf-8", "replace")
    rhs = data[op.end() : stmt_end].decode("utf-8", "replace").strip()
    o = op.group("op").decode()
    value = rhs if o == "=" else f"{recv}.{name}() {o[:-1]} ({rhs})"
    original = data[recv_start:stmt_end].decode("utf-8", "replace")
    edit = Edit(
        file_name,
        recv_start,
        stmt_end,
        f"{recv}.set_{name}(({value}) as _)",
        RULE_BITFIELD,
        authority,
        original,
        line,
        column,
        targets=(ctx.target,),
    )
    return [edit], [], True


def _uninit_local_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C local declared without an initialiser: `let x: T;` -> `= core::mem::zeroed();`.

    C lets a local be read, or filled field by field (`InitializeObjectAttributes`, an
    `IO_STATUS_BLOCK` handed to a syscall to fill), without ever being initialised; rustc's
    E0381 refuses. Reading it in C is undefined, so a defined zero is no change to any
    defined behaviour. The declaration comes from rustc's own secondary span ("binding
    declared here but left uninitialized"); one without a TYPE is left RED, because
    `zeroed()` needs one. setuplib: 27 sites.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0381":
        return [], [], False
    decl = next(
        (
            s
            for s in diagnostic.get("spans") or []
            if "declared here but left uninitialized" in str(s.get("label") or "")
        ),
        None,
    )
    if decl is None:
        return [], [], False
    file_name = str(decl.get("file_name") or "")
    line = int(decl.get("line_start") or 0)
    data = load(file_name)
    if data is None:
        return [], [], False
    masked = _mask_lexical(data)
    i, depth = int(decl.get("byte_end") or 0), 0
    colon = -1
    while i < len(masked):
        c = masked[i : i + 1]
        if c in b"([{":
            depth += 1
        elif c in b")]}":
            depth -= 1
        elif depth == 0 and c == b":" and colon < 0:
            colon = i
        elif depth == 0 and c in b"=;":
            break
        i += 1
    if i >= len(masked) or masked[i : i + 1] != b";" or colon < 0:
        return (
            [],
            [Skipped(RULE_UNINIT_LOCAL, "untyped_or_initialised", file_name, line)],
            True,
        )
    original = data[i : i + 1].decode()
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=i,
                byte_end=i + 1,
                replacement=" = core::mem::zeroed();",
                rule=RULE_UNINIT_LOCAL,
                authority="rustc E0381: a C local read or field-filled before any initialiser",
                original=original,
                line=line,
                column=int(decl.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _mut_binding_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """C parameters and locals are assignable; Rust bindings are not unless `mut`.

    rustc's own MachineApplicable child suggestion for E0384 (assign twice / assign to an
    immutable argument) and E0596 (borrow as mutable) is the zero-width insertion `mut `,
    which is exactly C's semantics. Several errors share one binding, so each insertion
    point is claimed once.
    """
    code = (diagnostic.get("code") or {}).get("code")
    if code not in ("E0384", "E0596"):
        return [], [], False
    edits: list[Edit] = []
    for child in diagnostic.get("children") or []:
        for span in child.get("spans") or []:
            if span.get("suggested_replacement") != "mut " or (
                span.get("suggestion_applicability") != "MachineApplicable"
            ):
                continue
            file_name = str(span.get("file_name") or "")
            at = int(span.get("byte_start") or 0)
            if int(span.get("byte_end") or 0) != at or (file_name, at, at) in ctx.claimed:
                continue
            # E0596 through a `*const T` parameter offers the same `mut ` right after its `*`:
            # `*mut const T` does not parse, and C said `const`. That is a type change, not
            # a binding one -- comctl32's TRACKBAR_DrawChannel took the whole pass down.
            data = load(file_name)
            if data is None or data[at : at + 5] == b"const":
                continue
            ctx.claimed.add((file_name, at, at))
            edits.append(
                Edit(
                    file_name=file_name,
                    byte_start=at,
                    byte_end=at,
                    replacement="mut ",
                    rule=RULE_MUT_BINDING,
                    authority=f"rustc {code} MachineApplicable: C bindings are assignable",
                    original="",
                    line=int(span.get("line_start") or 0),
                    column=int(span.get("column_start") or 0),
                    targets=(ctx.target,),
                )
            )
    return edits, [], bool(edits)


def _anon_literal_rewrite(
    span: Mapping[str, Any], load: FileReader, ctx: _Ctx, struct: str
) -> Edit | None:
    """`T { Status: a, Information: b }` -> a zeroed `T` with each field stored by its path.

    A Rust struct literal cannot name a field that lives in an anonymous union
    (`__bindgen_anon_1.Status`), so the literal becomes a block: `{ let mut __lit: T =
    core::mem::zeroed(); __lit.__bindgen_anon_1.Status = a; __lit.Information = b; __lit }`.
    That is C's semantics exactly -- an aggregate initialiser zero-fills what it does not
    name. Declines (None) on anything but plain `field: expr` / shorthand entries, a
    functional-update `..base`, or an unbalanced span; the earlier hand-rolled rewrite turned
    every site into a parse error, which `count_halting_errors` now also catches.
    """
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return None
    masked = _mask_lexical(data)
    pos = int(span.get("byte_start") or 0)
    depth, i = 0, pos - 1
    while i >= 0:
        c = masked[i : i + 1]
        if c in b")]}":
            depth += 1
        elif c in b"([{":
            if depth == 0:
                break
            depth -= 1
        i -= 1
    if i < 0 or masked[i : i + 1] != b"{":
        return None
    open_brace = i
    head = re.search(rb"([A-Za-z_][\w:]*)\s*$", masked[max(0, open_brace - 200) : open_brace])
    if head is None:
        return None
    ty_start = open_brace - (len(masked[max(0, open_brace - 200) : open_brace]) - head.start())
    ty = data[ty_start : ty_start + len(head.group(1))].decode()
    close, depth = None, 0
    for k in range(open_brace, len(masked)):
        c = masked[k : k + 1]
        if c in b"([{":
            depth += 1
        elif c in b")]}":
            depth -= 1
            if depth == 0:
                close = k
                break
    if close is None or (file_name, ty_start, close + 1) in ctx.claimed:
        return None
    body = data[open_brace + 1 : close].decode("utf-8", "replace")
    stores = []
    for part in _split_top_level_commas(body):
        part = part.strip()
        if not part:
            continue
        if part.startswith(".."):
            return None
        name, sep, value = part.partition(":")
        name = name.strip()
        if not re.fullmatch(r"[A-Za-z_]\w*", name):
            return None
        value = value.strip() if sep else name
        path = ctx.anon_unions.get((struct, name), name)
        stores.append(f"__lit.{path} = {value};")
    ctx.claimed.add((file_name, ty_start, close + 1))
    original = data[ty_start : close + 1].decode("utf-8", "replace")
    return Edit(
        file_name=file_name,
        byte_start=ty_start,
        byte_end=close + 1,
        replacement="{ let mut __lit: "
        + ty
        + " = core::mem::zeroed(); "
        + " ".join(stores)
        + " __lit }",
        rule=RULE_ANON_UNION,
        authority=f"`{struct}` literal names an anonymous-union field; C zero-fills the rest",
        original=original,
        line=int(span.get("line_start") or 0),
        column=int(span.get("column_start") or 0),
        targets=(ctx.target,),
    )


def _split_top_level_commas(text: str) -> list[str]:
    parts, depth, cur = [], 0, []
    for ch in text:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
        else:
            cur.append(ch)
    parts.append("".join(cur))
    return parts


def _int_width(ty: str, pointer: int) -> int:
    return pointer if ty in POINTER_SIZED else INT_WIDTH[ty]


def _infer_widen_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """An integer WIDENED into its context -> `(x) as _`, which each target infers for itself.

    `IoStatusBlock.Information == FILE_CREATED` (ULONG_PTR vs a u32 constant), `SIZE_T n =
    wcslen(..)`: the context's type is pointer-width, so it differs per arch, and a site that
    errors on amd64 compiles on i386. A width-baked `as u64` is rightly withheld for that
    reason. But when the value only WIDENS -- same signedness and no wider than the context,
    or unsigned into a strictly wider signed type -- the conversion is exactly C's integer
    promotion on every target, and `as _` lets rustc pick each target's type. A narrowing
    or sign change stays with the other rules (that is C truncation, and it needs a name).
    """
    edits: list[Edit] = []
    for span, expected, found in _mismatch_sites(diagnostic):
        if expected not in INT_PRIMITIVES or found not in INT_PRIMITIVES or expected == found:
            continue
        we, wf = _int_width(expected, ctx.width), _int_width(found, ctx.width)
        se, sf = not expected.startswith("u"), not found.startswith("u")
        widen = (se == sf and wf <= we) or (not sf and se and wf < we)
        if not widen:
            continue
        file_name = str(span.get("file_name") or "")
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        if (file_name, start, end) in ctx.claimed or end <= start:
            continue
        data = load(file_name)
        if data is None:
            continue
        masked = _mask_lexical(data)
        if _in_macro_definition(masked, start) or not _balanced(masked[start:end]):
            continue
        # A match-arm PATTERN cannot hold `(X) as _`: pciidex's intel.rs lost every arm of a
        # `match (*Info).Type` to this rule, and the parse errors reverted the whole pass.
        if _in_match_pattern(masked, end):
            continue
        if _next_code_byte(masked, end) == b"(":
            continue  # a callee, not a value
        text = data[start:end].decode("utf-8", "replace")
        ctx.claimed.add((file_name, start, end))
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({text}) as _",
                rule=RULE_INFER_WIDEN,
                authority=(
                    f"rustc: `{found}` into `{expected}` on {ctx.target} only widens; `as _` is "
                    "C's promotion on every target, with no width baked in"
                ),
                original=text,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    return edits, [], bool(edits)


def _no_std_path_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    message = str(diagnostic.get("message") or "")
    if _STD_UNRESOLVED not in message and message != "unresolved import `std`":
        return [], [], False
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    for span in diagnostic.get("spans") or []:
        if not span.get("is_primary"):
            continue
        file_name = str(span.get("file_name") or "")
        line = int(span.get("line_start") or 0)
        start = int(span.get("byte_start") or 0)
        end = int(span.get("byte_end") or 0)

        def decline(reason: str, detail: str = "", line: int = line, name: str = file_name) -> None:
            skipped.append(Skipped(RULE_NO_STD_PATH, reason, name, line, detail))

        data = load(file_name)
        if data is None:
            decline("unreadable_file")
            continue
        if data[start:end] != b"std":
            decline("span_is_not_std", data[start:end].decode("utf-8", "replace"))
            continue
        following = re.compile(rb"\s*::\s*([A-Za-z_]\w*)").match(data, end)
        segment = following.group(1).decode() if following else ""
        if segment not in CORE_MODULES:
            # `std::collections`, `std::vec`, ... live in alloc/std only: rewriting to `core`
            # would be a guess. Report it and leave the code untouched.
            decline("std_path_not_in_core", f"std::{segment or '?'}")
            continue
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement="core",
                rule=RULE_NO_STD_PATH,
                authority=(
                    f"rustc reports `std` unresolved (crate is no_std) and `{segment}` is a "
                    "module that exists in `core`, so `core` is the only resolution"
                ),
                original="std",
                line=line,
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        )
    return edits, skipped, True


def _enclosing_match_scrutinee(masked: bytes, start: int) -> tuple[int, int] | None:
    """Byte range of the scrutinee of the ``match`` whose ARM contains ``start``.

    Walks out to the nearest UNCLOSED ``{`` — the brace that opens the arm block — then back
    over the scrutinee to a ``match`` keyword at depth 0. Returns ``None`` unless BOTH are
    found, so a span in some other braced construct (a struct literal arm body, a nested block)
    is never mistaken for an arm.
    """
    depth, index, brace = 0, start - 1, None
    while index >= 0:
        char = masked[index : index + 1]
        if char in b")]}":
            depth += 1
        elif char in b"([":
            depth -= 1
        elif char == b"{":
            if depth == 0:
                brace = index
                break
            depth -= 1
        index -= 1
    if brace is None:
        return None
    depth, index = 0, brace - 1
    while index >= 0:
        char = masked[index : index + 1]
        if char in b")]}":
            depth += 1
        elif char in b"([{":
            depth -= 1
            if depth < 0:
                return None  # the scrutinee cannot span out of its own enclosing group
        elif depth == 0 and char == b"h" and masked[index - 4 : index + 1] == b"match":
            keyword = index - 4
            before = masked[keyword - 1 : keyword]
            after = masked[index + 1 : index + 2]
            if (before and (before.isalnum() or before == b"_")) or after.isalnum():
                index -= 1
                continue
            body = masked[index + 1 : brace]
            if not body.strip():
                return None
            lead = len(body) - len(body.lstrip())
            return index + 1 + lead, brace - (len(body) - len(body.rstrip()))
        elif depth == 0 and char in b";":
            return None
        index -= 1
    return None


def _bare_arm_pattern(data: bytes, start: int, end: int) -> tuple[int, int, str] | None:
    """The whole arm pattern around the constant at ``start:end`` when it is bare constant
    paths joined by ``|`` (``WMSZ_TOP | WMSZ_BOTTOM =>``), as ``(start, end, text)``."""
    path = rb"[A-Za-z_][\w:]*"
    head = re.search(rb"(?:" + path + rb"\s*\|\s*)*\Z", data[max(0, start - 400) : start])
    tail = re.match(rb"(?:\s*\|\s*" + path + rb")*(?=\s*=>)", data[end:])
    if head is None or tail is None or not re.fullmatch(path, data[start:end]):
        return None
    p_start, p_end = start - len(head.group(0)), end + tail.end()
    if not re.search(rb"[{,}]\s*\Z", data[:p_start]):
        return None  # a `|` inside a larger pattern, or not an arm at all
    return p_start, p_end, data[p_start:p_end].decode("utf-8", "replace")


def _match_scrutinee_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """int-vs-int on a match ARM -> cast the SCRUTINEE, never the pattern.

    ``_in_match_pattern`` declines these for ``RULE_NUMERIC_CAST``, and is right to: an
    expression is not legal in a pattern, and rewriting the CONSTANT in an arm is the §9.112
    shape — an arm whose name stops resolving becomes a fresh binding that matches EVERYTHING,
    silently. It was also by far the largest decline bucket on the modules that motivated this
    (293 sites on desk.cpl alone), so leaving it unanswered means paying an LLM for it.

    The scrutinee is the one edit that fixes every arm at once and cannot change which constant
    an arm compares against. It is exactly what C's ``switch`` does implicitly: ``match
    LOWORD!(wParam)`` yields a WORD and the arms are the island's UINT ``IDC_*``; ``match
    (*lpnm).code`` yields a UINT and the arms are prsht.h's NEGATIVE ``PSN_*``.

    EVERY ARM MUST WANT THE SAME TYPE. One ``match`` emits one diagnostic per offending arm, so
    the wants are accumulated on ``ctx`` and ``propose_edits`` drops the whole match if they
    disagree — two expectations mean the arms are not one constant family and a human should
    look.
    """
    edits: list[Edit] = []
    for span, expected, found in _mismatch_sites(diagnostic):
        # `expected` is the SCRUTINEE's type and `found` the ARM constant's: rustc reports an
        # arm mismatch from the arm's side. Casting to `expected` writes `match (x as u16) as
        # u16` — a no-op that leaves the error and burns a pass.
        if _normalise_type(expected) != "int" or found not in INT_PRIMITIVES:
            continue
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        masked = _mask_lexical(data)
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        if not _in_match_pattern(masked, end) or _in_macro_definition(masked, start):
            continue
        scrutinee = _enclosing_match_scrutinee(masked, start)
        if scrutinee is None:
            continue
        key = (file_name, scrutinee[0], scrutinee[1])
        ctx.match_wants.setdefault(key, set()).add(found)
        pattern = _bare_arm_pattern(data, start, end)
        if pattern is not None:
            ctx.match_arm_sites.setdefault(key, []).append((*pattern, expected))
        # A scrutinee WIDER than the arm truncates under the cast: `match wParam as u32` on
        # amd64 matches VK_ESCAPE for 0x1_0000_001B. C converts the LABEL instead, which is
        # the guard fallback; it also needs no cross-target agreement (i386's WPARAM is
        # already u32 and reports nothing, which vetoed the cast and left mspaint red).
        if INT_WIDTH.get(found, 8) < INT_WIDTH.get(expected, 8):
            ctx.match_wants[key].add("<narrowing>")
            continue
        text = data[scrutinee[0] : scrutinee[1]].decode("utf-8", "replace")
        trailing = _RE_TRAILING_CAST.search(text)
        if trailing and trailing.group(0).split()[-1] == found:
            continue  # a previous pass already answered this match
        # An earlier pass cast to `found` and a later one cast it AWAY: the arms disagree
        # across passes (each pass sees one arm), and answering again builds a ladder. main's
        # keyboard.rs reached `as i32) as u32` forty deep between a consensus-copied
        # `PSN_APPLY: UINT` and the island's `PSN_RESET: i32`. Decline: the arms need a human.
        if re.search(rf"\bas {re.escape(found)}\)\s*as \w+\s*$", text):
            ctx.match_wants[key].add("<ladder>")  # -> the guard fallback in propose_edits
            continue
        # A cast to some OTHER type is the translation's own -- C's integer promotion, written
        # out (`match (*Cdb).AsByte[0] as i32` against u32 SCSIOP_* arms). Skipping it as
        # "answered" left all 30 of atapi's satl.rs arms red; the outer cast keeps the C's
        # conversion chain (u8 -> int -> the arms' type) intact.
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=scrutinee[0],
                byte_end=scrutinee[1],
                replacement=f"({text}) as {found}",
                rule=RULE_MATCH_SCRUTINEE,
                authority=(
                    f"arm at line {span.get('line_start')} is `{found}` and the scrutinee is "
                    f"`{expected}`; C's switch converts the scrutinee, and a cast is not legal "
                    "in a pattern"
                ),
                original=text,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
                # `found` is a resolved primitive, so on a width-varying family the three arches
                # can legitimately ask for different ones. Let agreement settle it.
                requires_target_agreement=True,
            )
        )
    return edits, [], bool(edits)


#: E0609 ``no field `X` on type `Y` `` — ``Y`` may arrive as a raw pointer or a reference.
_RE_NO_FIELD = re.compile(r"no field `(?P<field>\w+)` on type `(?P<ty>[&*\w: \n]+?)`", re.S)
#: E0560 ``struct `Y` has no field named `X` `` — a struct-LITERAL field, which is not fixable
#: by the same substitution (see the rule).
_RE_NO_LITERAL_FIELD = re.compile(
    r"(?:struct|union) `(?P<ty>[\w:]+)` has no field named `(?P<field>\w+)`"
)


_AUTOREF_LINT = "implicit autoref creates a reference to the dereference of a raw pointer"


def _anon_union_autoref(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`(*p).a.Field` where `a`'s type reaches `Field` only through the island's `Deref` impl.

    The islands emulate C's anonymous members with `impl Deref for _U { .. &self.anon }`, so a
    read through a raw pointer calls `deref(&self)` -- a reference made from a raw pointer,
    which rustc's deny-by-default `dangerous_implicit_autorefs` refuses. Naming the anonymous
    member (`a.__bindgen_anon_1.Field`) is the same place with no reference at all. rustc's own
    suggestion (`(&(*p).a).Field`) only makes the reference explicit; it is not taken.
    """
    span = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    inserts = [
        s
        for c in diagnostic.get("children") or []
        for s in c.get("spans") or []
        if s.get("suggested_replacement") == ")"
    ]
    if span is None or len(inserts) != 1:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    cut = int(inserts[0].get("byte_start") or 0)
    prefix = data[int(span.get("byte_start") or 0) : cut].decode("utf-8", "replace")
    rest = re.match(rb"\.\s*(\w+)", data[cut:])
    owner = re.search(r"\.\s*(\w+)\s*$", prefix)
    if rest is None or owner is None:
        return [], [], False
    field_name = rest.group(1).decode()
    types = ctx.field_types.get(owner.group(1), frozenset())
    paths = {ctx.anon_unions.get((t, field_name)) for t in types}
    if len(types) != 1 or len(paths) != 1 or None in paths:
        return (
            [],
            [
                Skipped(
                    RULE_ANON_UNION,
                    "autoref_owner_ambiguous",
                    file_name,
                    int(span.get("line_start") or 0),
                    f"`.{owner.group(1)}` has types {sorted(types) or '?'}; no single anonymous "
                    f"member reaches `{field_name}`",
                )
            ],
            False,
        )
    path = next(iter(paths))
    assert path is not None  # `None in paths` returned above
    start = cut + rest.start(1)
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=start + len(field_name),
                replacement=path,
                rule=RULE_ANON_UNION,
                authority=(
                    f"`{next(iter(types))}` reaches `{field_name}` only through its Deref to "
                    f"`{path.rsplit('.', 1)[0]}`; naming it avoids the raw-pointer autoref"
                ),
                original=field_name,
            )
        ],
        [],
        True,
    )


def _anon_union_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A field C reached through an ANONYMOUS union -> route it through ``__bindgen_anon_N``.

    C injects an anonymous union's members into the enclosing struct's name scope, so prsht.h's
    ``PROPSHEETHEADERW`` is written ``psh.nStartPage`` even though ``nStartPage`` shares a union
    with ``pStartPage``. Rust has no anonymous members, so bindgen names the union
    ``__bindgen_anon_N`` and the flattened spelling stops compiling. Every property-sheet module
    hits this (desk.cpl did; sysdm.cpl's ``psh.nStartPage``/``pszCaption``/``phpage`` will).

    THE MAP IS DERIVED, NEVER GUESSED. ``harvest_anon_unions`` reads the crate's own ``pub
    struct``/``pub union`` declarations and walks the ``__bindgen_anon_N`` members; a field the
    outer struct resolves directly is left alone, which reproduces exactly the shadowing C does.
    With no map the rule contributes nothing rather than inventing a path.

    WHY E0560 IS DECLINED. rustc reports the struct-LITERAL spelling of the same mistake as
    E0560, and it looks fixable by the same substitution — but it is not: ``S { a.b: v }`` is not
    legal Rust at all. The faithful repair is a zeroed value plus field stores, which is a
    rewrite and not a byte substitution, so the rule records the site and stands aside. (desk's
    hand-rolled version of this pass did rewrite E0560 sites, and every one of them came back as
    a parse error on the next pass.)
    """
    if not ctx.anon_unions:
        return [], [], False
    message = str(diagnostic.get("message") or "")

    literal = _RE_NO_LITERAL_FIELD.search(message)
    if literal is not None:
        span = next(
            (item for item in diagnostic.get("spans") or [] if item.get("is_primary")), None
        )
        struct, field_name = literal.group("ty"), literal.group("field")
        struct = struct.split("::")[-1]  # `ntdll_sys::nt::_IO_STATUS_BLOCK`
        if (struct, field_name) not in ctx.anon_unions:
            return [], [], False
        rewrite = _anon_literal_rewrite(span or {}, load, ctx, struct)
        if rewrite is not None:
            return [rewrite], [], True
        return (
            [],
            [
                Skipped(
                    RULE_ANON_UNION,
                    "struct_literal_field",
                    str((span or {}).get("file_name") or ""),
                    int((span or {}).get("line_start") or 0),
                    f"`{struct}.{field_name}` lives in "
                    f"{ctx.anon_unions[(struct, field_name)]}, but a struct literal cannot name "
                    "a nested field. Rewrite as a zeroed value plus field stores.",
                )
            ],
            False,
        )

    if message.startswith(_AUTOREF_LINT):
        return _anon_union_autoref(diagnostic, load, ctx)
    match = _RE_NO_FIELD.search(message)
    if match is None:
        return [], [], False
    field_name = match.group("field")
    # `*mut _PROPSHEETHEADERW`, `&_PROPSHEETHEADERW`, `_PROPSHEETHEADERW` all name one struct.
    struct = match.group("ty").replace("\n", " ").strip().lstrip("&*").strip()
    for prefix in ("mut ", "const "):
        if struct.startswith(prefix):
            struct = struct[len(prefix) :].strip()
    # rustc names a per-arch island's type by its module path (`types_arch::_IDE_REGISTERS`),
    # and the map is keyed by the bare name, as the struct-literal branch above already knows.
    struct = struct.split("::")[-1]
    path = ctx.anon_unions.get((struct, field_name))
    if path is None:
        return [], [], False

    span = next((item for item in diagnostic.get("spans") or [] if item.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    actual = data[start:end].decode("utf-8", "replace")
    if actual != field_name:
        # The span did not land on the bare identifier (a macro expansion, or a rustc shape we
        # have not seen). Say so rather than rewriting bytes we did not recognise.
        return (
            [],
            [
                Skipped(
                    RULE_ANON_UNION,
                    "span_is_not_the_field",
                    file_name,
                    int(span.get("line_start") or 0),
                    f"expected `{field_name}` at the primary span, found `{actual}`",
                )
            ],
            False,
        )
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=path,
                rule=RULE_ANON_UNION,
                authority=(
                    f"`{struct}` declares no `{field_name}`; its `{path.rsplit('.', 1)[0]}` "
                    f"anonymous union does, and no outer field shadows the name"
                ),
                original=field_name,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


#: ``.as_ptr()`` / ``.as_mut_ptr()`` immediately after the offending place expression.
_RE_AS_PTR = re.compile(r"\.as_(?P<mut>mut_)?ptr\(\)")


_CPP_BASE_PATH = re.compile(r"(?:__base\d+\.)+")


def _cpp_base_field_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`lock.m_hr` where `m_hr` is a C++ BASE class's member -> `lock.__base0.m_hr`.

    C++ name lookup finds inherited members; cppapp lays a class out with each base as a
    `__baseN` field, so the Rust spelling goes through it. rustc names the one path that
    reaches the field ("one of the expressions' fields has a field of the same name"), and the
    rule takes it only when that path is made of `__baseN.` hops and nothing else -- a base
    subobject is exactly what C++ lookup walks, so this is the C++ meaning, not a guess. Two
    candidate paths (a member reachable through two bases, ambiguous in C++ too) stay red.
    """
    if (diagnostic.get("code") or {}).get("code") not in ("E0609", "E0599"):
        return [], [], False
    # E0599 is the same lookup for a METHOD: msutb's CUTBMenuItem : CTipbarAccItem,
    # CUIFMenuItem calls `Check` -- CUIFMenuItem's, behind `__base1`, which Deref (primary
    # base only) cannot reach.
    sugg = [
        sp
        for ch in diagnostic.get("children") or []
        if re.search(r"has a (?:field|method) of the same name", str(ch.get("message") or ""))
        for sp in ch.get("spans") or []
    ]
    if len(sugg) != 1:
        return [], [], False
    sp = sugg[0]
    path = str(sp.get("suggested_replacement") or "")
    start, end = int(sp.get("byte_start") or 0), int(sp.get("byte_end") or 0)
    if not _CPP_BASE_PATH.fullmatch(path) or start != end or sp.get("expansion"):
        return [], [], False
    file_name = str(sp.get("file_name") or "")
    data = load(file_name)
    if data is None or (file_name, start, end) in ctx.claimed:
        return [], [], False
    ctx.claimed.add((file_name, start, end))
    edit = Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=end,
        replacement=path,
        original="",
        rule=RULE_CPP_BASE_FIELD,
        authority="C++ name lookup finds a base class's member; cppapp lays bases out as __baseN",
        line=int(sp.get("line_start") or 0),
        column=int(sp.get("column_start") or 0),
        requires_target_agreement=False,
        targets=(ctx.target,),
    )
    return [edit], [], True


_GUID_NAMES = r"(?:_GUID|GUID|IID|CLSID)"
_REFGUID_LABEL = re.compile(
    rf"expected `&(?P<mut>mut )?{_GUID_NAMES}`, found `\*(?P<ptr>const|mut) {_GUID_NAMES}`"
)


def _refguid_deref_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A GUID pointer where a GUID reference is wanted -> `&*(p)`.

    C++'s REFIID/REFGUID/REFCLSID are `const GUID &`, so a translated method rightly takes
    `riid: &IID`; the COM ABI (and C's REFIID) passes the same thing as `*const GUID`, which is
    what a vtable thunk or a C caller holds. `&*` is exactly the reference binding C++ does,
    and REFIID is never null by contract. Only GUID types: a general pointer-to-reference
    conversion would hide a null the C allowed.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0308":
        return [], [], False
    # One mismatched argument labels the primary span; several ("arguments to this method
    # are incorrect") give each its own child, whose message is the label.
    found: list[tuple[Mapping[str, Any], re.Match[str]]] = []
    for sp in diagnostic.get("spans") or []:
        m = _REFGUID_LABEL.fullmatch(str(sp.get("label") or ""))
        if sp.get("is_primary") and m:
            found.append((sp, m))
    for ch in diagnostic.get("children") or []:
        m = _REFGUID_LABEL.fullmatch(str(ch.get("message") or ""))
        sp = next((s for s in ch.get("spans") or [] if s.get("is_primary")), None)
        if m and sp is not None:
            found.append((sp, m))
    edits: list[Edit] = []
    for sp, m in found:
        if sp.get("expansion") or (m.group("mut") and m.group("ptr") != "mut"):
            continue
        file_name = str(sp.get("file_name") or "")
        start, end = int(sp.get("byte_start") or 0), int(sp.get("byte_end") or 0)
        data = load(file_name)
        if data is None or start == end or (file_name, start, end) in ctx.claimed:
            continue
        ctx.claimed.add((file_name, start, end))
        expr = data[start:end].decode()
        amp = "&mut *" if m.group("mut") else "&*"
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"{amp}({expr})",
                original=expr,
                rule=RULE_REFGUID_DEREF,
                authority="C++ REFIID is `const IID &`; the COM ABI passes it as a pointer",
                line=int(sp.get("line_start") or 0),
                column=int(sp.get("column_start") or 0),
                requires_target_agreement=False,
                targets=(ctx.target,),
            )
        )
    return edits, [], bool(edits)


_DEREF_MACRO = re.compile(rb"\*\s*[A-Za-z_]\w*!\(")
_NOT_DEREFABLE = re.compile(
    r"type `(?:[iu](?:8|16|32|64|128|size)|f32|f64|bool)` cannot be dereferenced"
)


def _macro_place_deref_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`*V_VT!(&v) = VT_I4` -> `V_VT!(&v) = VT_I4`: a deref of a macro that already yields
    the scalar place.

    The VARIANT accessors are C lvalues (`V_VT(p)` is `(p)->n1.n2.vt`), and `port macros`
    lowers them to exactly that place; the translation then added a `*` as if the macro
    returned a pointer. rustc names the macro's type, and a scalar cannot be dereferenced,
    so the `*` has no meaning to keep. ctfmon, msctfime and msutb (30 sites) all wrote it.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0614":
        return [], [], False
    if not _NOT_DEREFABLE.search(str(diagnostic.get("message") or "")):
        return [], [], False
    sp = next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)
    if sp is None or sp.get("expansion"):
        return [], [], False
    file_name = str(sp.get("file_name") or "")
    start, end = int(sp.get("byte_start") or 0), int(sp.get("byte_end") or 0)
    data = load(file_name)
    if data is None or not _DEREF_MACRO.match(data[start:end]):
        return [], [], False
    star_end = start + 1
    while data[star_end : star_end + 1] in (b" ", b"\t"):
        star_end += 1
    if (file_name, start, star_end) in ctx.claimed:
        return [], [], False
    ctx.claimed.add((file_name, start, star_end))
    edit = Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=star_end,
        replacement="",
        original=data[start:star_end].decode(),
        rule=RULE_MACRO_PLACE_DEREF,
        authority="rustc: the macro yields a scalar, which cannot be dereferenced",
        line=int(sp.get("line_start") or 0),
        column=int(sp.get("column_start") or 0),
        requires_target_agreement=False,
        targets=(ctx.target,),
    )
    return [edit], [], True


def _packed_field_ptr_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """``packed.array.as_ptr()`` -> ``addr_of!(packed.array).cast()`` (E0793).

    ``as_ptr()`` goes through ``&[T; N]``, and forming a reference to a field of a
    ``#[repr(packed)]`` struct is undefined behaviour even when it is never read — which is why
    rustc makes it a hard error rather than a lint. ``SP_DRVINFO_DETAIL_DATA`` and ``SHFILEINFO``
    are both packed in the SDK and the C simply takes the address, so the faithful translation
    is the one that never forms the reference.

    ``addr_of!`` does exactly that; ``.cast()`` turns the resulting ``*const [T; N]`` into the
    ``*const T`` the callee wants, which is what C's array-to-pointer decay says anyway. No
    width is baked, so no cross-target agreement is needed.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0793":
        return [], [], False
    span = next((item for item in diagnostic.get("spans") or [] if item.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start):
        return [], [], False
    # The span covers the PLACE expression; the offending `.as_ptr()` follows it.
    tail = _RE_AS_PTR.match(data[end : end + 16].decode("utf-8", "replace"))
    place = data[start:end].decode("utf-8", "replace")
    # C's `sizeof(p->Field)` came across as `size_of_val(&p.Field)`: the size of the field's
    # TYPE, read off a raw place pointer, never a reference.
    wrap = re.search(
        rb"(?:(?:core|std)::mem::)?size_of_val\s*\(\s*&\s*$", masked[max(0, start - 48) : start]
    )
    if wrap is not None and _next_code_byte(masked, end) == b")" and _balanced(masked[start:end]):
        lo = start - (len(masked[max(0, start - 48) : start]) - wrap.start())
        hi = masked.index(b")", end) + 1
        return (
            [
                Edit(
                    file_name=file_name,
                    byte_start=lo,
                    byte_end=hi,
                    replacement=(
                        "{ const fn pointee_size<T>(_: *const T) -> usize { "
                        "core::mem::size_of::<T>() } "
                        f"pointee_size(core::ptr::addr_of!({place})) }}"
                    ),
                    rule=RULE_PACKED_FIELD_PTR,
                    authority=(
                        f"rustc E0793: `{place[:40]}` is a packed field; its size is its type's, "
                        "taken through addr_of! so no reference is formed"
                    ),
                    original=data[lo:hi].decode("utf-8", "replace"),
                    line=int(span.get("line_start") or 0),
                    column=int(span.get("column_start") or 0),
                    targets=(ctx.target,),
                )
            ],
            [],
            True,
        )
    if tail is None or not _balanced(masked[start:end]):
        return (
            [],
            [
                Skipped(
                    RULE_PACKED_FIELD_PTR,
                    "not_an_as_ptr_call",
                    file_name,
                    int(span.get("line_start") or 0),
                    f"`{place[:60]}` is a packed-field reference rustc rejects, but it is not "
                    "the `.as_ptr()` shape this rule knows how to rewrite",
                )
            ],
            False,
        )
    macro = "addr_of_mut!" if tail.group("mut") else "addr_of!"
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end + tail.end(),
                replacement=f"core::ptr::{macro}({place}).cast()",
                rule=RULE_PACKED_FIELD_PTR,
                authority=(
                    f"rustc E0793: `{place[:40]}` is a field of a #[repr(packed)] struct, so "
                    f"`as_ptr()`'s intermediate reference is UB; `{macro}` takes the address "
                    "without ever forming one"
                ),
                original=f"{place}{tail.group(0)}",
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


#: E0277 ``the type `[T]` cannot be indexed by `u32` ``.
_RE_CANNOT_INDEX = re.compile(r"cannot be indexed by `(?P<index>\w+)`")


def _index_usize_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """Array/slice index of a non-``usize`` integer -> ``as usize``.

    C indexes with whatever integer is handy. rustc reports the Rust requirement as E0277 (no
    ``Index<u32>`` impl), NOT as a type mismatch, so no ``expected X, found Y`` label exists and
    ``RULE_NUMERIC_CAST`` never sees the site at all.

    ``usize`` is the width-varying alias itself, so the cast is portable by construction and
    needs no cross-target agreement — unlike a concrete ``as u32``. The index type is required
    to be an integer primitive so the rule cannot "fix" indexing by a struct.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0277":
        return [], [], False
    match = _RE_CANNOT_INDEX.search(str(diagnostic.get("message") or ""))
    if match is None or match.group("index") not in INT_PRIMITIVES:
        return [], [], False
    span = next((item for item in diagnostic.get("spans") or [] if item.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    if start >= end:
        return [], [], False
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start) or not _balanced(masked[start:end]):
        return [], [], False
    text = data[start:end].decode("utf-8", "replace")
    if re.search(r"\bas\s+usize\s*$", text):
        return [], [], False
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"({text}) as usize",
                rule=RULE_INDEX_USIZE,
                authority=(
                    f"rustc E0277: the indexed type has no `Index<{match.group('index')}>`; "
                    "`usize` is the width-varying alias itself, so the cast is portable"
                ),
                original=text,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


#: E0423 ``expected function[, tuple struct or tuple variant], found macro `X` `` — a header
#: macro rendered as a call. rustc uses the long wording at most call sites and the short one at
#: the rest; matching only the short one found 1 site of 61 on sysdm.cpl and looked like a rule
#: that simply had little to do.
_RE_FOUND_MACRO = re.compile(
    r"expected function(?:, tuple struct or tuple variant)?, found macro `(?P<name>\w+)`"
)
#: E0423 ``expected value, found type alias `X` `` — a C ``sizeof(TYPE)`` that lost its operator.
_RE_FOUND_TYPE = re.compile(
    r"expected value, found (?:type alias|struct|builtin type) `(?P<name>\w+)`"
)


def _macro_call_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """``FOO(a, b)`` where the crate declares ``macro_rules! FOO`` -> ``FOO!(a, b)``.

    A C header macro has no Rust function to be, so the prelude spells it ``macro_rules!`` — and
    the translation writes the CALL exactly as the C did, without the ``!``. rustc names the
    macro it found, so the fix carries its own authority: there is nothing to infer, only a
    token to insert. This is the largest single bucket on a freshly translated Win32 module
    (60 sites on sysdm.cpl, and desk.cpl needed the same pass by hand).

    Nothing about it is width- or arch-dependent, so it needs no cross-target agreement.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0423":
        return [], [], False
    match = _RE_FOUND_MACRO.search(str(diagnostic.get("message") or ""))
    if match is None:
        return [], [], False
    span = next((item for item in diagnostic.get("spans") or [] if item.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    name = match.group("name")
    if data[start:end].decode("utf-8", "replace") != name:
        return [], [], False
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start):
        return [], [], False
    if _next_code_byte(masked, end) != b"(":
        # Not a call at all — rustc found the macro in some other value position, and appending
        # `!` there would not even parse.
        return (
            [],
            [
                Skipped(
                    RULE_MACRO_CALL,
                    "not_a_call",
                    file_name,
                    int(span.get("line_start") or 0),
                    f"`{name}` is a macro used in value position but not followed by `(`",
                )
            ],
            False,
        )
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f"{name}!",
                rule=RULE_MACRO_CALL,
                authority=(
                    f"rustc E0423 names `{name}` as a MACRO in call position; the crate declares "
                    "it with `macro_rules!`, so the `!` is the whole of the fix"
                ),
                original=name,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _sizeof_type_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C ``sizeof(TYPE)`` that came across as the bare TYPE NAME -> ``size_of::<TYPE>()``.

    C spells the size of a type and the size of a value the same way, and Rust does not:
    ``size_of::<T>()`` vs ``size_of_val(&v)``. A translation that keeps the C's single spelling
    lands the type name where a value belongs, in exactly two shapes — bare
    (``x.cbSize = BITMAPINFO``) and wrapped (``size_of_val(&NONCLIENTMETRICS)``). rustc names the
    type it found, so both are answerable without inference.

    The wrapped shape MUST swallow the whole call: rewriting only the operand leaves
    ``size_of_val(&size_of::<T>())``, which is the size of a `usize` — a plausible-looking number
    that is wrong on every arch, and compiles.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0423":
        return [], [], False
    match = _RE_FOUND_TYPE.search(str(diagnostic.get("message") or ""))
    if match is None:
        return [], [], False
    span = next((item for item in diagnostic.get("spans") or [] if item.get("is_primary")), None)
    if span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    name = match.group("name")
    if data[start:end].decode("utf-8", "replace") != name:
        return [], [], False
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start):
        return [], [], False
    # A TYPE argument is a C macro's contract, not a lost `sizeof`: setuplib's
    # `RTL_NUMBER_OF_FIELD(VOLINFO, DeviceName)` was rewritten to take
    # `size_of::<VOLINFO>()` while the macro was still unported (so a plain call to rustc),
    # and stayed broken once it was. C macros are spelled in capitals; Rust ones carry `!`.
    depth, i = 0, start - 1
    while i >= 0 and not (masked[i : i + 1] == b"(" and depth == 0):
        depth += {b")": 1, b"]": 1, b"}": 1, b"(": -1, b"[": -1, b"{": -1}.get(masked[i : i + 1], 0)
        i -= 1
    head = re.search(rb"([A-Za-z_]\w*)\s*(!?)\s*$", masked[max(0, i - 80) : max(0, i)])
    if i >= 0 and head is not None:
        callee = head.group(1).decode()
        if head.group(2) or re.fullmatch(r"[A-Z][A-Z0-9_]*", callee):
            return (
                [],
                [
                    Skipped(
                        RULE_SIZEOF_TYPE,
                        "macro_type_argument",
                        file_name,
                        int(span.get("line_start") or 0),
                        f"`{name}` is an argument of `{callee}`, a macro that takes a type",
                    )
                ],
                False,
            )

    replacement = f"core::mem::size_of::<{name}>()"
    lo, hi = start, end
    # `size_of_val(&NAME)` / `core::mem::size_of_val(&NAME)` — take the whole call.
    prefix = data[max(0, start - 64) : start].decode("utf-8", "replace")
    wrapper = re.search(r"(?:core::mem::)?size_of_val\s*\(\s*&\s*$", prefix)
    if wrapper is not None and _next_code_byte(masked, end) == b")":
        lo = start - (len(prefix) - wrapper.start())
        hi = masked.index(b")", end) + 1
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=lo,
                byte_end=hi,
                replacement=replacement,
                rule=RULE_SIZEOF_TYPE,
                authority=(
                    f"rustc E0423 names `{name}` as a TYPE in value position; C's `sizeof(T)` is "
                    "Rust's `size_of::<T>()`, and the operand is the type rustc just named"
                ),
                original=data[lo:hi].decode("utf-8", "replace"),
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _primary(diagnostic: Mapping[str, Any]) -> dict[str, Any] | None:
    return next((s for s in diagnostic.get("spans") or [] if s.get("is_primary")), None)


def _option_is_null_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """C's `if (pfn != NULL)` on a function pointer bindgen types `Option<fn>`: `is_none()`.

    rustc E0599 puts the primary span on the method NAME, so the edit is that one word and the
    receiver is untouched. setuplib: six `pSetupData->ErrorRoutine` tests in install.c.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0599":
        return [], [], False
    message = str(diagnostic.get("message") or "")
    if not message.startswith("no method named `is_null` found for enum `Option<"):
        return [], [], False
    span = _primary(diagnostic)
    file_name = str((span or {}).get("file_name") or "")
    data = load(file_name) if span else None
    if span is None or data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    if data[start:end] != b"is_null" or _in_macro_definition(_mask_lexical(data), start):
        return [], [], False
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement="is_none",
                rule=RULE_OPTION_IS_NULL,
                authority="rustc E0599: the pointer is an Option<fn>, whose NULL is None",
                original="is_null",
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _enclosing_brace(masked: bytes, pos: int) -> int | None:
    depth = 0
    for i in range(pos - 1, -1, -1):
        char = masked[i : i + 1]
        if char == b"}":
            depth += 1
        elif char == b"{":
            if depth == 0:
                return i
            depth -= 1
    return None


def _brace_closer(masked: bytes, opening: int) -> int | None:
    depth = 0
    for pos in range(opening, len(masked)):
        char = masked[pos : pos + 1]
        if char == b"{":
            depth += 1
        elif char == b"}":
            depth -= 1
            if depth == 0:
                return pos
    return None


def _shadowed_const_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C local named like a constant: rename the LOCAL for the rest of its block.

    C lets `ADAPTER_TYPE AdapterType;` shadow the `AdapterType` enumerator of
    CONFIGURATION_TYPE, and every later mention in that block is the local. Rust forbids a
    `let` binding a constant's name (E0530) -- it would parse as a refutable pattern -- so the
    faithful repair is C's own scoping, spelled as a rename: every mention from the `let` to the
    end of the enclosing block becomes `<name>_local`. Field accesses (`.Name`), paths
    (`::Name`, `Name::`) and struct-literal keys (`{ Name: ..`) are other namespaces and stay.
    Mentions BEFORE the `let` are still the constant, as in C. setuplib: arcname, filesup.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0530":
        return [], [], False
    # Statics too: taskmgr's `ULONG SessionId;` local met ntdll-sys's glob-imported
    # `static mut SessionId` -- same C scoping, same rename.
    if not re.search(r"cannot shadow (?:constants|statics)", str(diagnostic.get("message") or "")):
        return [], [], False
    span = _primary(diagnostic)
    file_name = str((span or {}).get("file_name") or "")
    data = load(file_name) if span else None
    if span is None or data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    name = data[start:end].decode("utf-8", "replace")
    masked = _mask_lexical(data)
    if not re.fullmatch(r"[A-Za-z_]\w*", name) or _in_macro_definition(masked, start):
        return [], [], False
    head = masked[max(0, start - 24) : start]
    if re.search(rb"\blet\s+(?:mut\s+)?$", head):
        opening = _enclosing_brace(masked, start)
    elif re.search(rb"[(,]\s*(?:mut\s+)?$", head) and masked[end:].lstrip().startswith(b":"):
        # A PARAMETER named like a static (autochk's `OpenKeyboard(KeyboardHandle: PHANDLE)`
        # beside `static mut KeyboardHandle`; welcome's `RunAction(nTopic)`): in C the
        # parameter shadows the global for the whole body, so the body is the scope.
        opening = _fn_body_brace(masked, start)
    else:
        return (
            [],
            [Skipped(RULE_SHADOWED_CONST, "not_a_let", file_name, int(span["line_start"]), name)],
            False,
        )
    closer = _brace_closer(masked, opening) if opening is not None else None
    fresh = f"{name}_local"
    if closer is None or re.search(rb"\b" + fresh.encode() + rb"\b", masked):
        return [], [], False
    edits = []
    for m in re.finditer(rb"\b" + re.escape(name.encode()) + rb"\b", masked[start:closer]):
        at = start + m.start()
        before = masked[:at].rstrip()
        after = masked[at + len(name) :].lstrip()
        if before.endswith((b".", b"::")) or after.startswith(b"::"):
            continue
        if after.startswith(b":") and before.endswith((b"{", b",")):
            continue  # struct-literal key
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=at,
                byte_end=at + len(name),
                replacement=fresh,
                rule=RULE_SHADOWED_CONST,
                authority=(
                    f"rustc E0530: local `{name}` shadows a constant/static; C scoping makes every "
                    "later mention in this block the local, so the local is renamed"
                ),
                original=name,
                line=_line_of(data, at),
                targets=(ctx.target,),
            )
        )
    return edits, [], True


def _fn_body_brace(masked: bytes, at: int) -> int | None:
    """The `{` opening the body of the fn whose parameter list contains offset `at`."""
    depth = 0
    for i in range(at, len(masked)):
        c = masked[i : i + 1]
        if c in b"([":
            depth += 1
        elif c in b")]":
            depth -= 1
            if depth < 0:
                brace = masked.find(b"{", i)
                semi = masked.find(b";", i)
                return brace if brace >= 0 and (semi < 0 or brace < semi) else None
    return None


def _sizeof_const_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """C ``sizeof(CONSTANT)`` that came across as ``size_of::<CONSTANT>()``: the size of a VALUE.

    E0747 puts the constant in type position. ``size_of_val(&CONSTANT)`` is C's meaning
    provided the constant carries C's type, which both of its sources do: hand-written
    substrate spells the C cast (``UNICODE_NULL: WCHAR``), and bindgen sizes an integer
    macro as C would type the literal (int -> 4, a 64-bit value -> 8).
    """
    if (diagnostic.get("code") or {}).get("code") != "E0747":
        return [], [], False
    if "constant provided when a type was expected" not in str(diagnostic.get("message") or ""):
        return [], [], False
    span = _primary(diagnostic)
    file_name = str((span or {}).get("file_name") or "")
    data = load(file_name) if span else None
    if span is None or data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    name = data[start:end].decode("utf-8", "replace")
    masked = _mask_lexical(data)
    if not re.fullmatch(r"[A-Za-z_]\w*", name) or _in_macro_definition(masked, start):
        return [], [], False
    pre = re.search(
        rb"(?:(?:core|std)::mem::)?size_of\s*::\s*<\s*$", masked[max(0, start - 40) : start]
    )
    post = re.match(rb"\s*>\s*\(\s*\)", masked[end : end + 16])
    if pre is None or post is None:
        return (
            [],
            [Skipped(RULE_SIZEOF_CONST, "not_size_of", file_name, int(span["line_start"]), name)],
            False,
        )
    lo = start - (len(masked[max(0, start - 40) : start]) - pre.start())
    hi = end + post.end()
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=lo,
                byte_end=hi,
                replacement=f"core::mem::size_of_val(&{name})",
                rule=RULE_SIZEOF_CONST,
                authority=f"rustc E0747: `{name}` is a constant, so C's sizeof is of its value",
                original=data[lo:hi].decode("utf-8", "replace"),
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


#: E0605 on a fn ITEM (rustc prints its type with the item's path in braces: `... {VfatFormat}`).
_RE_FN_ITEM_CAST = re.compile(
    r"^non-primitive cast: `[^`]*\bfn\([^`]*\{(?P<item>[\w:]+)\}>?` as `[^`]*\bfn\("
)


def _fn_item_cast_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """`f as unsafe extern "C" fn(..) -> R` where `f` is a function: drop the cast.

    It is c2rust's spelling of C's implicit function-to-pointer conversion, and c2rust writes
    `extern "C"` for every function pointer -- including NTAPI (stdcall) ones, so on i386 it
    is a different type from the function and rustc refuses (E0605; setuplib's
    RegisteredFileSystems table: VfatFormat/VfatChkdsk). Without the cast the function item
    coerces to whatever slot it is stored in -- `Some(f)` into an `Option<fn>` field, a call
    argument -- and if THAT slot's ABI differs, rustc says so as a plain type mismatch, which
    is the real finding the wrong-ABI cast was hiding.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0605":
        return [], [], False
    m = _RE_FN_ITEM_CAST.match(str(diagnostic.get("message") or ""))
    span = _primary(diagnostic)
    if m is None or span is None:
        return [], [], False
    file_name = str(span.get("file_name") or "")
    data = load(file_name)
    if data is None:
        return [], [], False
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start):
        return [], [], False
    item = m.group("item").split("::")[-1]
    head = re.match(rb"\s*(" + re.escape(item.encode()) + rb")\s+as\s", masked[start:end])
    # The same conversion already wrapped for an `Option<fn>` slot: `Some(f) as Option<fn(..)>`.
    # `as` never converts between Option types, even with identical signatures (ndis's
    # `DriverUnload = Some(MainUnload) as PDRIVER_UNLOAD`); `Some(f)` coerces on its own.
    some = re.match(
        rb"\s*(Some\(\s*" + re.escape(item.encode()) + rb"\s*,?\s*\))\s+as\s", masked[start:end]
    )
    if head is None and some is None:
        return [], [], False
    replacement = item if head is not None else f"Some({item})"
    return (
        [
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=replacement,
                rule=RULE_FN_ITEM_CAST,
                authority=(
                    f"rustc E0605: `{item}` is a function; C converts it to the pointer "
                    "implicitly, and Rust coerces it to the slot's own fn-pointer type"
                ),
                original=data[start:end].decode("utf-8", "replace"),
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                targets=(ctx.target,),
            )
        ],
        [],
        True,
    )


def _simple_edit(rule, file_name, data, start, end, replacement, authority, span, ctx) -> Edit:
    return Edit(
        file_name=file_name,
        byte_start=start,
        byte_end=end,
        replacement=replacement,
        rule=rule,
        authority=authority,
        original=data[start:end].decode("utf-8", "replace"),
        line=int(span.get("line_start") or 0),
        column=int(span.get("column_start") or 0),
        targets=(ctx.target,),
    )


def _primary_bytes(diagnostic, load):
    span = _primary(diagnostic)
    file_name = str((span or {}).get("file_name") or "")
    data = load(file_name) if span else None
    if span is None or data is None:
        return None
    start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
    masked = _mask_lexical(data)
    if _in_macro_definition(masked, start):
        return None
    return span, file_name, data, masked, start, end


def _variadic_promotion_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0617: an integer narrower than `int` passed to a C `...`: rustc's own `as c_int` /
    `as c_uint` (MachineApplicable) is C's default argument promotion, value for value."""
    if (diagnostic.get("code") or {}).get("code") != "E0617":
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    for child in diagnostic.get("children") or []:
        for sp in child.get("spans") or []:
            rep = sp.get("suggested_replacement")
            if (
                rep in (" as c_int", " as c_uint", " as c_double")
                and sp.get("suggestion_applicability") == "MachineApplicable"
                and sp.get("file_name") == file_name
            ):
                at = int(sp["byte_start"])
                if at != end:
                    continue  # rustc's suggestion is always "append to the argument"
                cast = rep.replace(" as c_", " as core::ffi::c_")
                arg = data[start:end].decode("utf-8", "replace")
                # rustc's suggestion only APPENDS, and `as` binds tighter than every binary
                # operator: `a & VT_TYPEMASK as c_uint` casts the right operand, not the
                # argument. The u16-vs-c_uint mismatch that leaves was then "fixed" back to u16
                # by truncating_numeric_cast, and the two rules traded the site every pass
                # (oleaut32's VarDecFromXxx FIXME!). Replace the whole argument, so the edit
                # also records what it rewrote for the oscillation guard.
                wrapped = arg if _RE_ATOMIC_EXPR.fullmatch(arg.strip()) else f"({arg})"
                edit = _simple_edit(
                    RULE_VARIADIC_PROMOTION,
                    file_name,
                    data,
                    start,
                    end,
                    f"{wrapped}{cast}",
                    "rustc E0617: C promotes a narrow integer / float passed to `...`",
                    span,
                    ctx,
                )
                return [edit], [], True
    return [], [], False


#: An expression `as` can follow without parentheses: unary operators (which bind tighter
#: than `as`) on a path or one parenthesized group, then any chain of `.field`, `(args)` and
#: `[index]`. Anything with a binary operator outside parentheses is not one.
_PAREN = r"\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)"
_RE_ATOMIC_EXPR = re.compile(rf"[*!&-]*(?:[\w:]+|{_PAREN})(?:\.\w+|{_PAREN}|\[[^\[\]]*\])*")


def _local_static_mut_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0277 `Sync` on a `static` holding raw pointers: C's `static` is plain mutable storage,
    and only `static mut` can hold a pointer (setuplib's `static UNICODE_STRING DeviceFloppy =
    RTL_CONSTANT_STRING(..)` and a `static PCSTR FormatStateNames[]`)."""
    if (diagnostic.get("code") or {}).get("code") != "E0277":
        return [], [], False
    text = json.dumps(diagnostic.get("children") or [])
    if "cannot be shared between threads safely" not in str(diagnostic.get("message") or ""):
        return [], [], False
    if "shared static variables must have a type that implements `Sync`" not in text and (
        "static" not in text
    ):
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, masked, start, _end = got
    line_start = masked.rfind(b"\n", 0, start) + 1
    head = re.match(rb"\s*(?:pub(?:\([^)]*\))?\s+)?static\s+(?!mut\b)", masked[line_start:start])
    if head is None:
        return [], [], False
    at = line_start + head.end()
    return (
        [
            _simple_edit(
                RULE_LOCAL_STATIC_MUT,
                file_name,
                data,
                at,
                at,
                "mut ",
                "rustc E0277: a C static holding a pointer is mutable storage (`static mut`)",
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _pointer_not_not_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0600 `!!p` on a pointer: C's normalise-to-boolean is `!p.is_null()`."""
    if (diagnostic.get("code") or {}).get("code") != "E0600":
        return [], [], False
    if not re.search(r"`!` to type `\*(?:const|mut) ", str(diagnostic.get("message") or "")):
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, masked, start, end = got
    inner = data[start:end].decode("utf-8", "replace")
    if not inner.startswith("!") or not _balanced(masked[start + 1 : end]):
        return [], [], False
    lo = start - 1 if data[start - 1 : start] == b"!" else None
    if lo is None:
        return [], [], False
    return (
        [
            _simple_edit(
                RULE_POINTER_NOT_NOT,
                file_name,
                data,
                lo,
                end,
                f"(!({inner[1:]}).is_null())",
                "rustc E0600: C's `!!p` on a pointer is `p != NULL`",
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _zeroed_default_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0277 `Default` not implemented (a bindgen union-bearing struct): the C was a `{0}`
    zero initialiser, which is `core::mem::zeroed()`."""
    if (diagnostic.get("code") or {}).get("code") != "E0277":
        return [], [], False
    if "the trait `Default` is not implemented" not in json.dumps(diagnostic):
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    text = data[start:end].decode("utf-8", "replace")
    if not re.fullmatch(r"(?:core::default::)?Default::default\(\)", text):
        return [], [], False
    return (
        [
            _simple_edit(
                RULE_ZEROED_DEFAULT,
                file_name,
                data,
                start,
                end,
                "core::mem::zeroed()",
                "rustc E0277: no Default; C's `{0}` is all-bits-zero",
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _receiver_start(masked: bytes, dot: int) -> int | None:
    """Start of the postfix expression ending just before `masked[dot]` (a `.`): a path, a
    call or macro call `f(..)` / `m!(..)`, an index `[..]`, chained by `.`, or a `(..)`."""
    i = dot
    while True:
        j = i - 1
        while j >= 0 and masked[j : j + 1] in (b" ", b"\t"):
            j -= 1
        if j < 0:
            return None
        ch = masked[j : j + 1]
        if ch in (b")", b"]"):
            opener = b"(" if ch == b")" else b"["
            depth, k = 0, j
            while k >= 0:
                c = masked[k : k + 1]
                if c in (b")", b"]"):
                    depth += 1
                elif c in (b"(", b"["):
                    depth -= 1
                    if depth == 0:
                        break
                k -= 1
            if k < 0 or masked[k : k + 1] != opener:
                return None
            i = k
            m = re.search(rb"[A-Za-z_][\w:]*!?$", masked[max(0, k - 80) : k])
            if m is not None:
                i = k - (len(masked[max(0, k - 80) : k]) - m.start())
        else:
            m = re.search(rb"[A-Za-z_][\w:]*$", masked[max(0, j - 80) : j + 1])
            if m is None:
                return None
            i = j + 1 - (len(masked[max(0, j - 80) : j + 1]) - m.start())
        before = i - 1
        while before >= 0 and masked[before : before + 1] in (b" ", b"\t"):
            before -= 1
        if (
            before >= 0
            and masked[before : before + 1] == b"."
            and masked[before - 1 : before] != b"."
        ):
            i = before
            continue
        return i


def _arrow_field_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0609 `no field F on type *mut T`: C's `p->F`, written `p.F`. The receiver -- however
    it is spelled, including a `CONTAINING_RECORD!(..)` call whose own suggestion points into
    the macro -- becomes `(*receiver)`."""
    if (diagnostic.get("code") or {}).get("code") != "E0609":
        return [], [], False
    if not re.match(
        r"no field `\w+` on type `\*(?:mut|const) ", str(diagnostic.get("message") or "")
    ):
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, masked, start, _end = got
    dot = start - 1
    while dot >= 0 and masked[dot : dot + 1] in (b" ", b"\t", b"\n"):
        dot -= 1
    if masked[dot : dot + 1] != b".":
        return [], [], False
    lo = _receiver_start(masked, dot)
    if lo is None:
        return [], [], False
    recv = data[lo:dot].decode("utf-8", "replace").rstrip()
    return (
        [
            _simple_edit(
                RULE_ARROW_FIELD,
                file_name,
                data,
                lo,
                dot,
                f"(*{recv})",
                "rustc E0609: the receiver is a raw pointer; C's `p->f` is `(*p).f`",
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _unsigned_negate_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0600 unary `-` on an unsigned type: C's `(ULONG)-1` is the negated value in that
    type, which is `(1 as u32).wrapping_neg()` -- u32::MAX for -1, and right for any N."""
    if (diagnostic.get("code") or {}).get("code") != "E0600":
        return [], [], False
    message = str(diagnostic.get("message") or "")
    if re.search(r"unary operator `!` to type `\*(?:mut|const) ", message):
        # C's `if (!mainWindow.DoCreate())` on an HWND: logical NOT of a pointer is "is NULL".
        got = _primary_bytes(diagnostic, load)
        if got is None:
            return [], [], False
        span, file_name, data, _masked, start, end = got
        text = data[start:end].decode("utf-8", "replace")
        inner = re.fullmatch(r"(?s)!\s*(.+)", text)
        if data[:start].rstrip().endswith(b"!"):
            return [], [], False  # `!!p`: RULE_POINTER_NOT_NOT's
        if inner is None or not _balanced(inner.group(1).encode()):
            return [], [], False
        rep = f"({inner.group(1)}).is_null()"
        note = "rustc E0600: C's `!p` on a pointer is a NULL test"
        return (
            [_simple_edit(RULE_UNSIGNED_NEGATE, file_name, data, start, end, rep, note, span, ctx)],
            [],
            True,
        )
    m = re.search(r"unary operator `-` to type `(u8|u16|u32|u64|usize)`", message)
    if m is None:
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    text = data[start:end].decode("utf-8", "replace")
    lit = re.fullmatch(r"\(?\s*-\s*\(?\s*(\d+|0[xX][0-9a-fA-F]+)\s*\)?\s*\)?", text)
    named = re.fullmatch(r"\(?\s*-\s*\(?\s*([A-Za-z_]\w*)\s*\)?\s*\)?", text)
    value = ctx.constants.get(named.group(1)) if named else None
    if lit is None and named is not None and value is not None and 0 <= value <= 0x7FFF_FFFF:
        # `-GRIP_SIZE` of a `#define GRIP_SIZE 3`: C's macro is an `int`, so the negation
        # is signed and `-GRIP_SIZE / 2` is -1. bindgen's `u32` is the literal's width only;
        # wrapping in u32 would make that division 0x7FFF_FFFE.
        return (
            [
                _simple_edit(
                    RULE_UNSIGNED_NEGATE,
                    file_name,
                    data,
                    start,
                    end,
                    f"(-({named.group(1)} as i32))",
                    "rustc E0600 on a constant whose value fits `int`: C negates the int macro",
                    span,
                    ctx,
                )
            ],
            [],
            True,
        )
    if lit is None:
        return [], [], False
    return (
        [
            _simple_edit(
                RULE_UNSIGNED_NEGATE,
                file_name,
                data,
                start,
                end,
                f"({lit.group(1)} as {m.group(1)}).wrapping_neg()",
                "rustc E0600: C negates in the unsigned type, wrapping",
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _option_fn_as_ptr_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0605 `f as *mut c_void` where `f` is an `Option<fn>`: C passes the function pointer as
    a PVOID; `None` is NULL. `f.map_or(null_mut(), |p| p as *mut _)`."""
    if (diagnostic.get("code") or {}).get("code") != "E0605":
        return [], [], False
    message = str(diagnostic.get("message") or "")
    m = re.match(r"non-primitive cast: `Option<[^`]*fn\([^`]*` as `\*(mut|const) ", message)
    # `(ULONG_PTR)GetProcAddress(..)` / `GetProcAddress(..) as usize`: the address as an
    # integer, and None is 0. mspaint's OpenMailer tests MAPISendMailW's this way.
    as_int = re.match(r"non-primitive cast: `Option<[^`]*fn\([^`]*` as `(usize|u64|u32)`", message)
    if m is None and as_int is None:
        return [], [], False
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    text = data[start:end].decode("utf-8", "replace")
    if as_int is not None:
        # rustc's span may carry the source's own parentheses: `((*this_).m_Shutdown as usize)`.
        while (
            text.startswith("(")
            and text.endswith(")")
            and _matching_closer(_mask_lexical(text.encode()), 0) == len(text) - 1
        ):
            text = text[1:-1].strip()
        cast = re.fullmatch(r"(?s)(?P<e>.+?)\s+as\s+(?P<t>usize|u64|u32|[A-Z_]+)", text)
        if cast is None or not _balanced(cast.group("e").encode()):
            return [], [], False
        rep = f"({cast.group('e')}).map_or(0, |f| f as usize) as {cast.group('t')}"
        edit = _simple_edit(
            RULE_OPTION_FN_AS_PTR,
            file_name,
            data,
            start,
            end,
            rep,
            "rustc E0605: C casts the function pointer to an integer; None is 0",
            span,
            ctx,
        )
        return [edit], [], True
    cast = re.fullmatch(r"(?s)(?P<e>.+?)\s+as\s+(?P<t>\*(?:mut|const)\s+[\w:]+)", text)
    if cast is None or not _balanced(cast.group("e").encode()):
        return [], [], False
    if m is None:
        return [], [], False
    null = "core::ptr::null_mut()" if m.group(1) == "mut" else "core::ptr::null()"
    rep = f"({cast.group('e')}).map_or({null}, |f| f as {cast.group('t')})"
    return (
        [
            _simple_edit(
                RULE_OPTION_FN_AS_PTR,
                file_name,
                data,
                start,
                end,
                rep,
                "rustc E0605: an Option<fn> as a pointer; C's NULL is None",
                span,
                ctx,
            )
        ],
        [],
        True,
    )


def _some_fn_item_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A function passed where bindgen types the slot `Option<fn>`: C's implicit conversion of
    a function to its pointer, which is `Some(f)`. rustc's own two-part suggestion, taken only
    when the found side is a fn ITEM (setuplib: nine EnumerateBootStoreEntries callbacks)."""
    if (diagnostic.get("code") or {}).get("code") != "E0308":
        return [], [], False
    if "found fn item" not in json.dumps(diagnostic):
        return [], [], False
    edits: list[Edit] = []
    for child in diagnostic.get("children") or []:
        msg = str(child.get("message") or "")
        if not msg.startswith("try wrapping the expression in ") or not msg.endswith("Some`"):
            continue
        parts = [
            sp for sp in child.get("spans") or [] if sp.get("suggested_replacement") is not None
        ]
        if len(parts) != 2:
            continue
        open_sp, close_sp = sorted(parts, key=lambda sp: int(sp["byte_start"]))
        file_name = str(open_sp.get("file_name") or "")
        data = load(file_name)
        if data is None or close_sp.get("file_name") != file_name:
            continue
        lo, hi = int(open_sp["byte_start"]), int(close_sp["byte_start"])
        inner = data[lo:hi].decode("utf-8", "replace")
        if not re.fullmatch(r"\s*[A-Za-z_][\w:]*\s*", inner):
            continue  # only a bare function name: anything else is not a fn item
        masked = _mask_lexical(data)
        if _in_macro_definition(masked, lo):
            continue
        edits.append(
            _simple_edit(
                RULE_SOME_FN_ITEM,
                file_name,
                data,
                lo,
                hi,
                f"Some({inner.strip()})",
                "rustc E0308: a function into an Option<fn> slot; C's implicit &f is Some(f)",
                open_sp,
                ctx,
            )
        )
    return edits, [], bool(edits)


_C_LITERAL_PTR = re.compile(r"(?:RUSTCONV_WSTR_\w+|b\"(?:[^\"\\]|\\.)*\")\.as_ptr\(\)")


def _literal_mut_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A C string literal passed to a non-const pointer parameter: `.cast_mut()`.

    In C a string literal is `wchar_t[N]` and decays to `wchar_t *`, so const-incorrect
    prototypes take it without a cast -- diskpart's `HasPrefix(PWSTR, PWSTR, PWSTR *)`, called
    with `L"primary"` at 22 sites. The sealed literal is a Rust static, `.as_ptr()` is
    `*const`, and rustc says "types differ in mutability". Only for an expression that IS the
    literal (a `RUSTCONV_WSTR_*` static or a byte string); C's semantics, nothing invented --
    writing through it is undefined in both languages.
    """
    if (diagnostic.get("code") or {}).get("code") != "E0308":
        return [], [], False
    text_all = json.dumps(diagnostic)
    if "types differ in mutability" not in text_all:
        return [], [], False
    if "found reference `&" in text_all and "expected raw pointer `*mut" in text_all:
        return _shared_ref_to_mut_ptr(diagnostic, load, ctx)
    if "found raw pointer `*const" not in text_all:
        return [], [], False
    edits: list[Edit] = []
    for span in diagnostic.get("spans") or []:
        if "types differ in mutability" not in str(span.get("label") or ""):
            continue
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        lo, hi = int(span["byte_start"]), int(span["byte_end"])
        inner = data[lo:hi].decode("utf-8", "replace")
        addr = re.fullmatch(r"((?:core::ptr::|ptr::)?)addr_of!(\(.*\))", inner.strip(), re.S)
        if addr is not None and not _in_macro_definition(_mask_lexical(data), lo):
            # C's `&x` is mutable; the translation spelled it `addr_of!`. A C local is mutable
            # too: an immutable `let` of the same name gets its `mut` in the same pass (fdo.c's
            # ObjectAttributes), anything else that cannot be borrowed mutably stays E0596.
            local = re.fullmatch(r"\(\s*([A-Za-z_]\w*)\s*\)", addr.group(2))
            if local is not None:
                lets = list(re.finditer(rb"\blet\s+" + local.group(1).encode() + rb"\b", data[:lo]))
                if lets:
                    at = lets[-1]
                    edits.append(
                        _simple_edit(
                            RULE_LITERAL_MUT,
                            file_name,
                            data,
                            at.start(),
                            at.end(),
                            f"let mut {local.group(1)}",
                            "the C local is mutable; its address is taken `*mut`",
                            span,
                            ctx,
                        )
                    )
            edits.append(
                _simple_edit(
                    RULE_LITERAL_MUT,
                    file_name,
                    data,
                    lo,
                    hi,
                    f"{addr.group(1)}addr_of_mut!{addr.group(2)}",
                    "rustc E0308 (mutability): C's address-of is mutable; the callee takes `*mut`",
                    span,
                    ctx,
                )
            )
            continue
        if not _C_LITERAL_PTR.fullmatch(inner.strip()):
            continue
        if _in_macro_definition(_mask_lexical(data), lo):
            continue
        edits.append(
            _simple_edit(
                RULE_LITERAL_MUT,
                file_name,
                data,
                lo,
                hi,
                f"{inner.strip()}.cast_mut()",
                "rustc E0308 (mutability): a C string literal decays to a non-const pointer",
                span,
                ctx,
            )
        )
    return edits, [], bool(edits)


_CONST_DEF = re.compile(
    r"^\s*pub const ([A-Za-z_]\w*)\s*:\s*[\w:]+\s*=\s*(-?)\s*(0[xX][0-9a-fA-F_]+|[0-9][0-9_]*)"
    r"(?:[iu](?:8|16|32|64|128|size))?\s*;",
    re.M,
)


def local_constants(crate_dir: Path) -> dict[str, int]:
    """``RULE_CONST_OPERAND``'s authority: integer constants THIS crate defines, by value.

    A name defined with two different values (an arch-dependent island constant) is left out:
    "fits" must hold on every target the edit lands on.
    """
    values: dict[str, set[int]] = {}
    for path in sorted((crate_dir / "src").rglob("*.rs")):
        for m in _CONST_DEF.finditer(path.read_text(errors="replace")):
            value = int(m.group(3).replace("_", ""), 0) * (-1 if m.group(2) else 1)
            values.setdefault(m.group(1), set()).add(value)
    own = {k: v.pop() for k, v in values.items() if len(v) == 1}
    # The usermode island's PUBLIC `pub use ntdll_sys::nt::*;` reaches the caller's scope the
    # same way `harvest`'s callee signatures do: mspaint's `GetKeyState(VK_SHIFT)` is a
    # winuser.h `VK_SHIFT: u32 = 16` into a `c_int`, and only ntdll-sys defines it. Every
    # bindgen_<arch>.rs is read, so a value that differs by arch drops out like a local one.
    islands = [p.read_bytes() for p in (crate_dir / "src").glob("types_*.rs")]
    if any(re.search(rb"^pub use ntdll_sys::nt::\*;", d, re.M) for d in islands):
        sys_values: dict[str, set[int]] = {}
        for path in sorted((crate_dir.parent / "ntdll-sys" / "src").glob("bindgen_*.rs")):
            for m in _CONST_DEF.finditer(path.read_text(errors="replace")):
                value = int(m.group(3).replace("_", ""), 0) * (-1 if m.group(2) else 1)
                sys_values.setdefault(m.group(1), set()).add(value)
        for name, vals in sys_values.items():
            if name not in values and len(vals) == 1:
                own[name] = vals.pop()
    return own


def _int_range(ty: str, width: int) -> tuple[int, int] | None:
    bits = {"8": 8, "16": 16, "32": 32, "64": 64, "128": 128, "size": width}.get(ty[1:])
    if ty[:1] not in ("i", "u") or bits is None:
        return None
    return (0, 2**bits - 1) if ty[0] == "u" else (-(2 ** (bits - 1)), 2 ** (bits - 1) - 1)


def _sibling_declared_type(data: bytes, start: int, end: int) -> str | None:
    """`wParam & K` / `K | wParam`: the type NAME the other operand was declared with
    (`wParam: WPARAM` earlier in the function), when the constant at start..end is one side
    of a bitwise or arithmetic operator and the other side is a bare identifier."""
    before = data[:start].decode("utf-8", "replace")
    after = data[end:].decode("utf-8", "replace")
    m = re.search(r"\b([A-Za-z_]\w*)\s*[&|^+*/-]\s*$", before) or re.match(
        r"\s*[&|^+*/-]\s*([A-Za-z_]\w*)\b(?!\s*[.(\[])", after
    )
    if m is None:
        return None
    decl = None
    for d in re.finditer(rf"\b{re.escape(m.group(1))}\s*:\s*([A-Za-z_]\w*)\b", before):
        decl = d.group(1)
    return decl if decl and decl not in INT_PRIMITIVES else None


def _infer_constant_cast(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """E0282 at the `_` of `(MK_LBUTTON as _)` under a bitwise/arithmetic operator: name the
    other operand's declared type instead (`wParam: WPARAM` -> `as WPARAM`)."""
    got = _primary_bytes(diagnostic, load)
    if got is None:
        return [], [], False
    span, file_name, data, _masked, start, end = got
    if data[start:end] != b"_":
        return [], [], False
    m = re.search(rb"\(\s*([A-Za-z_]\w*)\s+as\s*\Z", data[:start])
    close = re.match(rb"\s*\)", data[end:])
    if m is None or close is None or m.group(1).decode() not in ctx.constants:
        return [], [], False
    sibling = _sibling_declared_type(data, m.start(), end + close.end())
    if sibling is None:
        return [], [], False
    note = f"the other operand is declared `{sibling}`; `as _` cannot be inferred here"
    return (
        [_simple_edit(RULE_CONST_OPERAND, file_name, data, start, end, sibling, note, span, ctx)],
        [],
        True,
    )


def _constant_operand_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """A named integer CONSTANT whose VALUE fits the type rustc wants there: `(NAME as T)`.

    bindgen gives a `#define` the type of its literal (`QUEUE_FLAG_FROZEN_POWER: u32 = 32`),
    while C gives it whatever its use converts it to: atapi's `Device->QueueFlags &
    QUEUE_FLAG_FROZEN_POWER` is LONG & int. A cast of a constant that fits is
    value-preserving by construction, so it needs neither a promotion proof nor a width vote.
    One that does not fit (`0x80000000` into `i32`) is left to the rules that reason about
    promotion.
    """
    code = (diagnostic.get("code") or {}).get("code")
    if code == "E0282" and ctx.constants:
        return _infer_constant_cast(diagnostic, load, ctx)
    if code != "E0308" or not ctx.constants:
        return [], [], False
    edits: list[Edit] = []
    for span, expected, found in _mismatch_sites(diagnostic):
        if expected not in INT_PRIMITIVES or found not in INT_PRIMITIVES:
            continue
        file_name = str(span.get("file_name") or "")
        start, end = int(span.get("byte_start") or 0), int(span.get("byte_end") or 0)
        data = load(file_name)
        if data is None or end <= start or (file_name, start, end) in ctx.claimed:
            continue
        original = data[start:end].decode("utf-8", "replace")
        name = original.strip()
        # A constant the translation already cast to one arch's width, `(IDD_X as u64)`:
        # re-spelled like a bare one (the model baked amd64's WPARAM into an i386 build).
        baked = re.fullmatch(r"\(\s*([A-Za-z_]\w*)\s+as\s+[iu](?:8|16|32|64|size)\s*\)", name)
        if baked is not None:
            name = baked.group(1)
        value = ctx.constants.get(name)
        bounds = _int_range(expected, ctx.width)
        if value is None or bounds is None or not bounds[0] <= value <= bounds[1]:
            continue
        masked = _mask_lexical(data)
        # A match ARM is a pattern, where a cast does not parse: the scrutinee rule owns it.
        if _in_macro_definition(masked, start) or _in_match_pattern(masked, end):
            continue
        target = expected
        if baked is not None or expected in ("u64", "i64", "usize", "isize"):
            # A baked width is arch-dependent by construction, whatever this target wants.
            # `as _` cannot be inferred under an arithmetic/bitwise operator (`wParam &
            # (MK_LBUTTON as _)`); there the other operand's DECLARED type name is portable.
            target = _sibling_declared_type(data, start, end) or "_"
        ctx.claimed.add((file_name, start, end))
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                # A 64-bit or pointer-sized want is usually a width-varying alias (WPARAM is
                # u64 on amd64, u32 on i386): `as _` lets each target's context say, where
                # `as u64` broke every i386 `wParam == (VK_ESCAPE as u64)`.
                replacement=f"({name} as {target})",
                rule=RULE_CONST_OPERAND,
                authority=(
                    f"`{name}` is the constant {value}, which `{expected}` holds exactly; C "
                    "converts a constant to the type its use needs"
                ),
                original=original,
                line=int(span.get("line_start") or 0),
                column=int(span.get("column_start") or 0),
                requires_target_agreement=False,
                targets=(ctx.target,),
            )
        )
    return edits, [], bool(edits)


def _shared_ref_to_mut_ptr(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    """C's `&s->f` passed to a NON-const pointer parameter, written as a shared `&`: `&raw mut`.

    C has one address-of, and it is mutable unless the type says otherwise; the translation
    spelled it `&`, which coerces only to `*const`. atapi had 8, all
    `AtaDevHasFlushCache(&DevExt->IdentifyDeviceData)` against identify_funcs.h's
    `PIDENTIFY_DEVICE_DATA`. `&raw mut` makes no reference at all. A place that cannot be
    borrowed mutably (an immutable local, a temporary) stays a loud E0596, never a silent change.
    """
    edits: list[Edit] = []
    for span in diagnostic.get("spans") or []:
        if "types differ in mutability" not in str(span.get("label") or ""):
            continue
        file_name = str(span.get("file_name") or "")
        data = load(file_name)
        if data is None:
            continue
        lo, hi = int(span["byte_start"]), int(span["byte_end"])
        inner = data[lo:hi].decode("utf-8", "replace").strip()
        if not inner.startswith("&") or re.match(r"&\s*(?:mut\b|raw\b|&)", inner):
            continue
        masked = _mask_lexical(data)
        if _in_macro_definition(masked, lo) or not _balanced(masked[lo:hi]):
            continue
        edits.append(
            _simple_edit(
                RULE_LITERAL_MUT,
                file_name,
                data,
                lo,
                hi,
                f"&raw mut {inner[1:].lstrip()}",
                "rustc E0308 (mutability): C's address-of is mutable; the callee takes `*mut`",
                span,
                ctx,
            )
        )
    return edits, [], bool(edits)


_RULE_TABLE: dict[
    str, Callable[[Mapping[str, Any], FileReader, _Ctx], tuple[list[Edit], list[Skipped], bool]]
] = {
    RULE_WIDTH_ALIAS: _width_alias_rule,
    RULE_LITERAL_MUT: _literal_mut_rule,
    RULE_NUMERIC_CAST: _numeric_cast_rule,
    RULE_COMPARISON: _comparison_rule,
    RULE_MATCH_SCRUTINEE: _match_scrutinee_rule,
    # Emitted by propose_edits from the scrutinee rule's evidence, never per diagnostic.
    RULE_MATCH_ARM_GUARD: lambda _d, _l, _c: ([], [], False),
    RULE_NO_STD_PATH: _no_std_path_rule,
    RULE_ARRAY_DECAY: _array_decay_rule,
    RULE_POINTER_CAST: _pointer_cast_rule,
    RULE_ANON_UNION: _anon_union_rule,
    RULE_PACKED_FIELD_PTR: _packed_field_ptr_rule,
    RULE_CPP_BASE_FIELD: _cpp_base_field_rule,
    RULE_REFGUID_DEREF: _refguid_deref_rule,
    RULE_MACRO_PLACE_DEREF: _macro_place_deref_rule,
    RULE_INDEX_USIZE: _index_usize_rule,
    RULE_MACRO_CALL: _macro_call_rule,
    RULE_SIZEOF_TYPE: _sizeof_type_rule,
    RULE_C_INT_SUFFIX: _c_int_suffix_rule,
    RULE_NULL_FN_POINTER: _null_fn_pointer_rule,
    RULE_C_TRUTHINESS: _c_truthiness_rule,
    RULE_OPTION_FN_CALL: _option_fn_call_rule,
    RULE_UNINIT_LOCAL: _uninit_local_rule,
    RULE_MUT_BINDING: _mut_binding_rule,
    RULE_INFER_WIDEN: _infer_widen_rule,
    RULE_OPTION_IS_NULL: _option_is_null_rule,
    RULE_SHADOWED_CONST: _shadowed_const_rule,
    RULE_SIZEOF_CONST: _sizeof_const_rule,
    RULE_FN_ITEM_CAST: _fn_item_cast_rule,
    RULE_VARIADIC_PROMOTION: _variadic_promotion_rule,
    RULE_LOCAL_STATIC_MUT: _local_static_mut_rule,
    RULE_POINTER_NOT_NOT: _pointer_not_not_rule,
    RULE_ZEROED_DEFAULT: _zeroed_default_rule,
    RULE_ARROW_FIELD: _arrow_field_rule,
    RULE_UNSIGNED_NEGATE: _unsigned_negate_rule,
    RULE_OPTION_FN_AS_PTR: _option_fn_as_ptr_rule,
    RULE_SOME_FN_ITEM: _some_fn_item_rule,
    RULE_KEYWORD_FIELD: _keyword_field_rule,
    RULE_C2RUST_HOP: _c2rust_hop_rule,
    RULE_BITFIELD: _bitfield_rule,
    RULE_C2RUST_ANON: _c2rust_anon_rule,
    RULE_FLOAT_LITERAL: _float_literal_rule,
    RULE_OBJECT_DEREF: lambda d, load, c: _merge_rules(
        d,
        load,
        c,
        _object_deref_rule,
        _raw_receiver_rule,
        _atl_format_id_rule,
        _cstring_conversion_rule,
        _ref_compound_assign_rule,
        _zero_constant_pointer_rule,
        _int_float_arith_rule,
        _guid_eq_rule,
        _heap_ptr_conversion_rule,
        _com_ptr_receiver_rule,
        _hwnd_to_cwindow_rule,
    ),
    RULE_CONST_OPERAND: _constant_operand_rule,
}


def propose_edits(
    diagnostics: Sequence[Mapping[str, Any]],
    read_file: FileReader,
    *,
    target: str,
    rules: Collection[str] = DIAGNOSTIC_RULES,
    signatures: Mapping[str, Sequence[str]] | None = None,
    local_aliases: Collection[str] = (),
    anon_unions: Mapping[tuple[str, str], str] | None = None,
    constants: Mapping[str, int] | None = None,
    own_prefix: str = "",
    field_types: Mapping[str, frozenset[str]] | None = None,
    alias_scope: Mapping[str, frozenset[str]] | None = None,
) -> RepairPlan:
    """PURE: parsed rustc diagnostics -> proposed edits, declined sites, and residual RED.

    ``read_file`` maps a diagnostic's ``file_name`` to its bytes (or ``None``); no cargo run and
    no filesystem knowledge is baked in, so tests drive this with fixtures.

    ``signatures``/``local_aliases`` are the authority for ``RULE_WIDTH_ALIAS`` and
    ``anon_unions`` is the authority for ``RULE_ANON_UNION``; all three must be harvested from
    the crate's OWN sources — see those rules for why provenance is the guard. Omitting one
    disables its rule instead of loosening it.
    """
    ctx = _Ctx(
        target=target,
        width=pointer_width(target),
        signatures=signatures or {},
        local_aliases=frozenset(local_aliases),
        anon_unions=anon_unions or {},
        constants=constants or {},
        own_prefix=own_prefix,
        field_types=field_types or {},
        alias_scope=alias_scope or {},
    )
    cache: dict[str, bytes | None] = {}

    def load(name: str) -> bytes | None:
        if name not in cache:
            cache[name] = read_file(name)
        return cache[name]

    edits: dict[tuple[str, int, int, str], Edit] = {}
    skipped: list[Skipped] = []
    red: list[Red] = []
    for diagnostic in diagnostics:
        if not is_error_diagnostic(diagnostic):
            continue
        claimed = False
        for name in DIAGNOSTIC_RULES:
            if name not in rules:
                continue
            rule_edits, rule_skipped, rule_claimed = _RULE_TABLE[name](diagnostic, load, ctx)
            claimed = claimed or rule_claimed
            skipped.extend(rule_skipped)
            for edit in rule_edits:
                edits.setdefault(edit.key, edit)
        if claimed:
            continue
        primary = next(
            (span for span in diagnostic.get("spans") or [] if span.get("is_primary")), None
        )
        red.append(
            Red(
                code=(diagnostic.get("code") or {}).get("code"),
                shape=_shape_of(diagnostic),
                file_name=str(primary.get("file_name") or "") if primary else "",
                line=int(primary.get("line_start") or 0) if primary else 0,
                message=str(diagnostic.get("message") or ""),
            )
        )
    # One `match` emits one diagnostic per offending arm, so arm-vs-arm disagreement is only
    # visible now that every diagnostic has been seen. A scrutinee cast that satisfies one arm
    # and breaks another is not a repair; drop the whole match and say which types collided.
    for key, wants in ctx.match_wants.items():
        if len(wants) < 2 and "<narrowing>" not in wants:
            continue
        edit = edits.pop((*key, RULE_MATCH_SCRUTINEE), None)
        # C converts every `case` label to the scrutinee's type, so the faithful answer to
        # arms of DIFFERENT integer types is that conversion, per arm, as a guard:
        # `code if code == PSN_APPLY as u32 =>`. (bindgen types `PSN_APPLY` i32 by its value
        # where C says unsigned; clang-evaluated facts say u32. A guard also cannot silently
        # BIND the way a bare unresolved pattern does.) Only bare-path arms; anything else is
        # left to a human as before.
        sites = ctx.match_arm_sites.get(key, [])
        if sites and RULE_MATCH_ARM_GUARD in rules:
            # Narrowing: `as _` lets the comparison name the scrutinee's own type, so the
            # one edit is right on every arch whatever width the alias has there.
            narrowing = "<narrowing>" in wants
            for a_start, a_end, arm, expected in sites:
                target_type = "_" if narrowing else expected
                guard = "__c if " + " || ".join(
                    f"__c == {name.strip()} as {target_type}" for name in arm.split("|")
                )
                guard_edit = Edit(
                    file_name=key[0],
                    byte_start=a_start,
                    byte_end=a_end,
                    replacement=guard,
                    rule=RULE_MATCH_ARM_GUARD,
                    authority=(
                        f"arms want {', '.join(sorted(w for w in wants if w[:1] != '<'))}; "
                        f"C converts the `case {arm}` label to the scrutinee type `{expected}`"
                    ),
                    original=arm,
                    targets=(target,),
                    requires_target_agreement=not narrowing,
                )
                edits[guard_edit.key] = guard_edit
            continue
        skipped.append(
            Skipped(
                RULE_MATCH_SCRUTINEE,
                "arms_disagree",
                key[0],
                edit.line if edit is not None else 0,
                f"arms want {', '.join(sorted(wants))} — not one constant family, so the "
                "scrutinee has no single correct type. Fix where the constants are DECLARED.",
            )
        )
    ordered = sorted(edits.values(), key=lambda item: (item.file_name, item.byte_start))
    return RepairPlan(target=target, edits=tuple(ordered), skipped=tuple(skipped), red=tuple(red))


# --- pure analysis: flexible-array members ----------------------------------------------------

# Deliberately tolerant of formatting: a version of this pattern that assumed rustfmt's
# one-field-per-line-with-trailing-comma output found ZERO fields in a hand-written types module
# and the whole rule silently became a no-op — the exact silent degradation AGENTS.md forbids.
_ARRAY_FIELD = re.compile(
    # NOTE the suffix alternation: real bindgen writes `[SOCKET_ADDRESS; 1usize]`, not `[T; 1]`.
    # Without it `\d+` matches the `1`, the following `\]` hits the `u`, and the whole rule finds
    # ZERO fields on every generated types module in this tree (iphlpapi declares 32). That is a
    # silent no-op, so the spelling is pinned by test_flex_array_decls_parses_real_bindgen_suffix.
    r"\bpub\s+(?P<name>\w+)\s*:\s*\[\s*(?P<elem>[^;\[\]]+?)\s*;\s*"
    r"(?P<count>\d+)(?:usize|isize|u8|u16|u32|u64|i8|i16|i32|i64)?\s*\]"
)


def flex_array_decls(types_source: str, *, source_name: str = "types.rs") -> FlexArrayDecls:
    """Find bindgen flexible-array members (``[T; 1]``) in a generated types module.

    A C flexible array member (``T Foo[]`` / ``T Foo[1]``) is rendered by bindgen as ``[T; 1]``,
    so Rust ``arr[i]`` for i>=1 bounds-check-PANICS at runtime.  A field name that ALSO appears
    with a length != 1 is ambiguous (we match sites lexically by field name) and is refused.
    """
    fields: dict[str, str] = {}
    ambiguous: set[str] = set()
    for match in _ARRAY_FIELD.finditer(types_source):
        name = match.group("name")
        if int(match.group("count")) == 1:
            # The element type is recorded for the audit trail only — the rewrite never names it.
            fields[name] = match.group("elem").strip()
        else:
            ambiguous.add(name)
    return FlexArrayDecls(
        fields={name: elem for name, elem in sorted(fields.items()) if name not in ambiguous},
        ambiguous=frozenset(ambiguous & set(fields)),
        source=source_name,
    )


#: A place expression we can safely prefix with `*`: a plain identifier or a single parenthesised
#: deref, followed by any number of `.field` hops.  Anything else is refused (``base_not_closed``)
#: rather than guessed at.
_FLEX_BASE = (
    r"(?P<base>(?:\(\s*\*[^()]*\)|[A-Za-z_]\w*)(?:\s*\.\s*[A-Za-z_]\w*)*)"
    r"\s*\.\s*(?P<field>{field})\s*\["
)
_IDENT_BYTE = re.compile(rb"[A-Za-z_0-9]")


def propose_flex_array_edits(
    file_name: str,
    data: bytes,
    decls: FlexArrayDecls,
    *,
    target: str = "",
) -> tuple[list[Edit], list[Skipped]]:
    """PURE: rewrite ``place.flex[i]`` to a parenthesised pointer-arithmetic deref.

    Authority: ``decls`` says the field's bindgen type is a 1-element array, i.e. a C flexible
    array member, so indexing it in Rust is a latent panic and pointer arithmetic is what the C
    meant.  Matches the house idiom already in the tree:
    ``(*(*table).routes.as_ptr().add(ndx))``.
    """
    if not decls.fields:
        return [], []
    masked = _mask_lexical(data)
    unsafe = _unsafe_regions(masked)
    text = masked.decode("latin-1")  # length-preserving: char offsets == byte offsets
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    pattern = re.compile(_FLEX_BASE.format(field="|".join(map(re.escape, decls.fields))))
    for match in pattern.finditer(text):
        field_name = match.group("field")
        start = match.start("base")
        open_bracket = match.end() - 1
        line = _line_of(data, start)

        def decline(reason: str, detail: str, line: int = line) -> None:
            skipped.append(Skipped(RULE_FLEX_ARRAY, reason, file_name, line, detail))

        closing = _match_forward(masked, open_bracket)
        if closing is None:
            decline("unclosed_index", field_name)
            continue
        index_text = data[open_bracket + 1 : closing - 1].decode("utf-8", "replace").strip()
        if not index_text or index_text == "0":
            continue  # `[0]` is in bounds for `[T; 1]` — nothing to repair
        previous = data[start - 1 : start] if start else b""
        if _IDENT_BYTE.match(previous) or previous in (b".", b")", b"]"):
            # Our base regex did not capture the whole place expression; prefixing `*` here
            # would splice a `*` into the middle of an expression. Refuse loudly.
            decline("base_not_closed", f"{field_name} preceded by {previous!r}")
            continue
        if not _in_regions(unsafe, start):
            # A raw-pointer deref needs an unsafe context; we will not silently add one.
            decline("not_unsafe_context", f"{field_name}[{index_text}]")
            continue
        borrowed_mut = re.search(r"&\s*mut\s*$", text[max(0, start - 16) : start]) is not None
        mutable = borrowed_mut or _is_assignment_target(masked, closing)
        accessor = "as_mut_ptr" if mutable else "as_ptr"
        base = data[start : match.end("base")].decode("utf-8", "replace")
        original = data[start:closing].decode("utf-8", "replace")
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=closing,
                replacement=f"(*{base}.{field_name}.{accessor}().add({index_text}))",
                rule=RULE_FLEX_ARRAY,
                authority=(
                    f"`{field_name}` is declared `[{decls.fields[field_name]}; 1]` in "
                    f"{decls.source} — a bindgen C flexible-array member, so `[{index_text}]` "
                    "would bounds-check-panic at runtime; pointer arithmetic is the C semantics"
                ),
                original=original,
                line=line,
                column=start - (data.rfind(b"\n", 0, start) + 1) + 1,
                targets=(target,) if target else (),
            )
        )
    for name in sorted(decls.ambiguous):
        skipped.append(
            Skipped(
                RULE_FLEX_ARRAY,
                "ambiguous_field_length",
                decls.source,
                0,
                f"`{name}` is declared both `[T; 1]` and `[T; N>1]` — a lexical field-name match "
                "cannot tell the sites apart",
            )
        )
    return edits, skipped


# --- cross-target agreement -------------------------------------------------------------------


def merge_plans(
    plans: Mapping[str, RepairPlan],
    *,
    allow_single_target_width: bool = False,
) -> MergedPlan:
    """Keep only edits every proposing target agrees on; withhold width-unproven ones.

    THE correctness property of this module: a concrete ``as u32`` inferred from an i386 run is
    wrong on amd64/arm64 wherever the real parameter is a width-varying alias.  Such an edit is
    applied only when EVERY analysed target proposes the IDENTICAL replacement; a disagreement is
    reported as ``target_divergent`` so a human can write the semantic alias instead.

    "Every analysed target", not ">= 2 of them".  A target that proposes NOTHING at a site is not
    abstaining -- it is saying the site already compiles there, which is the strongest possible
    evidence against baking some other target's width into it.  desk.cpl found this the hard way:
    ``SendDlgItemMessage(.., CB_INSERTSTRING, PLACEMENT_CENTER, ..)`` is clean on i386 (WPARAM is
    u32, and so is the constant) and mismatched on the 64-bit pair, so amd64 and arm64 agreed on
    ``as u64``, the old >= 2 rule applied it, and 54 call sites that had been CORRECT on i386
    stopped compiling there.  The right answer at such a site is always the callee's declared type
    NAME (``as WPARAM``), which RULE_WIDTH_ALIAS writes and this rule must not pre-empt.
    """
    grouped: dict[tuple[str, int, int, str], list[Edit]] = {}
    for plan in plans.values():
        for edit in plan.edits:
            grouped.setdefault(edit.key, []).append(edit)
    analysed = tuple(sorted(plans))
    edits: list[Edit] = []
    withheld: list[Withheld] = []
    for _key, group in sorted(grouped.items()):
        proto = group[0]
        by_replacement: dict[str, list[str]] = {}
        for edit in group:
            by_replacement.setdefault(edit.replacement, []).extend(edit.targets)
        if len(by_replacement) > 1:
            detail = " | ".join(
                f"{','.join(sorted(targets))}=>{replacement}"
                for replacement, targets in sorted(by_replacement.items())
            )
            withheld.append(
                Withheld(
                    proto.rule,
                    "target_divergent",
                    proto.file_name,
                    proto.line,
                    f"targets disagree at `{proto.original}`: {detail}. Hand-write the semantic "
                    "alias (SIZE_T/ULONG_PTR/usize) here — do not bake either width.",
                )
            )
            continue
        targets = tuple(sorted({target for edit in group for target in edit.targets}))
        # Sensitivity is a property of the SITE, not of whichever plan happened to be first:
        # `as u32` is pointer-width on i386 and fixed on amd64, so one flag is enough to gate.
        sensitive = any(edit.requires_target_agreement for edit in group)
        if sensitive and (len(targets) < 2 or set(targets) != set(analysed)):
            if not allow_single_target_width:
                withheld.append(
                    Withheld(
                        proto.rule,
                        "width_unproven",
                        proto.file_name,
                        proto.line,
                        f"`{proto.replacement}` bakes a pointer-width type proven only on "
                        f"{','.join(targets) or 'no target'} (analysed: {','.join(analysed)}). "
                        + (
                            f"{','.join(sorted(set(analysed) - set(targets)))} reported nothing "
                            "here, which means the site already compiles there — applying this "
                            "would BREAK it. Hand-write the callee's declared type name."
                            if set(targets) < set(analysed)
                            else "Re-run with the other targets, or hand-write the semantic alias."
                        ),
                    )
                )
                continue
            edits.append(
                replace(
                    proto,
                    targets=targets,
                    authority=proto.authority
                    + f"; APPLIED UNDER single_target_width_override({','.join(targets)})",
                )
            )
            continue
        suffix = f"; agreed by {','.join(targets)}" if targets else ""
        edits.append(replace(proto, targets=targets, authority=proto.authority + suffix))
    return MergedPlan(
        analysed_targets=analysed,
        edits=tuple(edits),
        withheld=tuple(withheld),
        skipped=tuple(item for plan in plans.values() for item in plan.skipped),
        red=tuple(item for plan in plans.values() for item in plan.red),
    )


def select_non_overlapping(edits: Sequence[Edit]) -> tuple[list[Edit], list[Skipped]]:
    """Keep the outermost of any overlapping spans; splices must not corrupt each other."""
    chosen: list[Edit] = []
    dropped: list[Skipped] = []
    by_file: dict[str, list[Edit]] = {}
    for edit in edits:
        by_file.setdefault(edit.file_name, []).append(edit)
    for file_name, group in sorted(by_file.items()):
        last_end: int | None = None
        for edit in sorted(group, key=lambda item: (item.byte_start, -item.byte_end, item.rule)):
            if last_end is not None and edit.byte_start < last_end:
                dropped.append(
                    Skipped(
                        edit.rule,
                        "overlapping_span",
                        file_name,
                        edit.line,
                        f"`{edit.original}` overlaps an outer edit; retry next pass",
                    )
                )
                continue
            chosen.append(edit)
            last_end = edit.byte_end
    return chosen, dropped


# --- applying ---------------------------------------------------------------------------------


def assert_faithful(edit: Edit) -> None:
    """Refuse to write a replacement that panics where the C silently truncated.

    A backstop, not decoration: it is exactly what a future rule would regress into if someone
    "simplified" it by taking rustc's MachineApplicable suggestion, and it would ship a panic
    into a running service (also a ``tools/scan_silent_failures.py`` violation).
    """
    for token in PANICKING_SUGGESTIONS:
        if token in edit.replacement:
            raise RepairError(
                f"crate_repair: rule {edit.rule} tried to write `{token}` at "
                f"{edit.file_name}:{edit.line} — a panicking conversion is never a faithful "
                "translation of a C implicit conversion. Emit `as <ty>` instead."
            )


_TRAILING_CAST = re.compile(r"\bas\s+[A-Za-z_][\w:]*\s*$")


def _guard_cast_before_lt(replacement: str, rest: bytes) -> str:
    """Parenthesize a cast that a `<` follows: `x as i32 < y` does not parse.

    rustc reads `i32 <` as the start of generic arguments. usetup's partlist.rs got
    `(a - b) as i32 < (Height) as i32` from a promotion cast, and that one parse error hid
    every other error in the crate. A suffix-only edit (` as T`) has no start to wrap, so
    it is left alone.
    """
    if replacement.startswith(" ") or not _TRAILING_CAST.search(replacement):
        return replacement
    if not rest.lstrip(b" \t\r\n").startswith(b"<"):
        return replacement
    return f"({replacement})"


def apply_edits(
    edits: Sequence[Edit], root: Path, within: Path | None = None
) -> tuple[list[Edit], dict[str, bytes], list[Skipped]]:
    """Splice edits back-to-front so byte offsets stay valid. Returns (applied, snapshot, failed).

    ``snapshot`` holds each touched file's ORIGINAL bytes so a caller can undo the whole pass.
    ``within``: the crate being repaired. An edit whose file lies outside it is refused --
    rustc reports a span INSIDE a shared crate's macro when the misuse is at the call site,
    and the truthiness rule once rewrote win32macros-rust's IsRecognizedPartition to fit
    setuplib, breaking it for every other consumer.
    """
    by_file: dict[str, list[Edit]] = {}
    for edit in edits:
        by_file.setdefault(edit.file_name, []).append(edit)
    applied: list[Edit] = []
    snapshot: dict[str, bytes] = {}
    failed: list[Skipped] = []
    for file_name, group in sorted(by_file.items()):
        path = root / file_name
        if within is not None and not path.resolve().is_relative_to(within.resolve()):
            failed.extend(
                Skipped(edit.rule, "outside_crate", file_name, edit.line, str(path))
                for edit in group
            )
            continue
        if not path.is_file():
            failed.extend(
                Skipped(edit.rule, "missing_file", file_name, edit.line, str(path))
                for edit in group
            )
            continue
        data = path.read_bytes()
        snapshot[file_name] = data
        ordered = sorted(group, key=lambda item: -item.byte_start)
        last_start = len(data) + 1
        for edit in ordered:
            assert_faithful(edit)
            if edit.byte_end > last_start:
                raise RepairError(
                    f"crate_repair: overlapping edits in {file_name} at byte {edit.byte_start} — "
                    "call select_non_overlapping() before apply_edits()"
                )
            if data[edit.byte_start : edit.byte_end].decode("utf-8", "replace") != edit.original:
                failed.append(
                    Skipped(
                        edit.rule,
                        "stale_span",
                        file_name,
                        edit.line,
                        f"expected `{edit.original}` at byte {edit.byte_start}",
                    )
                )
                continue
            replacement = _guard_cast_before_lt(edit.replacement, data[edit.byte_end :])
            data = data[: edit.byte_start] + replacement.encode() + data[edit.byte_end :]
            last_start = edit.byte_start
            applied.append(edit)
        path.write_bytes(data)
    return applied, snapshot, failed


def restore_snapshot(snapshot: Mapping[str, bytes], root: Path) -> None:
    for file_name, data in snapshot.items():
        (root / file_name).write_bytes(data)


# --- driver -----------------------------------------------------------------------------------


def run_cargo_check(
    crate: str,
    target: str,
    *,
    root: Path,
    timeout: int = 1800,
    extra_args: Sequence[str] = (),
    target_dir: Path | None = None,
) -> CargoCheckRun:
    """Shell out to ``cargo check -p <crate> --target <triple> --message-format=json``.

    ``target_dir``: a CARGO_TARGET_DIR of this arch's own. cargo locks a build directory as a
    whole, so three arches sharing one run one after another however they are launched.
    """
    command = [
        "cargo",
        "check",
        "-p",
        crate,
        "--target",
        resolve_target(target),
        "--message-format=json",
        *extra_args,
    ]
    try:
        env = None
        if target_dir is not None:
            env = {**os.environ, "CARGO_TARGET_DIR": str(target_dir)}
        process = subprocess.run(
            command,
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
            env=env,
        )
    except FileNotFoundError as error:
        raise RepairError("crate_repair: `cargo` not found on PATH") from error
    except subprocess.TimeoutExpired as error:
        raise RepairError(
            f"crate_repair: `cargo check -p {crate} --target {target}` timed out after {timeout}s"
        ) from error
    diagnostics = parse_cargo_messages(process.stdout)
    if process.returncode != 0 and not diagnostics:
        raise RepairError(
            f"crate_repair: cargo check failed with no parsable diagnostics (rc="
            f"{process.returncode}). Not a repairable state:\n{process.stderr[-2000:]}"
        )
    return CargoCheckRun(
        target=target,
        diagnostics=tuple(diagnostics),
        error_count=count_errors(diagnostics),
        returncode=process.returncode,
        stderr_tail=process.stderr[-2000:],
    )


def _file_reader(root: Path) -> FileReader:
    cache: dict[str, bytes | None] = {}

    def read(name: str) -> bytes | None:
        if name not in cache:
            path = root / name
            cache[name] = path.read_bytes() if path.is_file() else None
        return cache[name]

    return read


#: `fn NAME(params) [-> ret]` — extern-block decls and plain definitions alike. Bodies are
#: irrelevant here; only the parameter list is read.
_RE_FN_DECL = re.compile(rb"\bfn\s+(\w+)\s*(?:<([^>]*)>)?\s*\(", re.S)
_RE_TRY_INTO_UNWRAP = re.compile(rb"\.try_into\(\)\s*\.unwrap\(\)")


def _split_params(text: str) -> list[str]:
    """Top-level comma split of a parameter list (generics/tuples/fn-ptrs nest)."""
    out, depth, cur = [], 0, []
    for ch in text:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    if "".join(cur).strip():
        out.append("".join(cur).strip())
    return out


#: `#[no_mangle] pub .. fn NAME(` — a DEFINITION, in either the one-line or multi-line form.
_RE_FN_DEF = re.compile(
    rb"#\[no_mangle\]\s*(?:#\[[^\]]*\]\s*)*pub\s+(?:unsafe\s+)?"
    rb'(?:extern\s+"[A-Za-z-]+"\s+)?fn\s+(\w+)\s*\('
)


# A method's receiver is not an argument position: `obj.DefWindowProc(msg, w, l)` passes
# `w` as argument 1, which is parameter 2 of `fn DefWindowProc(&mut self, msg, w, l)`.
# Counting `&mut self` checked every WPARAM argument against `msg: UINT` (mspaint: 41).
_SELF_RECEIVER = re.compile(r"(?:&\s*(?:'\w+\s+)?(?:mut\s+)?|mut\s+)?self(?:\s*:.*)?", re.S)


def harvest_signatures(sources: Mapping[str, bytes]) -> dict[str, list[str]]:
    """``fn name -> [param type, ...]``, preferring the DEFINITION over declarations.

    A name declared twice with different parameter types is dropped, not merged: two
    disagreeing authorities are no authority, and ``clashing_extern_declarations`` says the
    shape is real here (828 symbols). That rule alone left 109 of 264 sites unresolvable —
    by far the largest skip bucket — because forward declarations in this tree routinely
    disagree with each other in spelling.

    But when the crate DEFINES the name with ``#[no_mangle]``, the disagreement is settled:
    that body is what the linker binds the call to, whatever the forward decls claim. So a
    definition overrides, and only decl-vs-decl conflicts with no definition stay dropped.
    (Same reasoning as verify_ffi's local-shim rule: an in-crate definition outranks an
    external description of the symbol.)
    """
    defined: dict[str, list[str]] = {}
    for data in sources.values():
        masked = _mask_lexical(data)
        for match in _RE_FN_DEF.finditer(data):
            open_paren = match.end() - 1
            close = _match_forward(masked, open_paren)
            if close is None:
                continue
            # `close` is the index PAST the `)`, so slice to `close - 1`. Including it left the
            # LAST parameter of every one-line declaration spelled `SIZE_T)`, which failed the
            # plain-identifier check in every consumer — so the final parameter of a wrapped-form
            # declaration was silently unresolvable, and nothing said so.
            raw = data[open_paren + 1 : close - 1].decode("utf-8", "replace")
            defined[match.group(1).decode("utf-8", "replace")] = [
                part.split(":", 1)[1].strip() if ":" in part else ""
                for part in _split_params(raw)
                if not _SELF_RECEIVER.fullmatch(part.strip())
            ]

    found: dict[str, list[str] | None] = {}
    for data in sources.values():
        masked = _mask_lexical(data)
        for match in _RE_FN_DECL.finditer(masked):
            open_paren = match.end() - 1
            close = _match_forward(masked, open_paren)
            if close is None:
                continue
            name = match.group(1).decode("utf-8", "replace")
            # `close` is the index PAST the `)`, so slice to `close - 1`. Including it left the
            # LAST parameter of every one-line declaration spelled `SIZE_T)`, which failed the
            # plain-identifier check in every consumer — so the final parameter of a wrapped-form
            # declaration was silently unresolvable, and nothing said so.
            raw = data[open_paren + 1 : close - 1].decode("utf-8", "replace")
            params = []
            for part in _split_params(raw):
                if _SELF_RECEIVER.fullmatch(part.strip()):
                    continue  # see _SELF_RECEIVER
                # `name: Type` -> Type; `...` carries no type.
                if ":" not in part:
                    params.append("")
                    continue
                params.append(part.split(":", 1)[1].strip())
            # A generic parameter is no type NAME to cast to: `fn min<T>(a: T, b: T)` once got
            # `(MAXLONG) as T` written into two pnpmgr arbiters. Unknown, so no authority.
            generics = {
                g.split(":", 1)[0].strip()
                for g in (match.group(2) or b"").decode("utf-8", "replace").split(",")
            }
            params = ["" if p in generics else p for p in params]
            if name in found and found[name] != params:
                found[name] = None
            else:
                found[name] = params
    resolved = {k: v for k, v in found.items() if v is not None}
    resolved.update(defined)  # the definition settles any decl-vs-decl disagreement
    return resolved


#: ``pub struct S {`` / ``pub union S {`` and its body, up to the matching column-0 ``}``.
#: bindgen and rustfmt both emit that shape; a nested brace inside a field type (``[T; N]``,
#: ``Option<unsafe extern "C" fn()>``) never starts at column 0, so the terminator is unambiguous.
_RE_TYPE_BLOCK = re.compile(r"^pub (?:struct|union) (\w+)\s*\{(.*?)^\}", re.S | re.M)
_RE_TYPE_FIELD = re.compile(r"^\s*pub (\w+)\s*:\s*(.+?),\s*$", re.M)
#: How deep a chain of nested anonymous unions to follow. prsht.h and objidl.h both nest twice;
#: the bound exists so a malformed island cannot make the walk unbounded.
_ANON_DEPTH = 3


def harvest_anon_unions(sources: Mapping[str, bytes]) -> dict[tuple[str, str], str]:
    """``(struct, flattened field) -> dotted path`` through bindgen's ``__bindgen_anon_N``.

    The authority for ``RULE_ANON_UNION``. Reads every ``pub struct``/``pub union`` the crate
    declares — which on a translated module means all three ``types_<arch>.rs`` islands plus any
    hand-written types — and walks each ``__bindgen_anon_N`` member to the fields C would have
    injected into the enclosing scope.

    Two guards, both of which are the difference between a repair and a corruption:

    * A field the outer struct declares ITSELF is never mapped. C's anonymous-member injection
      loses to an outer member of the same name, and so must this.
    * A ``(struct, field)`` that two islands resolve to DIFFERENT paths is dropped. The three
      arches are separate bindgen runs and their union numbering need not agree; two disagreeing
      authorities are no authority (same rule as ``harvest_signatures``).
    """
    per_source: list[dict[tuple[str, str], str]] = []
    #: ``(struct, field)`` pairs some island declares DIRECTLY. A field that is direct on one
    #: arch and unionised on another must not be routed: the edit would fix the arch that
    #: reported the error and break the one that did not. (The pass loop's regression guard
    #: would revert it, but a whole pass is a real cost and the answer is knowable here.)
    direct_anywhere: set[tuple[str, str]] = set()
    for data in sources.values():
        text = data.decode("utf-8", "replace")
        items: dict[str, list[tuple[str, str]]] = {}
        for match in _RE_TYPE_BLOCK.finditer(text):
            items[match.group(1)] = [
                (f.group(1), f.group(2).strip()) for f in _RE_TYPE_FIELD.finditer(match.group(2))
            ]
        if not items:
            continue
        direct = {name: {f for f, _ in fields} for name, fields in items.items()}
        for name, fields in direct.items():
            direct_anywhere.update((name, f) for f in fields)
        local: dict[tuple[str, str], str] = {}

        def walk(
            owner: str,
            container: str,
            prefix: str,
            depth: int,
            _items: dict[str, list[tuple[str, str]]] = items,
            _direct: dict[str, set[str]] = direct,
            _local: dict[tuple[str, str], str] = local,
        ) -> None:
            if depth > _ANON_DEPTH:
                return
            for field_name, field_type in _items.get(container, []):
                if field_name.startswith("__bindgen_anon"):
                    walk(owner, field_type, f"{prefix}{field_name}.", depth + 1)
                    continue
                if field_name in _direct[owner]:
                    continue  # the outer struct has its own; C would shadow it the same way
                _local.setdefault((owner, field_name), f"{prefix}{field_name}")

        for name, fields in items.items():
            for field_name, field_type in fields:
                if field_name.startswith("__bindgen_anon"):
                    walk(name, field_type, f"{field_name}.", 1)
        per_source.append(local)

    resolved: dict[tuple[str, str], str] = {}
    dropped: set[tuple[str, str]] = set()
    for local in per_source:
        for key, path in local.items():
            if resolved.get(key, path) != path:
                dropped.add(key)
            resolved[key] = path
    for key in dropped | direct_anywhere:
        resolved.pop(key, None)
    return resolved


def propose_panicking_cast_edits(
    file_name: str, data: bytes, signatures: Mapping[str, Sequence[str]]
) -> tuple[list[Edit], list[Skipped]]:
    """Replace ``X.try_into().unwrap()`` with ``X as <T>``, where <T> is READ OFF the callee.

    C's implicit narrowing is silent; ``try_into().unwrap()`` makes it a panic. In a driver or
    a live service that converts a benign truncation into a bugcheck or a dead service, which
    is why ``crate-repair`` refuses to EMIT the shape (``PANICKING_SUGGESTIONS``). This rule
    removes the ones already in the tree — 264 of them across 25 crates, none of which rustc
    will ever complain about, because the code is perfectly well-typed. It is only wrong.

    The target type is never inferred. It comes from one of two syntactic authorities:
      * argument position — the callee's declared parameter type at that index;
      * ``let name: T = ...`` — the ascription.
    Anything else (a `return`, a tail expression, a generic parameter, a callee declared
    inconsistently) is SKIPPED with the reason recorded. Guessing the width here would
    reintroduce exactly the silent truncation the panic was hiding.
    """
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    masked = _mask_lexical(data)
    for match in _RE_TRY_INTO_UNWRAP.finditer(masked):
        start, end = match.start(), match.end()
        line = _line_of(data, start)

        def _skip(reason: str, detail: str, line: int = line) -> None:
            skipped.append(Skipped(RULE_PANICKING_CAST, reason, file_name, line, detail))

        # Walk out to the nearest unclosed `(` — that is the enclosing call, if any.
        depth, index, open_paren = 0, start - 1, None
        while index >= 0:
            char = masked[index : index + 1]
            if char in b")]}":
                depth += 1
            elif char in b"([{":
                if depth == 0:
                    open_paren = index if char == b"(" else None
                    break
                depth -= 1
            elif char == b";" and depth == 0:
                break
            index -= 1

        target: str | None = None
        authority = ""
        if open_paren is not None:
            head = masked[:open_paren].rstrip()
            name_match = re.search(rb"(\w+)$", head)
            callee = name_match.group(1).decode() if name_match else None
            if callee and callee in signatures:
                # `<`/`>` must NOT count as nesting: `if (CurrentOffset >= size_of::<T>())`
                # is a comparison, and treating it as a bracket desynchronises the depth for
                # the rest of the argument list — which silently shifts the argument index
                # and would read the WRONG parameter's type off the callee.
                arg_index = 0
                depth = 0
                for pos in range(open_paren + 1, start):
                    char = masked[pos : pos + 1]
                    if char in b"([{":
                        depth += 1
                    elif char in b")]}":
                        depth -= 1
                    elif char == b"," and depth == 0:
                        arg_index += 1
                params = signatures[callee]
                if arg_index < len(params) and params[arg_index]:
                    target = params[arg_index]
                    authority = f"{callee}() parameter {arg_index}: `{target}`"
                else:
                    _skip(
                        "callee_arity",
                        f"{callee}() declares {len(params)} params; this is argument {arg_index}",
                    )
                    continue
            else:
                _skip(
                    "callee_unresolved",
                    f"no unambiguous `fn {callee}` declaration in this crate"
                    if callee
                    else "enclosing call has no simple callee name",
                )
                continue
        else:
            # Statement context: only a `let name: T =` ascription is an authority.
            line_start = data.rfind(b"\n", 0, start) + 1
            prefix = data[line_start:start].decode("utf-8", "replace")
            ascription = re.match(r"\s*let\s+(?:mut\s+)?\w+\s*:\s*([^=]+?)\s*=", prefix)
            if ascription:
                target = ascription.group(1).strip()
                authority = f"`let` ascription: `{target}`"
            else:
                _skip(
                    "no_type_authority",
                    "not an argument to a declared fn and no `let x: T =` ascription — "
                    "the target width would have to be guessed",
                )
                continue

        if target is None or not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", target):
            _skip("non_scalar_target", f"target type `{target}` is not a plain scalar name")
            continue
        if target not in INT_PRIMITIVES:
            # The cast is written in the CALLER's scope but the type name was read from the
            # CALLEE's. An alias that resolves differently in the two therefore produces a
            # well-formed cast to the wrong type. Real hit: rtl-misc spells `SIZE_T` = usize
            # while ntkrnl-sys's amd64 bindgen declares NtReadVirtualMemory's parameter as
            # u64 — and usize and u64 are DISTINCT types to rustc even at equal width, so
            # `as SIZE_T` compiled in the caller and failed at the call (E0308, gate RED
            # 2026-07-25). Only the primitives mean the same thing in every scope.
            _skip(
                "alias_target",
                f"`{target}` is a type alias, not a primitive — it may resolve differently "
                f"in this crate than in the callee's",
            )
            continue
        edits.append(
            Edit(
                file_name=file_name,
                byte_start=start,
                byte_end=end,
                replacement=f" as {target}",
                rule=RULE_PANICKING_CAST,
                authority=authority,
                original=data[start:end].decode("utf-8", "replace"),
                line=line,
            )
        )
    return edits, skipped


_RE_CARGO_PATH_DEP = re.compile(r'path\s*=\s*"([^"]+)"')


def _path_dep_dirs(crate_dir: Path) -> list[Path]:
    """Sibling crates reachable by `path = "..."`, one level deep.

    The callee is very often NOT in the crate being repaired: `RtlAllocateHeap` and friends
    are declared in the `-sys` island next door. Harvesting only the crate's own sources left
    85 of 264 sites unresolvable purely for want of looking one directory over.
    """
    cargo = crate_dir / "Cargo.toml"
    if not cargo.is_file():
        return []
    out = []
    for rel in _RE_CARGO_PATH_DEP.findall(cargo.read_text(encoding="utf-8", errors="replace")):
        dep = (crate_dir / rel).resolve()
        if (dep / "Cargo.toml").is_file():
            out.append(dep)
    return out


def _panicking_cast_edits_for_crate(
    crate_dir: Path, root: Path
) -> tuple[list[Edit], list[Skipped]]:
    sources: dict[str, bytes] = {}
    for path in sorted((crate_dir / "src").rglob("*.rs")):
        try:
            relative = path.relative_to(root).as_posix()
        except ValueError:
            relative = path.as_posix()
        sources[relative] = path.read_bytes()
    # Declarations from dependencies are authority-only: they widen what we can resolve,
    # they are never themselves edited (the edit loop only walks `sources`).
    decl_sources = dict(sources)
    for dep in _path_dep_dirs(crate_dir):
        for path in sorted((dep / "src").rglob("*.rs")):
            decl_sources[f"dep:{path}"] = path.read_bytes()
    signatures = harvest_signatures(decl_sources)
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    for relative, data in sources.items():
        file_edits, file_skipped = propose_panicking_cast_edits(relative, data, signatures)
        edits.extend(file_edits)
        skipped.extend(file_skipped)
    return edits, skipped


#: ``pub type NAME = ...;`` — an alias the crate declares outright.
_RE_TYPE_ALIAS = re.compile(rb"^\s*pub\s+type\s+(\w+)\s*=", re.M)
#: ``name: Type`` in a declaration — a type name the crate already uses somewhere.
_RE_USED_TYPE = re.compile(rb"\b\w+\s*:\s*([A-Z]\w*)\s*[,)]")


def _local_anon_unions(crate_dir: Path) -> dict[tuple[str, str], str]:
    """``RULE_ANON_UNION``'s authority: type declarations from THIS crate ONLY.

    Same provenance rule as ``_local_authorities`` — the edit writes a field path that has to
    resolve in the caller's scope, and a sibling crate's ``_PROPSHEETHEADERW`` need not be this
    crate's. All three ``types_<arch>.rs`` islands are read together; ``harvest_anon_unions``
    drops any field the arches route differently.
    """
    sources = {
        path.as_posix(): path.read_bytes() for path in sorted((crate_dir / "src").rglob("*.rs"))
    }
    unions = harvest_anon_unions(sources)
    # The same one-link extension as `_local_authorities`: a usermode island's PUBLIC
    # `pub use ntdll_sys::nt::*;` is the path `_IO_STATUS_BLOCK`/`_LARGE_INTEGER`/
    # `_DISK_PARTITION_INFO` reach the caller by, so their anonymous unions route the same
    # way there. setuplib had ~50 `.Status`/`.LowPart`/`.Mbr` errors on ntdll-sys types.
    islands = [d for p, d in sources.items() if re.search(r"/types_[a-z0-9]+\.rs$", p)]
    if any(re.search(rb"^pub use ntdll_sys::nt::\*;", d, re.M) for d in islands):
        sys_src = crate_dir.parent / "ntdll-sys" / "src"
        sys_sources = {p.as_posix(): p.read_bytes() for p in sorted(sys_src.glob("bindgen_*.rs"))}
        for key, path in harvest_anon_unions(sys_sources).items():
            unions.setdefault(key, path)
    # A kernel island's `use ntkrnl_sys::ddk::*;` is private; the crate PRELUDE re-exports it
    # (`rustconv port` crates), which is the same one link for `_LARGE_INTEGER.LowPart` & co.
    prelude = sources.get((crate_dir / "src" / "prelude.rs").as_posix(), b"")
    if re.search(rb"^pub use ntkrnl_sys::ddk::\*;", prelude, re.M):
        sys_src = crate_dir.parent / "ntkrnl-sys" / "src"
        sys_sources = {p.as_posix(): p.read_bytes() for p in sorted(sys_src.glob("bindgen_*.rs"))}
        for key, path in harvest_anon_unions(sys_sources).items():
            unions.setdefault(key, path)
    return unions


_RE_PLAIN_ALIAS = re.compile(r"^pub type (\w+) = (\w+);", re.M)

#: ``[pub[(..)]] type NAME[<..>] =`` at line start: an alias declaration of any visibility.
_RE_ANY_ALIAS = re.compile(r"^[ \t]*(pub\b(?:\([^)]*\))?\s+)?type\s+(\w+)\b[^=;]*=", re.M)
#: ``[pub] use PATH;`` with PATH possibly a one-level ``{a, b::*}`` group.
_RE_USE = re.compile(r"^[ \t]*(pub\b(?:\([^)]*\))?\s+)?use\s+([^;]+);", re.M)


#: ``[#[path = "P"]] [pub] mod NAME;`` — an out-of-line module, possibly at a #[path].
_RE_MOD = re.compile(
    r"^[ \t]*((?:#\[[^\]]*\][ \t]*)*)(?:pub\b(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;", re.M
)
_RE_PATH_ATTR = re.compile(r'#\[path\s*=\s*"([^"]+)"\]')


def _module_of(rel: str) -> tuple[str, ...]:
    """``src/a/b.rs`` / ``src/a/b/mod.rs`` -> ``("a", "b")``; ``src/lib.rs`` -> ``()``."""
    parts = rel.removeprefix("src/").removesuffix(".rs").split("/")
    if parts[-1] in ("mod", "lib", "main"):
        parts = parts[:-1]
    return tuple(p for p in parts if p)


def _use_targets(
    path: str, here: tuple[str, ...], children: set[str]
) -> list[tuple[tuple[str, ...] | None, str]]:
    """``use`` PATH -> ``[(module, leaf)]``; leaf is ``*`` or the name it binds (the ``as``
    rename when there is one). ``children`` are the modules ``here`` declares, which a bare
    first segment names. A path outside the crate (``ntdll_sys::..``) has module ``None``:
    its glob is unknowable here, but a name it imports is still bound."""
    path = re.sub(r"\s+", " ", path).strip()
    m = re.fullmatch(r"(.*?)::\s*\{(.*)\}", path)
    leaves = (
        [f"{m.group(1)}::{x.strip()}" for x in m.group(2).split(",") if x.strip()] if m else [path]
    )
    out = []
    for leaf in leaves:
        path_part, _, rename = leaf.partition(" as ")
        segs = [x.strip() for x in path_part.split("::")]
        bound = rename.strip() or segs[-1]
        if segs[0] == "crate":
            base, rest = (), segs[1:]
        elif segs[0] in ("self", "super"):
            base = here
            while segs and segs[0] in ("self", "super"):
                base = base[:-1] if segs.pop(0) == "super" else base
            rest = segs
        elif segs[0] in children:
            base, rest = here, segs
        else:
            if bound != "*":
                out.append((None, bound))
            continue
        if rest:
            out.append(((*base, *rest[:-1]), bound if rest[-1] != "*" else "*"))
    return out


def harvest_alias_scope(sources: Mapping[str, bytes]) -> dict[str, frozenset[str]]:
    """``alias -> {crate-relative file that can name it}`` for every alias the crate declares
    only privately (``type X = ..;`` without ``pub`` in every declaration).

    ``sources`` is keyed ``src/<path>.rs``. A file sees an alias it declares, names in a
    ``use``, or reaches by a glob over a module that declares it (any glob for ``pub`` /
    ``pub(..)``; ``super::*`` also for private) or re-exports it with ``pub use``. Modules
    mounted at a ``#[path]`` (the per-arch ``types_arch`` islands) count as that module.
    That is the subset of Rust's resolution translated crates use; an alias reached some
    other way reads as out of scope, so the rule declines rather than writing a name that
    does not resolve.
    """
    texts = {rel: data.decode("utf-8", "replace") for rel, data in sources.items()}
    module = {rel: _module_of(rel) for rel in texts}
    children: dict[tuple[str, ...], set[str]] = {}
    for rel, text in texts.items():
        for m in _RE_MOD.finditer(text):
            children.setdefault(module[rel], set()).add(m.group(2))
    # A #[path] is relative to the declaring file's directory (all our uses are outside
    # inline `mod {}` blocks). Applied after the scan: the mounted file's own children are
    # keyed by the module it was first given, so remap those too.
    for rel, text in list(texts.items()):
        for m in _RE_MOD.finditer(text):
            if attr := _RE_PATH_ATTR.search(m.group(1)):
                target = str(Path(rel).parent / attr.group(1)).replace("\\", "/")
                if target in module:
                    module[target] = (*module[rel], m.group(2))
    declared: dict[tuple[str, ...], dict[str, bool]] = {}
    uses: dict[tuple[str, ...], list[tuple[bool, tuple[str, ...] | None, str]]] = {}
    for rel, text in texts.items():
        mod = module[rel]
        for m in _RE_ANY_ALIAS.finditer(text):
            d = declared.setdefault(mod, {})
            d[m.group(2)] = d.get(m.group(2), False) or bool(m.group(1))
        uses.setdefault(mod, []).extend(
            (bool(m.group(1)), target, leaf)
            for m in _RE_USE.finditer(text)
            for target, leaf in _use_targets(m.group(2), mod, children.get(mod, set()))
        )

    def exported(mod: tuple[str, ...], seen: frozenset = frozenset()) -> set[str]:
        """Names a glob over ``mod`` brings in: its non-private aliases and its ``pub use``s."""
        if mod in seen:
            return set()
        names = {n for n, public in declared.get(mod, {}).items() if public}
        for public, target, leaf in uses.get(mod, []):
            if public:
                names |= (
                    {leaf} if leaf != "*" else exported(target, seen | {mod}) if target else set()
                )
        return names

    # Only aliases the crate declares PRIVATELY everywhere. A name with any `pub` declaration
    # is usually also the -sys island's (`BOOLEAN`, `WPARAM`), reaching files through
    # external globs this harvest cannot follow: measured over the tree, scoping those
    # wrongly refused 7,950 of 72,926 uses that compile.
    all_declared = {n for d in declared.values() for n in d} - {
        n for d in declared.values() for n, public in d.items() if public
    }
    scope: dict[str, set[str]] = {n: set() for n in all_declared}
    for rel in texts:
        mod = module[rel]
        visible = set(declared.get(mod, {}))
        for _, target, leaf in uses.get(mod, []):
            if leaf != "*":
                visible.add(leaf)
            elif target is None:
                continue
            elif mod and target == mod[:-1]:  # `use super::*` also reaches private items
                visible |= set(declared.get(target, {})) | exported(target)
            else:
                visible |= exported(target)
        for n in visible & all_declared:
            scope[n].add(rel)
    return {n: frozenset(v) for n, v in scope.items()}


#: A type a -sys crate declares, or re-exports under a name (bindgen's
#: ``pub use self::_NOTIFICATION_TYPE as NOTIFICATION_TYPE;`` for an enum typedef).
_RE_SYS_TYPE = re.compile(
    rb"^\s*pub\s+(?:(?:type|struct|union|enum)\s+(\w+)|use\s+[\w:]+\s+as\s+(\w+)\s*;)", re.M
)


@functools.lru_cache(maxsize=4)
def _sys_type_names(rust_root: Path) -> frozenset[str]:
    """Every type name a ``*-sys`` crate declares: names a file may reach by an external glob."""
    names: set[str] = set()
    for path in sorted(rust_root.glob("*-sys/src/**/*.rs")):
        for m in _RE_SYS_TYPE.finditer(path.read_bytes()):
            names.add((m.group(1) or m.group(2)).decode("utf-8", "replace"))
    return frozenset(names)


def _local_alias_scope(crate_dir: Path) -> dict[str, frozenset[str]]:
    """``harvest_alias_scope`` over the crate, less the SDK names a ``-sys`` crate also
    declares: a module's private ``type BOOLEAN = u8;`` says nothing about where else
    ``BOOLEAN`` resolves (790 compiling uses in the tree reach those through globs)."""
    src = crate_dir / "src"
    scope = harvest_alias_scope(
        {f"src/{p.relative_to(src).as_posix()}": p.read_bytes() for p in sorted(src.rglob("*.rs"))}
    )
    sdk = _sys_type_names(crate_dir.resolve().parent)
    return {n: v for n, v in scope.items() if n not in sdk}


def harvest_field_types(sources: Mapping[str, bytes]) -> dict[str, frozenset[str]]:
    """``field name -> {canonical type of every field so named}`` across the sources.

    A `pub type A = B;` chain is followed to the struct it names, so `AtaScsiAddress:
    ATA_SCSI_ADDRESS` reads as `_ATA_SCSI_ADDRESS`, the key the anon-union map uses.
    """
    aliases: dict[str, str] = {}
    fields: dict[str, set[str]] = {}
    for data in sources.values():
        text = data.decode("utf-8", "replace")
        aliases.update(_RE_PLAIN_ALIAS.findall(text))
        for block in _RE_TYPE_BLOCK.finditer(text):
            for f in _RE_TYPE_FIELD.finditer(block.group(2)):
                fields.setdefault(f.group(1), set()).add(f.group(2).strip())

    def canonical(t: str) -> str:
        for _ in range(6):
            if t not in aliases:
                break
            t = aliases[t]
        return t

    return {name: frozenset(canonical(t) for t in types) for name, types in fields.items()}


def _local_field_types(crate_dir: Path) -> dict[str, frozenset[str]]:
    """``harvest_field_types`` over the crate's own sources (the islands it declares)."""
    # Imported here, with crate_integrate's fallback: rustconv also runs with cwd
    # tools/rustconv, where `tools.` is not importable.
    try:
        from tools.source_walk import source_files
    except ModuleNotFoundError:
        import sys

        sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
        from tools.source_walk import source_files

    return harvest_field_types(
        {p.as_posix(): p.read_bytes() for p in sorted(source_files(crate_dir / "src"))}
    )


def _local_authorities(crate_dir: Path) -> tuple[dict[str, list[str]], frozenset[str]]:
    """``RULE_WIDTH_ALIAS``'s authority: signatures and type names from THIS crate ONLY.

    Deliberately narrower than ``_panicking_cast_edits_for_crate``'s harvest, which widens with
    ``path =`` dependencies. A dependency's declaration is fine to READ a width off, but this
    rule copies a NAME into the caller's scope, and a sibling crate's ``SIZE_T`` need not be
    this crate's — that is the bug that rule ate. Same crate, same island, one meaning.

    WHAT THE NAME SET PROVES, AND WHAT IT DOES NOT. Most of these aliases are not DECLARED by
    the crate at all: the island opens with ``pub use ntdll_sys::nt::*;``, so ``SIZE_T`` is
    glob-reachable rather than written down, and a declarations-only sweep finds none of them
    and disables the rule completely. What is actually true is narrower and sufficient: the
    caller can already name the CALLEE, and the callee arrives through the same glob chain
    (``prelude::*`` -> ``types::*`` -> island -> ``ntdll_sys::nt::*``) that carries its
    parameter types, so a name that resolves in the island resolves identically in the caller.
    This set is therefore a sanity filter — "a type name this crate already uses" — not a proof
    of scope. The proof is the pass loop: a repair pass that raises the error count on ANY
    target is reverted whole, so a name that does not resolve costs a pass, never a silent
    mis-cast.
    """
    sources: dict[str, bytes] = {}
    names: set[str] = set()
    for path in sorted((crate_dir / "src").rglob("*.rs")):
        data = path.read_bytes()
        sources[path.as_posix()] = data
        names.update(m.group(1).decode("utf-8", "replace") for m in _RE_TYPE_ALIAS.finditer(data))
        names.update(m.group(1).decode("utf-8", "replace") for m in _RE_USED_TYPE.finditer(data))
    signatures = harvest_signatures(sources)
    # The SAME glob chain, one link further: a usermode island opens with the PUBLIC
    # `pub use ntdll_sys::nt::*;`, so ntdll-sys's `RtlAllocateHeap(.., Size: SIZE_T)` and its
    # `SIZE_T` reach the caller through one path, exactly as the argument above requires.
    # setuplib's 23 `callee_not_local` declines were all of this shape. Local declarations
    # still win, and a kernel island's glob is private (`use`), so it is never followed.
    islands = [sources[p] for p in sources if re.search(r"/types_[a-z0-9]+\.rs$", p)]
    if any(re.search(rb"^pub use ntdll_sys::nt::\*;", d, re.M) for d in islands):
        sys_src = crate_dir.parent / "ntdll-sys" / "src"
        sys_sources = {p.as_posix(): p.read_bytes() for p in sorted(sys_src.glob("bindgen_*.rs"))}
        for name, params in harvest_signatures(sys_sources).items():
            signatures.setdefault(name, params)
        for data in sys_sources.values():
            names.update(
                m.group(1).decode("utf-8", "replace") for m in _RE_TYPE_ALIAS.finditer(data)
            )
    return signatures, frozenset(names)


def _flex_edits_for_crate(
    crate_dir: Path, root: Path, decls: FlexArrayDecls
) -> tuple[list[Edit], list[Skipped]]:
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    for path in sorted((crate_dir / "src").rglob("*.rs")):
        if path.name == decls.source:
            continue
        try:
            relative = path.relative_to(root).as_posix()
        except ValueError:
            relative = path.as_posix()
        file_edits, file_skipped = propose_flex_array_edits(relative, path.read_bytes(), decls)
        edits.extend(file_edits)
        skipped.extend(file_skipped)
    return edits, skipped


def repair_crate(
    crate: str,
    *,
    root: Path,
    targets: Sequence[str],
    rules: Collection[str] = ALL_RULES,
    crate_dir: Path | None = None,
    types_module: str = "types.rs",
    max_passes: int = 6,
    allow_single_target_width: bool = False,
    dry_run: bool = False,
    timeout: int = 1800,
    check_runner: Callable[[str], CargoCheckRun] | None = None,
    target_dirs: Mapping[str, Path] | None = None,
) -> RepairSession:
    """Iterate propose/apply to a fixpoint, never leaving the tree worse than we found it.

    One ``cargo check`` per target per pass; the NEXT pass's check is the verification of the
    previous one.  If the total error count rises after a pass, that pass's bytes are restored
    and the session aborts with ``status="regressed"`` — a bounded pass must never make a crate
    less compilable than it started.

    ``check_runner`` (target -> ``CargoCheckRun``) exists so the abort invariant can be tested
    without a real cargo tree; it defaults to a real ``cargo check`` of ``crate``.
    """
    session = RepairSession(
        crate=crate,
        targets=tuple(targets),
        rules=tuple(sorted(rules)),
        dry_run=dry_run,
    )
    if not targets:
        raise RepairError("crate_repair: at least one target is required")
    resolved_dir = crate_dir or (root / crate)
    decls = FlexArrayDecls()
    flex_note: Skipped | None = None
    if RULE_FLEX_ARRAY in rules:
        # `types.rs` in this repo is a 6-line #[cfg] DISPATCH STUB; the real bindgen output —
        # and every `[T; 1]` flexible-array member — lives in `types_<arch>.rs`. Parsing the
        # stub finds zero fields on EVERY crate here (iphlpapi alone declares 32 such fields
        # in types_i386.rs), so the rule would contribute nothing while looking healthy.
        # Resolve the per-arch modules for the targets under analysis; an explicit
        # --types-module always wins.
        types_paths: list[Path] = []
        if types_module == "types.rs":
            types_paths = [
                path
                for path in (resolved_dir / "src" / f"types_{target}.rs" for target in targets)
                if path.is_file()
            ]
        if not types_paths:
            types_paths = [resolved_dir / "src" / types_module]

        present = [path for path in types_paths if path.is_file()]
        if present:
            merged_fields: dict[str, str] = {}
            merged_ambiguous: set[str] = set()
            for path in present:
                part = flex_array_decls(
                    path.read_text(encoding="utf-8", errors="replace"), source_name=path.name
                )
                merged_fields.update(part.fields)
                merged_ambiguous.update(part.ambiguous)
            # A field ambiguous on ANY arch is ambiguous everywhere: layouts differ per arch
            # and one rewrite has to be correct on all three.
            for name in merged_ambiguous:
                merged_fields.pop(name, None)
            decls = FlexArrayDecls(
                fields=merged_fields,
                ambiguous=frozenset(merged_ambiguous),
                source=", ".join(path.name for path in present),
            )
            if not decls.fields:
                # Say so out loud: "no flexible-array members" and "the parser found nothing"
                # look identical from the outside, and only one of them is good news.
                flex_note = Skipped(
                    RULE_FLEX_ARRAY,
                    "no_flex_array_members",
                    decls.source,
                    0,
                    f"{decls.source} declares no `[T; 1]` field (ambiguous: "
                    f"{', '.join(sorted(decls.ambiguous)) or 'none'}) — rule contributed nothing.",
                )
        else:
            flex_note = Skipped(
                RULE_FLEX_ARRAY,
                "no_bindgen_types_module",
                types_module,
                0,
                f"{', '.join(str(path) for path in types_paths)} not found — the `[T; 1]` "
                "authority is unavailable, so the flexible-array rule is DISABLED for this "
                "run (not guessed).",
            )
    diagnostic_rules = [name for name in rules if name in DIAGNOSTIC_RULES]
    check = check_runner or (
        lambda target: run_cargo_check(
            crate,
            target,
            root=root,
            timeout=timeout,
            target_dir=(target_dirs or {}).get(target),
        )
    )
    snapshot: dict[str, bytes] = {}
    previous_total: int | None = None
    previous_halting: int | None = None
    # (file, text) for every replacement a pass wrote. A rule asked to edit text a repair
    # wrote in an earlier pass is oscillating: that answer did not hold, and answering again
    # builds a ladder (`((x != 0) != 0)`, `((x) as i32) as u32`) one pass at a time while
    # the error count stays flat. TWO rules can trade a site the same way -- oleaut32's
    # variadic_promotion and truncating_numeric_cast stacked `as c_uint) as u16` ten deep
    # -- so the key is the text, not the rule. Declining it lets the run reach a fixpoint.
    produced: set[tuple[str, str]] = set()
    for index in range(1, max_passes + 1):
        if target_dirs and all(t in target_dirs for t in targets):
            # Separate build dirs: the arches really run at once (3x on every pass).
            from concurrent.futures import ThreadPoolExecutor

            with ThreadPoolExecutor(max_workers=len(targets)) as pool:
                runs = dict(zip(targets, pool.map(check, targets), strict=True))
        else:
            runs = {target: check(target) for target in targets}
        counts = {target: run.error_count for target, run in runs.items()}
        total = sum(counts.values())
        halting = sum(count_halting_errors(run.diagnostics) for run in runs.values())
        current = RepairPass(index=index, error_counts=counts)
        if previous_halting is not None and halting > previous_halting:
            restore_snapshot(snapshot, root)
            session.passes.append(current)
            session.status = "regressed"
            session.detail = (
                f"pass {index - 1} added parse/expansion errors ({previous_halting} -> "
                f"{halting}), which hide every later error -- its total of {total} is not a "
                "real count. Its edits were REVERTED; fix the rule (or the macro it fed)."
            )
            return session
        if previous_total is not None and total > previous_total:
            restore_snapshot(snapshot, root)
            session.passes.append(current)
            session.status = "regressed"
            session.detail = (
                f"pass {index - 1} raised the error count {previous_total} -> {total}; its edits "
                "were REVERTED. Inspect that pass's ledger and fix the rule before rerunning."
            )
            return session
        # A FRESH reader every pass: the previous pass rewrote these files, and every guard
        # (callee_span/already_cast/balance) must judge the bytes rustc just parsed.
        reader = _file_reader(root)
        # Re-harvested each pass alongside the reader, for the same reason: the previous pass
        # rewrote these files.
        local_sigs, local_aliases = (
            _local_authorities(resolved_dir) if RULE_WIDTH_ALIAS in rules else ({}, frozenset())
        )
        alias_scope = _local_alias_scope(resolved_dir) if RULE_WIDTH_ALIAS in rules else {}
        anon_unions = _local_anon_unions(resolved_dir) if RULE_ANON_UNION in rules else {}
        field_types = _local_field_types(resolved_dir) if RULE_ANON_UNION in rules else {}
        constants = local_constants(resolved_dir) if RULE_CONST_OPERAND in rules else {}
        try:
            own_prefix = resolved_dir.resolve().relative_to(root.resolve()).as_posix() + "/"
        except ValueError:
            own_prefix = ""
        plans = {
            target: propose_edits(
                run.diagnostics,
                reader,
                target=target,
                rules=diagnostic_rules,
                signatures=local_sigs,
                local_aliases=local_aliases,
                alias_scope=alias_scope,
                anon_unions=anon_unions,
                constants=constants,
                own_prefix=own_prefix,
                field_types=field_types,
            )
            for target, run in runs.items()
        }
        merged = merge_plans(plans, allow_single_target_width=allow_single_target_width)
        candidates = list(merged.edits)
        current.withheld.extend(merged.withheld)
        current.skipped.extend(merged.skipped)
        current.red.extend(merged.red)
        if flex_note is not None:
            current.skipped.append(flex_note)
        if decls.fields:
            flex_edits, flex_skipped = _flex_edits_for_crate(resolved_dir, root, decls)
            candidates.extend(flex_edits)
            current.skipped.extend(flex_skipped)
        if RULE_PANICKING_CAST in rules:
            panic_edits, panic_skipped = _panicking_cast_edits_for_crate(resolved_dir, root)
            candidates.extend(panic_edits)
            current.skipped.extend(panic_skipped)
        fresh: list[Edit] = []
        for edit in candidates:
            if edit.original.strip() and (edit.file_name, edit.original.strip()) in produced:
                current.skipped.append(
                    Skipped(
                        edit.rule,
                        "reapplied_own_edit",
                        edit.file_name,
                        edit.line,
                        f"`{edit.original}` is a repair's own output from an earlier pass; "
                        f"re-editing it to `{edit.replacement}` would oscillate. The site "
                        "needs a different rule or a human.",
                    )
                )
            else:
                fresh.append(edit)
        candidates = fresh
        selected, dropped = select_non_overlapping(candidates)
        current.skipped.extend(dropped)
        if not selected:
            session.passes.append(current)
            session.status = "fixpoint"
            session.detail = f"no further deterministic edits; {total} errors remain"
            session.final_red = list(merged.red)
            return session
        if dry_run:
            current.applied.extend(selected)
            session.passes.append(current)
            session.status = "dry_run"
            session.detail = f"{len(selected)} edits proposed, nothing written"
            session.final_red = list(merged.red)
            return session
        applied, snapshot, failed = apply_edits(selected, root, within=resolved_dir)
        produced.update((e.file_name, e.replacement.strip()) for e in applied)
        current.applied.extend(applied)
        current.skipped.extend(failed)
        session.passes.append(current)
        session.final_red = list(merged.red)
        if not applied:
            # Every proposal was refused at write time (outside the crate, or stale): the next
            # pass sees the same diagnostics and proposes the same edits. setuplib spent five
            # passes (~30 s each, three cargo checks) re-proposing them.
            session.status = "fixpoint"
            session.detail = (
                f"{len(failed)} proposed edit(s) refused at write; {total} errors remain"
            )
            return session
        previous_total = total
        previous_halting = halting
    session.status = "max_passes"
    session.detail = f"stopped after {max_passes} passes without reaching a fixpoint"
    return session


def print_session(session: RepairSession, *, as_json: bool = False, limit: int = 20) -> None:
    if as_json:
        print(json.dumps(session.to_dict(), indent=2, sort_keys=True))
        return
    print(f"crate-repair {session.crate}: status={session.status} ({session.detail})")
    print(f"  targets: {', '.join(session.targets)}   rules: {', '.join(session.rules)}")
    for item in session.passes:
        counts = ", ".join(f"{k}={v}" for k, v in sorted(item.error_counts.items()))
        print(
            f"  pass {item.index}: errors[{counts}] applied={len(item.applied)} "
            f"withheld={len(item.withheld)} skipped={len(item.skipped)}"
        )
        for edit in item.applied[:limit]:
            print(
                f"    + {edit.rule} {edit.file_name}:{edit.line} `{edit.original}` -> "
                f"`{edit.replacement}`"
            )
        if len(item.applied) > limit:
            print(f"    ... {len(item.applied) - limit} more (use --json for the full ledger)")
        for held in item.withheld[:limit]:
            print(f"    ! {held.rule}/{held.reason} {held.file_name}:{held.line} {held.detail}")
        guards: dict[str, int] = {}
        for guard in item.skipped:
            key = f"{guard.rule}/{guard.reason}"
            guards[key] = guards.get(key, 0) + 1
        for key, count in sorted(guards.items(), key=lambda kv: (-kv[1], kv[0])):
            print(f"    - declined {key}: {count}")
    shapes: dict[str, int] = {}
    for red in session.final_red:
        shapes[red.shape] = shapes.get(red.shape, 0) + 1
    if shapes:
        print("  RED (left for human review, never guessed):")
        for shape, count in sorted(shapes.items(), key=lambda kv: (-kv[1], kv[0]))[:limit]:
            print(f"    {shape}: {count}")
