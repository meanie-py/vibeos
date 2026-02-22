use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KeDisableInterrupts, KeRestoreInterrupts, KeTryToAcquireSpinLockAtDpcLevel, KiAcquireSpinLock, KiReleaseSpinLock};
use core::ffi::c_void;

extern "system" {
    fn KdReceivePacket( PacketType: ULONG, MessageHeader: PSTRING, MessageData: PSTRING, DataLength: PULONG, KdContext: *mut _KD_CONTEXT, ) -> KD_RECV_CODE;
}

#[no_mangle]
pub unsafe extern "system" fn KdpPortLock() {
    KiAcquireSpinLock(core::ptr::addr_of_mut!(KdpDebuggerLock));
}

#[no_mangle]
pub unsafe extern "system" fn KdpPortUnlock() {
    KiReleaseSpinLock(core::ptr::addr_of_mut!(KdpDebuggerLock));
}

#[no_mangle]
pub unsafe extern "system" fn KdpPollBreakInWithPortLock() -> BOOLEAN {
    let mut DoBreak: BOOLEAN = 0u8;

    if KdDebuggerEnabled != 0 {
        if KdpContext.KdpControlCPending != 0 {
            DoBreak = 1u8;
            KdpContext.KdpControlCPending = 0u8;
        } else {

            if KdReceivePacket(
                PACKET_TYPE_KD_POLL_BREAKIN,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == KdPacketReceived
            {
                DoBreak = 1u8;
            }
        }
    }

    DoBreak
}

#[no_mangle]
pub unsafe extern "system" fn KdPollBreakIn() -> u8 {
    let mut DoBreak: u8 = 0u8;
    let Enable: u8;

    if KdDebuggerEnabled != 0 {
        Enable = KeDisableInterrupts();

        if KdpContext.KdpControlCPending != 0 {
            KdpControlCPressed = 1u8;
            DoBreak = 1u8;
            KdpContext.KdpControlCPending = 0u8;
        } else {
            let mut OldIrql: KIRQL = 0;
            KeRaiseIrql(0x1Fu8, core::ptr::addr_of_mut!(OldIrql));
            if KeTryToAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KdpDebuggerLock)) != 0 {
                if KdReceivePacket(
                    PACKET_TYPE_KD_POLL_BREAKIN,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) == KdPacketReceived
                {
                    DoBreak = 1u8;
                    KdpControlCPressed = 1u8;
                }

                KdpPortUnlock();
            }
            KeLowerIrql(OldIrql);
        }

        KeRestoreInterrupts(Enable);
    }

    DoBreak
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/kd64/kdlock.c
//  * PURPOSE:         KD64 Port Lock and Breakin Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KdpPortLock(VOID)
// {
//     /* Acquire the lock */
//     KiAcquireSpinLock(&KdpDebuggerLock);
// }
//
// VOID
// NTAPI
// KdpPortUnlock(VOID)
// {
//     /* Release the lock */
//     KiReleaseSpinLock(&KdpDebuggerLock);
// }
//
// BOOLEAN
// NTAPI
// KdpPollBreakInWithPortLock(VOID)
// {
//     BOOLEAN DoBreak = FALSE;
//
//     /* First make sure that KD is enabled */
//     if (KdDebuggerEnabled)
//     {
//         /* Check if a CTRL-C is in the queue */
//         if (KdpContext.KdpControlCPending)
//         {
//             /* Set it and prepare for break */
//             DoBreak = TRUE;
//             KdpContext.KdpControlCPending = FALSE;
//         }
//         else
//         {
//             /* Now get a packet */
//             if (KdReceivePacket(PACKET_TYPE_KD_POLL_BREAKIN,
//                                 NULL,
//                                 NULL,
//                                 NULL,
//                                 NULL) == KdPacketReceived)
//             {
//                 /* Successful breakin */
//                 DoBreak = TRUE;
//             }
//         }
//     }
//
//     /* Tell the caller to do a break */
//     return DoBreak;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KdPollBreakIn(VOID)
// {
//     BOOLEAN DoBreak = FALSE, Enable;
//
//     /* First make sure that KD is enabled */
//     if (KdDebuggerEnabled)
//     {
//         /* Disable interrupts */
//         Enable = KeDisableInterrupts();
//
//         /* Check if a CTRL-C is in the queue */
//         if (KdpContext.KdpControlCPending)
//         {
//             /* Set it and prepare for break */
//             KdpControlCPressed = TRUE;
//             DoBreak = TRUE;
//             KdpContext.KdpControlCPending = FALSE;
//         }
//         else
//         {
//             KIRQL OldIrql;
//             /* Try to acquire the lock */
//             KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//             if (KeTryToAcquireSpinLockAtDpcLevel(&KdpDebuggerLock))
//             {
//                 /* Now get a packet */
//                 if (KdReceivePacket(PACKET_TYPE_KD_POLL_BREAKIN,
//                                     NULL,
//                                     NULL,
//                                     NULL,
//                                     NULL) == KdPacketReceived)
//                 {
//                     /* Successful breakin */
//                     DoBreak = TRUE;
//                     KdpControlCPressed = TRUE;
//                 }
//
//                 /* Let go of the port */
//                 KdpPortUnlock();
//             }
//             KeLowerIrql(OldIrql);
//         }
//
//         /* Re-enable interrupts */
//         KeRestoreInterrupts(Enable);
//     }
//
//     /* Tell the caller to do a break */
//     return DoBreak;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
