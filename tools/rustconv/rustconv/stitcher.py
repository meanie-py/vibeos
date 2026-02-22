"""Assemble individually-translated functions into a complete .rs file."""

from __future__ import annotations

import re
from typing import TYPE_CHECKING

from .callgraph import FunctionDep
from .resolver import ResolveResult

if TYPE_CHECKING:
    from .scanner import ScanResult, CGlobal


def _extract_use_statements(code: str) -> tuple[list[str], str]:
    """Extract `use` statements from a code fragment and return (uses, remaining_code).

    Returns:
        Tuple of (list of use statement strings, code with uses removed).
    """
    uses = []
    remaining_lines = []

    for line in code.splitlines():
        stripped = line.strip()
        if stripped.startswith("use "):
            uses.append(stripped)
        else:
            remaining_lines.append(line)

    # Strip leading blank lines from remaining
    while remaining_lines and not remaining_lines[0].strip():
        remaining_lines.pop(0)

    return uses, "\n".join(remaining_lines)


def _extract_extern_blocks(code: str) -> tuple[list[str], str]:
    """Extract standalone extern block declarations from code.

    Finds `extern "X" { ... }` blocks and extracts them.
    Returns (list of extern block strings, code with blocks removed).
    """
    extern_blocks = []
    remaining = code

    # Match extern "X" { ... } blocks (possibly multiline)
    pattern = re.compile(
        r'^(extern\s+"[^"]+"\s*\{[^}]*\})',
        re.MULTILINE | re.DOTALL,
    )

    # Simpler approach: find extern blocks line by line
    lines = code.splitlines()
    result_lines = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        # Detect start of extern block
        if re.match(r'extern\s+"[^"]*"\s*\{', stripped):
            block_lines = [line]
            brace_count = stripped.count('{') - stripped.count('}')
            i += 1
            while i < len(lines) and brace_count > 0:
                block_lines.append(lines[i])
                brace_count += lines[i].count('{') - lines[i].count('}')
                i += 1
            extern_blocks.append("\n".join(block_lines))
        else:
            result_lines.append(line)
            i += 1

    return extern_blocks, "\n".join(result_lines)


def _dedup_uses(use_statements: list[str]) -> list[str]:
    """Deduplicate and sort use statements."""
    # Normalize: remove trailing semicolons for comparison, re-add
    seen = set()
    deduped = []
    for stmt in use_statements:
        normalized = stmt.strip().rstrip(';') + ';'
        if normalized not in seen:
            seen.add(normalized)
            deduped.append(normalized)

    # Sort: hand-written modules first, then ddk, then core
    def sort_key(s):
        if 'ntkrnl_sys::ddk' in s:
            return (1, s)
        elif 'ntkrnl_sys::' in s:
            return (0, s)
        elif 'core::' in s:
            return (2, s)
        return (3, s)

    return sorted(deduped, key=sort_key)


def _extract_decl_name(decl: str) -> str | None:
    """Extract the symbol name from an extern declaration for deduplication.

    E.g., 'fn CCTRACE(level: u32, format: *const i8, ...);' → 'CCTRACE'
          'static mut CcDirtyPages: u32;' → 'CcDirtyPages'
          'static CcZeroPage: usize;' → 'CcZeroPage'
    """
    # Match fn declarations
    m = re.match(r'fn\s+(\w+)\s*\(', decl.strip())
    if m:
        return m.group(1)
    # Match static declarations
    m = re.match(r'static\s+(?:mut\s+)?(\w+)\s*:', decl.strip())
    if m:
        return m.group(1)
    return None


def _merge_extern_blocks(blocks: list[str]) -> list[str]:
    """Merge extern blocks with the same linkage into one block each.

    Deduplicates by symbol name — if multiple functions declared the same extern
    with different parameter names, keep only the first.
    """
    by_linkage: dict[str, list[str]] = {}
    seen_names: dict[str, set[str]] = {}  # linkage → set of declared symbol names

    for block in blocks:
        # Extract linkage type
        m = re.match(r'extern\s+"([^"]+)"\s*\{', block.strip())
        if not m:
            continue
        linkage = m.group(1)

        # Extract inner declarations
        inner = re.search(r'\{(.*)\}', block, re.DOTALL)
        if not inner:
            continue

        decls_text = inner.group(1).strip()
        if decls_text:
            if linkage not in by_linkage:
                by_linkage[linkage] = []
                seen_names[linkage] = set()

            # Join multi-line declarations into single lines first
            # A declaration ends with ';' — accumulate lines until we hit one
            joined_decls = []
            current = ""
            for line in decls_text.splitlines():
                stripped = line.strip()
                if not stripped:
                    continue
                current = (current + " " + stripped).strip() if current else stripped
                if current.endswith(';'):
                    joined_decls.append(current)
                    current = ""
            if current:  # Leftover without semicolon
                joined_decls.append(current)

            for decl in joined_decls:
                name = _extract_decl_name(decl)
                if name:
                    if name in seen_names[linkage]:
                        continue  # Skip duplicate
                    seen_names[linkage].add(name)
                by_linkage[linkage].append(decl)

    result = []
    for linkage in sorted(by_linkage.keys()):
        decls = by_linkage[linkage]
        if decls:
            result.append(f'extern "{linkage}" {{')
            for decl in decls:
                result.append(f"    {decl}")
            result.append("}")

    return result


def _generate_imports_from_resolve(resolve: ResolveResult) -> list[str]:
    """Generate use statements from a file-level ResolveResult.

    Since the per-function prompt tells the LLM not to emit use statements,
    we must generate them from the resolved imports.
    """
    uses = []

    for module, group in sorted(resolve.imports.items()):
        names = sorted(s.name for s in group.symbols)
        if not names:
            continue

        if module == "ddk":
            # Selective imports from ddk (never glob)
            # Group into reasonable line lengths
            if len(names) <= 8:
                uses.append(f"use ntkrnl_sys::ddk::{{{', '.join(names)}}};")
            else:
                # Multi-line import
                items = ", ".join(names)
                uses.append(f"use ntkrnl_sys::ddk::{{{items}}};")
        else:
            # Hand-written modules get glob imports
            uses.append(f"use ntkrnl_sys::{module}::*;")

    # Always include core::ffi::c_void — used by almost every file
    uses.append("use core::ffi::c_void;")

    return uses


def _build_globals_section(scan: ScanResult, all_resolves: dict[str, ResolveResult]) -> str:
    """Build the globals section from scan result.

    This is a placeholder — the LLM may have emitted global declarations in
    individual function outputs. We collect them here for deduplication.
    """
    # Globals are typically emitted by the first function or as standalone.
    # The stitcher will preserve any static mut declarations found in function outputs.
    return ""


def stitch_file(
    functions: dict[str, str],
    deps: list[FunctionDep],
    scan: ScanResult,
    file_resolve: ResolveResult,
) -> str:
    """Assemble individually-translated functions into a complete .rs file.

    Args:
        functions: dict of function_name → Rust code from LLM
        deps: list of FunctionDep in topological order
        scan: ScanResult for the original C file
        file_resolve: File-level ResolveResult for overall imports

    Returns:
        Complete .rs file contents as a string.
    """
    all_uses: list[str] = []
    all_externs: list[str] = []
    function_bodies: list[tuple[int, str, str]] = []  # (line_start, name, code)
    standalone_items: list[str] = []  # const, static, type alias outside functions

    for dep in deps:
        if dep.name not in functions:
            continue

        code = functions[dep.name]

        # Extract use statements
        uses, code_no_uses = _extract_use_statements(code)
        all_uses.extend(uses)

        # Extract extern blocks
        externs, code_no_externs = _extract_extern_blocks(code_no_uses)
        all_externs.extend(externs)

        # Extract standalone items (static mut, const, type alias) that appear
        # before the function definition
        fn_lines = code_no_externs.splitlines()
        fn_start_idx = None
        for i, line in enumerate(fn_lines):
            stripped = line.strip()
            # Detect function definition start
            if (re.match(r'(#\[no_mangle\]|#\[cfg\(|pub\s+unsafe|unsafe\s+fn|pub\s+fn)', stripped)
                    or re.match(r'// CODE_SEG', stripped)):
                fn_start_idx = i
                break

        if fn_start_idx is not None and fn_start_idx > 0:
            preamble = "\n".join(fn_lines[:fn_start_idx]).strip()
            if preamble:
                standalone_items.append(preamble)
            code_no_externs = "\n".join(fn_lines[fn_start_idx:])

        function_bodies.append((dep.line_start, dep.name, code_no_externs.strip()))

    # Sort function bodies by original C source line order
    function_bodies.sort(key=lambda x: x[0])

    # Assemble the file
    parts = []

    # 1. Use statements — generate from file_resolve + collect any LLM-emitted ones
    resolve_uses = _generate_imports_from_resolve(file_resolve)
    all_uses_combined = resolve_uses + all_uses
    deduped_uses = _dedup_uses(all_uses_combined)
    if deduped_uses:
        for stmt in deduped_uses:
            parts.append(stmt)
        parts.append("")

    # 2. Merged extern blocks
    merged_externs = _merge_extern_blocks(all_externs)
    if merged_externs:
        for line in merged_externs:
            parts.append(line)
        parts.append("")

    # 3. Standalone items (static mut, const, etc.) — deduped
    seen_items = set()
    for item in standalone_items:
        normalized = item.strip()
        if normalized and normalized not in seen_items:
            seen_items.add(normalized)
            parts.append(normalized)
            parts.append("")

    # 4. Functions in source order
    for line_start, name, code in function_bodies:
        parts.append(code)
        parts.append("")

    return "\n".join(parts)
