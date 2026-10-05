"""Deterministically protect UTF-16 literals while an LLM translates control flow.

A wide string is C syntax with NO Rust equivalent — Rust has no UTF-16 literal at all —
so a model asked to translate one has nothing correct to write and invents something.
Every invention is a hand-count no compiler checks:

  * `b"text\\0".as_ptr() as PWSTR` — ASCII behind a wide pointer (mswsock, 3 sites).
  * the interleaved `b"t\\0e\\0x\\0"` — right in hex, ALIGNMENT 1 for a `u16` read,
    and one miscounted `\\0` truncates the string.
  * a hand-encoded `[u16; N]` with the terminator left off (18 shipped arrays), or one
    character short (wkssvc, 39 registry names), or `L"0000%s"` banked as FIVE zeros
    (intl kblayouts).

A prompt rule tells the model not to do that, and measurably helps — but it is ADVICE
to a sampler. This module removes the job instead: every literal occurrence becomes an
opaque identifier, the response must return each identifier, and rustconv substitutes
the real UTF-16 afterward. The model never sees the bytes and cannot alter them.

TWO C SHAPES, AND THE SECOND ONE IS THE BIG ONE. Across the tree's 5,987 `.c` files,
`L"..."` / `L'x'` is 27,545 tokens; the brace list `{'C','o','m','b','o','L','B','o','x',0}`
is 4,272 DECLARATIONS carrying 55,928 elements, and it is Wine's house style — which means
it is most of ReactOS. Every one of the 18 arrays task #60 repaired was a brace list,
not an `L"..."`; a seal that covered only the quoted form would have missed the class
that actually shipped the bugs. Note the asymmetry the two shapes force: `L"..."` IMPLIES
a terminator, so the seal appends one; a brace list STATES its own, and 195 of them end
without one on purpose, so the seal copies exactly what the C wrote and never invents a
zero. Getting that backwards is the same defect pointing the other way.

WHAT IS REVEALED, AND WHY: the token's Rust TYPE (`*const u16`, `[u16; 14]`, `u16`) is
in the contract; its value is not. The type is what the model needs to declare around
the token (`let mut buf: [u16; 14] = TOKEN;`), and rustconv computed it — leaving it out
would push the length back onto the model's fingers, which is the bug being fixed.

RESTORE SPELLING IS DECIDED AT SEAL TIME, from the C context, so it is derived rather
than guessed: an array initializer (`WCHAR buf[] = L"x"`, or any brace list) becomes an
inline `[u16; N]`, `sizeof(L"x")` likewise, and every other position becomes a
rustconv-generated `RUSTCONV_WSTR_<hash>` static behind `.as_ptr()`. Deliberately NOT a
crate macro: `w!` is defined per-crate in this tree and its result type is NOT uniform
(secur32's yields `&[u16; N]`, aclui's yields `*const u16`), so a deterministic restore
naming it would be right in one crate and a type error in the next.

RELATION TO THE COM PATH: `cpp_com.rewrite_wide_literals` already lowers shell32's C++
literals to `CPP_COM_WSTR_*` statics. That mechanism is proven and stays frozen — its
prompts are banked. What it does NOT have is an envelope: it substitutes a plausible
Rust expression the model may silently drop. This module keeps the statics and adds the
closed-set check, for plain C, under its own `--seal-wide` flag and its own resume-cache
token (`wideseal_v1`) so no banked work is re-billed.

THE BLIND SPOT WE DO NOT INHERIT: the constant seal's invariant is a closed SET, which a
PERMUTATION satisfies (msvcrt's `_tzset`). Recall-substitution cannot hide here — a model
writing a famous body from memory emits the literal TEXT, so the placeholder goes missing
and the envelope fires. A permutation of placeholders is the residual risk, and it is
reported (`order_findings`) rather than rejected, because legitimate reordering exists.

NOT SEALED, and reported rather than swallowed (see `SealedWides.skipped`): a literal
pasted against a macro (`REGSTR_PATH_EXPLORER L"\\IconUnderline"` — the macro's own text
is not in the body, so the run cannot be folded into one value), a SIZED array
initializer (`WCHAR k[200] = L"..."`, whose Rust lowering is a zero-fill plus a copy, not
one expression), a brace element that is a named constant rather than a literal, and an
escape this module refuses to guess at. Those fall back to the prompt contract, which is
exactly where they were before. Measured over those 5,987 files: **31,127 sealed, 76
reported** (0.24%) — and the report deliberately excludes pure zero-fills (`= {0}`,
`= L""`), because a list padded with non-findings is a list nobody reads.
"""

from __future__ import annotations

import hashlib
import re
from collections.abc import Mapping
from dataclasses import dataclass

from .cpp_core import _cpp_literal_spans
from .dead_regions import dead_region_spans, in_dead_region
from .lex_mask import comment_spans, directive_spans, mask_code


class WideSealError(RuntimeError):
    """The model damaged the deterministic UTF-16 envelope."""


#: Kinds, and the Rust type each token stands for.
KIND_POINTER = "pointer"
KIND_ARRAY = "array"
KIND_CHAR = "char"

_PLACEHOLDER_RE = re.compile(r"\b__RUSTCONV_WIDE_[0-9]{4}__\b")
_STATIC_PREFIX = "RUSTCONV_WSTR_"

# Deliberately distinct from cpp_com's `CPP_COM_WSTR_`: the two generators coexist in one
# tree and their names must not be confusable, and `candidate_oracle`/`repair_classifier`
# carry statics by that exact prefix on the shell32 path only.
_STATIC_NAME_RE = re.compile(rf"\b{_STATIC_PREFIX}[0-9A-F]{{12}}\b")


#: Elements per line in a rendered array. The tree's biggest sealed wide string is
#: casemap.c's 4,570-element Unicode table; on one line that is 40KB of hex nobody can
#: review or diff, and these tables are exactly the ones a human is expected to check.
#: Ten keeps the widest row inside rustfmt's 100 columns (4 indent + 11 + 9x8 = 87).
_UNITS_PER_LINE = 10


def _render_units(units: tuple[int, ...], indent: str = "    ") -> str:
    """`[0x0061u16, 0x0000]` — the suffix on the FIRST element types the whole array."""
    parts = [f"0x{unit:04X}u16" if i == 0 else f"0x{unit:04X}" for i, unit in enumerate(units)]
    if len(parts) <= _UNITS_PER_LINE:
        return "[" + ", ".join(parts) + "]"
    rows = [
        indent + ", ".join(parts[i : i + _UNITS_PER_LINE])
        for i in range(0, len(parts), _UNITS_PER_LINE)
    ]
    return "[\n" + ",\n".join(rows) + ",\n" + indent[:-4] + "]"


@dataclass(frozen=True)
class SealedWide:
    placeholder: str
    kind: str
    #: UTF-16 code units. String kinds carry the terminating 0; `char` carries one unit.
    units: tuple[int, ...]
    #: Name of the generated static, for `pointer` only.
    static_name: str | None
    #: The exact C text this token replaced, for reporting.
    source: str
    #: This occurrence sits in code the build never executes — permitted to be omitted,
    #: never demanded. See dead_regions.py.
    dead: bool = False
    #: `(arch, units)` per built arch when a folded macro expands differently per arch
    #: (`L"AddReg.NT" INF_ARCH`); `pointer` kind only, rendered as `#[cfg]` statics.
    #: Empty for every literal whose value is the same everywhere.
    arch_units: tuple[tuple[str, tuple[int, ...]], ...] = ()

    @property
    def rust_type(self) -> str:
        if self.kind == KIND_CHAR:
            return "u16"
        if self.kind == KIND_ARRAY:
            return f"[u16; {len(self.units)}]"
        return "*const u16"

    @property
    def restore(self) -> str:
        if self.kind == KIND_CHAR:
            return f"0x{self.units[0]:04X}u16"
        if self.kind == KIND_ARRAY:
            # One suffix carries the whole array: Rust requires every element to share a
            # type, so `[0x0061u16, 0x0000]` is `[u16; 2]` even with no annotation around
            # it (`size_of_val(&TOKEN)` has none).
            return _render_units(self.units)
        return f"{self.static_name}.as_ptr()"


@dataclass(frozen=True)
class UnsealedWide:
    """A literal deliberately left for the prompt contract, and why."""

    source: str
    reason: str
    #: Byte offset in the body, so a report can point at the ONE occurrence — several of
    #: these are `L""`, and searching for the text finds the wrong one every time.
    offset: int = -1


@dataclass(frozen=True)
class SealedWides:
    text: str
    wides: tuple[SealedWide, ...]
    skipped: tuple[UnsealedWide, ...] = ()


# ── decoding ────────────────────────────────────────────────────────────────────────

_SIMPLE_ESCAPES = {
    "a": 0x07,
    "b": 0x08,
    "f": 0x0C,
    "n": 0x0A,
    "r": 0x0D,
    "t": 0x09,
    "v": 0x0B,
    "\\": 0x5C,
    "'": 0x27,
    '"': 0x22,
    "?": 0x3F,
}


class _UndecodableWide(Exception):
    """This literal's bytes cannot be derived with certainty — do not seal it."""


def _decode_c_wide(body: str) -> tuple[int, ...]:
    """UTF-16 code units for the CONTENTS of a wide literal (no terminator appended).

    Hand-written rather than routed through `ast.literal_eval` (which is what the frozen
    COM path uses) because C and Python disagree on escapes that are live in this tree:

      * `\\xNNNN` — C consumes every hex digit, Python exactly two. `L"\\x00b7"` is ONE
        code unit in C and THREE characters in Python (`\\x00`, `b`, `7`). riched20 and
        every `kbd*` layout carry these.
      * `\\?` — C yields `?`; Python keeps the backslash and warns.

    An escape contributes its numeric value directly (a wide literal's `\\x41` is the
    code UNIT 0x41, not a byte to re-encode). Everything else is source text, decoded as
    UTF-16 so a non-BMP character correctly becomes its surrogate pair.
    """
    units: list[int] = []
    plain: list[str] = []

    def flush() -> None:
        if plain:
            encoded = "".join(plain).encode("utf-16-le", "surrogatepass")
            units.extend(encoded[i] | (encoded[i + 1] << 8) for i in range(0, len(encoded), 2))
            plain.clear()

    index = 0
    while index < len(body):
        char = body[index]
        if char != "\\":
            plain.append(char)
            index += 1
            continue
        index += 1
        if index >= len(body):
            raise _UndecodableWide("trailing backslash")
        escape = body[index]
        if escape in _SIMPLE_ESCAPES:
            flush()
            units.append(_SIMPLE_ESCAPES[escape])
            index += 1
        elif escape in "01234567":
            digits = ""
            while index < len(body) and len(digits) < 3 and body[index] in "01234567":
                digits += body[index]
                index += 1
            flush()
            units.append(int(digits, 8))
        elif escape == "x":
            index += 1
            digits = ""
            while index < len(body) and body[index] in "0123456789abcdefABCDEF":
                digits += body[index]
                index += 1
            if not digits:
                raise _UndecodableWide("\\x with no hex digits")
            value = int(digits, 16)
            if value > 0xFFFF:
                raise _UndecodableWide(f"\\x{digits} exceeds one UTF-16 code unit")
            flush()
            units.append(value)
        elif escape in "uU":
            width = 4 if escape == "u" else 8
            digits = body[index + 1 : index + 1 + width]
            if len(digits) != width or any(c not in "0123456789abcdefABCDEF" for c in digits):
                raise _UndecodableWide(f"malformed \\{escape} universal character name")
            index += 1 + width
            code_point = int(digits, 16)
            if code_point > 0x10FFFF or 0xD800 <= code_point <= 0xDFFF:
                raise _UndecodableWide(f"\\{escape}{digits} is not a scalar value")
            plain.append(chr(code_point))
        else:
            # Refuse rather than guess. An unknown escape is undefined behaviour in C, so
            # there is no correct answer to substitute, and the prompt contract still
            # covers the literal.
            raise _UndecodableWide(f"unsupported escape \\{escape}")
    flush()
    return tuple(units)


def _literal_body(token: str) -> str:
    """The text between the quotes of `L"..."` / `"..."` / `L'x'`."""
    start = token.index(token[-1])
    return token[start + 1 : -1]


def _static_name(units: tuple[int, ...]) -> str:
    payload = b"".join(unit.to_bytes(2, "little") for unit in units)
    return _STATIC_PREFIX + hashlib.sha256(payload).hexdigest()[:12].upper()


#: Built arches and the `target_arch` each one is.
ARCH_CFG = {"i386": "x86", "amd64": "x86_64", "arm64": "aarch64"}


def _arch_static_name(arch_units: tuple[tuple[str, tuple[int, ...]], ...]) -> str:
    payload = b"".join(
        arch.encode() + b":" + b"".join(u.to_bytes(2, "little") for u in units) + b";"
        for arch, units in arch_units
    )
    return _STATIC_PREFIX + hashlib.sha256(payload).hexdigest()[:12].upper()


_NEXT_PIECE = re.compile(r'\s*(L?"(?:[^"\\\n]|\\.)*"|[A-Za-z_]\w*)')


def _fold_with_macros(
    text: str, start: int, end: int, string_macros: Mapping[str, Mapping[str, str]]
) -> tuple[int, int, dict[str, tuple[int, ...]]] | None:
    """Extend a literal run across adjacent string-literal MACROS: (start, end, units/arch).

    None unless at least one macro joined the run and every piece has a value on every
    built arch -- a macro the preprocessor did not report is not ours to guess.
    """
    pieces: list[tuple[bool, str]] = [(False, text[start:end])]
    pos = end
    while (m := _NEXT_PIECE.match(text, pos)) is not None:
        token = m.group(1)
        if token.startswith(('"', 'L"')):
            pieces.append((False, token))
        elif token in string_macros:
            pieces.append((True, token))
        else:
            break
        pos = m.end()
    first = start
    while (m := _IDENTIFIER_TAIL.search(text[:first].rstrip())) and m.group(0) in string_macros:
        pieces.insert(0, (True, m.group(0)))
        first = m.start()
    if not any(is_macro for is_macro, _ in pieces):
        return None
    per_arch: dict[str, tuple[int, ...]] = {}
    for arch in ARCH_CFG:
        units: list[int] = []
        for is_macro, value in pieces:
            literal = string_macros[value].get(arch) if is_macro else value
            if literal is None:
                return None
            try:
                units.extend(_run_units(literal, 0, len(literal)))
            except _UndecodableWide:
                return None
        per_arch[arch] = tuple(units)
    return first, pos, per_arch


# ── position classification ─────────────────────────────────────────────────────────

_IDENTIFIER_TAIL = re.compile(r"[A-Za-z_]\w*$")

#: C keywords that may legally stand immediately before a string literal.
_LITERAL_ADJACENT_KEYWORDS = frozenset({"return", "sizeof", "case", "else", "do"})


def _matching_open_bracket(masked: str, closing: int) -> int | None:
    depth = 0
    for index in range(closing, -1, -1):
        if masked[index] == "]":
            depth += 1
        elif masked[index] == "[":
            depth -= 1
            if depth == 0:
                return index
    return None


def _classify(masked: str, start: int) -> tuple[str, str | None]:
    """`(kind, skip_reason)` for the wide string literal beginning at `start`.

    Three positions matter and the C says which one this is:

    * `sizeof(L"x")` wants the ARRAY, not a pointer — `size_of_val(&[…])` is the faithful
      lowering and `.as_ptr()` there would silently measure a pointer instead. 60 sites.
    * A DECLARATION with an empty bound (`WCHAR buf[] = L"x"`) wants the array too, and
      rustconv knows its length where the model would have to count. 219 sites.
    * A declaration with a NON-empty bound (`WCHAR k[200] = L"..."`) is skipped: its
      faithful Rust is a zero-filled array plus a copy, which is a statement, not a value
      a single token can stand for. 65 sites.

    Everything else — argument, comparison, `return`, pointer assignment, and the array
    ELEMENT assignment `names[0] = L"x"` (no type before the name, so not a declaration) —
    is a pointer.
    """
    before = masked[:start].rstrip()
    if before.endswith("("):
        head = _IDENTIFIER_TAIL.search(before[:-1].rstrip())
        if head is not None and head.group(0) == "sizeof":
            return KIND_ARRAY, None
    if not before.endswith("="):
        return KIND_POINTER, None
    declarator = before[:-1].rstrip()
    if not declarator.endswith("]"):
        return KIND_POINTER, None
    opening = _matching_open_bracket(declarator, len(declarator) - 1)
    if opening is None:
        return KIND_POINTER, None
    name = _IDENTIFIER_TAIL.search(declarator[:opening].rstrip())
    if name is None:
        return KIND_POINTER, None
    # A DECLARATION has a type before the declared name; `names[0] = L"x"` does not, and
    # is an ordinary pointer store into an existing array.
    lead = declarator[: name.start()].rstrip().rstrip("*").rstrip()
    if _IDENTIFIER_TAIL.search(lead) is None:
        return KIND_POINTER, None
    if declarator[opening + 1 : -1].strip():
        return KIND_POINTER, "sized array initializer — the lowering is a copy, not a value"
    return KIND_ARRAY, None


# ── sealing ─────────────────────────────────────────────────────────────────────────


def _literal_runs(text: str) -> list[tuple[int, int, bool]]:
    """`(start, end, is_wide)` for each run of ADJACENT C string literals.

    C concatenates adjacent literals, and one of them being wide makes the whole run
    wide. Folding the run into a single token is what closes the gap `cpp_com` documents
    and leaves open: lowering only the `L"..."` half of `L"a" L"b"` emits two Rust
    expressions side by side, which is not syntax. 41 sites in the tree, most of them a
    long registry path split across two source lines.

    Character literals never participate: `L'a'` is not concatenable.
    """
    spans = _cpp_literal_spans(text)
    runs: list[tuple[int, int, bool]] = []
    index = 0
    while index < len(spans):
        start, end = spans[index]
        token = text[start:end]
        # Classify by the OPENING quote. Asking whether the token ends in `'` calls the
        # perfectly ordinary narrow string `"it's"` a character literal.
        if token.startswith("'") or token.startswith("L'"):  # never folded
            runs.append((start, end, token.startswith("L'")))
            index += 1
            continue
        wide = token.startswith('L"')
        while index + 1 < len(spans):
            next_start, next_end = spans[index + 1]
            if text[end:next_start].strip():
                break
            next_token = text[next_start:next_end]
            if not next_token.startswith(('"', 'L"')):
                break
            wide = wide or next_token.startswith('L"')
            end = next_end
            index += 1
        runs.append((start, end, wide))
        index += 1
    return runs


def _run_units(text: str, start: int, end: int) -> tuple[int, ...]:
    units: list[int] = []
    for span_start, span_end in _cpp_literal_spans(text[start:end]):
        units.extend(_decode_c_wide(_literal_body(text[start + span_start : start + span_end])))
    return tuple(units)


#: The wide element types a brace-initialised string can be declared with.
_WIDE_ELEM_TYPE = r"(?:WCHAR|TCHAR|OLECHAR|OLECHAR16|wchar_t|MSVCRT_wchar_t)"

_BRACE_DECL = re.compile(
    r"\b(?:(?:static|const|volatile|extern)\s+)*"
    rf"{_WIDE_ELEM_TYPE}\s+"
    r"(?P<name>[A-Za-z_]\w*)\s*\[\s*(?P<bound>[^\]]*?)\s*\]\s*=\s*(?=\{)"
)

_ELEM_INT = re.compile(r"^(?:0[xX][0-9a-fA-F]+|0[0-7]*|[1-9][0-9]*)[uUlL]*$")
_CHAR_ELEM = re.compile(r"L?'(?:\\.|[^'\\])'")


def _int_literal(token: str) -> int | None:
    """A C integer literal's value, or None if `token` is not one.

    `int(x, 0)` is not this function: Python rejects C's `010` (its octal wants `0o`), so
    routing a leading-zero literal through it raises instead of answering 8.
    """
    if not _ELEM_INT.match(token):
        return None
    digits = token.rstrip("uUlL")
    if digits[:2].lower() == "0x":
        return int(digits, 16)
    if digits.startswith("0") and len(digits) > 1:
        return int(digits, 8)
    return int(digits, 10)


def _matching_brace(masked: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == "{":
            depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def _decode_brace_elements(
    text: str, masked: str, comment: list[bool], start: int, end: int
) -> tuple[int, ...]:
    """Code units for a `{'C','o','m','b',0}` initializer body (exclusive of the braces).

    NOTE what this does NOT do: it never appends a terminator. `L"..."` implies one and the
    hand-encodings forgot it; a brace list states its own, and 195 of the tree's 4,126 sealed
    ones deliberately end without one (fixed-width fields, byte tables). Copying exactly what
    the C wrote is the whole guarantee — inventing a zero here would be the same class of
    error in the other direction.

    Commas are found in MASKED text so `','` — an element whose value IS a comma — does not
    split itself in half, and comment characters are dropped from each element so the two
    keyboard-layout tables that write `{ /* index */ 0x01bf, ... }` decode rather than
    reading as a non-literal expression.
    """
    units: list[int] = []
    cursor = start
    while cursor <= end:
        comma = masked.find(",", cursor, end)
        stop = end if comma < 0 else comma
        token = "".join(text[index] for index in range(cursor, stop) if not comment[index]).strip()
        cursor = stop + 1
        if not token:
            if comma < 0:
                break
            continue  # a trailing comma before `}`
        if _CHAR_ELEM.fullmatch(token):
            # fullmatch, not startswith: `'0' + (n / 100) % 10` also begins with a quote, and
            # slicing that as a char literal decodes 29 units of arithmetic before anything
            # notices. Two of msi's stream-number builders are exactly that shape.
            decoded = _decode_c_wide(_literal_body(token))
            if len(decoded) != 1:
                raise _UndecodableWide(f"element {token} is {len(decoded)} code units, not 1")
            units.append(decoded[0])
        elif (value := _int_literal(token)) is not None:
            if value > 0xFFFF:
                raise _UndecodableWide(f"element {token} exceeds one UTF-16 code unit")
            units.append(value)
        else:
            # A named constant, a macro call, an arithmetic expression. Its value is not
            # derivable here, and a seal that guessed would be worse than no seal.
            raise _UndecodableWide(f"element {token[:32]} is not a literal")
        if comma < 0:
            break
    return tuple(units)


def _brace_string_spans(
    text: str, masked: str, directive: list[bool], comment: list[bool]
) -> tuple[list[tuple[int, int, str, tuple[int, ...]]], list[UnsealedWide]]:
    """`{'C','o','m','b','o','L','B','o','x',0}` — Wine's house style, and the shape that
    actually shipped the bugs.

    Every one of the 18 arrays task #60 repaired came from HERE, not from `L"..."`:
    `static const WCHAR clbName[] = {'C','o','m','b','o','L','B','o','x',0};` and its 4,271
    siblings. The model retypes the element list by hand and drops the final `0` — which no
    compiler can catch, because a `[u16; 9]` is a perfectly good array right up until
    `wcslen` walks off the end of it.

    Returns the sealable spans plus the ones deliberately left alone.
    """
    spans: list[tuple[int, int, str, tuple[int, ...]]] = []
    skipped: list[UnsealedWide] = []
    for match in _BRACE_DECL.finditer(masked):
        opening = match.end()
        closing = _matching_brace(masked, opening)
        if closing is None or any(directive[match.start() : closing + 1]):
            continue
        if "{" in masked[opening + 1 : closing]:
            continue  # an array of arrays or of structs — not one string
        source = text[opening : closing + 1]
        try:
            units = _decode_brace_elements(text, masked, comment, opening + 1, closing)
        except _UndecodableWide as error:
            skipped.append(UnsealedWide(source[:80], str(error), opening))
            continue
        if not any(units):
            # `WCHAR buf[MAX_PATH] = {0};` is a zero-FILL, not a string: it carries no bytes
            # to protect, and demanding a token there would reject the very lowerings that
            # are correct for it (`[0u16; N]`, `core::mem::zeroed()`). Exactly the judgement
            # fact_seal._aggregate_zero_init_spans already makes about the same construct —
            # and not an audit finding either, because a report of non-findings gets ignored.
            continue
        bound = match.group("bound").strip()
        if bound:
            # C zero-fills a sized array's tail, so the Rust value is N units, not the
            # written count. Derivable only when the bound is a literal; `WCHAR x[MAX_PATH]`
            # is not something to guess at.
            declared = _int_literal(bound)
            if declared is None or declared < len(units):
                skipped.append(
                    UnsealedWide(
                        source[:80],
                        f"array bound `{bound}` is not a literal this can zero-fill to",
                        opening,
                    )
                )
                continue
            units = units + (0,) * (declared - len(units))
        spans.append((opening, closing + 1, source, units))
    return spans, skipped


def seal_wide_literals(
    text: str, string_macros: Mapping[str, Mapping[str, str]] | None = None
) -> SealedWides:
    """Replace each wide-literal occurrence with a unique opaque identifier.

    `string_macros` (macro_authority.string_literal_macros: name -> arch -> literal text)
    lets a run pasted against such a macro be folded instead of skipped.
    """
    directive = directive_spans(text)
    comment = comment_spans(text)
    masked = mask_code(text)
    dead_spans = dead_region_spans(text, masked)

    wides: list[SealedWide] = []
    skipped: list[UnsealedWide] = []
    edits: list[tuple[int, int, str]] = []

    brace_spans, brace_skipped = _brace_string_spans(text, masked, directive, comment)
    skipped.extend(brace_skipped)
    claimed = [(start, end) for start, end, _source, _units in brace_spans]

    # One numbering over both shapes, in source order, so the contract reads in the order
    # the C does and the reorder audit means what it says.
    candidates: list[tuple[int, int, str, tuple[int, ...] | None]] = [
        (start, end, source, units) for start, end, source, units in brace_spans
    ]
    candidates += [
        (start, end, text[start:end], None)
        for start, end, is_wide in _literal_runs(text)
        if is_wide
    ]
    candidates.sort()
    folded_to = -1  # a fold across a macro may swallow the next run (`L"a" M L"b"`)

    for start, end, source, brace_units in candidates:
        if start < folded_to:
            continue
        if brace_units is not None:
            placeholder = f"__RUSTCONV_WIDE_{len(wides):04d}__"
            wides.append(
                SealedWide(
                    placeholder=placeholder,
                    kind=KIND_ARRAY,
                    units=brace_units,
                    static_name=None,
                    source=source,
                    dead=in_dead_region(dead_spans, start, end),
                )
            )
            edits.append((start, end, placeholder))
            continue
        if any(a <= start and end <= b for a, b in claimed):
            continue  # an `L'x'` element inside a brace list the array seal already owns
        # Nothing inside a `#define`/`#if` can be sealed: the token has no place to go in
        # the Rust output (lex_mask.directive_spans).
        if any(directive[start:end]):
            continue
        is_char = text[start:end].startswith("L'")
        if not is_char and _pasted_against_identifier(masked, directive, start, end):
            folded = _fold_with_macros(text, start, end, string_macros) if string_macros else None
            if folded is None:
                skipped.append(
                    UnsealedWide(source, "pasted against a macro — the run cannot be folded", start)
                )
                continue
            start, end, per_arch = folded
            source = text[start:end]
            if any(directive[start:end]):
                continue
            kind, reason = _classify(masked, start)
            variants = {arch: (*units, 0) for arch, units in per_arch.items()}
            distinct = set(variants.values())
            if reason is None and len(distinct) > 1 and kind != KIND_POINTER:
                reason = "the macro's value differs per arch, outside a pointer position"
            if reason is not None:
                skipped.append(UnsealedWide(source, reason, start))
                continue
            arch_units: tuple[tuple[str, tuple[int, ...]], ...] = ()
            if len(distinct) == 1:
                units = distinct.pop()
                static_name = _static_name(units) if kind == KIND_POINTER else None
            else:
                arch_units = tuple(sorted(variants.items()))
                units = variants["amd64"]
                static_name = _arch_static_name(arch_units)
            placeholder = f"__RUSTCONV_WIDE_{len(wides):04d}__"
            wides.append(
                SealedWide(
                    placeholder=placeholder,
                    kind=kind,
                    units=units,
                    static_name=static_name,
                    source=source,
                    dead=in_dead_region(dead_spans, start, end),
                    arch_units=arch_units,
                )
            )
            edits.append((start, end, placeholder))
            folded_to = end
            continue
        try:
            units = _run_units(text, start, end)
        except _UndecodableWide as error:
            skipped.append(UnsealedWide(source, str(error), start))
            continue
        if is_char:
            if len(units) != 1:
                skipped.append(
                    UnsealedWide(
                        source,
                        f"wide character is {len(units)} code units, not 1",
                        start,
                    )
                )
                continue
            kind, static_name = KIND_CHAR, None
        else:
            kind, reason = _classify(masked, start)
            if reason is not None:
                # `WCHAR CurLocale[256] = L"";` is 33 of the 48 sized-array sites and is the
                # same zero-fill the brace path skips silently, for the same reason: no bytes
                # to protect, and no report, so what IS reported stays worth reading.
                if units:
                    skipped.append(UnsealedWide(source, reason, start))
                continue
            units = (*units, 0)  # C carries the terminator; the hand-encodings forgot it
            static_name = _static_name(units) if kind == KIND_POINTER else None
        placeholder = f"__RUSTCONV_WIDE_{len(wides):04d}__"
        wides.append(
            SealedWide(
                placeholder=placeholder,
                kind=kind,
                units=units,
                static_name=static_name,
                source=source,
                dead=in_dead_region(dead_spans, start, end),
            )
        )
        edits.append((start, end, placeholder))

    result = text
    for start, end, placeholder in reversed(edits):
        result = result[:start] + placeholder + result[end:]
    return SealedWides(result, tuple(wides), tuple(skipped))


def _pasted_against_identifier(masked: str, directive: list[bool], start: int, end: int) -> bool:
    """True when an identifier abuts the run — a macro that expands to more literal.

    `REGSTR_PATH_EXPLORER L"\\\\IconUnderline"` is one C string; the macro's text is not in
    the body being translated, so the value cannot be derived here. Sealing only the
    visible half would produce `MACRO __RUSTCONV_WIDE_0000__`, which restores to two Rust
    expressions with nothing between them.

    A KEYWORD before the literal is not a paste: `return L"x";` and `case L'a':` are the
    only way to write those, and reading `return` as a macro would unseal the commonest
    position there is. Neither is a word that belongs to a DIRECTIVE — a literal on the
    line after `#endif`, or after `#ifdef EXTENDED_ATTRIBUTES`, only LOOKS pasted, and
    shell32's three `CSIDL_Type_User`/`#endif`/`L"My Music"` rows are exactly that shape.
    """
    before = masked[:start].rstrip()
    word = _IDENTIFIER_TAIL.search(before)
    if (
        word is not None
        and word.group(0) not in _LITERAL_ADJACENT_KEYWORDS
        and not directive[len(before) - 1]
    ):
        return True
    after = masked[end:].lstrip()
    return bool(after) and (after[0].isalpha() or after[0] == "_")


def wide_contract(sealed: SealedWides) -> str:
    """Prompt contract that reveals each token's TYPE, never its value."""
    if not sealed.wides:
        return ""
    lines = "\n".join(f"  {wide.placeholder} : {wide.rust_type}" for wide in sealed.wides)
    return (
        "\n## Deterministic UTF-16 literal envelope\n"
        "The identifiers below stand for C wide literals that rustconv owns. Rust has no "
        "UTF-16 literal, so there is nothing correct for you to write in their place: use "
        "each token exactly as spelled, wherever the C used the literal. The Rust type is "
        "given so you can declare around the token; the VALUE is not yours to reproduce, "
        "count, encode, or index into. Do not rename, define, or omit one; rustconv "
        "substitutes the real UTF-16 after your response.\n"
        f"Tokens ({len(sealed.wides)}):\n{lines}\n"
    )


# ── unsealing ───────────────────────────────────────────────────────────────────────


def unseal_wide_literals(
    rust: str,
    sealed: SealedWides,
    *,
    dead_omissions: list[str] | None = None,
    order_findings: list[str] | None = None,
) -> str:
    """Verify the closed token set and substitute the deterministic UTF-16.

    `order_findings`, when given, collects a note if the tokens appear in a different
    ORDER than the C used them. That is the one way a damaged envelope can still satisfy
    a closed-set check — the permutation the constant seal cannot see (`_tzset`) — and it
    is reported rather than rejected because a faithful translation may legitimately
    reorder, so a human decides.
    """
    expected = {wide.placeholder for wide in sealed.wides}
    present = _PLACEHOLDER_RE.findall(rust)
    unknown = sorted(set(present) - expected)
    optional = {wide.placeholder for wide in sealed.wides if wide.dead}
    missing_all = expected - set(present)
    omitted_dead = sorted(missing_all & optional)
    missing = sorted(missing_all - optional)

    if omitted_dead and dead_omissions is not None:
        by_name = {wide.placeholder: wide for wide in sealed.wides}
        dead_omissions.extend(f"{name}={by_name[name].source}" for name in omitted_dead)

    findings = []
    if missing:
        findings.append("missing: " + ", ".join(missing))
    if unknown:
        findings.append("invented: " + ", ".join(unknown))
    # A static name is rustconv's to write, never the model's: one spelled by hand is a
    # guess at a content hash and would silently reference the wrong bytes (or nothing).
    invented_statics = sorted(set(_STATIC_NAME_RE.findall(rust)))
    if invented_statics:
        findings.append("invented UTF-16 static(s): " + ", ".join(invented_statics))
    if findings:
        raise WideSealError(
            "model damaged deterministic UTF-16 envelope (" + "; ".join(findings) + ")"
        )

    if order_findings is not None:
        wanted = [w.placeholder for w in sealed.wides if w.placeholder not in set(omitted_dead)]
        first_seen: list[str] = []
        for name in present:
            if name not in first_seen:
                first_seen.append(name)
        if first_seen != wanted:
            order_findings.append(
                "UTF-16 tokens appear in a different order than the C used them "
                f"(C: {' '.join(wanted)}; output: {' '.join(first_seen)}) — check each "
                "literal landed in the right slot"
            )

    for wide in sealed.wides:
        rust = rust.replace(wide.placeholder, wide.restore)
    return rust


def render_wide_statics(sealed: list[SealedWides], referenced_in: str) -> str:
    """The generated UTF-16 statics the assembled module actually references.

    Raises on a reference with no source, the way `render_wide_literal_statics` does for
    the COM path: a dangling name is a missing definition, and finding it here beats
    finding it as an unresolved symbol at link time.
    """
    used = set(_STATIC_NAME_RE.findall(referenced_in))
    if not used:
        return ""
    by_name: dict[str, SealedWide] = {}
    for group in sealed:
        for wide in group.wides:
            if wide.static_name is not None and wide.static_name in used:
                by_name[wide.static_name] = wide
    orphans = used - set(by_name)
    if orphans:
        raise WideSealError(
            "lost deterministic UTF-16 literal source: " + ", ".join(sorted(orphans))
        )
    lines = [
        "// Deterministic UTF-16 literals, generated from the C by rustconv "
        "(--seal-wide); the model never saw the bytes."
    ]
    for name, wide in sorted(by_name.items()):
        for arch, units in wide.arch_units or (("", wide.units),):
            cfg = f'#[cfg(target_arch = "{ARCH_CFG[arch]}")]\n' if arch else ""
            lines.append(f"{cfg}pub static {name}: [u16; {len(units)}] = {_render_units(units)};")
    return "\n".join(lines)
