#!/usr/bin/env python3
"""
check_freeldr_size.py -- the BIOS FreeLoader must fit the real-mode memory it runs in.

freeldr.sys is loaded below MEMORY_MARGIN, and MmCheckFreeldrImageFile bug-checks
(FREELDR_IMAGE_CORRUPTION, "FreeLdr OptionalHeader is invalid") when the PE's
SizeOfImage exceeds MAX_FREELDR_PE_SIZE = MEMORY_MARGIN - FREELDR_PE_BASE - PAGE_SIZE.
Nothing checked that at BUILD time, so the first sign of an overflow was every
amd64-bios boot hanging in the merge-queue gate (2026-10-10: restoring ~960 freeldr
debug calls took freeldr.sys from 433,664 to 493,568 bytes against a 487,424 limit).
This reads the built PE and fails the build instead.

The limit is parsed from boot/freeldr/freeldr/include/arch/pc/x86common.h -- the
header the boot-time check is compiled from -- never hardcoded here.

Usage:
    python3 tools/check_freeldr_size.py --check build-amd64
    python3 tools/check_freeldr_size.py --self-test
"""

from __future__ import annotations

import argparse
import re
import struct
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
HEADER = REPO / "boot" / "freeldr" / "freeldr" / "include" / "arch" / "pc" / "x86common.h"
PE = Path("boot") / "freeldr" / "freeldr" / "freeldr_pe.exe"
PAGE_SIZE = 0x1000
# Warn before the wall: master had 29,184 bytes of SizeOfImage headroom when this was
# written (2026-10-10); 12 KB left is the point to start budgeting.
WARN_HEADROOM = 0x3000


def limit_from_header(text: str) -> int:
    """MAX_FREELDR_PE_SIZE, evaluated from the header's own #defines."""
    defs = {
        m.group(1): int(m.group(2), 16)
        for m in re.finditer(r"#define\s+(\w+)\s+HEX\(([0-9A-Fa-f]+)\)", text)
    }
    expr = re.search(r"#define\s+MAX_FREELDR_PE_SIZE\s+\(([^)]*)\)", text)
    if expr is None or "MEMORY_MARGIN" not in defs or "FREELDR_PE_BASE" not in defs:
        raise ValueError("x86common.h no longer defines MAX_FREELDR_PE_SIZE as expected")
    if expr.group(1).split() != ["MEMORY_MARGIN", "-", "FREELDR_PE_BASE", "-", "PAGE_SIZE"]:
        raise ValueError(f"MAX_FREELDR_PE_SIZE changed shape: {expr.group(1)!r}; update this tool")
    return defs["MEMORY_MARGIN"] - defs["FREELDR_PE_BASE"] - PAGE_SIZE


def size_of_image(pe: bytes) -> int:
    if pe[:2] != b"MZ":
        raise ValueError("not a PE (no MZ)")
    off = struct.unpack_from("<I", pe, 0x3C)[0]
    if pe[off : off + 4] != b"PE\0\0":
        raise ValueError("not a PE (no PE signature)")
    return struct.unpack_from("<I", pe, off + 24 + 56)[0]  # OptionalHeader.SizeOfImage


def check(build_dir: Path) -> int:
    pe = build_dir / PE
    if not pe.is_file():
        print(f"SKIP: {pe} not built (no BIOS loader on this arch)")
        return 0
    limit = limit_from_header(HEADER.read_text("utf8", "replace"))
    size = size_of_image(pe.read_bytes())
    room = limit - size
    msg = f"freeldr_pe SizeOfImage 0x{size:x} ({size}) vs MAX_FREELDR_PE_SIZE 0x{limit:x}"
    if room < 0:
        print(f"FAIL: {msg} -- {-room} bytes OVER: every BIOS boot bug-checks in")
        print("  MmCheckFreeldrImageFile. Shrink the BIOS image (freeldr-rust without")
        print("  `uefi`/`rosload` is the one with the budget).")
        return 1
    tag = "WARN" if room < WARN_HEADROOM else "OK"
    print(f"{tag}: {msg} -- {room} bytes of headroom")
    return 0


def self_test() -> int:
    hdr = (
        "#define FREELDR_PE_BASE    HEX(10000)\n#define MEMORY_MARGIN      HEX(88000)\n"
        "#define MAX_FREELDR_PE_SIZE (MEMORY_MARGIN - FREELDR_PE_BASE - PAGE_SIZE)\n"
    )
    ok = limit_from_header(hdr) == 0x77000
    pe = bytearray(0x200)
    pe[:2] = b"MZ"
    struct.pack_into("<I", pe, 0x3C, 0x80)
    pe[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<I", pe, 0x80 + 24 + 56, 0x70200)
    ok &= size_of_image(bytes(pe)) == 0x70200
    try:
        limit_from_header(hdr.replace("- PAGE_SIZE", "- PAGE_SIZE * 2"))
        ok = False
    except ValueError:
        pass
    ok &= limit_from_header(HEADER.read_text("utf8", "replace")) == 0x77000
    print("self-test:", "passed" if ok else "FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--check", type=Path, metavar="BUILD_DIR")
    ap.add_argument("--self-test", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        return self_test()
    if a.check is None:
        ap.error("--check BUILD_DIR or --self-test")
    return check(a.check)


if __name__ == "__main__":
    sys.exit(main())
