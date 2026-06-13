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
pub const NDEBUG: bool = true;
#[no_mangle]
pub unsafe extern "system" fn FsRecIsExtVolume(SuperBlock: PEXT_SUPER_BLOCK) -> BOOLEAN {
    ((*SuperBlock).Magic as i32 == EXT_SUPER_MAGIC as i32) as u8
}

#[no_mangle]
pub unsafe extern "system" fn FsRecExtFsControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let Stack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let MountDevice: PDEVICE_OBJECT;
    let mut Spb: PEXT_SUPER_BLOCK = core::ptr::null_mut();
    let mut SectorSize: ULONG = 0;
    let mut Offset: LARGE_INTEGER = LARGE_INTEGER {
        __bindgen_anon_1: ::core::mem::zeroed(),
    };
    let mut DeviceError: BOOLEAN = 0u8;
    let mut _inline_irql: u32;
    _inline_irql = (KeGetCurrentIrql()) as u32;
    if _inline_irql > (APC_LEVEL) as u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR,
            _inline_irql as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"/home/curi/Projects/os-playground/vibeos-wt-fsrec/drivers/filesystems/fs_rec/ext.c\0"
                    .as_ptr() as *const i8 as PVOID,
                40u32,
                core::ptr::null_mut(),
            );
        }
    }
    Stack = IoGetCurrentIrpStackLocation(Irp);
    match (*Stack).MinorFunction as u32 {
        IRP_MN_MOUNT_VOLUME => {
            Status = STATUS_UNRECOGNIZED_VOLUME;
            MountDevice = (*Stack).Parameters.MountVolume.DeviceObject;
            if FsRecGetDeviceSectorSize(MountDevice, core::ptr::addr_of_mut!(SectorSize)) != 0 {
                Offset.QuadPart = EXT_SB_OFFSET as i64;
                if FsRecReadBlock(
                    MountDevice,
                    core::ptr::addr_of_mut!(Offset),
                    EXT_SB_SIZE as u32,
                    SectorSize,
                    (core::ptr::addr_of_mut!(Spb)) as *mut PVOID,
                    core::ptr::addr_of_mut!(DeviceError),
                ) != 0
                {
                    if FsRecIsExtVolume(Spb) != 0 {
                        Status = STATUS_FS_DRIVER_REQUIRED;
                    }
                }
                ExFreePool(Spb as PVOID);
            } else {
                DeviceError = 1u8;
            }
            if DeviceError != 0 {
                if ((*MountDevice).Characteristics as u32 & FILE_FLOPPY_DISKETTE as u32) != 0 {
                    Status = STATUS_FS_DRIVER_REQUIRED;
                }
            }
        }
        IRP_MN_LOAD_FILE_SYSTEM => {
            let ext_path = core::mem::transmute::<[u8; 118], [u16; 59]>(*b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0E\0x\0t\x002\0f\0s\0\0\0");
            Status = FsRecLoadFileSystem(DeviceObject, ext_path.as_ptr() as PWCHAR);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }
    return Status;
}
