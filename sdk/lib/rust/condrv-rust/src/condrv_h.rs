//! Hand-written translation of the private types from
//! `drivers/base/condrv/condrv.h` (the driver-object extension). Layout must
//! match the C — CONDRV_DRIVER is the IoAllocateDriverObjectExtension blob.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use ntkrnl_sys::ddk::{PDEVICE_OBJECT, UNICODE_STRING};

// typedef struct _CONDRV_DRIVER { UNICODE_STRING RegistryPath;
//                                 PDEVICE_OBJECT Controller; } CONDRV_DRIVER;
#[repr(C)]
pub struct CONDRV_DRIVER {
    pub RegistryPath: UNICODE_STRING,
    pub Controller: PDEVICE_OBJECT,
}
pub type PCONDRV_DRIVER = *mut CONDRV_DRIVER;

// L"\Device\ConDrv"  (DD_CONDRV_CTRL_DEVICE_NAME_U, ntddcon.h), NUL-terminated.
pub static DD_CONDRV_CTRL_DEVICE_NAME_U: [u16; 15] = [
    0x5C, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x5C, 0x43, 0x6F, 0x6E, 0x44, 0x72, 0x76, 0x00,
];
// L"\DosDevices\Global\ConDrv"  (DD_CONDRV_CTRL_SYMLNK_NAME_U), NUL-terminated.
pub static DD_CONDRV_CTRL_SYMLNK_NAME_U: [u16; 26] = [
    0x5C, 0x44, 0x6F, 0x73, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x73, 0x5C, 0x47, 0x6C, 0x6F,
    0x62, 0x61, 0x6C, 0x5C, 0x43, 0x6F, 0x6E, 0x44, 0x72, 0x76, 0x00,
];
