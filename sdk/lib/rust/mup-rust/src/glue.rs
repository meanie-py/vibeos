//! glue.rs — every integer constant mup.c/dfs.c references, with EXACT values harvested
//! from the SDK headers (sdk/include/{xdk,psdk}/*.h) and computed CTL_CODEs. Hand-checked,
//! never fabricated — the field manual's "LLM-invented constant" hazard does not apply to
//! a header-driven hand translation, but the values are still pinned to their source here.
#![allow(non_upper_case_globals, dead_code)]

use ntkrnl_sys::base::{NTSTATUS, ULONG};

// ---- mup.h node tags ------------------------------------------------------
pub const NODE_TYPE_VCB: ULONG = 0x1;
pub const NODE_TYPE_UNC: ULONG = 0x2;
pub const NODE_TYPE_PFX: ULONG = 0x3;
pub const NODE_TYPE_FCB: ULONG = 0x4;
pub const NODE_TYPE_CCB: ULONG = 0x5;
pub const NODE_TYPE_MIC: ULONG = 0x6;
pub const NODE_TYPE_MQC: ULONG = 0x8;

pub const NODE_STATUS_HEALTHY: ULONG = 0x1;
pub const NODE_STATUS_CLEANUP: ULONG = 0x2;

// mup.h: #define TAG_MUP ' puM' — multi-char constant, first char is the high byte:
// (' '<<24)|('p'<<16)|('u'<<8)|'M' = 0x2070754D.
pub const TAG_MUP: ULONG = 0x2070_754D;

// ---- access masks (xdk) ---------------------------------------------------
pub const SYNCHRONIZE: ULONG = 0x0010_0000;
pub const STANDARD_RIGHTS_REQUIRED: ULONG = 0x000F_0000;
// FILE_ALL_ACCESS = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x1FF
pub const FILE_ALL_ACCESS: ULONG = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x1FF;
// mup.h: FILE_ALL_ACCESS & ~STANDARD_RIGHTS_REQUIRED & ~SYNCHRONIZE == 0x1FF
pub const FILE_SIMPLE_RIGHTS_MASK: ULONG = FILE_ALL_ACCESS & !STANDARD_RIGHTS_REQUIRED & !SYNCHRONIZE;

// ---- dfs.h open-context cookies -------------------------------------------
pub const DFS_OPEN_CONTEXT: ULONG = 0xFF44_4653;
pub const DFS_DOWNLEVEL_OPEN_CONTEXT: ULONG = 0x1144_4653;
pub const DFS_CSCAGENT_NAME_CONTEXT: ULONG = 0xAAAA_AAAA;
pub const DFS_USER_NAME_CONTEXT: ULONG = 0xBBBB_BBBB;

// ---- CTL codes (computed: (Device<<16)|(Access<<14)|(Function<<2)|Method) --
// FSCTL_MUP_REGISTER_PROVIDER = CTL_CODE(FILE_DEVICE_MULTI_UNC_PROVIDER=0x10,1,BUFFERED,ANY)
pub const FSCTL_MUP_REGISTER_PROVIDER: ULONG = 0x0010_0004;
// IOCTL_REDIR_QUERY_PATH = CTL_CODE(FILE_DEVICE_NETWORK_FILE_SYSTEM=0x14,99,NEITHER,ANY)
pub const IOCTL_REDIR_QUERY_PATH: ULONG = 0x0014_018F;

// ---- device types (xdk) ---------------------------------------------------
pub const FILE_DEVICE_DFS: ULONG = 0x0000_0006;
pub const FILE_DEVICE_MULTI_UNC_PROVIDER: ULONG = 0x0000_0010;
pub const FILE_DEVICE_DFS_FILE_SYSTEM: ULONG = 0x0000_0035;

// ---- I/O method bits (CTL code low 2 bits) --------------------------------
pub const METHOD_BUFFERED: ULONG = 0;
pub const METHOD_IN_DIRECT: ULONG = 1;
pub const METHOD_OUT_DIRECT: ULONG = 2;
pub const METHOD_NEITHER: ULONG = 3;

// ---- IRP / DEVICE / FILE_OBJECT flags (xdk) -------------------------------
pub const IRP_NOCACHE: ULONG = 0x0000_0001;
pub const IRP_BUFFERED_IO: ULONG = 0x0000_0010;
pub const IRP_DEALLOCATE_BUFFER: ULONG = 0x0000_0020;
pub const IRP_INPUT_OPERATION: ULONG = 0x0000_0040;
pub const IRP_WRITE_OPERATION: ULONG = 0x0000_0200;
pub const DO_BUFFERED_IO: ULONG = 0x0000_0004;
pub const DO_DIRECT_IO: ULONG = 0x0000_0010;
pub const FO_NO_INTERMEDIATE_BUFFERING: ULONG = 0x0000_0008;
pub const FO_WRITE_THROUGH: ULONG = 0x0000_0010;
pub const SL_WRITE_THROUGH: ULONG = 0x0000_0004;

// ---- object / key / share / create flags (xdk) ----------------------------
pub const OBJ_CASE_INSENSITIVE: ULONG = 0x0000_0040;
pub const OBJ_KERNEL_HANDLE: ULONG = 0x0000_0200;
pub const KEY_READ: ULONG = 0x0002_0019;
pub const KEY_QUERY_VALUE: ULONG = 0x0000_0001;
pub const REG_DWORD: ULONG = 4;
pub const FILE_SHARE_READ: ULONG = 0x0000_0001;
pub const FILE_SHARE_WRITE: ULONG = 0x0000_0002;
pub const FILE_OPEN: ULONG = 0x0000_0001;
pub const FILE_DIRECTORY_FILE: ULONG = 0x0000_0001;
pub const FILE_TRAVERSE: ULONG = 0x0000_0020;
pub const FILE_ATTRIBUTE_VALID_FLAGS: ULONG = 0x0000_7FB7;
pub const FILE_SHARE_VALID_FLAGS: ULONG = 0x0000_0007;
pub const FILE_VALID_SET_FLAGS: ULONG = 0x0000_0036;
pub const IO_NO_PARAMETER_CHECKING: ULONG = 0x0000_0100;

// ---- IoStatus.Information dispositions -------------------------------------
pub const FILE_SUPERSEDED: ULONG = 0x0000_0000;
pub const FILE_OPENED: ULONG = 0x0000_0001;

// ---- IRP major function indices (xdk) -------------------------------------
pub const IRP_MJ_CREATE: usize = 0x00;
pub const IRP_MJ_CREATE_NAMED_PIPE: usize = 0x01;
pub const IRP_MJ_CLOSE: usize = 0x02;
pub const IRP_MJ_WRITE: usize = 0x04;
pub const IRP_MJ_DEVICE_CONTROL: usize = 0x0E;
pub const IRP_MJ_FILE_SYSTEM_CONTROL: usize = 0x0D;
pub const IRP_MJ_CLEANUP: usize = 0x12;
pub const IRP_MJ_CREATE_MAILSLOT: usize = 0x13;

// ---- KPROCESSOR_MODE (CHAR/i8): KernelMode=0, UserMode=1 ------------------
pub const KernelMode: i8 = 0;
pub const UserMode: i8 = 1;

// ---- misc -----------------------------------------------------------------
pub const IO_DISK_INCREMENT: i8 = 1; // CCHAR PriorityBoost
pub const MAXLONG: ULONG = 0x7FFF_FFFF;
pub const MAXUSHORT: ULONG = 0x0000_FFFF;
pub const UNICODE_NULL: u16 = 0;
pub const DISPATCH_LEVEL: u8 = 2;
// FILE_SYSTEM bugcheck code (documented Windows value 0x00000022).
pub const FILE_SYSTEM: ULONG = 0x0000_0022;

// ---- NTSTATUS (psdk/ntstatus.h, exact) ------------------------------------
pub const STATUS_SUCCESS: NTSTATUS = 0x0000_0000u32 as NTSTATUS;
pub const STATUS_PENDING: NTSTATUS = 0x0000_0103u32 as NTSTATUS;
pub const STATUS_REPARSE: NTSTATUS = 0x0000_0104u32 as NTSTATUS;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC000_0016u32 as NTSTATUS;
pub const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as NTSTATUS;
pub const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS;
pub const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;
pub const STATUS_NO_SUCH_FILE: NTSTATUS = 0xC000_000Fu32 as NTSTATUS;
pub const STATUS_INVALID_DEVICE_REQUEST: NTSTATUS = 0xC000_0010u32 as NTSTATUS;
pub const STATUS_INVALID_HANDLE: NTSTATUS = 0xC000_0008u32 as NTSTATUS;
pub const STATUS_INVALID_USER_BUFFER: NTSTATUS = 0xC000_00E8u32 as NTSTATUS;
pub const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
pub const STATUS_ACCESS_DENIED: NTSTATUS = 0xC000_0022u32 as NTSTATUS;
pub const STATUS_NAME_TOO_LONG: NTSTATUS = 0xC000_0106u32 as NTSTATUS;
pub const STATUS_BAD_NETWORK_PATH: NTSTATUS = 0xC000_00BEu32 as NTSTATUS;
pub const STATUS_BAD_NETWORK_NAME: NTSTATUS = 0xC000_00CCu32 as NTSTATUS;
pub const STATUS_REDIRECTOR_NOT_STARTED: NTSTATUS = 0xC000_00FBu32 as NTSTATUS;
pub const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC000_0023u32 as NTSTATUS;
