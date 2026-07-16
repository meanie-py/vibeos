"""Closed-set sealing for runtime callee occurrences in body-worker prompts."""

from __future__ import annotations

import re
from dataclasses import dataclass


class CallSealError(RuntimeError):
    pass


@dataclass(frozen=True)
class SealedCall:
    placeholder: str
    original: str


@dataclass(frozen=True)
class SealedCalls:
    text: str
    calls: tuple[SealedCall, ...]


_PLACEHOLDER_RE = re.compile(r"\b__RUSTCONV_CALL_[0-9]{4}__\b")
_EXCLUDED = {
    "TRACE",
    "WARN",
    "ERR",
    "FIXME",
    "MESSAGE",
    "SUCCEEDED",
    "FAILED",
    "FAILED_UNEXPECTEDLY",
    "sizeof",
    "_countof",
}


def _code_mask(text: str) -> str:
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
            i += 1
            while i < len(text):
                if text[i] == "\\":
                    out[i] = " "
                    if i + 1 < len(text):
                        out[i + 1] = " "
                    i += 2
                    continue
                if text[i] == quote:
                    i += 1
                    break
                if out[i] != "\n":
                    out[i] = " "
                i += 1
        else:
            i += 1
    return "".join(out)


def seal_calls(text: str, required_calls: set[str]) -> SealedCalls:
    """Replace each known runtime-callee occurrence with a unique token."""
    required = required_calls - _EXCLUDED
    if not required:
        return SealedCalls(text, ())
    masked = _code_mask(text)
    body_start = masked.find("{")
    spans = []
    for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
        name = match.group(1)
        if match.start(1) <= body_start or name not in required:
            continue
        # C++ direct initialization looks call-like to a lexical scanner:
        # ``CComHeapPtr<ITEMIDLIST> pidl(factory())``. The declarator name is
        # storage, not a runtime callee.
        statement_start = max(
            body_start,
            masked.rfind(";", body_start, match.start(1)),
            masked.rfind("{", body_start, match.start(1)),
            masked.rfind("}", body_start, match.start(1)),
        )
        prefix = masked[statement_start + 1 : match.start(1)].lstrip()
        if re.fullmatch(r"[A-Z_]\w*(?:\s*<[^;{}()]+>)?\s+\**\s*", prefix):
            continue
        spans.append((match.start(1), match.end(1), name))

    result = text
    calls = []
    for index, (start, end, original) in reversed(list(enumerate(spans))):
        placeholder = f"__RUSTCONV_CALL_{index:04d}__"
        calls.append(SealedCall(placeholder, original))
        result = result[:start] + placeholder + result[end:]
    calls.reverse()
    return SealedCalls(result, tuple(calls))


def call_contract(sealed: SealedCalls) -> str:
    if not sealed.calls:
        return ""
    tokens = ", ".join(call.placeholder for call in sealed.calls)
    return (
        "\n## Deterministic call envelope\n"
        "Each identifier below is one opaque runtime callee occurrence owned by rustconv. "
        "Preserve EACH token exactly once as the called function/method identifier in "
        "executable Rust. Do not rename, define, merge, duplicate, move into a comment, "
        "or omit one; rustconv restores the callee names after your response.\n"
        f"Call tokens ({len(sealed.calls)}): {tokens}\n"
    )


def unseal_calls(rust: str, sealed: SealedCalls) -> str:
    expected = {call.placeholder for call in sealed.calls}
    masked = _code_mask(rust)
    executable = _PLACEHOLDER_RE.findall(masked)
    missing_tokens = expected - set(executable)

    # A model sometimes sees a generated sibling-worker declaration in context
    # and writes that exact callee name instead of preserving its opaque token.
    # Recover only the mechanically unambiguous case: every missing token for one
    # callee has exactly one matching executable direct-call identifier.  Comments,
    # declarations, duplicates, and approximate spellings remain hard failures.
    missing_by_original: dict[str, list[SealedCall]] = {}
    for call in sealed.calls:
        if call.placeholder in missing_tokens:
            missing_by_original.setdefault(call.original, []).append(call)
    replacements: list[tuple[int, int, str]] = []
    for original, calls in missing_by_original.items():
        matches = []
        for match in re.finditer(rf"\b{re.escape(original)}\s*(?=\()", masked):
            prefix = masked[max(0, match.start() - 8) : match.start()]
            if re.search(r"\bfn\s*$", prefix):
                continue
            matches.append(match)
        if len(matches) == len(calls):
            replacements.extend(
                (match.start(), match.start() + len(original), call.placeholder)
                for match, call in zip(matches, calls, strict=True)
            )
    for start, end, placeholder in sorted(replacements, reverse=True):
        rust = rust[:start] + placeholder + rust[end:]

    executable = _PLACEHOLDER_RE.findall(_code_mask(rust))
    all_present = _PLACEHOLDER_RE.findall(rust)
    missing = sorted(expected - set(executable))
    duplicates = sorted(token for token in expected if all_present.count(token) > 1)
    unknown = sorted(set(all_present) - expected)
    findings = []
    if missing:
        findings.append("missing/non-executable: " + ", ".join(missing))
    if duplicates:
        findings.append("not exactly once: " + ", ".join(duplicates))
    if unknown:
        findings.append("invented: " + ", ".join(unknown))
    if findings:
        raise CallSealError(
            "model damaged deterministic call envelope (" + "; ".join(findings) + ")"
        )
    for call in sealed.calls:
        rust = rust.replace(call.placeholder, call.original)
    return rust
