#!/bin/bash
# Cargo-driven uefildr.efi build PoC — i386 + amd64 + arm64.
#
# Usage: build-and-test.sh [i386|amd64|arm64]   (default i386)
#
# 1. Run cmake/ninja once so the prebuilt strsafe_shim.c.obj + sibling
#    static libs + libfreeldr_rust_uefi.a exist (cmake does the heavy
#    lifting; cargo just drives the final EFI link).
# 2. Drive the cargo build of `uefildr-cargo` against the custom per-arch
#    kernel target json. build.rs links the cmake-built inputs via
#    reactos-link's ModuleKind::EfiApplication.
# 3. Post-build: rename uefildr.exe -> uefildr.efi and strip .rossym +
#    all symbols (matches the cmake POST_BUILD).
# 4. (Opt-in $RUN_SMOKE=1) swap the cargo-built uefildr.efi into
#    build-<arch>/, rebuild the LiveCD, and run smoke. Restores the cmake
#    binary on exit.
#
# Run from anywhere (cd's to repo root).

set -euo pipefail

ARCH="${1:-i386}"
case "$ARCH" in
  i386)  TARGET_JSON="uefildr-cargo/i686-kernel.json";    SMOKE_ARCH="i386-uefi" ;;
  amd64) TARGET_JSON="uefildr-cargo/x86_64-kernel.json";  SMOKE_ARCH="amd64-uefi" ;;
  arm64) TARGET_JSON="uefildr-cargo/aarch64-kernel.json"; SMOKE_ARCH="arm64" ;;
  *) echo "usage: $0 [i386|amd64|arm64]" >&2; exit 1 ;;
esac

cd "$(dirname "$0")/../../../.."  # → repo root

# Pinned nightly, single-sourced from the repo root (see rust-toolchain.toml).
VIBEOS_NIGHTLY="$(cat "$PWD/.rust-nightly")"
REPO="$(pwd)"
STRIP="$REPO/.build-tools/llvm-mingw/bin/llvm-strip"
BUILD_DIR="build-$ARCH"

# ---- 0. Sanity checks -----------------------------------------------------
if [ ! -d "$BUILD_DIR" ]; then
  echo "$BUILD_DIR/ does not exist — run cmake first." >&2
  exit 1
fi
if ! rustup toolchain list 2>&1 | grep -q nightly; then
  echo "nightly toolchain required for build-std." >&2
  exit 1
fi

# ---- 1. cmake/ninja produces the prebuilt inputs -------------------------
echo "[1/4] Refreshing cmake-built inputs (ninja uefildr, $ARCH) ..."
( cd "$BUILD_DIR" && ninja uefildr >/dev/null )

# ---- 2. Cargo-driven EFI link -------------------------------------------
echo "[2/4] Cargo build (uefildr-cargo, $ARCH) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target "$TARGET_JSON" \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p uefildr-cargo --bin uefildr

TGT_TRIPLE="$(basename "$TARGET_JSON" .json)"
CARGO_EXE="$(pwd)/target/$TGT_TRIPLE/release/uefildr.exe"
CARGO_EFI="$(pwd)/target/$TGT_TRIPLE/release/uefildr.efi"

# ---- 3. Post-build: rename + strip (matches cmake POST_BUILD) ------------
echo "[3/4] Strip + rename -> uefildr.efi ..."
cp "$CARGO_EXE" "$CARGO_EFI"
"$STRIP" --remove-section=.rossym "$CARGO_EFI"
"$STRIP" --strip-all "$CARGO_EFI"
echo "    produced: $CARGO_EFI ($(stat -c %s "$CARGO_EFI") bytes)"

# ---- 4. Swap into livecd + smoke (opt-in via $RUN_SMOKE=1) ---------------
if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd "$REPO"
  TGT="$BUILD_DIR/boot/freeldr/freeldr/uefildr.efi"
  echo "[4/4] Swapping cargo uefildr.efi into $BUILD_DIR/ + smoke ..."
  cp "$TGT" "$TGT.cmake-backup"
  cp "$CARGO_EFI" "$TGT"
  ( cd "$BUILD_DIR" && ninja livecd >/dev/null )
  trap 'mv "'"$REPO"'/'"$TGT"'.cmake-backup" "'"$REPO"'/'"$TGT"'"; (cd "'"$REPO"'/'"$BUILD_DIR"'" && ninja livecd >/dev/null) || true' EXIT
  /home/curi/Projects/os-playground/vibeos/.venv/bin/python -m pytest \
      tests/qemu/test_smoke.py --arch "$SMOKE_ARCH" --timeout 450 --skip-clean-build
fi

echo "Done."
