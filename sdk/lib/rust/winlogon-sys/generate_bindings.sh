#!/bin/bash
# Generate the winlogon.exe FFI substrate for all 3 architectures.
#
# Like ntdll-sys/win32k-sys, this DERIVES the exact compile flags per
# arch from build-<arch>/build.ninja's winlogon rule (environment.c) --
# winlogon pulls winwlx.h + ndk + undocuser/undocmpr via its own -I set,
# so lifting its real build flags keeps the bindings honest against the
# actual build.
#
# Prerequisites: cargo install bindgen-cli; a configured build-<arch>/.
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
BUILD_ROOT="${VIBEOS_BUILD_ROOT:-$SRC}"
RES="${SRC}/.build-tools/llvm-mingw/lib/clang/19/include"
OUT_DIR="${SCRIPT_DIR}/src"
WRAPPER="${SCRIPT_DIR}/wrapper.h"

# The ninja build statement we lift DEFINES/INCLUDES/FLAGS from. Must be a
# still-compiled TU (environment.c is now flipped to Rust → no .obj rule).
# A TU that is still COMPILED as C. The original rule's .c went to Rust when the module
# was cohab-flipped, the grep below then found nothing, and bindgen ran with no
# -I/-D at all (host mingw headers) -- so this island could not be regenerated.
NINJA_RULE='base/system/winlogon/CMakeFiles/winlogon_c.dir/winlogon_rust_stub.c.obj:'

# Type/var allowlist '.+' (struct field types chain everywhere).
# Function allowlist: broad Win32 + GINA + native families the winlogon
# leaves call (registry, process/desktop, security/LSA/token,
# screensaver, native Rtl/Nt/Zw for security.c & sas.c).
FN_ALLOW='(Reg|Create|Open|Close|Wait|Get|Set|Switch|Expand|Impersonate|Revert|Exit|Lsa|Logon|Load|Free|Send|Post|Duplicate|Allocate|Lookup|Query|Start|Run|Terminate|Wlx|Nt|Zw|Rtl|Ldr|Dbg|Heap|Dialog|End|Delete|Kill|Def|Register|Unregister|Translate|Dispatch|Destroy|Show|Update|Move|Enum|Find|System|Map|Begin|Draw|Release|Peek|Is|Adjust|Check|Enable|Disable|Format|Message|Convert|Equal|Copy|Make|Initialize|Add|Display|Crypt|Activate|Broadcast|Unload|Notify|Insert|Remove|Read|Write|Sm|Csr|Base|Alloc|Local|Global|Default|Sleep|Lock|Unlock|Acquire|Validate|Verify|Report|Change|Connect|Disconnect|Generate|Prepare|Cancel|Reset|Call|Hook|Unhook|Peek|Wait|Signal|Release|Impersonate)[A-Za-z0-9_]*'

run_bindgen() {
    local ARCH=$1 TARGET=$2 BUILD_DIR="${BUILD_ROOT}/build-$1"
    local RAW="/tmp/winlogon_bindgen_${ARCH}_raw.rs"
    local FINAL="${OUT_DIR}/bindgen_${ARCH}.rs"
    local NINJA="${BUILD_DIR}/build.ninja"

    if [ ! -f "$NINJA" ]; then
        echo "!! ${ARCH}: ${NINJA} missing -- skipping (configure build-${ARCH})"
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
