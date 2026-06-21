"""
verify_abi.py — Check that hand-written extern blocks use the correct calling convention.

On i386, extern "C" is cdecl and extern "system" is stdcall. NTAPI kernel functions
are stdcall, so declaring them as extern "C" causes LLVM to insert ud2 on i386.

Usage:
    python -m rustconv verify-abi [file.rs ...]
    python -m rustconv verify-abi --fix [file.rs ...]

Checks hand-written extern "C" blocks in Rust files against:
  1. ntoskrnl.spec (1,592 exports with calling convention)
  2. bindgen_i386.rs (~3,360 functions with correct conventions)
"""

import argparse
import re
import sys
from pathlib import Path

from rustconv.config import REACTOS_ROOT as _RUSTCONV_REACTOS_ROOT  # invoked tree root

# ── Spec Parser ──────────────────────────────────────────────────────


def parse_spec(spec_path: Path) -> dict[str, str]:
    """Parse ntoskrnl.spec → {name: convention} mapping."""
    catalog = {}
    spec_re = re.compile(
        r"^@\s+(stdcall|cdecl|fastcall)\s+"
        r"(?:-arch=[^\s]+\s+)?"
        r"(\w+)"
    )
    if not spec_path.exists():
        return catalog
    for line in spec_path.read_text().splitlines():
        m = spec_re.match(line.strip())
        if m:
            conv, name = m.group(1), m.group(2)
            catalog[name] = conv
    return catalog


# ── Bindgen Convention Parser ────────────────────────────────────────


def parse_bindgen_conventions(bindgen_path: Path) -> dict[str, str]:
    """Parse bindgen_i386.rs → {name: convention} mapping."""
    catalog = {}
    if not bindgen_path.exists():
        return catalog

    current_conv = None
    extern_re = re.compile(r'^\s*unsafe\s+extern\s+"(\w+)"')
    fn_re = re.compile(r"^\s*pub\s+fn\s+(\w+)\s*\(")

    for line in bindgen_path.read_text().splitlines():
        m = extern_re.match(line)
        if m:
            current_conv = m.group(1)
            continue
        if line.strip() == "}":
            current_conv = None
            continue
        m = fn_re.match(line)
        if m and current_conv:
            catalog[m.group(1)] = current_conv

    return catalog


# ── Rust File Scanner ────────────────────────────────────────────────

# Functions that are legitimately cdecl (C runtime, variadic, shims)
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
}

# Suffixes that indicate our own C shim wrappers (always cdecl)
SHIM_SUFFIXES = ("_shim", "_c")


def scan_rust_file(rs_path: Path, spec: dict, bindgen: dict) -> list[dict]:
    """Scan a Rust file for ABI mismatches. Returns list of issues."""
    issues = []
    lines = rs_path.read_text().splitlines()

    current_conv = None
    # Track extern blocks
    extern_re = re.compile(r'^\s*extern\s+"(C|system|stdcall|cdecl)"\s*\{')
    fn_re = re.compile(r"^\s*(?:pub\s+)?fn\s+(\w+)\s*\(")
    # Also catch link_name attributes
    link_name_re = re.compile(r'#\[link_name\s*=\s*"(\w+)"\]')
    brace_depth = 0
    in_extern = False
    pending_link_name = None

    for lineno, line in enumerate(lines, 1):
        stripped = line.strip()

        # Track link_name attribute
        lm = link_name_re.search(stripped)
        if lm:
            pending_link_name = lm.group(1)
            continue

        # Enter extern block
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
                continue

            # Function declaration inside extern block
            fm = fn_re.match(stripped)
            if fm:
                fn_name = pending_link_name or fm.group(1)
                rust_name = fm.group(1)
                pending_link_name = None

                # Skip allowlisted functions
                if fn_name in CDECL_ALLOWLIST:
                    continue
                # Skip shim wrappers
                if any(fn_name.endswith(s) for s in SHIM_SUFFIXES):
                    continue
                # Skip variadic functions
                if "..." in line or (lineno < len(lines) and "..." in lines[lineno]):
                    continue

                # Only flag extern "C" functions that should be stdcall
                if current_conv == "C":
                    expected = None
                    source = None

                    # Check spec first
                    if fn_name in spec and spec[fn_name] in ("stdcall", "fastcall"):
                        expected = spec[fn_name]
                        source = "ntoskrnl.spec"

                    # Check bindgen
                    if fn_name in bindgen:
                        bg_conv = bindgen[fn_name]
                        if bg_conv in ("stdcall", "system"):
                            expected = expected or bg_conv
                            source = source or "bindgen_i386.rs"

                    if expected:
                        issues.append(
                            {
                                "file": str(rs_path),
                                "line": lineno,
                                "fn_name": rust_name,
                                "link_name": fn_name if fn_name != rust_name else None,
                                "declared": "C (cdecl)",
                                "expected": expected,
                                "source": source,
                            }
                        )
            else:
                pending_link_name = None

    return issues


# ── Main ─────────────────────────────────────────────────────────────


def main():
    parser = argparse.ArgumentParser(
        description="Verify calling conventions in Rust extern blocks against C headers"
    )
    parser.add_argument("files", nargs="*", help="Rust files to check (default: all)")
    args = parser.parse_args()

    reactos_root = _RUSTCONV_REACTOS_ROOT  # tools/rustconv → reactos/

    # Build catalogs
    spec = parse_spec(reactos_root / "ntoskrnl/ntoskrnl.spec")
    print(f"Spec catalog: {len(spec)} exports")

    bindgen = parse_bindgen_conventions(
        reactos_root / "sdk/lib/rust/ntkrnl-sys/src/bindgen_i386.rs"
    )
    # Also fold in the win32k surface (win32k-sys bindgen + win32k.spec):
    # the gate must cover win32k exports/callees too, not just ntoskrnl.
    # Names are effectively disjoint and the convention for a given name
    # is consistent across sources, so a merged superset is safe.
    w32_bindgen = parse_bindgen_conventions(
        reactos_root / "sdk/lib/rust/win32k-sys/src/bindgen_i386.rs"
    )
    for k, v in w32_bindgen.items():
        bindgen.setdefault(k, v)
    w32_spec = parse_spec(reactos_root / "win32ss/win32k.spec")
    for k, v in w32_spec.items():
        spec.setdefault(k, v)
    print(
        f"Bindgen catalog: {len(bindgen)} functions "
        f"(+{len(w32_bindgen)} win32k, +{len(w32_spec)} win32k.spec)"
    )

    if args.files:
        rs_files = [Path(f) for f in args.files]
    else:
        rs_files = sorted((reactos_root / "sdk/lib/rust/ntoskrnl-rust/src").rglob("*.rs"))

    total_issues = 0
    for rs_file in rs_files:
        issues = scan_rust_file(rs_file, spec, bindgen)
        if issues:
            for issue in issues:
                link_info = f" (link_name={issue['link_name']})" if issue["link_name"] else ""
                print(
                    f"  ABI MISMATCH: {issue['file']}:{issue['line']}: "
                    f"{issue['fn_name']}{link_info}: "
                    f"declared {issue['declared']}, "
                    f"should be {issue['expected']} "
                    f"(per {issue['source']})"
                )
            total_issues += len(issues)

    print(f"\n{total_issues} ABI mismatches found across {len(rs_files)} files")
    return 1 if total_issues > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
