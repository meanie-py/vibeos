#!/usr/bin/env python3
"""scan_extern_static_width.py -- a Rust `extern static` read WIDER than its C definition.

A cohab shim exports a header #define to Rust as real data, e.g.

    const ULONG __c_MAX_LFN_CHARACTERS __asm__(FF_SYM("MAX_LFN_CHARACTERS")) = 260;

and the Rust side declares the import itself: `static MAX_LFN_CHARACTERS: usize;`. Nothing
checks that the two widths agree. When the Rust type is wider, the load takes its high
bytes from whatever the linker placed next, which is zero on one arch and not another.

It happened: fastfat's FatSetRenameInfo read MAX_LFN_CHARACTERS as usize against a 4-byte
ULONG. On arm64 the neighbour was nonzero, every UnicodeBuffer.offset(MAX_LFN_CHARACTERS * k)
pointed at a non-canonical address, and every rename bugchecked 0x23; amd64 read a zero and
passed. A narrower Rust read is harmless on these little-endian targets and is not flagged.

Pairs a Rust `static NAME: T;` (a declaration, no initializer) with C definitions that name
the symbol explicitly through `__asm__("NAME")` (the cohab shims' idiom). Widths come from
a small type table; a type it does not know is skipped, never guessed. usize/isize/pointers
are 8 bytes on amd64/arm64, which is where an over-read against a 4-byte C type bites.

Output: one line per site. Exit 1 on findings.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
C_ROOTS = ["drivers", "dll", "ntoskrnl", "base", "win32ss", "subsystems", "hal", "boot", "sdk/lib"]

# Width in bytes on the 64-bit targets (the ones where usize and ULONG differ).
C_WIDTH = {
    "UCHAR": 1,
    "CHAR": 1,
    "BOOLEAN": 1,
    "CCHAR": 1,
    "BYTE": 1,
    "USHORT": 2,
    "SHORT": 2,
    "CSHORT": 2,
    "WCHAR": 2,
    "WORD": 2,
    "ULONG": 4,
    "LONG": 4,
    "INT": 4,
    "UINT": 4,
    "DWORD": 4,
    "NTSTATUS": 4,
    "int": 4,
    "unsigned": 4,
    "unsigned int": 4,
    "LONGLONG": 8,
    "ULONGLONG": 8,
    "LONG64": 8,
    "ULONG64": 8,
    "SIZE_T": 8,
    "ULONG_PTR": 8,
    "LONG_PTR": 8,
    "PVOID": 8,
}
RUST_WIDTH = {
    "u8": 1,
    "i8": 1,
    "c_char": 1,
    "c_uchar": 1,
    "c_schar": 1,
    "BOOLEAN": 1,
    "UCHAR": 1,
    "u16": 2,
    "i16": 2,
    "c_short": 2,
    "c_ushort": 2,
    "USHORT": 2,
    "WCHAR": 2,
    "CSHORT": 2,
    "u32": 4,
    "i32": 4,
    "c_int": 4,
    "c_uint": 4,
    "c_long": 4,
    "c_ulong": 4,
    "ULONG": 4,
    "LONG": 4,
    "NTSTATUS": 4,
    "DWORD": 4,
    "u64": 8,
    "i64": 8,
    "LONGLONG": 8,
    "ULONGLONG": 8,
    "usize": 8,
    "isize": 8,
    "SIZE_T": 8,
    "ULONG_PTR": 8,
    "PVOID": 8,
}

C_DEF_RE = re.compile(
    r"^\s*(?:static\s+)?(?:const\s+)?(?P<ctype>[A-Za-z_][A-Za-z0-9_]*(?:\s+int)?)\s*(?P<ptr>\*?)\s*"
    r"(?:const\s+)?[A-Za-z_]\w*\s+__asm__\s*\(\s*(?:\w+\s*\(\s*)?\"_?(?P<name>[A-Za-z_]\w*)\""
)
RUST_DECL_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?static\s+(?:mut\s+)?(?P<name>[A-Za-z_]\w*)\s*:\s*(?P<ty>[^=;]+);"
)


def c_definitions(text: str) -> dict[str, int]:
    """symbol -> width for every `TYPE ident __asm__("symbol")` definition in a C file."""
    out = {}
    for line in text.splitlines():
        m = C_DEF_RE.match(line)
        if not m:
            continue
        width = 8 if m.group("ptr") else C_WIDTH.get(m.group("ctype"))
        if width is not None:
            out[m.group("name")] = width
    return out


def _rust_width(ty: str) -> int | None:
    ty = ty.strip()
    if ty.startswith(("*mut ", "*const ", "&")):
        return 8
    return RUST_WIDTH.get(ty.rsplit("::", 1)[-1])


def scan_rust(text: str, c_widths: dict[str, int]) -> list[tuple[int, str, int, int]]:
    """(line, name, rust_width, c_width) for each declaration read wider than its C."""
    hits = []
    for n, line in enumerate(text.splitlines(), 1):
        m = RUST_DECL_RE.match(line.split("//", 1)[0])
        if not m or m.group("name") not in c_widths:
            continue
        rw = _rust_width(m.group("ty"))
        cw = c_widths[m.group("name")]
        if rw is not None and rw > cw:
            hits.append((n, m.group("name"), rw, cw))
    return hits


def main() -> int:
    c_widths: dict[str, int] = {}
    for root in C_ROOTS:
        base = REPO / root
        if not base.is_dir():
            continue
        for path in source_files(base, "*.c"):
            text = path.read_text(errors="replace")
            if "__asm__" in text:
                c_widths.update(c_definitions(text))
    findings = []
    for path in sorted(source_files(RUST_ROOT)):
        rel = path.relative_to(REPO).as_posix()
        if "/target/" in rel:
            continue
        for n, name, rw, cw in scan_rust(path.read_text(errors="replace"), c_widths):
            findings.append(
                f"{rel}:{n}: [EXTERN_STATIC_OVERREAD] {name} is read as {rw} bytes but its C "
                f"definition is {cw} -- declare the C width, or use the crate constant"
            )
    for line in findings:
        print(line)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
