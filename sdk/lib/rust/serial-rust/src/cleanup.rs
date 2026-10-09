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
pub unsafe extern "system" fn SerialCleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) IRP_MJ_CLEANUP\n\0" as *const u8 as PCSTR,
        b"cleanup.c\0" as *const u8 as *const core::ffi::c_char,
        19 as core::ffi::c_int,
    );
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
//  * FILE:            drivers/dd/serial/cleanup.c
//  * PURPOSE:         Serial IRP_MJ_CLEANUP operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// NTSTATUS NTAPI
// SerialCleanup(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	TRACE_!(SERIAL, "IRP_MJ_CLEANUP\n");
// 	Irp->IoStatus.Information = 0;
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
