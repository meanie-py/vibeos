use super::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::CONTAINING_RECORD;

use ntkrnl_sys::ddk::{
    KPROCESSOR_MODE, KQUEUE, KTHREAD, KWAIT_BLOCK, LONG_PTR,
    KiAcquireDispatcherLock, KiAcquireDispatcherLockAtSynchLevel,
    KiExitDispatcher, KiReadyThread, KiRecalculateDueTime,
    KiReleaseDispatcherLock, KiReleaseDispatcherLockFromSynchLevel,
    KiSetThreadSwapBusy, KiSwapThread, KiUnwaitThread,
    LARGE_INTEGER, PKQUEUE, PKTHREAD, PKTIMER, PKWAIT_BLOCK,
    PLARGE_INTEGER, PWORK_QUEUE_ITEM,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn KxRemoveTreeTimer(Timer: PKTIMER);
    fn KxQueueThreadWait(
        Queue: PVOID,
        Timeout: PLARGE_INTEGER,
        WaitMode: KPROCESSOR_MODE,
        Swappable: *mut BOOLEAN,
        DueTime: PLARGE_INTEGER,
        Hand: *mut ULONG,
    );
    fn KiAddThreadToWaitList(Thread: PKTHREAD, Swappable: BOOLEAN);
    fn KxInsertTimer(Timer: PKTIMER, Hand: ULONG);
}

use super::processor::KeNumberProcessors;

const TIMER_WAIT_BLOCK: usize = 3;
const IO_NO_INCREMENT: i32 = 0;
const QueueObject: u8 = 4;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiActivateWaiterQueue(Queue: PKQUEUE) {
    KiActivateWaiterQueueImpl(Queue)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiActivateWaiterQueue(Queue: PKQUEUE) {
    KiActivateWaiterQueueImpl(Queue)
}

#[inline(always)]
unsafe fn KiActivateWaiterQueueImpl(Queue: PKQUEUE) {
    let mut QueueEntry: PLIST_ENTRY;
    let mut WaitEntry: PLIST_ENTRY;
    let mut WaitBlock: PKWAIT_BLOCK;
    let mut Thread: PKTHREAD;

    (*Queue).CurrentCount -= 1;

    if (*Queue).CurrentCount < (*Queue).MaximumCount {
        QueueEntry = (*Queue).EntryListHead.Flink;
        WaitEntry = (*Queue).Header.WaitListHead.Blink;

        if (WaitEntry != core::ptr::addr_of_mut!((*Queue).Header.WaitListHead) as PLIST_ENTRY) &&
           (QueueEntry != core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY) {
            // waitdep: WaitBlock->Thread is consumed off the queue's wait list
            // below with no validation (unchecked KiUnwaitThread funnel; see
            // eventobj.rs KeSetEventBoostPriority for the family).
            super::waitdep::check_waittest(
                Queue as usize,
                core::ptr::addr_of_mut!((*Queue).Header.WaitListHead) as *mut _,
                false,
            );
            RemoveEntryList(QueueEntry);
            (*QueueEntry).Flink = core::ptr::null_mut();

            (*Queue).Header.SignalState -= 1;

            WaitBlock = CONTAINING_RECORD!(WaitEntry, KWAIT_BLOCK, WaitListEntry);
            Thread = (*WaitBlock).Thread;
            KiUnwaitThread(Thread, QueueEntry as LONG_PTR, IO_NO_INCREMENT);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn KiInsertQueue(
    Queue: PKQUEUE,
    Entry: PLIST_ENTRY,
    Head: BOOLEAN,
) -> LONG {
    let Entry = opaque_ptr(Entry);
    let InitialState: LONG;
    let mut Thread: PKTHREAD = KeGetCurrentThread() as PKTHREAD;
    let mut WaitBlock: PKWAIT_BLOCK;
    let mut WaitEntry: PLIST_ENTRY;
    let mut Timer: PKTIMER;

    InitialState = (*Queue).Header.SignalState;
    WaitEntry = (*Queue).Header.WaitListHead.Blink;

    if ((*Queue).CurrentCount < (*Queue).MaximumCount) &&
       (WaitEntry != core::ptr::addr_of_mut!((*Queue).Header.WaitListHead) as PLIST_ENTRY) &&
       (((*Thread).Queue != Queue) || ((*Thread).WaitReason != _KWAIT_REASON_WrQueue as u8)) {
        RemoveEntryList(WaitEntry);

        WaitBlock = CONTAINING_RECORD!(WaitEntry, KWAIT_BLOCK, WaitListEntry);
        Thread = (*WaitBlock).Thread;

        // §9.56 tripwire 0x60 — the LAST unguarded readier. This manual queue
        // wake bypasses KiUnwaitThread (so 0x5E can't see it) and its
        // fingerprint (WaitReason=0) matched 2/3 of the 0x5D victims. A popped
        // queue waiter must be Waiting; anything else = a stale queue wait
        // block waking a running thread.
        #[cfg(feature = "smp")]
        {
            if *kt_State(Thread) != 5u8 {
                crate::dbg_print!(
                    b"[RQ60] queue wake popped non-Waiting thr=%p state=%u queue=%p reason=%u\n\0",
                    Thread,
                    *kt_State(Thread) as u32,
                    Queue,
                    (*Thread).WaitReason as u32,
                );
                if ntkrnl_sys::inline::ke::RQ_ORACLE_ARMED {
                KeBugCheckEx(
                    ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                    Thread as ULONG_PTR,
                    ((0x60u64 << 40) | ((*kt_State(Thread)) as u64) << 8) as ULONG_PTR,
                    Queue as ULONG_PTR,
                    WaitBlock as ULONG_PTR,
                );
                }
            }
        }

        (*Thread).WaitStatus = Entry as LONG_PTR;
        if !(*Thread).__bindgen_anon_3.WaitListEntry.Flink.is_null() {
            RemoveEntryList(core::ptr::addr_of_mut!((*Thread).__bindgen_anon_3.WaitListEntry) as PLIST_ENTRY);
        }

        (*Queue).CurrentCount += 1;
        (*Thread).WaitReason = 0;

        Timer = kt_Timer(Thread) as *mut _ as PKTIMER;
        if (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted != 0 {
            KxRemoveTreeTimer(Timer);
        }

        KiReadyThread(Thread);
    } else {
        (*Queue).Header.SignalState += 1;

        if Head != 0 {
            InsertHeadList(core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY, Entry);
        } else {
            InsertTailList(core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY, Entry);
        }
    }

    InitialState
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeQueue(
    Queue: PKQUEUE,
    Count: ULONG,
) {
    (*Queue).Header.__bindgen_anon_1.__bindgen_anon_1.Type = QueueObject;
    (*Queue).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.Abandoned = 0;
    (*Queue).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<KQUEUE>() / core::mem::size_of::<ULONG>()) as UCHAR;
    (*Queue).Header.SignalState = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Queue).Header.WaitListHead) as PLIST_ENTRY);

    InitializeListHead(core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY);
    InitializeListHead(core::ptr::addr_of_mut!((*Queue).ThreadListHead) as PLIST_ENTRY);

    (*Queue).CurrentCount = 0;
    (*Queue).MaximumCount = if Count == 0 { KeNumberProcessors as ULONG } else { Count };
}

#[no_mangle]
pub unsafe extern "system" fn KeInsertHeadQueue(
    Queue: PKQUEUE,
    Entry: PLIST_ENTRY,
) -> LONG {
    let Entry = opaque_ptr(Entry);
    let PreviousState: LONG;
    let OldIrql: KIRQL;

    OldIrql = KiAcquireDispatcherLock();
    PreviousState = KiInsertQueue(Queue, Entry, 1u8);
    KiReleaseDispatcherLock(OldIrql);

    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeInsertQueue(
    Queue: PKQUEUE,
    Entry: PLIST_ENTRY,
) -> LONG {
    let Entry = opaque_ptr(Entry);
    let PreviousState: LONG;
    let OldIrql: KIRQL;

    OldIrql = KiAcquireDispatcherLock();
    PreviousState = KiInsertQueue(Queue, Entry, 0u8);
    KiReleaseDispatcherLock(OldIrql);

    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeReadStateQueue(Queue: PKQUEUE) -> LONG {
    (*Queue).Header.SignalState as LONG
}

/// `KeRemoveQueue` and `KeRemoveQueueEx` share this. `Alertable` is what the wait
/// stores in `Thread->Alertable` (KxQueueThreadWait clears it): with it set, a user APC
/// queued to a UserMode waiter unwaits it with STATUS_USER_APC (KiInsertQueueApc), which
/// is what GetQueuedCompletionStatusEx(fAlertable=TRUE) relies on to run APCs.
unsafe fn ki_remove_queue(
    Queue: PKQUEUE,
    WaitMode: KPROCESSOR_MODE,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
) -> PLIST_ENTRY {
    let Timeout = opaque_ptr(Timeout);
    let mut QueueEntry: PLIST_ENTRY;
    let mut Status: LONG_PTR;
    let Thread: PKTHREAD = KeGetCurrentThread() as PKTHREAD;
    let mut PreviousQueue: PKQUEUE;
    let WaitBlock: PKWAIT_BLOCK = &mut kt_WaitBlock(Thread)[0] as PKWAIT_BLOCK;
    let _TimerBlock: PKWAIT_BLOCK = &mut kt_WaitBlock(Thread)[TIMER_WAIT_BLOCK] as PKWAIT_BLOCK;
    let Timer: PKTIMER = kt_Timer(Thread) as *mut _ as PKTIMER;
    let mut Swappable: BOOLEAN = 0u8;
    let OriginalDueTime: PLARGE_INTEGER = Timeout;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    let mut NewDueTime: LARGE_INTEGER = core::mem::zeroed();
    let mut InterruptTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Hand: ULONG = 0;
    let mut Timeout = Timeout;

    if (*Thread).WaitNext != 0 {
        (*Thread).WaitNext = 0u8;
        KxQueueThreadWait(
                Queue as PVOID, Timeout, WaitMode,
                core::ptr::addr_of_mut!(Swappable),
                core::ptr::addr_of_mut!(DueTime),
                core::ptr::addr_of_mut!(Hand),
            );
        (*Thread).Alertable = Alertable;
    } else {
        *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
        KxQueueThreadWait(
                Queue as PVOID, Timeout, WaitMode,
                core::ptr::addr_of_mut!(Swappable),
                core::ptr::addr_of_mut!(DueTime),
                core::ptr::addr_of_mut!(Hand),
            );
        (*Thread).Alertable = Alertable;
        KiAcquireDispatcherLockAtSynchLevel();
    }

    PreviousQueue = (*Thread).Queue;
    (*Thread).Queue = Queue;

    if Queue != PreviousQueue {
        QueueEntry = core::ptr::addr_of_mut!((*Thread).QueueListEntry) as PLIST_ENTRY;
        if !PreviousQueue.is_null() {
            RemoveEntryList(QueueEntry);
            KiActivateWaiterQueue(PreviousQueue);
        }

        InsertTailList(
            core::ptr::addr_of_mut!((*Queue).ThreadListHead) as PLIST_ENTRY,
            QueueEntry
        );
    } else {
        (*Queue).CurrentCount -= 1;
    }

    loop {
        QueueEntry = (*Queue).EntryListHead.Flink;
        if ((*Queue).CurrentCount < (*Queue).MaximumCount) &&
           (QueueEntry != core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY) {
            (*Queue).Header.SignalState -= 1;
            (*Queue).CurrentCount += 1;

            if (*QueueEntry).Flink.is_null() || (*QueueEntry).Blink.is_null() {
                KeBugCheckEx(
                    ntkrnl_sys::bugcodes::INVALID_WORK_QUEUE_ITEM,
                    QueueEntry as ULONG_PTR,
                    Queue as ULONG_PTR,
                    0,
                    // WorkerRoutine is Option<fn>, extract raw pointer value
                    match (*(QueueEntry as PWORK_QUEUE_ITEM)).WorkerRoutine {
                        Some(f) => f as usize as ULONG_PTR,
                        None => 0 as ULONG_PTR,
                    },
                );
            }

            RemoveEntryList(QueueEntry);
            (*QueueEntry).Flink = core::ptr::null_mut();

            break;
        } else {
            if (kt_ApcState(Thread).KernelApcPending != 0) &&
               (*kt_SpecialApcDisable(Thread) == 0) &&
               (*kt_WaitIrql(Thread) < APC_LEVEL) {
                (*Queue).CurrentCount += 1;
                KiReleaseDispatcherLockFromSynchLevel();
                KiExitDispatcher(*kt_WaitIrql(Thread));
            } else {
                // The same test every dispatcher wait makes: with Alertable clear it is the
                // old "user APC already pending" check; with it set, a user APC queued
                // BEFORE the wait (or an alert) also ends it, which the APC-insert wakeup
                // cannot cover because the thread was not waiting yet.
                let alert_status = ntkrnl_sys::inline::ke::KiCheckAlertability(
                    Thread as _,
                    Alertable,
                    WaitMode,
                );
                if alert_status != 0 {
                    QueueEntry = alert_status as isize as PLIST_ENTRY;
                    (*Queue).CurrentCount += 1;
                    break;
                }

                if !Timeout.is_null() {
                    InterruptTime.QuadPart = KeQueryInterruptTime() as i64;
                    if (InterruptTime.QuadPart as u64) >= (*Timer).DueTime.QuadPart {
                        QueueEntry = STATUS_TIMEOUT as isize as PLIST_ENTRY;
                        (*Queue).CurrentCount += 1;
                        break;
                    }

                    (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 1u8;
                }

                InsertTailList(
                    core::ptr::addr_of_mut!((*Queue).Header.WaitListHead) as PLIST_ENTRY,
                    core::ptr::addr_of_mut!((*WaitBlock).WaitListEntry) as PLIST_ENTRY,
                );

                *kt_State(Thread) = 5u8; // Waiting

                KiAddThreadToWaitList(Thread, Swappable);
                KiSetThreadSwapBusy(Thread);

                if !Timeout.is_null() {
                    KxInsertTimer(Timer, Hand);
                } else {
                    KiReleaseDispatcherLockFromSynchLevel();
                }

                Status = KiSwapThread(Thread, KeGetCurrentPrcb());
                (*Thread).WaitReason = 0;

                if Status != STATUS_KERNEL_APC as LONG_PTR {
                    return Status as usize as PLIST_ENTRY;
                }

                if !Timeout.is_null() {
                    Timeout = KiRecalculateDueTime(
                        OriginalDueTime,
                        core::ptr::addr_of_mut!(DueTime),
                        core::ptr::addr_of_mut!(NewDueTime),
                    );
                }
            }

            *kt_WaitIrql(Thread) = KeRaiseIrqlToSynchLevel();
            KxQueueThreadWait(
                Queue as PVOID, Timeout, WaitMode,
                core::ptr::addr_of_mut!(Swappable),
                core::ptr::addr_of_mut!(DueTime),
                core::ptr::addr_of_mut!(Hand),
            );
            (*Thread).Alertable = Alertable;
            KiAcquireDispatcherLockAtSynchLevel();
            (*Queue).CurrentCount -= 1;
        }
    }

    KiReleaseDispatcherLockFromSynchLevel();
    KiExitDispatcher(*kt_WaitIrql(Thread));
    QueueEntry
}

#[no_mangle]
pub unsafe extern "system" fn KeRemoveQueue(
    Queue: PKQUEUE,
    WaitMode: KPROCESSOR_MODE,
    Timeout: PLARGE_INTEGER,
) -> PLIST_ENTRY {
    ki_remove_queue(Queue, WaitMode, 0u8, Timeout)
}

/// Windows 7's batched dequeue: up to `Count` entries into `EntryArray`. Only the first
/// removal waits (alertably if asked, until `Timeout`); the rest take what is already
/// queued. Returns the number of slots written. When the wait ends without an entry,
/// slot 0 holds the wait status (STATUS_TIMEOUT, STATUS_USER_APC...) and the return is 1,
/// as from `KeRemoveQueue`, so a caller tests each entry the same way.
#[no_mangle]
pub unsafe extern "system" fn KeRemoveQueueEx(
    Queue: PKQUEUE,
    WaitMode: KPROCESSOR_MODE,
    Alertable: BOOLEAN,
    Timeout: PLARGE_INTEGER,
    EntryArray: *mut PLIST_ENTRY,
    Count: ULONG,
) -> ULONG {
    if Count == 0 || EntryArray.is_null() {
        return 0;
    }
    let first = ki_remove_queue(Queue, WaitMode, Alertable, Timeout);
    core::ptr::write_volatile(EntryArray, first);
    if queue_status(first) {
        return 1;
    }
    let mut zero: LARGE_INTEGER = core::mem::zeroed();
    let mut n: ULONG = 1;
    while n < Count {
        let entry = ki_remove_queue(Queue, WaitMode, 0u8, &mut zero);
        if queue_status(entry) {
            break;
        }
        core::ptr::write_volatile(EntryArray.add(n as usize), entry);
        n += 1;
    }
    n
}

/// A KeRemoveQueue result that is an NTSTATUS, not a list entry (no entry lives in the
/// first page).
#[inline]
pub fn queue_status(entry: PLIST_ENTRY) -> bool {
    (entry as usize) < 0x1000
}

#[no_mangle]
pub unsafe extern "system" fn KeRundownQueue(Queue: PKQUEUE) -> PLIST_ENTRY {
    let mut FirstEntry: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut Thread: PKTHREAD;
    let OldIrql: KIRQL;

    OldIrql = KiAcquireDispatcherLock();

    FirstEntry = (*Queue).EntryListHead.Flink;
    if FirstEntry == core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY {
        FirstEntry = core::ptr::null_mut();
    } else {
        RemoveEntryList(core::ptr::addr_of_mut!((*Queue).EntryListHead) as PLIST_ENTRY);
    }

    while !IsListEmpty(core::ptr::addr_of_mut!((*Queue).ThreadListHead) as PLIST_ENTRY) {
        NextEntry = (*Queue).ThreadListHead.Flink;

        Thread = CONTAINING_RECORD!(NextEntry, KTHREAD, QueueListEntry);

        (*Thread).Queue = core::ptr::null_mut();
        RemoveEntryList(NextEntry);
    }

    KiReleaseDispatcherLockFromSynchLevel();
    KiExitDispatcher(OldIrql);
    FirstEntry
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/queue.c
//  * PURPOSE:         Implements kernel queues
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Gunnar Dalsnes
//  *                  Eric Kohl
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
// /*
//  * Called when a thread which has a queue entry is entering a wait state
//  */
// VOID
// FASTCALL
// KiActivateWaiterQueue(IN PKQUEUE Queue)
// {
//     PLIST_ENTRY QueueEntry;
//     PLIST_ENTRY WaitEntry;
//     PKWAIT_BLOCK WaitBlock;
//     PKTHREAD Thread;
//     ASSERT_QUEUE(Queue);
//
//     /* Decrement the number of active threads */
//     Queue->CurrentCount--;
//
//     /* Make sure the counts are OK */
//     if (Queue->CurrentCount < Queue->MaximumCount)
//     {
//         /* Get the Queue Entry */
//         QueueEntry = Queue->EntryListHead.Flink;
//
//         /* Get the Wait Entry */
//         WaitEntry = Queue->Header.WaitListHead.Blink;
//
//         /* Make sure that the Queue entries are not part of empty lists */
//         if ((WaitEntry != &Queue->Header.WaitListHead) &&
//             (QueueEntry != &Queue->EntryListHead))
//         {
//             /* Remove this entry */
//             RemoveEntryList(QueueEntry);
//             QueueEntry->Flink = NULL;
//
//             /* Decrease the Signal State */
//             Queue->Header.SignalState--;
//
//             /* Unwait the Thread */
//             WaitBlock = CONTAINING_RECORD(WaitEntry,
//                                           KWAIT_BLOCK,
//                                           WaitListEntry);
//             Thread = WaitBlock->Thread;
//             KiUnwaitThread(Thread, (LONG_PTR)QueueEntry, IO_NO_INCREMENT);
//         }
//     }
// }
//
// /*
//  * Returns the previous number of entries in the queue
//  */
// LONG
// NTAPI
// KiInsertQueue(IN PKQUEUE Queue,
//               IN PLIST_ENTRY Entry,
//               IN BOOLEAN Head)
// {
//     ULONG InitialState;
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKWAIT_BLOCK WaitBlock;
//     PLIST_ENTRY WaitEntry;
//     PKTIMER Timer;
//     ASSERT_QUEUE(Queue);
//
//     /* Save the old state */
//     InitialState = Queue->Header.SignalState;
//
//     /* Get the Entry */
//     WaitEntry = Queue->Header.WaitListHead.Blink;
//
//     /*
//      * Why the KeGetCurrentThread()->Queue != Queue?
//      * KiInsertQueue might be called from an APC for the current thread.
//      * -Gunnar
//      */
//     if ((Queue->CurrentCount < Queue->MaximumCount) &&
//         (WaitEntry != &Queue->Header.WaitListHead) &&
//         ((Thread->Queue != Queue) ||
//          (Thread->WaitReason != WrQueue)))
//     {
//         /* Remove the wait entry */
//         RemoveEntryList(WaitEntry);
//
//         /* Get the Wait Block and Thread */
//         WaitBlock = CONTAINING_RECORD(WaitEntry, KWAIT_BLOCK, WaitListEntry);
//         Thread = WaitBlock->Thread;
//
//         /* Remove the queue from the thread's wait list */
//         Thread->WaitStatus = (LONG_PTR)Entry;
//         if (Thread->WaitListEntry.Flink) RemoveEntryList(&Thread->WaitListEntry);
//
//         /* Increase the active threads and remove any wait reason */
//         Queue->CurrentCount++;
//         Thread->WaitReason = 0;
//
//         /* Check if there's a Thread Timer */
//         Timer = &Thread->Timer;
//         if (Timer->Header.Inserted) KxRemoveTreeTimer(Timer);
//
//         /* Reschedule the Thread */
//         KiReadyThread(Thread);
//     }
//     else
//     {
//         /* Increase the Entries */
//         Queue->Header.SignalState++;
//
//         /* Check which mode we're using */
//         if (Head)
//         {
//             /* Insert in the head */
//             InsertHeadList(&Queue->EntryListHead, Entry);
//         }
//         else
//         {
//             /* Insert at the end */
//             InsertTailList(&Queue->EntryListHead, Entry);
//         }
//     }
//
//     /* Return the previous state */
//     return InitialState;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeQueue(IN PKQUEUE Queue,
//                   IN ULONG Count OPTIONAL)
// {
//     /* Initialize the Header */
//     Queue->Header.Type = QueueObject;
//     Queue->Header.Abandoned = 0;
//     Queue->Header.Size = sizeof(KQUEUE) / sizeof(ULONG);
//     Queue->Header.SignalState = 0;
//     InitializeListHead(&(Queue->Header.WaitListHead));
//
//     /* Initialize the Lists */
//     InitializeListHead(&Queue->EntryListHead);
//     InitializeListHead(&Queue->ThreadListHead);
//
//     /* Set the Current and Maximum Count */
//     Queue->CurrentCount = 0;
//     Queue->MaximumCount = (Count == 0) ? (ULONG) KeNumberProcessors : Count;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeInsertHeadQueue(IN PKQUEUE Queue,
//                   IN PLIST_ENTRY Entry)
// {
//     LONG PreviousState;
//     KIRQL OldIrql;
//     ASSERT_QUEUE(Queue);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the Dispatcher Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Insert the Queue */
//     PreviousState = KiInsertQueue(Queue, Entry, TRUE);
//
//     /* Release the Dispatcher Lock */
//     KiReleaseDispatcherLock(OldIrql);
//
//     /* Return previous State */
//     return PreviousState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeInsertQueue(IN PKQUEUE Queue,
//               IN PLIST_ENTRY Entry)
// {
//     LONG PreviousState;
//     KIRQL OldIrql;
//     ASSERT_QUEUE(Queue);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the Dispatcher Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Insert the Queue */
//     PreviousState = KiInsertQueue(Queue, Entry, FALSE);
//
//     /* Release the Dispatcher Lock */
//     KiReleaseDispatcherLock(OldIrql);
//
//     /* Return previous State */
//     return PreviousState;
// }
//
// /*
//  * @implemented
//  *
//  * Returns number of entries in the queue
//  */
// LONG
// NTAPI
// KeReadStateQueue(IN PKQUEUE Queue)
// {
//     /* Returns the Signal State */
//     ASSERT_QUEUE(Queue);
//     return Queue->Header.SignalState;
// }
//
// /*
//  * @implemented
//  */
// PLIST_ENTRY
// NTAPI
// KeRemoveQueue(IN PKQUEUE Queue,
//               IN KPROCESSOR_MODE WaitMode,
//               IN PLARGE_INTEGER Timeout OPTIONAL)
// {
//     PLIST_ENTRY QueueEntry;
//     LONG_PTR Status;
//     PKTHREAD Thread = KeGetCurrentThread();
//     PKQUEUE PreviousQueue;
//     PKWAIT_BLOCK WaitBlock = &Thread->WaitBlock[0];
//     PKWAIT_BLOCK TimerBlock = &Thread->WaitBlock[TIMER_WAIT_BLOCK];
//     PKTIMER Timer = &Thread->Timer;
//     BOOLEAN Swappable;
//     PLARGE_INTEGER OriginalDueTime = Timeout;
//     LARGE_INTEGER DueTime = {{0}}, NewDueTime, InterruptTime;
//     ULONG Hand = 0;
//     ASSERT_QUEUE(Queue);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Check if the Lock is already held */
//     if (Thread->WaitNext)
//     {
//         /* It is, so next time don't do expect this */
//         Thread->WaitNext = FALSE;
//         KxQueueThreadWait();
//     }
//     else
//     {
//         /* Raise IRQL to synch, prepare the wait, then lock the database */
//         Thread->WaitIrql = KeRaiseIrqlToSynchLevel();
//         KxQueueThreadWait();
//         KiAcquireDispatcherLockAtSynchLevel();
//     }
//
//     /*
//      * This is needed so that we can set the new queue right here,
//      * before additional processing
//      */
//     PreviousQueue = Thread->Queue;
//     Thread->Queue = Queue;
//
//     /* Check if this is a different queue */
//     if (Queue != PreviousQueue)
//     {
//         /* Get the current entry */
//         QueueEntry = &Thread->QueueListEntry;
//         if (PreviousQueue)
//         {
//             /* Remove from this list */
//             RemoveEntryList(QueueEntry);
//
//             /* Wake the queue */
//             KiActivateWaiterQueue(PreviousQueue);
//         }
//
//         /* Insert in this new Queue */
//         InsertTailList(&Queue->ThreadListHead, QueueEntry);
//     }
//     else
//     {
//         /* Same queue, decrement waiting threads */
//         Queue->CurrentCount--;
//     }
//
//     /* Loop until the queue is processed */
//     while (TRUE)
//     {
//         /* Check if the counts are valid and if there is still a queued entry */
//         QueueEntry = Queue->EntryListHead.Flink;
//         if ((Queue->CurrentCount < Queue->MaximumCount) &&
//             (QueueEntry != &Queue->EntryListHead))
//         {
//             /* Decrease the number of entries */
//             Queue->Header.SignalState--;
//
//             /* Increase numbef of running threads */
//             Queue->CurrentCount++;
//
//             /* Check if the entry is valid. If not, bugcheck */
//             if (!(QueueEntry->Flink) || !(QueueEntry->Blink))
//             {
//                 /* Invalid item */
//                 KeBugCheckEx(INVALID_WORK_QUEUE_ITEM,
//                              (ULONG_PTR)QueueEntry,
//                              (ULONG_PTR)Queue,
//                              (ULONG_PTR)NULL,
//                              (ULONG_PTR)((PWORK_QUEUE_ITEM)QueueEntry)->
//                                          WorkerRoutine);
//             }
//
//             /* Remove the Entry */
//             RemoveEntryList(QueueEntry);
//             QueueEntry->Flink = NULL;
//
//             /* Nothing to wait on */
//             break;
//         }
//         else
//         {
//             /* Check if a kernel APC is pending and we're below APC_LEVEL */
//             if ((Thread->ApcState.KernelApcPending) &&
//                 !(Thread->SpecialApcDisable) && (Thread->WaitIrql < APC_LEVEL))
//             {
//                 /* Increment the count and unlock the dispatcher */
//                 Queue->CurrentCount++;
//                 KiReleaseDispatcherLockFromSynchLevel();
//                 KiExitDispatcher(Thread->WaitIrql);
//             }
//             else
//             {
//                 /* Fail if there's a User APC Pending */
//                 if ((WaitMode != KernelMode) &&
//                     (Thread->ApcState.UserApcPending))
//                 {
//                     /* Return the status and increase the pending threads */
//                     QueueEntry = (PLIST_ENTRY)STATUS_USER_APC;
//                     Queue->CurrentCount++;
//                     break;
//                 }
//
//                 /* Enable the Timeout Timer if there was any specified */
//                 if (Timeout)
//                 {
//                     /* Check if the timer expired */
//                     InterruptTime.QuadPart = KeQueryInterruptTime();
//                     if ((ULONG64)InterruptTime.QuadPart >= Timer->DueTime.QuadPart)
//                     {
//                         /* It did, so we don't need to wait */
//                         QueueEntry = (PLIST_ENTRY)STATUS_TIMEOUT;
//                         Queue->CurrentCount++;
//                         break;
//                     }
//
//                     /* It didn't, so activate it */
//                     Timer->Header.Inserted = TRUE;
//                 }
//
//                 /* Insert the wait block in the list */
//                 InsertTailList(&Queue->Header.WaitListHead,
//                                &WaitBlock->WaitListEntry);
//
//                 /* Setup the wait information */
//                 Thread->State = Waiting;
//
//                 /* Add the thread to the wait list */
//                 KiAddThreadToWaitList(Thread, Swappable);
//
//                 /* Activate thread swap */
//                 ASSERT(Thread->WaitIrql <= DISPATCH_LEVEL);
//                 KiSetThreadSwapBusy(Thread);
//
//                 /* Check if we have a timer */
//                 if (Timeout)
//                 {
//                     /* Insert it */
//                     KxInsertTimer(Timer, Hand);
//                 }
//                 else
//                 {
//                     /* Otherwise, unlock the dispatcher */
//                     KiReleaseDispatcherLockFromSynchLevel();
//                 }
//
//                 /* Do the actual swap */
//                 Status = KiSwapThread(Thread, KeGetCurrentPrcb());
//
//                 /* Reset the wait reason */
//                 Thread->WaitReason = 0;
//
//                 /* Check if we were executing an APC */
//                 if (Status != STATUS_KERNEL_APC) return (PLIST_ENTRY)Status;
//
//                 /* Check if we had a timeout */
//                 if (Timeout)
//                 {
//                     /* Recalculate due times */
//                     Timeout = KiRecalculateDueTime(OriginalDueTime,
//                                                    &DueTime,
//                                                    &NewDueTime);
//                 }
//             }
//
//             /* Start another wait */
//             Thread->WaitIrql = KeRaiseIrqlToSynchLevel();
//             KxQueueThreadWait();
//             KiAcquireDispatcherLockAtSynchLevel();
//             Queue->CurrentCount--;
//         }
//     }
//
//     /* Unlock Database and return */
//     KiReleaseDispatcherLockFromSynchLevel();
//     KiExitDispatcher(Thread->WaitIrql);
//     return QueueEntry;
// }
//
// /*
//  * @implemented
//  */
// PLIST_ENTRY
// NTAPI
// KeRundownQueue(IN PKQUEUE Queue)
// {
//     PLIST_ENTRY FirstEntry, NextEntry;
//     PKTHREAD Thread;
//     KIRQL OldIrql;
//     ASSERT_QUEUE(Queue);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//     ASSERT(IsListEmpty(&Queue->Header.WaitListHead));
//
//     /* Get the Dispatcher Lock */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if the list is empty */
//     FirstEntry = Queue->EntryListHead.Flink;
//     if (FirstEntry == &Queue->EntryListHead)
//     {
//         /* We won't return anything */
//         FirstEntry = NULL;
//     }
//     else
//     {
//         /* Remove this entry */
//         RemoveEntryList(&Queue->EntryListHead);
//     }
//
//     /* Loop the list */
//     while (!IsListEmpty(&Queue->ThreadListHead))
//     {
//         /* Get the next entry */
//         NextEntry = Queue->ThreadListHead.Flink;
//
//         /* Get the associated thread */
//         Thread = CONTAINING_RECORD(NextEntry, KTHREAD, QueueListEntry);
//
//         /* Clear its queue */
//         Thread->Queue = NULL;
//
//         /* Remove this entry */
//         RemoveEntryList(NextEntry);
//     }
//
//     /* Release the dispatcher lock */
//     KiReleaseDispatcherLockFromSynchLevel();
//
//     /* Exit the dispatcher and return the first entry (if any) */
//     KiExitDispatcher(OldIrql);
//     return FirstEntry;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
