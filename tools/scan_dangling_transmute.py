#!/usr/bin/env python3
"""
scan_dangling_transmute.py — catch dangling pointers to transmute temporaries.

A rustconv translation artifact: C's `WCHAR foo[] = L"..."` has real array
storage, so `&foo[0]` is a valid pointer for the array's whole scope. The
mechanical translation becomes

    let p = core::mem::transmute::<[u8; N], [u16; M]>(bytes).as_ptr();
    ...use p...                         // p is ALREADY DANGLING here

or, as a struct field that outlives the initializer:

    Name: core::mem::transmute::<[u8; N], [u16; M]>(bytes).as_ptr() as PCWSTR,

The transmute produces a *storageless temporary* `[u16; M]`; `.as_ptr()` borrows
it; the temporary is dropped at the end of the *statement*; the stored pointer
then dangles. Confirmed real (2026-07-12 warning survey):

  - disk-rust/src/pnp.rs — the dangling pointer was the printf FORMAT STRING
    handed to RtlStringCchPrintfW on the disk symbolic-link path.
  - lsasrv-rust/.../authpackage.rs — dangling `Name` in an RTL_QUERY_REGISTRY_TABLE
    read later by RtlQueryRegistryValues.

The authoritative oracle is rustc's own `dangling_pointers_from_temporaries`
lint — surface warnings and it fires precisely. This scanner is the cheap
grep-time gate (no nightly build needed) for the confirmed escaping shape.

**Sound and NOT flagged:** the same transmute as a *direct call argument*
(`func(transmute(x).as_ptr())`) — there the temporary lives to the end of the
call statement, so the pointer is valid during the call. Only the *escaping*
forms (let-bound, or stored in a struct field) are bugs.

FIX: bind the transmuted array to a local that outlives the use, then take
`.as_ptr()` of the *binding*:

    let foo_wide: [u16; M] = core::mem::transmute::<[u8; N], [u16; M]>(bytes);
    let p = foo_wide.as_ptr();          // foo_wide has scope storage

Usage:
    python3 tools/scan_dangling_transmute.py [paths...]  # default: sweep sdk/lib/rust
    python3 tools/scan_dangling_transmute.py --lint      # terse (CLI baseline gate)

Suppress a verified false positive on its line:
    let p = transmute(x).as_ptr(); // nolint: dangling-transmute (callee copies immediately)

Exit 1 if any finding (the CLI gate diffs against tools/lint-baseline/).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

SKIP_FILE_RE = re.compile(r"(/target/|/\.responses/|bindgen_[a-z0-9_]+\.rs$)")
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*dangling-transmute\b|\s*$|:?\s*\()")

# A transmute expression whose result is immediately `.as_ptr()`d / `.as_mut_ptr()`d.
# `.*?` (not `[^;]*?`): array type args carry a semicolon (`[u8; 60]`). Line-based
# scanning already bounds the match to a single line.
TRANSMUTE_AS_PTR = r"(?:::)?core::mem::transmute::<.*?>\(.*?\)\.as_(?:mut_)?ptr\(\)"

# ESCAPING forms — the pointer to the temporary is STORED and outlives the stmt:
#   A. let-bound:    `let <ident> ... = <transmute>.as_ptr()`
#   B. struct field: `<ident>: <transmute>.as_ptr()`
LET_BOUND_RE = re.compile(r"\blet\b[^=]*=\s*" + TRANSMUTE_AS_PTR)
STRUCT_FIELD_RE = re.compile(r"^\s*[A-Za-z_]\w*\s*:\s*" + TRANSMUTE_AS_PTR)


TRANSMUTE_RE = re.compile(TRANSMUTE_AS_PTR)


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    findings: list[tuple[int, str, str]] = []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return findings
    for i, line in enumerate(text.splitlines(), 1):
        if NOLINT_RE.search(line):
            continue
        m = TRANSMUTE_RE.search(line)
        if not m:
            continue
        # A REFERENCE transmute (`transmute::<&[u8; N], &[u16; M]>(b"...")`) yields a
        # reference to the SOURCE storage — for a `b"..."` static literal that is
        # 'static, so `.as_ptr()` is sound. Only the by-VALUE array transmute
        # (`<[u8; N], [u16; M]>`) makes a droppable temporary that dangles.
        if "&[" in m.group(0):
            continue
        if LET_BOUND_RE.search(line):
            findings.append((i, "let-bound", line.strip()))
        elif STRUCT_FIELD_RE.search(line):
            findings.append((i, "struct-field", line.strip()))
    return findings


def iter_rust_files(paths: list[str]):
    roots = [Path(p) for p in paths] if paths else [RUST_ROOT]
    for root in roots:
        if root.is_file() and root.suffix == ".rs":
            yield root
        elif root.is_dir():
            for f in source_files(root):
                if not SKIP_FILE_RE.search(str(f)):
                    yield f


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--lint"]
    lint = "--lint" in sys.argv[1:]
    total = 0
    for f in iter_rust_files(args):
        for lineno, kind, snippet in scan_file(f):
            total += 1
            rel = f.relative_to(REPO) if str(f).startswith(str(REPO)) else f
            if lint:
                print(f"{rel}:{lineno}: dangling-transmute [{kind}]")
            else:
                print(f"{rel}:{lineno}: dangling pointer to transmute temporary ({kind})")
                print(f"    {snippet}")
    if not lint and total:
        print(
            f"\n{total} finding(s). Fix: bind the transmuted array to a scoped local, "
            f"then take .as_ptr() of the binding."
        )
    return 1 if total else 0


if __name__ == "__main__":
    sys.exit(main())
