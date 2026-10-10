//! msconfig-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const PATH_MAX: u32 = 259; // ntkrnl-sys/src/hal_bindgen_amd64.rs
pub const SC_ACTION_REBOOT: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_REBOOT; // island enum
pub const SC_ENUM_PROCESS_INFO: _SC_ENUM_TYPE = _SC_ENUM_TYPE_SC_ENUM_PROCESS_INFO; // island enum
extern "C" {
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32; // kernel32-rust/src/client/vdm.rs
    pub fn _wfopen(file: *const u16, mode: *const u16) -> *mut c_void; // ucrt-rust/src/stdio/files/open.rs
    pub fn _wtoi(s: *const u16) -> i32; // userinit-rust/src/preamble.rs
    pub fn fclose(stream: *mut FILE) -> c_int; // dnsrslvr-rust/src/prelude.rs
    pub fn feof(file: *mut FILE) -> i32; // msvcrt-crt-rust/src/msvcrt/file.rs
    pub fn fgetws(s: *mut wchar_t, size: i32, file: *mut FILE) -> *mut wchar_t; // msvcrt-crt-rust/src/msvcrt/file.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn BringWindowToTop(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // netcfgx-rust/src/prelude.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindWindowExW(arg1: HWND, arg2: HWND, arg3: LPCWSTR, arg4: LPCWSTR) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsWindowEnabled(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn MessageBeep(uType: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::msconfig::freeldrpage::hFreeLdrPage;
pub use crate::msconfig::generalpage::hGeneralPage;
pub use crate::msconfig::msconfig::hInst;
pub use crate::msconfig::srvpage::hServicesListCtrl;
pub use crate::msconfig::srvpage::hServicesPage;
pub use crate::msconfig::startuppage::hStartupListCtrl;
pub use crate::msconfig::startuppage::hStartupPage;
pub use crate::msconfig::systempage::hSystemPage;
pub use crate::msconfig::toolspage::hToolsListCtrl;
pub use crate::msconfig::toolspage::hToolsPage;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const LVM_SETITEMTEXT: i32 = 0x1074; // clang, toolspage.c
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // clang, toolspage.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HTREEITEM: c2rust, systempage.c
pub type HTREEITEM = *mut _TREEITEM;
// ==== end port local types ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateDialog { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! DialogBox { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetSelectionMark { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 66)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 77)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_SetCheckState { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = ((0xF000) as _); }; { _ms_lvi.state = ((((if ($p2) as isize != 0 { 2 } else { 1 }) << 12)) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! TabCtrl_GetCurSel { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 11)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! TabCtrl_InsertItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 62)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! TreeView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 50)) as _), ((0) as _), (((($p1) as LPTVINSERTSTRUCTW)) as LPARAM))) as HTREEITEM) }; }
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub type LV_COLUMN = LVCOLUMNW;
pub type LV_ITEM = LVITEMW;
pub type TCITEM = TCITEMW;
pub type TVINSERTSTRUCT = TVINSERTSTRUCTW;
pub const TVI_LAST: HTREEITEM = (((((-(0xfffe as c_int))) as ULONG_PTR)) as HTREEITEM);
pub use crate::{CreateDialog, DialogBox, ListView_GetItemCount, ListView_GetSelectionMark, ListView_InsertColumn, ListView_InsertItem, ListView_SetCheckState, MAKEINTRESOURCE, TabCtrl_GetCurSel, TabCtrl_InsertItem, TreeView_InsertItem};
// ==== end SDK header macros ====
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // startuppage.c
    pub fn wcscat(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t; // toolspage.c
    pub fn wcscmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> ::core::ffi::c_int; // startuppage.c
    pub fn wcscspn(_Str: *const wchar_t, _Control: *const wchar_t) -> size_t; // srvpage.c
    pub fn wsprintfW(_: LPWSTR, _: LPCWSTR, ...) -> ::core::ffi::c_int; // srvpage.c
}
extern "system" {
    pub fn CreateProcessW(
        _: LPCWSTR,
        _: LPWSTR,
        _: LPSECURITY_ATTRIBUTES,
        _: LPSECURITY_ATTRIBUTES,
        _: BOOL,
        _: DWORD,
        _: PVOID,
        _: LPCWSTR,
        _: LPSTARTUPINFOW,
        _: LPPROCESS_INFORMATION,
    ) -> BOOL; // toolspage.c
    pub fn DefWindowProcW(_: HWND, _: UINT, _: WPARAM, _: LPARAM) -> LRESULT; // msconfig.c
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
    ) -> BOOL; // srvpage.c
    pub fn GetFileVersionInfoSizeW(_: LPCWSTR, _: PDWORD) -> DWORD; // srvpage.c
    pub fn GetFileVersionInfoW(_: LPCWSTR, _: DWORD, _: DWORD, _: PVOID) -> BOOL; // srvpage.c
    pub fn GetLongPathNameW(_: LPCWSTR, _: LPWSTR, _: DWORD) -> DWORD; // startuppage.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // msconfig.c
    pub fn LoadImageW(
        hInst_0: HINSTANCE,
        name: LPCWSTR,
        type_0: UINT,
        cx: ::core::ffi::c_int,
        cy: ::core::ffi::c_int,
        fuLoad: UINT,
    ) -> HANDLE; // msconfig.c
    pub fn LoadLibraryW(_: LPCWSTR) -> HINSTANCE; // msconfig.c
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int; // toolspage.c
    pub fn OpenSCManagerW(_: LPCWSTR, _: LPCWSTR, _: DWORD) -> SC_HANDLE; // srvpage.c
    pub fn OpenServiceW(_: SC_HANDLE, _: LPCWSTR, _: DWORD) -> SC_HANDLE; // srvpage.c
    pub fn PathAddBackslashW(_: LPWSTR) -> LPWSTR; // toolspage.c
    pub fn QueryServiceConfig2W(
        hService: SC_HANDLE,
        dwInfoLevel: DWORD,
        lpBuffer: LPBYTE,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
    ) -> BOOL; // srvpage.c
    pub fn QueryServiceConfigW(
        hService: SC_HANDLE,
        lpServiceConfig: LPQUERY_SERVICE_CONFIGW,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
    ) -> BOOL; // srvpage.c
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // startuppage.c
    pub fn RegEnumValueW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpValueName: LPWSTR,
        lpcchValueName: LPDWORD,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS; // startuppage.c
    pub fn RegOpenKeyExW(_: HKEY, _: LPCWSTR, _: DWORD, _: REGSAM, _: PHKEY) -> LSTATUS; // startuppage.c
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
    ) -> LSTATUS; // startuppage.c
    pub fn SHGetSpecialFolderPathW(
        hwndOwner: HWND,
        szPath: LPWSTR,
        nFolder: ::core::ffi::c_int,
        bCreate: BOOL,
    ) -> BOOL; // toolspage.c
    pub fn SendDlgItemMessageW(
        _: HWND,
        _: ::core::ffi::c_int,
        _: UINT,
        _: WPARAM,
        _: LPARAM,
    ) -> LRESULT; // toolspage.c
    pub fn SetWindowTextW(_: HWND, _: LPCWSTR) -> BOOL; // msconfig.c
    pub fn VerQueryValueW(_: LPCVOID, _: LPCWSTR, _: *mut LPVOID, _: PUINT) -> BOOL; // srvpage.c
}
pub use crate::prelude::CreateProcessW as CreateProcess; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::EnumServicesStatusExW as EnumServicesStatusEx; // sdk alias
pub use crate::prelude::GetFileVersionInfoW as GetFileVersionInfo; // sdk alias
pub use crate::prelude::GetFileVersionInfoSizeW as GetFileVersionInfoSize; // sdk alias
pub use crate::prelude::GetLongPathNameW as GetLongPathName; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::OpenSCManagerW as OpenSCManager; // sdk alias
pub use crate::prelude::OpenServiceW as OpenService; // sdk alias
pub use crate::prelude::PathAddBackslashW as PathAddBackslash; // sdk alias
pub use crate::prelude::QueryServiceConfigW as QueryServiceConfig; // sdk alias
pub use crate::prelude::QueryServiceConfig2W as QueryServiceConfig2; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegEnumValueW as RegEnumValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryInfoKeyW as RegQueryInfoKey; // sdk alias
pub use crate::prelude::SHGetSpecialFolderPathW as SHGetSpecialFolderPath; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use strsafe_rust::StringCchCopyNW as StringCchCopyN; // sdk alias
pub use crate::prelude::VerQueryValueW as VerQueryValue; // sdk alias
pub use crate::prelude::_swprintf as _stprintf; // sdk alias
pub use crate::prelude::wcscat as _tcscat; // sdk alias
pub use crate::prelude::wcslen as _tcsclen; // sdk alias
pub use crate::prelude::wcscmp as _tcscmp; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::wcscspn as _tcscspn; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
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
// ==== end UNICODE TCHAR family ====
