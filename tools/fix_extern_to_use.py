#!/usr/bin/env python3
"""Convert Rust-to-Rust extern declarations to proper `use` imports.

Many functions in ntoskrnl-rust are defined with #[no_mangle] in one module
but called via `extern { fn Foo(); }` declarations in other modules — a C
translation artifact. With CGU=1, LLVM can see both sides and may detect
ABI mismatches (extern "C" vs extern "system") causing ud2.

This script finds these Rust-to-Rust extern calls and replaces them with
proper `use crate::module::Function` imports.

Usage:
    python3 tools/fix_extern_to_use.py --scan          # report only
    python3 tools/fix_extern_to_use.py --fix            # apply fixes
    python3 tools/fix_extern_to_use.py --scan --file dbgk/dbgkobj.rs  # single file
"""

import re
import sys
from collections import defaultdict
from pathlib import Path

SRC_DIR = Path("sdk/lib/rust/ntoskrnl-rust/src")
SKIP_DIRS = {"old_translations"}
SKIP_FILES = {"shim_exports.rs", "global_barrier.rs"}


def find_definitions(src_dir: Path) -> dict[str, str]:
    """Find all #[no_mangle] pub unsafe extern fn definitions.

    Returns {function_name: "crate::module::path::function_name"}.
    """
    defs = {}

    for rs_file in sorted(src_dir.rglob("*.rs")):
        if any(s in str(rs_file) for s in SKIP_DIRS):
            continue
        if rs_file.name in SKIP_FILES:
            continue
        if ".bak" in rs_file.name:
            continue

        try:
            source = rs_file.read_text()
        except Exception:
            continue

        # Match: #[no_mangle] ... pub unsafe extern "..." fn FunctionName(
        # The #[no_mangle] might be several lines before the fn
        for match in re.finditer(
            r"#\[no_mangle\]\s*(?:#\[.*?\]\s*)*"
            r'pub\s+unsafe\s+extern\s+"[^"]+"\s+fn\s+(\w+)',
            source,
            re.DOTALL,
        ):
            fn_name = match.group(1)
            # Convert file path to module path
            rel = rs_file.relative_to(src_dir)
            parts = list(rel.parts)
            # Remove .rs extension from last part
            parts[-1] = parts[-1].replace(".rs", "")
            # Handle mod.rs → parent module
            if parts[-1] == "mod":
                parts = parts[:-1]
            # Handle lib.rs → crate root
            if parts[-1] == "lib":
                parts = parts[:-1]

            # Handle Rust reserved keywords in module names
            RESERVED = {
                "virtual",
                "type",
                "fn",
                "mod",
                "use",
                "self",
                "super",
                "crate",
                "as",
                "break",
                "const",
                "continue",
                "else",
                "enum",
                "extern",
                "false",
                "for",
                "if",
                "impl",
                "in",
                "let",
                "loop",
                "match",
                "move",
                "mut",
                "pub",
                "ref",
                "return",
                "static",
                "struct",
                "trait",
                "true",
                "unsafe",
                "where",
                "while",
                "async",
                "await",
                "dyn",
            }
            parts = [f"r#{p}" if p in RESERVED else p for p in parts]

            module_path = "crate::" + "::".join(parts) + "::" + fn_name

            # Don't override — first definition wins
            if fn_name not in defs:
                defs[fn_name] = module_path

    return defs


def find_extern_declarations(rs_file: Path) -> list[dict]:
    """Find extern { fn Foo(); } declarations in a file.

    Returns list of {name, line, block_start, block_end, convention, link_name}.
    """
    source = rs_file.read_text()
    lines = source.splitlines()
    declarations = []

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Match extern block: extern "C" { or extern "system" {
        extern_match = re.match(r'^(?:#\[.*?\]\s*)*(?:unsafe\s+)?extern\s+"(\w+)"\s*\{', line)
        if extern_match:
            convention = extern_match.group(1)
            block_start = i
            brace_count = line.count("{") - line.count("}")

            j = i + 1
            while j < len(lines) and brace_count > 0:
                brace_count += lines[j].count("{") - lines[j].count("}")

                fn_line = lines[j].strip()
                # Match function declaration: fn FunctionName(...)
                fn_match = re.match(
                    r'(?:#\[link_name\s*=\s*"(\w+)"\]\s*)?(?:pub\s+)?fn\s+(\w+)\s*\(', fn_line
                )
                if fn_match:
                    link_name = fn_match.group(1)  # might be None
                    fn_name = fn_match.group(2)
                    # Skip variadic functions (can't be imported via use)
                    if "..." not in fn_line:
                        declarations.append(
                            {
                                "name": fn_name,
                                "link_name": link_name,
                                "line": j + 1,
                                "block_start": block_start + 1,
                                "block_end": None,  # filled in when block closes
                                "convention": convention,
                            }
                        )

                j += 1

            # Set block_end for all declarations in this block
            for d in declarations:
                if d["block_end"] is None:
                    d["block_end"] = j + 1

        i += 1

    return declarations


def scan_file(rs_file: Path, definitions: dict[str, str]) -> list[dict]:
    """Find Rust-to-Rust extern declarations that could be use imports."""
    decls = find_extern_declarations(rs_file)

    replaceable = []
    for d in decls:
        # Check if this function has a Rust definition
        # Use link_name if present, otherwise function name
        lookup_name = d["link_name"] or d["name"]
        if lookup_name in definitions:
            # Also check the declared name (in case link_name maps to a different name)
            d["import_path"] = definitions[lookup_name]
            d["file"] = str(rs_file)
            replaceable.append(d)
        elif d["name"] in definitions:
            d["import_path"] = definitions[d["name"]]
            d["file"] = str(rs_file)
            replaceable.append(d)

    return replaceable


def scan_all(
    src_dir: Path, definitions: dict[str, str], target_file: str | None = None
) -> list[dict]:
    """Scan all files for replaceable extern declarations."""
    all_replaceable = []

    for rs_file in sorted(src_dir.rglob("*.rs")):
        if any(s in str(rs_file) for s in SKIP_DIRS):
            continue
        if rs_file.name in SKIP_FILES:
            continue
        if ".bak" in rs_file.name:
            continue
        if target_file and target_file not in str(rs_file):
            continue

        replaceable = scan_file(rs_file, definitions)
        all_replaceable.extend(replaceable)

    return replaceable if target_file else all_replaceable


def load_ddk_functions(src_dir: Path) -> set[str]:
    """Load function names from bindgen_i386.rs (the ddk::* wildcard source)."""
    bindgen = src_dir.parent / "ntkrnl-sys" / "src" / "bindgen_i386.rs"
    fns = set()
    if bindgen.exists():
        for line in bindgen.read_text().splitlines():
            m = re.match(r"\s+pub fn (\w+)", line)
            if m:
                fns.add(m.group(1))
    return fns


def apply_fixes(
    src_dir: Path, by_file: dict[str, list[dict]], ddk_functions: set[str] | None = None
) -> None:
    """Apply fixes: remove extern fn declarations, add use imports."""
    fixed_files = 0
    fixed_decls = 0

    for filepath, items in sorted(by_file.items()):
        full_path = src_dir / filepath
        source = full_path.read_text()
        lines = source.splitlines()

        # Collect line numbers to remove (the fn declaration lines + parameters)
        lines_to_remove = set()
        for item in items:
            fn_line_idx = item["line"] - 1  # 0-indexed
            lines_to_remove.add(fn_line_idx)

            # Check if the line above is a #[link_name = ...] attribute
            if fn_line_idx > 0:
                prev = lines[fn_line_idx - 1].strip()
                if prev.startswith("#[link_name"):
                    lines_to_remove.add(fn_line_idx - 1)

            # If this is a multi-line fn declaration, remove continuation lines
            # until we find the closing `;` or `-> ... ;`
            if ";" not in lines[fn_line_idx]:
                j = fn_line_idx + 1
                while j < len(lines):
                    lines_to_remove.add(j)
                    if ";" in lines[j]:
                        break
                    j += 1

        # Collect use imports to add
        # For functions that exist in ddk::* (bindgen), don't add use — just
        # remove the extern declaration and let ddk::* provide the symbol.
        # For internal-only functions, add use crate::... import.
        imports = set()
        for item in items:
            fn_name = item["name"]

            # Check if this function exists in ddk (bindgen) AND the file
            # has use ntkrnl_sys::ddk::*
            has_ddk_wildcard = any(
                "ntkrnl_sys::ddk::*" in line or "ddk::*" in line for line in lines
            )
            in_ddk = fn_name in ddk_functions if ddk_functions else False

            if has_ddk_wildcard and in_ddk:
                # ddk::* already provides this — just remove extern, no use needed
                continue

            # Skip arch-specific modules not available on all targets
            import_path = item["import_path"]
            skip = False
            for mod_name in [
                "kd64",
                "ke::amd64",
                "ke::arm64",
                "ke::i386",
                "amd64::",
                "arm64::",
                "i386::",
            ]:
                if mod_name in import_path:
                    # Would need cfg guard — skip for now
                    skip = True
                    break
            if skip:
                continue

            if item["link_name"] and item["link_name"] != item["name"]:
                real_path = item["import_path"]
                imports.add(f"use {real_path} as {item['name']};")
            else:
                imports.add(f"use {import_path};")

        # Build new source:
        # 1. Remove fn lines from extern blocks
        new_lines = []
        for i, line in enumerate(lines):
            if i in lines_to_remove:
                continue
            new_lines.append(line)

        # 2. Clean up empty extern blocks: "extern "..." {\n}" → remove
        cleaned = []
        i = 0
        while i < len(new_lines):
            line = new_lines[i].strip()
            # Check for empty extern block (opening brace, next non-blank is closing)
            extern_match = re.match(r'^(?:#\[.*?\]\s*)*(?:unsafe\s+)?extern\s+"[^"]+"\s*\{', line)
            if extern_match:
                # Look ahead for closing brace (possibly with blank lines)
                j = i + 1
                while j < len(new_lines) and new_lines[j].strip() == "":
                    j += 1
                if j < len(new_lines) and new_lines[j].strip() == "}":
                    # Empty extern block — skip it entirely
                    i = j + 1
                    # Also skip trailing blank line if present
                    if i < len(new_lines) and new_lines[i].strip() == "":
                        i += 1
                    continue
            cleaned.append(new_lines[i])
            i += 1

        # 3. Add use imports at the top of the file (after existing use statements)
        # Find the last complete use statement (handling multi-line use blocks)
        insert_pos = 0
        in_use_block = False
        for i, line in enumerate(cleaned):
            stripped = line.strip()
            if stripped.startswith("use ") or stripped.startswith("pub use "):
                if "{" in stripped and "}" not in stripped:
                    # Multi-line use block — find the closing }
                    in_use_block = True
                else:
                    insert_pos = i + 1
            elif in_use_block:
                if "}" in stripped:
                    in_use_block = False
                    insert_pos = i + 1
            elif stripped.startswith("extern crate"):
                insert_pos = i + 1

        # If no use statements found, insert after the first blank line
        if insert_pos == 0:
            for i, line in enumerate(cleaned):
                if line.strip() == "" and i > 0:
                    insert_pos = i
                    break

        # Insert sorted imports
        for imp in sorted(imports, reverse=True):
            cleaned.insert(insert_pos, imp)

        # Write back
        full_path.write_text("\n".join(cleaned) + "\n")
        fixed_files += 1
        fixed_decls += len(items)
        print(f"  Fixed {filepath}: {len(items)} declarations → use imports")

    print(f"\nFixed {fixed_decls} declarations in {fixed_files} files")


def main():
    scan_mode = "--scan" in sys.argv
    fix_mode = "--fix" in sys.argv

    target_file = None
    for i, arg in enumerate(sys.argv):
        if arg == "--file" and i + 1 < len(sys.argv):
            target_file = sys.argv[i + 1]

    if not scan_mode and not fix_mode:
        scan_mode = True  # default to scan

    if not SRC_DIR.exists():
        print(f"ERROR: {SRC_DIR} not found. Run from vibeos/ root.", file=sys.stderr)
        sys.exit(1)

    print("Finding #[no_mangle] definitions...", file=sys.stderr)
    definitions = find_definitions(SRC_DIR)
    print(f"Found {len(definitions)} Rust function definitions", file=sys.stderr)

    print("Scanning for replaceable extern declarations...", file=sys.stderr)
    replaceable = scan_all(SRC_DIR, definitions, target_file)

    # Group by file
    by_file = defaultdict(list)
    for r in replaceable:
        short = r["file"].replace(str(SRC_DIR) + "/", "")
        by_file[short].append(r)

    if not replaceable:
        print("No replaceable extern declarations found.")
        return

    print(
        f"\n{'Would fix' if fix_mode else 'Found'} {len(replaceable)} "
        f"Rust-to-Rust extern declarations in {len(by_file)} files:\n"
    )

    for filepath, items in sorted(by_file.items()):
        print(f"  {filepath}:")
        for item in items:
            conv = item["convention"]
            name = item["name"]
            link = f" (link_name={item['link_name']})" if item["link_name"] else ""
            path = item["import_path"]
            print(f'    L{item["line"]:4d} extern "{conv}" fn {name}{link}')
            print(f"           → use {path};")
        print()

    print(f"Total: {len(replaceable)} declarations → use imports")

    if fix_mode:
        ddk_fns = load_ddk_functions(SRC_DIR)
        print(f"Loaded {len(ddk_fns)} ddk function names", file=sys.stderr)
        apply_fixes(SRC_DIR, by_file, ddk_fns)


if __name__ == "__main__":
    main()
