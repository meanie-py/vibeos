#!/usr/bin/env python3
"""freeldr_debugcast — coerce every variadic debug-call arg to `usize`.

TRACE/ERR/WARN/FIXME/DPRINTM/DbgPrint are no-op stubs in release (DBG off;
boot-to-shell needs no debug output — same disposition as the debug.rs unit).
The *calls* still have to typecheck against the C-variadic `...` decl, and
Rust forbids u8/u16/usize-via-arith/pointers etc. as un-promoted variadic
args (E0617) and flags the width mismatches (E0308). Since the callee does
nothing, the only requirement is type validity, so we uniformly coerce
every arg after the format string to `usize` (FFI-safe, Copy, accepts
ints/pointers/enums alike via `as`).

Paren/bracket/brace/string-aware top-level arg split. Idempotent (skips
args already wrapped `... as usize`). Reusable for fs + ui units.

Usage: python3 freeldr_debugcast.py <file.rs> [<file2.rs> ...]
"""
import re
import sys
from pathlib import Path

FNS = ("TRACE", "ERR", "WARN", "FIXME", "DPRINTM", "DbgPrint")
NAME_RE = re.compile(r'(?<![A-Za-z0-9_])(' + "|".join(FNS) + r')\(')


def split_args(s: str):
    """Top-level comma split, aware of () [] {} and "" '' and raw byte strs."""
    args, depth, i, start = [], 0, 0, 0
    in_str = None  # the closing quote char, or None
    while i < len(s):
        c = s[i]
        if in_str is not None:
            if c == '\\':
                i += 2
                continue
            if c == in_str:
                in_str = None
        elif c in '"\'':
            in_str = c
        elif c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        elif c == ',' and depth == 0:
            args.append(s[start:i])
            start = i + 1
        i += 1
    args.append(s[start:])
    return args


def find_call_end(s: str, open_paren: int) -> int:
    """Index of the matching ')' for the '(' at open_paren."""
    depth, i, in_str = 0, open_paren, None
    while i < len(s):
        c = s[i]
        if in_str is not None:
            if c == '\\':
                i += 2
                continue
            if c == in_str:
                in_str = None
        elif c in '"\'':
            in_str = c
        elif c == '(':
            depth += 1
        elif c == ')':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def fix(path: Path) -> None:
    s = path.read_text()
    out = []
    pos = 0
    n_calls = 0
    for m in NAME_RE.finditer(s):
        # skip the variadic *declaration* `fn TRACE(fmt: *const i8, ...)`
        pre = s[max(0, m.start() - 8):m.start()]
        if re.search(r'\bfn\s+$', pre):
            continue
        op = m.end() - 1  # index of '('
        end = find_call_end(s, op)
        if end < 0:
            continue
        inner = s[op + 1:end]
        if inner.strip() == "":
            continue
        args = split_args(inner)
        if len(args) < 2:
            continue
        new_args = [args[0]]
        changed = False
        for a in args[1:]:
            a_str = a.strip()
            if not a_str:
                new_args.append(a)
                continue
            if re.search(r'\bas\s+usize\s*$', a_str):
                new_args.append(a)
                continue
            new_args.append(f" ({a_str}) as usize")
            changed = True
        if not changed:
            continue
        out.append(s[pos:op + 1])
        out.append(",".join(new_args))
        pos = end
        n_calls += 1
    out.append(s[pos:])
    path.write_text("".join(out))
    print(f"{path.name}: debugcast {n_calls} call(s)")


if __name__ == "__main__":
    for f in sys.argv[1:]:
        fix(Path(f))
