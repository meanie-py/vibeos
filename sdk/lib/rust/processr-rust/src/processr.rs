use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PUNICODE_STRING,
    PDRIVER_DISPATCH, PDRIVER_ADD_DEVICE, PDRIVER_UNLOAD, _DEVICE_OBJECT, _IRP, _DRIVER_OBJECT,
};
use ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation;
use ntkrnl_sys::base::STATUS_SUCCESS;
use crate::shared::PDEVICE_EXTENSION;

// ProcessorPnp / ProcessorAddDevice are NTAPI, defined in pnp.rs.
extern "system" {
    fn ProcessorPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn ProcessorAddDevice(DriverObject: PDRIVER_OBJECT, Pdo: PDEVICE_OBJECT) -> NTSTATUS;
}

unsafe extern "system" fn ProcessorUnload(_DriverObject: PDRIVER_OBJECT) {}

unsafe extern "system" fn ProcessorPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DeviceExtension: PDEVICE_EXTENSION = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    PoCallDriver((*DeviceExtension).LowerDevice as PDEVICE_OBJECT, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // NTAPI dispatch/callback routines → their bindgen fn-pointer types via the
    // transmute bridge (i386 nominal extern "system" ≠ extern "stdcall").
    type DispatchFn = Option<unsafe extern "system" fn(*mut _DEVICE_OBJECT, *mut _IRP) -> NTSTATUS>;
    type AddDeviceFn =
        Option<unsafe extern "system" fn(*mut _DRIVER_OBJECT, *mut _DEVICE_OBJECT) -> NTSTATUS>;
    type UnloadFn = Option<unsafe extern "system" fn(*mut _DRIVER_OBJECT)>;

    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Some(ProcessorPnp));
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        core::mem::transmute::<DispatchFn, PDRIVER_DISPATCH>(Some(ProcessorPower));
    (*(*DriverObject).DriverExtension).AddDevice =
        core::mem::transmute::<AddDeviceFn, PDRIVER_ADD_DEVICE>(Some(ProcessorAddDevice));
    (*DriverObject).DriverUnload =
        core::mem::transmute::<UnloadFn, PDRIVER_UNLOAD>(Some(ProcessorUnload));
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:        ReactOS Generic CPU Driver
//  * LICENSE:        GNU GPLv2 only as published by the Free Software Foundation
//  * FILE:           drivers/processor/processr/processr.c
//  * PURPOSE:        Main Driver Routines
//  * PROGRAMMERS:    Eric Kohl <eric.kohl@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "processr.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS ******************************************************************/
//
// static
// VOID
// NTAPI
// ProcessorUnload(
//     IN PDRIVER_OBJECT DriverObject)
// {
//     DPRINT("ProcessorUnload()\n");
// }
//
//
// static
// NTSTATUS
// NTAPI
// ProcessorPower(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
// //    PIO_STACK_LOCATION IrpSp;
// //    NTSTATUS Status = Irp->IoStatus.Status;
//     PDEVICE_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
//
//     DPRINT("ProcessorPower()\n");
//
// //    IrpSp = IoGetCurrentIrpStackLocation(Irp);
//
//     PoStartNextPowerIrp(Irp);
//     IoSkipCurrentIrpStackLocation(Irp);
//     return PoCallDriver(DeviceExtension->LowerDevice, Irp);
// }
//
//
// NTSTATUS
// NTAPI
// DriverEntry(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PUNICODE_STRING RegistryPath)
// {
//     DPRINT("Processr: DriverEntry()\n");
//
//     DriverObject->MajorFunction[IRP_MJ_PNP] = ProcessorPnp;
//     DriverObject->MajorFunction[IRP_MJ_POWER] = ProcessorPower;
//
//     DriverObject->DriverExtension->AddDevice = ProcessorAddDevice;
//     DriverObject->DriverUnload = ProcessorUnload;
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
