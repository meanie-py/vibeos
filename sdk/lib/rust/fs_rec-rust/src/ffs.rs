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
pub const NDEBUG: bool = true;
#[no_mangle]
pub unsafe extern "system" fn FsRecIsFfsDiskLabel(dl: PFFSD_DISKLABEL) -> BOOLEAN {
    ((*dl).d_magic == DISKMAGIC) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn FsRecIsFfs1Volume(sb: *mut FFSD_SUPER_BLOCK) -> BOOLEAN {
    if (*sb).fs_magic == (FS_UFS1_MAGIC) as i32 {
        1u8
    } else {
        0u8
    }
}
#[no_mangle]
pub unsafe extern "system" fn FsRecIsFfs2Volume(sb: PFFSD_SUPER_BLOCK) -> BOOLEAN {
    return ((*sb).fs_magic == (FS_UFS2_MAGIC) as i32) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn FsRecFfsFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let MountDevice: PDEVICE_OBJECT;
    let mut Spb: PFFSD_SUPER_BLOCK = core::ptr::null_mut();
    let mut DiskLabel: PFFSD_DISKLABEL = core::ptr::null_mut();
    let mut FSOffset: ULONGLONG = 0;
    let mut i: i32;
    let mut SectorSize: ULONG = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut DeviceError: BOOLEAN = 0u8;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/filesystems/fs_rec/ffs.c\0"
                    as *const u8 as *const i8 as PVOID,
                56 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    Stack = IoGetCurrentIrpStackLocation(Irp);
    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            if FsRecGetDeviceSectorSize(MountDevice, &raw mut SectorSize) != 0 {
                Offset.QuadPart = (LABELSECTOR as ULONG).wrapping_mul(SectorSize) as LONGLONG;
                if FsRecReadBlock(
                    MountDevice,
                    &raw mut Offset,
                    SectorSize,
                    SectorSize,
                    &raw mut DiskLabel as PVOID as *mut PVOID,
                    &raw mut DeviceError,
                ) != 0
                {
                    if FsRecIsFfsDiskLabel(DiskLabel) != 0 {
                        i = 0;
                        while i < (MAXPARTITIONS) as i32 {
                            if (*DiskLabel).d_partitions[i as usize].p_fstype as i32 == (FS_BSDFFS) as i32 {
                                FSOffset = (*DiskLabel).d_partitions[i as usize].p_offset as ULONGLONG;
                                FSOffset = (FSOffset as u64)
                                    .wrapping_mul(SectorSize as u64) as ULONGLONG;
                                Offset.QuadPart = (FSOffset as u64)
                                    .wrapping_add(SBLOCK_UFS1 as u64) as LONGLONG;
                                if FsRecReadBlock(
                                    MountDevice,
                                    &raw mut Offset,
                                    SBLOCKSIZE as ULONG,
                                    SectorSize,
                                    &raw mut Spb as PVOID as *mut PVOID,
                                    &raw mut DeviceError,
                                ) != 0
                                {
                                    if FsRecIsFfs1Volume(Spb) != 0 {
                                        Status = STATUS_FS_DRIVER_REQUIRED;
                                    } else {
                                        ExFreePool(Spb as PVOID);
                                        Spb = core::ptr::null_mut();
                                        Offset.QuadPart = (FSOffset as u64)
                                            .wrapping_add(SBLOCK_UFS2 as u64) as LONGLONG;
                                        if FsRecReadBlock(
                                            MountDevice,
                                            &raw mut Offset,
                                            SBLOCKSIZE as ULONG,
                                            SectorSize,
                                            &raw mut Spb as PVOID as *mut PVOID,
                                            &raw mut DeviceError,
                                        ) != 0
                                        {
                                            if FsRecIsFfs2Volume(Spb) != 0 {
                                                Status = STATUS_FS_DRIVER_REQUIRED;
                                            }
                                        }
                                    }
                                }
                                ExFreePool(Spb as PVOID);
                                Spb = core::ptr::null_mut();
                            }
                            i += 1;
                        }
                    } else {
                        Offset.QuadPart = (FSOffset as u64)
                            .wrapping_add(SBLOCK_UFS1 as u64) as LONGLONG;
                        if FsRecReadBlock(
                            MountDevice,
                            &raw mut Offset,
                            SBLOCKSIZE as ULONG,
                            SectorSize,
                            &raw mut Spb as PVOID as *mut PVOID,
                            &raw mut DeviceError,
                        ) != 0
                        {
                            if FsRecIsFfs1Volume(Spb) != 0 {
                                Status = STATUS_FS_DRIVER_REQUIRED;
                            } else {
                                ExFreePool(Spb as PVOID);
                                Spb = core::ptr::null_mut();
                                Offset.QuadPart = (FSOffset as u64)
                                    .wrapping_add(SBLOCK_UFS2 as u64) as LONGLONG;
                                if FsRecReadBlock(
                                    MountDevice,
                                    &raw mut Offset,
                                    SBLOCKSIZE as ULONG,
                                    SectorSize,
                                    &raw mut Spb as PVOID as *mut PVOID,
                                    &raw mut DeviceError,
                                ) != 0
                                {
                                    if FsRecIsFfs2Volume(Spb) != 0 {
                                        Status = STATUS_FS_DRIVER_REQUIRED;
                                    }
                                }
                            }
                        }
                        ExFreePool(Spb as PVOID);
                    }
                }
                ExFreePool(DiskLabel as PVOID);
            } else {
                DeviceError = 1u8;
            }
            if DeviceError != 0 {
                if (*MountDevice).Characteristics as u32 & FILE_FLOPPY_DISKETTE as u32 != 0 {
                    Status = STATUS_FS_DRIVER_REQUIRED;
                }
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            let path: &[u16; 56] = &unsafe {
                core::mem::transmute::<[u8; 112], [u16; 56]>(*b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0f\0f\0s\0\0\0")
            };
            Status = FsRecLoadFileSystem(DeviceObject, path.as_ptr() as PWCHAR);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    return Status;
}
