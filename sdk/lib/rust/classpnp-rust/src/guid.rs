// @generated: c2rust file-scope data (compiler-exact). Pure data file.
#![allow(non_upper_case_globals, non_camel_case_types, dead_code)]
use crate::types::*;
use ntkrnl_sys::driver_ddk::*;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{GUID};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{_GUID};

pub static mut GUID_IO_VOLUME_CHANGE: GUID = _GUID {
    Data1: 0x7373654a as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x812a as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_DISMOUNT: GUID = _GUID {
    Data1: 0xd16a55e8 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x1059 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_DISMOUNT_FAILED: GUID = _GUID {
    Data1: 0xe3c5b178 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x105d as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_MOUNT: GUID = _GUID {
    Data1: 0xb5804878 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x1a96 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_LOCK: GUID = _GUID {
    Data1: 0x50708874 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xc9af as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_LOCK_FAILED: GUID = _GUID {
    Data1: 0xae2eed10 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xba8 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_UNLOCK: GUID = _GUID {
    Data1: 0x9a8c3d68 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xd0cb as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_NAME_CHANGE: GUID = _GUID {
    Data1: 0x2de97f83 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4c06 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_PREPARING_EJECT: GUID = _GUID {
    Data1: 0xc79eb16e as ::core::ffi::c_uint,
    Data2: 0xdac as ::core::ffi::c_ushort,
    Data3: 0x4e7a as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_PHYSICAL_CONFIGURATION_CHANGE: GUID = _GUID {
    Data1: 0x2de97f84 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4c06 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_FVE_STATUS_CHANGE: GUID = _GUID {
    Data1: 0x62998b2 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xee1f as ::core::ffi::c_ushort,
    Data3: 0x4b6a as ::core::ffi::c_ushort,
    Data4: [
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_DEVICE_INTERFACE: GUID = _GUID {
    Data1: 0x53f5630d as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb6bf as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_VOLUME_CHANGE_SIZE: GUID = _GUID {
    Data1: 0x3a1625be as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xad03 as ::core::ffi::c_ushort,
    Data3: 0x49f1 as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_MEDIA_ARRIVAL: GUID = _GUID {
    Data1: 0xd07433c0 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_MEDIA_REMOVAL: GUID = _GUID {
    Data1: 0xd07433c1 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_CDROM_EXCLUSIVE_LOCK: GUID = _GUID {
    Data1: 0xbc56c139 as ::core::ffi::c_uint,
    Data2: 0x7a10 as ::core::ffi::c_ushort,
    Data3: 0x47ee as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_CDROM_EXCLUSIVE_UNLOCK: GUID = _GUID {
    Data1: 0xa3b6d27d as ::core::ffi::c_uint,
    Data2: 0x5e35 as ::core::ffi::c_ushort,
    Data3: 0x4885 as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DEVICE_BECOMING_READY: GUID = _GUID {
    Data1: 0xd07433f0 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DEVICE_EXTERNAL_REQUEST: GUID = _GUID {
    Data1: 0xd07433d0 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_MEDIA_EJECT_REQUEST: GUID = _GUID {
    Data1: 0xd07433d1 as ::core::ffi::c_uint,
    Data2: 0xa98e as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DRIVE_REQUIRES_CLEANING: GUID = _GUID {
    Data1: 0x7207877c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x90ed as ::core::ffi::c_ushort,
    Data3: 0x44e5 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_TAPE_ERASE: GUID = _GUID {
    Data1: 0x852d11eb as ::core::ffi::c_uint,
    Data2: 0x4bb8 as ::core::ffi::c_ushort,
    Data3: 0x4507 as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DISK_CLONE_ARRIVAL: GUID = _GUID {
    Data1: 0x6a61885b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7c39 as ::core::ffi::c_ushort,
    Data3: 0x43dd as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IO_DISK_LAYOUT_CHANGE: GUID = _GUID {
    Data1: 0x11dff54c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8469 as ::core::ffi::c_ushort,
    Data3: 0x41f9 as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_MAX_POWER_SAVINGS: GUID = _GUID {
    Data1: 0xa1841308 as ::core::ffi::c_uint,
    Data2: 0x3541 as ::core::ffi::c_ushort,
    Data3: 0x4fab as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_MIN_POWER_SAVINGS: GUID = _GUID {
    Data1: 0x8c5e7fda as ::core::ffi::c_uint,
    Data2: 0xe8bf as ::core::ffi::c_ushort,
    Data3: 0x4a96 as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_TYPICAL_POWER_SAVINGS: GUID = _GUID {
    Data1: 0x381b4222 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf694 as ::core::ffi::c_ushort,
    Data3: 0x41f0 as ::core::ffi::c_ushort,
    Data4: [
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut NO_SUBGROUP_GUID: GUID = _GUID {
    Data1: 0xfea3413e as ::core::ffi::c_uint,
    Data2: 0x7e05 as ::core::ffi::c_ushort,
    Data3: 0x4911 as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut ALL_POWERSCHEMES_GUID: GUID = _GUID {
    Data1: 0x68a1e95e as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x13ea as ::core::ffi::c_ushort,
    Data3: 0x41e1 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_POWERSCHEME_PERSONALITY: GUID = _GUID {
    Data1: 0x245d8541 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x3943 as ::core::ffi::c_ushort,
    Data3: 0x4422 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ACTIVE_POWERSCHEME: GUID = _GUID {
    Data1: 0x31f9f286 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5084 as ::core::ffi::c_ushort,
    Data3: 0x42fe as ::core::ffi::c_ushort,
    Data4: [
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_SUBGROUP: GUID = _GUID {
    Data1: 0x7516b95f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf776 as ::core::ffi::c_ushort,
    Data3: 0x4464 as ::core::ffi::c_ushort,
    Data4: [
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_POWERDOWN_TIMEOUT: GUID = _GUID {
    Data1: 0x3c0bc021 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc8a8 as ::core::ffi::c_ushort,
    Data3: 0x4e07 as ::core::ffi::c_ushort,
    Data4: [
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_ANNOYANCE_TIMEOUT: GUID = _GUID {
    Data1: 0x82dbcf2d as ::core::ffi::c_uint,
    Data2: 0xcd67 as ::core::ffi::c_ushort,
    Data3: 0x40c5 as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_ADAPTIVE_PERCENT_INCREASE: GUID = _GUID {
    Data1: 0xeed904df as ::core::ffi::c_uint,
    Data2: 0xb142 as ::core::ffi::c_ushort,
    Data3: 0x4183 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_DIM_TIMEOUT: GUID = _GUID {
    Data1: 0x17aaa29b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8b43 as ::core::ffi::c_ushort,
    Data3: 0x4b94 as ::core::ffi::c_ushort,
    Data4: [
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_ADAPTIVE_POWERDOWN: GUID = _GUID {
    Data1: 0x90959d22 as ::core::ffi::c_uint,
    Data2: 0xd6a1 as ::core::ffi::c_ushort,
    Data3: 0x49b9 as ::core::ffi::c_ushort,
    Data4: [
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_MONITOR_POWER_ON: GUID = _GUID {
    Data1: 0x2731015 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4510 as ::core::ffi::c_ushort,
    Data3: 0x4526 as ::core::ffi::c_ushort,
    Data4: [
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_POWER_POLICY_VIDEO_BRIGHTNESS: GUID = _GUID {
    Data1: 0xaded5e82 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xb909 as ::core::ffi::c_ushort,
    Data3: 0x4619 as ::core::ffi::c_ushort,
    Data4: [
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_POWER_POLICY_VIDEO_DIM_BRIGHTNESS: GUID = _GUID {
    Data1: 0xf1fbfde2 as ::core::ffi::c_uint,
    Data2: 0xa960 as ::core::ffi::c_ushort,
    Data3: 0x4165 as ::core::ffi::c_ushort,
    Data4: [
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_CURRENT_MONITOR_BRIGHTNESS: GUID = _GUID {
    Data1: 0x8ffee2c6 as ::core::ffi::c_uint,
    Data2: 0x2d01 as ::core::ffi::c_ushort,
    Data3: 0x46be as ::core::ffi::c_ushort,
    Data4: [
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VIDEO_ADAPTIVE_DISPLAY_BRIGHTNESS: GUID = _GUID {
    Data1: 0xfbd9aa66 as ::core::ffi::c_uint,
    Data2: 0x9553 as ::core::ffi::c_ushort,
    Data3: 0x4097 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SESSION_DISPLAY_STATE: GUID = _GUID {
    Data1: 0x73a5e93a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5bb1 as ::core::ffi::c_ushort,
    Data3: 0x4f93 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_CONSOLE_DISPLAY_STATE: GUID = _GUID {
    Data1: 0x6fe69556 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x704a as ::core::ffi::c_ushort,
    Data3: 0x47a0 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ALLOW_DISPLAY_REQUIRED: GUID = _GUID {
    Data1: 0xa9ceb8da as ::core::ffi::c_uint,
    Data2: 0xcd46 as ::core::ffi::c_ushort,
    Data3: 0x44fb as ::core::ffi::c_ushort,
    Data4: [
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DISK_SUBGROUP: GUID = _GUID {
    Data1: 0x12ee47 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x9041 as ::core::ffi::c_ushort,
    Data3: 0x4b5d as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DISK_POWERDOWN_TIMEOUT: GUID = _GUID {
    Data1: 0x6738e2c4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe8a5 as ::core::ffi::c_ushort,
    Data3: 0x4a42 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DISK_IDLE_TIMEOUT: GUID = _GUID {
    Data1: 0x58e39ba8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb8e6 as ::core::ffi::c_ushort,
    Data3: 0x4ef6 as ::core::ffi::c_ushort,
    Data4: [
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DISK_BURST_IGNORE_THRESHOLD: GUID = _GUID {
    Data1: 0x80e3c60e as ::core::ffi::c_uint,
    Data2: 0xbb94 as ::core::ffi::c_ushort,
    Data3: 0x4ad8 as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DISK_ADAPTIVE_POWERDOWN: GUID = _GUID {
    Data1: 0x396a32e1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x499a as ::core::ffi::c_ushort,
    Data3: 0x40b2 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SLEEP_SUBGROUP: GUID = _GUID {
    Data1: 0x238c9fa8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xaad as ::core::ffi::c_ushort,
    Data3: 0x41ed as ::core::ffi::c_ushort,
    Data4: [
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SLEEP_IDLE_THRESHOLD: GUID = _GUID {
    Data1: 0x81cd32e0 as ::core::ffi::c_uint,
    Data2: 0x7833 as ::core::ffi::c_ushort,
    Data3: 0x44f3 as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_STANDBY_TIMEOUT: GUID = _GUID {
    Data1: 0x29f6c1db as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x86da as ::core::ffi::c_ushort,
    Data3: 0x48c5 as ::core::ffi::c_ushort,
    Data4: [
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_UNATTEND_SLEEP_TIMEOUT: GUID = _GUID {
    Data1: 0x7bc4a2f9 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xd8fc as ::core::ffi::c_ushort,
    Data3: 0x4469 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_HIBERNATE_TIMEOUT: GUID = _GUID {
    Data1: 0x9d7815a6 as ::core::ffi::c_uint,
    Data2: 0x7ee4 as ::core::ffi::c_ushort,
    Data3: 0x497e as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_HIBERNATE_FASTS4_POLICY: GUID = _GUID {
    Data1: 0x94ac6d29 as ::core::ffi::c_uint,
    Data2: 0x73ce as ::core::ffi::c_ushort,
    Data3: 0x41a6 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_CRITICAL_POWER_TRANSITION: GUID = _GUID {
    Data1: 0xb7a27025 as ::core::ffi::c_uint,
    Data2: 0xe569 as ::core::ffi::c_ushort,
    Data3: 0x46c2 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SYSTEM_AWAYMODE: GUID = _GUID {
    Data1: 0x98a7f580 as ::core::ffi::c_uint,
    Data2: 0x1f7 as ::core::ffi::c_ushort,
    Data3: 0x48aa as ::core::ffi::c_ushort,
    Data4: [
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ALLOW_AWAYMODE: GUID = _GUID {
    Data1: 0x25dfa149 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5dd1 as ::core::ffi::c_ushort,
    Data3: 0x4736 as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ALLOW_STANDBY_STATES: GUID = _GUID {
    Data1: 0xabfc2519 as ::core::ffi::c_uint,
    Data2: 0x3608 as ::core::ffi::c_ushort,
    Data3: 0x4c2a as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ALLOW_RTC_WAKE: GUID = _GUID {
    Data1: 0xbd3b718a as ::core::ffi::c_uint,
    Data2: 0x680 as ::core::ffi::c_ushort,
    Data3: 0x4d9d as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ALLOW_SYSTEM_REQUIRED: GUID = _GUID {
    Data1: 0xa4b195f5 as ::core::ffi::c_uint,
    Data2: 0x8225 as ::core::ffi::c_ushort,
    Data3: 0x47d8 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SYSTEM_BUTTON_SUBGROUP: GUID = _GUID {
    Data1: 0x4f971e89 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xeebd as ::core::ffi::c_ushort,
    Data3: 0x4455 as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_POWERBUTTON_ACTION: GUID = _GUID {
    Data1: 0x7648efa3 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xdd9c as ::core::ffi::c_ushort,
    Data3: 0x4e3e as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_POWERBUTTON_ACTION_FLAGS: GUID = _GUID {
    Data1: 0x857e7fac as ::core::ffi::c_uint,
    Data2: 0x34b as ::core::ffi::c_ushort,
    Data3: 0x4704 as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SLEEPBUTTON_ACTION: GUID = _GUID {
    Data1: 0x96996bc0 as ::core::ffi::c_uint,
    Data2: 0xad50 as ::core::ffi::c_ushort,
    Data3: 0x47ec as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SLEEPBUTTON_ACTION_FLAGS: GUID = _GUID {
    Data1: 0x2a160ab1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb69d as ::core::ffi::c_ushort,
    Data3: 0x4743 as ::core::ffi::c_ushort,
    Data4: [
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_USERINTERFACEBUTTON_ACTION: GUID = _GUID {
    Data1: 0xa7066653 as ::core::ffi::c_uint,
    Data2: 0x8d6c as ::core::ffi::c_ushort,
    Data3: 0x40a8 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_LIDCLOSE_ACTION: GUID = _GUID {
    Data1: 0x5ca83367 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6e45 as ::core::ffi::c_ushort,
    Data3: 0x459f as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_LIDCLOSE_ACTION_FLAGS: GUID = _GUID {
    Data1: 0x97e969ac as ::core::ffi::c_uint,
    Data2: 0xd6c as ::core::ffi::c_ushort,
    Data3: 0x4d08 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_LIDOPEN_POWERSTATE: GUID = _GUID {
    Data1: 0x99ff10e7 as ::core::ffi::c_uint,
    Data2: 0x23b1 as ::core::ffi::c_ushort,
    Data3: 0x4c07 as ::core::ffi::c_ushort,
    Data4: [
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_SUBGROUP: GUID = _GUID {
    Data1: 0xe73a048d as ::core::ffi::c_uint,
    Data2: 0xbf27 as ::core::ffi::c_ushort,
    Data3: 0x4f12 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_ACTION_0: GUID = _GUID {
    Data1: 0x637ea02f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xbbcb as ::core::ffi::c_ushort,
    Data3: 0x4015 as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_LEVEL_0: GUID = _GUID {
    Data1: 0x9a66d8d7 as ::core::ffi::c_uint,
    Data2: 0x4ff7 as ::core::ffi::c_ushort,
    Data3: 0x4ef9 as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_FLAGS_0: GUID = _GUID {
    Data1: 0x5dbb7c9f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x38e9 as ::core::ffi::c_ushort,
    Data3: 0x40d2 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_ACTION_1: GUID = _GUID {
    Data1: 0xd8742dcb as ::core::ffi::c_uint,
    Data2: 0x3e6a as ::core::ffi::c_ushort,
    Data3: 0x4b3c as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_LEVEL_1: GUID = _GUID {
    Data1: 0x8183ba9a as ::core::ffi::c_uint,
    Data2: 0xe910 as ::core::ffi::c_ushort,
    Data3: 0x48da as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_FLAGS_1: GUID = _GUID {
    Data1: 0xbcded951 as ::core::ffi::c_uint,
    Data2: 0x187b as ::core::ffi::c_ushort,
    Data3: 0x4d05 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_ACTION_2: GUID = _GUID {
    Data1: 0x421cba38 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1a8e as ::core::ffi::c_ushort,
    Data3: 0x4881 as ::core::ffi::c_ushort,
    Data4: [
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_LEVEL_2: GUID = _GUID {
    Data1: 0x7a07ca2 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xadaf as ::core::ffi::c_ushort,
    Data3: 0x40d7 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_FLAGS_2: GUID = _GUID {
    Data1: 0x7fd2f0c4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xfeb7 as ::core::ffi::c_ushort,
    Data3: 0x4da3 as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_ACTION_3: GUID = _GUID {
    Data1: 0x80472613 as ::core::ffi::c_uint,
    Data2: 0x9780 as ::core::ffi::c_ushort,
    Data3: 0x455e as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_LEVEL_3: GUID = _GUID {
    Data1: 0x58afd5a6 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc2dd as ::core::ffi::c_ushort,
    Data3: 0x47d2 as ::core::ffi::c_ushort,
    Data4: [
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_DISCHARGE_FLAGS_3: GUID = _GUID {
    Data1: 0x73613ccf as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xdbfa as ::core::ffi::c_ushort,
    Data3: 0x4279 as ::core::ffi::c_ushort,
    Data4: [
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_SETTINGS_SUBGROUP: GUID = _GUID {
    Data1: 0x54533251 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x82be as ::core::ffi::c_ushort,
    Data3: 0x4824 as ::core::ffi::c_ushort,
    Data4: [
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_THROTTLE_POLICY: GUID = _GUID {
    Data1: 0x57027304 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4af6 as ::core::ffi::c_ushort,
    Data3: 0x4104 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_THROTTLE_MAXIMUM: GUID = _GUID {
    Data1: 0xbc5038f7 as ::core::ffi::c_uint,
    Data2: 0x23e0 as ::core::ffi::c_ushort,
    Data3: 0x4960 as ::core::ffi::c_ushort,
    Data4: [
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_THROTTLE_MINIMUM: GUID = _GUID {
    Data1: 0x893dee8e as ::core::ffi::c_uint,
    Data2: 0x2bef as ::core::ffi::c_ushort,
    Data3: 0x41e0 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_ALLOW_THROTTLING: GUID = _GUID {
    Data1: 0x3b04d4fd as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1cc7 as ::core::ffi::c_ushort,
    Data3: 0x4f23 as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_IDLESTATE_POLICY: GUID = _GUID {
    Data1: 0x68f262a7 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf621 as ::core::ffi::c_ushort,
    Data3: 0x4069 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERFSTATE_POLICY: GUID = _GUID {
    Data1: 0xbbdc3814 as ::core::ffi::c_uint,
    Data2: 0x18e9 as ::core::ffi::c_ushort,
    Data3: 0x4463 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_INCREASE_THRESHOLD: GUID = _GUID {
    Data1: 0x6cadf0e as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x64ed as ::core::ffi::c_ushort,
    Data3: 0x448a as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_DECREASE_THRESHOLD: GUID = _GUID {
    Data1: 0x12a0ab44 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xfe28 as ::core::ffi::c_ushort,
    Data3: 0x4fa9 as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_INCREASE_POLICY: GUID = _GUID {
    Data1: 0x465e1f50 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb610 as ::core::ffi::c_ushort,
    Data3: 0x473a as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_DECREASE_POLICY: GUID = _GUID {
    Data1: 0x40fbefc7 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x2e9d as ::core::ffi::c_ushort,
    Data3: 0x4d25 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_INCREASE_TIME: GUID = _GUID {
    Data1: 0x984cf492 as ::core::ffi::c_uint,
    Data2: 0x3bed as ::core::ffi::c_ushort,
    Data3: 0x4488 as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_DECREASE_TIME: GUID = _GUID {
    Data1: 0xd8edeb9b as ::core::ffi::c_uint,
    Data2: 0x95cf as ::core::ffi::c_ushort,
    Data3: 0x4f95 as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_TIME_CHECK: GUID = _GUID {
    Data1: 0x4d2b0152 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7d5c as ::core::ffi::c_ushort,
    Data3: 0x498b as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_BOOST_POLICY: GUID = _GUID {
    Data1: 0x45bcc044 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xd885 as ::core::ffi::c_ushort,
    Data3: 0x43e2 as ::core::ffi::c_ushort,
    Data4: [
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_IDLE_ALLOW_SCALING: GUID = _GUID {
    Data1: 0x6c2993b0 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8f48 as ::core::ffi::c_ushort,
    Data3: 0x481f as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_IDLE_DISABLE: GUID = _GUID {
    Data1: 0x5d76a2ca as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe8c0 as ::core::ffi::c_ushort,
    Data3: 0x402f as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_IDLE_TIME_CHECK: GUID = _GUID {
    Data1: 0xc4581c31 as ::core::ffi::c_uint,
    Data2: 0x89ab as ::core::ffi::c_ushort,
    Data3: 0x4597 as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_IDLE_DEMOTE_THRESHOLD: GUID = _GUID {
    Data1: 0x4b92d758 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5a24 as ::core::ffi::c_ushort,
    Data3: 0x4851 as ::core::ffi::c_ushort,
    Data4: [
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_IDLE_PROMOTE_THRESHOLD: GUID = _GUID {
    Data1: 0x7b224883 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb3cc as ::core::ffi::c_ushort,
    Data3: 0x4d79 as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_INCREASE_THRESHOLD: GUID = _GUID {
    Data1: 0xdf142941 as ::core::ffi::c_uint,
    Data2: 0x20f3 as ::core::ffi::c_ushort,
    Data3: 0x4edf as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_DECREASE_THRESHOLD: GUID = _GUID {
    Data1: 0x68dd2f27 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xa4ce as ::core::ffi::c_ushort,
    Data3: 0x4e11 as ::core::ffi::c_ushort,
    Data4: [
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_INCREASE_POLICY: GUID = _GUID {
    Data1: 0xc7be0679 as ::core::ffi::c_uint,
    Data2: 0x2817 as ::core::ffi::c_ushort,
    Data3: 0x4d69 as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_DECREASE_POLICY: GUID = _GUID {
    Data1: 0x71021b41 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc749 as ::core::ffi::c_ushort,
    Data3: 0x4d21 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_MAX_CORES: GUID = _GUID {
    Data1: 0xea062031 as ::core::ffi::c_uint,
    Data2: 0xe34 as ::core::ffi::c_ushort,
    Data3: 0x4ff1 as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_MIN_CORES: GUID = _GUID {
    Data1: 0xcc5b647 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc1df as ::core::ffi::c_ushort,
    Data3: 0x4637 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_INCREASE_TIME: GUID = _GUID {
    Data1: 0x2ddd5a84 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5a71 as ::core::ffi::c_ushort,
    Data3: 0x437e as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_DECREASE_TIME: GUID = _GUID {
    Data1: 0xdfd10d17 as ::core::ffi::c_uint,
    Data2: 0xd5eb as ::core::ffi::c_ushort,
    Data3: 0x45dd as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_AFFINITY_HISTORY_DECREASE_FACTOR: GUID = _GUID {
    Data1: 0x8f7b45e3 as ::core::ffi::c_uint,
    Data2: 0xc393 as ::core::ffi::c_ushort,
    Data3: 0x480a as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_AFFINITY_HISTORY_THRESHOLD: GUID = _GUID {
    Data1: 0x5b33697b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe89d as ::core::ffi::c_ushort,
    Data3: 0x4d38 as ::core::ffi::c_ushort,
    Data4: [
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_AFFINITY_WEIGHTING: GUID = _GUID {
    Data1: 0xe70867f1 as ::core::ffi::c_uint,
    Data2: 0xfa2f as ::core::ffi::c_ushort,
    Data3: 0x4f4e as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_OVER_UTILIZATION_HISTORY_DECREASE_FACTOR: GUID = _GUID {
    Data1: 0x1299023c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xbc28 as ::core::ffi::c_ushort,
    Data3: 0x4f0a as ::core::ffi::c_ushort,
    Data4: [
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_OVER_UTILIZATION_HISTORY_THRESHOLD: GUID = _GUID {
    Data1: 0x9ac18e92 as ::core::ffi::c_uint,
    Data2: 0xaa3c as ::core::ffi::c_ushort,
    Data3: 0x4e27 as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_OVER_UTILIZATION_WEIGHTING: GUID = _GUID {
    Data1: 0x8809c2d8 as ::core::ffi::c_uint,
    Data2: 0xb155 as ::core::ffi::c_ushort,
    Data3: 0x42d4 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_CORE_PARKING_OVER_UTILIZATION_THRESHOLD: GUID = _GUID {
    Data1: 0x943c8cb6 as ::core::ffi::c_uint,
    Data2: 0x6f93 as ::core::ffi::c_ushort,
    Data3: 0x4227 as ::core::ffi::c_ushort,
    Data4: [
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xec as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PARKING_CORE_OVERRIDE: GUID = _GUID {
    Data1: 0xa55612aa as ::core::ffi::c_uint,
    Data2: 0xf624 as ::core::ffi::c_ushort,
    Data3: 0x42c6 as ::core::ffi::c_ushort,
    Data4: [
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PARKING_PERF_STATE: GUID = _GUID {
    Data1: 0x447235c7 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6a8d as ::core::ffi::c_ushort,
    Data3: 0x4cc0 as ::core::ffi::c_ushort,
    Data4: [
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PROCESSOR_PERF_HISTORY: GUID = _GUID {
    Data1: 0x7d24baa7 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb84 as ::core::ffi::c_ushort,
    Data3: 0x480f as ::core::ffi::c_ushort,
    Data4: [
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_SYSTEM_COOLING_POLICY: GUID = _GUID {
    Data1: 0x94d3a615 as ::core::ffi::c_uint,
    Data2: 0xa899 as ::core::ffi::c_ushort,
    Data3: 0x4ac5 as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_LOCK_CONSOLE_ON_WAKE: GUID = _GUID {
    Data1: 0xe796bdb as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x100d as ::core::ffi::c_ushort,
    Data3: 0x47d6 as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_DEVICE_IDLE_POLICY: GUID = _GUID {
    Data1: 0x4faab71a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x92e5 as ::core::ffi::c_ushort,
    Data3: 0x4726 as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x59 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ACDC_POWER_SOURCE: GUID = _GUID {
    Data1: 0x5d3e9a59 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe9d5 as ::core::ffi::c_ushort,
    Data3: 0x4b00 as ::core::ffi::c_ushort,
    Data4: [
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_LIDSWITCH_STATE_CHANGE: GUID = _GUID {
    Data1: 0xba3e0f4d as ::core::ffi::c_uint,
    Data2: 0xb817 as ::core::ffi::c_ushort,
    Data3: 0x4094 as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BATTERY_PERCENTAGE_REMAINING: GUID = _GUID {
    Data1: 0xa7ad8041 as ::core::ffi::c_uint,
    Data2: 0xb45a as ::core::ffi::c_ushort,
    Data3: 0x4cae as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_IDLE_BACKGROUND_TASK: GUID = _GUID {
    Data1: 0x515c31d8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf734 as ::core::ffi::c_ushort,
    Data3: 0x163d as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_BACKGROUND_TASK_NOTIFICATION: GUID = _GUID {
    Data1: 0xcf23f240 as ::core::ffi::c_uint,
    Data2: 0x2a54 as ::core::ffi::c_ushort,
    Data3: 0x48d8 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_APPLAUNCH_BUTTON: GUID = _GUID {
    Data1: 0x1a689231 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7399 as ::core::ffi::c_ushort,
    Data3: 0x4e9a as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCIEXPRESS_SETTINGS_SUBGROUP: GUID = _GUID {
    Data1: 0x501a4d13 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x42af as ::core::ffi::c_ushort,
    Data3: 0x4429 as ::core::ffi::c_ushort,
    Data4: [
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCIEXPRESS_ASPM_POLICY: GUID = _GUID {
    Data1: 0xee12f906 as ::core::ffi::c_uint,
    Data2: 0xd277 as ::core::ffi::c_ushort,
    Data3: 0x404b as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ENABLE_SWITCH_FORCED_SHUTDOWN: GUID = _GUID {
    Data1: 0x833a6b62 as ::core::ffi::c_uint,
    Data2: 0xdfa4 as ::core::ffi::c_ushort,
    Data3: 0x46d1 as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCI_CARDBUS_INTERFACE_PRIVATE: GUID = _GUID {
    Data1: 0xcca82f31 as ::core::ffi::c_uint,
    Data2: 0x54d6 as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_PCI_PME_INTERFACE: GUID = _GUID {
    Data1: 0xaac7e6ac as ::core::ffi::c_uint,
    Data2: 0xbb0b as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut FILE_TYPE_NOTIFICATION_GUID_PAGE_FILE: GUID = _GUID {
    Data1: 0xd0a64a1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x38fc as ::core::ffi::c_ushort,
    Data3: 0x4db8 as ::core::ffi::c_ushort,
    Data4: [
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut FILE_TYPE_NOTIFICATION_GUID_HIBERNATION_FILE: GUID = _GUID {
    Data1: 0xb7624d64 as ::core::ffi::c_uint,
    Data2: 0xb9a3 as ::core::ffi::c_ushort,
    Data3: 0x4cf8 as ::core::ffi::c_ushort,
    Data4: [
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut FILE_TYPE_NOTIFICATION_GUID_CRASHDUMP_FILE: GUID = _GUID {
    Data1: 0x9d453eb7 as ::core::ffi::c_uint,
    Data2: 0xd2a6 as ::core::ffi::c_ushort,
    Data3: 0x4dbd as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ECP_OPLOCK_KEY: GUID = _GUID {
    Data1: 0x48850596 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x3050 as ::core::ffi::c_ushort,
    Data3: 0x4be7 as ::core::ffi::c_ushort,
    Data4: [
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ECP_NETWORK_OPEN_CONTEXT: GUID = _GUID {
    Data1: 0xc584edbf as ::core::ffi::c_uint,
    Data2: 0xdf as ::core::ffi::c_ushort,
    Data3: 0x4d28 as ::core::ffi::c_ushort,
    Data4: [
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ECP_PREFETCH_OPEN: GUID = _GUID {
    Data1: 0xe1777b21 as ::core::ffi::c_uint,
    Data2: 0x847e as ::core::ffi::c_ushort,
    Data3: 0x4837 as ::core::ffi::c_ushort,
    Data4: [
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ECP_NFS_OPEN: GUID = _GUID {
    Data1: 0xf326d30c as ::core::ffi::c_uint,
    Data2: 0xe5f8 as ::core::ffi::c_ushort,
    Data3: 0x4fe7 as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ECP_SRV_OPEN: GUID = _GUID {
    Data1: 0xbebfaebc as ::core::ffi::c_uint,
    Data2: 0xaabf as ::core::ffi::c_ushort,
    Data3: 0x489d as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};
