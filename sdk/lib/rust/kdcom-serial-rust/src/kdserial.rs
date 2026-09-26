use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;

use core::ffi::c_void;

use crate::KDDBGPRINT;
use crate::kddll_h::*;
// The transport's own internal byte primitives live in kdcom.rs.
use crate::kdcom::{KdpReceiveByte, KdpSendByte};

// SharedUserData: fixed-address KUSER_SHARED_DATA (see lib.rs). Explicit use
// shadows the ntkrnl_sys::globals glob (which is an unresolvable extern here).
use crate::SharedUserData;

// KD_DEBUGGER_NOT_PRESENT is a cross-DLL ntoskrnl DATA export. Declared via
// raw-dylib so rustc emits the `__imp_` thunk (a plain extern static won't link
// cross-DLL on i386). The kddll.h alias maps it to KdDebuggerNotPresent.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    #[allow(improper_ctypes)]
    static mut KdDebuggerNotPresent: BOOLEAN;
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpSendBuffer(
    Buffer: PVOID,
    Size: ULONG,
) {

    let mut ByteBuffer = Buffer as PUCHAR;
    let mut remaining = Size;

    while remaining > 0 {
        KdpSendByte(*ByteBuffer);
        ByteBuffer = ByteBuffer.offset(1);
        remaining -= 1;
    }
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpReceiveBuffer(
    Buffer: PVOID,
    Size: ULONG,
) -> KDP_STATUS {
    let mut ByteBuffer = Buffer as PUCHAR;
    let mut Byte: UCHAR = 0;
    let mut Status: KDP_STATUS;
    let mut SizeRemaining = Size;

    while SizeRemaining > 0 {
        SizeRemaining = SizeRemaining.wrapping_sub(1);

        Status = KdpReceiveByte(&mut Byte);
        if Status != KDP_PACKET_RECEIVED {
            return Status;
        }

        *ByteBuffer = Byte;
        ByteBuffer = ByteBuffer.offset(1);
    }

    KDP_PACKET_RECEIVED
}
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpReceivePacketLeader(
    PacketLeader: *mut u32
) -> KDP_STATUS {
    let mut Index: u8 = 0;
    let mut Byte: u8 = 0;
    let mut Buffer: [u8; 4] = [0; 4];
    let mut KdStatus: KDP_STATUS;

    Buffer[0] = 0;

    // Faithful translation of the C `do { ... } while (Index < 4)`. Each C
    // `continue` jumps to the loop condition test, so the equivalent here is a
    // `loop` whose body ends by checking `Index < 4` and breaking otherwise.
    loop {
        KdStatus = KdpReceiveByte(&mut Byte);

        if KdStatus == KDP_PACKET_TIMEOUT {
            if Buffer[0] == BREAKIN_PACKET_BYTE {
                return KDP_PACKET_RESEND;
            }

            return KDP_PACKET_TIMEOUT;
        }

        if KdStatus == KDP_PACKET_RECEIVED {
            if Byte == PACKET_LEADER_BYTE ||
                Byte == CONTROL_PACKET_LEADER_BYTE
            {
                if Byte != Buffer[0] {
                    Index = 0;
                }

                Buffer[Index as usize] = Byte;

                Index += 1;

                // C `continue` -> re-test the while condition.
                if Index < 4 { continue; } else { break; }
            }

            if Byte == BREAKIN_PACKET_BYTE {
                KDDBGPRINT!(b"BREAKIN_PACKET_BYTE\n\0".as_ptr());
                Index = 0;
                Buffer[0] = Byte;

                // C `continue` -> re-test the while condition.
                if Index < 4 { continue; } else { break; }
            }
        }

        Index = 0;
        Buffer[0] = 0;

        // Bottom of the C loop: `while (Index < 4)`.
        if Index < 4 { continue; } else { break; }
    }

    KdDebuggerNotPresent = 0u8;
    (*SharedUserData).KdDebuggerEnabled |= 0x00000002;

    *PacketLeader = (Buffer.as_ptr() as *const u32).read_unaligned();

    return KDP_PACKET_RECEIVED;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/kdserial.c
//  * PURPOSE:         Serial communication functions for the kernel debugger.
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include "kddll.h"
//
// /* FUNCTIONS ******************************************************************/
//
// /******************************************************************************
//  * \name KdpSendBuffer
//  * \brief Sends a buffer of data to the serial KD port.
//  * \param Buffer Pointer to the data.
//  * \param Size Size of data in bytes.
//  */
// VOID
// NTAPI
// KdpSendBuffer(
//     IN PVOID Buffer,
//     IN ULONG Size)
// {
//     PUCHAR ByteBuffer = Buffer;
//
//     while (Size-- > 0)
//     {
//         KdpSendByte(*ByteBuffer++);
//     }
// }
//
// /******************************************************************************
//  * \name KdpReceiveBuffer
//  * \brief Receives data from the KD port and fills a buffer.
//  * \param Buffer Pointer to a buffer that receives the data.
//  * \param Size Size of data to receive in bytes.
//  * \return KDP_PACKET_RECEIVED if successful.
//  *         KDP_PACKET_TIMEOUT if the receive timed out.
//  */
// KDP_STATUS
// NTAPI
// KdpReceiveBuffer(
//     OUT PVOID Buffer,
//     IN  ULONG Size)
// {
//     PUCHAR ByteBuffer = Buffer;
//     UCHAR Byte;
//     KDP_STATUS Status;
//
//     while (Size-- > 0)
//     {
//         /* Try to get a byte from the port */
//         Status = KdpReceiveByte(&Byte);
//         if (Status != KDP_PACKET_RECEIVED)
//             return Status;
//
//         *ByteBuffer++ = Byte;
//     }
//
//     return KDP_PACKET_RECEIVED;
// }
//
//
// /******************************************************************************
//  * \name KdpReceivePacketLeader
//  * \brief Receives a packet leader from the KD port.
//  * \param PacketLeader Pointer to an ULONG that receives the packet leader.
//  * \return KDP_PACKET_RECEIVED if successful.
//  *         KDP_PACKET_TIMEOUT if the receive timed out.
//  *         KDP_PACKET_RESEND if a breakin byte was detected.
//  */
// KDP_STATUS
// NTAPI
// KdpReceivePacketLeader(
//     OUT PULONG PacketLeader)
// {
//     UCHAR Index = 0, Byte, Buffer[4];
//     KDP_STATUS KdStatus;
//
//     /* Set first character to 0 */
//     Buffer[0] = 0;
//
//     do
//     {
//         /* Receive a single byte */
//         KdStatus = KdpReceiveByte(&Byte);
//
//         /* Check for timeout */
//         if (KdStatus == KDP_PACKET_TIMEOUT)
//         {
//             /* Check if we already got a breakin byte */
//             if (Buffer[0] == BREAKIN_PACKET_BYTE)
//             {
//                 return KDP_PACKET_RESEND;
//             }
//
//             /* Report timeout */
//             return KDP_PACKET_TIMEOUT;
//         }
//
//         /* Check if we received a byte */
//         if (KdStatus == KDP_PACKET_RECEIVED)
//         {
//             /* Check if this is a valid packet leader byte */
//             if (Byte == PACKET_LEADER_BYTE ||
//                 Byte == CONTROL_PACKET_LEADER_BYTE)
//             {
//                 /* Check if we match the first byte */
//                 if (Byte != Buffer[0])
//                 {
//                     /* No, this is the new byte 0! */
//                     Index = 0;
//                 }
//
//                 /* Store the byte in the buffer */
//                 Buffer[Index] = Byte;
//
//                 /* Continue with next byte */
//                 Index++;
//                 continue;
//             }
//
//             /* Check for breakin byte */
//             if (Byte == BREAKIN_PACKET_BYTE)
//             {
//                 KDDBGPRINT("BREAKIN_PACKET_BYTE\n");
//                 Index = 0;
//                 Buffer[0] = Byte;
//                 continue;
//             }
//         }
//
//         /* Restart */
//         Index = 0;
//         Buffer[0] = 0;
//     }
//     while (Index < 4);
//
//     /* Enable the debugger */
//     KD_DEBUGGER_NOT_PRESENT = FALSE;
//     SharedUserData->KdDebuggerEnabled |= 0x00000002;
//
//     /* Return the received packet leader */
//     *PacketLeader = *(PULONG)Buffer;
//
//     return KDP_PACKET_RECEIVED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
