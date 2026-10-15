# shellcheck shell=sh
# Shared cargo-invocation policy for the `*-cargo` link cells.
#
# THE POLICY (stable-toolchain-plan.md, Axis B):
#   amd64 / arm64 → the BUILTIN triple on pinned STABLE. Precompiled
#                   core/alloc/compiler_builtins from the rustup `rust-std-*`
#                   component; NO `-Z build-std`, NO custom target json. Two
#                   `-C` flags restore what the retired json used to carry
#                   (`link-self-contained=no`, `force-frame-pointers=yes`); the
#                   linker is pinned in sdk/lib/rust/.cargo/config.toml.
#   i386 user     → (KIND win32dll / win32exe, or a kernel kind whose crate has no
#                   i686-kernel.json: the generated drivers) the builtin
#                   `i686-pc-windows-gnu` on stable, like amd64, plus the bodies'
#                   `-sse,-sse2`.
#   i386 kernel   → the custom `i686-kernel.json` + nightly `-Z build-std`. It
#                   needs `rustc-abi: x86-softfloat`, which no builtin triple
#                   offers. Retired by the wow64 lane, not fixed here.
#
# `gcc.cmake:add_cargo_module` is the PRODUCTION copy of this policy (cmake
# cannot source shell), and tests/qemu/cli/lints.py + tools/cargo_orchestrate.py
# hold the Python copies. This file exists so the six dev helper scripts don't
# make a seventh and an eighth. Change one → change all five.
#
# Usage (from the repo root, before `cd sdk/lib/rust`):
#     . sdk/lib/rust/cargo-cell-args.sh
#     cell_args "$arch" ntdll-cargo win32dll    # KIND as add_cargo_module has it
#     cargo $CELL_TOOLCHAIN build --release --target "$CELL_TARGET" $CELL_STD \
#         -p ntdll-cargo --bin ntdll
#     out="sdk/lib/rust/target/$CELL_STEM/release/ntdll.exe"
#
# Sets: CELL_TARGET (--target value)  CELL_STEM (cargo's target/<stem>/ subdir)
#       CELL_STD (the -Z build-std args, or empty)  CELL_TOOLCHAIN (+<nightly>, or empty)
#       and EXPORTS RUSTFLAGS on the stable arches.

cell_args() {
    _arch="$1"
    _crate="$2"
    _kind="${3:-kerneldll}"
    if [ "$_arch" = i386 ] && { [ "$_kind" = win32dll ] || [ "$_kind" = win32exe ] ||
        [ ! -e "${VIBEOS_ROOT:-.}/sdk/lib/rust/$_crate/i686-kernel.json" ]; }; then
        _arch=i386-user
    fi
    case "$_arch" in
        i386-user)
            CELL_STEM="i686-pc-windows-gnu"
            CELL_TARGET="$CELL_STEM"
            CELL_STD=""
            CELL_TOOLCHAIN=""
            RUSTFLAGS="-C force-frame-pointers=yes -C link-self-contained=no -C target-feature=-sse,-sse2"
            export RUSTFLAGS
            ;;
        i386)
            CELL_STEM="i686-kernel"
            CELL_TARGET="$_crate/i686-kernel.json"
            CELL_STD="-Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem"
            # Pinned nightly, single-sourced from the repo root.
            CELL_TOOLCHAIN="+$(cat "${VIBEOS_ROOT:-.}/.rust-nightly")"
            ;;
        amd64)
            CELL_STEM="x86_64-pc-windows-gnu"
            CELL_TARGET="$CELL_STEM"
            CELL_STD=""
            CELL_TOOLCHAIN=""
            # link-self-contained kills x86_64-pc-windows-gnu's user-mode CRT startup
            # objects (crt2.o/dllcrt2.o). aarch64-pc-windows-gnullvm REJECTS the flag
            # ("not supported on this target") and needs no equivalent.
            RUSTFLAGS="-C force-frame-pointers=yes -C link-self-contained=no"
            export RUSTFLAGS
            ;;
        arm64)
            CELL_STEM="aarch64-pc-windows-gnullvm"
            CELL_TARGET="$CELL_STEM"
            CELL_STD=""
            CELL_TOOLCHAIN=""
            RUSTFLAGS="-C force-frame-pointers=yes"
            export RUSTFLAGS
            ;;
        *)
            echo "cell_args: unsupported arch '$_arch' (i386|amd64|arm64)" >&2
            return 2
            ;;
    esac
}
