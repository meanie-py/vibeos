#!/bin/bash
# Generate full kernel FFI bindings for all 3 architectures.
#
# Uses ntoskrnl.h (the kernel's master internal header) + ARM3 miarm.h
# to get BOTH public DDK functions AND internal kernel types/functions.
#
# Prerequisites: cargo install bindgen-cli
# Run from anywhere (paths are computed from script location).
# Stages only: [--output-dir NEW_DIRECTORY], default .build-tools/bindgen-runs/.
# COMPLETE marks success; raw inputs/depfiles and normalized src/ are retained.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# ntkrnl-sys is at sdk/lib/rust/ntkrnl-sys, so reactos root is 4 levels up
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
# In-repo toolchain (was a stale macOS-era sibling path).
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../../../../tools/bindgen-stage.sh
source "${SRC}/tools/bindgen-stage.sh"
bindgen_inputs arm64 amd64 i386
bindgen_stage kernel "$@"

# Wrapper header — includes full ntoskrnl internals + ARM3 MM
WRAPPER="${BINDGEN_RUN}/raw/ntkrnl_sys_wrapper.h"
cat > "$WRAPPER" << 'HEADER'
#define _NTOSKRNL_
#define _NTSYSTEM_
#define _USE_DUMMY_PSEH 1
#define __REACTOS__
/* Skip intrin.h — compiler intrinsics not available in libclang */
#define _INTRIN_H_
#define _INTRIN_X86_H_
/* __int64 needed by various headers */
#ifndef __int64
#define __int64 long long
#endif
/* NO __wfi()/__yield() stubs here. They were "ARM64 intrinsic stubs", but as
   zero-arg function-like macros they turn intrin.h's `void __yield(void);`
   prototype into a one-argument macro invocation -> "too many arguments provided
   to function-like macro invocation" -> bindgen ABORTS on arm64. That made arm64
   bindings ungeneratable from the day intrin_arm64.h landed (2026-09-20) while
   this script still exited 0, and the substrate silently drifted from the C.
   Note also that the two defines below are INERT: intrin.h is `#pragma once`
   with an unconditional `#define __INTRIN_H_` and no #ifndef, and
   _INTRIN_X86_H_ matches nothing in the tree. Left in place only because
   removing them changes nothing. */
#undef FORCEINLINE
#define FORCEINLINE
/* Full ntoskrnl master header (DDK + NDK + internal) */
#include <ntoskrnl.h>
/* ARM3 memory manager internals (POOL_HEADER, MMPTE, etc.) */
#include "mm/ARM3/miarm.h"
/* Input class-driver IOCTL surface — KEYBOARD_INPUT_DATA /
   MOUSE_INPUT_DATA (+ P* aliases). Not pulled by ntoskrnl.h; needed by
   the arm64input virtio driver translation (drivers/input/arm64input). */
#include <ntddkbd.h>
#include <ntddmou.h>
HEADER

# Common bindgen flags
COMMON_FLAGS=(
    --use-core --no-layout-tests --no-doc-comments
    --allowlist-type '.+'
    --allowlist-function '(Ex|Ke|Ki|Mi|Mm|Ps|Ob|Rtl|Zw|Nt|Se|Io|Hal|Cc|Cm|Dbgk|Ldr|Pop|Inbv|Lpc|Vf).*'
)

run_bindgen() {
    local ARCH=$1
    # Generated headers belong to this worktree; bindgen_inputs checked isolation.
    local BUILD_DIR="${SRC}/build-${ARCH}"
    local TARGET=$2
    shift 2
    local EXTRA_DEFS=("$@")
    local RAW="${BINDGEN_RUN}/raw/bindgen_${ARCH}.rs"
    local FINAL="${OUT_DIR}/bindgen_${ARCH}.rs"

    echo "=== Generating ${ARCH} (target: ${TARGET}) ==="

    bindgen "$WRAPPER" \
        "${COMMON_FLAGS[@]}" \
        -o "$RAW" \
        --depfile "${BINDGEN_RUN}/raw/bindgen_${ARCH}.d" \
        -- \
        -ferror-limit=0 -Wno-everything \
        -D_NTOSKRNL_ -D_NTSYSTEM_ \
        -D_USE_DUMMY_PSEH=1 -D__REACTOS__ \
        "${EXTRA_DEFS[@]}" \
        -DDBG=1 -DWINVER=0x502 -D_WIN32_WINNT=0x502 \
        -I"${BUILD_DIR}/ntoskrnl" \
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
        -isystem "${TOOLCHAIN}/lib/clang/19/include" \
        --target="${TARGET}"

    python3 "${SCRIPT_DIR}/postprocess_bindgen.py" "$RAW" "$ARCH" "$FINAL"
}

run_bindgen arm64 aarch64-pc-windows-gnu \
    -D_WIN64 -D_M_ARM64 -D__aarch64__ -D__arm64__ -D_ARM64_

run_bindgen amd64 x86_64-pc-windows-gnu \
    -D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_

run_bindgen i386 i686-pc-windows-gnu \
    -D_M_IX86 -D__i386__ -D_X86_

echo ""
echo "=== Generated. Normalizing staged files in ${OUT_DIR}/ ==="
ls -la "${OUT_DIR}"/bindgen_*.rs

# Spell ABI-identical callback slots `extern "system"` across the three islands this run just
# wrote, projecting per symbol from the i386 witness (the only arch that still records which
# callbacks are stdcall). Without this the slots keep the arch's literal convention, a
# translated `extern "system"` callback will not unify with them, and the standing workaround
# is a transmute that erases the whole fn type. Idempotent — safe to re-run.
#
# This lives in EVERY generator on purpose: normalization that only some of them apply is
# normalization that the next regeneration of any other crate silently undoes.
python3 "${SRC}/tools/rustconv/normalize_island_abi.py" --apply --stem bindgen "$BINDGEN_RUN"
touch "${BINDGEN_RUN}/COMPLETE"
echo "bindgen: complete: ${BINDGEN_RUN}"
