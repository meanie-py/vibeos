use super::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::KeGetCurrentThread;

use ntkrnl_sys::ddk::{
    KGATE,
    KPROCESSOR_MODE,
    KWAIT_BLOCK,
    KeRaiseIrqlToSynchLevel,
    KiAcquireApcLockRaiseToSynch,
    KiAcquireDispatcherLockAtSynchLevel,
    KiAcquireDispatcherObject,
    KiAcquireThreadLock,
    KiActivateWaiterQueue,
    KiExitDispatcher,
    KiReadyThread,
    KiReleaseApcLock,
    KiReleaseApcLockFromSynchLevel,
    KiReleaseDispatcherLockFromSynchLevel,
    KiReleaseDispatcherObject,
    KiReleaseThreadLock,
    KiSetThreadSwapBusy,
    KiSwapThread,
    KiTryThreadLock,
    PKGATE,
    PKQUEUE,
    PKTHREAD,
    PKWAIT_BLOCK,
};
// Dispatcher object type constants
const GateObject: u32 = 7;
const EventSynchronizationObject: u32 = 1;
const KOBJECT_TYPE_MASK: u8 = 0x7F;
const GateWait: u32 = 8;
const DeferredReady: u32 = 7;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeInitializeGate(Gate: PKGATE) {
    (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type = GateObject as u8;
    (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.Signalling = 0u8;
    (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<KGATE>() / core::mem::size_of::<ULONG>()) as u8;
    (*Gate).Header.SignalState = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Gate).Header.WaitListHead));
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeInitializeGate(Gate: PKGATE) {
    (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type = GateObject as u8;
    (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.Signalling = 0u8;
    (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<KGATE>() / core::mem::size_of::<ULONG>()) as u8;
    (*Gate).Header.SignalState = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Gate).Header.WaitListHead));
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeWaitForGate(
    Gate: PKGATE,
    WaitReason: u8,
    WaitMode: KPROCESSOR_MODE,
) {
    KeWaitForGateImpl(Gate, WaitReason, WaitMode)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeWaitForGate(
    Gate: PKGATE,
    WaitReason: u8,
    WaitMode: KPROCESSOR_MODE,
) {
    KeWaitForGateImpl(Gate, WaitReason, WaitMode)
}

unsafe fn KeWaitForGateImpl(
    Gate: PKGATE,
    WaitReason: u8,
    WaitMode: KPROCESSOR_MODE,
) {
    let mut ApcLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let Thread: PKTHREAD = KeGetCurrentThread() as PKTHREAD;
    let mut GateWaitBlock: PKWAIT_BLOCK;
    let mut Status: isize;
    let mut Queue: PKQUEUE;

    if !(((*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type & KOBJECT_TYPE_MASK) == GateObject as u8) &&
       !(((*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type & KOBJECT_TYPE_MASK) == EventSynchronizationObject as u8)
    {
        crate::dbg_print!(b"KeWaitForGate: BAD gate %p Type=%u Size=%u SignalState=%d caller=%p caller2=%p\n\0",
            Gate,
            (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type as u32,
            (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size as u32,
            (*Gate).Header.SignalState,
            core::ptr::null::<()>(),
            core::ptr::null::<()>(),
        );
    }

    loop {
        KiAcquireApcLockRaiseToSynch(Thread, core::ptr::addr_of_mut!(ApcLock));

        if kt_ApcState(Thread).KernelApcPending != 0 &&
           *kt_SpecialApcDisable(Thread) == 0 &&
           ApcLock.OldIrql < APC_LEVEL
        {
            KiReleaseApcLock(core::ptr::addr_of_mut!(ApcLock));
        } else {
            Queue = (*Thread).Queue;
            if !Queue.is_null() {
                KiAcquireDispatcherLockAtSynchLevel();
            }

            KiAcquireThreadLock(Thread);

            KiAcquireDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));

            if (*Gate).Header.SignalState != 0 {
                (*Gate).Header.SignalState = 0;

                KiReleaseDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));
                KiReleaseThreadLock(Thread);

                if !Queue.is_null() {
                    KiReleaseDispatcherLockFromSynchLevel();
                }

                KiReleaseApcLock(core::ptr::addr_of_mut!(ApcLock));
                break;
            }

            GateWaitBlock = &mut kt_WaitBlock(Thread)[0] as PKWAIT_BLOCK;
            (*GateWaitBlock).Object = Gate as PVOID;
            (*GateWaitBlock).Thread = Thread;

            *kt_WaitMode(Thread) = WaitMode;
            (*Thread).WaitReason = WaitReason;
            *kt_WaitIrql(Thread) = ApcLock.OldIrql;
            *kt_State(Thread) = GateWait as u8;
            (*Thread).__bindgen_anon_2.GateObject = Gate;

            InsertTailList(
                core::ptr::addr_of_mut!((*Gate).Header.WaitListHead),
                core::ptr::addr_of_mut!((*GateWaitBlock).WaitListEntry),
            );

            // synctrace §3: this waiter is committed to parking (wait-list
            // insert done, swap next) — count it before the locks drop.
            super::synctrace::synctrace_wait_park(Gate as _, Thread as _);

            KiReleaseDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));

            KiSetThreadSwapBusy(Thread);

            KiReleaseThreadLock(Thread);

            if !Queue.is_null() {
                KiActivateWaiterQueue(Queue);

                KiReleaseDispatcherLockFromSynchLevel();
            }

            KiReleaseApcLockFromSynchLevel(core::ptr::addr_of_mut!(ApcLock));

            Status = KiSwapThread(Thread, KeGetCurrentPrcb()) as isize;

            if Status == STATUS_SUCCESS as isize {
                // synctrace §3: the parked wait was satisfied.
                super::synctrace::synctrace_wait_satisfied(Gate as _);
                return;
            }
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeSignalGateBoostPriority(Gate: PKGATE) {
    KeSignalGateBoostPriorityImpl(Gate)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeSignalGateBoostPriority(Gate: PKGATE) {
    KeSignalGateBoostPriorityImpl(Gate)
}

#[inline(always)]
unsafe fn KeSignalGateBoostPriorityImpl(Gate: PKGATE) {
    let mut WaitThread: PKTHREAD;
    let mut WaitBlock: PKWAIT_BLOCK;
    let mut OldIrql: KIRQL;

    if !(((*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type & KOBJECT_TYPE_MASK) == GateObject as u8) &&
       !(((*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type & KOBJECT_TYPE_MASK) == EventSynchronizationObject as u8)
    {
        crate::dbg_print!(b"KeSignalGateBoostPriority: BAD gate %p Type=%u Size=%u SignalState=%d caller=%p\n\0",
            Gate,
            (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.Type as u32,
            (*Gate).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size as u32,
            (*Gate).Header.SignalState,
            core::ptr::null::<()>(),
        );
    }

    // synctrace §3: count every signal intent (waiter woken, state banked, or
    // no-op on an already-set gate) — the census compares this against
    // waits_parked to discriminate lost-wakeup from holder-stuck.
    super::synctrace::synctrace_signal(Gate as _, KeGetCurrentThread() as _);

    loop {
        OldIrql = KeRaiseIrqlToSynchLevel();

        KiAcquireDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));

        if (*Gate).Header.SignalState != 0 {
            break;
        }

        if IsListEmpty(core::ptr::addr_of_mut!((*Gate).Header.WaitListHead)) != false {
            (*Gate).Header.SignalState = 1;
            break;
        } else {
            // waitdep: WaitBlock->Thread is consumed off the gate's wait list
            // and fed to KiTryThreadLock/KiReadyThread with NO KiUnlinkThread
            // in the path — a garbage thread here would crash with no waitdep
            // report at all (unchecked funnel; see eventobj.rs for the family).
            super::waitdep::check_waittest(
                Gate as usize,
                core::ptr::addr_of_mut!((*Gate).Header.WaitListHead) as *mut _,
                true,
            );
            WaitBlock = (((*Gate).Header.WaitListHead.Flink as usize) -
                        core::mem::offset_of!(KWAIT_BLOCK, WaitListEntry)) as PKWAIT_BLOCK;

            WaitThread = (*WaitBlock).Thread;

            if KiTryThreadLock(WaitThread) != 0 {
                KiReleaseDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));

                KeLowerIrql(OldIrql);
                continue;
            }

            RemoveEntryList(core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry));

            (*WaitThread).WaitStatus = STATUS_SUCCESS as _;

            *kt_State(WaitThread) = DeferredReady as u8;
            (*WaitThread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor = (*KeGetCurrentPrcb()).Number as u8;

            KiReleaseDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));

            KiReleaseThreadLock(WaitThread);

            if !(*WaitThread).Queue.is_null() {
                KiAcquireDispatcherLockAtSynchLevel();

                if !(*WaitThread).Queue.is_null() {
                    (*(*WaitThread).Queue).CurrentCount += 1;
                }

                KiReleaseDispatcherLockFromSynchLevel();
            }

            KiReadyThread(WaitThread);

            KiExitDispatcher(OldIrql);
            return;
        }
    }

    KiReleaseDispatcherObject(core::ptr::addr_of_mut!((*Gate).Header));
    KeLowerIrql(OldIrql);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/gate.c
//  * PURPOSE:         Implements the Gate Dispatcher Object
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ****************************************************************/
//
// VOID
// FASTCALL
// KeInitializeGate(IN PKGATE Gate)
// {
//     /* Initialize the Dispatcher Header */
//     Gate->Header.Type = GateObject;
//     Gate->Header.Signalling = FALSE;
//     Gate->Header.Size = sizeof(KGATE) / sizeof(ULONG);
//     Gate->Header.SignalState = 0;
//     InitializeListHead(&(Gate->Header.WaitListHead));
// }
//
// VOID
// FASTCALL
// KeWaitForGate(IN PKGATE Gate,
//               IN KWAIT_REASON WaitReason,
//               IN KPROCESSOR_MODE WaitMode)
// {
//     KLOCK_QUEUE_HANDLE ApcLock;
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKWAIT_BLOCK GateWaitBlock;
//     LONG_PTR Status;
//     PKQUEUE Queue;
//     if (!(((Gate)->Header.Type & KOBJECT_TYPE_MASK) == GateObject) &&
//         !(((Gate)->Header.Type & KOBJECT_TYPE_MASK) == EventSynchronizationObject))
//     {
//         DbgPrint("KeWaitForGate: BAD gate %p Type=%u Size=%u SignalState=%d caller=%p caller2=%p\n",
//                  Gate, (unsigned)Gate->Header.Type, (unsigned)Gate->Header.Size,
//                  Gate->Header.SignalState, __builtin_return_address(0), __builtin_return_address(1));
//     }
//     ASSERT_GATE(Gate);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Start wait loop */
//     do
//     {
//         /* Acquire the APC lock */
//         KiAcquireApcLockRaiseToSynch(Thread, &ApcLock);
//
//         /* Check if a kernel APC is pending and we're below APC_LEVEL */
//         if ((Thread->ApcState.KernelApcPending) &&
//             !(Thread->SpecialApcDisable) &&
//             (ApcLock.OldIrql < APC_LEVEL))
//         {
//             /* Release the lock, this will fire the APC */
//             KiReleaseApcLock(&ApcLock);
//         }
//         else
//         {
//             /* Check if we have a queue and lock the dispatcher if so */
//             Queue = Thread->Queue;
//             if (Queue) KiAcquireDispatcherLockAtSynchLevel();
//
//             /* Lock the thread */
//             KiAcquireThreadLock(Thread);
//
//             /* Lock the gate */
//             KiAcquireDispatcherObject(&Gate->Header);
//
//             /* Check if it's already signaled */
//             if (Gate->Header.SignalState)
//             {
//                 /* Unsignal it */
//                 Gate->Header.SignalState = 0;
//
//                 /* Release the gate and thread locks */
//                 KiReleaseDispatcherObject(&Gate->Header);
//                 KiReleaseThreadLock(Thread);
//
//                 /* Release the gate lock */
//                 if (Queue) KiReleaseDispatcherLockFromSynchLevel();
//
//                 /* Release the APC lock and return */
//                 KiReleaseApcLock(&ApcLock);
//                 break;
//             }
//
//             /* Setup a Wait Block */
//             GateWaitBlock = &Thread->WaitBlock[0];
//             GateWaitBlock->Object = (PVOID)Gate;
//             GateWaitBlock->Thread = Thread;
//
//             /* Set the Thread Wait Data */
//             Thread->WaitMode = WaitMode;
//             Thread->WaitReason = WaitReason;
//             Thread->WaitIrql = ApcLock.OldIrql;
//             Thread->State = GateWait;
//             Thread->GateObject = Gate;
//
//             /* Insert into the Wait List */
//             InsertTailList(&Gate->Header.WaitListHead,
//                            &GateWaitBlock->WaitListEntry);
//
//             /* Release the gate lock */
//             KiReleaseDispatcherObject(&Gate->Header);
//
//             /* Set swap busy */
//             KiSetThreadSwapBusy(Thread);
//
//             /* Release the thread lock */
//             KiReleaseThreadLock(Thread);
//
//             /* Check if we had a queue */
//             if (Queue)
//             {
//                 /* Wake it up */
//                 KiActivateWaiterQueue(Queue);
//
//                 /* Release the dispatcher lock */
//                 KiReleaseDispatcherLockFromSynchLevel();
//             }
//
//             /* Release the APC lock but stay at DPC level */
//             KiReleaseApcLockFromSynchLevel(&ApcLock);
//
//             /* Find a new thread to run */
//             Status = KiSwapThread(Thread, KeGetCurrentPrcb());
//
//             /* Make sure we weren't executing an APC */
//             if (Status == STATUS_SUCCESS) return;
//         }
//     } while (TRUE);
// }
//
// VOID
// FASTCALL
// KeSignalGateBoostPriority(IN PKGATE Gate)
// {
//     PKTHREAD WaitThread;
//     PKWAIT_BLOCK WaitBlock;
//     KIRQL OldIrql;
//     if (!(((Gate)->Header.Type & KOBJECT_TYPE_MASK) == GateObject) &&
//         !(((Gate)->Header.Type & KOBJECT_TYPE_MASK) == EventSynchronizationObject))
//     {
//         DbgPrint("KeSignalGateBoostPriority: BAD gate %p Type=%u Size=%u SignalState=%d caller=%p\n",
//                  Gate, (unsigned)Gate->Header.Type, (unsigned)Gate->Header.Size,
//                  Gate->Header.SignalState, __builtin_return_address(0));
//     }
//     ASSERT_GATE(Gate);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Start entry loop */
//     for (;;)
//     {
//         /* Raise to synch level */
//         OldIrql = KeRaiseIrqlToSynchLevel();
//
//         /* Lock the gate */
//         KiAcquireDispatcherObject(&Gate->Header);
//
//         /* Make sure we're not already signaled or that the list is empty */
//         if (Gate->Header.SignalState) break;
//
//         /* Check if our wait list is empty */
//         if (IsListEmpty(&Gate->Header.WaitListHead))
//         {
//             /* It is, so signal the event */
//             Gate->Header.SignalState = 1;
//             break;
//         }
//         else
//         {
//             /* Get WaitBlock */
//             WaitBlock = CONTAINING_RECORD(Gate->Header.WaitListHead.Flink,
//                                           KWAIT_BLOCK,
//                                           WaitListEntry);
//
//             /* Get the Associated thread */
//             WaitThread = WaitBlock->Thread;
//
//             /* Check to see if the waiting thread is locked */
//             if (KiTryThreadLock(WaitThread))
//             {
//                 /* Unlock the gate */
//                 KiReleaseDispatcherObject(&Gate->Header);
//
//                 /* Lower IRQL and loop again */
//                 KeLowerIrql(OldIrql);
//                 continue;
//             }
//
//             /* Remove it */
//             RemoveEntryList(&WaitBlock->WaitListEntry);
//
//             /* Clear wait status */
//             WaitThread->WaitStatus = STATUS_SUCCESS;
//
//             /* Set state and CPU */
//             WaitThread->State = DeferredReady;
//             WaitThread->DeferredProcessor = KeGetCurrentPrcb()->Number;
//
//             /* Release the gate lock */
//             KiReleaseDispatcherObject(&Gate->Header);
//
//             /* Release the thread lock */
//             KiReleaseThreadLock(WaitThread);
//
//             /* FIXME: Boosting */
//
//             /* Check if we have a queue */
//             if (WaitThread->Queue)
//             {
//                 /* Acquire the dispatcher lock */
//                 KiAcquireDispatcherLockAtSynchLevel();
//
//                 /* Check if we still have one */
//                 if (WaitThread->Queue)
//                 {
//                     /* Increment active threads */
//                     WaitThread->Queue->CurrentCount++;
//                 }
//
//                 /* Release lock */
//                 KiReleaseDispatcherLockFromSynchLevel();
//             }
//
//             /* Make the thread ready */
//             KiReadyThread(WaitThread);
//
//             /* Exit the dispatcher */
//             KiExitDispatcher(OldIrql);
//             return;
//         }
//     }
//
//     /* If we got here, then there's no rescheduling. */
//     KiReleaseDispatcherObject(&Gate->Header);
//     KeLowerIrql(OldIrql);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
