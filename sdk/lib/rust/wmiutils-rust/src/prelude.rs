//! wmiutils-rust prelude — the single import surface the translated modules use.
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

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const WBEMPATH_GET_NAMESPACE_ONLY: tag_WBEM_GET_TEXT_FLAGS = tag_WBEM_GET_TEXT_FLAGS_WBEMPATH_GET_NAMESPACE_ONLY; // island enum
pub const WBEMPATH_GET_ORIGINAL: tag_WBEM_GET_TEXT_FLAGS = tag_WBEM_GET_TEXT_FLAGS_WBEMPATH_GET_ORIGINAL; // island enum
pub const WBEMPATH_GET_RELATIVE_ONLY: tag_WBEM_GET_TEXT_FLAGS = tag_WBEM_GET_TEXT_FLAGS_WBEMPATH_GET_RELATIVE_ONLY; // island enum
pub const WBEMPATH_GET_SERVER_AND_NAMESPACE_ONLY: tag_WBEM_GET_TEXT_FLAGS = tag_WBEM_GET_TEXT_FLAGS_WBEMPATH_GET_SERVER_AND_NAMESPACE_ONLY; // island enum
pub const WBEMPATH_GET_SERVER_TOO: tag_WBEM_GET_TEXT_FLAGS = tag_WBEM_GET_TEXT_FLAGS_WBEMPATH_GET_SERVER_TOO; // island enum
pub const WBEMPATH_INFO_ANON_LOCAL_MACHINE: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_ANON_LOCAL_MACHINE; // island enum
pub const WBEMPATH_INFO_CIM_COMPLIANT: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_CIM_COMPLIANT; // island enum
pub const WBEMPATH_INFO_HAS_IMPLIED_KEY: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_HAS_IMPLIED_KEY; // island enum
pub const WBEMPATH_INFO_HAS_MACHINE_NAME: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_HAS_MACHINE_NAME; // island enum
pub const WBEMPATH_INFO_HAS_SUBSCOPES: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_HAS_SUBSCOPES; // island enum
pub const WBEMPATH_INFO_IS_CLASS_REF: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_IS_CLASS_REF; // island enum
pub const WBEMPATH_INFO_IS_INST_REF: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_IS_INST_REF; // island enum
pub const WBEMPATH_INFO_PATH_HAD_SERVER: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_PATH_HAD_SERVER; // island enum
pub const WBEMPATH_INFO_SERVER_NAMESPACE_ONLY: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_SERVER_NAMESPACE_ONLY; // island enum
pub const WBEMPATH_INFO_V1_COMPLIANT: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_V1_COMPLIANT; // island enum
pub const WBEMPATH_INFO_V2_COMPLIANT: tag_WBEM_PATH_STATUS_FLAG = tag_WBEM_PATH_STATUS_FLAG_WBEMPATH_INFO_V2_COMPLIANT; // island enum
pub const WBEM_E_INVALID_OBJECT_PATH: tag_WBEMSTATUS = tag_WBEMSTATUS_WBEM_E_INVALID_OBJECT_PATH; // island enum
pub const WBEM_E_INVALID_PARAMETER: tag_WBEMSTATUS = tag_WBEMSTATUS_WBEM_E_INVALID_PARAMETER; // island enum
pub const WBEM_E_NOT_AVAILABLE: tag_WBEMSTATUS = tag_WBEMSTATUS_WBEM_E_NOT_AVAILABLE; // island enum
pub const WBEM_E_OUT_OF_MEMORY: tag_WBEMSTATUS = tag_WBEMSTATUS_WBEM_E_OUT_OF_MEMORY; // island enum
pub const WBEM_S_NO_ERROR: tag_WBEMSTATUS = tag_WBEMSTATUS_WBEM_S_NO_ERROR; // island enum
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/dll.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn realloc(p: *mut c_void, size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn InitializeCriticalSectionEx( lpCriticalSection: LPCRITICAL_SECTION, dwSpinCount: u32, Flags: u32, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::wmiutils::path::WbemPath_create;
pub use crate::wmiutils::statuscode::WbemStatusCodeText_create;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // main.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // path.c
}
pub use crate::prelude::_wcsdup as wcsdup; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_wmiutils: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x6d\x69\x75\x74\x69\x6c\x73\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_wmiutils };
// ==== end wine debug channels ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = ((0x80040111u32) as HRESULT); // gen:1197fd83f980
pub const CLASS_E_NOAGGREGATION: HRESULT = ((0x80040110u32) as HRESULT); // gen:01676cfea511
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
#[inline] // gen:dc9a16403a7a
pub unsafe fn IClassFactory_AddRef(This: *mut IClassFactory) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:cadbabb17eeb
pub unsafe fn IClassFactory_QueryInterface(This: *mut IClassFactory, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:38acc17a31e7
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline] // gen:aa234a85a98a
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:78a3a4d727ee
pub unsafe fn IWbemPathKeyList_AddRef(This: *mut IWbemPathKeyList) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:0b2b41f5ca44
pub unsafe fn IWbemPath_AddRef(This: *mut IWbemPath) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d228c2aa6146
pub unsafe fn IWbemPath_Release(This: *mut IWbemPath) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:3cced553c560
pub unsafe fn IWbemStatusCodeText_AddRef(This: *mut IWbemStatusCodeText) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
// ==== end SDK header macros ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static IID_IWbemPath: GUID = GUID { Data1: 0x3bc15af2, Data2: 0x736c, Data3: 0x477e, Data4: [0x9e, 0x51, 0x23, 0x8a, 0xf8, 0x66, 0x7d, 0xcc] };
#[no_mangle]
pub static IID_IWbemPathKeyList: GUID = GUID { Data1: 0x9ae62877, Data2: 0x7544, Data3: 0x4bb0, Data4: [0xaa, 0x26, 0xa1, 0x38, 0x24, 0x65, 0x9e, 0xd6] };
#[no_mangle]
pub static IID_IWbemStatusCodeText: GUID = GUID { Data1: 0xeb87e1bc, Data2: 0x3233, Data3: 0x11d2, Data4: [0xae, 0xc9, 0x00, 0xc0, 0x4f, 0xb6, 0x88, 0x20] };
// ==== end INITGUID storage ====
