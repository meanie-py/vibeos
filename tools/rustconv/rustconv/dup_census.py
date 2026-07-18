"""Read-only exact and alpha-normalized C/C++ function duplication census.

This intentionally reports opportunity without reusing translations.  Alpha
normalization is useful evidence, but safe substitution additionally requires a
target/signature/substrate/version key and scope-aware proof.
"""

from __future__ import annotations

import json
import re
from collections import defaultdict
from collections.abc import Iterable
from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path

from tree_sitter import Node, Parser

from .outcome_ledger import lexical_fingerprint
from .scanner import C_LANG, CPP_LANG, CFunction, scan_file

SOURCE_SUFFIXES = {".c", ".cc", ".cpp", ".cxx", ".c++"}
SKIP_DIRS = {".git", ".responses", "node_modules", "target"}


@dataclass(frozen=True)
class FunctionEntry:
    path: str
    line: int
    name: str
    exact_hash: str
    alpha_hash: str
    alpha_complete: bool

    @property
    def label(self) -> str:
        return f"{self.path}:{self.line}:{self.name}"


def _signature_shape(function: CFunction) -> str:
    ret = re.sub(r"\s+", "", function.return_type or "")
    params = ",".join(re.sub(r"\s+", "", c_type) for c_type, _ in function.params)
    kind = "dtor" if function.is_dtor else "ctor" if function.is_ctor else "fn"
    return f"{kind}:{ret}({params})"


def _declarator_identifier(node: Node) -> Node | None:
    current = node
    while current.type != "identifier":
        child = current.child_by_field_name("declarator")
        if child is None:
            identifiers = [item for item in current.named_children if item.type == "identifier"]
            return identifiers[0] if len(identifiers) == 1 else None
        current = child
    return current


def _walk(node: Node) -> Iterable[Node]:
    yield node
    for child in node.named_children:
        yield from _walk(child)


def _compound_body(root: Node) -> Node | None:
    for node in _walk(root):
        if node.type == "function_definition":
            body = node.child_by_field_name("body")
            if body is not None:
                return body
    return next((node for node in _walk(root) if node.type == "compound_statement"), None)


def _parsed_function(source: str, suffix: str) -> tuple[bytes, Node | None, Node | None]:
    encoded = source.encode("utf-8", "replace")
    language = CPP_LANG if suffix.lower() != ".c" else C_LANG
    root = Parser(language).parse(encoded).root_node
    function_node = next((node for node in _walk(root) if node.type == "function_definition"), None)
    body = _compound_body(root)
    return encoded, function_node, body


def behavior_source(function: CFunction, suffix: str) -> tuple[str, bool]:
    """Source after the declarator: qualifiers/ctor initializers plus body."""
    encoded, function_node, body = _parsed_function(function.body_text, suffix)
    if function_node is None or body is None:
        brace = function.body_text.find("{")
        fallback = function.body_text[brace:] if brace >= 0 else function.body_text
        return fallback, False
    declarator = function_node.child_by_field_name("declarator")
    if declarator is None:
        return encoded[body.start_byte :].decode("utf-8", "replace"), False
    return encoded[declarator.end_byte : function_node.end_byte].decode("utf-8", "replace"), True


def alpha_normalized_source(function: CFunction, suffix: str) -> tuple[str, bool]:
    """Rename parameters/locals by declaration order, preserving external names."""
    source = function.body_text
    encoded, function_node, body = _parsed_function(source, suffix)
    if function_node is None or body is None or body.has_error:
        return source, False
    declarator = function_node.child_by_field_name("declarator")
    if declarator is None:
        return source, False
    behavior_start = declarator.end_byte

    ordered_names = [name for _, name in function.params if name]
    for node in _walk(body):
        if node.type != "declaration":
            continue
        type_node = node.child_by_field_name("type")
        for child in node.named_children:
            if child == type_node:
                continue
            declared = _declarator_identifier(child)
            if declared is None:
                continue
            name = encoded[declared.start_byte : declared.end_byte].decode("utf-8", "replace")
            if name not in ordered_names:
                ordered_names.append(name)

    rename = {name: f"__v{index}" for index, name in enumerate(ordered_names)}
    behavior_bytes = encoded[behavior_start : function_node.end_byte]
    edits: list[tuple[int, int, bytes]] = []
    for node in _walk(function_node):
        if node.type != "identifier":
            continue
        if node.start_byte < behavior_start:
            continue
        name = encoded[node.start_byte : node.end_byte].decode("utf-8", "replace")
        replacement = rename.get(name)
        if replacement is not None:
            edits.append(
                (
                    node.start_byte - behavior_start,
                    node.end_byte - behavior_start,
                    replacement.encode(),
                )
            )
    for start, end, replacement in reversed(edits):
        behavior_bytes = behavior_bytes[:start] + replacement + behavior_bytes[end:]
    return behavior_bytes.decode("utf-8", "replace"), True


def function_hashes(function: CFunction, suffix: str) -> tuple[str, str, bool]:
    shape = _signature_shape(function)
    behavior, exact_complete = behavior_source(function, suffix)
    exact = sha256((shape + "\n" + lexical_fingerprint(behavior)).encode()).hexdigest()
    alpha_source, complete = alpha_normalized_source(function, suffix)
    alpha = sha256((shape + "\n" + lexical_fingerprint(alpha_source)).encode()).hexdigest()
    return exact, alpha, complete and exact_complete


def source_files(paths: Iterable[Path]) -> list[Path]:
    files: set[Path] = set()
    for path in paths:
        if path.is_file() and path.suffix.lower() in SOURCE_SUFFIXES:
            files.add(path.resolve())
            continue
        if not path.is_dir():
            continue
        for candidate in path.rglob("*"):
            if not candidate.is_file() or candidate.suffix.lower() not in SOURCE_SUFFIXES:
                continue
            relative_parts = candidate.relative_to(path).parts[:-1]
            if any(part in SKIP_DIRS or part.startswith("build-") for part in relative_parts):
                continue
            files.add(candidate.resolve())
    return sorted(files)


def _group(entries: list[FunctionEntry], attribute: str) -> list[list[FunctionEntry]]:
    buckets: dict[str, list[FunctionEntry]] = defaultdict(list)
    for entry in entries:
        buckets[getattr(entry, attribute)].append(entry)
    return sorted(
        (
            sorted(group, key=lambda item: item.label)
            for group in buckets.values()
            if len(group) > 1
        ),
        key=lambda group: (-len(group), group[0].label),
    )


def _group_summary(groups: list[list[FunctionEntry]]) -> dict[str, int]:
    return {
        "groups": len(groups),
        "functions": sum(len(group) for group in groups),
        "reusable_copies": sum(len(group) - 1 for group in groups),
    }


def run_census(paths: Iterable[Path], root: Path, top: int = 20) -> dict:
    files = source_files(paths)
    entries: list[FunctionEntry] = []
    errors: list[dict[str, str]] = []
    unextracted = 0
    for path in files:
        try:
            scan = scan_file(path)
        except Exception as error:
            errors.append({"path": str(path), "error": str(error)})
            continue
        unextracted += len(scan.unextracted)
        try:
            display_path = str(path.relative_to(root))
        except ValueError:
            display_path = str(path)
        for function in scan.functions:
            exact, alpha, complete = function_hashes(function, path.suffix)
            entries.append(
                FunctionEntry(
                    display_path,
                    function.line_start,
                    function.display_name,
                    exact,
                    alpha,
                    complete,
                )
            )

    exact_groups = _group(entries, "exact_hash")
    alpha_groups = _group(entries, "alpha_hash")
    alpha_only = [group for group in alpha_groups if len({item.exact_hash for item in group}) > 1]

    def render(groups):
        return [[entry.label for entry in group] for group in groups[:top]]

    return {
        "schema": 1,
        "scope": [str(path) for path in paths],
        "files": len(files),
        "functions": len(entries),
        "unextracted": unextracted,
        "alpha_fallbacks": sum(not entry.alpha_complete for entry in entries),
        "scan_errors": errors,
        "exact": {**_group_summary(exact_groups), "top": render(exact_groups)},
        "alpha": {**_group_summary(alpha_groups), "top": render(alpha_groups)},
        "alpha_only": {**_group_summary(alpha_only), "top": render(alpha_only)},
        "warning": "Alpha-normalized matches are census-only and not safe reuse keys.",
    }


def print_census(report: dict, as_json: bool = False) -> None:
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return
    print(
        f"dup-census: {report['functions']:,} functions / {report['files']:,} files "
        f"({report['unextracted']} unextracted, {len(report['scan_errors'])} scan errors)"
    )
    for key, label in (("exact", "exact"), ("alpha_only", "alpha-only")):
        stats = report[key]
        print(
            f"  {label}: {stats['groups']:,} groups, {stats['functions']:,} functions, "
            f"{stats['reusable_copies']:,} duplicate copies"
        )
        for group in stats["top"]:
            print(f"    x{len(group)}  " + " | ".join(group[:4]))
    if report["alpha_fallbacks"]:
        print(f"  warning: {report['alpha_fallbacks']} functions could not be AST alpha-normalized")
    print("  alpha matches are census-only; no translations were reused or modified")
