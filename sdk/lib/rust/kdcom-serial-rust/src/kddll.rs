use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{KD_PACKET, PKD_CONTEXT, PSTRING};
use core::ffi::c_void;

use crate::KDDBGPRINT;
use crate::kddll_h::*;
// The transport's own primitives translated into sibling modules.
use crate::kdserial::{KdpReceiveBuffer, KdpReceivePacketLeader, KdpSendBuffer};
use crate::kdcom::{KdpPollBreakIn, KdpSendByte};
// SharedUserData: fixed-address KUSER_SHARED_DATA (see lib.rs). Explicit use
// shadows the ntkrnl_sys::globals glob (an unresolvable extern here).
use crate::SharedUserData;

// KD_DEBUGGER_NOT_PRESENT (kddll.h alias) -> ntoskrnl DATA export. raw-dylib so
// rustc emits the `__imp_` thunk for cross-DLL linkage on i386.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    #[allow(improper_ctypes)]
    static mut KdDebuggerNotPresent: BOOLEAN;
}

// ---- file-scope preamble (translated) ----
pub static mut CurrentPacketId: ULONG = INITIAL_PACKET_ID | SYNC_PACKET_ID;
pub static mut RemotePacketId: ULONG = INITIAL_PACKET_ID;
#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpCalculateChecksum(
    Buffer: PVOID,
    mut Length: ULONG,
) -> ULONG {
    let mut ByteBuffer = Buffer as PUCHAR;
    let mut Checksum: ULONG = 0;

    while Length > 0 {
        Length = Length.wrapping_sub(1);
        Checksum = Checksum.wrapping_add(*ByteBuffer as ULONG);
        ByteBuffer = ByteBuffer.offset(1);
    }
    Checksum
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdpSendControlPacket(
    PacketType: u16,
    PacketId: u32,
) {
    let mut Packet: KD_PACKET = core::mem::zeroed();

    Packet.PacketLeader = CONTROL_PACKET_LEADER;
    Packet.PacketId = PacketId;
    Packet.ByteCount = 0;
    Packet.Checksum = 0;
    Packet.PacketType = PacketType;

    KdpSendBuffer(&mut Packet as *mut _ as PVOID, core::mem::size_of::<KD_PACKET>() as u32);
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdReceivePacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    DataLength: PULONG,
    KdContext: PKD_CONTEXT,
) -> KDP_STATUS {
    let mut Byte: UCHAR = 0;
    let mut KdStatus: KDP_STATUS;
    let mut Packet: KD_PACKET = core::mem::zeroed();
    let mut Checksum: ULONG;

    if PacketType == PACKET_TYPE_KD_POLL_BREAKIN {
        return KdpPollBreakIn();
    }

    loop {
        KdStatus = KdpReceivePacketLeader(core::ptr::addr_of_mut!(Packet.PacketLeader));
        if KdStatus != KDP_PACKET_RECEIVED {
            if KdStatus == KDP_PACKET_RESEND {
                (*KdContext).KdpControlCPending = TRUE;
            }
            return KdStatus;
        }

        KdStatus = KdpReceiveBuffer(
            core::ptr::addr_of_mut!(Packet.PacketType) as PVOID,
            core::mem::size_of::<USHORT>() as ULONG,
        );
        if KdStatus != KDP_PACKET_RECEIVED {
            return KdStatus;
        }

        if Packet.PacketLeader == CONTROL_PACKET_LEADER &&
            Packet.PacketType as ULONG == PACKET_TYPE_KD_RESEND
        {
            return KDP_PACKET_RESEND;
        }

        KdStatus = KdpReceiveBuffer(
            core::ptr::addr_of_mut!(Packet.ByteCount) as PVOID,
            core::mem::size_of::<USHORT>() as ULONG,
        );
        if KdStatus != KDP_PACKET_RECEIVED {
            return KdStatus;
        }

        KdStatus = KdpReceiveBuffer(
            core::ptr::addr_of_mut!(Packet.PacketId) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if KdStatus != KDP_PACKET_RECEIVED {
            return KdStatus;
        }

        KdStatus = KdpReceiveBuffer(
            core::ptr::addr_of_mut!(Packet.Checksum) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
        );
        if KdStatus != KDP_PACKET_RECEIVED {
            return KdStatus;
        }

        if Packet.PacketLeader == CONTROL_PACKET_LEADER {
            match Packet.PacketType as ULONG {
                PACKET_TYPE_KD_ACKNOWLEDGE => {
                    if PacketType == PACKET_TYPE_KD_ACKNOWLEDGE &&
                        Packet.PacketId == (CurrentPacketId & !SYNC_PACKET_ID)
                    {
                        CurrentPacketId ^= 1;
                        return KDP_PACKET_RECEIVED;
                    }
                    continue;
                }
                PACKET_TYPE_KD_RESET => {
                    KDDBGPRINT!(b"KdReceivePacket - got PACKET_TYPE_KD_RESET\n\0".as_ptr() as PCHAR);
                    CurrentPacketId = INITIAL_PACKET_ID;
                    RemotePacketId = INITIAL_PACKET_ID;
                    KdpSendControlPacket(PACKET_TYPE_KD_RESET as USHORT, 0);
                    return KDP_PACKET_RESEND;
                }
                PACKET_TYPE_KD_RESEND => {
                    KDDBGPRINT!(b"KdReceivePacket - got PACKET_TYPE_KD_RESEND\n\0".as_ptr() as PCHAR);
                    return KDP_PACKET_RESEND;
                }
                _ => {
                    KDDBGPRINT!(b"KdReceivePacket - got unknown control packet\n\0".as_ptr() as PCHAR);
                    continue;
                }
            }
        }

        if PacketType == PACKET_TYPE_KD_ACKNOWLEDGE {
            KdpSendControlPacket(PACKET_TYPE_KD_RESEND as USHORT, 0);
            CurrentPacketId ^= 1;
            return KDP_PACKET_RECEIVED;
        }

        (*MessageHeader).Length = (*MessageHeader).MaximumLength;

        if Packet.ByteCount < (*MessageHeader).Length as USHORT ||
            Packet.ByteCount > PACKET_MAX_SIZE as USHORT
        {
            KDDBGPRINT!(b"KdReceivePacket - too few data (%d) for type %d\n\0".as_ptr() as PCHAR);
            (*MessageHeader).Length = Packet.ByteCount;
            KdpSendControlPacket(PACKET_TYPE_KD_RESEND as USHORT, 0);
            continue;
        }

        KdStatus = KdpReceiveBuffer(
            (*MessageHeader).Buffer as PVOID,
            (*MessageHeader).Length as ULONG,
        );
        if KdStatus != KDP_PACKET_RECEIVED {
            KDDBGPRINT!(b"KdReceivePacket - Didn't receive message header data.\n\0".as_ptr() as PCHAR);
            KdpSendControlPacket(PACKET_TYPE_KD_RESEND as USHORT, 0);
            continue;
        }

        Checksum = KdpCalculateChecksum(
            (*MessageHeader).Buffer as PVOID,
            (*MessageHeader).Length as ULONG,
        );

        *DataLength = Packet.ByteCount as ULONG - (*MessageHeader).Length as ULONG;

        if !MessageData.is_null() {
            (*MessageData).Length = *DataLength as USHORT;

            if (*MessageData).Length != 0 {
                KDDBGPRINT!(b"KdReceivePacket - got data\n\0".as_ptr() as PCHAR);

                KdStatus = KdpReceiveBuffer(
                    (*MessageData).Buffer as PVOID,
                    (*MessageData).Length as ULONG,
                );
                if KdStatus != KDP_PACKET_RECEIVED {
                    KDDBGPRINT!(b"KdReceivePacket - Didn't receive message data.\n\0".as_ptr() as PCHAR);
                    KdpSendControlPacket(PACKET_TYPE_KD_RESEND as USHORT, 0);
                    continue;
                }

                Checksum = Checksum.wrapping_add(KdpCalculateChecksum(
                    (*MessageData).Buffer as PVOID,
                    (*MessageData).Length as ULONG,
                ));
            }
        }

        KdStatus = KdpReceiveBuffer(
            core::ptr::addr_of_mut!(Byte) as PVOID,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
        if KdStatus != KDP_PACKET_RECEIVED || Byte != PACKET_TRAILING_BYTE as UCHAR {
            KDDBGPRINT!(b"KdReceivePacket - wrong trailing byte (0x%x), status 0x%x\n\0".as_ptr() as PCHAR);
            KdpSendControlPacket(PACKET_TYPE_KD_RESEND as USHORT, 0);
            continue;
        }

        if Packet.Checksum != Checksum {
            KDDBGPRINT!(b"KdReceivePacket - wrong cheksum, got %x, calculated %x\n\0".as_ptr() as PCHAR);
            KdpSendControlPacket(PACKET_TYPE_KD_RESEND as USHORT, 0);
            continue;
        }

        KdpSendControlPacket(PACKET_TYPE_KD_ACKNOWLEDGE as USHORT, Packet.PacketId);

        if Packet.PacketId != RemotePacketId {
            continue;
        }

        if PacketType == Packet.PacketType as ULONG {
            RemotePacketId ^= 1;
            return KDP_PACKET_RECEIVED;
        }

        KDDBGPRINT!(b"KdReceivePacket - wrong PacketType\n\0".as_ptr() as PCHAR);
    }
}

#[cfg_attr(not(feature = "embedded"), no_mangle)]
pub unsafe extern "system" fn KdSendPacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    KdContext: PKD_CONTEXT,
) {
    let mut Packet: KD_PACKET = core::mem::zeroed();
    let mut KdStatus: KDP_STATUS;
    let mut Retries: ULONG;

    Packet.PacketLeader = PACKET_LEADER;
    Packet.PacketType = PacketType as USHORT;
    Packet.ByteCount = (*MessageHeader).Length;
    Packet.Checksum = KdpCalculateChecksum(
        (*MessageHeader).Buffer as PVOID,
        (*MessageHeader).Length as ULONG,
    );

    if !MessageData.is_null() {
        Packet.ByteCount = Packet.ByteCount.wrapping_add((*MessageData).Length);
        Packet.Checksum = Packet.Checksum.wrapping_add(KdpCalculateChecksum(
            (*MessageData).Buffer as PVOID,
            (*MessageData).Length as ULONG,
        ));
    }

    Retries = (*KdContext).KdpDefaultRetries;

    loop {
        Packet.PacketId = CurrentPacketId;

        KdpSendBuffer(
            core::ptr::addr_of_mut!(Packet) as PVOID,
            core::mem::size_of::<KD_PACKET>() as ULONG,
        );

        KdpSendBuffer(
            (*MessageHeader).Buffer as PVOID,
            (*MessageHeader).Length as ULONG,
        );

        if !MessageData.is_null() {
            KdpSendBuffer(
                (*MessageData).Buffer as PVOID,
                (*MessageData).Length as ULONG,
            );
        }

        KdpSendByte(PACKET_TRAILING_BYTE);

        KdStatus = KdReceivePacket(
            PACKET_TYPE_KD_ACKNOWLEDGE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            KdContext,
        );

        if KdStatus == KDP_PACKET_RECEIVED {
            CurrentPacketId &= !SYNC_PACKET_ID;
            Retries = (*KdContext).KdpDefaultRetries;
            break;
        } else if KdStatus == KDP_PACKET_TIMEOUT {
            if Retries > 0 {
                Retries = Retries.wrapping_sub(1);
            }

            if Retries == 0 {
                let MessageId = *((*MessageHeader).Buffer as PULONG);
                match PacketType {
                    PACKET_TYPE_KD_DEBUG_IO => {
                        if MessageId != DbgKdPrintStringApi {
                            continue;
                        }
                    }
                    PACKET_TYPE_KD_STATE_CHANGE32 | ntkrnl_sys::constants::PACKET_TYPE_KD_STATE_CHANGE64 => {
                        if MessageId != DbgKdLoadSymbolsStateChange {
                            continue;
                        }
                    }
                    PACKET_TYPE_KD_FILE_IO => {
                        if MessageId != DbgKdCreateFileApi {
                            continue;
                        }
                    }
                    _ => {}
                }

                KdDebuggerNotPresent = 1u8;
                (*SharedUserData).KdDebuggerEnabled &= !0x00000002;
                CurrentPacketId = INITIAL_PACKET_ID | SYNC_PACKET_ID;
                RemotePacketId = INITIAL_PACKET_ID;

                return;
            }
        }

        KDDBGPRINT!(b"KdSendPacket got KdStatus 0x%x\n\0".as_ptr() as PCHAR, KdStatus);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/kddll.c
//  * PURPOSE:         Base functions for the kernel debugger.
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// #include "kddll.h"
//
// /* GLOBALS ********************************************************************/
//
// ULONG CurrentPacketId = INITIAL_PACKET_ID | SYNC_PACKET_ID;
// ULONG RemotePacketId  = INITIAL_PACKET_ID;
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /******************************************************************************
//  * \name KdpCalculateChecksum
//  * \brief Calculates the checksum for the packet data.
//  * \param Buffer Pointer to the packet data.
//  * \param Length Length of data in bytes.
//  * \return The calculated checksum.
//  * \sa http://www.vista-xp.co.uk/forums/technical-reference-library/2540-basics-debugging.html (DEAD_LINK)
//  */
// ULONG
// NTAPI
// KdpCalculateChecksum(
//     IN PVOID Buffer,
//     IN ULONG Length)
// {
//     PUCHAR ByteBuffer = Buffer;
//     ULONG Checksum = 0;
//
//     while (Length-- > 0)
//     {
//         Checksum += (ULONG)*ByteBuffer++;
//     }
//     return Checksum;
// }
//
// VOID
// NTAPI
// KdpSendControlPacket(
//     IN USHORT PacketType,
//     IN ULONG PacketId OPTIONAL)
// {
//     KD_PACKET Packet;
//
//     Packet.PacketLeader = CONTROL_PACKET_LEADER;
//     Packet.PacketId = PacketId;
//     Packet.ByteCount = 0;
//     Packet.Checksum = 0;
//     Packet.PacketType = PacketType;
//
//     KdpSendBuffer(&Packet, sizeof(KD_PACKET));
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /******************************************************************************
//  * \name KdReceivePacket
//  * \brief Receive a packet from the KD port.
//  * \param [in] PacketType Describes the type of the packet to receive.
//  *        This can be one of the PACKET_TYPE_ constants.
//  * \param [out] MessageHeader Pointer to a STRING structure for the header.
//  * \param [out] MessageData Pointer to a STRING structure for the data.
//  * \return KdPacketReceived if successful, KdPacketTimedOut if the receive
//  *         timed out, KdPacketNeedsResend to signal that the last packet needs
//  *         to be sent again.
//  * \note If PacketType is PACKET_TYPE_KD_POLL_BREAKIN, the function doesn't
//  *       wait for any data, but returns KdPacketTimedOut instantly if no breakin
//  *       packet byte is received.
//  * \sa http://www.nynaeve.net/?p=169
//  */
// KDP_STATUS
// NTAPI
// KdReceivePacket(
//     IN ULONG PacketType,
//     OUT PSTRING MessageHeader,
//     OUT PSTRING MessageData,
//     OUT PULONG DataLength,
//     IN OUT PKD_CONTEXT KdContext)
// {
//     UCHAR Byte = 0;
//     KDP_STATUS KdStatus;
//     KD_PACKET Packet;
//     ULONG Checksum;
//
//     /* Special handling for breakin packet */
//     if (PacketType == PACKET_TYPE_KD_POLL_BREAKIN)
//     {
//         return KdpPollBreakIn();
//     }
//
//     for (;;)
//     {
//         /* Step 1 - Read PacketLeader */
//         KdStatus = KdpReceivePacketLeader(&Packet.PacketLeader);
//         if (KdStatus != KDP_PACKET_RECEIVED)
//         {
//             /* Check if we got a breakin  */
//             if (KdStatus == KDP_PACKET_RESEND)
//             {
//                 KdContext->KdpControlCPending = TRUE;
//             }
//             return KdStatus;
//         }
//
//         /* Step 2 - Read PacketType */
//         KdStatus = KdpReceiveBuffer(&Packet.PacketType, sizeof(USHORT));
//         if (KdStatus != KDP_PACKET_RECEIVED)
//         {
//             /* Didn't receive a PacketType. */
//             return KdStatus;
//         }
//
//         /* Check if we got a resend packet */
//         if (Packet.PacketLeader == CONTROL_PACKET_LEADER &&
//             Packet.PacketType == PACKET_TYPE_KD_RESEND)
//         {
//             return KDP_PACKET_RESEND;
//         }
//
//         /* Step 3 - Read ByteCount */
//         KdStatus = KdpReceiveBuffer(&Packet.ByteCount, sizeof(USHORT));
//         if (KdStatus != KDP_PACKET_RECEIVED)
//         {
//             /* Didn't receive ByteCount. */
//             return KdStatus;
//         }
//
//         /* Step 4 - Read PacketId */
//         KdStatus = KdpReceiveBuffer(&Packet.PacketId, sizeof(ULONG));
//         if (KdStatus != KDP_PACKET_RECEIVED)
//         {
//             /* Didn't receive PacketId. */
//             return KdStatus;
//         }
//
// /*
//         if (Packet.PacketId != ExpectedPacketId)
//         {
//             // Ask for a resend!
//             continue;
//         }
// */
//
//         /* Step 5 - Read Checksum */
//         KdStatus = KdpReceiveBuffer(&Packet.Checksum, sizeof(ULONG));
//         if (KdStatus != KDP_PACKET_RECEIVED)
//         {
//             /* Didn't receive Checksum. */
//             return KdStatus;
//         }
//
//         /* Step 6 - Handle control packets */
//         if (Packet.PacketLeader == CONTROL_PACKET_LEADER)
//         {
//             switch (Packet.PacketType)
//             {
//                 case PACKET_TYPE_KD_ACKNOWLEDGE:
//                     /* Are we waiting for an ACK packet? */
//                     if (PacketType == PACKET_TYPE_KD_ACKNOWLEDGE &&
//                         Packet.PacketId == (CurrentPacketId & ~SYNC_PACKET_ID))
//                     {
//                         /* Remote acknowledges the last packet */
//                         CurrentPacketId ^= 1;
//                         return KDP_PACKET_RECEIVED;
//                     }
//                     /* That's not what we were waiting for, start over */
//                     continue;
//
//                 case PACKET_TYPE_KD_RESET:
//                     KDDBGPRINT("KdReceivePacket - got PACKET_TYPE_KD_RESET\n");
//                     CurrentPacketId = INITIAL_PACKET_ID;
//                     RemotePacketId  = INITIAL_PACKET_ID;
//                     KdpSendControlPacket(PACKET_TYPE_KD_RESET, 0);
//                     /* Fall through */
//
//                 case PACKET_TYPE_KD_RESEND:
//                     KDDBGPRINT("KdReceivePacket - got PACKET_TYPE_KD_RESEND\n");
//                     /* Remote wants us to resend the last packet */
//                     return KDP_PACKET_RESEND;
//
//                 default:
//                     KDDBGPRINT("KdReceivePacket - got unknown control packet\n");
//                     /* We got an invalid packet, ignore it and start over */
//                     continue;
//             }
//         }
//
//         /* Did we wait for an ack packet? */
//         if (PacketType == PACKET_TYPE_KD_ACKNOWLEDGE)
//         {
//             /* We received something different */
//             KdpSendControlPacket(PACKET_TYPE_KD_RESEND, 0);
//             CurrentPacketId ^= 1;
//             return KDP_PACKET_RECEIVED;
//         }
//
//         /* Get size of the message header */
//         MessageHeader->Length = MessageHeader->MaximumLength;
//
//         /* Packet smaller than expected or too big? */
//         if (Packet.ByteCount < MessageHeader->Length ||
//             Packet.ByteCount > PACKET_MAX_SIZE)
//         {
//             KDDBGPRINT("KdReceivePacket - too few data (%d) for type %d\n",
//                           Packet.ByteCount, MessageHeader->Length);
//             MessageHeader->Length = Packet.ByteCount;
//             KdpSendControlPacket(PACKET_TYPE_KD_RESEND, 0);
//             continue;
//         }
//
//         //KDDBGPRINT("KdReceivePacket - got normal PacketType, Buffer = %p\n", MessageHeader->Buffer);
//
//         /* Receive the message header data */
//         KdStatus = KdpReceiveBuffer(MessageHeader->Buffer,
//                                     MessageHeader->Length);
//         if (KdStatus != KDP_PACKET_RECEIVED)
//         {
//             /* Didn't receive data. Packet needs to be resent. */
//             KDDBGPRINT("KdReceivePacket - Didn't receive message header data.\n");
//             KdpSendControlPacket(PACKET_TYPE_KD_RESEND, 0);
//             continue;
//         }
//
//         //KDDBGPRINT("KdReceivePacket - got normal PacketType 3\n");
//
//         /* Calculate checksum for the header data */
//         Checksum = KdpCalculateChecksum(MessageHeader->Buffer,
//                                         MessageHeader->Length);
//
//         /* Calculate the length of the message data */
//         *DataLength = Packet.ByteCount - MessageHeader->Length;
//
//         /* Shall we receive message data? */
//         if (MessageData)
//         {
//             /* Set the length of the message data */
//             MessageData->Length = (USHORT)*DataLength;
//
//             /* Do we have data? */
//             if (MessageData->Length)
//             {
//                 KDDBGPRINT("KdReceivePacket - got data\n");
//
//                 /* Receive the message data */
//                 KdStatus = KdpReceiveBuffer(MessageData->Buffer,
//                                            MessageData->Length);
//                 if (KdStatus != KDP_PACKET_RECEIVED)
//                 {
//                     /* Didn't receive data. Start over. */
//                     KDDBGPRINT("KdReceivePacket - Didn't receive message data.\n");
//                     KdpSendControlPacket(PACKET_TYPE_KD_RESEND, 0);
//                     continue;
//                 }
//
//                 /* Add cheksum for message data */
//                 Checksum += KdpCalculateChecksum(MessageData->Buffer,
//                                                  MessageData->Length);
//             }
//         }
//
//         /* We must receive a PACKET_TRAILING_BYTE now */
//         KdStatus = KdpReceiveBuffer(&Byte, sizeof(UCHAR));
//         if (KdStatus != KDP_PACKET_RECEIVED || Byte != PACKET_TRAILING_BYTE)
//         {
//             KDDBGPRINT("KdReceivePacket - wrong trailing byte (0x%x), status 0x%x\n", Byte, KdStatus);
//             KdpSendControlPacket(PACKET_TYPE_KD_RESEND, 0);
//             continue;
//         }
//
//         /* Compare checksum */
//         if (Packet.Checksum != Checksum)
//         {
//             KDDBGPRINT("KdReceivePacket - wrong cheksum, got %x, calculated %x\n",
//                        Packet.Checksum, Checksum);
//             KdpSendControlPacket(PACKET_TYPE_KD_RESEND, 0);
//             continue;
//         }
//
//         /* Acknowledge the received packet */
//         KdpSendControlPacket(PACKET_TYPE_KD_ACKNOWLEDGE, Packet.PacketId);
//
//         /* Check if the received PacketId is ok */
//         if (Packet.PacketId != RemotePacketId)
//         {
//             /* Continue with next packet */
//             continue;
//         }
//
//         /* Did we get the right packet type? */
//         if (PacketType == Packet.PacketType)
//         {
//             /* Yes, return success */
//             //KDDBGPRINT("KdReceivePacket - all ok\n");
//             RemotePacketId ^= 1;
//             return KDP_PACKET_RECEIVED;
//         }
//
//         /* We received something different, ignore it. */
//         KDDBGPRINT("KdReceivePacket - wrong PacketType\n");
//     }
//
//     return KDP_PACKET_RECEIVED;
// }
//
// VOID
// NTAPI
// KdSendPacket(
//     IN ULONG PacketType,
//     IN PSTRING MessageHeader,
//     IN PSTRING MessageData,
//     IN OUT PKD_CONTEXT KdContext)
// {
//     KD_PACKET Packet;
//     KDP_STATUS KdStatus;
//     ULONG Retries;
//
//     /* Initialize a KD_PACKET */
//     Packet.PacketLeader = PACKET_LEADER;
//     Packet.PacketType = (USHORT)PacketType;
//     Packet.ByteCount = MessageHeader->Length;
//     Packet.Checksum = KdpCalculateChecksum(MessageHeader->Buffer,
//                                            MessageHeader->Length);
//
//     /* If we have message data, add it to the packet */
//     if (MessageData)
//     {
//         Packet.ByteCount += MessageData->Length;
//         Packet.Checksum += KdpCalculateChecksum(MessageData->Buffer,
//                                                 MessageData->Length);
//     }
//
//     Retries = KdContext->KdpDefaultRetries;
//
//     for (;;)
//     {
//         /* Set the packet id */
//         Packet.PacketId = CurrentPacketId;
//
//         /* Send the packet header to the KD port */
//         KdpSendBuffer(&Packet, sizeof(KD_PACKET));
//
//         /* Send the message header */
//         KdpSendBuffer(MessageHeader->Buffer, MessageHeader->Length);
//
//         /* If we have message data, also send it */
//         if (MessageData)
//         {
//             KdpSendBuffer(MessageData->Buffer, MessageData->Length);
//         }
//
//         /* Finalize with a trailing byte */
//         KdpSendByte(PACKET_TRAILING_BYTE);
//
//         /* Wait for acknowledge */
//         KdStatus = KdReceivePacket(PACKET_TYPE_KD_ACKNOWLEDGE,
//                                    NULL,
//                                    NULL,
//                                    NULL,
//                                    KdContext);
//
//         /* Did we succeed? */
//         if (KdStatus == KDP_PACKET_RECEIVED)
//         {
//             /* Packet received, we can quit the loop */
//             CurrentPacketId &= ~SYNC_PACKET_ID;
//             Retries = KdContext->KdpDefaultRetries;
//             break;
//         }
//         else if (KdStatus == KDP_PACKET_TIMEOUT)
//         {
//             /* Timeout, decrement the retry count */
//             if (Retries > 0)
//                 Retries--;
//
//             /*
//              * If the retry count reaches zero, bail out
//              * for packet types allowed to timeout.
//              */
//             if (Retries == 0)
//             {
//                 ULONG MessageId = *(PULONG)MessageHeader->Buffer;
//                 switch (PacketType)
//                 {
//                     case PACKET_TYPE_KD_DEBUG_IO:
//                     {
//                         if (MessageId != DbgKdPrintStringApi) continue;
//                         break;
//                     }
//
//                     case PACKET_TYPE_KD_STATE_CHANGE32:
//                     case PACKET_TYPE_KD_STATE_CHANGE64:
//                     {
//                         if (MessageId != DbgKdLoadSymbolsStateChange) continue;
//                         break;
//                     }
//
//                     case PACKET_TYPE_KD_FILE_IO:
//                     {
//                         if (MessageId != DbgKdCreateFileApi) continue;
//                         break;
//                     }
//                 }
//
//                 /* Reset debugger state */
//                 KD_DEBUGGER_NOT_PRESENT = TRUE;
//                 SharedUserData->KdDebuggerEnabled &= ~0x00000002;
//                 CurrentPacketId = INITIAL_PACKET_ID | SYNC_PACKET_ID;
//                 RemotePacketId  = INITIAL_PACKET_ID;
//
//                 return;
//             }
//         }
//         // else (KdStatus == KDP_PACKET_RESEND) /* Resend the packet */
//
//         /* Packet timed out, send it again */
//         KDDBGPRINT("KdSendPacket got KdStatus 0x%x\n", KdStatus);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
