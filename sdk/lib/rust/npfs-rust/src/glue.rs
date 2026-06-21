//! glue.rs — npfs.h-internal constants (pool tags, bugcheck file IDs, node-type codes) plus
//! the SDK named-pipe constants the modules reference, all with EXACT header-harvested values.
//! Module translations should use these; if a module needs a constant not here, add it to the
//! module file as a `const` with the exact SDK-header value + a citing comment (never invent).
#![allow(non_upper_case_globals, dead_code)]

use ntkrnl_sys::base::{NTSTATUS, ULONG, USHORT};
pub type PCVOID = *const core::ffi::c_void;

// ---- pool tags (C multichar: leftmost char is the high byte) --------------
pub const fn tag4(s: &[u8; 4]) -> ULONG {
    ((s[0] as ULONG) << 24) | ((s[1] as ULONG) << 16) | ((s[2] as ULONG) << 8) | (s[3] as ULONG)
}
pub const NPFS_CCB_TAG: ULONG = tag4(b"cFpN");
pub const NPFS_ROOT_DCB_CCB_TAG: ULONG = tag4(b"CFpN");
pub const NPFS_DCB_TAG: ULONG = tag4(b"DFpN");
pub const NPFS_FCB_TAG: ULONG = tag4(b"fFpN");
pub const NPFS_GLOBAL_TAG: ULONG = tag4(b"gFpN");
pub const NPFS_CLIENT_INFO_TAG: ULONG = tag4(b"iFpN");
pub const NPFS_NAME_BLOCK_TAG: ULONG = tag4(b"nFpN");
pub const NPFS_QUERY_TEMPLATE_TAG: ULONG = tag4(b"qFpN");
pub const NPFS_DATA_ENTRY_TAG: ULONG = tag4(b"rFpN");
pub const NPFS_CLIENT_SEC_CTX_TAG: ULONG = tag4(b"sFpN");
pub const NPFS_WAIT_BLOCK_TAG: ULONG = tag4(b"tFpN");
pub const NPFS_WRITE_BLOCK_TAG: ULONG = tag4(b"wFpN");

// ---- indexed-length bounds (npfs.h) ---------------------------------------
pub const MIN_INDEXED_LENGTH: usize = 5;
pub const MAX_INDEXED_LENGTH: usize = 9;

// ---- bugcheck (Bug Check 0x25: NPFS_FILE_SYSTEM) --------------------------
pub const NPFS_FILE_SYSTEM: ULONG = 0x25;
pub const NPFS_BUGCHECK_CLEANUP: ULONG = 0x0001;
pub const NPFS_BUGCHECK_CLOSE: ULONG = 0x0002;
pub const NPFS_BUGCHECK_CREATE: ULONG = 0x0003;
pub const NPFS_BUGCHECK_DATASUP: ULONG = 0x0004;
pub const NPFS_BUGCHECK_FILEINFO: ULONG = 0x0005;
pub const NPFS_BUGCHECK_FILEOBSUP: ULONG = 0x0006;
pub const NPFS_BUGCHECK_FLUSHBUF: ULONG = 0x0007;
pub const NPFS_BUGCHECK_FSCTRL: ULONG = 0x0008;
pub const NPFS_BUGCHECK_MAIN: ULONG = 0x0009;
pub const NPFS_BUGCHECK_PREFXSUP: ULONG = 0x000a;
pub const NPFS_BUGCHECK_READ: ULONG = 0x000b;
pub const NPFS_BUGCHECK_READSUP: ULONG = 0x000c;
pub const NPFS_BUGCHECK_SECURSUP: ULONG = 0x000d;
pub const NPFS_BUGCHECK_SEINFO: ULONG = 0x000e;
pub const NPFS_BUGCHECK_STATESUP: ULONG = 0x000f;
pub const NPFS_BUGCHECK_STRUCSUP: ULONG = 0x0010;
pub const NPFS_BUGCHECK_VOLINFO: ULONG = 0x0011;
pub const NPFS_BUGCHECK_WAITSUP: ULONG = 0x0012;
pub const NPFS_BUGCHECK_WRITE: ULONG = 0x0013;
pub const NPFS_BUGCHECK_WRITESUP: ULONG = 0x0014;

// ---- node-type codes (npfs.h) ---------------------------------------------
pub const NPFS_NTC_VCB: USHORT = 1;
pub const NPFS_NTC_ROOT_DCB: USHORT = 2;
pub const NPFS_NTC_FCB: USHORT = 4;
pub const NPFS_NTC_CCB: USHORT = 6;
pub const NPFS_NTC_NONPAGED_CCB: USHORT = 7;
pub const NPFS_NTC_ROOT_DCB_CCB: USHORT = 8;

// ---- named-pipe constants (SDK, exact) ------------------------------------
pub const IO_NAMED_PIPE_INCREMENT: i8 = 2; // CCHAR PriorityBoost
pub const IO_NO_INCREMENT: i8 = 0; // CCHAR PriorityBoost

pub const FILE_PIPE_BYTE_STREAM_TYPE: ULONG = 0x0;
pub const FILE_PIPE_MESSAGE_TYPE: ULONG = 0x1;
pub const FILE_PIPE_TYPE_VALID_MASK: ULONG = 0x3;
pub const FILE_PIPE_BYTE_STREAM_MODE: ULONG = 0x0;
pub const FILE_PIPE_MESSAGE_MODE: ULONG = 0x1;
pub const FILE_PIPE_QUEUE_OPERATION: ULONG = 0x0;
pub const FILE_PIPE_COMPLETE_OPERATION: ULONG = 0x1;
pub const FILE_PIPE_INBOUND: ULONG = 0x0;
pub const FILE_PIPE_OUTBOUND: ULONG = 0x1;
pub const FILE_PIPE_FULL_DUPLEX: ULONG = 0x2;
pub const FILE_PIPE_CLIENT_END: ULONG = 0x0;
pub const FILE_PIPE_SERVER_END: ULONG = 0x1;
pub const FILE_PIPE_DISCONNECTED_STATE: ULONG = 0x1;
pub const FILE_PIPE_LISTENING_STATE: ULONG = 0x2;
pub const FILE_PIPE_CONNECTED_STATE: ULONG = 0x3;
pub const FILE_PIPE_CLOSING_STATE: ULONG = 0x4;
pub const FILE_PIPE_READ_DATA: ULONG = 0x0;
pub const FILE_PIPE_WRITE_SPACE: ULONG = 0x1;

// ---- IRP major function indices (xdk) -------------------------------------
pub const IRP_MJ_CREATE: usize = 0x00;
pub const IRP_MJ_CREATE_NAMED_PIPE: usize = 0x01;
pub const IRP_MJ_CLOSE: usize = 0x02;
pub const IRP_MJ_READ: usize = 0x03;
pub const IRP_MJ_WRITE: usize = 0x04;
pub const IRP_MJ_QUERY_INFORMATION: usize = 0x05;
pub const IRP_MJ_SET_INFORMATION: usize = 0x06;
pub const IRP_MJ_QUERY_VOLUME_INFORMATION: usize = 0x0A;
pub const IRP_MJ_DIRECTORY_CONTROL: usize = 0x0C;
pub const IRP_MJ_FILE_SYSTEM_CONTROL: usize = 0x0D;
pub const IRP_MJ_FLUSH_BUFFERS: usize = 0x09;
pub const IRP_MJ_CLEANUP: usize = 0x12;
pub const IRP_MJ_QUERY_SECURITY: usize = 0x14;
pub const IRP_MJ_SET_SECURITY: usize = 0x15;

// ---- BOOLEAN / mode --------------------------------------------------------
pub const TRUE: ntkrnl_sys::base::BOOLEAN = 1;
pub const FALSE: ntkrnl_sys::base::BOOLEAN = 0;
pub const KernelMode: i8 = 0;
pub const UserMode: i8 = 1;
pub const NonPagedPool: ntkrnl_sys::ddk::POOL_TYPE = 0;
pub const PagedPool: ntkrnl_sys::ddk::POOL_TYPE = 1;

// ---- NTSTATUS (psdk/ntstatus.h, exact) ------------------------------------
pub const STATUS_SUCCESS: NTSTATUS = 0x0000_0000u32 as NTSTATUS;
pub const STATUS_PENDING: NTSTATUS = 0x0000_0103u32 as NTSTATUS;
pub const STATUS_BUFFER_OVERFLOW: NTSTATUS = 0x8000_0005u32 as NTSTATUS;
pub const STATUS_NO_MORE_FILES: NTSTATUS = 0x8000_0006u32 as NTSTATUS;
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS;
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as NTSTATUS;
pub const STATUS_INVALID_INFO_CLASS: NTSTATUS = 0xC000_0003u32 as NTSTATUS;
pub const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC000_0004u32 as NTSTATUS;
pub const STATUS_INVALID_HANDLE: NTSTATUS = 0xC000_0008u32 as NTSTATUS;
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;
pub const STATUS_NO_SUCH_FILE: NTSTATUS = 0xC000_000Fu32 as NTSTATUS;
pub const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC000_0010u32 as NTSTATUS;
pub const STATUS_END_OF_FILE: NTSTATUS = 0xC000_0011u32 as NTSTATUS;
pub const STATUS_NO_MEMORY: NTSTATUS = 0xC000_0017u32 as NTSTATUS;
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC000_0022u32 as NTSTATUS;
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC000_0023u32 as NTSTATUS;
pub const STATUS_OBJECT_NAME_INVALID: NTSTATUS = 0xC000_0033u32 as NTSTATUS;
pub const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC000_0034u32 as NTSTATUS;
pub const STATUS_OBJECT_NAME_COLLISION: NTSTATUS = 0xC000_0035u32 as NTSTATUS;
pub const STATUS_OBJECT_PATH_NOT_FOUND: NTSTATUS = 0xC000_003Au32 as NTSTATUS;
pub const STATUS_OBJECT_PATH_SYNTAX_BAD: NTSTATUS = 0xC000_003Bu32 as NTSTATUS;
pub const STATUS_SHARING_VIOLATION: NTSTATUS = 0xC000_0043u32 as NTSTATUS;
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
pub const STATUS_PIPE_NOT_AVAILABLE: NTSTATUS = 0xC000_00ACu32 as NTSTATUS;
pub const STATUS_INVALID_PIPE_STATE: NTSTATUS = 0xC000_00ADu32 as NTSTATUS;
pub const STATUS_PIPE_BUSY: NTSTATUS = 0xC000_00AEu32 as NTSTATUS;
pub const STATUS_PIPE_DISCONNECTED: NTSTATUS = 0xC000_00B0u32 as NTSTATUS;
pub const STATUS_PIPE_CLOSING: NTSTATUS = 0xC000_00B1u32 as NTSTATUS;
pub const STATUS_PIPE_CONNECTED: NTSTATUS = 0xC000_00B2u32 as NTSTATUS;
pub const STATUS_PIPE_LISTENING: NTSTATUS = 0xC000_00B3u32 as NTSTATUS;
pub const STATUS_INVALID_READ_MODE: NTSTATUS = 0xC000_00B4u32 as NTSTATUS;
pub const STATUS_IO_TIMEOUT: NTSTATUS = 0xC000_00B5u32 as NTSTATUS;
pub const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC000_00BBu32 as NTSTATUS;
pub const STATUS_INSTANCE_NOT_AVAILABLE: NTSTATUS = 0xC000_00ABu32 as NTSTATUS;
pub const STATUS_CANCELLED: NTSTATUS = 0xC000_0120u32 as NTSTATUS;
pub const STATUS_PIPE_BROKEN: NTSTATUS = 0xC000_014Bu32 as NTSTATUS;
pub const STATUS_PIPE_EMPTY: NTSTATUS = 0xC000_00D9u32 as NTSTATUS;
pub const STATUS_NOTIFY_ENUM_DIR: NTSTATUS = 0x0000_010Cu32 as NTSTATUS;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC000_0016u32 as NTSTATUS;
pub const STATUS_BAD_NETWORK_PATH: NTSTATUS = 0xC000_00BEu32 as NTSTATUS;
pub const STATUS_DATATYPE_MISALIGNMENT: NTSTATUS = 0x8000_0002u32 as NTSTATUS;
