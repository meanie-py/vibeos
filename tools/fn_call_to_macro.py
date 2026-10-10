#!/usr/bin/env python3
"""
fn_call_to_macro.py -- turn calls of a C debug macro that a translation lowered to an
out-of-line `fn` back into calls of the crate's `macro_rules!`.

    TRACE(b"Cache miss! BlockNumber: %d\\n\\0".as_ptr() as *const i8, BlockNumber);
 -> TRACE!("Cache miss! BlockNumber: %d\\n", BlockNumber);

WHY A TOOL. `scan_macro_fn_shadow` reports ONE row per no-op definition, but the
definition is not where the damage is: freeldr's four `#[no_mangle]` no-ops (TRACE,
ERR, WARN, FIXME) served ~1,900 call sites through 80+ variadic `extern` declarations,
and every one of them was a diagnostic the DBG build prints and ours dropped. That is
far past hand-editing, and each site has the same shape, so it is a rewrite with a
refusal rule rather than a judgement call.

THE REFUSAL RULE. With `--fmt-literal` the first argument must be a NUL-terminated
string literal handed over as a pointer (`b"...\\0".as_ptr() as T`, `c"...".as_ptr()`),
because the target macro takes `$fmt:literal` and appends its own NUL. A byte escape
that has no `str` spelling (`\\xNN` with NN >= 0x80) is refused too. A refused site is
printed and left untouched, and so is the `extern` declaration that still serves it:
the tool never leaves a call without a callee.

Without `--fmt-literal` the arguments are copied verbatim, which is only right when the
macro discards its tokens (`($($t:tt)*) => {{}}`): ole32's TRACE/WARN.

Usage:
    python3 tools/fn_call_to_macro.py --names TRACE,ERR,WARN,FIXME --fmt-literal \\
        sdk/lib/rust/freeldr-rust/src [--dry-run]
    python3 tools/fn_call_to_macro.py --self-test
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

_IDENT = re.compile(r"[A-Za-z0-9_]")


def _skip_literal(src: str, i: int) -> int | None:
    """If a string/char literal or comment starts at `i`, return the index just past it."""
    if src.startswith("//", i):
        j = src.find("\n", i)
        return len(src) if j < 0 else j
    if src.startswith("/*", i):
        depth, j = 0, i
        while j < len(src):
            if src.startswith("/*", j):
                depth, j = depth + 1, j + 2
            elif src.startswith("*/", j):
                depth, j = depth - 1, j + 2
                if depth == 0:
                    return j
            else:
                j += 1
        return len(src)
    m = re.compile(r'(?:b|c)?r(#*)"').match(src, i)
    if m and (i == 0 or not _IDENT.match(src[i - 1])):
        end = src.find('"' + m.group(1), m.end())
        return len(src) if end < 0 else end + 1 + len(m.group(1))
    m = re.compile(r'(?:b|c)?"').match(src, i)
    if m and (i == 0 or not _IDENT.match(src[i - 1]) or src[i] == '"'):
        j = m.end()
        while j < len(src) and src[j] != '"':
            j += 2 if src[j] == "\\" else 1
        return j + 1
    m = re.compile(r"b?'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'])'").match(src, i)
    if m and (i == 0 or not _IDENT.match(src[i - 1]) or src[i] == "'"):
        return m.end()
    return None


def _match_paren(src: str, open_at: int) -> int | None:
    """Index of the `)` closing the `(` at `open_at`, skipping literals and comments."""
    depth, i = 0, open_at
    while i < len(src):
        skip = _skip_literal(src, i)
        if skip is not None:
            i = skip
            continue
        ch = src[i]
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def _split_args(inner: str) -> list[str]:
    args, depth, start, i = [], 0, 0, 0
    while i < len(inner):
        skip = _skip_literal(inner, i)
        if skip is not None:
            i = skip
            continue
        ch = inner[i]
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            args.append(inner[start:i])
            start = i + 1
        i += 1
    if inner[start:].strip():
        args.append(inner[start:])
    return [a.strip() for a in args]


_FMT_RE = re.compile(
    r'^(?P<kind>b|c)"(?P<body>(?:[^"\\]|\\.)*)"\s*\.as_ptr\(\)(?:\s+as\s+[\w:*\s]+?)*$', re.S
)


def fmt_to_str_literal(arg: str) -> str | None:
    """`b"x\\n\\0".as_ptr() as T` -> `"x\\n"`; None when it has no faithful `str` form."""
    while arg.startswith("(") and _match_paren(arg, 0) == len(arg) - 1:
        arg = arg[1:-1].strip()
    m = _FMT_RE.match(arg)
    if not m:
        return None
    body = m.group("body")
    if m.group("kind") == "b":
        if not body.endswith("\\0"):
            return None
        body = body[:-2]
    # The macro appends the NUL; an embedded one would truncate the format there.
    for esc in re.finditer(r"\\(x[0-9a-fA-F]{2}|.)", body, re.S):
        e = esc.group(1)
        if e == "0" or (e.startswith("x") and int(e[1:], 16) >= 0x80):
            return None
    if any(ord(ch) >= 0x80 for ch in body):
        return None
    return f'"{body}"'


def rewrite(
    src: str, names: set[str], fmt_literal: bool, prefix: str = ""
) -> tuple[str, int, list[tuple[int, str]]]:
    """(new source, converted count, (1-based line, name) of each refused call)."""
    alt = "|".join(sorted(names, key=len, reverse=True))
    call = re.compile(rf"(?<![A-Za-z0-9_:!.])({alt})\s*\(")
    out, i, converted, refused = [], 0, 0, []
    while i < len(src):
        skip = _skip_literal(src, i)
        if skip is not None:
            out.append(src[i:skip])
            i = skip
            continue
        m = call.match(src, i)
        if m is None:
            out.append(src[i])
            i += 1
            continue
        # A declaration or definition (`fn TRACE(`), not a call.
        if re.search(r"\bfn\s+$", src[max(0, i - 16) : i]):
            out.append(m.group(0))
            i = m.end()
            continue
        close = _match_paren(src, m.end() - 1)
        if close is None:
            refused.append((src.count("\n", 0, i) + 1, m.group(1)))
            out.append(m.group(0))
            i = m.end()
            continue
        args = _split_args(src[m.end() : close])
        if fmt_literal:
            lit = fmt_to_str_literal(args[0]) if args else None
            if lit is None:
                refused.append((src.count("\n", 0, i) + 1, m.group(1)))
                out.append(m.group(0))
                i = m.end()
                continue
            args[0] = lit
        out.append(f"{prefix}{m.group(1)}!({', '.join(args)})")
        converted += 1
        i = close + 1
    return "".join(out), converted, refused


_EXTERN_DECL = (
    r"^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?fn\s+({alt})\s*"
    r"\([^;{{]*\)\s*(?:->\s*[^;{{]+)?;[ \t]*\n"
)


def drop_extern_decls(src: str, names: set[str]) -> str:
    """Remove bodyless `fn NAME(..);` declarations (they live only in `extern` blocks)."""
    alt = "|".join(sorted(names))
    return re.sub(_EXTERN_DECL.format(alt=alt), "", src, flags=re.M)


def add_import(src: str, used: list[str]) -> str:
    """`use crate::{..}` after the file's leading inner attributes and `//!` docs."""
    line = f"#[allow(unused_imports)] use crate::{{{', '.join(used)}}};\n"
    lines = src.splitlines(keepends=True)
    at = 0
    for idx, ln in enumerate(lines):
        s = ln.strip()
        if s.startswith("//!") or s.startswith("#![") or not s:
            at = idx + 1
            continue
        break
    return "".join([*lines[:at], line, *lines[at:]])


def process(
    path: Path,
    names: set[str],
    fmt_literal: bool,
    dry: bool,
    import_: bool = True,
    prefix: str = "",
) -> tuple[int, list[tuple[int, str]]]:
    src = path.read_text("utf8")
    new, converted, refused = rewrite(src, names, fmt_literal, prefix)
    # Only a name with no refused call left in this file may lose its declaration.
    new = drop_extern_decls(new, names - {n for _, n in refused})
    used = sorted(n for n in names if re.search(rf"\b{n}!\(", new))
    have = re.search(r"use crate::\{[^}]*\}|use crate::\w+;", new) is not None and all(
        re.search(rf"use crate::(?:\{{[^}}]*\b{n}\b[^}}]*\}}|{n};)", new) for n in used
    )
    if import_ and not prefix and used and not have:
        new = add_import(new, used)
    if not dry and new != src:
        path.write_text(new, "utf8")
    return converted, refused


SELF_TEST = [
    (
        'TRACE(b"a %d\\n\\0".as_ptr() as *const i8, x);',
        True,
        'TRACE!("a %d\\n", x);',
    ),
    ('ERR(b"plain\\0".as_ptr() as PCHAR);', True, 'ERR!("plain");'),
    ('ERR((b"paren\\0".as_ptr() as PCHAR));', True, 'ERR!("paren");'),
    (
        'WARN(c"c-lit %s".as_ptr(), (*p).Name.as_ptr());',
        True,
        'WARN!("c-lit %s", (*p).Name.as_ptr());',
    ),
    # nested parens/strings/commas inside an argument
    (
        'TRACE(b"%s\\0".as_ptr() as PCHAR, f(a, ")"), g[1]);',
        True,
        'TRACE!("%s", f(a, ")"), g[1]);',
    ),
    # refused: not a literal / high byte / embedded NUL / no terminator
    ("TRACE(fmt, x);", True, "TRACE(fmt, x);"),
    ('TRACE(b"\\xff\\0".as_ptr() as PCHAR);', True, 'TRACE(b"\\xff\\0".as_ptr() as PCHAR);'),
    ('TRACE(b"a\\0b\\0".as_ptr() as PCHAR);', True, 'TRACE(b"a\\0b\\0".as_ptr() as PCHAR);'),
    ('TRACE(b"a".as_ptr() as PCHAR);', True, 'TRACE(b"a".as_ptr() as PCHAR);'),
    # never touched: macro calls, paths, methods, definitions, strings, comments
    (
        'TRACE!("x"); m::TRACE(y); o.TRACE(z); MYTRACE(q);',
        True,
        'TRACE!("x"); m::TRACE(y); o.TRACE(z); MYTRACE(q);',
    ),
    ("fn TRACE(fmt: *const i8, ...);", True, "fn TRACE(fmt: *const i8, ...);"),
    ('let s = "TRACE(b)"; // TRACE(c)', True, 'let s = "TRACE(b)"; // TRACE(c)'),
    # a macro exported by a dependency
    ("_Analysis_assume_(p != 0);", "ntkrnl_sys::", "ntkrnl_sys::_Analysis_assume_!(p != 0);"),
    # verbatim mode for discarding macros
    (
        'TRACE(b"%p\\n\\0".as_ptr() as LPCSTR, this);',
        False,
        'TRACE!(b"%p\\n\\0".as_ptr() as LPCSTR, this);',
    ),
]


def self_test() -> int:
    bad = 0
    names = {"TRACE", "ERR", "WARN", "_Analysis_assume_"}
    for src, mode, want in SELF_TEST:
        lit, prefix = (mode, "") if isinstance(mode, bool) else (False, mode)
        got, _, _ = rewrite(src, names, lit, prefix)
        if got != want:
            bad += 1
            print(f"FAIL: {src!r}\n  got  {got!r}\n  want {want!r}")
    decl = 'extern "C" {\n    fn TRACE(fmt: *const i8, ...);\n    fn Other(x: i32);\n}\n'
    if drop_extern_decls(decl, {"TRACE"}) != 'extern "C" {\n    fn Other(x: i32);\n}\n':
        bad += 1
        print("FAIL: drop_extern_decls")
    n = len(SELF_TEST) + 1
    print(f"self-test: {n - bad}/{n} passed")
    return 1 if bad else 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument("--names", default="")
    ap.add_argument("--fmt-literal", action="store_true")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument(
        "--no-import",
        action="store_true",
        help="the macros are already in textual scope (#[macro_use]); add no `use crate::`",
    )
    ap.add_argument(
        "--prefix",
        default="",
        help="path the macro is called by, e.g. `ntkrnl_sys::` for a #[macro_export] in a dep",
    )
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    names = {n for n in a.names.split(",") if n}
    if not names or not a.paths:
        ap.error("--names and at least one path are required")
    files = [f for p in a.paths for f in (sorted(source_files(p)) if p.is_dir() else [p])]
    total, refused_all = 0, []
    for f in files:
        conv, refused = process(f, names, a.fmt_literal, a.dry_run, not a.no_import, a.prefix)
        total += conv
        refused_all += [f"{f}:{ln} {n}" for ln, n in refused]
    for r in refused_all:
        print(f"REFUSED {r}")
    print(f"{total} call(s) converted, {len(refused_all)} refused")
    return 0


if __name__ == "__main__":
    sys.exit(main())
