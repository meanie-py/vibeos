"""Generate and execute bounded, explicit-contract C/Rust differential harnesses.

Host-only stateless buffer/string contracts and stateful scalar-operation sequences.
No inferred pointer ownership, no guessed stubs, no paid calls. A contract is a
reviewed claim about valid inputs, not something compilation proves.
"""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import itertools
import json
import mmap
import random
import re
import shutil
import signal
import subprocess
import sys
import time
from collections.abc import Iterator
from pathlib import Path
from typing import Any


class ContractError(ValueError):
    """The requested grade cannot be established."""


# C spelling, ctypes representation, minimum, maximum. Fixed-width types deliberately
# exclude C long/wchar_t: callers must resolve those against their target first.
TYPES = {
    name: (c, ct, low, high)
    for name, c, ct, low, high in (
        ("u8", "uint8_t", ctypes.c_uint8, 0, 2**8 - 1),
        ("i8", "int8_t", ctypes.c_int8, -(2**7), 2**7 - 1),
        ("u16", "uint16_t", ctypes.c_uint16, 0, 2**16 - 1),
        ("i16", "int16_t", ctypes.c_int16, -(2**15), 2**15 - 1),
        ("u32", "uint32_t", ctypes.c_uint32, 0, 2**32 - 1),
        ("i32", "int32_t", ctypes.c_int32, -(2**31), 2**31 - 1),
        ("u64", "uint64_t", ctypes.c_uint64, 0, 2**64 - 1),
        ("i64", "int64_t", ctypes.c_int64, -(2**63), 2**63 - 1),
        ("usize", "size_t", ctypes.c_size_t, 0, 2 ** (8 * ctypes.sizeof(ctypes.c_size_t)) - 1),
        (
            "isize",
            "intptr_t",
            ctypes.c_ssize_t,
            -(2 ** (8 * ctypes.sizeof(ctypes.c_ssize_t) - 1)),
            2 ** (8 * ctypes.sizeof(ctypes.c_ssize_t) - 1) - 1,
        ),
    )
}
POINTERS = {
    "bytes": ("uint8_t *", "*mut u8"),
    "const_bytes": ("const uint8_t *", "*const u8"),
    "void_bytes": ("void *", "*mut core::ffi::c_void"),
    "const_void_bytes": ("const void *", "*const core::ffi::c_void"),
    "const_cstr": ("const char *", "*const u8"),
    "mut_cstr": ("char *", "*mut u8"),
    "const_wstr": ("const uint16_t *", "*const u16"),
    "mut_wstr": ("uint16_t *", "*mut u16"),
}
OUT_POINTERS = {"cstr_out": ("char **", "*mut *mut u8")}
STRING_POINTERS = {"const_cstr", "mut_cstr", "const_wstr", "mut_wstr"}
READONLY = {"const_bytes", "const_void_bytes", "const_cstr", "const_wstr"}
GUARD = 32
SENTINEL = 0xA5
MAX_STATEFUL_CASES = 512
MAX_SEQUENCE_STEPS = 64
MAX_FIXTURE_CALLS = 128
MAX_FIXTURE_EVENTS = 256
MAX_FIXTURE_ARGS = 8
MAX_RECORD_DEPTH = 4
MAX_RECORD_FIELDS = 256
MAX_NESTED_RECORD_COUNT = 64
MAX_POINTEE_RECORDS = 16
IDENT = re.compile(r"[A-Za-z_][A-Za-z_0-9]*\Z")
RUST_PATH = re.compile(r"[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*\Z")
FIELD_PATH = re.compile(r"[A-Za-z_][A-Za-z_0-9]*(?:\.[A-Za-z_][A-Za-z_0-9]*)*\Z")
C_POINTER_TYPE = re.compile(r"(?:const )?[A-Za-z_][A-Za-z_0-9]*(?: \*)*\Z")
RUST_RAW_POINTER = re.compile(
    r"\*(?:const|mut) (?:core::ffi::c_void|[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*)\Z"
)
RUST_POINTER_TYPES = {
    "*const u8",
    "*mut u8",
    "*const i8",
    "*mut i8",
    "*const u16",
    "*mut u16",
    "*const core::ffi::c_char",
    "*mut core::ffi::c_char",
    "*const core::ffi::c_void",
    "*mut core::ffi::c_void",
}
C_RETURN_TYPES = {
    "signed char",
    "unsigned char",
    "short",
    "unsigned short",
    "int",
    "unsigned int",
    "long",
    "unsigned long",
    "long long",
    "unsigned long long",
    "int8_t",
    "uint8_t",
    "int16_t",
    "uint16_t",
    "int32_t",
    "uint32_t",
    "int64_t",
    "uint64_t",
    "intptr_t",
    "uintptr_t",
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _keys(obj: dict, required: set[str], optional: set[str], label: str) -> None:
    if not isinstance(obj, dict) or required - obj.keys() or obj.keys() - required - optional:
        raise ContractError(
            f"{label}: required keys {sorted(required)}, optional {sorted(optional)}"
        )


def _int(value: Any, low: int, high: int, label: str) -> int:
    if type(value) is not int or not low <= value <= high:
        raise ContractError(f"{label}: expected integer in [{low}, {high}]")
    return value


def _text_units(value: str, encoding: str, label: str) -> int:
    if not isinstance(value, str):
        raise ContractError(f"{label} must be text")
    try:
        if encoding == "ascii":
            return len(value.encode("ascii"))
        return len(value.encode("utf-16-le")) // 2
    except UnicodeEncodeError as exc:
        expected = "ASCII" if encoding == "ascii" else "valid Unicode scalars"
        raise ContractError(f"{label} must contain {expected}") from exc


def _encode_string_input(value: str, kind: str) -> bytes:
    if kind in ("const_cstr", "mut_cstr"):
        return value.encode("ascii") + b"\0"
    if kind in ("const_wstr", "mut_wstr"):
        return value.encode("utf-16-le") + b"\0\0"
    raise ContractError(f"unsupported generated string pointer: {kind}")


def validate_string_generator(spec: Any, label: str, encoding: str = "ascii") -> None:
    """Validate a small finite grammar AST and its explicit edge examples."""
    _keys(spec, {"max_bytes", "examples", "grammar"}, {"encoding"}, f"{label} generator")
    if spec.get("encoding", "ascii") != encoding or encoding not in ("ascii", "utf16le"):
        raise ContractError(f"{label} generator encoding must be {encoding}")
    maximum = _int(spec["max_bytes"], 1, 65536, f"{label} max_bytes")
    unit_bytes = 1 if encoding == "ascii" else 2
    examples = spec["examples"]
    if not isinstance(examples, list) or not examples or len(examples) > 64:
        raise ContractError(f"{label} examples must contain 1..64 strings")
    for example in examples:
        units = _text_units(example, encoding, f"{label} example")
        if (units + 1) * unit_bytes > maximum:
            raise ContractError(f"{label} example plus terminator exceeds max_bytes")

    budget = [0]

    def visit(node: Any, depth: int) -> int:
        budget[0] += 1
        if budget[0] > 512 or depth > 16:
            raise ContractError(f"{label} grammar exceeds node/depth limits")
        operators = {"literal", "charset", "sequence", "choice", "repeat"}
        if not isinstance(node, dict) or len(operators & node.keys()) != 1:
            raise ContractError(f"{label} grammar nodes must have exactly one operator")
        if "literal" in node:
            _keys(node, {"literal"}, set(), f"{label} literal")
            value = node["literal"]
            return _text_units(value, encoding, f"{label} literal")
        if "charset" in node:
            _keys(node, {"charset"}, {"min", "max"}, f"{label} charset")
            chars = node["charset"]
            if not isinstance(chars, str) or not chars:
                raise ContractError(f"{label} charset must be nonempty text")
            if len(set(chars)) != len(chars):
                raise ContractError(f"{label} charset must not contain duplicate characters")
            unit_max = max(_text_units(char, encoding, f"{label} charset") for char in chars)
            low = _int(node.get("min", 0), 0, 64, f"{label} charset min")
            high = _int(node.get("max", low), low, 64, f"{label} charset max")
            return high * unit_max
        if "sequence" in node:
            _keys(node, {"sequence"}, set(), f"{label} sequence")
            parts = node["sequence"]
            if not isinstance(parts, list) or len(parts) > 64:
                raise ContractError(f"{label} sequence must contain at most 64 nodes")
            return sum(visit(part, depth + 1) for part in parts)
        if "choice" in node:
            _keys(node, {"choice"}, set(), f"{label} choice")
            options = node["choice"]
            if not isinstance(options, list) or not options or len(options) > 32:
                raise ContractError(f"{label} choice must contain 1..32 nodes")
            return max(visit(option, depth + 1) for option in options)
        if "repeat" in node:
            _keys(node, {"repeat"}, {"min", "max"}, f"{label} repeat")
            low = _int(node.get("min", 0), 0, 64, f"{label} repeat min")
            high = _int(node.get("max", low), low, 64, f"{label} repeat max")
            return high * visit(node["repeat"], depth + 1)
        raise ContractError(
            f"{label} grammar operator must be literal, charset, sequence, choice or repeat"
        )

    generated_max = visit(spec["grammar"], 1)
    if (generated_max + 1) * unit_bytes > maximum:
        raise ContractError(f"{label} grammar maximum plus terminator exceeds max_bytes")


def validate_record(
    spec: Any,
    label: str,
    depth: int = 0,
    field_budget: list[int] | None = None,
    allow_flexible_tail: bool = True,
    allow_pointees: bool = True,
) -> None:
    """Validate a C-layout-checked record with bounded inline and hidden records."""
    if depth > MAX_RECORD_DEPTH:
        raise ContractError(f"{label} record nesting exceeds {MAX_RECORD_DEPTH}")
    if field_budget is None:
        field_budget = [0]
    _keys(
        spec,
        {"layout", "base_size", "fields"},
        {"pointees"} if allow_pointees else set(),
        f"{label} record",
    )
    if not isinstance(spec["layout"], str) or not IDENT.fullmatch(spec["layout"]):
        raise ContractError(f"{label} record layout must be a C type identifier")
    base_size = _int(spec["base_size"], 1, 65536, f"{label} record base_size")
    fields = spec["fields"]
    if not isinstance(fields, list) or not fields or len(fields) > 64:
        raise ContractError(f"{label} record fields must contain 1..64 fields")
    field_budget[0] += len(fields)
    if field_budget[0] > MAX_RECORD_FIELDS:
        raise ContractError(f"{label} record tree exceeds {MAX_RECORD_FIELDS} fields")

    seen: dict[str, dict] = {}
    scalar_ranges: list[tuple[int, int, str]] = []
    flexible_fields: list[dict] = []
    for index, field in enumerate(fields):
        _keys(
            field,
            {"name", "offset"},
            {
                "type",
                "values",
                "points_to",
                "nullable",
                "one_past",
                "record",
                "count",
                "count_from",
                "min_count",
            },
            f"{label} record field",
        )
        name = field["name"]
        if not isinstance(name, str) or not FIELD_PATH.fullmatch(name) or name in seen:
            raise ContractError(f"{label} record field has an invalid/duplicate name")
        offset = _int(field["offset"], 0, 65535, f"{label}.{name} offset")
        if "record" in field:
            if any(
                key in field
                for key in (
                    "type",
                    "values",
                    "points_to",
                    "nullable",
                    "one_past",
                    "count_from",
                    "min_count",
                )
            ):
                raise ContractError(f"{label}.{name}: nested record cannot have scalar options")
            count = _int(
                field.get("count", 1),
                1,
                MAX_NESTED_RECORD_COUNT,
                f"{label}.{name} nested record count",
            )
            nested_field_budget = [0]
            validate_record(
                field["record"],
                f"{label}.{name}",
                depth + 1,
                nested_field_budget,
                allow_flexible_tail=False,
                allow_pointees=False,
            )
            field_budget[0] += nested_field_budget[0] * count
            if field_budget[0] > MAX_RECORD_FIELDS:
                raise ContractError(f"{label} record tree exceeds {MAX_RECORD_FIELDS} fields")
            scalar_ranges.append((offset, offset + field["record"]["base_size"] * count, name))
            seen[name] = {"record": field["record"]}
            if "count" in field:
                field["count"] = count
            continue
        if "points_to" in field:
            target = field["points_to"]
            if isinstance(target, dict):
                _keys(target, {"pointee"}, set(), f"{label}.{name} points_to")
                if not isinstance(target["pointee"], str) or not IDENT.fullmatch(target["pointee"]):
                    raise ContractError(f"{label}.{name}: points_to pointee must be an identifier")
            elif not isinstance(target, str) or not IDENT.fullmatch(target):
                raise ContractError(
                    f"{label}.{name}: points_to must name a pointer argument or pointee"
                )
            if any(key in field for key in ("type", "values", "count", "count_from", "min_count")):
                raise ContractError(f"{label}.{name}: pointer fields cannot declare scalar values")
            for flag in ("nullable", "one_past"):
                if type(field.setdefault(flag, False)) is not bool:
                    raise ContractError(f"{label}.{name}: {flag} must be boolean")
            scalar_ranges.append((offset, offset + ctypes.sizeof(ctypes.c_void_p), name))
            seen[name] = field
            continue
        if "type" not in field or "values" not in field:
            raise ContractError(f"{label}.{name}: scalar fields require type and values")
        if field["type"] not in TYPES or field["type"] in ("usize", "isize"):
            raise ContractError(f"{label}.{name}: record fields require fixed-width integer types")
        width = ctypes.sizeof(TYPES[field["type"]][1])
        values = field["values"]
        low, high = TYPES[field["type"]][2:]
        if not isinstance(values, list) or not values or len(values) > 64:
            raise ContractError(f"{label}.{name} values must contain 1..64 integers")
        for value in values:
            _int(value, low, high, f"{label}.{name} value")
        has_count, has_count_from = "count" in field, "count_from" in field
        if has_count == has_count_from:
            raise ContractError(f"{label}.{name} needs exactly one of count or count_from")
        if has_count:
            count = _int(field["count"], 1, 65536, f"{label}.{name} count")
            if "min_count" in field:
                raise ContractError(f"{label}.{name}: min_count applies only to count_from arrays")
            if count == 1:
                scalar_ranges.append((offset, offset + width, name))
            else:
                scalar_ranges.append((offset, offset + width * count, name))
        else:
            if not allow_flexible_tail:
                raise ContractError(f"{label}.{name}: nested records cannot have flexible tails")
            if index != len(fields) - 1:
                raise ContractError(f"{label}.{name}: a count_from array must be the final field")
            count_name = field["count_from"]
            if not isinstance(count_name, str) or count_name not in seen:
                raise ContractError(f"{label}.{name}: count_from must name an earlier scalar field")
            count_field = seen[count_name]
            if (
                "type" not in count_field
                or count_field.get("count", 1) != 1
                or "count_from" in count_field
            ):
                raise ContractError(f"{label}.{name}: count_from must name a scalar field")
            count_low = TYPES[count_field["type"]][2]
            if count_low < 0:
                raise ContractError(f"{label}.{name}: count_from field must be unsigned")
            max_count = max(count_field["values"])
            min_count = _int(field.get("min_count", 0), 0, 65536, f"{label}.{name} min_count")
            if max_count > 65536 or min_count > 65536:
                raise ContractError(f"{label}.{name}: generated array exceeds 65536 elements")
            field["min_count"] = min_count
            flexible_fields.append(field)
            scalar_ranges.append((offset, offset + width * min_count, name))
        seen[name] = field

    ordered = sorted(scalar_ranges)
    for left, right in itertools.pairwise(ordered):
        if right[0] < left[1]:
            raise ContractError(f"{label} record fields overlap: {left[2]} and {right[2]}")
    if any(end > base_size for _, end, _ in scalar_ranges):
        raise ContractError(f"{label} record base_size does not contain its declared fields")
    if flexible_fields:
        tail = flexible_fields[0]
        count_field = seen[tail["count_from"]]
        count_max = max(count_field["values"])
        width = ctypes.sizeof(TYPES[tail["type"]][1])
        if tail["offset"] + width * max(count_max, tail["min_count"]) > 65536:
            raise ContractError(f"{label} record generated size exceeds 65536 bytes")
    if "pointees" in spec:
        pointees = spec["pointees"]
        if not isinstance(pointees, list) or not pointees or len(pointees) > MAX_POINTEE_RECORDS:
            raise ContractError(f"{label} pointees must contain 1..{MAX_POINTEE_RECORDS} records")
        names = set()
        for pointee in pointees:
            _keys(pointee, {"name"}, {"record", "bytes", "mutable"}, f"{label} pointee")
            name = pointee["name"]
            if not isinstance(name, str) or not IDENT.fullmatch(name) or name in names:
                raise ContractError(f"{label} pointee name is invalid or duplicated")
            if type(pointee.setdefault("mutable", False)) is not bool:
                raise ContractError(f"{label}.pointees.{name} mutable must be boolean")
            names.add(name)
            if ("record" in pointee) == ("bytes" in pointee):
                raise ContractError(f"{label}.pointees.{name} needs exactly one of record/bytes")
            if "bytes" in pointee:
                validate_byte_pointee(pointee["bytes"], spec, f"{label}.pointees.{name}")
                continue
            validate_record(
                pointee["record"],
                f"{label}.pointees.{name}",
                field_budget=field_budget,
                allow_flexible_tail=False,
                allow_pointees=False,
            )


def validate_byte_pointee(spec: Any, root: dict, label: str) -> None:
    """A hidden byte buffer a record points at, e.g. UNICODE_STRING.Buffer.

    Its length is fixed or read from a root scalar field (`"length": "Length"`), so the
    allocation always ends exactly where the record says the data ends: a read past it
    hits the guard page. `c_type` names the element type the C field points to.
    """
    _keys(spec, {"length"}, {"c_type", "align"}, f"{label} bytes")
    if not isinstance(spec.setdefault("c_type", "unsigned char"), str) or not re.fullmatch(
        r"(?:unsigned )?[A-Za-z_][A-Za-z_0-9]*", spec["c_type"]
    ):
        raise ContractError(f"{label}: bytes c_type must name a C element type")
    if spec.setdefault("align", 1) not in (1, 2, 4, 8, 16) or type(spec["align"]) is not int:
        raise ContractError(f"{label}: bytes align must be 1, 2, 4, 8 or 16")
    length = spec["length"]
    if isinstance(length, str):
        field = next((f for f in root["fields"] if f["name"] == length), None)
        if (
            field is None
            or field.get("count") != 1
            or field.get("type") not in ("u8", "u16", "u32")
        ):
            raise ContractError(
                f"{label}: bytes length must name a root u8/u16/u32 field with count 1"
            )
        if max(field["values"]) > 65536:
            raise ContractError(f"{label}: bytes length values exceed 65536")
    else:
        _int(length, 0, 65536, f"{label} bytes length")


def iter_record_leaves(
    spec: dict, prefix: str = "", base_offset: int = 0
) -> Iterator[tuple[str, int, dict]]:
    """Yield leaf member paths, absolute offsets and validated field descriptions."""
    for field in spec["fields"]:
        path = f"{prefix}.{field['name']}" if prefix else field["name"]
        offset = base_offset + field["offset"]
        if "record" in field:
            is_array = "count" in field
            count = field.get("count", 1)
            stride = field["record"]["base_size"]
            for index in range(count):
                element_path = f"{path}[{index}]" if is_array else path
                yield from iter_record_leaves(
                    field["record"], element_path, offset + index * stride
                )
        else:
            yield path, offset, field


def record_alignment(spec: dict) -> int:
    """Upper bound on the C alignment of a record: its widest scalar or pointer leaf.

    `#pragma pack` can only make the real alignment smaller, and over-aligning costs at
    most a few canary bytes before the right guard page.
    """
    return max(
        ctypes.sizeof(ctypes.c_void_p)
        if "points_to" in field
        else ctypes.sizeof(TYPES[field["type"]][1])
        for _, _, field in iter_record_leaves(spec)
    )


def iter_record_instances(
    arg: dict,
) -> Iterator[tuple[dict, str, dict, str, str, str | None]]:
    """Yield a root record and its separately allocated, named pointee records."""
    yield arg, arg["name"], arg["record"], "", "", None
    for pointee in arg["record"].get("pointees", []):
        if "record" not in pointee:
            continue
        name = pointee["name"]
        binding = f"@{arg['name']}/pointee/{name}"
        prefix = f"pointees.{name}."
        yield arg, binding, pointee["record"], prefix, prefix, name


def record_instance_values(
    arg_name: str, value_prefix: str, record_spec: dict, values: dict
) -> dict[str, int]:
    """Select the sampled scalar values belonging to one record allocation."""
    return {
        path: values[(arg_name, f"{value_prefix}{path}")]
        for path, _, field in iter_record_leaves(record_spec)
        if "points_to" not in field and "count" in field and field["count"] == 1
    }


def generate_string(node: dict, rng: random.Random) -> str:
    """Sample one string from a grammar already checked by validate_string_generator."""
    if "literal" in node:
        return node["literal"]
    if "charset" in node:
        low, high = node.get("min", 0), node.get("max", node.get("min", 0))
        return "".join(rng.choice(node["charset"]) for _ in range(rng.randint(low, high)))
    if "sequence" in node:
        return "".join(generate_string(part, rng) for part in node["sequence"])
    if "choice" in node:
        return generate_string(rng.choice(node["choice"]), rng)
    if "repeat" in node:
        low, high = node.get("min", 0), node.get("max", node.get("min", 0))
        return "".join(generate_string(node["repeat"], rng) for _ in range(rng.randint(low, high)))
    raise ContractError("unvalidated string grammar reached generator")


def load_contract(path: Path) -> dict:
    data = json.loads(path.read_text())
    _keys(
        data,
        {"version", "scope", "validity", "c_source", "rust_source", "functions", "mutants"},
        {
            "c_flags",
            "rust_flags",
            "rust_support",
            "c_prelude",
            "seed",
            "random_cases",
            "rust_items",
            "c_include_dirs",
            "memory",
            "rust_helpers",
            "rust_imports",
            "sequences",
            "fixtures",
            "c_allow_ub",
        },
        "contract",
    )
    if data["version"] != 1 or data["scope"] not in ("stateless-host", "stateful-host"):
        raise ContractError("only version 1 stateless-host or stateful-host is supported")
    if data["scope"] == "stateless-host" and ("sequences" in data or "fixtures" in data):
        raise ContractError("sequences and fixtures require scope=stateful-host")
    if not isinstance(data["validity"], str) or not data["validity"].strip():
        raise ContractError("validity must explain the reviewed input/environment assumptions")
    for key in ("c_source", "rust_source", "c_prelude"):
        if key in data:
            data[key] = str((path.parent / data[key]).resolve(strict=True))
    if data.setdefault("memory", "canary") not in ("canary", "guard-pages"):
        raise ContractError("memory must be canary or guard-pages")
    if data["scope"] == "stateful-host" and data["memory"] != "canary":
        raise ContractError("stateful-host v1 has no pointer memory model")
    helpers = data.setdefault("rust_helpers", [])
    if not isinstance(helpers, list) or len(helpers) > 16:
        raise ContractError("rust_helpers must be a list of up to 16 source/item selections")
    for helper in helpers:
        _keys(helper, {"source", "items"}, set(), "Rust helper")
        helper["source"] = str((path.parent / helper["source"]).resolve(strict=True))
        validate_items(helper["items"])
    if helpers and "rust_items" not in data:
        raise ContractError("rust_helpers requires explicit rust_items selection")
    support = data.setdefault("rust_support", {})
    if not isinstance(support, dict):
        raise ContractError("rust_support must map module names to source paths")
    for name, value in support.items():
        if not IDENT.fullmatch(name) or name == "candidate":
            raise ContractError(f"invalid support module: {name}")
        support[name] = str((path.parent / value).resolve(strict=True))
    data["c_include_dirs"] = [
        str((path.parent / p).resolve(strict=True)) for p in data.get("c_include_dirs", [])
    ]
    items = data.get("rust_items")
    if items is not None:
        validate_items(items)
    imports = data.setdefault("rust_imports", [])
    if (
        not isinstance(imports, list)
        or len(imports) > 64
        or any(not isinstance(item, str) or not RUST_PATH.fullmatch(item) for item in imports)
        or len(set(imports)) != len(imports)
    ):
        raise ContractError("rust_imports must be up to 64 distinct exact Rust paths")
    if imports and items is None:
        raise ContractError("rust_imports requires explicit rust_items selection")
    for flag_key in ("c_flags", "rust_flags"):
        flags = data.setdefault(flag_key, [])
        if not isinstance(flags, list) or any(not isinstance(f, str) for f in flags):
            raise ContractError(f"{flag_key} must be an argv array")
        # Keep the compiler in the supported host mode with signature errors enabled.
        allowed = ("-D", "-U") if flag_key == "c_flags" else ("--cfg=",)
        exact = (
            {"-fno-builtin", "-fsigned-char", "-funsigned-char"} if flag_key == "c_flags" else set()
        )
        if any(not f.startswith(allowed) and f not in exact for f in flags):
            raise ContractError(
                f"{flag_key}: unsupported flag; use explicit include dirs and defines"
            )
    # Windows C routinely reads a WORD/ULONG through a byte pointer at any offset. That is
    # UB, and UBSan's alignment check kills the reference on every such input, so the
    # source could never be graded. Waiving it is a declared, reported property of the
    # contract — the only waivable check — never a default. The validity text must say why.
    allow_ub = data.setdefault("c_allow_ub", [])
    if not isinstance(allow_ub, list) or any(u != "alignment" for u in allow_ub):
        raise ContractError('c_allow_ub may only list "alignment"')
    if allow_ub and "align" not in data["validity"].lower():
        raise ContractError("c_allow_ub requires the validity text to justify the alignment waiver")
    _int(data.setdefault("seed", 0), 0, 2**32 - 1, "seed")
    _int(data.setdefault("random_cases", 128), 0, 10000, "random_cases")
    funcs = data["functions"]
    if not isinstance(funcs, list) or not funcs:
        raise ContractError("functions must be nonempty")
    seen = set()
    for fn in funcs:
        _keys(
            fn,
            {"c", "rust", "return", "args"},
            {"rust_abi", "rust_unsafe", "layouts", "c_return_type"},
            "function",
        )
        if not IDENT.fullmatch(fn["c"]) or not RUST_PATH.fullmatch(fn["rust"]):
            raise ContractError("invalid function identifier")
        if fn["c"] in seen:
            raise ContractError("duplicate C function")
        seen.add(fn["c"])
        result = fn["return"]
        if data["scope"] == "stateful-host" and (
            not isinstance(result, str) or (result not in TYPES and result != "void")
        ):
            raise ContractError("stateful-host functions require scalar or void results")
        if isinstance(result, dict):
            if result.get("type") == "owned_cstr":
                _keys(
                    result,
                    {"type", "nullable", "max_bytes"},
                    {"c_type", "rust_type"},
                    "owned C-string result",
                )
                if type(result["nullable"]) is not bool:
                    raise ContractError("owned C-string nullable must be boolean")
                _int(result["max_bytes"], 1, 65536, "owned C-string max_bytes")
                if result.get("c_type", "char *") != "char *":
                    raise ContractError("owned_cstr requires C return type char *")
                if (
                    not isinstance(result.get("rust_type", "*mut u8"), str)
                    or result.get("rust_type", "*mut u8") not in RUST_POINTER_TYPES
                ):
                    raise ContractError("owned_cstr requires a supported Rust pointer type")
            elif result.get("type") == "owned_bytes":
                _keys(
                    result,
                    {"type", "allocator", "nullable", "length"},
                    {
                        "c_type",
                        "rust_type",
                        "c_free",
                        "rust_free",
                        "free_abi",
                        "free_args",
                    },
                    "owned byte result",
                )
                if result["allocator"] not in ("libc", "custom"):
                    raise ContractError("owned_bytes allocator must be libc or custom")
                if type(result["nullable"]) is not bool:
                    raise ContractError("owned byte nullable must be boolean")
                c_type = result.get("c_type", "void *")
                if not isinstance(c_type, str) or not C_POINTER_TYPE.fullmatch(c_type):
                    raise ContractError("owned_bytes requires a simple C pointer return type")
                rust_type = result.get("rust_type", "*mut u8")
                if not isinstance(rust_type, str) or (
                    rust_type not in RUST_POINTER_TYPES
                    and not RUST_RAW_POINTER.fullmatch(rust_type)
                ):
                    raise ContractError("owned_bytes requires a supported Rust pointer type")
                if not rust_type.startswith("*mut "):
                    raise ContractError("owned_bytes results must use a mutable Rust pointer")
                if result["allocator"] == "libc":
                    if any(
                        key in result for key in ("c_free", "rust_free", "free_abi", "free_args")
                    ):
                        raise ContractError(
                            "allocator=libc uses host free; custom deallocator fields are invalid"
                        )
                else:
                    if not isinstance(result.get("c_free"), str) or not IDENT.fullmatch(
                        result["c_free"]
                    ):
                        raise ContractError("allocator=custom requires a C c_free function name")
                    if not isinstance(result.get("rust_free"), str) or not RUST_PATH.fullmatch(
                        result["rust_free"]
                    ):
                        raise ContractError(
                            "allocator=custom requires an exact Rust rust_free path"
                        )
                    if result.setdefault("free_abi", "C") not in ("C", "system", "Rust"):
                        raise ContractError("owned byte free_abi must be C, system or Rust")
            else:
                _keys(
                    result,
                    {"type", "within", "nullable", "one_past"},
                    {"c_type", "rust_type"},
                    "pointer result",
                )
                if result["type"] not in POINTERS:
                    raise ContractError("unsupported pointer return type")
                if type(result["nullable"]) is not bool or type(result["one_past"]) is not bool:
                    raise ContractError("nullable and one_past must be boolean")
                if not isinstance(result["within"], list) or not result["within"]:
                    raise ContractError("pointer return requires explicit within argument names")
                validate_c_pointer(result)
                if (
                    not isinstance(result.get("rust_type", POINTERS[result["type"]][1]), str)
                    or result.get("rust_type", POINTERS[result["type"]][1])
                    not in RUST_POINTER_TYPES
                ):
                    raise ContractError("pointer result has unsupported Rust pointer type")
        elif result not in TYPES and result != "void":
            raise ContractError("unsupported result type")
        if "c_return_type" in fn and (
            not isinstance(fn["c_return_type"], str)
            or fn["c_return_type"] not in C_RETURN_TYPES
            or isinstance(result, dict)
            or result == "void"
        ):
            raise ContractError("c_return_type must be a supported scalar C type override")
        if fn.setdefault("rust_abi", "C") not in ("C", "system", "Rust"):
            raise ContractError("unsupported Rust ABI")
        if type(fn.setdefault("rust_unsafe", True)) is not bool:
            raise ContractError("rust_unsafe must be boolean")
        if not isinstance(fn["args"], list):
            raise ContractError("args must be an array")
        names = set()
        scalars = {}
        for arg in fn["args"]:
            _keys(
                arg,
                {"name", "type"},
                {
                    "values",
                    "length",
                    "random_range",
                    "c_type",
                    "rust_type",
                    "generator",
                    "record",
                    "nullable",
                    "within",
                    "one_past",
                    "align",
                    "examples",
                },
                "argument",
            )
            if not IDENT.fullmatch(arg["name"]) or arg["name"] in names:
                raise ContractError("invalid/duplicate argument name")
            names.add(arg["name"])
            kind = arg["type"]
            if data["scope"] == "stateful-host" and (
                not isinstance(kind, str) or kind not in TYPES
            ):
                raise ContractError("stateful-host v1 supports scalar arguments only")
            validate_c_pointer(arg)
            if kind in TYPES:
                if any(key in arg for key in ("generator", "nullable", "within", "one_past")):
                    raise ContractError("scalar argument has pointer-only fields")
                _, _, low, high = TYPES[kind]
                values = arg.get("values")
                if not isinstance(values, list) or not values or "length" in arg:
                    raise ContractError("scalar requires nonempty values and no length")
                for value in values:
                    _int(value, low, high, arg["name"])
                if "random_range" in arg:
                    bounds = arg["random_range"]
                    if not isinstance(bounds, list) or len(bounds) != 2:
                        raise ContractError("random_range must be [min, max]")
                    a, b = [_int(v, low, high, arg["name"]) for v in bounds]
                    if a > b:
                        raise ContractError("inverted random_range")
                scalars[arg["name"]] = arg
            elif kind in POINTERS:
                record = arg.get("record")
                if record is not None:
                    if kind in STRING_POINTERS or "length" in arg or "generator" in arg:
                        raise ContractError(
                            "record pointers cannot have length, string or generator fields"
                        )
                    if not isinstance(arg.get("c_type"), str) or not C_POINTER_TYPE.fullmatch(
                        arg["c_type"]
                    ):
                        raise ContractError(
                            "record pointers require a simple, explicit C pointer type"
                        )
                    validate_record(record, arg["name"])
                elif "record" in arg:
                    raise ContractError("record must be an object")
                if "nullable" in arg and type(arg["nullable"]) is not bool:
                    raise ContractError("pointer nullable must be boolean")
                if "within" in arg or "one_past" in arg:
                    raise ContractError("within and one_past apply only to cstr_out")
                if kind in STRING_POINTERS:
                    if "generator" not in arg or any(
                        key in arg for key in ("length", "values", "random_range")
                    ):
                        raise ContractError(
                            "string pointer requires generator and no length/values"
                        )
                    encoding = "utf16le" if kind in ("const_wstr", "mut_wstr") else "ascii"
                    validate_string_generator(arg["generator"], arg["name"], encoding)
                elif (
                    ("length" not in arg and record is None)
                    or ("length" in arg and record is not None)
                    or "values" in arg
                    or "random_range" in arg
                    or "generator" in arg
                ):
                    raise ContractError("byte pointer requires length or a structured record")
                if "examples" in arg:
                    validate_byte_examples(arg, fn["args"])
            elif kind in OUT_POINTERS:
                if any(
                    key in arg
                    for key in ("values", "length", "random_range", "generator", "c_type")
                ):
                    raise ContractError("cstr_out does not accept input-buffer fields")
                if type(arg.get("nullable")) is not bool:
                    raise ContractError("cstr_out requires boolean nullable for the pointer slot")
                targets = arg.get("within")
                if (
                    not isinstance(targets, list)
                    or not targets
                    or any(
                        not isinstance(name, str) or not IDENT.fullmatch(name) for name in targets
                    )
                    or len(set(targets)) != len(targets)
                ):
                    raise ContractError("cstr_out requires distinct within input names")
                if type(arg.setdefault("one_past", False)) is not bool:
                    raise ContractError("cstr_out one_past must be boolean")
            else:
                raise ContractError(f"unsupported argument type: {kind}")
        if data["scope"] == "stateful-host":
            if result not in TYPES and result != "void":
                raise ContractError("stateful-host functions require scalar or void results")
            if any(arg["type"] not in TYPES for arg in fn["args"]):
                raise ContractError("stateful-host v1 supports scalar arguments only")
            if "layouts" in fn:
                raise ContractError("stateful-host v1 does not support memory layouts")
        if isinstance(result, dict) and result["type"] == "owned_bytes":
            free_args = result.setdefault("free_args", [])
            if result["allocator"] == "custom":
                if not isinstance(free_args, list) or len(free_args) > 8:
                    raise ContractError("custom owned_bytes free_args must contain up to 8 values")
                arg_by_name = {arg["name"]: arg for arg in fn["args"]}
                for free_arg in free_args:
                    if not isinstance(free_arg, dict):
                        raise ContractError("custom free_args entries must be objects")
                    if set(free_arg) == {"arg"}:
                        source = arg_by_name.get(free_arg["arg"])
                        if source is None or source["type"] not in (*TYPES, *POINTERS):
                            raise ContractError(
                                "custom free_args arg must name a scalar or pointer input"
                            )
                    elif set(free_arg) == {"type", "value"}:
                        kind = free_arg["type"]
                        if kind not in TYPES:
                            raise ContractError("custom free_args literals require an integer type")
                        _int(free_arg["value"], TYPES[kind][2], TYPES[kind][3], "free arg value")
                    else:
                        raise ContractError(
                            "custom free_args entries use an arg reference or typed integer literal"
                        )
            elif free_args:
                raise ContractError("free_args are only valid for custom owned_bytes")
        combinations = 1
        for scalar in scalars.values():
            combinations *= len(scalar["values"])
        if combinations > 4096:
            raise ContractError("scalar edge Cartesian product exceeds 4096; narrow the contract")
        for arg in fn["args"]:
            if arg["type"] not in POINTERS or arg["type"] in STRING_POINTERS or "record" in arg:
                continue
            length = arg["length"]
            if type(length) is int:
                _int(length, 0, 65536, "buffer length")
            elif isinstance(length, str) and length in scalars:
                scalar = scalars[length]
                for value in scalar["values"] + scalar.get("random_range", []):
                    _int(value, 0, 65536, "buffer length")
            else:
                raise ContractError(
                    "length must be a scalar argument name or fixed nonnegative size"
                )
        if isinstance(result, dict) and result["type"] == "owned_bytes":
            length = result["length"]
            if type(length) is int:
                _int(length, 0, 65536, "owned byte result length")
            elif isinstance(length, str) and length in scalars:
                scalar = scalars[length]
                for value in scalar["values"] + scalar.get("random_range", []):
                    _int(value, 0, 65536, "owned byte result length")
            else:
                raise ContractError("owned_bytes length must be a fixed size or scalar argument")
        pointers = {a["name"]: a for a in fn["args"] if a["type"] in POINTERS}
        output_pointers = {a["name"]: a for a in fn["args"] if a["type"] in OUT_POINTERS}
        for record_arg in pointers.values():
            if "record" not in record_arg:
                continue
            if record_arg.get("nullable", False):
                raise ContractError("record arguments with pointer fields cannot be nullable")
            pointee_names = {
                pointee["name"] for pointee in record_arg["record"].get("pointees", [])
            }
            root_pointees = set()
            pointee_edges = {name: set() for name in pointee_names}
            for _, _, record_spec, _, path_prefix, source_pointee in iter_record_instances(
                record_arg
            ):
                for field_path, _, field in iter_record_leaves(record_spec):
                    if "points_to" not in field:
                        continue
                    target_ref = field["points_to"]
                    if isinstance(target_ref, dict):
                        target_name = target_ref["pointee"]
                        if target_name not in pointee_names:
                            raise ContractError(
                                f"{record_arg['name']}.{path_prefix}{field_path}: "
                                f"unknown pointee {target_name}"
                            )
                        if source_pointee is None:
                            root_pointees.add(target_name)
                        else:
                            pointee_edges[source_pointee].add(target_name)
                        continue
                    target = pointers.get(target_ref)
                    if (
                        target is None
                        or target["name"] == record_arg["name"]
                        or "record" in target
                        or target.get("nullable", False)
                    ):
                        raise ContractError(
                            f"{record_arg['name']}.{path_prefix}{field_path}: "
                            "points_to must name a "
                            "non-null, non-record pointer argument or declared pointee"
                        )
            reachable = set()
            pending = list(root_pointees)
            while pending:
                name = pending.pop()
                if name in reachable:
                    continue
                reachable.add(name)
                pending.extend(pointee_edges[name] - reachable)
            if reachable != pointee_names:
                unused = sorted(pointee_names - reachable)
                raise ContractError(
                    f"{record_arg['name']}: pointee records are not reachable from the root: "
                    f"{unused}"
                )
        for name, arg in output_pointers.items():
            if any(target not in pointers for target in arg["within"]):
                raise ContractError(f"{name}: cstr_out within must name byte/string inputs")
            if any(pointers[target].get("nullable", False) for target in arg["within"]):
                raise ContractError(f"{name}: cstr_out targets cannot be nullable")
        string_combinations = 1
        for arg in pointers.values():
            if arg["type"] in STRING_POINTERS:
                string_combinations *= len(arg["generator"]["examples"])
        if string_combinations > 4096:
            raise ContractError("C-string example Cartesian product exceeds 4096")
        if (
            isinstance(result, dict)
            and result["type"] not in ("owned_cstr", "owned_bytes")
            and any(name not in pointers for name in result["within"])
        ):
            raise ContractError("returned pointer must be within declared pointer arguments")
        layouts = fn.get("layouts")
        if layouts is not None:
            if any("record" in arg for arg in pointers.values()):
                raise ContractError(
                    "structured record inputs cannot be combined with explicit layouts"
                )
            if any(arg["type"] in STRING_POINTERS for arg in pointers.values()):
                raise ContractError("explicit layouts with generated strings are unsupported")
            if any("align" in arg or "examples" in arg for arg in pointers.values()):
                raise ContractError(
                    "byte-buffer align/examples are unsupported with explicit layouts"
                )
            if any(arg.get("nullable", False) for arg in pointers.values()):
                raise ContractError("explicit layouts with nullable pointers are unsupported")
            if not isinstance(layouts, list) or not layouts or len(layouts) > 32 or not pointers:
                raise ContractError("layouts requires pointers and 1..32 layouts")
            layout_names = set()
            for layout in layouts:
                _keys(layout, {"name", "allocations", "bindings"}, set(), "layout")
                if not IDENT.fullmatch(layout["name"]) or layout["name"] in layout_names:
                    raise ContractError("invalid/duplicate layout name")
                layout_names.add(layout["name"])
                if not isinstance(layout["allocations"], dict) or not layout["allocations"]:
                    raise ContractError("layout requires named allocations")
                if not isinstance(layout["bindings"], dict) or set(layout["bindings"]) != set(
                    pointers
                ):
                    raise ContractError("layout must bind every pointer exactly once")
                for name, length in layout["allocations"].items():
                    if not IDENT.fullmatch(name):
                        raise ContractError("invalid allocation name")
                    validate_extent(length, scalars)
                used = set()
                for binding in layout["bindings"].values():
                    _keys(binding, {"allocation", "offset"}, set(), "binding")
                    if binding["allocation"] not in layout["allocations"]:
                        raise ContractError("binding names an unknown allocation")
                    used.add(binding["allocation"])
                    validate_extent(binding["offset"], scalars)
                if used != set(layout["allocations"]):
                    raise ContractError("unused allocation in layout")
    if data["scope"] == "stateful-host":
        fixtures = data.setdefault("fixtures", [])
        if not isinstance(fixtures, list) or len(fixtures) > 16:
            raise ContractError("stateful-host fixtures must contain at most 16 entries")
        fixture_names = set()
        fixture_symbols = set()
        function_names = {fn["c"] for fn in funcs}
        for fixture in fixtures:
            _keys(
                fixture,
                {"name", "c", "rust_decl", "return", "args"},
                {"rust_abi"},
                "fixture",
            )
            if (
                not isinstance(fixture["name"], str)
                or not IDENT.fullmatch(fixture["name"])
                or fixture["name"] in fixture_names
            ):
                raise ContractError("invalid/duplicate fixture name")
            if (
                not isinstance(fixture["c"], str)
                or not IDENT.fullmatch(fixture["c"])
                or fixture["c"] in fixture_symbols
                or fixture["c"] in function_names
            ):
                raise ContractError("invalid/duplicate fixture C symbol")
            if not isinstance(fixture["rust_decl"], str) or not IDENT.fullmatch(
                fixture["rust_decl"]
            ):
                raise ContractError("fixture rust_decl must name a module-level Rust extern item")
            if not isinstance(fixture["return"], str) or (
                fixture["return"] not in TYPES and fixture["return"] != "void"
            ):
                raise ContractError("fixture return must be a supported scalar or void")
            if fixture.setdefault("rust_abi", "C") not in ("C", "system"):
                raise ContractError("fixture rust_abi must be C or system")
            if not isinstance(fixture["args"], list) or len(fixture["args"]) > MAX_FIXTURE_ARGS:
                raise ContractError("fixture args must contain at most 8 scalar arguments")
            arg_names = set()
            for arg in fixture["args"]:
                _keys(arg, {"name", "type"}, set(), "fixture argument")
                if (
                    not isinstance(arg["name"], str)
                    or not IDENT.fullmatch(arg["name"])
                    or arg["name"] in arg_names
                ):
                    raise ContractError("invalid/duplicate fixture argument name")
                if not isinstance(arg["type"], str) or arg["type"] not in TYPES:
                    raise ContractError("fixture arguments must use supported scalar types")
                arg_names.add(arg["name"])
            fixture_names.add(fixture["name"])
            fixture_symbols.add(fixture["c"])

        sequences = data.get("sequences")
        if not isinstance(sequences, list) or not sequences or len(sequences) > 32:
            raise ContractError("stateful-host requires 1..32 sequence templates")
        sequence_names = set()
        function_indices = {fn["c"]: index for index, fn in enumerate(funcs)}
        covered_functions = set()
        estimated_cases = 0
        for sequence in sequences:
            _keys(sequence, {"name", "steps"}, {"fixtures"}, "sequence")
            if (
                not isinstance(sequence["name"], str)
                or not IDENT.fullmatch(sequence["name"])
                or sequence["name"] in sequence_names
            ):
                raise ContractError("invalid/duplicate sequence name")
            if (
                not isinstance(sequence["steps"], list)
                or not sequence["steps"]
                or len(sequence["steps"]) > MAX_SEQUENCE_STEPS
            ):
                raise ContractError("sequence steps must contain 1..64 operations")
            product = 1
            for step in sequence["steps"]:
                _keys(step, {"function"}, set(), "sequence step")
                if not isinstance(step["function"], str):
                    raise ContractError("sequence step function must be a C identifier")
                index = function_indices.get(step["function"])
                if index is None:
                    raise ContractError(f"sequence names unknown function {step['function']}")
                step["function"] = index
                covered_functions.add(index)
                for arg in funcs[index]["args"]:
                    product *= len(arg["values"])
                    if product > MAX_STATEFUL_CASES:
                        raise ContractError(
                            "sequence edge product exceeds the stateful case budget"
                        )
            expected_fixtures = sequence.setdefault("fixtures", {})
            if not isinstance(expected_fixtures, dict) or expected_fixtures.keys() - fixture_names:
                raise ContractError("sequence fixtures must name declared external fixtures")
            for fixture in fixtures:
                calls = expected_fixtures.setdefault(fixture["name"], [])
                if not isinstance(calls, list):
                    raise ContractError("fixture scripts must be arrays of expected calls")
                if len(calls) > MAX_FIXTURE_CALLS:
                    raise ContractError("fixture script exceeds 128 calls")
                for call in calls:
                    _keys(call, {"args", "return"}, set(), "fixture script call")
                    if not isinstance(call["args"], list) or len(call["args"]) != len(
                        fixture["args"]
                    ):
                        raise ContractError("fixture script call has the wrong argument count")
                    for expected, arg in zip(call["args"], fixture["args"], strict=True):
                        kind = arg["type"]
                        low, high = TYPES[kind][2:]
                        if isinstance(expected, dict):
                            _keys(
                                expected,
                                {"step", "arg"},
                                set(),
                                "fixture argument expectation",
                            )
                            if not isinstance(expected["arg"], str):
                                raise ContractError(
                                    "fixture argument reference name must be a string"
                                )
                            step_index = _int(
                                expected["step"], 0, len(sequence["steps"]) - 1, "fixture step"
                            )
                            step_fn = funcs[sequence["steps"][step_index]["function"]]
                            step_args = {item["name"]: item for item in step_fn["args"]}
                            source = step_args.get(expected["arg"])
                            if source is None or source["type"] != kind:
                                raise ContractError(
                                    "fixture argument reference must name a same-typed "
                                    "step argument"
                                )
                        else:
                            _int(expected, low, high, "fixture expected argument")
                    if fixture["return"] == "void":
                        if call["return"] is not None:
                            raise ContractError("void fixture calls require return: null")
                    else:
                        low, high = TYPES[fixture["return"]][2:]
                        _int(call["return"], low, high, "fixture response")
            fixture_call_count = sum(len(calls) for calls in expected_fixtures.values())
            if fixture_call_count > MAX_FIXTURE_CALLS:
                raise ContractError("sequence declares more than 128 total fixture calls")
            sequence_names.add(sequence["name"])
            estimated_cases += product + data["random_cases"]
        if covered_functions != set(range(len(funcs))):
            missing = sorted(
                funcs[index]["c"] for index in set(range(len(funcs))) - covered_functions
            )
            raise ContractError(f"stateful sequences do not exercise functions: {missing}")
        if fixtures and "rust_items" in data:
            raise ContractError(
                "stateful fixtures require the full Rust module so extern declarations "
                "stay in scope"
            )
        if estimated_cases > MAX_STATEFUL_CASES:
            raise ContractError("stateful generated corpus exceeds 512 sequences")
    elif "sequences" not in data:
        data["fixtures"] = []

    mutants = data["mutants"]
    if not isinstance(mutants, list) or not mutants:
        raise ContractError("at least one explicit sensitivity mutant is required")
    names = set()
    for mutant in mutants:
        _keys(mutant, {"name", "find", "replace"}, {"expect", "module"}, "mutant")
        # A support module can be shipping code the selected shims call (compress's
        # lznt1.rs); `module` lets a sensitivity control prove the harness reaches it.
        if "module" in mutant and mutant["module"] not in data["rust_support"]:
            raise ContractError(f"mutant {mutant.get('name')}: module must name a rust_support key")
        if mutant.setdefault("expect", "difference") not in ("difference", "memory_fault"):
            raise ContractError("unsupported mutant expectation")
        if mutant["expect"] == "memory_fault" and data["memory"] != "guard-pages":
            raise ContractError("memory_fault controls require guard-pages")
        if mutant["expect"] == "memory_fault" and data["scope"] == "stateful-host":
            raise ContractError("stateful-host does not support memory-fault controls")
        if not IDENT.fullmatch(mutant["name"]) or mutant["name"] in names:
            raise ContractError("invalid/duplicate mutant name")
        names.add(mutant["name"])
        if not mutant["find"] or mutant["find"] == mutant["replace"]:
            raise ContractError("mutant must change a unique nonempty source fragment")
    return data


def validate_items(items: Any) -> None:
    if (
        not isinstance(items, list)
        or not items
        or any(not isinstance(i, str) or not RUST_PATH.fullmatch(i) for i in items)
        or len(set(items)) != len(items)
    ):
        raise ContractError("rust_items must be distinct item paths")


def validate_byte_examples(arg: dict, args: list[dict]) -> None:
    """Explicit payloads for a byte buffer — the structured inputs (a valid compressed
    stream, a well-formed header) that seeded patterns essentially never produce.

    Each example is hex. Its length becomes the value of the buffer's length argument
    for that case, so `length` must name an unsigned scalar that can hold it.
    """
    examples = arg["examples"]
    if (
        arg["type"] in STRING_POINTERS
        or "record" in arg
        or not isinstance(examples, list)
        or not examples
        or len(examples) > 64
    ):
        raise ContractError(f"{arg['name']}: examples must be 1..64 hex payloads on a byte buffer")
    length = arg["length"]
    scalar = (
        next((a for a in args if a["name"] == length), None) if isinstance(length, str) else None
    )
    if scalar is None or scalar["type"] not in ("u8", "u16", "u32", "u64", "usize"):
        raise ContractError(f"{arg['name']}: examples need a length named by an unsigned scalar")
    for example in examples:
        try:
            payload = bytes.fromhex(example)
        except (TypeError, ValueError) as exc:
            raise ContractError(f"{arg['name']}: example is not hex") from exc
        _int(len(payload), 0, min(TYPES[scalar["type"]][3], 65536), f"{arg['name']} example length")


def validate_c_pointer(arg: dict) -> None:
    if "rust_type" in arg and (
        arg.get("type") not in POINTERS
        or not isinstance(arg["rust_type"], str)
        or (
            arg["rust_type"] not in RUST_POINTER_TYPES
            and not RUST_RAW_POINTER.fullmatch(arg["rust_type"])
        )
    ):
        raise ContractError("rust_type overrides are limited to supported raw pointer types")
    if arg.get("type") == "const_wstr" and "c_type" in arg:
        raise ContractError("const_wstr has the fixed C type const uint16_t *")
    if "align" in arg and (
        arg.get("type") not in POINTERS
        or arg["type"] in STRING_POINTERS
        or "record" in arg
        or arg["align"] not in (1, 2, 4, 8, 16)
        or type(arg["align"]) is not int
    ):
        raise ContractError("align applies to byte buffers only and must be 1, 2, 4, 8 or 16")
    if "c_type" in arg and (
        arg["type"] not in POINTERS
        or (
            arg.get("record") is None
            and arg["c_type"]
            not in ("char *", "const char *", "unsigned char *", "const unsigned char *")
            # A wider element type (`ULONG *`) is honest only with its alignment declared:
            # the right-guard placement would otherwise hand the C a misaligned pointer.
            and not ("align" in arg and C_POINTER_TYPE.fullmatch(str(arg["c_type"])))
        )
        or (
            arg.get("record") is not None
            and (not isinstance(arg["c_type"], str) or not C_POINTER_TYPE.fullmatch(arg["c_type"]))
        )
    ):
        raise ContractError("c_type overrides are limited to byte pointers or record pointer types")
    if arg.get("type") == "const_cstr" and arg.get("c_type", "const char *") != "const char *":
        raise ContractError("const_cstr requires the exact C type const char *")
    if arg.get("type") == "mut_cstr" and arg.get("c_type", "char *") != "char *":
        raise ContractError("mut_cstr requires the exact C type char *")


def extent(value: Any, values: dict) -> int:
    if isinstance(value, dict):
        return values[value["arg"]] + value["add"]
    return values[value] if isinstance(value, str) else value


def validate_extent(value: Any, scalars: dict) -> None:
    if type(value) is int:
        _int(value, 0, 65536, "extent")
        return
    if isinstance(value, dict):
        _keys(value, {"arg", "add"}, set(), "extent")
        name = value["arg"]
        _int(value["add"], -65536, 65536, "extent addend")
    else:
        name = value
    if not isinstance(name, str) or name not in scalars:
        raise ContractError("extent requires a scalar name, integer, or {arg, add}")
    for n in scalars[name]["values"] + scalars[name].get("random_range", []):
        _int(extent(value, {name: n}), 0, 65536, "resolved extent")


def encode_record(
    spec: dict,
    values: dict[str, int],
    pattern: int,
    rng: random.Random,
    prefix: str = "",
) -> tuple[bytes, dict[str, Any]]:
    """Pack a validated nested record tree and any root trailing array."""
    fields = spec["fields"]
    size = spec["base_size"]
    counts: dict[str, int] = {}
    for field in fields:
        if "record" in field:
            size = max(
                size,
                field["offset"] + field["record"]["base_size"] * field.get("count", 1),
            )
            continue
        if "points_to" in field:
            size = max(size, field["offset"] + ctypes.sizeof(ctypes.c_void_p))
            continue
        path = f"{prefix}.{field['name']}" if prefix else field["name"]
        count = (
            field["count"]
            if "count" in field
            else max(
                values[f"{prefix}.{field['count_from']}" if prefix else field["count_from"]],
                field["min_count"],
            )
        )
        counts[field["name"]] = count
        size = max(size, field["offset"] + count * ctypes.sizeof(TYPES[field["type"]][1]))
    payload = bytearray(size)
    details: dict[str, Any] = {}
    for field in fields:
        name = field["name"]
        path = f"{prefix}.{name}" if prefix else name
        offset = field["offset"]
        if "record" in field:
            is_array = "count" in field
            count = field.get("count", 1)
            stride = field["record"]["base_size"]
            child_details = []
            for index in range(count):
                child_prefix = f"{path}[{index}]" if is_array else path
                child, item_details = encode_record(
                    field["record"], values, pattern, rng, prefix=child_prefix
                )
                start = offset + index * stride
                payload[start : start + len(child)] = child
                child_details.append(item_details)
            details[name] = child_details if is_array else child_details[0]
            continue
        if "points_to" in field:
            details[name] = {
                "points_to": field["points_to"],
                "nullable": field["nullable"],
                "one_past": field["one_past"],
            }
            continue
        kind = field["type"]
        width = ctypes.sizeof(TYPES[kind][1])
        count = counts[name]
        domain = field["values"]
        if "count" in field and field["count"] == 1:
            elements = [values[path]]
        elif pattern == 0:
            elements = [domain[0]] * count
        elif pattern == 1:
            elements = [domain[-1]] * count
        elif pattern == 2:
            elements = [domain[i % len(domain)] for i in range(count)]
        elif pattern == 3:
            reverse = list(reversed(domain))
            elements = [reverse[i % len(reverse)] for i in range(count)]
        else:
            elements = [rng.choice(domain) for _ in range(count)]
        for index, value in enumerate(elements):
            element_offset = offset + index * width
            payload[element_offset : element_offset + width] = value.to_bytes(
                width, "little", signed=TYPES[kind][2] < 0
            )
        details[name] = elements[0] if count == 1 else elements
    return bytes(payload), details


def generate_cases(data: dict) -> list[dict]:
    rng = random.Random(data["seed"])
    cases: list[dict] = []
    payload_bytes = 0
    for index, fn in enumerate(data["functions"]):
        scalars = [a for a in fn["args"] if a["type"] in TYPES]
        pointers = [a for a in fn["args"] if a["type"] in POINTERS]
        records = [a for a in pointers if "record" in a]
        record_instances = [instance for arg in records for instance in iter_record_instances(arg)]
        output_pointers = [a for a in fn["args"] if a["type"] in OUT_POINTERS]
        nullable_args = [a for a in (*pointers, *output_pointers) if a.get("nullable", False)]
        strings = [a for a in pointers if a["type"] in STRING_POINTERS]
        byte_buffers = [
            a for a in pointers if a["type"] not in STRING_POINTERS and "record" not in a
        ]
        layouts = fn.get("layouts")
        if layouts is None:
            allocations = {
                a["name"]: (
                    {"string": a["name"]}
                    if a["type"] in STRING_POINTERS
                    else {"record": a["name"]}
                    if "record" in a
                    else a["length"]
                )
                for a in pointers
            }
            layouts = [
                {
                    "name": "disjoint",
                    "allocations": allocations,
                    "bindings": {
                        a["name"]: {"allocation": a["name"], "offset": 0} for a in pointers
                    },
                }
            ]
        string_domains = [a["generator"]["examples"] for a in strings]
        record_fields = [
            (arg, binding, value_prefix, path_prefix, path, field)
            for arg, binding, record_spec, value_prefix, path_prefix, _ in record_instances
            for path, _, field in iter_record_leaves(record_spec)
            if "points_to" not in field and "count" in field and field["count"] == 1
        ]
        record_domains = [field["values"] for _, _, _, _, _, field in record_fields]

        def contains_array(record: dict) -> bool:
            return any(
                ("record" in field and "count" in field)
                or ("record" not in field and field.get("count", 1) > 1)
                or "count_from" in field
                or ("record" in field and contains_array(field["record"]))
                for field in record["fields"]
            )

        has_record_arrays = any(
            contains_array(record_spec) for _, _, record_spec, _, _, _ in record_instances
        )
        patterns = range(5 if byte_buffers or has_record_arrays else 1)
        placements = (
            ("left", "right") if pointers and data["memory"] == "guard-pages" else ("canary",)
        )

        def combinations_count(domains: list[list]) -> int:
            count = 1
            for domain in domains:
                count *= len(domain)
            return count

        prospective = (
            (
                (
                    combinations_count([arg["values"] for arg in scalars])
                    * combinations_count(string_domains)
                    * combinations_count(record_domains)
                    * len(patterns)
                    + data["random_cases"]
                )
                * (2 ** len(nullable_args))
            )
            * len(layouts)
            * len(placements)
        )
        if prospective > 100000:
            raise ContractError("corpus exceeds 100000 case budget; narrow contract")

        scalar_rows = [
            dict(zip((a["name"] for a in scalars), row, strict=True))
            for row in itertools.product(*(a["values"] for a in scalars))
        ]
        string_combinations = [
            dict(zip((a["name"] for a in strings), row, strict=True))
            for row in itertools.product(*string_domains)
        ]
        record_combinations = [
            dict(
                zip(
                    (
                        (arg["name"], f"{value_prefix}{path}")
                        for arg, _, value_prefix, _, path, _ in record_fields
                    ),
                    row,
                    strict=True,
                )
            )
            for row in itertools.product(*record_domains)
        ] or [{}]
        null_combinations = [
            dict(zip((a["name"] for a in nullable_args), row, strict=True))
            for row in itertools.product(*((False, True) for _ in nullable_args))
        ]
        record_pointer_fields = []
        for arg, binding, record_spec, _, path_prefix, pointee_name in record_instances:
            for field_path, field_offset, field in iter_record_leaves(record_spec):
                if "points_to" not in field:
                    continue
                target_ref = field["points_to"]
                target_binding = (
                    f"@{arg['name']}/pointee/{target_ref['pointee']}"
                    if isinstance(target_ref, dict)
                    else target_ref
                )
                record_pointer_fields.append(
                    {
                        "record_arg": arg["name"],
                        "source_binding": binding,
                        "field": f"{path_prefix}{field_path}",
                        "offset": field_offset,
                        "target_arg": target_binding,
                        "target_pointee": target_ref.get("pointee")
                        if isinstance(target_ref, dict)
                        else None,
                        "source_pointee": pointee_name,
                        "nullable": field["nullable"],
                        "one_past": field["one_past"],
                    }
                )
        rows = [
            (values, pattern, string_values, record_values, null_values, {})
            for values in scalar_rows
            for pattern in patterns
            for string_values in string_combinations
            for record_values in record_combinations
            for null_values in null_combinations
        ]
        for _ in range(data["random_cases"]):
            values = {
                a["name"]: (
                    rng.randint(*a["random_range"])
                    if "random_range" in a
                    else rng.choice(a["values"])
                )
                for a in scalars
            }
            string_values = {
                a["name"]: generate_string(a["generator"]["grammar"], rng) for a in strings
            }
            record_values = {
                (arg["name"], f"{value_prefix}{path}"): rng.choice(field["values"])
                for arg, _, value_prefix, _, path, field in record_fields
            }
            for null_values in null_combinations:
                rows.append((values, 4, string_values, record_values, null_values, {}))
        # Example payloads replace one buffer's bytes and set its length argument; every
        # other scalar keeps its edge values (deduplicated once the length is overridden).
        for arg in byte_buffers:
            for example in arg.get("examples", []):
                payload = bytes.fromhex(example)
                seen_rows = set()
                for scalar_row in scalar_rows:
                    values = {**scalar_row, arg["length"]: len(payload)}
                    key = tuple(sorted(values.items()))
                    if key in seen_rows:
                        continue
                    seen_rows.add(key)
                    for null_values in null_combinations:
                        if null_values.get(arg["name"], False):
                            continue
                        rows.append(
                            (
                                values,
                                2,
                                string_combinations[0],
                                record_combinations[0],
                                null_values,
                                {arg["name"]: payload},
                            )
                        )
        for values, pattern, string_values, record_values, null_values, overrides in rows:
            for layout in layouts:
                allocations = {}
                allocation_lengths = {}
                # Natural alignment per allocation, so a right-guard placement flush
                # against the page end cannot misalign a record or a UTF-16 string.
                alignments: dict[str, int] = {}
                record_details = {}

                for name, size in layout["allocations"].items():
                    if null_values.get(name, False):
                        continue
                    if isinstance(size, dict) and set(size) == {"string"}:
                        string_name = size["string"]
                        if string_name not in string_values:
                            raise ContractError("automatic string allocation names no input")
                        string_arg = next(arg for arg in strings if arg["name"] == string_name)
                        payload = _encode_string_input(
                            string_values[string_name], string_arg["type"]
                        )
                        if string_arg["type"] in ("const_wstr", "mut_wstr"):
                            alignments[name] = 2
                    elif isinstance(size, dict) and set(size) == {"record"}:
                        record_name = size["record"]
                        record_arg = next(arg for arg in records if arg["name"] == record_name)
                        field_values = record_instance_values(
                            record_arg["name"], "", record_arg["record"], record_values
                        )
                        payload, record_details[record_name] = encode_record(
                            record_arg["record"], field_values, pattern, rng
                        )
                        alignments[name] = record_alignment(record_arg["record"])
                    else:
                        n = extent(size, values)
                        byte_arg = next((a for a in byte_buffers if a["name"] == name), None)
                        if byte_arg is not None and byte_arg.get("align", 1) > 1:
                            alignments[name] = byte_arg["align"]
                        if name in overrides:
                            payload = overrides[name]
                        elif pattern == 4:
                            payload = bytes(rng.randrange(256) for _ in range(n))
                        else:
                            payload = bytes(
                                (0, 255, i % 256, (255 - i) % 256)[pattern] for i in range(n)
                            )
                    n = len(payload)
                    payload_bytes += n * len(placements)
                    if payload_bytes > 16 * 1024 * 1024 or len(cases) >= 100000:
                        raise ContractError(
                            "corpus exceeds 16 MiB/100000 case budget; narrow contract"
                        )
                    allocations[name] = payload.hex()
                    allocation_lengths[name] = n
                pointee_details = {}
                for arg, binding, record_spec, value_prefix, _, pointee_name in record_instances:
                    if pointee_name is None:
                        continue
                    field_values = record_instance_values(
                        arg["name"], value_prefix, record_spec, record_values
                    )
                    payload, details = encode_record(record_spec, field_values, pattern, rng)
                    n = len(payload)
                    payload_bytes += n * len(placements)
                    if payload_bytes > 16 * 1024 * 1024 or len(cases) >= 100000:
                        raise ContractError(
                            "corpus exceeds 16 MiB/100000 case budget; narrow contract"
                        )
                    allocations[binding] = payload.hex()
                    allocation_lengths[binding] = n
                    alignments[binding] = record_alignment(record_spec)
                    pointee_details.setdefault(arg["name"], {})[pointee_name] = details
                byte_pointee_bindings = {}
                for arg in records:
                    for pointee in arg["record"].get("pointees", []):
                        if "bytes" not in pointee:
                            continue
                        spec = pointee["bytes"]
                        binding = f"@{arg['name']}/pointee/{pointee['name']}"
                        n = (
                            record_values[(arg["name"], spec["length"])]
                            if isinstance(spec["length"], str)
                            else spec["length"]
                        )
                        if pattern == 4:
                            payload = bytes(rng.randrange(256) for _ in range(n))
                        else:
                            payload = bytes(
                                (0, 255, i % 256, (255 - i) % 256)[pattern] for i in range(n)
                            )
                        payload_bytes += n * len(placements)
                        if payload_bytes > 16 * 1024 * 1024 or len(cases) >= 100000:
                            raise ContractError(
                                "corpus exceeds 16 MiB/100000 case budget; narrow contract"
                            )
                        allocations[binding] = payload.hex()
                        allocation_lengths[binding] = n
                        if spec["align"] > 1:
                            alignments[binding] = spec["align"]
                        byte_pointee_bindings[binding] = {
                            "allocation": binding,
                            "offset": 0,
                            "length": n,
                            "readonly": not pointee["mutable"],
                        }
                bindings = {}
                for arg in pointers:
                    if null_values.get(arg["name"], False):
                        bindings[arg["name"]] = {"null": True}
                        continue
                    binding = layout["bindings"][arg["name"]]
                    offset = extent(binding["offset"], values)
                    size = (
                        len(_encode_string_input(string_values[arg["name"]], arg["type"]))
                        if arg["type"] in STRING_POINTERS
                        else allocation_lengths[binding["allocation"]]
                        if "record" in arg
                        else extent(arg["length"], values)
                    )
                    capacity = len(allocations[binding["allocation"]]) // 2
                    if not 0 <= offset <= offset + size <= capacity:
                        raise ContractError(
                            f"layout {layout['name']}: {arg['name']} view outside allocation"
                        )
                    bindings[arg["name"]] = {
                        "allocation": binding["allocation"],
                        "offset": offset,
                        "length": size,
                        "readonly": arg["type"] in READONLY,
                    }
                for arg, binding, _, _, _, pointee_name in record_instances:
                    if pointee_name is not None:
                        pointee = next(
                            item
                            for item in arg["record"]["pointees"]
                            if item["name"] == pointee_name
                        )
                        bindings[binding] = {
                            "allocation": binding,
                            "offset": 0,
                            "length": allocation_lengths[binding],
                            "readonly": not pointee["mutable"],
                        }
                bindings.update(byte_pointee_bindings)
                for placement in placements if allocations else ("canary",):
                    if len(cases) >= 100000:
                        raise ContractError("corpus exceeds 100000 case budget; narrow contract")
                    case = {
                        "function": index,
                        "args": values,
                        "allocations": allocations,
                        "bindings": bindings,
                        "layout": layout["name"],
                        "placement": placement if allocations else "canary",
                    }
                    if any(value > 1 for value in alignments.values()):
                        case["alignments"] = {k: v for k, v in alignments.items() if v > 1}
                    if string_values:
                        case["strings"] = string_values
                    if record_details:
                        case["records"] = record_details
                    if pointee_details:
                        case["pointees"] = pointee_details
                    if record_pointer_fields:
                        case["pointer_fields"] = [
                            {
                                **field,
                                "one_past": field["one_past"]
                                or bindings[field["target_arg"]]["length"] == 0,
                            }
                            for field in record_pointer_fields
                        ]
                    if nullable_args:
                        case["nulls"] = null_values
                    if output_pointers:
                        case["outptrs"] = [arg["name"] for arg in output_pointers]
                    cases.append(case)
    return cases


def generate_stateful_cases(data: dict) -> list[dict]:
    """Expand reviewed operation templates into isolated scalar-call sequences."""
    rng = random.Random(data["seed"])
    cases: list[dict] = []
    for sequence_index, sequence in enumerate(data["sequences"]):
        columns = [
            (step_index, arg)
            for step_index, step in enumerate(sequence["steps"])
            for arg in data["functions"][step["function"]]["args"]
        ]
        domains = [arg["values"] for _, arg in columns]
        samples = itertools.product(*domains) if domains else [()]
        rows: list[tuple[int, ...]] = list(samples)
        for _ in range(data["random_cases"]):
            values = tuple(
                rng.randint(*arg["random_range"])
                if "random_range" in arg
                else rng.choice(arg["values"])
                for _, arg in columns
            )
            rows.append(values)
        for row in rows:
            if len(cases) >= MAX_STATEFUL_CASES:
                raise ContractError("stateful generated corpus exceeds 512 sequences")
            step_values = [dict() for _ in sequence["steps"]]
            for (step_index, arg), value in zip(columns, row, strict=True):
                step_values[step_index][arg["name"]] = value
            steps = [
                {"function": step["function"], "args": step_values[index]}
                for index, step in enumerate(sequence["steps"])
            ]
            fixture_scripts = {}
            for fixture in data["fixtures"]:
                resolved_calls = []
                for call in sequence["fixtures"][fixture["name"]]:
                    args = []
                    for value in call["args"]:
                        if isinstance(value, dict):
                            args.append(step_values[value["step"]][value["arg"]])
                        else:
                            args.append(value)
                    resolved_calls.append({"args": args, "return": call["return"]})
                fixture_scripts[fixture["name"]] = resolved_calls
            cases.append(
                {
                    "scenario": sequence_index,
                    "name": sequence["name"],
                    "steps": steps,
                    "fixtures": fixture_scripts,
                }
            )
    return cases


def generate_contract_cases(data: dict) -> list[dict]:
    if data["scope"] == "stateful-host":
        return generate_stateful_cases(data)
    return generate_cases(data)


def case_functions(case: dict) -> set[int]:
    if "function" in case:
        return {case["function"]}
    return {step["function"] for step in case["steps"]}


def case_mismatch_functions(case: dict, reference: dict, candidate: dict) -> set[int]:
    if "function" in case:
        return {case["function"]}
    involved = set()
    reference_steps = reference.get("steps", [])
    candidate_steps = candidate.get("steps", [])
    for index, (left, right) in enumerate(zip(reference_steps, candidate_steps, strict=False)):
        if left != right:
            involved.update(step["function"] for step in case["steps"][: index + 1])
    if len(reference_steps) != len(candidate_steps) or reference.get("violations") != candidate.get(
        "violations"
    ):
        involved.update(case_functions(case))
    return involved


def _u64_bits(value: int) -> int:
    return value & ((1 << 64) - 1)


def _from_u64_bits(value: int, kind: str) -> int:
    bits = 8 * ctypes.sizeof(TYPES[kind][1])
    value &= (1 << bits) - 1
    if TYPES[kind][2] < 0 and value >= 1 << (bits - 1):
        value -= 1 << bits
    return value


# Items copied as exact bytes. Functions are what gets graded; the rest are what a selected
# body names — a lookup table, a layout, a C macro kept as `macro_rules!` — and would
# otherwise force compiling a whole module whose other items the host cannot build.
SELECTABLE_ITEMS = {
    "function_item",
    "static_item",
    "const_item",
    "struct_item",
    "union_item",
    "enum_item",
    "type_item",
    "macro_definition",
}


def selected_rust(source: str, names: list[str], *, preserve_cfg: bool = False) -> str:
    """Copy exact top-level item bytes, retaining applicable attributes. No repair."""
    import tree_sitter_rust
    from tree_sitter import Language, Parser

    raw = source.encode()
    tree = Parser(Language(tree_sitter_rust.language())).parse(raw)
    if tree.root_node.has_error:
        raise ContractError("cannot select items from syntactically invalid Rust")
    found: dict[str, list[str]] = {}
    unguarded: set[str] = set()

    def visit(parent, prefix: str, inherited: list[str]) -> None:
        inherited = list(inherited)
        for child in parent.children:
            if child.type == "inner_attribute_item":
                attribute = raw[child.start_byte : child.end_byte].decode()
                if "cfg" not in attribute:
                    continue
                # Only `#![cfg(..)]` decides whether the items exist, so only it is carried
                # onto each selection. A crate-level `#![cfg_attr(not(test), no_std)]` is
                # configuration of the crate, not of an item; re-emitted outer it would
                # attach `no_std` to a function. A cfg_attr that expands to a cfg is refused.
                if re.match(r"#!\[\s*cfg\s*\(", attribute):
                    if not preserve_cfg:
                        raise ContractError(
                            "conditional crate/module attributes cannot be discarded"
                        )
                    inherited.append(attribute.replace("#![", "#[", 1))
                elif re.search(r"\bcfg\s*\(", attribute.split("(", 1)[1].split(",", 1)[-1]):
                    raise ContractError(f"conditional cfg_attr cannot be carried: {attribute}")
        attrs: list[str] = []
        for node in parent.children:
            if node.type == "attribute_item":
                attrs.append(raw[node.start_byte : node.end_byte].decode())
                continue
            if node.type in ("line_comment", "block_comment"):
                continue
            name_node = node.child_by_field_name("name")
            name = raw[name_node.start_byte : name_node.end_byte].decode() if name_node else ""
            qualified = prefix + name
            guards = inherited + [a for a in attrs if "cfg" in a]
            if node.type in SELECTABLE_ITEMS and qualified in names:
                # Same-named definitions are kept only when every copy is cfg-guarded
                # (`#[cfg(test)]` beside `#[cfg(not(test))]`): rustc then keeps exactly one.
                if (guards and not preserve_cfg) or (
                    qualified in found and (not guards or qualified in unguarded)
                ):
                    raise ContractError(f"conditional/duplicate selected item: {qualified}")
                if not guards:
                    unguarded.add(qualified)
                stack = [node]
                while stack:
                    child = stack.pop()
                    if child.type in ("scoped_identifier", "scoped_type_identifier"):
                        spelling = raw[child.start_byte : child.end_byte].decode()
                        if spelling.startswith(("self::", "super::")):
                            raise ContractError(
                                f"relative module path in selected item: {qualified}"
                            )
                    stack.extend(child.named_children)
                found.setdefault(qualified, []).append(
                    "\n".join([*inherited, *attrs, raw[node.start_byte : node.end_byte].decode()])
                )
            elif node.type == "mod_item" and any(n.startswith(qualified + "::") for n in names):
                body = node.child_by_field_name("body")
                if body is None:
                    raise ContractError("selection through external modules is unsupported")
                visit(body, qualified + "::", guards)
            attrs = []

    visit(tree.root_node, "", [])
    if set(found) != set(names):
        raise ContractError(f"missing Rust items: {sorted(set(names) - set(found))}")
    return "\n".join(copy for n in names for copy in found[n]) + "\n"


def _fixture_c_type(kind: str) -> str:
    return "void" if kind == "void" else TYPES[kind][0]


def _fixture_rust_type(kind: str) -> str:
    return "()" if kind == "void" else kind


def _c_fixture_prototype(fixture: dict) -> str:
    args = [TYPES[arg["type"]][0] for arg in fixture["args"]]
    return f"{_fixture_c_type(fixture['return'])} {fixture['c']}({', '.join(args) or 'void'});"


def _c_fixture_runtime(data: dict, cases: list[dict]) -> list[str]:
    fixtures = data["fixtures"]
    if not fixtures:
        return []
    lines = [
        "struct difftest_fixture_event {",
        "  uint32_t fixture; uint32_t argc; uint64_t args[8]; uint64_t result;",
        "};",
        "static uint32_t __df_case;",
        "static uint32_t __df_error;",
        "static uint32_t __df_event_count;",
        f"static uint32_t __df_calls[{len(fixtures)}];",
        f"static struct difftest_fixture_event __df_events[{MAX_FIXTURE_EVENTS}];",
        "static void __df_record(uint32_t fixture, uint32_t argc, const uint64_t *args, "
        "uint64_t result) {",
        f"  if (__df_event_count >= {MAX_FIXTURE_EVENTS}) {{ __df_error = 1; return; }}",
        "  struct difftest_fixture_event *event = &__df_events[__df_event_count++];",
        "  event->fixture = fixture; event->argc = argc; event->result = result;",
        "  for (uint32_t i = 0; i < 8; ++i) event->args[i] = i < argc ? args[i] : 0;",
        "}",
        "void diff_fixture_reset(uint32_t case_id) {",
        f"  if (case_id >= {len(cases)}) {{ __df_error = 1; __df_case = 0; }}",
        "  else { __df_error = 0; __df_case = case_id; }",
        "  __df_event_count = 0;",
        f"  for (uint32_t i = 0; i < {len(fixtures)}; ++i) __df_calls[i] = 0;",
        "}",
        "uint32_t diff_fixture_event_count(void) { return __df_event_count; }",
        "uint32_t diff_fixture_event_id(uint32_t i) {\n"
        "  return i < __df_event_count ? __df_events[i].fixture : 0;\n}",
        "uint32_t diff_fixture_event_argc(uint32_t i) {\n"
        "  return i < __df_event_count ? __df_events[i].argc : 0;\n}",
        "uint64_t diff_fixture_event_arg(uint32_t i, uint32_t a) {\n"
        "  return i < __df_event_count && a < 8 ? __df_events[i].args[a] : 0;\n}",
        "uint64_t diff_fixture_event_result(uint32_t i) {\n"
        "  return i < __df_event_count ? __df_events[i].result : 0;\n}",
        "uint32_t diff_fixture_finish(void) {",
        "  uint32_t expected[" + str(len(fixtures)) + "] = {0};",
        "  switch (__df_case) {",
    ]
    for case_index, case in enumerate(cases):
        lines.append(f"    case {case_index}:")
        for fixture_index, fixture in enumerate(fixtures):
            count = len(case["fixtures"][fixture["name"]])
            if count:
                lines.append(f"      expected[{fixture_index}] = {count};")
        lines.append("      break;")
    lines.extend(
        [
            "    default: __df_error = 1; break;",
            "  }",
        ]
    )
    for fixture_index in range(len(fixtures)):
        lines.append(
            f"  if (__df_calls[{fixture_index}] != expected[{fixture_index}]) __df_error = 1;"
        )
    lines.extend(["  return __df_error;", "}"])
    for fixture_index, fixture in enumerate(fixtures):
        arg_types = [TYPES[arg["type"]][0] for arg in fixture["args"]]
        arg_names = [f"arg_{index}" for index in range(len(arg_types))]
        params = (
            ", ".join(f"{kind} {name}" for kind, name in zip(arg_types, arg_names, strict=True))
            or "void"
        )
        actuals = ", ".join(f"(uint64_t){name}" for name in arg_names) or "0"
        lines.extend(
            [
                f"{_fixture_c_type(fixture['return'])} {fixture['c']}({params}) {{",
                f"  uint32_t call = __df_calls[{fixture_index}]++;",
                f"  uint64_t actual[8] = {{{actuals}}};",
                "  uint64_t response = 0;",
                "  switch (__df_case) {",
            ]
        )
        for case_index, case in enumerate(cases):
            script = case["fixtures"][fixture["name"]]
            lines.extend([f"    case {case_index}: switch (call) {{"])
            for call_index, expected in enumerate(script):
                checks = [
                    f"actual[{arg_index}] != UINT64_C({_u64_bits(value)})"
                    for arg_index, value in enumerate(expected["args"])
                ]
                if checks:
                    lines.append(
                        f"      case {call_index}: if ({' || '.join(checks)}) __df_error = 1;"
                    )
                else:
                    lines.append(f"      case {call_index}:")
                if fixture["return"] != "void":
                    lines.append(f"        response = UINT64_C({_u64_bits(expected['return'])});")
                lines.append("        break;")
            lines.extend(["      default: __df_error = 1; break;", "    } break;"])
        lines.extend(
            [
                "    default: __df_error = 1; break;",
                "  }",
                f"  __df_record({fixture_index}, {len(arg_types)}, actual, response);",
            ]
        )
        if fixture["return"] == "void":
            lines.append("  return;")
        else:
            lines.append(f"  return ({_fixture_c_type(fixture['return'])})response;")
        lines.append("}")
    return lines


def _rust_fixture_runtime(data: dict, cases: list[dict]) -> list[str]:
    fixtures = data["fixtures"]
    if not fixtures:
        return []
    lines = [
        "#[derive(Clone, Copy)]\n"
        "struct DifftestFixtureEvent {\n"
        "  fixture: u32, argc: u32, args: [u64; 8], result: u64,\n}",
        "static mut __DF_CASE: u32 = 0;",
        "static mut __DF_ERROR: u32 = 0;",
        "static mut __DF_EVENT_COUNT: u32 = 0;",
        f"static mut __DF_CALLS: [u32; {len(fixtures)}] = [0; {len(fixtures)}];",
        "static mut __DF_EVENTS: [DifftestFixtureEvent; "
        f"{MAX_FIXTURE_EVENTS}] = [\n"
        "  DifftestFixtureEvent { fixture: 0, argc: 0, args: [0; 8], result: 0 };\n"
        f"  {MAX_FIXTURE_EVENTS}\n];",
        "fn __df_record(fixture: u32, args: &[u64], result: u64) {",
        "  unsafe {",
        f"    if __DF_EVENT_COUNT >= {MAX_FIXTURE_EVENTS} {{ __DF_ERROR = 1; return; }}",
        "    let index = __DF_EVENT_COUNT as usize;",
        "    let mut stored = [0u64; 8];",
        "    for (to, from) in stored.iter_mut().zip(args.iter()) { *to = *from; }",
        "    __DF_EVENTS[index] = DifftestFixtureEvent {\n"
        "      fixture, argc: args.len() as u32, args: stored, result,\n    };",
        "    __DF_EVENT_COUNT += 1;",
        "  }",
        "}",
        '#[no_mangle] pub extern "C" fn diff_fixture_reset(case_id: u32) {',
        "  unsafe {",
        f"    if case_id >= {len(cases)} {{\n"
        "      __DF_ERROR = 1; __DF_CASE = 0;\n"
        "    } else {\n"
        "      __DF_ERROR = 0; __DF_CASE = case_id;\n    }",
        "    __DF_EVENT_COUNT = 0;",
        "    for call in &mut __DF_CALLS { *call = 0; }",
        "  }",
        "}",
        '#[no_mangle]\npub extern "C" fn diff_fixture_event_count() -> u32 {\n'
        "  unsafe { __DF_EVENT_COUNT }\n}",
        '#[no_mangle]\npub extern "C" fn diff_fixture_event_id(i: u32) -> u32 {\n'
        "  unsafe { if i < __DF_EVENT_COUNT { __DF_EVENTS[i as usize].fixture } else { 0 } }\n}",
        '#[no_mangle]\npub extern "C" fn diff_fixture_event_argc(i: u32) -> u32 {\n'
        "  unsafe { if i < __DF_EVENT_COUNT { __DF_EVENTS[i as usize].argc } else { 0 } }\n}",
        '#[no_mangle]\npub extern "C" fn diff_fixture_event_arg(i: u32, a: u32) -> u64 {\n'
        "  unsafe {\n"
        "    if i < __DF_EVENT_COUNT && a < 8 { __DF_EVENTS[i as usize].args[a as usize] }\n"
        "    else { 0 }\n  }\n}",
        '#[no_mangle]\npub extern "C" fn diff_fixture_event_result(i: u32) -> u64 {\n'
        "  unsafe { if i < __DF_EVENT_COUNT { __DF_EVENTS[i as usize].result } else { 0 } }\n}",
        '#[no_mangle] pub extern "C" fn diff_fixture_finish() -> u32 {',
        "  unsafe {",
        f"    let mut expected = [0u32; {len(fixtures)}];",
        "    match __DF_CASE {",
    ]
    for case_index, case in enumerate(cases):
        lines.append(f"      {case_index} => {{")
        for fixture_index, fixture in enumerate(fixtures):
            count = len(case["fixtures"][fixture["name"]])
            if count:
                lines.append(f"        expected[{fixture_index}] = {count};")
        lines.append("      },")
    lines.extend(["      _ => __DF_ERROR = 1,", "    }"])
    for fixture_index in range(len(fixtures)):
        lines.append(
            f"    if __DF_CALLS[{fixture_index}] != expected[{fixture_index}] {{ __DF_ERROR = 1; }}"
        )
    lines.extend(["    __DF_ERROR", "  }", "}"])
    for fixture_index, fixture in enumerate(fixtures):
        arg_names = [f"arg_{index}" for index in range(len(fixture["args"]))]
        arg_types = [arg["type"] for arg in fixture["args"]]
        params = ", ".join(
            f"{name}: {kind}" for name, kind in zip(arg_names, arg_types, strict=True)
        )
        rust_abi = fixture["rust_abi"]
        rust_signature = f'extern "{rust_abi}" ' if rust_abi != "Rust" else ""
        actuals = ", ".join(f"{name} as u64" for name in arg_names) or ""
        lines.extend(
            [
                "#[no_mangle]",
                f"pub unsafe {rust_signature}fn {fixture['c']}({params}) -> "
                f"{_fixture_rust_type(fixture['return'])} {{",
                f"  let call = unsafe {{ __DF_CALLS[{fixture_index}] }};",
                f"  unsafe {{ __DF_CALLS[{fixture_index}] = call.wrapping_add(1); }}",
                f"  let actual: [u64; {max(1, len(arg_types))}] = [{actuals or '0'}];",
                "  let case_id = unsafe { __DF_CASE };",
                "  let mut response = 0u64;",
                "  match case_id {",
            ]
        )
        for case_index, case in enumerate(cases):
            script = case["fixtures"][fixture["name"]]
            lines.extend([f"    {case_index} => match call {{"])
            for call_index, expected in enumerate(script):
                checks = [
                    f"actual[{arg_index}] != {_u64_bits(value)}u64"
                    for arg_index, value in enumerate(expected["args"])
                ]
                lines.append(f"      {call_index} => {{")
                if checks:
                    lines.append(
                        f"        if {' || '.join(checks)} {{ unsafe {{ __DF_ERROR = 1; }} }}"
                    )
                if fixture["return"] != "void":
                    lines.append(f"        response = {_u64_bits(expected['return'])}u64;")
                lines.extend(["      },"])
            lines.extend(["      _ => unsafe { __DF_ERROR = 1; },", "    },"])
        lines.extend(
            [
                "    _ => unsafe { __DF_ERROR = 1; },",
                "  }",
                f"  __df_record({fixture_index}, &actual[..{len(arg_types)}], response);",
            ]
        )
        if fixture["return"] == "void":
            lines.append("  ()")
        else:
            lines.append(f"  response as {fixture['return']}")
        lines.append("}")
    return lines


def _rust_fixture_type_checks(data: dict) -> str:
    checks = []
    for fixture in data["fixtures"]:
        arg_types = ", ".join(arg["type"] for arg in fixture["args"])
        result = _fixture_rust_type(fixture["return"])
        abi = fixture["rust_abi"]
        checks.append(
            f'const _: unsafe extern "{abi}" fn({arg_types}) -> {result} = {fixture["rust_decl"]};'
        )
    return "\n".join(checks)


def _record_layout_checks(
    record: dict,
    fn: dict,
    label: str,
    pointer_counter: list[int],
    nested_counter: list[int],
    root_record: dict | None = None,
) -> list[str]:
    """Emit host-C size, field-offset, width and recursive-type checks."""
    if root_record is None:
        root_record = record
    layout = record["layout"]
    lines = [
        f'_Static_assert(sizeof({layout}) == {record["base_size"]}, "{label} record base size");'
    ]
    for field in record["fields"]:
        member = field["name"]
        path = f"{label}.{member}"
        element = f"(({layout} *)0)->{member}"
        lines.append(
            f'_Static_assert(offsetof({layout}, {member}) == {field["offset"]}, "{path} offset");'
        )
        if "record" in field:
            child = field["record"]
            is_array = "count" in field
            count = field.get("count", 1)
            child_element = f"{element}[0]" if is_array else element
            if is_array:
                lines.append(
                    f"_Static_assert(sizeof({element}) / sizeof({child_element}) == "
                    f'{count}, "{path} nested count");'
                )
                lines.append(
                    f"_Static_assert(_Generic(&{element}, {child['layout']} (*)[{count}]: 1, "
                    f'default: 0), "{path} nested array type");'
                )
            lines.append(
                f"_Static_assert(sizeof({child_element}) == {child['base_size']}, "
                f'"{path} nested size");'
            )
            lines.extend(
                [
                    f"static void __df_check_nested_record_{nested_counter[0]}(",
                    f"    {layout} *parent, {child['layout']} *nested) {{",
                    f"  parent->{member}{'[0]' if is_array else ''} = *nested;",
                    "}",
                ]
            )
            nested_counter[0] += 1
            lines.extend(
                _record_layout_checks(child, fn, path, pointer_counter, nested_counter, root_record)
            )
            continue
        width = (
            ctypes.sizeof(ctypes.c_void_p)
            if "points_to" in field
            else ctypes.sizeof(TYPES[field["type"]][1])
        )
        if "count_from" in field or field.get("count", 1) > 1:
            element += "[0]"
        lines.append(f'_Static_assert(sizeof({element}) == {width}, "{path} element width");')
        if "points_to" in field:
            target_ref = field["points_to"]
            if isinstance(target_ref, dict):
                pointee = next(
                    item
                    for item in root_record.get("pointees", [])
                    if item["name"] == target_ref["pointee"]
                )
                const = "" if pointee["mutable"] else "const "
                target_element = (
                    pointee["record"]["layout"]
                    if "record" in pointee
                    else pointee["bytes"]["c_type"]
                )
                target_c_type = f"{const}{target_element} *"
            else:
                target = next(
                    candidate for candidate in fn["args"] if candidate["name"] == target_ref
                )
                target_c_type = target.get("c_type", POINTERS[target["type"]][0])
            lines.append(
                f"_Static_assert(_Generic(&{element}, {target_c_type} *: 1, default: 0), "
                f'"{path} pointer target type");'
            )
            lines.extend(
                [
                    f"static void __df_check_record_pointer_{pointer_counter[0]}(",
                    f"    {target_c_type} target) {{",
                    f"  (({layout} *)0)->{member} = target;",
                    "}",
                ]
            )
            pointer_counter[0] += 1
    return lines


def emit_wrappers(
    data: dict, rust_source: Path, cases: list[dict] | None = None
) -> tuple[str, str]:
    cases = cases or []
    c = ["#include <stdint.h>", "#include <stddef.h>"]
    if "c_prelude" in data:
        c.append(f"#include {json.dumps(data['c_prelude'])}")
    if data["scope"] == "stateful-host":
        c.extend(_c_fixture_prototype(fixture) for fixture in data["fixtures"])
    c.append(f"#include {json.dumps(data['c_source'])}")
    record_pointer_typecheck = [0]
    nested_record_typecheck = [0]
    for fn in data["functions"]:
        for arg in fn["args"]:
            if "record" not in arg:
                continue
            root_record = arg["record"]
            c.extend(
                _record_layout_checks(
                    root_record,
                    fn,
                    arg["name"],
                    record_pointer_typecheck,
                    nested_record_typecheck,
                    root_record,
                )
            )
            for pointee in root_record.get("pointees", []):
                if "record" not in pointee:
                    continue
                c.extend(
                    _record_layout_checks(
                        pointee["record"],
                        fn,
                        f"{arg['name']}.pointees.{pointee['name']}",
                        record_pointer_typecheck,
                        nested_record_typecheck,
                        root_record,
                    )
                )
    if data["scope"] == "stateful-host":
        c.extend(_c_fixture_runtime(data, cases))
    rust = [
        "#![no_builtins]",
        "#![allow(dead_code, non_snake_case, non_camel_case_types, unused_imports)]",
    ]
    for name, path in data["rust_support"].items():
        rust.append(f"#[path = {json.dumps(path)}] mod {name};")
    rust.append(f"#[path = {json.dumps(str(rust_source))}] mod candidate;")
    for index, fn in enumerate(data["functions"]):
        c_types = [
            TYPES[a["type"]][0]
            if a["type"] in TYPES
            else a.get("c_type", POINTERS[a["type"]][0])
            if a["type"] in POINTERS
            else OUT_POINTERS[a["type"]][0]
            for a in fn["args"]
        ]
        r_types = [
            a["type"]
            if a["type"] in TYPES
            else a.get("rust_type", POINTERS[a["type"]][1])
            if a["type"] in POINTERS
            else OUT_POINTERS[a["type"]][1]
            for a in fn["args"]
        ]
        result = fn["return"]
        if isinstance(result, dict) and result["type"] in ("owned_cstr", "owned_bytes"):
            default_c = "char *" if result["type"] == "owned_cstr" else "void *"
            c_ret = result.get("c_type", "char *")
            if result["type"] == "owned_bytes":
                c_ret = result.get("c_type", default_c)
            r_ret = result.get("rust_type", "*mut u8")
        elif isinstance(result, dict):
            c_ret = result.get("c_type", POINTERS[result["type"]][0])
            r_ret = result.get("rust_type", POINTERS[result["type"]][1])
        else:
            c_ret = "void" if result == "void" else fn.get("c_return_type", TYPES[result][0])
            r_ret = "()" if result == "void" else result
        names = [f"a{i}" for i in range(len(c_types))]
        c_args = ", ".join(f"{t} {n}" for t, n in zip(c_types, names, strict=True)) or "void"
        r_args = ", ".join(f"{n}: {t}" for t, n in zip(r_types, names, strict=True))
        call = ", ".join(names)
        # Assignment, not a cast: the compiler checks the claimed function prototype.
        c.extend(
            [
                f"{c_ret} diff_{index}({c_args}) {{",
                f"  {c_ret} (*checked)({', '.join(c_types) or 'void'}) = &{fn['c']};",
                f"  {'' if c_ret == 'void' else 'return '}checked({call});",
                "}",
            ]
        )
        abi = "" if fn["rust_abi"] == "Rust" else f'extern "{fn["rust_abi"]}" '
        unsafe = "unsafe " if fn["rust_unsafe"] else ""
        rust.extend(
            [
                f'#[no_mangle] pub unsafe extern "C" fn diff_{index}({r_args}) -> {r_ret} {{',
                f"  let checked: {unsafe}{abi}fn({', '.join(r_types)}) -> {r_ret} = {fn['rust']};",
                f"  unsafe {{ checked({call}) }}",
                "}",
            ]
        )
        if (
            isinstance(result, dict)
            and result["type"] == "owned_bytes"
            and result["allocator"] == "custom"
        ):
            c_pointer = result.get("c_type", "void *")
            args_by_name = {arg["name"]: arg for arg in fn["args"]}
            free_c_types = []
            free_r_types = []
            free_names = []
            for free_index, free_arg in enumerate(result.get("free_args", [])):
                if "arg" in free_arg:
                    source = args_by_name[free_arg["arg"]]
                    kind = source["type"]
                    free_c_types.append(
                        TYPES[kind][0] if kind in TYPES else source.get("c_type", POINTERS[kind][0])
                    )
                    free_r_types.append(
                        kind if kind in TYPES else source.get("rust_type", POINTERS[kind][1])
                    )
                    free_names.append(
                        f"a{[arg['name'] for arg in fn['args']].index(free_arg['arg'])}"
                    )
                else:
                    kind = free_arg["type"]
                    free_c_types.append(TYPES[kind][0])
                    free_r_types.append(kind)
                    name = f"free_arg_{free_index}"
                    free_names.append(name)
            c_free_params = ", ".join(
                f"{kind} {name}" for kind, name in zip(free_c_types, free_names, strict=True)
            )
            c_wrapper_params = f"{c_free_params}, " if c_free_params else ""
            c_checked_types = ", ".join([*free_c_types, c_pointer])
            c_checked_args = ", ".join([*free_names, "ptr"])
            c.extend(
                [
                    f"void diff_free_{index}({c_wrapper_params}{c_pointer} ptr) {{",
                    f"  void (*checked)({c_checked_types}) = &{result['c_free']};",
                    f"  checked({c_checked_args});",
                    "}",
                ]
            )
            free_abi = "" if result["free_abi"] == "Rust" else f'extern "{result["free_abi"]}" '
            r_pointer = result.get("rust_type", "*mut u8")
            rust_free_args = ", ".join(
                f"{name}: {kind}" for name, kind in zip(free_names, free_r_types, strict=True)
            )
            rust_free_signature = f"{rust_free_args}, " if rust_free_args else ""
            rust_checked_types = ", ".join([*free_r_types, r_pointer])
            rust_checked_args = ", ".join([*free_names, "ptr"])
            rust_checked_decl = (
                f"  let checked: unsafe {free_abi}fn({rust_checked_types}) = {result['rust_free']};"
            )
            rust.extend(
                [
                    f'#[no_mangle] pub unsafe extern "C" fn diff_free_{index}(',
                    f"    {rust_free_signature}ptr: {r_pointer},",
                    ") {",
                    rust_checked_decl,
                    f"  unsafe {{ checked({rust_checked_args}) }}",
                    "}",
                ]
            )
    if data["scope"] == "stateful-host":
        rust.extend(_rust_fixture_runtime(data, cases))
    return "\n".join(c) + "\n", "\n".join(rust) + "\n"


class Allocation:
    """One owned allocation, with either canaries or inaccessible page boundaries."""

    def __init__(self, initial: bytes, placement: str, alignment: int = 1):
        self.initial = initial
        self.mapping = None
        self.storage = None
        if placement == "canary":
            self.storage = (ctypes.c_uint8 * (len(initial) + 2 * GUARD))()
            self.region = ctypes.addressof(self.storage)
            self.region_size = len(self.storage)
            self.address = self.region + GUARD
        else:
            if not sys.platform.startswith("linux"):
                raise ContractError("guard-pages requires Linux mmap/mprotect")
            page = mmap.PAGESIZE
            accessible = max(page, ((len(initial) + page - 1) // page) * page)
            self.mapping = mmap.mmap(
                -1, accessible + 2 * page, prot=mmap.PROT_READ | mmap.PROT_WRITE
            )
            base = ctypes.addressof(ctypes.c_uint8.from_buffer(self.mapping))
            libc = ctypes.CDLL(None, use_errno=True)
            libc.mprotect.argtypes = [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_int]
            libc.mprotect.restype = ctypes.c_int
            for address in (base, base + page + accessible):
                if libc.mprotect(address, page, 0):
                    error = ctypes.get_errno()
                    self.mapping.close()
                    raise ContractError(f"mprotect failed: errno {error}")
            self.region, self.region_size = base + page, accessible
            # Right placement ends flush with the guard page unless that would misalign
            # the payload; then at most alignment-1 canary bytes sit before the guard.
            right = (self.region + accessible - len(initial)) & ~(alignment - 1)
            self.address = self.region if placement == "left" else right
        ctypes.memset(self.region, SENTINEL, self.region_size)
        ctypes.memmove(self.address, initial, len(initial))

    def observe(self, writable: bytearray) -> tuple[str, list[str]]:
        content = ctypes.string_at(self.region, self.region_size)
        offset = self.address - self.region
        payload = content[offset : offset + len(self.initial)]
        violations = []
        if content[:offset] != bytes([SENTINEL]) * offset or content[
            offset + len(payload) :
        ] != bytes([SENTINEL]) * (len(content) - offset - len(payload)):
            violations.append("guard overwritten")
        if any(
            a != b and not writable[i]
            for i, (a, b) in enumerate(zip(self.initial, payload, strict=True))
        ):
            violations.append("bytes outside writable views modified")
        return payload.hex(), violations

    def close(self) -> None:
        if self.mapping is not None:
            self.mapping.close()


def patch_record_pointers(case: dict, memory: dict[str, Allocation]) -> None:
    """Relocate declared record pointers to this worker's generated allocations."""
    width = ctypes.sizeof(ctypes.c_void_p)
    for field in case.get("pointer_fields", []):
        source_binding = field.get("source_binding", field["record_arg"])
        record_binding = case["bindings"][source_binding]
        target_binding = case["bindings"][field["target_arg"]]
        if record_binding.get("null", False) or target_binding.get("null", False):
            raise ContractError("record pointer relocation requires non-null bindings")
        record = memory[record_binding["allocation"]]
        target = memory[target_binding["allocation"]]
        pointer = target.address + target_binding["offset"]
        offset = record_binding["offset"] + field["offset"]
        if offset < 0 or offset + width > len(record.initial):
            raise ContractError("record pointer relocation is outside its allocation")
        ctypes.memmove(record.address + offset, ctypes.byref(ctypes.c_void_p(pointer)), width)
        initial = bytearray(record.initial)
        initial[offset : offset + width] = pointer.to_bytes(width, sys.byteorder)
        record.initial = bytes(initial)


def normalized_pointer(
    value: int | None, rule: dict, case: dict, memory: dict[str, Allocation]
) -> tuple[dict, list[str]]:
    if value is None:
        return {"null": True}, [] if rule["nullable"] else ["unexpected null return"]
    for name in rule["within"]:
        binding = case["bindings"][name]
        if binding.get("null", False):
            continue
        allocation = memory[binding["allocation"]]
        offset = value - allocation.address
        start, length = binding["offset"], binding["length"]
        if start <= offset < start + length or (rule["one_past"] and offset == start + length):
            return {"allocation": binding["allocation"], "offset": offset}, []
    # Never dereference or leak process-specific invalid pointer values into comparisons.
    return {"invalid": True}, ["returned pointer outside declared views"]


def _stateful_worker_case(
    data: dict, dll: ctypes.CDLL, funcs: list, case: dict, case_index: int
) -> None:
    fixtures = data["fixtures"]
    violations = []
    if fixtures:
        reset = dll.diff_fixture_reset
        reset.argtypes = [ctypes.c_uint32]
        reset.restype = None
        reset(case_index)
        event_count = dll.diff_fixture_event_count
        event_count.argtypes = []
        event_count.restype = ctypes.c_uint32
        event_id = dll.diff_fixture_event_id
        event_id.argtypes = [ctypes.c_uint32]
        event_id.restype = ctypes.c_uint32
        event_argc = dll.diff_fixture_event_argc
        event_argc.argtypes = [ctypes.c_uint32]
        event_argc.restype = ctypes.c_uint32
        event_arg = dll.diff_fixture_event_arg
        event_arg.argtypes = [ctypes.c_uint32, ctypes.c_uint32]
        event_arg.restype = ctypes.c_uint64
        event_result = dll.diff_fixture_event_result
        event_result.argtypes = [ctypes.c_uint32]
        event_result.restype = ctypes.c_uint64
        finish = dll.diff_fixture_finish
        finish.argtypes = []
        finish.restype = ctypes.c_uint32
    observations = []
    observed_events = 0
    for step in case["steps"]:
        fn = data["functions"][step["function"]]
        args = [step["args"][arg["name"]] for arg in fn["args"]]
        result = funcs[step["function"]](*args)
        step_fixtures = []
        if fixtures:
            count = int(event_count())
            if count < observed_events or count > MAX_FIXTURE_EVENTS:
                violations.append("fixture event count is invalid")
                count = min(max(count, observed_events), MAX_FIXTURE_EVENTS)
            for event_index in range(observed_events, count):
                fixture_index = int(event_id(event_index))
                if fixture_index >= len(fixtures):
                    violations.append("fixture event names an unknown fixture")
                    continue
                fixture = fixtures[fixture_index]
                argc = int(event_argc(event_index))
                if argc != len(fixture["args"]):
                    violations.append("fixture event argument count is invalid")
                    continue
                actual_args = [
                    _from_u64_bits(int(event_arg(event_index, arg_index)), arg["type"])
                    for arg_index, arg in enumerate(fixture["args"])
                ]
                actual_result = (
                    None
                    if fixture["return"] == "void"
                    else _from_u64_bits(int(event_result(event_index)), fixture["return"])
                )
                step_fixtures.append(
                    {
                        "fixture": fixture["name"],
                        "args": actual_args,
                        "return": actual_result,
                    }
                )
            observed_events = count
        observations.append(
            {
                "function": step["function"],
                "return": result,
                "fixture_calls": step_fixtures,
            }
        )
    if fixtures and finish():
        violations.append("external fixture arguments or call counts violated the script")
    print(
        json.dumps(
            {
                "case": case_index,
                "steps": observations,
                "violations": violations,
            }
        ),
        flush=True,
    )


def worker(contract: Path, corpus: Path, library: Path, case_index: int | None = None) -> None:
    """Private subprocess entry: crashes cannot kill the grading coordinator."""
    import resource

    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    data = json.loads(contract.read_text())
    dll = ctypes.CDLL(str(library))
    has_libc_owned = any(
        isinstance(fn["return"], dict)
        and (
            fn["return"]["type"] == "owned_cstr"
            or (fn["return"]["type"] == "owned_bytes" and fn["return"]["allocator"] == "libc")
        )
        for fn in data["functions"]
    )
    host_libc = ctypes.CDLL(None) if has_libc_owned else None
    host_free = host_libc.free if host_libc is not None else None
    if host_free is not None:
        host_free.argtypes = [ctypes.c_void_p]
        host_free.restype = None
    funcs = []
    owned_releasers = []
    for index, fn in enumerate(data["functions"]):
        func = getattr(dll, f"diff_{index}")
        func.argtypes = [
            TYPES[a["type"]][1] if a["type"] in TYPES else ctypes.c_void_p for a in fn["args"]
        ]
        result = fn["return"]
        func.restype = (
            ctypes.c_void_p
            if isinstance(result, dict)
            else None
            if result == "void"
            else TYPES[result][1]
        )
        funcs.append(func)
        result = fn["return"]
        if (
            isinstance(result, dict)
            and result["type"] == "owned_bytes"
            and result["allocator"] == "custom"
        ):
            releaser = getattr(dll, f"diff_free_{index}")
            args_by_name = {arg["name"]: arg for arg in fn["args"]}
            context_types = []
            for free_arg in result.get("free_args", []):
                if "arg" in free_arg:
                    source = args_by_name[free_arg["arg"]]
                    context_types.append(
                        TYPES[source["type"]][1] if source["type"] in TYPES else ctypes.c_void_p
                    )
                else:
                    context_types.append(TYPES[free_arg["type"]][1])
            releaser.argtypes = [*context_types, ctypes.c_void_p]
            releaser.restype = None
            owned_releasers.append(releaser)
        else:
            owned_releasers.append(None)
    corpus_rows = json.loads(corpus.read_text())
    if data["scope"] == "stateful-host":
        if case_index is None or not 0 <= case_index < len(corpus_rows):
            raise ContractError("stateful worker requires one valid isolated case index")
        _stateful_worker_case(data, dll, funcs, corpus_rows[case_index], case_index)
        return
    for index, case in enumerate(corpus_rows):
        fn = data["functions"][case["function"]]
        memory = {
            name: Allocation(
                bytes.fromhex(payload),
                case["placement"],
                case.get("alignments", {}).get(name, 1),
            )
            for name, payload in case["allocations"].items()
        }
        try:
            patch_record_pointers(case, memory)
            writable = {name: bytearray(len(obj.initial)) for name, obj in memory.items()}
            args = []
            args_by_name = {}
            pointer_slots = {}
            for arg in fn["args"]:
                name = arg["name"]
                if arg["type"] in TYPES:
                    value = case["args"][name]
                    args.append(value)
                    args_by_name[name] = value
                elif arg["type"] in OUT_POINTERS:
                    if case.get("nulls", {}).get(name, False):
                        args.append(None)
                    else:
                        slot = ctypes.c_void_p()
                        pointer_slots[name] = slot
                        args.append(ctypes.byref(slot))
                    args_by_name[name] = args[-1]
                else:
                    if case.get("nulls", {}).get(name, False):
                        args.append(None)
                        args_by_name[name] = None
                        continue
                    binding = case["bindings"][name]
                    obj = memory[binding["allocation"]]
                    value = ctypes.c_void_p(obj.address + binding["offset"])
                    args.append(value)
                    args_by_name[name] = value
                    if not binding["readonly"]:
                        start, length = binding["offset"], binding["length"]
                        writable[binding["allocation"]][start : start + length] = (
                            bytes([1]) * length
                        )
            for name, binding in case["bindings"].items():
                if name.startswith("@") and not binding["readonly"]:
                    start, length = binding["offset"], binding["length"]
                    writable[binding["allocation"]][start : start + length] = bytes([1]) * length
            # These markers distinguish a native-call fault from setup/observation failures.
            print(json.dumps({"event": "call", "case": index}), flush=True)
            result = funcs[case["function"]](*args)
            print(json.dumps({"event": "returned", "case": index}), flush=True)
            violations = []
            if isinstance(fn["return"], dict) and fn["return"]["type"] == "owned_cstr":
                if result is None:
                    result = {"null": True}
                    if not fn["return"]["nullable"]:
                        violations.append("unexpected null owned C-string")
                else:
                    pointer = result
                    if host_free is None:
                        raise ContractError("owned_cstr requires host libc free")
                    payload = bytearray()
                    try:
                        for offset in range(fn["return"]["max_bytes"]):
                            byte = ctypes.c_ubyte.from_address(pointer + offset).value
                            payload.append(byte)
                            if byte == 0:
                                break
                        else:
                            violations.append("owned C-string missing NUL within max_bytes")
                    finally:
                        host_free(ctypes.c_void_p(pointer))
                    result = {"bytes": payload.hex()}
            elif isinstance(fn["return"], dict) and fn["return"]["type"] == "owned_bytes":
                if result is None:
                    result = {"null": True}
                    if not fn["return"]["nullable"]:
                        violations.append("unexpected null owned byte result")
                else:
                    pointer = result
                    length_spec = fn["return"]["length"]
                    length = (
                        case["args"][length_spec] if isinstance(length_spec, str) else length_spec
                    )
                    if fn["return"]["allocator"] == "libc":
                        if host_free is None:
                            raise ContractError("owned_bytes requires host libc free")
                        release = host_free
                    else:
                        release = owned_releasers[case["function"]]
                        if release is None:
                            raise ContractError("owned_bytes custom deallocator wrapper is missing")
                        release_args = [
                            args_by_name[free_arg["arg"]]
                            if "arg" in free_arg
                            else free_arg["value"]
                            for free_arg in fn["return"].get("free_args", [])
                        ]
                    try:
                        payload = ctypes.string_at(pointer, length)
                    finally:
                        if fn["return"]["allocator"] == "custom":
                            release(*release_args, ctypes.c_void_p(pointer))
                        else:
                            release(ctypes.c_void_p(pointer))
                    result = {"bytes": payload.hex()}
            elif isinstance(fn["return"], dict):
                result, violations = normalized_pointer(result, fn["return"], case, memory)
            pointer_outputs = {}
            for arg in fn["args"]:
                if arg["type"] not in OUT_POINTERS:
                    continue
                name = arg["name"]
                if name not in pointer_slots:
                    pointer_outputs[name] = {"argument_null": True}
                    continue
                pointer_outputs[name], problems = normalized_pointer(
                    pointer_slots[name].value,
                    {
                        "within": arg["within"],
                        "nullable": False,
                        "one_past": arg["one_past"],
                    },
                    case,
                    memory,
                )
                violations.extend(f"{name}: {problem}" for problem in problems)
            buffers = {}
            for name, obj in memory.items():
                payload, problems = obj.observe(writable[name])
                buffers[name] = payload
                violations.extend(f"{name}: {problem}" for problem in problems)
            pointer_field_outputs = {}
            for field in case.get("pointer_fields", []):
                source_binding = field.get("source_binding", field["record_arg"])
                record_binding = case["bindings"][source_binding]
                record = memory[record_binding["allocation"]]
                offset = record_binding["offset"] + field["offset"]
                value = ctypes.c_void_p.from_address(record.address + offset).value
                normalized, problems = normalized_pointer(
                    value,
                    {
                        "within": [field["target_arg"]],
                        "nullable": field["nullable"],
                        "one_past": field["one_past"],
                    },
                    case,
                    memory,
                )
                pointer_field_outputs[f"{field['record_arg']}.{field['field']}"] = normalized
                violations.extend(
                    f"{field['record_arg']}.{field['field']}: {problem}" for problem in problems
                )
                payload = bytearray.fromhex(buffers[record_binding["allocation"]])
                payload[offset : offset + ctypes.sizeof(ctypes.c_void_p)] = bytes(
                    ctypes.sizeof(ctypes.c_void_p)
                )
                buffers[record_binding["allocation"]] = payload.hex()
            observation = {
                "case": index,
                "return": result,
                "buffers": buffers,
                "violations": violations,
            }
            if pointer_outputs:
                observation["pointer_outputs"] = pointer_outputs
            if pointer_field_outputs:
                observation["pointer_fields"] = pointer_field_outputs
            print(json.dumps(observation), flush=True)
        finally:
            for obj in memory.values():
                obj.close()


def apply_mutant(source: str, mutant: dict) -> str:
    """Apply one exact-text mutant, tolerant only of whitespace reflow.

    A unique exact match wins, so authors may still use indentation to pick between
    otherwise identical sites. Only when the exact text is absent do the tokens match
    across any whitespace, so a rustfmt reflow of the shipping source does not break
    every contract that mutates it; that match must still be unique.
    """
    exact = source.count(mutant["find"])
    if exact == 1:
        return source.replace(mutant["find"], mutant["replace"], 1)
    if exact > 1:
        raise ContractError(
            f"mutant {mutant['name']}: replacement must match exactly once (found {exact})"
        )
    tokens = mutant["find"].split()
    if not tokens:
        raise ContractError(f"mutant {mutant['name']}: empty find text")
    pattern = re.compile(r"\s+".join(map(re.escape, tokens)))
    matches = list(pattern.finditer(source))
    if len(matches) != 1:
        raise ContractError(
            f"mutant {mutant['name']}: replacement must match exactly once "
            f"(found {len(matches)}, whitespace-insensitive)"
        )
    start, end = matches[0].span()
    return source[:start] + mutant["replace"].strip() + source[end:]


def _command(
    argv: list[str], directory: Path, label: str, timeout: int = 120
) -> subprocess.CompletedProcess:
    (directory / f"{label}.command.json").write_text(json.dumps(argv, indent=2) + "\n")
    start = time.monotonic()
    timed_out = False
    try:
        result = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired as exc:
        timed_out = True

        def decoded(value):
            return value.decode(errors="replace") if isinstance(value, bytes) else (value or "")

        result = subprocess.CompletedProcess(argv, 124, decoded(exc.stdout), decoded(exc.stderr))
    (directory / f"{label}.result.json").write_text(
        json.dumps(
            {
                "exit_code": result.returncode,
                "timed_out": timed_out,
                "elapsed_seconds": time.monotonic() - start,
            }
        )
        + "\n"
    )
    (directory / f"{label}.stdout").write_text(result.stdout)
    (directory / f"{label}.stderr").write_text(result.stderr)
    return result


def _execute_stateful(directory: Path, library: Path, label: str, count: int) -> list[dict]:
    corpus = json.loads((directory / "cases.json").read_text())
    if not isinstance(corpus, list) or len(corpus) != count:
        actual = len(corpus) if isinstance(corpus, list) else "non-array"
        raise ContractError(f"{label}: expected {count} stateful cases, got {actual}")
    contract_path = directory / "contract.json"
    case_dir = directory / f"{label}.cases"
    case_dir.mkdir(exist_ok=False)
    rows = []
    start = time.monotonic()
    for index, case in enumerate(corpus):
        result = _command(
            [
                sys.executable,
                str(Path(__file__).resolve()),
                "--worker",
                str(contract_path),
                str(directory / "cases.json"),
                str(library),
                "--case-index",
                str(index),
            ],
            case_dir,
            f"case-{index:04d}",
        )
        if result.returncode:
            (directory / f"{label}.fault.json").write_text(
                json.dumps(
                    {
                        "case": index,
                        "exit_code": result.returncode,
                        "sequence": case["name"],
                    },
                    indent=2,
                )
                + "\n"
            )
            raise ContractError(
                f"{label}: stateful case {index} exited {result.returncode}; see case logs"
            )
        try:
            emitted = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
        except json.JSONDecodeError as exc:
            raise ContractError(
                f"{label}: invalid stateful worker output for case {index}"
            ) from exc
        if len(emitted) != 1:
            raise ContractError(f"{label}: stateful worker emitted the wrong row count")
        row = emitted[0]
        if (
            set(row) != {"case", "steps", "violations"}
            or row["case"] != index
            or not isinstance(row["steps"], list)
            or len(row["steps"]) != len(case["steps"])
            or not isinstance(row["violations"], list)
        ):
            raise ContractError(f"{label}: invalid stateful worker row for case {index}")
        for actual, expected in zip(row["steps"], case["steps"], strict=True):
            if (
                set(actual) != {"function", "return", "fixture_calls"}
                or actual["function"] != expected["function"]
                or not isinstance(actual["fixture_calls"], list)
            ):
                raise ContractError(f"{label}: invalid operation observation in case {index}")
        rows.append(row)
    (directory / f"{label}.stdout").write_text(
        "".join(json.dumps(row, sort_keys=True) + "\n" for row in rows)
    )
    (directory / f"{label}.result.json").write_text(
        json.dumps(
            {
                "exit_code": 0,
                "timed_out": False,
                "cases": len(rows),
                "elapsed_seconds": time.monotonic() - start,
            },
            indent=2,
        )
        + "\n"
    )
    (directory / f"{label}.stderr").write_text("")
    return rows


def _execute(
    directory: Path, library: Path, label: str, count: int, allow_memory_fault: bool = False
) -> list[dict]:
    contract_path = directory / "contract.json"
    contract = json.loads(contract_path.read_text()) if contract_path.is_file() else None
    if contract is not None and contract.get("scope") == "stateful-host":
        if allow_memory_fault:
            raise ContractError("stateful-host does not support memory-fault mutants")
        return _execute_stateful(directory, library, label, count)
    corpus_path = directory / "cases.json"
    corpus_rows = (
        json.loads(corpus_path.read_text()) if corpus_path.is_file() else [{} for _ in range(count)]
    )
    result = _command(
        [
            sys.executable,
            str(Path(__file__).resolve()),
            "--worker",
            str(directory / "contract.json"),
            str(directory / "cases.json"),
            str(library),
        ],
        directory,
        label,
    )
    rows: list[dict] = []
    phase = "ready"
    try:
        for line in result.stdout.splitlines():
            row = json.loads(line)
            if not isinstance(row, dict) or row.get("case") != len(rows) or len(rows) >= count:
                raise ContractError(f"{label}: unexpected case/protocol output")
            event = row.get("event")
            expected = {"case", "return", "buffers", "violations"}
            outptrs = set(corpus_rows[len(rows)].get("outptrs", []))
            pointer_fields = {
                f"{field['record_arg']}.{field['field']}"
                for field in corpus_rows[len(rows)].get("pointer_fields", [])
            }
            if outptrs:
                expected.add("pointer_outputs")
            if pointer_fields:
                expected.add("pointer_fields")
            if event == "call" and phase == "ready":
                phase = "calling"
            elif event == "returned" and phase == "calling":
                phase = "observing"
            elif (
                event is None
                and phase == "observing"
                and set(row) == expected
                and set(row.get("pointer_outputs", {})) == outptrs
                and set(row.get("pointer_fields", {})) == pointer_fields
            ):
                rows.append(row)
                phase = "ready"
            else:
                raise ContractError(f"{label}: invalid worker event sequence")
    except json.JSONDecodeError as exc:
        raise ContractError(f"{label}: invalid worker output") from exc
    if result.returncode:
        fault = {
            "case": len(rows),
            "signal": -result.returncode if result.returncode < 0 else None,
            "exit_code": result.returncode,
            "phase": phase,
        }
        (directory / f"{label}.fault.json").write_text(json.dumps(fault, indent=2) + "\n")
        if (
            allow_memory_fault
            and phase == "calling"
            and result.returncode in (-signal.SIGSEGV, -signal.SIGBUS)
        ):
            corpus = json.loads((directory / "cases.json").read_text())
            if corpus[len(rows)]["placement"] not in ("left", "right"):
                raise ContractError(f"{label}: memory fault occurred outside protected-page case")
            return [*rows, {"case": len(rows), "fault": fault}]
        raise ContractError(f"{label}: execution failed ({result.returncode}); see {label}.stderr")
    if len(rows) != count or phase != "ready":
        raise ContractError(f"{label}: missing/extra cases")
    return rows


def _dependencies(path: Path) -> set[Path]:
    # Both compilers emit make-style depfiles. Backslash escapes spaces and continuations.
    text = path.read_text().replace("\\\n", " ")
    first = text.splitlines()[0].split(": ", 1)
    if len(first) != 2:
        raise ContractError(f"invalid compiler dependency file: {path}")
    tokens = re.findall(r"(?:\\.|[^\s])+", first[1])
    return {Path(re.sub(r"\\(.)", r"\1", token)).resolve(strict=True) for token in tokens}


class _CandidateCrashed(Exception):
    """Control flow only: the candidate crash is already recorded as the mismatch."""


def _candidate_crash(
    output: Path, cases: list[dict], reference: list[dict], failure: ContractError
) -> dict:
    """A Rust fault on an input the clean C reference handled IS a mismatch, not an
    invalid run. Only a fault inside the native call counts, and only when a second
    independent process faults on the same case; anything else re-raises `failure`.
    """
    fault_path = output / "candidate.fault.json"
    if not fault_path.is_file():
        raise failure
    fault = json.loads(fault_path.read_text())
    if fault["phase"] != "calling":
        raise failure
    try:
        _execute(output, output / "candidate.so", "candidate-repeat", len(cases))
    except ContractError:
        repeat_path = output / "candidate-repeat.fault.json"
        if not repeat_path.is_file():
            raise
        repeat = json.loads(repeat_path.read_text())
        if (repeat["case"], repeat["phase"], repeat["exit_code"]) != (
            fault["case"],
            fault["phase"],
            fault["exit_code"],
        ):
            raise ContractError("Rust candidate faults unstably across isolated runs") from None
    else:
        raise ContractError("Rust candidate faulted once and then completed") from None
    index = fault["case"]
    (output / "counterexample.json").write_text(
        json.dumps(
            {"input": cases[index], "reference": reference[index], "candidate": {"fault": fault}},
            indent=2,
        )
        + "\n"
    )
    return {
        "mismatches": 1,
        "candidate_fault": fault,
        "mutant_error": "not run: the unmodified candidate faults on a valid input",
    }


def grade(path: Path, output: Path) -> dict:
    """Write a fresh durable run; invalid/unsupported runs never become passes."""
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    report: dict = {
        "schema": 16,
        "status": "invalid",
        "cost_usd": 0,
        "scope": "unvalidated",
        "limitations": [
            "Reviewed contract validity is assumed, not proved.",
            "Sampled inputs; no branch-coverage or equivalence claim.",
            "Host ABI only; no guest/architecture or concurrency coverage.",
            "Guard pages check allocation edges, not every subview bound or undefined behavior.",
            "Nullable byte/string inputs, C-string output slots and bounded owned string/byte "
            "results have explicit contracts; unmodeled ownership and external effects remain "
            "outside the grade.",
            "Stateful sequences reset each case in a new process and check only declared scalar "
            "fixture calls; they do not explore concurrent interleavings.",
            "Generated string inputs follow a reviewed bounded grammar; grammar validity and "
            "semantic completeness are not proved.",
            "Structured records assert the declared C base size, field offsets, element widths "
            "and fixed inline-array counts; declared pointer fields relocate to named arguments "
            "and compare by allocation offset. Field values and bounded trailing arrays are "
            "sampled; pointer graphs are not inferred.",
        ],
    }
    try:
        data = load_contract(path.resolve())
        report["scope"] = data["scope"]
        cases = generate_contract_cases(data)
        (output / "contract.json").write_text(json.dumps(data, indent=2) + "\n")
        (output / "cases.json").write_text(json.dumps(cases, indent=2) + "\n")
        report.update(
            {
                "contract_sha256": digest(path),
                "seed": data["seed"],
                "cases": len(cases),
                "functions": [f["c"] for f in data["functions"]],
                "validity": data["validity"],
                "mutants": [],
                "memory": data["memory"],
                "c_allowed_ub": data["c_allow_ub"],
            }
        )
        if data["c_allow_ub"]:
            report["limitations"].append(
                "The C reference is compiled without UBSan checks for: "
                + ", ".join(data["c_allow_ub"])
                + "; its behavior there is the host compiler's, not the C standard's."
            )
        if data["scope"] == "stateful-host":
            report["operations"] = sum(len(case["steps"]) for case in cases)
        original_source = Path(data["rust_source"])
        rust_source = original_source
        original_hashes = {str(original_source): digest(original_source)}
        for helper in data["rust_helpers"]:
            original_hashes[helper["source"]] = digest(Path(helper["source"]))
        source = original_source.read_text()
        if "rust_items" in data:
            source = selected_rust(source, data["rust_items"], preserve_cfg=True)
            for helper in data["rust_helpers"]:
                source += selected_rust(
                    Path(helper["source"]).read_text(), helper["items"], preserve_cfg=True
                )
            if data["rust_imports"]:
                source = "".join(f"use {item};\n" for item in data["rust_imports"]) + source
            rust_source = output / "selected.rs"
            rust_source.write_text(source)
            report["selected_rust_sha256"] = digest(rust_source)
        if data["fixtures"]:
            source = f"{source}\n{_rust_fixture_type_checks(data)}\n"
            rust_source = output / "candidate_contract.rs"
            rust_source.write_text(source)
            report["selected_rust_sha256"] = digest(rust_source)
        c, rust = emit_wrappers(data, rust_source, cases)
        (output / "reference.c").write_text(c)
        (output / "candidate.rs").write_text(rust)
        cc, rustc = shutil.which("cc"), shutil.which("rustc")
        if not cc or not rustc:
            raise ContractError("cc and rustc must be available")
        report["compilers"] = {}
        for name, executable in (("cc", cc), ("rustc", rustc)):
            version = _command([executable, "--version"], output, f"{name}-version")
            if version.returncode:
                raise ContractError(f"cannot identify {name}")
            report["compilers"][name] = {
                "path": executable,
                "sha256": digest(Path(executable)),
                "version": version.stdout,
            }
        sysroot = _command([rustc, "--print", "sysroot"], output, "rustc-sysroot")
        real_rustc = Path(sysroot.stdout.strip()) / "bin/rustc"
        if sysroot.returncode or not real_rustc.is_file():
            raise ContractError("cannot identify actual Rust toolchain compiler")
        report["compilers"]["rustc"]["toolchain_path"] = str(real_rustc)
        report["compilers"]["rustc"]["toolchain_sha256"] = digest(real_rustc)
        report["grader_sha256"] = digest(Path(__file__))
        explicit = {
            Path(data["c_source"]),
            Path(data["rust_source"]),
            *map(Path, data["rust_support"].values()),
            *(Path(h["source"]) for h in data["rust_helpers"]),
        }
        if "c_prelude" in data:
            explicit.add(Path(data["c_prelude"]))
        c_command = [
            cc,
            "-std=c11",
            "-shared",
            "-fPIC",
            "-O2",
            *data["c_flags"],
            *(flag for p in data["c_include_dirs"] for flag in ("-I", p)),
            "-fno-builtin",
            "-Wl,-Bsymbolic",
            "-fsanitize=undefined",
            *(f"-fno-sanitize={check}" for check in data["c_allow_ub"]),
            "-fno-sanitize-recover=all",
            "-Wl,-z,defs",
            "-Werror=strict-prototypes",
            "-Werror=old-style-definition",
            "-Werror=incompatible-pointer-types",
            "-Werror=implicit-function-declaration",
            "-MD",
            "-MF",
            str(output / "reference.d"),
            str(output / "reference.c"),
            "-o",
            str(output / "reference.so"),
        ]
        rust_command = [
            rustc,
            "--edition=2021",
            "--crate-type=cdylib",
            "--crate-name=contract_candidate",
            "-C",
            "link-arg=-Wl,-Bsymbolic",
            "-O",
            *data["rust_flags"],
            f"--emit=link,dep-info={output / 'candidate.d'}",
            str(output / "candidate.rs"),
            "-o",
            str(output / "candidate.so"),
        ]
        # Discover compiler-resolved includes BEFORE linking, then check their bytes
        # after linking and again after execution. Include system headers too.
        c_deps = [*c_command[:-2], "-M", "-MF", str(output / "reference.d")]
        rust_deps = [a.replace("--emit=link,dep-info=", "--emit=dep-info=") for a in rust_command]
        for label, command in (("deps-c", c_deps), ("deps-rust", rust_deps)):
            result = _command(command, output, label)
            if result.returncode:
                raise ContractError(f"{label} failed; see {label}.stderr")
        dependencies = (
            _dependencies(output / "reference.d")
            | _dependencies(output / "candidate.d")
            | explicit
            | {path.resolve(), Path(__file__).resolve(), output / "cases.json"}
        )
        before = {str(p): digest(p) for p in dependencies}
        if any(before[p] != h for p, h in original_hashes.items()):
            raise ContractError("Rust source changed while selecting/generating wrappers")
        for label, command in (("build-c", c_command), ("build-rust", rust_command)):
            result = _command(command, output, label)
            if result.returncode:
                raise ContractError(f"{label} failed; see {label}.stderr")
        if (
            not (_dependencies(output / "reference.d") | _dependencies(output / "candidate.d"))
            <= dependencies
        ):
            raise ContractError("compiler dependency set changed during compilation")
        pins = {str(p): digest(p) for p in dependencies}
        if any(pins.get(p) != value for p, value in before.items()):
            raise ContractError("input changed during compilation")
        report["inputs"] = pins
        report["libraries"] = {
            name: digest(output / f"{name}.so") for name in ("reference", "candidate")
        }
        reference = _execute(output, output / "reference.so", "reference", len(cases))
        again = _execute(output, output / "reference.so", "reference-repeat", len(cases))
        if reference != again or any(row["violations"] for row in reference):
            raise ContractError("C reference is unstable or violates the input contract/guards")
        try:
            candidate = _execute(output, output / "candidate.so", "candidate", len(cases))
        except ContractError as failure:
            report.update(_candidate_crash(output, cases, reference, failure))
            report["status"] = "mismatch"
            raise _CandidateCrashed() from None
        repeated = _execute(output, output / "candidate.so", "candidate-repeat", len(cases))
        if candidate != repeated:
            raise ContractError("Rust candidate is unstable across isolated runs")
        mismatches = [
            i for i, (a, b) in enumerate(zip(reference, candidate, strict=True)) if a != b
        ]
        report["mismatches"] = len(mismatches)
        if mismatches:
            # A mismatch against a clean, stable reference is a finding on its own; the
            # mutant phase below only decides whether a PASS is trustworthy, so a later
            # failure there must not bury it under "invalid".
            if any(digest(Path(p)) != value for p, value in pins.items()):
                raise ContractError("input changed during grading")
            report["status"] = "mismatch"
            i = mismatches[0]
            (output / "counterexample.json").write_text(
                json.dumps(
                    {"input": cases[i], "reference": reference[i], "candidate": candidate[i]},
                    indent=2,
                )
                + "\n"
            )
        for mutant in data["mutants"]:
            mutant_source = output / f"mutant_{mutant['name']}.rs"
            mutant_data = data
            if "module" in mutant:
                module_path = Path(data["rust_support"][mutant["module"]])
                mutant_source.write_text(apply_mutant(module_path.read_text(), mutant))
                mutant_data = {
                    **data,
                    "rust_support": {**data["rust_support"], mutant["module"]: str(mutant_source)},
                }
                _, wrapper = emit_wrappers(mutant_data, rust_source, cases)
            else:
                mutant_source.write_text(apply_mutant(source, mutant))
                _, wrapper = emit_wrappers(data, mutant_source, cases)
            wrapper_path = output / f"wrapper_{mutant['name']}.rs"
            wrapper_path.write_text(wrapper)
            library = output / f"mutant_{mutant['name']}.so"
            command = [
                rustc,
                "--edition=2021",
                "--crate-type=cdylib",
                "--crate-name=contract_mutant",
                "-C",
                "link-arg=-Wl,-Bsymbolic",
                "-O",
                *data["rust_flags"],
                str(wrapper_path),
                "-o",
                str(library),
            ]
            label = f"mutant_{mutant['name']}"
            result = _command(command, output, f"build-{label}")
            if result.returncode:
                raise ContractError(f"{label}: mutant did not compile; not evidence of sensitivity")
            expects_fault = mutant["expect"] == "memory_fault"
            rows = _execute(output, library, label, len(cases), allow_memory_fault=expects_fault)
            fault = rows[-1].get("fault")
            if expects_fault:
                repeat = _execute(
                    output, library, label + "-repeat", len(cases), allow_memory_fault=True
                )
                if repeat != rows:
                    raise ContractError(f"{label}: unstable memory fault")
                differences = [rows[-1]["case"]] if fault else []
            else:
                differences = [
                    i for i, (a, b) in enumerate(zip(candidate, rows, strict=True)) if a != b
                ]
            report["mutants"].append(
                {
                    "name": mutant["name"],
                    "detected": bool(differences),
                    "expect": mutant["expect"],
                    "fault": fault,
                    "completed_cases": len(rows) - (1 if fault else 0),
                    "mismatches": len(differences),
                    "functions": sorted(
                        {
                            data["functions"][function_index]["c"]
                            for i in differences
                            for function_index in case_mismatch_functions(
                                cases[i], candidate[i], rows[i]
                            )
                        }
                    ),
                    "first_input": cases[differences[0]] if differences else None,
                    "library_sha256": digest(library),
                }
            )
        if any(digest(Path(p)) != value for p, value in pins.items()):
            raise ContractError("input changed during grading")
        if any(not m["detected"] for m in report["mutants"]):
            raise ContractError("a sensitivity mutant survived; harness is not accepted")
        sensitive = {f for mutant in report["mutants"] for f in mutant["functions"]}
        if sensitive != set(report["functions"]):
            missing = sorted(set(report["functions"]) - sensitive)
            raise ContractError(f"no sensitivity control detected for: {missing}")
        per_function = []
        for index, fn in enumerate(data["functions"]):
            hit_cases = [i for i, case in enumerate(cases) if index in case_functions(case)]
            mismatch_cases = [
                i
                for i in mismatches
                if index in case_mismatch_functions(cases[i], reference[i], candidate[i])
            ]
            row = {
                "name": fn["c"],
                "cases": len(hit_cases),
                "mismatches": len(mismatch_cases),
                "detected_mutants": [
                    mutant["name"] for mutant in report["mutants"] if fn["c"] in mutant["functions"]
                ],
            }
            if data["scope"] == "stateful-host":
                row["sequences"] = sorted({cases[i]["name"] for i in hit_cases})
                row["steps"] = sum(
                    sum(step["function"] == index for step in cases[i]["steps"]) for i in hit_cases
                )
            else:
                row["layouts"] = sorted({cases[i]["layout"] for i in hit_cases})
                row["placements"] = sorted({cases[i]["placement"] for i in hit_cases})
            per_function.append(row)
        report["per_function"] = per_function
        report["status"] = "mismatch" if mismatches else "pass"
    except _CandidateCrashed:
        pass
    except ContractError as exc:
        report["error"] = str(exc)
    except Exception as exc:  # report.json is authoritative: never leave a run without one
        report["error"] = f"{type(exc).__name__}: {exc}"
    if "error" in report and report["status"] == "mismatch":
        report["mutant_error"] = report.pop("error")
    (output / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--worker", nargs=3, type=Path, required=True)
    parser.add_argument("--case-index", type=int)
    args = parser.parse_args()
    worker(*args.worker, case_index=args.case_index)
