//! sc-rust prelude — the single import surface the translated modules use.
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
pub const SC_ACTION_NONE: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_NONE; // island enum
pub const SC_ACTION_REBOOT: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_REBOOT; // island enum
pub const SC_ACTION_RESTART: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_RESTART; // island enum
pub const SC_ACTION_RUN_COMMAND: _SC_ACTION_TYPE = _SC_ACTION_TYPE_SC_ACTION_RUN_COMMAND; // island enum
pub const SC_ENUM_PROCESS_INFO: _SC_ENUM_TYPE = _SC_ENUM_TYPE_SC_ENUM_PROCESS_INFO; // island enum
pub const SC_STATUS_PROCESS_INFO: _SC_STATUS_TYPE = _SC_STATUS_TYPE_SC_STATUS_PROCESS_INFO; // island enum
extern "C" {
    pub fn _getch() -> i32; // msvcrt-crt-rust/src/scanf.rs
    pub fn tolower(c: c_int) -> c_int; // kernel32-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // netcfgx-rust/src/prelude.rs
    pub fn ControlService(arg1: SC_HANDLE, arg2: DWORD, arg3: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn DeleteService(arg1: SC_HANDLE) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_amd64.rs
    pub fn QueryServiceObjectSecurity( hService: SC_HANDLE, dwSecurityInformation: SECURITY_INFORMATION, lpSecurityDescriptor: PSECURITY_DESCRIPTOR, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, ) -> BOOL; // ntmarta-rust/src/prelude.rs
    pub fn QueryServiceStatusEx( hService: SC_HANDLE, InfoLevel: SC_STATUS_TYPE, lpBuffer: LPBYTE, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn SetServiceObjectSecurity( arg1: SC_HANDLE, arg2: SECURITY_INFORMATION, arg3: PSECURITY_DESCRIPTOR, ) -> BOOL; // ntmarta-rust/src/prelude.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "C" {
    pub fn _wtoi(_Str: *const wchar_t) -> ::core::ffi::c_int; // misc.c
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcstoul(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: ::core::ffi::c_int,
    ) -> ::core::ffi::c_ulong; // misc.c
    pub fn wprintf(_Format: *const wchar_t, ...) -> ::core::ffi::c_int; // config.c
}
extern "system" {
    pub fn ChangeServiceConfig2W(_: SC_HANDLE, _: DWORD, _: LPVOID) -> BOOL; // description.c
    pub fn ChangeServiceConfigW(
        _: SC_HANDLE,
        _: DWORD,
        _: DWORD,
        _: DWORD,
        _: LPCWSTR,
        _: LPCWSTR,
        _: LPDWORD,
        _: LPCWSTR,
        _: LPCWSTR,
        _: LPCWSTR,
        _: LPCWSTR,
    ) -> BOOL; // config.c
    pub fn ConvertSecurityDescriptorToStringSecurityDescriptorW(
        _: PSECURITY_DESCRIPTOR,
        _: DWORD,
        _: SECURITY_INFORMATION,
        _: *mut LPWSTR,
        _: PULONG,
    ) -> BOOL; // sdshow.c
    pub fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        _: LPCWSTR,
        _: DWORD,
        _: *mut PSECURITY_DESCRIPTOR,
        _: PULONG,
    ) -> BOOL; // sdset.c
    pub fn CreateServiceW(
        _: SC_HANDLE,
        _: LPCWSTR,
        _: LPCWSTR,
        _: DWORD,
        _: DWORD,
        _: DWORD,
        _: DWORD,
        _: LPCWSTR,
        _: LPCWSTR,
        _: PDWORD,
        _: LPCWSTR,
        _: LPCWSTR,
        _: LPCWSTR,
    ) -> SC_HANDLE; // create.c
    pub fn EnumDependentServicesW(
        hService: SC_HANDLE,
        dwServiceState: DWORD,
        lpServices: LPENUM_SERVICE_STATUSW,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
        lpServicesReturned: LPDWORD,
    ) -> BOOL; // depend.c
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
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetServiceDisplayNameW(
        hSCManager: SC_HANDLE,
        lpServiceName: LPCWSTR,
        lpDisplayName: LPWSTR,
        lpcchBuffer: LPDWORD,
    ) -> BOOL; // name.c
    pub fn GetServiceKeyNameW(
        hSCManager: SC_HANDLE,
        lpDisplayName: LPCWSTR,
        lpServiceName: LPWSTR,
        lpcchBuffer: LPDWORD,
    ) -> BOOL; // name.c
    pub fn OpenSCManagerW(_: LPCWSTR, _: LPCWSTR, _: DWORD) -> SC_HANDLE; // config.c
    pub fn OpenServiceW(_: SC_HANDLE, _: LPCWSTR, _: DWORD) -> SC_HANDLE; // config.c
    pub fn QueryServiceConfig2W(
        hService: SC_HANDLE,
        dwInfoLevel: DWORD,
        lpBuffer: LPBYTE,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
    ) -> BOOL; // description.c
    pub fn QueryServiceConfigW(
        hService: SC_HANDLE,
        lpServiceConfig: LPQUERY_SERVICE_CONFIGW,
        cbBufSize: DWORD,
        pcbBytesNeeded: LPDWORD,
    ) -> BOOL; // config.c
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: *mut LPCWSTR,
    ) -> BOOL; // start.c
    pub fn lstrcmpiW(_: LPCWSTR, _: LPCWSTR) -> ::core::ffi::c_int; // misc.c
    pub fn lstrlenW(_: LPCWSTR) -> ::core::ffi::c_int; // create.c
}
pub use crate::prelude::ChangeServiceConfigW as ChangeServiceConfig; // sdk alias
pub use crate::prelude::ChangeServiceConfig2W as ChangeServiceConfig2; // sdk alias
pub use crate::prelude::ConvertSecurityDescriptorToStringSecurityDescriptorW as ConvertSecurityDescriptorToStringSecurityDescriptor; // sdk alias
pub use crate::prelude::ConvertStringSecurityDescriptorToSecurityDescriptorW as ConvertStringSecurityDescriptorToSecurityDescriptor; // sdk alias
pub use crate::prelude::CreateServiceW as CreateService; // sdk alias
pub use crate::prelude::EnumDependentServicesW as EnumDependentServices; // sdk alias
pub use crate::prelude::EnumServicesStatusExW as EnumServicesStatusEx; // sdk alias
pub use crate::prelude::FormatMessageW as FormatMessage; // sdk alias
pub use crate::prelude::GetServiceDisplayNameW as GetServiceDisplayName; // sdk alias
pub use crate::prelude::GetServiceKeyNameW as GetServiceKeyName; // sdk alias
pub use crate::prelude::OpenSCManagerW as OpenSCManager; // sdk alias
pub use crate::prelude::OpenServiceW as OpenService; // sdk alias
pub use crate::prelude::QueryServiceConfigW as QueryServiceConfig; // sdk alias
pub use crate::prelude::QueryServiceConfig2W as QueryServiceConfig2; // sdk alias
pub use crate::prelude::StartServiceW as StartService; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::wcstoul as _tcstoul; // sdk alias
pub use crate::prelude::wprintf as _tprintf; // sdk alias
pub use crate::prelude::_wtoi as _ttoi; // sdk alias
pub use crate::prelude::lstrcmpiW as lstrcmpi; // sdk alias
pub use crate::prelude::lstrlenW as lstrlen; // sdk alias
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
