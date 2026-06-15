#!/usr/bin/env python3
"""
Check for i386 calling convention mismatches between function DEFINITIONS
in ntoskrnl-rust/src/ and DECLARATIONS in ntkrnl-sys/src/bindgen_i386.rs
and hand-written ntkrnl-sys modules.

On i386:
- extern "C" (cdecl) produces _Name
- extern "system"/"stdcall" produces _Name@N
- extern "fastcall" produces @Name@N

Mismatches cause linker errors or runtime stack corruption.
"""

import re
import sys
from collections import defaultdict
from pathlib import Path

NTOSKRNL_RUST_DIR = Path("/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntoskrnl-rust/src")
BINDGEN_I386 = Path(
    "/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntkrnl-sys/src/bindgen_i386.rs"
)
NTKRNL_SYS_SRC = Path("/home/curi/Projects/os-playground/reactos/sdk/lib/rust/ntkrnl-sys/src")


def normalize_convention(conv: str) -> str:
    """Normalize calling convention strings to canonical form."""
    conv = conv.strip().strip('"').strip("'")
    if conv in ("C", "cdecl"):
        return "C"
    if conv in ("system", "stdcall"):
        return "stdcall"
    if conv in ("fastcall",):
        return "fastcall"
    return conv


def is_excluded_by_cfg(lines: list, fn_line_idx: int) -> bool:
    """
    Check if a function at the given line index is gated to exclude i386.
    Look backwards from the function definition for cfg attributes.
    Returns True if the function should be SKIPPED (not relevant to i386).
    """
    cfg_lines = []
    j = fn_line_idx - 1
    while j >= 0:
        prev = lines[j].strip()
        if prev.startswith("#[cfg(") or prev.startswith("#[cfg_attr("):
            cfg_lines.append(prev)
            j -= 1
        elif prev.startswith("#[") or prev == "" or prev.startswith("//"):
            j -= 1
        else:
            break

    for cfg in cfg_lines:
        # Positive arch gate for non-i386 only
        if (
            'target_arch = "aarch64"' in cfg
            and 'target_arch = "x86"' not in cfg
            and "not(" not in cfg
        ):
            return True
        if (
            'target_arch = "x86_64"' in cfg
            and 'target_arch = "x86"' not in cfg
            and "not(" not in cfg
        ):
            return True
        if 'not(target_arch = "x86")' in cfg:
            return True
        if 'any(target_arch = "aarch64", target_arch = "x86_64")' in cfg:
            return True
        if 'any(target_arch = "x86_64", target_arch = "aarch64")' in cfg:
            return True
        if 'target_pointer_width = "64"' in cfg and "not(" not in cfg:
            return True

    return False


def is_i386_specific(lines: list, line_idx: int) -> bool:
    """
    Check if a line is gated specifically for i386 (target_arch = "x86").
    """
    j = line_idx - 1
    while j >= 0:
        prev = lines[j].strip()
        if prev.startswith("#[cfg("):
            if 'target_arch = "x86"' in prev and "not(" not in prev:
                return True
            break
        elif prev.startswith("#[") or prev == "" or prev.startswith("//"):
            j -= 1
        else:
            break
    return False


def extract_definitions(src_dir: Path):
    """
    Extract all #[no_mangle] and #[export_name] function definitions.
    Returns list of dicts with file, func_name, convention, line info.
    """
    results = []

    for rs_file in sorted(src_dir.rglob("*.rs")):
        rel_path = rs_file.relative_to(src_dir)

        # Skip files in arch-specific directories that are NOT i386
        parts = rel_path.parts
        if "amd64" in parts or "arm64" in parts:
            continue

        content = rs_file.read_text(errors="replace")
        lines = content.split("\n")

        # Check for module-level cfg that excludes i386
        module_excluded = False
        for line in lines[:20]:
            stripped = line.strip()
            if stripped.startswith("#![cfg("):
                if 'target_arch = "aarch64"' in stripped and 'target_arch = "x86"' not in stripped:
                    module_excluded = True
                if 'target_arch = "x86_64"' in stripped and 'target_arch = "x86"' not in stripped:
                    module_excluded = True
                if 'not(target_arch = "x86")' in stripped:
                    module_excluded = True
        if module_excluded:
            continue

        i = 0
        while i < len(lines):
            line = lines[i].strip()

            is_no_mangle = line == "#[no_mangle]"
            is_export_name = line.startswith("#[export_name")

            if is_no_mangle or is_export_name:
                export_name = None
                if is_export_name:
                    m = re.search(r'export_name\s*=\s*"([^"]+)"', line)
                    if m:
                        export_name = m.group(1)

                attr_line = i

                # Look forward past more attributes to find fn signature
                k = i + 1
                while k < len(lines):
                    next_line = lines[k].strip()
                    if next_line.startswith("#[") or next_line == "" or next_line.startswith("//"):
                        k += 1
                    elif "fn " in next_line:
                        break
                    else:
                        break
                    if k - i > 20:
                        break

                if k < len(lines) and "fn " in lines[k]:
                    fn_line = lines[k].strip()

                    # Check if excluded by cfg
                    if is_excluded_by_cfg(lines, attr_line):
                        i = k + 1
                        continue

                    # Extract calling convention
                    conv_match = re.search(r'extern\s+"([^"]+)"\s+fn\s+(\w+)', fn_line)
                    if conv_match:
                        convention = conv_match.group(1)
                        func_name = conv_match.group(2)
                    else:
                        fn_match = re.search(r"fn\s+(\w+)", fn_line)
                        if fn_match:
                            func_name = fn_match.group(1)
                            convention = "Rust"
                        else:
                            i = k + 1
                            continue

                    if export_name:
                        func_name = export_name

                    results.append(
                        {
                            "file": str(rel_path),
                            "func_name": func_name,
                            "convention": normalize_convention(convention),
                            "line": k + 1,
                        }
                    )

                i = k + 1 if k > i else i + 1
            else:
                i += 1

    return results


def extract_bindgen_declarations(bindgen_file: Path):
    """
    Extract extern function declarations from bindgen_i386.rs.
    Format: unsafe extern "stdcall" { pub fn Name(...); }
    Returns dict: func_name -> (convention, line_no)
    """
    declarations = {}
    content = bindgen_file.read_text(errors="replace")
    lines = content.split("\n")

    current_conv = None
    brace_depth = 0

    for i, line in enumerate(lines):
        stripped = line.strip()

        extern_match = re.match(r'(?:unsafe\s+)?extern\s+"([^"]+)"\s*\{', stripped)
        if extern_match:
            current_conv = normalize_convention(extern_match.group(1))
            brace_depth = 1
            fn_match = re.search(r"pub\s+fn\s+(\w+)", stripped)
            if fn_match:
                declarations[fn_match.group(1)] = (current_conv, i + 1)
            continue

        if current_conv and brace_depth > 0:
            brace_depth += stripped.count("{") - stripped.count("}")

            if brace_depth <= 0:
                current_conv = None
                brace_depth = 0
                continue

            fn_match = re.search(r"pub\s+fn\s+(\w+)", stripped)
            if fn_match:
                declarations[fn_match.group(1)] = (current_conv, i + 1)

    return declarations


def extract_handwritten_declarations(ntkrnl_sys_dir: Path):
    """
    Extract extern function declarations from hand-written ntkrnl-sys modules.
    Handles cfg(target_arch) gating — returns the i386-applicable convention.
    Returns dict: func_name -> (convention, line_no, source_file)
    """
    declarations = {}

    # Scan all .rs files in ntkrnl-sys/src/ except bindgen_* and size_check_*
    for rs_file in sorted(ntkrnl_sys_dir.glob("*.rs")):
        fname = rs_file.name
        if fname.startswith("bindgen_") or fname.startswith("size_check_"):
            continue
        if fname == "lib.rs":
            continue

        content = rs_file.read_text(errors="replace")
        lines = content.split("\n")

        current_conv = None
        brace_depth = 0
        current_cfg = None  # Track cfg gating of current extern block

        for i, line in enumerate(lines):
            stripped = line.strip()

            # Check for cfg before extern block
            if stripped.startswith("#[cfg("):
                current_cfg = stripped
                continue

            extern_match = re.match(r'(?:unsafe\s+)?extern\s+"([^"]+)"\s*\{', stripped)
            if extern_match:
                raw_conv = normalize_convention(extern_match.group(1))
                brace_depth = 1

                # Determine if this extern block applies to i386
                applies_to_i386 = True
                if current_cfg and (
                    'not(target_arch = "x86")' in current_cfg
                    or (
                        'target_arch = "x86_64"' in current_cfg
                        and 'target_arch = "x86"' not in current_cfg
                    )
                    or (
                        'target_arch = "aarch64"' in current_cfg
                        and 'target_arch = "x86"' not in current_cfg
                    )
                    or 'target_pointer_width = "64"' in current_cfg
                ):
                    applies_to_i386 = False

                if applies_to_i386:
                    current_conv = raw_conv
                else:
                    current_conv = None

                current_cfg = None  # Reset cfg tracker

                fn_match = re.search(r"pub\s+fn\s+(\w+)", stripped)
                if fn_match and current_conv:
                    declarations[fn_match.group(1)] = (current_conv, i + 1, fname)
                continue

            if brace_depth > 0:
                brace_depth += stripped.count("{") - stripped.count("}")

                if brace_depth <= 0:
                    current_conv = None
                    brace_depth = 0
                    current_cfg = None
                    continue

                fn_match = re.search(r"pub\s+fn\s+(\w+)", stripped)
                if fn_match and current_conv:
                    declarations[fn_match.group(1)] = (current_conv, i + 1, fname)
            else:
                # Reset cfg if we see something that isn't an extern block
                if (
                    not stripped.startswith("#[")
                    and not stripped.startswith("//")
                    and stripped != ""
                ):
                    current_cfg = None

    return declarations


def check_original_c_convention(func_name: str) -> str:
    """
    Look up the original C declaration to determine the true calling convention.
    Check ntoskrnl.spec and C headers.
    """
    # This is a stub - for manual verification
    return ""


def main():
    print("=" * 130)
    print("i386 CALLING CONVENTION MISMATCH CHECKER")
    print("=" * 130)

    print("\n1. Extracting function definitions from ntoskrnl-rust/src/...")
    definitions = extract_definitions(NTOSKRNL_RUST_DIR)
    print(f"   Found {len(definitions)} #[no_mangle] function definitions (i386-relevant)")

    conv_counts = defaultdict(int)
    for d in definitions:
        conv_counts[d["convention"]] += 1
    for conv, count in sorted(conv_counts.items()):
        print(f"     {conv}: {count}")

    print("\n2. Extracting declarations from bindgen_i386.rs...")
    bindgen_decls = extract_bindgen_declarations(BINDGEN_I386)
    print(f"   Found {len(bindgen_decls)} extern function declarations")

    conv_counts = defaultdict(int)
    for _name, (conv, _) in bindgen_decls.items():
        conv_counts[conv] += 1
    for conv, count in sorted(conv_counts.items()):
        print(f"     {conv}: {count}")

    print("\n3. Extracting declarations from hand-written ntkrnl-sys modules...")
    handwritten_decls = extract_handwritten_declarations(NTKRNL_SYS_SRC)
    print(f"   Found {len(handwritten_decls)} extern function declarations (i386-applicable)")

    # Show handwritten modules found
    hw_files = defaultdict(int)
    for _name, (_conv, _line, fname) in handwritten_decls.items():
        hw_files[fname] += 1
    for fname, count in sorted(hw_files.items()):
        print(f"     {fname}: {count}")

    # Cross-reference
    mismatches = []
    matched_bindgen = 0
    matched_handwritten = 0
    not_found = 0
    matched_correct = 0

    for defn in definitions:
        func_name = defn["func_name"]
        def_conv = defn["convention"]

        found = False

        # Check bindgen first (this is the authoritative source for C ABI)
        if func_name in bindgen_decls:
            decl_conv, decl_line = bindgen_decls[func_name]
            found = True
            matched_bindgen += 1

            if def_conv != decl_conv:
                mismatches.append(
                    {
                        "file": defn["file"],
                        "func_name": func_name,
                        "def_conv": def_conv,
                        "decl_conv": decl_conv,
                        "def_line": defn["line"],
                        "decl_source": "bindgen_i386.rs",
                        "decl_line": decl_line,
                    }
                )
            else:
                matched_correct += 1

        # Check hand-written modules (override bindgen if also found there)
        if func_name in handwritten_decls:
            decl_conv, decl_line, decl_file = handwritten_decls[func_name]
            if not found:
                matched_handwritten += 1
                found = True

            if def_conv != decl_conv:
                # Only add if not already mismatched via bindgen
                already_added = any(m["func_name"] == func_name for m in mismatches)
                if not already_added:
                    mismatches.append(
                        {
                            "file": defn["file"],
                            "func_name": func_name,
                            "def_conv": def_conv,
                            "decl_conv": decl_conv,
                            "def_line": defn["line"],
                            "decl_source": decl_file,
                            "decl_line": decl_line,
                        }
                    )
            else:
                # If handwritten matches but bindgen didn't, remove the bindgen mismatch
                # since hand-written is the override
                mismatches = [m for m in mismatches if m["func_name"] != func_name]
                matched_correct += 1

        if not found:
            not_found += 1

    print("\n4. Cross-reference results:")
    print(f"   Matched in bindgen:      {matched_bindgen}")
    print(f"   Matched in handwritten:  {matched_handwritten}")
    print(f"   Convention matches:      {matched_correct}")
    print(f"   Not in any declaration:  {not_found}")
    print(f"   *** MISMATCHES:          {len(mismatches)} ***")

    if mismatches:
        # Categorize mismatches
        cat1 = [m for m in mismatches if m["def_conv"] == "stdcall" and m["decl_conv"] == "C"]
        cat2 = [m for m in mismatches if m["def_conv"] == "C" and m["decl_conv"] == "stdcall"]
        cat3 = [
            m for m in mismatches if m["def_conv"] == "fastcall" or m["decl_conv"] == "fastcall"
        ]
        cat4 = [m for m in mismatches if m not in cat1 and m not in cat2 and m not in cat3]

        print("\n" + "=" * 130)
        print("CALLING CONVENTION MISMATCHES")
        print("=" * 130)
        print(f"\n{'File':<50} {'Function':<50} {'Definition':<12} {'Bindgen':<12} {'Source':<20}")
        print("-" * 130)
        for m in sorted(mismatches, key=lambda x: (x["file"], x["func_name"])):
            print(
                f"{m['file']:<50} {m['func_name']:<50} {m['def_conv']:<12} "
                f"{m['decl_conv']:<12} {m['decl_source']:<20}"
            )

        if cat1:
            print(f"\n--- Category 1: Definition=stdcall, Bindgen=C ({len(cat1)} functions) ---")
            print("    Rust defines _Name@N but C expects _Name")
            print('    FIX: Change definition to extern "C"')
            for m in sorted(cat1, key=lambda x: x["func_name"]):
                print(f"      {m['func_name']:<50} ({m['file']}:{m['def_line']})")

        if cat2:
            print(f"\n--- Category 2: Definition=C, Bindgen=stdcall ({len(cat2)} functions) ---")
            print("    Rust defines _Name but C expects _Name@N")
            print('    FIX: Change definition to extern "system"')
            for m in sorted(cat2, key=lambda x: x["func_name"]):
                print(f"      {m['func_name']:<50} ({m['file']}:{m['def_line']})")

        if cat3:
            print(f"\n--- Category 3: Fastcall mismatch ({len(cat3)} functions) ---")
            print("    FIX: Verify original C calling convention")
            for m in sorted(cat3, key=lambda x: x["func_name"]):
                print(
                    f"      {m['func_name']:<50} def={m['def_conv']}, "
                    f"decl={m['decl_conv']} ({m['file']}:{m['def_line']})"
                )

        if cat4:
            print(f"\n--- Category 4: Other ({len(cat4)} functions) ---")
            for m in sorted(cat4, key=lambda x: x["func_name"]):
                print(
                    f"      {m['func_name']:<50} def={m['def_conv']}, "
                    f"decl={m['decl_conv']} ({m['file']}:{m['def_line']})"
                )

    else:
        print("\n  No mismatches found! All calling conventions are consistent.")

    if "--verbose" in sys.argv:
        print("\n" + "=" * 130)
        print("FUNCTIONS NOT FOUND IN ANY DECLARATION")
        print("(Internal Sep*/Obp*/Psp*/Ki* functions - no C extern needed)")
        print("=" * 130)
        for defn in sorted(definitions, key=lambda x: (x["file"], x["func_name"])):
            if (
                defn["func_name"] not in bindgen_decls
                and defn["func_name"] not in handwritten_decls
            ):
                print(f'  {defn["file"]:<50} {defn["func_name"]:<50} extern "{defn["convention"]}"')

    return len(mismatches)


if __name__ == "__main__":
    sys.exit(main())
