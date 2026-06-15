"""Per-function identifier resolution — adapts resolver.py for function granularity."""

from __future__ import annotations

from .callgraph import FunctionDep
from .catalog import Catalog
from .resolver import BUILTIN_NAMES, GAP_FUNCTIONS, ImportGroup, ResolveResult
from .spec_parser import SpecEntry


def resolve_function(
    func: FunctionDep,
    catalog: Catalog,
    spec_exports: dict[str, SpecEntry],
    file_defines: list[str] | None = None,
) -> ResolveResult:
    """Resolve identifiers for a single function.

    Same logic as file-level resolver but scoped to one function's identifier set.

    Args:
        func: FunctionDep with identifiers and callees
        catalog: ntkrnl-sys symbol catalog
        spec_exports: dict of function_name → SpecEntry
        file_defines: #define names from the file (local constants)

    Returns:
        ResolveResult with imports, gap_functions, unresolved, and exports.
    """
    if file_defines is None:
        file_defines = []

    defines_set = set(file_defines)
    result = ResolveResult()

    # Check if this function is exported
    if func.name in spec_exports:
        result.exports[func.name] = spec_exports[func.name]

    # Resolve each identifier
    for ident in func.identifiers:
        if ident in BUILTIN_NAMES:
            continue

        # Check catalog
        sym = catalog.lookup(ident)
        if sym:
            if sym.module not in result.imports:
                result.imports[sym.module] = ImportGroup(module=sym.module)
            if not any(s.name == sym.name for s in result.imports[sym.module].symbols):
                result.imports[sym.module].symbols.append(sym)
            continue

        # Check gap functions
        if ident in GAP_FUNCTIONS:
            result.gap_functions.add(ident)
            continue

        # Check file-local defines
        if ident in defines_set:
            continue

        # Remaining — unresolved (filter short names and lowercase)
        if len(ident) > 2 and ident[0].isupper():
            result.unresolved.add(ident)

    return result
