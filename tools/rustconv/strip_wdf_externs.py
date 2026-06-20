#!/usr/bin/env python3
"""strip_wdf_externs.py — prepare a c2rust'd KMDF driver crate for the kmdf-sys substrate.

c2rust translates a KMDF driver as if every `Wdf*` API were a direct C export, emitting
per-module `extern "C" { fn WdfXxx(..); }` declarations with loose/wrong signatures
(e.g. `MinimumRequiredLength: u32` where the real API is `usize`). Real KMDF has no such
exports — the APIs dispatch through the WdfFunctions[] table, which kmdf-sys provides as
inline `pub unsafe fn WdfXxx`. A local extern decl SHADOWS the kmdf-sys import (and is
undefined at link), so each must be removed and replaced by `use kmdf_sys::*`.

This does two mechanical edits per module:
  1. strip every `fn Wdf...( .. );` declaration (single- or multi-line) from extern blocks;
  2. inject `use crate::types::*;` + `use kmdf_sys::*;` at the top (WDF types + dispatch
     fns; types are single-identity with crate::types so the two globs don't conflict).

Idempotent: skips injection if the imports are already present.

Usage: python3 tools/rustconv/strip_wdf_externs.py <crate-src-dir> [module.rs ...]
       (default: all *.rs in the dir except lib.rs/glue.rs/types*.rs)
"""
import re
import sys
import pathlib

IMPORTS = "use crate::types::*;\nuse kmdf_sys::*;\n"


def strip_wdf_decls(text: str):
    """Remove `fn Wdf...(...);` statements (declarations) wherever they appear."""
    lines = text.splitlines(keepends=True)
    out = []
    i = 0
    removed = 0
    while i < len(lines):
        if re.match(r"\s*fn Wdf[A-Za-z0-9_]*\s*\(", lines[i]):
            # consume through the statement-terminating ';' at paren depth <= 0
            depth = 0
            j = i
            done = False
            while j < len(lines):
                for ch in lines[j]:
                    if ch == "(":
                        depth += 1
                    elif ch == ")":
                        depth -= 1
                    elif ch == ";" and depth <= 0:
                        done = True
                        break
                if done:
                    break
                j += 1
            removed += 1
            i = j + 1
            continue
        out.append(lines[i])
        i += 1
    return "".join(out), removed


def inject_imports(text: str):
    if "use kmdf_sys::" in text:
        return text, False
    return IMPORTS + text, True


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    src = pathlib.Path(sys.argv[1])
    if len(sys.argv) > 2:
        files = [src / f for f in sys.argv[2:]]
    else:
        skip = {"lib.rs", "glue.rs", "types.rs"}
        files = [
            f
            for f in sorted(src.glob("*.rs"))
            if f.name not in skip and not f.name.startswith("types_")
        ]
    for f in files:
        text = f.read_text()
        text, removed = strip_wdf_decls(text)
        text, injected = inject_imports(text)
        f.write_text(text)
        print(f"  {f.name}: stripped {removed} Wdf decls, imports {'added' if injected else 'present'}")


if __name__ == "__main__":
    main()
