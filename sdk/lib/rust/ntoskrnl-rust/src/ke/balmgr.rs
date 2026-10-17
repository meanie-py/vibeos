use super::ke_defs::*;
use crate::CONTAINING_RECORD;
use ntkrnl_sys::inline::ke::KiInsertDeferredReadyList; // §9.56: inline (not the extern-C shim) so #[track_caller] names the true site
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm symbols not in ntkrnl-sys — declare as extern


use ntkrnl_sys::ddk::{
    _KTHREAD,
    KPRIORITY,
    KTIMER,
    KWAIT_BLOCK,
    LARGE_INTEGER as DDK_LARGE_INTEGER,
    PKTHREAD,
    KeInitializeDpc,
    KeInitializeTimerEx,
    KeSetPriorityThread,
    KeSetTimerEx,
    KeWaitForMultipleObjects,
    KiAcquireDispatcherLock,
    KiAcquirePrcbLock,
    KiReleaseDispatcherLock,
    KiReleasePrcbLock,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn MmWorkingSetManager();
    static mut MmWorkingSetManagerEvent: ntkrnl_sys::ddk::_KEVENT;
}
use super::processor::KeNumberProcessors;
use super::krnlinit::KiProcessorBlock;
// Use actual definition from clock.rs, not extern (lto=false splits them)
use super::clock::KeTickCount;

const LOW_REALTIME_PRIORITY: ULONG = 16;
// balmgr.c: `(LOW_REALTIME_PRIORITY - 1)`, the top dynamic priority. It was an
// invented 23 until 2026-10-17: the starvation scan then covered priorities up to
// 22 and boosted dynamic threads INTO the realtime range, where
// KiComputeNewPriority never decays them (its restored ASSERT caught pri=23 base=8
// dec=15 under kmtest).
const THREAD_BOOST_PRIORITY: ULONG = LOW_REALTIME_PRIORITY - 1;
const WAIT_QUANTUM_DECREMENT: ULONG = 1;

static mut KiReadyScanLast: ULONG = 0;

#[inline]
fn PRIORITY_MASK(priority: ULONG) -> ULONG {
    1 << priority
}

#[no_mangle]
pub unsafe extern "system" fn KiScanReadyQueues(
    Dpc: *mut ntkrnl_sys::ddk::_KDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let Dpc = opaque_ptr(Dpc);
    let DeferredContext = opaque_ptr(DeferredContext);
    let SystemArgument1 = opaque_ptr(SystemArgument1);
    let SystemArgument2 = opaque_ptr(SystemArgument2);
    let ScanLast = DeferredContext as *mut ULONG;
    let mut ScanIndex = *ScanLast;
    let mut Count = 10u32;
    let mut Number = 16u32;
    let Prcb = *(core::ptr::addr_of!(KiProcessorBlock) as *const PKPRCB).add(ScanIndex as usize);
    let mut Index = (*Prcb).QueueIndex;
    let WaitLimit = (*core::ptr::addr_of!(KeTickCount)).LowPart - 300;
    let mut Summary: ULONG;
    let mut OldIrql: KIRQL;
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut Thread: PKTHREAD;

    OldIrql = KiAcquireDispatcherLock();
    KiAcquirePrcbLock(Prcb);

    Summary = (*Prcb).ReadySummary & ((1 << THREAD_BOOST_PRIORITY) - 2);
    if Summary != 0 {
        loop {
            if Index > (THREAD_BOOST_PRIORITY - 1) {
                Index = 1;
            }

            if (Summary & PRIORITY_MASK(Index)) != 0 {
                ntkrnl_sys::ASSERT!(!IsListEmpty(core::ptr::addr_of_mut!((*Prcb).DispatcherReadyListHead[Index as usize]) as PLIST_ENTRY));

                Summary ^= PRIORITY_MASK(Index);
                ListHead = core::ptr::addr_of_mut!((*Prcb).DispatcherReadyListHead[Index as usize]) as PLIST_ENTRY;
                NextEntry = (*ListHead).Flink;

                loop {
                    Thread = CONTAINING_RECORD!(NextEntry, _KTHREAD, __bindgen_anon_3.WaitListEntry) as PKTHREAD;
                    ntkrnl_sys::ASSERT!((*Thread).Priority == Index as i8);

                    if WaitLimit >= *kt_WaitTime(Thread) {
                        NextEntry = (*NextEntry).Blink;
                        ntkrnl_sys::ASSERT!(((*Prcb).ReadySummary & PRIORITY_MASK(Index)) != 0);
                        if RemoveEntryList((*NextEntry).Flink) {
                            (*Prcb).ReadySummary ^= PRIORITY_MASK(Index);
                        }

                        ntkrnl_sys::ASSERT!(((*Thread).PriorityDecrement >= 0) &&
                            ((*Thread).PriorityDecrement <= (*Thread).Priority));
                        (*Thread).PriorityDecrement += (THREAD_BOOST_PRIORITY as i8 - (*Thread).Priority);
                        ntkrnl_sys::ASSERT!(((*Thread).PriorityDecrement >= 0) &&
                            ((*Thread).PriorityDecrement <= THREAD_BOOST_PRIORITY as i8));

                        (*Thread).Priority = THREAD_BOOST_PRIORITY as i8;
                        *kt_Quantum(Thread) = (WAIT_QUANTUM_DECREMENT * 4) as i8;
                        // §9.56 in-flight oracle: legal repositioning (removed
                        // from the ready queue above) — requeue, not a wake.
                        #[cfg(feature = "smp")]
                        ntkrnl_sys::inline::ke::KiInsertDeferredReadyListRequeue(Thread);
                        #[cfg(not(feature = "smp"))]
                        KiInsertDeferredReadyList(Thread);
                        Count -= 1;
                    }

                    NextEntry = (*NextEntry).Flink;
                    Number -= 1;

                    if NextEntry == ListHead || Number == 0 || Count == 0 {
                        break;
                    }
                }
            }

            Index += 1;

            if Summary == 0 || Number == 0 || Count == 0 {
                break;
            }
        }
    }

    // §9.56: balmgr is the one cross-CPU ready-list writer (it locks and
    // mutates the SCANNED Prcb, not the current one). Validate every priority
    // it may have touched before releasing — site 0x44 fires if balmgr left a
    // torn list. Cheap: 32 checks once per (infrequent) scan.
    #[cfg(feature = "smp")]
    for p in 1..32u32 {
        ntkrnl_sys::inline::ke::ki_rq_integrity(Prcb, p, 0x44);
    }

    KiReleasePrcbLock(Prcb);
    KiReleaseDispatcherLock(OldIrql);

    if Count != 0 && Number != 0 {
        (*Prcb).QueueIndex = 1;
    } else {
        (*Prcb).QueueIndex = Index;
    }

    ScanIndex += 1;
    if ScanIndex == KeNumberProcessors as ULONG {
        ScanIndex = 0;
    }

    *ScanLast = ScanIndex;
}

#[no_mangle]
pub unsafe extern "system" fn KeBalanceSetManager(Context: PVOID) {
    let Context = opaque_ptr(Context);
    let mut ScanDpc: KDPC = core::mem::zeroed();
    let mut PeriodTimer: KTIMER = core::mem::zeroed();
    let mut DueTime: DDK_LARGE_INTEGER = core::mem::zeroed();
    let mut WaitBlockArray: [KWAIT_BLOCK; 1] = core::mem::zeroed();
    let mut WaitObjects: [PVOID; 1] = [core::ptr::null_mut()];
    let mut Status: NTSTATUS;

    KeSetPriorityThread(PsGetCurrentThread(), LOW_REALTIME_PRIORITY as KPRIORITY);

    // SynchronizationTimer (1) — auto-reset on wait satisfaction. Using
    // NotificationTimer (0) makes this loop spin: the timer stays signaled
    // after the first fire and KeWaitForMultipleObjects returns immediately
    // forever, burning ~40% CPU.
    KeInitializeTimerEx(&mut PeriodTimer as *mut KTIMER, 1);
    KeInitializeDpc(
        &mut ScanDpc as *mut KDPC as *mut _,
        core::mem::transmute(KiScanReadyQueues as usize),
        core::ptr::addr_of_mut!(KiReadyScanLast) as PVOID,
    );

    DueTime.QuadPart = -1i64 * 10 * 1000 * 1000;
    KeSetTimerEx(
        &mut PeriodTimer as *mut KTIMER,
        DueTime,
        1000,
        &mut ScanDpc as *mut KDPC as *mut _,
    );

    WaitObjects[0] = &mut PeriodTimer as *mut KTIMER as PVOID;

    loop {
        Status = KeWaitForMultipleObjects(
            1,
            WaitObjects.as_mut_ptr(),
            ntkrnl_sys::ddk::_WAIT_TYPE_WaitAny as _,
            0,
            0i8,
            0u8,
            core::ptr::null_mut(),
            WaitBlockArray.as_mut_ptr(),
        );

        match Status {
            ntkrnl_sys::mm::STATUS_WAIT_0 => {
            }
            ntkrnl_sys::mm::STATUS_WAIT_1 => {
            }
            _ => {
            }
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * PROJECT:         ReactOS Kernel
// * LICENSE:         GPL - See COPYING in the top level directory
// * FILE:            ntoskrnl/ke/balmgr.c
// * PURPOSE:         Balance Set Manager
// * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
// */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// #define THREAD_BOOST_PRIORITY (LOW_REALTIME_PRIORITY - 1)
// ULONG KiReadyScanLast;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KiScanReadyQueues(IN PKDPC Dpc,
//                   IN PVOID DeferredContext,
//                   IN PVOID SystemArgument1,
//                   IN PVOID SystemArgument2)
// {
//     PULONG ScanLast = DeferredContext;
//     ULONG ScanIndex = *ScanLast;
//     ULONG Count = 10, Number = 16;
//     PKPRCB Prcb = KiProcessorBlock[ScanIndex];
//     ULONG Index = Prcb->QueueIndex;
//     ULONG WaitLimit = KeTickCount.LowPart - 300;
//     ULONG Summary;
//     KIRQL OldIrql;
//     PLIST_ENTRY ListHead, NextEntry;
//     PKTHREAD Thread;
//
//     /* Lock the dispatcher and PRCB */
//     OldIrql = KiAcquireDispatcherLock();
//     KiAcquirePrcbLock(Prcb);
//     /* Check if there's any thread that need help */
//     Summary = Prcb->ReadySummary & ((1 << THREAD_BOOST_PRIORITY) - 2);
//     if (Summary)
//     {
//         /* Start scan loop */
//         do
//         {
//             /* Normalize the index */
//             if (Index > (THREAD_BOOST_PRIORITY - 1)) Index = 1;
//
//             /* Loop for ready threads */
//             if (Summary & PRIORITY_MASK(Index))
//             {
//                 /* Sanity check */
//                 ASSERT(!IsListEmpty(&Prcb->DispatcherReadyListHead[Index]));
//
//                 /* Update summary and select list */
//                 Summary ^= PRIORITY_MASK(Index);
//                 ListHead = &Prcb->DispatcherReadyListHead[Index];
//                 NextEntry = ListHead->Flink;
//                 do
//                 {
//                     /* Select a thread */
//                     Thread = CONTAINING_RECORD(NextEntry,
//                                                KTHREAD,
//                                                WaitListEntry);
//                     ASSERT(Thread->Priority == Index);
//
//                     /* Check if the thread has been waiting too long */
//                     if (WaitLimit >= Thread->WaitTime)
//                     {
//                         /* Remove the thread from the queue */
//                         NextEntry = NextEntry->Blink;
//                         ASSERT((Prcb->ReadySummary & PRIORITY_MASK(Index)));
//                         if (RemoveEntryList(NextEntry->Flink))
//                         {
//                             /* The list is empty now */
//                             Prcb->ReadySummary ^= PRIORITY_MASK(Index);
//                         }
//
//                         /* Verify priority decrement and set the new one */
//                         ASSERT((Thread->PriorityDecrement >= 0) &&
//                                (Thread->PriorityDecrement <=
//                                 Thread->Priority));
//                         Thread->PriorityDecrement += (THREAD_BOOST_PRIORITY -
//                                                       Thread->Priority);
//                         ASSERT((Thread->PriorityDecrement >= 0) &&
//                                (Thread->PriorityDecrement <=
//                                 THREAD_BOOST_PRIORITY));
//
//                         /* Update priority and insert into ready list */
//                         Thread->Priority = THREAD_BOOST_PRIORITY;
//                         Thread->Quantum = WAIT_QUANTUM_DECREMENT * 4;
//                         KiInsertDeferredReadyList(Thread);
//                         Count --;
//                     }
//
//                     /* Go to the next entry */
//                     NextEntry = NextEntry->Flink;
//                     Number--;
//                 } while((NextEntry != ListHead) && (Number) && (Count));
//             }
//
//             /* Increase index */
//             Index++;
//         } while ((Summary) && (Number) && (Count));
//     }
//
//     /* Release the locks and dispatcher */
//     KiReleasePrcbLock(Prcb);
//     KiReleaseDispatcherLock(OldIrql);
//
//     /* Update the queue index for next time */
//     if ((Count) && (Number))
//     {
//         /* Reset the queue at index 1 */
//         Prcb->QueueIndex = 1;
//     }
//     else
//     {
//         /* Set the index we're in now */
//         Prcb->QueueIndex = Index;
//     }
//
//     /* Increment the CPU number for next time and normalize to CPU count */
//     ScanIndex++;
//     if (ScanIndex == KeNumberProcessors) ScanIndex = 0;
//
//     /* Return the index */
//     *ScanLast = ScanIndex;
// }
//
// VOID
// NTAPI
// KeBalanceSetManager(IN PVOID Context)
// {
//     KDPC ScanDpc;
//     KTIMER PeriodTimer;
//     LARGE_INTEGER DueTime;
//     KWAIT_BLOCK WaitBlockArray[1];
//     PVOID WaitObjects[1];
//     NTSTATUS Status;
//
//     /* Set us at a low real-time priority level */
//     KeSetPriorityThread(KeGetCurrentThread(), LOW_REALTIME_PRIORITY);
//
//     /* Setup the timer and scanner DPC */
//     KeInitializeTimerEx(&PeriodTimer, SynchronizationTimer);
//     KeInitializeDpc(&ScanDpc, KiScanReadyQueues, &KiReadyScanLast);
//
//     /* Setup the periodic timer */
//     DueTime.QuadPart = -1 * 10 * 1000 * 1000;
//     KeSetTimerEx(&PeriodTimer, DueTime, 1000, &ScanDpc);
//
//     /* Setup the wait objects */
//     WaitObjects[0] = &PeriodTimer;
//     //WaitObjects[1] = MmWorkingSetManagerEvent; // NO WS Management Yet!
//
//     /* Start wait loop */
//     do
//     {
//         /* Wait on our objects */
//         Status = KeWaitForMultipleObjects(1,
//                                           WaitObjects,
//                                           WaitAny,
//                                           Executive,
//                                           KernelMode,
//                                           FALSE,
//                                           NULL,
//                                           WaitBlockArray);
//         switch (Status)
//         {
//             /* Check if our timer expired */
//             case STATUS_WAIT_0:
//
//                 /* Adjust lookaside lists */
//                 //ExAdjustLookasideDepth();
//
//                 /* Call the working set manager */
//                 //MmWorkingSetManager();
//
//                 /* FIXME: Outswap stacks */
//
//                 /* Done */
//                 break;
//
//             /* Check if the working set manager notified us */
//             case STATUS_WAIT_1:
//
//                 /* Call the working set manager */
//                 //MmWorkingSetManager();
//                 break;
//
//             /* Anything else */
//             default:
//                 DPRINT1("BALMGR: Illegal wait status, %lx =\n", Status);
//                 break;
//         }
//     } while (TRUE);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
