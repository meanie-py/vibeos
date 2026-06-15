#!/usr/bin/env python3
"""
scan_inline_flag_literals.py — catch WRONG inlined flag/constant literals.

verify_constants.py only checks `const NAME: T = VALUE;` definitions. The LLM
grind also inlines BARE literals annotated with the intended name in a comment,
e.g.

    SetFlag(&Flags, 0x2 /* IRP_CONTEXT_FLAG_TOP_LEVEL */);   // WRONG: 0x2 = MORE_PROCESSING
    if SafeNodeType(x) != /* CDFS_NTC_IRP_CONTEXT */ 0x0 {   // WRONG: should be 0x0308

Those are invisible to verify_constants.py (no named const). This scans Rust for
`<literal> /* NAME */`, `/* NAME */ <literal>`, and `<literal>, // NAME` where NAME
is a known C constant, and flags any literal that disagrees with the header value.

Usage:
    python3 tools/scan_inline_flag_literals.py <file.rs ...> [--headers DIR ...]
    python3 tools/scan_inline_flag_literals.py sdk/lib/rust/cdfs-rust/src/*.rs \
        --headers drivers/filesystems/cdfs

Builds the name->value catalog from the kernel headers verify_constants.py uses
PLUS every *.h under each --headers dir (driver-local flags like IRP_CONTEXT_FLAG_*,
CCB_FLAG_*, CDFS_NTC_*). Exit 1 if any mismatch is found.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools" / "rustconv"))
from rustconv.verify_constants import build_c_catalog, parse_c_hex  # noqa: E402

DEFINE_RE = re.compile(r"^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)(?:\s*//.*)?$")

# A literal (hex or decimal) adjacent to a /* NAME */ or // NAME comment naming a
# constant. Three spellings the grind emits.
PATTERNS = [
    re.compile(
        r"(0x[0-9a-fA-F]+|\b\d+)\s*(?:u32|i32|u16|u8)?\s*/\*\s*([A-Za-z_][A-Za-z0-9_]*)[^*]*\*/"
    ),
    re.compile(r"/\*\s*([A-Za-z_][A-Za-z0-9_]*)[^*]*\*/\s*(0x[0-9a-fA-F]+|\b\d+)\b"),
    re.compile(
        r"(0x[0-9a-fA-F]+|\b\d+)\s*(?:u32|i32|u16|u8)?\s*,?\s*//\s*([A-Za-z_][A-Za-z0-9_]*)"
    ),
]


def parse_c_value(val):
    """Parse a #define RHS: 0x10, 16, (0x10), ((NODE_TYPE_CODE)0x0308), (ULONG)5."""
    s = val.strip()
    # strip balanced outer parens repeatedly
    while s.startswith("(") and s.endswith(")"):
        s = s[1:-1].strip()
    # strip a leading C cast like (NODE_TYPE_CODE) / (ULONG) / (NTSTATUS)
    s = re.sub(r"^\(\s*[A-Za-z_][A-Za-z0-9_ ]*\)\s*", "", s).strip()
    s = s.rstrip("uUlL")  # trailing int suffixes
    try:
        return int(s, 16) if s.lower().startswith("0x") else int(s)
    except ValueError:
        return parse_c_hex(s)


def add_headers(catalog, header_dir):
    for hf in Path(header_dir).rglob("*.h"):
        try:
            text = hf.read_text(errors="ignore")
        except OSError:
            continue
        text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
        for line in text.splitlines():
            m = DEFINE_RE.match(line.strip())
            if not m:
                continue
            name, val = m.group(1), m.group(2).strip()
            v = parse_c_value(val)
            if v is not None:
                catalog.setdefault(name, v)  # don't override kernel catalog


def lit_value(s):
    # strip a Rust integer type SUFFIX (not chars — rstrip would eat 0x2 -> 0x).
    s = re.sub(r"(u8|u16|u32|u64|usize|i8|i16|i32|i64|isize)$", "", s.strip())
    try:
        return int(s, 16) if s.lower().startswith("0x") else int(s)
    except ValueError:
        return None


def main():
    args = sys.argv[1:]
    files, header_dirs, i = [], [], 0
    while i < len(args):
        if args[i] == "--headers":
            i += 1
            header_dirs.append(args[i])
        else:
            files.append(args[i])
        i += 1
    if not files:
        print("usage: scan_inline_flag_literals.py <file.rs ...> [--headers DIR ...]")
        return 2

    catalog = build_c_catalog(REPO, include_enums=True)
    for d in header_dirs:
        add_headers(catalog, d)

    mismatches = 0
    for f in files:
        p = Path(f)
        if not p.exists():
            continue
        for ln, line in enumerate(p.read_text(errors="ignore").splitlines(), 1):
            if line.lstrip().startswith("//"):
                continue
            for pat in PATTERNS:
                for m in pat.finditer(line):
                    g = m.groups()
                    # group order differs per pattern; find the name + literal
                    name = next((x for x in g if x and re.match(r"^[A-Za-z_]", x)), None)
                    lit = next((x for x in g if x and re.match(r"^(0x|\d)", x)), None)
                    if not name or lit is None or name not in catalog:
                        continue
                    lv = lit_value(lit)
                    if lv is None:
                        continue
                    want = catalog[name]
                    if lv != want:
                        # ignore obviously-unrelated comments (e.g. a value tagged
                        # with a function name); only flag when the value differs.
                        print(
                            f"{f}:{ln}: literal {lit} tagged /* {name} */ "
                            f"but {name} = {hex(want)} ({want})"
                        )
                        print(f"    {line.strip()}")
                        mismatches += 1
    if mismatches:
        print(f"\n{mismatches} inlined-literal mismatch(es) — fix to the header value.")
        return 1
    print("OK: no inlined flag-literal mismatches.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
