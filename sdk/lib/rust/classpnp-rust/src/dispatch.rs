use crate::types::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{DRIVER_DISPATCH, IoGetCurrentIrpStackLocation, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
use core::ffi::c_void;

extern "system" {
    fn ClassCreateClose(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassReadWrite(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassDeviceControlDispatch(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassInternalIoControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassShutdownFlush(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassDispatchPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassDispatchPower(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
    fn ClassSystemControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> i32;
}

// ---- file-scope preamble (translated) ----
pub static mut ClassDispatchUnimplemented: DRIVER_DISPATCH = core::ptr::null_mut::<core::ffi::c_void>() as DRIVER_DISPATCH;

const IRP_MJ_SCSI: u32 = 0x0B;

const IRP_MJ_POWER: u32 = 0x17;
#[no_mangle]
pub unsafe extern "C" fn ClassInitializeDispatchTables(
    DriverExtension: *mut CLASS_DRIVER_EXTENSION,
) {
    // PAGED_CODE();
    for idx in 0..=IRP_MJ_MAXIMUM_FUNCTION as u32 {
        (*DriverExtension).DeviceMajorFunctionTable[idx as usize] = ClassDispatchUnimplemented;
    }

    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_CREATE as usize] = ClassCreateClose;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_CLOSE as usize] = ClassCreateClose;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_READ as usize] = ClassReadWrite;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_WRITE as usize] = ClassReadWrite;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_DEVICE_CONTROL as usize] = ClassDeviceControlDispatch;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_SCSI as usize] = ClassInternalIoControl;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_SHUTDOWN as usize] = ClassShutdownFlush;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_FLUSH_BUFFERS as usize] = ClassShutdownFlush;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_PNP as usize] = ClassDispatchPnp;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_POWER as usize] = ClassDispatchPower;
    (*DriverExtension).DeviceMajorFunctionTable[IRP_MJ_SYSTEM_CONTROL as usize] = ClassSystemControl;
}

#[no_mangle]
pub unsafe extern "system" fn ClassGlobalDispatch(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    unsafe {
        let commonExtension: PCOMMON_DEVICE_EXTENSION = (*DeviceObject).DeviceExtension;
        let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
        // Code Analysis cannot analyze the code paths specific to clients.
        // _Analysis_assume_(FALSE);
        return ((*commonExtension).DispatchTable[(*irpStack).MajorFunction as usize])(DeviceObject, Irp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClassDispatchUnimplemented(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;

    if (*IoGetCurrentIrpStackLocation(Irp)).MajorFunction == IRP_MJ_POWER {
        PoStartNextPowerIrp(Irp);
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    STATUS_INVALID_DEVICE_REQUEST
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 2005
//
// Module Name:
//
//     dispatch.c
//
// Abstract:
//
//     Code to support multiple dispatch tables.
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #include "classp.h"
//
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, ClassInitializeDispatchTables)
// #endif
//
// DRIVER_DISPATCH ClassDispatchUnimplemented;
//
// //
// // Routines start
// //
//
//
// VOID
// ClassInitializeDispatchTables(
//     PCLASS_DRIVER_EXTENSION DriverExtension
//     )
// {
//     ULONG idx;
//
//     PAGED_CODE();
//
//     //
//     // Initialize the standard device dispatch table
//     //
//
//     for (idx = 0; idx <= IRP_MJ_MAXIMUM_FUNCTION; idx++) {
//         DriverExtension->DeviceMajorFunctionTable[idx] = ClassDispatchUnimplemented;
//     }
//
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_CREATE]         = ClassCreateClose;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_CLOSE]          = ClassCreateClose;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_READ]           = ClassReadWrite;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_WRITE]          = ClassReadWrite;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_DEVICE_CONTROL] = ClassDeviceControlDispatch;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_SCSI]           = ClassInternalIoControl;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_SHUTDOWN]       = ClassShutdownFlush;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_FLUSH_BUFFERS]  = ClassShutdownFlush;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_PNP]            = ClassDispatchPnp;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_POWER]          = ClassDispatchPower;
//     DriverExtension->DeviceMajorFunctionTable[IRP_MJ_SYSTEM_CONTROL] = ClassSystemControl;
//
//
//     return;
// }
//
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassGlobalDispatch(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
//
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//     // Code Analysis cannot analyze the code paths specific to clients.
//     _Analysis_assume_(FALSE);
//     return (commonExtension->DispatchTable[irpStack->MajorFunction])(DeviceObject, Irp);
//
// }
//
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassDispatchUnimplemented(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
// /*++
//
// Routine Description:
//
//     This function is the default dispatch routine.  Its
//     responsibility is simply to set the status in the packet to indicate
//     that the operation requested is invalid for this device type, and then
//     complete the packet.
//
// Arguments:
//
//     DeviceObject - Specifies the device object for which this request is
//         bound.  Ignored by this routine.
//
//     Irp - Specifies the address of the I/O Request Packet (IRP) for this
//         request.
//
// Return Value:
//
//     The final status is always STATUS_INVALID_DEVICE_REQUEST.
//
//
// --*/
//
// {
//     UNREFERENCED_PARAMETER( DeviceObject );
//
//     //
//     // Simply store the appropriate status, complete the request, and return
//     // the same status stored in the packet.
//     //
//
//     if ((IoGetCurrentIrpStackLocation(Irp))->MajorFunction == IRP_MJ_POWER) {
//         PoStartNextPowerIrp(Irp);
//     }
//     Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
//     IoCompleteRequest( Irp, IO_NO_INCREMENT );
//     return STATUS_INVALID_DEVICE_REQUEST;
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
