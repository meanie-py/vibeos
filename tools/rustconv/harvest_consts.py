#!/usr/bin/env python3
"""harvest_consts.py — build a `glue.rs` of #define constants for a translated driver crate.

bindgen does NOT emit C `#define` macros, so LLM-translated driver crates are riddled with
E0425 "cannot find value FILE_DEVICE_DISK / TRACE_LEVEL_ERROR / STATUS_*". The mechanical LLM
pass FABRICATES these with WRONG values (the grind prompt's #1 hazard). This harvests the REAL
values straight from the C headers so they can't be guessed wrong.

For each unresolved name it greps the SDK headers for `#define NAME <value>` and parses the
UNAMBIGUOUS integer forms only:
  decimal / hex            -> pub const NAME: u32 = <v>;
  ((NTSTATUS)0xC000....)   -> pub const NAME: i32 = <v> as i32;   (status/error codes)
  ((TYPE)<int>)            -> cast stripped, typed by TYPE
Anything non-integer (GUIDs/DEFINE_GUID, expressions, string macros) is REPORTED, never guessed.

Usage: harvest_consts.py <crate-src-dir> <name1> <name2> ...
   (names default to stdin, one per line, if none given)
Writes <crate-src-dir>/glue.rs and prints a `pub mod glue;` reminder + the unharvested names.
"""
import sys, re, os, glob

CRATE_SRC = sys.argv[1]
names = sys.argv[2:] or [l.strip() for l in sys.stdin if l.strip()]
SRC = os.path.abspath(os.path.join(CRATE_SRC, "../../../../.."))  # repo root (src->crate->rust->lib->sdk->root)
HEADER_DIRS = [f"{SRC}/sdk/include/{d}" for d in
               ("ddk", "psdk", "xdk", "ndk", "reactos", "reactos/drivers",
                "reactos/drivers/pci", "")]
# driver-PRIVATE headers carry many #defines (pool tags, debug sizes) not in the SDK tree.
HEADER_DIRS += [f"{SRC}/drivers/storage/class/classpnp",
                f"{SRC}/drivers/bus/pcix",
                f"{SRC}/drivers/storage/partmgr",
                f"{SRC}/drivers/storage/mountmgr"]
HEADERS = []
for d in HEADER_DIRS:
    HEADERS += glob.glob(f"{d}/*.h")
HTEXT = {}
for h in HEADERS:
    try: HTEXT[h] = open(h, errors="ignore").read()
    except OSError: pass

def find_define(name):
    pat = re.compile(r"^[ \t]*#define[ \t]+" + re.escape(name) + r"[ \t]+(.+?)[ \t]*(?:/\*|//|$)", re.M)
    for txt in HTEXT.values():
        m = pat.search(txt)
        if m:
            return m.group(1).strip()
    return None

INT = re.compile(r"^(0[xX][0-9A-Fa-f]+|\d+)[uUlL]*$")
CAST = re.compile(r"^\(\(\s*([A-Za-z_][A-Za-z0-9_ ]*?)\s*\)\s*(0[xX][0-9A-Fa-f]+|\d+)[uUlL]*\s*\)$")
# C multi-char constant pool tag, e.g. 'ALcS' — packs big-endian (first char = high byte),
# which is exactly the value a C compiler computes and what the kernel's pool-tag dumps show.
FOURCC = re.compile(r"^'([\x20-\x7e]{1,4})'$")
SIGNED = ("NTSTATUS", "LONG", "int", "HRESULT")

def parse(name, v):
    # strip a single outer paren pair: `(0x18)` -> `0x18` (debug.h size defines wrap their value)
    vs = v.strip()
    if vs.startswith("(") and vs.endswith(")") and vs.count("(") == 1 and vs.count(")") == 1:
        v = vs[1:-1].strip()
    m = INT.match(v)
    if m:
        return f"pub const {name}: u32 = {m.group(1)};"
    m = CAST.match(v)
    if m:
        ty, num = m.group(1).strip(), m.group(2)
        if any(s in ty for s in SIGNED):
            return f"pub const {name}: i32 = {num}u32 as i32;"
        return f"pub const {name}: u32 = {num};"
    m = FOURCC.match(v)
    if m:
        val = 0
        for c in m.group(1):
            val = (val << 8) | ord(c)
        return f"pub const {name}: u32 = {hex(val)};  // {v}"
    return None  # GUID / expression / multi-token — don't guess

def main():
    out, unharvested = [], []
    for n in sorted(set(names)):
        v = find_define(n)
        line = parse(n, v) if v else None
        if line: out.append(line)
        else: unharvested.append(n)
    path = os.path.join(CRATE_SRC, "glue.rs")
    body = ("//! glue.rs — #define constants harvested from the C headers by\n"
            "//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).\n"
            "#![allow(non_upper_case_globals, dead_code)]\n\n" + "\n".join(out) + "\n")
    # merge with any existing hand-added glue
    if os.path.exists(path):
        prev = open(path).read()
        extra = [l for l in out if l.split("=")[0] not in prev]
        if extra:
            open(path, "a").write("\n// harvested:\n" + "\n".join(extra) + "\n")
    else:
        open(path, "w").write(body)
    print(f"harvested {len(out)} constants -> {path}")
    print("add `pub mod glue;` to lib.rs and `use crate::glue::*;` (or explicit) to modules.")
    if unharvested:
        print(f"NOT harvested ({len(unharvested)}, GUID/expr/enum-variant — handle separately):")
        print("  " + " ".join(unharvested))

if __name__ == "__main__":
    main()
