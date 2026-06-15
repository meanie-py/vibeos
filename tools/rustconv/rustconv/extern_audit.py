"""Audit extern block declarations vs #[no_mangle] definitions in Rust crate.

Finds calling convention mismatches between:
  - extern "X" { fn Foo(...); }   (how callers import the function)
  - #[no_mangle] pub unsafe extern "Y" fn Foo(...)  (the actual definition)

These are invisible to objdiff (which compares C vs Rust object symbols) but
cause identical stack corruption on i386 when conventions disagree.

Pipeline:
  1. Regex-scan .rs files for extern blocks → extract (name, convention, file, line)
  2. Regex-scan .rs files for #[no_mangle] fn definitions → extract (name, convention, file, line)
  3. Load ntoskrnl.spec via spec_parser → authoritative conventions for exports
  4. Cross-reference: for each extern declaration, find the definition, compare
  5. Report mismatches with severity levels
"""

import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

from .config import OUTPUT_CRATE
from .spec_parser import parse_spec

# ── Data structures ──────────────────────────────────────────────────────────


@dataclass
class ExternDecl:
    """A function declared inside an extern block."""

    name: str
    convention: str  # "system" | "C" | "fastcall" | "stdcall"
    file: str  # Relative path from src/
    line: int
    cfg: str = ""  # cfg guard if any (e.g., 'target_arch = "x86"')


@dataclass
class FnDefinition:
    """A #[no_mangle] function definition."""

    name: str
    convention: str  # "system" | "C" | "fastcall" | "stdcall"
    file: str
    line: int
    cfg: str = ""


@dataclass
class AuditMismatch:
    decl: ExternDecl
    defn: FnDefinition | None
    spec_convention: str | None  # from ntoskrnl.spec
    severity: str  # "critical" | "warning" | "info"
    detail: str


@dataclass
class AuditReport:
    total_files: int
    total_decls: int
    total_blocks: int
    total_defs: int
    mismatches: list[AuditMismatch]
    intra_crate_count: int  # extern decl for a function also defined in crate


# ── Regex patterns ───────────────────────────────────────────────────────────

# Matches an extern "ABI" { ... } block.  We use a state-machine parser instead
# of a single DOTALL regex because blocks can be nested in cfg and can contain
# nested braces in function signatures (e.g., fn pointers).

# Match #[cfg(...)] immediately before an extern block
RE_CFG = re.compile(r"#\[cfg\((.+?)\)\]\s*$")

# Match extern "ABI" { at line start (possibly after whitespace)
RE_EXTERN_OPEN = re.compile(r'^\s*extern\s+"(\w+)"\s*\{')

# Match fn declarations inside extern blocks: fn Name(
RE_FN_DECL = re.compile(r"\bfn\s+(\w+)\s*\(")

# Match #[no_mangle] pub [unsafe] extern "ABI" fn Name(
# Handles optional attributes between #[no_mangle] and fn (up to 5 lines)
RE_NO_MANGLE_DEF = re.compile(
    r"#\[no_mangle\]"
    r"[\s\S]*?"
    r'pub\s+(?:unsafe\s+)?extern\s+"(\w+)"\s+fn\s+(\w+)',
)

# Match #[cfg(...)] on the line immediately before a #[no_mangle] definition
RE_CFG_DEF = re.compile(r"#\[cfg\((.+?)\)\]")


# ── Scanning ─────────────────────────────────────────────────────────────────


def scan_extern_decls(rs_path: Path, rel_path: str) -> tuple[list[ExternDecl], int]:
    """Parse a .rs file for extern block function declarations.

    Returns (list of ExternDecl, number of extern blocks found).
    """
    text = rs_path.read_text()
    lines = text.splitlines()
    decls: list[ExternDecl] = []
    block_count = 0
    pending_cfg = ""

    i = 0
    while i < len(lines):
        line = lines[i]

        # Track cfg guards
        cfg_m = RE_CFG.search(line.rstrip())
        if cfg_m and "extern" not in line:
            pending_cfg = cfg_m.group(1)
            i += 1
            continue

        # Look for extern "ABI" {
        m = RE_EXTERN_OPEN.match(line)
        if m:
            convention = m.group(1)
            block_count += 1
            block_cfg = pending_cfg
            pending_cfg = ""

            # Find all fn declarations inside this block
            brace_depth = 1
            i += 1
            while i < len(lines) and brace_depth > 0:
                bline = lines[i]
                brace_depth += bline.count("{") - bline.count("}")
                if brace_depth <= 0:
                    break
                # Look for fn declarations
                for fn_m in RE_FN_DECL.finditer(bline):
                    decls.append(
                        ExternDecl(
                            name=fn_m.group(1),
                            convention=convention,
                            file=rel_path,
                            line=i + 1,
                            cfg=block_cfg,
                        )
                    )
                i += 1
        else:
            pending_cfg = ""
            i += 1

    return decls, block_count


def scan_fn_definitions(rs_path: Path, rel_path: str) -> list[FnDefinition]:
    """Parse a .rs file for #[no_mangle] function definitions."""
    text = rs_path.read_text()
    lines = text.splitlines()
    defs: list[FnDefinition] = []

    for m in RE_NO_MANGLE_DEF.finditer(text):
        convention = m.group(1)
        name = m.group(2)
        # Find line number
        line_num = text[: m.start()].count("\n") + 1

        # Check for cfg guard in the 3 lines before #[no_mangle]
        cfg = ""
        no_mangle_line = text[: m.start()].count("\n")
        for look_back in range(1, 4):
            check_line = no_mangle_line - look_back
            if check_line >= 0 and check_line < len(lines):
                cfg_m = RE_CFG_DEF.search(lines[check_line])
                if cfg_m:
                    cfg = cfg_m.group(1)
                    break
                # Stop looking if we hit a non-attribute, non-empty line
                stripped = lines[check_line].strip()
                if stripped and not stripped.startswith("#[") and not stripped.startswith("//"):
                    break

        defs.append(
            FnDefinition(
                name=name,
                convention=convention,
                file=rel_path,
                line=line_num,
                cfg=cfg,
            )
        )

    return defs


# ── Convention normalization ─────────────────────────────────────────────────


def normalize_convention(conv: str) -> str:
    """Normalize Rust ABI string to a canonical form for comparison.

    On i386: "system" = stdcall, "C" = cdecl, "fastcall" = fastcall
    On amd64/arm64: "system" = "C" = cdecl (all identical)

    For mismatch detection we compare the raw strings — they matter on i386.
    """
    return conv.lower()


def conventions_match(decl_conv: str, def_conv: str, cfg: str = "") -> bool:
    """Check if two convention strings are compatible.

    "system" and "stdcall" are equivalent.
    "C" and "cdecl" are equivalent.
    On non-x86 (cfg excludes x86), "fastcall" == "C" == "system" (all are cdecl ABI).
    Everything else must match exactly.
    """
    a = decl_conv.lower()
    b = def_conv.lower()

    if a == b:
        return True

    # system == stdcall
    equiv = {frozenset({"system", "stdcall"}), frozenset({"c", "cdecl"})}
    if frozenset({a, b}) in equiv:
        return True

    # On non-x86 platforms, fastcall/stdcall/system/C are all equivalent (cdecl
    # ABI) — all conventions collapse to cdecl on non-x86.
    return bool(cfg and 'not(target_arch = "x86")' in cfg)


def spec_to_rust_convention(spec_conv: str) -> str:
    """Map ntoskrnl.spec convention to Rust extern string (without quotes)."""
    mapping = {
        "stdcall": "system",
        "fastcall": "fastcall",
        "cdecl": "C",
        "varargs": "C",
    }
    return mapping.get(spec_conv, "C")


# ── Cross-reference engine ───────────────────────────────────────────────────


def cross_reference(
    all_decls: list[ExternDecl],
    all_defs: list[FnDefinition],
    spec: dict | None = None,
) -> AuditReport:
    """Cross-reference declarations against definitions and spec."""

    # Build lookup: name → list of definitions (can have cfg variants)
    def_map: dict[str, list[FnDefinition]] = {}
    for d in all_defs:
        def_map.setdefault(d.name, []).append(d)

    # Build lookup: name → list of declarations
    decl_map: dict[str, list[ExternDecl]] = {}
    for d in all_decls:
        decl_map.setdefault(d.name, []).append(d)

    mismatches: list[AuditMismatch] = []
    intra_crate = 0
    seen_pairs: set[tuple[str, str, str, str]] = set()  # (name, decl_file, def_file, decl_cfg)

    for decl in all_decls:
        defs_for_name = def_map.get(decl.name)
        if not defs_for_name:
            # Not defined in this crate — probably in ntkrnl-sys or C. Fine.
            continue

        intra_crate += 1

        # Find the best matching definition considering cfg guards
        best_def = _find_matching_def(decl, defs_for_name)
        if not best_def:
            # All definitions are for different cfg targets
            continue

        # Deduplicate: same name + same file pair + same cfg → report once
        pair_key = (decl.name, decl.file, best_def.file, decl.cfg)
        if pair_key in seen_pairs:
            continue
        seen_pairs.add(pair_key)

        # Check convention match
        spec_conv = None
        if spec and decl.name in spec:
            spec_conv = spec_to_rust_convention(spec[decl.name].convention)

        if not conventions_match(decl.convention, best_def.convention, cfg=decl.cfg):
            # The correct convention (spec > definition) is conveyed via
            # spec_convention + detail on the AuditMismatch below.
            mismatches.append(
                AuditMismatch(
                    decl=decl,
                    defn=best_def,
                    spec_convention=spec_conv,
                    severity="critical",
                    detail=(
                        f'declared "{decl.convention}", defined "{best_def.convention}"'
                        + (f', spec says "{spec_conv}"' if spec_conv else "")
                    ),
                )
            )
        elif spec_conv and not conventions_match(best_def.convention, spec_conv):
            # Definition doesn't match spec — the declaration and definition agree
            # with each other but both disagree with spec
            mismatches.append(
                AuditMismatch(
                    decl=decl,
                    defn=best_def,
                    spec_convention=spec_conv,
                    severity="warning",
                    detail=(
                        f'declared "{decl.convention}", defined "{best_def.convention}", '
                        f'but spec says "{spec_conv}"'
                    ),
                )
            )

    # Count unique intra-crate function names
    intra_names = set()
    for decl in all_decls:
        if decl.name in def_map:
            intra_names.add(decl.name)

    return AuditReport(
        total_files=0,  # filled by caller
        total_decls=len(all_decls),
        total_blocks=0,  # filled by caller
        total_defs=len(all_defs),
        mismatches=mismatches,
        intra_crate_count=len(intra_names),
    )


def _find_matching_def(decl: ExternDecl, defs: list[FnDefinition]) -> FnDefinition | None:
    """Find the definition that matches the declaration's cfg context.

    If decl has cfg 'target_arch = "x86"', prefer a def with the same cfg.
    If no cfg match, fall back to any definition without cfg.
    """
    # Exact cfg match
    for d in defs:
        if d.cfg == decl.cfg:
            return d

    # Decl has cfg but no def matches — find un-gated def
    for d in defs:
        if not d.cfg:
            return d

    # No exact match, return first
    return defs[0] if defs else None


# ── Report output ────────────────────────────────────────────────────────────


def print_audit_report(report: AuditReport) -> None:
    """Print human-readable audit report to stderr."""
    out = sys.stderr

    print("\nextern-audit: Cross-referencing declarations vs definitions", file=out)
    print(f"  Files scanned: {report.total_files}", file=out)
    print(f"  Extern declarations: {report.total_decls} in {report.total_blocks} blocks", file=out)
    print(f"  #[no_mangle] definitions: {report.total_defs}", file=out)
    print(
        f"  Intra-crate declarations: {report.intra_crate_count} "
        f"(declared AND defined in same crate)",
        file=out,
    )
    print(file=out)

    critical = [m for m in report.mismatches if m.severity == "critical"]
    warnings = [m for m in report.mismatches if m.severity == "warning"]

    if critical:
        print(f"CRITICAL ({len(critical)}):", file=out)
        for m in critical:
            defn_loc = f"{m.defn.file}:{m.defn.line}" if m.defn else "???"
            print(f"  {m.decl.file}:{m.decl.line} -> {defn_loc}", file=out)
            print(f"    {m.decl.name}: {m.detail}", file=out)
        print(file=out)

    if warnings:
        print(f"WARNING ({len(warnings)}):", file=out)
        for m in warnings:
            defn_loc = f"{m.defn.file}:{m.defn.line}" if m.defn else "???"
            print(f"  {m.decl.file}:{m.decl.line} -> {defn_loc}", file=out)
            print(f"    {m.decl.name}: {m.detail}", file=out)
        print(file=out)

    # Summary
    print(
        f"Summary: {len(critical)} critical, {len(warnings)} warnings, "
        f"{report.intra_crate_count} intra-crate declarations",
        file=out,
    )


def report_to_json(report: AuditReport) -> str:
    """Convert audit report to JSON."""
    data = {
        "total_files": report.total_files,
        "total_decls": report.total_decls,
        "total_blocks": report.total_blocks,
        "total_defs": report.total_defs,
        "intra_crate_count": report.intra_crate_count,
        "mismatches": [
            {
                "severity": m.severity,
                "name": m.decl.name,
                "decl_file": m.decl.file,
                "decl_line": m.decl.line,
                "decl_convention": m.decl.convention,
                "def_file": m.defn.file if m.defn else None,
                "def_line": m.defn.line if m.defn else None,
                "def_convention": m.defn.convention if m.defn else None,
                "spec_convention": m.spec_convention,
                "detail": m.detail,
            }
            for m in report.mismatches
        ],
        "summary": {
            "critical": sum(1 for m in report.mismatches if m.severity == "critical"),
            "warning": sum(1 for m in report.mismatches if m.severity == "warning"),
        },
    }
    return json.dumps(data, indent=2)


# ── Main pipeline ────────────────────────────────────────────────────────────


def run_extern_audit(json_output: bool = False) -> AuditReport:
    """Run the full extern audit pipeline."""
    out = sys.stderr
    src_dir = OUTPUT_CRATE / "src"

    if not src_dir.exists():
        print(f"Error: ntoskrnl-rust/src/ not found at {src_dir}", file=out)
        sys.exit(1)

    # Find all .rs files
    rs_files = sorted(src_dir.rglob("*.rs"))
    # Exclude mod.rs and lib.rs — they don't contain function bodies
    rs_files = [f for f in rs_files if f.name not in ("mod.rs", "lib.rs")]
    print(f"Scanning {len(rs_files)} .rs files...", file=out, flush=True)

    all_decls: list[ExternDecl] = []
    all_defs: list[FnDefinition] = []
    total_blocks = 0

    for rs_file in rs_files:
        rel = str(rs_file.relative_to(src_dir))
        decls, blocks = scan_extern_decls(rs_file, rel)
        defs = scan_fn_definitions(rs_file, rel)
        all_decls.extend(decls)
        all_defs.extend(defs)
        total_blocks += blocks

    print(f"  {len(all_decls)} extern declarations in {total_blocks} blocks", file=out)
    print(f"  {len(all_defs)} #[no_mangle] definitions", file=out)

    # Load spec
    spec = None
    try:
        spec = parse_spec()
        print(f"  Loaded {len(spec)} spec entries", file=out)
    except Exception as e:
        print(f"  Warning: could not load spec: {e}", file=out)

    # Cross-reference
    print("  Cross-referencing...", file=out, flush=True)
    report = cross_reference(all_decls, all_defs, spec)
    report.total_files = len(rs_files)
    report.total_blocks = total_blocks

    # Output
    if json_output:
        print(report_to_json(report))
    else:
        print_audit_report(report)

    return report


# ── Automated fixer ──────────────────────────────────────────────────────────


@dataclass
class _ExternBlock:
    """Parsed extern block with line range."""

    convention: str  # "system" | "C" | "fastcall" | "stdcall"
    start_line: int  # Line number of 'extern "X" {' (0-indexed)
    end_line: int  # Line number of closing '}' (0-indexed)
    cfg: str  # cfg guard on the line before, if any
    fn_lines: dict  # {fn_name: line_index} for functions in this block


def _parse_extern_blocks(lines: list[str]) -> list[_ExternBlock]:
    """Parse all extern blocks in a file, tracking their line ranges and contents."""
    blocks: list[_ExternBlock] = []
    i = 0
    while i < len(lines):
        line = lines[i]

        # Check for cfg guard before extern
        cfg = ""
        cfg_m = RE_CFG.search(line.rstrip())
        if cfg_m and "extern" not in line:
            cfg = cfg_m.group(1)
            # Peek ahead for extern block
            if i + 1 < len(lines):
                next_line = lines[i + 1]
                ext_m = RE_EXTERN_OPEN.match(next_line)
                if ext_m:
                    convention = ext_m.group(1)
                    start_line = i  # Include the cfg line
                    # Find matching close brace
                    brace_depth = 1
                    fn_lines = {}
                    j = i + 2
                    while j < len(lines) and brace_depth > 0:
                        brace_depth += lines[j].count("{") - lines[j].count("}")
                        if brace_depth > 0:
                            for fn_m in RE_FN_DECL.finditer(lines[j]):
                                fn_lines[fn_m.group(1)] = j
                        j += 1
                    blocks.append(
                        _ExternBlock(
                            convention=convention,
                            start_line=start_line,
                            end_line=j - 1,
                            cfg=cfg,
                            fn_lines=fn_lines,
                        )
                    )
                    i = j
                    continue
            i += 1
            continue

        # Check for extern block without cfg
        ext_m = RE_EXTERN_OPEN.match(line)
        if ext_m:
            convention = ext_m.group(1)
            start_line = i
            brace_depth = 1
            fn_lines = {}
            j = i + 1
            while j < len(lines) and brace_depth > 0:
                brace_depth += lines[j].count("{") - lines[j].count("}")
                if brace_depth > 0:
                    for fn_m in RE_FN_DECL.finditer(lines[j]):
                        fn_lines[fn_m.group(1)] = j
                j += 1
            blocks.append(
                _ExternBlock(
                    convention=convention,
                    start_line=start_line,
                    end_line=j - 1,
                    cfg="",
                    fn_lines=fn_lines,
                )
            )
            i = j
            continue

        i += 1

    return blocks


def _extract_fn_decl(lines: list[str], fn_line: int) -> tuple[str, int, int]:
    """Extract a full function declaration that might span multiple lines.

    Returns (declaration_text, first_line, last_line) where lines are 0-indexed.
    """
    # Start from fn_line, go forward until we find the closing semicolon
    first_line = fn_line
    last_line = fn_line

    # The declaration might have a comment on the preceding line
    text = lines[fn_line]
    if ";" in text:
        return text, first_line, last_line

    # Multi-line declaration
    j = fn_line + 1
    while j < len(lines):
        text += "\n" + lines[j]
        last_line = j
        if ";" in lines[j]:
            break
        j += 1

    return text, first_line, last_line


def _determine_target_convention(mismatch: AuditMismatch) -> str:
    """Determine the correct convention for a mismatched declaration.

    Priority: spec > definition.
    Returns "fastcall", "system", or "C".
    """
    if not mismatch.defn:
        return mismatch.decl.convention

    def_conv = mismatch.defn.convention
    spec_conv = mismatch.spec_convention

    # If spec agrees with definition, use that
    if spec_conv and conventions_match(def_conv, spec_conv):
        return def_conv

    # If spec disagrees with definition, prefer spec for exported functions
    if spec_conv and not conventions_match(def_conv, spec_conv):
        return spec_conv

    # No spec — use definition
    return def_conv


def fix_file(file_path: Path, mismatches: list[AuditMismatch], dry_run: bool = False) -> int:
    """Fix extern declaration mismatches in a single file.

    Returns the number of fixes applied.
    """
    text = file_path.read_text()
    lines = text.splitlines()
    blocks = _parse_extern_blocks(lines)

    # Group mismatches by target convention
    fixes: list[tuple[str, str, int, str]] = []  # (fn_name, target_conv, decl_line, full_decl)

    for m in mismatches:
        target = _determine_target_convention(m)
        decl_line = m.decl.line - 1  # Convert to 0-indexed

        # Find the block containing this declaration
        found_block = None
        for block in blocks:
            if m.decl.name in block.fn_lines and abs(block.fn_lines[m.decl.name] - decl_line) <= 2:
                found_block = block
                break

        if not found_block:
            continue

        # Already correct convention?
        if conventions_match(found_block.convention, target):
            continue

        # Extract the full declaration text
        actual_line = found_block.fn_lines[m.decl.name]
        full_decl, first, last = _extract_fn_decl(lines, actual_line)

        # Determine visibility prefix: check if any function in this block has 'pub'
        is_pub = "pub fn" in full_decl or "pub static" in full_decl  # noqa: F841  # TODO: visibility prefix is computed but never applied when rebuilding the extern block below (drops `pub`)

        fixes.append((m.decl.name, target, actual_line, full_decl.strip()))

    if not fixes:
        return 0

    # Apply fixes: remove lines, then add new blocks
    lines_to_remove: set[int] = set()
    new_blocks: list[str] = []  # New extern blocks to append after the last extern block

    for _fn_name, target_conv, decl_line, full_decl in fixes:
        # Mark lines for removal
        _, first, last = _extract_fn_decl(lines, decl_line)
        for i in range(first, last + 1):
            lines_to_remove.add(i)

        # Clean up the declaration text (remove leading whitespace)
        decl_clean = full_decl.strip()

        # Build the new extern block
        if target_conv == "fastcall":
            # Need cfg-gated blocks
            new_blocks.append(
                f'#[cfg(target_arch = "x86")]\n'
                f'extern "fastcall" {{\n'
                f"    {decl_clean}\n"
                f"}}\n"
                f'#[cfg(not(target_arch = "x86"))]\n'
                f'extern "C" {{\n'
                f"    {decl_clean}\n"
                f"}}"
            )
        else:
            new_blocks.append(f'extern "{target_conv}" {{\n    {decl_clean}\n}}')

    # Find the insertion point: after the last extern block in the file
    last_block_end = 0
    for block in blocks:
        if block.end_line > last_block_end:
            last_block_end = block.end_line

    # Build new file content
    new_lines: list[str] = []
    for i, line in enumerate(lines):
        if i in lines_to_remove:
            continue
        new_lines.append(line)

        # After the last extern block's closing brace, insert new blocks
        if i == last_block_end and new_blocks:
            for nb in new_blocks:
                new_lines.append(nb)

    # Clean up: remove empty extern blocks (blocks that now only have { })
    result = "\n".join(new_lines)
    # Remove empty blocks: extern "X" {\n}\n or extern "X" {\n\n}\n
    result = re.sub(r'extern\s+"[^"]+"\s*\{\s*\}\n?', "", result)
    # Also clean up cfg-guarded empty blocks
    result = re.sub(r'#\[cfg\([^\)]+\)\]\s*extern\s+"[^"]+"\s*\{\s*\}\n?', "", result)
    # Clean up excessive blank lines (3+ → 2)
    result = re.sub(r"\n{3,}", "\n\n", result)

    if not dry_run:
        file_path.write_text(result)

    return len(fixes)


def run_extern_fix(dry_run: bool = False) -> int:
    """Run extern audit and fix all critical mismatches.

    Returns total number of fixes applied.
    """
    out = sys.stderr

    # First, run the audit
    print("Running extern audit...", file=out, flush=True)
    report = run_extern_audit(json_output=False)

    critical = [m for m in report.mismatches if m.severity == "critical"]
    if not critical:
        print("\nNo critical mismatches to fix.", file=out)
        return 0

    print(f"\nFixing {len(critical)} critical mismatches...", file=out, flush=True)

    # Group by declaring file
    src_dir = OUTPUT_CRATE / "src"
    by_file: dict[str, list[AuditMismatch]] = {}
    for m in critical:
        by_file.setdefault(m.decl.file, []).append(m)

    total_fixed = 0
    for rel_path, file_mismatches in sorted(by_file.items()):
        file_path = src_dir / rel_path
        if not file_path.exists():
            print(f"  {rel_path}: SKIP (file not found)", file=out)
            continue

        count = fix_file(file_path, file_mismatches, dry_run=dry_run)
        if count > 0:
            verb = "would fix" if dry_run else "fixed"
            print(f"  {rel_path}: {verb} {count} declarations", file=out)
            total_fixed += count

    verb = "would fix" if dry_run else "fixed"
    print(f"\nTotal: {verb} {total_fixed} declarations in {len(by_file)} files", file=out)

    if not dry_run:
        # Re-run audit to verify
        print("\nRe-running audit to verify...", file=out, flush=True)
        report2 = run_extern_audit(json_output=False)
        remaining = sum(1 for m in report2.mismatches if m.severity == "critical")
        if remaining > 0:
            print(f"\n⚠ {remaining} critical mismatches remain (may need manual review)", file=out)
        else:
            print("\nAll critical mismatches resolved.", file=out)

    return total_fixed
