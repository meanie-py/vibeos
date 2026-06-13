//! glue.rs — mountmgr-specific constants the translation references that aren't in
//! ntkrnl-sys (pool tags, ioctl codes, status values). Populated as cargo check surfaces them.
#![allow(non_upper_case_globals, dead_code, unused_imports)]
use ntkrnl_sys::ddk::*;

// harvested:
pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
pub const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
pub const FILE_DEVICE_DISK: u32 = 7;
pub const FILE_DEVICE_VIRTUAL_DISK: u32 = 36;
pub const FILE_REMOTE_DEVICE: u32 = 0x00000010;

// CTL_CODE() ioctls (computed; CTL_CODE = (dt<<16)|(acc<<14)|(fn<<2)|mth):
pub const IOCTL_DISK_GET_PARTITION_INFO: u32 = 0x74004; // CTL_CODE(IOCTL_DISK_BASE=7, 1, BUFFERED, READ)
pub const IOCTL_MOUNTDEV_QUERY_SUGGESTED_LINK_NAME: u32 = 0x4d000c; // CTL_CODE('M', 3, BUFFERED, ANY)

// I/O event GUIDs (DEFINE_GUID from ioevent.h / wdmguid.h) — mountmgr compares
// PnP TargetDeviceChange custom-notification GUIDs against these.
pub static GUID_IO_VOLUME_MOUNT: GUID = GUID {
    Data1: 0xB5804878, Data2: 0x1A96, Data3: 0x11D2,
    Data4: [0x8F, 0xFD, 0x00, 0xA0, 0xC9, 0xA0, 0x6D, 0x32] };
pub static GUID_IO_VOLUME_NAME_CHANGE: GUID = GUID {
    Data1: 0x2DE97F83, Data2: 0x4C06, Data3: 0x11D2,
    Data4: [0xA5, 0x32, 0x00, 0x60, 0x97, 0x13, 0x05, 0x5A] };
pub static GUID_TARGET_DEVICE_REMOVE_COMPLETE: GUID = GUID {
    Data1: 0xcb3a4008, Data2: 0x46f0, Data3: 0x11d0,
    Data4: [0xb0, 0x8f, 0x00, 0x60, 0x97, 0x13, 0x05, 0x3f] };
// CTL_CODE('M', 1, METHOD_BUFFERED, FILE_READ|WRITE_ACCESS) — NTDDI>=WS03 EXPECTED_ACCESS=3
pub const IOCTL_MOUNTDEV_UNIQUE_ID_CHANGE_NOTIFY: u32 = 0x4dc004;
