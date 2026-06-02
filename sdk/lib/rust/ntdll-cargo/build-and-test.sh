#!/bin/bash
# Cargo-driven ntdll.dll build PoC — i386 only (phase 1).
# See docs/engineering/ntdll-cargo-drive-audit.md.
#
# 1. cmake/ninja once so the sibling static libs + the lean csrlib archive
#    exist (cmake does the heavy lifting; cargo drives the final link and
#    builds ntdll-rust + the pre-expanded dispatch asm + spec stubs).
# 2. Drive the cargo link of `ntdll-cargo` against `i686-kernel.json`.
# 3. Validate the produced PE (MZ, console subsystem, export parity vs the
#    cmake-built ntdll.dll).
#
# Run from anywhere.

set -euo pipefail

cd "$(dirname "$0")/../../../.."  # → repo root

OBJDUMP=".build-tools/llvm-mingw/bin/llvm-objdump"

# ---- 0. Sanity checks -----------------------------------------------------
if [ ! -d build-i386 ]; then
  echo "build-i386/ does not exist — run cmake first." >&2
  exit 1
fi
if ! rustup toolchain list 2>&1 | grep -q nightly; then
  echo "nightly toolchain required for build-std (rustup toolchain install nightly + rust-src)." >&2
  exit 1
fi

# ---- 1. cmake/ninja produces the reused static libs ---------------------
echo "[1/3] Refreshing cmake-built inputs (ntdll → sibling .a) ..."
( cd build-i386 && ninja ntdll >/dev/null )

# ---- 2. Cargo-driven link -----------------------------------------------
echo "[2/3] Cargo build (ntdll-rust + ntdll-cargo) ..."
cd sdk/lib/rust
cargo +nightly build --release \
    --target ntdll-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p ntdll-cargo --bin ntdll
cd ../../..

CARGO_OUT="sdk/lib/rust/target/i686-kernel/release/ntdll.exe"   # -shared DLL, .exe bin name
CMAKE_OUT="build-i386/dll/ntdll/ntdll.dll"
echo "    produced: $CARGO_OUT ($(stat -c %s "$CARGO_OUT") bytes)"

# ---- 3. Validate PE + export parity -------------------------------------
# NB: capture objdump output into vars rather than piping into `grep -q`
# (grep -q closes the pipe early → objdump SIGPIPE → `set -o pipefail`
# would spuriously fail the check).
echo "[3/3] Validating PE ..."
test "$(head -c2 "$CARGO_OUT")" = "MZ" || { echo "FAIL: not a PE (no MZ)"; exit 1; }
HDR="$("$OBJDUMP" -p "$CARGO_OUT" 2>/dev/null)"
case "$HDR" in *"Windows CUI"*) ;; *) echo "FAIL: wrong subsystem"; exit 1;; esac
cexp() { "$OBJDUMP" -p "$1" 2>/dev/null | grep -oE '\b[A-Za-z_][A-Za-z0-9_]+$' | grep -vxE 'exe|dll' | sort -u; }
if diff <(cexp "$CARGO_OUT") <(cexp "$CMAKE_OUT") >/dev/null; then
  echo "    OK: export set identical to cmake ntdll.dll"
else
  echo "FAIL: export set differs from cmake ntdll.dll"; exit 1
fi
echo "Done — cargo-driven ntdll.dll matches cmake (headers + exports)."
