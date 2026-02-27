use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity

use ntkrnl_sys::ddk::{
    KPRIORITY, KSEMAPHORE, PKSEMAPHORE, PKTHREAD,
    KiAcquireDispatcherLock, KiReleaseDispatcherLock, KiWaitTest,
};

#[no_mangle]
pub unsafe extern "system" fn KeInitializeSemaphore(
    Semaphore: PKSEMAPHORE,
    Count: LONG,
    Limit: LONG,
) {
    (*Semaphore).Header.__bindgen_anon_1.__bindgen_anon_1.Type = SemaphoreObject as u8;
    (*Semaphore).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<KSEMAPHORE>() / core::mem::size_of::<ULONG>()) as u8;
    (*Semaphore).Header.SignalState = Count;
    InitializeListHead(core::ptr::addr_of_mut!((*Semaphore).Header.WaitListHead) as PLIST_ENTRY);

    (*Semaphore).Limit = Limit;
}

#[no_mangle]
pub unsafe extern "system" fn KeReadStateSemaphore(Semaphore: PKSEMAPHORE) -> LONG {
    (*Semaphore).Header.SignalState
}

#[no_mangle]
pub unsafe extern "system" fn KeReleaseSemaphore(
    Semaphore: PKSEMAPHORE,
    Increment: KPRIORITY,
    Adjustment: LONG,
    Wait: BOOLEAN,
) -> LONG {
    let OldIrql: KIRQL;
    let CurrentThread: PKTHREAD;

    OldIrql = KiAcquireDispatcherLock();

    let InitialState = (*Semaphore).Header.SignalState;
    let State = InitialState + Adjustment;

    if (*Semaphore).Limit < State || InitialState > State {
        KiReleaseDispatcherLock(OldIrql);
        ExRaiseStatus(STATUS_SEMAPHORE_LIMIT_EXCEEDED);
    }

    (*Semaphore).Header.SignalState = State;

    if InitialState == 0 && !IsListEmpty(core::ptr::addr_of_mut!((*Semaphore).Header.WaitListHead) as PLIST_ENTRY) {
        KiWaitTest(core::ptr::addr_of_mut!((*Semaphore).Header) as *mut _, Increment);
    }

    if Wait == 0u8 {
        KiReleaseDispatcherLock(OldIrql);
    } else {
        CurrentThread = PsGetCurrentThread();
        (*CurrentThread).WaitNext = 1u8;
        *kt_WaitIrql(CurrentThread) = OldIrql;
    }

    InitialState
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/semphobj.c
//  * PURPOSE:         Implements the Semaphore Dispatcher Object
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
// KeInitializeSemaphore(IN PKSEMAPHORE Semaphore,
//                       IN LONG Count,
//                       IN LONG Limit)
// {
//     /* Simply Initialize the Header */
//     Semaphore->Header.Type = SemaphoreObject;
//     Semaphore->Header.Size = sizeof(KSEMAPHORE) / sizeof(ULONG);
//     Semaphore->Header.SignalState = Count;
//     InitializeListHead(&(Semaphore->Header.WaitListHead));
//
//     /* Set the Limit */
//     Semaphore->Limit = Limit;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeReadStateSemaphore(IN PKSEMAPHORE Semaphore)
// {
//     ASSERT_SEMAPHORE(Semaphore);
//
//     /* Just return the Signal State */
//     return Semaphore->Header.SignalState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeReleaseSemaphore(IN PKSEMAPHORE Semaphore,
//                    IN KPRIORITY Increment,
//                    IN LONG Adjustment,
//                    IN BOOLEAN Wait)
// {
//     LONG InitialState, State;
//     KIRQL OldIrql;
//     PKTHREAD CurrentThread;
//     ASSERT_SEMAPHORE(Semaphore);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the Dispatcher Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Save the Old State and get new one */
//     InitialState = Semaphore->Header.SignalState;
//     State = InitialState + Adjustment;
//
//     /* Check if the Limit was exceeded */
//     if ((Semaphore->Limit < State) || (InitialState > State))
//     {
//         /* Raise an error if it was exceeded */
//         KiReleaseDispatcherLock(OldIrql);
//         ExRaiseStatus(STATUS_SEMAPHORE_LIMIT_EXCEEDED);
//     }
//
//     /* Now set the new state */
//     Semaphore->Header.SignalState = State;
//
//     /* Check if we should wake it */
//     if (!(InitialState) && !(IsListEmpty(&Semaphore->Header.WaitListHead)))
//     {
//         /* Wake the Semaphore */
//         KiWaitTest(&Semaphore->Header, Increment);
//     }
//
//     /* Check if the caller wants to wait after this release */
//     if (Wait == FALSE)
//     {
//         /* Release the Lock */
//         KiReleaseDispatcherLock(OldIrql);
//     }
//     else
//     {
//         /* Set a wait */
//         CurrentThread = KeGetCurrentThread();
//         CurrentThread->WaitNext = TRUE;
//         CurrentThread->WaitIrql = OldIrql;
//     }
//
//     /* Return the previous state */
//     return InitialState;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
