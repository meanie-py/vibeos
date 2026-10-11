//! mplay32-rust prelude — the single import surface the translated modules use.
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
pub const LANG_USER_DEFAULT: u32 = 1024; // diskpart-rust/src/prelude.rs
extern "C" {
    pub fn free(p: *mut c_void); // desk-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // avltable/src/tree.rs
}
extern "system" {
    pub fn CheckMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn DestroyAcceleratorTable(arg1: HACCEL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DragAcceptFiles(arg1: HWND, arg2: BOOL); // desk-rust/src/types_amd64.rs
    pub fn DragFinish(arg1: HDROP); // desk-rust/src/types_amd64.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_AddMasked( himl: HIMAGELIST, hbmImage: HBITMAP, crMask: COLORREF, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_amd64.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn MoveWindow( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn SetParent(arg1: HWND, arg2: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const MCI_ALL_DEVICE_ID: MCIDEVICEID = 0xFFFFFFFF; // clang, mplay32.c
pub const TB_ADDBUTTONS: i32 = 0x444; // clang, mplay32.c
pub const TTN_GETDISPINFO: u32 = 0xFFFFFDEE; // clang, mplay32.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, mplay32.c
pub type HIMAGELIST = *mut _IMAGELIST;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // mplay32.c
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t; // mplay32.c
}
extern "system" {
    pub fn CharLowerBuffW(lpsz: LPWSTR, cchLength: DWORD) -> DWORD; // mplay32.c
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
        hInstance_0: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND; // mplay32.c
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // mplay32.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // mplay32.c
    pub fn DragQueryFileW(hDrop: HDROP, iFile: UINT, lpszFile: LPWSTR, cch: UINT) -> UINT; // mplay32.c
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // mplay32.c
    pub fn GetMenuItemInfoW(_: HMENU, _: UINT, _: BOOL, _: LPMENUITEMINFOW) -> BOOL; // mplay32.c
    pub fn GetMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT) -> BOOL; // mplay32.c
    pub fn GetOpenFileNameW(_: LPOPENFILENAMEW) -> BOOL; // mplay32.c
    pub fn InsertMenuItemW(_: HMENU, _: UINT, _: BOOL, _: LPCMENUITEMINFOW) -> BOOL; // mplay32.c
    pub fn LoadAcceleratorsW(_: HINSTANCE, _: LPCWSTR) -> HACCEL; // mplay32.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // mplay32.c
    pub fn LoadIconW(hInstance_0: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // mplay32.c
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: LPCWSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // mplay32.c
    pub fn LoadStringW(
        hInstance_0: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // mplay32.c
    pub fn MessageBoxW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
    ) -> ::core::ffi::c_int; // mplay32.c
    pub fn PostMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // mplay32.c
    pub fn RegEnumValueW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpValueName: LPWSTR,
        lpcchValueName: LPDWORD,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // mplay32.c
    pub fn RegOpenKeyExW(_: HKEY, _: LPCWSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // mplay32.c
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpReserved: LPDWORD,
        lpcSubKeys: LPDWORD,
        lpcbMaxSubKeyLen: LPDWORD,
        lpcbMaxClassLen: LPDWORD,
        lpcValues: LPDWORD,
        lpcbMaxValueNameLen: LPDWORD,
        lpcbMaxValueLen: LPDWORD,
        lpcbSecurityDescriptor: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // mplay32.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // mplay32.c
    pub fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD; // mplay32.c
    pub fn SendMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // mplay32.c
    pub fn SetWindowTextW(_: HWND, _: LPCWSTR) -> BOOL; // mplay32.c
    pub fn ShellAboutW(
        hWnd: HWND,
        szApp: LPCWSTR,
        szOtherStuff: LPCWSTR,
        hIcon: HICON,
    ) -> ::core::ffi::c_int; // mplay32.c
    pub fn ShellExecuteW(
        hwnd: HWND,
        lpOperation: LPCWSTR,
        lpFile: LPCWSTR,
        lpParameters: LPCWSTR,
        lpDirectory: LPCWSTR,
        nShowCmd: INT,
    ) -> HINSTANCE; // mplay32.c
    pub fn TranslateAcceleratorW(_: HWND, _: HACCEL, _: LPMSG) -> ::core::ffi::c_int; // mplay32.c
    pub fn mciGetErrorStringW(mcierr: MCIERROR, pszText: LPWSTR, cchText: UINT) -> BOOL; // mplay32.c
    pub fn mciSendCommandW(_: MCIDEVICEID, _: UINT, _: DWORD_PTR, _: DWORD_PTR) -> MCIERROR; // mplay32.c
}
pub use crate::prelude::CharLowerBuffW as CharLowerBuff; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DragQueryFileW as DragQueryFile; // sdk alias
pub use crate::prelude::FormatMessageW as FormatMessage; // sdk alias
pub use crate::prelude::GetFileAttributesW as GetFileAttributes; // sdk alias
pub use crate::prelude::GetMenuItemInfoW as GetMenuItemInfo; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetOpenFileNameW as GetOpenFileName; // sdk alias
pub use crate::prelude::InsertMenuItemW as InsertMenuItem; // sdk alias
pub use crate::prelude::LoadAcceleratorsW as LoadAccelerators; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegEnumValueW as RegEnumValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryInfoKeyW as RegQueryInfoKey; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SearchPathW as SearchPath; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::ShellAboutW as ShellAbout; // sdk alias
pub use crate::prelude::ShellExecuteW as ShellExecute; // sdk alias
pub use strsafe_rust::StringCbCatW as StringCbCat; // sdk alias
pub use strsafe_rust::StringCbCopyW as StringCbCopy; // sdk alias
pub use strsafe_rust::StringCbCopyExW as StringCbCopyEx; // sdk alias
pub use strsafe_rust::StringCbPrintfW as StringCbPrintf; // sdk alias
pub use strsafe_rust::StringCbPrintfExW as StringCbPrintfEx; // sdk alias
pub use crate::prelude::TranslateAcceleratorW as TranslateAccelerator; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
pub use crate::prelude::wcsrchr as _tcsrchr; // sdk alias
pub use crate::prelude::mciGetErrorStringW as mciGetErrorString; // sdk alias
pub use crate::prelude::mciSendCommandW as mciSendCommand; // sdk alias
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
#[macro_export] macro_rules! CreateWindow { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub type LPTOOLTIPTEXT = LPNMTTDISPINFOW;
pub const TOOLBARCLASSNAME: *const u16 = (&[0x0054, 0x006f, 0x006f, 0x006c, 0x0062, 0x0061, 0x0072, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"ToolbarWindow32"
pub const TRACKBAR_CLASS: *const u16 = (&[0x006d, 0x0073, 0x0063, 0x0074, 0x006c, 0x0073, 0x005f, 0x0074, 0x0072, 0x0061, 0x0063, 0x006b, 0x0062, 0x0061, 0x0072, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"msctls_trackbar32"
pub use crate::{CreateWindow, MAKEINTRESOURCE};
// ==== end SDK header macros ====
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
