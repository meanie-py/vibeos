"""Closed-set sealing for runtime callee occurrences in body-worker prompts."""

from __future__ import annotations

import re
from dataclasses import dataclass

from .lex_mask import comment_spans, mask_code


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
# Superseded by `macro_authority.function_like_macros`, which derives this set from
# the real preprocessor instead of remembering it. Kept as a floor for the two cases
# the preprocessor cannot name: `sizeof` is a KEYWORD, not a macro, and the debug
# macros are stripped from the body before sealing anyway. Everything else here is a
# function-like macro the derived set already covers — see that module for why a
# macro can never satisfy the "exactly once as a called identifier" envelope.
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


# Shared with fact_seal and cpp_com. It masks Rust as well as C++, which the three
# private copies it replaced did not: each treated `'cleanup:` as an open char
# literal and blanked the body to the next apostrophe. See `lex_mask`.
_code_mask = mask_code


def scrub_comment_token_echoes(text: str) -> str:
    """Neutralize seal tokens repeated only in model-authored comments."""
    out = list(text)
    comment = comment_spans(text)
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


def seal_calls(
    text: str,
    required_calls: set[str],
    function_like_macros: frozenset[str] = frozenset(),
) -> SealedCalls:
    """Replace each known runtime-callee occurrence with a unique token.

    ``function_like_macros`` names every `#define NAME(...)` visible to the TU (see
    `macro_authority`). Those are excluded: a macro occurrence has no 1:1 Rust call
    to carry the token, so sealing one makes the envelope unsatisfiable while
    protecting nothing — the macro's name is not in the output to begin with.
    """
    required = required_calls - _EXCLUDED - function_like_macros
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


_GOTO_RE = re.compile(r"\bgoto\s+([A-Za-z_]\w*)\s*;")
_LABEL_RE = re.compile(r"(?m)^[ \t]*([A-Za-z_]\w*)[ \t]*:(?!:)")


def _name_satisfies(token: str, sealed: SealedCalls, replicable: set[str], exec_text: str) -> bool:
    """Whether a token's own callee NAME, written out in call position, stands in.

    Only inside the `goto` replication region, where occurrence identity is already
    conceded (see `_replicable_after_goto`). Duplicating the cleanup tail makes the
    tail's tokens ambiguous, and a model that has just written the same block four
    times routinely stops threading opaque tokens through it and writes what the
    callee-context block told it the callee is called:

        HeapFree(GetProcessHeap(), 0, nFileOp.pFrom as *mut c_void)

    That is not a dropped call, which is the only thing `missing` exists to catch —
    and it is character-for-character what unsealing would have produced anyway, so
    accepting it changes no output byte. A name the model INVENTED still fails: it
    would not match `original`. Outside the region, "exactly once" is untouched.
    """
    if token not in replicable:
        return False
    original = next((call.original for call in sealed.calls if call.placeholder == token), None)
    if not original:
        return False
    return re.search(rf"(?<!fn )\b{re.escape(original)}\s*\(", exec_text) is not None


def _replicable_after_goto(sealed: SealedCalls) -> set[str]:
    """Tokens a structured lowering of C `goto` may legitimately write more than once.

    Rust has no `goto`, so every lowering DUPLICATES code, and how much is not a
    property the translation is free to choose consistently. Three consecutive paid
    responses for the same two functions produced three different faithful shapes:

      * the label's tail inlined at each jump site (`SHFileOperationA`, 4 copies);
      * the same, 3 copies (`SHELL32_FileOperation`);
      * a nested cascade duplicating the WHOLE remainder per path — which replicates
        calls that sit nowhere near the label.

    A count-based budget was tried first (`#goto + 1`, tail only) and the third shape
    walked straight through it. So the rule is positional, not numeric: from the point
    where duplication can begin — the earlier of the first `goto` and its target label,
    which also covers the backwards retry-loop idiom — a token may repeat.

    "Exactly once" survives everywhere else, which is most bodies. And this weakens
    only the DUPLICATE half of the envelope: `missing` still catches a dropped or
    commented-out call, and `invented` still catches a hallucinated token — that half
    caught three invented FACT tokens on the very run this was written for.

    Residual risk, stated plainly: inside the replication region this cannot tell "the
    same call on N distinct exit paths" (correct) from "the same call twice on one
    path" (a double free). Neither could the old check — it rejected the whole TU
    either way, and one rejected worker assembles no file at all.
    """
    masked = _code_mask(sealed.text)
    gotos = _GOTO_RE.findall(masked)
    if not gotos:
        return set()
    first = _GOTO_RE.search(masked)
    assert first is not None
    start = first.start()
    targets = set(gotos)
    for match in _LABEL_RE.finditer(masked):
        # `case X:` / `default:` / a ternary are not jump targets.
        if match.group(1) in targets:
            start = min(start, match.start())
    # Positions are taken in the SEALED text, where each placeholder is unique; a
    # pre-seal offset diverges as soon as a token replaces a shorter name.
    return {
        call.placeholder for call in sealed.calls if sealed.text.find(call.placeholder) >= start
    }


def call_contract(sealed: SealedCalls) -> str:
    if not sealed.calls:
        return ""
    tokens = ", ".join(call.placeholder for call in sealed.calls)
    # "Do not duplicate" is a lie when the C body uses `goto`, and the models paid to
    # obey it acted accordingly: needing the cleanup tail on three paths and forbidden
    # to repeat a token, they MINTED NEW NUMBERS past the end of the list
    # (`dde_connect` invented three, `SIC_IconAppend` two). Saying what is actually
    # true costs nothing and removes the incentive.
    if _GOTO_RE.search(_code_mask(sealed.text)):
        rule = (
            "Preserve EACH token as the called function/method identifier in executable "
            "Rust. This body uses `goto`, which Rust has no equivalent for, so a faithful "
            "lowering may need the same call on more than one path: where that happens, "
            "REPEAT the identical token — repetition is expected here. Never invent a token "
            "number that is not listed below; the list is complete."
        )
    else:
        rule = (
            "Preserve EACH token exactly once as the called function/method identifier in "
            "executable Rust. Do not duplicate one."
        )
    return (
        "\n## Deterministic call envelope\n"
        "Each identifier below is one opaque runtime callee occurrence owned by rustconv. "
        f"{rule} Do not rename, define, merge, move into a comment, or omit one; rustconv "
        "restores the callee names after your response.\n"
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
    replicable = _replicable_after_goto(sealed)
    exec_text = "".join(exec_mask)
    missing = sorted(
        token
        for token in expected - set(executable)
        if not _name_satisfies(token, sealed, replicable, exec_text)
    )
    duplicates = sorted(
        token
        for token in expected
        if executable.count(token) > 1
        and token not in allowed_duplicates
        and token not in replicable
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
