use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KDSTATUS, PKD_CONTEXT};
use core::ffi::c_void;

use crate::KDDBGPRINT;

extern "C" {
    fn KdpPollByte(byte: *mut UCHAR) -> KDSTATUS;
    fn KdpReceiveByte(byte: *mut UCHAR) -> KDSTATUS;
    fn KdpSendByte(byte: UCHAR) -> KDSTATUS;
}

// ---- file-scope preamble (translated) ----
pub static mut gdb_input: [CHAR; 0x1000] = [0; 0x1000];
#[no_mangle]
pub unsafe extern "C" fn hex_value(ch: CHAR) -> CHAR {
    if (ch >= b'0' as CHAR) && (ch <= b'9' as CHAR) {
        return ch - b'0' as CHAR;
    }

    if (ch >= b'a' as CHAR) && (ch <= b'f' as CHAR) {
        return ch - b'a' as CHAR + 10;
    }

    if (ch >= b'A' as CHAR) && (ch <= b'F' as CHAR) {
        return ch - b'A' as CHAR + 10;
    }

    return -1i8 as CHAR;
}
#[no_mangle]
pub unsafe extern "system" fn gdb_receive_packet(KdContext: PKD_CONTEXT) -> KDSTATUS {

    let mut ByteBuffer: *mut UCHAR;
    let mut Byte: UCHAR = 0;
    let mut Status: KDSTATUS;
    let mut CheckSum: CHAR;
    let mut ReceivedCheckSum: CHAR;

    loop {
        Status = KdpReceiveByte(&mut Byte);
        if Status != KdPacketReceived {
            return Status;
        }
        if Byte == b'$' {
            break;
        }
    }

    'get_packet: loop {
        CheckSum = 0;
        ByteBuffer = gdb_input.as_mut_ptr() as *mut UCHAR;

        loop {
            Status = KdpReceiveByte(&mut Byte);
            if Status != KdPacketReceived {
                return Status;
            }

            if Byte == b'#' {
                *ByteBuffer = b'\0';
                break;
            }
            CheckSum = CheckSum.wrapping_add(Byte as CHAR);

            if Byte == 0x7d {
                Status = KdpReceiveByte(&mut Byte);
                if Status != KdPacketReceived {
                    return Status;
                }
                CheckSum = CheckSum.wrapping_add(Byte as CHAR);
                Byte ^= 0x20;
            }
            *ByteBuffer = Byte;
            ByteBuffer = ByteBuffer.offset(1);
        }

        /* Get Check sum (two bytes). C `goto end` on failure → fall through
         * to the checksum compare with the partial ReceivedCheckSum, which
         * mismatches and triggers a resend. */
        ReceivedCheckSum = 0;
        Status = KdpReceiveByte(&mut Byte);
        if Status == KdPacketReceived {
            ReceivedCheckSum = hex_value(Byte as CHAR) << 4;

            Status = KdpReceiveByte(&mut Byte);
            if Status == KdPacketReceived {
                ReceivedCheckSum = ReceivedCheckSum.wrapping_add(hex_value(Byte as CHAR));
            }
        }

        // end:
        if ReceivedCheckSum != CheckSum {
            /* Do not acknowledge to GDB */
            KDDBGPRINT!(b"Check sums don't match!\0".as_ptr() as *const CHAR);
            KdpSendByte(b'-');
            return KdPacketNeedsResend;
        }

        /* Ensure there is nothing left in the pipe */
        let mut restart = false;
        while KdpPollByte(&mut Byte) == KdPacketReceived {
            match Byte {
                b'$' => {
                    KDDBGPRINT!(b"Received new packet just after %s.\n\0".as_ptr() as *const CHAR, gdb_input.as_ptr());
                    restart = true;
                    break;
                }
                0x03 => {
                    (*KdContext).KdpControlCPending = 1u8;
                }
                _ => {}
            }
        }
        if restart {
            continue 'get_packet;
        }

        /* Acknowledge */
        KdpSendByte(b'+');
        return KdPacketReceived;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/gdb_receive.c
//  * PURPOSE:         Base functions for the kernel debugger.
//  */
//
// #include "kdgdb.h"
//
// /* GLOBALS ********************************************************************/
// CHAR gdb_input[0x1000];
//
// /* GLOBAL FUNCTIONS ***********************************************************/
// char
// hex_value(char ch)
// {
//     if ((ch >= '0') && (ch <= '9'))
//         return (ch - '0');
//
//     if ((ch >= 'a') && (ch <= 'f'))
//         return (ch - 'a' + 10);
//
//     if ((ch >= 'A') && (ch <= 'F'))
//         return (ch - 'A' + 10);
//
//     return -1;
// }
//
// KDSTATUS
// NTAPI
// gdb_receive_packet( PKD_CONTEXT KdContext)
// {
//     UCHAR* ByteBuffer;
//     UCHAR Byte;
//     KDSTATUS Status;
//     CHAR CheckSum, ReceivedCheckSum;
//
//     do
//     {
//         Status = KdpReceiveByte(&Byte);
//         if (Status != KdPacketReceived)
//             return Status;
//     } while (Byte != '$');
//
// get_packet:
//     CheckSum = 0;
//     ByteBuffer = (UCHAR*)gdb_input;
//
//     while (TRUE)
//     {
//         /* Try to get a byte from the port */
//         Status = KdpReceiveByte(&Byte);
//         if (Status != KdPacketReceived)
//             return Status;
//
//         if (Byte == '#')
//         {
//             *ByteBuffer = '\0';
//             break;
//         }
//         CheckSum += (CHAR)Byte;
//
//         /* See if we should escape */
//         if (Byte == 0x7d)
//         {
//             Status = KdpReceiveByte(&Byte);
//             if (Status != KdPacketReceived)
//                 return Status;
//             CheckSum += (CHAR)Byte;
//             Byte ^= 0x20;
//         }
//         *ByteBuffer++ = Byte;
//     }
//
//     /* Get Check sum (two bytes) */
//     Status = KdpReceiveByte(&Byte);
//     if (Status != KdPacketReceived)
//         goto end;
//     ReceivedCheckSum = hex_value(Byte) << 4;
//
//     Status = KdpReceiveByte(&Byte);
//     if (Status != KdPacketReceived)
//         goto end;
//     ReceivedCheckSum += hex_value(Byte);
//
// end:
//     if (ReceivedCheckSum != CheckSum)
//     {
//         /* Do not acknowledge to GDB */
//         KDDBGPRINT!("Check sums don't match!");
//         KdpSendByte('-');
//         return KdPacketNeedsResend;
//     }
//
//     /* Ensure there is nothing left in the pipe */
//     while (KdpPollByte(&Byte) == KdPacketReceived)
//     {
//         switch (Byte)
//         {
//             case '$':
//                 KDDBGPRINT!("Received new packet just after %s.\n", gdb_input);
//                 goto get_packet;
//             case 0x03:
//                 KdContext->KdpControlCPending = TRUE;
//                 break;
//         }
//     }
//
//     /* Acknowledge */
//     KdpSendByte('+');
//
//     return KdPacketReceived;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
