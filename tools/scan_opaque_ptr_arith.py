#!/usr/bin/env python3
"""
scan_opaque_ptr_arith.py — pointer arithmetic on a field bindgen typed as OPAQUE.

bindgen emits a type whose body it cannot see as a 1-byte stub:

    pub struct _CONSOLE_IO_HANDLE { pub _address: u8 }

A struct field declared `*mut _CONSOLE_IO_HANDLE` (or through a `P...` alias of
one) therefore steps ONE BYTE per `.add(i)`, whatever element type the code
casts to afterwards:

    (*ProcessData).HandleTable.add(i) as PCONSOLE_IO_HANDLE    // stride 1, not 16/24

Found 2026-09-23 in consrv: the handle table was allocated with the real element
size but indexed with stride 1, so handle entries overlapped and the first
console with handles crashed csrss on close (BUGCHECK 0xF4). It compiles on every
arch and fails only once an index is non-zero.

FIX: cast to the real element type BEFORE the arithmetic:
    ((*ProcessData).HandleTable as PCONSOLE_IO_HANDLE).add(i)

Fields and opaque types come from each `*-sys` crate's bindgen_*.rs. Any use of
`.<field>.add(` / `.offset(` / `.sub(` / `.wrapping_add(` is flagged, because the
arithmetic always happens at the opaque stride.

Usage:
    python3 tools/scan_opaque_ptr_arith.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_opaque_ptr_arith.py --lint

Suppress a verified false positive on its line:
    ... // nolint: opaque-ptr-arith (reason)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_FILE_RE = re.compile(
    r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$|/island_[a-z0-9_]+\.rs$)"
)
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*opaque-ptr-arith\b|\s*$|:?\s*\()")
OPAQUE_RE = re.compile(r"pub struct (\w+) \{\s*pub _address: u8,\s*\}")
ALIAS_RE = re.compile(r"pub type (\w+) = \*(?:mut|const) (\w+);")
PTR_FIELD_RE = re.compile(r"pub (\w+): \*(?:mut|const) (\w+),")
ALIAS_FIELD_RE = re.compile(r"pub (\w+): (\w+),")
USE_RE = re.compile(r"\.(\w+)\.(add|offset|sub|wrapping_add|wrapping_sub)\(")


def opaque_fields(bindgen_texts: list[str]) -> set[str]:
    fields: set[str] = set()
    for text in bindgen_texts:
        opaque = set(OPAQUE_RE.findall(text))
        if not opaque:
            continue
        aliases = {a for a, t in ALIAS_RE.findall(text) if t in opaque}
        fields |= {f for f, t in PTR_FIELD_RE.findall(text) if t in opaque}
        fields |= {f for f, t in ALIAS_FIELD_RE.findall(text) if t in aliases}
    fields.discard("_address")
    return fields


def scan(paths: list[Path], bindgen_root: Path) -> list[tuple[Path, int, str]]:
    texts = [p.read_text(errors="replace") for p in bindgen_root.glob("*/src/bindgen_*.rs")]
    texts += [
        p.read_text(errors="replace") for p in bindgen_root.glob("**/opaque_fixture_bindgen.rs")
    ]
    fields = opaque_fields(texts)
    findings: list[tuple[Path, int, str]] = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if SKIP_FILE_RE.search(str(f)) or f.name == "opaque_fixture_bindgen.rs":
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            lines = text.splitlines()
            for m in USE_RE.finditer(text):
                if m.group(1) not in fields:
                    continue
                lineno = text.count("\n", 0, m.start()) + 1
                if NOLINT_RE.search(lines[lineno - 1]):
                    continue
                findings.append((f, lineno, m.group(1)))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    # A fixture directory carries its own bindgen stand-in; the tree uses sdk/lib/rust.
    bindgen_root = paths[0] if paths[0] != RUST_ROOT and paths[0].is_dir() else RUST_ROOT
    findings = scan(paths, bindgen_root)
    for f, lineno, field in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: opaque-ptr-arith .{field}")
        else:
            print(f"{rel}:{lineno}: arithmetic on opaque-typed `.{field}` (1-byte stride)")
    if findings and not lint:
        print(
            f"\n{len(findings)} finding(s): cast to the real element type before .add()/.offset()."
        )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
