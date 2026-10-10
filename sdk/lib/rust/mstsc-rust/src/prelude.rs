//! mstsc-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedDecrement, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const HYBRID_REQUIRED_BY_SERVER: RDP_NEG_FAILURE_CODE = RDP_NEG_FAILURE_CODE_HYBRID_REQUIRED_BY_SERVER; // island enum
pub const INCONSISTENT_FLAGS: RDP_NEG_FAILURE_CODE = RDP_NEG_FAILURE_CODE_INCONSISTENT_FLAGS; // island enum
pub const RDP_DATA_PDU_AUTORECONNECT_STATUS: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_AUTORECONNECT_STATUS; // island enum
pub const RDP_DATA_PDU_BELL: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_BELL; // island enum
pub const RDP_DATA_PDU_CONTROL: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_CONTROL; // island enum
pub const RDP_DATA_PDU_DISCONNECT: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_DISCONNECT; // island enum
pub const RDP_DATA_PDU_LOGON: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_LOGON; // island enum
pub const RDP_DATA_PDU_POINTER: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_POINTER; // island enum
pub const RDP_DATA_PDU_SYNCHRONISE: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_SYNCHRONISE; // island enum
pub const RDP_DATA_PDU_UPDATE: RDP_DATA_PDU_TYPE = RDP_DATA_PDU_TYPE_RDP_DATA_PDU_UPDATE; // island enum
pub const RDP_NULL_POINTER: RDP_SYSTEM_POINTER_TYPE = RDP_SYSTEM_POINTER_TYPE_RDP_NULL_POINTER; // island enum
pub const RDP_ORDER_BMPCACHE2: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_BMPCACHE2; // island enum
pub const RDP_ORDER_BMPCACHE: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_BMPCACHE; // island enum
pub const RDP_ORDER_BRUSHCACHE: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_BRUSHCACHE; // island enum
pub const RDP_ORDER_COLCACHE: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_COLCACHE; // island enum
pub const RDP_ORDER_DESKSAVE: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_DESKSAVE; // island enum
pub const RDP_ORDER_DESTBLT: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_DESTBLT; // island enum
pub const RDP_ORDER_ELLIPSE2: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_ELLIPSE2; // island enum
pub const RDP_ORDER_ELLIPSE: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_ELLIPSE; // island enum
pub const RDP_ORDER_FONTCACHE: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_FONTCACHE; // island enum
pub const RDP_ORDER_LINE: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_LINE; // island enum
pub const RDP_ORDER_MEMBLT: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_MEMBLT; // island enum
pub const RDP_ORDER_PATBLT: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_PATBLT; // island enum
pub const RDP_ORDER_POLYGON2: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_POLYGON2; // island enum
pub const RDP_ORDER_POLYGON: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_POLYGON; // island enum
pub const RDP_ORDER_POLYLINE: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_POLYLINE; // island enum
pub const RDP_ORDER_RAW_BMPCACHE2: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_RAW_BMPCACHE2; // island enum
pub const RDP_ORDER_RAW_BMPCACHE: RDP_SECONDARY_ORDER_TYPE = RDP_SECONDARY_ORDER_TYPE_RDP_ORDER_RAW_BMPCACHE; // island enum
pub const RDP_ORDER_RECT: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_RECT; // island enum
pub const RDP_ORDER_SCREENBLT: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_SCREENBLT; // island enum
pub const RDP_ORDER_TEXT2: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_TEXT2; // island enum
pub const RDP_ORDER_TRIBLT: RDP_ORDER_TYPE = RDP_ORDER_TYPE_RDP_ORDER_TRIBLT; // island enum
pub const RDP_PDU_DATA: RDP_PDU_TYPE = RDP_PDU_TYPE_RDP_PDU_DATA; // island enum
pub const RDP_PDU_DEACTIVATE: RDP_PDU_TYPE = RDP_PDU_TYPE_RDP_PDU_DEACTIVATE; // island enum
pub const RDP_PDU_DEMAND_ACTIVE: RDP_PDU_TYPE = RDP_PDU_TYPE_RDP_PDU_DEMAND_ACTIVE; // island enum
pub const RDP_PDU_ENHANCED_REDIRECT: RDP_PDU_TYPE = RDP_PDU_TYPE_RDP_PDU_ENHANCED_REDIRECT; // island enum
pub const RDP_PDU_REDIRECT: RDP_PDU_TYPE = RDP_PDU_TYPE_RDP_PDU_REDIRECT; // island enum
pub const RDP_POINTER_CACHED: RDP_POINTER_PDU_TYPE = RDP_POINTER_PDU_TYPE_RDP_POINTER_CACHED; // island enum
pub const RDP_POINTER_COLOR: RDP_POINTER_PDU_TYPE = RDP_POINTER_PDU_TYPE_RDP_POINTER_COLOR; // island enum
pub const RDP_POINTER_MOVE: RDP_POINTER_PDU_TYPE = RDP_POINTER_PDU_TYPE_RDP_POINTER_MOVE; // island enum
pub const RDP_POINTER_NEW: RDP_POINTER_PDU_TYPE = RDP_POINTER_PDU_TYPE_RDP_POINTER_NEW; // island enum
pub const RDP_POINTER_SYSTEM: RDP_POINTER_PDU_TYPE = RDP_POINTER_PDU_TYPE_RDP_POINTER_SYSTEM; // island enum
pub const RDP_UPDATE_BITMAP: RDP_UPDATE_PDU_TYPE = RDP_UPDATE_PDU_TYPE_RDP_UPDATE_BITMAP; // island enum
pub const RDP_UPDATE_ORDERS: RDP_UPDATE_PDU_TYPE = RDP_UPDATE_PDU_TYPE_RDP_UPDATE_ORDERS; // island enum
pub const RDP_UPDATE_PALETTE: RDP_UPDATE_PDU_TYPE = RDP_UPDATE_PDU_TYPE_RDP_UPDATE_PALETTE; // island enum
pub const RDP_UPDATE_SYNCHRONIZE: RDP_UPDATE_PDU_TYPE = RDP_UPDATE_PDU_TYPE_RDP_UPDATE_SYNCHRONIZE; // island enum
pub const SSL_CERT_NOT_ON_SERVER: RDP_NEG_FAILURE_CODE = RDP_NEG_FAILURE_CODE_SSL_CERT_NOT_ON_SERVER; // island enum
pub const SSL_NOT_ALLOWED_BY_SERVER: RDP_NEG_FAILURE_CODE = RDP_NEG_FAILURE_CODE_SSL_NOT_ALLOWED_BY_SERVER; // island enum
pub const SSL_REQUIRED_BY_SERVER: RDP_NEG_FAILURE_CODE = RDP_NEG_FAILURE_CODE_SSL_REQUIRED_BY_SERVER; // island enum
pub const SSL_WITH_USER_AUTH_REQUIRED_BY_SERVER: RDP_NEG_FAILURE_CODE = RDP_NEG_FAILURE_CODE_SSL_WITH_USER_AUTH_REQUIRED_BY_SERVER; // island enum
extern "C" {
    pub fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32; // kernel32-rust/src/client/dllmain.rs
    pub fn _wtoi(s: *const u16) -> i32; // userinit-rust/src/preamble.rs
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32; // acpi-rust/src/buspdo.rs
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstok(s: *mut u16, delim: *const u16) -> *mut u16; // desk-rust/src/prelude.rs
}
extern "system" {
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn BringWindowToTop(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CoTaskMemFree(memory: LPVOID); // menuband-rust/src/enum_merged_folder.rs
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_amd64.rs
    pub fn CreateICW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: LPCWSTR, arg4: *const DEVMODEW) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn DPtoLP(hdc: HDC, lppt: LPPOINT, c: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteDC(arg1: HDC) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn DrawTextW( hdc: HDC, lpchText: LPCWSTR, cchText: ::core::ffi::c_int, lprc: LPRECT, format: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumDisplayDevicesW( arg1: LPCWSTR, arg2: DWORD, arg3: PDISPLAY_DEVICEW, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumDisplaySettingsExW( arg1: LPCWSTR, arg2: DWORD, arg3: LPDEVMODEW, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDeviceCaps(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetOpenFileNameW(arg1: LPOPENFILENAMEW) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetSaveFileNameW(arg1: LPOPENFILENAMEW) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn SHGetFolderLocation( hwndOwner: HWND, nFolder: ::core::ffi::c_int, hToken: HANDLE, dwReserved: DWORD, ppidl: *mut PIDLIST_ABSOLUTE, ) -> HRESULT; // desk-rust/src/types_amd64.rs
    pub fn SHGetPathFromIDListW(arg1: PCIDLIST_ABSOLUTE, arg2: LPWSTR) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SelectObject(arg1: HDC, arg2: HGDIOBJ) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetBkMode(arg1: HDC, arg2: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetTextColor(arg1: HDC, arg2: COLORREF) -> COLORREF; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StretchBlt( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HDC, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: ::core::ffi::c_int, arg11: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// LPITEMIDLIST: c2rust, settings.c
pub type LPITEMIDLIST = *mut ITEMIDLIST;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn GetDlgItemTextW(
        hDlg: HWND,
        nIDDlgItem: ::core::ffi::c_int,
        lpString: LPWSTR,
        cchMax: ::core::ffi::c_int,
    ) -> UINT; // settings.c
}
pub use crate::prelude::GetDlgItemTextW as GetDlgItemText; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
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
#[macro_export] macro_rules! TabCtrl_GetCurSel { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 11)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! TabCtrl_InsertItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 62)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
pub const ENUM_CURRENT_SETTINGS: DWORD = (((-(1 as c_int))) as DWORD);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub type TCITEM = TCITEMW;
pub use crate::{TabCtrl_GetCurSel, TabCtrl_InsertItem};
// ==== end SDK header macros ====
