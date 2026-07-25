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

import json
import re
import subprocess
from collections.abc import Callable, Collection, Mapping, Sequence
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any

from .config import RUST_TARGETS

# --- rules ------------------------------------------------------------------------------------

RULE_NUMERIC_CAST = "truncating_numeric_cast"
RULE_NO_STD_PATH = "no_std_path"
RULE_POINTER_CAST = "raw_pointer_cast"
RULE_FLEX_ARRAY = "flex_array_index"
RULE_PANICKING_CAST = "panicking_cast"

#: Rules driven by rustc diagnostics.
DIAGNOSTIC_RULES: tuple[str, ...] = (RULE_NUMERIC_CAST, RULE_NO_STD_PATH, RULE_POINTER_CAST)
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


def _mask_lexical(data: bytes) -> bytes:
    """Blank comments, string literals and char literals, preserving every byte offset.

    Byte-exact on purpose: rustc reports BYTE offsets, and translated sources routinely carry
    non-ASCII in comments, so any str round-trip would shift the offsets we splice at.
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
    for span in diagnostic.get("spans") or []:
        match = _LABEL_MISMATCH.match(str(span.get("label") or ""))
        if match is None:
            continue
        sites.append((dict(span), match.group("expected"), match.group("found")))
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


def _in_match_pattern(masked: bytes, end: int) -> bool:
    """True when the span sits in a `match` arm PATTERN, where a cast is not grammatical.

    rustc reports a constant arm whose type disagrees with the scrutinee exactly like any
    other integer mismatch, so the cast rule fires and writes `(_MB_CP_SBCS) as i32 => {` —
    which is not a pattern and does not parse. (Seen on msvcrt's `_setmbcp_l`, where bindgen
    typed one member of a four-macro `#define` block `u32` because its value is 0 and the
    other three `i32` because theirs are negative.)

    Scans forward on the same line: a `=>` reached before any `;`, `{` or `}` means everything
    to our left is a pattern. A one-line `match x { A => 1, B => 2 }` can trip this on an arm
    BODY and decline a legitimate cast — the safe direction for a rule that never guesses.
    """
    for i in range(end, min(len(masked), end + 200)):
        if masked[i : i + 2] == b"=>":
            return True
        if masked[i : i + 1] in (b"\n", b";", b"{", b"}"):
            return False
    return False


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
        hint = hints.get(end)
        if hint is not None and hint != expected:
            decline("hint_label_disagreement", f"label={expected} hint={hint}")
            continue
        data = load(file_name)
        if data is None:
            decline("unreadable_file")
            continue
        masked = _mask_lexical(data)
        if _next_code_byte(masked, end) == b"(":
            # Guard: a span followed by `(` is a CALLEE name (the primary span of the
            # "arguments to this function are incorrect" shape), not a value to cast.
            decline("callee_span", data[start:end].decode("utf-8", "replace"))
            continue
        if _in_match_pattern(masked, end):
            # Guard: a cast is not a pattern. The type disagreement is real, but it has to be
            # resolved where the constant is DECLARED, not at the arm.
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


def _pointer_cast_rule(
    diagnostic: Mapping[str, Any], load: FileReader, ctx: _Ctx
) -> tuple[list[Edit], list[Skipped], bool]:
    edits: list[Edit] = []
    skipped: list[Skipped] = []
    claimed = False
    for span, expected, found in _mismatch_sites(diagnostic):
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


_RULE_TABLE: dict[
    str, Callable[[Mapping[str, Any], FileReader, _Ctx], tuple[list[Edit], list[Skipped], bool]]
] = {
    RULE_NUMERIC_CAST: _numeric_cast_rule,
    RULE_NO_STD_PATH: _no_std_path_rule,
    RULE_POINTER_CAST: _pointer_cast_rule,
}


def propose_edits(
    diagnostics: Sequence[Mapping[str, Any]],
    read_file: FileReader,
    *,
    target: str,
    rules: Collection[str] = DIAGNOSTIC_RULES,
) -> RepairPlan:
    """PURE: parsed rustc diagnostics -> proposed edits, declined sites, and residual RED.

    ``read_file`` maps a diagnostic's ``file_name`` to its bytes (or ``None``); no cargo run and
    no filesystem knowledge is baked in, so tests drive this with fixtures.
    """
    ctx = _Ctx(target=target, width=pointer_width(target))
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
    applied only when >= 2 targets propose the IDENTICAL replacement; a disagreement is reported
    as ``target_divergent`` so a human can write the semantic alias instead.
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
        if sensitive and len(targets) < 2:
            if not allow_single_target_width:
                withheld.append(
                    Withheld(
                        proto.rule,
                        "width_unproven",
                        proto.file_name,
                        proto.line,
                        f"`{proto.replacement}` bakes a pointer-width type proven only on "
                        f"{','.join(targets) or 'no target'} (analysed: {','.join(analysed)}). "
                        "Re-run with the other targets, or hand-write the semantic alias.",
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


def apply_edits(
    edits: Sequence[Edit], root: Path
) -> tuple[list[Edit], dict[str, bytes], list[Skipped]]:
    """Splice edits back-to-front so byte offsets stay valid. Returns (applied, snapshot, failed).

    ``snapshot`` holds each touched file's ORIGINAL bytes so a caller can undo the whole pass.
    """
    by_file: dict[str, list[Edit]] = {}
    for edit in edits:
        by_file.setdefault(edit.file_name, []).append(edit)
    applied: list[Edit] = []
    snapshot: dict[str, bytes] = {}
    failed: list[Skipped] = []
    for file_name, group in sorted(by_file.items()):
        path = root / file_name
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
            data = data[: edit.byte_start] + edit.replacement.encode() + data[edit.byte_end :]
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
) -> CargoCheckRun:
    """Shell out to ``cargo check -p <crate> --target <triple> --message-format=json``."""
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
        process = subprocess.run(
            command, cwd=str(root), capture_output=True, text=True, timeout=timeout, check=False
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
_RE_FN_DECL = re.compile(rb"\bfn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", re.S)
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
            raw = data[open_paren + 1 : close].decode("utf-8", "replace")
            defined[match.group(1).decode("utf-8", "replace")] = [
                part.split(":", 1)[1].strip() if ":" in part else "" for part in _split_params(raw)
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
            raw = data[open_paren + 1 : close].decode("utf-8", "replace")
            params = []
            for part in _split_params(raw):
                # `name: Type` -> Type; `&self`/`mut self` and `...` carry no type.
                if ":" not in part:
                    params.append("")
                    continue
                params.append(part.split(":", 1)[1].strip())
            if name in found and found[name] != params:
                found[name] = None
            else:
                found[name] = params
    resolved = {k: v for k, v in found.items() if v is not None}
    resolved.update(defined)  # the definition settles any decl-vs-decl disagreement
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
        lambda target: run_cargo_check(crate, target, root=root, timeout=timeout)
    )
    snapshot: dict[str, bytes] = {}
    previous_total: int | None = None
    for index in range(1, max_passes + 1):
        runs = {target: check(target) for target in targets}
        counts = {target: run.error_count for target, run in runs.items()}
        total = sum(counts.values())
        current = RepairPass(index=index, error_counts=counts)
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
        plans = {
            target: propose_edits(run.diagnostics, reader, target=target, rules=diagnostic_rules)
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
        applied, snapshot, failed = apply_edits(selected, root)
        current.applied.extend(applied)
        current.skipped.extend(failed)
        session.passes.append(current)
        session.final_red = list(merged.red)
        previous_total = total
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
