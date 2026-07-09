#!/usr/bin/env python3
"""scan_bool_width.py — find extern decls that WIDEN a BOOLEAN/u8 return to i32/u32.

The bug class (field manual §"BOOLEAN return declared as i32"):
  A C function (or Rust #[no_mangle] fn) returns BOOLEAN/u8 — on x86 the callee
  only defines AL; the upper 3 bytes of EAX are garbage. If a Rust extern decl
  declares the same function `-> i32`/`-> u32`, every `!= 0` test reads those
  garbage bytes and randomly returns true. This made fastfat raise
  STATUS_INVALID_DEVICE_REQUEST on healthy volume reads (FatDeviceIsFatFsdo
  declared -> i32) and corrupted mountmgr drive-letter logic (IsDriveLetter).

  The REVERSE direction (definition i32, decl u8) is safe for boolean semantics:
  the callee writes all of EAX, the caller reads AL — truncation preserves 0/non-0
  ... except for NTSTATUS-vs-BOOLEAN confusion, which this tool also flags when
  a decl says NTSTATUS but the definition returns BOOLEAN (semantics bug:
  TRUE(1) and FALSE(0) are both >= 0, so `if !NT_SUCCESS(x)` never fires).

Usage:
  python3 tools/scan_bool_width.py            # scan sdk/lib/rust
  python3 tools/scan_bool_width.py --json     # machine-readable
Exit 1 if findings (suitable for a 0-NEW baseline gate).
Suppress a verified FP with: // nolint: bool-width (reason)
"""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUST_ROOT = ROOT / "sdk" / "lib" / "rust"
WIDE_TYPES = ("i32", "u32", "LONG", "ULONG", "NTSTATUS", "BOOL", "INT", "UINT")
SKIP_PARTS = {"target", "old_translations", ".responses"}

DEF_RE = re.compile(
    r'pub (?:unsafe )?extern "(?:system|C|stdcall|win64|fastcall)" fn (\w+)'
    r"\s*\((?:[^;{]*?)\)\s*->\s*(\w+)",
    re.S,
)
C_BOOL_PROTO_RES = [
    re.compile(r"\bBOOLEAN\s+(?:NTAPI\s+)?(\w+)\s*\("),
    re.compile(r"\bBOOLEAN\s*\n(?:NTAPI\s*\n)?\s*(\w+)\s*\("),
]
EXTERN_OPEN_RE = re.compile(r'extern "(?:system|C|stdcall|win64|fastcall)"\s*\{')
DECL_RE = re.compile(r"^\s*(?:pub )?fn (\w+)\s*\(.*\)\s*->\s*(\w+)\s*;")


def rust_files():
    for f in RUST_ROOT.rglob("*.rs"):
        if SKIP_PARTS.isdisjoint(f.parts):
            yield f


def collect_u8_defs():
    """name -> (file, ret) for every u8/BOOLEAN-returning definition."""
    defs = {}
    for f in rust_files():
        text = f.read_text(errors="ignore")
        for m in DEF_RE.finditer(text):
            if m.group(2) in ("u8", "BOOLEAN"):
                defs[m.group(1)] = (str(f.relative_to(ROOT)), m.group(2))
    for base in (ROOT / "drivers", ROOT / "sdk" / "include", ROOT / "boot"):
        if not base.exists():
            continue
        for f in list(base.rglob("*.h")) + list(base.rglob("rust_cohab*.c")):
            text = f.read_text(errors="ignore")
            for rx in C_BOOL_PROTO_RES:
                for m in rx.finditer(text):
                    defs.setdefault(m.group(1), (str(f.relative_to(ROOT)), "BOOLEAN"))
    return defs


def collect_i32_defs():
    """Names DEFINED with a wide return somewhere in Rust (consistent-i32 crates)."""
    defs = set()
    for f in rust_files():
        for m in DEF_RE.finditer(f.read_text(errors="ignore")):
            if m.group(2) in WIDE_TYPES:
                defs.add(m.group(1))
    return defs


def scan():
    u8_defs = collect_u8_defs()
    wide_defs = collect_i32_defs()
    findings = []
    for f in rust_files():
        lines = f.read_text(errors="ignore").splitlines()
        in_extern = False
        for i, line in enumerate(lines, 1):
            if EXTERN_OPEN_RE.search(line):
                in_extern = True
            elif in_extern and re.match(r"\s*\}", line) and "{" not in line:
                in_extern = False
            if not in_extern or "nolint: bool-width" in line:
                continue
            m = DECL_RE.match(line)
            if not m or m.group(2) not in WIDE_TYPES:
                continue
            name = m.group(1)
            # a wide Rust definition anywhere means the crate family is
            # consistently wide — skip (safe direction)
            if name in wide_defs or name not in u8_defs:
                continue
            def_file, def_ret = u8_defs[name]
            findings.append(
                {
                    "file": str(f.relative_to(ROOT)),
                    "line": i,
                    "name": name,
                    "declared": m.group(2),
                    "defined": def_ret,
                    "definition": def_file,
                }
            )
    return findings


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    findings = scan()
    if args.json:
        print(json.dumps(findings, indent=2))
    else:
        for x in findings:
            print(
                f"{x['file']}:{x['line']}: {x['name']} declared -> {x['declared']}, "
                f"defined -> {x['defined']} ({x['definition']})"
            )
        print(f"{len(findings)} findings")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
