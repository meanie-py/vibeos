#!/bin/bash
# Generate the user-mode Native API + SM/CSR/BASE subsystem-protocol
# FFI surface for all 3 architectures (SMSS→CSRSS translation track).
#
# Like win32k-sys (and unlike ntkrnl-sys), this DERIVES the exact
# compile flags per arch from build-<arch>/build.ninja's csrsrv rule —
# csrsrv pulls the broadest header set in this stack (ndk + the sm/csr/
# win subsys headers via -Isdk/include/reactos/subsys), so lifting its
# real build flags keeps the bindings honest against the actual build.
#
# Prerequisites: cargo install bindgen-cli; a configured build-<arch>/.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
RES="${SRC}/.build-tools/llvm-mingw/lib/clang/19/include"
OUT_DIR="${SCRIPT_DIR}/src"
WRAPPER="${SCRIPT_DIR}/wrapper.h"

# The ninja build statement we lift DEFINES/INCLUDES/FLAGS from.
NINJA_RULE='csrsrv.dir/init.c.obj:'

# Type/var allowlist '.+' (struct field types chain everywhere).
# Function allowlist: user-mode Native API families + the SM/CSR/BASE
# subsystem-protocol helpers this stack actually calls.
FN_ALLOW='(Nt|Zw)[A-Z][A-Za-z0-9_]*|Rtl[A-Z][A-Za-z0-9_]*|Ldr[A-Z][A-Za-z0-9_]*|Dbg[A-Z][A-Za-z0-9_]*|Csr[A-Z][A-Za-z0-9_]*|Sm[A-Z][A-Za-z0-9_]*|Base[A-Z][A-Za-z0-9_]*|Sb[A-Z][A-Za-z0-9_]*'

run_bindgen() {
    local ARCH=$1 TARGET=$2 BUILD_DIR="${SRC}/build-$1"
    local RAW="/tmp/ntdll_bindgen_${ARCH}_raw.rs"
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
    # parsing. NEVER add -fms-compatibility.
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

    python3 "${SCRIPT_DIR}/postprocess_bindgen.py" "$RAW" "$ARCH" "$FINAL"
}

run_bindgen i386  i686-pc-windows-gnu
run_bindgen amd64 x86_64-pc-windows-gnu
run_bindgen arm64 aarch64-pc-windows-gnu

echo ""
echo "=== Done. Files in ${OUT_DIR}/ ==="
ls -la "${OUT_DIR}"/bindgen_*.rs 2>/dev/null
