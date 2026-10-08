//! powercfg-rust prelude — the single import surface the translated modules use.
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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UNICODE_NULL, UNREFERENCED_PARAMETER, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// resource.h:17 `#define IDC_CPLICON IDI_AC`.
pub const IDC_CPLICON: u32 = IDI_AC;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ANSI_NULL: CHAR = 0; // freeldr-rust/src/ui_keystone.rs
pub const BatteryDeviceName: _BATTERY_QUERY_INFORMATION_LEVEL = _BATTERY_QUERY_INFORMATION_LEVEL_BatteryDeviceName; // island enum
pub const BatteryEstimatedTime: _BATTERY_QUERY_INFORMATION_LEVEL = _BATTERY_QUERY_INFORMATION_LEVEL_BatteryEstimatedTime; // island enum
pub const BatteryInformation: _BATTERY_QUERY_INFORMATION_LEVEL = _BATTERY_QUERY_INFORMATION_LEVEL_BatteryInformation; // island enum
pub const BatteryManufactureName: _BATTERY_QUERY_INFORMATION_LEVEL = _BATTERY_QUERY_INFORMATION_LEVEL_BatteryManufactureName; // island enum
pub const BatteryUniqueID: _BATTERY_QUERY_INFORMATION_LEVEL = _BATTERY_QUERY_INFORMATION_LEVEL_BatteryUniqueID; // island enum
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY; // dhcpcsvc-rust/src/prelude.rs
pub const HWND_TOP: HWND = 0 as HWND; // syssetup-rust/src/prelude.rs
pub const PBM_SETPOS: u32 = 1026; // appwiz-rust/src/types_amd64.rs
pub const PBM_SETRANGE: u32 = 1025; // appwiz-rust/src/types_amd64.rs
pub const TBM_SETPOS: u32 = 1029; // appwiz-rust/src/types_amd64.rs
pub const TBM_SETRANGE: u32 = 1030; // appwiz-rust/src/types_amd64.rs
pub const TBM_SETTICFREQ: u32 = 1044; // appwiz-rust/src/types_amd64.rs
extern "C" {
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16; // advapi32-rust/src/prelude.rs
}
extern "system" {
    pub fn CallNtPowerInformation( arg1: POWER_INFORMATION_LEVEL, arg2: PVOID, arg3: ULONG, arg4: PVOID, arg5: ULONG, ) -> NTSTATUS; // shell32-rust/src/island_amd64.rs
    pub fn CanUserWritePwrScheme() -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn DeletePwrScheme(arg1: UINT) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn DeviceIoControl( hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, nInBufferSize: DWORD, lpOutBuffer: LPVOID, nOutBufferSize: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED, ) -> BOOL; // iphlpapi-rust/src/prelude.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnumPwrSchemes(arg1: PWRSCHEMESENUMPROC, arg2: LPARAM) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn FindWindowW(arg1: LPCWSTR, arg2: LPCWSTR) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetActivePwrScheme(arg1: PUINT) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn GetDesktopWindow() -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetPwrCapabilities(arg1: PSYSTEM_POWER_CAPABILITIES) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn GetSystemPowerStatus(arg1: LPSYSTEM_POWER_STATUS) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GlobalMemoryStatusEx(arg1: LPMEMORYSTATUSEX) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn MonitorFromWindow(arg1: HWND, arg2: DWORD) -> HMONITOR; // aclui-rust/src/types_amd64.rs
    pub fn ReadGlobalPwrPolicy(arg1: PGLOBAL_POWER_POLICY) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn ReadPwrScheme(arg1: UINT, arg2: PPOWER_POLICY) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn SHAddFromPropSheetExtArray( arg1: HPSXA, arg2: LPFNADDPROPSHEETPAGE, arg3: LPARAM, ) -> UINT; // desk-rust/src/types_amd64.rs
    pub fn SHCreatePropSheetExtArray(arg1: HKEY, arg2: LPCWSTR, arg3: UINT) -> HPSXA; // desk-rust/src/types_amd64.rs
    pub fn SHDestroyPropSheetExtArray(arg1: HPSXA); // desk-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetActivePwrScheme( arg1: UINT, arg2: PGLOBAL_POWER_POLICY, arg3: PPOWER_POLICY, ) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn SetDlgItemTextA(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiEnumDeviceInterfaces( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: *const GUID, arg4: DWORD, arg5: PSP_DEVICE_INTERFACE_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetClassDevsW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, ) -> HDEVINFO; // desk-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SwitchToThisWindow(arg1: HWND, arg2: BOOL); // aclui-rust/src/types_amd64.rs
    pub fn ValidatePowerPolicies(arg1: PGLOBAL_POWER_POLICY, arg2: PPOWER_POLICY) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn WriteGlobalPwrPolicy(arg1: PGLOBAL_POWER_POLICY) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
    pub fn WritePwrScheme(arg1: PUINT, arg2: LPWSTR, arg3: LPWSTR, arg4: PPOWER_POLICY) -> BOOLEAN; // shell32-rust/src/island_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::powercfg::advanced::AdvancedDlgProc;
pub use crate::powercfg::alarms::AlarmsDlgProc;
pub use crate::powercfg::hibernate::HibernateDlgProc;
pub use crate::powercfg::powermeter::PowerMeterDlgProc;
pub use crate::powercfg::powershemes::PowerSchemesDlgProc;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const IOCTL_BATTERY_QUERY_INFORMATION: i32 = 0x294044; // clang, powermeter.c
pub const IOCTL_BATTERY_QUERY_STATUS: i32 = 0x29404C; // clang, powermeter.c
pub const IOCTL_BATTERY_QUERY_TAG: i32 = 0x294040; // clang, powermeter.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _swprintf(_Dest: *mut wchar_t, _Format: *const wchar_t, ...) -> ::core::ffi::c_int; // powershemes.c
    pub fn wcslen(_Str: *const wchar_t) -> ::core::ffi::c_ulonglong; // powershemes.c
}
extern "system" {
    pub fn CreatePropertySheetPageW(_: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE; // powercfg.c
    pub fn DialogBoxParamW(_: HINSTANCE, _: LPCWSTR, _: HWND, _: DLGPROC, _: LPARAM) -> INT_PTR; // powershemes.c
    pub fn GetDiskFreeSpaceExW(
        _: LPCWSTR,
        _: PULARGE_INTEGER,
        _: PULARGE_INTEGER,
        _: PULARGE_INTEGER,
    ) -> BOOL; // hibernate.c
    pub fn GetDlgItemTextW(
        hDlg: HWND,
        nIDDlgItem: ::core::ffi::c_int,
        lpString: LPWSTR,
        cchMax: ::core::ffi::c_int,
    ) -> UINT; // powershemes.c
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int) -> LONG_PTR; // powershemes.c
    pub fn GetWindowsDirectoryW(_: LPWSTR, _: UINT) -> UINT; // hibernate.c
    pub fn PropertySheetW(_: LPCPROPSHEETHEADERW) -> INT_PTR; // powercfg.c
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(_: HWND, _: ::core::ffi::c_int, _: LONG_PTR) -> LONG_PTR; // powershemes.c
    pub fn SetupDiGetDeviceInterfaceDetailW(
        DeviceInfoSet: HDEVINFO,
        DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA,
        DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W,
        DeviceInterfaceDetailDataSize: DWORD,
        RequiredSize: PDWORD,
        DeviceInfoData: PSP_DEVINFO_DATA,
    ) -> BOOL; // powermeter.c
    pub fn StrFormatByteSizeW(_: LONGLONG, _: LPWSTR, cchBuf: UINT) -> LPWSTR; // hibernate.c
}
pub use crate::prelude::CreatePropertySheetPageW as CreatePropertySheetPage; // sdk alias
pub use crate::prelude::DialogBoxParamW as DialogBoxParam; // sdk alias
pub use crate::prelude::GetDiskFreeSpaceExW as GetDiskFreeSpaceEx; // sdk alias
pub use crate::prelude::GetDlgItemTextW as GetDlgItemText; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::GetWindowsDirectoryW as GetWindowsDirectory; // sdk alias
pub use crate::prelude::LoadStringW as LoadString; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::PropertySheetW as PropertySheet; // sdk alias
pub use crate::prelude::SendDlgItemMessageW as SendDlgItemMessage; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetDlgItemTextW as SetDlgItemText; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::SetupDiGetDeviceInterfaceDetailW as SetupDiGetInterfaceDeviceDetailW; // sdk alias
pub use crate::prelude::StrFormatByteSizeW as StrFormatByteSize; // sdk alias
pub use crate::prelude::_swprintf as _stprintf; // sdk alias
pub use crate::prelude::wcscpy as _tcscpy; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
// ==== end port externs ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! DialogBoxW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), ((0) as _)) }; }
#[macro_export] macro_rules! IS_PWR_HIBERNATE_ALLOWED { ($p0:expr $(,)?) => { ((((((*$p0).SystemS4) as isize) != 0) && ((((*$p0).HiberFilePresent) as isize) != 0)) as c_int) }; }
#[macro_export] macro_rules! IS_PWR_POWEROFF_ALLOWED { ($p0:expr $(,)?) => { (((((((*$p0).SystemS5) as isize == 0) as c_int)) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! IS_PWR_SUSPEND_ALLOWED { ($p0:expr $(,)?) => { (((((((((((*$p0).SystemS1) as isize) != 0) || ((((*$p0).SystemS2) as isize) != 0)) as c_int)) as isize) != 0) || ((((*$p0).SystemS3) as isize) != 0)) as c_int) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! PropSheet_Changed { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), (((1024 + 104)) as _), (($p1) as WPARAM), ((0) as _)) }; }
pub use crate::{DialogBoxW, IS_PWR_HIBERNATE_ALLOWED, IS_PWR_POWEROFF_ALLOWED, IS_PWR_SUSPEND_ALLOWED, MAKEINTRESOURCE, MAKELPARAM, PropSheet_Changed};
// ==== end SDK header macros ====

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
