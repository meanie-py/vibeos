use core::ffi::c_void;
use ntkrnl_sys::base::*;
use crate::frldr::*;

// REGS / BYTEREGS / WORDREGS / Int386 — shared pc-hw real-mode surface
// from the freeldr-rust keystone (frldr.rs). REGS is a `union` (b/w/x/d
// aliases over the same 40-byte buffer), so field access uses `.b.field`
// / `.w.field` / `.x.field` — same shape as the C `REGS` typedef.
extern "C" {
    fn Int386(interrupt: i32, in_regs: *mut REGS, out_regs: *mut REGS);
}

// pcbios.h: #define EFLAGS_ZF HEX(40) — zero flag bit in EFLAGS.
const EFLAGS_ZF: u32 = 0x40;

static mut ExtendedKey: BOOLEAN = 0;
static mut ExtendedScanCode: CHAR = 0;

#[no_mangle]
pub unsafe extern "C" fn PcConsPutChar(Ch: i32) {
    let mut Regs: REGS = core::mem::zeroed();

    if Ch == b'\n' as i32 {
        PcConsPutChar(b'\r' as i32);
    }

    if Ch == b'\t' as i32 {
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        PcConsPutChar(b' ' as i32);
        return;
    }

    Regs.b.ah = 0x0E;
    Regs.b.al = Ch as u8;
    Regs.w.bx = 1;
    Int386(0x10, &mut Regs, &mut Regs);
}

#[no_mangle]
pub unsafe extern "C" fn PcConsKbHit() -> BOOLEAN {
    let mut Regs: REGS = core::mem::zeroed();

    Regs.b.ah = 0x01;
    Int386(0x16, &mut Regs, &mut Regs);

    if (Regs.x.eflags & EFLAGS_ZF) != 0 { 0 } else { 1 }
}

#[no_mangle]
pub unsafe extern "C" fn PcConsGetCh() -> i32 {
    let mut Regs: REGS = core::mem::zeroed();

    if ExtendedKey != 0 {
        ExtendedKey = 0;
        return ExtendedScanCode as i32;
    }

    Regs.b.ah = 0x00;
    Int386(0x16, &mut Regs, &mut Regs);

    if Regs.b.al == 0 {
        ExtendedKey = 1;
        ExtendedScanCode = Regs.b.ah as CHAR;
    }

    Regs.b.al as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  FreeLoader
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
//
// #include <freeldr.h>
//
// VOID
// PcConsPutChar(int Ch)
// {
//     REGS Regs;
//
//     /* If we are displaying a CR '\n' then do a LF also */
//     if (Ch == '\n')
//     {
//         /* Display the LF */
//         PcConsPutChar('\r');
//     }
//
//     /* If we are displaying a TAB '\t' then display 8 spaces ' ' */
//     if (Ch == '\t')
//     {
//         /* Display the 8 spaces ' ' */
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         PcConsPutChar(' ');
//         return;
//     }
//
//     /* Int 10h AH=0Eh
//      * VIDEO - TELETYPE OUTPUT
//      *
//      * AH = 0Eh
//      * AL = character to write
//      * BH = page number
//      * BL = foreground color (graphics modes only)
//      */
//     Regs.b.ah = 0x0E;
//     Regs.b.al = Ch;
//     Regs.w.bx = 1;
//     Int386(0x10, &Regs, &Regs);
// }
//
// BOOLEAN
// PcConsKbHit(VOID)
// {
//     REGS Regs;
//
//     /* Int 16h AH=01h
//      * KEYBOARD - CHECK FOR KEYSTROKE
//      *
//      * AH = 01h
//      * Return:
//      * ZF set if no keystroke available
//      * ZF clear if keystroke available
//      * AH = BIOS scan code
//      * AL = ASCII character
//      */
//     Regs.b.ah = 0x01;
//     Int386(0x16, &Regs, &Regs);
//
//     return !(Regs.x.eflags & EFLAGS_ZF);
// }
//
// int
// PcConsGetCh(void)
// {
//     REGS Regs;
//     static BOOLEAN ExtendedKey = FALSE;
//     static char ExtendedScanCode = 0;
//
//     /* If the last time we were called an
//      * extended key was pressed then return
//      * that keys scan code. */
//     if (ExtendedKey)
//     {
//         ExtendedKey = FALSE;
//         return ExtendedScanCode;
//     }
//
//     /* Int 16h AH=00h
//      * KEYBOARD - GET KEYSTROKE
//      *
//      * AH = 00h
//      * Return:
//      * AH = BIOS scan code
//      * AL = ASCII character
//      */
//     Regs.b.ah = 0x00;
//     Int386(0x16, &Regs, &Regs);
//
//     /* Check for an extended keystroke */
//     if (Regs.b.al == 0)
//     {
//         ExtendedKey = TRUE;
//         ExtendedScanCode = Regs.b.ah;
//     }
//
//     /* Return keystroke */
//     return Regs.b.al;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
