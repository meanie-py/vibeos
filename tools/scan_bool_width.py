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

  The OTHER way to get the same mismatch: the Rust DEFINITION is the narrow side. A
  `#[no_mangle]` fn returning u8/BOOLEAN whose C original (tools/c_twin.py, paired by
  name within the same module) returns BOOL/INT/…: every C caller, and every island decl
  generated from the header, reads all of EAX. That is the direction the first 202
  findings actually were (2026-09-26): 25 definitions — userenv's profile getters,
  win32k REGION_*/NtGdi*, advapi32 EncryptFile* — declared `BOOL` everywhere else. The
  decl-side check above cannot see a narrow definition only C calls, so this one reports
  the DEFINITION.

Usage:
  python3 tools/scan_bool_width.py            # scan sdk/lib/rust
  python3 tools/scan_bool_width.py --lint     # findings only (CLI baseline gate)
  python3 tools/scan_bool_width.py --json     # machine-readable
Exit 1 if findings (suitable for a 0-NEW baseline gate).
Suppress a verified FP with: // nolint: bool-width (reason)
"""

import argparse
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.c_twin import CIndex, prefer, same_module
from tools.source_walk import source_files

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


ROOTS = [RUST_ROOT]


def rust_files():
    for root in ROOTS:
        for f in [root] if root.is_file() else source_files(root):
            if SKIP_PARTS.isdisjoint(f.parts):
                yield f


def _rel(f: Path) -> str:
    try:
        return str(f.relative_to(ROOT))
    except ValueError:
        return str(f)


def collect_u8_defs():
    """name -> (file, ret) for every u8/BOOLEAN-returning definition."""
    defs = {}
    for f in rust_files():
        text = f.read_text(errors="ignore")
        for m in DEF_RE.finditer(text):
            if m.group(2) in ("u8", "BOOLEAN"):
                defs[m.group(1)] = (_rel(f), m.group(2))
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
                    "file": _rel(f),
                    "line": i,
                    "name": name,
                    "declared": m.group(2),
                    "defined": def_ret,
                    "definition": def_file,
                }
            )
    return findings


# A definition that C code can call: exported by symbol name.
EXPORT_DEF_RE = re.compile(
    r"#\[no_mangle\][^\n]*\n(?:\s*#\[[^\n]*\n)*\s*pub (?:unsafe )?extern "
    r'"(?:system|C|stdcall|win64|fastcall)" fn (\w+)\s*\((?:[^;{]*?)\)\s*->\s*(\w+)',
    re.S,
)
C_WIDE = {"BOOL", "INT", "int", "LONG", "ULONG", "DWORD", "UINT", "NTSTATUS", "HRESULT"}


def scan_narrow_defs():
    """Exported Rust definitions returning u8/BOOLEAN whose same-module C twin is wide."""
    defs = []
    rets: dict[tuple[Path, str], set[str]] = {}
    for f in rust_files():
        text = f.read_text(errors="ignore")
        for m in EXPORT_DEF_RE.finditer(text):
            rets.setdefault((f, m.group(1)), set()).add(m.group(2))
            if m.group(2) not in ("u8", "BOOLEAN"):
                continue
            ln = text.count("\n", 0, m.start(1)) + 1
            if "nolint: bool-width" in text.split("\n")[ln - 1]:
                continue
            defs.append((f, ln, m.group(1), m.group(2)))
    index = CIndex()
    index.load({d[2] for d in defs})
    findings = []
    for f, ln, name, ret in defs:
        twins = [t for t in prefer(f, index.get(name)) if same_module(f, t)]
        # Per-arch cfg twins of ONE export (i386 `fastcall` + a `C` body elsewhere) that
        # disagree on width: whichever is narrow is wrong on its arches, C twin or not.
        # Widening only the first copy left win32k's REGION_* narrow off-x86 (2026-09-26).
        wide_sibling = sorted(r for r in rets.get((f, name), ()) if r in WIDE_TYPES)
        if wide_sibling and not (twins and twins[0].ret in C_WIDE):
            findings.append(
                {
                    "file": _rel(f),
                    "line": ln,
                    "name": name,
                    "defined": ret,
                    "c_returns": "",
                    "c_file": "",
                    "twin_returns": wide_sibling[0],
                }
            )
            continue
        if not twins or twins[0].ret not in C_WIDE:
            continue
        findings.append(
            {
                "file": _rel(f),
                "line": ln,
                "name": name,
                "defined": ret,
                "c_returns": twins[0].ret,
                "c_file": str(twins[0].path.relative_to(ROOT)),
            }
        )
    return findings


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--lint", action="store_true")
    ap.add_argument("paths", nargs="*", type=Path, help="scan these instead of sdk/lib/rust")
    args = ap.parse_args()
    if args.paths:
        ROOTS[:] = args.paths
    findings = scan()
    narrow = scan_narrow_defs()
    if args.json:
        print(json.dumps({"decls": findings, "narrow_defs": narrow}, indent=2))
    else:
        for x in findings:
            print(
                f"{x['file']}:{x['line']}: {x['name']} declared -> {x['declared']}, "
                f"defined -> {x['defined']} ({x['definition']})"
            )
        for x in narrow:
            why = (
                f"its cfg twin returns {x['twin_returns']}"
                if x.get("twin_returns")
                else f"C returns {x['c_returns']} ({x['c_file']})"
            )
            print(f"{x['file']}:{x['line']}: {x['name']} defined -> {x['defined']}, {why}")
        if not args.lint:
            print(f"{len(findings) + len(narrow)} findings")
            if narrow:
                print("fix: return the C's type (BOOL is i32) from the definition, not u8.")
    return 1 if findings or narrow else 0


if __name__ == "__main__":
    sys.exit(main())
