"""Call graph extraction and topological sorting for per-function translation."""

from __future__ import annotations

from dataclasses import dataclass, field

import tree_sitter_c as tsc
from tree_sitter import Language, Parser

from .scanner import ScanResult

C_LANG = Language(tsc.language())


@dataclass
class FunctionDep:
    """Per-function dependency info extracted from C source."""

    name: str
    callees: set[str] = field(default_factory=set)  # Functions this function calls
    identifiers: set[str] = field(
        default_factory=set
    )  # All identifiers (types, globals, constants)
    body_text: str = ""  # Full C function body
    return_type: str = ""
    calling_convention: str | None = None
    params: list[tuple[str, str]] = field(default_factory=list)
    is_static: bool = False
    is_exported: bool = False  # In ntoskrnl.spec
    code_seg: str | None = None
    seh_count: int = 0
    line_start: int = 0
    line_end: int = 0
    # C++ method origin (explorer/shell translation). For methods `name` is
    # the UNIQUE TARGET SYMBOL (`Cls_method`, `Cls_ctor`, `Cls_dtor`, with a
    # _2/_3 suffix for overloads) — filesystem-safe, collision-free across
    # classes in one file, and equal to the Rust #[no_mangle] name. The
    # original C++ identity lives here for prompts/diagnostics.
    cpp_class: str | None = None
    cpp_method: str | None = None


def _extract_text(node, source: bytes) -> str:
    return source[node.start_byte : node.end_byte].decode("utf-8", errors="replace")


def _collect_call_targets(node, source: bytes, targets: set[str]) -> None:
    """Collect function names from call_expression nodes."""
    if node.type == "call_expression":
        # The first child of a call_expression is the function being called
        func_node = node.children[0] if node.children else None
        if func_node:
            if func_node.type == "identifier":
                targets.add(_extract_text(func_node, source))
            elif func_node.type == "field_expression":
                # Method-style calls like obj->Method — skip, not direct function calls
                pass
            elif func_node.type == "parenthesized_expression":
                # Cast-then-call like ((PFUNC)ptr)(args) — skip
                pass
    for child in node.children:
        _collect_call_targets(child, source, targets)


def _collect_identifiers(node, source: bytes, identifiers: set[str]) -> None:
    """Recursively collect all identifier tokens from a tree-sitter node."""
    if node.type == "identifier" or node.type == "type_identifier":
        name = _extract_text(node, source)
        if len(name) > 1:  # skip single-char vars
            identifiers.add(name)
    for child in node.children:
        _collect_identifiers(child, source, identifiers)


def extract_call_graph(scan: ScanResult, spec_exports: dict | None = None) -> list[FunctionDep]:
    """Extract per-function dependencies from a scanned C file.

    Args:
        scan: ScanResult from scanner.scan_file()
        spec_exports: dict of function_name → SpecEntry for exports

    Returns:
        List of FunctionDep, one per function in the file.
    """
    if spec_exports is None:
        spec_exports = {}

    # C keywords that tree-sitter may misidentify as function names
    C_KEYWORDS = {
        "if",
        "else",
        "while",
        "for",
        "do",
        "switch",
        "case",
        "return",
        "break",
        "continue",
        "goto",
        "default",
        "sizeof",
        "typeof",
    }

    parser = Parser(C_LANG)
    deps: list[FunctionDep] = []
    seen_names: dict[str, int] = {}

    for func in scan.functions:
        # Skip spurious tree-sitter parse artifacts
        if func.name in C_KEYWORDS:
            continue
        # Unique target-symbol name. C++ methods get Cls_method (the Rust
        # #[no_mangle] name); ctors/dtors get Cls_ctor/Cls_dtor; overloads
        # (and same-named methods across classes are already split by the
        # Cls_ prefix) get a _2/_3 suffix.
        if func.cpp_class:
            if func.is_ctor:
                sym_name = f"{func.cpp_class}_ctor"
            elif func.is_dtor:
                sym_name = f"{func.cpp_class}_dtor"
            else:
                sym_name = f"{func.cpp_class}_{func.name}"
        else:
            sym_name = func.name
        n = seen_names.get(sym_name, 0) + 1
        seen_names[sym_name] = n
        if n > 1:
            sym_name = f"{sym_name}_{n}"
        # Parse the function body text to extract call targets and identifiers
        source = func.body_text.encode("utf-8")
        tree = parser.parse(source)

        callees: set[str] = set()
        identifiers: set[str] = set()

        _collect_call_targets(tree.root_node, source, callees)
        _collect_identifiers(tree.root_node, source, identifiers)

        dep = FunctionDep(
            name=sym_name,
            callees=callees,
            identifiers=identifiers,
            body_text=func.body_text,
            return_type=func.return_type,
            calling_convention=func.calling_convention,
            params=func.params,
            is_static=func.is_static,
            is_exported=func.name in spec_exports,
            code_seg=func.code_seg,
            seh_count=func.seh_count,
            line_start=func.line_start,
            line_end=func.line_end,
            cpp_class=func.cpp_class,
            cpp_method=func.name if func.cpp_class else None,
        )
        deps.append(dep)

    return deps


def topological_sort(deps: list[FunctionDep]) -> list[FunctionDep]:
    """Sort functions so callees come before callers.

    Only considers local callees (functions defined in the same file).
    Breaks cycles by placing the function with fewer local callees first.
    """
    # Build name → dep mapping
    by_name: dict[str, FunctionDep] = {d.name: d for d in deps}
    local_names = set(by_name.keys())

    # Build adjacency: caller → set of local callees
    local_callees: dict[str, set[str]] = {}
    for dep in deps:
        local_callees[dep.name] = dep.callees & local_names

    # Kahn's algorithm with cycle-breaking
    in_degree: dict[str, int] = {name: 0 for name in by_name}
    for name, callees in local_callees.items():
        for callee in callees:
            if callee != name:  # skip self-recursion
                in_degree[callee] = in_degree.get(callee, 0)
                in_degree[name] = in_degree.get(name, 0)

    # Recompute: in_degree[X] = number of functions that X calls locally
    # Wait — we want callees before callers, so:
    # edge: caller → callee means callee should come first
    # in_degree[caller] += 1 for each callee
    in_degree = {name: 0 for name in by_name}
    for name, callees in local_callees.items():
        for callee in callees:
            if callee != name and callee in by_name:
                in_degree[name] += 1

    # Reverse adjacency: callee → set of callers
    reverse: dict[str, set[str]] = {name: set() for name in by_name}
    for name, callees in local_callees.items():
        for callee in callees:
            if callee != name and callee in by_name:
                reverse[callee].add(name)

    result: list[FunctionDep] = []
    queue = [name for name in by_name if in_degree[name] == 0]
    # Sort queue by line_start for deterministic ordering among peers
    queue.sort(key=lambda n: by_name[n].line_start)

    visited = set()
    while queue:
        name = queue.pop(0)
        if name in visited:
            continue
        visited.add(name)
        result.append(by_name[name])

        # For each caller of this function, decrement in_degree
        for caller in sorted(reverse[name], key=lambda n: by_name[n].line_start):
            in_degree[caller] -= 1
            if in_degree[caller] == 0:
                queue.append(caller)

    # Handle cycles: add remaining functions in source order
    remaining = [dep for dep in deps if dep.name not in visited]
    remaining.sort(key=lambda d: d.line_start)
    result.extend(remaining)

    return result


def print_call_graph(deps: list[FunctionDep], local_only: bool = True) -> None:
    """Print call graph in human-readable form."""
    local_names = {d.name for d in deps}

    for dep in deps:
        exported = " [EXPORTED]" if dep.is_exported else ""
        static = "static " if dep.is_static else ""
        cc = f" {dep.calling_convention}" if dep.calling_convention else ""
        seg = f"[{dep.code_seg}] " if dep.code_seg else ""
        seh = f" (SEH×{dep.seh_count})" if dep.seh_count else ""

        print(
            f"{seg}{static}{dep.return_type}{cc} {dep.name}{exported}{seh}  "
            f"L{dep.line_start}-{dep.line_end}"
        )

        if local_only:
            local = dep.callees & local_names
        else:
            local = dep.callees

        if local:
            print(f"  calls: {', '.join(sorted(local))}")

        print(f"  identifiers: {len(dep.identifiers)}")
