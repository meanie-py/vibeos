#!/bin/bash
# Cargo-driven kdcom.dll build PoC — i386 only. KD-COM sibling of
# hal-cargo/build-and-test.sh.
#
# NOTE: as of the cmake→cargo wiring, the MAIN build already drives this
# link (drivers/base/kdrosdbg/CMakeLists.txt → add_cargo_kernel_dll), so a
# plain `ninja kdcom` in build-<arch>/ produces the shipping cargo-linked
# kdcom.dll. This script remains as an isolated i386 link+swap-smoke helper.
#
# On the llvm-mingw (non-MSVC) build, kdcom.dll is built from the
# 116-LOC stub drivers/base/kdrosdbg/kdrosdbg.c (NOT drivers/base/kdcom;
# see memory feedback_kdcom_is_kdrosdbg). The cmake target is *named*
# `kdcom` and emits build-i386/drivers/base/kdrosdbg/kdcom.dll.
#
# 1. Run ninja kdcom once so the sibling static libs (cportlib, memcmp)
#    + the ntoskrnl import lib exist (cargo drives only the final link).
# 2. Drive the cargo build of `kdcom-cargo` against the custom
#    i686-kernel.json target. build.rs builds `kdcom-rust` as a sub-step.
# 3. (Optional, $RUN_SMOKE=1) swap the cargo-built kdcom.dll into
#    build-i386/, rebuild the LiveCD, run the i386 smoke tests, restore.
#
# kdrosdbg.spec has NO forwarder exports (unlike hal.spec's Ex*FastMutex
# → ntoskrnl), so the cmake post-link for kdcom.dll is pefixup only —
# no fix_lld_forwarding.py.
#
# Run from the repo root or anywhere — script cd's to the repo root.

set -euo pipefail

cd "$(dirname "$0")/../../../.."  # → repo root

# Pinned nightly, single-sourced from the repo root (see rust-toolchain.toml).
VIBEOS_NIGHTLY="$(cat "$PWD/.rust-nightly")"

if [ ! -d build-i386 ]; then
  echo "build-i386/ does not exist — run cmake first." >&2
  exit 1
fi
if ! rustup toolchain list 2>&1 | grep -q nightly; then
  echo "nightly toolchain required for build-std." >&2
  exit 1
fi

KDCOM_DLL="build-i386/drivers/base/kdrosdbg/kdcom.dll"

echo "[1/3] Refreshing cmake-built inputs (ninja kdcom) ..."
( cd build-i386 && ninja kdcom >/dev/null )

echo "[2/3] Cargo build (kdcom-rust + kdcom-cargo) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target kdcom-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p kdcom-cargo --bin kdcom

# cargo's [[bin]] uses the target's exe-suffix (.exe); `-shared` makes
# it a PE *DLL* despite the name.
CARGO_OUT="$(pwd)/target/i686-kernel/release/kdcom.exe"
echo "    produced: $CARGO_OUT ($(stat -c %s "$CARGO_OUT") bytes, $(file -b "$CARGO_OUT"))"

if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd ../../..
  echo "[3/3] Swapping into $KDCOM_DLL + post-link + smoke ..."
  cp "$KDCOM_DLL" "$KDCOM_DLL.cmake-backup"
  cp "$CARGO_OUT" "$KDCOM_DLL"
  # Same post-link cmake applies to kdcom.dll: kernel-DLL PE fixup.
  # (No forwarder fix — kdrosdbg.spec has none.) pefixup is
  # self-compiled from source with the host cc — no cmake host tool.
  PEFIXUP="$(mktemp -d)/pefixup"
  cc -O2 -DTARGET_i386 -D_CRT_NON_CONFORMING_SWPRINTFS -D__REACTOS__ \
     -I sdk/include/host -I sdk/tools -o "$PEFIXUP" sdk/tools/pefixup.c
  "$PEFIXUP" --kerneldll "$KDCOM_DLL"
  ( cd build-i386 && ninja livecd >/dev/null )
  source .venv/bin/activate
  trap 'mv "$KDCOM_DLL.cmake-backup" "$KDCOM_DLL"; (cd build-i386 && ninja livecd >/dev/null) || true' EXIT
  pytest tests/qemu/test_smoke.py --arch i386-uefi --timeout 400 --skip-clean-build
fi

echo "Done."
