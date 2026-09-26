#!/usr/bin/env python3
"""
scan_wide_byte_copy.py — a C BYTE count passed to a WCHAR-typed Rust copy.

C's RtlCopyMemory/memcpy count BYTES. core::ptr::copy[_nonoverlapping] counts
ELEMENTS of the pointer's type. A translation that keeps the C count but types
the pointers as u16 moves twice the bytes:

    // C:   RtlCopyMemory(Dst, LongFileName.Buffer, BytesConverted);
    core::ptr::copy_nonoverlapping(LongFileName.Buffer, dst as *mut u16,
                                   BytesConverted as usize);   // 2x over-read/over-write

Found 2026-09-23 in fastfat's FatQueryDirectory: every long file name was
copied at double length, past the source string and past the directory entry.
It stayed silent until the over-read of a long winsxs name crossed an unmapped
page (arm64 BUGCHECK 0x50 at smss). The same shape was in the registry value
cache (cmvalche) for uncompressed value names.

Both pointers of copy<T> share one T, so an explicit cast on EITHER side fixes
the element type. A copy is flagged when a pointer is cast to a wide type (u16,
WCHAR, wchar_t, PWSTR/LPWSTR/PWCHAR), neither is cast to a byte type, the count
names a byte quantity (`...Length`, `...Bytes...`, `cb...`, `...Size`), and the
count is not divided by 2 or by a size_of. An uncast `.Buffer` is NOT enough:
ANSI_STRING and u8 message buffers have one too, and guessing made the scan
mostly noise. The cost is a known blind spot: iomgr's IoVolumeDeviceToDosName
copied `MultiSz.as_ptr()` into `DosName.Buffer`, both u16 by declaration and
uncast, and only the broader guess found it. Five mountmgr sites were caught
by the rule as written.

FIX: copy bytes, as the C does:
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, bytes);
or divide the count: `bytes as usize / 2`.

Usage:
    python3 tools/scan_wide_byte_copy.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_wide_byte_copy.py --lint

Suppress a verified false positive (e.g. a `...Length` that is a character
count) on the call's line:
    core::ptr::copy_nonoverlapping(  // nolint: wide-byte-copy (reason)
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
NOLINT_RE = re.compile(r"//\s*nolint(?::\s*wide-byte-copy\b|\s*$|:?\s*\()")
CALL_RE = re.compile(r"\b(?:core::ptr::|std::ptr::|ptr::)?(?:copy_nonoverlapping|copy)\s*\(")
WIDE_CAST_RE = re.compile(
    r"\bas\s+(?:\*(?:mut|const)\s+(?:u16|WCHAR|wchar_t)\b|(?:L?PC?WSTR|PWCHAR)\b)"
)
BYTE_CAST_RE = re.compile(
    r"\*(?:mut|const)\s+(?:(?:core|std)::ffi::)?(?:u8|i8|c_void|c_char|UCHAR|CHAR|BYTE)\b"
)
BYTE_COUNT_RE = re.compile(r"\b(?:\w*Length|\w*Bytes\w*|\w*ByteCount|cb\w*|\w*Size)\b")
DIVIDED_RE = re.compile(r"/\s*(?:2\b|\(?\s*(?:core::|std::)?(?:mem::)?size_of)|>>\s*1\b")


def _call_args(text: str, start: int) -> list[str] | None:
    """Top-level comma-split arguments of the call whose '(' ends at start."""
    depth = 1
    k = start
    last = start
    parts: list[str] = []
    n = len(text)
    while k < n:
        c = text[k]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                parts.append(text[last:k])
                return [p.strip() for p in parts if p.strip()]
        elif c == "," and depth == 1:
            parts.append(text[last:k])
            last = k + 1
        k += 1
    return None


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings: list[tuple[Path, int, str]] = []
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
                src, dst, count = args
                ptrs = src + "\n" + dst
                if not WIDE_CAST_RE.search(ptrs) or BYTE_CAST_RE.search(ptrs):
                    continue
                if not BYTE_COUNT_RE.search(count) or DIVIDED_RE.search(count):
                    continue
                lineno = text.count("\n", 0, m.start()) + 1
                if NOLINT_RE.search(lines[lineno - 1]):
                    continue
                findings.append((f, lineno, " ".join(count.split())))
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, count in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        if lint:
            print(f"{rel}:{lineno}: wide-byte-copy {count}")
        else:
            print(f"{rel}:{lineno}: byte count `{count}` passed to a u16-typed copy (moves 2x)")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): copy as u8, or divide the count by 2.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
