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
# build-<arch>/ dirs are gitignored — a fresh worktree has none, and
# vibeos has no cold-cmake-build path (warm dirs only). Point at a tree
# that HAS warm build dirs (typically the main checkout) via
# VIBEOS_BUILD_ROOT; headers + ninja flags are all bindgen needs.
BUILD_ROOT="${VIBEOS_BUILD_ROOT:-$SRC}"
RES="${SRC}/.build-tools/llvm-mingw/lib/clang/19/include"
OUT_DIR="${SCRIPT_DIR}/src"
WRAPPER="${SCRIPT_DIR}/wrapper.h"

# The ninja build statement we lift DEFINES/INCLUDES/FLAGS from.
# A TU that is still COMPILED as C. The original rule's .c went to Rust when the module
# was cohab-flipped, the grep below then found nothing, and bindgen ran with no
# -I/-D at all (host mingw headers) -- so this island could not be regenerated.
NINJA_RULE='subsystems/csr/csrlib/CMakeFiles/csrlib.dir/csrlib_rust_stub.c.obj:'

# Type/var allowlist '.+' (struct field types chain everywhere).
# Function allowlist: user-mode Native API families + the SM/CSR/BASE
# subsystem-protocol helpers this stack actually calls.
# Component families use a broad suffix ([A-Za-z0-9_]+, not [A-Z]…) so
# the lowercase-second-segment internal prefixes are caught too:
# Smp*/Sm* (smss), Basep*/Base* (basesrv), Csrp*/Csr* (csrsrv), Sb*.
FN_ALLOW='(Nt|Zw)[A-Z][A-Za-z0-9_]*|Rtl[A-Z][A-Za-z0-9_]*|Ldr[A-Z][A-Za-z0-9_]*|Dbg[A-Z][A-Za-z0-9_]*|Csr[A-Za-z0-9_]+|Sm[A-Za-z0-9_]+|Sb[A-Za-z0-9_]+|Base[A-Za-z0-9_]+'

run_bindgen() {
    local ARCH=$1 TARGET=$2 BUILD_DIR="${BUILD_ROOT}/build-$1"
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
    if [ -z "$DEFINES" ] || [ -z "$INCLUDES" ]; then
        echo "!! ${ARCH}: no DEFINES/INCLUDES for $NINJA_RULE in $NINJA" >&2
        exit 1
    fi
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
        -I"${SRC}/sdk/include/reactos/subsys" \
        -I"${SRC}/subsystems/csr/csrsrv" \
        -I"${SRC}/subsystems/win/basesrv" \
        -I"${SRC}/base/system/smss" \
        -isystem "$RES" --target="${TARGET}"

    python3 "${SCRIPT_DIR}/postprocess_bindgen.py" "$RAW" "$ARCH" "$FINAL"
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
