"""Deterministically protect C facts while an LLM translates control flow.

Cheap models are useful at syntax and notoriously unreliable at copying numeric
and symbolic constants.  This module removes that job from the model: every
constant occurrence becomes an opaque identifier, the response must return each
identifier at least once, and rustconv restores the original spelling afterward.

The transform is deliberately lexical.  It never touches comments, string/char
literals, callable macro names, or declaration/cast type names.

DIALECTS
--------
`seal_facts(dialect=...)` selects which value families are protected:

* ``"cpp"`` (default) — the original shell32/COM behaviour, **frozen**.  Changing
  what it seals changes the prompt, which changes the response-cache key, which
  would silently invalidate every cached *paid* shell32 worker and re-bill it.
  Do not "improve" this dialect; add to ``"c"`` instead.
* ``"c"`` — plain C (kernel drivers, Win32 DLLs).  Adds two protections the
  ALL_CAPS matcher cannot express:
    1. **CamelCase enum constants** (`NonPagedPool`, `KernelMode`, `Executive`,
       the `Ndis*` vocabulary) via the generated ground truth in
       `ddk_enum_constants`.  Measured 345 otherwise-unprotected occurrences
       across the network DLLs and drivers.  Safe to restore because bindgen
       spells these identically to C (`pub const NonPagedPool: POOL_TYPE = 0;`).
    2. A deliberate NON-feature: there is no `sizeof(TYPE)` exclusion.  Sealing a
       type name is benign (restoring it literally still yields valid Rust, e.g.
       `size_of::<NDIS_REQUEST>()`), whereas skipping `sizeof(UNICODE_NULL)` would
       leave a real VALUE unprotected.  Over-seal a type, never under-seal a value.
  It also refuses to seal any name *declared in the body being translated*:
  sealing storage is the actively harmful direction, because the contract tells
  the model the token is an opaque value when it is really a variable to read.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from .ddk_enum_constants import DDK_ENUM_CONSTANTS
from .dead_regions import dead_region_spans, in_dead_region
from .lex_mask import directive_spans, opens_a_literal


class FactSealError(RuntimeError):
    """The model damaged the deterministic fact envelope."""


@dataclass(frozen=True)
class SealedFact:
    placeholder: str
    original: str
    kind: str
    # This occurrence sits in code the build never executes (a `#if`-disabled arm, or a
    # DUMMY_PSEH except-block). Defaults False so an older pickled/cached fact stays
    # required. See dead_regions.py for why these must not be DEMANDED of the model.
    dead: bool = False
    # The occurrence is the whole operand of `sizeof(...)`. A model that hands such a token
    # back bare has dropped the sizeof (Haiku did it in appwiz and hdwwiz: `psp.dwSize =
    # PROPSHEETPAGE as DWORD`); unseal restores `size_of::<T>()`. A VALUE in that position
    # (`sizeof(UNICODE_NULL)`) then fails to compile, which is the loud outcome.
    in_sizeof: bool = False


@dataclass(frozen=True)
class SealedSource:
    text: str
    facts: tuple[SealedFact, ...]


_PLACEHOLDER_RE = re.compile(r"\b__RUSTCONV_FACT_[0-9]{4}__\b")
# A digit run beside a `.` is part of a FLOAT literal, never an integer of its own: wordpad's
# `0.0138` was sealed as the integers `0` and `0138` (an invalid octal that crashed the
# placement audit), and `1.5e10f` as `1` and `.5e10f`.
_INTEGER_RE = re.compile(
    r"(?<![A-Za-z0-9_.])(?:0[xX][0-9A-Fa-f]+|0[oO][0-7]+|[0-9]+)"
    r"(?:[uUlL]{0,3})(?![A-Za-z0-9_.])"
)
_SYMBOL_RE = re.compile(r"\b[A-Z][A-Z0-9_]+\b")
_C_SYMBOL_RE = re.compile(r"\b_*[A-Z][A-Z0-9_]+\b")

# Any capitalised identifier; only those present in DDK_ENUM_CONSTANTS are sealed,
# so this pattern is a candidate filter, never an authority.  Disjoint from
# _SYMBOL_RE by construction: the generated set excludes ALL_CAPS names.
_CAMEL_RE = re.compile(r"\b[A-Z][A-Za-z0-9_]*\b")

# `TYPE name;` / `TYPE *name = ...` / `TYPE name[8]` / `(TYPE name,` — the trailing
# identifier is storage being introduced.  Used only to EXCLUDE names from sealing,
# so over-matching costs a missed seal (safe) and never a sealed variable (unsafe).
_DECLARATOR_RE = re.compile(r"\b[A-Za-z_]\w*\s+\**\s*([A-Za-z_]\w*)\s*(?=[;,=)\[])")

# These are syntax/type/debug words rather than values.  Callable identifiers
# are excluded independently, so HRESULT_FROM_WIN32(...) need not be listed.
_NON_FACT_WORDS = {
    "BOOL",
    "BYTE",
    "CLSID",
    "DWORD",
    "ERR",
    "FIXME",
    "GUID",
    "HRESULT",
    "INT",
    "LONG",
    "LPARAM",
    "NULL",
    "POINTL",
    "SIZE_T",
    "TRACE",
    "UINT",
    "ULONG",
    "WCHAR",
    "WARN",
    "WPARAM",
}


# PSEH's statement macros. They are syntax, not values, and the C symbol pattern's leading
# underscores admit them: tdihelpers' `_SEH2_TRY { ... } _SEH2_EXCEPT(...) { ... } _SEH2_END;`
# came back sealed as `__RUSTCONV_FACT_0001__ { ... } _SEH2_EXCEPT(...) { ... }
# __RUSTCONV_FACT_0002__;`, which hid the construct the prompt's SEH section teaches, and the
# model echoed the C shape, which does not parse. (_SEH2_EXCEPT, _SEH2_YIELD and
# _SEH2_GetExceptionCode are call-shaped and were never sealed.) The cache key is the
# unsealed C body, so narrowing the seal re-bills nothing.
_C_STATEMENT_MACROS = frozenset({"_SEH2_TRY", "_SEH2_END", "_SEH2_FINALLY", "_SEH2_LEAVE"})


_BLOCK_SCOPE_PROTOTYPE_RE = re.compile(r"\bextern\s+[^;{}=]*?\([^;{}]*\)\s*;")
# A C++ NAMED cast's template argument is a type, exactly like the `(TYPE *)value`
# C-style cast the symbol scanner already skips — the C-style form just happens to
# be the only one it learned to recognise.
_NAMED_CAST_RE = re.compile(
    r"\b(?:const_cast|static_cast|reinterpret_cast|dynamic_cast)\s*<[^<>;{}]*>"
)


def _named_cast_spans(masked: str) -> list[tuple[int, int]]:
    """Spans of `const_cast<LPWSTR>` and friends — the `<...>` is a TYPE.

    `dde_connect` lost `const_cast<LPWSTR>(strchrW(p, ' '))` to this on every paid
    attempt: Rust writes that as `x as LPWSTR`, or elides the alias entirely for
    `*mut u16`, so a sealed type token there can vanish through no fault of the model.
    """
    return [(match.start(), match.end()) for match in _NAMED_CAST_RE.finditer(masked)]


def _block_scope_prototype_spans(masked: str) -> list[tuple[int, int]]:
    """Spans of a body-local `extern RET Callee(TYPE, TYPE);` forward declaration.

    Its parameter types are ABI CONTEXT, not values — the same judgement `seal_calls`
    already makes about the callee name in the very same construct. Sealing them
    builds an envelope nothing can satisfy: a faithful lowering writes a Rust
    `extern "system" { fn Callee(a: LPCWSTR, ...); }`, and Rust is entitled to spell
    that type differently (`*const u16`), at which point the token is simply gone.
    `InvokeOpenWith` failed this way on four consecutive paid attempts, and one
    rejected worker assembles no file at all.

    The values that sealing exists to protect — `SEE_MASK_NOCLOSEPROCESS` and its
    kind (§9.113) — never appear in a prototype's type list, so nothing is lost.
    """
    return [(match.start(), match.end()) for match in _BLOCK_SCOPE_PROTOTYPE_RE.finditer(masked)]


def _code_mask(text: str) -> str:
    """Return text with comments and literals blanked, preserving offsets."""
    out = list(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                out[pos] = " "
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(i, end):
                if out[pos] != "\n":
                    out[pos] = " "
            i = end
        elif text[i] == "'" and not opens_a_literal(text, i):
            i += 1  # Rust label/lifetime (`'cleanup:`) — opens nothing. See lex_mask.
        elif text[i] in ('"', "'"):
            quote = text[i]
            # Include a preceding L/u/U prefix in the literal mask.
            if i > 0 and text[i - 1] in "LuU" and (i < 2 or not text[i - 2].isalnum()):
                out[i - 1] = " "
            out[i] = " "
            i += 1
            while i < len(text):
                out[i] = " " if text[i] != "\n" else "\n"
                if text[i] == "\\" and i + 1 < len(text):
                    i += 1
                    out[i] = " " if text[i] != "\n" else "\n"
                elif text[i] == quote:
                    i += 1
                    break
                i += 1
        else:
            i += 1
    return "".join(out)


def _matching_paren(text: str, opening: int) -> int | None:
    depth = 0
    for pos in range(opening, len(text)):
        if text[pos] == "(":
            depth += 1
        elif text[pos] == ")":
            depth -= 1
            if depth == 0:
                return pos
    return None


def canonicalize_hresult_predicates(text: str) -> str:
    """Lower SUCCEEDED/FAILED calls to their exact signed-HRESULT tests."""
    while True:
        masked = _code_mask(text)
        matches = list(re.finditer(r"\b(SUCCEEDED|FAILED)\s*\(", masked))
        if not matches:
            return text
        # Rightmost-first naturally handles nested predicates without offset drift.
        changed = False
        for match in reversed(matches):
            opening = masked.find("(", match.start(1) + len(match.group(1)))
            closing = _matching_paren(masked, opening)
            if closing is None:
                continue
            expression = text[opening + 1 : closing]
            operator = ">=" if match.group(1) == "SUCCEEDED" else "<"
            text = text[: match.start(1)] + f"(({expression}) {operator} 0)" + text[closing + 1 :]
            changed = True
        if not changed:
            return text


def _declared_names(masked: str) -> frozenset[str]:
    """Identifiers this body introduces as storage (parameters and locals).

    Sealing storage is the one actively harmful direction: the fact contract tells
    the model the token is an opaque value to reproduce, when it is really a
    variable it needs to read or assign.  A missed seal just costs a paid call.
    """
    return frozenset(match.group(1) for match in _DECLARATOR_RE.finditer(masked))


_ZERO_INIT_RE = re.compile(r"=\s*\{\s*(0)\s*\}")


def _aggregate_zero_init_spans(masked: str) -> set[tuple[int, int]]:
    """Spans of the `0` in a C aggregate zero-initializer `= {0}`.

    `PROPSHEETPAGE psp = {0};` is SYNTAX, not a value: it says "all bytes zero", and the
    faithful Rust lowerings (`core::mem::zeroed()`, `Default::default()`, a struct literal
    naming every field) have nowhere to put the token. Demanding it rejects a correct
    translation -- netid's `CreateNetIDPropertyPage` died exactly here, on output that was
    otherwise perfect.

    Same class the shell32 wave generalized: the envelope assumes the lowering preserves a
    1:1 token correspondence with the C, and every construct that must be RESTRUCTURED breaks
    that assumption. Narrow by design -- only the single-`0` form, which is unambiguous. A
    `{0, 1}` or `{NULL}` initializer still seals, because there the values carry meaning.
    """
    return {match.span(1) for match in _ZERO_INIT_RE.finditer(masked)}


def _signature_array_bound_spans(masked: str) -> list[tuple[int, int]]:
    """The function's OWN parameter list — nothing in it can be sealed.

    One rule, and by now three constructs that proved it: the thing sealing would protect
    exists only in C/C++ declarator syntax, so there is nowhere in the TRANSLATED signature
    to put a token standing for it. rustconv generates that signature deterministically and
    the model never writes it, so the audit rejects every attempt no matter how correct the
    body is — the envelope is unsatisfiable, not merely hard.

    * An ARRAY PARAMETER's bound. A C array parameter decays to a pointer, so the `2` in
      `LPITEMIDLIST apidls[2]` has no counterpart in the Rust signature. shell32's
      `DoGetAliasPIDLs` burned four paid retries on this before the prompt was read.
    * A DEFAULT ARGUMENT's value. Rust has no default arguments at all, so the `FALSE` in
      `BOOL AllowFolder = FALSE` cannot appear either — `ExtensionFromPidl` burned three.
    * A PARAMETER TYPE that is a typedef. `IEnumFORMATETCImpl::Clone(LPENUMFORMATETC* ppenum)`
      sealed `LPENUMFORMATETC`, and for a COM method the signature comes from the flattened
      vtable authority rather than the C declaration, so the token had nowhere to land at
      all. Rejected on every sample, including the sanctioned retry.

    That last one generalises the whole helper: a parameter list holds types, parameter
    names, bounds and defaults, and NONE of those is a value. So the exclusion is now the
    entire list rather than two special cases. This only ever seals LESS, and the resume
    cache compares seals directionally — a banked entry sealed more tightly than a run wants
    is still sound — so widening it re-bills nothing.

    Scoped to the PARAMETER LIST specifically — the first balanced `(...)` of the signature —
    and not merely "before the first brace". `int a[2] = {0, 1};` has its first brace at the
    INITIALIZER, so the looser rule silently stopped sealing a real local array's bound.
    A bound, an `=`, or a type name in the BODY is a genuine value and must stay sealed.
    """
    body = masked.find("{")
    opening = masked.find("(")
    if opening < 0 or (body >= 0 and body < opening):
        return []  # no signature here (a bare statement, or a body with no parameter list)
    depth = 0
    closing = -1
    for index in range(opening, len(masked)):
        if masked[index] == "(":
            depth += 1
        elif masked[index] == ")":
            depth -= 1
            if depth == 0:
                closing = index
                break
    if closing < 0:
        return []
    # PROVE it is a definition header before excluding the whole span. The old
    # bounds-and-defaults rule was safe on a fragment that merely happens to start with a
    # call — `Call(TRUE); Call(FALSE);` has no `[` and no top-level `=`, so it excluded
    # nothing. Excluding the whole parenthesised range is not safe that way: it would
    # unseal TRUE, and `n = Len - sizeof(UNICODE_NULL)` likewise. A real definition's
    # parameter list is followed by the body `{`, a ctor initialiser list `:`, or a C++
    # trailing qualifier and then one of those.
    tail = masked[closing + 1 :]
    tail = re.sub(r"^(?:\s|\bconst\b|\bvolatile\b|\bnoexcept\b|\boverride\b|\bfinal\b)*", "", tail)
    if not tail.startswith(("{", ":")):
        return []
    return [(opening, closing + 1)]


_ISLAND_CONST_RE = re.compile(r"^\s*pub const ([A-Za-z_]\w*)\s*:", re.M)


def island_value_names(crate_src) -> frozenset[str]:
    """The MIXED-case constants of a crate's bindgen islands (`types_<arch>.rs`), for
    `seal_facts(extra_values=...)`. A cppapp lowering renames a nested enum constant to its
    island name, `CImage::excludeDefaultLoad` -> `ATL_CImage_ExcludeFlags_excludeDefaultLoad`,
    which neither the ALL_CAPS matcher nor the DDK CamelCase list can see: mspaint's main.cpp
    came back with `1 /* ..excludeDefaultLoad */` for a value of 0, and ShouldExcludeFormat
    with all eight flags permuted. A `pub const` shares the value namespace with fns and
    statics, so the name is unambiguously a constant in that crate."""
    names: set[str] = set()
    for path in sorted(crate_src.glob("types_*.rs")):
        names.update(_ISLAND_CONST_RE.findall(path.read_text(errors="replace")))
    return frozenset(n for n in names if any(c.islower() for c in n))


def seal_facts(
    text: str, *, dialect: str = "cpp", extra_values: frozenset[str] = frozenset()
) -> SealedSource:
    """Replace each value occurrence with a unique opaque identifier.

    `dialect` selects the protected value families; see the module docstring.
    ``"cpp"`` is frozen for response-cache compatibility. `extra_values` (the ``"c"``
    dialect only) are further names known to be constants: `island_value_names`.
    """
    if dialect not in ("c", "cpp"):
        # Fail loud: a typo'd dialect must not silently fall back to weaker sealing.
        raise ValueError(f"seal_facts: unknown dialect {dialect!r} (expected 'c' or 'cpp')")
    masked = _code_mask(text)
    spans: list[tuple[int, int, str]] = []
    declared = _declared_names(masked) if dialect == "c" else frozenset()
    # Positions where a symbol is a TYPE, not a value. Both are the same judgement
    # the scanner already makes for `(TYPE *)value`; neither can hold a constant.
    # Applied to the INTEGER scanner as well as the symbol one — an array bound is very nearly
    # always a literal, so filtering only symbols would have missed every real case.
    signature_bounds = _signature_array_bound_spans(masked)
    type_spans = _block_scope_prototype_spans(masked) + _named_cast_spans(masked) + signature_bounds
    # Nothing inside a `#define`/`#undef`/`#if` can be sealed — see lex_mask.directive_spans.
    directive = directive_spans(text)

    zero_init = _aggregate_zero_init_spans(masked)
    for match in _INTEGER_RE.finditer(masked):
        if (match.start(), match.end()) in zero_init:
            continue
        if any(a <= match.start() < b for a, b in signature_bounds):
            continue
        spans.append((match.start(), match.end(), "integer"))

    for match in (_C_SYMBOL_RE if dialect == "c" else _SYMBOL_RE).finditer(masked):
        word = match.group(0)
        if word in _NON_FACT_WORDS or word.startswith("__RUSTCONV_"):
            continue
        if dialect == "c" and word in _C_STATEMENT_MACROS:
            continue
        tail = masked[match.end() :]
        if tail.lstrip().startswith("("):
            continue
        # Exclude `(TYPE *)value` and `TYPE *name` contexts.
        start = match.start()
        closing = re.match(r"\s*\**\s*\)", tail)
        after_cast = tail[closing.end() :].lstrip() if closing is not None else ""
        cast_type = (
            start > 0
            and masked[start - 1] == "("
            and closing is not None
            and bool(after_cast)
            and after_cast[0] not in ",);]}"
        )
        declaration_type = bool(re.match(r"\s+\**\s*[A-Za-z_]\w*", tail))
        if cast_type or declaration_type:
            continue
        if any(a <= start < b for a, b in type_spans):
            continue
        spans.append((match.start(), match.end(), "symbol"))

    if dialect == "c":
        for match in _CAMEL_RE.finditer(masked):
            word = match.group(0)
            # The generated ground truth is the ONLY authority for what is a value
            # here; a capitalised identifier is otherwise far more likely to be a
            # function, type, or variable.
            if word not in DDK_ENUM_CONSTANTS or word in declared:
                continue
            tail = masked[match.end() :]
            if tail.lstrip().startswith("("):  # callee, not a value
                continue
            before = masked[: match.start()].rstrip()
            if before.endswith(".") or before.endswith("->"):  # struct field access
                continue
            if re.match(r"\s+\**\s*[A-Za-z_]\w*", tail):  # `Type name` declaration
                continue
            spans.append((match.start(), match.end(), "enum"))

    if dialect == "c" and extra_values:
        taken = {(a, b) for a, b, _ in spans}
        for match in re.finditer(r"\b[A-Za-z_]\w*\b", masked):
            word = match.group(0)
            if word not in extra_values or word in declared:
                continue
            if (match.start(), match.end()) in taken:
                continue
            tail = masked[match.end() :]
            if tail.lstrip().startswith("("):
                continue
            before = masked[: match.start()].rstrip()
            if before.endswith(".") or before.endswith("->") or before.endswith("::"):
                continue
            spans.append((match.start(), match.end(), "enum"))

    # No token can match both patterns (the generated set excludes ALL_CAPS names),
    # but sorting also makes numbering stable.
    spans = [s for s in spans if not any(directive[i] for i in range(s[0], s[1]))]
    spans.sort()
    dead_spans = dead_region_spans(text, masked)
    facts: list[SealedFact] = []
    result = text
    for index, (start, end, kind) in reversed(list(enumerate(spans))):
        placeholder = f"__RUSTCONV_FACT_{index:04d}__"
        original = text[start:end]
        in_sizeof = bool(
            re.search(r"\bsizeof\s*\(\s*$", masked[:start]) and re.match(r"\s*\)", masked[end:])
        )
        facts.append(
            SealedFact(
                placeholder, original, kind, in_dead_region(dead_spans, start, end), in_sizeof
            )
        )
        result = result[:start] + placeholder + result[end:]
    facts.reverse()
    return SealedSource(result, tuple(facts))


def prepare_sealed_source(
    text: str, *, dialect: str = "cpp", extra_values: frozenset[str] = frozenset()
) -> SealedSource:
    """Canonicalize deterministic macros, then seal the resulting facts."""
    return seal_facts(
        canonicalize_hresult_predicates(text), dialect=dialect, extra_values=extra_values
    )


def fact_contract(sealed: SealedSource) -> str:
    """Prompt contract that reveals tokens, never their guarded values."""
    if not sealed.facts:
        return ""
    tokens = ", ".join(fact.placeholder for fact in sealed.facts)
    return (
        "\n## Deterministic fact envelope\n"
        "The identifiers below are opaque value tokens owned by rustconv. Preserve EACH "
        "token at least once in executable code. Repeating a token is allowed when Rust "
        "syntax needs the same value more than once. Do not rename, define, evaluate, "
        "replace, or omit one; rustconv restores their values after your response.\n"
        f"Tokens ({len(sealed.facts)}): {tokens}\n"
    )


_USE_OF_TOKEN_RE = re.compile(
    r"^[ \t]*use\s+__RUSTCONV_(?:CALL|FACT|WIDE)_[0-9]{4}__\s*;[ \t]*\n?", re.M
)


def drop_token_use_items(rust: str) -> str:
    """Delete `use <seal-token>;` items a model wrote.

    A seal token stands for a VALUE, so it can never be a path. Cheap models read
    the token as an unresolved name and emit `use __RUSTCONV_FACT_0000__;`, which
    unseals to `use MAX_PATH;` / `use 100;`; the assembler then hoists both to file
    scope and the module does not parse.

    Handled deterministically rather than by adding a rule to `fact_contract`,
    because the contract text is hashed into the resume-cache key: rewording it
    would invalidate 1,006 banked sealed entries and re-charge for translations we
    already own. Deleting the item is also strictly safer than trusting a prompt —
    if the `use` was the token's ONLY occurrence, the closed-set check below still
    reports it missing and fails loudly.

    The COM path never hit this: its helper contract already says "use its supplied
    name directly", and normalize_worker_body strips model-authored items.
    """
    return _USE_OF_TOKEN_RE.sub("", rust)


def unseal_facts(
    rust: str,
    sealed: SealedSource,
    *,
    dead_omissions: list[str] | None = None,
    ignored_envelope: list[str] | None = None,
    strict: bool = True,
) -> str:
    """Verify the closed token set and restore exact original spellings.

    `dead_omissions`, when given, collects `TOKEN=SPELLING` for each sealed value the
    model omitted that lives only in non-executed code -- accepted, but reported, so a
    relaxed guarantee is never silent.

    `strict` False is the LAST-ATTEMPT mode: a body that ignored the envelope wholesale
    is accepted rather than rejected, and the fact is appended to `ignored_envelope` so
    the run reports it. Same bargain as `dead_omissions` -- the guarantee relaxes, and
    it never relaxes quietly. See the RESCUE_FLOOR block for what is being given up.
    """
    rust = drop_token_use_items(rust)
    expected = {fact.placeholder for fact in sealed.facts}
    present = _PLACEHOLDER_RE.findall(rust)
    unknown = sorted(set(present) - expected)
    missing_names = expected - set(present)

    # ── the recovery path has a floor, and it is load-bearing ────────────────────
    #
    # Recovery exists for a model that replaced ONE opaque token with a correctly
    # guessed spelling (usually zero). It re-tokenizes matching spellings in SOURCE
    # ORDER, which is sound for equal-valued occurrences of one spelling.
    #
    # What it must never do is rescue a body that ignored the envelope WHOLESALE.
    # msvcrt's `_tzset` is the worked example: 35 tokens, none preserved, the model
    # wrote real spellings from memory instead — and it put them in the wrong slots.
    # `_lock(0)` for `_lock(_TIME_LOCK)`, `memset(&tzi, 0, 3)` for `sizeof(tzi)`,
    # `*tz == 3600` for `*tz == '-'`, `tzi.Bias * CP_ACP` for `tzi.Bias * 60`. Every
    # value the C owns was PRESENT, so the closed-set check passed; the multiset was
    # right and the PERMUTATION was wrong. Recovery then re-tokenized the guesses and
    # restoration became an identity: the seal reported success while shipping the
    # model's memory of the function verbatim.
    #
    # There is no way to verify placement from the response alone — that is exactly
    # what preserving the tokens is FOR. So the honest answer is to refuse: a body
    # that preserved nothing has an unverified constant set, whatever its multiset
    # says, and a paid retry is cheaper than a silent wrong answer.
    #
    # The floor counts DISTINCT SPELLINGS, not occurrences, because that is the size of
    # the permutation space: `return 0 + 0 + S_OK` has three tokens and two spellings,
    # and there is no way to get it wrong that recovery would not catch. `_tzset` has
    # thirty-five tokens and TEN spellings — {0, 1, 3, 10, 60, 3600, TRUE, FALSE,
    # CP_ACP, TIME_ZONE_ID_INVALID} — and got nearly all of them into the wrong slots.
    # Four is where "the model happened to write the same values" stops being a
    # plausible account of a body that preserved no tokens at all.
    RESCUE_FLOOR = 4
    distinct_missing = {f.original for f in sealed.facts if f.placeholder in missing_names}
    if missing_names and not unknown and not set(present) and len(distinct_missing) >= RESCUE_FLOOR:
        note = (
            f"model ignored deterministic fact envelope entirely "
            f"({len(missing_names)} tokens over {len(distinct_missing)} distinct "
            f"spellings, none preserved) — its constants are "
            f"unverified even if every value appears, because the seal cannot check "
            f"PLACEMENT without the tokens"
        )
        if strict:
            raise FactSealError(note)
        if ignored_envelope is not None:
            ignored_envelope.append(note)

    # Cheap models sometimes replace an opaque token with a correctly guessed
    # spelling, especially zero. Recover only exact executable occurrences of
    # the source-owned spelling. Since equal-valued occurrence tokens restore
    # identically, assigning them in source order does not delegate a value or
    # weaken the closed value set. Comments and string/char literals are masked.
    if missing_names and not unknown:
        masked = _code_mask(rust)
        replacements: list[tuple[int, int, str]] = []
        by_original: dict[tuple[str, str], list[SealedFact]] = {}
        for fact in sealed.facts:
            if fact.placeholder in missing_names:
                by_original.setdefault((fact.original, fact.kind), []).append(fact)
        for (original, kind), facts in by_original.items():
            if kind in ("symbol", "enum"):
                pattern = re.compile(rf"\b{re.escape(original)}\b")
            else:
                pattern = re.compile(rf"(?<![A-Za-z0-9_]){re.escape(original)}(?![A-Za-z0-9_])")
            spans = [(match.start(), match.end()) for match in pattern.finditer(masked)]
            if len(spans) < len(facts):
                continue
            replacements.extend(
                (start, end, fact.placeholder)
                for (start, end), fact in zip(spans, facts, strict=False)
            )
        for start, end, placeholder in sorted(replacements, reverse=True):
            rust = rust[:start] + placeholder + rust[end:]
        present = _PLACEHOLDER_RE.findall(rust)

    # A token whose only occurrence is in code the build never executes is PERMITTED,
    # not demanded: the model is right to omit it, and rejecting a correct translation
    # costs a paid retry that will be rejected identically (dead_regions.py).
    optional = {fact.placeholder for fact in sealed.facts if fact.dead}
    missing_all = expected - set(present)
    omitted_dead = sorted(missing_all & optional)
    missing = sorted(missing_all - optional)
    if omitted_dead and dead_omissions is not None:
        dead_omissions.extend(
            f"{fact.placeholder}={fact.original}"
            for fact in sealed.facts
            if fact.placeholder in set(omitted_dead)
        )
    findings = []
    if missing:
        findings.append("missing: " + ", ".join(missing))
    if unknown:
        findings.append("invented: " + ", ".join(unknown))
    if findings:
        raise FactSealError(
            "model damaged deterministic fact envelope (" + "; ".join(findings) + ")"
        )

    for fact in sealed.facts:
        spelling = rust_integer_spelling(fact.original) if fact.kind == "integer" else fact.original
        if fact.in_sizeof and fact.kind == "symbol":
            rust = _restore_dropped_sizeof(rust, fact.placeholder)
        rust = rust.replace(fact.placeholder, spelling)
    return rust


def _restore_dropped_sizeof(rust: str, placeholder: str) -> str:
    """`TOKEN` where the C had `sizeof(TOKEN)` -> `core::mem::size_of::<TOKEN>()`, unless the
    model already measured it (`size_of::<TOKEN>`, `size_of_val(&TOKEN)`)."""

    def repl(m: re.Match[str]) -> str:
        before = rust[: m.start()]
        if re.search(
            r"size_of(?:_val)?\s*(?:::\s*<\s*|\(\s*&?\s*(?:raw\s+(?:const|mut)\s+)?)$", before
        ):
            return m.group(0)
        return f"core::mem::size_of::<{placeholder}>()"

    return re.sub(rf"\b{re.escape(placeholder)}\b", repl, rust)


def rust_integer_spelling(c: str) -> str:
    """The same VALUE as a C integer literal, spelled as valid Rust.

    The seal restores what the C wrote, and two C spellings are not Rust: a `U`/`L`/`ULL`
    suffix does not compile (setuplib fsrec.c's `8192ULL`), and a leading-zero OCTAL literal
    compiles to the wrong number -- `0644` is decimal 644 in Rust. The type a suffix asked
    for is the surrounding code's business, exactly as for an unsuffixed literal.
    """
    m = re.fullmatch(r"(0[xX][0-9A-Fa-f]+|0[oO][0-7]+|[0-9]+)([uUlL]*)", c)
    if not m:
        return c
    digits = m.group(1)
    if len(digits) > 1 and digits[0] == "0" and digits[1].isdigit():
        return "0o" + digits[1:]
    return digits
