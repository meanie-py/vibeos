//! shlwapi-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
// <stdint.h>/<stddef.h> names: no island allowlists those headers (ddraw used 26 of them).
// Fixed-width, so identical on every target; an island that does carry one shadows nothing
// (a glob import never conflicts with an item the prelude defines).
pub type int8_t = i8;
pub type int16_t = i16;
pub type int32_t = i32;
pub type int64_t = i64;
pub type uint8_t = u8;
pub type uint16_t = u16;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub type intptr_t = isize;
pub type uintptr_t = usize;
pub type ptrdiff_t = isize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME): wine's own flag test on the default
// channel, then ros_dbg_log -- win32macros-rust's WINE_*! macros, which expand HERE (the
// channel storage and the ros_dbg_log extern are this crate's; `port resolve` writes them).
// MACROS, never out-of-line fns: an out-of-line wrapper is an optimization barrier that
// uncloaks latent ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => { win32macros_rust::WINE_TRACE!($($arg)*) }; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { win32macros_rust::WINE_ERR!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => { win32macros_rust::WINE_WARN!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { win32macros_rust::WINE_FIXME!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

// ================= ATL ========================================================================
// ATL's CStringW/CComPtr/containers, ported once in atlclasses-rust.
pub use atl_rust::{atl_format, atl_format_id, CA2WEX, CW2AEX, CAtlArray, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CLocalAllocator, CSimpleMap, CStringA, CStringW, POSITION, _AtlBaseModule};
pub use atl_rust::CStringW as CSimpleString;
pub use atl_rust::CStringW as CString;
pub use atl_rust::CComHeapPtr;

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_TYPECHECK, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedCompareExchangePointer, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlSecureZeroMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, SecureZeroMemory, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_ERR_, WINE_FIXME, WINE_FIXME_, WINE_TRACE, WINE_TRACE_, WINE_WARN, WINE_WARN_, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_fourcc, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, strlenW, tolowerW, wide_bytes, wine_dbgstr_a, wine_dbgstr_fourcc, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_rect, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= hand-written (shlwapi) =======================================================
// Defined by the C++ TUs this port keeps C (propbag.cpp, utils.cpp), all with C linkage:
// propbag's `extern "C" { CRITICAL_SECTION g_csBagCacheLock; }` and two EXTERN_C functions.
extern "C" {
    pub static mut g_csBagCacheLock: CRITICAL_SECTION;
    pub fn FreeViewStatePropertyBagCache();
}
extern "system" {
    pub fn SHInvokeCommandWithFlagsAndSite(
        hWnd: HWND,
        pUnk: *mut IUnknown,
        pShellFolder: *mut IShellFolder,
        pidl: LPCITEMIDLIST,
        fCMIC: UINT,
        pszVerb: LPCSTR,
    ) -> HRESULT;
    // uuid.lib (the module links it): IID_NULL is winnt's alias of GUID_NULL.
    pub static GUID_NULL: GUID;
}
pub use self::GUID_NULL as IID_NULL;
pub use self::GUID_NULL as CLSID_NULL; // guiddef.h: `#define CLSID_NULL GUID_NULL`
// winerror.h: HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER) -- a function-like expansion, so
// neither the facts pass nor the SDK lowering takes it.
pub const E_NOT_SUFFICIENT_BUFFER: HRESULT = 0x8007_007Au32 as HRESULT;
// winnt.h `{0,0,0,0,0,0}`: a brace initializer the SDK lowering refuses.
pub const SECURITY_NULL_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: [0; 6] };
// ordinal.c:5471, over FDTF_* (u32 in the island).
pub const SHFORMATDT_UNSUPPORTED_FLAGS: u32 =
    FDTF_RELATIVE | FDTF_LTRDATE | FDTF_RTLDATE | FDTF_NOAUTOREADINGORDER;

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const GB: ULONGLONG = 0x40000000; // clang, string.c
pub const KB: ULONGLONG = 0x400; // clang, string.c
pub const MB: ULONGLONG = 0x100000; // clang, string.c
pub const OLECMDERR_E_NOTSUPPORTED: HRESULT = 0x80040100_u32 as HRESULT; // clang, ordinal.c
pub const OLECMDERR_E_UNKNOWNGROUP: HRESULT = 0x80040104_u32 as HRESULT; // clang, ordinal.c
pub const PB: ULONGLONG = 0x4000000000000; // clang, string.c
pub const PUAF_ISFILE: i32 = 2; // clang, zonechk.c
pub const SHELL_NO_POLICY: DWORD = 0xFFFFFFFF; // clang, ordinal.c
pub const SHLWAPI_DEF_ASSOCF: i32 = 0x40F; // clang, assoc.c
pub const SHREGENUM_DEFAULT: i32 = 0; // clang, reg.c
pub const SHREGENUM_HKCU: i32 = 1; // clang, reg.c
pub const SHREGENUM_HKLM: i32 = 0x10; // clang, reg.c
pub const TB: ULONGLONG = 0x10000000000; // clang, string.c
pub const URLIS_HASQUERY: i32 = 6; // clang, url.c
pub const URLIS_NOHISTORY: i32 = 2; // clang, url.c
pub const URL_E_INVALID_SYNTAX: SCODE = -2147217407; // clang, url.c
pub const URL_SCHEME_GOPHER: i32 = 3; // clang, url.c
pub const URL_SCHEME_LOCAL: i32 = 0xE; // clang, url.c
pub const URL_SCHEME_MK: i32 = 0xA; // clang, url.c
pub const URL_SCHEME_NEWS: i32 = 5; // clang, url.c
pub const URL_SCHEME_NNTP: i32 = 6; // clang, url.c
pub const URL_SCHEME_RES: i32 = 0x12; // clang, url.c
pub const URL_SCHEME_SNEWS: i32 = 0xD; // clang, url.c
pub const URL_SCHEME_TELNET: i32 = 7; // clang, url.c
pub const URL_SCHEME_UNKNOWN: i32 = 0; // clang, url.c
pub const URL_SCHEME_WAIS: i32 = 8; // clang, url.c
pub const VT_EMPTY: i32 = 0; // clang, ordinal.c
pub const VT_I1: i32 = 0x10; // clang, ordinal.c
pub const VT_I2: i32 = 2; // clang, ordinal.c
pub const VT_INT: i32 = 0x16; // clang, ordinal.c
pub const VT_UI1: i32 = 0x11; // clang, ordinal.c
pub const VT_UI2: i32 = 0x12; // clang, ordinal.c
pub const VT_UI4: i32 = 0x13; // clang, ordinal.c
pub const VT_UINT: i32 = 0x17; // clang, ordinal.c
// ==== end port facts ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn IUnknown_CPContainerInvokeParam(container: *mut IUnknown, riid: *const IID, dispId: DISPID, buffer: *mut VARIANTARG, cParams: DWORD, ...) -> HRESULT;
    pub fn SHPackDispParams(params: *mut DISPPARAMS, args: *mut VARIANTARG, cnt: UINT, ...) -> HRESULT;
    pub fn ShellMessageBoxWrapW(hInstance: HINSTANCE, hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ...) -> INT;
    pub fn wnsprintfA(lpOut: LPSTR, cchLimitIn: INT, lpFmt: LPCSTR, ...) -> INT;
    pub fn wnsprintfW(lpOut: LPWSTR, cchLimitIn: INT, lpFmt: LPCWSTR, ...) -> INT;
}
// ==== end port variadic ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: tagCLSCTX = tagCLSCTX_CLSCTX_INPROC_SERVER; // island enum // gen:a0abdc106437
pub const COINIT_APARTMENTTHREADED: tagCOINIT = tagCOINIT_COINIT_APARTMENTTHREADED; // island enum // gen:7310b6b36881
pub const COINIT_DISABLE_OLE1DDE: tagCOINIT = tagCOINIT_COINIT_DISABLE_OLE1DDE; // island enum // gen:d470b30bc3ab
pub const DISPATCH_METHOD: u32 = 1; // jsproxy-rust/src/prelude.rs // gen:c2c5d5394d80
pub const OLECMDID_ENABLE_INTERACTION: OLECMDID = OLECMDID_OLECMDID_ENABLE_INTERACTION; // island enum // gen:adba90fb773f
pub const OLECMDID_HIDETOOLBARS: OLECMDID = OLECMDID_OLECMDID_HIDETOOLBARS; // island enum // gen:9d38a99e92c8
pub const OLECMDID_PASTE: OLECMDID = OLECMDID_OLECMDID_PASTE; // island enum // gen:ff97f1a5bad1
pub const OLECMDID_PROPERTIES: OLECMDID = OLECMDID_OLECMDID_PROPERTIES; // island enum // gen:f3cc8ce63340
pub const OLECMDID_SELECTALL: OLECMDID = OLECMDID_OLECMDID_SELECTALL; // island enum // gen:433977dd2087
pub const OLECMDID_SHOWFIND: OLECMDID = OLECMDID_OLECMDID_SHOWFIND; // island enum // gen:369449a3d01e
pub const OLECMDID_SHOWPRINT: OLECMDID = OLECMDID_OLECMDID_SHOWPRINT; // island enum // gen:d3721afc8cd3
pub const OLECMDID_UPDATECOMMANDS: OLECMDID = OLECMDID_OLECMDID_UPDATECOMMANDS; // island enum // gen:f00257c73cc1
pub const PERCEIVED_TYPE_APPLICATION: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_APPLICATION; // island enum // gen:7f5a0d48cafc
pub const PERCEIVED_TYPE_AUDIO: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_AUDIO; // island enum // gen:738ed052db80
pub const PERCEIVED_TYPE_COMPRESSED: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_COMPRESSED; // island enum // gen:af9261acfc1c
pub const PERCEIVED_TYPE_DOCUMENT: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_DOCUMENT; // island enum // gen:21186dd14502
pub const PERCEIVED_TYPE_IMAGE: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_IMAGE; // island enum // gen:0236188a1805
pub const PERCEIVED_TYPE_SYSTEM: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_SYSTEM; // island enum // gen:7406af09d96c
pub const PERCEIVED_TYPE_TEXT: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_TEXT; // island enum // gen:e87d9cb5ab58
pub const PERCEIVED_TYPE_UNSPECIFIED: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_UNSPECIFIED; // island enum // gen:d97d41cb6ef1
pub const PERCEIVED_TYPE_VIDEO: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_VIDEO; // island enum // gen:51ac7f9123ca
pub const SHKEY_Key_Classes: _shellkey_flags = _shellkey_flags_SHKEY_Key_Classes; // island enum // gen:e443f262593c
pub const SHKEY_Key_Explorer: _shellkey_flags = _shellkey_flags_SHKEY_Key_Explorer; // island enum // gen:8111965655d4
pub const SHKEY_Key_Shell: _shellkey_flags = _shellkey_flags_SHKEY_Key_Shell; // island enum // gen:885375ff5fb7
pub const SHKEY_Key_ShellNoRoam: _shellkey_flags = _shellkey_flags_SHKEY_Key_ShellNoRoam; // island enum // gen:4da73a4828de
pub const SHKEY_Root_HKCU: _shellkey_flags = _shellkey_flags_SHKEY_Root_HKCU; // island enum // gen:579c5871ac2b
pub const SHKEY_Subkey_Associations: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_Associations; // island enum // gen:01c0f5cb506b
pub const SHKEY_Subkey_Default: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_Default; // island enum // gen:d274172ca968
pub const SHKEY_Subkey_FileExts: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_FileExts; // island enum // gen:cb626b5e2c42
pub const SHKEY_Subkey_Handlers: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_Handlers; // island enum // gen:475ab01bc3e7
pub const SHKEY_Subkey_MUICache: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_MUICache; // island enum // gen:c9de6f613b63
pub const SHKEY_Subkey_ResourceName: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_ResourceName; // island enum // gen:625b5126d3a8
pub const SHKEY_Subkey_Volatile: _shellkey_flags = _shellkey_flags_SHKEY_Subkey_Volatile; // island enum // gen:0d287169b07a
pub const STATFLAG_NONAME: tagSTATFLAG = tagSTATFLAG_STATFLAG_NONAME; // island enum // gen:59d11b33bab1
pub const STGTY_STREAM: DWORD = 2; // ole32-rust/src/consts.rs // gen:cd07357387d8
pub const STIF_SUPPORT_HEX: u32 = 1; // acppage-rust/src/types_amd64.rs // gen:4d62c0461ae4
pub const STREAM_SEEK_CUR: DWORD = 1; // comctl32-rust/src/prelude.rs // gen:fb3424458022
pub const STREAM_SEEK_END: DWORD = 2; // ole32-rust/src/consts.rs // gen:a8d972053bc6
pub const STREAM_SEEK_SET: DWORD = 0; // comctl32-rust/src/prelude.rs // gen:7efdff3b6280
pub const STRRET_CSTR: tagSTRRET_TYPE = tagSTRRET_TYPE_STRRET_CSTR; // island enum // gen:0f50788097a1
pub const STRRET_OFFSET: tagSTRRET_TYPE = tagSTRRET_TYPE_STRRET_OFFSET; // island enum // gen:60dc7bcc36a7
pub const STRRET_WSTR: tagSTRRET_TYPE = tagSTRRET_TYPE_STRRET_WSTR; // island enum // gen:da593b4e8e46
pub const TLS_OUT_OF_INDEXES: DWORD = 0xFFFF_FFFF; // menuband-rust/src/prelude.rs // gen:08ba168bc7a9
pub const URLIS_APPLIABLE: URLIS = URLIS_URLIS_APPLIABLE; // island enum // gen:072f43ed43fb
pub const URLIS_DIRECTORY: URLIS = URLIS_URLIS_DIRECTORY; // island enum // gen:0ce9d806dfaa
pub const URLIS_FILEURL: URLIS = URLIS_URLIS_FILEURL; // island enum // gen:d9eedc7eb537
pub const URLIS_OPAQUE: URLIS = URLIS_URLIS_OPAQUE; // island enum // gen:104854b171d7
pub const URLIS_URL: URLIS = URLIS_URLIS_URL; // island enum // gen:53f439c0c817
pub const URLZONE_UNTRUSTED: tagURLZONE = tagURLZONE_URLZONE_UNTRUSTED; // island enum // gen:3c205b88aa94
pub const URL_PART_HOSTNAME: URL_PART = URL_PART_URL_PART_HOSTNAME; // island enum // gen:5548ecdbcc00
pub const URL_PART_PASSWORD: URL_PART = URL_PART_URL_PART_PASSWORD; // island enum // gen:5d2d29437a84
pub const URL_PART_PORT: URL_PART = URL_PART_URL_PART_PORT; // island enum // gen:29a0c86fa238
pub const URL_PART_QUERY: URL_PART = URL_PART_URL_PART_QUERY; // island enum // gen:50b3b90eb8b7
pub const URL_PART_SCHEME: URL_PART = URL_PART_URL_PART_SCHEME; // island enum // gen:dd7a2c37c260
pub const URL_PART_USERNAME: URL_PART = URL_PART_URL_PART_USERNAME; // island enum // gen:6bebc14d68cc
pub const URL_SCHEME_ABOUT: URL_SCHEME = URL_SCHEME_URL_SCHEME_ABOUT; // island enum // gen:e99508888a7e
pub const URL_SCHEME_FILE: URL_SCHEME = URL_SCHEME_URL_SCHEME_FILE; // island enum // gen:ba297e24a7dc
pub const URL_SCHEME_FTP: URL_SCHEME = URL_SCHEME_URL_SCHEME_FTP; // island enum // gen:aba2ca2c1799
pub const URL_SCHEME_HTTP: URL_SCHEME = URL_SCHEME_URL_SCHEME_HTTP; // island enum // gen:e27f8ef8e570
pub const URL_SCHEME_HTTPS: URL_SCHEME = URL_SCHEME_URL_SCHEME_HTTPS; // island enum // gen:4e2f5656f685
pub const URL_SCHEME_INVALID: URL_SCHEME = URL_SCHEME_URL_SCHEME_INVALID; // island enum // gen:f7ecc77c670f
pub const URL_SCHEME_JAVASCRIPT: URL_SCHEME = URL_SCHEME_URL_SCHEME_JAVASCRIPT; // island enum // gen:63c76d58b0c1
pub const URL_SCHEME_MAILTO: URL_SCHEME = URL_SCHEME_URL_SCHEME_MAILTO; // island enum // gen:d9b6163a4f41
pub const URL_SCHEME_SHELL: URL_SCHEME = URL_SCHEME_URL_SCHEME_SHELL; // island enum // gen:7233c4d8e9ae
pub const URL_SCHEME_VBSCRIPT: URL_SCHEME = URL_SCHEME_URL_SCHEME_VBSCRIPT; // island enum // gen:985773b1923e
pub const VT_ARRAY: VARENUM = VARENUM_VT_ARRAY; // island enum // gen:b7cf6ea32767
pub const VT_BOOL: VARENUM = VARENUM_VT_BOOL; // island enum // gen:c4d7afe1136c
pub const VT_BSTR: VARENUM = VARENUM_VT_BSTR; // island enum // gen:a461f7adde45
pub const VT_BYREF: VARENUM = VARENUM_VT_BYREF; // island enum // gen:39e8ae9dde5e
pub const VT_DECIMAL: VARENUM = VARENUM_VT_DECIMAL; // island enum // gen:e0a77b5247cd
pub const VT_DISPATCH: VARENUM = VARENUM_VT_DISPATCH; // island enum // gen:7be29f2c4786
pub const VT_I4: VARENUM = VARENUM_VT_I4; // island enum // gen:325d4c78ebc7
pub const VT_NULL: VARENUM = VARENUM_VT_NULL; // island enum // gen:4386074ba27a
pub const VT_UNKNOWN: VARENUM = VARENUM_VT_UNKNOWN; // island enum // gen:b467b4210365
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs // gen:845fcd2ce5ae
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // acppage-rust/src/prelude.rs // gen:f10f403758d0
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // activeds-rust/src/prelude.rs // gen:1e75f90b8cc3
    pub fn _wtoi(s: *const u16) -> i32; // chcp-rust/src/prelude.rs // gen:5780a4605643
    pub fn floor(x: f64) -> f64; // cnt-rust/src/cabi/math.rs // gen:76be2167a6a3
    pub fn isalnum(c: i32) -> i32; // cmd-rust/src/prelude.rs // gen:e6acc01ed9c4
    pub fn isdigit(c: i32) -> i32; // arp-rust/src/prelude.rs // gen:98c05ad9bc93
    pub fn islower(c: i32) -> i32; // cnt-ctype/src/lib.rs // gen:cf6a8cb93202
    pub fn iswctype(c: u16, mask: u16) -> c_int; // cmd-rust/src/prelude.rs // gen:7b02109a9720
    pub fn isxdigit(c: i32) -> i32; // arp-rust/src/prelude.rs // gen:e0fbb3c5fdfc
    pub fn tolower(c: c_int) -> c_int; // cmd-rust/src/prelude.rs // gen:a39b275333a1
    pub fn toupper(c: i32) -> i32; // apphelp-rust/src/prelude.rs // gen:0bc5a45681e8
    pub fn towupper(c: wint_t) -> wint_t; // at-rust/src/prelude.rs // gen:5ec8b0002df2
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs // gen:587742793a2c
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs // gen:6f1e26b8a292
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // activeds-rust/src/prelude.rs // gen:13391f57321c
    pub fn wcscspn(s: *const u16, set: *const u16) -> usize; // comctl32-rust/src/prelude.rs // gen:9dfd97657af6
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs // gen:1246e81e5c62
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // apphelp-rust/src/prelude.rs // gen:12a747e973fa
    pub fn wcspbrk(s: *const u16, set: *const u16) -> *const u16; // apphelp-rust/src/prelude.rs // gen:9e2b24ec05d2
    pub fn wcsspn(s: *const u16, accept: *const u16) -> usize; // doskey-rust/src/prelude.rs // gen:c3d1584de98b
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // appwiz-rust/src/prelude.rs // gen:27e3f6f3f416
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:4bffac11aa02
    pub static IID_IStream: GUID; // aclui-rust/src/types_amd64.rs // gen:fa5c22881679
}
extern "system" {
    pub fn AddAccessAllowedAce(arg1: PACL, arg2: DWORD, arg3: DWORD, arg4: PSID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:3568d495adae
    pub fn AddAccessAllowedAceEx( arg1: PACL, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: PSID, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:6affab193d07
    pub fn AddAccessDeniedAce(arg1: PACL, arg2: DWORD, arg3: DWORD, arg4: PSID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:639bee657098
    pub fn AddAccessDeniedAceEx( arg1: PACL, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: PSID, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:8a1da649a0fd
    pub fn AllocateAndInitializeSid( arg1: PSID_IDENTIFIER_AUTHORITY, arg2: BYTE, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: DWORD, arg9: DWORD, arg10: DWORD, arg11: *mut PSID, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5b2895e03b1d
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs // gen:027bd053c436
    pub fn CharNextA(arg1: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:37a4868db9f3
    pub fn CharPrevA(arg1: LPCSTR, arg2: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:27af408a3759
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_i386.rs // gen:57a58cacce1f
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4a43278b42fd
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs // gen:37c979b076a1
    pub fn CoGetClassObject( rclsid: *const CLSID, dwClsContext: DWORD, pServerInfo: *mut COSERVERINFO, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs // gen:5b93887992d0
    pub fn CoInitializeEx(lpReserved: LPVOID, dwCoInit: DWORD) -> HRESULT; // aclui-rust/src/types_i386.rs // gen:2431352e9bcb
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs // gen:08edf66ac946
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs // gen:846a5162273d
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs // gen:663c56975056
    pub fn CompareStringA( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:8863f3372ac1
    pub fn CompareStringW( Locale: LCID, dwCmpFlags: DWORD, lpString1: LPCWSTR, cchCount1: ::core::ffi::c_int, lpString2: LPCWSTR, cchCount2: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:56e947675cd0
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs // gen:791328af02e4
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:f162982b6326
    pub fn CreateFileMappingA( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:f59cf4d2e729
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:90b2fdf62411
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_i386.rs // gen:6613fc3d512a
    pub fn CreateHalftonePalette(arg1: HDC) -> HPALETTE; // aclui-rust/src/types_i386.rs // gen:b7a75164f345
    pub fn CreatePalette(arg1: *const LOGPALETTE) -> HPALETTE; // aclui-rust/src/types_i386.rs // gen:089016e1e9dc
    pub fn CreateSemaphoreW( arg1: LPSECURITY_ATTRIBUTES, arg2: LONG, arg3: LONG, arg4: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:a8f0a29b02d6
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:29214eec4ab3
    pub fn CreateTimerQueueTimer( arg1: PHANDLE, arg2: HANDLE, arg3: WAITORTIMERCALLBACK, arg4: PVOID, arg5: DWORD, arg6: DWORD, arg7: ULONG, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a51e2406f1a1
    pub fn CreateWindowExA( dwExStyle: DWORD, lpClassName: LPCSTR, lpWindowName: LPCSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs // gen:2a5bd341d50b
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs // gen:13b93f968a36
    pub fn DefWindowProcA(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:68bcc01160cf
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:aca94375b75d
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:7893eb6ab4e4
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs // gen:879520c6bc24
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:000f9d795415
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5f33e0f590fb
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs // gen:ff91da798971
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:111127023c7f
    pub fn DragQueryFileW(hDrop: HDROP, iFile: UINT, lpszFile: LPWSTR, cch: UINT) -> UINT; // acppage-rust/src/types_i386.rs // gen:37811971a12f
    pub fn DuplicateHandle( arg1: HANDLE, arg2: HANDLE, arg3: HANDLE, arg4: PHANDLE, arg5: DWORD, arg6: BOOL, arg7: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:adcab531758b
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:2b056abaf099
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:0772d2cbe8ed
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:b4d73724c7aa
    pub fn EnumChildWindows(arg1: HWND, arg2: WNDENUMPROC, arg3: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs // gen:15c6f8ba6002
    pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5f83927fc162
    pub fn ExpandEnvironmentStringsA(arg1: LPCSTR, arg2: LPSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:61cdfe0cffd8
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:5d86047a10cc
    pub fn ExtTextOutA( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, options: UINT, lprect: *const RECT, lpString: LPCSTR, c: UINT, lpDx: *const INT, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ea103f378e3d
    pub fn ExtractIconW(hInst: HINSTANCE, pszExeFileName: LPCWSTR, nIconIndex: UINT) -> HICON; // acppage-rust/src/types_i386.rs // gen:177a1f969d21
    pub fn FileTimeToLocalFileTime(arg1: *const FILETIME, arg2: LPFILETIME) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a287c77c8b41
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ad07598c9e6d
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:0f80793a5268
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:08dcd32fadea
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_i386.rs // gen:1a65a8f74265
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs // gen:a7412a9e388e
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:fcc80d36382e
    pub fn FreeSid(arg1: PSID) -> PVOID; // aclui-rust/src/types_i386.rs // gen:2c57e8cd9b64
    pub fn GetClassInfoA(arg1: HINSTANCE, arg2: LPCSTR, arg3: LPWNDCLASSA) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4695f0b2b3d7
    pub fn GetClassInfoW(arg1: HINSTANCE, arg2: LPCWSTR, arg3: LPWNDCLASSW) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9190407e04dc
    pub fn GetClassNameA( hWnd: HWND, lpClassName: LPSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:a680c4066ae3
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ef10fc0cc6f5
    pub fn GetCurrentHwProfileA(arg1: LPHW_PROFILE_INFOA) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4fff8b4af7fe
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:0dfc645e02a4
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs // gen:5381fab34a2f
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs // gen:39c5b894d19a
    pub fn GetDIBits( hdc: HDC, hbm: HBITMAP, start: UINT, cLines: UINT, lpvBits: LPVOID, lpbmi: LPBITMAPINFO, usage: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:501e605948b1
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:e5a81696b92f
    pub fn GetDeviceCaps(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:6d490a0cfd9b
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs // gen:102d2f98c4f8
    pub fn GetEnvironmentVariableW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:320968944414
    pub fn GetFileAttributesA(lpFileName: LPCSTR) -> DWORD; // aclui-rust/src/types_i386.rs // gen:c3caa0621a20
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs // gen:9a4eff018916
    pub fn GetFileInformationByHandle(arg1: HANDLE, arg2: LPBY_HANDLE_FILE_INFORMATION) -> BOOL; // aclui-rust/src/types_i386.rs // gen:51d18f8fc8ff
    pub fn GetFileVersionInfoA(arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: PVOID) -> BOOL; // drwtsn32-rust/src/types_i386.rs // gen:29bd643d9213
    pub fn GetFileVersionInfoSizeA(arg1: LPCSTR, arg2: PDWORD) -> DWORD; // drwtsn32-rust/src/types_i386.rs // gen:dfe07d18402d
    pub fn GetFileVersionInfoSizeW(arg1: LPCWSTR, arg2: PDWORD) -> DWORD; // drwtsn32-rust/src/types_i386.rs // gen:900d32fd18f5
    pub fn GetFileVersionInfoW(arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: PVOID) -> BOOL; // drwtsn32-rust/src/types_i386.rs // gen:30676ff976f3
    pub fn GetFullPathNameA(arg1: LPCSTR, arg2: DWORD, arg3: LPSTR, arg4: *mut LPSTR) -> DWORD; // aclui-rust/src/types_i386.rs // gen:77233b22cdee
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs // gen:496f60eb8c23
    pub fn GetIconInfo(arg1: HICON, arg2: PICONINFO) -> BOOL; // aclui-rust/src/types_i386.rs // gen:1a504cdda83b
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs // gen:50e246193424
    pub fn GetLengthSid(arg1: PSID) -> DWORD; // aclui-rust/src/types_i386.rs // gen:858c800d61a4
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:89449470c118
    pub fn GetMenuItemCount(arg1: HMENU) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:ef15aafc1058
    pub fn GetMenuItemInfoW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9e4d6ffce0dd
    pub fn GetModuleFileNameA(hModule: HINSTANCE, lpFilename: LPSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:51bffb082f29
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:43bdbe105f12
    pub fn GetNumberFormatW( Locale: LCID, dwFlags: DWORD, lpValue: LPCWSTR, lpFormat: *const NUMBERFMTW, lpNumberStr: LPWSTR, cchNumber: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:0b047346ac3c
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:aec63b366473
    pub fn GetOpenFileNameW(arg1: LPOPENFILENAMEW) -> BOOL; // desk-rust/src/types_i386.rs // gen:21ae51e78a6e
    pub fn GetPaletteEntries( hpal: HPALETTE, iStart: UINT, cEntries: UINT, pPalEntries: LPPALETTEENTRY, ) -> UINT; // aclui-rust/src/types_i386.rs // gen:61dd82692c40
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs // gen:b32eec491703
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs // gen:2f213353c565
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs // gen:8b50fa1e44f3
    pub fn GetProcessVersion(arg1: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:39c9b91da6ed
    pub fn GetPropA(arg1: HWND, arg2: LPCSTR) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:c4348930e427
    pub fn GetPropW(arg1: HWND, arg2: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:cfdf49a723d0
    pub fn GetSaveFileNameW(arg1: LPOPENFILENAMEW) -> BOOL; // desk-rust/src/types_i386.rs // gen:818d5208a0f6
    pub fn GetStringTypeA( Locale: LCID, dwInfoType: DWORD, lpSrcStr: LPCSTR, cchSrc: ::core::ffi::c_int, lpCharType: LPWORD, ) -> BOOL; // desk-rust/src/types_i386.rs // gen:8e18867bcb3e
    pub fn GetStringTypeW( dwInfoType: DWORD, lpSrcStr: LPCWSTR, cchSrc: ::core::ffi::c_int, lpCharType: LPWORD, ) -> BOOL; // desk-rust/src/types_i386.rs // gen:28898547624c
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs // gen:96a3f8dc3fb9
    pub fn GetSystemDefaultLCID() -> LCID; // desk-rust/src/types_i386.rs // gen:fa2baee89b17
    pub fn GetSystemDefaultUILanguage() -> LANGID; // desk-rust/src/types_i386.rs // gen:0b1682488471
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs // gen:c8aa2a87df2b
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:33a0dab07dca
    pub fn GetSystemPaletteEntries( hdc: HDC, iStart: UINT, cEntries: UINT, pPalEntries: LPPALETTEENTRY, ) -> UINT; // aclui-rust/src/types_i386.rs // gen:ac45e937228b
    pub fn GetSystemWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs // gen:b80cf4ec0187
    pub fn GetTextExtentPointW( hdc: HDC, lpString: LPCWSTR, c: ::core::ffi::c_int, lpsz: LPSIZE, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:6f10bf4d47f4
    pub fn GetThreadLocale() -> LCID; // desk-rust/src/types_i386.rs // gen:e54bf57bfcbf
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs // gen:f624a448a1f6
    pub fn GetTimeFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:86df0163d090
    pub fn GetTokenInformation( arg1: HANDLE, arg2: TOKEN_INFORMATION_CLASS, arg3: PVOID, arg4: DWORD, arg5: PDWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:b5f5325287e0
    pub fn GetUserDefaultLCID() -> LCID; // desk-rust/src/types_i386.rs // gen:4bcb488108d7
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_i386.rs // gen:0a7a5ee21a72
    pub fn GetVersion() -> DWORD; // aclui-rust/src/types_i386.rs // gen:4652a0b3e9c8
    pub fn GetVersionExA(arg1: LPOSVERSIONINFOA) -> BOOL; // aclui-rust/src/types_i386.rs // gen:21a88984aff7
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs // gen:79545aae4953
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs // gen:f8e45f883779
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:6e6ec69d00ea
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs // gen:6b988ffe950e
    pub fn GlobalMemoryStatus(arg1: LPMEMORYSTATUS); // aclui-rust/src/types_i386.rs // gen:c87a175956e7
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:3aaa7317691b
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:cd40b76d9b6b
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs // gen:13245079d396
    pub fn InitializeAcl(arg1: PACL, arg2: DWORD, arg3: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c6b99e1f6f8f
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:4cc9a88dd118
    pub fn InitializeSecurityDescriptor(arg1: PSECURITY_DESCRIPTOR, arg2: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:729c1a54fbfc
    pub fn IsBadStringPtrA(arg1: LPCSTR, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:570c69b1d448
    pub fn IsBadStringPtrW(arg1: LPCWSTR, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ed0df7d3f454
    pub fn IsBadWritePtr(arg1: PVOID, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:51ceeac7b1ed
    pub fn IsChild(arg1: HWND, arg2: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9e5d054d1c63
    pub fn IsDBCSLeadByte(arg1: BYTE) -> BOOL; // desk-rust/src/types_i386.rs // gen:adf822ea948c
    pub fn IsNetDrive(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // acppage-rust/src/types_i386.rs // gen:fb723f08962d
    pub fn IsWindowUnicode(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs // gen:53b47819bd0e
    pub fn IsWow64Process(arg1: HANDLE, arg2: PBOOL) -> BOOL; // aclui-rust/src/types_i386.rs // gen:6144d25fd435
    pub fn LcidToRfc1766W(lcid: u32, pszRfc1766: *mut u16, nChar: i32,) -> i32; // mlang-rust/src/mlang/mlang.rs // gen:568609271233
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs // gen:d32c25758174
    pub fn LoadCursorA(arg1: HINSTANCE, arg2: LPCSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs // gen:ca3ae8b0eca5
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs // gen:88639451dc1c
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs // gen:fd1cf5811182
    pub fn LoadLibraryExA(arg1: LPCSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_i386.rs // gen:420db4c19c0a
    pub fn LoadLibraryExW(arg1: LPCWSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_i386.rs // gen:04330743bd1e
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs // gen:5bdbdeebfcb7
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs // gen:48f45fc09854
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:cb1bdfb02278
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs // gen:68642d9620a9
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs // gen:7fcc067b765c
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL; // aclui-rust/src/types_i386.rs // gen:bb008ec0b758
    pub fn LocalSize(arg1: HLOCAL) -> SIZE_T; // aclui-rust/src/types_i386.rs // gen:3936c9775728
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs // gen:dbd1b9e35493
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:b02599d26998
    pub fn MsgWaitForMultipleObjectsEx( nCount: DWORD, pHandles: *const HANDLE, dwMilliseconds: DWORD, dwWakeMask: DWORD, dwFlags: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs // gen:aff79228c265
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:364345b52ec2
    pub fn OpenProcess(arg1: DWORD, arg2: BOOL, arg3: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:1a246a26c648
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:b240d6950849
    pub fn OpenSemaphoreW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:c257a7bb7bf0
    pub fn PageSetupDlgW(arg1: LPPAGESETUPDLGW) -> BOOL; // desk-rust/src/types_i386.rs // gen:666f151d70b6
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:8662c61ca102
    pub fn PlaySoundW(arg1: LPCWSTR, arg2: HMODULE, arg3: DWORD) -> BOOL; // dsound-rust/src/types_i386.rs // gen:07eac25cb3fe
    pub fn PrintDlgW(arg1: LPPRINTDLGW) -> BOOL; // desk-rust/src/types_i386.rs // gen:9bf86685e6c0
    pub fn QueryPerformanceCounter(arg1: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs // gen:e1a525d90389
    pub fn QueryPerformanceFrequency(arg1: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs // gen:9475484b8364
    pub fn QueueUserWorkItem(arg1: LPTHREAD_START_ROUTINE, arg2: PVOID, arg3: ULONG) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ce48ecbbe986
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:49bda6930c6d
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:d83c13224bef
    pub fn RegCreateKeyExA( hKey: HKEY, lpSubKey: LPCSTR, Reserved: DWORD, lpClass: LPSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:a9cd1c4e99f2
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:987a8cb30309
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:d7a8638de5e7
    pub fn RegDeleteKeyA(hKey: HKEY, lpSubKey: LPCSTR) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:f0125c1c036d
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:76786a34a2a4
    pub fn RegDeleteValueA(arg1: HKEY, arg2: LPCSTR) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:cc889005a0ac
    pub fn RegDeleteValueW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:305078a7d5c4
    pub fn RegEnumKeyA(hKey: HKEY, dwIndex: DWORD, lpName: LPSTR, cchName: DWORD) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:86f9323c9285
    pub fn RegEnumKeyExA( hKey: HKEY, dwIndex: DWORD, lpName: LPSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:b803d8f05287
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:8762af9c2c00
    pub fn RegEnumValueA( hKey: HKEY, dwIndex: DWORD, lpValueName: LPSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:bc7328e7d80e
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:9e72c4863aed
    pub fn RegGetValueW( hkey: HKEY, lpSubKey: LPCWSTR, lpValue: LPCWSTR, dwFlags: DWORD, pdwType: LPDWORD, pvData: PVOID, pcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:6b953dafc98b
    pub fn RegOpenKeyA(arg1: HKEY, arg2: LPCSTR, arg3: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:86e841a4c587
    pub fn RegOpenKeyExA( arg1: HKEY, arg2: LPCSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:e57575460c08
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:a5045db5aff0
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:e1e8c8c8bfe6
    pub fn RegQueryInfoKeyW( hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:39a7cbd12206
    pub fn RegQueryValueExA( hKey: HKEY, lpValueName: LPCSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:6bcb7f4421b5
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:cef016468a4f
    pub fn RegSetValueExA( hKey: HKEY, lpValueName: LPCSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:dc4e7a64bf23
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // ddraw-rust/src/types_i386.rs // gen:0e225f333c0f
    pub fn RegisterClassA(arg1: *const WNDCLASSA) -> ATOM; // aclui-rust/src/types_i386.rs // gen:c110e5303288
    pub fn RegisterClassW(arg1: *const WNDCLASSW) -> ATOM; // aclui-rust/src/types_i386.rs // gen:a0f3499ecb71
    pub fn RegisterClipboardFormatA(arg1: LPCSTR) -> UINT; // aclui-rust/src/types_i386.rs // gen:49d09c1d2697
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:0952239cd2d2
    pub fn ReleaseSemaphore(arg1: HANDLE, arg2: LONG, arg3: LPLONG) -> BOOL; // aclui-rust/src/types_i386.rs // gen:1ea006080465
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs // gen:29459fc6ec50
    pub fn RemovePropA(arg1: HWND, arg2: LPCSTR) -> HANDLE; // aclui-rust/src/types_i386.rs // gen:c20ae408e095
    pub fn SHBrowseForFolderW(lpbi: LPBROWSEINFOW) -> PIDLIST_ABSOLUTE; // acppage-rust/src/types_i386.rs // gen:8eed51518205
    pub fn SHChangeNotify(wEventId: LONG, uFlags: UINT, dwItem1: LPCVOID, dwItem2: LPCVOID); // acppage-rust/src/types_i386.rs // gen:56b5dca644de
    pub fn SHCoCreateInstance( arg1: LPCWSTR, arg2: *const CLSID, arg3: *mut IUnknown, arg4: *const IID, arg5: *mut LPVOID, ) -> HRESULT; // acppage-rust/src/types_i386.rs // gen:40317d14ff89
    pub fn SHDefExtractIconW( pszIconFile: LPCWSTR, iIndex: ::core::ffi::c_int, uFlags: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT, ) -> HRESULT; // acppage-rust/src/types_i386.rs // gen:a95241273f11
    pub fn SHFileOperationW(arg1: LPSHFILEOPSTRUCTW) -> ::core::ffi::c_int; // acppage-rust/src/types_i386.rs // gen:359b3f5a4867
    pub fn SHGetFileInfoW( pszPath: LPCWSTR, dwFileAttributes: DWORD, psfi: *mut SHFILEINFOW, cbFileInfo: UINT, uFlags: UINT, ) -> DWORD_PTR; // acppage-rust/src/types_i386.rs // gen:ea5492b52f3a
    pub fn SHGetInstanceExplorer(arg1: *mut *mut IUnknown) -> HRESULT; // acppage-rust/src/types_i386.rs // gen:3e020dd17e7b
    pub fn SHGetNewLinkInfoW( pszLinkTo: LPCWSTR, pszDir: LPCWSTR, pszName: LPWSTR, pfMustCopy: *mut BOOL, uFlags: UINT, ) -> BOOL; // acppage-rust/src/types_i386.rs // gen:f680f4d6a83a
    pub fn SHGetPathFromIDListW(arg1: PCIDLIST_ABSOLUTE, arg2: LPWSTR) -> BOOL; // acppage-rust/src/types_i386.rs // gen:2c52d853e7a9
    pub fn SafeArrayAccessData( arg1: *mut SAFEARRAY, arg2: *mut *mut ::core::ffi::c_void, ) -> HRESULT; // msctf-rust/src/types_i386.rs // gen:a317d39637f0
    pub fn SafeArrayGetDim(arg1: *mut SAFEARRAY) -> UINT; // msctf-rust/src/types_i386.rs // gen:b863a6b798a6
    pub fn SafeArrayGetLBound(arg1: *mut SAFEARRAY, arg2: UINT, arg3: *mut LONG) -> HRESULT; // msctf-rust/src/types_i386.rs // gen:41abbb0c0670
    pub fn SafeArrayGetUBound(arg1: *mut SAFEARRAY, arg2: UINT, arg3: *mut LONG) -> HRESULT; // msctf-rust/src/types_i386.rs // gen:db5b48036143
    pub fn SafeArrayUnaccessData(arg1: *mut SAFEARRAY) -> HRESULT; // msctf-rust/src/types_i386.rs // gen:ba1bb9061edb
    pub fn SearchPathA( lpPath: LPCSTR, lpFileName: LPCSTR, lpExtension: LPCSTR, nBufferLength: DWORD, lpBuffer: LPSTR, lpFilePart: *mut LPSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs // gen:968f04902dba
    pub fn SearchPathW( lpPath: LPCWSTR, lpFileName: LPCWSTR, lpExtension: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs // gen:48975d036414
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs // gen:6f8d653c63e8
    pub fn SendMessageTimeoutA( arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM, arg5: UINT, arg6: UINT, arg7: PDWORD_PTR, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:0da1f448c217
    pub fn SendMessageTimeoutW( arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM, arg5: UINT, arg6: UINT, arg7: PDWORD_PTR, ) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:5c8462911a8d
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs // gen:9028b9ea3245
    pub fn SetBkColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs // gen:22a8c2b814b1
    pub fn SetEndOfFile(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:43dfd8404fb7
    pub fn SetErrorMode(arg1: UINT) -> UINT; // aclui-rust/src/types_i386.rs // gen:5d4cd40b4dce
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:198f85e2baee
    pub fn SetFileAttributesA(lpFileName: LPCSTR, dwFileAttributes: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4b89b3f59e3c
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:81c6ae746086
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:5bf26683f6db
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs // gen:08f4ac87288b
    pub fn SetParent(arg1: HWND, arg2: HWND) -> HWND; // aclui-rust/src/types_i386.rs // gen:64886cd53ce0
    pub fn SetPropW(arg1: HWND, arg2: LPCWSTR, arg3: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:543c23f38d91
    pub fn SetSecurityDescriptorDacl( arg1: PSECURITY_DESCRIPTOR, arg2: BOOL, arg3: PACL, arg4: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:f3432952b9cd
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongA")]
    pub fn SetWindowLongPtrA(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> LONG_PTR; // winhlp32-rust/src/prelude.rs // gen:466eb41b0730
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs // gen:07cfdf4cb040
    pub fn SetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LONG) -> LONG; // aclui-rust/src/types_i386.rs // gen:d2848a99266a
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs // gen:ae46b8770144
    pub fn ShellExecuteExW(arg1: LPSHELLEXECUTEINFOW) -> BOOL; // acppage-rust/src/types_i386.rs // gen:03f45e880901
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs // gen:a99a94e9f6ba
    pub fn StringFromCLSID(id: *const CLSID, arg1: *mut LPOLESTR) -> HRESULT; // aclui-rust/src/types_i386.rs // gen:0946afa6da89
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs // gen:03b4c5b87ae3
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs // gen:08ee99caa9e6
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:4749b0b0b1c0
    pub fn TlsAlloc() -> DWORD; // aclui-rust/src/types_i386.rs // gen:67961a36e037
    pub fn TlsFree(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs // gen:c939accad583
    pub fn TlsGetValue(arg1: DWORD) -> PVOID; // aclui-rust/src/types_i386.rs // gen:b6c80b186b22
    pub fn TlsSetValue(arg1: DWORD, arg2: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:3adc17cda88c
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs // gen:80b0457f5c41
    pub fn UnregisterClassA(arg1: LPCSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:d14f916dd30c
    pub fn UnregisterClassW(arg1: LPCWSTR, arg2: HINSTANCE) -> BOOL; // aclui-rust/src/types_i386.rs // gen:840c9d3f3b80
    pub fn VariantChangeType( arg1: *mut VARIANTARG, arg2: *const VARIANTARG, arg3: USHORT, arg4: ::core::ffi::c_ushort, ) -> HRESULT; // msctf-rust/src/types_i386.rs // gen:2b9e75060127
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs // gen:8c6dc22b6e59
    pub fn VariantInit(arg1: *mut VARIANT); // msctf-rust/src/types_i386.rs // gen:fca98c79a4f1
    pub fn VerQueryValueA(arg1: LPCVOID, arg2: LPCSTR, arg3: *mut LPVOID, arg4: PUINT) -> BOOL; // drwtsn32-rust/src/types_i386.rs // gen:e47f87729d0a
    pub fn VerQueryValueW(arg1: LPCVOID, arg2: LPCWSTR, arg3: *mut LPVOID, arg4: PUINT) -> BOOL; // drwtsn32-rust/src/types_i386.rs // gen:259caf971476
    pub fn WNetGetLastErrorW( lpError: LPDWORD, lpErrorBuf: LPWSTR, nErrorBufSize: u32, lpNameBuf: LPWSTR, nNameBufSize: u32, ) -> DWORD; // shell32-rust/src/island_i386.rs // gen:c81c987c8597
    pub fn WNetRestoreConnectionW(arg1: HWND, arg2: LPCWSTR) -> DWORD; // shell32-rust/src/island_i386.rs // gen:cf65b733324d
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs // gen:0d4f240a125e
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs // gen:0e129f7cfff2
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs // gen:5af3a6d97065
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:09f7e4d3dd9f
    pub fn lstrcmpA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:6254df28289b
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:4862658b57c4
    pub fn lstrcmpiA(arg1: LPCSTR, arg2: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:2950b2f3c515
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:f3454dea79d1
    pub fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:035558f8c98c
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:eea6eb419b40
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_i386.rs // gen:dcf787d63821
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs // gen:0abd4a9770ae
    pub fn lstrlenA(arg1: LPCSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:144411e62218
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs // gen:244f3df56f54
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::shlwapi::path::PathCombineA;
pub use crate::shlwapi::path::PathCombineW;
pub use crate::shlwapi::path::PathFindFileNameA;
pub use crate::shlwapi::path::PathRemoveFileSpecA;
pub use crate::shlwapi::reg::SHGetValueA;
pub use crate::shlwapi::reg::SHGetValueW;
pub use crate::shlwapi::shlwapi_main::SHLWAPI_ThreadRef_index;
pub use crate::shlwapi::string::SHLoadIndirectString;
pub use crate::shlwapi::reg::SHQueryValueExW;
pub use crate::shlwapi::reg::SHRegGetBoolUSValueW;
pub use crate::shlwapi::reg::SHRegSetUSValueW;
pub use crate::shlwapi::string::StrCatBuffW;
pub use crate::shlwapi::string::StrChrA;
pub use crate::shlwapi::string::StrChrW;
pub use crate::shlwapi::string::StrCmpNA;
pub use crate::shlwapi::string::StrCmpNIA;
pub use crate::shlwapi::string::StrCmpNIW;
pub use crate::shlwapi::string::StrCmpNW;
pub use crate::shlwapi::string::StrCpyNW;
pub use crate::shlwapi::string::StrCpyW;
pub use crate::shlwapi::string::StrDupA;
pub use crate::shlwapi::string::StrDupW;
pub use crate::shlwapi::string::StrToIntA;
pub use crate::shlwapi::string::StrToIntExW;
pub use crate::shlwapi::string::StrToIntW;
pub use crate::shlwapi::url::UrlUnescapeW;
pub use crate::shlwapi::shlwapi_main::shlwapi_hInstance;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// PERCEIVED: c2rust, assoc.c
pub type PERCEIVED = tagPERCEIVED;
// tagAPPCOMPATINFO: c2rust, appcompat.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagAPPCOMPATINFO {
    pub pszAppName: PCSTR,
    pub pszAppVersion: PCSTR,
    pub dwCompatFlags: DWORD,
}
// tagFLAGMAP: c2rust, appcompat.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagFLAGMAP {
    pub flags: DWORD,
    pub name: LPCSTR,
}
// tagSHLWAPI_BYTEFORMATS: c2rust, string.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagSHLWAPI_BYTEFORMATS {
    pub dLimit: LONGLONG,
    pub dDivisor: ::core::ffi::c_double,
    pub dNormaliser: ::core::ffi::c_double,
    pub nDecimals: ::core::ffi::c_int,
    pub nFormatID: UINT,
}
// tagWNDCOMPATINFO: c2rust, appcompat.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagWNDCOMPATINFO {
    pub pszLengthAndClassName: PCSTR,
    pub dwCompatFlags: DWORD,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snwprintf(
        _Dest: *mut wchar_t,
        _Count: size_t,
        _Format: *const wchar_t,
        ...
    ) -> ::core::ffi::c_int; // string.c
    pub fn _stricmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // path.c
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // ordinal.c
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // appcompat.c
    pub fn sprintf(
        _Dest: *mut ::core::ffi::c_char,
        _Format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // ordinal.c
    pub fn strcat(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // ordinal.c
    pub fn strchr(
        _Str: *const ::core::ffi::c_char,
        _Val: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // path.c
    pub fn strcmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int; // path.c
    pub fn strcpy(
        _Dest: *mut ::core::ffi::c_char,
        _Source: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char; // ordinal.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // ordinal.c
    pub fn strncmp(
        _Str1: *const ::core::ffi::c_char,
        _Str2: *const ::core::ffi::c_char,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // url.c
    pub fn strrchr(
        _Str: *const ::core::ffi::c_char,
        _Ch: ::core::ffi::c_int,
    ) -> *mut ::core::ffi::c_char; // ordinal.c
    pub fn strtol(
        _Str: *const ::core::ffi::c_char,
        _EndPtr: *mut *mut ::core::ffi::c_char,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_long; // url.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // path.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // ordinal.c
}
extern "system" {
    pub fn ConvertINetUnicodeToMultiByte(
        _: LPDWORD,
        _: DWORD,
        _: LPCWSTR,
        _: LPINT,
        _: LPSTR,
        _: LPINT,
    ) -> HRESULT; // string.c
    pub fn CreateFormatEnumerator(_: UINT, _: *mut FORMATETC, _: *mut *mut IEnumFORMATETC) -> HRESULT; // ordinal.c
    pub fn RegisterFormatEnumerator(_: LPBC, _: *mut IEnumFORMATETC, _: DWORD) -> HRESULT; // ordinal.c
}
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::lstrcpynA as StrCpyNA; // sdk alias
pub use crate::prelude::_snwprintf as snprintfW; // sdk alias
pub use crate::prelude::_swprintf as sprintfW; // sdk alias
pub use crate::prelude::_stricmp as strcasecmp; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_shell: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x73\x68\x65\x6c\x6c\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_shell };
// ==== end wine debug channels ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! HRESULT_CODE { ($p0:expr $(,)?) => { ($p0 & 0xFFFF) }; } // gen:cf237b6d05e9
#[macro_export] macro_rules! IQueryAssociations_GetKey { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr $(,)?) => { ((*(*$p0).lpVtbl).GetKey).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _)) }; } // gen:76942a27d739
#[macro_export] macro_rules! IQueryAssociations_GetString { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr $(,)?) => { ((*(*$p0).lpVtbl).GetString).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _)) }; } // gen:29cbe7992cf3
#[macro_export] macro_rules! IQueryAssociations_Init { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr $(,)?) => { ((*(*$p0).lpVtbl).Init).unwrap_unchecked()((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _)) }; } // gen:6bab3b17c250
#[macro_export] macro_rules! IQueryAssociations_Release { ($p0:expr $(,)?) => { ((*(*$p0).lpVtbl).Release).unwrap_unchecked()((($p0) as _)) }; } // gen:e0b001377fe6
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:aa85287a8771
#[macro_export] macro_rules! MAKEDLLVERULL { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { (((((($p0) as ULONGLONG) << 48) | ((($p1) as ULONGLONG) << 32)) | ((($p2) as ULONGLONG) << 16)) | (($p3) as ULONGLONG)) }; } // gen:6411ee9a3522
#[macro_export] macro_rules! MAKE_HRESULT { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((((($p0) as c_uint) << 31) | ((($p1) as c_uint) << 16)) | (($p2) as c_uint))) as HRESULT) }; } // gen:829efe363f45
#[macro_export] macro_rules! V_ARRAY { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.parray }; } // gen:3d65c5e7153c
#[macro_export] macro_rules! V_BOOL { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.boolVal }; } // gen:10deecc53a7c
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_I4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.lVal }; } // gen:2acba8bc60fa
#[macro_export] macro_rules! V_UI2 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.uiVal }; } // gen:107b1969f865
#[macro_export] macro_rules! V_UI4 { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.ulVal }; } // gen:419fb51bc82e
#[macro_export] macro_rules! V_UNKNOWN { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.punkVal }; } // gen:79ae51ee2046
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
#[macro_export] macro_rules! atoiW { ($p0:expr $(,)?) => { _wtoi((($p0) as _)) }; } // gen:cb82c5a9cf83
#[macro_export] macro_rules! isalnumW { ($p0:expr $(,)?) => { iswctype((($p0) as _), (((((0x0100 | 0x1) | 0x2) | 0x4)) as _)) }; } // gen:e3717c787abe
#[macro_export] macro_rules! isalphaW { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((((0x0100 | 0x1) | 0x2)) as _)) }; } // gen:44303b2e327b
#[macro_export] macro_rules! isdigitW { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x4) as _)) }; } // gen:b54b8534a56c
#[macro_export] macro_rules! islowerW { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x2) as _)) }; } // gen:5f264a7d736c
#[macro_export] macro_rules! isxdigitW { ($p0:expr $(,)?) => { iswctype((($p0) as _), ((0x80) as _)) }; } // gen:713d0510825c
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:29bb71d79dfd
#[macro_export] macro_rules! memicmpW { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { _wcsnicmp((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:5e770392fe8d
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; } // gen:fb1d8a90940a
#[macro_export] macro_rules! strcatW { ($p0:expr, $p1:expr $(,)?) => { wcscat((($p0) as _), (($p1) as _)) }; } // gen:4beeb00700cf
#[macro_export] macro_rules! strchrW { ($p0:expr, $p1:expr $(,)?) => { wcschr((($p0) as _), (($p1) as _)) }; } // gen:8b2976cd818c
#[macro_export] macro_rules! strcmpW { ($p0:expr, $p1:expr $(,)?) => { wcscmp((($p0) as _), (($p1) as _)) }; } // gen:190f72af0ac0
#[macro_export] macro_rules! strcmpiW { ($p0:expr, $p1:expr $(,)?) => { _wcsicmp((($p0) as _), (($p1) as _)) }; } // gen:0fb11c747e64
#[macro_export] macro_rules! strcpyW { ($p0:expr, $p1:expr $(,)?) => { wcscpy((($p0) as _), (($p1) as _)) }; } // gen:e40c0222f10c
#[macro_export] macro_rules! strcspnW { ($p0:expr, $p1:expr $(,)?) => { wcscspn((($p0) as _), (($p1) as _)) }; } // gen:65776a5b1610
#[macro_export] macro_rules! strncmpW { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { wcsncmp((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:e63748b5d881
#[macro_export] macro_rules! strncmpiW { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { _wcsnicmp((($p0) as _), (($p1) as _), (($p2) as _)) }; } // gen:fc92b80e457d
#[macro_export] macro_rules! strpbrkW { ($p0:expr, $p1:expr $(,)?) => { wcspbrk((($p0) as _), (($p1) as _)) }; } // gen:ea1de03f9c3e
#[macro_export] macro_rules! strrchrW { ($p0:expr, $p1:expr $(,)?) => { wcsrchr((($p0) as _), (($p1) as _)) }; } // gen:e3c9b19b6a59
#[macro_export] macro_rules! strspnW { ($p0:expr, $p1:expr $(,)?) => { wcsspn((($p0) as _), (($p1) as _)) }; } // gen:4a255c2baf0d
#[macro_export] macro_rules! strstrW { ($p0:expr, $p1:expr $(,)?) => { wcsstr((($p0) as _), (($p1) as _)) }; } // gen:55d685185873
#[macro_export] macro_rules! toupperW { ($p0:expr $(,)?) => { towupper((($p0) as _)) }; } // gen:8a82fc31754e
pub const ANSI_NULL: CHAR = ((0) as CHAR); // gen:c71ec491a7a6
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const E_ACCESSDENIED: HRESULT = ((0x80070005u32) as HRESULT); // gen:97ace9e06777
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_POINTER: HRESULT = ((0x80004003u32) as HRESULT); // gen:f72eeacc5d42
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY); // gen:76d941b315d7
pub const HKEY_CURRENT_CONFIG: HKEY = ((((((0x80000005u32) as LONG)) as LONG_PTR)) as HKEY); // gen:9d32956405e7
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
pub const HKEY_DYN_DATA: HKEY = ((((((0x80000006u32) as LONG)) as LONG_PTR)) as HKEY); // gen:7b1db82feb3f
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY); // gen:6463b26ca00d
pub const HKEY_PERFORMANCE_DATA: HKEY = ((((((0x80000004u32) as LONG)) as LONG_PTR)) as HKEY); // gen:0a4f563aa909
pub const HKEY_USERS: HKEY = ((((((0x80000003u32) as LONG)) as LONG_PTR)) as HKEY); // gen:2df773c10cde
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND; // gen:7bbe1c304e55
#[inline] // gen:ac6dac8d11ae
pub unsafe fn IBindCtx_GetObjectParam(This: *mut IBindCtx, pszKey: LPOLESTR, ppunk: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObjectParam.unwrap_unchecked())(This, pszKey, ppunk) } // widl C wrapper
#[inline] // gen:c61e27f1df65
pub unsafe fn IConnectionPointContainer_FindConnectionPoint(This: *mut IConnectionPointContainer, riid: *const IID, ppCP: *mut *mut IConnectionPoint) -> HRESULT { ((*(*This).lpVtbl).FindConnectionPoint.unwrap_unchecked())(This, riid, ppCP) } // widl C wrapper
#[inline] // gen:9735df059a4c
pub unsafe fn IConnectionPointContainer_Release(This: *mut IConnectionPointContainer) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ec11aabc41eb
pub unsafe fn IConnectionPoint_Advise(This: *mut IConnectionPoint, pUnkSink: *mut IUnknown, pdwCookie: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).Advise.unwrap_unchecked())(This, pUnkSink, pdwCookie) } // widl C wrapper
#[inline] // gen:2fa8f7f06794
pub unsafe fn IConnectionPoint_EnumConnections(This: *mut IConnectionPoint, ppEnum: *mut *mut IEnumConnections) -> HRESULT { ((*(*This).lpVtbl).EnumConnections.unwrap_unchecked())(This, ppEnum) } // widl C wrapper
#[inline] // gen:2ae594580f76
pub unsafe fn IConnectionPoint_GetConnectionInterface(This: *mut IConnectionPoint, pIID: *mut IID) -> HRESULT { ((*(*This).lpVtbl).GetConnectionInterface.unwrap_unchecked())(This, pIID) } // widl C wrapper
#[inline] // gen:bec7969f18ef
pub unsafe fn IConnectionPoint_Release(This: *mut IConnectionPoint) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:481991cddfa5
pub unsafe fn IConnectionPoint_Unadvise(This: *mut IConnectionPoint, dwCookie: DWORD) -> HRESULT { ((*(*This).lpVtbl).Unadvise.unwrap_unchecked())(This, dwCookie) } // widl C wrapper
#[inline] // gen:d2395c2675fa
pub unsafe fn IContextMenu2_HandleMenuMsg(This: *mut IContextMenu2, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT { ((*(*This).lpVtbl).HandleMenuMsg.unwrap_unchecked())(This, uMsg, wParam, lParam) } // widl C wrapper
#[inline] // gen:0c02faf5410b
pub unsafe fn IContextMenu2_Release(This: *mut IContextMenu2) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d7ab12fab189
pub unsafe fn IContextMenu3_HandleMenuMsg2(This: *mut IContextMenu3, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, plResult: *mut LRESULT) -> HRESULT { ((*(*This).lpVtbl).HandleMenuMsg2.unwrap_unchecked())(This, uMsg, wParam, lParam, plResult) } // widl C wrapper
#[inline] // gen:116d7d06482e
pub unsafe fn IContextMenu3_Release(This: *mut IContextMenu3) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const IDC_ARROW: ULONG_PTR = ((((32512) as WORD)) as ULONG_PTR); // gen:1fd690149ef3
#[inline] // gen:0e6599a31c2e
pub unsafe fn IDispatch_Invoke(This: *mut IDispatch, dispIdMember: DISPID, riid: *const IID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT { ((*(*This).lpVtbl).Invoke.unwrap_unchecked())(This, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr) } // widl C wrapper
#[inline] // gen:2dd34cc31a72
pub unsafe fn IDispatch_Release(This: *mut IDispatch) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:cdd4209ca2c9
pub unsafe fn IDocHostUIHandler_EnableModeless(This: *mut IDocHostUIHandler, fEnable: BOOL) -> HRESULT { ((*(*This).lpVtbl).EnableModeless.unwrap_unchecked())(This, fEnable) } // widl C wrapper
#[inline] // gen:a8ade706cd39
pub unsafe fn IDropTarget_DragEnter(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).DragEnter.unwrap_unchecked())(This, pDataObj, grfKeyState, pt, pdwEffect) } // widl C wrapper
#[inline] // gen:bf414b549805
pub unsafe fn IDropTarget_DragLeave(This: *mut IDropTarget) -> HRESULT { ((*(*This).lpVtbl).DragLeave.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:378f52e38624
pub unsafe fn IDropTarget_Drop(This: *mut IDropTarget, pDataObj: *mut IDataObject, grfKeyState: DWORD, pt: POINTL, pdwEffect: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).Drop.unwrap_unchecked())(This, pDataObj, grfKeyState, pt, pdwEffect) } // widl C wrapper
#[inline] // gen:1645fea352b8
pub unsafe fn IEnumConnections_Next(This: *mut IEnumConnections, cConnections: ULONG, rgcd: LPCONNECTDATA, pcFetched: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Next.unwrap_unchecked())(This, cConnections, rgcd, pcFetched) } // widl C wrapper
#[inline] // gen:b685a9d0fb25
pub unsafe fn IEnumConnections_Release(This: *mut IEnumConnections) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:a518ad42cc48
pub unsafe fn IEnumFORMATETC_Clone(This: *mut IEnumFORMATETC, ppenum: *mut *mut IEnumFORMATETC) -> HRESULT { ((*(*This).lpVtbl).Clone.unwrap_unchecked())(This, ppenum) } // widl C wrapper
#[inline] // gen:49439c43cd92
pub unsafe fn IEnumFORMATETC_Release(This: *mut IEnumFORMATETC) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:75b3a81fb9f1
pub unsafe fn IInputObjectSite_OnFocusChangeIS(This: *mut IInputObjectSite, lpUnknown: LPUNKNOWN, bFocus: BOOL) -> HRESULT { ((*(*This).lpVtbl).OnFocusChangeIS.unwrap_unchecked())(This, lpUnknown, bFocus) } // widl C wrapper
#[inline] // gen:8b433d4dea20
pub unsafe fn IInputObjectSite_Release(This: *mut IInputObjectSite) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:86ea9fb72101
pub unsafe fn IInputObject_HasFocusIO(This: *mut IInputObject) -> HRESULT { ((*(*This).lpVtbl).HasFocusIO.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0466e5d21a21
pub unsafe fn IInputObject_Release(This: *mut IInputObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:323720355300
pub unsafe fn IInputObject_TranslateAcceleratorIO(This: *mut IInputObject, lpMsg: LPMSG) -> HRESULT { ((*(*This).lpVtbl).TranslateAcceleratorIO.unwrap_unchecked())(This, lpMsg) } // widl C wrapper
#[inline] // gen:1752adef9b7f
pub unsafe fn IInputObject_UIActivateIO(This: *mut IInputObject, bActivating: BOOL, lpMsg: LPMSG) -> HRESULT { ((*(*This).lpVtbl).UIActivateIO.unwrap_unchecked())(This, bActivating, lpMsg) } // widl C wrapper
#[inline] // gen:86ed9463e787
pub unsafe fn IInternetSecurityManager_Release(This: *mut IInternetSecurityManager) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:ac866470e1fb
pub unsafe fn IInternetSecurityManager_SetSecuritySite(This: *mut IInternetSecurityManager, pSite: *mut IInternetSecurityMgrSite) -> HRESULT { ((*(*This).lpVtbl).SetSecuritySite.unwrap_unchecked())(This, pSite) } // widl C wrapper
#[inline] // gen:8b3e29bde19a
pub unsafe fn IInternetSecurityMgrSite_EnableModeless(This: *mut IInternetSecurityMgrSite, fEnable: BOOL) -> HRESULT { ((*(*This).lpVtbl).EnableModeless.unwrap_unchecked())(This, fEnable) } // widl C wrapper
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD); // gen:da773c86944d
pub const INVALID_SET_FILE_POINTER: DWORD = (((-(1 as c_int))) as DWORD); // gen:213ce84313f7
#[inline] // gen:0344623202d1
pub unsafe fn IObjectWithSite_GetSite(This: *mut IObjectWithSite, riid: *const IID, ppvSite: *mut PVOID) -> HRESULT { ((*(*This).lpVtbl).GetSite.unwrap_unchecked())(This, riid, ppvSite) } // widl C wrapper
#[inline] // gen:080577aa1ac2
pub unsafe fn IObjectWithSite_Release(This: *mut IObjectWithSite) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e132732e6e0a
pub unsafe fn IObjectWithSite_SetSite(This: *mut IObjectWithSite, pUnkSite: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetSite.unwrap_unchecked())(This, pUnkSite) } // widl C wrapper
#[inline] // gen:77df342103ff
pub unsafe fn IOleCommandTarget_Exec(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT { ((*(*This).lpVtbl).Exec.unwrap_unchecked())(This, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut) } // widl C wrapper
#[inline] // gen:50de878d0c7c
pub unsafe fn IOleCommandTarget_QueryStatus(This: *mut IOleCommandTarget, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT { ((*(*This).lpVtbl).QueryStatus.unwrap_unchecked())(This, pguidCmdGroup, cCmds, prgCmds, pCmdText) } // widl C wrapper
#[inline] // gen:3b56447766ad
pub unsafe fn IOleCommandTarget_Release(This: *mut IOleCommandTarget) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6b2ad61c3b6b
pub unsafe fn IOleControlSite_OnFocus(This: *mut IOleControlSite, fGotFocus: BOOL) -> HRESULT { ((*(*This).lpVtbl).OnFocus.unwrap_unchecked())(This, fGotFocus) } // widl C wrapper
#[inline] // gen:9ef08a005711
pub unsafe fn IOleControlSite_Release(This: *mut IOleControlSite) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:4b96cf2f4d1c
pub unsafe fn IOleControlSite_TranslateAccelerator(This: *mut IOleControlSite, pMsg: *mut MSG, grfModifiers: DWORD) -> HRESULT { ((*(*This).lpVtbl).TranslateAccelerator.unwrap_unchecked())(This, pMsg, grfModifiers) } // widl C wrapper
#[inline] // gen:834671b8e34f
pub unsafe fn IOleInPlaceActiveObject_EnableModeless(This: *mut IOleInPlaceActiveObject, fEnable: BOOL) -> HRESULT { ((*(*This).lpVtbl).EnableModeless.unwrap_unchecked())(This, fEnable) } // widl C wrapper
#[inline] // gen:78c44960daf4
pub unsafe fn IOleInPlaceFrame_EnableModeless(This: *mut IOleInPlaceFrame, fEnable: BOOL) -> HRESULT { ((*(*This).lpVtbl).EnableModeless.unwrap_unchecked())(This, fEnable) } // widl C wrapper
#[inline] // gen:c93df0b8635a
pub unsafe fn IOleWindow_GetWindow(This: *mut IOleWindow, phwnd: *mut HWND) -> HRESULT { ((*(*This).lpVtbl).GetWindow.unwrap_unchecked())(This, phwnd) } // widl C wrapper
#[inline] // gen:2d2a8f29212a
pub unsafe fn IParseDisplayName_ParseDisplayName(This: *mut IParseDisplayName, pbc: *mut IBindCtx, pszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppmkOut: *mut *mut IMoniker) -> HRESULT { ((*(*This).lpVtbl).ParseDisplayName.unwrap_unchecked())(This, pbc, pszDisplayName, pchEaten, ppmkOut) } // widl C wrapper
#[inline] // gen:3419e79287c1
pub unsafe fn IPersistPropertyBag_Load(This: *mut IPersistPropertyBag, pPropBag: *mut IPropertyBag, pErrorLog: *mut IErrorLog) -> HRESULT { ((*(*This).lpVtbl).Load.unwrap_unchecked())(This, pPropBag, pErrorLog) } // widl C wrapper
#[inline] // gen:c62c66abb6b8
pub unsafe fn IPersistPropertyBag_Release(This: *mut IPersistPropertyBag) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:f2801da44a6d
pub unsafe fn IPersist_GetClassID(This: *mut IPersist, pClassID: *mut CLSID) -> HRESULT { ((*(*This).lpVtbl).GetClassID.unwrap_unchecked())(This, pClassID) } // widl C wrapper
#[inline] // gen:022f3a02d470
pub unsafe fn IPersist_Release(This: *mut IPersist) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ef567db8e40
pub unsafe fn IProfferService_ProfferService(This: *mut IProfferService, service: *const GUID, pService: *mut IServiceProvider, pCookie: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).ProfferService.unwrap_unchecked())(This, service, pService, pCookie) } // widl C wrapper
#[inline] // gen:c58bbf569672
pub unsafe fn IProfferService_Release(This: *mut IProfferService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:cffe13117e07
pub unsafe fn IProfferService_RevokeService(This: *mut IProfferService, cookie: DWORD) -> HRESULT { ((*(*This).lpVtbl).RevokeService.unwrap_unchecked())(This, cookie) } // widl C wrapper
#[inline] // gen:4fc8c1efeb41
pub unsafe fn IPropertyBag_Read(This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT, pErrorLog: *mut IErrorLog) -> HRESULT { ((*(*This).lpVtbl).Read.unwrap_unchecked())(This, pszPropName, pVar, pErrorLog) } // widl C wrapper
#[inline] // gen:b69c86059611
pub unsafe fn IPropertyBag_Write(This: *mut IPropertyBag, pszPropName: LPCOLESTR, pVar: *mut VARIANT) -> HRESULT { ((*(*This).lpVtbl).Write.unwrap_unchecked())(This, pszPropName, pVar) } // widl C wrapper
#[inline] // gen:9c66587a52f4
pub unsafe fn IPropertyNotifySink_OnChanged(This: *mut IPropertyNotifySink, dispID: DISPID) -> HRESULT { ((*(*This).lpVtbl).OnChanged.unwrap_unchecked())(This, dispID) } // widl C wrapper
#[inline] // gen:c79a0350c10e
pub unsafe fn IPropertyNotifySink_Release(This: *mut IPropertyNotifySink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:f80fbd3db8b5
pub unsafe fn IQueryInfo_GetInfoFlags(This: *mut IQueryInfo, lpFlags: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).GetInfoFlags.unwrap_unchecked())(This, lpFlags) } // widl C wrapper
#[inline] // gen:0e608cfa40fd
pub unsafe fn IQueryInfo_Release(This: *mut IQueryInfo) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:95899110c9d2
pub unsafe fn IServiceProvider_QueryService(This: *mut IServiceProvider, guidService: *const GUID, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryService.unwrap_unchecked())(This, guidService, riid, ppvObject) } // widl C wrapper
#[inline] // gen:8748b95f2e30
pub unsafe fn IServiceProvider_Release(This: *mut IServiceProvider) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:f83222dcf8c9
pub unsafe fn IShellBrowser_EnableModelessSB(This: *mut IShellBrowser, fEnable: BOOL) -> HRESULT { ((*(*This).lpVtbl).EnableModelessSB.unwrap_unchecked())(This, fEnable) } // widl C wrapper
#[inline] // gen:ef699355d0a8
pub unsafe fn IShellFolder_EnumObjects(This: *mut IShellFolder, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT { ((*(*This).lpVtbl).EnumObjects.unwrap_unchecked())(This, hwndOwner, grfFlags, ppenumIDList) } // widl C wrapper
#[inline] // gen:61b6311d1652
pub unsafe fn IShellFolder_GetUIObjectOf(This: *mut IShellFolder, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: *const IID, prgfInOut: *mut UINT, ppvOut: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).GetUIObjectOf.unwrap_unchecked())(This, hwndOwner, cidl, apidl, riid, prgfInOut, ppvOut) } // widl C wrapper
#[inline] // gen:39aee485b8a3
pub unsafe fn IShellService_Release(This: *mut IShellService) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:134d9b80151d
pub unsafe fn IShellService_SetOwner(This: *mut IShellService, pUnk: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).SetOwner.unwrap_unchecked())(This, pUnk) } // widl C wrapper
#[inline] // gen:0a1347f446dd
pub unsafe fn IStream_AddRef(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:9ae2265f34c0
pub unsafe fn IStream_QueryInterface(This: *mut IStream, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:57df1036880d
pub unsafe fn IStream_Read(This: *mut IStream, pv: *mut ::core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Read.unwrap_unchecked())(This, pv, cb, pcbRead) } // widl C wrapper
#[inline] // gen:41d6f1edc6ec
pub unsafe fn IStream_Release(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:74bd0263bdaf
pub unsafe fn IStream_Seek(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT { ((*(*This).lpVtbl).Seek.unwrap_unchecked())(This, dlibMove, dwOrigin, plibNewPosition) } // widl C wrapper
#[inline] // gen:3226c366765c
pub unsafe fn IStream_Stat(This: *mut IStream, pstatstg: *mut STATSTG, grfStatFlag: DWORD) -> HRESULT { ((*(*This).lpVtbl).Stat.unwrap_unchecked())(This, pstatstg, grfStatFlag) } // widl C wrapper
#[inline] // gen:79f3871c5a05
pub unsafe fn IStream_Write(This: *mut IStream, pv: *const ::core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Write.unwrap_unchecked())(This, pv, cb, pcbWritten) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:aa234a85a98a
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:13cb62a339d5
pub unsafe fn IWebBrowserApp_GetProperty(This: *mut IWebBrowserApp, Property: BSTR, pvtValue: *mut VARIANT) -> HRESULT { ((*(*This).lpVtbl).GetProperty.unwrap_unchecked())(This, Property, pvtValue) } // widl C wrapper
#[inline] // gen:d568791e9fd0
pub unsafe fn IWebBrowserApp_PutProperty(This: *mut IWebBrowserApp, Property: BSTR, vtValue: VARIANT) -> HRESULT { ((*(*This).lpVtbl).PutProperty.unwrap_unchecked())(This, Property, vtValue) } // widl C wrapper
#[inline] // gen:0a418b0fc8d3
pub unsafe fn IWebBrowserApp_Release(This: *mut IWebBrowserApp) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const LOCALE_INVARIANT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x00) as USHORT)) as c_int) << 10) | ((((0x7f) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:dde9a4be80d5
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:2d64a44f0c5a
pub type REFCLSID = *const CLSID; // gen:4a91e3a9efb2
pub type REFGUID = *const GUID; // gen:4c78e76f64d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const STG_E_ACCESSDENIED: HRESULT = ((0x80030005u32) as HRESULT); // gen:273b6a1444cc
pub const STG_E_INSUFFICIENTMEMORY: HRESULT = ((0x80030008u32) as HRESULT); // gen:3e00ce984444
pub const STG_E_INVALIDFUNCTION: HRESULT = ((0x80030001u32) as HRESULT); // gen:208886da942c
pub const STG_E_INVALIDPARAMETER: HRESULT = ((0x80030057u32) as HRESULT); // gen:792298eeff11
pub const STG_E_INVALIDPOINTER: HRESULT = ((0x80030009u32) as HRESULT); // gen:f6fa9efa2e9d
pub const STG_E_MEDIUMFULL: HRESULT = ((0x80030070u32) as HRESULT); // gen:31f72f527c59
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub const VARIANT_FALSE: VARIANT_BOOL = 0x0usize as VARIANT_BOOL; // gen:80e3b4c527b0
pub const VARIANT_TRUE: VARIANT_BOOL = 0xffffusize as VARIANT_BOOL; // gen:80f078b25726
pub use crate::{HRESULT_CODE, IQueryAssociations_GetKey, IQueryAssociations_GetString, IQueryAssociations_Init, IQueryAssociations_Release, IsEqualIID, MAKEDLLVERULL, MAKE_HRESULT, V_ARRAY, V_BOOL, V_BSTR, V_I4, V_UI2, V_UI4, V_UNKNOWN, V_VT, atoiW, isalnumW, isalphaW, isdigitW, islowerW, isxdigitW, max, memicmpW, min, strcatW, strchrW, strcmpW, strcmpiW, strcpyW, strcspnW, strncmpW, strncmpiW, strpbrkW, strrchrW, strspnW, strstrW, toupperW};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/win32/shlwapi/ordinal.c:4025: #define GET_RGB(h) (ConvertHue(h, wMid1, wMid2) * 255 + 120) / 240 // gen:161242aca6da
#[allow(unused_macros)]
macro_rules! GET_RGB {
    ($h:expr $(,)?) => {
        ((ConvertHue($h, wMid1, wMid2) * 255 + 120) / 240)
    };
}
#[allow(unused_imports)]
pub(crate) use GET_RGB;
// dll/win32/shlwapi/string.c:812: #define IS_DIGIT(c) ((UINT)(c) - '0' <= 9) // gen:3f47283746a8
#[allow(unused_macros)]
macro_rules! IS_DIGIT {
    ($c:expr $(,)?) => {
        ((($c) as u32).wrapping_sub(b'0' as u32) <= 9)
    };
}
#[allow(unused_imports)]
pub(crate) use IS_DIGIT;
// dll/win32/shlwapi/istream.c:37: #define STGM_ACCESS_MODE(stgm) ((stgm)&0x0000f) // gen:0ad892f929c6
#[allow(unused_macros)]
macro_rules! STGM_ACCESS_MODE {
    ($stgm:expr $(,)?) => {
        (($stgm) & 0x0000f)
    };
}
#[allow(unused_imports)]
pub(crate) use STGM_ACCESS_MODE;
// dll/win32/shlwapi/istream.c:39: #define STGM_CREATE_MODE(stgm) ((stgm)&0x0f000) // gen:1a02503545bc
#[allow(unused_macros)]
macro_rules! STGM_CREATE_MODE {
    ($stgm:expr $(,)?) => {
        (($stgm) & 0x0f000)
    };
}
#[allow(unused_imports)]
pub(crate) use STGM_CREATE_MODE;
// dll/win32/shlwapi/istream.c:38: #define STGM_SHARE_MODE(stgm) ((stgm)&0x000f0) // gen:6bf6ef2fae9b
#[allow(unused_macros)]
macro_rules! STGM_SHARE_MODE {
    ($stgm:expr $(,)?) => {
        (($stgm) & 0x000f0)
    };
}
#[allow(unused_imports)]
pub(crate) use STGM_SHARE_MODE;
// ==== end port macros ====
