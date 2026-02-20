"""Assemble system + per-file prompts for C-to-Rust translation."""

from pathlib import Path

from .config import TEMPLATES_DIR
from .resolver import ResolveResult
from .scanner import ScanResult


def load_system_prompt() -> str:
    """Load the system prompt template."""
    path = TEMPLATES_DIR / "system_prompt.md"
    return path.read_text()


def build_file_prompt(scan: ScanResult, resolve: ResolveResult) -> str:
    """Build the per-file prompt with C source, imports, exports, and gap functions."""
    parts = []

    # Section 1: File metadata
    rel_path = scan.path.name
    parts.append(f"## File: `{rel_path}`\n")

    # Section 2: Exports from ntoskrnl.spec
    if resolve.exports:
        parts.append("### Exported Functions (from ntoskrnl.spec)")
        parts.append("These functions are exported from ntoskrnl.exe and MUST be `#[no_mangle] pub unsafe`:\n")
        for name, entry in sorted(resolve.exports.items()):
            fwd = f" (forwards to {entry.forward_target})" if entry.forward_target else ""
            arch = f" [arch: {','.join(entry.arch_filter)}]" if entry.arch_filter else ""
            parts.append(f"- `{entry.convention} {name}({', '.join(entry.params)})`{fwd}{arch}")
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
        parts.append("These C macros/inline functions must be expanded or replaced per the system prompt rules:\n")
        for name in sorted(resolve.gap_functions):
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
