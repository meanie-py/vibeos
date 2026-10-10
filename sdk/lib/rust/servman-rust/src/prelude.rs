//! servman-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT; // comctl32-rust/src/prelude.rs
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR; // comctl32-rust/src/prelude.rs
pub const LANG_USER_DEFAULT: u32 = 1024; // cacls-rust/src/prelude.rs
pub const LVM_SETITEMTEXT: i32 = 0x1074; // msconfig-rust/src/prelude.rs
pub const RT_STRING: LPCWSTR = 6usize as LPCWSTR; // comctl32-rust/src/prelude.rs
pub const SB_SETTEXT: i32 = 0x40B; // charmap-rust/src/prelude.rs
pub const SC_ACTION_NONE: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_NONE; // island enum
pub const SC_ACTION_REBOOT: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_REBOOT; // island enum
pub const SC_ACTION_RESTART: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_RESTART; // island enum
pub const SC_ACTION_RUN_COMMAND: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_RUN_COMMAND; // island enum
pub const SC_ENUM_PROCESS_INFO: _SC_ENUM_TYPE = _SC_ENUM_TYPE_SC_ENUM_PROCESS_INFO; // island enum
pub const SC_STATUS_PROCESS_INFO: _SC_STATUS_TYPE = _SC_STATUS_TYPE_SC_STATUS_PROCESS_INFO; // island enum
pub const TB_ADDBUTTONS: i32 = 0x444; // mplay32-rust/src/prelude.rs
extern "C" {
    pub fn _beginthreadex( security: *mut c_void, stack_size: u32, start_address: Option<unsafe extern "system" fn(*mut c_void) -> u32>, arglist: *mut c_void, initflag: u32, thrdaddr: *mut u32, ) -> usize; // explorer-rust/src/explorer/syspager.rs
    pub fn _endthreadex(retval: u32); // msvcrt-crt-rust/src/msvcrt/thread.rs
    pub fn _snwprintf(buf: *mut u16, count: usize, format: *const u16, ...) -> c_int; // browser-rust/src/prelude.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // control-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
}
extern "system" {
    pub fn ChangeServiceConfig2W(arg1: SC_HANDLE, arg2: DWORD, arg3: LPVOID) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn ChangeServiceConfigW( arg1: SC_HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: LPCWSTR, arg6: LPCWSTR, arg7: LPDWORD, arg8: LPCWSTR, arg9: LPCWSTR, arg10: LPCWSTR, arg11: LPCWSTR, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn CheckMenuRadioItem(arg1: HMENU, arg2: UINT, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn CommDlgExtendedError() -> DWORD; // desk-rust/src/types_i386.rs
    pub fn ControlService(arg1: SC_HANDLE, arg2: DWORD, arg3: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateServiceW( arg1: SC_HANDLE, arg2: LPCWSTR, arg3: LPCWSTR, arg4: DWORD, arg5: DWORD, arg6: DWORD, arg7: DWORD, arg8: LPCWSTR, arg9: LPCWSTR, arg10: PDWORD, arg11: LPCWSTR, arg12: LPCWSTR, arg13: LPCWSTR, ) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteService(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnumDependentServicesW( hService: SC_HANDLE, dwServiceState: DWORD, lpServices: LPENUM_SERVICE_STATUSW, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, lpServicesReturned: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn FindResourceW(arg1: HMODULE, arg2: LPCWSTR, arg3: LPCWSTR) -> HRSRC; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMenuState(arg1: HMENU, arg2: UINT, arg3: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetTickCount() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetUserDefaultUILanguage() -> LANGID; // desk-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn ImageList_AddMasked( himl: HIMAGELIST, hbmImage: HBITMAP, crMask: COLORREF, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LoadAcceleratorsW(arg1: HINSTANCE, arg2: LPCWSTR) -> HACCEL; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn LoadResource(arg1: HINSTANCE, arg2: HRSRC) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LockResource(arg1: HGLOBAL) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn LockServiceDatabase(arg1: SC_HANDLE) -> SC_LOCK; // syssetup-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // syssetup-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn PropertySheetW(arg1: LPCPROPSHEETHEADERW) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn PtInRect(arg1: LPCRECT, arg2: POINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn QueryServiceConfig2W( hService: SC_HANDLE, dwInfoLevel: DWORD, lpBuffer: LPBYTE, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn QueryServiceConfigW( hService: SC_HANDLE, lpServiceConfig: LPQUERY_SERVICE_CONFIGW, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn QueryServiceStatusEx( hService: SC_HANDLE, InfoLevel: SC_STATUS_TYPE, lpBuffer: LPBYTE, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SetMenuDefaultItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR; // winuser.h (shell32-rust spells LONG_PTR isize: same width, a different Rust type on i386)
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShellAboutW( hWnd: HWND, szApp: LPCWSTR, szOtherStuff: LPCWSTR, hIcon: HICON, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn StartServiceW( hService: SC_HANDLE, dwNumServiceArgs: DWORD, lpServiceArgVectors: *mut LPCWSTR, ) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn TrackPopupMenuEx( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: HWND, arg6: LPTPMPARAMS, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateAcceleratorW(arg1: HWND, arg2: HACCEL, arg3: LPMSG) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnlockServiceDatabase(arg1: SC_LOCK) -> BOOL; // syssetup-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::servman::servman::ProcessHeap;
pub use crate::servman::servman::hInstance;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const LVM_GETITEM: i32 = 0x104B; // clang, query.c
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // clang, mainwnd.c
pub const NM_RETURN: u32 = 0xFFFFFFFC; // clang, mainwnd.c
pub const TTN_GETDISPINFO: u32 = 0xFFFFFDEE; // clang, mainwnd.c
pub const TVN_ITEMEXPANDING: u32 = 0xFFFFFE3A; // clang, propsheet_depends.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// COLUMN_LIST: c2rust, listview.c
pub type COLUMN_LIST = _COLUMN_LIST;
// HIMAGELIST: c2rust, listview.c
pub type HIMAGELIST = *mut _IMAGELIST;
// HTREEITEM: c2rust, dependencies_tv1.c
pub type HTREEITEM = *mut _TREEITEM;
// _COLUMN_LIST: c2rust, listview.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _COLUMN_LIST {
    pub iSubItem: ::core::ffi::c_int,
    pub cx: ::core::ffi::c_int,
    pub idsText: UINT,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn wsprintfW(_: LPWSTR, _: LPCWSTR, ...) -> ::core::ffi::c_int; // propsheet_recovery.c
}
extern "system" {
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // mainwnd.c
    pub fn EnumServicesStatusExW(
        hSCManager: SC_HANDLE,
        InfoLevel: SC_ENUM_TYPE,
        dwServiceType: DWORD,
        dwServiceState: DWORD,
        lpServices: LPBYTE,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
        lpServicesReturned: LPDWORD,
        lpResumeHandle: LPDWORD,
        pszGroupName: LPCWSTR,
    ) -> BOOL; // query.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // propsheet_logon.c
    pub fn GetOpenFileNameW(_: LPOPENFILENAMEW) -> BOOL; // propsheet_recovery.c
    pub fn GetSaveFileNameW(_: LPOPENFILENAMEW) -> BOOL; // export.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // mainwnd.c
    pub fn LoadCursorW(_: HINSTANCE, _: LPCWSTR) -> HCURSOR; // mainwnd.c
    pub fn LoadMenuW(_: HINSTANCE, _: LPCWSTR) -> HMENU; // mainwnd.c
    pub fn PostMessageW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> BOOL; // mainwnd.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // mainwnd.c
    pub fn SetDlgItemTextW(_: HWND, _: ::core::ffi::c_int, _: LPCWSTR) -> BOOL; // propsheet_logon.c
    pub fn UnregisterClassW(_: LPCWSTR, _: HINSTANCE) -> BOOL; // mainwnd.c
}
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DialogBoxParamW as DialogBoxParam; // sdk alias
pub use crate::prelude::EnumDependentServicesW as EnumDependentServices; // sdk alias
pub use crate::prelude::EnumServicesStatusExW as EnumServicesStatusEx; // sdk alias
pub use crate::prelude::GetDlgItemTextW as GetDlgItemText; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetOpenFileNameW as GetOpenFileName; // sdk alias
pub use crate::prelude::GetSaveFileNameW as GetSaveFileName; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::LoadMenuW as LoadMenu; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::OpenSCManagerW as OpenSCManager; // sdk alias
pub use crate::prelude::OpenServiceW as OpenService; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::QueryServiceConfig2W as QueryServiceConfig2; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use strsafe_rust::StringCchCopyW as StringCchCopy; // sdk alias
pub use crate::prelude::UnregisterClassW as UnregisterClass; // sdk alias
pub use crate::prelude::wsprintfW as wsprintf; // sdk alias
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
#[macro_export] macro_rules! CreateDialog { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3 /* DLGPROC: an Option has no `as` cast */, ((0) as _)) }; }
#[macro_export] macro_rules! Header_GetItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 11)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! Header_SetItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 12)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! ListView_Arrange { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 22)) as _), (((($p1) as UINT)) as WPARAM), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 9)) as _), ((0) as _), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_DeleteItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 8)) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_FindItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 83)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetHeader { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 31)) as _), ((0) as _), ((0) as _))) as HWND) }; }
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetItemText { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.iSubItem = (($p2) as _); }; { _ms_lvi.cchTextMax = (($p4) as _); }; { _ms_lvi.pszText = (($p3) as _); }; SendMessageW((($p0) as _), (((0x1000 + 115)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 77)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_SetExtendedListViewStyle { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), ((0) as _), (($p1) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 3)) as _), (($p2) as WPARAM), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! ListView_SortItemsEx { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 81)) as _), (((($p2) as LPARAM)) as WPARAM), (($p1) as usize as LPARAM) /* PFNLVCOMPARE is Option<fn>: an Option has no integer cast */)) as BOOL) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as WPARAM) }; }
#[macro_export] macro_rules! PropSheet_Changed { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), (((1024 + 104)) as _), (($p1) as WPARAM), ((0) as _)) }; }
#[macro_export] macro_rules! PropSheet_GetCurrentPageHwnd { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), ((1142) as _), ((0) as _), ((0) as _))) as HWND) }; }
#[macro_export] macro_rules! PropSheet_IsDialogMessage { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), ((1141) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetChild { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x4) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetFirstVisible { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x5) as WPARAM), ((core::ptr::null_mut::<c_void>()) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 62)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetNextSibling { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x1) as WPARAM), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 50)) as _), ((0) as _), (($p1) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 9)) as _), (($p2) as _), (($p1) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub type HDITEM = HDITEMW;
pub type LPNMTREEVIEW = LPNMTREEVIEWW;
pub type LPPROPSHEETPAGE = LPPROPSHEETPAGEW;
pub type LPTOOLTIPTEXT = LPNMTTDISPINFOW;
pub type LVITEM = LVITEMW;
pub type PROPSHEETHEADER = PROPSHEETHEADERW;
pub type PROPSHEETPAGE = PROPSHEETPAGEW;
pub const STATUSCLASSNAME: *const u16 = (&[0x006d, 0x0073, 0x0063, 0x0074, 0x006c, 0x0073, 0x005f, 0x0073, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0062, 0x0061, 0x0072, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"msctls_statusbar32"
pub const TOOLBARCLASSNAME: *const u16 = (&[0x0054, 0x006f, 0x006f, 0x006c, 0x0062, 0x0061, 0x0072, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"ToolbarWindow32"
pub use crate::{CreateDialog, Header_GetItem, Header_SetItem, ImageList_AddIcon, ListView_Arrange, ListView_DeleteAllItems, ListView_DeleteItem, ListView_FindItem, ListView_GetHeader, ListView_GetItemCount, ListView_GetItemText, ListView_InsertColumn, ListView_InsertItem, ListView_SetExtendedListViewStyle, ListView_SetImageList, ListView_SortItemsEx, MAKEINTRESOURCE, MAKELPARAM, MAKEWPARAM, PropSheet_Changed, PropSheet_GetCurrentPageHwnd, PropSheet_IsDialogMessage, TreeView_GetChild, TreeView_GetFirstVisible, TreeView_GetItem, TreeView_GetNextSibling, TreeView_InsertItem, TreeView_SetImageList, min, va_end, va_start};
// ==== end SDK header macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn LoadAndFormatString(hInstance: HINSTANCE, uID: UINT, lpTarget: *mut LPWSTR, ...) -> DWORD;
    pub fn StatusBarLoadAndFormatString(hStatusBar: HWND, PartId: INT, hInstance_0: HINSTANCE, uID: UINT, ...) -> BOOL;
}
// ==== end port variadic ====
