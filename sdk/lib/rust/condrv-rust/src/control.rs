//! Rust translation of `drivers/base/condrv/control.c` — the Controller device
//! (\Device\ConDrv + its \DosDevices\Global\ConDrv symlink).

use crate::condrv_h::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{
    PDRIVER_OBJECT, PUNICODE_STRING, UNICODE_STRING, PDEVICE_OBJECT,
    IoCreateDevice, IoDeleteDevice, IoCreateSymbolicLink, IoDeleteSymbolicLink,
    IoGetDriverObjectExtension, RtlInitUnicodeString,
};

pub unsafe fn ConDrvCreateController(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DeviceName: UNICODE_STRING = core::mem::zeroed();
    let mut SymlinkName: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut DeviceName, DD_CONDRV_CTRL_DEVICE_NAME_U.as_ptr());
    RtlInitUnicodeString(&mut SymlinkName, DD_CONDRV_CTRL_SYMLNK_NAME_U.as_ptr());

    /* Get the driver extension */
    let DriverExtension =
        IoGetDriverObjectExtension(DriverObject, DriverObject as PVOID) as PCONDRV_DRIVER;

    /* Create the Controller device */
    let mut Controller: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status = IoCreateDevice(
        DriverObject,
        0,
        &mut DeviceName,
        FILE_DEVICE_UNKNOWN,
        FILE_DEVICE_SECURE_OPEN,
        0u8,
        &mut Controller,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = IoCreateSymbolicLink(&mut SymlinkName, &mut DeviceName);
    if !NT_SUCCESS(Status) {
        IoDeleteDevice(Controller);
        return Status;
    }

    (*Controller).Flags &= !DO_DEVICE_INITIALIZING;

    /* Save the Controller device */
    (*DriverExtension).Controller = Controller;

    Status
}

pub unsafe fn ConDrvDeleteController(DriverObject: PDRIVER_OBJECT) -> NTSTATUS {
    /* Retrieve the Controller device */
    let Controller = (*(IoGetDriverObjectExtension(DriverObject, DriverObject as PVOID)
        as PCONDRV_DRIVER))
        .Controller;
    if Controller.is_null() {
        return STATUS_OBJECT_TYPE_MISMATCH;
    }

    let mut SymlinkName: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut SymlinkName, DD_CONDRV_CTRL_SYMLNK_NAME_U.as_ptr());
    IoDeleteSymbolicLink(&mut SymlinkName);

    /* Delete the controller device itself */
    IoDeleteDevice(Controller);

    STATUS_SUCCESS
}
