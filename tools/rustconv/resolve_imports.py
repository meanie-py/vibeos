#!/usr/bin/env python3
"""resolve_imports.py — deterministic import resolver for LLM-translated driver crates.

DeepSeek's mechanical translation leaves many `cannot find type/value X` (E0412/E0425/E0432)
errors: the symbol exists in ntkrnl_sys or the crate's own `types` module but was never
imported (or imported via the wrong path). This is pure mechanical resolution — far more
reliable done deterministically than by the LLM grind (which adds glob imports → ambiguity).

For each missing name it searches, in priority order:
  crate::types  →  ntkrnl_sys::ddk  →  ntkrnl_sys::{base,ke,ex,rtl,constants,globals,
                   driver_ddk,interlocked,ob,mm,ps}
resolving bare names, `_Name`→`Name` typedef aliases, and `_ENUM_Variant` enum constants.
It then adds a grouped explicit `use <module>::{...};` to each file that needs it (NEVER a
glob — globs cause E0659 ambiguity). Names it can't place are reported (likely #define
constants needing a glue.rs, or bindgen-missed private structs).

Usage: resolve_imports.py <crate-dir> <target-triple>
  e.g. resolve_imports.py .../classpnp-rust/src i686-pc-windows-gnu
"""
import sys, re, subprocess, os
from collections import defaultdict

CRATE_SRC = sys.argv[1]
TARGET = sys.argv[2]
CRATE = os.path.basename(os.path.dirname(CRATE_SRC))  # e.g. classpnp-rust
WS = os.path.abspath(os.path.join(CRATE_SRC, "../.."))  # sdk/lib/rust
NTK = os.path.join(WS, "ntkrnl-sys/src")
arch = {"i686-pc-windows-gnu": "i386", "x86_64-pc-windows-gnu": "amd64",
        "aarch64-pc-windows-gnu": "arm64"}[TARGET]

SOURCES = [("crate::types", os.path.join(CRATE_SRC, f"types_{arch}.rs")),
           ("crate::glue", os.path.join(CRATE_SRC, "glue.rs")),
           # crate-local pure-data modules (file-scope statics/GUID tables) some drivers
           # carry — module bodies reference these globals but don't import them.
           ("crate::data", os.path.join(CRATE_SRC, "data.rs")),
           ("crate::guid", os.path.join(CRATE_SRC, "guid.rs")),
           ("crate::globals", os.path.join(CRATE_SRC, "globals.rs")),
           ("ntkrnl_sys::ddk", os.path.join(NTK, f"bindgen_{arch}.rs"))]
for m in ("base", "ke", "ex", "rtl", "constants", "globals", "driver_ddk",
          "interlocked", "ob", "mm", "ps"):
    SOURCES.append((f"ntkrnl_sys::{m}", os.path.join(NTK, f"{m}.rs")))
SRC_TEXT = {p: open(p).read() for _, p in SOURCES if os.path.exists(p)}

# Crate-own MODULE files (sibling .rs) — for cross-module FUNCTION resolution: a `#[no_mangle]
# pub fn NAME` defined in module M is callable from module B only via `use crate::M::NAME`. We
# match only COLUMN-0 definitions (`^pub ... fn`) so indented extern-block decls (non-exportable)
# don't masquerade as the home of a symbol.
import glob as _glob
_SKIP = {"lib.rs", "types.rs", "glue.rs", "data.rs", "guid.rs", "globals.rs", "shims.rs"}
FN_SOURCES = [(f"crate::{os.path.basename(p)[:-3]}", p)
              for p in sorted(_glob.glob(os.path.join(CRATE_SRC, "*.rs")))
              if os.path.basename(p) not in _SKIP and not os.path.basename(p).startswith("types_")]
FN_TEXT = {p: open(p).read() for _, p in FN_SOURCES if os.path.exists(p)}
FN_DEF = r"(?m)^(?:#\[no_mangle\]\s*)?pub (?:unsafe )?(?:extern \"[A-Za-z]+\" )?fn "

DECL = r"\bpub (?:type|struct|enum|union|const|static(?: mut)?|fn) "
def resolve(name, cur_file=None):
    """Return (module, import_clause) or None. import_clause is what goes in use {...}."""
    for mod, p in SOURCES:
        s = SRC_TEXT.get(p, "")
        if re.search(DECL + re.escape(name) + r"\b", s) or \
           re.search(r"\bpub use [^\n;]*\b" + re.escape(name) + r"(\s|,|;|\})", s):
            return mod, name
    for mod, p in SOURCES:                       # _Name typedef alias
        if re.search(DECL + "_" + re.escape(name) + r"\b", SRC_TEXT.get(p, "")):
            return mod, f"_{name} as {name}"
    for mod, p in SOURCES:                       # _ENUM_Variant enum const
        m = re.search(r"\b(_[A-Z][A-Za-z0-9_]*_" + re.escape(name) + r")\b", SRC_TEXT.get(p, ""))
        if m:
            return mod, f"{m.group(1)} as {name}"
    for mod, p in FN_SOURCES:                    # cross-module pub fn definition
        if os.path.basename(p) == cur_file:      # don't self-import
            continue
        if re.search(FN_DEF + re.escape(name) + r"\b", FN_TEXT.get(p, "")):
            return mod, name
    return None

def check_errors():
    r = subprocess.run(["cargo", "check", "-p", CRATE, "--target", TARGET],
                       cwd=WS, capture_output=True, text=True)
    return r.stderr

def collect_missing(stderr):
    """file -> set(name). Parses cannot-find + unresolved-import with their file location."""
    out = defaultdict(set)
    cur = None
    for ln in stderr.splitlines():
        # E0412/E0425 (type/value), E0422 (struct/variant/union literal), and the
        # E0408 "variable `X` is not bound in all patterns" — all are missing-import symptoms
        # (an unimported uppercase const in a match arm silently becomes a binding).
        m = (re.search(r"cannot find (?:type|value) `([A-Za-z0-9_]+)`", ln)
             or re.search(r"cannot find struct, variant or union type `([A-Za-z0-9_]+)`", ln)
             or re.search(r"cannot find function(?:, tuple struct or tuple variant)? `([A-Za-z0-9_]+)`", ln)
             or re.search(r"variable `([A-Za-z0-9_]+)` is not bound in all patterns", ln))
        if m: cur = m.group(1); continue
        m = re.search(r"-->\s+\S*?/src/([A-Za-z0-9_]+\.rs):", ln)
        if m and cur: out[m.group(1)].add(cur); cur = None
    return out

def main():
    missing = collect_missing(check_errors())
    per_file_adds = defaultdict(lambda: defaultdict(list))  # file -> module -> [clauses]
    unresolved = set()
    for fname, names in missing.items():
        for n in sorted(names):
            r = resolve(n, cur_file=fname)
            if r: per_file_adds[fname][r[0]].append(r[1])
            else: unresolved.add(n)
    added = 0
    for fname, mods in per_file_adds.items():
        path = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(path): continue
        s = open(path).read()
        lines = []
        for mod, clauses in sorted(mods.items()):
            clauses = sorted(set(clauses))
            lines.append(f"use {mod}::{{{', '.join(clauses)}}};")
            added += len(clauses)
        # insert after the last existing top `use ...;` line
        m = list(re.finditer(r"^use [^\n]*;\n", s, re.M))
        ins = m[-1].end() if m else 0
        s = s[:ins] + "// resolve_imports.py: explicit kernel/private-type imports\n" + \
            "\n".join(lines) + "\n" + s[ins:]
        open(path, "w").write(s)
    print(f"added {added} explicit imports across {len(per_file_adds)} files")
    if unresolved:
        print(f"UNRESOLVED ({len(unresolved)}) — likely #define consts (need glue.rs) "
              f"or bindgen-missed structs:")
        print("  " + " ".join(sorted(unresolved)))

if __name__ == "__main__":
    main()
