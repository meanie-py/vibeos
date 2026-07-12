#!/usr/bin/env bash
# Regenerate com-sys bindgen bindings (COM interface vtbls + automation types)
# for all three architectures. Bindings are CHECKED IN (src/bindgen_<arch>.rs);
# rerun this only when the SDK COM headers change, then diff-review.
#
# Prerequisites: cargo install bindgen-cli; per-arch cmake-configured build dirs
# (for the widl-generated psdk headers: objidl.h, oaidl.h, ocidl.h, ...).
#
# Calling-convention note: no postprocess needed — the headers declare
# STDMETHODCALLTYPE (= __stdcall), so with the right --target libclang gives
# bindgen the correct convention per arch automatically (stdcall on i686,
# C on x86_64/aarch64).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
OUT_DIR="${SCRIPT_DIR}/src"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
# Worktrees don't carry build dirs; fall back to the main checkout's.
MAIN_SRC="${VIBEOS_MAIN:-/home/curi/Projects/os-playground/vibeos}"
[ -d "${TOOLCHAIN}" ] || TOOLCHAIN="${MAIN_SRC}/.build-tools/llvm-mingw"

# Interfaces: every COM interface type (+ its Vtbl) the wrapper headers declare.
# Data types: the automation/marshaling structs the ole32/oleaut32 bodies use.
COMMON_FLAGS=(
    --use-core --no-layout-tests --no-doc-comments
    --allowlist-type 'I[A-Z][A-Za-z0-9_]*'
    --allowlist-type '(GUID|IID|CLSID|FMTID|LP(OLE)?STR|OLECHAR|BSTR|LPBSTR)'
    --allowlist-type '(VARIANT(ARG)?|VARTYPE|VARENUM|SAFEARRAY(BOUND)?|DECIMAL|CY|DATE|SCODE)'
    --allowlist-type '(DISPPARAMS|EXCEPINFO|DISPID|MEMBERID|HREFTYPE|TYPEKIND|SYSKIND|INVOKEKIND|CALLCONV|FUNCKIND|VARKIND|DESCKIND)'
    --allowlist-type '(TYPEATTR|FUNCDESC|VARDESC|ELEMDESC|TYPEDESC|IDLDESC|PARAMDESC(EX)?|ARRAYDESC|TLIBATTR|CUSTDATA(ITEM)?|BINDPTR)'
    --allowlist-type '(FORMATETC|STGMEDIUM|STATSTG|STATDATA|DVTARGETDEVICE|BIND_OPTS[23]?|MULTI_QI|COSERVERINFO|COAUTHINFO|COAUTHIDENTITY)'
    --allowlist-type '(PROPVARIANT|PROPSPEC|STATPROPSTG|STATPROPSETSTG|CLIPDATA)'
    --allowlist-type '(MSHLFLAGS|MSHCTX|CLSCTX|COINIT|REGCLS|EXTCONN|APTTYPE(QUALIFIER)?|SHUTDOWN_TYPE|OLE(GETMONIKER|WHICHMK|CLOSE|RENDER|UPDATE|LINKBIND|CONTF))'
    --allowlist-type '(HRESULT|LCID|LANGID|DISPATCH_[A-Z]+)'
)

run_bindgen() {
    local ARCH=$1 TARGET=$2; shift 2
    local BUILD_DIR="${SRC}/build-${ARCH}"
    [ -d "${BUILD_DIR}/sdk/include/psdk" ] || BUILD_DIR="${MAIN_SRC}/build-${ARCH}"
    local FINAL="${OUT_DIR}/bindgen_${ARCH}.rs"

    echo "=== Generating ${ARCH} (target: ${TARGET}) ==="
    bindgen "${SCRIPT_DIR}/wrapper.h" \
        "${COMMON_FLAGS[@]}" \
        -o "$FINAL" \
        -- --target="${TARGET}" \
        -I"${SRC}/sdk/include" \
        -I"${SRC}/sdk/include/psdk" \
        -I"${SRC}/sdk/include/xdk" \
        -I"${SRC}/sdk/include/ddk" \
        -I"${SRC}/sdk/include/ndk" \
        -I"${SRC}/sdk/include/vcruntime" \
        -I"${SRC}/sdk/include/reactos" \
        -I"${SRC}/sdk/include/reactos/libs" \
        -I"${SRC}/sdk/lib/pseh/include" \
        -I"${BUILD_DIR}/sdk/include" \
        -I"${BUILD_DIR}/sdk/include/psdk" \
        -I"${BUILD_DIR}/sdk/include/reactos" \
        -isystem "${SRC}/sdk/include/crt" \
        -isystem "${TOOLCHAIN}/lib/clang/19/include" \
        -D__REACTOS__ -D_WIN32_WINNT=0x600 -DWINVER=0x502 -D_USE_DUMMY_PSEH=1 \
        "$@"
    echo "    -> $(wc -l < "$FINAL") lines, $(grep -c 'Vtbl {' "$FINAL" || true) vtbls"
}

run_bindgen amd64 x86_64-pc-windows-gnu -D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_
run_bindgen i386  i686-pc-windows-gnu   -D_M_IX86 -D__i386__ -D_X86_
run_bindgen arm64 aarch64-pc-windows-gnullvm -D_WIN64 -D_M_ARM64 -D__aarch64__ -D__arm64__ -D_ARM64_

echo "=== Done ==="
ls -la "${OUT_DIR}"/bindgen_*.rs
