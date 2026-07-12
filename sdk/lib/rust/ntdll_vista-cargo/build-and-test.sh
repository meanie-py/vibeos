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

# Pinned nightly, single-sourced from the repo root (see rust-toolchain.toml).
VIBEOS_NIGHTLY="$(cat "$PWD/.rust-nightly")"
OBJDUMP=".build-tools/llvm-mingw/bin/llvm-objdump"
SPEC="dll/ntdll/nt_0600/ntdll_vista.spec"

arch="${1:-i386}"
case "$arch" in
  i386)  tgt="i686-kernel" ;;
  amd64) tgt="x86_64-kernel" ;;
  arm64) tgt="aarch64-kernel" ;;
  *) echo "usage: $0 [i386|amd64|arm64]" >&2; exit 2 ;;
esac

if [ ! -d "build-$arch" ]; then echo "build-$arch/ missing — run cmake first." >&2; exit 1; fi
rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required." >&2; exit 1; }

echo "[1/3] Refreshing cmake-built inputs (build-$arch: ntdll_vista → sibling .a) ..."
( cd "build-$arch" && ninja ntdll_vista >/dev/null )

echo "[2/3] Cargo build (ntdll-rust[vista] + ntdll_vista-cargo, $tgt) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target "ntdll_vista-cargo/$tgt.json" \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p ntdll_vista-cargo --bin ntdll_vista
cd ../../..

OUT="sdk/lib/rust/target/$tgt/release/ntdll_vista.exe"
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
