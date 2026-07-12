#!/bin/bash
# Cargo-driven ntoskrnl.exe build PoC — i386 only.
#
# 1. Run cmake/ninja once so the C shim .obj files + sibling static libs
#    exist (cmake does the heavy lifting; cargo just drives the final
#    link).
# 2. Drive the cargo build of `ntoskrnl-cargo` against the custom
#    `i686-kernel.json` target (strips user-mode CRT). Cargo builds
#    `kernel-rust` as a sub-step from build.rs.
# 3. (Optional) swap the cargo-built binary into `build-i386/ntoskrnl/`,
#    rebuild the LiveCD, and run the smoke tests.
#
# Run from the repo root.

set -euo pipefail

cd "$(dirname "$0")/../../../.."  # → repo root

# Pinned nightly, single-sourced from the repo root (see rust-toolchain.toml).
VIBEOS_NIGHTLY="$(cat "$PWD/.rust-nightly")"

# ---- 0. Sanity checks -----------------------------------------------------
if [ ! -d build-i386 ]; then
  echo "build-i386/ does not exist — run cmake first." >&2
  exit 1
fi
if ! rustup toolchain list 2>&1 | grep -q nightly; then
  echo "nightly toolchain required for build-std." >&2
  echo "  rustup toolchain install nightly" >&2
  echo "  rustup component add rust-src --toolchain nightly" >&2
  exit 1
fi

# ---- 1. cmake/ninja produces .obj files + static libs --------------------
echo "[1/3] Refreshing cmake-built inputs ..."
( cd build-i386 && ninja ntoskrnl >/dev/null )

# ---- 2. Cargo-driven link -----------------------------------------------
echo "[2/3] Cargo build (kernel-rust + ntoskrnl-cargo) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target ntoskrnl-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p ntoskrnl-cargo --bin ntoskrnl

CARGO_OUT="$(pwd)/target/i686-kernel/release/ntoskrnl.exe"
echo "    produced: $CARGO_OUT ($(stat -c %s "$CARGO_OUT") bytes)"

# ---- 3. Swap into livecd + run smoke (opt-in via $RUN_SMOKE=1) ----------
if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd ../../..
  echo "[3/3] Swapping into build-i386/ntoskrnl/ + smoke ..."
  cp "build-i386/ntoskrnl/ntoskrnl.exe" "build-i386/ntoskrnl/ntoskrnl.exe.cmake-backup"
  cp "$CARGO_OUT" "build-i386/ntoskrnl/ntoskrnl.exe"
  ( cd build-i386 && ninja livecd >/dev/null )
  source .venv/bin/activate
  trap 'mv build-i386/ntoskrnl/ntoskrnl.exe.cmake-backup build-i386/ntoskrnl/ntoskrnl.exe; (cd build-i386 && ninja livecd >/dev/null) || true' EXIT
  pytest tests/qemu/test_smoke.py --arch i386 --timeout 300 --skip-clean-build
fi

echo "Done."
