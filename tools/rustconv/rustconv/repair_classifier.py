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


def _call_sites(function: str, callee: str) -> list[tuple[int, int, list[tuple[int, int]]]]:
    """Return call and argument spans for a named callee, including nested syntax."""
    masked = masked_rust(function)
    sites = []
    for match in re.finditer(rf"\b{re.escape(callee)}\s*\)?\s*\(", masked):
        opening = masked.rfind("(", match.start(), match.end())
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
        sites.append((match.start(), closing + 1, arguments))
    return sites


def _strip_void_pointer_cast(expression: str) -> str:
    return re.sub(r"\s+as\s+\*(?:mut|const)\s+(?:core::ffi::)?c_void\s*$", "", expression).strip()


def _generated_wide_units(rust: str) -> dict[str, tuple[int, ...]]:
    values = {}
    pattern = re.compile(
        r"(?m)^pub static (CPP_COM_WSTR_[0-9A-F]{12}):\s*\[WCHAR;\s*\d+\]\s*=\s*"
        r"\[([^\]]*)\];"
    )
    for match in pattern.finditer(rust):
        try:
            values[match.group(1)] = tuple(
                int(token.strip(), 0) for token in match.group(2).split(",") if token.strip()
            )
        except ValueError:
            continue
    return values


def _struct_initializer_fields(function: str) -> dict[tuple[str, str], dict[str, tuple[int, int]]]:
    """Return field-value spans for simple named local struct initializers."""
    masked = masked_rust(function)
    initializers = {}
    pattern = re.compile(r"\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\s*(?::[^=;]+)?=\s*([A-Za-z_]\w*)\s*\{")
    for match in pattern.finditer(masked):
        opening = masked.find("{", match.start(), match.end())
        depth = 0
        closing = None
        for pos in range(opening, len(masked)):
            if masked[pos] == "{":
                depth += 1
            elif masked[pos] == "}":
                depth -= 1
                if depth == 0:
                    closing = pos
                    break
        if closing is None:
            continue
        fields = {}
        start = opening + 1
        nested = 0
        for pos in range(start, closing + 1):
            char = masked[pos] if pos < closing else ","
            if char in "([{":
                nested += 1
            elif char in ")]}":
                nested -= 1
            elif char == "," and nested == 0:
                item = function[start:pos]
                field = re.match(r"\s*([A-Za-z_]\w*)\s*:\s*", item)
                if field:
                    value_start = start + field.end()
                    value_end = pos
                    while value_end > value_start and function[value_end - 1].isspace():
                        value_end -= 1
                    fields[field.group(1)] = (value_start, value_end)
                start = pos + 1
        initializers[(match.group(1), match.group(2))] = fields
    return initializers


def _restore_abi_struct_fields(base: str, candidate: str) -> tuple[str, int]:
    """Restore ABI field spellings and pointer nulls from an accepted initializer."""
    base_initializers = _struct_initializer_fields(base)
    candidate_initializers = _struct_initializer_fields(candidate)
    edits = []
    for key, candidate_fields in candidate_initializers.items():
        base_fields = base_initializers.get(key)
        if not base_fields:
            continue
        base_names = set(base_fields)
        for candidate_name, candidate_span in candidate_fields.items():
            base_name = candidate_name
            if base_name not in base_names:
                matches = sorted(
                    name
                    for name in base_names
                    if len(candidate_name) >= 4 and name.startswith(candidate_name)
                )
                if len(matches) != 1:
                    continue
                base_name = matches[0]
                # Replace the field name immediately preceding its value.
                colon = candidate.rfind(":", 0, candidate_span[0])
                name_match = re.search(r"([A-Za-z_]\w*)\s*$", candidate[:colon])
                if name_match:
                    edits.append((name_match.start(1), name_match.end(1), base_name))
            candidate_value = candidate[slice(*candidate_span)].strip()
            base_value = base[slice(*base_fields[base_name])].strip()
            if candidate_value == "0" and base_value in {
                "core::ptr::null()",
                "core::ptr::null_mut()",
            }:
                edits.append((*candidate_span, base_value))
    for start, end, replacement in reversed(edits):
        candidate = candidate[:start] + replacement + candidate[end:]
    return candidate, len(edits)


def _lower_known_c_idioms(
    base: str, candidate: str, fact_source: str | None = None
) -> tuple[str, list[str]]:
    """Lower only closed C idioms backed by symbols present in the accepted module."""
    applied = []
    replacements = 0

    candidate, count = re.subn(r"\bstd::", "core::", candidate)
    if count:
        applied.append(f"no_std_paths:{count}")

    aliases = {
        "InitFormatEtc": "init_format_etc",
        "strlenW": "wide_len",
        "wcslen": "wide_len",
    }
    for source, target in aliases.items():
        if re.search(rf"\bfn\s+{re.escape(target)}\b", base):
            candidate, count = re.subn(rf"\b{re.escape(source)}\b", target, candidate)
            if count:
                applied.append(f"callee_alias:{source}->{target}:{count}")

    if "crate::wine::shellole::SHAlloc" in base:
        candidate, count = re.subn(
            r"(?<![:\w])SHAlloc\s*\(", "crate::wine::shellole::SHAlloc(", candidate
        )
        if count:
            applied.append(f"callee_path:SHAlloc:{count}")

    candidate, field_repairs = _restore_abi_struct_fields(base, candidate)
    if field_repairs:
        applied.append(f"abi_struct_fields:{field_repairs}")

    # C pointer arithmetic commonly carries a signed INT result. Rust's `add`
    # requires usize; use `offset` only when the binding is proven to come from
    # wsprintfW (whose deterministic declaration returns INT).
    signed_offsets = set(re.findall(r"\blet\s+([A-Za-z_]\w*)\s*=\s*wsprintfW\s*\(", candidate))
    offset_edits = 0
    for name in signed_offsets:
        candidate, count = re.subn(
            rf"\.add\(\s*{re.escape(name)}\s*\)",
            f".offset({name} as isize)",
            candidate,
        )
        offset_edits += count
    if offset_edits:
        applied.append(f"signed_pointer_offsets:{offset_edits}")

    if (
        re.search(r"\blet\s+[A-Za-z_]\w*\s*:\s*u32\s*=\s*if\b", candidate)
        and "ERROR_SUCCESS as DWORD" in base
    ):
        candidate, count = re.subn(
            r"(?<![\w:])ERROR_SUCCESS(?!\s+as\b)", "ERROR_SUCCESS as u32", candidate
        )
        if count:
            applied.append(f"typed_error_success:{count}")

    # Bindgen represents STGMEDIUM's anonymous C union through `u`. Only touch
    # locals whose declared type proves that member path.
    medium_names = [
        match.group("name")
        for match in _LOCAL_RE.finditer(candidate)
        if re.sub(r"\s", "", match.group("type")) == "STGMEDIUM"
    ]
    for name in medium_names:
        if re.search(rf"\b{re.escape(name)}\.u\.hGlobal\b", base):
            candidate, count = re.subn(
                rf"\b{re.escape(name)}\.hGlobal\b", f"{name}.u.hGlobal", candidate
            )
            replacements += count
    if replacements:
        applied.append(f"anonymous_union_members:{replacements}")

    # IDataObject::GetData is ABI-owned: its two out parameters are opaque in
    # the generated COM vtable, while the worker owns concrete local mirrors.
    typed_locals = {
        match.group("name"): re.sub(r"\s", "", match.group("type"))
        for match in _LOCAL_RE.finditer(candidate)
    }
    edits = []
    for _, _, arguments in _call_sites(candidate, "GetData"):
        for index, expected in ((1, "FORMATETC"), (2, "STGMEDIUM")):
            if index >= len(arguments):
                continue
            start, end = arguments[index]
            expression = candidate[start:end]
            local = re.fullmatch(r"&mut\s+([A-Za-z_]\w*)", expression.strip())
            if local and typed_locals.get(local.group(1)) == expected:
                edits.append((start, end, f"(&mut {local.group(1)} as *mut {expected}).cast()"))
    for start, end, replacement in reversed(edits):
        candidate = candidate[:start] + replacement + candidate[end:]
    if edits:
        applied.append(f"com_out_pointer_casts:{len(edits)}")

    # `memcpy(dst, src, bytes)` has a mechanically identical Rust primitive.
    # Preserve its byte count exactly; do not infer an element type from sizeof.
    edits = []
    for start, end, arguments in _call_sites(candidate, "memcpy"):
        if len(arguments) != 3:
            continue
        dst = _strip_void_pointer_cast(candidate[slice(*arguments[0])])
        src = _strip_void_pointer_cast(candidate[slice(*arguments[1])])
        byte_count = candidate[slice(*arguments[2])].strip()
        if (
            dst != candidate[slice(*arguments[0])].strip()
            and src != candidate[slice(*arguments[1])].strip()
        ):
            edits.append(
                (
                    start,
                    end,
                    "core::ptr::copy_nonoverlapping("
                    f"({src}) as *const u8, ({dst}) as *mut u8, {byte_count})",
                )
            )
    for start, end, replacement in reversed(edits):
        candidate = candidate[:start] + replacement + candidate[end:]
    if edits:
        applied.append(f"typed_memcpy:{len(edits)}")

    # The two strsafe calls in this C idiom are append-with-optional-quotes.
    # Decode the fact-sealed UTF-16 format; do not infer from a model string.
    wide_units = _generated_wide_units(fact_source if fact_source is not None else candidate)
    edits = []
    if re.search(r"\bfn\s+append_argument\b", base):
        for start, end, arguments in _call_sites(candidate, "StringCchPrintfExW"):
            if len(arguments) != 7:
                continue
            flags = candidate[slice(*arguments[4])].strip()
            format_arg = candidate[slice(*arguments[5])].strip()
            format_match = re.fullmatch(r"(CPP_COM_WSTR_[0-9A-F]{12})\.as_ptr\(\)", format_arg)
            if flags != "0" or format_match is None:
                continue
            units = wide_units.get(format_match.group(1))
            if units is None:
                continue
            if units == (0x25, 0x6C, 0x73, 0x20, 0):
                quote = False
            elif units == (0x22, 0x25, 0x6C, 0x73, 0x22, 0x20, 0):
                quote = True
            else:
                continue
            end_arg = candidate[slice(*arguments[2])].strip()
            remaining_arg = candidate[slice(*arguments[3])].strip()
            source = candidate[slice(*arguments[6])].strip()
            if not end_arg.startswith("&mut ") or not remaining_arg.startswith("&mut "):
                continue
            edits.append(
                (
                    start,
                    end,
                    f"append_argument({end_arg}, {remaining_arg}, {source}, "
                    f"{'true' if quote else 'false'})",
                )
            )
    for start, end, replacement in reversed(edits):
        candidate = candidate[:start] + replacement + candidate[end:]
    if edits:
        applied.append(f"strsafe_append:{len(edits)}")

    return candidate, applied


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


def _restore_base_mutability(base_function: str, candidate_function: str) -> tuple[str, int]:
    base_locals = {match.group("name"): match for match in _LOCAL_RE.finditer(base_function)}
    insertions = []
    for match in _LOCAL_RE.finditer(candidate_function):
        base_match = base_locals.get(match.group("name"))
        if (
            base_match is not None
            and base_match.group("mutable")
            and not match.group("mutable")
            and re.sub(r"\s", "", match.group("type"))
            == re.sub(r"\s", "", base_match.group("type"))
        ):
            insertions.append(match.start("name"))
    for position in reversed(insertions):
        candidate_function = candidate_function[:position] + "mut " + candidate_function[position:]
    return candidate_function, len(insertions)


def _restore_base_argument_values(base_function: str, candidate_function: str) -> tuple[str, int]:
    base_calls = _call_arguments(base_function)
    candidate_calls = _call_arguments(candidate_function)
    replacements = []
    equivalents = {("TRUE", "true"), ("FALSE", "false")}
    for callee, base_occurrences in base_calls.items():
        candidate_occurrences = candidate_calls.get(callee, [])
        if len(base_occurrences) != len(candidate_occurrences):
            continue
        for base_args, candidate_args in zip(base_occurrences, candidate_occurrences, strict=True):
            if len(base_args) != len(candidate_args):
                continue
            for base_span, candidate_span in zip(base_args, candidate_args, strict=True):
                base_arg = base_function[slice(*base_span)].strip()
                candidate_arg = candidate_function[slice(*candidate_span)].strip()
                if (base_arg, candidate_arg) in equivalents:
                    replacements.append((*candidate_span, base_arg))
    for start, end, replacement in reversed(replacements):
        candidate_function = candidate_function[:start] + replacement + candidate_function[end:]
    return candidate_function, len(replacements)


def _restore_base_integer_predicates(
    base_function: str, candidate_function: str
) -> tuple[str, int]:
    replacements = []
    pattern = re.compile(r"\bif\s+([A-Z][A-Za-z0-9_]*\s*\([^{};]+\))\s*\{")
    compact_base = re.sub(r"\s+", "", base_function)
    for match in pattern.finditer(candidate_function):
        expression = match.group(1)
        compact_expression = re.sub(r"\s+", "", expression)
        if f"if{compact_expression}!=0{{" in compact_base:
            replacements.append((match.start(1), match.end(1), expression + " != 0"))
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
    normalized, idiom_repairs = _lower_known_c_idioms(base, normalized, candidate)
    applied.extend(idiom_repairs)
    normalized, restored_casts = _restore_base_argument_casts(base_function, normalized)
    if restored_casts:
        applied.append(f"base_argument_casts:{restored_casts}")
    normalized, restored_initializers = _restore_base_initializers(base_function, normalized)
    if restored_initializers:
        applied.append(f"base_initializers:{restored_initializers}")
    normalized, restored_mutability = _restore_base_mutability(base_function, normalized)
    if restored_mutability:
        applied.append(f"base_mutability:{restored_mutability}")
    normalized, restored_values = _restore_base_argument_values(base_function, normalized)
    if restored_values:
        applied.append(f"base_argument_values:{restored_values}")
    normalized, restored_predicates = _restore_base_integer_predicates(base_function, normalized)
    if restored_predicates:
        applied.append(f"base_integer_predicates:{restored_predicates}")
    normalized, mutable_locals = _repair_reassigned_locals(normalized)
    if mutable_locals:
        applied.append(f"mutable_reassignments:{mutable_locals}")
    repaired = candidate[:candidate_start] + normalized + candidate[candidate_end:]
    return RepairResult(repaired, tuple(applied))
