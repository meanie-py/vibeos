#!/bin/bash
# Verify that sacred IRQL/spinlock primitives survive LLVM optimization.
#
# Usage: bash tools/verify_sacred_primitives.sh [build-dir]
#   build-dir defaults to build-i386
#
# Checks that critical kernel functions (KfRaiseIrql, KfLowerIrql,
# KeAcquireSpinLock, etc.) appear as actual call/branch instructions
# in the final ntoskrnl binary. If LLVM silently eliminates these calls,
# the kernel will have invisible IRQL/lock corruption.

set -euo pipefail

BUILD_DIR="${1:-build-i386}"
NTOSKRNL="$BUILD_DIR/ntoskrnl/ntoskrnl.exe"

if [ ! -f "$NTOSKRNL" ]; then
    echo "ERROR: $NTOSKRNL not found. Build first."
    exit 1
fi

# Detect architecture from build dir name
case "$BUILD_DIR" in
    *arm64*) OBJDUMP="aarch64-w64-mingw32-objdump"; ARCH="arm64" ;;
    *amd64*) OBJDUMP="x86_64-w64-mingw32-objdump"; ARCH="amd64" ;;
    *i386*)  OBJDUMP="i686-w64-mingw32-objdump"; ARCH="i386" ;;
    *)       echo "Cannot determine arch from $BUILD_DIR"; exit 1 ;;
esac

# Resolve a working objdump: the per-arch mingw objdump or llvm-objdump on
# PATH, else the same from the repo's bundled llvm-mingw toolchain. The
# bundled fallback matters for arm64 — a stock box has the x86_64/i686 mingw
# objdumps in /usr/bin but neither aarch64-w64-mingw32-objdump nor
# llvm-objdump, and .build-tools/llvm-mingw/bin isn't necessarily on PATH.
BUNDLED_BIN="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/.build-tools/llvm-mingw/bin"
resolve_objdump() {
    for c in "$OBJDUMP" "llvm-objdump" "$BUNDLED_BIN/$OBJDUMP" "$BUNDLED_BIN/llvm-objdump"; do
        if command -v "$c" &>/dev/null; then
            OBJDUMP="$c"
            return 0
        fi
    done
    return 1
}
if ! resolve_objdump; then
    echo "ERROR: no objdump found (tried $OBJDUMP, llvm-objdump, and $BUNDLED_BIN)"
    exit 1
fi

echo "Checking $NTOSKRNL ($ARCH) with $OBJDUMP..."
echo ""

# Sacred primitives that MUST appear in the binary
PRIMITIVES=(
    "KfRaiseIrql"
    "KfLowerIrql"
    "KeAcquireSpinLock"
    "KeReleaseSpinLock"
)

# amd64 uses CR8 directly for IRQL, so KfRaiseIrql/KfLowerIrql
# may not appear — check for CR8 access instead
if [ "$ARCH" = "amd64" ]; then
    PRIMITIVES=(
        "KeAcquireSpinLock"
        "KeReleaseSpinLock"
    )
fi

DISASM=$("$OBJDUMP" -d "$NTOSKRNL" 2>/dev/null)
FAIL=0

for prim in "${PRIMITIVES[@]}"; do
    COUNT=$(echo "$DISASM" | grep -c "$prim" || true)
    if [ "$COUNT" -eq 0 ]; then
        echo "FAIL: $prim — not found in disassembly! LLVM may have eliminated it."
        FAIL=1
    else
        echo "  OK: $prim — $COUNT references"
    fi
done

# For amd64, check that CR8 accesses exist (mov cr8)
if [ "$ARCH" = "amd64" ]; then
    CR8_COUNT=$(echo "$DISASM" | grep -c "cr8" || true)
    if [ "$CR8_COUNT" -eq 0 ]; then
        echo "FAIL: No CR8 accesses found — IRQL primitives may be eliminated"
        FAIL=1
    else
        echo "  OK: CR8 accesses — $CR8_COUNT references"
    fi
fi

echo ""
if [ "$FAIL" -eq 0 ]; then
    echo "All sacred primitives verified."
else
    echo "WARNING: Some sacred primitives missing from binary!"
    echo "Check --emit=llvm-ir output for the affected functions."
    exit 1
fi
