//! sxs-rust prelude — the single import surface the translated modules use.
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
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const NAME_ATTR_ID_ARCH: name_attr_id = name_attr_id_NAME_ATTR_ID_ARCH; // island enum
pub const NAME_ATTR_ID_NAME: name_attr_id = name_attr_id_NAME_ATTR_ID_NAME; // island enum
pub const NAME_ATTR_ID_TOKEN: name_attr_id = name_attr_id_NAME_ATTR_ID_TOKEN; // island enum
pub const NAME_ATTR_ID_TYPE: name_attr_id = name_attr_id_NAME_ATTR_ID_TYPE; // island enum
pub const NAME_ATTR_ID_VERSION: name_attr_id = name_attr_id_NAME_ATTR_ID_VERSION; // island enum
pub const VT_BSTR: DWORD = 8; // cleanmgr-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/dll.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // credui-rust/src/prelude.rs
    pub fn towlower(c: u16) -> u16; // cleanmgr-rust/src/prelude.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // apphelp-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
}
extern "system" {
    pub fn ActivateActCtx(arg1: HANDLE, arg2: *mut ULONG_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateDirectoryW( lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn DeactivateActCtx(arg1: DWORD, arg2: ULONG_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindActCtxSectionGuid( arg1: DWORD, arg2: *const GUID, arg3: ULONG, arg4: *const GUID, arg5: PACTCTX_SECTION_KEYED_DATA, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn QueryActCtxW( dwFlags: DWORD, hActCtx: HANDLE, pvSubInstance: PVOID, ulInfoClass: ULONG, pvBuffer: PVOID, cbBuffer: SIZE_T, pcbWrittenOrRequired: *mut SIZE_T, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseActCtx(arg1: HANDLE); // aclui-rust/src/types_i386.rs
    pub fn ReleaseMutex(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RemoveDirectoryW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn VariantInit(arg1: *mut VARIANT); // msctf-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::sxs::name::CreateAssemblyNameObject;
pub use crate::sxs::name::get_name_attribute;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
pub use winelist_rust::{list_add_after, list_add_before, list_add_head, list_add_tail, list_count, list_empty, list_head, list_init, list_move_head, list_move_tail, list_next, list_prev, list_remove, list_tail};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CANOF_PARSE_DISPLAY_NAME: i32 = 1; // clang, cache.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsdup(arg1: *const wchar_t) -> *mut wchar_t; // msvcrt-sys/src/bindgen_i386.rs
    pub fn _wcslwr(s: LPWSTR) -> LPWSTR; // localspl-rust/src/prelude.rs
    pub fn ros_dbg_log(
        cls: __wine_debug_class,
        ch: *mut __wine_debug_channel,
        file: *const ::core::ffi::c_char,
        func: *const ::core::ffi::c_char,
        line: ::core::ffi::c_int,
        format: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int; // cache.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // name.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // cache.c
}
pub use crate::types::CLSID_DOMDocument2 as CLSID_DOMDocument; // sdk alias
pub use crate::prelude::_wcsdup as wcsdup; // sdk alias
pub use crate::prelude::_wcslwr as wcslwr; // sdk alias
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub type __wine_debug_class = ::core::ffi::c_int; // enum, wine/debug.h
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_sxs: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x73\x78\x73\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_sxs };
// ==== end wine debug channels ====

// ==== port resolve: INITGUID storage (generated) ====
#[no_mangle]
pub static CLSID_DOMDocument2: GUID = GUID { Data1: 0xf6d90f11, Data2: 0x9c73, Data3: 0x11d3, Data4: [0xb3, 0x2e, 0x00, 0xc0, 0x4f, 0x99, 0x0b, 0xb4] };
#[no_mangle]
pub static IID_IAssemblyName: GUID = GUID { Data1: 0xcd193bc0, Data2: 0xb4bc, Data3: 0x11d2, Data4: [0x98, 0x33, 0x00, 0xc0, 0x4f, 0xc3, 0x1d, 0x2e] };
// ==== end INITGUID storage ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; } // gen:aa85287a8771
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; } // gen:4066e24ecd24
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; } // gen:bb0eb6ad6416
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT); // gen:cd8a52171603
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT); // gen:80daa5b2ffa6
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT); // gen:cbf314acc21c
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT); // gen:98bee37aa995
#[inline] // gen:ec85556d2f3e
pub unsafe fn IAssemblyCache_AddRef(This: *mut IAssemblyCache) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:d16afa8d48cc
pub unsafe fn IAssemblyName_AddRef(This: *mut IAssemblyName) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:c14099b1141d
pub unsafe fn IAssemblyName_Release(This: *mut IAssemblyName) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD); // gen:da773c86944d
#[inline] // gen:903cd70d4a1b
pub unsafe fn IXMLDOMDocument_Release(This: *mut IXMLDOMDocument) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:3668d5030b6a
pub unsafe fn IXMLDOMDocument_getElementsByTagName(This: *mut IXMLDOMDocument, p: BSTR, resultList: *mut *mut IXMLDOMNodeList) -> HRESULT { ((*(*This).lpVtbl).getElementsByTagName.unwrap_unchecked())(This, p, resultList) } // widl C wrapper
#[inline] // gen:f84ef0e22e47
pub unsafe fn IXMLDOMDocument_load(This: *mut IXMLDOMDocument, var1: VARIANT, pbool: *mut VARIANT_BOOL) -> HRESULT { ((*(*This).lpVtbl).load.unwrap_unchecked())(This, var1, pbool) } // widl C wrapper
#[inline] // gen:217f53d01d67
pub unsafe fn IXMLDOMNamedNodeMap_Release(This: *mut IXMLDOMNamedNodeMap) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:1146d9b2c8b7
pub unsafe fn IXMLDOMNamedNodeMap_getNamedItem(This: *mut IXMLDOMNamedNodeMap, p: BSTR, outNode: *mut *mut IXMLDOMNode) -> HRESULT { ((*(*This).lpVtbl).getNamedItem.unwrap_unchecked())(This, p, outNode) } // widl C wrapper
#[inline] // gen:47d3c939ea61
pub unsafe fn IXMLDOMNodeList_Release(This: *mut IXMLDOMNodeList) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:af7875416304
pub unsafe fn IXMLDOMNodeList_get_length(This: *mut IXMLDOMNodeList, lLength: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).get_length.unwrap_unchecked())(This, lLength) } // widl C wrapper
#[inline] // gen:cb7db8171a59
pub unsafe fn IXMLDOMNodeList_nextNode(This: *mut IXMLDOMNodeList, outNode: *mut *mut IXMLDOMNode) -> HRESULT { ((*(*This).lpVtbl).nextNode.unwrap_unchecked())(This, outNode) } // widl C wrapper
#[inline] // gen:b8d526e2e916
pub unsafe fn IXMLDOMNode_Release(This: *mut IXMLDOMNode) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline] // gen:3c3e75c7dbc1
pub unsafe fn IXMLDOMNode_get_attributes(This: *mut IXMLDOMNode, attributeMap: *mut *mut IXMLDOMNamedNodeMap) -> HRESULT { ((*(*This).lpVtbl).get_attributes.unwrap_unchecked())(This, attributeMap) } // widl C wrapper
#[inline] // gen:858bd1cfb55a
pub unsafe fn IXMLDOMNode_get_nodeValue(This: *mut IXMLDOMNode, var1: *mut VARIANT) -> HRESULT { ((*(*This).lpVtbl).get_nodeValue.unwrap_unchecked())(This, var1) } // widl C wrapper
pub type REFIID = *const IID; // gen:c79168ffaea6
pub const S_FALSE: HRESULT = ((1) as HRESULT); // gen:b1b8213e7dac
pub const S_OK: HRESULT = ((0) as HRESULT); // gen:9969a2c48b97
pub use crate::{IsEqualIID, V_BSTR, V_VT};
// ==== end SDK header macros ====
