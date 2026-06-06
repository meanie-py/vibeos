//! Rust translation of `drivers/base/condrv/condrv.c` — DriverEntry / Unload.

use crate::condrv_h::*;
use crate::control::{ConDrvCreateController, ConDrvDeleteController};
use crate::dispatch::ConDrvDispatch;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PUNICODE_STRING, IoAllocateDriverObjectExtension};

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    (*DriverObject).DriverUnload = Some(core::mem::transmute(ConDrvUnload as *const ()));

    /* Initialize the different callback function pointers */
    let dispatch = Some(core::mem::transmute(ConDrvDispatch as *const ()));
    let mut i: usize = 0;
    while i <= IRP_MJ_MAXIMUM_FUNCTION as usize {
        (*DriverObject).MajorFunction[i] = dispatch;
        i += 1;
    }

    let mut DriverExtension: PVOID = core::ptr::null_mut();
    let Status = IoAllocateDriverObjectExtension(
        DriverObject,
        DriverObject as PVOID, // Unique ID == the driver object's own address
        core::mem::size_of::<CONDRV_DRIVER>() as u32,
        &mut DriverExtension,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }
    core::ptr::write_bytes(DriverExtension as *mut u8, 0, core::mem::size_of::<CONDRV_DRIVER>());

    ConDrvCreateController(DriverObject, RegistryPath)
}

unsafe extern "system" fn ConDrvUnload(DriverObject: PDRIVER_OBJECT) {
    /* Delete the Controller device (and thus all terminals). */
    ConDrvDeleteController(DriverObject);
}
