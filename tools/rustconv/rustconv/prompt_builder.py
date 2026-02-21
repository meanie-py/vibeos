"""Assemble system + per-file prompts for C-to-Rust translation."""

import re
from pathlib import Path

from .config import TEMPLATES_DIR, NTOSKRNL_DIR
from .resolver import ResolveResult
from .scanner import ScanResult
from .spec_parser import rust_calling_convention


def load_system_prompt() -> str:
    """Load the system prompt template."""
    path = TEMPLATES_DIR / "system_prompt.md"
    return path.read_text()


def load_shim_signatures(subsystem: str) -> dict[str, str]:
    """Parse existing shim file Phase 5 to extract gap function signatures.

    Returns dict of function_name → C signature string.
    """
    shim_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
    if not shim_path.exists():
        return {}

    sigs: dict[str, str] = {}
    text = shim_path.read_text()

    # Find Phase 5 section
    phase5_marker = "Phase 5:"
    phase5_start = text.find(phase5_marker)
    if phase5_start < 0:
        return sigs

    phase5_text = text[phase5_start:]

    # Match function definitions: ReturnType [NTAPI] FuncName(params)
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


def build_file_prompt(scan: ScanResult, resolve: ResolveResult) -> str:
    """Build the per-file prompt with C source, imports, exports, and gap functions."""
    parts = []

    # Section 1: File metadata
    rel_path = scan.path.name
    parts.append(f"## File: `{rel_path}`\n")

    # Determine subsystem for shim lookup
    subsystem = scan.path.parent.name

    # Section 2: Exports from ntoskrnl.spec
    if resolve.exports:
        parts.append("### Exported Functions (from ntoskrnl.spec)")
        parts.append("These functions are exported from ntoskrnl.exe and MUST be `#[no_mangle] pub unsafe`:\n")
        for name, entry in sorted(resolve.exports.items()):
            fwd = f" (forwards to {entry.forward_target})" if entry.forward_target else ""
            arch = f" [arch: {','.join(entry.arch_filter)}]" if entry.arch_filter else ""
            conv = rust_calling_convention(entry)
            if entry.convention == "fastcall":
                parts.append(f"- `#[no_mangle] pub unsafe {conv} fn {name}(...)` — "
                             f"Use `extern \"fastcall\"` on `#[cfg(target_arch = \"x86\")]` "
                             f"and `extern \"C\"` on other arches{fwd}{arch}")
            else:
                parts.append(f"- `#[no_mangle] pub unsafe {conv} fn {name}(...)`{fwd}{arch}")
        parts.append("")

    # Section 3: Available imports from ntkrnl-sys
    if resolve.imports:
        parts.append("### Available Imports from `ntkrnl_sys`")
        parts.append("These types and functions are available. Use them in your `use` declarations:\n")
        for module, group in sorted(resolve.imports.items()):
            names = sorted(s.name for s in group.symbols)
            if module == "ddk":
                parts.append(f"From `ntkrnl_sys::ddk::*`: {', '.join(f'`{n}`' for n in names[:50])}")
                if len(names) > 50:
                    parts.append(f"  ... and {len(names) - 50} more")
            else:
                parts.append(f"From `ntkrnl_sys::{module}::*`: {', '.join(f'`{n}`' for n in names)}")
        parts.append("")

    # Section 4: Gap functions
    if resolve.gap_functions:
        parts.append("### Gap Functions (macros/FORCEINLINE — not importable)")
        parts.append("These C macros/inline functions must be expanded or replaced per the system prompt rules.")
        parts.append("They are provided as real symbols via the shim file. Declare them in `extern \"system\" { ... }` blocks:\n")

        shim_sigs = load_shim_signatures(subsystem)

        for name in sorted(resolve.gap_functions):
            if name in shim_sigs:
                parts.append(f"- `{shim_sigs[name]}` (shimmed)")
            else:
                parts.append(f"- `{name}`")
        parts.append("")

    # Section 5: Architecture-specific code
    if scan.arch_ifdefs:
        parts.append("### Architecture Conditionals")
        parts.append(f"This file uses: {', '.join(f'`{m}`' for m in scan.arch_ifdefs)}")
        parts.append("Map these to `#[cfg(target_arch = ...)]` per the system prompt rules.\n")

    # Section 6: The C source code
    parts.append("### C Source Code\n")
    parts.append("Translate the following C code to Rust:\n")
    parts.append("```c")
    parts.append(scan.raw_source)
    parts.append("```")

    return "\n".join(parts)


def build_full_prompt(scan: ScanResult, resolve: ResolveResult) -> str:
    """Build the complete prompt (system + file)."""
    system = load_system_prompt()
    file_prompt = build_file_prompt(scan, resolve)
    return system + "\n\n---\n\n" + file_prompt


def estimate_tokens(text: str) -> int:
    """Rough token estimate (1 token ≈ 4 chars for code)."""
    return len(text) // 4
