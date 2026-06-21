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

// ---- file-scope preamble (translated) ----
// pshpack1.h in fatx.c → 1-byte packed (Reserved at offset 14, total 4096 = 1 sector).
// Without `packed`, repr(C) inserts 2 pad bytes after FatCount, misaligning Reserved/
// Unused vs the on-disk layout. The recognizer only reads SysType[0..3]@0 and
// SectorsPerCluster@8 (before the misalignment), so it works either way — but keep it
// layout-faithful. Fields are read by value, so no unaligned-ref hazard.
#[repr(C, packed)]
pub struct _FATX_BOOT_SECTOR {
    pub SysType: [u8; 4],
    pub VolumeId: u32,
    pub SectorsPerCluster: u32,
    pub FatCount: u16,
    pub Reserved: u32,
    pub Unused: [u8; 4078],
}
pub type FATX_BOOT_SECTOR = _FATX_BOOT_SECTOR;
pub type PFATX_BOOT_SECTOR = *mut _FATX_BOOT_SECTOR;

#[no_mangle]
pub unsafe extern "system" fn FsRecIsFatxVolume(BootSector: PFATX_BOOT_SECTOR) -> BOOLEAN {
    let mut Result: BOOLEAN = 1u8; // TRUE
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filesystems/fs_rec/fatx.c\0".as_ptr() as *const i8 as PVOID,
            37 as ULONG,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (*BootSector).SysType[0] != b'F'
        || (*BootSector).SysType[1] != b'A'
        || (*BootSector).SysType[2] != b'T'
        || (*BootSector).SysType[3] != b'X'
    {
        Result = 0u8; // FALSE
    } else if (*BootSector).SectorsPerCluster != 1
        && (*BootSector).SectorsPerCluster != 2
        && (*BootSector).SectorsPerCluster != 4
        && (*BootSector).SectorsPerCluster != 8
        && (*BootSector).SectorsPerCluster != 16
        && (*BootSector).SectorsPerCluster != 32
        && (*BootSector).SectorsPerCluster != 64
        && (*BootSector).SectorsPerCluster != 128
    {
        Result = 0u8;
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn FsRecFatxFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS;
    let mut Bpb: PFATX_BOOT_SECTOR = core::ptr::null_mut();
    let mut SectorSize: ULONG = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut DeviceError: BOOLEAN = 0;

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filesystems/fs_rec/fatx.c\0".as_ptr() as *const i8 as PVOID,
            76u32,
            core::ptr::null_mut::<i8>(),
        );
    }

    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            let MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            if FsRecGetDeviceSectorSize(MountDevice, &mut SectorSize) != 0 {
                if FsRecReadBlock(
                    MountDevice,
                    &mut Offset,
                    512,
                    SectorSize,
                    &mut Bpb as *mut PFATX_BOOT_SECTOR as *mut PVOID,
                    &mut DeviceError,
                ) != 0 {
                    if FsRecIsFatxVolume(Bpb) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                }
                ExFreePool(Bpb as PVOID);
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            const REGISTRY_PATH: [u16; 59] = [
                '\\' as u16, 'R' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16,
                '\\' as u16, 'M' as u16, 'a' as u16, 'c' as u16, 'h' as u16, 'i' as u16, 'n' as u16, 'e' as u16,
                '\\' as u16, 'S' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16, 'm' as u16,
                '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16, 'n' as u16, 't' as u16,
                'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16, 'l' as u16,
                'S' as u16, 'e' as u16, 't' as u16,
                '\\' as u16, 'S' as u16, 'e' as u16, 'r' as u16, 'v' as u16, 'i' as u16, 'c' as u16, 'e' as u16, 's' as u16,
                '\\' as u16, 'v' as u16, 'f' as u16, 'a' as u16, 't' as u16, 'f' as u16, 's' as u16,
                0u16,
            ];
            Status = FsRecLoadFileSystem(DeviceObject, &REGISTRY_PATH as *const u16 as PWCHAR);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    Status
}
