#!/bin/bash
# Cargo-driven hal.dll build PoC — i386 only.
#
# 1. Run cmake/ninja once so the sibling static libs + ntoskrnl import
#    lib exist (cargo drives only the final hal.dll link).
# 2. Drive the cargo build of `hal-cargo` against the custom
#    `i686-kernel.json` target. build.rs builds `hal-rust` as a sub-step.
# 3. (Optional, $RUN_SMOKE=1) swap the cargo-built hal.dll into
#    build-i386/, rebuild the LiveCD, run the i386 smoke tests, restore.
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

echo "[1/3] Refreshing cmake-built inputs (ninja hal) ..."
( cd build-i386 && ninja hal >/dev/null )

echo "[2/3] Cargo build (hal-rust + hal-cargo) ..."
cd sdk/lib/rust
cargo "+${VIBEOS_NIGHTLY}" build --release \
    --target hal-cargo/i686-kernel.json \
    -Z build-std=core,alloc,compiler_builtins \
    -Z build-std-features=compiler-builtins-mem \
    -p hal-cargo --bin hal

# cargo's [[bin]] uses the target's exe-suffix (.exe); `-shared` makes
# it a PE *DLL* despite the name.
CARGO_OUT="$(pwd)/target/i686-kernel/release/hal.exe"
echo "    produced: $CARGO_OUT ($(stat -c %s "$CARGO_OUT") bytes, $(file -b "$CARGO_OUT"))"

if [ "${RUN_SMOKE:-0}" = "1" ]; then
  cd ../../..
  echo "[3/3] Swapping into build-i386/hal/halx86/ + post-link + smoke ..."
  HALDLL=build-i386/hal/halx86/hal.dll
  BAK=$HALDLL.cmake-backup
  # Restore the cmake hal.dll on ANY exit (set the trap BEFORE the swap so an
  # early failure can't leave the cargo binary dangling in the build tree).
  trap 'if [ -f "$BAK" ]; then mv -f "$BAK" "$HALDLL"; (cd build-i386 && ninja livecd >/dev/null 2>&1) || true; fi' EXIT
  cp "$HALDLL" "$BAK"
  cp "$CARGO_OUT" "$HALDLL"
  # Same post-link chain cmake applies to hal.dll: kernel-DLL PE fixup
  # then strip LLD's spurious i386 `_` on forwarder export targets
  # (hal.spec forwards Ex*FastMutex → ntoskrnl.Exi*). pefixup is
  # self-compiled from source with the host cc — no cmake host tool.
  PEFIXUP="$(mktemp -d)/pefixup"
  cc -O2 -DTARGET_i386 -D_CRT_NON_CONFORMING_SWPRINTFS -D__REACTOS__ \
     -I sdk/include/host -I sdk/tools -o "$PEFIXUP" sdk/tools/pefixup.c
  "$PEFIXUP" --kerneldll "$HALDLL"
  python3 sdk/tools/fix_lld_forwarding.py "$HALDLL"
  swapped_sha=$(sha256sum "$HALDLL" | cut -d' ' -f1)
  ( cd build-i386 && ninja livecd >/dev/null )
  # Assert ninja packed OUR (post-linked cargo) bytes — didn't relink the cmake
  # hal.dll over them. If it relinked, the sha changes and the cargo hal never
  # boots (false green). Mirrors the ntoskrnl-cargo boot-swap assert.
  if [ "$(sha256sum "$HALDLL" | cut -d' ' -f1)" != "$swapped_sha" ]; then
    echo "FAIL: ninja relinked hal.dll — swap was overwritten (cargo bytes not booted)." >&2
    exit 1
  fi
  echo "    confirmed: livecd carries the cargo-built hal.dll ($(stat -c %s "$HALDLL") bytes)"
  # Boot the swapped livecd via the front door (worktree-portable; no .venv).
  # --allow-stale: the swap is a deliberate out-of-band media mutation.
  ./vibeos-cli smoke --arch i386 --allow-stale
fi

echo "Done."
