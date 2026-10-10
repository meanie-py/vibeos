//! mmc-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};
// mscfile.c's `#include <initguid.h>` + <msxml2.h> DEFINED these in the C object. The Rust
// takes them from the uuid library mmc already links (sdk/lib/uuid builds msxml.idl).
extern "C" {
    pub static IID_IXMLDOMElement: IID;
    pub static IID_IXMLDOMDocument: IID;
    pub static IID_IXMLDOMNode: IID;
}

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // explorer-sys/src/lib.rs
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // aclui-rust/src/prelude.rs
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // comctl32-rust/src/prelude.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const VARIANT_FALSE: VARIANT_BOOL = 0; // oleaut32-rust/src/olesys.rs
pub const VT_BSTR: DWORD = 8; // shell32-rust/src/shell32/CFolderItem.rs
extern "system" {
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_amd64.rs
    pub fn CommandLineToArgvW(arg1: LPCWSTR, arg2: *mut ::core::ffi::c_int) -> *mut LPWSTR; // desk-rust/src/types_amd64.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindResourceW(arg1: HMODULE, arg2: LPCWSTR, arg3: LPCWSTR) -> HRSRC; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_amd64.rs
    pub fn LoadResource(arg1: HINSTANCE, arg2: HRSRC) -> HGLOBAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn LockResource(arg1: HGLOBAL) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetMenu(arg1: HWND, arg2: HMENU) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShellExecuteExW(arg1: LPSHELLEXECUTEINFOW) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SysAllocString(psz: *const u16) -> BSTR; // ole32-rust/src/ole32_exports.rs
    pub fn SysFreeString(bstr: BSTR); // oleaut32-rust/src/typelib.rs
    pub fn TranslateMDISysAccel(arg1: HWND, arg2: LPMSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UrlCreateFromPathW( arg1: LPCWSTR, arg2: LPWSTR, pcchUrl: LPDWORD, arg3: DWORD, ) -> HRESULT; // desk-rust/src/types_amd64.rs
    pub fn VariantClear(pvarg: *mut VARIANT) -> HRESULT; // oleaut32-rust/src/typelib/sys.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mmc::mmc::hAppHeap;
pub use crate::mmc::mmc::hAppInstance;
pub use crate::mmc::mmc::hwndMDIClient;
pub use crate::mmc::mmc::hwndMainConsole;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IDI_APPLICATION: LPWSTR = 0x7F00usize as LPWSTR; // clang, console.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // console.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        X: ::core::ffi::c_int,
        Y: ::core::ffi::c_int,
        nWidth: ::core::ffi::c_int,
        nHeight: ::core::ffi::c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND; // console.c
    pub fn DefFrameProcW(_: HWND, _: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // console.c
    pub fn DefMDIChildProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // console.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // mmc.c
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // mmc.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // console.c
    pub fn GetSaveFileNameW(_: LPOPENFILENAMEW) -> BOOL; // console.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // console.c
    pub fn GetWindowTextW(
        hWnd: HWND,
        lpString: LPWSTR,
        nMaxCount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // console.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // console.c
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // console.c
    pub fn LoadMenuW(_: HINSTANCE, _: LPCWSTR) -> HMENU; // console.c
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // misc.c
    pub fn PostMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // console.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // console.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // console.c
    pub fn SetWindowTextW(_: HWND, _: LPCWSTR) -> BOOL; // console.c
    pub fn ShellAboutW(
        hWnd: HWND,
        szApp: LPCWSTR,
        szOtherStuff: LPCWSTR,
        hIcon: HICON,
    ) -> ::core::ffi::c_int; // console.c
    pub fn UnregisterClassW(_: LPCWSTR, _: HINSTANCE) -> BOOL; // console.c
}
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefFrameProcW as DefFrameProc; // sdk alias
pub use crate::prelude::DefMDIChildProcW as DefMDIChildProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::FormatMessageW as FormatMessage; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetSaveFileNameW as GetSaveFileName; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadMenuW as LoadMenu; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::ShellAboutW as ShellAbout; // sdk alias
pub use strsafe_rust::StringCbCatW as StringCbCat; // sdk alias
pub use strsafe_rust::StringCbCopyW as StringCbCopy; // sdk alias
pub use crate::prelude::UnregisterClassW as UnregisterClass; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
// ==== end port externs ====

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
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
#[inline]
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMDocument_get_documentElement(This: *mut IXMLDOMDocument, DOMElement: *mut *mut IXMLDOMElement) -> HRESULT { ((*(*This).lpVtbl).get_documentElement.unwrap_unchecked())(This, DOMElement) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMDocument_load(This: *mut IXMLDOMDocument, xmlSource: VARIANT, isSuccessful: *mut VARIANT_BOOL) -> HRESULT { ((*(*This).lpVtbl).load.unwrap_unchecked())(This, xmlSource, isSuccessful) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMElement_getElementsByTagName(This: *mut IXMLDOMElement, tagName: BSTR, resultList: *mut *mut IXMLDOMNodeList) -> HRESULT { ((*(*This).lpVtbl).getElementsByTagName.unwrap_unchecked())(This, tagName, resultList) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMElement_get_nodeName(This: *mut IXMLDOMElement, name: *mut BSTR) -> HRESULT { ((*(*This).lpVtbl).get_nodeName.unwrap_unchecked())(This, name) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMNamedNodeMap_Release(This: *mut IXMLDOMNamedNodeMap) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMNamedNodeMap_getNamedItem(This: *mut IXMLDOMNamedNodeMap, name: BSTR, namedItem: *mut *mut IXMLDOMNode) -> HRESULT { ((*(*This).lpVtbl).getNamedItem.unwrap_unchecked())(This, name, namedItem) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMNodeList_get_item(This: *mut IXMLDOMNodeList, index: LONG, listItem: *mut *mut IXMLDOMNode) -> HRESULT { ((*(*This).lpVtbl).get_item.unwrap_unchecked())(This, index, listItem) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMNode_Release(This: *mut IXMLDOMNode) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMNode_get_attributes(This: *mut IXMLDOMNode, attributeMap: *mut *mut IXMLDOMNamedNodeMap) -> HRESULT { ((*(*This).lpVtbl).get_attributes.unwrap_unchecked())(This, attributeMap) } // widl C wrapper
#[inline]
pub unsafe fn IXMLDOMNode_get_text(This: *mut IXMLDOMNode, text: *mut BSTR) -> HRESULT { ((*(*This).lpVtbl).get_text.unwrap_unchecked())(This, text) } // widl C wrapper
pub const RT_STRING: LPWSTR = 0x6usize as LPWSTR;
pub use crate::{MAKEINTRESOURCE, va_end, va_start};
// ==== end SDK header macros ====
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn LoadAndFormatString(hInstance: HINSTANCE, uID: UINT, lpTarget: *mut LPTSTR, ...) -> DWORD;
}
// ==== end port variadic ====
