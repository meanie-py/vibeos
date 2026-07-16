"""Deterministically protect C facts while an LLM translates control flow.

Cheap models are useful at syntax and notoriously unreliable at copying numeric
and symbolic constants.  This module removes that job from the model: every
constant occurrence becomes an opaque identifier, the response must return each
identifier at least once, and rustconv restores the original spelling afterward.

The transform is deliberately lexical.  It never touches comments, string/char
literals, callable macro names, or declaration/cast type names.
"""

from __future__ import annotations

import re
from dataclasses import dataclass


class FactSealError(RuntimeError):
    """The model damaged the deterministic fact envelope."""


@dataclass(frozen=True)
class SealedFact:
    placeholder: str
    original: str
    kind: str


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


def seal_facts(text: str) -> SealedSource:
    """Replace each value occurrence with a unique opaque identifier."""
    masked = _code_mask(text)
    spans: list[tuple[int, int, str]] = []

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

    # No token can match both regexes, but sorting also makes numbering stable.
    spans.sort()
    facts: list[SealedFact] = []
    result = text
    for index, (start, end, kind) in reversed(list(enumerate(spans))):
        placeholder = f"__RUSTCONV_FACT_{index:04d}__"
        original = text[start:end]
        facts.append(SealedFact(placeholder, original, kind))
        result = result[:start] + placeholder + result[end:]
    facts.reverse()
    return SealedSource(result, tuple(facts))


def prepare_sealed_source(text: str) -> SealedSource:
    """Canonicalize deterministic macros, then seal the resulting facts."""
    return seal_facts(canonicalize_hresult_predicates(text))


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


def unseal_facts(rust: str, sealed: SealedSource) -> str:
    """Verify the closed token set and restore exact original spellings."""
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
            if kind == "symbol":
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

    missing = sorted(expected - set(present))
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
