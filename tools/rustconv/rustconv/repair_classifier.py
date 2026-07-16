"""Deterministic classification and bounded repair for candidate workers."""

from __future__ import annotations

import re
from dataclasses import dataclass

from .candidate_oracle import function_span, masked_rust
from .cpp_com import normalize_worker_body


@dataclass(frozen=True)
class RepairResult:
    code: str
    applied: tuple[str, ...]


RUSTC_CATEGORIES = {
    "E0435": "const_context",
    "E0308": "type_mismatch",
    "E0425": "unresolved_symbol",
    "E0433": "unresolved_symbol",
    "E0659": "ambiguous_import",
    "E0133": "unsafe_operation",
    "E0381": "uninitialized_binding",
    "E0384": "immutable_reassignment",
}


def classify_diagnostic(diagnostic: dict) -> str:
    code = diagnostic.get("code")
    if code in RUSTC_CATEGORIES:
        return RUSTC_CATEGORIES[code]
    message = (diagnostic.get("message") or "").lower()
    if "expected expression" in message or "expected one of" in message:
        return "raw_c_syntax"
    if "cannot find" in message or "failed to resolve" in message:
        return "unresolved_symbol"
    return "other"


def source_findings(rust: str) -> list[str]:
    findings = []
    if re.search(r"(?m)^\s*use\s+[^;]+::\*\s*;", rust):
        findings.append("glob_import")
    if re.search(r"(?m)\bgoto\b|^\s*[A-Za-z_]\w*:\s*$", rust):
        findings.append("raw_c_goto")
    if re.search(r"\?[^;{}]+:", rust):
        findings.append("raw_c_ternary")
    if re.search(r"\{\s*(?:let\s+_\s*=\s*[^;]+;\s*)+\}", rust, re.S):
        findings.append("stub_body")
    return findings


def _signature(function: str) -> str:
    masked = function
    opening = masked.find("{")
    if opening < 0:
        return function.strip()
    return function[:opening].strip()


_CAST_RE = re.compile(
    r"\s+as\s+(?:(?:\*const|\*mut)\s+)?"
    r"[A-Za-z_]\w*(?:::\w+)*(?:\s*<[^;\n,()]+>)?"
)


def _without_casts(expression: str) -> str:
    prior = None
    while expression != prior:
        prior = expression
        expression = _CAST_RE.sub("", expression)
    return re.sub(r"[\s()]", "", expression)


def _call_arguments(function: str) -> dict[str, list[list[tuple[int, int]]]]:
    """Return trimmed argument spans for each body call, grouped by callee."""
    masked = masked_rust(function)
    body_start = masked.find("{")
    calls: dict[str, list[list[tuple[int, int]]]] = {}
    for match in re.finditer(r"\b([A-Za-z_]\w*)\s*\(", masked):
        if match.start() <= body_start:
            continue
        opening = masked.find("(", match.start(1) + len(match.group(1)))
        depth = 0
        closing = None
        for pos in range(opening, len(masked)):
            char = masked[pos]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    closing = pos
                    break
        if closing is None:
            continue
        arguments = []
        start = opening + 1
        nested = 0
        for pos in range(start, closing + 1):
            char = masked[pos] if pos < closing else ","
            if char in "([{":
                nested += 1
            elif char in ")]}":
                nested -= 1
            elif char == "," and nested == 0:
                left, right = start, pos
                while left < right and function[left].isspace():
                    left += 1
                while right > left and function[right - 1].isspace():
                    right -= 1
                if left < right:
                    arguments.append((left, right))
                start = pos + 1
        calls.setdefault(match.group(1), []).append(arguments)
    return calls


def _restore_base_argument_casts(base_function: str, candidate_function: str) -> tuple[str, int]:
    """Copy cast-only argument shapes from an already accepted worker."""
    base_calls = _call_arguments(base_function)
    candidate_calls = _call_arguments(candidate_function)
    replacements = []
    for callee, base_occurrences in base_calls.items():
        candidate_occurrences = candidate_calls.get(callee, [])
        if len(base_occurrences) != len(candidate_occurrences):
            continue
        for base_args, candidate_args in zip(base_occurrences, candidate_occurrences, strict=True):
            if len(base_args) != len(candidate_args):
                continue
            for base_span, candidate_span in zip(base_args, candidate_args, strict=True):
                base_arg = base_function[slice(*base_span)]
                candidate_arg = candidate_function[slice(*candidate_span)]
                if (
                    base_arg != candidate_arg
                    and " as " in base_arg
                    and _without_casts(base_arg) == _without_casts(candidate_arg)
                ):
                    replacements.append((*candidate_span, base_arg))
    for start, end, replacement in reversed(replacements):
        candidate_function = candidate_function[:start] + replacement + candidate_function[end:]
    return candidate_function, len(replacements)


_LOCAL_RE = re.compile(
    r"\blet\s+(?P<mutable>mut\s+)?(?P<name>[A-Za-z_]\w*)\s*"
    r":\s*(?P<type>[^;=]+?)(?:\s*=\s*(?P<value>[^;]+))?\s*;"
)


def _restore_base_initializers(base_function: str, candidate_function: str) -> tuple[str, int]:
    base_locals = {match.group("name"): match for match in _LOCAL_RE.finditer(base_function)}
    replacements = []
    for match in _LOCAL_RE.finditer(candidate_function):
        base_match = base_locals.get(match.group("name"))
        if (
            base_match is None
            or match.group("value") is not None
            or base_match.group("value") is None
            or re.sub(r"\s", "", match.group("type")) != re.sub(r"\s", "", base_match.group("type"))
        ):
            continue
        replacement = match.group(0).rstrip()[:-1] + f" = {base_match.group('value').strip()};"
        replacements.append((match.start(), match.end(), replacement))
    for start, end, replacement in reversed(replacements):
        candidate_function = candidate_function[:start] + replacement + candidate_function[end:]
    return candidate_function, len(replacements)


def _repair_reassigned_locals(candidate_function: str) -> tuple[str, int]:
    masked = masked_rust(candidate_function)

    def assignment_paths(name: str) -> list[tuple[int, ...]]:
        positions = [match.start() for match in re.finditer(rf"\b{re.escape(name)}\s*=", masked)]
        paths = []
        stack: list[int] = []
        next_block = 0
        position_index = 0
        for pos, char in enumerate(masked):
            if char == "{":
                stack.append(next_block)
                next_block += 1
            elif char == "}" and stack:
                stack.pop()
            while position_index < len(positions) and positions[position_index] == pos:
                paths.append(tuple(stack))
                position_index += 1
        return paths

    def has_sequential_assignment(name: str) -> bool:
        paths = assignment_paths(name)
        return any(
            left == right[: len(left)] or right == left[: len(right)]
            for index, left in enumerate(paths)
            for right in paths[index + 1 :]
        )

    replacements = []
    for match in _LOCAL_RE.finditer(candidate_function):
        name = match.group("name")
        if match.group("mutable") or not has_sequential_assignment(name):
            continue
        insert = match.start("name")
        replacements.append((insert, insert, "mut "))
    for start, end, replacement in reversed(replacements):
        candidate_function = candidate_function[:start] + replacement + candidate_function[end:]
    return candidate_function, len(replacements)


def repair_worker(base: str, candidate: str, function: str) -> RepairResult:
    """Apply only repairs whose authority comes from the accepted base worker."""
    base_start, base_end = function_span(base, function)
    candidate_start, candidate_end = function_span(candidate, function)
    base_function = base[base_start:base_end]
    candidate_function = candidate[candidate_start:candidate_end]
    expected_signature = _signature(base_function)
    normalized = normalize_worker_body(candidate_function, expected_signature)
    applied = []
    if normalized != candidate_function:
        if re.search(r"\blet\s+[A-Za-z_]\w*\s*:\s*usize\s*=", candidate_function) and re.search(
            r"\bconst\s+[A-Za-z_]\w*\s*:\s*usize\s*=", normalized
        ):
            applied.append("const_context")
        if _signature(candidate_function) != _signature(normalized):
            applied.append("generated_signature")
        if not applied:
            applied.append("worker_normalization")
    normalized, restored_casts = _restore_base_argument_casts(base_function, normalized)
    if restored_casts:
        applied.append(f"base_argument_casts:{restored_casts}")
    normalized, restored_initializers = _restore_base_initializers(base_function, normalized)
    if restored_initializers:
        applied.append(f"base_initializers:{restored_initializers}")
    normalized, mutable_locals = _repair_reassigned_locals(normalized)
    if mutable_locals:
        applied.append(f"mutable_reassignments:{mutable_locals}")
    repaired = candidate[:candidate_start] + normalized + candidate[candidate_end:]
    return RepairResult(repaired, tuple(applied))
