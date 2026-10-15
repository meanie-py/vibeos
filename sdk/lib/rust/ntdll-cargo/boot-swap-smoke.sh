#!/bin/bash
# Boot-swap smoke for the cargo-driven ntdll DLLs — proves RUNTIME equivalence,
# not just PE shape. build-and-test.sh only checks export/header parity; this
# drops the cargo-built DLL into the real livecd and boots it to `shell ready`
# (the full kernel → SMSS → csrss → win32k → winlogon → explorer → desktop
# chain — all of which depends on ntdll / ntdll_vista).
#
# Usage: boot-swap-smoke.sh [i386|amd64] [ntdll|ntdll_vista]
#        (default: i386 ntdll; x86 livecd only — arm64 has no livecd)
#
# Mechanism: cargo builds the DLL, we overwrite the cmake output with it
# (making it the newest file so `ninja livecd` re-rolls the iso WITHOUT
# relinking), assert ninja kept our bytes, then run the normal smoke verdict
# with --allow-stale (the swap is a deliberate out-of-band media mutation the
# staleness preflight is built to catch). The cmake DLL is backed up and
# ALWAYS restored on exit (trap), leaving the tree as cmake built it.

set -euo pipefail
cd "$(dirname "$0")/../../../.."  # → repo root

arch="${1:-i386}"
module="${2:-ntdll}"
case "$arch" in
  i386|amd64) ;;
  *) echo "usage: $0 [i386|amd64] [ntdll|ntdll_vista]  (arm64 has no livecd)" >&2; exit 2 ;;
esac
case "$module" in
  ntdll)       crate="ntdll-cargo";       dll="build-$arch/dll/ntdll/ntdll.dll" ;;
  ntdll_vista) crate="ntdll_vista-cargo"; dll="build-$arch/dll/ntdll/nt_0600/ntdll_vista.dll" ;;
  *) echo "module must be ntdll or ntdll_vista" >&2; exit 2 ;;
esac
# Shared cargo-invocation policy (ntdll is a user-mode cell: the builtin triple,
# stable, on every arch). Sets CELL_TARGET / CELL_STEM / CELL_STD / CELL_TOOLCHAIN.
VIBEOS_ROOT="$PWD" . sdk/lib/rust/cargo-cell-args.sh
cell_args "$arch" "$crate" win32dll || exit 2

BUILD="build-$arch"
CARGO_OUT="sdk/lib/rust/target/$CELL_STEM/release/$module.exe"
BAK="$dll.cmake-bak"

[ -d "$BUILD" ] || { echo "$BUILD/ missing — run cmake first." >&2; exit 1; }
rustup toolchain list 2>&1 | grep -q nightly || { echo "nightly + rust-src required." >&2; exit 1; }

restore() {
  if [ -f "$BAK" ]; then
    mv -f "$BAK" "$dll"
    echo "[restore] cmake $module restored ($dll)"
    # Re-roll the iso back to the cmake DLL so the tree's media is honest.
    ( cd "$BUILD" && ninja livecd >/dev/null 2>&1 ) || true
  fi
}
trap restore EXIT

echo "[1/5] cmake baseline ($arch $module → sibling .a + cmake DLL) ..."
( cd "$BUILD" && ninja "$module" >/dev/null )

echo "[2/5] Cargo build ($crate, $CELL_STEM) ..."
# shellcheck disable=SC2086  # CELL_TOOLCHAIN/CELL_STD are deliberately word-split
( cd sdk/lib/rust && cargo $CELL_TOOLCHAIN build --release \
    --target "$CELL_TARGET" \
    $CELL_STD \
    -p "$crate" --bin "$module" )
test "$(head -c2 "$CARGO_OUT")" = "MZ" || { echo "FAIL: cargo $module not a PE"; exit 1; }

echo "[3/5] Swapping cargo $module into the cmake tree + re-rolling iso ..."
cp -f "$dll" "$BAK"
cp -f "$CARGO_OUT" "$dll"   # now newest → ninja won't relink, livecd repacks it
( cd "$BUILD" && ninja livecd >/dev/null )

# Assert ninja packed OUR bytes (didn't relink the cmake DLL over them).
if ! cmp -s "$CARGO_OUT" "$dll"; then
  echo "FAIL: ninja relinked the cmake $module — swap was overwritten." >&2
  echo "  (livecd's DAG decided $module was out of date; cargo bytes not booted.)" >&2
  exit 1
fi
echo "    confirmed: livecd carries the cargo-built $module ($(stat -c %s "$dll") bytes)"

echo "[4/5] Booting the swapped livecd (smoke verdict) ..."
# --allow-stale is REQUIRED: we deliberately mutated the boot media out of band
# (the "rebuilt out of band?" condition staleness is built to catch). We already
# cmp-asserted the iso carries the cargo bytes; only thing left is that they BOOT.
rc=0
./vibeos-cli smoke --arch "$arch" --allow-stale || rc=$?

if [ "$rc" -eq 0 ]; then
  echo "[5/5] PASS — cargo-driven $module booted to the smoke marker ($arch)."
else
  echo "[5/5] FAIL — smoke verdict nonzero ($rc) with the cargo $module ($arch)."
fi
exit "$rc"
