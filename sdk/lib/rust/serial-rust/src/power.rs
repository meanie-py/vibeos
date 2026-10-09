use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP};
// IoSkipCurrentIrpStackLocation → crate::glue (tier1 FORCEINLINE)
use core::ffi::c_void;

extern "C" {
    fn DbgPrintEx( ComponentId: ULONG, Level: ULONG, Format: PCSTR, ... ) -> ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn SerialPower(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    const DPFLTR_SERIAL_ID: ULONG = 39;
    const DPFLTR_TRACE_LEVEL: ULONG = 2;

    let mut DeviceExtension: PSERIAL_DEVICE_EXTENSION =
        core::ptr::null_mut::<_SERIAL_DEVICE_EXTENSION>();
    DbgPrintEx(
        DPFLTR_SERIAL_ID as ULONG,
        DPFLTR_TRACE_LEVEL as ULONG,
        b"(%s:%d) IRP_MJ_POWER dispatch\n\0" as *const u8 as PCSTR,
        b"drivers/serial/serial/power.c\0"
            as *const u8 as *const core::ffi::c_char,
        21 as core::ffi::c_int,
    );
    DeviceExtension = (*DeviceObject).DeviceExtension as PSERIAL_DEVICE_EXTENSION;
    PoStartNextPowerIrp(Irp as *mut _IRP);
    IoSkipCurrentIrpStackLocation(Irp);
    return PoCallDriver(
        (*DeviceExtension).LowerDevice as *mut _DEVICE_OBJECT,
        Irp as *mut _IRP,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         Serial port driver
//  * FILE:            drivers/dd/serial/power.c
//  * PURPOSE:         Serial IRP_MJ_POWER operations
//  *
//  * PROGRAMMERS:     Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// #include "serial.h"
//
// #include <debug.h>
//
// NTSTATUS NTAPI
// SerialPower(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PSERIAL_DEVICE_EXTENSION DeviceExtension;
//
// 	TRACE_!(SERIAL, "IRP_MJ_POWER dispatch\n");
//
// 	DeviceExtension = DeviceObject->DeviceExtension;
// 	PoStartNextPowerIrp(Irp);
// 	IoSkipCurrentIrpStackLocation(Irp);
// 	return PoCallDriver(DeviceExtension->LowerDevice, Irp);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
