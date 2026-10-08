#!/usr/bin/env python3
"""
scan_typed_byte_copy.py — a C BYTE count passed to a struct-typed Rust copy.

C's RtlCopyMemory/memcpy/RtlZeroMemory count BYTES. core::ptr::copy[_nonoverlapping] and
write_bytes count ELEMENTS of the pointer's type, so a translation that keeps the C count
multiplies it by the element size. The u16 case is scan_wide_byte_copy.py; this is every
other element type, including the one `as *const _` infers from the source variable:

    // C:   RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer, layoutEx, size);
    core::ptr::copy_nonoverlapping(layoutEx as *const _,       // _ = DRIVE_LAYOUT_INFORMATION_EX
                                   SystemBuffer as *mut _, size); // copies size * 192 bytes

Found 2026-10-07 in partmgr's FdoIoctlDiskGetDriveLayoutEx: every IOCTL_DISK_GET_DRIVE_LAYOUT_EX
wrote ~117 KB through a 624-byte system buffer. Smoke never sends that IOCTL; the first
program that did (diskpart, under tools/payloads/modrun) bugchecked 0x19.

The element type comes from an explicit `as *mut T` on either pointer, else -- for
`x as *mut _` -- from x's declared type in the enclosing text (`x: *mut T`, `x: PT`). A
call is flagged when that type is not byte-sized and the count is a byte quantity: named
like one (`size`, `...Size`, `...Length`, `...Bytes`, `cb...`) or multiplied by a
`size_of` (clippy's size_of_in_element_count), and not divided by a size_of.

FIX: copy bytes, as the C does:  `src as *const u8, dst as *mut u8, bytes`.

Usage:
    python3 tools/scan_typed_byte_copy.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_typed_byte_copy.py --lint
Suppress a verified false positive (a `...Length` that counts elements) on the call's line:
    core::ptr::copy_nonoverlapping(  // nolint: typed-byte-copy (reason)
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_wide_byte_copy import SKIP_FILE_RE, _call_args
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

NOLINT_RE = re.compile(r"//\s*nolint(?::\s*typed-byte-copy\b|\s*$|:?\s*\()")
CALL_RE = re.compile(
    r"\b(?:core::ptr::|std::ptr::|ptr::)?(copy_nonoverlapping|copy|write_bytes)\s*(?:::<\s*([^>]+)>\s*)?\("
)
CAST_RE = re.compile(r"\bas\s+\*(?:mut|const)\s+((?:[\w:]+::)?\w+|_)")
BYTE_TYPES = {
    "u8", "i8", "c_void", "c_char", "c_uchar", "c_schar", "UCHAR", "CHAR", "BYTE", "BOOLEAN",
    "INT8", "UINT8", "CCHAR", "__u8",
}  # fmt: skip
WIDE_TYPES = {"u16", "WCHAR", "wchar_t", "c_ushort"}  # scan_wide_byte_copy's
BYTE_PTR_ALIASES = {"PVOID", "LPVOID", "PCVOID", "LPCVOID", "PUCHAR", "PBYTE", "LPBYTE", "PCHAR",
                    "PSTR", "LPSTR", "PCSTR", "LPCSTR", "PCCH", "PCH", "PBOOLEAN"}  # fmt: skip
BYTE_COUNT_RE = re.compile(
    r"\b(?:size|\w*Size|\w*_size|\w*Length|\w*Bytes\w*|\w*ByteCount|cb[A-Z_]\w*|bytes)\b"
)
TIMES_SIZEOF_RE = re.compile(r"\*\s*(?:\(\s*)?(?:core::|std::)?(?:mem::)?size_of\b|size_of\b[^/]*\*")
DIVIDED_RE = re.compile(r"/\s*\(?\s*(?:core::|std::)?(?:mem::)?size_of")


def _leaf(t: str) -> str:
    return t.split("::")[-1]


def _declared(text: str, before: int, name: str) -> str | None:
    """The pointee of `name`'s nearest preceding declaration (`name: *mut T` / `name: PT`)."""
    window = text[max(0, before - 20000) : before]
    best = None
    for m in re.finditer(
        rf"\b{re.escape(name)}\s*:\s*(?:\*(?:mut|const)\s+((?:[\w:]+::)?\w+)|((?:[\w:]+::)?\w+))", window
    ):
        best = m
    if best is None:
        return None
    if best.group(1):
        return _leaf(best.group(1))
    alias = _leaf(best.group(2))
    if alias in BYTE_PTR_ALIASES:
        return "u8"
    if alias in ("PWSTR", "LPWSTR", "PCWSTR", "LPCWSTR", "PWCHAR", "PWCH", "PCWCH"):
        return "u16"
    if re.fullmatch(r"L?P[A-Z0-9_]+", alias):
        return alias[2:] if alias.startswith("LP") else alias[1:]
    return None  # a non-pointer declaration: not something this scan can type


def element_type(
    text: str, at: int, ptrs: list[str], turbofish: str | None
) -> tuple[str | None, bool]:
    """(element type, whether a cast or turbofish chose it). An uncast pointer's type is only
    its declaration, and the translator kept the C's count there far less often."""
    if turbofish:
        return _leaf(turbofish.strip()), True
    inferred = None
    uncast = None
    for p in ptrs:
        casts = list(CAST_RE.finditer(p))
        if not casts:
            bare = re.fullmatch(r"\s*([A-Za-z_]\w*)\s*", p)  # an uncast variable: its declaration
            if bare and uncast is None:
                uncast = _declared(text, at, bare.group(1))
            continue
        t = casts[-1].group(1)  # the outermost cast: `x as *const T as *const u8` is bytes
        if t != "_":
            return _leaf(t), True
        base = re.match(r"\s*\(?\s*([A-Za-z_]\w*)\s*\)?\s+as\b", p)
        if base and len(casts) == 1 and inferred is None:
            inferred = _declared(text, at, base.group(1))
    if inferred is not None:
        return inferred, True
    return uncast, False


def scan(paths: list[Path]) -> list[tuple[Path, int, str, str]]:
    findings: list[tuple[Path, int, str, str]] = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            lines = text.splitlines()
            for m in CALL_RE.finditer(text):
                args = _call_args(text, m.end())
                if args is None or len(args) != 3:
                    continue
                ptrs, count = ([args[0]], args[2]) if m.group(1) == "write_bytes" else (args[:2], args[2])
                elem, cast = element_type(text, m.start(), ptrs, m.group(2))
                if elem is None or elem in BYTE_TYPES or elem in WIDE_TYPES:
                    continue
                if m.group(1) == "write_bytes" and count.strip() in ("1", "1usize"):
                    continue
                byteish = TIMES_SIZEOF_RE.search(count) or (cast and BYTE_COUNT_RE.search(count))
                if not byteish or DIVIDED_RE.search(count):
                    continue
                lineno = text.count("\n", 0, m.start()) + 1
                if NOLINT_RE.search(lines[lineno - 1]):
                    continue
                findings.append((f, lineno, elem, " ".join(count.split())))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, elem, count in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: typed-byte-copy {elem} {count}")
        else:
            print(f"{rel}:{lineno}: byte count `{count}` passed to a {elem}-typed copy")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): copy as u8 (`as *const u8` / `as *mut u8`).")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
