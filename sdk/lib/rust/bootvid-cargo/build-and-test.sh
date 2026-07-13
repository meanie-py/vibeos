#!/bin/bash
# Cargo-driven bootvid.dll build PoC — i386 only. Boot-video sibling of
# hal-cargo / kdcom-cargo build-and-test.sh.
#
# NOTE: as of the cmake→cargo wiring, the MAIN build already drives this
# link (drivers/base/bootvid/CMakeLists.txt → add_cargo_kernel_dll), so a
# plain `ninja bootvid` in build-<arch>/ produces the shipping cargo-linked
# bootvid.dll. This script remains as an isolated i386 link+swap-smoke helper.
#
# On the non-MSVC i386/amd64 build, bootvid.dll is the i386/pc VGA path
# (i386/pc/{bootvid,bootdata,vga}.c + common/console/fontdata.c); the
# framebuf/ GOP path is the separate lfbbvid.dll. cmake target name is
# `bootvid` → build-i386/drivers/base/bootvid/bootvid.dll.
#
# 1. ninja bootvid once so the leaf static lib (memcmp) + the ntoskrnl
#    & hal import libs / spec2def host tool exist.
# 2. cargo build `bootvid-cargo` against i686-kernel.json. build.rs
#    builds `bootvid-rust` as a sub-step.
# 3. (Optional, $RUN_SMOKE=1) swap the cargo bootvid.dll into
#    build-i386/, pefixup --kerneldll, ninja livecd, run i386 smoke,
#    restore.
#
# bootvid.spec has NO forwarder exports, so post-link is pefixup only
# (no fix_lld_forwarding.py), same as kdcom.
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

BOOTVID_DLL="build-i386/drivers/base/bootvid/bootvid.dll"

echo "[1/3] Refreshing cmake-built inputs (ninja bootvid) ..."
( cd build-i386 && ninja bootvid >/dev/null )

echo "[2/3] Cargo build (bootvid-rust + bootvid-cargo) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target bootvid-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p bootvid-cargo --bin bootvid

# cargo's [[bin]] uses the target's exe-suffix (.exe); `-shared` makes
# it a PE *DLL* despite the name.
CARGO_OUT="$(pwd)/target/i686-kernel/release/bootvid.exe"
echo "    produced: $CARGO_OUT ($(stat -c %s "$CARGO_OUT") bytes, $(file -b "$CARGO_OUT"))"

if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd ../../..
  echo "[3/3] Swapping into $BOOTVID_DLL + post-link + smoke ..."
  cp "$BOOTVID_DLL" "$BOOTVID_DLL.cmake-backup"
  cp "$CARGO_OUT" "$BOOTVID_DLL"
  PEFIXUP="$(mktemp -d)/pefixup"
  cc -O2 -DTARGET_i386 -D_CRT_NON_CONFORMING_SWPRINTFS -D__REACTOS__ \
     -I sdk/include/host -I sdk/tools -o "$PEFIXUP" sdk/tools/pefixup.c
  "$PEFIXUP" --kerneldll "$BOOTVID_DLL"
  ( cd build-i386 && ninja livecd >/dev/null )
  source .venv/bin/activate
  trap 'mv "$BOOTVID_DLL.cmake-backup" "$BOOTVID_DLL"; (cd build-i386 && ninja livecd >/dev/null) || true' EXIT
  pytest tests/qemu/test_smoke.py --arch i386-uefi --timeout 400 --skip-clean-build
fi

echo "Done."
