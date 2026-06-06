#!/bin/bash
# Boot-swap smoke for the cargo-driven ntoskrnl.exe. Proves RUNTIME
# equivalence, not just a clean relink: it builds the cargo ntoskrnl.exe,
# applies the kernel post-link (native-pefixup --kernel + the i386 lld forwarder
# fix — exactly what set_module_type(ntoskrnl kernel) applies, which the
# standalone cargo link does NOT), swaps it into the cmake tree, refreshes the
# boot media (livecd for x86, ramdisk for arm64), and boots via
# `vibeos-cli smoke --allow-stale`.
#
# Usage: boot-swap-smoke.sh [arch]   (arch = i386 | amd64 | arm64; default i386)
#
# Mechanism: overwrite the cmake output with the (post-linked) cargo binary,
# assert `ninja livecd` packs OUR bytes (didn't relink the cmake exe over them),
# boot --allow-stale (the swap is a deliberate out-of-band media mutation), and
# ALWAYS restore the cmake exe on exit (trap).

set -euo pipefail
cd "$(dirname "$0")/../../../.."  # → repo root

ARCH="${1:-i386}"
case "$ARCH" in
  i386)  TARGET_ARG=ntoskrnl-cargo/i686-kernel.json; TARGET_STEM=i686-kernel; BUILD_STD=1; BUILD_STD_MEM=1 ;;
  amd64) TARGET_ARG=ntoskrnl-cargo/x86_64-kernel.json; TARGET_STEM=x86_64-kernel; BUILD_STD=1; BUILD_STD_MEM=1 ;;
  arm64) TARGET_ARG=ntoskrnl-cargo/aarch64-kernel.json; TARGET_STEM=aarch64-kernel; BUILD_STD=1; BUILD_STD_MEM=0 ;;
  *) echo "unsupported arch '$ARCH' (i386|amd64|arm64)" >&2; exit 2 ;;
esac

BUILD=build-$ARCH
EXE=$BUILD/ntoskrnl/ntoskrnl.exe
CARGO_OUT=sdk/lib/rust/target/$TARGET_STEM/release/ntoskrnl.exe
BAK=$EXE.cmake-bak
PEFIXUP=$BUILD/host-tools/bin/pefixup
FIXFWD=sdk/tools/fix_lld_forwarding.py
REFRESH_MEDIA=livecd
if [ "$ARCH" = "arm64" ]; then
  REFRESH_MEDIA=ramdisk
fi

[ -d "$BUILD" ] || { echo "$BUILD/ missing — run cmake first." >&2; exit 1; }
if [ "$BUILD_STD" = "1" ]; then
  rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required." >&2; exit 1; }
fi

export CARGO_TARGET_I686_KERNEL_LINKER="$PWD/.build-tools/llvm-mingw/bin/i686-w64-mingw32-clang"
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="$PWD/.build-tools/llvm-mingw/bin/x86_64-w64-mingw32-clang"
export CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER="$PWD/.build-tools/llvm-mingw/bin/aarch64-w64-mingw32-clang"
export CARGO_TARGET_AARCH64_KERNEL_LINKER="$PWD/.build-tools/llvm-mingw/bin/aarch64-w64-mingw32-clang"

restore() {
  if [ -f "$BAK" ]; then
    mv -f "$BAK" "$EXE"
    echo "[restore] cmake ntoskrnl.exe restored ($ARCH)"
    if [ "$REFRESH_MEDIA" = "livecd" ]; then
      ( cd "$BUILD" && ninja livecd >/dev/null 2>&1 ) || true
    else
      bash qemu/mkramdisk.sh --refresh >/dev/null 2>&1 || true
    fi
  fi
}
trap restore EXIT

echo "[1/5] cmake baseline (ninja ntoskrnl → sibling .a + cmake exe) [$ARCH] ..."
( cd "$BUILD" && ninja ntoskrnl >/dev/null )

echo "[2/5] Cargo build (ntoskrnl-cargo, $TARGET_STEM) ..."
if [ "$BUILD_STD" = "1" ]; then
  build_std_args=(-Z build-std=core,alloc,compiler_builtins)
  if [ "$BUILD_STD_MEM" = "1" ]; then
    build_std_args+=(-Z build-std-features=compiler-builtins-mem)
  fi
  ( cd sdk/lib/rust && cargo +nightly build --release \
      --target "$TARGET_ARG" \
      "${build_std_args[@]}" \
      -p ntoskrnl-cargo --bin ntoskrnl )
else
  ( cd sdk/lib/rust && rustup run stable cargo build --release \
      --target "$TARGET_ARG" \
      -p ntoskrnl-cargo --bin ntoskrnl )
fi
test "$(head -c2 "$CARGO_OUT")" = "MZ" || { echo "FAIL: cargo ntoskrnl not a PE"; exit 1; }

echo "[3/5] Swap + kernel post-link (native-pefixup --kernel + i386 fwd fix) ..."
cp -f "$EXE" "$BAK"
cp -f "$CARGO_OUT" "$EXE"
"$PEFIXUP" --kernel "$EXE"
# The lld forwarder `_`-prefix bug is i386-only; the fix is a no-op elsewhere
# but gate it so amd64 doesn't depend on the i386 quirk.
[ "$ARCH" = "i386" ] && python3 "$FIXFWD" "$EXE"
swapped_sha=$(sha256sum "$EXE" | cut -d' ' -f1)
if [ "$REFRESH_MEDIA" = "livecd" ]; then
  ( cd "$BUILD" && ninja livecd >/dev/null )
else
  bash qemu/mkramdisk.sh --refresh >/dev/null
fi

# Assert the media refresh kept OUR (post-linked cargo) bytes in the build tree
# and didn't relink the cmake exe over them. If it relinked, the sha changes and
# the cargo code never boots.
if [ "$(sha256sum "$EXE" | cut -d' ' -f1)" != "$swapped_sha" ]; then
  echo "FAIL: media refresh relinked ntoskrnl.exe — swap was overwritten (cargo bytes not booted)." >&2
  exit 1
fi
echo "    confirmed: $REFRESH_MEDIA carries the cargo-built ntoskrnl.exe ($(stat -c %s "$EXE") bytes, $ARCH)"

echo "[4/5] Booting the swapped livecd (smoke verdict) ..."
rc=0
./vibeos-cli smoke --arch "$ARCH" --allow-stale || rc=$?

if [ "$rc" -eq 0 ]; then
  echo "[5/5] PASS — cargo-driven ntoskrnl.exe booted to the smoke marker ($ARCH)."
else
  echo "[5/5] FAIL — smoke verdict nonzero ($rc) with the cargo ntoskrnl.exe ($ARCH)."
fi
exit "$rc"
