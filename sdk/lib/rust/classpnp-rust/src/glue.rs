//! glue.rs — #define constants harvested from the C headers by
//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).
#![allow(non_upper_case_globals, dead_code)]

pub const DO_VERIFY_VOLUME: u32 = 0x00000002;
pub const DUID_HARDWARE_IDS_ONLY: u32 = 0;
pub const DUID_INCLUDE_SOFTWARE_IDS: u32 = 1;
pub const FILE_DEVICE_DISK: u32 = 7;
pub const FILE_PORTABLE_DEVICE: u32 = 0x00004000;
pub const IO_WRN_FAILURE_PREDICTED: i32 = 0x80040034u32 as i32;
pub const IRP_MJ_SCSI: u32 = 0x0f;
pub const KEY_ALL_ACCESS: u32 = 0xf003f;
pub const PLUGPLAY_REGKEY_DEVICE: u32 = 1;
pub const REG_OPTION_NON_VOLATILE: u32 = 0;
pub const RTL_QUERY_REGISTRY_TYPECHECK: u32 = 0x00000100;
pub const SL_OVERRIDE_VERIFY_VOLUME: u32 = 0x02;
pub const SL_WRITE_THROUGH: u32 = 0x04;
pub const STATUS_DATA_ERROR: i32 = 0xC000003Eu32 as i32;
pub const STATUS_DEVICE_BUSY: i32 = 0x80000011u32 as i32;
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32;
pub const STATUS_DEVICE_PROTOCOL_ERROR: i32 = 0xC0000186u32 as i32;
pub const STATUS_INVALID_DEVICE_STATE: i32 = 0xC0000184u32 as i32;
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
pub const STATUS_MEDIA_CHANGED: i32 = 0x8000001Cu32 as i32;
pub const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32;
pub const TRACE_LEVEL_ERROR: u32 = 2;
pub const TRACE_LEVEL_INFORMATION: u32 = 4;
pub const TRACE_LEVEL_WARNING: u32 = 3;
pub const VPB_MOUNTED: u32 = 0x0001;
