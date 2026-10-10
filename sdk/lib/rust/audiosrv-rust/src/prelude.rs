//! audiosrv-rust prelude — the single import surface the translated modules use.
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
pub const SC_STATUS_PROCESS_INFO: _SC_STATUS_TYPE = _SC_STATUS_TYPE_SC_STATUS_PROCESS_INFO; // island enum
extern "C" {
    pub fn malloc(size: usize) -> *mut c_void; // avltable/src/tree.rs
}
extern "system" {
    pub fn BroadcastSystemMessageW( flags: DWORD, lpInfo: LPDWORD, Msg: UINT, wParam: WPARAM, lParam: LPARAM, ) -> LONG; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // netcfgx-rust/src/prelude.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn InitializeSecurityDescriptor(arg1: PSECURITY_DESCRIPTOR, arg2: DWORD) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn QueryServiceStatusEx( hService: SC_HANDLE, InfoLevel: SC_STATUS_TYPE, lpBuffer: LPBYTE, cbBufSize: DWORD, pcbBytesNeeded: LPDWORD, ) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn RegisterDeviceNotificationW(arg1: HANDLE, arg2: LPVOID, arg3: DWORD) -> HDEVNOTIFY; // aclui-rust/src/types_amd64.rs
    pub fn RegisterServiceCtrlHandlerExW( arg1: LPCWSTR, arg2: LPHANDLER_FUNCTION_EX, arg3: LPVOID, ) -> SERVICE_STATUS_HANDLE; // syssetup-rust/src/types_amd64.rs
    pub fn RegisterWindowMessageW(arg1: LPCWSTR) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn ReleaseMutex(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetSecurityDescriptorDacl( arg1: PSECURITY_DESCRIPTOR, arg2: BOOL, arg3: PACL, arg4: BOOL, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetServiceStatus(arg1: SERVICE_STATUS_HANDLE, arg2: LPSERVICE_STATUS) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiEnumDeviceInterfaces( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: *const GUID, arg4: DWORD, arg5: PSP_DEVICE_INTERFACE_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetClassDevsExW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, arg5: HDEVINFO, arg6: PCWSTR, arg7: PVOID, ) -> HDEVINFO; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetDeviceInterfaceDetailW( DeviceInfoSet: HDEVINFO, DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA, DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W, DeviceInterfaceDetailDataSize: DWORD, RequiredSize: PDWORD, DeviceInfoData: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_amd64.rs
    pub fn StartServiceCtrlDispatcherW(arg1: *const SERVICE_TABLE_ENTRYW) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn UnregisterDeviceNotification(arg1: HDEVNOTIFY) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::audiosrv::pnp_list_manager::AppendAudioDeviceToList;
pub use crate::audiosrv::pnp_list_manager::CreateAudioDeviceList;
pub use crate::audiosrv::pnp_list_manager::CreateDeviceDescriptor;
pub use crate::audiosrv::pnp_list_manager::DestroyAudioDeviceList;
pub use crate::audiosrv::pnp::HandleDeviceEvent;
pub use crate::audiosrv::pnp_list_lock::InitializeAudioDeviceListLock;
pub use crate::audiosrv::pnp_list_lock::KillAudioDeviceListLock;
pub use crate::audiosrv::pnp_list_lock::LockAudioDeviceList;
pub use crate::audiosrv::pnp::ProcessExistingDevices;
pub use crate::audiosrv::pnp::RegisterForDeviceNotifications;
pub use crate::audiosrv::services::StartSystemAudioServices;
pub use crate::audiosrv::pnp_list_lock::UnlockAudioDeviceList;
pub use crate::audiosrv::pnp::UnregisterDeviceNotifications;
pub use crate::audiosrv::main::service_status_handle;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CreateMutexW(_: LPSECURITY_ATTRIBUTES, _: BOOL, _: LPCWSTR) -> HANDLE; // pnp_list_lock.c
    pub fn OpenSCManagerW(_: LPCWSTR, _: LPCWSTR, _: DWORD) -> SC_HANDLE; // services.c
    pub fn OpenServiceW(_: SC_HANDLE, _: LPCWSTR, _: DWORD) -> SC_HANDLE; // services.c
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: *mut LPCWSTR,
    ) -> BOOL; // services.c
}
pub use crate::prelude::CreateMutexW as CreateMutex; // sdk alias
pub use crate::prelude::OpenSCManagerW as OpenSCManager; // sdk alias
pub use crate::prelude::OpenServiceW as OpenService; // sdk alias
pub use crate::prelude::StartServiceW as StartService; // sdk alias
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
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// ==== end port macros ====

// ================= hand: what the generated blocks above cannot supply ==========================
// winnt.h `#define VOID void`.
pub type VOID = c_void;
// ksmedia.h's STATIC_KSCATEGORY_AUDIO initializer. pnp.c builds its GUID from this rather than
// reading the KSCATEGORY_AUDIO object, so audiosrv links no GUID library; keep it a value.
pub const STATIC_KSCATEGORY_AUDIO: GUID = GUID {
    Data1: 0x6994AD04,
    Data2: 0x93EF,
    Data3: 0x11D0,
    Data4: [0xA3, 0xCC, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};
extern "system" {
    pub fn lstrlenW(lpString: LPCWSTR) -> c_int;
}
extern "C" {
    pub fn free(p: *mut c_void);
}
// audiosrv.h `WideStringSize(str)`: `(lstrlenW(str) + 1) * sizeof(WCHAR)`, a size_t -- the int
// operand converts to size_t, as C's usual arithmetic conversions do.
#[macro_export]
macro_rules! WideStringSize {
    ($p0:expr $(,)?) => {
        ((lstrlenW(($p0) as _) + 1) as usize * core::mem::size_of::<WCHAR>())
    };
}
pub use crate::WideStringSize;
// audiosrv.h:30 `#define DestroyDeviceDescriptor(descriptor) free(descriptor)`: C converts the
// PnP_AudioDevice* to free's void* implicitly; Rust needs it written.
#[allow(unused_macros)]
macro_rules! DestroyDeviceDescriptor {
    ($descriptor:expr $(,)?) => {
        (free(($descriptor) as *mut c_void))
    };
}
#[allow(unused_imports)]
pub(crate) use DestroyDeviceDescriptor;
