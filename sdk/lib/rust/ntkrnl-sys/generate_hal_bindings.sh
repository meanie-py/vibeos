#!/bin/bash
# Generate HAL FFI bindings for i386 and amd64.
#
# arm64 is skipped — arm64 HAL is C-stub only, uses standard NDK types
# already covered by the kernel ntkrnl-sys bindgen output.
#
# Mirrors the structure of generate_bindings.sh (the kernel one) but
# uses wrapper_hal.h and outputs to hal_bindgen_{i386,amd64}.rs.
#
# Prerequisites: cargo install bindgen-cli
# Run from anywhere (paths computed from script location).
# Stages only: [--output-dir NEW_DIRECTORY], default .build-tools/bindgen-runs/.
# COMPLETE marks success; raw inputs/depfiles and normalized src/ are retained.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../../../../tools/bindgen-stage.sh
source "${SRC}/tools/bindgen-stage.sh"
bindgen_inputs amd64 i386
bindgen_stage hal "$@"
WRAPPER="${SCRIPT_DIR}/wrapper_hal.h"

# Verify clang version dir under the toolchain
CLANG_VER_DIR="${TOOLCHAIN}/lib/clang/19"
if [ ! -d "${CLANG_VER_DIR}/include" ]; then
    echo "ERROR: cannot locate ${TOOLCHAIN}/lib/clang/<ver>/ — is llvm-mingw installed?"
    exit 1
fi

# Common bindgen flags.  Allowlist Hal* and Acpi* externs + every type.
COMMON_FLAGS=(
    --use-core --no-layout-tests --no-doc-comments
    --allowlist-type '.+'
    --allowlist-function '(Hal|Halp|Acpi|Hv|HV|Hci).*'
    --allowlist-var '.+'
)

run_bindgen() {
    local ARCH=$1
    local BUILD_DIR="${SRC}/build-${ARCH}"
    local TARGET=$2
    shift 2
    local EXTRA_DEFS=("$@")
    local RAW="${BINDGEN_RUN}/raw/hal_bindgen_${ARCH}.rs"
    local FINAL="${OUT_DIR}/hal_bindgen_${ARCH}.rs"

    echo "=== Generating HAL bindings for ${ARCH} (target: ${TARGET}) ==="

    bindgen "$WRAPPER" \
        "${COMMON_FLAGS[@]}" \
        -o "$RAW" \
        --depfile "${BINDGEN_RUN}/raw/hal_bindgen_${ARCH}.d" \
        -- \
        -ferror-limit=0 -Wno-everything \
        -D_NTHAL_ -D_NTSYSTEM_ \
        -D_USE_DUMMY_PSEH=1 -D__REACTOS__ \
        "${EXTRA_DEFS[@]}" \
        -DDBG=1 -DWINVER=0x502 -D_WIN32_WINNT=0x502 \
        -I"${BUILD_DIR}/ntoskrnl" \
        -I"${SRC}/hal/halx86/include" \
        -I"${SRC}/vendor/acpica/include" \
        -I"${SRC}/ntoskrnl" \
        -I"${SRC}/sdk/include" \
        -I"${SRC}/sdk/include/ddk" \
        -I"${SRC}/sdk/include/ndk" \
        -I"${SRC}/sdk/include/psdk" \
        -I"${SRC}/sdk/include/xdk" \
        -I"${SRC}/sdk/include/reactos" \
        -I"${SRC}/sdk/include/reactos/libs" \
        -I"${SRC}/sdk/include/reactos/drivers" \
        -I"${SRC}/sdk/include/vcruntime" \
        -I"${SRC}/sdk/include/winrt" \
        -I"${BUILD_DIR}/sdk/include" \
        -I"${BUILD_DIR}/sdk/include/psdk" \
        -I"${BUILD_DIR}/sdk/include/ddk" \
        -I"${BUILD_DIR}/sdk/include/dxsdk" \
        -I"${BUILD_DIR}/sdk/include/reactos" \
        -I"${BUILD_DIR}/sdk/include/reactos/mc" \
        -I"${SRC}/sdk/include/dxsdk" \
        -I"${SRC}/sdk/lib/pseh/include" \
        -I"${SRC}" \
        -I"${SRC}/sdk/lib/drivers/arbiter" \
        -I"${SRC}/sdk/lib/cmlib" \
        -I"${SRC}/ntoskrnl/include" \
        -I"${BUILD_DIR}/ntoskrnl/include" \
        -I"${BUILD_DIR}/ntoskrnl/include/internal" \
        -isystem "${SRC}/sdk/include/crt" \
        -isystem "${CLANG_VER_DIR}/include" \
        --target="${TARGET}"

    # Reuse the kernel bindgen post-processor (renames helpers, adds module
    # header, etc.).  Output filename has the hal_ prefix so it slots into
    # src/hal_bindgen_{arch}.rs and never collides with the kernel bindings.
    # The --hal flag wraps kernel-data extern statics with the raw-dylib
    # link attribute needed for cross-DLL data references from hal.dll.
    python3 "${SCRIPT_DIR}/postprocess_bindgen.py" --hal "$RAW" "$ARCH" "$FINAL"

    echo "  → $(wc -l < "$FINAL") lines, $(wc -c < "$FINAL" | numfmt --to=iec) bytes"
}

run_bindgen amd64 x86_64-pc-windows-gnu \
    -D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_

run_bindgen i386 i686-pc-windows-gnu \
    -D_M_IX86 -D__i386__ -D_X86_

echo ""
echo "=== Generated. Normalizing staged files in ${OUT_DIR}/ ==="
ls -la "${OUT_DIR}"/hal_bindgen_*.rs

# Spell ABI-identical callback slots `extern "system"` across the islands this run just
# wrote, projecting per symbol from the i386 witness (the only arch that still records which
# callbacks are stdcall). Without this the slots keep the arch's literal convention, a
# translated `extern "system"` callback will not unify with them, and the standing workaround
# is a transmute that erases the whole fn type. Idempotent — safe to re-run.
#
# This lives in EVERY generator on purpose: normalization that only some of them apply is
# normalization that the next regeneration of any other crate silently undoes.
python3 "${SRC}/tools/rustconv/normalize_island_abi.py" --apply --stem hal_bindgen "$BINDGEN_RUN"
touch "${BINDGEN_RUN}/COMPLETE"
echo "bindgen: complete: ${BINDGEN_RUN}"
