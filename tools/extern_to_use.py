#!/usr/bin/env python3
"""Replace a crate's per-module `extern` redeclarations with `use` of ONE declaration.

    python3 tools/extern_to_use.py sdk/lib/rust/fastfat-rust/src FatRaiseStatus,FatIsFat32
    python3 tools/extern_to_use.py <src> NAME,NAME --to externs [--dry-run]

Translated crates declare the same function in every module that calls it, each with its own
guess at the signature; `clashing_extern_declarations` counts the disagreements and nothing
checks them against the truth. Two cases, one move:

  * the crate DEFINES the function (a `#[no_mangle] pub ... fn` in one module): every other
    module's declaration becomes `use crate::<that module>::<name>;`, so each call is checked
    against the real definition.
  * the function lives outside the crate: write one canonical declaration in a module (e.g.
    `externs.rs`, from the C or the -sys bindgen) and pass `--to externs`.

Then build: the type errors that appear are exactly the call sites that disagreed with the
truth. Run `rustconv crate-repair` for the mechanical ones and read the rest against the C
(fastfat, 2026-10-11: 311 redeclarations, 242 errors, a handful real). Only declarations
starting a line are removed; a one-line `} extern "C" { fn X(..); }` fragment is reported
by the build as a duplicate name and needs a hand edit.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

DEF = r"^\s*pub(?:\(crate\))?\s+(?:unsafe\s+)?(?:extern\s+\"[^\"]+\"\s+)?fn\s+{n}\b"
MARK_OWN = "// The crate's own functions: the definition is the declaration."
MARK_TO = "// One declaration per external symbol: {to}.rs."


def decl_re(name: str) -> re.Pattern[str]:
    """A bodiless `fn NAME(..) [-> T];` starting a line, with nested parens, and its newline."""
    return re.compile(
        rf"^[ \t]*(?:pub\s+)?fn\s+{name}\s*\((?:[^()]|\((?:[^()]|\([^()]*\))*\))*\)"
        rf"\s*(?:->\s*[^;{{]+?)?\s*;[ \t]*\n?",
        re.M,
    )


def find_defs(files: dict[str, str], names: list[str]) -> dict[str, str]:
    """name -> the module that defines it. Refuses a name defined zero or several times."""
    out = {}
    for n in names:
        hits = [m for m, t in files.items() if re.search(DEF.format(n=n), t, re.M)]
        if len(hits) != 1:
            raise SystemExit(f"{n}: defined in {hits or 'no module'}; pass --to <module>")
        out[n] = hits[0]
    return out


def insert_uses(text: str, uses: list[str], mark: str) -> str:
    """After the first top-level `use` statement, or after the leading comments/attributes."""
    m = re.search(r"^use [^;]*;\n", text, re.M)
    at = m.end() if m else re.match(r"(?:(?://[^\n]*|#!\[[^\n]*\]|[ \t]*)\n)*", text).end()
    return text[:at] + mark + "\n" + "\n".join(uses) + "\n" + text[at:]


def convert(files: dict[str, str], home: dict[str, str], mark: str) -> tuple[dict, list[str]]:
    """({module: new text} for changed modules, report lines)."""
    changed, report = {}, []
    for mod, text in files.items():
        new, uses = text, []
        for n, where in home.items():
            if where == mod:
                continue
            r, found = decl_re(n), False
            while m := r.search(new):
                line = new.count("\n", 0, m.start()) + 1
                report.append(f"{mod}.rs:{line}: {' '.join(m.group(0).split())}")
                new = new[: m.start()] + new[m.end() :]
                found = True
            if found:
                uses.append(f"use crate::{where}::{n};")
        if uses:
            changed[mod] = insert_uses(new, uses, mark)
    return changed, report


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("src", type=Path, help="the crate's src/ directory")
    ap.add_argument("names", help="comma-separated function names")
    ap.add_argument("--to", help="module holding the one declaration (external symbols)")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args(argv)

    files = {p.stem: p.read_text() for p in sorted(a.src.glob("*.rs"))}
    names = [n for n in a.names.split(",") if n]
    home = {n: a.to for n in names} if a.to else find_defs(files, names)
    mark = MARK_TO.format(to=a.to) if a.to else MARK_OWN
    changed, report = convert(files, home, mark)
    print("\n".join(report))
    print(f"{len(report)} declaration(s) removed in {len(changed)} module(s)", file=sys.stderr)
    if not a.dry_run:
        for mod, text in changed.items():
            (a.src / f"{mod}.rs").write_text(text)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
