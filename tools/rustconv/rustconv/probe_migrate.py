"""Probe API migration pipeline for SE/OB Rust files.

Scans Rust files for old ProbeForRead/ProbeForWrite patterns, builds
migration prompts, submits to LLM, and applies postprocessing.

Usage:
    python -m rustconv probe-migrate src/se/ -m sonnet
    python -m rustconv probe-migrate src/se/sqos.rs -m haiku --dry-run
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

from .config import OUTPUT_CRATE, TEMPLATES_DIR


# ============================================================================
# Scanner: find probe call sites in Rust files
# ============================================================================

# Patterns that indicate a file needs migration
PROBE_PATTERNS = [
    re.compile(r'\bProbeForRead\b'),
    re.compile(r'\bProbeForWrite\b'),
    re.compile(r'\bProbeForWriteHandle\b'),
    re.compile(r'\bProbeForWriteUlong\b'),
    re.compile(r'\bProbeForWriteBoolean\b'),
    re.compile(r'\bProbeForReadUnicodeString\b'),
    re.compile(r'\bProbeForReadLargeInteger\b'),
    re.compile(r'\bProbeForReadUchar\b'),
    re.compile(r'\bProbeForReadUshort\b'),
    re.compile(r'\bProbeAndZeroHandle\b'),
]

# Extern declaration pattern for probe functions
PROBE_EXTERN_RE = re.compile(
    r'^\s*fn\s+(ProbeForRead|ProbeForWrite|ProbeForWriteHandle|'
    r'ProbeForWriteUlong|ProbeForWriteBoolean|ProbeForReadUnicodeString|'
    r'ProbeForReadLargeInteger|ProbeForReadUchar|ProbeForReadUshort|'
    r'ProbeAndZeroHandle)\b',
    re.MULTILINE,
)


@dataclass
class ProbeScanResult:
    """Result of scanning a Rust file for probe call sites."""
    path: Path
    raw_source: str
    probe_call_count: int = 0          # total probe calls in active code
    probe_extern_count: int = 0        # extern declarations for probe fns
    probe_helper_count: int = 0        # inline helper function definitions
    has_c_source_comment: bool = False  # has trailing C source block
    active_source: str = ""            # source WITHOUT trailing C comments
    mode_variable: str = ""            # detected PreviousMode variable name


def scan_rust_file(path: Path) -> ProbeScanResult:
    """Scan a Rust file for probe patterns."""
    source = path.read_text()
    result = ProbeScanResult(path=path, raw_source=source)

    # Split off trailing C source comments (everything after
    # "// ============================================================================\n// ORIGINAL C SOURCE")
    c_source_marker = "// ORIGINAL C SOURCE"
    marker_pos = source.find(c_source_marker)
    if marker_pos >= 0:
        # Find the line with "// ====..." before it
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
    for name in ['PreviousMode', 'previous_mode', 'AccessMode', 'CurrentMode', 'ProcessorMode']:
        if name in active:
            result.mode_variable = name
            break

    return result


def find_rust_files(directory: Path, subsystems: list[str] | None = None) -> list[Path]:
    """Find Rust files that need probe migration."""
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
                if scan.probe_call_count > 0:
                    files.append(rs_file)

    return files


# ============================================================================
# Preprocessor: strip trailing C source to save tokens
# ============================================================================

def preprocess(source: str) -> str:
    """Strip trailing C source comments to reduce token count.

    The LLM only needs the active Rust code, not the 100+ lines of
    commented-out original C source appended by the assembler.
    """
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
    """Load the probe migration system prompt."""
    path = TEMPLATES_DIR / "probe_migrate_prompt.md"
    return path.read_text()


def build_probe_file_prompt(scan: ProbeScanResult) -> str:
    """Build the per-file prompt for probe migration."""
    parts = []

    parts.append(f"## File: `{scan.path.name}`")
    parts.append(f"Probe calls to migrate: {scan.probe_call_count}")
    if scan.mode_variable:
        parts.append(f"Detected mode variable: `{scan.mode_variable}`")
    parts.append("")

    # Include only the active Rust source (no trailing C comments)
    active = preprocess(scan.raw_source)

    parts.append("### Current Rust Source\n")
    parts.append("Migrate the probe calls in this file:\n")
    parts.append("```rust")
    parts.append(active)
    parts.append("```")

    return "\n".join(parts)


# ============================================================================
# Postprocessor: mechanical fixes after LLM output
# ============================================================================

def postprocess_probe(code: str, original_source: str) -> str:
    """Apply mechanical fixes to LLM probe migration output.

    Safe transforms only:
    1. Ensure `use ntkrnl_sys::probe::*;` is present
    2. Remove leftover probe extern declarations
    3. Re-append the trailing C source comments if they existed
    """
    # 1. Ensure probe import exists
    if "use ntkrnl_sys::probe::*;" not in code:
        # Insert after last existing use statement
        last_use = -1
        for m in re.finditer(r'^use\s+\S+;', code, re.MULTILINE):
            last_use = m.end()
        if last_use >= 0:
            code = code[:last_use] + "\nuse ntkrnl_sys::probe::*;" + code[last_use:]
        else:
            code = "use ntkrnl_sys::probe::*;\n" + code

    # 2. Remove leftover ProbeForRead/ProbeForWrite extern declarations
    #    Pattern: "fn ProbeForRead(...);", "fn ProbeForWrite(...);" inside extern blocks
    code = re.sub(
        r'^\s*fn\s+(?:ProbeForRead|ProbeForWrite)\s*\([^)]*\)\s*(?:->\s*\S+\s*)?;\s*\n',
        '',
        code,
        flags=re.MULTILINE,
    )

    # Remove typed probe helper extern declarations
    for helper in ['ProbeForWriteHandle', 'ProbeForWriteUlong', 'ProbeForWriteBoolean',
                   'ProbeForReadUnicodeString', 'ProbeForReadLargeInteger',
                   'ProbeForReadUchar', 'ProbeForReadUshort', 'ProbeAndZeroHandle']:
        code = re.sub(
            rf'^\s*fn\s+{re.escape(helper)}\s*\([^)]*\)\s*(?:->\s*[^;]+)?\s*;\s*\n',
            '',
            code,
            flags=re.MULTILINE,
        )

    # 3. Clean up empty extern blocks left after removing probe declarations
    # Match extern "system" { \n } or extern "C" { \n } with only whitespace inside
    code = re.sub(
        r'extern\s+"(?:system|C)"\s*\{\s*\}\s*\n?',
        '',
        code,
    )

    # 4. Re-append C source comments from original if they existed
    c_source_marker = "// ORIGINAL C SOURCE"
    marker_pos = original_source.find(c_source_marker)
    if marker_pos >= 0:
        block_start = original_source.rfind("// ====", 0, marker_pos)
        if block_start >= 0:
            c_comment_block = original_source[block_start:]
            # Make sure we don't double-append
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
    """Run the full probe migration pipeline on a list of Rust files.

    Returns dict of filename → success.
    """
    from .batch_runner import submit_single

    if not system_prompt:
        system_prompt = load_probe_system_prompt()

    results: dict[str, bool] = {}
    total = len(files)

    for i, filepath in enumerate(files, 1):
        scan = scan_rust_file(filepath)
        file_prompt = build_probe_file_prompt(scan)

        print(f"  [{i}/{total}] {filepath.name} ({scan.probe_call_count} probe calls)...",
              file=sys.stderr)

        result = submit_single(
            file_prompt,
            model=model,
            system_prompt=system_prompt,
        )

        if not result.success:
            print(f"    FAIL: {result.error}", file=sys.stderr)
            results[filepath.name] = False
            continue

        print(f"    {result.input_tokens} in / {result.output_tokens} out tokens",
              file=sys.stderr)

        # Postprocess
        migrated = postprocess_probe(result.rust_code, scan.raw_source)

        if dry_run:
            # Show diff summary
            old_lines = scan.raw_source.count('\n')
            new_lines = migrated.count('\n')
            print(f"    Would write {new_lines} lines (was {old_lines})", file=sys.stderr)
        else:
            # Write back to the same file
            filepath.write_text(migrated)
            print(f"    Written to {filepath}", file=sys.stderr)

        results[filepath.name] = True

    return results
