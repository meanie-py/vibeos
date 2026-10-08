use ntkrnl_sys::base::*;
use crate::frldr::*;
// `Unused` was a bogus rustconv ddk import (it's a local variable name).
use core::ffi::c_void;

extern "C" {
    fn UefiVideoGetDisplaySize(Width: *mut u32, Height: *mut u32, Unused: *mut u32);
    fn FbConsScrollUp(Attr: u32);
    fn UefiVideoPutChar(c: i32, Attr: u32, X: u32, Y: u32);
    // cross-module data from uefildr.c.
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
}

// ui_keystone.rs owns the crate-shared ATTR/COLOR_*; ueficon only globs
// crate::frldr, so define what it needs file-local.
#[inline]
fn ATTR(fg: UCHAR, bg: UCHAR) -> UCHAR { (bg << 4) | fg }
const COLOR_BLACK: UCHAR = 0;
const COLOR_GRAY: UCHAR = 7; // nolint: const-contradiction (unrelated namespaces: an ICM colour type vs a console colour)

// keycodes.h scancodes returned by ConvertToBiosExtValue (UCHAR).
const KEY_EXTENDED: UCHAR = 0x00;
const KEY_DELETE: UCHAR = 0x53;
const KEY_HOME: UCHAR = 0x47;
const KEY_UP: UCHAR = 0x48;
const KEY_DOWN: UCHAR = 0x50;
const KEY_LEFT: UCHAR = 0x4B;
const KEY_RIGHT: UCHAR = 0x4D;
const KEY_ESC: UCHAR = 0x1B;
const KEY_END: UCHAR = 0x4F;
const KEY_F1: UCHAR = 0x3B;
const KEY_F2: UCHAR = 0x3C;
const KEY_F3: UCHAR = 0x3D;
const KEY_F4: UCHAR = 0x3E;
const KEY_F5: UCHAR = 0x3F;
const KEY_F6: UCHAR = 0x40;
const KEY_F7: UCHAR = 0x41;
const KEY_F8: UCHAR = 0x42;
const KEY_F9: UCHAR = 0x43;
const KEY_F10: UCHAR = 0x44;

// SimpleTextIn.h UEFI scan codes (matched against EFI_INPUT_KEY.ScanCode).
const SCAN_UP: UCHAR = 0x01;
const SCAN_DOWN: UCHAR = 0x02;
const SCAN_RIGHT: UCHAR = 0x03;
const SCAN_LEFT: UCHAR = 0x04;
const SCAN_HOME: UCHAR = 0x05;
const SCAN_END: UCHAR = 0x06;
const SCAN_DELETE: UCHAR = 0x08;
const SCAN_F1: UCHAR = 0x0B;
const SCAN_F2: UCHAR = 0x0C;
const SCAN_F3: UCHAR = 0x0D;
const SCAN_F4: UCHAR = 0x0E;
const SCAN_F5: UCHAR = 0x0F;
const SCAN_F6: UCHAR = 0x10;
const SCAN_F7: UCHAR = 0x11;
const SCAN_F8: UCHAR = 0x12;
const SCAN_F9: UCHAR = 0x13;
const SCAN_F10: UCHAR = 0x14;
const SCAN_ESC: UCHAR = 0x17;

// ueficon.c file-scope globals. MachDefaultTextColor has external linkage
// (ui_keystone.rs only extern-declares it) → this TU is the definer.
#[no_mangle]
pub static mut MachDefaultTextColor: UCHAR = COLOR_GRAY;
static mut CurrentCursorX: ULONG = 0;
static mut CurrentCursorY: ULONG = 0;
static mut CurrentAttr: UCHAR = (COLOR_BLACK << 4) | COLOR_GRAY;
static mut Key: EFI_INPUT_KEY = EFI_INPUT_KEY { ScanCode: 0, UnicodeChar: 0 };
static mut ExtendedKey: BOOLEAN = 0;
static mut ExtendedScanCode: i8 = 0;

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiConsPutChar(c: i32) {
    let mut Width: u32 = 0;
    let mut Height: u32 = 0;
    let mut Unused: u32 = 0;
    let NeedScroll: u8;

    UefiVideoGetDisplaySize(&mut Width, &mut Height, &mut Unused);

    NeedScroll = if CurrentCursorY >= Height { 1u8 } else { 0u8 };
    if NeedScroll != 0 {
        FbConsScrollUp((CurrentAttr) as u32 );
        CurrentCursorY -= 1;
    }
    if c == b'\r' as i32 {
        CurrentCursorX = 0;
    } else if c == b'\n' as i32 {
        CurrentCursorX = 0;
        if NeedScroll == 0 {
            CurrentCursorY += 1;
        }
    } else if c == b'\t' as i32 {
        CurrentCursorX = (CurrentCursorX + 8) & !7;
    } else {
        UefiVideoPutChar(c, (CurrentAttr) as u32 , CurrentCursorX, CurrentCursorY);
        CurrentCursorX += 1;
    }
    if CurrentCursorX >= Width {
        CurrentCursorX = 0;
        CurrentCursorY += 1;
    }
}

unsafe fn ConvertToBiosExtValue(KeyIn: UCHAR) -> UCHAR {
    match KeyIn {
        SCAN_UP => KEY_UP,
        SCAN_DOWN => KEY_DOWN,
        SCAN_RIGHT => KEY_RIGHT,
        SCAN_LEFT => KEY_LEFT,
        SCAN_HOME => KEY_HOME,
        SCAN_END => KEY_END,
        SCAN_DELETE => KEY_DELETE,
        SCAN_F1 => KEY_F1,
        SCAN_F2 => KEY_F2,
        SCAN_F3 => KEY_F3,
        SCAN_F4 => KEY_F4,
        SCAN_F5 => KEY_F5,
        SCAN_F6 => KEY_F6,
        SCAN_F7 => KEY_F7,
        SCAN_F8 => KEY_F8,
        SCAN_F9 => KEY_F9,
        SCAN_F10 => KEY_F10,
        SCAN_ESC => KEY_ESC,
        _ => 0,
    }
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiConsKbHit() -> u8 {
    // C shares the file-scope `static EFI_INPUT_KEY Key` across KbHit/GetCh.
    //
    // The C upstream uses `!= EFI_NOT_READY` as the "key available" predicate,
    // which is wrong: only `EFI_SUCCESS` (0) means a valid key was read.
    // Some firmwares (e.g. OVMF) appear to return success-with-empty-key in
    // certain states, so additionally require non-zero key data. Otherwise
    // every poll returns "key pressed" and the boot-menu timeout never ticks.
    let status = ((*(*GlobalSystemTable).ConIn).ReadKeyStroke.unwrap())(
        (*GlobalSystemTable).ConIn,
        core::ptr::addr_of_mut!(Key),
    );
    if status == EFI_SUCCESS && (Key.UnicodeChar != 0 || Key.ScanCode != 0) {
        1u8
    } else {
        Key.UnicodeChar = 0;
        Key.ScanCode = 0;
        0u8
    }
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiConsGetCh() -> i32 {
    let mut KeyOutput: UCHAR = 0;
    // Say so before blocking: on a headless machine (a server with no display or
    // keyboard) nobody will press one, and otherwise the boot just goes silent.
    crate::frldr::dbgcon_str(b"[FLDR] waiting for a key on the UEFI console\r\n");

    if ExtendedKey != 0 {
        ExtendedKey = 0u8;
        return ExtendedScanCode as i32;
    }

    if Key.UnicodeChar != 0 {
        KeyOutput = Key.UnicodeChar as UCHAR;
    } else {
        ExtendedKey = 1u8;
        ExtendedScanCode = (ConvertToBiosExtValue(Key.ScanCode as UCHAR)) as i8 ;
        KeyOutput = KEY_EXTENDED;
    }

    Key.UnicodeChar = 0;
    Key.ScanCode = 0;
    return KeyOutput as i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Console output
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// #include <uefildr.h>
// #include "../vidfb.h"
//
// /* GLOBALS ********************************************************************/
//
// UCHAR MachDefaultTextColor = COLOR_GRAY;
//
// static unsigned CurrentCursorX = 0;
// static unsigned CurrentCursorY = 0;
// static UCHAR CurrentAttr = ATTR(COLOR_GRAY, COLOR_BLACK);
//
// extern EFI_SYSTEM_TABLE* GlobalSystemTable;
// static EFI_INPUT_KEY Key;
// static BOOLEAN ExtendedKey = FALSE;
// static char ExtendedScanCode = 0;
//
// /* FUNCTIONS ******************************************************************/
//
// VOID
// UefiConsPutChar(int c)
// {
//     ULONG Width, Height, Unused;
//     BOOLEAN NeedScroll;
//
//     UefiVideoGetDisplaySize(&Width, &Height, &Unused);
//
//     NeedScroll = (CurrentCursorY >= Height);
//     if (NeedScroll)
//     {
//         FbConsScrollUp(CurrentAttr);
//         --CurrentCursorY;
//     }
//     if (c == '\r')
//     {
//         CurrentCursorX = 0;
//     }
//     else if (c == '\n')
//     {
//         CurrentCursorX = 0;
//         if (!NeedScroll)
//             ++CurrentCursorY;
//     }
//     else if (c == '\t')
//     {
//         CurrentCursorX = (CurrentCursorX + 8) & ~7;
//     }
//     else
//     {
//         UefiVideoPutChar(c, CurrentAttr, CurrentCursorX, CurrentCursorY);
//         CurrentCursorX++;
//     }
//     if (CurrentCursorX >= Width)
//     {
//         CurrentCursorX = 0;
//         CurrentCursorY++;
//     }
// }
//
// static
// UCHAR
// ConvertToBiosExtValue(UCHAR KeyIn)
// {
//     switch (KeyIn)
//     {
//         case SCAN_UP:
//             return KEY_UP;
//         case SCAN_DOWN:
//             return KEY_DOWN;
//         case SCAN_RIGHT:
//             return KEY_RIGHT;
//         case SCAN_LEFT:
//             return KEY_LEFT;
//         case SCAN_HOME:
//             return KEY_HOME;
//         case SCAN_END:
//             return KEY_END;
//
//         // case SCAN_INSERT:
//         //     break;
//
//         case SCAN_DELETE:
//             return KEY_DELETE;
//
//         // case SCAN_PAGE_UP:
//         // case SCAN_PAGE_DOWN:
//         //     break;
//
//         case SCAN_F1:
//             return KEY_F1;
//         case SCAN_F2:
//             return KEY_F2;
//         case SCAN_F3:
//             return KEY_F3;
//         case SCAN_F4:
//             return KEY_F4;
//         case SCAN_F5:
//             return KEY_F5;
//         case SCAN_F6:
//             return KEY_F6;
//         case SCAN_F7:
//             return KEY_F7;
//         case SCAN_F8:
//             return KEY_F8;
//         case SCAN_F9:
//             return KEY_F9;
//         case SCAN_F10:
//             return KEY_F10;
//         case SCAN_ESC:
//             return KEY_ESC;
//     }
//     return 0;
// }
//
// BOOLEAN
// UefiConsKbHit(VOID)
// {
//     return (GlobalSystemTable->ConIn->ReadKeyStroke(GlobalSystemTable->ConIn, &Key) != EFI_NOT_READY);
// }
//
// int
// UefiConsGetCh(VOID)
// {
//     UCHAR KeyOutput = 0;
//
//     /* If an extended key press was detected the last time we were called
//      * then return the scan code of that key. */
//     if (ExtendedKey)
//     {
//         ExtendedKey = FALSE;
//         return ExtendedScanCode;
//     }
//
//     if (Key.UnicodeChar != 0)
//     {
//         KeyOutput = Key.UnicodeChar;
//     }
//     else
//     {
//         ExtendedKey = TRUE;
//         ExtendedScanCode = ConvertToBiosExtValue(Key.ScanCode);
//         KeyOutput = KEY_EXTENDED;
//     }
//
//     /* UEFI will stack input requests, we have to clear it */
//     Key.UnicodeChar = 0;
//     Key.ScanCode = 0;
//     return KeyOutput;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
