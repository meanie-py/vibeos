#!/bin/bash
# Cargo-driven rosload.exe build PoC — i386 + amd64 (arm64 has no rosload:
# UEFI-only boot).
#
# Usage: build-and-test.sh [i386|amd64]   (default i386)
#
# 1. Run cmake/ninja once so the prebuilt rosload.dir objects + sibling
#    static libs + libfreeldr_rust_rosload.a + the freeldr import lib exist
#    (cmake does the heavy lifting; cargo just drives the final link).
# 2. Drive the cargo build of `rosload-cargo` against the custom per-arch
#    kernel target json. build.rs links the cmake-built inputs via
#    reactos-link's ModuleKind::NativeExe.
# 3. rosload.exe needs no post-build (cmake POST_BUILD = `:`), so the cargo
#    binary is used as-is.
# 4. (Opt-in $RUN_SMOKE=1) swap the cargo-built rosload.exe into
#    build-<arch>/, rebuild the LiveCD, and run smoke. Restores the cmake
#    binary on exit.
#
# Run from anywhere (cd's to repo root).

set -euo pipefail

ARCH="${1:-i386}"
case "$ARCH" in
  i386)  SMOKE_ARCH="i386-uefi" ;;
  amd64) SMOKE_ARCH="amd64-uefi" ;;
  *) echo "usage: $0 [i386|amd64]  (arm64 is UEFI-only, no rosload)" >&2; exit 1 ;;
esac

cd "$(dirname "$0")/../../../.."  # → repo root

# Shared cargo-invocation policy (builtin triple on amd64, nightly + build-std
# on i386). Sets CELL_TARGET / CELL_STEM / CELL_STD / CELL_TOOLCHAIN.
VIBEOS_ROOT="$PWD" . sdk/lib/rust/cargo-cell-args.sh
cell_args "$ARCH" rosload-cargo || exit 1

REPO="$(pwd)"
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
echo "[1/3] Refreshing cmake-built inputs (ninja rosload, $ARCH) ..."
( cd "$BUILD_DIR" && ninja rosload >/dev/null )

# ---- 2. Cargo-driven native link ----------------------------------------
echo "[2/3] Cargo build (rosload-cargo, $ARCH) ..."
cd sdk/lib/rust
# shellcheck disable=SC2086  # CELL_TOOLCHAIN/CELL_STD are deliberately word-split
cargo $CELL_TOOLCHAIN build --release \
    --target "$CELL_TARGET" \
    $CELL_STD \
    -p rosload-cargo --bin rosload

CARGO_EXE="$(pwd)/target/$CELL_STEM/release/rosload.exe"
echo "    produced: $CARGO_EXE ($(stat -c %s "$CARGO_EXE") bytes)"

# ---- 3. Swap into livecd + smoke (opt-in via $RUN_SMOKE=1) ---------------
if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd "$REPO"
  TGT="$BUILD_DIR/boot/freeldr/freeldr/rosload.exe"
  echo "[3/3] Swapping cargo rosload.exe into $BUILD_DIR/ + smoke ..."
  cp "$TGT" "$TGT.cmake-backup"
  cp "$CARGO_EXE" "$TGT"
  ( cd "$BUILD_DIR" && ninja livecd >/dev/null )
  trap 'mv "'"$REPO"'/'"$TGT"'.cmake-backup" "'"$REPO"'/'"$TGT"'"; (cd "'"$REPO"'/'"$BUILD_DIR"'" && ninja livecd >/dev/null) || true' EXIT
  /home/curi/Projects/os-playground/vibeos/.venv/bin/python -m pytest \
      tests/qemu/test_smoke.py --arch "$SMOKE_ARCH" --timeout 450 --skip-clean-build
fi

echo "Done."
