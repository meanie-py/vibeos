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
// @c2rust file-scope data (compiler-exact tables/statics)
pub const REISER2FS_SUPER_MAGIC_STRING: [UCHAR; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [UCHAR; 10]>(*b"ReIsEr2Fs\0") };

pub const NDEBUG: bool = !cfg!(debug_assertions);
#[no_mangle]
pub unsafe extern "system" fn FsRecIsReiserfsVolume(sb: PRFSD_SUPER_BLOCK) -> BOOLEAN {
    let sz_MagicKey: [u8; 10] = REISER2FS_SUPER_MAGIC_STRING;
    for i in 0..MAGIC_KEY_LENGTH {
        let currentChar = (*sb).s_magic[i as usize];
        if currentChar != (sz_MagicKey[i as usize]) as i8 {
            return 0u8;
        }
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRecReiserfsFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Stack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut MountDevice: PDEVICE_OBJECT;
    let mut Spb: PRFSD_SUPER_BLOCK = core::ptr::null_mut();
    let mut SectorSize: ULONG = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut DeviceError: BOOLEAN = 0u8;
    // PAGED_CODE(); inline translation
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR,
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"/home/curi/Projects/os-playground/vibeos-wt-fsrec/drivers/filesystems/fs_rec/reiserfs.c\0"
                    as *const u8 as *const i8 as PVOID,
                53u32,
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
                Offset.QuadPart = REISERFS_DISK_OFFSET_IN_BYTES as i64;
                if FsRecReadBlock(
                    MountDevice,
                    &raw mut Offset,
                    SectorSize,
                    SectorSize,
                    &raw mut Spb as *mut PVOID,
                    &raw mut DeviceError,
                ) != 0
                {
                    if FsRecIsReiserfsVolume(Spb) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                }
                ExFreePool(Spb as PVOID);
            } else {
                DeviceError = 1u8;
            }
            if DeviceError != 0 {
                if ((*MountDevice).Characteristics as u32 & FILE_FLOPPY_DISKETTE) != 0 {
                    Status = STATUS_FS_DRIVER_REQUIRED;
                }
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            Status = FsRecLoadFileSystem(
                DeviceObject,
                core::mem::transmute::<[u8; 122], [u16; 61]>(
                    *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0r\0e\0i\0s\0e\0r\0f\0s\0\0\0",
                ).as_ptr() as PWCHAR,
            );
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    Status
}
