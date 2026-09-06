//! glue.rs — #define constants harvested from the C headers by
//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).
#![allow(non_upper_case_globals, dead_code)]

pub const IO_WARNING_WRITE_FUA_PROBLEM: i32 = 0x80040084u32 as i32;
pub const RTL_QUERY_REGISTRY_TYPECHECK: u32 = 0x00000100;
pub const SCSI_SENSE_OPTIONS_FIXED_FORMAT_IF_UNKNOWN_FORMAT_INDICATED: u32 = 0x00000001;
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32;
pub const STATUS_DEVICE_NOT_READY: i32 = 0xC00000A3u32 as i32;
pub const STATUS_INTERNAL_ERROR: i32 = 0xC00000E5u32 as i32;
pub const STATUS_IO_TIMEOUT: i32 = 0xC00000B5u32 as i32;
pub const STATUS_UNRECOGNIZED_MEDIA: i32 = 0xC0000014u32 as i32;
pub const STATUS_WRONG_VOLUME: i32 = 0xC0000012u32 as i32;
pub const WMIREG_ACTION_UPDATE_GUIDS: u32 = 4;

// --- computed #defines (CTL_CODE / FourCC pool tags / sizeof) hand-derived from
//     the C headers (harvest_consts can't eval CTL_CODE/sizeof). ---
use crate::types::{DISK_DATA, FUNCTIONAL_DEVICE_EXTENSION, SCSI_REQUEST_BLOCK, SENSE_DATA_EX,
    STORAGE_REQUEST_BLOCK, STOR_ADDR_BTL8, SRBEX_DATA_SCSI_CDB16};

// CTL_CODE(IOCTL_DISK_BASE=FILE_DEVICE_DISK(0x7), 0, METHOD_BUFFERED(0), FILE_ANY_ACCESS(0))
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY: u32 = 0x00070000;

// storswtr.h WPP_DEFINE_BIT sequential bit positions (GENERAL=1, PNP=2, POWER=3,
// RW=4, IOCTL=5, QUEUE=6, WMI=7). Debug-trace flags only.
pub const TRACE_FLAG_IOCTL: u32 = 4; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_PNP: u32 = 1; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)

// FourCC pool tags ('ADcS' / 'SDcS'), C multichar value (MSB-first).
pub const DISK_TAG_INFO_EXCEPTION: u32 = 0x41446353; // 'ADcS'
pub const DISK_TAG_SRB: u32 = 0x53446353;            // 'SDcS'

// POOL_TYPE: NonPagedPoolNx(512) | CacheAligned(4)
// disk.h:75: `#define NonPagedPoolNxCacheAligned NonPagedPoolCacheAligned` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
pub const NonPagedPoolNxCacheAligned: i32 = 4;

// sizeof-based #defines.
// C: #define FUNCTIONAL_EXTENSION_SIZE sizeof(FUNCTIONAL_DEVICE_EXTENSION) + sizeof(DISK_DATA)
// The `+ sizeof(DISK_DATA)` term was DROPPED in the original translation: disk then
// registered a device extension with no room for DISK_DATA, classpnp packed its
// PrivateCommonData + remove-lock rundown right after the FDE, and disk's first
// DISK_DATA.Mbr.{Signature,CheckSum} write smashed the rundown's RunRefs pointer —
// AV on the next remove-lock acquire with the disk's MBR signature IN the fault
// register (USB-disk START, 2026-07-06).
pub const FUNCTIONAL_EXTENSION_SIZE: u32 =
    (core::mem::size_of::<FUNCTIONAL_DEVICE_EXTENSION>() + core::mem::size_of::<DISK_DATA>()) as u32;
pub const SCSI_REQUEST_BLOCK_SIZE: u32 = core::mem::size_of::<SCSI_REQUEST_BLOCK>() as u32;
pub const SENSE_BUFFER_SIZE_EX: u32 = core::mem::size_of::<SENSE_DATA_EX>() as u32;
// (20*sizeof(UCHAR)) + sizeof(ULONG) + sizeof(PVOID)
pub const SRBEX_DATA_SCSI_CDB16_LENGTH: u32 = (20 + 4 + core::mem::size_of::<*mut core::ffi::c_void>()) as u32;
pub const CLASS_SRBEX_SCSI_CDB16_BUFFER_SIZE: u32 = (core::mem::size_of::<STORAGE_REQUEST_BLOCK>()
    + core::mem::size_of::<STOR_ADDR_BTL8>()
    + core::mem::size_of::<SRBEX_DATA_SCSI_CDB16>()) as u32;

// WORK_QUEUE_TYPE enum value (ntkrnl_sys::ddk has it prefixed).
pub const DelayedWorkQueue: ntkrnl_sys::ddk::_WORK_QUEUE_TYPE = ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue;

// harvested:
pub const FILE_DEVICE_DISK: u32 = 7;
pub const IO_WRITE_CACHE_DISABLED: i32 = 0x80040022u32 as i32;
pub const IO_WRITE_CACHE_ENABLED: i32 = 0x80040020u32 as i32;
pub const IRP_MJ_SCSI: u32 = 0x0f;
pub const STATUS_IO_DEVICE_ERROR: i32 = 0xC0000185u32 as i32;
pub const STATUS_NONEXISTENT_SECTOR: i32 = 0xC0000015u32 as i32;
pub const STATUS_VERIFY_REQUIRED: i32 = 0x80000016u32 as i32;
pub const TRACE_LEVEL_ERROR: u32 = 2; // storswtr.h:26-30
pub const TRACE_LEVEL_INFORMATION: u32 = 4; // storswtr.h:26-30
pub const TRACE_LEVEL_VERBOSE: u32 = 5; // storswtr.h:26-30

// storswtr.h WPP bit positions (continued) + disk.h FourCC pool tags + pool type.
pub const TRACE_FLAG_GENERAL: u32 = 0; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_WMI: u32 = 6; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const TRACE_FLAG_SCSI: u32 = 17; // storswtr.h:80-98 (an ORDINAL enum, not a bitmask)
pub const DISK_TAG_DISABLE_CACHE: u32 = 0x43446353; // 'CDcS'
pub const DISK_TAG_CCONTEXT: u32 = 0x63446353;      // 'cDcS'
// disk.h:75: `#define NonPagedPoolNx NonPagedPool` under
// `#ifdef __REACTOS__` -- NX pool is NT6.2 and this kernel has none, so the
// Windows value (0x200/0x204) is not what any TU here compiles.
pub const NonPagedPoolNx: i32 = 0;

// harvested:
pub const DO_VERIFY_VOLUME: u32 = 0x00000002;
pub const STATUS_INVALID_LEVEL: i32 = 0xC0000148u32 as i32;
