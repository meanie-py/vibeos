#!/usr/bin/env bash
# Regenerate the win_sys ABI island — the non-COM Win32 surface the Tier-D modules
# (olefont/olepicture/olepropframe/varformat) need and com-sys does not cover.
#
# com-sys wraps the COM/automation headers (objbase/objidl/oaidl/ocidl/propidl); it
# deliberately does NOT cover wingdi.h, olectl.h, prsht.h or wincodec.h. Tier-D needs those:
#
#   * wincodec.h — olepicture.c decodes images through WIC (IWICStream ->
#                  IWICBitmapDecoder -> IWICBitmapFrameDecode -> IWICBitmapSource).
#                  oleaut32 delay-imports windowscodecs (CMakeLists.txt:118), so this is a
#                  live path, not dead weight.
#   * olectl.h   — PICTDESC / FONTDESC / OCPFIPARAMS (IPicture/IFont/property-frame structs).
#   * prsht.h    — PROPSHEETHEADERW / PROPSHEETPAGEW / HPROPSHEETPAGE (olepropframe.c builds
#                  a real comctl32 property sheet).
#   * wingdi.h   — BITMAP / BITMAPFILEHEADER / ENHMETAHEADER; winuser.h — DLGTEMPLATE.
#
# GENERATED, not hand-transcribed, and that is the whole point: the callers do
# sizeof(PROPSHEETHEADERW), sizeof(OCPFIPARAMS), sizeof(DLGTEMPLATE)/sizeof(WCHAR) and index
# fields by offset. A mis-transcribed field is a silent ABI break — no compile error, no link
# error, just a property sheet that reads garbage. Likewise a COM vtbl: one transposed slot
# calls the wrong method, silently.
#
# PER-ARCH, and that is not optional: STDMETHODCALLTYPE is __stdcall on i386 and cdecl on
# amd64/arm64, so every vtbl slot's fn type differs per target (and so do struct paddings).
#
# Everything com-sys ALREADY provides — base scalar/handle types, POINT/SIZE/RECT/LOGFONTW/
# TEXTMETRICW/PALETTEENTRY/ICONINFO, and the COM interfaces — is BLOCKLISTED here and
# re-imported from com-sys, so the shared surface stays ONE nominal type. Without that,
# bindgen re-emits a structurally-identical-but-nominally-distinct IStream / HBITMAP / RECT,
# and every call crossing the boundary fails to typecheck.
#
# Prereqs: cargo install bindgen-cli; per-arch cmake-configured build dirs (wincodec.h is
# widl-generated from sdk/include/psdk/wincodec.idl into build-<arch>/sdk/include/psdk).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"
OUT_DIR="${SCRIPT_DIR}/src"
TOOLCHAIN="${SRC}/.build-tools/llvm-mingw"
MAIN_SRC="${VIBEOS_MAIN:-/home/curi/Projects/os-playground/vibeos}"
[ -d "${TOOLCHAIN}" ] || TOOLCHAIN="${MAIN_SRC}/.build-tools/llvm-mingw"

WRAPPER="$(mktemp /tmp/win_sys_wrapper_XXXX.h)"
trap 'rm -f "${WRAPPER}"' EXIT
cat > "${WRAPPER}" <<'EOF'
/* oleaut32-rust win_sys bindgen wrapper — the non-COM Win32 surface Tier-D uses. */
#define WIN32_NO_STATUS
#define COBJMACROS
#include <windef.h>
#include <winbase.h>
#include <wingdi.h>
#include <winuser.h>
#include <objbase.h>
#include <ocidl.h>
#include <olectl.h>
#include <prsht.h>
#include <wincodec.h>
EOF

COMMON_FLAGS=(
    --use-core --no-layout-tests --no-doc-comments

    # --- WIC (wincodec.h): the five interfaces olepicture.c drives + WICRect + the
    # decode-options enum. Recursive allowlisting pulls in the vtbl-referenced WIC types
    # (IWICPalette, IWICColorContext, IWICBitmapCodecInfo, ...) automatically — intended:
    # the vtbl LAYOUT needs them even where olepicture never calls those slots.
    --allowlist-type 'IWIC(BitmapSource|BitmapDecoder|BitmapFrameDecode|ImagingFactory|Stream)'
    --allowlist-type 'WICRect'
    --allowlist-var 'WICDecodeMetadataCacheOnLoad'

    # --- olectl.h / prsht.h / wingdi.h / winuser.h structs (sizeof'd + field-indexed)
    --allowlist-type '(tag)?(PICTDESC|FONTDESC|OCPFIPARAMS)'
    --allowlist-type 'PROPSHEET(HEADER|PAGE)W'
    --allowlist-type 'HPROPSHEETPAGE'
    --allowlist-type '(tagBITMAP|BITMAPFILEHEADER|ENHMETAHEADER|DLGTEMPLATE)'
    --allowlist-type '(HMETAFILE|HMODULE|HRSRC|HGDIOBJ)'

    # --- Provided by com-sys — do NOT duplicate.
    #
    # The invariant: any type com-sys ALSO defines must be blocklisted here, because the
    # generated COBJMACRO wrappers (commacros_*.rs) do `use crate::prelude::*`, where the bare
    # name resolves to COM-SYS's type — while the vtbl slot they call through wants WIN_SYS's.
    # Two structurally identical, nominally distinct types => E0308 at the call. That is how
    # `IWICImagingFactory::CreateComponentEnumerator` (IEnumUnknown**) and `IWICStream::Stat`
    # (STATSTG*) first broke. Keep this list ⊇ (win_sys types ∩ com-sys types); check with:
    #
    #   comm -12 <(grep -oP '^pub (struct|union) \K\w+(?= \{)' src/win_sys_i386.rs | sort -u) \
    #            <(grep -oP '^pub (struct|union) \K\w+(?= \{)' ../com-sys/src/bindgen_i386.rs | sort -u)
    --blocklist-type '(IUnknown|IStream|ISequentialStream|IDispatch)'
    --blocklist-type 'I(Unknown|Stream|SequentialStream|Dispatch)Vtbl'
    # objidl/oaidl interfaces the WIC vtbls reference
    --blocklist-type 'I(EnumUnknown|EnumString|EnumSTATSTG|ErrorLog|PropertyBag2|RecordInfo|Storage|TypeComp|TypeInfo|TypeLib)(Vtbl)?'
    # objidl/oaidl/propidl data types
    --blocklist-type '(tag)?(STATSTG|PROPBAG2|BLOB|BSTRBLOB|CLIPDATA|PROPVARIANT|DISPPARAMS|EXCEPINFO)'
    --blocklist-type '(tag)?(TYPEATTR|FUNCDESC|VARDESC|ELEMDESC|TYPEDESC|IDLDESC|PARAMDESC|PARAMDESCEX|ARRAYDESC|TLIBATTR|BINDPTR|DEC)'
    --blocklist-type 'tagCA[A-Z]+'
    # The oaidl enum family. Also blocklisted so bindgen stops emitting its
    # `pub use self::tagCALLCONV as CALLCONV;` aliases — those name a type that arrives via
    # the `use com_sys::*` glob (a private import), which cannot be `pub use`d: E0365.
    --blocklist-type '(tag)?(TYPEKIND|CALLCONV|FUNCKIND|INVOKEKIND|VARKIND|DESCKIND|SYSKIND)'
    --blocklist-type '(GUID|_GUID|IID|CLSID|LPCLSID|REFGUID|REFIID|REFCLSID|WICPixelFormatGUID)'
    --blocklist-type '(ULONG|USHORT|UCHAR|BOOL|BYTE|WORD|DWORD|FLOAT|INT|UINT|wchar_t)'
    --blocklist-type '(ULONG_PTR|LONG_PTR|UINT_PTR|INT_PTR|DWORD_PTR|SIZE_T|PVOID|HANDLE)'
    --blocklist-type '(CHAR|SHORT|LONG|LONGLONG|ULONGLONG|HRESULT|SCODE|DOUBLE)'
    --blocklist-type '(LPSTR|WCHAR|LPWSTR|LPCWSTR|LCID|OLECHAR|LPOLESTR|LPCOLESTR|BSTR)'
    --blocklist-type '(_?(LARGE|ULARGE)_INTEGER|_?FILETIME|CLIPFORMAT|DATE|VARIANT_BOOL|VARTYPE)'
    --blocklist-type '(LPARAM|WPARAM|LRESULT)'
    --blocklist-type 'H(BITMAP|DC|ENHMETAFILE|FONT|GLOBAL|ICON|MENU|PALETTE|RGN|WND|ACCEL|METAFILEPICT)'
    --blocklist-type '(tag)?(POINT|SIZE|RECT|LOGFONTW|TEXTMETRICW|PALETTEENTRY)'
    --blocklist-type '_?ICONINFO'
    # CY leaks through FONTDESC.cySize; DECIMAL/VARIANT/SAFEARRAY are the same hazard for any
    # future allowlist entry. These are STRUCTS/UNIONS, so a duplicate is nominally distinct
    # and will not unify with the crate's — unlike the scalar aliases above, where a duplicate
    # would be harmless. Blocklist them all and take com-sys's.
    --blocklist-type '(tag)?(CY|DECIMAL|VARIANT|VARIANTARG|SAFEARRAY|SAFEARRAYBOUND)'

    # NOTE: raw-lines must not be inner attributes (`#![..]`) or inner docs (`//!`) — these
    # files are `include!`d from win_sys.rs, where an inner attribute would no longer be at
    # the start of the enclosing item. win_sys.rs carries them instead.
    #
    # A GLOB, not a name list: everything blocklisted above has to come back from somewhere,
    # and the blocklist covers ~50 base/handle/COM types (plus whatever a future header pulls
    # in). A glob keeps the two lists from drifting apart. Rust resolves an explicit local
    # item over a glob import, so any type win_sys DOES emit still wins here.
    --raw-line 'use com_sys::*;'
    # REF*/WICPixelFormatGUID are `#define`s or typedefs the C headers spell inline; com-sys
    # has no item for them. They are pure pointer aliases — no layout to get wrong.
    --raw-line 'pub type REFGUID = *const GUID;'
    --raw-line 'pub type REFIID = *const IID;'
    --raw-line 'pub type REFCLSID = *const CLSID;'
    --raw-line 'pub type WICPixelFormatGUID = GUID;'
)

run_bindgen() {
    local ARCH=$1 TARGET=$2; shift 2
    local BUILD_DIR="${SRC}/build-${ARCH}"
    [ -d "${BUILD_DIR}/sdk/include/psdk" ] || BUILD_DIR="${MAIN_SRC}/build-${ARCH}"
    local FINAL="${OUT_DIR}/win_sys_${ARCH}.rs"

    echo "=== Generating win_sys ${ARCH} (target: ${TARGET}) ==="
    bindgen "${WRAPPER}" "${COMMON_FLAGS[@]}" -o "$FINAL" \
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
        "$@" 2>/dev/null
    echo "    -> $(wc -l < "$FINAL") lines, $(grep -c 'Vtbl {' "$FINAL" || true) vtbls"
}

run_bindgen amd64 x86_64-pc-windows-gnu -D_WIN64 -D_M_AMD64 -D__x86_64__ -D_AMD64_
run_bindgen i386  i686-pc-windows-gnu   -D_M_IX86 -D__i386__ -D_X86_
run_bindgen arm64 aarch64-pc-windows-gnullvm -D_WIN64 -D_M_ARM64 -D__aarch64__ -D__arm64__ -D_ARM64_

echo "=== Done ==="
ls -la "${OUT_DIR}"/win_sys_*.rs
