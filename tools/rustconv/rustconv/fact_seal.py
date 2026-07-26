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


@dataclass(frozen=True)
class SealedSource:
    text: str
    facts: tuple[SealedFact, ...]


_PLACEHOLDER_RE = re.compile(r"\b__RUSTCONV_FACT_[0-9]{4}__\b")
_INTEGER_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?:0[xX][0-9A-Fa-f]+|0[oO][0-7]+|[0-9]+)"
    r"(?:[uUlL]{0,3})(?![A-Za-z0-9_])"
)
_SYMBOL_RE = re.compile(r"\b[A-Z][A-Z0-9_]+\b")

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


def seal_facts(text: str, *, dialect: str = "cpp") -> SealedSource:
    """Replace each value occurrence with a unique opaque identifier.

    `dialect` selects the protected value families; see the module docstring.
    ``"cpp"`` is frozen for response-cache compatibility.
    """
    if dialect not in ("c", "cpp"):
        # Fail loud: a typo'd dialect must not silently fall back to weaker sealing.
        raise ValueError(f"seal_facts: unknown dialect {dialect!r} (expected 'c' or 'cpp')")
    masked = _code_mask(text)
    spans: list[tuple[int, int, str]] = []
    declared = _declared_names(masked) if dialect == "c" else frozenset()

    for match in _INTEGER_RE.finditer(masked):
        spans.append((match.start(), match.end(), "integer"))

    for match in _SYMBOL_RE.finditer(masked):
        word = match.group(0)
        if word in _NON_FACT_WORDS:
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

    # No token can match both patterns (the generated set excludes ALL_CAPS names),
    # but sorting also makes numbering stable.
    spans.sort()
    dead_spans = dead_region_spans(text, masked)
    facts: list[SealedFact] = []
    result = text
    for index, (start, end, kind) in reversed(list(enumerate(spans))):
        placeholder = f"__RUSTCONV_FACT_{index:04d}__"
        original = text[start:end]
        facts.append(
            SealedFact(placeholder, original, kind, in_dead_region(dead_spans, start, end))
        )
        result = result[:start] + placeholder + result[end:]
    facts.reverse()
    return SealedSource(result, tuple(facts))


def prepare_sealed_source(text: str, *, dialect: str = "cpp") -> SealedSource:
    """Canonicalize deterministic macros, then seal the resulting facts."""
    return seal_facts(canonicalize_hresult_predicates(text), dialect=dialect)


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
    r"^[ \t]*use\s+__RUSTCONV_(?:CALL|FACT)_[0-9]{4}__\s*;[ \t]*\n?", re.M
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
    rust: str, sealed: SealedSource, *, dead_omissions: list[str] | None = None
) -> str:
    """Verify the closed token set and restore exact original spellings.

    `dead_omissions`, when given, collects `TOKEN=SPELLING` for each sealed value the
    model omitted that lives only in non-executed code -- accepted, but reported, so a
    relaxed guarantee is never silent.
    """
    rust = drop_token_use_items(rust)
    expected = {fact.placeholder for fact in sealed.facts}
    present = _PLACEHOLDER_RE.findall(rust)
    unknown = sorted(set(present) - expected)
    missing_names = expected - set(present)

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
        rust = rust.replace(fact.placeholder, fact.original)
    return rust
