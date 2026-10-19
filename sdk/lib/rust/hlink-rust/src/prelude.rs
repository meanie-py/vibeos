//! hlink-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedExchangePointer, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // acppage-rust/src/prelude.rs
pub const HLSR_HISTORYFOLDER: _HLSR_NOREDEF10 = _HLSR_NOREDEF10_HLSR_HISTORYFOLDER; // island enum
pub const HLSR_HOME: _HLSR_NOREDEF10 = _HLSR_NOREDEF10_HLSR_HOME; // island enum
pub const HLSR_SEARCHPAGE: _HLSR_NOREDEF10 = _HLSR_NOREDEF10_HLSR_SEARCHPAGE; // island enum
pub const MKSYS_NONE: tagMKSYS = tagMKSYS_MKSYS_NONE; // island enum
pub const STREAM_SEEK_SET: DWORD = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // activeds-rust/src/prelude.rs
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // activeds-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // activeds-rust/src/prelude.rs
}
extern "system" {
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemRealloc(ptr: LPVOID, size: SIZE_T) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn CreateBindCtx(reserved: DWORD, ppbc: *mut LPBC) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateFileMoniker(lpszPathName: LPCOLESTR, ppmk: *mut LPMONIKER) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateGenericComposite( pmkFirst: LPMONIKER, pmkRest: LPMONIKER, ppmkComposite: *mut LPMONIKER, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateItemMoniker( lpszDelim: LPCOLESTR, lpszItem: LPCOLESTR, ppmk: *mut LPMONIKER, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateStreamOnHGlobal( hGlobal: HGLOBAL, fDeleteOnRelease: BOOL, ppstm: *mut LPSTREAM, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CreateURLMoniker( pMkCtx: *mut IMoniker, szURL: *const u16, ppmk: *mut *mut IMoniker, ) -> HRESULT; // oleaut32-rust/src/olepicture.rs
    pub fn GetRunningObjectTable(reserved: DWORD, pprot: *mut LPRUNNINGOBJECTTABLE) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn MkParseDisplayName( pbc: LPBC, szUserName: LPCOLESTR, pchEaten: *mut ULONG, ppmk: *mut LPMONIKER, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn OleLoadFromStream( pStm: *mut IStream, iidInterface: *const IID, ppvObj: *mut *mut ::core::ffi::c_void, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn OleSaveToStream(pPStm: *mut IPersistStream, pStm: *mut IStream) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ShellExecuteW( hwnd: HWND, lpOperation: LPCWSTR, lpFile: LPCWSTR, lpParameters: LPCWSTR, lpDirectory: LPCWSTR, nShowCmd: INT, ) -> HINSTANCE; // acppage-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::hlink::link::HLink_Constructor;
pub use crate::hlink::hlink_main::HlinkCreateFromMoniker;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbLengthW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfExW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const HLFNAMEF_DEFAULT: i32 = 0; // clang, hlink_main.c
pub const HLINKGETREF_ABSOLUTE: i32 = 1; // clang, link.c
pub const HLINKGETREF_DEFAULT: i32 = 0; // clang, hlink_main.c
pub const HLINKGETREF_RELATIVE: i32 = 2; // clang, link.c
pub const HLINKSETF_LOCATION: i32 = 2; // clang, hlink_main.c
pub const HLINKSETF_TARGET: i32 = 1; // clang, hlink_main.c
pub const OLEGETMONIKER_FORCEASSIGN: i32 = 2; // clang, link.c
pub const OLEWHICHMK_CONTAINER: i32 = 1; // clang, link.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // acppage-rust/src/prelude.rs
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // activeds-rust/src/prelude.rs
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // browse_ctx.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // hlink_main.c
}
extern "system" {
    pub fn MkParseDisplayNameEx(
        _: *mut IBindCtx,
        _: LPCWSTR,
        _: *mut ULONG,
        _: *mut *mut IMoniker,
    ) -> HRESULT; // hlink_main.c
    pub fn RegisterBindStatusCallback(
        pbc: *mut IBindCtx,
        pbsc: *mut IBindStatusCallback,
        ppbsc: *mut *mut IBindStatusCallback,
        dwReserved: DWORD,
    ) -> HRESULT; // hlink_main.c
    pub fn RevokeBindStatusCallback(_: LPBC, _: *mut IBindStatusCallback) -> HRESULT; // link.c
}
pub use crate::prelude::_wcsdup as wcsdup; // sdk alias
pub use crate::prelude::_wcsicmp as wcsicmp; // sdk alias
pub use crate::prelude::_wcsnicmp as wcsnicmp; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_hlink: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x68\x6c\x69\x6e\x6b\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_hlink };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:aa85287a8771
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:01676cfea511
pub const CO_E_ALREADYINITIALIZED: HRESULT = ((0x800401F1u32) as HRESULT); // gen:367db7802a96
pub const DRAGDROP_S_DROP: HRESULT = ((0x00040100) as HRESULT); // gen:2e454147f436
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT); // gen:1275c1cb9e8d
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
pub const E_UNEXPECTED: HRESULT = ((0x8000FFFFu32) as HRESULT); // gen:c1fe647c5e73
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY); // gen:f003d9cba1a1
#[inline] // gen:6f23f50a8915
pub unsafe fn IBindCtx_Release(This: *mut IBindCtx) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:54f09c37d6ef
pub unsafe fn IBindStatusCallback_AddRef(This: *mut IBindStatusCallback) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:985b4f5e2372
pub unsafe fn IBindStatusCallback_GetBindInfo(This: *mut IBindStatusCallback, grfBINDF: *mut DWORD, pbindinfo: *mut BINDINFO) -> HRESULT { ((*(*This).lpVtbl).GetBindInfo.unwrap_unchecked())(This, grfBINDF, pbindinfo) } // widl C wrapper
#[inline] // gen:0ede0f972490
pub unsafe fn IBindStatusCallback_OnLowResource(This: *mut IBindStatusCallback, reserved: DWORD) -> HRESULT { ((*(*This).lpVtbl).OnLowResource.unwrap_unchecked())(This, reserved) } // widl C wrapper
#[inline] // gen:b39998da1b49
pub unsafe fn IBindStatusCallback_OnObjectAvailable(This: *mut IBindStatusCallback, riid: *const IID, punk: *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).OnObjectAvailable.unwrap_unchecked())(This, riid, punk) } // widl C wrapper
#[inline] // gen:b8bc17fb05fa
pub unsafe fn IBindStatusCallback_OnProgress(This: *mut IBindStatusCallback, ulProgress: ULONG, ulProgressMax: ULONG, ulStatusCode: ULONG, szStatusText: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).OnProgress.unwrap_unchecked())(This, ulProgress, ulProgressMax, ulStatusCode, szStatusText) } // widl C wrapper
#[inline] // gen:6944af63ec23
pub unsafe fn IBindStatusCallback_OnStartBinding(This: *mut IBindStatusCallback, dwReserved: DWORD, pib: *mut IBinding) -> HRESULT { ((*(*This).lpVtbl).OnStartBinding.unwrap_unchecked())(This, dwReserved, pib) } // widl C wrapper
#[inline] // gen:eacb33d15cb1
pub unsafe fn IBindStatusCallback_OnStopBinding(This: *mut IBindStatusCallback, hresult: HRESULT, szError: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).OnStopBinding.unwrap_unchecked())(This, hresult, szError) } // widl C wrapper
#[inline] // gen:11bd4094608a
pub unsafe fn IBindStatusCallback_Release(This: *mut IBindStatusCallback) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:cadbabb17eeb
pub unsafe fn IClassFactory_QueryInterface(This: *mut IClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:62084b921f24
pub unsafe fn IHlinkBrowseContext_AddRef(This: *mut IHlinkBrowseContext) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e6bdb1a31118
pub unsafe fn IHlinkBrowseContext_GetObject(This: *mut IHlinkBrowseContext, pimk: *mut IMoniker, fBindIfRootRegistered: BOOL, ppiunk: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, pimk, fBindIfRootRegistered, ppiunk) } // widl C wrapper
#[inline] // gen:63170b5f6b6a
pub unsafe fn IHlinkBrowseContext_OnNavigateHlink(This: *mut IHlinkBrowseContext, grfHLNF: DWORD, pimkTarget: *mut IMoniker, pwzLocation: LPCWSTR, pwzFriendlyName: LPCWSTR, puHLID: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).OnNavigateHlink.unwrap_unchecked())(This, grfHLNF, pimkTarget, pwzLocation, pwzFriendlyName, puHLID) } // widl C wrapper
#[inline] // gen:15008766b22b
pub unsafe fn IHlinkBrowseContext_QueryInterface(This: *mut IHlinkBrowseContext, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:4cb786167f03
pub unsafe fn IHlinkBrowseContext_Release(This: *mut IHlinkBrowseContext) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:9f6bcf71f65a
pub unsafe fn IHlinkBrowseContext_UpdateHlink(This: *mut IHlinkBrowseContext, uHLID: ULONG, pimkTarget: *mut IMoniker, pwzLocation: LPCWSTR, pwzFriendlyName: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).UpdateHlink.unwrap_unchecked())(This, uHLID, pimkTarget, pwzLocation, pwzFriendlyName) } // widl C wrapper
#[inline] // gen:f6f4df2ec9cc
pub unsafe fn IHlinkFrame_Navigate(This: *mut IHlinkFrame, grfHLNF: DWORD, pbc: LPBC, pibsc: *mut IBindStatusCallback, pihlNavigate: *mut IHlink) -> HRESULT { ((*(*This).lpVtbl).Navigate.unwrap_unchecked())(This, grfHLNF, pbc, pibsc, pihlNavigate) } // widl C wrapper
#[inline] // gen:497fa953d4ca
pub unsafe fn IHlinkFrame_OnNavigate(This: *mut IHlinkFrame, grfHLNF: DWORD, pimkTarget: *mut IMoniker, pwzLocation: LPCWSTR, pwzFriendlyName: LPCWSTR, dwreserved: DWORD) -> HRESULT { ((*(*This).lpVtbl).OnNavigate.unwrap_unchecked())(This, grfHLNF, pimkTarget, pwzLocation, pwzFriendlyName, dwreserved) } // widl C wrapper
#[inline] // gen:046b993b1f3d
pub unsafe fn IHlinkFrame_UpdateHlink(This: *mut IHlinkFrame, uHLID: ULONG, pimkTarget: *mut IMoniker, pwzLocation: LPCWSTR, pwzFriendlyName: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).UpdateHlink.unwrap_unchecked())(This, uHLID, pimkTarget, pwzLocation, pwzFriendlyName) } // widl C wrapper
#[inline] // gen:0bb2756acc3b
pub unsafe fn IHlinkSite_AddRef(This: *mut IHlinkSite) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:5aed865616a2
pub unsafe fn IHlinkSite_GetMoniker(This: *mut IHlinkSite, dwSiteData: DWORD, dwAssign: DWORD, dwWhich: DWORD, ppimk: *mut *mut IMoniker) -> HRESULT { ((*(*This).lpVtbl).GetMoniker.unwrap_unchecked())(This, dwSiteData, dwAssign, dwWhich, ppimk) } // widl C wrapper
#[inline] // gen:455d59616225
pub unsafe fn IHlinkSite_OnNavigationComplete(This: *mut IHlinkSite, dwSiteData: DWORD, dwreserved: DWORD, hrError: HRESULT, pwzError: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).OnNavigationComplete.unwrap_unchecked())(This, dwSiteData, dwreserved, hrError, pwzError) } // widl C wrapper
#[inline] // gen:7f4edc9fb950
pub unsafe fn IHlinkSite_Release(This: *mut IHlinkSite) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:969b70c67546
pub unsafe fn IHlinkTarget_Navigate(This: *mut IHlinkTarget, grfHLNF: DWORD, pwzJumpLocation: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).Navigate.unwrap_unchecked())(This, grfHLNF, pwzJumpLocation) } // widl C wrapper
#[inline] // gen:8c6e7664faa6
pub unsafe fn IHlinkTarget_Release(This: *mut IHlinkTarget) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0a7f65c64df2
pub unsafe fn IHlinkTarget_SetBrowseContext(This: *mut IHlinkTarget, pihlbc: *mut IHlinkBrowseContext) -> HRESULT { ((*(*This).lpVtbl).SetBrowseContext.unwrap_unchecked())(This, pihlbc) } // widl C wrapper
#[inline] // gen:766fef2c724c
pub unsafe fn IHlink_AddRef(This: *mut IHlink) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:336ef70e6b1c
pub unsafe fn IHlink_GetFriendlyName(This: *mut IHlink, grfHLFNAMEF: DWORD, ppwzFriendlyName: *mut LPWSTR) -> HRESULT { ((*(*This).lpVtbl).GetFriendlyName.unwrap_unchecked())(This, grfHLFNAMEF, ppwzFriendlyName) } // widl C wrapper
#[inline] // gen:63d2e570a551
pub unsafe fn IHlink_GetMonikerReference(This: *mut IHlink, dwWhichRef: DWORD, ppimkTarget: *mut *mut IMoniker, ppwzLocation: *mut LPWSTR) -> HRESULT { ((*(*This).lpVtbl).GetMonikerReference.unwrap_unchecked())(This, dwWhichRef, ppimkTarget, ppwzLocation) } // widl C wrapper
#[inline] // gen:be89de9de1ee
pub unsafe fn IHlink_GetStringReference(This: *mut IHlink, dwWhichRef: DWORD, ppwzTarget: *mut LPWSTR, ppwzLocation: *mut LPWSTR) -> HRESULT { ((*(*This).lpVtbl).GetStringReference.unwrap_unchecked())(This, dwWhichRef, ppwzTarget, ppwzLocation) } // widl C wrapper
#[inline] // gen:97bf4dda7dcf
pub unsafe fn IHlink_Navigate(This: *mut IHlink, grfHLNF: DWORD, pibc: LPBC, pibsc: *mut IBindStatusCallback, pihlbc: *mut IHlinkBrowseContext) -> HRESULT { ((*(*This).lpVtbl).Navigate.unwrap_unchecked())(This, grfHLNF, pibc, pibsc, pihlbc) } // widl C wrapper
#[inline] // gen:6160b3b8450e
pub unsafe fn IHlink_QueryInterface(This: *mut IHlink, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:ff9a268e6e3f
pub unsafe fn IHlink_Release(This: *mut IHlink) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:57065731e3c2
pub unsafe fn IHlink_SetFriendlyName(This: *mut IHlink, pwzFriendlyName: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetFriendlyName.unwrap_unchecked())(This, pwzFriendlyName) } // widl C wrapper
#[inline] // gen:5ac4a389b1fc
pub unsafe fn IHlink_SetHlinkSite(This: *mut IHlink, pihlSite: *mut IHlinkSite, dwSiteData: DWORD) -> HRESULT { ((*(*This).lpVtbl).SetHlinkSite.unwrap_unchecked())(This, pihlSite, dwSiteData) } // widl C wrapper
#[inline] // gen:5aaff2966be1
pub unsafe fn IHlink_SetMonikerReference(This: *mut IHlink, grfHLSETF: DWORD, pimkTarget: *mut IMoniker, pwzLocation: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetMonikerReference.unwrap_unchecked())(This, grfHLSETF, pimkTarget, pwzLocation) } // widl C wrapper
#[inline] // gen:104bef70886e
pub unsafe fn IHlink_SetStringReference(This: *mut IHlink, grfHLSETF: DWORD, pwzTarget: LPCWSTR, pwzLocation: LPCWSTR) -> HRESULT { ((*(*This).lpVtbl).SetStringReference.unwrap_unchecked())(This, grfHLSETF, pwzTarget, pwzLocation) } // widl C wrapper
#[inline] // gen:da886eba39c7
pub unsafe fn IMoniker_AddRef(This: *mut IMoniker) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:e4dd8fbc1708
pub unsafe fn IMoniker_BindToObject(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riidResult: *const IID, ppvResult: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).BindToObject.unwrap_unchecked())(This, pbc, pmkToLeft, riidResult, ppvResult) } // widl C wrapper
#[inline] // gen:7a3284198eb0
pub unsafe fn IMoniker_BindToStorage(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, riid: *const IID, ppvObj: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).BindToStorage.unwrap_unchecked())(This, pbc, pmkToLeft, riid, ppvObj) } // widl C wrapper
#[inline] // gen:c67cfd026378
pub unsafe fn IMoniker_ComposeWith(This: *mut IMoniker, pmkRight: *mut IMoniker, fOnlyIfNotGeneric: BOOL, ppmkComposite: *mut *mut IMoniker) -> HRESULT { ((*(*This).lpVtbl).ComposeWith.unwrap_unchecked())(This, pmkRight, fOnlyIfNotGeneric, ppmkComposite) } // widl C wrapper
#[inline] // gen:a73daeddcc95
pub unsafe fn IMoniker_GetDisplayName(This: *mut IMoniker, pbc: *mut IBindCtx, pmkToLeft: *mut IMoniker, ppszDisplayName: *mut LPOLESTR) -> HRESULT { ((*(*This).lpVtbl).GetDisplayName.unwrap_unchecked())(This, pbc, pmkToLeft, ppszDisplayName) } // widl C wrapper
#[inline] // gen:f9237b9426bb
pub unsafe fn IMoniker_IsSystemMoniker(This: *mut IMoniker, pdwMksys: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).IsSystemMoniker.unwrap_unchecked())(This, pdwMksys) } // widl C wrapper
#[inline] // gen:1a73284c7b43
pub unsafe fn IMoniker_QueryInterface(This: *mut IMoniker, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:614795dfe34f
pub unsafe fn IMoniker_Release(This: *mut IMoniker) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:3b76803f9569
pub unsafe fn IPersistStream_GetSizeMax(This: *mut IPersistStream, pcbSize: *mut ULARGE_INTEGER) -> HRESULT { ((*(*This).lpVtbl).GetSizeMax.unwrap_unchecked())(This, pcbSize) } // widl C wrapper
#[inline] // gen:c8380c005683
pub unsafe fn IPersistStream_Release(This: *mut IPersistStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1e34b913db2e
pub unsafe fn IRunningObjectTable_GetObject(This: *mut IRunningObjectTable, pmkObjectName: *mut IMoniker, ppunkObject: *mut *mut IUnknown) -> HRESULT { ((*(*This).lpVtbl).GetObject.unwrap_unchecked())(This, pmkObjectName, ppunkObject) } // widl C wrapper
#[inline] // gen:c54446865b9d
pub unsafe fn IRunningObjectTable_Register(This: *mut IRunningObjectTable, grfFlags: DWORD, punkObject: *mut IUnknown, pmkObjectName: *mut IMoniker, pdwRegister: *mut DWORD) -> HRESULT { ((*(*This).lpVtbl).Register.unwrap_unchecked())(This, grfFlags, punkObject, pmkObjectName, pdwRegister) } // widl C wrapper
#[inline] // gen:1742adf6e68e
pub unsafe fn IRunningObjectTable_Release(This: *mut IRunningObjectTable) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:aeac096f6197
pub unsafe fn IRunningObjectTable_Revoke(This: *mut IRunningObjectTable, dwRegister: DWORD) -> HRESULT { ((*(*This).lpVtbl).Revoke.unwrap_unchecked())(This, dwRegister) } // widl C wrapper
#[inline] // gen:57df1036880d
pub unsafe fn IStream_Read(This: *mut IStream, pv: *mut ::core::ffi::c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Read.unwrap_unchecked())(This, pv, cb, pcbRead) } // widl C wrapper
#[inline] // gen:41d6f1edc6ec
pub unsafe fn IStream_Release(This: *mut IStream) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:74bd0263bdaf
pub unsafe fn IStream_Seek(This: *mut IStream, dlibMove: LARGE_INTEGER, dwOrigin: DWORD, plibNewPosition: *mut ULARGE_INTEGER) -> HRESULT { ((*(*This).lpVtbl).Seek.unwrap_unchecked())(This, dlibMove, dwOrigin, plibNewPosition) } // widl C wrapper
#[inline] // gen:79f3871c5a05
pub unsafe fn IStream_Write(This: *mut IStream, pv: *const ::core::ffi::c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Write.unwrap_unchecked())(This, pv, cb, pcbWritten) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:aa234a85a98a
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const MK_S_ASYNCHRONOUS: HRESULT = ((0x000401E8) as HRESULT); // gen:6645716c582e
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const STG_E_READFAULT: HRESULT = ((0x8003001Eu32) as HRESULT); // gen:6de8dd7ad009
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{IsEqualIID};
// ==== end SDK header macros ====

// hand-edited, so kept out of the generated block below (generated_items.py); delete to regenerate
// dll/win32/hlink/hlink_private.h:34: header-inline `hlink_co_strdupW`
#[allow(dead_code)]
#[inline]
pub unsafe fn hlink_co_strdupW(str: LPCWSTR) -> LPWSTR {
    let mut ret: LPWSTR = core::ptr::null_mut();

    if !str.is_null() {
        let size: DWORD;

        size = ((lstrlenW(str) + 1) as DWORD) * (core::mem::size_of::<WCHAR>() as DWORD);
        ret = (CoTaskMemAlloc((size) as SIZE_T)).cast();
        memcpy((ret).cast(), (str).cast(), (size) as _);
    }

    ret
}

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// ==== end port macros ====
