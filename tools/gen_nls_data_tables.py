#!/usr/bin/env python3
"""One-off: convert kernel32 winnls/string pure-data tables (C → Rust).

casemap.c / collation.c are 100% Unicode data tables ("DO NOT EDIT", generated
from UnicodeData.txt / allkeys.txt) with zero logic, so rustconv yields 0
functions. This script does the mechanical C-array-literal → Rust-const lift so
the NLS family can be fully Rust-bodied in the cohab crate.

Each `const <TYPE> <name>[<LEN>] = { ... };` becomes
`#[no_mangle] pub static <name>: [<rust_ty>; <LEN>] = [ ... ];`. The C consumers
(sortkey.c, k32.h tolowerW/toupperW) link against the exact symbol — internal,
not in the .spec, WINE_UNICODE_API is empty — so #[no_mangle] (→ `_name` on
i686-gnu, `name` on amd64/arm64) matches the C extern references on every arch.

Usage:  python3 tools/gen_nls_data_tables.py
Writes: sdk/lib/rust/kernel32-rust/src/string/{casemap,collation}.rs
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "dll/win32/kernel32/winnls/string"
OUT = ROOT / "sdk/lib/rust/kernel32-rust/src/string"

# C type -> (Rust element type, value mask for sanity-check)
TYPE_MAP = {
    "WCHAR": ("u16", 0xFFFF),
    "unsigned int": ("u32", 0xFFFFFFFF),
}

# const <TYPE> <name>[<LEN>] = {   (TYPE may be multi-word like "unsigned int")
DECL_RE = re.compile(
    r"const\s+(unsigned\s+int|WCHAR)\s+(\w+)\s*\[\s*(\d+)\s*\]\s*=\s*\{",
)
COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
NUM_RE = re.compile(r"0[xX][0-9a-fA-F]+|\d+")


def strip_comments(text: str) -> str:
    return COMMENT_RE.sub(" ", text)


def find_matching_brace(text: str, open_idx: int) -> int:
    """Return index of the `}` matching the `{` at open_idx (no nested braces here)."""
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    raise ValueError("unbalanced braces")


def convert(c_path: Path, rs_path: Path) -> None:
    raw = c_path.read_text()
    arrays = []  # (name, rust_ty, declared_len, [values])
    for m in DECL_RE.finditer(raw):
        c_type = re.sub(r"\s+", " ", m.group(1)).strip()
        name = m.group(2)
        declared_len = int(m.group(3))
        rust_ty, mask = TYPE_MAP[c_type]
        open_idx = m.end() - 1  # position of the '{'
        close_idx = find_matching_brace(raw, open_idx)
        body = strip_comments(raw[open_idx + 1 : close_idx])
        vals = [int(tok, 0) for tok in NUM_RE.findall(body)]
        if len(vals) != declared_len:
            sys.exit(
                f"ERROR {c_path.name}:{name}: parsed {len(vals)} values "
                f"but declaration says [{declared_len}]"
            )
        bad = [v for v in vals if v < 0 or v > mask]
        if bad:
            sys.exit(f"ERROR {c_path.name}:{name}: {len(bad)} values out of {rust_ty} range")
        arrays.append((name, rust_ty, declared_len, vals))

    if not arrays:
        sys.exit(f"ERROR: no `const ... []= {{` arrays found in {c_path}")

    lines = [
        f"//! Auto-generated from winnls/string/{c_path.name} by "
        "tools/gen_nls_data_tables.py — DO NOT EDIT.",
        f"//! Pure Unicode data tables ({c_path.name} is generated, zero logic). The "
        "C consumers",
        "//! (sortkey.c / k32.h tolowerW/toupperW) link these #[no_mangle] symbols "
        "directly.",
        "#![allow(non_upper_case_globals)]",
        "",
    ]
    for name, rust_ty, n, vals in arrays:
        # #[used] so rustc keeps the symbol even though nothing in the Rust crate
        # references it — the only consumers are external C TUs (sortkey.c / k32.h).
        lines.append("#[no_mangle]")
        lines.append("#[used]")
        lines.append(f"pub static {name}: [{rust_ty}; {n}] = [")
        # 12 values per row for readability
        per_row = 12
        for i in range(0, n, per_row):
            row = ", ".join(f"0x{v:04x}" if rust_ty == "u16" else f"0x{v:08x}"
                            for v in vals[i : i + per_row])
            lines.append(f"    {row},")
        lines.append("];")
        lines.append("")
        print(f"  {c_path.name}: {name}[{n}] {rust_ty} -> {rs_path.name}")

    rs_path.write_text("\n".join(lines))


def main() -> None:
    convert(SRC / "casemap.c", OUT / "casemap.rs")
    convert(SRC / "collation.c", OUT / "collation.rs")
    print("done.")


if __name__ == "__main__":
    main()
