use crate::types::*;
use crate::glue::*;
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_DumpHwTD(mut TD: PEHCI_HCD_TD) {
    while !TD.is_null() {
        if DbgPrint(
            b"(%s:%d) : TD                       - %p\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/debug.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            19 as ::core::ffi::c_int,
            TD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/usb/usbehci/debug.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                19 as ::core::ffi::c_int,
            );
        }
        if DbgPrint(
            b"(%s:%d) : TD->PhysicalAddress      - %lx\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/debug.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            20 as ::core::ffi::c_int,
            (*TD).PhysicalAddress,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/usb/usbehci/debug.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                20 as ::core::ffi::c_int,
            );
        }
        if DbgPrint(
            b"(%s:%d) : TD->HwTD.NextTD          - %lx\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/debug.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            21 as ::core::ffi::c_int,
            (*TD).HwTD.NextTD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/usb/usbehci/debug.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                21 as ::core::ffi::c_int,
            );
        }
        if DbgPrint(
            b"(%s:%d) : TD->HwTD.AlternateNextTD - %lx\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/debug.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            22 as ::core::ffi::c_int,
            (*TD).HwTD.AlternateNextTD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/usb/usbehci/debug.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                22 as ::core::ffi::c_int,
            );
        }
        if DbgPrint(
            b"(%s:%d) : TD->HwTD.Token.AsULONG   - %lx\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/debug.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            23 as ::core::ffi::c_int,
            (*TD).HwTD.Token.AsULONG,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/usb/usbehci/debug.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                23 as ::core::ffi::c_int,
            );
        }
        TD = (*TD).NextHcdTD as PEHCI_HCD_TD;
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_DumpHwQH(QH: PEHCI_HCD_QH) {

    if QH.is_null() {
        return;
    }

    if DbgPrint(
        b"(%s:%d) : QH->sqh.HwQH.CurrentTD       - %lx\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const core::ffi::c_char,
        36 as core::ffi::c_int,
        (*QH).sqh.HwQH.CurrentTD,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const core::ffi::c_char,
            36 as core::ffi::c_int,
        );
    }

    if DbgPrint(
        b"(%s:%d) : QH->sqh.HwQH.NextTD          - %lx\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const core::ffi::c_char,
        37 as core::ffi::c_int,
        (*QH).sqh.HwQH.NextTD,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const core::ffi::c_char,
            37 as core::ffi::c_int,
        );
    }

    if DbgPrint(
        b"(%s:%d) : QH->sqh.HwQH.AlternateNextTD - %lx\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const core::ffi::c_char,
        38 as core::ffi::c_int,
        (*QH).sqh.HwQH.AlternateNextTD,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const core::ffi::c_char,
            38 as core::ffi::c_int,
        );
    }

    if DbgPrint(
        b"(%s:%d) : QH->sqh.HwQH.Token.AsULONG   - %lx\n\0" as *const u8 as PCSTR,
        b"debug.c\0" as *const u8 as *const core::ffi::c_char,
        39 as core::ffi::c_int,
        (*QH).sqh.HwQH.Token.AsULONG,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"debug.c\0" as *const u8 as *const core::ffi::c_char,
            39 as core::ffi::c_int,
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB EHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBEHCI debugging functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbehci.h"
//
// //#define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// EHCI_DumpHwTD(IN PEHCI_HCD_TD TD)
// {
//     while (TD)
//     {
//         DPRINT(": TD                       - %p\n", TD);
//         DPRINT(": TD->PhysicalAddress      - %lx\n", TD->PhysicalAddress);
//         DPRINT(": TD->HwTD.NextTD          - %lx\n", TD->HwTD.NextTD);
//         DPRINT(": TD->HwTD.AlternateNextTD - %lx\n", TD->HwTD.AlternateNextTD);
//         DPRINT(": TD->HwTD.Token.AsULONG   - %lx\n", TD->HwTD.Token.AsULONG);
//
//         TD = TD->NextHcdTD;
//     }
// }
//
// VOID
// NTAPI
// EHCI_DumpHwQH(IN PEHCI_HCD_QH QH)
// {
//     if (!QH)
//         return;
//
//     DPRINT(": QH->sqh.HwQH.CurrentTD       - %lx\n", QH->sqh.HwQH.CurrentTD);
//     DPRINT(": QH->sqh.HwQH.NextTD          - %lx\n", QH->sqh.HwQH.NextTD);
//     DPRINT(": QH->sqh.HwQH.AlternateNextTD - %lx\n", QH->sqh.HwQH.AlternateNextTD);
//     DPRINT(": QH->sqh.HwQH.Token.AsULONG   - %lx\n", QH->sqh.HwQH.Token.AsULONG);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
