use core::ffi::c_void;
use ntkrnl_sys::base::*;
use crate::frldr::*;

extern "C" {
    fn StallExecutionProcessor(Microseconds: ULONG);
}

// ---- file-scope preamble (translated) ----
pub const CLOCK_TICK_RATE: i32 = 1193182;

unsafe fn Sound(Frequency: u16) {
    let Scale: u16;

    if Frequency == 0 {
        WRITE_PORT_UCHAR(0x61 as *mut u8, READ_PORT_UCHAR(0x61 as *mut u8) & !3);
        return;
    }

    Scale = (CLOCK_TICK_RATE as u32 / Frequency as u32) as u16;
    WRITE_PORT_UCHAR(0x43 as *mut u8, 0xB6);
    WRITE_PORT_UCHAR(0x42 as *mut u8, (Scale & 0xFF) as u8);
    WRITE_PORT_UCHAR(0x42 as *mut u8, (Scale >> 8) as u8);
    WRITE_PORT_UCHAR(0x61 as *mut u8, READ_PORT_UCHAR(0x61 as *mut u8) | 3);
}

#[no_mangle]
pub unsafe extern "C" fn PcBeep() {
    Sound(700);
    StallExecutionProcessor(100000);
    Sound(0);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Hardware-specific beep routine
//  * COPYRIGHT:   Copyright 1998-2003 Brian Palmer (brianp@reactos.org)
//  */
//
// #include <freeldr.h>
//
// #define CLOCK_TICK_RATE 1193182
//
// static VOID
// Sound(USHORT Frequency)
// {
//     USHORT Scale;
//
//     if (Frequency == 0)
//     {
//         WRITE_PORT_UCHAR((PUCHAR)0x61, READ_PORT_UCHAR((PUCHAR)0x61) & ~3);
//         return;
//     }
//
//     Scale = CLOCK_TICK_RATE / Frequency;
//     WRITE_PORT_UCHAR((PUCHAR)0x43, 0xB6);
//     WRITE_PORT_UCHAR((PUCHAR)0x42, Scale & 0xFF);
//     WRITE_PORT_UCHAR((PUCHAR)0x42, Scale >> 8);
//     WRITE_PORT_UCHAR((PUCHAR)0x61, READ_PORT_UCHAR((PUCHAR)0x61) | 3);
// }
//
// VOID PcBeep(VOID)
// {
//     Sound(700);
//     StallExecutionProcessor(100000);
//     Sound(0);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
