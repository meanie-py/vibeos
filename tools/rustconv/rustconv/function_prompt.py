"""Per-function prompt builder for function-by-function translation."""

from __future__ import annotations

import re
from pathlib import Path
from typing import TYPE_CHECKING

from .callgraph import FunctionDep
from .config import TEMPLATES_DIR, NTOSKRNL_DIR, OUTPUT_CRATE
from .resolver import ResolveResult, GAP_FUNCTIONS
from .spec_parser import rust_calling_convention

if TYPE_CHECKING:
    from .scanner import ScanResult, CGlobal


def load_function_system_prompt() -> str:
    """Load the abbreviated system prompt for per-function translation."""
    path = TEMPLATES_DIR / "function_prompt.md"
    return path.read_text()


def _format_import_context(resolve: ResolveResult) -> str:
    """Format pre-resolved imports as context for the LLM."""
    parts = []
    parts.append("## Available Imports (pre-resolved for this function)")
    parts.append("These types/functions are available from ntkrnl_sys. Use them directly:\n")

    for module, group in sorted(resolve.imports.items()):
        names = sorted(s.name for s in group.symbols)
        if module == "ddk":
            parts.append(f"From `ntkrnl_sys::ddk`: {', '.join(f'`{n}`' for n in names[:60])}")
            if len(names) > 60:
                parts.append(f"  ... and {len(names) - 60} more")
        else:
            parts.append(f"From `ntkrnl_sys::{module}`: {', '.join(f'`{n}`' for n in names)}")

    return "\n".join(parts)


def _format_gap_functions(gap_fns: set[str], subsystem: str) -> str:
    """Format gap function info for the LLM."""
    if not gap_fns:
        return ""

    parts = []
    parts.append("\n## Gap Functions (macros/FORCEINLINE — use extern or expand)")

    # Try to load shim signatures
    shim_sigs = _load_shim_signatures(subsystem)

    for name in sorted(gap_fns):
        if name in shim_sigs:
            parts.append(f"- `{shim_sigs[name]}` — available via extern")
        else:
            parts.append(f"- `{name}` — expand inline per rules")

    return "\n".join(parts)


def _load_shim_signatures(subsystem: str) -> dict[str, str]:
    """Load gap function C signatures from the shim file."""
    shim_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
    if not shim_path.exists():
        return {}

    sigs: dict[str, str] = {}
    text = shim_path.read_text()

    phase5_marker = "Phase 5:"
    phase5_start = text.find(phase5_marker)
    if phase5_start < 0:
        return sigs

    phase5_text = text[phase5_start:]

    for m in re.finditer(
        r'^(\w[\w\s\*]*?)\s+(?:NTAPI\s+)?(\w+)\s*\(([^)]*)\)\s*\{',
        phase5_text,
        re.MULTILINE,
    ):
        ret_type = m.group(1).strip()
        name = m.group(2)
        params = m.group(3).strip()
        if name.startswith('_tramp_') or name.startswith('_fi_'):
            continue
        sigs[name] = f"{ret_type} {name}({params})"

    return sigs


def _format_callee_context(
    func: FunctionDep,
    translated: dict[str, str],
    all_deps: list[FunctionDep],
    catalog_fns: set[str],
) -> str:
    """Build callee context section.

    For each callee of the function being translated:
    - If already translated to Rust: show the full Rust function
    - If extern (in catalog): show extern declaration
    - If local but not yet translated: show C signature + first lines
    - If gap function: skip (handled by gap section)
    """
    if not func.callees:
        return ""

    local_names = {d.name for d in all_deps}
    by_name = {d.name: d for d in all_deps}

    parts = []
    parts.append("\n## Callee Context")
    parts.append("Functions called by this function and their signatures:\n")

    for callee_name in sorted(func.callees):
        if callee_name in GAP_FUNCTIONS:
            continue  # Handled by gap functions section
        if callee_name == func.name:
            continue  # Self-recursion, skip

        if callee_name in translated:
            # Already translated — show Rust source
            rust_src = translated[callee_name]
            # Show just the signature + first ~10 lines for context
            lines = rust_src.strip().splitlines()
            preview = "\n".join(lines[:15])
            if len(lines) > 15:
                preview += "\n    // ... (truncated)"
            parts.append(f"### `{callee_name}` (already translated to Rust)")
            parts.append(f"```rust\n{preview}\n```\n")

        elif callee_name in local_names:
            # Local function not yet translated — show C signature
            dep = by_name[callee_name]
            params_str = ", ".join(f"{t} {n}" for t, n in dep.params)
            sig = f"{dep.return_type} {dep.calling_convention or ''} {dep.name}({params_str})".strip()
            # Show first ~5 lines of body
            body_lines = dep.body_text.strip().splitlines()
            preview = "\n".join(body_lines[:8])
            if len(body_lines) > 8:
                preview += "\n    // ... (truncated)"
            parts.append(f"### `{callee_name}` (local C function, not yet translated)")
            parts.append(f"```c\n{preview}\n```\n")

        elif callee_name in catalog_fns:
            # Extern function from catalog
            parts.append(f"- `{callee_name}` — extern function (available from ntkrnl_sys)")

        else:
            # Unknown — also look in existing translated Rust files
            rust_sig = _find_rust_function_sig(callee_name)
            if rust_sig:
                parts.append(f"### `{callee_name}` (from another Rust module)")
                parts.append(f"```rust\n{rust_sig}\n```\n")
            else:
                parts.append(f"- `{callee_name}` — unknown extern (declare in extern block if needed)")

    if len(parts) <= 2:
        return ""  # No meaningful callee context

    return "\n".join(parts)


def _find_rust_function_sig(name: str) -> str | None:
    """Search existing ntoskrnl-rust .rs files for a function signature."""
    src_dir = OUTPUT_CRATE / "src"
    if not src_dir.exists():
        return None

    # Search for #[no_mangle] pub unsafe ... fn <name>(
    pattern = re.compile(
        rf'((?:#\[no_mangle\]\s*)?pub\s+unsafe\s+(?:extern\s+"[^"]+"\s+)?fn\s+{re.escape(name)}\s*\([^)]*\)\s*(?:->\s*\S+)?)',
        re.MULTILINE,
    )

    for rs_file in src_dir.rglob("*.rs"):
        if rs_file.name == "mod.rs" or rs_file.name == "lib.rs":
            continue
        try:
            text = rs_file.read_text()
            m = pattern.search(text)
            if m:
                return m.group(1)
        except (OSError, UnicodeDecodeError):
            continue

    return None


def _format_globals_context(globals_list: list[CGlobal], globals_text: str) -> str:
    """Format file-level globals context."""
    if not globals_list and not globals_text.strip():
        return ""

    parts = []
    parts.append("\n## File-Level Globals")
    parts.append("These globals are declared at file scope. Reference them directly:\n")

    if globals_text.strip():
        parts.append("```c")
        parts.append(globals_text.strip())
        parts.append("```")
    elif globals_list:
        for g in globals_list:
            init = f" = {g.initializer}" if g.initializer else ""
            parts.append(f"- `{g.type_str} {g.name}{init};`")

    return "\n".join(parts)


def _format_export_info(func: FunctionDep, spec_exports: dict) -> str:
    """Format export info for this specific function."""
    if not func.is_exported or func.name not in spec_exports:
        return ""

    entry = spec_exports[func.name]
    conv = rust_calling_convention(entry)
    fwd = f" (forwards to {entry.forward_target})" if entry.forward_target else ""
    arch = f" [arch: {','.join(entry.arch_filter)}]" if entry.arch_filter else ""

    parts = []
    parts.append("\n## Export Info")
    if entry.convention == "fastcall":
        parts.append(f"This function is EXPORTED as `{func.name}`. Use `#[no_mangle] pub unsafe`.")
        parts.append(f"Use `extern \"fastcall\"` on `#[cfg(target_arch = \"x86\")]` "
                      f"and `extern \"C\"` on other arches.{fwd}{arch}")
    else:
        parts.append(f"This function is EXPORTED as `{func.name}`. "
                      f"Use `#[no_mangle] pub unsafe {conv} fn {func.name}(...)`{fwd}{arch}")

    return "\n".join(parts)


def build_function_prompt(
    func: FunctionDep,
    resolve: ResolveResult,
    translated: dict[str, str],
    all_deps: list[FunctionDep],
    scan: ScanResult,
    catalog_fn_names: set[str],
    spec_exports: dict | None = None,
) -> str:
    """Build a complete prompt for translating a single C function.

    Args:
        func: The function to translate
        resolve: Per-function ResolveResult (from function_resolver)
        translated: dict of function_name → Rust code (already translated in this batch)
        all_deps: All FunctionDeps from this file (for local callee context)
        scan: ScanResult for the file (for globals)
        catalog_fn_names: Set of function names in the catalog
        spec_exports: dict of function_name → SpecEntry

    Returns:
        User-message prompt string (system prompt is separate for caching).
    """
    if spec_exports is None:
        spec_exports = {}

    subsystem = scan.path.parent.name
    parts = []

    # Header
    parts.append(f"## Translate: `{func.name}` from `{scan.path.name}`\n")

    # Export info
    export_info = _format_export_info(func, spec_exports)
    if export_info:
        parts.append(export_info)

    # Imports
    parts.append(_format_import_context(resolve))

    # Gap functions
    gap_info = _format_gap_functions(resolve.gap_functions, subsystem)
    if gap_info:
        parts.append(gap_info)

    # Globals
    globals_info = _format_globals_context(scan.globals, scan.globals_text)
    if globals_info:
        parts.append(globals_info)

    # Callee context
    callee_info = _format_callee_context(func, translated, all_deps, catalog_fn_names)
    if callee_info:
        parts.append(callee_info)

    # The C function source
    parts.append("\n## C Function to Translate\n")
    parts.append("```c")
    parts.append(func.body_text)
    parts.append("```")

    return "\n".join(parts)
