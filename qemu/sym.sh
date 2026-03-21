#!/bin/bash
#
# Symbolicate a crash address from uefildr.efi
#
# Usage:
#   ./sym.sh 0xC804           # Look up single address
#   ./sym.sh 0xC804 0xC76C    # Look up multiple addresses
#

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_DIR/build-arm64"
TOOLCHAIN="$PROJECT_DIR/llvm-mingw-20250114-ucrt-macos-universal"
DEBUG_EFI="$SCRIPT_DIR/uefildr-debug.efi"
OBJDUMP="$TOOLCHAIN/bin/llvm-objdump"

# Build unstripped copy if it doesn't exist or is stale
STRIPPED_EFI="$BUILD_DIR/boot/freeldr/freeldr/uefildr.efi"
if [ ! -f "$DEBUG_EFI" ] || [ "$STRIPPED_EFI" -nt "$DEBUG_EFI" ]; then
    echo "[*] Building unstripped uefildr-debug.efi..."
    LINK_CMD=$(cat "$BUILD_DIR/boot/freeldr/freeldr/CMakeFiles/uefildr.dir/link.txt" 2>/dev/null)
    # Extract the linker invocation (after the last "&&" before strip)
    LINK_ONLY=$(echo "$LINK_CMD" | grep -o ': && .*--end-group' | sed 's/^: && //')
    if [ -z "$LINK_ONLY" ]; then
        echo "[!] Could not extract link command. Rebuild with: ninja -C $BUILD_DIR uefildr"
        exit 1
    fi
    # Replace output path
    LINK_ONLY=$(echo "$LINK_ONLY" | sed "s|-o [^ ]*|-o $DEBUG_EFI|")
    (cd "$BUILD_DIR" && eval "$LINK_ONLY") 2>&1
fi

if [ $# -eq 0 ]; then
    echo "Usage: $0 <offset> [offset...]"
    echo "  Offset is relative to image base (from UEFI crash dump)"
    echo "  Example: ./sym.sh 0xC804"
    exit 0
fi

# Build symbol table once
SYMS=$("$OBJDUMP" -t "$DEBUG_EFI" 2>&1 | grep "(sec  1)")

for addr in "$@"; do
    # Strip leading 0x if present
    addr_hex=$(echo "$addr" | sed 's/^0[xX]//')
    addr_dec=$((16#$addr_hex))

    result=$(echo "$SYMS" | python3 -c "
import sys, re
target = $addr_dec
last = None
for line in sys.stdin:
    m = re.search(r'(0x[0-9a-f]+)\s+(\S+)\s*$', line.strip())
    if m:
        a = int(m.group(1), 16)
        n = m.group(2)
        if a <= target: last = (n, a)
if last: print(f'0x{target:x}: {last[0]} + 0x{target-last[1]:x} (func @ 0x{last[1]:x})')
else: print(f'0x{target:x}: (unknown)')
")
    echo "$result"
done
