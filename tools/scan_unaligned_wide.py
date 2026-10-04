#!/usr/bin/env python3
"""Flag UTF-16 literals spelled as byte strings and cast straight to a wide pointer.

    b"L\\0i\\0b\\0r\\0a\\0r\\0y\\0P\\0a\\0t\\0h\\0\\0\\0".as_ptr() as LPCWSTR

compiles and usually works. A byte string has ALIGNMENT 1, so the linker is free to put it at
an odd address; user-mode x86 reads it anyway, but the kernel does not: ProbeForRead(p, len,
sizeof(WCHAR)) raises STATUS_DATATYPE_MISALIGNMENT. When msvcrtex became Rust (2026-10-04)
the layout of every module's .rdata shifted, ws2_32's L"LibraryPath" landed on 0x7be41b61, and
nettest's first Winsock call bugchecked the i386 guest (0x1E). 146 such casts were in the tree,
each one an address-parity coin flip that any unrelated link change can lose.

The fix keeps the bytes exactly as written (tools/lint_wide_strings.py still checks their
encoding) and only fixes the placement: `win32macros_rust::wide_bytes!(b"...")` puts them in a
2-byte-aligned static and yields `*const u16`.

USAGE
    python3 tools/scan_unaligned_wide.py --lint   # findings on stdout, exit 1 if any
    python3 tools/scan_unaligned_wide.py --fix    # rewrite the sites, add the dependency
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RUST = REPO / "sdk" / "lib" / "rust"
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

# A byte-string literal containing a NUL, `.as_ptr()`, then `as` a wide pointer type. The cast
# target is what makes it a UTF-16 use; the NUL keeps ordinary C strings out.
WIDE_TYPES = (
    r"LPCWSTR|LPWSTR|PCWSTR|PWSTR|PWCHAR|PWCH|PCWCH|LPCOLESTR|LPOLESTR|BSTR"
    r"|u16|wchar_t|WCHAR|c_ushort|USHORT"
)
SITE = re.compile(
    r'(?P<lit>b"(?:[^"\\]|\\.)*\\0(?:[^"\\]|\\.)*")'
    r"\s*\.as_ptr\(\)"
    r"(?P<cast>\s*as\s*(?:\*\s*(?:const|mut)\s*)?(?:" + WIDE_TYPES + r")\b)"
)
MACRO = "win32macros_rust::wide_bytes!"


def rust_files(roots: list[Path] | None = None) -> list[Path]:
    out: list[Path] = []
    for root in roots or [RUST]:
        out.extend([root] if root.is_file() else source_files(root))
    return sorted(out)


def crate_of(path: Path) -> Path | None:
    for parent in path.parents:
        if (parent / "Cargo.toml").is_file():
            return parent
        if parent == RUST:
            return None
    return None


def findings(roots: list[Path] | None = None) -> list[tuple[Path, int, str]]:
    out = []
    for f in rust_files(roots):
        text = f.read_text(encoding="utf-8", errors="replace")
        if "as_ptr()" not in text:
            continue
        for m in SITE.finditer(text):
            line = text.count("\n", 0, m.start()) + 1
            out.append((f, line, m.group(0)))
    return out


def add_dependency(crate: Path) -> bool:
    toml = crate / "Cargo.toml"
    text = toml.read_text(encoding="utf-8")
    if re.search(r"^win32macros-rust\s*=", text, re.M) or crate.name == "win32macros-rust":
        return False
    rel = Path("..") / "win32macros-rust"
    line = f'win32macros-rust = {{ path = "{rel.as_posix()}" }}\n'
    m = re.search(r"^\[dependencies\]\n", text, re.M)
    if m:
        text = text[: m.end()] + line + text[m.end() :]
    else:
        text = text.rstrip("\n") + "\n\n[dependencies]\n" + line
    toml.write_text(text, encoding="utf-8")
    return True


def fix() -> int:
    changed: dict[Path, int] = {}
    for f in rust_files():
        text = f.read_text(encoding="utf-8", errors="replace")
        new, n = SITE.subn(lambda m: f"{MACRO}({m.group('lit')}){m.group('cast')}", text)
        if n:
            f.write_text(new, encoding="utf-8")
            changed[f] = n
    crates = {crate_of(f) for f in changed} - {None}
    added = [c.name for c in sorted(crates) if c is not None and add_dependency(c)]
    print(
        f"rewrote {sum(changed.values())} sites in {len(changed)} files; "
        f"added win32macros-rust to {len(added)} crates: {', '.join(added)}"
    )
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--lint", action="store_true")
    g.add_argument("--fix", action="store_true")
    ap.add_argument("paths", nargs="*", type=Path, help="files/dirs (default: sdk/lib/rust)")
    a = ap.parse_args()
    if a.fix:
        return fix()
    rows = findings(a.paths or None)
    for f, line, src in rows:
        where = f.relative_to(REPO) if f.is_relative_to(REPO) else f
        print(
            f"{where}:{line}: unaligned UTF-16 byte literal cast to a wide "
            f"pointer; use win32macros_rust::wide_bytes!(...): {src[:90]}"
        )
    return 1 if rows else 0


if __name__ == "__main__":
    sys.exit(main())
