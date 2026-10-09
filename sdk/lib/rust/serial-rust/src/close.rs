use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP};
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn SerialClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: ULONG = 39;
    const DPFLTR_TRACE_LEVEL: ULONG = 2;

    let mut pDeviceExtension: PSERIAL_DEVICE_EXTENSION =
        core::ptr::null_mut::<_SERIAL_DEVICE_EXTENSION>();

    DbgPrintEx(
        DPFLTR_SERIAL_ID,
        DPFLTR_TRACE_LEVEL,
        b"(%s:%d) IRP_MJ_CLOSE\n\0" as *const u8 as PCSTR,
        b"drivers/serial/serial/close.c\0"
            as *const u8 as *const core::ffi::c_char,
        21 as core::ffi::c_int,
    );

    pDeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
    (*pDeviceExtension).IsOpened = 0 as BOOLEAN;

    (*Irp).IoStatus.Information = 0 as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/close.c
//  * PURPOSE:         Serial IRP_MJ_CLOSE operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// NTSTATUS NTAPI
// SerialClose(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PSERIAL_DEVICE_EXTENSION pDeviceExtension;
//
// 	TRACE_!(SERIAL, "IRP_MJ_CLOSE\n");
// 	pDeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
// 	pDeviceExtension->IsOpened = FALSE;
//
// 	Irp->IoStatus.Information = 0;
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
