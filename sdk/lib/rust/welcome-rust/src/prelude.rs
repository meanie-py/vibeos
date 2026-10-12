//! welcome-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "system" {
    pub fn AdjustWindowRect(arg1: LPRECT, arg2: DWORD, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn BitBlt( hdcDest: HDC, xDest: INT, yDest: INT, cxDest: INT, cyDest: INT, hdcSrc: HDC, xSrc: INT, ySrc: INT, dwRop: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreatePen(arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: COLORREF) -> HPEN; // aclui-rust/src/types_i386.rs
    pub fn CreateSolidBrush(arg1: COLORREF) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetDlgCtrlID(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetForegroundWindow() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_i386.rs
    pub fn GetSystemMenu(arg1: HWND, arg2: BOOL) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowEnabled(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LineTo(arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn MoveToEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: LPPOINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn RemoveMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_i386.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MAX_NUMBER_INTERNAL_TOPICS: i32 = 3; // clang, welcome.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsicmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // welcome.c
    pub fn _wcsnicmp(
        _Str1: *const wchar_t,
        _Str2: *const wchar_t,
        _MaxCount: size_t,
    ) -> ::core::ffi::c_int; // welcome.c
    pub fn wcschr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut ::core::ffi::c_ushort; // welcome.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // welcome.c
    pub fn wcstoul(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong; // welcome.c
}
extern "system" {
    pub fn CallWindowProcW(_: WNDPROC, _: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // welcome.c
    pub fn CreateFontIndirectW(_: *const LOGFONTW) -> HFONT; // welcome.c
    pub fn CreateMutexW(_: LPSECURITY_ATTRIBUTES, _: BOOL, _: LPCWSTR) -> HANDLE; // welcome.c
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // welcome.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // welcome.c
    pub fn DrawTextW(
        hdc: HDC,
        lpchText: LPCWSTR,
        cchText: ::core::ffi::c_int,
        lprc: LPRECT,
        format: UINT,
    ) -> ::core::ffi::c_int; // welcome.c
    pub fn FindWindowW(_: LPCWSTR, _: LPCWSTR) -> HWND; // welcome.c
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // welcome.c
    pub fn GetLocaleInfoW(
        Locale: LCID,
        LCType: LCTYPE,
        lpLCData: LPWSTR,
        cchData: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // welcome.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // welcome.c
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // welcome.c
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // welcome.c
    pub fn GetPrivateProfileIntW(_: LPCWSTR, _: LPCWSTR, _: INT, _: LPCWSTR) -> UINT; // welcome.c
    pub fn GetPrivateProfileSectionNamesW(
        lpszReturnBuffer: LPWSTR,
        nSize: DWORD,
        lpFileName: LPCWSTR,
    ) -> DWORD; // welcome.c
    pub fn GetPrivateProfileStringW(
        lpAppName: LPCWSTR,
        lpKeyName: LPCWSTR,
        lpDefault: LPCWSTR,
        lpReturnedString: LPWSTR,
        nSize: DWORD,
        lpFileName: LPCWSTR,
    ) -> DWORD; // welcome.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // welcome.c
    pub fn GetWindowTextW(
        hWnd: HWND,
        lpString: LPWSTR,
        nMaxCount: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // welcome.c
    pub fn GetWindowsDirectoryW(_: LPWSTR, _: UINT) -> UINT; // welcome.c
    pub fn IsDialogMessageW(_: HWND, _: LPMSG) -> BOOL; // welcome.c
    pub fn LoadBitmapW(_: HINSTANCE, _: LPCWSTR) -> HBITMAP; // welcome.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // welcome.c
    pub fn LoadIconW(hInstance_0: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // welcome.c
    pub fn LoadStringW(
        hInstance_0: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // welcome.c
    pub fn MessageBoxW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
    ) -> ::core::ffi::c_int; // welcome.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // welcome.c
    pub fn SendMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // welcome.c
    pub fn SetCurrentDirectoryW(_: LPCWSTR) -> BOOL; // welcome.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // welcome.c
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: INT,
    ) -> HINSTANCE; // welcome.c
}
pub use crate::prelude::CallWindowProcW as CallWindowProc; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::CreateMutexW as CreateMutex; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DrawTextW as DrawText; // sdk alias
pub use crate::prelude::FindWindowW as FindWindow; // sdk alias
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::GetLocaleInfoW as GetLocaleInfo; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetModuleFileNameW as GetModuleFileName; // sdk alias
pub use crate::prelude::GetObjectW as GetObject; // sdk alias
pub use crate::prelude::GetPrivateProfileIntW as GetPrivateProfileInt; // sdk alias
pub use crate::prelude::GetPrivateProfileSectionNamesW as GetPrivateProfileSectionNames; // sdk alias
pub use crate::prelude::GetPrivateProfileStringW as GetPrivateProfileString; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::GetWindowsDirectoryW as GetWindowsDirectory; // sdk alias
pub use crate::prelude::IsDialogMessageW as IsDialogMessage; // sdk alias
pub use crate::prelude::LoadBitmapW as LoadBitmap; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetCurrentDirectoryW as SetCurrentDirectory; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::ShellExecuteW as ShellExecute; // sdk alias
pub use strsafe_rust::StringCchCatW as StringCchCat; // sdk alias
pub use strsafe_rust::StringCchCopyW as StringCchCopy; // sdk alias
pub use strsafe_rust::StringCchPrintfW as StringCchPrintf; // sdk alias
pub use crate::prelude::wcschr as _tcschr; // sdk alias
pub use crate::prelude::_wcsicmp as _tcsicmp; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::_wcsnicmp as _tcsnicmp; // sdk alias
pub use crate::prelude::wcsrchr as _tcsrchr; // sdk alias
pub use crate::prelude::wcstoul as _tcstoul; // sdk alias
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
pub type WNDCLASSEX = WNDCLASSEXW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateWindow { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub use crate::{CreateWindow, MAKEINTRESOURCE, MAKELPARAM};
// ==== end SDK header macros ====
