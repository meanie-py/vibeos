#!/usr/bin/env python3
"""Patch PE SizeOfStackCommit in a PE32+ executable.

Usage: patch_pe_stack.py <file.exe> <commit_size_hex_or_dec>

Example: patch_pe_stack.py csrss.exe 0x40000  # Set 256KB stack commit
"""
import struct
import sys

def patch(path, commit_size):
    with open(path, 'r+b') as f:
        # DOS header -> PE offset
        f.seek(0x3C)
        pe_offset = struct.unpack('<I', f.read(4))[0]

        # Check PE signature
        f.seek(pe_offset)
        sig = f.read(4)
        if sig != b'PE\x00\x00':
            print(f"Error: Not a PE file (signature: {sig})")
            sys.exit(1)

        # Optional header magic
        f.seek(pe_offset + 24)
        magic = struct.unpack('<H', f.read(2))[0]
        if magic != 0x20B:
            print(f"Error: Not PE32+ (magic: 0x{magic:04X})")
            sys.exit(1)

        # SizeOfStackReserve is at optional header offset 72 (PE32+)
        # SizeOfStackCommit is at optional header offset 80 (PE32+)
        stack_commit_offset = pe_offset + 24 + 80

        # Read current values
        f.seek(pe_offset + 24 + 72)
        stack_reserve = struct.unpack('<Q', f.read(8))[0]
        stack_commit_old = struct.unpack('<Q', f.read(8))[0]

        print(f"Stack Reserve: 0x{stack_reserve:X} ({stack_reserve // 1024}KB)")
        print(f"Stack Commit:  0x{stack_commit_old:X} ({stack_commit_old // 1024}KB) -> 0x{commit_size:X} ({commit_size // 1024}KB)")

        # Write new commit size
        f.seek(stack_commit_offset)
        f.write(struct.pack('<Q', commit_size))

    print(f"Patched {path}")

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)
    commit = int(sys.argv[2], 0)
    patch(sys.argv[1], commit)
