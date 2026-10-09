use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP};
// IoGetCurrentIrpStackLocation → crate::glue (tier1 FORCEINLINE)
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn SerialCreate(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: u32 = 39;
    const DPFLTR_TRACE_LEVEL: u32 = 2;
    const DPFLTR_INFO_LEVEL: u32 = 3;
    const DPFLTR_WARNING_LEVEL: u32 = 1;
    const STATUS_NOT_A_DIRECTORY: NTSTATUS = 0xC0000103u32 as NTSTATUS;
    const FILE_DIRECTORY_FILE: u32 = 0x00000001;

    let mut Stack: PIO_STACK_LOCATION;
    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION;
    let mut Status: NTSTATUS;

    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) IRP_MJ_CREATE\n\0" as *const u8 as PCSTR,
        b"create.c\0" as *const u8 as *const ::core::ffi::c_char,
        23 as ::core::ffi::c_int,
    );
    Stack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;

    if (*Stack).Parameters.Create.Options as ::core::ffi::c_ulong
        & FILE_DIRECTORY_FILE as ::core::ffi::c_ulong
        != 0
    {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_INFO_LEVEL as ULONG,
            b"(%s:%d) Not a directory\n\0" as *const u8 as PCSTR,
            b"create.c\0" as *const u8 as *const ::core::ffi::c_char,
            29 as ::core::ffi::c_int,
        );
        Status = STATUS_NOT_A_DIRECTORY;
    } else if (*DeviceExtension).IsOpened != 0 {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_WARNING_LEVEL as ULONG,
            b"(%s:%d) COM%lu is already opened\n\0" as *const u8 as PCSTR,
            b"create.c\0" as *const u8 as *const ::core::ffi::c_char,
            36 as ::core::ffi::c_int,
            (*DeviceExtension).ComPort,
        );
        Status = STATUS_ACCESS_DENIED;
    } else {
        DbgPrintEx(
            DPFLTR_SERIAL_ID as ULONG,
            DPFLTR_INFO_LEVEL as ULONG,
            b"(%s:%d) Open COM%lu: successfull\n\0" as *const u8 as PCSTR,
            b"create.c\0" as *const u8 as *const ::core::ffi::c_char,
            41 as ::core::ffi::c_int,
            (*DeviceExtension).ComPort,
        );
        (*DeviceExtension).IsOpened = 1 as BOOLEAN;
        Status = STATUS_SUCCESS;
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = 0 as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/create.c
//  * PURPOSE:         Serial IRP_MJ_CREATE operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// NTSTATUS NTAPI
// SerialCreate(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PIO_STACK_LOCATION Stack;
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
// 	NTSTATUS Status;
//
// 	TRACE_!(SERIAL, "IRP_MJ_CREATE\n");
// 	Stack = IoGetCurrentIrpStackLocation(Irp);
// 	DeviceExtension = (PSERIAL_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	if (Stack->Parameters.Create.Options & FILE_DIRECTORY_FILE)
// 	{
// 		INFO_(SERIAL, "Not a directory\n");
// 		Status = STATUS_NOT_A_DIRECTORY;
// 		goto ByeBye;
// 	}
//
// 	if(DeviceExtension->IsOpened)
// 	{
// 		WARN_(SERIAL, "COM%lu is already opened\n", DeviceExtension->ComPort);
// 		Status = STATUS_ACCESS_DENIED;
// 		goto ByeBye;
// 	}
//
// 	INFO_(SERIAL, "Open COM%lu: successfull\n", DeviceExtension->ComPort);
// 	DeviceExtension->IsOpened = TRUE;
// 	Status = STATUS_SUCCESS;
//
// ByeBye:
// 	Irp->IoStatus.Status = Status;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
