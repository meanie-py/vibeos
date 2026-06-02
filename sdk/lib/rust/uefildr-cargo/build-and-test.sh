#!/bin/bash
# Cargo-driven uefildr.efi build PoC — i386 only.
#
# 1. Run cmake/ninja once so the prebuilt strsafe_shim.c.obj + sibling
#    static libs + libfreeldr_rust_uefi.a exist (cmake does the heavy
#    lifting; cargo just drives the final EFI link).
# 2. Drive the cargo build of `uefildr-cargo` against the custom
#    `i686-kernel.json` target (softfloat, no SSE — freeldr runs before
#    CR4.OSFXSR). build.rs links the cmake-built inputs via reactos-link's
#    ModuleKind::EfiApplication.
# 3. Post-build: rename uefildr.exe -> uefildr.efi and strip .rossym +
#    all symbols (matches the cmake POST_BUILD).
# 4. (Opt-in $RUN_SMOKE=1) swap the cargo-built uefildr.efi into
#    build-i386/, rebuild the LiveCD (re-stamps efisys.bin with it), and
#    run the i386 smoke. Restores the cmake binary on exit.
#
# Run from anywhere (cd's to repo root).

set -euo pipefail

cd "$(dirname "$0")/../../../.."  # → repo root
REPO="$(pwd)"
STRIP="$REPO/.build-tools/llvm-mingw/bin/llvm-strip"

# ---- 0. Sanity checks -----------------------------------------------------
if [ ! -d build-i386 ]; then
  echo "build-i386/ does not exist — run cmake first." >&2
  exit 1
fi
if ! rustup toolchain list 2>&1 | grep -q nightly; then
  echo "nightly toolchain required for build-std." >&2
  exit 1
fi

# ---- 1. cmake/ninja produces the prebuilt inputs -------------------------
echo "[1/4] Refreshing cmake-built inputs (ninja uefildr) ..."
( cd build-i386 && ninja uefildr >/dev/null )

# ---- 2. Cargo-driven EFI link -------------------------------------------
echo "[2/4] Cargo build (uefildr-cargo) ..."
cd sdk/lib/rust
cargo +nightly build --release \
    --target uefildr-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p uefildr-cargo --bin uefildr

CARGO_EXE="$(pwd)/target/i686-kernel/release/uefildr.exe"
CARGO_EFI="$(pwd)/target/i686-kernel/release/uefildr.efi"

# ---- 3. Post-build: rename + strip (matches cmake POST_BUILD) ------------
echo "[3/4] Strip + rename -> uefildr.efi ..."
cp "$CARGO_EXE" "$CARGO_EFI"
"$STRIP" --remove-section=.rossym "$CARGO_EFI"
"$STRIP" --strip-all "$CARGO_EFI"
echo "    produced: $CARGO_EFI ($(stat -c %s "$CARGO_EFI") bytes)"

# ---- 4. Swap into livecd + smoke (opt-in via $RUN_SMOKE=1) ---------------
if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd "$REPO"
  TGT="build-i386/boot/freeldr/freeldr/uefildr.efi"
  echo "[4/4] Swapping cargo uefildr.efi into build-i386/ + smoke ..."
  cp "$TGT" "$TGT.cmake-backup"
  cp "$CARGO_EFI" "$TGT"
  ( cd build-i386 && ninja livecd >/dev/null )
  trap 'mv "'"$REPO"'/'"$TGT"'.cmake-backup" "'"$REPO"'/'"$TGT"'"; (cd "'"$REPO"'/build-i386" && ninja livecd >/dev/null) || true' EXIT
  /home/curi/Projects/os-playground/vibeos/.venv/bin/python -m pytest \
      tests/qemu/test_smoke.py --arch i386-uefi --timeout 450 --skip-clean-build
fi

echo "Done."
