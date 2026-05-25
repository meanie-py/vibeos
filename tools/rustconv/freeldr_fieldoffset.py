#!/usr/bin/env python3
"""freeldr_fieldoffset — rewrite rustconv's FIELD_OFFSET/RTL_SIZEOF_THROUGH_FIELD
call-syntax into the keystone macro invocation.

C `FIELD_OFFSET(type, field)` is a macro; rustconv emits it as a plain
call `FIELD_OFFSET(Type, field)` (and sometimes declares a bogus
`fn FIELD_OFFSET(...)` extern). The keystone defines a `macro_rules!
FIELD_OFFSET`, which must be invoked with `!`. This pass:

  1. drops any `fn FIELD_OFFSET(...)` / `fn RTL_SIZEOF_THROUGH_FIELD(...)`
     extern decl line (no such symbol; it's a macro).
  2. rewrites `FIELD_OFFSET(T, f)` → `FIELD_OFFSET!(T, f)` (idempotent:
     skips occurrences already followed by `!`).

Deterministic, idempotent. Reusable across every freeldr unit that
touches struct layout (pcdisk, winldr, uefi*, setup*).

Usage: python3 freeldr_fieldoffset.py <file.rs> [<file2.rs> ...]
"""
import re
import sys

NAMES = ("FIELD_OFFSET", "RTL_SIZEOF_THROUGH_FIELD")


def fix(path: str) -> None:
    s = open(path).read()
    o = s
    # 1. drop bogus extern fn decls for the macros
    for n in NAMES:
        s = re.sub(r'^\s*fn\s+' + n + r'\s*\([^;]*\)\s*->[^;]*;\s*\n', '', s,
                   flags=re.M)
    # 2. call → crate::macro!  (macro_export lives at crate root; a glob
    #    `use crate::frldr::*` does NOT bring it, so fully-qualify).
    #    Skip occurrences already `!`-invoked or already crate-qualified.
    for n in NAMES:
        s = re.sub(r'(?<!::)\b' + n + r'\s*\((?!\s*\))',
                   'crate::' + n + '!(', s)
        # qualify any already-rewritten bare `NAME!(` (prior script ver)
        s = re.sub(r'(?<!::)\b' + n + r'!\(', 'crate::' + n + '!(', s)
        s = s.replace('crate::crate::', 'crate::')
        s = s.replace(n + '!!(', n + '!(')  # idempotent guard
    if s != o:
        open(path, "w").write(s)
    print(f"{path}: fieldoffset {'changed' if s != o else 'noop'}")


if __name__ == "__main__":
    for p in sys.argv[1:]:
        fix(p)
