#!/bin/bash
# Generate the FreeType FFI surface (FT_*/TT_*) for all 3 architectures.
# FreeType is self-contained portable C (no build-version coupling), so unlike
# win32k-sys we use a fixed flag set rather than deriving from build.ninja.
#
# Prerequisites: cargo install bindgen-cli.  Run from anywhere.
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
RES="${TOOLCHAIN}/lib/clang/19/include"
FT_INC="${SRC}/sdk/lib/3rdparty/freetype/include"
WRAPPER="${SCRIPT_DIR}/wrapper.h"
OUT_DIR="${SCRIPT_DIR}/src"

run_bindgen() {
    local ARCH=$1 TARGET=$2 MINGW=$3
    local FINAL="${OUT_DIR}/bindgen_${ARCH}.rs"
    # llvm-mingw per-target libc headers (string.h etc.) — arm64 has no system
    # mingw to fall back on, so point clang at the toolchain sysroot explicitly.
    local SYSINC="${TOOLCHAIN}/${MINGW}/include"
    echo "=== Generating ${ARCH} (target: ${TARGET}) ==="
    bindgen "$WRAPPER" \
        --use-core --no-layout-tests --no-doc-comments \
        --allowlist-type '(FT_|TT_|PS_|FT2_).*' \
        --allowlist-function '(FT_|TT_).*' \
        --allowlist-var '(FT_|TT_|ft_).*' \
        -o "$FINAL" \
        -- -ferror-limit=0 -Wno-everything -DFT2_BUILD_LIBRARY \
           -I "$FT_INC" -isystem "$SYSINC" -isystem "$RES" --target="${TARGET}"
    echo "    $(grep -cE 'pub fn FT_|pub fn TT_' "$FINAL") functions, $(grep -cE 'pub (struct|type|enum)' "$FINAL") types"
}

run_bindgen i386  i686-pc-windows-gnu    i686-w64-mingw32
run_bindgen amd64 x86_64-pc-windows-gnu  x86_64-w64-mingw32
run_bindgen arm64 aarch64-pc-windows-gnu aarch64-w64-mingw32
echo "=== Done ==="; ls -la "${OUT_DIR}"/bindgen_*.rs
