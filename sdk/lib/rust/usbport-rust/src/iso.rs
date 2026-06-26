use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG  — function-like/empty macro, skipped

#[no_mangle]
pub unsafe extern "system" fn USBPORT_InitializeIsoTransfer(
    mut FdoDevice: PDEVICE_OBJECT,
    mut Urb: *mut _URB_ISOCH_TRANSFER,
    mut Transfer: PUSBPORT_TRANSFER,
) -> USBD_STATUS {
    if DbgPrint(
        b"(%s:%d) USBPORT_InitializeIsoTransfer: UNIMPLEMENTED. FIXME.\n\0" as *const u8
            as PCSTR,
        b"drivers/usb/usbport/iso.c\0"
            as *const u8 as *const core::ffi::c_char,
        19 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/iso.c\0"
                as *const u8 as *const core::ffi::c_char,
            19 as core::ffi::c_int,
        );
    }
    return USBD_STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn USBPORT_CompleteIsoTransfer(
    mut MiniPortExtension: PVOID,
    mut MiniPortEndpoint: PVOID,
    mut TransferParameters: PVOID,
    mut TransferLength: ULONG,
) -> ULONG {
    if DbgPrint(
        b"(%s:%d) USBPORT_CompleteIsoTransfer: UNIMPLEMENTED. FIXME.\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbport/iso.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        30 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbport/iso.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            30 as ::core::ffi::c_int,
        );
    }
    return 0 as ULONG;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB Port Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBPort isochronous transfer functions
//  * COPYRIGHT:   Copyright 2017 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbport.h"
//
// #define NDEBUG
// #include <debug.h>
//
// USBD_STATUS
// NTAPI
// USBPORT_InitializeIsoTransfer(PDEVICE_OBJECT FdoDevice,
//                               struct _URB_ISOCH_TRANSFER * Urb,
//                               PUSBPORT_TRANSFER Transfer)
// {
//     DPRINT1("USBPORT_InitializeIsoTransfer: UNIMPLEMENTED. FIXME.\n");
//     return USBD_STATUS_NOT_SUPPORTED;
// }
//
// ULONG
// NTAPI
// USBPORT_CompleteIsoTransfer(IN PVOID MiniPortExtension,
//                             IN PVOID MiniPortEndpoint,
//                             IN PVOID TransferParameters,
//                             IN ULONG TransferLength)
// {
//     DPRINT1("USBPORT_CompleteIsoTransfer: UNIMPLEMENTED. FIXME.\n");
//     return 0;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
