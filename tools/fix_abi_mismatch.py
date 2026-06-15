#!/usr/bin/env python3
"""Fix ABI mismatches between shim_exports definitions and caller declarations.

shim_exports.rs defines functions as extern "C" (cdecl on i386), but many callers
declare them as extern "system" (stdcall on i386). With CGU=1, rustc sees both
sides and emits unreachable → ud2.

This script finds all such mismatches and fixes the caller declarations to match
the actual definition convention.

Usage:
    python3 tools/fix_abi_mismatch.py --scan     # report only
    python3 tools/fix_abi_mismatch.py --fix       # apply fixes
"""

import re
import sys
from pathlib import Path

SRC_DIR = Path("sdk/lib/rust/ntoskrnl-rust/src")
SHIM_FILE = SRC_DIR / "shim_exports.rs"
SKIP_DIRS = {"old_translations"}


def get_shim_exports_functions() -> dict[str, str]:
    """Parse shim_exports.rs to get {fn_name: convention} for all exported functions."""
    source = SHIM_FILE.read_text()
    fns = {}

    # Match: pub unsafe extern "C" fn FunctionName(
    # or:   pub unsafe extern "system" fn FunctionName(
    for m in re.finditer(
        r"#\[(?:no_mangle|inline\(never\))\]\s*(?:#\[.*?\]\s*)*"
        r'pub\s+unsafe\s+extern\s+"(\w+)"\s+fn\s+(\w+)',
        source,
        re.DOTALL,
    ):
        conv = m.group(1)
        name = m.group(2)
        if name not in fns:
            fns[name] = conv

    # Remove arch-specific shims that exist only for one target.
    # On i386, KfRaiseIrql/KfLowerIrql are real C fastcall functions;
    # the shim_exports extern "C" versions are x86_64 only.
    # KeAcquireInStackQueuedSpinLock is i386-only fastcall in shim_exports
    # but callers already have cfg'd declarations.
    ARCH_SPECIFIC_SKIP = {
        "KfRaiseIrql",
        "KfLowerIrql",
        "KeAcquireInStackQueuedSpinLock",
        "KeReleaseInStackQueuedSpinLock",
        "KeAcquireInStackQueuedSpinLockRaiseToSynch",
        # amd64-only in shim_exports, i386 uses C stdcall:
        "KeQueryInterruptHandler",
        "KeRegisterInterruptHandler",
    }
    for name in ARCH_SPECIFIC_SKIP:
        fns.pop(name, None)

    return fns


def parse_extern_blocks(source: str) -> list[dict]:
    """Parse all extern blocks in a source file.

    Returns list of {convention, start_line, end_line, functions: [{name, line, link_name}]}.
    """
    lines = source.splitlines()
    blocks = []
    i = 0

    while i < len(lines):
        line = lines[i].strip()

        # Match extern block opening
        extern_match = re.match(r'^(?:unsafe\s+)?extern\s+"(\w+)"\s*\{', line)
        if extern_match:
            conv = extern_match.group(1)
            block_start = i
            brace_count = line.count("{") - line.count("}")
            functions = []

            j = i + 1
            while j < len(lines) and brace_count > 0:
                brace_count += lines[j].count("{") - lines[j].count("}")

                fn_line = lines[j].strip()
                # Check for link_name on the line above
                link_name = None
                if j > 0:
                    prev = lines[j - 1].strip()
                    link_match = re.match(r'#\[link_name\s*=\s*"(\w+)"\]', prev)
                    if link_match:
                        link_name = link_match.group(1)

                fn_match = re.match(r"(?:pub\s+)?fn\s+(\w+)\s*\(", fn_line)
                if fn_match:
                    functions.append(
                        {
                            "name": fn_match.group(1),
                            "line": j,
                            "link_name": link_name,
                        }
                    )

                j += 1

            blocks.append(
                {
                    "convention": conv,
                    "start_line": block_start,
                    "end_line": j - 1,
                    "functions": functions,
                }
            )
            i = j
        else:
            i += 1

    return blocks


def scan_file(filepath: Path, shim_fns: dict[str, str]) -> list[dict]:
    """Find mismatched extern declarations in a file."""
    source = filepath.read_text()
    blocks = parse_extern_blocks(source)
    mismatches = []

    for block in blocks:
        for fn in block["functions"]:
            fn_name = fn["name"]
            # Check link_name too (e.g., IoGetCurrentProcess → PsGetCurrentProcess)
            lookup = fn["link_name"] or fn_name

            if lookup in shim_fns:
                expected_conv = shim_fns[lookup]
                if block["convention"] != expected_conv:
                    mismatches.append(
                        {
                            "file": str(filepath),
                            "fn_name": fn_name,
                            "link_name": fn["link_name"],
                            "lookup": lookup,
                            "declared_as": block["convention"],
                            "should_be": expected_conv,
                            "fn_line": fn["line"],
                            "block_start": block["start_line"],
                            "block_end": block["end_line"],
                            "block_convention": block["convention"],
                            "block_functions": block["functions"],
                        }
                    )

    return mismatches


def fix_file(filepath: Path, mismatches: list[dict]) -> bool:
    """Fix mismatched extern declarations in a file.

    Strategy:
    - If ALL functions in a block are mismatched → change block convention
    - If SOME functions are mismatched → split block
    """
    source = filepath.read_text()
    lines = source.splitlines()

    # Group mismatches by block (start_line)
    blocks_to_fix = {}
    for m in mismatches:
        key = m["block_start"]
        if key not in blocks_to_fix:
            blocks_to_fix[key] = {
                "block_start": m["block_start"],
                "block_end": m["block_end"],
                "block_convention": m["block_convention"],
                "block_functions": m["block_functions"],
                "mismatched": [],
                "target_convention": m["should_be"],
            }
        blocks_to_fix[key]["mismatched"].append(m)

    # Process blocks in reverse order (so line numbers stay valid)
    for block_start in sorted(blocks_to_fix.keys(), reverse=True):
        info = blocks_to_fix[block_start]
        block_fns = info["block_functions"]
        mismatched_names = {m["fn_name"] for m in info["mismatched"]}
        target_conv = info["target_convention"]

        all_mismatched = len(mismatched_names) == len(block_fns)

        if all_mismatched:
            # Simple case: change the whole block's convention
            old_line = lines[info["block_start"]]
            new_line = old_line.replace(
                f'extern "{info["block_convention"]}"', f'extern "{target_conv}"'
            )
            lines[info["block_start"]] = new_line
        else:
            # Complex case: split the block
            # Extract the mismatched function declarations and their attributes
            block_end = info["block_end"]
            block_start_line = info["block_start"]

            # Collect lines for mismatched functions (including preceding attributes)
            mismatched_lines = []
            lines_to_remove = set()

            for fn in block_fns:
                if fn["name"] in mismatched_names:
                    fn_line_idx = fn["line"]
                    # Include preceding attribute lines (link_name, etc.)
                    attr_start = fn_line_idx
                    while attr_start > block_start_line + 1:
                        prev = lines[attr_start - 1].strip()
                        if prev.startswith("#["):
                            attr_start -= 1
                        else:
                            break

                    for k in range(attr_start, fn_line_idx + 1):
                        mismatched_lines.append(lines[k])
                        lines_to_remove.add(k)

                    # Handle multi-line fn declarations
                    if ";" not in lines[fn_line_idx]:
                        k = fn_line_idx + 1
                        while k <= block_end:
                            mismatched_lines.append(lines[k])
                            lines_to_remove.add(k)
                            if ";" in lines[k]:
                                break
                            k += 1

            # Build new block for the extracted functions
            indent = "".join(c for c in lines[block_start_line] if c in " \t")
            # Check if original block has 'unsafe'
            orig = lines[block_start_line].strip()
            unsafe_prefix = "unsafe " if orig.startswith("unsafe ") else ""

            new_block_lines = [f'{indent}{unsafe_prefix}extern "{target_conv}" {{']
            new_block_lines.extend(mismatched_lines)
            new_block_lines.append(f"{indent}}}")

            # Remove mismatched lines from original block
            new_lines = []
            for i, line in enumerate(lines):
                if i in lines_to_remove:
                    continue
                new_lines.append(line)

            # Insert new block after the original block
            # Find where the original block ends in new_lines
            # (line numbers shifted due to removal)
            orig_end_new = info["block_end"] - sum(
                1 for r in lines_to_remove if r <= info["block_end"]
            )
            insert_pos = orig_end_new + 1
            for nl in reversed(new_block_lines):
                new_lines.insert(insert_pos, nl)

            lines = new_lines

    # Clean up empty extern blocks
    cleaned = []
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        extern_match = re.match(r'^(?:unsafe\s+)?extern\s+"[^"]+"\s*\{', line)
        if extern_match:
            # Look ahead for closing brace
            j = i + 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            if j < len(lines) and lines[j].strip() == "}":
                i = j + 1
                if i < len(lines) and lines[i].strip() == "":
                    i += 1
                continue
        cleaned.append(lines[i])
        i += 1

    new_source = "\n".join(cleaned) + "\n"
    if new_source != source:
        filepath.write_text(new_source)
        return True
    return False


def main():
    scan_mode = "--scan" in sys.argv
    fix_mode = "--fix" in sys.argv

    if not scan_mode and not fix_mode:
        scan_mode = True

    if not SHIM_FILE.exists():
        print(f"ERROR: {SHIM_FILE} not found", file=sys.stderr)
        sys.exit(1)

    print("Parsing shim_exports.rs...", file=sys.stderr)
    shim_fns = get_shim_exports_functions()
    print(
        f"Found {len(shim_fns)} shim functions "
        f'({sum(1 for v in shim_fns.values() if v == "C")} extern "C", '
        f'{sum(1 for v in shim_fns.values() if v == "system")} extern "system")',
        file=sys.stderr,
    )

    all_mismatches = []
    for rs_file in sorted(SRC_DIR.rglob("*.rs")):
        if any(s in str(rs_file) for s in SKIP_DIRS):
            continue
        if rs_file.name == "shim_exports.rs":
            continue
        if ".bak" in rs_file.name:
            continue

        mismatches = scan_file(rs_file, shim_fns)
        all_mismatches.extend(mismatches)

    if not all_mismatches:
        print("No ABI mismatches found!")
        return

    # Group by file
    by_file = {}
    for m in all_mismatches:
        short = m["file"].replace(str(SRC_DIR) + "/", "")
        by_file.setdefault(short, []).append(m)

    print(f"\nFound {len(all_mismatches)} ABI mismatches in {len(by_file)} files:\n")
    for filepath, items in sorted(by_file.items()):
        print(f"  {filepath}:")
        for item in items:
            link = f" (link_name={item['link_name']})" if item["link_name"] else ""
            print(
                f"    L{item['fn_line'] + 1:4d} {item['fn_name']}{link}: "
                f'declared extern "{item["declared_as"]}" → should be extern "{item["should_be"]}"'
            )
        print()

    # Show unique function names
    unique_fns = sorted(set(m["lookup"] for m in all_mismatches))
    print(f"Unique mismatched functions ({len(unique_fns)}):")
    for fn in unique_fns:
        count = sum(1 for m in all_mismatches if m["lookup"] == fn)
        print(f"  {fn} ({count} sites)")

    if fix_mode:
        print("\nApplying fixes...")
        fixed = 0
        for filepath, items in sorted(by_file.items()):
            full_path = SRC_DIR / filepath
            if fix_file(full_path, items):
                print(f"  Fixed {filepath}")
                fixed += 1
        print(f"\nFixed {fixed} files")


if __name__ == "__main__":
    main()
