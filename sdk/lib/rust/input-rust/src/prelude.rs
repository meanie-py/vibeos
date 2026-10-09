//! input-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IntToPtr, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// muifonts.h's header-defined arrays (src/muifonts.rs).
pub use crate::muifonts::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsdup(text: LPCWSTR) -> LPWSTR; // shell32-rust/src/new_menu.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wtoi(s: *const u16) -> i32; // userinit-rust/src/preamble.rs
    pub fn free(p: *mut c_void); // desk-rust/src/prelude.rs
    pub fn malloc(size: usize) -> *mut c_void; // avltable/src/tree.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcstok(s: *mut u16, delim: *const u16) -> *mut u16; // desk-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // concfg-rust/src/font.rs
}
extern "system" {
    pub fn ActivateKeyboardLayout(arg1: HKL, arg2: UINT) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn AdjustTokenPrivileges( arg1: HANDLE, arg2: BOOL, arg3: PTOKEN_PRIVILEGES, arg4: DWORD, arg5: PTOKEN_PRIVILEGES, arg6: PDWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn BroadcastSystemMessageW( flags: DWORD, lpInfo: LPDWORD, Msg: UINT, wParam: WPARAM, lParam: LPARAM, ) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateBitmap( cx: INT, cy: INT, cPlanes: UINT, cBitsPerPel: UINT, pvBits: *const ::core::ffi::c_void, ) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP; // aclui-rust/src/types_amd64.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateIconIndirect(arg1: PICONINFO) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn DebugBreak(); // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumResourceNamesW( arg1: HMODULE, arg2: LPCWSTR, arg3: ENUMRESNAMEPROCW, arg4: LONG_PTR, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ExitWindowsEx(arg1: UINT, arg2: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardLayout(arg1: DWORD) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn GetKeyboardLayoutList(nBuff: ::core::ffi::c_int, lpList: *mut HKL) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemDefaultLCID() -> LCID; // desk-rust/src/types_amd64.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_Remove(himl: HIMAGELIST, i: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn LoadKeyboardLayoutW(arg1: LPCWSTR, arg2: UINT) -> HKL; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryExW(arg1: LPCWSTR, arg2: HANDLE, arg3: DWORD) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn LookupPrivilegeValueW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: PLUID) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn OpenProcessToken(arg1: HANDLE, arg2: DWORD, arg3: PHANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PatBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn PropertySheetW(arg1: LPCPROPSHEETHEADERW) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyW(hKey: HKEY, lpSubKey: LPCWSTR, phkResult: PHKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetDCBrushColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    pub fn SetRect( arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetupCloseInfFile(arg1: HINF); // desk-rust/src/types_amd64.rs
    pub fn SetupFindFirstLineW(arg1: HINF, arg2: PCWSTR, arg3: PCWSTR, arg4: PINFCONTEXT) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupGetFieldCount(arg1: PINFCONTEXT) -> DWORD; // desk-rust/src/types_amd64.rs
    pub fn SetupGetStringFieldW( Context: PINFCONTEXT, FieldIndex: DWORD, ReturnBuffer: PWSTR, ReturnBufferSize: DWORD, RequiredSize: PDWORD, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupOpenInfFileW( FileName: PCWSTR, InfClass: PCWSTR, InfStyle: DWORD, ErrorLine: PUINT, ) -> HINF; // desk-rust/src/types_amd64.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnloadKeyboardLayout(arg1: HKL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::input::add_dialog::AddDialogProc;
pub use crate::input::advanced_settings_page::AdvancedSettingsPageProc;
pub use crate::input::key_sequence_dialog::ChangeKeySeqDialogProc;
pub use crate::input::edit_dialog::EditDialogProc;
pub use crate::input::settings_page::EnableProcessPrivileges;
pub use crate::input::key_settings_dialog::KeySettingsDialogProc;
pub use crate::input::layout_list::LayoutList_Create;
pub use crate::input::layout_list::LayoutList_Destroy;
pub use crate::input::layout_list::LayoutList_GetByHkl;
pub use crate::input::layout_list::LayoutList_GetFirst;
pub use crate::input::locale_list::LocaleList_GetByHkl;
pub use crate::input::settings_page::SettingsPageProc;
pub use crate::input::input::g_bRebootNeeded;
pub use crate::input::input::hApplet;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const TVN_ITEMEXPANDING: u32 = 0xFFFFFE3A; // clang, settings_page.c
pub const TVN_SELCHANGED: u32 = 0xFFFFFE3D; // clang, settings_page.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, settings_page.c
pub type HIMAGELIST = *mut _IMAGELIST;
// HTREEITEM: c2rust, settings_page.c
pub type HTREEITEM = *mut _TREEITEM;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
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
#[macro_export] macro_rules! ComboBox_AddString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((323) as _), ((0) as _), (((($p1) as LPCTSTR)) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetCount { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((326) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((327) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetItemData { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((336) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as UINT_PTR)) as LRESULT) }; }
#[macro_export] macro_rules! ComboBox_SelectString { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((333) as _), (((($p1) as c_int)) as WPARAM), (((($p2) as LPCTSTR)) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_SetCurSel { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((334) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_SetItemData { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((337) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! DialogBoxW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! IS_IME_HKL { ($p0:expr $(,)?) => { ((((($p0) as ULONG_PTR) & 0xF0000000) == 0xE0000000) as c_int) }; }
#[macro_export] macro_rules! IS_IME_KLID { ($p0:expr $(,)?) => { ((((($p0) as ULONG) & 0xF0000000) == 0xE0000000) as c_int) }; }
#[macro_export] macro_rules! IS_SPECIAL_HKL { ($p0:expr $(,)?) => { ((((($p0) as ULONG_PTR) & 0xF0000000) == 0xF0000000) as c_int) }; }
#[macro_export] macro_rules! IS_SUBST_KLID { ($p0:expr $(,)?) => { ((((($p0) as ULONG) & 0xF0000000) == 0xD0000000) as c_int) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! ImageList_RemoveAll { ($p0:expr $(,)?) => { ImageList_Remove((($p0) as _), (((-(1 as c_int))) as _)) }; }
#[macro_export] macro_rules! ListView_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 9)) as _), ((0) as _), ((0) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 77)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_SetExtendedListViewStyle { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), ((0) as _), (($p1) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_SetItemText { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.iSubItem = (($p2) as _); }; { _ms_lvi.pszText = (($p3) as _); }; SendMessageW((($p0) as _), (((0x1000 + 116)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! PropSheet_Changed { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), (((1024 + 104)) as _), (($p1) as WPARAM), ((0) as _)) }; }
#[macro_export] macro_rules! SPECIALIDFROMHKL { ($p0:expr $(,)?) => { ((((((((((($p0) as DWORD_PTR) >> 16) & 0xffff)) as WORD)) as c_int) & 0x0FFF)) as WORD) }; }
#[macro_export] macro_rules! TreeView_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 1)) as _), ((0) as _), (((((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_Expand { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 2)) as _), (($p2) as WPARAM), (((($p1) as HTREEITEM)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetChild { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x4) as WPARAM), (((($p1) as HTREEITEM)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetImageList { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 8)) as _), (($p1) as _), ((0) as _))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 62)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetNextSibling { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x1) as WPARAM), (((($p1) as HTREEITEM)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetParent { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x3) as WPARAM), (((($p1) as HTREEITEM)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetRoot { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x0) as WPARAM), ((((core::ptr::null_mut::<c_void>()) as HTREEITEM)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_GetSelection { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x9) as WPARAM), ((((core::ptr::null_mut::<c_void>()) as HTREEITEM)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 50)) as _), ((0) as _), (((($p1) as LPTVINSERTSTRUCTW)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 9)) as _), (($p2) as _), (((($p1) as HIMAGELIST)) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeView_SetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 63)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub type LV_ITEM = LVITEMW;
pub type NM_TREEVIEW = NMTREEVIEWW;
pub const RT_GROUP_ICON: LPWSTR = 0xeusize as LPWSTR;
pub const SE_SHUTDOWN_NAME: *const u16 = (&[0x0053, 0x0065, 0x0053, 0x0068, 0x0075, 0x0074, 0x0064, 0x006f, 0x0077, 0x006e, 0x0050, 0x0072, 0x0069, 0x0076, 0x0069, 0x006c, 0x0065, 0x0067, 0x0065, 0x0000u16]).as_ptr(); // L"SeShutdownPrivilege"
pub const S_OK: HRESULT = ((0) as HRESULT);
pub const TVI_LAST: HTREEITEM = (((((-(0xfffe as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_ROOT: HTREEITEM = (((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM);
pub type TV_ITEM = TVITEMW;
pub use crate::{ComboBox_AddString, ComboBox_GetCount, ComboBox_GetCurSel, ComboBox_GetItemData, ComboBox_SelectString, ComboBox_SetCurSel, ComboBox_SetItemData, DialogBoxW, IS_IME_HKL, IS_IME_KLID, IS_SPECIAL_HKL, IS_SUBST_KLID, ImageList_AddIcon, ImageList_RemoveAll, ListView_DeleteAllItems, ListView_InsertColumn, ListView_InsertItem, ListView_SetExtendedListViewStyle, ListView_SetItemText, PropSheet_Changed, SPECIALIDFROMHKL, TreeView_DeleteAllItems, TreeView_Expand, TreeView_GetChild, TreeView_GetImageList, TreeView_GetItem, TreeView_GetNextSibling, TreeView_GetParent, TreeView_GetRoot, TreeView_GetSelection, TreeView_InsertItem, TreeView_SetImageList, TreeView_SetItem};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/cpl/input/input.h:79: header-inline `DWORDfromString`
#[allow(dead_code)]
#[inline]
pub unsafe fn DWORDfromString(pszString: *const WCHAR) -> DWORD {
    let mut pszEnd: *mut WCHAR = core::ptr::null_mut();
    wcstoul(pszString, &mut pszEnd, 16) as DWORD
}
// ==== end port macros ====
