use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP};
use ntkrnl_sys::inline::tier1::IoSkipCurrentIrpStackLocation;
use crate::shared::DEVICE_EXTENSION;

extern "system" {
    fn RtlAssert( assertion: *mut core::ffi::c_void, source_file: *mut core::ffi::c_void, line_number: u32, comment: *mut core::ffi::c_char, );
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: bool = true;

#[no_mangle]
pub unsafe extern "system" fn ForwardIrpAndForget(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut LowerDevice: PDEVICE_OBJECT = core::ptr::null_mut();
    LowerDevice = (*((*DeviceObject).DeviceExtension as *mut DEVICE_EXTENSION)).LowerDevice;
    if LowerDevice.is_null() {
        RtlAssert(
            b"LowerDevice\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"/home/curi/Projects/os-playground/vibeos/drivers/processor/processr/misc.c\0"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            27u32,
            core::ptr::null_mut::<core::ffi::c_char>(),
        );
    }
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver(LowerDevice, Irp);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:        ReactOS Generic CPU Driver
//  * LICENSE:        GNU GPLv2 only as published by the Free Software Foundation
//  * FILE:           drivers/processor/processr/misc.c
//  * PURPOSE:        Misc routines
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
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// ForwardIrpAndForget(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PDEVICE_OBJECT LowerDevice;
//
//     LowerDevice = ((PDEVICE_EXTENSION)DeviceObject->DeviceExtension)->LowerDevice;
//     ASSERT(LowerDevice);
//
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(LowerDevice, Irp);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
