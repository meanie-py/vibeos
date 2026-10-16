#!/usr/bin/env python3
"""scan_varargs_spec_arity.py — a variadic extern declared with a different FIXED arity
than the `.spec` export it binds to.

THE INCIDENT
    C's conforming `swprintf(buf, count, fmt, ...)` is a static inline in <stdio.h> over
    `_vsnwprintf`. msvcrt's EXPORTED `swprintf` is the legacy one:

        msvcrt.spec:  @ varargs swprintf(ptr wstr) _swprintf   # Non-conforming swprintf

    wscript-rust declared the C's shape and bound it to the export,

        pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;

    so msvcrt read the count (1024) as the format pointer and cscript crashed on every
    script error. msiexec, ole32 and shell32's recycle bin carried the same declaration,
    copied between preludes by `port resolve`'s tree consensus. scan_extern_arity cannot
    see it (it compares Rust with Rust, and skips variadics); rustc cannot either.

WHAT COUNTS AS A FINDING
    A Rust `fn NAME(a, b, c, ...)` (or `#[link_name = "NAME"]`) whose fixed-parameter count
    is not that of any `@ varargs NAME(...)` in the tree's .spec files. The fix is the
    function the C really calls: `#[link_name = "_snwprintf"]`. Baseline is EMPTY.

    Findings go to stdout for the baseline comparison; the tally goes to stderr.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
SPEC_ROOTS = ("dll", "ntoskrnl", "hal", "win32ss", "drivers", "base", "subsystems", "sdk/lib")
VARARGS = re.compile(r"^\s*@\s+varargs\s+(?:-[\w=.,-]+\s+)*(\w+)\(([^)]*)\)", re.M)
DECL = re.compile(
    r"(?:#\[link_name\s*=\s*\"(\w+)\"\]\s*)?(?:pub\s+)?fn\s+(\w+)\s*\(([^;{]*?)\)\s*(?:->[^;{]*)?;",
    re.S,
)
COMMENT = re.compile(r"//[^\n]*")


def spec_arities(texts: list[str]) -> dict[str, set[int]]:
    """{export: fixed-parameter counts} over every `@ varargs` line in the given .spec texts."""
    out: dict[str, set[int]] = defaultdict(set)
    for text in texts:
        for m in VARARGS.finditer(text):
            out[m.group(1)].add(len(m.group(2).split()))
    return dict(out)


def scan_text(text: str, spec: dict[str, set[int]]) -> list[tuple[int, str, int, list[int]]]:
    """(line, export, fixed params declared, the spec's) for each mismatched variadic decl."""
    code = COMMENT.sub("", text)
    out = []
    for m in DECL.finditer(code):
        params = m.group(3)
        if "..." not in params:
            continue
        name = m.group(1) or m.group(2)
        if name not in spec:
            continue
        flat = re.sub(r"\([^()]*\)", "", re.sub(r"<[^<>]*>", "", params))
        fixed = [a for a in flat.split(",") if a.strip() and a.strip() != "..."]
        if len(fixed) not in spec[name]:
            out.append((code.count("\n", 0, m.start(2)) + 1, name, len(fixed), sorted(spec[name])))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", action="append", default=None, help="defaults to sdk/lib/rust")
    args = ap.parse_args()
    spec = spec_arities(
        [
            p.read_text(errors="replace")
            for root in SPEC_ROOTS
            for p in sorted((REPO / root).rglob("*.spec"))
            if "target" not in p.parts
        ]
    )
    findings = files = 0
    for root in [REPO / r for r in (args.root or ["sdk/lib/rust"])]:
        for path in sorted(source_files(root)):
            if path.suffix != ".rs":
                continue
            files += 1
            for lineno, name, fixed, want in scan_text(path.read_text(errors="replace"), spec):
                print(
                    f"{path.relative_to(REPO)}:{lineno}: VARARGS_SPEC_ARITY {name} declared "
                    f"with {fixed} fixed parameter(s), the export takes {want}"
                )
                findings += 1
    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
