//! wmic-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const EOAC_NONE: DWORD = 0; // ole32-rust/src/marshal/types.rs
pub const RPC_C_AUTHN_LEVEL_DEFAULT: u32 = 0; // eventlog-rust/src/types_amd64.rs
pub const RPC_C_IMP_LEVEL_IMPERSONATE: u32 = 3; // eventlog-rust/src/types_amd64.rs
pub const STD_ERROR_HANDLE: u32 = 4294967284; // cmd-rust/src/prelude.rs
pub const STD_OUTPUT_HANDLE: u32 = 4294967285; // cmd-rust/src/prelude.rs
pub const VT_BSTR: DWORD = 8; // cleanmgr-rust/src/prelude.rs
pub const WBEM_FLAG_ALWAYS: tag_WBEM_CONDITION_FLAG_TYPE = tag_WBEM_CONDITION_FLAG_TYPE_WBEM_FLAG_ALWAYS; // island enum
pub const WBEM_FLAG_FORWARD_ONLY: tag_WBEM_GENERIC_FLAG_TYPE = tag_WBEM_GENERIC_FLAG_TYPE_WBEM_FLAG_FORWARD_ONLY; // island enum
pub const WBEM_FLAG_RETURN_IMMEDIATELY: tag_WBEM_GENERIC_FLAG_TYPE = tag_WBEM_GENERIC_FLAG_TYPE_WBEM_FLAG_RETURN_IMMEDIATELY; // island enum
pub const WBEM_INFINITE: tag_WBEM_TIMEOUT_TYPE = tag_WBEM_TIMEOUT_TYPE_WBEM_INFINITE; // island enum
pub const WBEM_S_NO_ERROR: tag_WBEMSTATUS = tag_WBEMSTATUS_WBEM_S_NO_ERROR; // island enum
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // apphelp-rust/src/prelude.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn WriteConsoleW( hConsoleOutput: *mut ::core::ffi::c_void, lpBuffer: *const ::core::ffi::c_void, nNumberOfCharsToWrite: u32, lpNumberOfCharsWritten: LPDWORD, lpReserved: *mut ::core::ffi::c_void, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CoCreateInstance( rclsid: *const GUID, pUnkOuter: *mut c_void, dwClsContext: u32, riid: *const GUID, ppv: *mut LPVOID, ) -> HRESULT; // schtasks-rust/src/prelude.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SafeArrayDestroy(arg1: *mut SAFEARRAY) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayGetElement( arg1: *mut SAFEARRAY, arg2: *mut LONG, arg3: *mut ::core::ffi::c_void, ) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayGetUBound(arg1: *mut SAFEARRAY, arg2: UINT, arg3: *mut LONG) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysAllocStringLen(arg1: *const OLECHAR, ui: UINT) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs
    pub fn VariantChangeType( arg1: *mut VARIANTARG, arg2: *const VARIANTARG, arg3: USHORT, arg4: ::core::ffi::c_ushort, ) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _snwprintf(
        _Dest: *mut wchar_t,
        _Count: size_t,
        _Format: *const wchar_t,
        ...
    ) -> ::core::ffi::c_int; // main.c
}
extern "system" {
    pub fn CoInitializeSecurity(
        pSecDesc: PSECURITY_DESCRIPTOR,
        cAuthSvc: LONG,
        asAuthSvc: *mut SOLE_AUTHENTICATION_SERVICE,
        pReserved1: *mut ::core::ffi::c_void,
        dwAuthnLevel: DWORD,
        dwImpLevel: DWORD,
        pReserved2: *mut ::core::ffi::c_void,
        dwCapabilities: DWORD,
        pReserved3: *mut ::core::ffi::c_void,
    ) -> HRESULT; // main.c
}
pub use crate::prelude::_snwprintf as snprintfW; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_wmic: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x6d\x69\x63\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_wmic };
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
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! strcatW { ($p0:expr, $p1:expr $(,)?) => { wcscat((($p0) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! strcmpiW { ($p0:expr, $p1:expr $(,)?) => { _wcsicmp((($p0) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! strcpyW { ($p0:expr, $p1:expr $(,)?) => { wcscpy((($p0) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! strlenW { ($p0:expr $(,)?) => { wcslen((($p0) as _)) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
#[inline]
pub unsafe fn IEnumWbemClassObject_Next(This: *mut IEnumWbemClassObject, lTimeout: LONG, uCount: ULONG, apObjects: *mut *mut IWbemClassObject, puReturned: *mut ULONG) -> HRESULT { ((*(*This).lpVtbl).Next.unwrap_unchecked())(This, lTimeout, uCount, apObjects, puReturned) } // widl C wrapper
#[inline]
pub unsafe fn IEnumWbemClassObject_Release(This: *mut IEnumWbemClassObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IEnumWbemClassObject_Reset(This: *mut IEnumWbemClassObject) -> HRESULT { ((*(*This).lpVtbl).Reset.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_Get(This: *mut IWbemClassObject, wszName: LPCWSTR, lFlags: LONG, pVal: *mut VARIANT, pType: *mut CIMTYPE, plFlavor: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).Get.unwrap_unchecked())(This, wszName, lFlags, pVal, pType, plFlavor) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_GetNames(This: *mut IWbemClassObject, wszQualifierName: LPCWSTR, lFlags: LONG, pQualifierVal: *mut VARIANT, pNames: *mut *mut SAFEARRAY) -> HRESULT { ((*(*This).lpVtbl).GetNames.unwrap_unchecked())(This, wszQualifierName, lFlags, pQualifierVal, pNames) } // widl C wrapper
#[inline]
pub unsafe fn IWbemClassObject_Release(This: *mut IWbemClassObject) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemLocator_ConnectServer(This: *mut IWbemLocator, strNetworkResource: BSTR, strUser: BSTR, strPassword: BSTR, strLocale: BSTR, lSecurityFlags: LONG, strAuthority: BSTR, pCtx: *mut IWbemContext, ppNamespace: *mut *mut IWbemServices) -> HRESULT { ((*(*This).lpVtbl).ConnectServer.unwrap_unchecked())(This, strNetworkResource, strUser, strPassword, strLocale, lSecurityFlags, strAuthority, pCtx, ppNamespace) } // widl C wrapper
#[inline]
pub unsafe fn IWbemLocator_Release(This: *mut IWbemLocator) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IWbemServices_ExecQuery(This: *mut IWbemServices, strQueryLanguage: BSTR, strQuery: BSTR, lFlags: LONG, pCtx: *mut IWbemContext, ppEnum: *mut *mut IEnumWbemClassObject) -> HRESULT { ((*(*This).lpVtbl).ExecQuery.unwrap_unchecked())(This, strQueryLanguage, strQuery, lFlags, pCtx, ppEnum) } // widl C wrapper
#[inline]
pub unsafe fn IWbemServices_Release(This: *mut IWbemServices) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{V_BSTR, max, strcatW, strcmpiW, strcpyW, strlenW, va_end, va_start};
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn output_string(handle: HANDLE, msg: *const WCHAR, ...) -> ::core::ffi::c_int;
}
// ==== end port variadic ====
