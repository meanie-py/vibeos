"""Generate C shim files from TOML manifests.

Produces 5-phase C shim files that provide real linker symbols for
FORCEINLINE/macro functions that Rust FFI extern declarations need.

Phase 1: #define FORCEINLINE names to _fi_* before headers
Phase 2: #include <ntoskrnl.h>
Phase 3: Trampoline static inlines for macros
Phase 4: #undef all renames and macros
Phase 5: Real (non-static, non-inline) function definitions
"""

from __future__ import annotations

import re
from dataclasses import dataclass

try:
    import tomllib  # type: ignore[import-not-found]  # stdlib on 3.11+; tomli backport on 3.10
except ImportError:
    import tomli as tomllib  # type: ignore[no-redef]

from .config import DATA_DIR, NTOSKRNL_DIR


@dataclass
class ShimEntry:
    """A single shim function."""

    name: str
    return_type: str
    calling_convention: str  # "" or "NTAPI"
    params: list[tuple[str, str]]  # [(type, name), ...]
    kind: str  # "forceinline", "macro", "macro_arch"
    arches: list[str] | None  # Only for macro_arch


def parse_signature(name: str, sig: str) -> ShimEntry:
    """Parse a C-style signature string into a ShimEntry.

    Formats:
        "void (POBJECT_HEADER ObjectHeader)"
        "PVOID (EX_FAST_REF FastRef)"
        "PETHREAD NTAPI (void)"
        "ULONGLONG NTAPI (void)"
    """
    # Split return type + calling conv from params
    m = re.match(r"^(.+?)\s*\(([^)]*)\)\s*$", sig)
    if not m:
        raise ValueError(f"Cannot parse signature for {name}: {sig}")

    ret_and_conv = m.group(1).strip()
    params_str = m.group(2).strip()

    # Extract calling convention
    calling_convention = ""
    if "NTAPI" in ret_and_conv:
        calling_convention = "NTAPI"
        ret_and_conv = ret_and_conv.replace("NTAPI", "").strip()

    return_type = ret_and_conv

    # Parse params
    params: list[tuple[str, str]] = []
    if params_str and params_str != "void":
        for p in params_str.split(","):
            p = p.strip()
            # Last word is the name, rest is type
            parts = p.rsplit(None, 1)
            if len(parts) == 2:
                params.append((parts[0], parts[1]))
            else:
                params.append((p, ""))

    return ShimEntry(
        name=name,
        return_type=return_type,
        calling_convention=calling_convention,
        params=params,
        kind="forceinline",
        arches=None,
    )


def load_manifest(subsystem: str) -> list[ShimEntry]:
    """Load a shim manifest TOML file."""
    manifest_path = DATA_DIR / "shims" / f"{subsystem}.toml"
    if not manifest_path.exists():
        raise FileNotFoundError(f"No manifest found: {manifest_path}")

    with open(manifest_path, "rb") as f:
        data = tomllib.load(f)

    entries: list[ShimEntry] = []

    # FORCEINLINE entries
    for name, sig in data.get("forceinline", {}).items():
        entry = parse_signature(name, sig)
        entry.kind = "forceinline"
        entries.append(entry)

    # Macro entries
    for name, sig in data.get("macro", {}).items():
        entry = parse_signature(name, sig)
        entry.kind = "macro"
        entries.append(entry)

    # Arch-specific macro entries
    for name, info in data.get("macro_arch", {}).items():
        entry = parse_signature(name, info["sig"])
        entry.kind = "macro_arch"
        entry.arches = info["arches"]
        entries.append(entry)

    return entries


def _format_param_list(params: list[tuple[str, str]]) -> str:
    """Format parameter list for C function signature."""
    if not params:
        return "void"
    return ", ".join(f"{typ} {name}" for typ, name in params)


def _format_arg_list(params: list[tuple[str, str]]) -> str:
    """Format argument list for calling the function."""
    if not params:
        return ""
    return ", ".join(name for _, name in params)


def _format_fn_signature(entry: ShimEntry, prefix: str = "") -> str:
    """Format a complete C function signature."""
    conv = f" {entry.calling_convention}" if entry.calling_convention else ""
    name = f"{prefix}{entry.name}" if prefix else entry.name
    params = _format_param_list(entry.params)
    return f"{entry.return_type}{conv} {name}({params})"


def generate_shims(subsystem: str) -> str:
    """Generate a complete 5-phase C shim file from the manifest."""
    entries = load_manifest(subsystem)

    forceinline_entries = [e for e in entries if e.kind == "forceinline"]
    macro_entries = [e for e in entries if e.kind == "macro"]
    macro_arch_entries = [e for e in entries if e.kind == "macro_arch"]

    lines: list[str] = []

    # Header comment
    lines.append("/*")
    lines.append(
        f' * {subsystem}_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro'
    )
    lines.append(" * functions that the Rust ntoskrnl-rust crate calls via extern declarations.")
    lines.append(" *")
    lines.append(
        " * Inside ntoskrnl, many kernel functions are macros or FORCEINLINE (static inline)."
    )
    lines.append(
        " * The C compiler inlines them, producing no linker symbol. Rust extern declarations"
    )
    lines.append(" * need real symbols. This file provides them.")
    lines.append(" *")
    lines.append(" * Technique:")
    lines.append(" *   1. #define FORCEINLINE function names to _fi_* BEFORE including headers,")
    lines.append(" *      so the static inline definitions use renamed identifiers.")
    lines.append(" *   2. Include ntoskrnl.h (which pulls in all internal headers).")
    lines.append(" *   3. Create trampoline helpers for macros (KeEnterCriticalRegion etc.).")
    lines.append(" *   4. #undef macros and renames.")
    lines.append(" *   5. Define real (non-static, non-inline) functions with the original names.")
    lines.append(" *")
    lines.append(f" * Generated by: python -m rustconv shims {subsystem}")
    lines.append(" */")
    lines.append("")

    # === Phase 1: Rename FORCEINLINE ===
    lines.append("/* === Phase 1: Rename FORCEINLINE static inline functions before headers === */")
    lines.append("")
    for e in forceinline_entries:
        pad = max(1, 42 - len(e.name))
        lines.append(f"#define {e.name}{' ' * pad}_fi_{e.name}")
    lines.append("")

    # === Phase 2: Include headers ===
    lines.append("/* === Phase 2: Include headers === */")
    lines.append("/* Now all FORCEINLINE static functions get renamed _fi_* names. */")
    lines.append("/* Macros are defined normally. */")
    lines.append("")
    lines.append("#include <ntoskrnl.h>")

    # === Phase 3: Trampolines for macros ===
    lines.append("")
    lines.append("/* === Phase 3: Trampolines for macro-defined functions === */")
    lines.append("/* These capture the macro expansion into static inline functions. */")
    lines.append("")
    for e in macro_entries:
        sig = _format_fn_signature(e, prefix="_tramp_")
        ret_stmt = "return " if e.return_type not in ("void", "VOID") else ""
        args = _format_arg_list(e.params)
        lines.append(f"static __inline {sig}")
        lines.append("{")
        lines.append(f"    {ret_stmt}{e.name}({args});")
        lines.append("}")
        lines.append("")
    for e in macro_arch_entries:
        arches_cond = " || ".join(f"defined({a})" for a in (e.arches or []))
        sig = _format_fn_signature(e, prefix="_tramp_")
        ret_stmt = "return " if e.return_type not in ("void", "VOID") else ""
        args = _format_arg_list(e.params)
        lines.append(f"#if {arches_cond}")
        lines.append(f"static __inline {sig}")
        lines.append("{")
        lines.append(f"    {ret_stmt}{e.name}({args});")
        lines.append("}")
        lines.append("#endif")
        lines.append("")

    # === Phase 4: Undef ===
    lines.append("/* === Phase 4: Undef all macros and renames === */")
    lines.append("")
    # Undef macros first
    for e in macro_entries:
        lines.append(f"#undef {e.name}")
    lines.append("")
    for e in macro_arch_entries:
        arches_cond = " || ".join(f"defined({a})" for a in (e.arches or []))
        lines.append(f"#if {arches_cond}")
        lines.append(f"#undef {e.name}")
        lines.append("#endif")
    lines.append("")
    # Undef forceinline renames
    for e in forceinline_entries:
        lines.append(f"#undef {e.name}")
    lines.append("")

    # === Phase 5: Real functions ===
    lines.append("/* === Phase 5: Real (non-static, non-inline) functions === */")
    lines.append("")
    # Macro-based functions call _tramp_
    for e in macro_entries:
        sig = _format_fn_signature(e)
        ret_stmt = "return " if e.return_type not in ("void", "VOID") else ""
        args = _format_arg_list(e.params)
        lines.append(f"{sig}")
        lines.append("{")
        lines.append(f"    {ret_stmt}_tramp_{e.name}({args});")
        lines.append("}")
        lines.append("")
    for e in macro_arch_entries:
        arches_cond = " || ".join(f"defined({a})" for a in (e.arches or []))
        sig = _format_fn_signature(e)
        ret_stmt = "return " if e.return_type not in ("void", "VOID") else ""
        args = _format_arg_list(e.params)
        lines.append(f"#if {arches_cond}")
        lines.append(f"{sig}")
        lines.append("{")
        lines.append(f"    {ret_stmt}_tramp_{e.name}({args});")
        lines.append("}")
        lines.append("#endif")
        lines.append("")
    # FORCEINLINE functions call _fi_
    for e in forceinline_entries:
        sig = _format_fn_signature(e)
        ret_stmt = "return " if e.return_type not in ("void", "VOID") else ""
        args = _format_arg_list(e.params)
        lines.append(f"{sig}")
        lines.append("{")
        lines.append(f"    {ret_stmt}_fi_{e.name}({args});")
        lines.append("}")
        lines.append("")

    return "\n".join(lines)


def check_shims(subsystem: str) -> tuple[bool, str]:
    """Compare generated shims against existing file.

    Returns (matches, diff_description).
    """
    existing_path = NTOSKRNL_DIR / subsystem / f"{subsystem}_rust_shims.c"
    if not existing_path.exists():
        return False, f"No existing file: {existing_path}"

    generated = generate_shims(subsystem)
    existing = existing_path.read_text()

    if generated.strip() == existing.strip():
        return True, "Files match"

    # Show summary of differences
    gen_lines = generated.strip().splitlines()
    ext_lines = existing.strip().splitlines()

    diff_lines = []
    max_lines = max(len(gen_lines), len(ext_lines))
    diffs = 0
    for i in range(max_lines):
        gl = gen_lines[i] if i < len(gen_lines) else "<missing>"
        el = ext_lines[i] if i < len(ext_lines) else "<missing>"
        if gl != el:
            diffs += 1
            if diffs <= 10:
                diff_lines.append(f"  line {i + 1}:")
                diff_lines.append(f"    generated: {gl}")
                diff_lines.append(f"    existing:  {el}")

    desc = f"{diffs} line(s) differ"
    if diff_lines:
        desc += ":\n" + "\n".join(diff_lines)
    if diffs > 10:
        desc += f"\n  ... and {diffs - 10} more"

    return False, desc
