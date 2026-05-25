#!/usr/bin/env python3
"""freeldr_bitfield — fix bindgen bitfield member access in rustconv output.

bindgen lowers C bitfield structs (HARDWARE_PTE_X86, KGDTENTRY bytes,
etc.) to an opaque `_bitfield_1` plus `Field()` / `set_Field(v)` accessor
methods — there is NO real `Field` struct member. rustconv frequently
emits direct C-style member access instead:

    (*pte).Valid = 1;            ->  (*pte).set_Valid(1 as ULONG);
    x = (*pte).PageFrameNumber;  ->  x = (*pte).PageFrameNumber();

This rewrites read + write member access for a fixed allow-list of
bitfield member names (the freeldr paging surface). Deterministic +
idempotent: an access that is already a `()` call or `set_*(` is left
alone. Only the assignment RHS up to the terminating `;` on the same
logical line is captured (rustconv emits these one-per-line).

Usage: freeldr_bitfield.py <file.rs> [<file2.rs> ...]
"""
import re
import sys
from pathlib import Path

# HARDWARE_PTE_X86 / paging bitfield members seen in freeldr ntldr.
FIELDS = [
    "Valid", "Write", "Owner", "WriteThrough", "CacheDisable",
    "Accessed", "Dirty", "LargePage", "Global", "CopyOnWrite",
    "Prototype", "Reserved", "PageFrameNumber",
]
F = "|".join(FIELDS)

# write:  <lhs>).Field = <rhs>;   (rhs is the rest of the line up to ;)
WRITE_RE = re.compile(
    r'(?P<pre>\)\s*)\.(?P<f>' + F + r')\s*=\s*(?P<rhs>[^;]+);')
# read:  ).Field   NOT followed by `(`, `=`, or already a method
READ_RE = re.compile(
    r'\)\.(?P<f>' + F + r')\b(?!\s*\()(?!\s*=[^=])')


def fix(path):
    s = Path(path).read_text()
    o = s

    def w(m):
        return (m.group('pre') + ".set_" + m.group('f')
                + "(" + m.group('rhs').strip() + " as ULONG);")
    s = WRITE_RE.sub(w, s)

    def r(m):
        return ")." + m.group('f') + "()"
    s = READ_RE.sub(r, s)

    if s != o:
        Path(path).write_text(s)
    print(f"{path}: bitfield {'changed' if s != o else 'noop'}")


if __name__ == "__main__":
    for p in sys.argv[1:]:
        fix(p)
