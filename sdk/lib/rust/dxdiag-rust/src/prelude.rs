//! dxdiag-rust prelude — the single import surface the translated modules use.
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
pub const SYS_PRODUCT: ULONG = 0x05; // i8042prt-rust/src/hwhacks.rs
pub const SYS_VENDOR: ULONG = 0x04; // i8042prt-rust/src/hwhacks.rs
pub const UINT_MAX: u32 = 4294967295; // comctl32-rust/src/prelude.rs
extern "C" {
    pub fn FreeSMBiosData(Buffer: PVOID); // sysdm-rust/src/types_amd64.rs
    pub fn GetSMBiosStringW(DmiString: PCSTR, pBuf: PWSTR, cchBuf: DWORD, bTrim: BOOL) -> SIZE_T; // sysdm-rust/src/types_amd64.rs
    pub fn LoadSMBiosData(Strings: *mut PCHAR) -> PVOID; // sysdm-rust/src/types_amd64.rs
    pub fn TrimDmiStringW(pStr: PWSTR); // sysdm-rust/src/types_amd64.rs
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // rtl-actctx/src/shim.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int; // desk-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR; // recyclebin-rust/src/shellrecyclebin/RecycleBin5.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
}
extern "system" {
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateSolidBrush(arg1: COLORREF) -> HBRUSH; // aclui-rust/src/types_amd64.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumDisplayDevicesW( arg1: LPCWSTR, arg2: DWORD, arg3: PDISPLAY_DEVICEW, arg4: DWORD, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumDisplaySettingsW(arg1: LPCWSTR, arg2: DWORD, arg3: PDEVMODEW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FillRect(arg1: HDC, arg2: LPCRECT, arg3: HBRUSH) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetComputerNameW(lpBuffer: LPWSTR, nSize: LPDWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetDateFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFileTime(arg1: HANDLE, arg2: LPFILETIME, arg3: LPFILETIME, arg4: LPFILETIME) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetFileVersionInfoSizeW(arg1: LPCWSTR, arg2: PDWORD) -> DWORD; // recyclebin-rust/src/types_amd64.rs
    pub fn GetFileVersionInfoW(arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: PVOID) -> BOOL; // recyclebin-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetLocaleInfoW( Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_amd64.rs
    pub fn GetNativeSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetStockObject(arg1: ::core::ffi::c_int) -> HGDIOBJ; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemInfo(arg1: LPSYSTEM_INFO); // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn GetTimeFormatW( arg1: LCID, arg2: DWORD, arg3: *const SYSTEMTIME, arg4: LPCWSTR, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_amd64.rs
    pub fn GetWindow(arg1: HWND, arg2: UINT) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalMemoryStatusEx(arg1: LPMEMORYSTATUSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn InitCommonControlsEx(arg1: *const INITCOMMONCONTROLSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryA(arg1: LPCSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn OffsetRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemInt(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetupCloseFileQueue(arg1: HSPFILEQ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiBuildDriverInfoList(arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: DWORD) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiCallClassInstaller( arg1: DI_FUNCTION, arg2: HDEVINFO, arg3: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiEnumDeviceInfo(arg1: HDEVINFO, arg2: DWORD, arg3: PSP_DEVINFO_DATA) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiEnumDriverInfoW( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: DWORD, arg4: DWORD, arg5: PSP_DRVINFO_DATA_W, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetClassDevsW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, ) -> HDEVINFO; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetDeviceInstallParamsW( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: PSP_DEVINSTALL_PARAMS_W, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetDeviceRegistryPropertyW( DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA, Property: DWORD, PropertyRegDataType: PDWORD, PropertyBuffer: PBYTE, PropertyBufferSize: DWORD, RequiredSize: PDWORD, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiSetDeviceInstallParamsW( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: PSP_DEVINSTALL_PARAMS_W, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiSetSelectedDriverW( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: PSP_DRVINFO_DATA_W, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupOpenFileQueue() -> HSPFILEQ; // desk-rust/src/types_amd64.rs
    pub fn SetupScanFileQueueW( arg1: HSPFILEQ, arg2: DWORD, arg3: HWND, arg4: PSP_FILE_CALLBACK_W, arg5: PVOID, arg6: PDWORD, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SystemTimeToTzSpecificLocalTime( arg1: *const TIME_ZONE_INFORMATION, arg2: *const SYSTEMTIME, arg3: LPSYSTEMTIME, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn VerQueryValueW(arg1: LPCVOID, arg2: LPCWSTR, arg3: *mut LPVOID, arg4: PUINT) -> BOOL; // recyclebin-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::dxdiag::dxdiag::hInst;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const BIOS_DATE: i32 = 3; // clang, system.c
pub const BIOS_VENDOR: i32 = 1; // clang, system.c
pub const D3D_OK: HRESULT = 0; // clang, display.c
pub const DIPROP_VIDPID: u32 = 0x18; // clang, input.c
pub const DI_FLAGSEX_INSTALLEDDRIVER: i32 = 0x4000000; // clang, display.c
pub const DI_NOTATTACHED: HRESULT = 1; // clang, input.c
pub const DI_OK: HRESULT = 0; // clang, input.c
pub const IDI_APPLICATION: LPWSTR = 0x7F00usize as LPWSTR; // clang, ddtest.c
pub const ID_STRINGS_MAX: i32 = 0xF; // clang, system.c
pub const LVM_INSERTITEM: i32 = 0x104D; // clang, input.c
pub const LVM_SETITEM: i32 = 0x104C; // clang, input.c
pub const TCM_INSERTITEM: i32 = 0x133E; // clang, dxdiag.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
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
    ) -> HWND; // ddtest.c
    pub fn DirectDrawCreate(
        lpGUID: *mut GUID,
        lplpDD: *mut LPDIRECTDRAW,
        pUnkOuter: *mut IUnknown,
    ) -> HRESULT; // ddtest.c
    pub fn DirectInput8Create(
        _: HINSTANCE,
        _: DWORD,
        _: *const IID,
        _: *mut LPVOID,
        _: LPUNKNOWN,
    ) -> HRESULT; // input.c
    pub fn DirectSoundCreate8(
        lpGUID: LPCGUID,
        ppDS8: *mut LPDIRECTSOUND8,
        pUnkOuter: LPUNKNOWN,
    ) -> HRESULT; // sound.c
    pub fn DirectSoundEnumerateW(_: LPDSENUMCALLBACKW, _: LPVOID) -> HRESULT; // sound.c
    pub fn DispatchMessageW(_: *const MSG) -> LRESULT; // ddtest.c
    pub fn GetVersionExW(_: LPOSVERSIONINFOW) -> BOOL; // system.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // display.c
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // ddtest.c
    pub fn MessageBoxW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
    ) -> ::core::ffi::c_int; // ddtest.c
    pub fn PeekMessageW(_: LPMSG, _: HWND, _: UINT, _: UINT, _: UINT) -> BOOL; // ddtest.c
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS; // network.c
    pub fn RegisterClassExW(_: *const WNDCLASSEXW) -> ATOM; // ddtest.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // display.c
    pub fn SetupDiGetDeviceInstanceIdW(
        DeviceInfoSet: HDEVINFO,
        DeviceInfoData: PSP_DEVINFO_DATA,
        DeviceInstanceId: PWSTR,
        DeviceInstanceIdSize: DWORD,
        RequiredSize: PDWORD,
    ) -> BOOL; // sound.c
    pub fn UnregisterClassW(_: LPCWSTR, _: HINSTANCE) -> BOOL; // ddtest.c
    pub fn waveOutGetDevCapsW(_: UINT_PTR, _: LPWAVEOUTCAPSW, _: UINT) -> MMRESULT; // sound.c
    pub fn waveOutGetNumDevs() -> UINT; // sound.c
}
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::GetDateFormatW as GetDateFormat; // sdk alias
pub use crate::prelude::GetLocaleInfoW as GetLocaleInfo; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetVersionExW as GetVersionEx; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadIconW as LoadIcon; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::PeekMessageW as PeekMessage; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetupDiGetDeviceInstanceIdW as SetupDiGetDeviceInstanceId; // sdk alias
pub use crate::prelude::SetupDiSetDeviceInstallParamsW as SetupDiSetDeviceInstallParams; // sdk alias
pub use crate::prelude::UnregisterClassW as UnregisterClass; // sdk alias
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
#[macro_export] macro_rules! DialogBox { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! TabCtrl_GetCurSel { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 11)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! TabCtrl_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1300 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
pub const DD_OK: HRESULT = ((0) as HRESULT);
pub const ENUM_CURRENT_SETTINGS: DWORD = (((-(1 as c_int))) as DWORD);
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
#[inline]
pub unsafe fn IDirect3D9_GetAdapterIdentifier(This: *mut IDirect3D9, Adapter: UINT, Flags: DWORD, pIdentifier: *mut D3DADAPTER_IDENTIFIER9) -> HRESULT { ((*(*This).lpVtbl).GetAdapterIdentifier.unwrap_unchecked())(This, Adapter, Flags, pIdentifier) } // widl C wrapper
#[inline]
pub unsafe fn IDirect3D9_Release(This: *mut IDirect3D9) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirectSound8_Release(This: *mut IDirectSound8) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IDirectSound8_VerifyCertification(This: *mut IDirectSound8, pdwCertified: LPDWORD) -> HRESULT { ((*(*This).lpVtbl).VerifyCertification.unwrap_unchecked())(This, pdwCertified) } // widl C wrapper
pub const LOCALE_SYSTEM_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x02) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub use crate::{DialogBox, ListView_GetItemCount, MAKEINTRESOURCE, TabCtrl_GetCurSel, TabCtrl_GetItemCount};
// ==== end SDK header macros ====
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
