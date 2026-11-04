#!/usr/bin/env bash
# Install the PINNED, PATCHED c2rust that rustconv's anchor pass requires.
#
# Why: stock c2rust 0.22.1 (the latest release; upstream master is unfixed as of
# 2026-10-26) panics whenever C takes the address of a __stdcall function — every COM
# vtable and every WndProc. See docs/engineering/c2rust-upstream-bugs.md §1.
# attributed-knr.patch is the one-line fix. rustconv refuses an unpatched binary
# (c2rust_anchor probes it with repro-stdcall.c), so run this once per machine:
#
#   bash tools/c2rust/install.sh
#
# Idempotent: re-running reinstalls the same bits. Installs to ~/.cargo/bin like a plain
# `cargo install c2rust` would, so nothing else needs to know where it lives.
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
VERSION=0.22.1
CRATE=c2rust-transpile
SHA256=a03afc6ca93f6b559d83ef564549ff48e2ff37b90bf9a7363bf50076ce401616
CACHE=${XDG_CACHE_HOME:-$HOME/.cache}/vibeos-c2rust
PATCH_ID=$(sha256sum "$HERE/attributed-knr.patch" | cut -c1-12)
SRC=$CACHE/$CRATE-$VERSION-$PATCH_ID

mkdir -p "$CACHE"
TARBALL=$CACHE/$CRATE-$VERSION.crate
if [[ ! -f $TARBALL ]] || ! echo "$SHA256  $TARBALL" | sha256sum -c --quiet - 2>/dev/null; then
    curl -fsSL -o "$TARBALL.part" "https://static.crates.io/crates/$CRATE/$CRATE-$VERSION.crate"
    mv "$TARBALL.part" "$TARBALL"
fi
echo "$SHA256  $TARBALL" | sha256sum -c --quiet - || {
    echo "install.sh: $TARBALL does not match the pinned sha256 — refusing to build it" >&2
    exit 1
}

if [[ ! -f $SRC/.vibeos-patched ]]; then
    rm -rf "$SRC" "$SRC.tmp"
    mkdir -p "$SRC.tmp"
    tar -xzf "$TARBALL" -C "$SRC.tmp" --strip-components=1
    patch -d "$SRC.tmp" -p1 --forward --quiet < "$HERE/attributed-knr.patch"
    touch "$SRC.tmp/.vibeos-patched"
    mv "$SRC.tmp" "$SRC"
fi

# --locked keeps every other dependency at the versions 0.22.1 shipped with; the [patch]
# swaps in only our c2rust-transpile (same version, so the lockfile still resolves).
cargo install c2rust --version "=$VERSION" --locked --force \
    --config "patch.crates-io.$CRATE.path=\"$SRC\""

# Prove it, by the route rustconv will: transpile the reproducer (stock panics on it).
PROBE=$(mktemp -d)
trap 'rm -rf "$PROBE"' EXIT
cp "$HERE/repro-stdcall.c" "$PROBE/repro.c"
"$HOME/.cargo/bin/c2rust" transpile --output-dir "$PROBE/out" "$PROBE/repro.c" \
    -- --target=i686-w64-mingw32 > "$PROBE/log" 2>&1 || true
if [[ ! -f $PROBE/out/src/repro.rs ]]; then
    cat "$PROBE/log" >&2
    echo "install.sh: the installed c2rust still fails the __stdcall reproducer" >&2
    exit 1
fi
[[ $(command -v c2rust) == "$HOME/.cargo/bin/c2rust" ]] ||
    echo "install.sh: WARNING: PATH resolves c2rust to $(command -v c2rust), not ~/.cargo/bin" >&2
echo "install.sh: patched c2rust $VERSION installed and verified"
