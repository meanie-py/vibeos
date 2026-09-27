#!/bin/bash
# Generate the internal win32k.sys FFI surface for all 3 architectures.
#
# Unlike ntkrnl-sys (which hard-codes its flag set), this DERIVES the
# exact compile flags per arch from build-<arch>/build.ninja's win32k
# rule — win32k's header set (freetype, dxsdk, generated mc/ headers,
# DXGKDDI/D3D version defines) is large and build-version-coupled, so
# self-deriving keeps the bindings honest against the real build.
#
# Prerequisites: cargo install bindgen-cli; a configured build-<arch>/.
# Run from anywhere.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
RES="${TOOLCHAIN}/lib/clang/19/include"
OUT_DIR="${SCRIPT_DIR}/src"
WRAPPER="${SCRIPT_DIR}/wrapper.h"

# The ninja build statement we lift DEFINES/INCLUDES/FLAGS from. win32k.sys is
# cargo-linked now and has no C compile of its own; the C residue archive
# (win32ss/CMakeLists.txt, `win32k_residue`) compiles with the same include dirs,
# defines and pch, so its TUs carry the flag set. The old `win32k.dir/…/main.c`
# rule vanished with the C tree and left this generator unable to run at all.
NINJA_RULE='win32k_residue.dir/user/ntuser/main_rust_shims.c.obj:'

# Type allowlist '.+' (struct field types chain everywhere — need all).
# Function allowlist covers the win32k-internal naming families; the
# ntoskrnl APIs win32k also calls (Ex/Ke/Mm/Ob/Ps/Rtl/Zw/Nt) stay in
# ntkrnl-sys to avoid duplicate-symbol noise across the two crates.
# FT_/TT_ added: win32k.h includes FreeType, so win32k struct layouts already
# embed FT_* types (131 fields). freetype.c's FT functions must live in the same
# crate as those types — a separate crate cross-crate-mismatches at every
# struct-field -> FT-function boundary. So win32k-sys owns the FreeType surface.
FN_ALLOW='(Nt|Zw)?(User|Gdi)[A-Za-z0-9_]*|Int[A-Z][A-Za-z0-9_]*|co_[A-Za-z0-9_]*|Gre[A-Z][A-Za-z0-9_]*|Eng[A-Z][A-Za-z0-9_]*|User[A-Z][A-Za-z0-9_]*|GDIOBJ[A-Za-z0-9_]*|Hmg[A-Za-z0-9_]*|(DC|REGION|PATH|TEXTOBJ|BRUSH|SURFACE|PDEVOBJ|PALETTE|XLATEOBJ|BITMAP|LFONT|RFONT)_[A-Za-z0-9_]*|FT_[A-Za-z0-9_]*|TT_[A-Za-z0-9_]*'

run_bindgen() {
    local ARCH=$1 TARGET=$2 BUILD_DIR="${SRC}/build-$1"
    local RAW="/tmp/win32k_bindgen_${ARCH}_raw.rs"
    local FINAL="${OUT_DIR}/bindgen_${ARCH}.rs"
    local NINJA="${BUILD_DIR}/build.ninja"

    if [ ! -f "$NINJA" ]; then
        echo "!! ${ARCH}: ${NINJA} missing — skipping (configure build-${ARCH})"
        return 0
    fi
    echo "=== Generating ${ARCH} (target: ${TARGET}) ==="

    local DEFINES INCLUDES RAWFLAGS SANITIZED
    DEFINES=$(grep -A8 "$NINJA_RULE" "$NINJA" | grep -m1 -E '^  DEFINES' | sed 's/^  DEFINES = //')
    INCLUDES=$(grep -A8 "$NINJA_RULE" "$NINJA" | grep -m1 -E '^  INCLUDES' | sed 's/^  INCLUDES = //')
    RAWFLAGS=$(grep -A8 "$NINJA_RULE" "$NINJA" | grep -m1 -E '^  FLAGS' | sed 's/^  FLAGS = //')
    # Drop codegen/PCH-only flags that break or are irrelevant to bindgen
    # parsing (keep -fms-extensions, -nostdinc, -isystem, -std=, the
    # -fno-* parse-affecting ones). NEVER add -fms-compatibility.
    SANITIZED=$(echo "$RAWFLAGS" | tr ' ' '\n' \
        | grep -vE '^-g|^-O|^-Winvalid-pch|^-ffile-prefix-map|^-include-pch|\.pch$|^-fno-omit|^-mstackrealign|^-fno-optimize|^-march|^-mtune' \
        | grep -vE '^-Xclang$|^-include$|cmake_pch' | tr '\n' ' ')

    bindgen "$WRAPPER" \
        --use-core --no-layout-tests --no-doc-comments \
        --allowlist-type '.+' \
        --allowlist-var '.+' \
        --allowlist-function "$FN_ALLOW" \
        -o "$RAW" \
        -- -ferror-limit=0 -Wno-everything $SANITIZED $DEFINES $INCLUDES \
        -isystem "$RES" --target="${TARGET}"

    # bindgen emits no constant for a cast macro (`#define OBJID_CLIENT ((LONG)0xFFFFFFFC)`,
    # winuser.h's spelling since the upstream merge); the preprocessor's final macro table
    # lets the postprocessor recover the integer-literal casts. One -E pass, seconds.
    local MACROS="/tmp/win32k_bindgen_${ARCH}_macros.h"
    "${TOOLCHAIN}/bin/clang" -dM -E -x c "$WRAPPER" -ferror-limit=0 -Wno-everything \
        $SANITIZED $DEFINES $INCLUDES -isystem "$RES" --target="${TARGET}" > "$MACROS"

    python3 "${SCRIPT_DIR}/postprocess_bindgen.py" "$RAW" "$ARCH" "$FINAL" --macro-dump "$MACROS"
}

run_bindgen i386  i686-pc-windows-gnu
run_bindgen amd64 x86_64-pc-windows-gnu
run_bindgen arm64 aarch64-pc-windows-gnu

echo ""
echo "=== Done. Files in ${OUT_DIR}/ ==="
ls -la "${OUT_DIR}"/bindgen_*.rs 2>/dev/null

# Spell ABI-identical callback slots `extern "system"` across the three islands this run just
# wrote, projecting per symbol from the i386 witness (the only arch that still records which
# callbacks are stdcall). Without this the slots keep the arch's literal convention, a
# translated `extern "system"` callback will not unify with them, and the standing workaround
# is a transmute that erases the whole fn type. Idempotent — safe to re-run.
#
# This lives in EVERY generator on purpose: normalization that only some of them apply is
# normalization that the next regeneration of any other crate silently undoes.
_isl_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "${_isl_dir}/../../../../tools/rustconv/normalize_island_abi.py" --apply "${_isl_dir}"
