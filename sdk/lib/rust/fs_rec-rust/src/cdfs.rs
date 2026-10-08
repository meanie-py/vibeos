// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExFreePool, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, RtlCompareMemory};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{_DEVICE_OBJECT};

use crate::glue::{IRP_MN_LOAD_FILE_SYSTEM, IRP_MN_MOUNT_VOLUME};
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const VD_IDENTIFIER: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"CD001\0") };

pub const NDEBUG: u8 = 1;
#[no_mangle]
pub unsafe extern "system" fn FsRecIsCdfsVolume(
    DeviceObject: *mut _DEVICE_OBJECT,
    SectorSize: u32,
) -> u8 {
    const VD_HEADER_OFFSET: i64 = 32768;
    const VD_TYPE_PRIMARY: u8 = 1;
    const VD_VERSION: u8 = 1;
    const VD_IDENTIFIER_LENGTH: usize = 5;
    const VD_IDENTIFIER: [u8; 5] = *b"CD001";

    let mut bReturnValue: u8 = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut pVdHeader: *mut _VD_HEADER = core::ptr::null_mut();

    unsafe {
        PAGED_CODE!();
    }

    Offset.QuadPart = VD_HEADER_OFFSET;

    let volume_valid = 'cleanup: {
        let read_result = unsafe {
            FsRecReadBlock(
                DeviceObject,
                core::ptr::addr_of_mut!(Offset),
                core::mem::size_of::<_VD_HEADER>() as u32,
                SectorSize,
                core::ptr::addr_of_mut!(pVdHeader) as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            )
        };
        if read_result == 0 {
            break 'cleanup false;
        }

        let header = unsafe { &*pVdHeader };
        if header.Type != (VD_TYPE_PRIMARY) as i8 {
            break 'cleanup false;
        }

        unsafe {
            if RtlCompareMemory(
                core::ptr::addr_of!(header.Identifier) as *const core::ffi::c_void,
                VD_IDENTIFIER.as_ptr() as *const core::ffi::c_void,
                (VD_IDENTIFIER_LENGTH as u32) as _,
            ) != (VD_IDENTIFIER_LENGTH) as _
            {
                break 'cleanup false;
            }
        }

        if header.Version != (VD_VERSION) as i8 {
            break 'cleanup false;
        }

        true
    };

    if volume_valid {
        bReturnValue = 1;
    }

    unsafe {
        if !pVdHeader.is_null() {
            ExFreePool(pVdHeader as *mut core::ffi::c_void);
        }
    }

    bReturnValue
}
#[no_mangle]
pub unsafe extern "system" fn FsRecCdfsFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> i32 {
    let stack = IoGetCurrentIrpStackLocation(Irp);
    let mut status;
    match (*stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            status = STATUS_UNRECOGNIZED_VOLUME;
            let mount_device = (*stack).Parameters.MountVolume.DeviceObject;
            let mut sector_size: u32 = 0;
            if FsRecGetDeviceSectorSize(mount_device, &mut sector_size) != 0 {
                if FsRecIsCdfsVolume(mount_device, sector_size) != 0 {
                    status = STATUS_FS_DRIVER_REQUIRED;
                }
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            let ws: &[u16] = &[
                0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79, 0x5C, 0x4D, 0x61, 0x63,
                0x68, 0x69, 0x6E, 0x65, 0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x5C, 0x43,
                0x75, 0x72, 0x72, 0x65, 0x6E, 0x74, 0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C,
                0x53, 0x65, 0x74, 0x5C, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0x5C,
                0x43, 0x64, 0x66, 0x73, 0x00,
            ];
            status = FsRecLoadFileSystem(DeviceObject, ws.as_ptr() as PWCHAR);
        }
        _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    status
}
