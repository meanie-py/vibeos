#!/usr/bin/env python3
"""
scan_double_fetch.py — the same user address read twice (a TOCTOU double-fetch).

THE POINT. A capturing read of user memory — `read_back`, `probe_read_user`,
`safe_copy_from_user`, win32k's `copy_from_user`/`capture_from_user` — takes ONE
snapshot of the caller's buffer. If a function reads the SAME user address twice and
uses both results, the two snapshots can differ: on SMP another CPU running the
caller's other thread changes the value between the fetches, so a length validated on
the first read no longer bounds the copy sized by the second. That is the classic
double-fetch, and it is exactly the bug the "read a value that sizes or validates
something ONCE and use the copy" rule exists to prevent. The safe shape is to capture
once into a kernel local and use the copy everywhere.

WHAT IS FLAGGED. Within one function, two or more capturing reads whose SOURCE
expression (the user pointer, normalised past `as T` casts, `.cast()`, and
whitespace) is the same. The reader's user-source argument is:

    read_back(SRC)   probe_read_user(SRC)   probe_read_generic(SRC)
    safe_copy_from_user(dst, SRC, ..)   copy_from_user(dst, SRC, ..)
    capture_from_user(dst, SRC, ..)

A read in a loop is ONE site (one source position), so iterating an array with
`read_back(P.add(i))` on a single line is not a double-fetch. The kernel-mode arm of
an explicit `if mode != KernelMode { .. } else { .. }` is skipped (a kernel caller's
pointer is trusted and re-reading it is not a cross-ring race).

WHAT IT CANNOT SEE. Two fetches split ACROSS functions (read in the syscall, re-read
in a worker it calls); a double-fetch through a primitive not in the reader list; and
whether a doubly-read value actually feeds a size/validation decision (so some rows
are benign re-reads — that is why this gates against a BASELINE, not zero). Suppress a
reviewed site with `// nolint: double-fetch <reason>` on the reader line.

Usage:
    python3 tools/scan_double_fetch.py            # rows: path: Fn SRC
    python3 tools/scan_double_fetch.py --lines     # also print each read line
    python3 tools/scan_double_fetch.py --lint      # terse rows (baseline gate)
    python3 tools/scan_double_fetch.py <path ...>
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from scan_raw_user_access import _kernel_only_spans, parse_file
from source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
DEFAULT_ROOT = REPO / "sdk" / "lib" / "rust"
SKIP_DIR_PARTS = ("-sys", "/target/", "rtl-strings", "ntdll")

# (reader name, index of its user-source argument)
READERS = (
    ("read_back", 0),
    ("probe_read_user", 0),
    ("probe_read_generic", 0),
    ("safe_copy_from_user", 1),
    ("copy_from_user", 1),
    ("capture_from_user", 1),
)
NOLINT_RE = re.compile(r"nolint:\s*(double-fetch|double_fetch|scan_double_fetch|all)\b")


def _arg_at(text: str, lpar: int, index: int) -> str | None:
    """The top-level argument `index` of the call whose '(' is at `lpar`."""
    depth = 0
    args: list[str] = []
    cur = ""
    k = lpar
    while k < len(text):
        c = text[k]
        if c in "([{":
            depth += 1
            if depth == 1 and c == "(":
                k += 1
                continue
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                break
        if c == "," and depth == 1:
            args.append(cur)
            cur = ""
        else:
            cur += c
        k += 1
    args.append(cur)
    return args[index] if index < len(args) else None


def _strip_trailing_cast(e: str) -> str:
    """Drop a trailing ` as <type>` only at paren depth 0, so a cast INSIDE the
    expression (`(*(p as *const T)).field`) is never mistaken for the outer cast."""
    depth = 0
    last = -1
    for i in range(len(e)):
        c = e[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif depth == 0 and e[i : i + 4] == " as " and (i == 0 or e[i - 1] != "_"):
            last = i
    return e[:last].rstrip() if last >= 0 else e


def _norm(expr: str) -> str:
    """Normalise a source expression so casts and spacing do not hide a repeat."""
    e = expr.strip()
    prev = None
    while e != prev:
        prev = e
        e = _strip_trailing_cast(e).strip()  # trailing top-level `as <type>`
        e = re.sub(r"\.\s*cast(?:_mut|_const)?\s*\(\s*\)$", "", e).strip()  # .cast()
    if e.startswith("(") and e.endswith(")"):
        depth = 0
        wrap = True
        for i, c in enumerate(e):
            depth += (c == "(") - (c == ")")
            if depth == 0 and i != len(e) - 1:
                wrap = False
                break
        if wrap:
            e = e[1:-1].strip()
    return re.sub(r"\s+", "", e)


def _enclosing_braces(text: str, pos: int) -> list[int]:
    """Offsets of the '{' of each block still open at `pos`, outermost first."""
    stack: list[int] = []
    for i in range(pos):
        c = text[i]
        if c == "{":
            stack.append(i)
        elif c == "}" and stack:
            stack.pop()
    return stack


def _is_arm(text: str, brace: int) -> bool:
    """Is the block opening at `brace` an arm of a selection (match arm / else)?"""
    pre = text[max(0, brace - 200) : brace].rstrip()
    return pre.endswith("=>") or re.search(r"\belse(\s+if\b[^{}]*)?$", pre) is not None


def _match_close(text: str, open_brace: int) -> int:
    depth = 0
    for i in range(open_brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return len(text)


def _block_exits(text: str, start: int, close: int) -> bool:
    """A `return`/`break`/`continue` at the top level of [start, close) — so entering
    this block transfers control away before any later statement outside it runs."""
    depth = 0
    for m in re.finditer(r"(?<![\w.])(return|break|continue)\b", text[start:close]):
        seg = text[start : start + m.start()]
        depth = seg.count("{") - seg.count("}")
        if depth <= 0:
            return True
    return False


def _mutually_exclusive(text: str, a: int, b: int) -> bool:
    """True if reads at offsets a, b cannot both run on one call: they are in sibling
    arms of a selection, or the earlier read's branch transfers control away (break /
    return / continue) before the later read is reachable. False when one block
    encloses the other with no such exit (sequential on some path)."""
    if a > b:
        a, b = b, a
    pa, pb = _enclosing_braces(text, a), _enclosing_braces(text, b)
    k = 0
    while k < len(pa) and k < len(pb) and pa[k] == pb[k]:
        k += 1
    if k < len(pa) and k < len(pb) and (_is_arm(text, pa[k]) or _is_arm(text, pb[k])):
        return True  # divergent arms of one match/if-else
    # The earlier read sits in a branch below the common block that exits before `b`.
    if k < len(pa):
        close = _match_close(text, pa[k])
        if b > close and _block_exits(text, a, close):
            return True
    return False


def _double_fetches(fn) -> dict[str, list[int]]:
    """source -> lines, for each user source read twice on one control-flow path."""
    body = fn.lines[fn.body0 : fn.end + 1]
    text = "\n".join(body)
    # Kernel-mode arms by CHARACTER span, so a one-line `else { .. }` is excluded too.
    kernel = _kernel_only_spans(text)
    sites: dict[str, list[int]] = {}
    for name, idx in READERS:
        for m in re.finditer(rf"(?<![\w.]){name}\s*(?:::<[^>]*>)?\s*\(", text):
            off = m.start()
            if any(a <= off <= b for a, b in kernel):
                continue
            if NOLINT_RE.search(fn.raw_lines[fn.body0 + text.count("\n", 0, off)]):
                continue
            arg = _arg_at(text, m.end() - 1, idx)
            if arg is None:
                continue
            src = _norm(arg)
            if src:
                sites.setdefault(src, []).append(off)
    simple = re.compile(r"\w+").fullmatch

    def reassigned(a: int, b: int, src: str) -> bool:
        # A pointer advanced between the reads (`p = p.add(i)`) addresses different
        # memory each time — a streaming copy, not a re-read of one location.
        if not simple(src):
            return False
        return bool(re.search(rf"(?<![\w.]){re.escape(src)}\s*(?:=|\+=|-=)(?!=)", text[a:b]))

    out: dict[str, list[int]] = {}
    for src, offs in sites.items():
        offs = sorted(set(offs))
        if len(offs) < 2:
            continue
        # A real double-fetch needs one pair reachable in sequence (not sibling arms)
        # and reading the SAME address (the source not reassigned between them).
        if any(
            not _mutually_exclusive(text, offs[i], offs[j])
            and not reassigned(offs[i], offs[j], src)
            for i in range(len(offs))
            for j in range(i + 1, len(offs))
        ):
            out[src] = [fn.body0 + text.count("\n", 0, o) for o in offs]
    return out


def scan_tree(files: list[tuple[Path, str]]) -> dict[str, list[tuple[int, str]]]:
    rows: dict[str, list[tuple[int, str]]] = {}
    for path, rel in files:
        for fn in parse_file(path, rel):
            for src, lines in _double_fetches(fn).items():
                key = f"{rel}: {fn.name} {src}"
                rows[key] = [(ln, fn.raw_lines[ln].strip()) for ln in sorted(set(lines))]
    return rows


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--lint", action="store_true", help="terse rows for the baseline gate")
    ap.add_argument("--lines", action="store_true", help="print each read line")
    ap.add_argument("paths", nargs="*", type=Path)
    args = ap.parse_args(argv)

    files: list[Path] = []
    for p in args.paths or [DEFAULT_ROOT]:
        if p.is_dir():
            files.extend(sorted(source_files(p)))
        elif p.suffix == ".rs":
            files.append(p)
    picked: list[tuple[Path, str]] = []
    for f in files:
        try:
            rel = f.resolve().relative_to(REPO).as_posix()
        except ValueError:
            rel = f.as_posix()
        if any(part in rel for part in SKIP_DIR_PARTS) or "bindgen" in f.name:
            continue
        picked.append((f, rel))
    rows = scan_tree(picked)
    for row in sorted(rows):
        print(row)
        if args.lines and not args.lint:
            for line, text in rows[row]:
                print(f"    {line}: {text[:120]}")
    if not args.lint:
        print(f"\n{len(rows)} (function, source) pairs read twice", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
