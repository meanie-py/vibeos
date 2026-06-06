//! basesrv-private VDM types + constants.
//!
//! These live in `subsystems/win/basesrv/vdm.h` and `sdk/include/reactos/
//! subsys/win/vdm.h` — they are NOT in ntdll-sys (not part of the NT API
//! surface), so the vdm.c translation references them but can't import them.
//! Define them ONCE here (layout-exact, matching the C headers) so vdm.rs and
//! proc.rs share the same view; proc.rs previously carried a TRUNCATED
//! `VDM_CONSOLE_RECORD` (ProcessId at offset 0 instead of the real offset 24)
//! — the same truncated-mirror footgun as SMP_SUBSYSTEM. Both now use these.

use core::ffi::c_void;
use ntdll_sys::nt::*;

// --- VDM task state (win/vdm.h) ---
pub const VDM_NOT_LOADED: USHORT = 0x01;
pub const VDM_NOT_READY: USHORT = 0x02;
pub const VDM_READY: USHORT = 0x04;

// --- binary type (winbase.h) ---
pub const BINARY_TYPE_SEPARATE_WOW: ULONG = 0x20;

// --- VDM undo flags (win/vdm.h) ---
pub const VDM_UNDO_PARTIAL: ULONG = 0x01;
pub const VDM_UNDO_FULL: ULONG = 0x02;
pub const VDM_UNDO_REUSE: ULONG = 0x04;
pub const VDM_UNDO_COMPLETED: ULONG = 0x08;

// --- VDM flags (win/vdm.h) ---
pub const VDM_FLAG_FIRST_TASK: ULONG = 0x01;
pub const VDM_FLAG_WOW: ULONG = 0x02;
pub const VDM_FLAG_DOS: ULONG = 0x04;
pub const VDM_FLAG_RETRY: ULONG = 0x08;
pub const VDM_INC_REENTER_COUNT: ULONG = 0x10;
pub const VDM_DEC_REENTER_COUNT: ULONG = 0x20;
pub const VDM_FLAG_NESTED_TASK: ULONG = 0x40;
pub const VDM_FLAG_DONT_WAIT: ULONG = 0x80;
pub const VDM_GET_FIRST_COMMAND: ULONG = 0x100;
pub const VDM_GET_ENVIRONMENT: ULONG = 0x400;
pub const VDM_FLAG_SEPARATE_WOW: ULONG = 0x800;
pub const VDM_LIST_WOW_PROCESSES: ULONG = 0x1000;
pub const VDM_LIST_WOW_TASKS: ULONG = 0x4000;
pub const VDM_ADD_WOW_TASK: ULONG = 0x8000;

// --- shared basesrv helpers (C macros / CRT) referenced by the dosdev/vdm
//     translations but absent from ntdll-sys ---
#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

#[inline]
pub unsafe fn HandleToUlong(h: HANDLE) -> ULONG {
    h as usize as ULONG
}

// CRT wide-string functions — provided by the C runtime at basesrv.dll link.
extern "C" {
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcsncpy(dst: *mut u16, src: *const u16, n: usize) -> *mut u16;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> i32;
    pub fn _snwprintf(buf: *mut u16, count: usize, fmt: *const u16, ...) -> i32;
}

// --- NTSTATUS not in the bindgen subset ---
pub const STATUS_VDM_DISALLOWED: NTSTATUS = 0xC0000414u32 as NTSTATUS;

// --- VDM policy registry strings (vdm.h, L"...") — [u16] statics (align 2),
//     consumed by RtlInitUnicodeString via `.as_ptr()`. ---
pub static VDM_POLICY_KEY_NAME_BUF: [u16; 46] = [
    83, 111, 102, 116, 119, 97, 114, 101, 92, 80, 111, 108, 105, 99, 105, 101, 115, 92, 77, 105,
    99, 114, 111, 115, 111, 102, 116, 92, 87, 105, 110, 100, 111, 119, 115, 92, 65, 112, 112, 67,
    111, 109, 112, 97, 116, 0,
];
pub static VDM_DISALLOWED_VALUE_NAME_BUF: [u16; 14] = [
    86, 68, 77, 68, 105, 115, 97, 108, 108, 111, 119, 101, 100, 0,
];

// --- VDM_COMMAND_INFO (win/vdm.h) ---
#[repr(C)]
pub struct _VDM_COMMAND_INFO {
    pub TaskId: ULONG,
    pub CreationFlags: ULONG,
    pub ExitCode: ULONG,
    pub CodePage: ULONG,
    pub StdIn: HANDLE,
    pub StdOut: HANDLE,
    pub StdErr: HANDLE,
    pub CmdLine: *mut i8,
    pub AppName: *mut i8,
    pub PifFile: *mut i8,
    pub CurDirectory: *mut i8,
    pub Env: *mut i8,
    pub EnvLen: ULONG,
    pub StartupInfo: STARTUPINFOA,
    pub Desktop: *mut i8,
    pub DesktopLen: ULONG,
    pub Title: *mut i8,
    pub TitleLen: ULONG,
    pub Reserved: *mut c_void,
    pub ReservedLen: ULONG,
    pub CmdLen: USHORT,
    pub AppLen: USHORT,
    pub PifLen: USHORT,
    pub CurDirectoryLen: USHORT,
    pub VDMState: USHORT,
    pub CurrentDrive: USHORT,
    pub ComingFromBat: BOOLEAN,
}
pub type VDM_COMMAND_INFO = _VDM_COMMAND_INFO;
pub type PVDM_COMMAND_INFO = *mut _VDM_COMMAND_INFO;

// --- VDM_CONSOLE_RECORD (vdm.h) ---
#[repr(C)]
pub struct _VDM_CONSOLE_RECORD {
    pub Entry: LIST_ENTRY,
    pub ConsoleHandle: HANDLE,
    pub ProcessHandle: HANDLE,
    pub ServerEvent: HANDLE,
    pub ClientEvent: HANDLE,
    pub ProcessId: ULONG,
    pub ReenterCount: ULONG,
    pub CurrentDirs: *mut i8,
    pub CurDirsLength: ULONG,
    pub SessionId: ULONG,
    pub DosListHead: LIST_ENTRY,
}
pub type VDM_CONSOLE_RECORD = _VDM_CONSOLE_RECORD;
pub type PVDM_CONSOLE_RECORD = *mut _VDM_CONSOLE_RECORD;

// --- VDM_DOS_RECORD (vdm.h) ---
#[repr(C)]
pub struct _VDM_DOS_RECORD {
    pub Entry: LIST_ENTRY,
    pub State: USHORT,
    pub ExitCode: ULONG,
    pub ServerEvent: HANDLE,
    pub ClientEvent: HANDLE,
    pub CommandInfo: PVDM_COMMAND_INFO,
}
pub type VDM_DOS_RECORD = _VDM_DOS_RECORD;
pub type PVDM_DOS_RECORD = *mut _VDM_DOS_RECORD;
