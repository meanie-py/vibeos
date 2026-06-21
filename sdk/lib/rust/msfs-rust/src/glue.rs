//! glue.rs — every integer/NTSTATUS constant + pool tag msfs.c references, with EXACT
//! values harvested from the SDK headers + the ntkrnl-sys bindgen substrate. Hand-checked,
//! never fabricated. Mirrors mup-rust/glue.rs.
#![allow(non_upper_case_globals, dead_code)]

use ntkrnl_sys::base::{NTSTATUS, ULONG};
use ntkrnl_sys::ddk::{EVENT_TYPE, KWAIT_REASON, POOL_TYPE};

// ---- pool tags (C multichar constants: leftmost char is the high byte) ----
// 'cFsM' / 'fFsM' / 'NFsM' / 'rFsM' — value = (s0<<24)|(s1<<16)|(s2<<8)|s3.
pub const fn tag4(s: &[u8; 4]) -> ULONG {
    ((s[0] as ULONG) << 24) | ((s[1] as ULONG) << 16) | ((s[2] as ULONG) << 8) | (s[3] as ULONG)
}
pub const TAG_CCB: ULONG = tag4(b"cFsM"); // Ccb allocations
pub const TAG_FCB: ULONG = tag4(b"fFsM"); // Fcb allocations
pub const TAG_NAME: ULONG = tag4(b"NFsM"); // Fcb name buffer + DPC context
pub const TAG_MSG: ULONG = tag4(b"rFsM"); // queued message buffers

// ---- IRP major function indices (xdk) -------------------------------------
pub const IRP_MJ_CREATE: usize = 0x00;
pub const IRP_MJ_CLOSE: usize = 0x02;
pub const IRP_MJ_READ: usize = 0x03;
pub const IRP_MJ_WRITE: usize = 0x04;
pub const IRP_MJ_QUERY_INFORMATION: usize = 0x05;
pub const IRP_MJ_SET_INFORMATION: usize = 0x06;
pub const IRP_MJ_FILE_SYSTEM_CONTROL: usize = 0x0D;
pub const IRP_MJ_CREATE_MAILSLOT: usize = 0x13;

// ---- device / object / IO flags (xdk; values pinned vs ntkrnl-sys ddk) ----
pub const FILE_DEVICE_MAILSLOT: ULONG = 12; // 0x0000000C
pub const DO_DIRECT_IO: ULONG = 0x0000_0010;
pub const FO_MAILSLOT: ULONG = 0x0000_0200; // 512
pub const IO_NO_INCREMENT: i8 = 0; // CCHAR PriorityBoost

// ---- pool type / event type / wait reason ---------------------------------
pub const NonPagedPool: POOL_TYPE = 0;
pub const SynchronizationEvent: EVENT_TYPE = 1;
pub const Executive: KWAIT_REASON = 0;
pub const KernelMode: i8 = 0; // KPROCESSOR_MODE (CHAR)

// ---- Mm priority + misc ----------------------------------------------------
pub const NormalPagePriority: ULONG = 16; // _MM_PAGE_PRIORITY::NormalPagePriority
pub const MAXULONG: ULONG = 0xFFFF_FFFF;
pub const UNICODE_NULL_SIZE: u16 = 2; // sizeof(UNICODE_NULL) == sizeof(WCHAR)

// ---- FILE_INFORMATION_CLASS values used by finfo.c ------------------------
pub const FileMailslotQueryInformation: u32 = 26;
pub const FileMailslotSetInformation: u32 = 27;

// ---- BOOLEAN -------------------------------------------------------------
pub const TRUE: ntkrnl_sys::base::BOOLEAN = 1;
pub const FALSE: ntkrnl_sys::base::BOOLEAN = 0;

// ---- NTSTATUS (psdk/ntstatus.h, exact) ------------------------------------
pub const STATUS_SUCCESS: NTSTATUS = 0x0000_0000u32 as NTSTATUS;
pub const STATUS_PENDING: NTSTATUS = 0x0000_0103u32 as NTSTATUS;
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x8000_0005u32 as NTSTATUS;
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS;
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as NTSTATUS;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC000_0017u32 as NTSTATUS;
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC000_0022u32 as NTSTATUS;
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
pub const STATUS_IO_TIMEOUT: NTSTATUS = 0xC000_00B5u32 as NTSTATUS;
pub const STATUS_CANCELLED: NTSTATUS = 0xC000_0120u32 as NTSTATUS;
