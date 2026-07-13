#!/bin/bash
# Cargo-driven ntdll_vista.dll build PoC — i386 / amd64 / arm64 (phase 1).
# See docs/engineering/ntdll-cargo-drive-audit.md.
#
# Usage: build-and-test.sh [i386|amd64|arm64]   (default: i386)
#
# Validates: valid PE, GUI subsystem (lld default, matches cmake's vista),
# and full ntdll_vista.spec export coverage. The full export set is NOT
# byte-equal to cmake's — both auto-export incidental compiler_builtins
# globals beyond the spec — so we assert the spec contract.

set -euo pipefail
cd "$(dirname "$0")/../../../.."  # → repo root

OBJDUMP=".build-tools/llvm-mingw/bin/llvm-objdump"
SPEC="dll/ntdll/nt_0600/ntdll_vista.spec"

arch="${1:-i386}"
# Shared cargo-invocation policy (builtin triple on amd64/arm64, nightly +
# build-std on i386). Sets CELL_TARGET / CELL_STEM / CELL_STD / CELL_TOOLCHAIN.
VIBEOS_ROOT="$PWD" . sdk/lib/rust/cargo-cell-args.sh
cell_args "$arch" ntdll_vista-cargo || { echo "usage: $0 [i386|amd64|arm64]" >&2; exit 2; }

if [ ! -d "build-$arch" ]; then echo "build-$arch/ missing — run cmake first." >&2; exit 1; fi
if [ -n "$CELL_STD" ]; then
  rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required (i386 build-std)." >&2; exit 1; }
fi

echo "[1/3] Refreshing cmake-built inputs (build-$arch: ntdll_vista → sibling .a) ..."
( cd "build-$arch" && ninja ntdll_vista >/dev/null )

echo "[2/3] Cargo build (ntdll-rust[vista] + ntdll_vista-cargo, $CELL_STEM) ..."
cd sdk/lib/rust
# shellcheck disable=SC2086  # CELL_TOOLCHAIN/CELL_STD are deliberately word-split
cargo $CELL_TOOLCHAIN build --release \
    --target "$CELL_TARGET" \
    $CELL_STD \
    -p ntdll_vista-cargo --bin ntdll_vista
cd ../../..

OUT="sdk/lib/rust/target/$CELL_STEM/release/ntdll_vista.exe"
echo "    produced: $OUT ($(stat -c %s "$OUT") bytes)"

echo "[3/3] Validating PE + spec-export coverage ..."
test "$(head -c2 "$OUT")" = "MZ" || { echo "FAIL: not a PE (no MZ)"; exit 1; }
HDR="$("$OBJDUMP" -p "$OUT" 2>/dev/null)"
case "$HDR" in *"Windows GUI"*) ;; *) echo "FAIL: wrong subsystem"; exit 1;; esac
EXPORTS="$("$OBJDUMP" -p "$OUT" 2>/dev/null | grep -oE '\b[A-Za-z_][A-Za-z0-9_]+$' | sort -u)"
missing=0
while read -r name; do
  [ -n "$name" ] || continue
  grep -qxF "$name" <<<"$EXPORTS" || { echo "  MISSING spec export: $name"; missing=1; }
done < <(grep -E '^@' "$SPEC" | awk '{print $3}' | sed 's/(.*//' | sort -u)
test "$missing" -eq 0 || { echo "FAIL: missing spec exports"; exit 1; }
echo "    OK ($arch): valid PE (GUI), all ntdll_vista.spec exports present."
echo "Done — cargo-driven ntdll_vista.dll ($arch) matches cmake's spec contract."
