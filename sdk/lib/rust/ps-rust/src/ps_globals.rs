//! PS-subsystem global storage — promoted from `ntoskrnl/kernel_c_helpers.c`.
//!
//! These statics were defined in C with no init or with `RTL_CONSTANT_STRING`
//! / `GENERIC_MAPPING { ... }` aggregate initialisers; the Rust ports in
//! `psmgr.rs` / `process.rs` / `security.rs` / `thread.rs` / `dbgkutil.rs`
//! imported them via `extern "system" { static mut X: T; }`. Moving the
//! storage to Rust means the corresponding extern decls in those files have
//! their lines deleted.
//!
//! Layout-trivial types (PVOID / ULONG / BOOLEAN / HANDLE) zero-init by
//! convention. The two non-trivial cases are:
//!   - `PsNtDllPathName: UNICODE_STRING` — `RTL_CONSTANT_STRING(L"...")`
//!     becomes a `pub static` with a `pub static` UTF-16 buffer.
//!   - `Psp{Process,Thread}Mapping: GENERIC_MAPPING` — compile-time const
//!     struct init using `ntkrnl_sys::constants::*` access flags.

use ntkrnl_sys::base::{BOOLEAN, PVOID, ULONG};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    HANDLE, PEPROCESS, PHANDLE_TABLE, PTOKEN, _GENERIC_MAPPING, _UNICODE_STRING,
};

// PSP* — psmgr.c
#[no_mangle]
pub static mut PspSystemDllBase: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut PspSystemDllSection: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut PspSystemDllEntryPoint: PVOID = core::ptr::null_mut();

// PsNtDllPathName — `RTL_CONSTANT_STRING(L"\\SystemRoot\\System32\\ntdll.dll")`.
// 30 wchars + NUL = 31 wchars total; UNICODE_STRING.Length excludes the NUL.
const NTDLL_PATH_BUF: [u16; 31] = [
    0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0052, 0x006F, 0x006F, // \SystemRoo
    0x0074, 0x005C, 0x0053, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, // t\System32
    0x005C, 0x006E, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, // \ntdll.dll
    0x0000,
];

#[no_mangle]
pub static mut PsNtDllPathName: _UNICODE_STRING = _UNICODE_STRING {
    Length: 60,         // 30 wchars * 2 bytes
    MaximumLength: 62,  // including NUL
    Buffer: NTDLL_PATH_BUF.as_ptr() as *mut u16,
};

#[no_mangle]
pub static mut PspCidTable: PHANDLE_TABLE = core::ptr::null_mut();

#[no_mangle]
pub static mut PsInitialSystemProcess: PEPROCESS = core::ptr::null_mut();
#[no_mangle]
pub static mut PsIdleProcess: PEPROCESS = core::ptr::null_mut();
#[no_mangle]
pub static mut PspInitialSystemProcessHandle: HANDLE = core::ptr::null_mut();

#[no_mangle]
pub static mut PsMinimumWorkingSet: ULONG = 0;
#[no_mangle]
pub static mut PsMaximumWorkingSet: ULONG = 0;
#[no_mangle]
pub static mut PspDefaultPagedLimit: ULONG = 0;
#[no_mangle]
pub static mut PspDefaultNonPagedLimit: ULONG = 0;
#[no_mangle]
pub static mut PspDefaultPagefileLimit: ULONG = 0;
#[no_mangle]
pub static mut PspDoingGiveBacks: BOOLEAN = 0;

// KeUserPopEntrySList* — KE-side, but defined in kernel_c_helpers.c.
#[no_mangle]
pub static mut KeUserPopEntrySListEnd: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut KeUserPopEntrySListFault: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut KeUserPopEntrySListResume: PVOID = core::ptr::null_mut();

// THREAD_ALERT — not in `ntkrnl_sys::constants` (yet); inline here.
const THREAD_ALERT: ULONG = 0x0004;

// GenericAll is the NT6 PROCESS_ALL_ACCESS (0x001FFFFF). It was once pinned to the NT5
// 0x001F0FFF because the C half of the kernel still built the NT5 macro, and a table
// wider than the C's mask hung session-manager startup. Upstream 4b4063a01d6 moved the
// headers to NT6 under __REACTOS__, so both halves now agree on the wide value.

// Psp{Process,Thread}Mapping — `GENERIC_MAPPING` access-rights tables.
#[no_mangle]
pub static mut PspProcessMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
    GenericWrite: STANDARD_RIGHTS_WRITE
        | PROCESS_CREATE_PROCESS
        | PROCESS_CREATE_THREAD
        | PROCESS_VM_OPERATION
        | PROCESS_VM_WRITE
        | PROCESS_DUP_HANDLE
        | PROCESS_TERMINATE
        | PROCESS_SET_QUOTA
        | PROCESS_SET_INFORMATION
        | PROCESS_SUSPEND_RESUME,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: PROCESS_ALL_ACCESS,
};

#[no_mangle]
pub static mut PspThreadMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION,
    GenericWrite: STANDARD_RIGHTS_WRITE
        | THREAD_TERMINATE
        | THREAD_SUSPEND_RESUME
        | THREAD_ALERT
        | THREAD_SET_INFORMATION
        | THREAD_SET_CONTEXT
        | THREAD_SET_LIMITED_INFORMATION,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE | THREAD_QUERY_LIMITED_INFORMATION,
    GenericAll: THREAD_ALL_ACCESS,
};

// security.c
#[no_mangle]
pub static mut PspBootAccessToken: PTOKEN = core::ptr::null_mut();

// SharedUserData — `((KUSER_SHARED_DATA * const) KI_USER_SHARED_DATA)`.
// Fixed kernel VA per arch — see `sdk/include/xdk/{x86,amd64,arm64}/ke.h`.
#[cfg(target_arch = "x86")]
const KI_USER_SHARED_DATA_ADDR: usize = 0xffdf0000;
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const KI_USER_SHARED_DATA_ADDR: usize = 0xFFFFF78000000000;

#[no_mangle]
pub static mut SharedUserData: *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA =
    KI_USER_SHARED_DATA_ADDR as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA;
