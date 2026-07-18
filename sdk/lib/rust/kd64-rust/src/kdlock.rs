use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    KD_CONTEXT,
    KeTryToAcquireSpinLockAtDpcLevel,
};

extern "system" {
    static mut KdpDebuggerLock: KSPIN_LOCK;
    static mut KdDebuggerEnabled: BOOLEAN;
    static mut KdpContext: KD_CONTEXT;
    static mut KdpControlCPressed: BOOLEAN;
    fn KdReceivePacket( packet_type: ULONG, message_header: PVOID, message_data: PVOID, data_length: *mut ULONG, context: PVOID, ) -> ULONG;
}
extern "C" {
}
unsafe extern "system" {
    fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: *mut KIRQL);
    fn KeLowerIrql(NewIrql: KIRQL);
}
extern "C" {
    fn KeDisableInterrupts() -> BOOLEAN;
    fn KeRestoreInterrupts(enable: BOOLEAN);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiAcquireSpinLock(lock: *mut KSPIN_LOCK);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiAcquireSpinLock(lock: *mut KSPIN_LOCK);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiReleaseSpinLock(lock: *mut KSPIN_LOCK);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiReleaseSpinLock(lock: *mut KSPIN_LOCK);
}

const PACKET_TYPE_KD_POLL_BREAKIN: ULONG = 8;
const KdPacketReceived: ULONG = 0;

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
    let mut do_break: BOOLEAN = 0u8;

    if KdDebuggerEnabled != 0 {
        if KdpContext.KdpControlCPending != 0 {
            do_break = 1u8;
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
                do_break = 1u8;
            }
        }
    }

    do_break
}

#[no_mangle]
pub unsafe extern "system" fn KdPollBreakIn() -> BOOLEAN {
    let mut do_break: BOOLEAN = 0u8;

    if KdDebuggerEnabled != 0 {
        let enable = KeDisableInterrupts();

        if KdpContext.KdpControlCPending != 0 {
            KdpControlCPressed = 1u8;
            do_break = 1u8;
            KdpContext.KdpControlCPending = 0u8;
        } else {
            let mut old_irql: KIRQL = 0;
            // [guard] KeRaiseIrql(HIGH_LEVEL, core::ptr::addr_of_mut!(old_irql));
            let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };
            if KeTryToAcquireSpinLockAtDpcLevel(core::ptr::addr_of_mut!(KdpDebuggerLock) as *mut _) != 0 {
                if KdReceivePacket(
                    PACKET_TYPE_KD_POLL_BREAKIN,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) == KdPacketReceived
                {
                    do_break = 1u8;
                    KdpControlCPressed = 1u8;
                }

                KdpPortUnlock();
            }
            drop(irql_guard);
        }

        KeRestoreInterrupts(enable);
    }

    do_break
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
