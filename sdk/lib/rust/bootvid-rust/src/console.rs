//! AI-translated from drivers/base/bootvid/ (console); extern blocks
//! stripped — header inlines/macros/FFI provided by crate::pc.

use crate::pc::*;
#[allow(unused_imports)] use crate::fontdata::*;
#[allow(unused_imports)] use crate::bootdata::*;
#[allow(unused_imports)] use crate::common::*;
#[allow(unused_imports)] use crate::vga::*;
#[allow(unused_imports)] use crate::bootvid::*;
use ntkrnl_sys::base::*;
use core::ffi::c_void;

// console.c `static BOOLEAN ClearRow = FALSE;`
static mut ClearRow: BOOLEAN = 0;

#[no_mangle]
pub unsafe extern "system" fn VidResetDisplay(
    SetMode: BOOLEAN
) {
    VidpCurrentX = 0;
    VidpCurrentY = 0;

    
    ResetDisplay(SetMode);
}

#[no_mangle]
pub unsafe extern "system" fn VidSetTextColor(
    Color: ULONG,
) -> ULONG {
    let OldColor: ULONG;

    OldColor = VidpTextColor as ULONG;
    VidpTextColor = Color as UCHAR;
    return OldColor;
}

#[no_mangle]
pub unsafe extern "system" fn VidSetScrollRegion(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
) {
    VidpScrollRegion.Left = Left;
    VidpScrollRegion.Top = Top;
    VidpScrollRegion.Right = Right;
    VidpScrollRegion.Bottom = Bottom;

    VidpCurrentX = Left;
    VidpCurrentY = Top;
}

#[no_mangle]
pub unsafe extern "system" fn VidDisplayStringXY(
    String: PCSTR,
    Left: u32,
    Top: u32,
    Transparent: u8,
) {

    let BackColor = if Transparent != 0 { BV_COLOR_NONE } else { BV_COLOR_LIGHT_CYAN };

    let mut String = String;
    let mut Left = Left;
    while *String != 0 {
        DisplayCharacter(*String as i8, Left, Top, BV_COLOR_LIGHT_BLUE, BackColor);
        String = String.offset(1);
        Left += BOOTCHAR_WIDTH;
    }
}

#[no_mangle]
pub unsafe extern "system" fn VidDisplayString(String: PCSTR) {

    let mut s = String;
    while *s != 0 {
        if *s == b'\n' as CHAR {
            VidpCurrentY += BOOTCHAR_HEIGHT + 1;
            if VidpCurrentY + BOOTCHAR_HEIGHT > VidpScrollRegion.Bottom {
                DoScroll(BOOTCHAR_HEIGHT + 1);
                VidpCurrentY -= BOOTCHAR_HEIGHT + 1;
                PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, 1u8);
            } else {
                PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, 0u8);
            }

            VidpCurrentX = VidpScrollRegion.Left;
            ClearRow = 0u8;
        } else if *s == b'\r' as CHAR {
            VidpCurrentX = VidpScrollRegion.Left;

            if *s.offset(1) != b'\n' as CHAR {
                ClearRow = 1u8;
            }
        } else {
            if ClearRow != 0 {
                PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, 1u8);
                ClearRow = 0u8;
            }

            DisplayCharacter(*s, VidpCurrentX, VidpCurrentY, VidpTextColor as ULONG, BV_COLOR_NONE);
            VidpCurrentX += BOOTCHAR_WIDTH;

            if VidpCurrentX + BOOTCHAR_WIDTH - 1 > VidpScrollRegion.Right {
                VidpCurrentY += BOOTCHAR_HEIGHT + 1;
                if VidpCurrentY + BOOTCHAR_HEIGHT > VidpScrollRegion.Bottom {
                    DoScroll(BOOTCHAR_HEIGHT + 1);
                    VidpCurrentY -= BOOTCHAR_HEIGHT + 1;
                    PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, 1u8);
                } else {
                    PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, 0u8);
                }

                VidpCurrentX = VidpScrollRegion.Left;
            }
        }

        s = s.offset(1);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Boot Video Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Platform-independent console functionality
//  * COPYRIGHT:   Copyright 2010 Gregor Schneider <gregor.schneider@reactos.org>
//  *              Copyright 2011 Rafal Harabien <rafalh@reactos.org>
//  *              Copyright 2020 Stanislav Motylkov <x86corez@gmail.com>
//  */
//
// #include "precomp.h"
//
// /* GLOBALS ********************************************************************/
//
// UCHAR VidpTextColor = BV_COLOR_WHITE;
// ULONG VidpCurrentX = 0;
// ULONG VidpCurrentY = 0;
// URECT VidpScrollRegion = {0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1};
//
// static BOOLEAN ClearRow = FALSE;
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// VOID
// NTAPI
// VidResetDisplay(
//     _In_ BOOLEAN SetMode)
// {
//     /* Clear the current position */
//     VidpCurrentX = 0;
//     VidpCurrentY = 0;
//
//     /* Invoke the hardware-specific routine */
//     ResetDisplay(SetMode);
// }
//
// ULONG
// NTAPI
// VidSetTextColor(
//     _In_ ULONG Color)
// {
//     ULONG OldColor;
//
//     /* Save the old color and set the new one */
//     OldColor = VidpTextColor;
//     VidpTextColor = Color;
//     return OldColor;
// }
//
// VOID
// NTAPI
// VidSetScrollRegion(
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ ULONG Right,
//     _In_ ULONG Bottom)
// {
//     /* Assert alignment */
//     ASSERT((Left % BOOTCHAR_WIDTH) == 0);
//     ASSERT((Right % BOOTCHAR_WIDTH) == BOOTCHAR_WIDTH - 1);
//
//     /* Set the scroll region */
//     VidpScrollRegion.Left = Left;
//     VidpScrollRegion.Top  = Top;
//     VidpScrollRegion.Right  = Right;
//     VidpScrollRegion.Bottom = Bottom;
//
//     /* Set the current X and Y */
//     VidpCurrentX = Left;
//     VidpCurrentY = Top;
// }
//
// VOID
// NTAPI
// VidDisplayStringXY(
//     _In_ PCSTR String,
//     _In_ ULONG Left,
//     _In_ ULONG Top,
//     _In_ BOOLEAN Transparent)
// {
//     ULONG BackColor;
//
//     /*
//      * If the caller wanted transparent, then send the special value (16),
//      * else use our default and call the helper routine.
//      */
//     BackColor = Transparent ? BV_COLOR_NONE : BV_COLOR_LIGHT_CYAN;
//
//     /* Loop every character and adjust the position */
//     for (; *String; ++String, Left += BOOTCHAR_WIDTH)
//     {
//         /* Display a character */
//         DisplayCharacter(*String, Left, Top, BV_COLOR_LIGHT_BLUE, BackColor);
//     }
// }
//
// VOID
// NTAPI
// VidDisplayString(
//     _In_ PCSTR String)
// {
//     /* Start looping the string */
//     for (; *String; ++String)
//     {
//         /* Treat new-line separately */
//         if (*String == '\n')
//         {
//             /* Modify Y position */
//             VidpCurrentY += BOOTCHAR_HEIGHT + 1;
//             if (VidpCurrentY + BOOTCHAR_HEIGHT > VidpScrollRegion.Bottom)
//             {
//                 /* Scroll the view and clear the current row */
//                 DoScroll(BOOTCHAR_HEIGHT + 1);
//                 VidpCurrentY -= BOOTCHAR_HEIGHT + 1;
//                 PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, TRUE);
//             }
//             else
//             {
//                 /* Preserve the current row */
//                 PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, FALSE);
//             }
//
//             /* Update current X */
//             VidpCurrentX = VidpScrollRegion.Left;
//
//             /* No need to clear this row */
//             ClearRow = FALSE;
//         }
//         else if (*String == '\r')
//         {
//             /* Update current X */
//             VidpCurrentX = VidpScrollRegion.Left;
//
//             /* If a new-line does not follow we will clear the current row */
//             if (String[1] != '\n')
//                 ClearRow = TRUE;
//         }
//         else
//         {
//             /* Clear the current row if we had a return-carriage without a new-line */
//             if (ClearRow)
//             {
//                 PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, TRUE);
//                 ClearRow = FALSE;
//             }
//
//             /* Display this character */
//             DisplayCharacter(*String, VidpCurrentX, VidpCurrentY, VidpTextColor, BV_COLOR_NONE);
//             VidpCurrentX += BOOTCHAR_WIDTH;
//
//             /* Check if we should scroll */
//             if (VidpCurrentX + BOOTCHAR_WIDTH - 1 > VidpScrollRegion.Right)
//             {
//                 /* Update Y position and check if we should scroll it */
//                 VidpCurrentY += BOOTCHAR_HEIGHT + 1;
//                 if (VidpCurrentY + BOOTCHAR_HEIGHT > VidpScrollRegion.Bottom)
//                 {
//                     /* Scroll the view and clear the current row */
//                     DoScroll(BOOTCHAR_HEIGHT + 1);
//                     VidpCurrentY -= BOOTCHAR_HEIGHT + 1;
//                     PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, TRUE);
//                 }
//                 else
//                 {
//                     /* Preserve the current row */
//                     PreserveRow(VidpCurrentY, BOOTCHAR_HEIGHT + 1, FALSE);
//                 }
//
//                 /* Update current X */
//                 VidpCurrentX = VidpScrollRegion.Left;
//             }
//         }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
