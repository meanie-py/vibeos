#!/bin/bash
# Cargo-driven ntdll.dll build PoC — i386 / amd64 / arm64 (phase 1).
# See docs/engineering/ntdll-cargo-drive-audit.md.
#
# Usage: build-and-test.sh [i386|amd64|arm64]   (default: i386)
#
# cmake builds the reused sibling .a (run ninja first); cargo drives the
# final link + builds ntdll-rust + the pre-expanded dispatch asm + spec
# stubs. Validates header + export parity vs the cmake-built ntdll.dll.

set -euo pipefail
cd "$(dirname "$0")/../../../.."  # → repo root

OBJDUMP=".build-tools/llvm-mingw/bin/llvm-objdump"

arch="${1:-i386}"
# Shared cargo-invocation policy (ntdll is a user-mode cell: the builtin triple,
# stable, on every arch). Sets CELL_TARGET / CELL_STEM / CELL_STD / CELL_TOOLCHAIN.
VIBEOS_ROOT="$PWD" . sdk/lib/rust/cargo-cell-args.sh
cell_args "$arch" ntdll-cargo win32dll || { echo "usage: $0 [i386|amd64|arm64]" >&2; exit 2; }

if [ ! -d "build-$arch" ]; then echo "build-$arch/ missing — run cmake first." >&2; exit 1; fi
if [ -n "$CELL_STD" ]; then
  rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required (i386 build-std)." >&2; exit 1; }
fi

echo "[1/3] Refreshing cmake-built inputs (build-$arch: ntdll → sibling .a) ..."
( cd "build-$arch" && ninja ntdll >/dev/null )

echo "[2/3] Cargo build (ntdll-rust + ntdll-cargo, $CELL_STEM) ..."
cd sdk/lib/rust
# shellcheck disable=SC2086  # CELL_TOOLCHAIN/CELL_STD are deliberately word-split
cargo $CELL_TOOLCHAIN build --release \
    --target "$CELL_TARGET" \
    $CELL_STD \
    -p ntdll-cargo --bin ntdll
cd ../../..

CARGO_OUT="sdk/lib/rust/target/$CELL_STEM/release/ntdll.exe"   # -shared DLL, .exe bin name
CMAKE_OUT="build-$arch/dll/ntdll/ntdll.dll"
echo "    produced: $CARGO_OUT ($(stat -c %s "$CARGO_OUT") bytes)"

echo "[3/3] Validating PE + export parity ..."
test "$(head -c2 "$CARGO_OUT")" = "MZ" || { echo "FAIL: not a PE (no MZ)"; exit 1; }
HDR="$("$OBJDUMP" -p "$CARGO_OUT" 2>/dev/null)"
case "$HDR" in *"Windows CUI"*) ;; *) echo "FAIL: wrong subsystem"; exit 1;; esac
cexp() { "$OBJDUMP" -p "$1" 2>/dev/null | grep -oE '\b[A-Za-z_][A-Za-z0-9_]+$' | grep -vxE 'exe|dll' | sort -u; }
if diff <(cexp "$CARGO_OUT") <(cexp "$CMAKE_OUT") >/dev/null; then
  echo "    OK ($arch): export set identical to cmake ntdll.dll"
else
  echo "FAIL: export set differs from cmake ntdll.dll"; exit 1
fi
echo "Done — cargo-driven ntdll.dll ($arch) matches cmake (headers + exports)."
