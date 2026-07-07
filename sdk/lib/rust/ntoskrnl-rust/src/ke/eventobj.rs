use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
// ntkrnl_sys::mm::* removed — conflicts with ke_defs::KeGetCurrentThread

use ntkrnl_sys::ddk::{
    _KEVENT, _KEVENT_PAIR, _KTHREAD, _KWAIT_BLOCK,
    PKEVENT, PKEVENT_PAIR, PKTHREAD, PKWAIT_BLOCK,
    KPRIORITY,
    KiAcquireDispatcherLock, KiReleaseDispatcherLock,
    KiWaitTest, KiUnlinkThread, KiReadyThread, KiComputeNewPriority,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn KxUnwaitThread(Object: PVOID, Increment: KPRIORITY);
    fn KxUnwaitThreadForEvent(Event: PKEVENT, Increment: KPRIORITY);
}

const EventNotificationObject: u8 = 0;
const EventSynchronizationObject: u8 = 1;
const EventPairObject: u8 = 0x15;
const NotificationEvent: u32 = 0;
const SynchronizationEvent: u32 = 1;
const WaitAll: u32 = 0;
const AdjustBoost: u8 = 2;
const EVENT_INCREMENT: KPRIORITY = 1;

#[no_mangle]
pub unsafe extern "system" fn KeClearEvent(Event: PKEVENT) {
    let Event = opaque_ptr(Event);
    (*Event).Header.SignalState = 0;
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeEvent(
    Event: PKEVENT,
    Type: u32,
    State: BOOLEAN,
) {
    let Event = opaque_ptr(Event);
    (*Event).Header.__bindgen_anon_1.__bindgen_anon_1.Type = EventNotificationObject + Type as u8;
    (*Event).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<_KEVENT>() / core::mem::size_of::<ULONG>()) as u8;
    (*Event).Header.SignalState = State as LONG;
    InitializeListHead(core::ptr::addr_of_mut!((*Event).Header.WaitListHead));
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeEventPair(EventPair: PKEVENT_PAIR) {
    (*EventPair).Type = EventPairObject as i16;
    (*EventPair).Size = core::mem::size_of::<_KEVENT_PAIR>() as i16;

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*EventPair).LowEvent),
        SynchronizationEvent,
        0u8,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*EventPair).HighEvent),
        SynchronizationEvent,
        0u8,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KePulseEvent(
    Event: PKEVENT,
    Increment: KPRIORITY,
    Wait: BOOLEAN,
) -> LONG {
    let Event = opaque_ptr(Event);
    let OldIrql = KiAcquireDispatcherLock();

    let PreviousState = (*Event).Header.SignalState;

    if PreviousState == 0 && !IsListEmpty(core::ptr::addr_of_mut!((*Event).Header.WaitListHead)) {
        (*Event).Header.SignalState = 1;
        KiWaitTest(core::ptr::addr_of_mut!((*Event).Header) as PVOID, Increment);
    }

    (*Event).Header.SignalState = 0;

    if Wait == 0u8 {
        KiReleaseDispatcherLock(OldIrql);
    } else {
        let Thread = PsGetCurrentThread() as PKTHREAD;
        (*Thread).WaitNext = 1u8;
        *kt_WaitIrql(Thread) = OldIrql;
    }

    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeReadStateEvent(Event: PKEVENT) -> LONG {
    let Event = opaque_ptr(Event);
    (*Event).Header.SignalState
}

#[no_mangle]
pub unsafe extern "system" fn KeResetEvent(Event: PKEVENT) -> LONG {
    let Event = opaque_ptr(Event);
    let OldIrql = KiAcquireDispatcherLock();

    let PreviousState = (*Event).Header.SignalState;

    (*Event).Header.SignalState = 0;

    KiReleaseDispatcherLock(OldIrql);
    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeSetEvent(
    Event: PKEVENT,
    Increment: KPRIORITY,
    Wait: BOOLEAN,
) -> LONG {
    let Event = opaque_ptr(Event);
    if (*Event).Header.__bindgen_anon_1.__bindgen_anon_1.Type == EventNotificationObject
        && (*Event).Header.SignalState == 1
        && Wait == 0u8
    {
        return 1;
    }

    let OldIrql = KiAcquireDispatcherLock();

    let PreviousState = (*Event).Header.SignalState;

    (*Event).Header.SignalState = 1;

    if PreviousState == 0 && !IsListEmpty(core::ptr::addr_of_mut!((*Event).Header.WaitListHead)) {
        if (*Event).Header.__bindgen_anon_1.__bindgen_anon_1.Type == EventNotificationObject {
            KxUnwaitThread(core::ptr::addr_of_mut!((*Event).Header) as PVOID, Increment);
        } else {
            KxUnwaitThreadForEvent(Event, Increment);
        }
    }

    if Wait == 0u8 {
        KiReleaseDispatcherLock(OldIrql);
    } else {
        let Thread = PsGetCurrentThread() as PKTHREAD;
        (*Thread).WaitNext = 1u8;
        *kt_WaitIrql(Thread) = OldIrql;
    }

    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeSetEventBoostPriority(
    Event: PKEVENT,
    WaitingThread: *mut PKTHREAD,
) {
    let Event = opaque_ptr(Event);
    let WaitingThread = opaque_ptr(WaitingThread);
    let Thread = PsGetCurrentThread() as PKTHREAD;

    let OldIrql = KiAcquireDispatcherLock();

    if IsListEmpty(core::ptr::addr_of_mut!((*Event).Header.WaitListHead)) {
        (*Event).Header.SignalState = 1;
        KiReleaseDispatcherLock(OldIrql);
        return;
    }

    // waitdep: this path consumes WaitBlock->Thread straight off the event's
    // wait list and hands it to KiUnlinkThread with no other validation — one
    // of the three unchecked funnels for the arm64 bad-thread-ptr flake (the
    // event here can be a STACK-resident EX_PUSH_LOCK_WAIT_BLOCK.WakeEvent).
    // A violation fires with owner = the EVENT address, which identifies the
    // owning frame — one frame earlier and far more specific than the
    // check_unwait(-1) it would otherwise become.
    super::waitdep::check_waittest(
        Event as usize,
        core::ptr::addr_of_mut!((*Event).Header.WaitListHead) as *mut _,
        true,
    );

    let WaitBlock = ((*Event).Header.WaitListHead.Flink as usize
        - core::mem::offset_of!(_KWAIT_BLOCK, WaitListEntry)) as PKWAIT_BLOCK;

    if (*WaitBlock).WaitType as u32 == WaitAll {
        (*Event).Header.SignalState = 1;
        KxUnwaitThreadForEvent(Event, EVENT_INCREMENT);
    } else {
        let WaitThread = (*WaitBlock).Thread;
        if !WaitingThread.is_null() {
            *WaitingThread = WaitThread;
        }

        (*Thread).Priority = KiComputeNewPriority(Thread, 0);

        KiUnlinkThread(WaitThread, STATUS_SUCCESS as _);

        (*WaitThread).__bindgen_anon_1.__bindgen_anon_1.AdjustIncrement = (*Thread).Priority;
        (*WaitThread).__bindgen_anon_1.__bindgen_anon_1.AdjustReason = AdjustBoost;

        KiReadyThread(WaitThread);
    }

    KiReleaseDispatcherLock(OldIrql);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/eventobj.c
//  * PURPOSE:         Implements the Event Dispatcher Object
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeClearEvent(IN PKEVENT Event)
// {
//     ASSERT_EVENT(Event);
//
//     /* Reset Signal State */
//     Event->Header.SignalState = FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeEvent(OUT PKEVENT Event,
//                   IN EVENT_TYPE Type,
//                   IN BOOLEAN State)
// {
//     /* Initialize the Dispatcher Header */
//     ASSERT((Type == NotificationEvent) || (Type == SynchronizationEvent));
//     Event->Header.Type = EventNotificationObject + Type;
//     //Event->Header.Signalling = FALSE; // fails in kmtest
//     Event->Header.Size = sizeof(KEVENT) / sizeof(ULONG);
//     Event->Header.SignalState = State;
//     InitializeListHead(&(Event->Header.WaitListHead));
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeEventPair(IN PKEVENT_PAIR EventPair)
// {
//     /* Initialize the Event Pair Type and Size */
//     EventPair->Type = EventPairObject;
//     EventPair->Size = sizeof(KEVENT_PAIR);
//
//     /* Initialize the two Events */
//     KeInitializeEvent(&EventPair->LowEvent, SynchronizationEvent, FALSE);
//     KeInitializeEvent(&EventPair->HighEvent, SynchronizationEvent, FALSE);
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KePulseEvent(IN PKEVENT Event,
//              IN KPRIORITY Increment,
//              IN BOOLEAN Wait)
// {
//     KIRQL OldIrql;
//     LONG PreviousState;
//     PKTHREAD Thread;
//     ASSERT_EVENT(Event);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the Dispatcher Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Save the Old State */
//     PreviousState = Event->Header.SignalState;
//
//     /* Check if we are non-signaled and we have stuff in the Wait Queue */
//     if (!PreviousState && !IsListEmpty(&Event->Header.WaitListHead))
//     {
//         /* Set the Event to Signaled */
//         Event->Header.SignalState = 1;
//
//         /* Wake the Event */
//         KiWaitTest(&Event->Header, Increment);
//     }
//
//     /* Unsignal it */
//     Event->Header.SignalState = 0;
//
//     /* Check what wait state was requested */
//     if (Wait == FALSE)
//     {
//         /* Wait not requested, release Dispatcher Database and return */
//         KiReleaseDispatcherLock(OldIrql);
//     }
//     else
//     {
//         /* Return Locked and with a Wait */
//         Thread = KeGetCurrentThread();
//         Thread->WaitNext = TRUE;
//         Thread->WaitIrql = OldIrql;
//     }
//
//     /* Return the previous State */
//     return PreviousState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeReadStateEvent(IN PKEVENT Event)
// {
//     ASSERT_EVENT(Event);
//
//     /* Return the Signal State */
//     return Event->Header.SignalState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeResetEvent(IN PKEVENT Event)
// {
//     KIRQL OldIrql;
//     LONG PreviousState;
//     ASSERT_EVENT(Event);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the Dispatcher Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Save the Previous State */
//     PreviousState = Event->Header.SignalState;
//
//     /* Set it to zero */
//     Event->Header.SignalState = 0;
//
//     /* Release Dispatcher Database and return previous state */
//     KiReleaseDispatcherLock(OldIrql);
//     return PreviousState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeSetEvent(IN PKEVENT Event,
//            IN KPRIORITY Increment,
//            IN BOOLEAN Wait)
// {
//     KIRQL OldIrql;
//     LONG PreviousState;
//     PKTHREAD Thread;
//     ASSERT_EVENT(Event);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /*
//      * Check if this is an signaled notification event without an upcoming wait.
//      * In this case, we can immediately return TRUE, without locking.
//      */
//     if ((Event->Header.Type == EventNotificationObject) &&
//         (Event->Header.SignalState == 1) &&
//         !(Wait))
//     {
//         /* Return the signal state (TRUE/Signalled) */
//         return TRUE;
//     }
//
//     /* Lock the Dispathcer Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Save the Previous State */
//     PreviousState = Event->Header.SignalState;
//
//     /* Set the Event to Signaled */
//     Event->Header.SignalState = 1;
//
//     /* Check if the event just became signaled now, and it has waiters */
//     if (!(PreviousState) && !(IsListEmpty(&Event->Header.WaitListHead)))
//     {
//         /* Check the type of event */
//         if (Event->Header.Type == EventNotificationObject)
//         {
//             /* Unwait the thread */
//             KxUnwaitThread(&Event->Header, Increment);
//         }
//         else
//         {
//             /* Otherwise unwait the thread and unsignal the event */
//             KxUnwaitThreadForEvent(Event, Increment);
//         }
//     }
//
//     /* Check what wait state was requested */
//     if (!Wait)
//     {
//         /* Wait not requested, release Dispatcher Database and return */
//         KiReleaseDispatcherLock(OldIrql);
//     }
//     else
//     {
//         /* Return Locked and with a Wait */
//         Thread = KeGetCurrentThread();
//         Thread->WaitNext = TRUE;
//         Thread->WaitIrql = OldIrql;
//     }
//
//     /* Return the previous State */
//     return PreviousState;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSetEventBoostPriority(IN PKEVENT Event,
//                         IN PKTHREAD *WaitingThread OPTIONAL)
// {
//     KIRQL OldIrql;
//     PKWAIT_BLOCK WaitBlock;
//     PKTHREAD Thread = KeGetCurrentThread(), WaitThread;
//     ASSERT(Event->Header.Type == EventSynchronizationObject);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Acquire Dispatcher Database Lock */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Check if the list is empty */
//     if (IsListEmpty(&Event->Header.WaitListHead))
//     {
//         /* Set the Event to Signaled */
//         Event->Header.SignalState = 1;
//
//         /* Return */
//         KiReleaseDispatcherLock(OldIrql);
//         return;
//     }
//
//     /* Get the Wait Block */
//     WaitBlock = CONTAINING_RECORD(Event->Header.WaitListHead.Flink,
//                                   KWAIT_BLOCK,
//                                   WaitListEntry);
//
//     /* Check if this is a WaitAll */
//     if (WaitBlock->WaitType == WaitAll)
//     {
//         /* Set the Event to Signaled */
//         Event->Header.SignalState = 1;
//
//         /* Unwait the thread and unsignal the event */
//         KxUnwaitThreadForEvent(Event, EVENT_INCREMENT);
//     }
//     else
//     {
//         /* Return waiting thread to caller */
//         WaitThread = WaitBlock->Thread;
//         if (WaitingThread) *WaitingThread = WaitThread;
//
//         /* Calculate new priority */
//         Thread->Priority = KiComputeNewPriority(Thread, 0);
//
//         /* Unlink the waiting thread */
//         KiUnlinkThread(WaitThread, STATUS_SUCCESS);
//
//         /* Request priority boosting */
//         WaitThread->AdjustIncrement = Thread->Priority;
//         WaitThread->AdjustReason = AdjustBoost;
//
//         /* Ready the thread */
//         KiReadyThread(WaitThread);
//     }
//
//     /* Release the Dispatcher Database Lock */
//     KiReleaseDispatcherLock(OldIrql);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
