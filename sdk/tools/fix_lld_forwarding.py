#!/usr/bin/env python3
"""
Fix lld i386 forwarding string underscore bug in PE binaries.

On i386, lld (LLVM linker) incorrectly adds a '_' prefix to forwarding
target strings in PE export tables. For example:
  ntdll.RtlDeleteCriticalSection -> _ntdll.RtlDeleteCriticalSection

The PE loader treats forwarding strings as literal "DLL.Function" pairs.
The '_' prefix causes it to look for "_ntdll.dll" instead of "ntdll.dll".

This script patches PE binaries by adjusting the RVA of each broken
forwarding entry to point past the underscore (RVA + 1), so the PE
loader sees the correct "DLL.Function" string.

Usage: python3 fix_lld_forwarding.py <file.dll> [file2.dll ...]
"""

import struct
import sys
import os


def read_u16(data, offset):
    return struct.unpack_from('<H', data, offset)[0]


def read_u32(data, offset):
    return struct.unpack_from('<I', data, offset)[0]


def write_u32(data, offset, value):
    struct.pack_into('<I', data, offset, value)


def rva_to_offset(sections, rva):
    """Convert an RVA to a file offset using the section table."""
    for vaddr, vsize, rawoff, rawsize in sections:
        if vaddr <= rva < vaddr + max(vsize, rawsize):
            return rva - vaddr + rawoff
    return None


def fix_pe_forwarding(filepath):
    """Fix forwarding strings in a PE binary. Returns number of fixes."""
    with open(filepath, 'rb') as f:
        data = bytearray(f.read())

    # DOS header
    if data[0:2] != b'MZ':
        return 0
    pe_offset = read_u32(data, 0x3C)

    # PE signature
    if data[pe_offset:pe_offset+4] != b'PE\0\0':
        return 0

    coff_offset = pe_offset + 4
    num_sections = read_u16(data, coff_offset + 2)
    optional_hdr_size = read_u16(data, coff_offset + 16)
    optional_offset = coff_offset + 20

    # Check PE32 (not PE32+)
    magic = read_u16(data, optional_offset)
    if magic != 0x10B:  # PE32
        return 0

    # Export directory RVA and size (data directory entry 0)
    export_dir_rva = read_u32(data, optional_offset + 96)
    export_dir_size = read_u32(data, optional_offset + 100)
    if export_dir_rva == 0:
        return 0

    # Parse section table
    section_table_offset = optional_offset + optional_hdr_size
    sections = []
    for i in range(num_sections):
        s = section_table_offset + i * 40
        vsize = read_u32(data, s + 8)
        vaddr = read_u32(data, s + 12)
        rawsize = read_u32(data, s + 16)
        rawoff = read_u32(data, s + 20)
        sections.append((vaddr, vsize, rawoff, rawsize))

    # Parse export directory
    export_offset = rva_to_offset(sections, export_dir_rva)
    if export_offset is None:
        return 0

    num_functions = read_u32(data, export_offset + 20)
    addr_table_rva = read_u32(data, export_offset + 28)
    addr_table_offset = rva_to_offset(sections, addr_table_rva)
    if addr_table_offset is None:
        return 0

    # Forwarding: an entry in AddressOfFunctions whose RVA falls within
    # the export directory is a forwarding string RVA.
    export_dir_end = export_dir_rva + export_dir_size
    fixes = 0

    for i in range(num_functions):
        entry_offset = addr_table_offset + i * 4
        func_rva = read_u32(data, entry_offset)

        # Check if this is a forwarding entry (RVA within export directory)
        if export_dir_rva <= func_rva < export_dir_end:
            str_offset = rva_to_offset(sections, func_rva)
            if str_offset is None:
                continue

            # Read the forwarding string
            end = data.index(b'\0', str_offset)
            fwd_str = data[str_offset:end].decode('ascii', errors='replace')

            # Check if it has the underscore bug: starts with '_' and
            # contains a dot (DLL.Function format)
            if fwd_str.startswith('_') and '.' in fwd_str:
                # Fix: advance the RVA by 1 to skip the underscore
                write_u32(data, entry_offset, func_rva + 1)
                fixes += 1

    if fixes > 0:
        with open(filepath, 'wb') as f:
            f.write(data)

    return fixes


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <file.dll> [file2.dll ...]", file=sys.stderr)
        sys.exit(1)

    total = 0
    for filepath in sys.argv[1:]:
        if not os.path.isfile(filepath):
            print(f"Skip (not found): {filepath}", file=sys.stderr)
            continue
        fixes = fix_pe_forwarding(filepath)
        if fixes > 0:
            print(f"Fixed {fixes} forwarding entries in {os.path.basename(filepath)}")
            total += fixes

    if total == 0:
        # Silent success when no fixes needed
        pass


if __name__ == '__main__':
    main()
