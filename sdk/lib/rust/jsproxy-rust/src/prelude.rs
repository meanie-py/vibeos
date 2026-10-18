//! jsproxy-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedDecrement64, InterlockedExchange, InterlockedIncrement, InterlockedIncrement64, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _byteswap_uint64, _byteswap_ulong, _byteswap_ushort, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, lrintf, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= hand ports (above the generated blocks: they survive replays) ==============
// activscp.h: `#define IActiveScriptParse IActiveScriptParse64` on _WIN64, `...32` otherwise,
// and the IActiveScriptParse_* macros pick the same. The island carries both vtables (the same
// hand port as wscript-rust's; the types are per-island, so it cannot be shared).
#[cfg(target_pointer_width = "64")]
pub type IActiveScriptParse = IActiveScriptParse64;
#[cfg(not(target_pointer_width = "64"))]
pub type IActiveScriptParse = IActiveScriptParse32;
macro_rules! activscript_parse_shims {
    ($I:ident, $Cookie:ty) => {
        pub unsafe fn IActiveScriptParse_Release(This: *mut $I) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) }
        pub unsafe fn IActiveScriptParse_InitNew(This: *mut $I) -> HRESULT { ((*(*This).lpVtbl).InitNew.unwrap_unchecked())(This) }
        pub unsafe fn IActiveScriptParse_ParseScriptText(This: *mut $I, pstrCode: LPCOLESTR, pstrItemName: LPCOLESTR, punkContext: *mut IUnknown, pstrDelimiter: LPCOLESTR, dwSourceContextCookie: $Cookie, ulStartingLineNumber: ULONG, dwFlags: DWORD, pvarResult: *mut VARIANT, pexcepinfo: *mut EXCEPINFO) -> HRESULT {
            ((*(*This).lpVtbl).ParseScriptText.unwrap_unchecked())(This, pstrCode, pstrItemName, punkContext, pstrDelimiter, dwSourceContextCookie, ulStartingLineNumber, dwFlags, pvarResult, pexcepinfo)
        }
    };
}
#[cfg(target_pointer_width = "64")]
activscript_parse_shims!(IActiveScriptParse64, DWORDLONG);
#[cfg(not(target_pointer_width = "64"))]
activscript_parse_shims!(IActiveScriptParse32, DWORD);
#[cfg(target_pointer_width = "64")]
pub use crate::types::IID_IActiveScriptParse64 as IID_IActiveScriptParse;
#[cfg(not(target_pointer_width = "64"))]
pub use crate::types::IID_IActiveScriptParse32 as IID_IActiveScriptParse;
// guiddef.h: `#define IID_NULL GUID_NULL`; uuid.lib (which jsproxy links) defines it.
extern "C" {
    pub static GUID_NULL: GUID;
}
pub use crate::prelude::GUID_NULL as IID_NULL;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const AF_INET: u32 = 2; // finger-rust/src/prelude.rs
pub const CLSCTX_INPROC_HANDLER: DWORD = 0x2; // ole32-rust/src/compobj/com.rs
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const DISPATCH_METHOD: u32 = 1; // mmc-rust/src/types_amd64.rs
pub const SCRIPTSTATE_STARTED: tagSCRIPTSTATE = tagSCRIPTSTATE_SCRIPTSTATE_STARTED; // island enum
pub const VT_BSTR: DWORD = 8; // cleanmgr-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn free(p: *mut c_void); // atlclasses-rust/src/dll.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub static IID_IDispatch: GUID; // aclui-rust/src/types_amd64.rs
    pub static IID_IUnknown: GUID; // cicero-rust/src/types_amd64.rs
}
extern "system" {
    pub fn CLSIDFromProgID(progid: LPCOLESTR, riid: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CreateFileA( arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn FindResourceW(arg1: HMODULE, arg2: LPCWSTR, arg3: LPCWSTR) -> HRSRC; // aclui-rust/src/types_i386.rs
    pub fn GetComputerNameExA(arg1: COMPUTER_NAME_FORMAT, arg2: LPSTR, arg3: LPDWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleA(arg1: LPCSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LoadResource(arg1: HINSTANCE, arg2: HRSRC) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SizeofResource(arg1: HINSTANCE, arg2: HRSRC) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysAllocStringLen(arg1: *const OLECHAR, ui: UINT) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn freeaddrinfo(pAddrInfo: PADDRINFOA); // tracert-rust/src/types_i386.rs
    pub fn getaddrinfo( pNodeName: PCSTR, pServiceName: PCSTR, pHints: *const ADDRINFOA, ppResult: *mut PADDRINFOA, ) -> INT; // tracert-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ntohl(netlong: ::core::ffi::c_ulong) -> u_long; // tracert-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // main.c
    pub fn strlen(_Str: *const ::core::ffi::c_char) -> size_t; // main.c
}
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_jsproxy: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x6a\x73\x70\x72\x6f\x78\x79\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_jsproxy };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
pub const DISP_E_BADPARAMCOUNT: HRESULT = ((0x8002000Eu32) as HRESULT); // gen:d806c8c10a46
pub const DISP_E_BADVARTYPE: HRESULT = ((0x80020008u32) as HRESULT); // gen:fe6e0ba7c0f1
pub const DISP_E_MEMBERNOTFOUND: HRESULT = ((0x80020003u32) as HRESULT); // gen:55554da60ac3
pub const DISP_E_UNKNOWNNAME: HRESULT = ((0x80020006u32) as HRESULT); // gen:73760490f905
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
#[inline] // gen:d9ae3aa8d2e3
pub unsafe fn IActiveScript_AddNamedItem(This: *mut IActiveScript, pstrName: LPCOLESTR, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).AddNamedItem.unwrap_unchecked())(This, pstrName, dwFlags) } // widl C wrapper
#[inline] // gen:e3556806a156
pub unsafe fn IActiveScript_GetScriptDispatch(This: *mut IActiveScript, pstrItemName: LPCOLESTR, ppdisp: *mut *mut IDispatch) -> HRESULT { ((*(*This).lpVtbl).GetScriptDispatch.unwrap_unchecked())(This, pstrItemName, ppdisp) } // widl C wrapper
#[inline] // gen:a8aa151fdfb5
pub unsafe fn IActiveScript_QueryInterface(This: *mut IActiveScript, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:66052458aa6b
pub unsafe fn IActiveScript_Release(This: *mut IActiveScript) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d8f9eabf531f
pub unsafe fn IActiveScript_SetScriptSite(This: *mut IActiveScript, pass: *mut IActiveScriptSite) -> HRESULT { ((*(*This).lpVtbl).SetScriptSite.unwrap_unchecked())(This, pass) } // widl C wrapper
#[inline] // gen:c1a62fe6e4ba
pub unsafe fn IActiveScript_SetScriptState(This: *mut IActiveScript, ss: SCRIPTSTATE) -> HRESULT { ((*(*This).lpVtbl).SetScriptState.unwrap_unchecked())(This, ss) } // widl C wrapper
#[inline] // gen:56d170a628cb
pub unsafe fn IDispatch_GetIDsOfNames(This: *mut IDispatch, riid: *const IID, rgszNames: *mut LPOLESTR, cNames: UINT, lcid: LCID, rgDispId: *mut DISPID) -> HRESULT { ((*(*This).lpVtbl).GetIDsOfNames.unwrap_unchecked())(This, riid, rgszNames, cNames, lcid, rgDispId) } // widl C wrapper
#[inline] // gen:0e6599a31c2e
pub unsafe fn IDispatch_Invoke(This: *mut IDispatch, dispIdMember: DISPID, riid: *const IID, lcid: LCID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT { ((*(*This).lpVtbl).Invoke.unwrap_unchecked())(This, dispIdMember, riid, lcid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr) } // widl C wrapper
#[inline] // gen:2dd34cc31a72
pub unsafe fn IDispatch_Release(This: *mut IDispatch) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:6ad0b58c268c
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
pub const LOCALE_SYSTEM_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x02) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD); // gen:2e6e0be784d6
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{V_BSTR, V_VT};
// ==== end SDK header macros ====
