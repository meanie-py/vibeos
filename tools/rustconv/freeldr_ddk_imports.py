#!/usr/bin/env python3
"""freeldr_ddk_imports — fix rustconv's broken `use ntkrnl_sys::ddk::{...}`.

rustconv's resolver emits a flat `use ntkrnl_sys::ddk::{A, B, ...}` that
mixes three things:
  1. real bare ddk symbols (types/aliases: ARC_STATUS, LARGE_INTEGER) — keep.
  2. bindgen *enum variants*, which bindgen names type-prefixed
     `_<TAG>_<Variant>` (e.g. `_CONFIGURATION_TYPE_DiskPeripheral`), NOT
     bare `DiskPeripheral` → rewrite to a `pub use ... as <Variant>;` alias.
  3. junk: local identifiers / params (Red, Init, Depth, Type, Output...)
     the resolver over-collected; they exist nowhere in ddk → drop.

Deterministic: classification is by grepping the arch bindgen file +
ddk re-export surface. Idempotent. Reusable across every freeldr unit
(this is the #1 systemic rustconv-output defect for hardware files).

Usage: python3 freeldr_ddk_imports.py <bindgen_i386.rs> <file.rs> [<file2.rs> ...]
"""
import re
import sys
from pathlib import Path

USE_RE = re.compile(
    r'^use ntkrnl_sys::ddk::\{([^}]*)\};[ \t]*\n', re.M)


def build_index(bindgen_path: str):
    txt = Path(bindgen_path).read_text()
    bare = set()      # names defined bare in ddk (type/struct/const/fn)
    prefixed = {}     # variant -> full _TAG_variant symbol (first wins)
    # NOTE: `pub fn`/`pub static` inside `extern "C" { ... }` blocks are
    # INDENTED — allow leading whitespace or the bare detector misses
    # every extern symbol (e.g. RtlUshortByteSwap) and the import-fixer
    # then wrongly strips them, re-breaking the build on re-run.
    for m in re.finditer(
            r'^\s*pub (?:type|struct|const|union|enum|static)\s+'
            r'(?:mut\s+)?([A-Za-z_]\w*)',
            txt, re.M):
        bare.add(m.group(1))
    # `pub fn` separately: a real importable bare fn lives in an
    # `extern { }` block or at module top-level. bindgen ALSO emits
    # `pub fn Name(&self)` / `pub unsafe fn Name_raw(this: *const Self)`
    # as bitfield ACCESSOR METHODS inside `impl` blocks — those are NOT
    # importable items; indexing them as bare wrongly keeps junk imports
    # like `File`/`Type`/`PaeEnabled` (a C param name colliding with a
    # bitfield accessor). Exclude method-shaped fns by their receiver.
    for m in re.finditer(
            r'^\s*pub (?:unsafe |const )*fn\s+([A-Za-z_]\w*)\s*\(([^)]*)',
            txt, re.M):
        name, args = m.group(1), m.group(2)
        if re.match(r'\s*(?:&|mut\s|self\b|this\s*:)', args):
            continue  # impl-method accessor — not an importable symbol
        bare.add(name)
    # bindgen enum variants: `pub const _SOMETAG_Variant: _SOMETAG = N;`
    for m in re.finditer(
            r'^\s*pub const (_[A-Z0-9]+(?:_[A-Z0-9]+)*_([A-Za-z]\w*))\s*:',
            txt, re.M):
        full, variant = m.group(1), m.group(2)
        prefixed.setdefault(variant, full)
    return bare, prefixed


def fix(path, bare, prefixed):
    s = Path(path).read_text()
    aliases = []
    changed = False

    def repl(m):
        nonlocal changed
        names = [n.strip() for n in m.group(1).split(',') if n.strip()]
        keep, dropped = [], []
        for n in names:
            if n in bare:
                keep.append(n)
            elif n in prefixed:
                aliases.append(
                    f"pub use ntkrnl_sys::ddk::{prefixed[n]} as {n};")
            else:
                dropped.append(n)
        changed = True
        line = ""
        if keep:
            line = "use ntkrnl_sys::ddk::{" + ", ".join(keep) + "};\n"
        if dropped:
            line += "// freeldr_ddk_imports dropped (not in ddk): " \
                    + ", ".join(dropped) + "\n"
        return line

    s2 = USE_RE.sub(repl, s, count=1)
    if aliases:
        # place alias lines right after the (rewritten) use block / top
        s2 = s2.replace(
            "use core::ffi::c_void;",
            "use core::ffi::c_void;\n" + "\n".join(aliases), 1)
        if "\n".join(aliases) not in s2:  # no c_void anchor — prepend
            s2 = "\n".join(aliases) + "\n" + s2
    if s2 != s:
        Path(path).write_text(s2)
    print(f"{path}: ddk_imports "
          f"{'changed (+%d alias)' % len(aliases) if s2 != s else 'noop'}")


if __name__ == "__main__":
    bindgen = sys.argv[1]
    bare, prefixed = build_index(bindgen)
    for p in sys.argv[2:]:
        fix(p, bare, prefixed)
