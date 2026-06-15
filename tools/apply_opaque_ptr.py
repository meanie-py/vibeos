#!/usr/bin/env python3
"""Apply opaque_ptr barriers to all #[no_mangle] functions with pointer parameters.

Prevents LLVM from deducing captures(none) and readnone on pointer parameters
of exported functions, which causes callers to pass poison → ud2 cascade.

Usage:
    python3 tools/apply_opaque_ptr.py --scan     # report what would change
    python3 tools/apply_opaque_ptr.py --fix       # apply changes
"""

import re
import sys
from pathlib import Path

SRC_DIR = Path("sdk/lib/rust/ntoskrnl-rust/src")

# Pointer type patterns that need opaque_ptr barriers
POINTER_TYPES = {
    # Explicit pointer types
    r"\*mut\s+\w+",
    r"\*const\s+\w+",
    # NT kernel pointer typedefs (all are *mut Something)
    r"PVOID",
    r"HANDLE",
    r"PHANDLE",
    r"PEPROCESS",
    r"PETHREAD",
    r"PKTHREAD",
    r"PKPROCESS",
    r"PFILE_OBJECT",
    r"PDEVICE_OBJECT",
    r"PDRIVER_OBJECT",
    r"PIRP",
    r"PMDL",
    r"PKEVENT",
    r"PKMUTANT",
    r"PKSEMAPHORE",
    r"PIO_STATUS_BLOCK",
    r"PIO_STACK_LOCATION",
    r"POBJECT_ATTRIBUTES",
    r"PUNICODE_STRING",
    r"PANSI_STRING",
    r"POBJECT_TYPE",
    r"POBJECT_HEADER",
    r"PACCESS_STATE",
    r"PSECURITY_DESCRIPTOR",
    r"PSID",
    r"PACL",
    r"PTOKEN",
    r"PGENERIC_MAPPING",
    r"PLARGE_INTEGER",
    r"PULARGE_INTEGER",
    r"PULONG",
    r"PULONG_PTR",
    r"PLONG",
    r"PUSHORT",
    r"PUCHAR",
    r"PBOOLEAN",
    r"PSIZE_T",
    r"PNTSTATUS",
    r"PLIST_ENTRY",
    r"PSLIST_ENTRY",
    r"PSINGLE_LIST_ENTRY",
    r"PCLIENT_ID",
    r"POBJECT_NAME_INFORMATION",
    r"PLUID",
    r"PPRIVILEGE_SET",
    r"PVPB",
    r"PFAST_IO_DISPATCH",
    r"PLOADER_PARAMETER_BLOCK",
    r"PIOP_FILE_OBJECT_EXTENSION",
    r"PIO_APC_ROUTINE",  # function pointer, still a pointer
    r"PKNORMAL_ROUTINE",
}

# Build a combined regex for matching pointer type in parameter declarations
POINTER_RE = re.compile(
    r"^\s*(?:mut\s+)?(\w+)\s*:\s*("
    + "|".join(sorted(POINTER_TYPES, key=len, reverse=True))
    + r")\b",
)


def parse_no_mangle_functions(source: str) -> list[dict]:
    """Find all #[no_mangle] pub unsafe extern functions and their pointer params."""
    lines = source.splitlines()
    functions = []
    i = 0

    while i < len(lines):
        line = lines[i].strip()

        # Look for #[no_mangle]
        if line == "#[no_mangle]":
            # Scan forward for the function signature
            j = i + 1
            while j < len(lines):
                fn_line = lines[j].strip()
                if fn_line.startswith("pub unsafe extern") and "fn " in fn_line:
                    break
                elif fn_line.startswith("pub static") or fn_line.startswith("pub const"):
                    # Skip statics/consts
                    j = -1
                    break
                elif fn_line.startswith("#["):
                    j += 1
                    continue
                else:
                    j = -1
                    break

            if j < 0 or j >= len(lines):
                i += 1
                continue

            # Found function. Collect the full signature until '{'
            sig_start = j
            sig_lines = []
            k = j
            while k < len(lines):
                sig_lines.append(lines[k])
                if "{" in lines[k]:
                    break
                k += 1

            if k >= len(lines):
                i = k
                continue

            brace_line = k  # line with opening '{'
            sig = "\n".join(sig_lines)

            # Extract function name
            fn_match = re.search(r"fn\s+(\w+)\s*\(", sig)
            if not fn_match:
                i = k + 1
                continue

            fn_name = fn_match.group(1)

            # Extract parameters
            # Get everything between ( and )
            paren_content = re.search(r"\((.*?)\)\s*->", sig, re.DOTALL)
            if not paren_content:
                paren_content = re.search(r"\((.*?)\)\s*\{", sig, re.DOTALL)
            if not paren_content:
                i = k + 1
                continue

            params_str = paren_content.group(1)
            # Split by comma, but handle nested generics
            params = split_params(params_str)

            # Find pointer parameters
            ptr_params = []
            for param in params:
                param = param.strip()
                if not param:
                    continue
                # Check if this is a pointer type parameter
                # Match: name: POINTER_TYPE or mut name: POINTER_TYPE
                m = re.match(r"(?:mut\s+)?(\w+)\s*:\s*(.+)", param)
                if m:
                    pname = m.group(1)
                    ptype = m.group(2).strip().rstrip(",")
                    if is_pointer_type(ptype):
                        is_const = ptype.startswith("*const ")
                        ptr_params.append((pname, is_const))

            # Check if opaque_ptr is already applied
            next_line = lines[brace_line + 1].strip() if brace_line + 1 < len(lines) else ""
            already_applied = "opaque_ptr(" in next_line

            if ptr_params and not already_applied:
                functions.append(
                    {
                        "name": fn_name,
                        "line": brace_line,  # line with '{'
                        "ptr_params": ptr_params,
                        "sig_start": sig_start,
                    }
                )

            i = k + 1
        else:
            i += 1

    return functions


def split_params(s: str) -> list[str]:
    """Split parameter string by commas, respecting nested angle brackets."""
    params = []
    depth = 0
    current = []
    for ch in s:
        if ch in "<(":
            depth += 1
            current.append(ch)
        elif ch in ">)":
            depth -= 1
            current.append(ch)
        elif ch == "," and depth == 0:
            params.append("".join(current))
            current = []
        else:
            current.append(ch)
    if current:
        params.append("".join(current))
    return params


_KNOWN_POINTER_TYPEDEFS = {
    "PVOID",
    "HANDLE",
    "PHANDLE",
    "PEPROCESS",
    "PETHREAD",
    "PKTHREAD",
    "PKPROCESS",
    "PFILE_OBJECT",
    "PDEVICE_OBJECT",
    "PDRIVER_OBJECT",
    "PIRP",
    "PMDL",
    "PKEVENT",
    "PKMUTANT",
    "PKSEMAPHORE",
    "PIO_STATUS_BLOCK",
    "PIO_STACK_LOCATION",
    "POBJECT_ATTRIBUTES",
    "PUNICODE_STRING",
    "PANSI_STRING",
    "POBJECT_TYPE",
    "POBJECT_HEADER",
    "PACCESS_STATE",
    "PSECURITY_DESCRIPTOR",
    "PSID",
    "PACL",
    "PTOKEN",
    "PGENERIC_MAPPING",
    "PLARGE_INTEGER",
    "PULARGE_INTEGER",
    "PULONG",
    "PULONG_PTR",
    "PLONG",
    "PUSHORT",
    "PUCHAR",
    "PBOOLEAN",
    "PSIZE_T",
    "PNTSTATUS",
    "PLIST_ENTRY",
    "PSLIST_ENTRY",
    "PSINGLE_LIST_ENTRY",
    "PCLIENT_ID",
    "POBJECT_NAME_INFORMATION",
    "PLUID",
    "PPRIVILEGE_SET",
    "PVPB",
    "PFAST_IO_DISPATCH",
    "PLOADER_PARAMETER_BLOCK",
    "PCWSTR",
    "PWSTR",
    "PCSTR",
    "PSTR",
    "PMMPFN",
    "PMMPTE",
    "PMMVAD",
    "PPOOL_HEADER",
    "PPOOL_DESCRIPTOR",
    "PHANDLE_TABLE",
    "PHANDLE_TABLE_ENTRY",
    "POBJECT_CREATE_INFORMATION",
    "PRTL_GENERIC_TABLE",
    "PRTL_SPLAY_LINKS",
    "PRTL_AVL_TABLE",
    "PSECURITY_SUBJECT_CONTEXT",
    "PAUX_ACCESS_DATA",
    "PKINTERRUPT",
    "PKDPC",
    "PKTIMER",
    "PKTHREAD_CONTROL_BLOCK",
    "PCUNICODE_STRING",
}


def is_pointer_type(ptype: str) -> bool:
    """Check if a type string is a pointer type."""
    ptype = ptype.strip()
    # Explicit pointer syntax
    if ptype.startswith("*mut ") or ptype.startswith("*const "):
        return True
    # Strip any leading module path (e.g., ntkrnl_sys::ddk::PFILE_OBJECT)
    base = ptype.split("::")[-1].strip()
    return base in _KNOWN_POINTER_TYPEDEFS


def apply_barriers(filepath: Path, functions: list[dict]) -> str:
    """Insert opaque_ptr barriers into a file. Returns modified source."""
    source = filepath.read_text()
    lines = source.splitlines()

    # Process in reverse order to preserve line numbers
    for fn in sorted(functions, key=lambda f: f["line"], reverse=True):
        brace_line = fn["line"]
        indent = "    "  # standard 4-space indent

        # Build barrier lines
        barriers = []
        for pname, is_const in fn["ptr_params"]:
            if is_const:
                barriers.append(f"{indent}let {pname} = opaque_ptr_const({pname});")
            else:
                barriers.append(f"{indent}let {pname} = opaque_ptr({pname});")

        # Insert after opening brace
        for barrier in reversed(barriers):
            lines.insert(brace_line + 1, barrier)

    return "\n".join(lines) + "\n"


def ensure_import(source: str) -> str:
    """Ensure the file imports opaque_ptr from ntkrnl_sys."""
    if "ntkrnl_sys::opaque_ptr" in source and "ntkrnl_sys::opaque_ptr_const" in source:
        return source

    # Find a safe insertion point: after the last COMPLETE ntkrnl_sys use statement
    # (must be a single-line `use ntkrnl_sys::...;` — NOT inside a multi-line block)
    lines = source.splitlines()
    insert_after = -1
    in_multiline_use = False

    for i, line in enumerate(lines):
        stripped = line.strip()
        # Track multi-line use blocks
        if stripped.startswith("use ") and "{" in stripped and ";" not in stripped:
            in_multiline_use = True
            continue
        if in_multiline_use:
            if "};" in stripped or (stripped == "}" and i + 1 < len(lines)):
                in_multiline_use = False
                if "ntkrnl_sys" in source.splitlines()[i - 1] or any(
                    "ntkrnl_sys" in source.splitlines()[j] for j in range(max(0, i - 20), i)
                ):
                    insert_after = i
            continue
        # Single-line complete use statement
        if stripped.startswith("use ntkrnl_sys::") and stripped.endswith(";"):
            insert_after = i

    if insert_after >= 0:
        lines.insert(insert_after + 1, "use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};")
    else:
        # No ntkrnl_sys imports — add at top after any existing use statements
        last_use = -1
        for i, line in enumerate(lines):
            if line.strip().startswith("use "):
                last_use = i
            elif (
                last_use >= 0
                and not line.strip().startswith("use ")
                and not line.strip().startswith("//")
                and line.strip()
            ):
                break
        if last_use >= 0:
            lines.insert(last_use + 1, "use ntkrnl_sys::opaque_ptr;")
        else:
            lines.insert(0, "use ntkrnl_sys::opaque_ptr;")

    return "\n".join(lines) + "\n"


def main():
    scan_mode = "--scan" in sys.argv
    fix_mode = "--fix" in sys.argv

    if not scan_mode and not fix_mode:
        scan_mode = True

    if not SRC_DIR.exists():
        print(f"ERROR: {SRC_DIR} not found", file=sys.stderr)
        sys.exit(1)

    total_functions = 0
    total_params = 0
    files_to_fix = {}

    for rs_file in sorted(SRC_DIR.rglob("*.rs")):
        if ".bak" in rs_file.name:
            continue
        if "old_translations" in str(rs_file):
            continue

        source = rs_file.read_text()
        functions = parse_no_mangle_functions(source)

        if functions:
            short = str(rs_file).replace(str(SRC_DIR) + "/", "")
            files_to_fix[rs_file] = functions
            total_functions += len(functions)
            total_params += sum(len(f["ptr_params"]) for f in functions)

            if scan_mode:
                print(f"\n  {short}:")
                for fn in functions:
                    params = ", ".join(p[0] for p in fn["ptr_params"])
                    print(f"    L{fn['line'] + 1:4d} {fn['name']}({params})")

    print(
        f"\nTotal: {total_functions} functions, {total_params} pointer params "
        f"across {len(files_to_fix)} files"
    )

    if fix_mode and files_to_fix:
        print("\nApplying opaque_ptr barriers...")
        fixed = 0
        for filepath, functions in sorted(files_to_fix.items()):
            short = str(filepath).replace(str(SRC_DIR) + "/", "")
            new_source = apply_barriers(filepath, functions)
            new_source = ensure_import(new_source)
            if new_source != filepath.read_text():
                filepath.write_text(new_source)
                n = sum(len(f["ptr_params"]) for f in functions)
                print(f"  {short}: {len(functions)} functions, {n} barriers")
                fixed += 1
        print(f"\nFixed {fixed} files")


if __name__ == "__main__":
    main()
