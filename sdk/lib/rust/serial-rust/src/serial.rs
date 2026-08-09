use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDRIVER_OBJECT, PUNICODE_STRING, PDRIVER_UNLOAD, PDRIVER_ADD_DEVICE, PDRIVER_DISPATCH};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// (file-scope: only function-pointer-typed forward declarations; no concrete declarations to emit)
unsafe extern "system" fn DriverUnload(mut DriverObject: PDRIVER_OBJECT) {
    // nothing to do here yet
}

// The dispatch routines and PDRIVER_DISPATCH are all spelled `extern "system"` now
// (normalize_island_abi.py), so every slot here takes `Some(f)` directly.

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut i: ULONG;

    if IsNEC_98 != 0 {
        return STATUS_NOT_IMPLEMENTED;
    }

    // DriverUnload used to need a per-arch transmute into the slot's literal ABI. That
    // transmute erased the whole fn type, so a signature drift here would have compiled.
    (*DriverObject).DriverUnload = Some(DriverUnload);
    (*(*DriverObject).DriverExtension).AddDevice = Some(SerialAddDevice);

    i = 0 as ULONG;
    while i <= IRP_MJ_MAXIMUM_FUNCTION as ULONG {
        (*DriverObject).MajorFunction[i as usize] = Some(ForwardIrpAndForget);
        i = i.wrapping_add(1);
    }
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = Some(SerialCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = Some(SerialClose);
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = Some(SerialCleanup);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] = Some(SerialRead);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] = Some(SerialWrite);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(SerialDeviceControl);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION as usize] = Some(SerialQueryInformation);
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(SerialPnp);
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(SerialPower);

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial driver
//  * FILE:            drivers/dd/serial/serial.c
//  * PURPOSE:         Serial driver loading/unloading
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// static DRIVER_UNLOAD DriverUnload;
// DRIVER_INITIALIZE DriverEntry;
//
// static VOID NTAPI
// DriverUnload(IN PDRIVER_OBJECT DriverObject)
// {
// 	// nothing to do here yet
// }
//
// /*
//  * Standard DriverEntry method.
//  */
// NTSTATUS NTAPI
// DriverEntry(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PUNICODE_STRING RegPath)
// {
// 	ULONG i;
//
//     if (IsNEC_98)
//     {
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
// 	DriverObject->DriverUnload = DriverUnload;
// 	DriverObject->DriverExtension->AddDevice = SerialAddDevice;
//
// 	for (i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++)
// 		DriverObject->MajorFunction[i] = ForwardIrpAndForget;
// 	DriverObject->MajorFunction[IRP_MJ_CREATE] = SerialCreate;
// 	DriverObject->MajorFunction[IRP_MJ_CLOSE] = SerialClose;
// 	DriverObject->MajorFunction[IRP_MJ_CLEANUP] = SerialCleanup;
// 	DriverObject->MajorFunction[IRP_MJ_READ] = SerialRead;
// 	DriverObject->MajorFunction[IRP_MJ_WRITE] = SerialWrite;
// 	DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = SerialDeviceControl;
// 	DriverObject->MajorFunction[IRP_MJ_QUERY_INFORMATION] = SerialQueryInformation;
// 	DriverObject->MajorFunction[IRP_MJ_PNP] = SerialPnp;
// 	DriverObject->MajorFunction[IRP_MJ_POWER] = SerialPower;
//
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
