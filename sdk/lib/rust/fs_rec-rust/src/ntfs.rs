// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ExFreePool, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PLARGE_INTEGER};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{RtlAssert};
use ntkrnl_sys::driver_ddk::{DbgPrint};

#[no_mangle]
pub unsafe extern "system" fn FsRecIsNtfsVolume(
    BootSector: *mut _PACKED_BOOT_SECTOR,
    BytesPerSector: u32,
    NumberOfSectors: *mut LARGE_INTEGER,
) -> u8 {
    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *mut ::core::ffi::c_void,
                b"drivers/filesystems/fs_rec/ntfs.c\0".as_ptr() as *mut ::core::ffi::c_void,
                31u32,
                ::core::ptr::null_mut::<i8>(),
            );
        }
    }
    let mut result: u8 = 0;
    if (*BootSector).Oem[0] as i32 == 'N' as i32
        && (*BootSector).Oem[1] as i32 == 'T' as i32
        && (*BootSector).Oem[2] as i32 == 'F' as i32
        && (*BootSector).Oem[3] as i32 == 'S' as i32
        && (*BootSector).Oem[4] as i32 == ' ' as i32
        && (*BootSector).Oem[5] as i32 == ' ' as i32
        && (*BootSector).Oem[6] as i32 == ' ' as i32
        && (*BootSector).Oem[7] as i32 == ' ' as i32
    {
        result = 1u8;
    }
    result
}

#[no_mangle]
pub unsafe extern "system" fn FsRecNtfsFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Stack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut MountDevice: PDEVICE_OBJECT;
    let mut Bpb: PPACKED_BOOT_SECTOR = core::ptr::null_mut::<_PACKED_BOOT_SECTOR>();
    let mut SectorSize: u32 = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset2: LARGE_INTEGER = core::mem::zeroed();
    let mut Offset3: LARGE_INTEGER = core::mem::zeroed();
    let mut SectorCount: LARGE_INTEGER = core::mem::zeroed();

    // PAGED_CODE()
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if true {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/filesystems/fs_rec/ntfs.c\0"
                    .as_ptr() as PVOID,
                61u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    Stack = IoGetCurrentIrpStackLocation(Irp);
    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            if FsRecGetDeviceSectorSize(MountDevice, &raw mut SectorSize) != 0
                && FsRecGetDeviceSectors(MountDevice, SectorSize, &raw mut SectorCount) != 0
            {
                Offset2.QuadPart = SectorCount.QuadPart >> 1;
                Offset2.QuadPart *= SectorSize as i64;
                Offset3.QuadPart = (SectorCount.QuadPart - 1i64) * SectorSize as i64;

                if FsRecReadBlock(
                    MountDevice,
                    &raw mut Offset,
                    512u32,
                    SectorSize,
                    (&raw mut Bpb) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut::<u8>(),
                ) != 0
                {
                    if FsRecIsNtfsVolume(Bpb, SectorSize, &raw mut SectorCount) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                } else if FsRecReadBlock(
                    MountDevice,
                    &raw mut Offset2,
                    512u32,
                    SectorSize,
                    (&raw mut Bpb) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut::<u8>(),
                ) != 0
                {
                    if FsRecIsNtfsVolume(Bpb, SectorSize, &raw mut SectorCount) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                } else if FsRecReadBlock(
                    MountDevice,
                    &raw mut Offset3,
                    512u32,
                    SectorSize,
                    (&raw mut Bpb) as *mut *mut core::ffi::c_void,
                    core::ptr::null_mut::<u8>(),
                ) != 0
                {
                    if FsRecIsNtfsVolume(Bpb, SectorSize, &raw mut SectorCount) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                }
                ExFreePool(Bpb as *mut core::ffi::c_void);
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            let wide_string: &[u16] = &core::mem::transmute::<[u8; 114], [u16; 57]>(
                *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0N\0t\0f\0s\0\0\0",
            );
            Status = FsRecLoadFileSystem(DeviceObject, wide_string.as_ptr() as PWCHAR);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    Status
}
