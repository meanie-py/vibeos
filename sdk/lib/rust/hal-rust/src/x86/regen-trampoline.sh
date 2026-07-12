#!/usr/bin/env bash
# Regenerate the AP trampoline flat binaries from source (SMP M2).
#
# Uses the SYSTEM GNU as/ld (ELF x86_64) — NOT clang (.code16, AGENTS.md) and
# NOT the mingw toolchain (we want a flat binary, PE adds headers). The .bin
# is include_bytes!'d by trampoline.rs; check in .S + .bin together and rerun
# this whenever the .S changes.
set -euo pipefail
cd "$(dirname "$0")"

as --64 -o aptramp64.o aptramp64.S
ld -Ttext=0 -e _start --oformat binary -o aptramp64.bin aptramp64.o
rm -f aptramp64.o

size=$(stat -c%s aptramp64.bin)
# Must stay clear of the param block at page offset 0x800 (with headroom).
if [ "$size" -gt 2032 ]; then
    echo "FATAL: aptramp64.bin is ${size}B — collides with the 0x800 param block" >&2
    exit 1
fi
echo "aptramp64.bin: ${size}B  sha256=$(sha256sum aptramp64.bin | cut -d' ' -f1)"

# i386 — same recipe, ELF32 target (as/ld are multilib on this host; system
# GNU as/ld, not the mingw/llvm-mingw cross toolchain — flat binary, no PE).
as --32 -o aptramp32.o aptramp32.S
ld -m elf_i386 -Ttext=0 -e _start --oformat binary -o aptramp32.bin aptramp32.o
rm -f aptramp32.o

size32=$(stat -c%s aptramp32.bin)
if [ "$size32" -gt 2032 ]; then
    echo "FATAL: aptramp32.bin is ${size32}B — collides with the 0x800 param block" >&2
    exit 1
fi
echo "aptramp32.bin: ${size32}B  sha256=$(sha256sum aptramp32.bin | cut -d' ' -f1)"
