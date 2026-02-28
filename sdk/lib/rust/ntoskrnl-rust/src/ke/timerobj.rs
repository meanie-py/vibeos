use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    KTIMER, KTIMER_TABLE_ENTRY, PKTIMER, PKDPC, PKEVENT,
    PKSPIN_LOCK_QUEUE, LARGE_INTEGER,
    HalRequestSoftwareInterrupt,
    KeInsertQueueDpc,
    KiAcquireTimerLock, KiReleaseTimerLock,
    KiAcquireDispatcherLock, KiReleaseDispatcherLock,
    KiAcquireDispatcherLockAtSynchLevel, KiReleaseDispatcherLockFromSynchLevel,
    KiComputeDueTime, KiComputeTimerTableIndex,
    KiRemoveEntryTimer, KiExitDispatcher,
};

extern "C" {
    fn KeQuerySystemTime(current_time: *mut LARGE_INTEGER);
}
extern "C" {
    fn KxRemoveTreeTimer(Timer: PKTIMER);
    fn KxInsertTimer(Timer: PKTIMER, Hand: ULONG);
    fn KxUnwaitThread(object: PVOID, increment: KPRIORITY);
    fn KxUnwaitThreadForEvent(event: PKEVENT, increment: KPRIORITY);
}

// KiInsertTimerTable is defined locally for all architectures below

const TIMER_TABLE_SIZE: usize = 256;
const TimerNotificationObject: u8 = 8;
const NotificationTimer: u32 = 0;
const SynchronizationTimer: u32 = 1;
const IO_NO_INCREMENT: KPRIORITY = 0;

#[no_mangle]
pub static mut KiTimerTableListHead: [KTIMER_TABLE_ENTRY; TIMER_TABLE_SIZE] = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KiTimeIncrementReciprocal: LARGE_INTEGER = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KiTimeIncrementShiftCount: UCHAR = 0;
#[no_mangle]
pub static mut KiEnableTimerWatchdog: BOOLEAN = 0u8;

#[no_mangle]
pub unsafe extern "system" fn KiInsertTreeTimer(
    Timer: PKTIMER,
    Interval: LARGE_INTEGER,
) -> BOOLEAN {
    let mut Inserted: BOOLEAN = 0u8;
    let mut Hand: ULONG = 0;
    let mut LockQueue: PKSPIN_LOCK_QUEUE;

    if KiComputeDueTime(Timer, Interval, &mut Hand) != 0 {
        LockQueue = KiAcquireTimerLock(Hand);

        if KiInsertTimerTable(Timer, Hand) != 0 {
            KiRemoveEntryTimer(Timer);
            (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 0u8;
        } else {
            Inserted = 1u8;
        }

        KiReleaseTimerLock(LockQueue);
    }

    Inserted
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiInsertTimerTable(
    Timer: PKTIMER,
    Hand: ULONG,
) -> BOOLEAN {
    KiInsertTimerTableImpl(Timer, Hand)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiInsertTimerTable(
    Timer: PKTIMER,
    Hand: ULONG,
) -> BOOLEAN {
    KiInsertTimerTableImpl(Timer, Hand)
}

#[inline(always)]
unsafe fn KiInsertTimerTableImpl(
    Timer: PKTIMER,
    Hand: ULONG,
) -> BOOLEAN {
    let InterruptTime: ULONGLONG;
    let DueTime: ULONGLONG = (*Timer).DueTime.QuadPart as ULONGLONG;
    let mut Expired: BOOLEAN = 0u8;
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut CurrentTimer: PKTIMER;

    if (*Timer).Period == 0 {
        (*Timer).Header.SignalState = 0;
    }

    ListHead = core::ptr::addr_of_mut!(KiTimerTableListHead[Hand as usize].Entry) as PLIST_ENTRY;
    NextEntry = (*ListHead).Blink;
    while NextEntry != ListHead {
        CurrentTimer = crate::CONTAINING_RECORD!(NextEntry, KTIMER, TimerListEntry);

        if DueTime >= (*CurrentTimer).DueTime.QuadPart as ULONGLONG {
            break;
        }

        NextEntry = (*NextEntry).Blink;
    }

    InsertHeadList(NextEntry, core::ptr::addr_of_mut!((*Timer).TimerListEntry) as PLIST_ENTRY);

    if NextEntry == ListHead {
        super::ke_defs::KiTimerTableListHead[Hand as usize].Time.QuadPart = DueTime;

        InterruptTime = KeQueryInterruptTime();
        if DueTime <= InterruptTime {
            Expired = 1u8;
        }
    }

    Expired
}

#[no_mangle]
pub unsafe extern "system" fn KiSignalTimer(Timer: PKTIMER) -> BOOLEAN {
    let mut RequestInterrupt: BOOLEAN = 0u8;
    let Dpc: PKDPC = (*Timer).Dpc;
    let Period: ULONG = (*Timer).Period;
    let mut Interval: LARGE_INTEGER = core::mem::zeroed();
    let mut SystemTime: LARGE_INTEGER = core::mem::zeroed();

    (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 0u8;
    (*Timer).Header.SignalState = 1;

    if !IsListEmpty(core::ptr::addr_of_mut!((*Timer).Header.WaitListHead) as PLIST_ENTRY) {
        if (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.Type == TimerNotificationObject {
            KxUnwaitThread(core::ptr::addr_of_mut!((*Timer).Header) as PVOID, IO_NO_INCREMENT);
        } else {
            KxUnwaitThreadForEvent(Timer as PKEVENT, IO_NO_INCREMENT);
        }
    }

    if Period != 0 {
        Interval.QuadPart = ((Period as i64) * (-10000i64));
        while KiInsertTreeTimer(Timer, Interval) == 0 {}
    }

    if !Dpc.is_null() {
        KeQuerySystemTime(&mut SystemTime);
        KeInsertQueueDpc(
            Dpc,
            SystemTime.__bindgen_anon_1.LowPart as PVOID,
            SystemTime.__bindgen_anon_1.HighPart as PVOID,
        );
        RequestInterrupt = 1u8;
    }

    RequestInterrupt
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiCompleteTimer(
    Timer: PKTIMER,
    LockQueue: PKSPIN_LOCK_QUEUE,
) {
    KiCompleteTimerImpl(Timer, LockQueue)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiCompleteTimer(
    Timer: PKTIMER,
    LockQueue: PKSPIN_LOCK_QUEUE,
) {
    KiCompleteTimerImpl(Timer, LockQueue)
}

#[inline(always)]
unsafe fn KiCompleteTimerImpl(
    Timer: PKTIMER,
    LockQueue: PKSPIN_LOCK_QUEUE,
) {
    let mut ListHead: LIST_ENTRY = core::mem::zeroed();
    let mut RequestInterrupt: BOOLEAN = 0u8;

    KiRemoveEntryTimer(Timer);

    ListHead.Flink = core::ptr::addr_of_mut!((*Timer).TimerListEntry) as PLIST_ENTRY;
    ListHead.Blink = core::ptr::addr_of_mut!((*Timer).TimerListEntry) as PLIST_ENTRY;
    (*Timer).TimerListEntry.Flink = &mut ListHead as *mut LIST_ENTRY as PLIST_ENTRY;
    (*Timer).TimerListEntry.Blink = &mut ListHead as *mut LIST_ENTRY as PLIST_ENTRY;

    KiReleaseTimerLock(LockQueue);

    KiAcquireDispatcherLockAtSynchLevel();

    if !IsListEmpty(&mut ListHead as *mut LIST_ENTRY as PLIST_ENTRY) {
        RequestInterrupt = KiSignalTimer(Timer);
    }

    KiReleaseDispatcherLockFromSynchLevel();

    if RequestInterrupt != 0 {
        HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn KeCancelTimer(Timer: PKTIMER) -> BOOLEAN {
    let OldIrql: KIRQL;
    let Inserted: BOOLEAN;

    OldIrql = KiAcquireDispatcherLock();

    Inserted = (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted;
    if Inserted != 0 {
        KxRemoveTreeTimer(Timer);
    }

    KiReleaseDispatcherLock(OldIrql);

    Inserted
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeTimer(Timer: PKTIMER) {
    KeInitializeTimerEx(Timer, NotificationTimer);
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeTimerEx(Timer: PKTIMER, Type: ULONG) {
    (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.Type = TimerNotificationObject + Type as u8;
    (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Hand = (core::mem::size_of::<KTIMER>() / core::mem::size_of::<ULONG>()) as u8;
    (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted = 0u8;
    (*Timer).Header.SignalState = 0;
    InitializeListHead(core::ptr::addr_of_mut!((*Timer).Header.WaitListHead) as PLIST_ENTRY);

    (*Timer).DueTime.QuadPart = 0;
    (*Timer).Period = 0;
}

#[no_mangle]
pub unsafe extern "system" fn KeReadStateTimer(Timer: PKTIMER) -> BOOLEAN {
    (*Timer).Header.SignalState as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn KeSetTimer(
    Timer: PKTIMER,
    DueTime: LARGE_INTEGER,
    Dpc: PKDPC,
) -> BOOLEAN {
    KeSetTimerEx(Timer, DueTime, 0, Dpc)
}

#[no_mangle]
pub unsafe extern "system" fn KeSetTimerEx(
    Timer: PKTIMER,
    DueTime: LARGE_INTEGER,
    Period: LONG,
    Dpc: PKDPC,
) -> BOOLEAN {
    let OldIrql: KIRQL;
    let Inserted: BOOLEAN;
    let mut Hand: ULONG = 0;
    let mut RequestInterrupt: BOOLEAN = 0u8;

    OldIrql = KiAcquireDispatcherLock();

    Inserted = (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_3.Inserted;
    if Inserted != 0 {
        KxRemoveTreeTimer(Timer);
    }

    (*Timer).Dpc = Dpc;
    (*Timer).Period = Period as ULONG;
    if KiComputeDueTime(Timer, DueTime, &mut Hand) == 0 {
        RequestInterrupt = KiSignalTimer(Timer);

        KiReleaseDispatcherLockFromSynchLevel();

        if RequestInterrupt != 0 {
            HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
        }
    } else {
        (*Timer).Header.SignalState = 0;
        KxInsertTimer(Timer, Hand);
    }

    KiExitDispatcher(OldIrql);

    Inserted
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/timerobj.c
//  * PURPOSE:         Handle Kernel Timers (Kernel-part of Executive Timers)
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// KTIMER_TABLE_ENTRY KiTimerTableListHead[TIMER_TABLE_SIZE];
// LARGE_INTEGER KiTimeIncrementReciprocal;
// UCHAR KiTimeIncrementShiftCount;
// BOOLEAN KiEnableTimerWatchdog = FALSE;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// BOOLEAN
// FASTCALL
// KiInsertTreeTimer(IN PKTIMER Timer,
//                   IN LARGE_INTEGER Interval)
// {
//     BOOLEAN Inserted = FALSE;
//     ULONG Hand = 0;
//     PKSPIN_LOCK_QUEUE LockQueue;
//     DPRINT("KiInsertTreeTimer(): Timer %p, Interval: %I64d\n", Timer, Interval.QuadPart);
//
//     /* Setup the timer's due time */
//     if (KiComputeDueTime(Timer, Interval, &Hand))
//     {
//         /* Acquire the lock */
//         LockQueue = KiAcquireTimerLock(Hand);
//
//         /* Insert the timer */
//         if (KiInsertTimerTable(Timer, Hand))
//         {
//             /* It was already there, remove it */
//             KiRemoveEntryTimer(Timer);
//             Timer->Header.Inserted = FALSE;
//         }
//         else
//         {
//             /* Otherwise, we're now inserted */
//             Inserted = TRUE;
//         }
//
//         /* Release the lock */
//         KiReleaseTimerLock(LockQueue);
//     }
//
//     /* Release the lock and return insert status */
//     return Inserted;
// }
//
// BOOLEAN
// FASTCALL
// KiInsertTimerTable(IN PKTIMER Timer,
//                    IN ULONG Hand)
// {
//     ULONGLONG InterruptTime;
//     ULONGLONG DueTime = Timer->DueTime.QuadPart;
//     BOOLEAN Expired = FALSE;
//     PLIST_ENTRY ListHead, NextEntry;
//     PKTIMER CurrentTimer;
//     DPRINT("KiInsertTimerTable(): Timer %p, Hand: %lu\n", Timer, Hand);
//
//     /* Check if the period is zero */
//     if (!Timer->Period) Timer->Header.SignalState = FALSE;
//
//     /* Sanity check */
//     ASSERT(Hand == KiComputeTimerTableIndex(DueTime));
//
//     /* Loop the timer list backwards */
//     ListHead = &KiTimerTableListHead[Hand].Entry;
//     NextEntry = ListHead->Blink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the timer */
//         CurrentTimer = CONTAINING_RECORD(NextEntry, KTIMER, TimerListEntry);
//
//         /* Now check if we can fit it before */
//         if ((ULONGLONG)DueTime >= CurrentTimer->DueTime.QuadPart) break;
//
//         /* Keep looping */
//         NextEntry = NextEntry->Blink;
//     }
//
//     /* Looped all the list, insert it here and get the interrupt time again */
//     InsertHeadList(NextEntry, &Timer->TimerListEntry);
//
//     /* Check if we didn't find it in the list */
//     if (NextEntry == ListHead)
//     {
//         /* Set the time */
//         KiTimerTableListHead[Hand].Time.QuadPart = DueTime;
//
//         /* Make sure it hasn't expired already */
//         InterruptTime = KeQueryInterruptTime();
//         if (DueTime <= InterruptTime) Expired = TRUE;
//     }
//
//     /* Return expired state */
//     return Expired;
// }
//
// BOOLEAN
// FASTCALL
// KiSignalTimer(IN PKTIMER Timer)
// {
//     BOOLEAN RequestInterrupt = FALSE;
//     PKDPC Dpc = Timer->Dpc;
//     ULONG Period = Timer->Period;
//     LARGE_INTEGER Interval, SystemTime;
//     DPRINT("KiSignalTimer(): Timer %p\n", Timer);
//
//     /* Set default values */
//     Timer->Header.Inserted = FALSE;
//     Timer->Header.SignalState = TRUE;
//
//     /* Check if the timer has waiters */
//     if (!IsListEmpty(&Timer->Header.WaitListHead))
//     {
//         /* Check the type of event */
//         if (Timer->Header.Type == TimerNotificationObject)
//         {
//             /* Unwait the thread */
//             KxUnwaitThread(&Timer->Header, IO_NO_INCREMENT);
//         }
//         else
//         {
//             /* Otherwise unwait the thread and signal the timer */
//             KxUnwaitThreadForEvent((PKEVENT)Timer, IO_NO_INCREMENT);
//         }
//     }
//
//     /* Check if we have a period */
//     if (Period)
//     {
//         /* Calculate the interval and insert the timer */
//         Interval.QuadPart = Int32x32To64(Period, -10000);
//         while (!KiInsertTreeTimer(Timer, Interval));
//     }
//
//     /* Check if we have a DPC */
//     if (Dpc)
//     {
//         /* Insert it in the queue */
//         KeQuerySystemTime(&SystemTime);
//         KeInsertQueueDpc(Dpc,
//                          ULongToPtr(SystemTime.LowPart),
//                          ULongToPtr(SystemTime.HighPart));
//         RequestInterrupt = TRUE;
//     }
//
//     /* Return whether we need to request a DPC interrupt or not */
//     return RequestInterrupt;
// }
//
// VOID
// FASTCALL
// KiCompleteTimer(IN PKTIMER Timer,
//                 IN PKSPIN_LOCK_QUEUE LockQueue)
// {
//     LIST_ENTRY ListHead;
//     BOOLEAN RequestInterrupt = FALSE;
//     DPRINT("KiCompleteTimer(): Timer %p, LockQueue: %p\n", Timer, LockQueue);
//
//     /* Remove it from the timer list */
//     KiRemoveEntryTimer(Timer);
//
//     /* Link the timer list to our stack */
//     ListHead.Flink = &Timer->TimerListEntry;
//     ListHead.Blink = &Timer->TimerListEntry;
//     Timer->TimerListEntry.Flink = &ListHead;
//     Timer->TimerListEntry.Blink = &ListHead;
//
//     /* Release the timer lock */
//     KiReleaseTimerLock(LockQueue);
//
//     /* Acquire dispatcher lock */
//     KiAcquireDispatcherLockAtSynchLevel();
//
//     /* Signal the timer if it's still on our list */
//     if (!IsListEmpty(&ListHead)) RequestInterrupt = KiSignalTimer(Timer);
//
//     /* Release the dispatcher lock */
//     KiReleaseDispatcherLockFromSynchLevel();
//
//     /* Request a DPC if needed */
//     if (RequestInterrupt) HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeCancelTimer(IN OUT PKTIMER Timer)
// {
//     KIRQL OldIrql;
//     BOOLEAN Inserted;
//     ASSERT_TIMER(Timer);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//     DPRINT("KeCancelTimer(): Timer %p\n", Timer);
//
//     /* Lock the Database and Raise IRQL */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if it's inserted, and remove it if it is */
//     Inserted = Timer->Header.Inserted;
//     if (Inserted) KxRemoveTreeTimer(Timer);
//
//     /* Release Dispatcher Lock */
//     KiReleaseDispatcherLock(OldIrql);
//
//     /* Return the old state */
//     return Inserted;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeTimer(OUT PKTIMER Timer)
// {
//     /* Call the New Function */
//     KeInitializeTimerEx(Timer, NotificationTimer);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeTimerEx(OUT PKTIMER Timer,
//                     IN TIMER_TYPE Type)
// {
//     DPRINT("KeInitializeTimerEx(): Timer %p, Type %s\n",
//             Timer, (Type == NotificationTimer) ?
//            "NotificationTimer" : "SynchronizationTimer");
//
//     /* Initialize the Dispatch Header */
//     ASSERT((Type == NotificationTimer) || (Type == SynchronizationTimer));
//     Timer->Header.Type = TimerNotificationObject + Type;
//     //Timer->Header.TimerControlFlags = 0; // win does not init this field
//     Timer->Header.Hand = sizeof(KTIMER) / sizeof(ULONG);
//     Timer->Header.Inserted = 0; // win7: Timer->Header.TimerMiscFlags = 0;
//     Timer->Header.SignalState = 0;
//     InitializeListHead(&(Timer->Header.WaitListHead));
//
//     /* Initialize the Other data */
//     Timer->DueTime.QuadPart = 0;
//     Timer->Period = 0;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeReadStateTimer(IN PKTIMER Timer)
// {
//     /* Return the Signal State */
//     ASSERT_TIMER(Timer);
//     return (BOOLEAN)Timer->Header.SignalState;
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeSetTimer(IN OUT PKTIMER Timer,
//            IN LARGE_INTEGER DueTime,
//            IN PKDPC Dpc OPTIONAL)
// {
//     /* Call the newer function and supply a period of 0 */
//     return KeSetTimerEx(Timer, DueTime, 0, Dpc);
// }
//
// /*
//  * @implemented
//  */
// BOOLEAN
// NTAPI
// KeSetTimerEx(IN OUT PKTIMER Timer,
//              IN LARGE_INTEGER DueTime,
//              IN LONG Period,
//              IN PKDPC Dpc OPTIONAL)
// {
//     KIRQL OldIrql;
//     BOOLEAN Inserted;
//     ULONG Hand = 0;
//     BOOLEAN RequestInterrupt = FALSE;
//     ASSERT_TIMER(Timer);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//     DPRINT("KeSetTimerEx(): Timer %p, DueTime %I64d, Period %d, Dpc %p\n",
//            Timer, DueTime.QuadPart, Period, Dpc);
//
//     /* Lock the Database and Raise IRQL */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if it's inserted, and remove it if it is */
//     Inserted = Timer->Header.Inserted;
//     if (Inserted) KxRemoveTreeTimer(Timer);
//
//     /* Set Default Timer Data */
//     Timer->Dpc = Dpc;
//     Timer->Period = Period;
//     if (!KiComputeDueTime(Timer, DueTime, &Hand))
//     {
//         /* Signal the timer */
//         RequestInterrupt = KiSignalTimer(Timer);
//
//         /* Release the dispatcher lock */
//         KiReleaseDispatcherLockFromSynchLevel();
//
//         /* Check if we need to do an interrupt */
//         if (RequestInterrupt) HalRequestSoftwareInterrupt(DISPATCH_LEVEL);
//     }
//     else
//     {
//         /* Insert the timer */
//         Timer->Header.SignalState = FALSE;
//         KxInsertTimer(Timer, Hand);
//     }
//
//     /* Exit the dispatcher */
//     KiExitDispatcher(OldIrql);
//
//     /* Return old state */
//     return Inserted;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
