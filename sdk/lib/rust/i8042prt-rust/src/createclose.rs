use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP};

#[no_mangle]
pub unsafe extern "system" fn i8042Create(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn i8042Cleanup(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> i32 {
    let _ = DeviceObject;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn i8042Close(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let _ = DeviceObject;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS i8042 (ps/2 keyboard-mouse controller) driver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/input/i8042prt/createclose.c
//  * PURPOSE:     IRP_MJ_CREATE, IRP_MJ_CLEANUP and IRP_MJ_CLOSE operations
//  * PROGRAMMERS: Copyright 2006-2007 Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "i8042prt.h"
//
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS NTAPI
// i8042Create(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	UNREFERENCED_PARAMETER(DeviceObject);
// 	TRACE_(I8042PRT, "IRP_MJ_CREATE\n");
//
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// i8042Cleanup(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	UNREFERENCED_PARAMETER(DeviceObject);
// 	TRACE_(I8042PRT, "IRP_MJ_CLEANUP\n");
//
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// NTSTATUS NTAPI
// i8042Close(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	UNREFERENCED_PARAMETER(DeviceObject);
// 	TRACE_(I8042PRT, "IRP_MJ_CLOSE\n");
//
// 	Irp->IoStatus.Status = STATUS_SUCCESS;
// 	Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
