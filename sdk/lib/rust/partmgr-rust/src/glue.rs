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

// --- IOCTLs used as match arms in the partition PDO dispatch (partition.rs) that had
//     NO definition -> bind-all catch-alls (IOCTL_DISK_GET_PARTITION_INFO swallowed all
//     later arms; sibling of the classpnp nest, 2026-07-06). CTL_CODE-resolved.
pub const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x00074004;     // CTL_CODE(7,0x01,BUFFERED,READ) [psdk/ntdddisk.h]
pub const IOCTL_DISK_GET_PARTITION_INFO_EX: u32 = 0x00070048;  // CTL_CODE(7,0x12,BUFFERED,ANY)
pub const IOCTL_DISK_SET_PARTITION_INFO: u32 = 0x0007C008;     // CTL_CODE(7,0x02,BUFFERED,RW)
pub const IOCTL_DISK_SET_PARTITION_INFO_EX: u32 = 0x0007C04C;  // CTL_CODE(7,0x13,BUFFERED,RW)
pub const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007405C;        // CTL_CODE(7,0x17,BUFFERED,READ)
pub const IOCTL_DISK_VERIFY: u32 = 0x00070014;                 // CTL_CODE(7,0x05,BUFFERED,ANY)
pub const IOCTL_DISK_UPDATE_PROPERTIES: u32 = 0x00070140;      // CTL_CODE(7,0x50,BUFFERED,ANY)
pub const IOCTL_STORAGE_GET_DEVICE_NUMBER: u32 = 0x002D1080;   // CTL_CODE(0x2D,0x420,BUFFERED,ANY)
pub const IOCTL_STORAGE_MEDIA_REMOVAL: u32 = 0x002D4804;       // CTL_CODE(0x2D,0x201,BUFFERED,READ)
pub const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x00560000; // CTL_CODE('V',0,BUFFERED,ANY) [psdk/ntddvol.h]
pub const IOCTL_VOLUME_ONLINE: u32 = 0x0056C008;               // CTL_CODE('V',2,BUFFERED,RW)
pub const IOCTL_VOLUME_QUERY_VOLUME_NUMBER: u32 = 0x0056001C;  // CTL_CODE('V',7,BUFFERED,ANY)
pub const IOCTL_VOLUME_IS_PARTITION: u32 = 0x00560028;         // CTL_CODE('V',10,BUFFERED,ANY)
pub const IOCTL_VOLUME_GET_GPT_ATTRIBUTES: u32 = 0x00560038;   // CTL_CODE('V',14,BUFFERED,ANY)
pub const IOCTL_MOUNTDEV_QUERY_UNIQUE_ID: u32 = 0x004D0000;    // CTL_CODE('M',0,BUFFERED,ANY) [ddk/mountdev.h]
pub const IOCTL_MOUNTDEV_QUERY_DEVICE_NAME: u32 = 0x004D0008;  // CTL_CODE('M',2,BUFFERED,ANY)
