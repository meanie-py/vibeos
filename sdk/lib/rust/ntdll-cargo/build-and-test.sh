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

# Pinned nightly, single-sourced from the repo root (see rust-toolchain.toml).
VIBEOS_NIGHTLY="$(cat "$PWD/.rust-nightly")"
OBJDUMP=".build-tools/llvm-mingw/bin/llvm-objdump"

arch="${1:-i386}"
case "$arch" in
  i386)  tgt="i686-kernel" ;;
  amd64) tgt="x86_64-kernel" ;;
  arm64) tgt="aarch64-kernel" ;;
  *) echo "usage: $0 [i386|amd64|arm64]" >&2; exit 2 ;;
esac

if [ ! -d "build-$arch" ]; then echo "build-$arch/ missing — run cmake first." >&2; exit 1; fi
rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required." >&2; exit 1; }

echo "[1/3] Refreshing cmake-built inputs (build-$arch: ntdll → sibling .a) ..."
( cd "build-$arch" && ninja ntdll >/dev/null )

echo "[2/3] Cargo build (ntdll-rust + ntdll-cargo, $tgt) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target "ntdll-cargo/$tgt.json" \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p ntdll-cargo --bin ntdll
cd ../../..

CARGO_OUT="sdk/lib/rust/target/$tgt/release/ntdll.exe"   # -shared DLL, .exe bin name
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
