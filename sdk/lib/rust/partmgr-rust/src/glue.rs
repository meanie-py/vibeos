//! glue.rs — #define constants harvested from the C headers by
//! tools/rustconv/harvest_consts.py (REAL values, never LLM-guessed).
#![allow(non_upper_case_globals, dead_code)]

pub const DPFLTR_ERROR_LEVEL: u32 = 0;
pub const DPFLTR_INFO_LEVEL: u32 = 3;
pub const DPFLTR_WARNING_LEVEL: u32 = 1;
pub const FILE_DEVICE_MASS_STORAGE: u32 = 45;
pub const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC00000C0u32 as i32;
pub const TAG_PARTMGR: u32 = 0x4d747250;  // 'MtrP'

// CTL_CODE() IOCTLs — clang-computed exact values (bindgen --allowlist-var over the
// real mountdev.h/mountmgr.h/ntdddisk.h; NOT hand-derived). CTL_CODE(t,f,m,a) =
// (t<<16)|(a<<14)|(f<<2)|m, MOUNTDEVCONTROLTYPE='M'=0x4D, MOUNTMGRCONTROLTYPE='m'=0x6D,
// EXPECTED_ACCESS = FILE_READ_ACCESS|FILE_WRITE_ACCESS = 0x3.
pub const IOCTL_DISK_GET_DRIVE_GEOMETRY_EX: u32 = 0x000700A0;        // 458912
pub const IOCTL_MOUNTDEV_QUERY_STABLE_GUID: u32 = 0x004D0018;        // 5046296
pub const IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME: u32 = 0x004D000C; // 5046284
pub const IOCTL_MOUNTDEV_LINK_CREATED: u32 = 0x004DC010;            // 5095440
pub const IOCTL_MOUNTDEV_LINK_DELETED: u32 = 0x004DC014;            // 5095444
pub const IOCTL_MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY: u32 = 0x004DC004; // 5095428
pub const IOCTL_MOUNTMGR_DELETE_POINTS: u32 = 0x006DC004;           // 7192580
pub const MOUNTDEVCONTROLTYPE: u32 = 0x4D;                          // 'M'
pub const STATUS_DEVICE_REMOVED: i32 = 0xC00002B6u32 as i32;
// (*(ULONGLONG*)"DMIO:ID:") — the 8 ASCII bytes read as a little-endian u64.
pub const DMIO_ID_SIGNATURE: u64 = u64::from_le_bytes(*b"DMIO:ID:");
pub type size_t = usize;

// enum-variant aliases — bindgen prefixes C enum constants as _ENUM_Variant; the C
// source (and the LLM) reference the bare variant name. Re-export with exact bindgen
// type (all these enums are c_uint) so comparisons against the enum field type-check.
pub use ntkrnl_sys::ddk::{
    _PARTITION_STYLE_PARTITION_STYLE_MBR as PARTITION_STYLE_MBR,
    _PARTITION_STYLE_PARTITION_STYLE_GPT as PARTITION_STYLE_GPT,
    _PARTITION_STYLE_PARTITION_STYLE_RAW as PARTITION_STYLE_RAW,
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_TargetDeviceRelation as TargetDeviceRelation,
    _BUS_QUERY_ID_TYPE_BusQueryDeviceID as BusQueryDeviceID,
    _BUS_QUERY_ID_TYPE_BusQueryHardwareIDs as BusQueryHardwareIDs,
    _BUS_QUERY_ID_TYPE_BusQueryInstanceID as BusQueryInstanceID,
};
