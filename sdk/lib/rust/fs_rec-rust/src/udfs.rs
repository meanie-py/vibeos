// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExFreePool, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::{DbgPrint};

use crate::glue::{IRP_MN_LOAD_FILE_SYSTEM, IRP_MN_MOUNT_VOLUME};
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const VSD_STD_ID_NSR02: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"NSR02\0") };

pub const VSD_STD_ID_BEA01: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"BEA01\0") };

pub const VSD_STD_ID_BOOT2: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"BOOT2\0") };

pub const VSD_STD_ID_CD001: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"CD001\0") };

pub const VSD_STD_ID_CDW02: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"CDW02\0") };

pub const VSD_STD_ID_NSR03: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"NSR03\0") };

pub const VSD_STD_ID_TEA01: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"TEA01\0") };
#[no_mangle]
pub unsafe extern "system" fn FsRecIsUdfsVolume(
    DeviceObject: PDEVICE_OBJECT,
    SectorSize: u32,
) -> u8 {
    let mut VolumeStructDesc: *mut _VOLSTRUCTDESC = core::ptr::null_mut();
    let mut Offset = core::mem::zeroed::<LARGE_INTEGER>();
    let mut ret: u8 = 0u8;
    let mut i: i32;

    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filesystems/fs_rec/udfs.c\0"
                    .as_ptr() as *const i8 as PVOID,
                30u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    Offset.QuadPart = (16u64 * SectorSize as u64) as i64;
    i = 0;
    while i < 16 {
        if FsRecReadBlock(
            DeviceObject,
            &mut Offset,
            SectorSize,
            SectorSize,
            &mut VolumeStructDesc as *mut *mut _VOLSTRUCTDESC as *mut PVOID,
            core::ptr::null_mut::<u8>(),
        ) == 0u8
        {
            break;
        }

        strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_BEA01.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        );

        if strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_NSR03.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        ) == 0
        {
            ret = 1u8;
        }

        if strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_NSR02.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        ) == 0
        {
            ret = 1u8;
        }

        strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_TEA01.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        );
        strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_CD001.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        );
        strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_CDW02.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        );
        strncmp(
            (*VolumeStructDesc).Ident.as_ptr() as *const i8,
            VSD_STD_ID_BOOT2.as_ptr() as *const i8,
            VSD_STD_ID_LEN as u64,
        );

        Offset.QuadPart += SectorSize as i64;
        i += 1;
    }

    if !VolumeStructDesc.is_null() {
        ExFreePool(VolumeStructDesc as *mut core::ffi::c_void);
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn FsRecUdfsFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filesystems/fs_rec/udfs.c\0".as_ptr() as *const i8 as PVOID,
                113 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }
    let Stack = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS;
    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            let MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            let mut SectorSize: ULONG = 0;
            if FsRecGetDeviceSectorSize(MountDevice, core::ptr::addr_of_mut!(SectorSize)) != 0 {
                if FsRecIsUdfsVolume(MountDevice, SectorSize) != 0 {
                    Status = STATUS_FS_DRIVER_REQUIRED;
                }
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            Status = FsRecLoadFileSystem(
                DeviceObject,
                win32macros_rust::wide_bytes!(b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0U\0d\0f\0s\0\0\0") as PWCHAR,
            );
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    Status
}
