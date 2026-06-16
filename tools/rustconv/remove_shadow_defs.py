#!/usr/bin/env python3
"""remove_shadow_defs.py — delete c2rust "fabrication stub" type definitions that
shadow the bindgen substrate (crate::types).

c2rust, unable to resolve a type, emits a local approximation (`struct
CDROM_DEVICE_EXTENSION { .. }`, `struct CDB;`, `type WDFREQUEST = ..`) in the module.
Once the real bindgen layout is available via `use crate::types::*`, these locals
SHADOW it — every field access then hits the stub's (wrong/partial) shape, and a
double-def is E0428. Removing the shadowing locals makes the module use the exact
substrate layout.

Removes top-level `struct|enum|union NAME {..}` / `struct NAME;` / `type NAME = ..;`
(plus contiguous preceding #[attr]/doc-comment lines) when NAME is in the provided
crate::types name set. Brace-matched for multi-line bodies. Leaves genuinely
module-local types (not in the substrate) untouched.

Usage: remove_shadow_defs.py <names-file> <crate-src-dir> [module.rs ...]
"""
import re
import sys
import pathlib

DEF_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:struct|enum|union)\s+([A-Za-z0-9_]+)\b"
    r"|^\s*(?:pub\s+)?type\s+([A-Za-z0-9_]+)\s*="
)


def process(text: str, names: set):
    lines = text.splitlines(keepends=True)
    out = []
    i = 0
    removed = []
    while i < len(lines):
        m = DEF_RE.match(lines[i])
        name = m.group(1) or m.group(2) if m else None
        if name and name in names:
            # find statement end: '{'-matched body, or first ';'
            depth = 0
            seen_brace = False
            j = i
            done = False
            while j < len(lines):
                for ch in lines[j]:
                    if ch == "{":
                        depth += 1
                        seen_brace = True
                    elif ch == "}":
                        depth -= 1
                        if seen_brace and depth == 0:
                            done = True
                            break
                    elif ch == ";" and not seen_brace and depth == 0:
                        done = True
                        break
                if done:
                    break
                j += 1
            # drop contiguous preceding attribute / doc-comment lines already in out
            while out and re.match(r"^\s*(#\[|///|//!)", out[-1]):
                out.pop()
            removed.append(name)
            i = j + 1
            continue
        out.append(lines[i])
        i += 1
    return "".join(out), removed


def main():
    names = set(open(sys.argv[1]).read().split())
    src = pathlib.Path(sys.argv[2])
    files = (
        [src / f for f in sys.argv[3:]]
        if len(sys.argv) > 3
        else [f for f in sorted(src.glob("*.rs"))
              if f.name not in {"lib.rs", "glue.rs", "types.rs"} and not f.name.startswith("types_")]
    )
    for f in files:
        text = f.read_text()
        text, removed = process(text, names)
        f.write_text(text)
        if removed:
            print(f"  {f.name}: removed {len(removed)} shadow defs: {', '.join(sorted(set(removed)))}")


if __name__ == "__main__":
    main()
