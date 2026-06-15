//! k32_types.rs — kernel32-internal types + constants not present in ntdll-sys.
//!
//! Hand-mirrored (repr(C)) from dll/win32/kernel32/include/{kernel32.h,vdm.h}
//! and the subsystem headers. The heavy types (PEB, UNICODE_STRING, LIST_ENTRY,
//! LARGE_INTEGER, RTL_CRITICAL_SECTION, BASE_API_MESSAGE, ...) come from
//! `ntdll_sys::nt` — see substrate.rs.

use ntdll_sys::nt::*;

/* ---- Callback function-pointer types (kernel32.h) ----------------------- */

/// `typedef DWORD (*WaitForInputIdleType)(HANDLE, DWORD);` — no SAL modifier in
/// the header, i.e. __cdecl. Kept faithful to the C declaration.
pub type WaitForInputIdleType =
    Option<unsafe extern "C" fn(hProcess: HANDLE, dwMilliseconds: DWORD) -> DWORD>;

/// `typedef DWORD (WINAPI *PPROCESS_START_ROUTINE)(VOID);`
pub type PPROCESS_START_ROUTINE = Option<unsafe extern "system" fn() -> DWORD>;

/// `NTAPI (*PBASEP_APPCERT_PLUGIN_FUNC)(LPWSTR, ULONG);`
pub type PBASEP_APPCERT_PLUGIN_FUNC =
    Option<unsafe extern "system" fn(ApplicationName: LPWSTR, CertFlag: ULONG) -> NTSTATUS>;

/// `NTAPI (*PBASEP_APPCERT_EMBEDDED_FUNC)(LPWSTR);`
pub type PBASEP_APPCERT_EMBEDDED_FUNC =
    Option<unsafe extern "system" fn(ApplicationName: LPWSTR) -> NTSTATUS>;

/// `NTAPI (*PSAFER_REPLACE_PROCESS_THREAD_TOKENS)(HANDLE, HANDLE, HANDLE);`
pub type PSAFER_REPLACE_PROCESS_THREAD_TOKENS =
    Option<unsafe extern "system" fn(Token: HANDLE, Process: HANDLE, Thread: HANDLE) -> NTSTATUS>;

/* ---- Structs (kernel32.h) ---------------------------------------------- */

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BASEP_APPCERT_ENTRY {
    pub Entry: LIST_ENTRY,
    pub Name: UNICODE_STRING,
    pub fPluginCertFunc: PBASEP_APPCERT_PLUGIN_FUNC,
}
pub type PBASEP_APPCERT_ENTRY = *mut BASEP_APPCERT_ENTRY;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BASE_MSG_SXS_HANDLES {
    pub File: HANDLE,
    pub Process: HANDLE,
    pub Section: HANDLE,
    pub ViewBase: LARGE_INTEGER,
}
pub type PBASE_MSG_SXS_HANDLES = *mut BASE_MSG_SXS_HANDLES;

/* ---- VDM constants (sdk/include/reactos/subsys/win/vdm.h) --------------- */

pub const VDM_UNDO_PARTIAL: ULONG = 0x01;
pub const VDM_UNDO_FULL: ULONG = 0x02;
pub const VDM_UNDO_REUSE: ULONG = 0x04;
pub const VDM_UNDO_COMPLETED: ULONG = 0x08;

pub const BINARY_TYPE_EXE: ULONG = 0x01;
pub const BINARY_TYPE_COM: ULONG = 0x02;
pub const BINARY_TYPE_PIF: ULONG = 0x03;
pub const BINARY_TYPE_DOS: ULONG = 0x10;
pub const BINARY_TYPE_SEPARATE_WOW: ULONG = 0x20;
pub const BINARY_TYPE_WOW: ULONG = 0x40;
pub const BINARY_TYPE_WOW_EX: ULONG = 0x80;

pub const VDM_NOT_LOADED: ULONG = 0x01;
pub const VDM_NOT_READY: ULONG = 0x02;
pub const VDM_READY: ULONG = 0x04;

/// `BaseUpdateVDMEntry` update-index values (win/vdm.h enum-ish).
pub const VdmEntryUndo: ULONG = 0;
pub const VdmEntryUpdateProcess: ULONG = 1;

/* ---- CMD string (proc.c #define CMD_STRING L"cmd /c ") ----------------- */
// The literal is built inline by the translator (wchar arrays); no constant
// needed here, but the length 7 (chars) is implied by wcslen(CMD_STRING).
