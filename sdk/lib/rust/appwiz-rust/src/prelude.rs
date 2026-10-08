//! appwiz-rust prelude — the single import surface the translated modules use.
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
// DPRINT1 PRINTS in this DBG=1 tree: it comes from win32macros-rust with the other macros.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= Hand-written: header names the island cannot carry =========================
// wine/debug.h `enum __wine_debug_class` (ERR/WARN/TRACE/FIXME): an int in every dialect.
pub type __wine_debug_class = c_int;
// guiddef.h `#define REFIID const IID *` (C, not C++).
pub type REFIID = *const IID;
// fcntl.h: addons.c opens the cached installer read-only.
pub const O_RDONLY: c_int = 0;

// The GUIDs the C takes from `uuid` (target_link_libraries(appwiz uuid wine)), which the
// Rust half links through the same CMake target.
extern "C" {
    pub static IID_IUnknown: IID;
    pub static IID_IPersistFile: IID;
    pub static CLSID_InternetShortcut: CLSID;
    pub static IID_IUniformResourceLocatorW: IID;
}

// wine/unicode.h `#define strcmpW(s1,s2) wcscmp((s1),(s2))`.
extern "C" {
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> c_int;
}
#[inline(always)]
pub unsafe fn strcmpW(s1: *const u16, s2: *const u16) -> c_int {
    wcscmp(s1, s2)
}

// COBJMACROS (objbase.h / urlmon.h): a call through the object's own vtable slot.
#[inline(always)]
pub unsafe fn IUnknown_QueryInterface<T>(p: *mut T, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IUnknownVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IUnknown, riid, ppv)
}
#[inline(always)]
pub unsafe fn IUnknown_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IUnknownVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IUnknown)
}
#[inline(always)]
pub unsafe fn IBinding_AddRef(p: *mut IBinding) -> ULONG {
    ((*(*p).lpVtbl).AddRef.unwrap_unchecked())(p)
}
#[inline(always)]
pub unsafe fn IBinding_Release(p: *mut IBinding) -> ULONG {
    ((*(*p).lpVtbl).Release.unwrap_unchecked())(p)
}
#[inline(always)]
pub unsafe fn IBinding_Abort(p: *mut IBinding) -> HRESULT {
    ((*(*p).lpVtbl).Abort.unwrap_unchecked())(p)
}

// winuser.h: on i386 `GetWindowLongPtrW`/`SetWindowLongPtrW` are MACROS for the 32-bit calls,
// and user32 exports no symbol by the Ptr name there.
#[cfg(target_arch = "x86")]
extern "system" {
    #[link_name = "GetWindowLongW"]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    #[link_name = "SetWindowLongW"]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}

// wine/debug.h, as appwiz.h's `WINE_DEFAULT_DEBUG_CHANNEL(appwiz)` expands it: a per-module
// pointer to the module's channel, and `debugstr_a` as the header's static inline over
// libwine's `wine_dbgstr_an` (appwiz links `wine`). Neither is exported by anything, so the
// translated TRACE/ERR sites need them defined here.
#[no_mangle]
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut crate::appwiz::appwiz_h::__wine_dbch_appwiz };
extern "C" {
    pub fn wine_dbgstr_an(s: *const c_char, n: c_int) -> *const c_char;
}
#[inline(always)]
pub unsafe fn debugstr_a(s: *const c_char) -> *const c_char {
    wine_dbgstr_an(s, -1)
}

// prsht.h `#define PropSheet_SetWizButtons(hDlg,dwFlags) PostMessage(hDlg,PSM_SETWIZBUTTONS,0,(LPARAM)dwFlags)`.
#[macro_export]
macro_rules! PropSheet_SetWizButtons {
    ($hdlg:expr, $flags:expr $(,)?) => {
        $crate::prelude::PostMessageW($hdlg, PSM_SETWIZBUTTONS, 0, $flags as LPARAM)
    };
}
pub use crate::PropSheet_SetWizButtons;
// winuser.h `#define DialogBoxW(i,t,p,f) DialogBoxParamW(i,t,p,f,0)`; DialogBox is the UNICODE spelling.
#[macro_export]
macro_rules! DialogBoxW {
    ($inst:expr, $tmpl:expr, $parent:expr, $proc:expr $(,)?) => {
        $crate::prelude::DialogBoxParamW($inst, $tmpl, $parent, $proc, 0)
    };
}
pub use crate::DialogBoxW;
#[macro_export]
macro_rules! DialogBox {
    ($inst:expr, $tmpl:expr, $parent:expr, $proc:expr $(,)?) => {
        $crate::DialogBoxW!($inst, $tmpl, $parent, $proc)
    };
}
pub use crate::DialogBox;
extern "system" {
    pub fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: DLGPROC,
        dwInitParam: LPARAM,
    ) -> INT_PTR;
}

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ADDON_GECKO: addon_t = addon_t_ADDON_GECKO; // island enum
pub const ADDON_MONO: addon_t = addon_t_ADDON_MONO; // island enum
pub const CLSCTX_ALL: DWORD = 0x17; // shell32-rust/src/shell32/cdefaultcontextmenu_support.rs
pub const COINIT_MULTITHREADED: DWORD = 0; // ole32-rust/src/marshal/types.rs
pub const E_ABORT: HRESULT = 0x80004004u32 as HRESULT; // ole32-rust/src/hresults.rs
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT; // ole32-rust/src/hresults.rs
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // desk-rust/src/prelude.rs
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as HKEY; // setuplib-rust/src/prelude.rs
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY; // dhcpcsvc-rust/src/prelude.rs
pub const STGM_READ: u32 = 0; // desk-rust/src/types_amd64.rs
pub const S_OK: HRESULT = 0; // aclui-rust/src/prelude.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // dnsapi-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // dnsapi-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // dnsapi-rust/src/prelude.rs
extern "C" {
    pub fn _close(fh: c_int) -> c_int; // ucrt-rust/src/conio/popen.rs
    pub fn _open(path: *const i8, flags: i32, ...) -> i32; // msvcrt-crt-rust/src/msvcrt/file.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const c_char, func: *const c_char, line: c_int, msg: *const c_char, ... ) -> c_int; // iphlpapi-rust/src/prelude.rs
    pub fn sprintf(buf: *mut i8, fmt: *const i8, ...) -> i32; // freeldr-rust/src/hwdisk.rs
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int; // browser-rust/src/prelude.rs
    pub fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8; // cabinet-rust/src/cabinet/cabinet_main.rs
    pub fn strlen(s: *const i8) -> usize; // advapi32-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
}
extern "system" {
    pub fn A_SHAFinal(Context: *mut SHA_CTX, Result: *mut u32); // cryptlib-rust/src/sha1.rs
    pub fn A_SHAInit(Context: *mut SHA_CTX); // cryptlib-rust/src/sha1.rs
    pub fn A_SHAUpdate(Context: *mut SHA_CTX, Buffer: *const u8, BufferSize: u32,); // cryptlib-rust/src/sha1.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CoInitializeEx(lpReserved: LPVOID, dwCoInit: DWORD) -> HRESULT; // aclui-rust/src/types_amd64.rs
    pub fn CoTaskMemFree(memory: LPVOID); // menuband-rust/src/enum_merged_folder.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_amd64.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ExpandEnvironmentStringsA(arg1: LPCSTR, arg2: LPSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItemTextW( hDlg: HWND, nIDDlgItem: ::core::ffi::c_int, lpString: LPWSTR, cchMax: ::core::ffi::c_int, ) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn InitCommonControls(); // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn PathAddBackslashW(arg1: LPWSTR) -> LPWSTR; // desk-rust/src/types_amd64.rs
    pub fn PathAddExtensionW(arg1: LPWSTR, arg2: LPCWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PathAppendW(arg1: LPWSTR, arg2: LPCWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PathCleanupSpec(arg1: LPCWSTR, arg2: LPWSTR) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn PathFileExistsW(pszPath: LPCWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR; // desk-rust/src/types_amd64.rs
    pub fn PathFindOnPathExW( lpszFile: LPWSTR, lppszOtherDirs: *mut LPCWSTR, dwWhich: DWORD, ) -> BOOL; // shell32-rust/src/island_amd64.rs
    pub fn PathGetArgsW(pszPath: LPCWSTR) -> LPWSTR; // desk-rust/src/types_amd64.rs
    pub fn PathIsDirectoryW(arg1: LPCWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PathIsURLW(arg1: LPCWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PathRemoveArgsW(arg1: LPWSTR); // desk-rust/src/types_amd64.rs
    pub fn PathRemoveExtensionW(arg1: LPWSTR); // desk-rust/src/types_amd64.rs
    pub fn PathRemoveFileSpecW(arg1: LPWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PathYetAnotherMakeUniqueName( arg1: LPWSTR, arg2: LPCWSTR, arg3: LPCWSTR, arg4: LPCWSTR, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegGetValueA( hkey: HKEY, lpSubKey: LPCSTR, lpValue: LPCSTR, dwFlags: DWORD, pdwType: LPDWORD, pvData: PVOID, pcbData: LPDWORD, ) -> LSTATUS; // shell32-rust/src/island_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn SHAutoComplete(arg1: HWND, arg2: DWORD) -> HRESULT; // desk-rust/src/types_amd64.rs
    pub fn SHBrowseForFolderW(lpbi: LPBROWSEINFOW) -> PIDLIST_ABSOLUTE; // desk-rust/src/types_amd64.rs
    pub fn SHChangeNotify(event: LONG, flags: UINT, item1: LPCVOID, item2: LPCVOID); // shell32-rust/src/dialogs/filetypes.rs
    pub fn SHGetFileInfoW( pszPath: LPCWSTR, dwFileAttributes: DWORD, psfi: *mut SHFILEINFOW, cbFileInfo: UINT, uFlags: UINT, ) -> DWORD_PTR; // desk-rust/src/types_amd64.rs
    pub fn SHGetPathFromIDListW(arg1: PCIDLIST_ABSOLUTE, arg2: LPWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn ShellExecuteW( hwnd: HWND, lpOperation: LPCWSTR, lpFile: LPCWSTR, lpParameters: LPCWSTR, lpDirectory: LPCWSTR, nShowCmd: INT, ) -> HINSTANCE; // desk-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StrTrimW(arg1: LPWSTR, arg2: LPCWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// SHA_CTX: c2rust, addons.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SHA_CTX {
    pub Unknown: [ULONG; 6],
    pub State: [ULONG; 5],
    pub Count: [ULONG; 2],
    pub Buffer: [UCHAR; 64],
}
// install_res: c2rust, addons.c
pub type install_res = ::core::ffi::c_uint;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CloseHandle(_: HANDLE) -> BOOL; // addons.c
    pub fn CreateFileW(
        _: LPCWSTR,
        _: DWORD,
        _: DWORD,
        _: LPSECURITY_ATTRIBUTES,
        _: DWORD,
        _: DWORD,
        _: HANDLE,
    ) -> HANDLE; // addons.c
    pub fn CreatePropertySheetPageW(_: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE; // createlink.c
    pub fn CreateThread(
        _: LPSECURITY_ATTRIBUTES,
        _: DWORD,
        _: LPTHREAD_START_ROUTINE,
        _: PVOID,
        _: DWORD,
        _: PDWORD,
    ) -> HANDLE; // addons.c
    pub fn DeleteCriticalSection(_: PCRITICAL_SECTION); // addons.c
    pub fn DeleteFileW(_: LPCWSTR) -> BOOL; // addons.c
    pub fn EnterCriticalSection(_: LPCRITICAL_SECTION); // addons.c
    pub fn GetFullPathNameW(_: LPCWSTR, _: DWORD, _: LPWSTR, _: *mut LPWSTR) -> DWORD; // createlink.c
    pub fn GetModuleHandleW(_: LPCWSTR) -> HMODULE; // createlink.c
    pub fn GetProcessHeap() -> HANDLE; // addons.c
    pub fn GetTempFileNameW(_: LPCWSTR, _: LPCWSTR, _: UINT, _: LPWSTR) -> UINT; // addons.c
    pub fn HeapAlloc(_: HANDLE, _: DWORD, _: SIZE_T) -> PVOID; // addons.c
    pub fn HeapFree(_: HANDLE, _: DWORD, _: PVOID) -> BOOL; // addons.c
    pub fn HeapReAlloc(_: HANDLE, _: DWORD, _: PVOID, _: SIZE_T) -> PVOID; // addons.c
    pub fn InitializeCriticalSection(_: LPCRITICAL_SECTION); // addons.c
    pub fn LeaveCriticalSection(_: LPCRITICAL_SECTION); // addons.c
    pub fn MsiInstallProductW(_: LPCWSTR, _: LPCWSTR) -> UINT; // addons.c
    pub fn PathFindFileNameW(_: LPCWSTR) -> LPWSTR; // createlink.c
    pub fn PathIsRelativeW(_: LPCWSTR) -> BOOL; // createlink.c
    pub fn PropertySheetW(_: LPCPROPSHEETHEADERW) -> INT_PTR; // createlink.c
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // createlink.c
    pub fn URLDownloadToFileW(
        _: LPUNKNOWN,
        _: LPCWSTR,
        _: LPCWSTR,
        _: DWORD,
        _: LPBINDSTATUSCALLBACK,
    ) -> HRESULT; // addons.c
}
pub use crate::prelude::CreatePropertySheetPageW as CreatePropertySheetPage; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::PropertySheetW as PropertySheet; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
// ==== end port externs ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/cpl/appwiz/appwiz.h:66: header-inline `heap_alloc`
#[allow(dead_code)]
#[inline]
pub unsafe fn heap_alloc(len: size_t) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), 0, len as SIZE_T)
}
// dll/cpl/appwiz/appwiz.h:76: header-inline `heap_free`
#[allow(dead_code)]
#[inline]
pub unsafe fn heap_free(mem: *mut core::ffi::c_void) -> BOOL {
    HeapFree(GetProcessHeap(), 0, mem)
}
// dll/cpl/appwiz/appwiz.h:71: header-inline `heap_realloc`
#[allow(dead_code)]
#[inline]
pub unsafe fn heap_realloc(mem: *mut core::ffi::c_void, len: size_t) -> *mut core::ffi::c_void {
    HeapReAlloc(GetProcessHeap(), 0, mem, len as SIZE_T)
}
// ==== end port macros ====
