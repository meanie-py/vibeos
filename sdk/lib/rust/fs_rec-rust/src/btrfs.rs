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
use ntkrnl_sys::ddk::{PCSTR, RtlAssert, _LARGE_INTEGER__bindgen_ty_1};
use ntkrnl_sys::driver_ddk::{DbgPrint};

use crate::glue::{IRP_MN_LOAD_FILE_SYSTEM, IRP_MN_MOUNT_VOLUME};
#[no_mangle]
pub unsafe extern "system" fn FsRecIsBtrfsVolume(SuperBlock: PBTRFS_SUPER_BLOCK) -> BOOLEAN {
    if (*SuperBlock).magic == BTRFS_MAGIC {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn FsRecBtrfsFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS;
    let MountDevice: PDEVICE_OBJECT;
    let mut Spb: PBTRFS_SUPER_BLOCK = core::ptr::null_mut::<_BTRFS_SUPER_BLOCK>();
    let mut SectorSize: ULONG = 0;
    let mut Offset: LARGE_INTEGER = LARGE_INTEGER {
        __bindgen_anon_1: _LARGE_INTEGER__bindgen_ty_1 {
            LowPart: 0,
            HighPart: 0,
        },
    };
    let mut DeviceError: BOOLEAN = 0u8;

    // PAGED_CODE() – inline
    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as PVOID,
            b"drivers/filesystems/fs_rec/btrfs.c\0"
                .as_ptr() as PVOID,
            40 as ULONG,
            core::ptr::null_mut::<c_char>(),
        );
    }

    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            if FsRecGetDeviceSectorSize(MountDevice, &raw mut SectorSize) != 0 {
                Offset.QuadPart = BTRFS_SB_OFFSET as i64;
                if FsRecReadBlock(
                    MountDevice,
                    &raw mut Offset,
                    BTRFS_SB_SIZE as ULONG,
                    SectorSize,
                    &raw mut Spb as *mut PBTRFS_SUPER_BLOCK as PVOID as *mut PVOID,
                    &raw mut DeviceError,
                ) != 0
                {
                    if FsRecIsBtrfsVolume(Spb) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                }
                ExFreePool(Spb as PVOID);
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
            let wide_path: [u16; 58] = [
                0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79, 0x5C,
                0x4D, 0x61, 0x63, 0x68, 0x69, 0x6E, 0x65, 0x5C, 0x53, 0x79,
                0x73, 0x74, 0x65, 0x6D, 0x5C, 0x43, 0x75, 0x72, 0x72, 0x65,
                0x6E, 0x74, 0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x53,
                0x65, 0x74, 0x5C, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65,
                0x73, 0x5C, 0x62, 0x74, 0x72, 0x66, 0x73, 0x00,
            ];
            Status = FsRecLoadFileSystem(DeviceObject, wide_path.as_ptr() as PWCHAR);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    Status
}
