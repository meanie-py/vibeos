#!/usr/bin/env python3
"""
scan_let_shadow.py — a C assignment translated as a shadowing `let _x`, so it is lost.

THE CLASS. The translator sometimes turns a C assignment to a variable that is already
in scope (`DiskCount++;`, `left = &tempLeft;`, `num_header = pOut;`) into a fresh
binding inside a nested block, and names it with a leading underscore because the new
binding is never read. The underscore silences rustc's unused warning, and the outer
variable silently keeps its old value. This shipped five times at once (2026-10-04):
freeldr's EnumerateHarddisks always reported 0 BIOS disks, oleaut32's VarSub ignored a
fetched dispatch value, its format tokenizer wrote every section into the first one,
and consrv's console writer kept stale start columns.

WHAT IS FLAGGED: `let [mut] _name` while a parameter or `let` named `name` is in scope
in the same function. If the C really does declare a separate `_name` (gdi32's
HANDLE_METADC `_Ret`) or never reads the value, mark the line
`// nolint: let-shadow (why)`.

Usage:
    python3 tools/scan_let_shadow.py [--lint] [paths...]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
NOLINT = "nolint: let-shadow"
FN_RE = re.compile(r"\bfn\s+\w+\s*(?:<[^>{]*>)?\s*\(")
LET_RE = re.compile(r"\blet\s+(?:mut\s+)?([A-Za-z_]\w*)")
PARAM_RE = re.compile(r"(?:^|[(,])\s*(?:mut\s+)?([A-Za-z_]\w*)\s*:(?!:)")


_LEX = re.compile(r"//[^\n]*|/\*.*?\*/|b?\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^\\'\n])'", re.S)


def _blank(text: str) -> str:
    """Comments and string/char literals to spaces, keeping offsets and newlines."""
    return _LEX.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def _params(text: str, open_idx: int) -> tuple[list[str], int]:
    """Parameter names of the fn whose `(` ends at open_idx, and the index after `)`."""
    depth = 0
    for i in range(open_idx, len(text)):
        if text[i] in "([<":
            depth += 1
        elif text[i] in ")]>":
            if depth == 0:
                return PARAM_RE.findall("(" + text[open_idx:i]), i + 1
            depth -= 1
    return [], len(text)


def scan_text(text: str) -> list[tuple[int, str]]:
    raw_lines = text.splitlines()
    code = _blank(text)
    # Each scope: [names, is_fn_body]. A pending fn's params open with its body brace.
    scopes: list[tuple[set[str], bool]] = [(set(), False)]
    pending: list[str] | None = None
    out = []
    events = []
    for m in FN_RE.finditer(code):
        events.append((m.end() - 1, "fn", m))
    for m in LET_RE.finditer(code):
        events.append((m.start(), "let", m))
    for m in re.finditer(r"[{};]", code):
        events.append((m.start(), m.group(0), m))
    events.sort(key=lambda e: e[0])
    skip_until = -1
    for pos, kind, m in events:
        if pos < skip_until:
            continue
        if kind == "fn":
            pending, skip_until = _params(code, pos + 1)
        elif kind == "{":
            scopes.append((set(pending or ()), pending is not None))
            pending = None
        elif kind == ";":
            pending = None  # a prototype (`fn f(..);`) has no body
        elif kind == "}":
            if len(scopes) > 1:
                scopes.pop()
        else:
            name = m.group(1)
            if (
                name.startswith("_")
                and len(name) > 1
                and not name.startswith("__")
                and not scopes[-1][1]
            ):
                outer = name[1:]
                # Strictly ENCLOSING scopes only: a `_x` declared beside `x` in the same
                # block is the C's own second variable (uniata's `ULONG c, _c;`), while
                # every lost assignment found so far sat in a nested block.
                for names, is_fn in reversed(scopes[:-1]):
                    if outer in names:
                        line = code.count("\n", 0, pos) + 1
                        if NOLINT not in raw_lines[line - 1]:
                            out.append((line, name))
                        break
                    if is_fn:
                        break
            scopes[-1][0].add(name)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("paths", nargs="*", type=Path)
    ap.add_argument("--lint", action="store_true", help="terse output; exit 1 on findings")
    args = ap.parse_args()

    files = []
    for p in args.paths or [RUST_ROOT]:
        files += [p] if p.is_file() else sorted(source_files(p))
    n = 0
    for f in files:
        if f.suffix != ".rs":
            continue
        for line, name in scan_text(f.read_text(errors="replace")):
            rel = f.resolve().relative_to(REPO) if f.resolve().is_relative_to(REPO) else f
            print(
                f"{rel}:{line}: `let {name}` shadows `{name[1:]}` in scope: "
                "a C assignment that is lost?"
            )
            n += 1
    if not args.lint:
        print(f"\n{n} finding(s)")
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main())
