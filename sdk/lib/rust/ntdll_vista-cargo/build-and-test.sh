#!/bin/bash
# Cargo-driven ntdll_vista.dll build PoC — i386 (phase 1 companion to
# ntdll-cargo). See docs/engineering/ntdll-cargo-drive-audit.md.
#
# Validates: valid PE, GUI subsystem (lld default, matches cmake's vista),
# and full ntdll_vista.spec export coverage. NB: unlike ntdll.dll, the full
# export set is NOT byte-equal to cmake's — both auto-export incidental
# globals beyond the spec (cmake: mem*/strlen from compiler_builtins; cargo:
# RtlInitializeCriticalSection) and those visibility quirks differ. The DLL
# contract is the spec, so we assert spec coverage.

set -euo pipefail
cd "$(dirname "$0")/../../../.."  # → repo root
OBJDUMP=".build-tools/llvm-mingw/bin/llvm-objdump"
SPEC="dll/ntdll/nt_0600/ntdll_vista.spec"

if [ ! -d build-i386 ]; then echo "build-i386/ missing — run cmake first." >&2; exit 1; fi
rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required." >&2; exit 1; }

echo "[1/3] Refreshing cmake-built inputs (ntdll_vista → sibling .a) ..."
( cd build-i386 && ninja ntdll_vista >/dev/null )

echo "[2/3] Cargo build (ntdll-rust[vista] + ntdll_vista-cargo) ..."
cd sdk/lib/rust
cargo +nightly build --release \
    --target ntdll_vista-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p ntdll_vista-cargo --bin ntdll_vista
cd ../../..

OUT="sdk/lib/rust/target/i686-kernel/release/ntdll_vista.exe"
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
echo "    OK: valid PE (GUI 6.0), all ntdll_vista.spec exports present."
echo "Done — cargo-driven ntdll_vista.dll matches cmake's spec contract."
