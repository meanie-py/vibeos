#!/usr/bin/env python3
"""Static linter for Rust kernel code — detects output pointer writes
that aren't protected by write_volatile.

LLVM's captures(none) optimization can prove output pointer writes are dead
and replace the entire function body with ud2. This linter catches the
pattern BEFORE compilation by scanning for `*param = value` writes in
#[no_mangle] functions where `param` is a `*mut` parameter.

Usage:
    python3 tools/lint_kernel.py                          # scan all
    python3 tools/lint_kernel.py sdk/lib/rust/ntoskrnl-rust/src/iomgr/file.rs  # single file
    python3 tools/lint_kernel.py --fix                    # show suggested fixes
"""

import re
import sys
from pathlib import Path

# Types that indicate output pointer parameters
# These are the types where *param = value writes are vulnerable
OUTPUT_PTR_TYPES = {
    # Explicit pointer-to-scalar types
    "*mut HANDLE",
    "*mut PVOID",
    "*mut BOOLEAN",
    "*mut NTSTATUS",
    "*mut KIRQL",
    "*mut ULONG",
    "*mut ULONG_PTR",
    "*mut LONG",
    "*mut SIZE_T",
    "*mut LARGE_INTEGER",
    "*mut ULONGLONG",
    "*mut ACCESS_MASK",
    "*mut DEVICE_TYPE",
    # Typedef pointer types (P-prefix = pointer to)
    "PHANDLE",
    "PBOOLEAN",
    "PKIRQL",
    "PULONG",
    "PULONG_PTR",
    "PLONG",
    "PSIZE_T",
    "PVOID",  # when used as output
    # Pointer-to-pointer types (output object pointers)
    "*mut PDEVICE_OBJECT",
    "*mut PFILE_OBJECT",
    "*mut PDRIVER_OBJECT",
    "*mut POBJECT_TYPE",
    "*mut PACCESS_TOKEN",
    "*mut PSID",
    "*mut PACL",
    "*mut PSECURITY_DESCRIPTOR",
    "*mut PEPROCESS",
    "*mut PETHREAD",
    "*mut PKTHREAD",
    "*mut PKEVENT",
    "*mut PMDL",
    # ("*mut PVOID" generic output object pointer already listed above)
    # NT-specific output types
    "POBJECT_HANDLE_INFORMATION",
    "*mut NT_PRODUCT_TYPE",
}

# Parameter name patterns that indicate output (heuristic)
OUTPUT_PARAM_NAMES = re.compile(
    r"^(Out|Result|Return|Output|New|Created|Returned|Found|"
    r".*Handle$|.*Object$|.*Token$|.*Sid$|.*Acl$|.*Descriptor$|"
    r".*Length$|.*Size$|.*Count$|.*Status$|.*Info$|.*Information$|"
    r".*Buffer$|.*Address$|.*Base$|.*Entry$|.*Extension$|"
    r"LockFailed|WasDirty|Page|SwapEntry|ConflictDetected|"
    r"DeviceMap|PortId|PreviousFlags|Event|Interfaces|"
    r"SymbolicLinkList|AllocatedResources|ModuleObject|DriverName|"
    r".*Irql$|.*Type$|.*Level$|.*State$|.*Flags$)",
    re.IGNORECASE,
)

# Files/patterns to skip
SKIP_PATTERNS = [
    "old_translations",
    ".bak",
    "shim_exports.rs",  # these are thin wrappers, handled separately
]


def find_exported_functions(source: str) -> list[dict]:
    """Find #[no_mangle] pub unsafe extern functions and their *mut params."""
    functions = []

    lines = source.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Look for #[no_mangle] (might have other attrs between)
        if line == "#[no_mangle]":
            # Scan forward for fn declaration
            j = i + 1
            fn_lines = []
            found_fn = False
            while j < len(lines) and j < i + 10:
                fn_line = lines[j].strip()
                if fn_line.startswith("pub unsafe extern"):
                    found_fn = True
                    # Collect until we find the opening brace
                    while j < len(lines):
                        fn_lines.append(lines[j])
                        if "{" in lines[j]:
                            break
                        j += 1
                    break
                elif fn_line.startswith("#[") or fn_line == "":
                    j += 1
                    continue
                else:
                    break

            if found_fn and fn_lines:
                fn_text = " ".join(line.strip() for line in fn_lines)
                fn_match = re.search(r"fn\s+(\w+)\s*\((.*?)\)", fn_text, re.DOTALL)
                if fn_match:
                    fn_name = fn_match.group(1)
                    params_text = fn_match.group(2)

                    # Parse parameters to find *mut ones
                    mut_params = []
                    for param in params_text.split(","):
                        param = param.strip()
                        if not param:
                            continue
                        # Match "ParamName: *mut Type" or "ParamName: PTYPE"
                        pm = re.match(r"(\w+)\s*:\s*(.*)", param)
                        if pm:
                            pname = pm.group(1)
                            ptype = pm.group(2).strip()
                            if "*mut" in ptype or (ptype.startswith("P") and ptype[1:2].isupper()):
                                mut_params.append(
                                    {
                                        "name": pname,
                                        "type": ptype,
                                    }
                                )

                    if mut_params:
                        # Find function body (from opening brace to matching close)
                        body_start = j
                        brace_count = 0
                        body_end = j
                        for k in range(j, len(lines)):
                            brace_count += lines[k].count("{") - lines[k].count("}")
                            if brace_count <= 0:
                                body_end = k
                                break
                        body = "\n".join(lines[body_start : body_end + 1])

                        functions.append(
                            {
                                "name": fn_name,
                                "line": i + 1,
                                "params": mut_params,
                                "body": body,
                                "body_start": body_start,
                                "body_end": body_end,
                            }
                        )
        i += 1

    return functions


def check_volatile_writes(fn: dict) -> list[dict]:
    """Check if *mut param writes use write_volatile. Return violations."""
    violations = []
    body_lines = fn["body"].splitlines()

    for param in fn["params"]:
        pname = param["name"]

        for line_idx, line in enumerate(body_lines):
            stripped = line.strip()

            # Skip comments
            if stripped.startswith("//"):
                continue

            # Look for *ParamName = value (direct write)
            # Match patterns like: *ParamName = expr;
            pattern = re.compile(
                r"^\s*\*" + re.escape(pname) + r"\s*=\s*",
            )
            if pattern.search(line):
                # Check it's not already write_volatile
                if "write_volatile" in line:
                    continue
                # Check it's not in a comment
                code_part = line.split("//")[0]
                if pattern.search(code_part):
                    violations.append(
                        {
                            "param": pname,
                            "type": param["type"],
                            "line": fn["body_start"] + line_idx + 1,
                            "code": stripped,
                            "function": fn["name"],
                        }
                    )

    return violations


def scan_file(filepath: Path) -> list[dict]:
    """Scan a single file for unprotected output pointer writes."""
    for skip in SKIP_PATTERNS:
        if skip in str(filepath):
            return []

    try:
        source = filepath.read_text()
    except Exception:
        return []

    functions = find_exported_functions(source)
    all_violations = []

    for fn in functions:
        violations = check_volatile_writes(fn)
        for v in violations:
            v["file"] = str(filepath)
        all_violations.extend(violations)

    return all_violations


def main():
    show_fix = "--fix" in sys.argv
    args = [a for a in sys.argv[1:] if not a.startswith("--")]

    if args:
        # Scan specific files
        files = [Path(a) for a in args]
    else:
        # Scan all ntoskrnl-rust source
        src_dir = Path("sdk/lib/rust/ntoskrnl-rust/src")
        if not src_dir.exists():
            print(f"ERROR: {src_dir} not found. Run from vibeos/ root.")
            sys.exit(1)
        files = sorted(src_dir.rglob("*.rs"))

    all_violations = []
    for f in files:
        violations = scan_file(f)
        all_violations.extend(violations)

    if not all_violations:
        print("No unprotected output pointer writes found.")
        return

    # Group by file
    by_file: dict[str, list] = {}
    for v in all_violations:
        by_file.setdefault(v["file"], []).append(v)

    print(
        f"Found {len(all_violations)} unprotected output pointer writes in {len(by_file)} files:\n"
    )

    for filepath, violations in sorted(by_file.items()):
        short = filepath.replace("sdk/lib/rust/ntoskrnl-rust/src/", "")
        print(f"  {short}:")
        for v in violations:
            print(f"    L{v['line']:4d} {v['function']}(): *{v['param']} = ...")
            print(f"           {v['code']}")
            if show_fix:
                # Suggest fix
                fixed = v["code"].replace(
                    f"*{v['param']} =", f"core::ptr::write_volatile({v['param']},"
                )
                if fixed != v["code"]:
                    fixed = fixed.rstrip(";") + ");"
                    print(f"       FIX: {fixed}")
        print()

    print(f"Total: {len(all_violations)} violations in {len(by_file)} files")
    if not show_fix:
        print("Run with --fix to see suggested replacements")


if __name__ == "__main__":
    main()
