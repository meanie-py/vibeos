//! fontview-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

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
extern "C" {
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // rtl-actctx/src/shim.rs
    pub fn free(p: *mut c_void); // desk-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // avltable/src/tree.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn AddFontResourceW(pszFilename: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn BeginDeferWindowPos(arg1: ::core::ffi::c_int) -> HDWP; // aclui-rust/src/types_amd64.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CommandLineToArgvW(arg1: LPCWSTR, arg2: *mut ::core::ffi::c_int) -> *mut LPWSTR; // desk-rust/src/types_amd64.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DeferWindowPos( arg1: HDWP, arg2: HWND, arg3: HWND, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: UINT, ) -> HDWP; // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDeferWindowPos(arg1: HDWP) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDoc(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EndPage(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumFontFamiliesExW( arg1: HDC, arg2: PLOGFONTW, arg3: FONTENUMPROCW, arg4: LPARAM, arg5: DWORD, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDeviceCaps(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetFontResourceInfoW(lpFileName: LPCWSTR, pdwBufSize: *mut DWORD, lpBuffer: PVOID, dwType: DWORD,) -> BOOL; // gdi32-rust/src/objects/text.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetOutlineTextMetricsW(hdc: HDC, cjCopy: UINT, potm: LPOUTLINETEXTMETRICW) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetScrollInfo(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPSCROLLINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_amd64.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LineTo(arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MoveToEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn RemoveFontResourceW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ScrollWindowEx( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPCRECT, arg5: LPCRECT, arg6: HRGN, arg7: LPRECT, arg8: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetScrollInfo( arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCSCROLLINFO, arg4: BOOL, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StartPage(arg1: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn TextOutW( hdc: HDC, x: ::core::ffi::c_int, y: ::core::ffi::c_int, lpString: LPCWSTR, c: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::fontview::display::g_szFontDisplayClassName;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const HWND_DESKTOP: HWND = 0x0usize as HWND; // clang, fontview.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // fontview.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // fontview.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // fontview.c
    pub fn GetTextMetricsW(_: HDC, _: LPTEXTMETRICW) -> BOOL; // display.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // display.c
    pub fn IsDialogMessageW(_: HWND, _: LPMSG) -> BOOL; // fontview.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // display.c
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // fontview.c
    pub fn PostMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // fontview.c
    pub fn PrintDlgW(_: LPPRINTDLGW) -> BOOL; // display.c
    pub fn SendDlgItemMessageW(
        _: HWND,
        _: ::core::ffi::c_int,
        _: UINT,
        _: WPARAM,
        _: LPARAM,
    ) -> LRESULT; // fontview.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // display.c
    pub fn StartDocW(_: HDC, _: *const DOCINFOW) -> ::core::ffi::c_int; // display.c
}
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetTextMetricsW as GetTextMetrics; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::IsDialogMessageW as IsDialogMessage; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::PrintDlgW as PrintDlg; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::StartDocW as StartDoc; // sdk alias
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
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND;
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub use crate::{MAKEINTRESOURCE, va_end, va_start};
// ==== end SDK header macros ====
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn ErrorMsgBox(hParent: HWND, dwMessageId: DWORD, ...);
    pub fn SuccessMsgBox(hParent: HWND, dwMessageId: DWORD, ...);
}
// ==== end port variadic ====
