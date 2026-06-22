// ---- crate-internal glue (private structs, no-op DDK shims, inlines) ----
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::inlines::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql; // FORCEINLINE on amd64/arm64

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExFreePool, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert};
use ntkrnl_sys::driver_ddk::{DbgPrint};
#[no_mangle]
pub unsafe extern "system" fn FsRecIsFatVolume(
    PackedBootSector: PPACKED_BOOT_SECTOR,
) -> BOOLEAN {
    let mut Bpb: BIOS_PARAMETER_BLOCK = core::mem::zeroed();
    let mut Result: BOOLEAN = 1u8;

    // PAGED_CODE() expansion
    if KeGetCurrentIrql() as i32 > APC_LEVEL as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/filesystems/fs_rec/fat.c\0"
                    .as_ptr() as *const i8 as PVOID,
                25u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {}
    }

    core::ptr::write_bytes(
        &raw mut Bpb as *mut u8,
        0,
        core::mem::size_of::<BIOS_PARAMETER_BLOCK>(),
    );

    // FatUnpackBios expanded inline
    unsafe {
        *(&raw mut Bpb.BytesPerSector as *mut u16) = *(&(*PackedBootSector).PackedBpb.BytesPerSector
            as *const _ as *const u16);
        *(&raw mut Bpb.SectorsPerCluster as *mut u8) = *(&(*PackedBootSector).PackedBpb.SectorsPerCluster
            as *const _ as *const u8);
        *(&raw mut Bpb.ReservedSectors as *mut u16) = *(&(*PackedBootSector).PackedBpb.ReservedSectors
            as *const _ as *const u16);
        *(&raw mut Bpb.Fats as *mut u8) = *(&(*PackedBootSector).PackedBpb.Fats as *const _
            as *const u8);
        *(&raw mut Bpb.RootEntries as *mut u16) = *(&(*PackedBootSector).PackedBpb.RootEntries
            as *const _ as *const u16);
        *(&raw mut Bpb.Sectors as *mut u16) = *(&(*PackedBootSector).PackedBpb.Sectors as *const _
            as *const u16);
        *(&raw mut Bpb.Media as *mut u8) = *(&(*PackedBootSector).PackedBpb.Media as *const _
            as *const u8);
        *(&raw mut Bpb.SectorsPerFat as *mut u16) = *(&(*PackedBootSector).PackedBpb.SectorsPerFat
            as *const _ as *const u16);
        *(&raw mut Bpb.SectorsPerTrack as *mut u16) = *(&(*PackedBootSector).PackedBpb.SectorsPerTrack
            as *const _ as *const u16);
        *(&raw mut Bpb.Heads as *mut u16) = *(&(*PackedBootSector).PackedBpb.Heads as *const _
            as *const u16);
        *(&raw mut Bpb.HiddenSectors as *mut u32) = *(&(*PackedBootSector).PackedBpb.HiddenSectors
            as *const _ as *const u32);
        *(&raw mut Bpb.LargeSectors as *mut u32) = *(&(*PackedBootSector).PackedBpb.LargeSectors
            as *const _ as *const u32);
    }

    if Bpb.Sectors != 0 {
        Bpb.LargeSectors = 0u32;
    }

    if (*PackedBootSector).Jump[0] as i32 != 0x49i32
        && (*PackedBootSector).Jump[0] as i32 != 0xE9i32
        && (*PackedBootSector).Jump[0] as i32 != 0xEBi32
    {
        Result = 0u8;
    } else if Bpb.BytesPerSector as i32 != 128i32
        && Bpb.BytesPerSector as i32 != 256i32
        && Bpb.BytesPerSector as i32 != 512i32
        && Bpb.BytesPerSector as i32 != 1024i32
        && Bpb.BytesPerSector as i32 != 2048i32
        && Bpb.BytesPerSector as i32 != 4096i32
    {
        Result = 0u8;
    } else if Bpb.SectorsPerCluster as i32 != 1i32
        && Bpb.SectorsPerCluster as i32 != 2i32
        && Bpb.SectorsPerCluster as i32 != 4i32
        && Bpb.SectorsPerCluster as i32 != 8i32
        && Bpb.SectorsPerCluster as i32 != 16i32
        && Bpb.SectorsPerCluster as i32 != 32i32
        && Bpb.SectorsPerCluster as i32 != 64i32
        && Bpb.SectorsPerCluster as i32 != 128i32
    {
        Result = 0u8;
    } else if Bpb.ReservedSectors == 0 {
        Result = 0u8;
    } else if Bpb.Sectors == 0 && Bpb.LargeSectors == 0 {
        Result = 0u8;
    } else if Bpb.Media as i32 != 0x00i32
        && Bpb.Media as i32 != 0x01i32
        && Bpb.Media as i32 != 0xF0i32
        && Bpb.Media as i32 != 0xF8i32
        && Bpb.Media as i32 != 0xF9i32
        && Bpb.Media as i32 != 0xFAi32
        && Bpb.Media as i32 != 0xFBi32
        && Bpb.Media as i32 != 0xFCi32
        && Bpb.Media as i32 != 0xFDi32
        && Bpb.Media as i32 != 0xFEi32
        && Bpb.Media as i32 != 0xFFi32
    {
        Result = 0u8;
    } else if Bpb.SectorsPerFat != 0 && Bpb.RootEntries == 0 {
        Result = 0u8;
    }

    Result
}
#[no_mangle]
pub unsafe extern "system" fn FsRecVfatFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let mut Status: NTSTATUS;
    let mut Bpb: PPACKED_BOOT_SECTOR = core::ptr::null_mut();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut DeviceError: BOOLEAN = 0u8;
    let mut SectorSize: ULONG = 0; // C-local the LLM dropped (ULONG SectorSize;)

    // PAGED_CODE expansion (c2rust preserved)
    if KeGetCurrentIrql() as u32 > (APC_LEVEL) as u32 {
        let msg = b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR;
        DbgPrint(msg, KeGetCurrentIrql() as i32);
        RtlAssert(
            b"0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/filesystems/fs_rec/fat.c\0"
                .as_ptr() as *const i8 as PVOID,
            110u32,
            (0 as *mut i8 as PVOID).cast(),
        );
    }

    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            let MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            if FsRecGetDeviceSectorSize(MountDevice, core::ptr::addr_of_mut!(SectorSize)) != 0 {
                if FsRecReadBlock(
                    MountDevice,
                    core::ptr::addr_of_mut!(Offset),
                    512u32,
                    SectorSize,
                    &raw mut Bpb as *mut PPACKED_BOOT_SECTOR as *mut PVOID,
                    core::ptr::addr_of_mut!(DeviceError),
                ) != 0
                {
                    if FsRecIsFatVolume(Bpb) != 0u8 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                }
                ExFreePool(Bpb as *mut core::ffi::c_void);
            } else {
                DeviceError = 1u8;
            }
            if DeviceError != 0u8 {
                if ((*MountDevice).Characteristics & FILE_FLOPPY_DISKETTE as u32) != 0 {
                    Status = STATUS_FS_DRIVER_REQUIRED;
                }
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            const WSTR: [u16; 60] = [
                0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
                0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
                0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
                0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
                0x0065, 0x0074, 0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065,
                0x0073, 0x005C, 0x0066, 0x0061, 0x0073, 0x0074, 0x0066, 0x0061, 0x0074, 0x0000,
            ];
            Status = FsRecLoadFileSystem(DeviceObject, WSTR.as_ptr() as PWCHAR);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    Status
}
