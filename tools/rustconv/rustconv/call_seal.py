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
_PURE_REUSABLE_CALLS = {"strlen", "wcslen"}
_ANY_SEAL_TOKEN_RE = re.compile(r"\b__RUSTCONV_(?:CALL|FACT)_[0-9]{4}__\b")


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


def scrub_comment_token_echoes(text: str) -> str:
    """Neutralize seal tokens repeated only in model-authored comments."""
    out = list(text)
    comment = [False] * len(text)
    i = 0
    while i < len(text):
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = len(text) if end < 0 else end
            for pos in range(i, end):
                comment[pos] = True
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            end = len(text) if end < 0 else end + 2
            for pos in range(i, end):
                comment[pos] = True
            i = end
        elif text[i] in ('"', "'"):
            quote = text[i]
            i += 1
            while i < len(text):
                if text[i] == "\\":
                    i += 2
                    continue
                i += 1
                if text[i - 1] == quote:
                    break
        else:
            i += 1
    for match in _ANY_SEAL_TOKEN_RE.finditer(text):
        if all(comment[match.start() : match.end()]):
            out[match.start() : match.end()] = "COMMENT_TOKEN_ECHO".ljust(
                match.end() - match.start(), "_"
            )
    return "".join(out)


def extern_declaration_spans(text: str) -> list[tuple[int, int]]:
    """Character spans of `extern "..." { … }` DECLARATION blocks.

    A forward declaration names a callee but does not call it, and the mechanical
    prompt explicitly ORDERS one: "emit a forward `extern "system" { fn Name(args...)
    -> Ret; }` block immediately above the function". So a correct mechanical
    translation writes every sealed callee twice — once declared, once called — and
    the "exactly once" duplicate check fails through no fault of the model.

    The shell32 COM path never hit this because `normalize_worker_body` deletes
    model-authored declaration-only extern blocks before unsealing. Plain C KEEPS
    them (they are how the assembler learns the signature), so the extra occurrence
    is sanctioned here instead — and only the extra one: a token must still appear
    exactly once in executable code.

    Only DECLARATION blocks count: a block whose `fn` items all end at `;`. An
    `extern "C" { fn f() { … } }` with a body is not one.
    """
    spans: list[tuple[int, int]] = []
    for match in re.finditer(r'\bextern\s*"[^"]*"\s*\{', text):
        depth = 0
        i = start = match.end() - 1
        while i < len(text):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        end = min(i + 1, len(text))
        if not re.search(r"\bfn\b[^;{]*\{", text[start:end]):
            spans.append((start, end))
    return spans


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
        # A C body may carry a block-scope ``extern RET Callee(...)`` prototype.
        # It is ABI context, not an executable call occurrence.
        if re.fullmatch(r"extern\s+[^;{}=()]+", prefix):
            continue
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
    rust = scrub_comment_token_echoes(rust)
    expected = {call.placeholder for call in sealed.calls}
    masked = _code_mask(rust)
    executable = _PLACEHOLDER_RE.findall(masked)
    missing_tokens = expected - set(executable)
    allowed_duplicates: set[str] = set()

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
        spellings = (original, f"{original}_body")
        pattern = "|".join(re.escape(spelling) for spelling in spellings)
        for match in re.finditer(rf"\b(?:{pattern})\s*(?=\()", masked):
            prefix = masked[max(0, match.start() - 8) : match.start()]
            if re.search(r"\bfn\s*$", prefix):
                continue
            matches.append(match)
        if len(matches) == len(calls):
            replacements.extend(
                (match.start(), match.end(), call.placeholder)
                for match, call in zip(matches, calls, strict=True)
            )
    for start, end, placeholder in sorted(replacements, reverse=True):
        rust = rust[:start] + placeholder + rust[end:]

    # Placeholder identity distinguishes source occurrences, not callee
    # semantics. If a model repeats one token while preserving the exact total
    # executable call count for that same callee, deterministically relabel the
    # occurrences in output order. Arguments and control flow remain untouched.
    for original in dict.fromkeys(call.original for call in sealed.calls):
        family = [call for call in sealed.calls if call.original == original]
        if len(family) < 2:
            continue
        family_names = {call.placeholder for call in family}
        masked = _code_mask(rust)
        matches = [
            match
            for match in _PLACEHOLDER_RE.finditer(masked)
            if match.group(0) in family_names and re.match(r"\s*\(", masked[match.end() :])
        ]
        if len(matches) != len(family):
            continue
        for match, call in reversed(list(zip(matches, family, strict=True))):
            rust = rust[: match.start()] + call.placeholder + rust[match.end() :]

    # C switch fallthrough becomes duplicated syntax in structured Rust when
    # the shared tail has side effects. Accept that one-to-many spelling only
    # for two byte-identical calls in mutually exclusive ``if/else if`` arms;
    # all other duplicated opaque calls remain a hard failure.
    if "switch" in sealed.text and "case" in sealed.text:
        masked = _code_mask(rust)
        for token in expected:
            occurrences = list(re.finditer(rf"\b{re.escape(token)}\s*\(", masked))
            if len(occurrences) != 2:
                continue
            calls = []
            for occurrence in occurrences:
                opening = masked.find("(", occurrence.start())
                depth = 0
                closing = -1
                for pos in range(opening, len(masked)):
                    if masked[pos] == "(":
                        depth += 1
                    elif masked[pos] == ")":
                        depth -= 1
                        if depth == 0:
                            closing = pos
                            break
                if closing < 0:
                    break
                calls.append(re.sub(r"\s+", "", masked[occurrence.start() : closing + 1]))
            between = masked[occurrences[0].end() : occurrences[1].start()]
            if len(calls) == 2 and calls[0] == calls[1] and re.search(r"}\s*else\s+if\b", between):
                allowed_duplicates.add(token)

    # Cheap models occasionally common-subexpression-eliminate a repeated pure
    # string-length call: ``let len = TOKEN_A(arg); ... len ... len``. The C
    # call envelope intentionally preserves occurrences, so restore TOKEN_B at
    # the final reuse only when the source calls the same proven-pure function,
    # exactly one token is missing, and the Rust binding/uses are unambiguous.
    masked = _code_mask(rust)
    executable_tokens = set(_PLACEHOLDER_RE.findall(masked))
    ordered = list(sealed.calls)
    for original in _PURE_REUSABLE_CALLS:
        family = [call for call in ordered if call.original == original]
        missing_family = [call for call in family if call.placeholder not in executable_tokens]
        if len(missing_family) != 1:
            continue
        missing_call = missing_family[0]
        missing_index = ordered.index(missing_call)
        prior = next(
            (
                call
                for call in reversed(ordered[:missing_index])
                if call.original == original and call.placeholder in executable_tokens
            ),
            None,
        )
        if prior is None:
            continue
        binding = re.search(
            rf"\blet\s+(?:mut\s+)?(?P<name>[A-Za-z_]\w*)"
            rf"(?:\s*:[^=;]+)?\s*=\s*{re.escape(prior.placeholder)}\s*"
            r"\(\s*(?P<arg>[^(),;]+?)\s*\)[^;]*;",
            masked,
        )
        if binding is None:
            continue
        name = binding.group("name")
        uses = list(re.finditer(rf"\b{re.escape(name)}\b", masked[binding.end() :]))
        if len(uses) < 2:
            continue
        use = uses[-1]
        start = binding.end() + use.start()
        end = binding.end() + use.end()
        arg = rust[binding.start("arg") : binding.end("arg")].strip()
        rust = rust[:start] + f"{missing_call.placeholder}({arg})" + rust[end:]
        masked = _code_mask(rust)
        executable_tokens.add(missing_call.placeholder)

    # Occurrences inside a forward `extern "..." { … }` declaration are not calls. The
    # mechanical contract REQUIRES those declarations, so they must not count toward
    # "exactly once" — but they must not satisfy it either, or a model could declare a
    # callee it never calls. Blank the declaration spans and judge both checks on
    # what is left, so a token still has to appear exactly once in executable code.
    exec_mask = list(_code_mask(rust))
    for a, b in extern_declaration_spans(rust):
        exec_mask[a:b] = " " * (b - a)
    executable = _PLACEHOLDER_RE.findall("".join(exec_mask))
    missing = sorted(expected - set(executable))
    duplicates = sorted(
        token
        for token in expected
        if executable.count(token) > 1 and token not in allowed_duplicates
    )
    all_present = _PLACEHOLDER_RE.findall(rust)
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
