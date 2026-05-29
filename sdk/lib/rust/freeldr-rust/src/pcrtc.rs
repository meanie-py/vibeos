use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::{TIMEINFO};
use core::ffi::c_void;

extern "C" {
    fn Int386(interrupt: i32, in_regs: *mut REGS, out_regs: *mut REGS);
}

unsafe fn BCD_INT(Bcd: u8) -> u8 {
    ((Bcd & 0xF0) >> 4) * 10 + (Bcd & 0x0F)
}

#[no_mangle]
pub unsafe extern "C" fn PcGetTime() -> *mut TIMEINFO {
    static mut TimeInfo: TIMEINFO = TIMEINFO {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
    };
    let mut Regs: REGS = core::mem::zeroed();

    loop {
        Regs.x.eflags = 0;

        Regs.b.ah = 0x04;
        Int386(0x1A, &mut Regs, &mut Regs);

        if INT386_SUCCESS(Regs) == 0 {
            continue;
        }

        TimeInfo.Year = 100 * BCD_INT(Regs.b.ch) as u16 + BCD_INT(Regs.b.cl) as u16;
        TimeInfo.Month = BCD_INT(Regs.b.dh) as u16;
        TimeInfo.Day = BCD_INT(Regs.b.dl) as u16;

        Regs.x.eflags = 0;

        Regs.b.ah = 0x02;
        Int386(0x1A, &mut Regs, &mut Regs);

        if INT386_SUCCESS(Regs) == 0 {
            continue;
        }

        TimeInfo.Hour = BCD_INT(Regs.b.ch) as u16;
        TimeInfo.Minute = BCD_INT(Regs.b.cl) as u16;
        TimeInfo.Second = BCD_INT(Regs.b.dh) as u16;

        break;
    }
    &mut TimeInfo
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
// static
// UCHAR
// BCD_INT(
//     _In_ UCHAR Bcd)
// {
//     return ((Bcd & 0xF0) >> 4) * 10 + (Bcd & 0x0F);
// }
//
// TIMEINFO*
// PcGetTime(VOID)
// {
//     static TIMEINFO TimeInfo;
//     REGS Regs;
//
//     for (;;)
//     {
//         /* Some BIOSes, such as the 1998/07/25 system ROM
//          * in the Compaq Deskpro EP/SB, leave CF unchanged
//          * if successful, so CF should be cleared before
//          * calling this function. */
//         Regs.x.eflags = 0;
// //        __writeeflags(__readeflags() & ~EFLAGS_CF);
//
//         /* Int 1Ah AH=04h
//          * TIME - GET REAL-TIME CLOCK DATE (AT,XT286,PS)
//          *
//          * AH = 04h
//          * CF clear to avoid bug
//          * Return:
//          * CF clear if successful
//          * CH = century (BCD)
//          * CL = year (BCD)
//          * DH = month (BCD)
//          * DL = day (BCD)
//          * CF set on error
//          */
//         Regs.b.ah = 0x04;
//         Int386(0x1A, &Regs, &Regs);
//
//         if (!INT386_SUCCESS(Regs)) continue;
//
//         TimeInfo.Year = 100 * BCD_INT(Regs.b.ch) + BCD_INT(Regs.b.cl);
//         TimeInfo.Month = BCD_INT(Regs.b.dh);
//         TimeInfo.Day = BCD_INT(Regs.b.dl);
//
//         /* Some BIOSes leave CF unchanged if successful,
//          * so CF should be cleared before calling this function. */
//         Regs.x.eflags = 0;
// //        __writeeflags(__readeflags() & ~EFLAGS_CF);
//
//         /* Int 1Ah AH=02h
//          * TIME - GET REAL-TIME CLOCK TIME (AT,XT286,PS)
//          *
//          * AH = 02h
//          * CF clear to avoid bug
//          * Return:
//          * CF clear if successful
//          * CH = hour (BCD)
//          * CL = minutes (BCD)
//          * DH = seconds (BCD)
//          * DL = daylight savings flag (00h standard time, 01h daylight time)
//          * CF set on error (i.e. clock not running or in middle of update)
//          */
//         Regs.b.ah = 0x02;
//         Int386(0x1A, &Regs, &Regs);
//
//         if (!INT386_SUCCESS(Regs)) continue;
//
//         TimeInfo.Hour = BCD_INT(Regs.b.ch);
//         TimeInfo.Minute = BCD_INT(Regs.b.cl);
//         TimeInfo.Second = BCD_INT(Regs.b.dh);
//
//         break;
//     }
//     return &TimeInfo;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
