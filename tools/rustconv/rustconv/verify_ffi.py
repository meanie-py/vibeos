"""
verify_ffi.py — Comprehensive FFI verification for Rust kernel code.

Checks hand-written extern blocks for:
  1. ABI mismatch (extern "C" vs extern "system" on i386)
  2. Parameter count mismatch against ntoskrnl.spec and bindgen
  3. C macros declared as extern fn (no real symbol exists)
  4. Flexible array direct indexing (UB for index > 0)

Usage:
    python tools/rustconv/rustconv/verify_ffi.py [file.rs ...]
"""

import argparse
import re
import sys
from pathlib import Path

from rustconv.config import REACTOS_ROOT as _RUSTCONV_REACTOS_ROOT  # invoked tree root

# ── Known C macros that should NOT be declared as extern fn ──────────

C_MACROS = {
    "CONTAINING_RECORD",
    "InitializeObjectAttributes",
    "IoGetCurrentIrpStackLocation",
    "IoGetNextIrpStackLocation",
    "IoSizeOfIrp",
    "ObReferenceObject",
    "ObDereferenceObject",
    "ExRaiseStatus",
    "IoSkipCurrentIrpStackLocation",
    "IoCopyCurrentIrpStackLocationToNext",
    "IoSetCompletionRoutine",
    "IoSetNextIrpStackLocation",
    "IoMarkIrpPending",
    "InsertTailList",
    "InsertHeadList",
    "RemoveHeadList",
    "RemoveTailList",
    "RemoveEntryList",
    "IsListEmpty",
    "InitializeListHead",
    "KeAcquireSpinLock",
    "KeReleaseSpinLock",
    "ExInitializeWorkItem",
    "RtlZeroMemory",
    "RtlCopyMemory",
    "RtlFillMemory",
    "RtlMoveMemory",
}

# Functions that ARE exported as real symbols despite being macro-like
C_MACRO_EXCEPTIONS = {
    "ObfReferenceObject",  # real fastcall export
    "ObfDereferenceObject",
    "RtlZeroMemory",  # real export on some arches
}

# Known flexible-array structs: struct name → field name
FLEXIBLE_ARRAYS = {
    "_DEVICE_RELATIONS": "Objects",
    "DEVICE_RELATIONS": "Objects",
    "_CM_RESOURCE_LIST": "List",
    "CM_RESOURCE_LIST": "List",
    "_CM_FULL_RESOURCE_DESCRIPTOR": "PartialResourceList",
    "_IO_RESOURCE_REQUIREMENTS_LIST": "List",
    "_IO_RESOURCE_LIST": "Descriptors",
    "_KEY_VALUE_FULL_INFORMATION": "Data",
    "_KEY_VALUE_PARTIAL_INFORMATION": "Data",
    "_FILE_BOTH_DIR_INFORMATION": "FileName",
    "_FILE_DIRECTORY_INFORMATION": "FileName",
    "_FILE_FULL_DIR_INFORMATION": "FileName",
    "_FILE_NAMES_INFORMATION": "FileName",
    "_UNICODE_STRING": None,  # Buffer is a pointer, not flexible array
}

# CDECl allowlist — functions that are legitimately cdecl
CDECL_ALLOWLIST = {
    "sprintf",
    "swprintf",
    "snprintf",
    "wcslen",
    "strlen",
    "strstr",
    "strcmp",
    "wcscmp",
    "memcpy",
    "memset",
    "memmove",
    "memcmp",
    "IsEqualGUID",
    "DbgPrint",
    "_snprintf",
    "_snwprintf",
}

SHIM_SUFFIXES = ("_shim", "_c")


# ── Spec Parser ──────────────────────────────────────────────────────


def parse_spec(spec_path: Path) -> dict[str, dict]:
    """Parse ntoskrnl.spec → {name: {conv, param_count}} mapping."""
    catalog = {}
    spec_re = re.compile(
        r"^@\s+(stdcall|cdecl|fastcall)\s+"
        r"(?:-arch=[^\s]+\s+)?"
        r"(\w+)\s*\(([^)]*)\)"
    )
    if not spec_path.exists():
        return catalog
    for line in spec_path.read_text().splitlines():
        m = spec_re.match(line.strip())
        if m:
            conv, name, params = m.group(1), m.group(2), m.group(3).strip()
            param_count = len(params.split()) if params else 0
            catalog[name] = {"conv": conv, "params": param_count}
    return catalog


# ── Bindgen Parser ───────────────────────────────────────────────────


def parse_bindgen(bindgen_path: Path) -> dict[str, dict]:
    """Parse bindgen_i386.rs → {name: {conv, param_count, return_type}} mapping."""
    catalog = {}
    if not bindgen_path.exists():
        return catalog

    current_conv = None
    in_fn = False
    fn_name = None
    param_count = 0
    return_type = None

    extern_re = re.compile(r'^\s*unsafe\s+extern\s+"(\w+)"')
    fn_start_re = re.compile(r"^\s*pub\s+fn\s+(\w+)\s*\(")
    fn_end_re = re.compile(r"\)\s*(?:->.*)?;")
    param_re = re.compile(r"^\s*\w+\s*:")

    for line in bindgen_path.read_text().splitlines():
        m = extern_re.match(line)
        if m:
            current_conv = m.group(1)
            continue
        if line.strip() == "}":
            if not in_fn:
                current_conv = None
            continue

        if not in_fn:
            m = fn_start_re.match(line)
            if m and current_conv:
                fn_name = m.group(1)
                in_fn = True
                param_count = 0
                # Check if single-line fn
                if fn_end_re.search(line):
                    # Count params on this line
                    params_part = line[line.index("(") + 1 : line.rindex(")")]
                    param_count = (
                        len([p for p in params_part.split(",") if ":" in p])
                        if params_part.strip()
                        else 0
                    )
                    ret_match = re.search(r"->\s*(\S+)", line)
                    return_type = ret_match.group(1).rstrip(";") if ret_match else "void"
                    catalog[fn_name] = {
                        "conv": current_conv,
                        "params": param_count,
                        "ret": return_type,
                    }
                    in_fn = False
                else:
                    # Count first line params
                    after_paren = line[line.index("(") + 1 :]
                    if param_re.match(after_paren.strip()):
                        param_count = 1
        else:
            if fn_end_re.search(line):
                # Last line of fn — might have a param and return type
                if param_re.match(line.strip()):
                    param_count += 1
                ret_match = re.search(r"->\s*(\S+)", line)
                return_type = ret_match.group(1).rstrip(";") if ret_match else "void"
                catalog[fn_name] = {"conv": current_conv, "params": param_count, "ret": return_type}
                in_fn = False
            elif param_re.match(line.strip()):
                param_count += 1

    return catalog


# ── Rust File Scanner ────────────────────────────────────────────────


def scan_extern_blocks(rs_path: Path, spec: dict, bindgen: dict) -> list[dict]:
    """Scan for ABI, param count, and C macro issues in extern blocks."""
    issues = []
    lines = rs_path.read_text().splitlines()

    current_conv = None
    in_extern = False
    brace_depth = 0
    pending_link_name = None

    # Also collect fn params for count check
    in_fn_decl = False
    fn_name = None
    fn_rust_name = None
    fn_line = 0
    fn_param_count = 0

    extern_re = re.compile(r'^\s*extern\s+"(C|system|stdcall|cdecl)"\s*\{')
    fn_re = re.compile(r"^\s*(?:pub\s+)?fn\s+(\w+)\s*\(")
    link_name_re = re.compile(r'#\[link_name\s*=\s*"(\w+)"\]')
    static_re = re.compile(r"^\s*(?:pub\s+)?(?:static\s+(?:mut\s+)?)\w+")
    fn_end_re = re.compile(r"\)\s*(?:->.*)?;")
    param_re = re.compile(r"^\s*\w+\s*:")

    for lineno, line in enumerate(lines, 1):
        stripped = line.strip()

        lm = link_name_re.search(stripped)
        if lm:
            pending_link_name = lm.group(1)
            continue

        em = extern_re.match(stripped)
        if em:
            current_conv = em.group(1)
            in_extern = True
            brace_depth = 1
            continue

        if in_extern:
            brace_depth += stripped.count("{") - stripped.count("}")
            if brace_depth <= 0:
                in_extern = False
                current_conv = None
                pending_link_name = None
                in_fn_decl = False
                continue

            # Skip statics
            if static_re.match(stripped):
                pending_link_name = None
                continue

            if not in_fn_decl:
                fm = fn_re.match(stripped)
                if fm:
                    fn_rust_name = fm.group(1)
                    fn_name = pending_link_name or fn_rust_name
                    fn_line = lineno
                    fn_param_count = 0
                    pending_link_name = None

                    # Check for C macro
                    if fn_name in C_MACROS and fn_name not in C_MACRO_EXCEPTIONS:
                        issues.append(
                            {
                                "file": str(rs_path),
                                "line": lineno,
                                "type": "C_MACRO",
                                "fn_name": fn_rust_name,
                                "detail": f"{fn_name} is a C macro, not a real function "
                                "— should be inlined in Rust",
                            }
                        )

                    # Check if single-line
                    if fn_end_re.search(stripped):
                        params_part = stripped[stripped.index("(") + 1 : stripped.rindex(")")]
                        fn_param_count = (
                            len([p for p in params_part.split(",") if ":" in p])
                            if params_part.strip()
                            else 0
                        )
                        _check_fn(
                            issues,
                            rs_path,
                            fn_line,
                            fn_name,
                            fn_rust_name,
                            current_conv,
                            fn_param_count,
                            spec,
                            bindgen,
                        )
                    else:
                        in_fn_decl = True
                        # Count params on first line
                        after_paren = stripped[stripped.index("(") + 1 :]
                        if param_re.match(after_paren.strip()):
                            fn_param_count = 1
                else:
                    pending_link_name = None
            else:
                if fn_end_re.search(stripped):
                    if param_re.match(stripped):
                        fn_param_count += 1
                    _check_fn(
                        issues,
                        rs_path,
                        fn_line,
                        fn_name,
                        fn_rust_name,
                        current_conv,
                        fn_param_count,
                        spec,
                        bindgen,
                    )
                    in_fn_decl = False
                elif param_re.match(stripped):
                    fn_param_count += 1

    return issues


def _check_fn(issues, rs_path, lineno, fn_name, rust_name, conv, param_count, spec, bindgen):
    """Check a single extern fn declaration against spec/bindgen."""
    # Skip allowlisted
    if fn_name in CDECL_ALLOWLIST:
        return
    if any(fn_name.endswith(s) for s in SHIM_SUFFIXES):
        return

    # ABI check
    if conv == "C":
        expected_conv = None
        source = None
        if fn_name in spec and spec[fn_name]["conv"] in ("stdcall", "fastcall"):
            expected_conv = spec[fn_name]["conv"]
            source = "ntoskrnl.spec"
        elif fn_name in bindgen and bindgen[fn_name]["conv"] in ("stdcall", "system"):
            expected_conv = bindgen[fn_name]["conv"]
            source = "bindgen_i386.rs"
        if expected_conv:
            issues.append(
                {
                    "file": str(rs_path),
                    "line": lineno,
                    "type": "ABI",
                    "fn_name": rust_name,
                    "detail": f'declared extern "C" (cdecl), '
                    f"should be {expected_conv} (per {source})",
                }
            )

    # Parameter count check
    ref = None
    ref_source = None
    if fn_name in spec:
        ref = spec[fn_name]
        ref_source = "ntoskrnl.spec"
    if fn_name in bindgen:
        ref = bindgen[fn_name]
        ref_source = "bindgen_i386.rs"

    if ref and ref["params"] > 0 and param_count > 0 and param_count != ref["params"]:
        issues.append(
            {
                "file": str(rs_path),
                "line": lineno,
                "type": "PARAM_COUNT",
                "fn_name": rust_name,
                "detail": f"Rust declares {param_count} params, {ref_source} has {ref['params']}",
            }
        )


# ── Flexible Array Scanner ───────────────────────────────────────────


def scan_flexible_arrays(rs_path: Path) -> list[dict]:
    """Scan for direct indexing of known flexible array fields."""
    issues = []
    lines = rs_path.read_text().splitlines()

    for lineno, line in enumerate(lines, 1):
        for struct_name, field_name in FLEXIBLE_ARRAYS.items():
            if field_name is None:
                continue
            # Match patterns like .Objects[i] or .Objects[i as usize]
            pattern = rf"\.{field_name}\s*\["
            if re.search(pattern, line):
                # Skip if it's .Objects[0] (that's safe)
                if re.search(rf"\.{field_name}\s*\[\s*0\s*\]", line):
                    continue
                # Skip if in a comment
                stripped = line.lstrip()
                if stripped.startswith("//") or stripped.startswith("*"):
                    continue
                issues.append(
                    {
                        "file": str(rs_path),
                        "line": lineno,
                        "type": "FLEX_ARRAY",
                        "fn_name": field_name,
                        "detail": f"Direct indexing of flexible array {struct_name}.{field_name} "
                        f"— use pointer arithmetic instead (bindgen declares as [T; 1])",
                    }
                )

    return issues


# ── Main ─────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(
        description="Comprehensive FFI verification for Rust kernel code"
    )
    parser.add_argument("files", nargs="*", help="Rust files to check (default: all)")
    args = parser.parse_args()

    reactos_root = _RUSTCONV_REACTOS_ROOT

    spec = parse_spec(reactos_root / "ntoskrnl/ntoskrnl.spec")
    print(f"Spec catalog: {len(spec)} exports")

    bindgen = parse_bindgen(reactos_root / "sdk/lib/rust/ntkrnl-sys/src/bindgen_i386.rs")
    print(f"Bindgen catalog: {len(bindgen)} functions")

    if args.files:
        rs_files = [Path(f) for f in args.files]
    else:
        rs_files = sorted((reactos_root / "sdk/lib/rust/ntoskrnl-rust/src").rglob("*.rs"))

    counts = {"ABI": 0, "PARAM_COUNT": 0, "C_MACRO": 0, "FLEX_ARRAY": 0}
    for rs_file in rs_files:
        issues = scan_extern_blocks(rs_file, spec, bindgen)
        issues += scan_flexible_arrays(rs_file)
        for issue in issues:
            tag = issue["type"]
            counts[tag] += 1
            print(
                f"  [{tag}] {issue['file']}:{issue['line']}: {issue['fn_name']}: {issue['detail']}"
            )

    print("\nSummary:")
    for tag, count in counts.items():
        if count > 0:
            print(f"  {tag}: {count}")
    total = sum(counts.values())
    print(f"  TOTAL: {total} issues across {len(rs_files)} files")
    return 1 if total > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
