#!/usr/bin/env python3
"""Strip C++-mangled (`_Z...`) extern fn decls from a bindgen types_*.rs.

A C++ driver (uniata) compiled without `extern "C"` makes bindgen emit its OWN
internal functions as `#[link_name="\\u{1}...Z..."] pub fn` blocks. In an all-Rust
cohab those decls are dead (the Rust modules DEFINE these functions) AND they collide
name-for-name with the translated modules under a flat re-export. A handful of genuine
kernel imports can also come out mangled (e.g. ScsiPortGetBusDataByOffset) — those are
re-provided with correct C linkage in the crate's foundation, so dropping them here is
safe. Removing every `unsafe extern {...}` block whose body carries a `_Z` link_name
leaves the structs, type aliases, consts and correctly-C-linked kernel decls intact.

Usage:  strip_cpp_mangled_externs.py types_i386.rs [types_amd64.rs ...]
Idempotent. Prints how many blocks were removed per file.
"""
import re
import sys


def strip(text: str) -> tuple[str, int]:
    lines = text.splitlines(keepends=True)
    out = []
    i = 0
    removed = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        if re.match(r"\s*unsafe extern \"[^\"]+\" \{\s*$", line):
            # collect the block until its closing brace at the same indent
            j = i + 1
            depth = 1
            block = [line]
            while j < n and depth > 0:
                block.append(lines[j])
                depth += lines[j].count("{") - lines[j].count("}")
                j += 1
            body = "".join(block)
            if re.search(r'link_name = "[^"]*_Z', body):
                removed += 1
                i = j
                continue  # drop the whole block
            out.extend(block)
            i = j
        else:
            out.append(line)
            i += 1
    return "".join(out), removed


def main() -> int:
    for path in sys.argv[1:]:
        with open(path, "r") as f:
            text = f.read()
        new, removed = strip(text)
        if removed:
            with open(path, "w") as f:
                f.write(new)
        print(f"{path}: removed {removed} C++-mangled extern blocks")
    return 0


if __name__ == "__main__":
    sys.exit(main())
