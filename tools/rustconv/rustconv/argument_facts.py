"""Keep directly sealed constant arguments attached to their sealed call occurrence.

This bounded check catches dispatch/error-code permutations. It does not claim general
expression equivalence: C++ implicit receivers/defaults and compound C arguments remain
the responsibility of their structural checks and source review.
"""

import re

from .call_seal import CallSealError, SealedCalls, _match_paren, extern_declaration_spans
from .lex_mask import mask_code

_FACT = re.compile(r"__RUSTCONV_FACT_[0-9]{4}__")


def _arguments(text: str, token: str) -> list[list[str]]:
    masked = mask_code(text)
    declarations = extern_declaration_spans(text)
    result = []
    for match in re.finditer(rf"\b{re.escape(token)}\s*\(", masked):
        if any(a <= match.start() < b for a, b in declarations):
            continue
        opening = match.end() - 1
        try:
            closing = _match_paren(masked, opening)
        except ValueError:
            continue  # the call-envelope syntax check owns malformed calls
        interior = masked[opening + 1 : closing]
        # Turbofish commas need the Rust parser, not a guessed angle-bracket balance.
        if "::<" in interior:
            continue
        # A `#if` arm inside a C argument list makes the positions depend on which arm
        # compiles (comctl32's TOOLTIPS_Show); call_seal's arity check skips these too.
        if re.search(r"(?m)^[ \t]*#[ \t]*(?:if|ifdef|ifndef|elif|else|endif)\b", interior):
            continue
        args, start, depth = [], 0, 0
        for i, char in enumerate(interior):
            if char in "([{":
                depth += 1
            elif char in ")]}":
                depth -= 1
            elif char == "," and depth == 0:
                args.append(interior[start:i].strip())
                start = i + 1
        args.append(interior[start:].strip())
        result.append(args)
    return result


def require_argument_facts(rust: str, calls: SealedCalls) -> None:
    for call in calls.calls:
        if call.receiver or call.hwnd_receiver or call.argc_max is not None:
            continue
        source_sites = _arguments(calls.text, call.placeholder)
        if len(source_sites) != 1:
            continue
        expected = source_sites[0]
        for actual in _arguments(rust, call.placeholder):
            if len(actual) != len(expected):
                continue  # existing arity check owns this
            for index, (before, after) in enumerate(zip(expected, actual, strict=True)):
                if _FACT.fullmatch(before) and set(_FACT.findall(after)) != {before}:
                    raise CallSealError(
                        f"{call.original} argument {index + 1}: sealed constant moved or "
                        f"replaced (expected {before}, got {after})"
                    )
