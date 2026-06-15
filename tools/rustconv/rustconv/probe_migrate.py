"""SEH migration pipeline for Rust kernel files.

Scans Rust files for patterns that need migration (bare probes, user-mode
pointer dereferences, unsafe copies), builds migration prompts with the
original C source as context (showing which operations were inside _SEH2_TRY),
submits to LLM, and applies postprocessing.

This is a unified pipeline that handles:
- ProbeForRead/Write → probe_read/write_user (Result<T>)
- *ptr = value (user-mode write-back) → probe_write_user
- RtlCopyMemory to/from user-mode → safe_copy_to/from_user
- EXCEPT cleanup (ExFreePool, ObDereference) → RAII guards
- goto Cleanup pattern → ? operator with Drop guards

Usage:
    python -m rustconv probe-migrate src/se/ -m haiku
    python -m rustconv probe-migrate src/se/sqos.rs -m sonnet --dry-run
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

from .batch_runner import submit_single
from .config import NTOSKRNL_DIR, TEMPLATES_DIR
from .seh_classifier import classify_file

# ============================================================================
# Scanner: find migration targets in Rust files
# ============================================================================

# Patterns that indicate a file needs probe/SEH migration
PROBE_PATTERNS = [
    re.compile(r"\bProbeForRead\b"),
    re.compile(r"\bProbeForWrite\b"),
    re.compile(r"\bProbeForWriteHandle\b"),
    re.compile(r"\bProbeForWriteUlong\b"),
    re.compile(r"\bProbeForWriteBoolean\b"),
    re.compile(r"\bProbeForReadUnicodeString\b"),
    re.compile(r"\bProbeForReadLargeInteger\b"),
    re.compile(r"\bProbeForReadUchar\b"),
    re.compile(r"\bProbeForReadUshort\b"),
    re.compile(r"\bProbeAndZeroHandle\b"),
]

# Extern declaration pattern for probe functions
PROBE_EXTERN_RE = re.compile(
    r"^\s*fn\s+(ProbeForRead|ProbeForWrite|ProbeForWriteHandle|"
    r"ProbeForWriteUlong|ProbeForWriteBoolean|ProbeForReadUnicodeString|"
    r"ProbeForReadLargeInteger|ProbeForReadUchar|ProbeForReadUshort|"
    r"ProbeAndZeroHandle)\b",
    re.MULTILINE,
)


@dataclass
class ProbeScanResult:
    """Result of scanning a Rust file for migration targets."""

    path: Path
    raw_source: str
    probe_call_count: int = 0
    probe_extern_count: int = 0
    has_c_source_comment: bool = False
    active_source: str = ""
    mode_variable: str = ""
    # SEH classification context (from C source)
    seh_context: str = ""
    c_source_path: Path | None = None


def scan_rust_file(path: Path) -> ProbeScanResult:
    """Scan a Rust file for probe/SEH migration targets."""
    source = path.read_text()
    result = ProbeScanResult(path=path, raw_source=source)

    # Split off trailing C source comments
    c_source_marker = "// ORIGINAL C SOURCE"
    marker_pos = source.find(c_source_marker)
    if marker_pos >= 0:
        block_start = source.rfind("// ====", 0, marker_pos)
        if block_start >= 0:
            result.has_c_source_comment = True
            result.active_source = source[:block_start].rstrip() + "\n"
        else:
            result.active_source = source[:marker_pos].rstrip() + "\n"
    else:
        result.active_source = source

    active = result.active_source

    # Count probe calls in active code
    for pat in PROBE_PATTERNS:
        result.probe_call_count += len(pat.findall(active))

    # Count extern declarations
    result.probe_extern_count = len(PROBE_EXTERN_RE.findall(active))

    # Detect mode variable name
    for name in ["PreviousMode", "previous_mode", "AccessMode", "CurrentMode", "ProcessorMode"]:
        if name in active:
            result.mode_variable = name
            break

    # Try to find corresponding C source and classify SEH patterns
    result.c_source_path = _find_c_source(path)
    if result.c_source_path and result.c_source_path.exists():
        result.seh_context = _build_seh_context(result.c_source_path)

    return result


def _find_c_source(rust_path: Path) -> Path | None:
    """Find the corresponding C source file for a Rust file.

    ntoskrnl-rust/src/se/tokencls.rs → ntoskrnl/se/tokencls.c
    ntoskrnl-rust/src/ob/oblife.rs → ntoskrnl/ob/oblife.c
    """
    # Extract subsystem and filename from Rust path
    parts = rust_path.parts
    try:
        src_idx = parts.index("src")
        subsys = parts[src_idx + 1]
        stem = rust_path.stem
        # Handle priv_ → priv mapping
        if stem.endswith("_"):
            stem = stem[:-1]
        c_path = NTOSKRNL_DIR / subsys / f"{stem}.c"
        if c_path.exists():
            return c_path
    except (ValueError, IndexError):
        pass
    return None


def _build_seh_context(c_path: Path) -> str:
    """Build SEH pattern context block from C source classification."""
    try:
        infos = classify_file(c_path)
    except Exception:
        return ""

    if not infos:
        return ""

    lines = ["[SEH_CONTEXT]", f"C source: {c_path.name}", ""]
    for info in infos:
        try_pats = ", ".join(p.name for p in info.all_try_patterns)
        handler_pats = ", ".join(p.name for p in info.all_handler_patterns)
        lines.append(f"Function: {info.name} ({len(info.seh_blocks)} SEH blocks)")
        lines.append(f"  TRY patterns: {try_pats}")
        lines.append(f"  Handler patterns: {handler_pats}")
        if info.needs_safe_copy:
            lines.append("  Needs: safe_copy_to/from_user")
        if info.needs_raii_guard:
            lines.append("  Needs: RAII guard")
        for block in info.seh_blocks:
            if block.write_backs:
                for wb in block.write_backs[:5]:
                    lines.append(f"  Write-back in TRY: {wb[:80]}")
            if block.copy_calls:
                for cc in block.copy_calls:
                    lines.append(f"  Copy in TRY: {cc}")
            if block.cleanup_calls:
                for cl in block.cleanup_calls:
                    lines.append(f"  Cleanup in EXCEPT: {cl}")
        lines.append("")

    return "\n".join(lines)


def find_rust_files(directory: Path, subsystems: list[str] | None = None) -> list[Path]:
    """Find Rust files that need migration (have probe calls OR SEH-affected code)."""
    if subsystems is None:
        subsystems = ["se", "ob"]

    files = []
    for subsys in subsystems:
        subsys_dir = directory / subsys
        if subsys_dir.is_dir():
            for rs_file in sorted(subsys_dir.glob("*.rs")):
                if rs_file.name == "mod.rs":
                    continue
                scan = scan_rust_file(rs_file)
                if scan.probe_call_count > 0 or scan.seh_context:
                    files.append(rs_file)

    return files


# ============================================================================
# Preprocessor: strip trailing C source to save tokens
# ============================================================================


def preprocess(source: str) -> str:
    """Strip trailing C source comments to reduce token count."""
    c_source_marker = "// ORIGINAL C SOURCE"
    marker_pos = source.find(c_source_marker)
    if marker_pos >= 0:
        block_start = source.rfind("// ====", 0, marker_pos)
        if block_start >= 0:
            return source[:block_start].rstrip() + "\n"
    return source


# ============================================================================
# Prompt builder
# ============================================================================


def load_probe_system_prompt() -> str:
    """Load the SEH migration system prompt."""
    path = TEMPLATES_DIR / "probe_migrate_prompt.md"
    return path.read_text()


def build_probe_file_prompt(scan: ProbeScanResult) -> str:
    """Build the per-file prompt for SEH migration."""
    parts = []

    parts.append(f"## File: `{scan.path.name}`")
    if scan.probe_call_count > 0:
        parts.append(f"Probe calls to migrate: {scan.probe_call_count}")
    if scan.mode_variable:
        parts.append(f"Detected mode variable: `{scan.mode_variable}`")
    parts.append("")

    # Include SEH classification context if available
    if scan.seh_context:
        parts.append("### SEH Pattern Analysis (from C source)\n")
        parts.append(scan.seh_context)
        parts.append("")

    # Include relevant C function bodies for context
    if scan.c_source_path and scan.c_source_path.exists():
        c_source = _extract_c_seh_functions(scan.c_source_path)
        if c_source:
            parts.append("### Original C Functions (showing _SEH2_TRY blocks)\n")
            parts.append("Use this to identify which Rust operations were inside `_SEH2_TRY`:")
            parts.append("")
            parts.append("```c")
            parts.append(c_source)
            parts.append("```")
            parts.append("")

    # Include the active Rust source (no trailing C comments)
    active = preprocess(scan.raw_source)

    parts.append("### Current Rust Source\n")
    parts.append("Migrate the SEH-affected patterns in this file:\n")
    parts.append("```rust")
    parts.append(active)
    parts.append("```")

    return "\n".join(parts)


def _extract_c_seh_functions(c_path: Path) -> str:
    """Extract C function bodies that contain SEH blocks.

    Only includes the functions with _SEH2_TRY, not the entire file,
    to keep the prompt size reasonable.
    """
    try:
        infos = classify_file(c_path)
    except Exception:
        return ""

    if not infos:
        return ""

    c_source = c_path.read_text(errors="replace")

    # Extract relevant function bodies
    parts = []
    for info in infos:
        # Find function in C source by line range
        lines = c_source.split("\n")
        start = max(0, info.line_start - 1)
        end = min(len(lines), info.line_end)
        func_body = "\n".join(lines[start:end])

        # Truncate very long functions to keep prompt size sane
        if len(func_body) > 4000:
            # Keep first 2000 and last 1000 chars
            func_body = func_body[:2000] + "\n\n// ... (truncated) ...\n\n" + func_body[-1000:]

        parts.append(f"// --- {info.name}() ---")
        parts.append(func_body)
        parts.append("")

    return "\n".join(parts)


# ============================================================================
# Postprocessor: mechanical fixes after LLM output
# ============================================================================


def _fix_question_mark_operator(code: str) -> str:
    """Convert `probe_call(...)?;` to `if let Err(s) = probe_call(...) { return s; }`.

    The Nt* exported functions return NTSTATUS (not Result), so `?` operator
    can't be used. Convert to explicit if-let-Err pattern.
    This also handles multi-line calls and assignment patterns like `let x = call(...)?;`.
    """
    # Pattern: standalone probe/safe_copy call with ? at end of statement
    # Matches: `probe_write_user(args)?;` or `safe_copy_from_user(args)?;`
    PROBE_FUNCS = (
        "probe_read_user",
        "probe_write_user",
        "probe_user_buffer",
        "probe_capture_unicode_string_header",
        "probe_read_ulong",
        "probe_write_ulong",
        "probe_write_handle",
        "probe_write_boolean",
        "probe_write_size_t",
        "safe_copy_from_user",
        "safe_copy_to_user",
    )
    func_alt = "|".join(re.escape(f) for f in PROBE_FUNCS)

    # Fix standalone `call(...)?;` → `if let Err(s) = call(...) { return s; }`
    # Need to handle nested parens in arguments
    lines = code.split("\n")
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.lstrip()

        # Check if line starts with a probe function call (with optional ::<Type>)
        match = re.match(rf"^(\s*)({func_alt})((?:::<[^>]+>)?)\s*(\(.*)", stripped)
        if match:
            indent = line[: len(line) - len(stripped)]
            func_name = match.group(2) + match.group(3)  # includes ::<Type> if present
            rest = match.group(4)

            # Collect full call (may span multiple lines)
            full_call = rest
            j = i + 1
            while full_call.count("(") > full_call.count(")") and j < len(lines):
                full_call += "\n" + lines[j]
                j += 1

            # Check if it ends with `?;`
            call_stripped = full_call.rstrip()
            if call_stripped.endswith("?;"):
                # Remove trailing `?;` and get the args
                call_body = call_stripped[:-2].rstrip()
                result.append(f"{indent}if let Err(s) = {func_name}{call_body} {{")
                result.append(f"{indent}    return s;")
                result.append(f"{indent}}}")
                i = j
                continue

        # Check for `let x = call(...)?;` pattern → `let x = call(...).map_err(|s| return s)?;`
        # Actually, for `let x = probe_read_user()?;` this IS valid in Result-returning fns,
        # but not in NTSTATUS-returning fns. Convert to `match`.
        assign_match = re.match(
            rf"^(\s*)(let\s+(?:mut\s+)?\w+)\s*=\s*({func_alt})((?:::<[^>]+>)?)\s*(\(.*)", stripped
        )
        if assign_match:
            indent = line[: len(line) - len(stripped)]
            let_part = assign_match.group(2)
            func_name = assign_match.group(3) + assign_match.group(4)
            rest = assign_match.group(5)

            full_call = rest
            j = i + 1
            while full_call.count("(") > full_call.count(")") and j < len(lines):
                full_call += "\n" + lines[j]
                j += 1

            call_stripped = full_call.rstrip()
            if call_stripped.endswith("?;"):
                call_body = call_stripped[:-2].rstrip()
                result.append(f"{indent}{let_part} = match {func_name}{call_body} {{")
                result.append(f"{indent}    Ok(v) => v,")
                result.append(f"{indent}    Err(s) => return s,")
                result.append(f"{indent}}};")
                i = j
                continue

        result.append(line)
        i += 1

    return "\n".join(result)


def _fix_convenience_wrapper_args(code: str) -> str:
    """Fix probe_write_ulong(ptr, val, mode) (3 args) → probe_write_user::<ULONG>(ptr, val, mode).

    The convenience wrappers (probe_write_ulong, probe_write_handle, etc.) only take
    2 args (ptr, mode) and do a read-then-write-back touch-test. When the LLM wants to
    write a specific value, it should use probe_write_user::<T> instead.
    """
    # Map convenience wrapper to the correct type
    WRAPPER_MAP = {
        "probe_write_ulong": "ULONG",
        "probe_write_handle": "PVOID",
        "probe_write_boolean": "BOOLEAN",
        "probe_write_size_t": "ULONG_PTR",
        "probe_read_ulong": "ULONG",
    }

    for wrapper, typ in WRAPPER_MAP.items():
        # Match 3-arg calls: wrapper(a, b, c)
        # The wrapper only takes 2 args, so 3 args means the LLM confused it
        # with probe_write_user/probe_read_user
        pattern = rf"\b{re.escape(wrapper)}\s*\(\s*([^,]+),\s*([^,]+),\s*([^)]+)\)"
        replacement = (
            rf"probe_write_user::<{typ}>(\1, \2, \3)"
            if "write" in wrapper
            else rf"probe_read_user::<{typ}>(\1, \2, \3)"
        )
        code = re.sub(pattern, replacement, code)

    return code


def postprocess_probe(code: str, original_source: str) -> str:
    """Apply mechanical fixes to LLM SEH migration output.

    Safe transforms only:
    1. Ensure `use ntkrnl_sys::probe::*;` is present
    2. Remove leftover probe extern declarations
    3. Fix `?` operator in NTSTATUS-returning functions
    4. Fix convenience wrapper 3-arg misuse
    5. Re-append the trailing C source comments if they existed
    """
    # 1. Ensure probe import exists
    if "use ntkrnl_sys::probe::*;" not in code:
        last_use = -1
        for m in re.finditer(r"^use\s+\S+;", code, re.MULTILINE):
            last_use = m.end()
        if last_use >= 0:
            code = code[:last_use] + "\nuse ntkrnl_sys::probe::*;" + code[last_use:]
        else:
            code = "use ntkrnl_sys::probe::*;\n" + code

    # 2. Remove leftover ProbeForRead/ProbeForWrite extern declarations
    code = re.sub(
        r"^\s*fn\s+(?:ProbeForRead|ProbeForWrite)\s*\([^)]*\)\s*(?:->\s*\S+\s*)?;\s*\n",
        "",
        code,
        flags=re.MULTILINE,
    )

    # Remove typed probe helper extern declarations
    for helper in [
        "ProbeForWriteHandle",
        "ProbeForWriteUlong",
        "ProbeForWriteBoolean",
        "ProbeForReadUnicodeString",
        "ProbeForReadLargeInteger",
        "ProbeForReadUchar",
        "ProbeForReadUshort",
        "ProbeAndZeroHandle",
    ]:
        code = re.sub(
            rf"^\s*fn\s+{re.escape(helper)}\s*\([^)]*\)\s*(?:->\s*[^;]+)?\s*;\s*\n",
            "",
            code,
            flags=re.MULTILINE,
        )

    # 3. Clean up empty extern blocks
    code = re.sub(
        r'extern\s+"(?:system|C)"\s*\{\s*\}\s*\n?',
        "",
        code,
    )

    # 4. Fix convenience wrapper 3-arg misuse (e.g. probe_write_ulong(ptr, val, mode))
    code = _fix_convenience_wrapper_args(code)

    # 5. Fix `?` operator → `if let Err(s) = ... { return s; }` for NTSTATUS fns
    code = _fix_question_mark_operator(code)

    # 6. Re-append C source comments from original if they existed
    c_source_marker = "// ORIGINAL C SOURCE"
    marker_pos = original_source.find(c_source_marker)
    if marker_pos >= 0:
        block_start = original_source.rfind("// ====", 0, marker_pos)
        if block_start >= 0:
            c_comment_block = original_source[block_start:]
            if c_source_marker not in code:
                code = code.rstrip() + "\n\n" + c_comment_block

    return code


# ============================================================================
# Pipeline: scan → preprocess → LLM → postprocess → write
# ============================================================================


def run_probe_migration(
    files: list[Path],
    model: str,
    system_prompt: str = "",
    dry_run: bool = False,
) -> dict[str, bool]:
    """Run the full SEH migration pipeline on a list of Rust files.

    Returns dict of filename → success.
    """
    if not system_prompt:
        system_prompt = load_probe_system_prompt()

    results: dict[str, bool] = {}
    total = len(files)

    for i, filepath in enumerate(files, 1):
        scan = scan_rust_file(filepath)
        file_prompt = build_probe_file_prompt(scan)

        label_parts = []
        if scan.probe_call_count > 0:
            label_parts.append(f"{scan.probe_call_count} probes")
        if scan.seh_context:
            # Count SEH functions in context
            seh_funcs = scan.seh_context.count("Function: ")
            label_parts.append(f"{seh_funcs} SEH fns")
        label = ", ".join(label_parts) or "no targets"

        print(f"  [{i}/{total}] {filepath.name} ({label})...", file=sys.stderr)

        result = submit_single(
            file_prompt,
            model=model,
            system_prompt=system_prompt,
        )

        if not result.success:
            print(f"    FAIL: {result.error}", file=sys.stderr)
            results[filepath.name] = False
            continue

        print(f"    {result.input_tokens} in / {result.output_tokens} out tokens", file=sys.stderr)

        # Postprocess
        migrated = postprocess_probe(result.rust_code, scan.raw_source)

        if dry_run:
            old_lines = scan.raw_source.count("\n")
            new_lines = migrated.count("\n")
            print(f"    Would write {new_lines} lines (was {old_lines})", file=sys.stderr)
        else:
            filepath.write_text(migrated)
            print(f"    Written to {filepath}", file=sys.stderr)

        results[filepath.name] = True

    return results
