use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _KMUTANT,
    PKMUTANT, PKMUTEX, PKTHREAD,
    KPRIORITY,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const MutantObject: u8 = 2;
const MUTANT_INCREMENT: KPRIORITY = 1;

#[no_mangle]
pub unsafe extern "system" fn KeInitializeMutant(
    Mutant: PKMUTANT,
    InitialOwner: BOOLEAN,
) {
    let Mutant = opaque_ptr(Mutant);
    let mut CurrentThread: PKTHREAD;
    let mut OldIrql: KIRQL;

    if InitialOwner != 0 {
        CurrentThread = PsGetCurrentThread();
        (*Mutant).OwnerThread = CurrentThread;

        OldIrql = KiAcquireDispatcherLock();

        InsertTailList(
            core::ptr::addr_of_mut!((*CurrentThread).MutantListHead),
            core::ptr::addr_of_mut!((*Mutant).MutantListEntry),
        );

        KiReleaseDispatcherLock(OldIrql);
    } else {
        (*Mutant).OwnerThread = core::ptr::null_mut();
    }

    (*Mutant).Header.__bindgen_anon_1.__bindgen_anon_1.Type = MutantObject;
    (*Mutant).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<_KMUTANT>() / core::mem::size_of::<ULONG>()) as u8;
    (*Mutant).Header.SignalState = if InitialOwner != 0 { 0 } else { 1 };
    InitializeListHead(core::ptr::addr_of_mut!((*Mutant).Header.WaitListHead));

    (*Mutant).Abandoned = 0u8;
    (*Mutant).ApcDisable = 0;
}

#[no_mangle]
pub unsafe extern "system" fn KeInitializeMutex(
    Mutex: PKMUTEX,
    Level: ULONG,
) {
    (*Mutex).Header.__bindgen_anon_1.__bindgen_anon_1.Type = MutantObject;
    (*Mutex).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Size = (core::mem::size_of::<_KMUTANT>() / core::mem::size_of::<ULONG>()) as UCHAR;
    (*Mutex).Header.SignalState = 1;
    InitializeListHead(core::ptr::addr_of_mut!((*Mutex).Header.WaitListHead));

    (*Mutex).OwnerThread = core::ptr::null_mut();
    (*Mutex).Abandoned = 0u8;
    (*Mutex).ApcDisable = 1;
}

#[no_mangle]
pub unsafe extern "system" fn KeReadStateMutant(Mutant: PKMUTANT) -> LONG {
    let Mutant = opaque_ptr(Mutant);
    (*Mutant).Header.SignalState
}

#[no_mangle]
pub unsafe extern "system" fn KeReleaseMutant(
    Mutant: PKMUTANT,
    Increment: KPRIORITY,
    Abandon: BOOLEAN,
    Wait: BOOLEAN,
) -> LONG {
    let Mutant = opaque_ptr(Mutant);
    let mut OldIrql: KIRQL;
    let PreviousState: LONG;
    let CurrentThread: PKTHREAD = PsGetCurrentThread();
    let mut EnableApc: BOOLEAN = 0u8;

    OldIrql = KiAcquireDispatcherLock();

    PreviousState = (*Mutant).Header.SignalState;

    if Abandon == 0 {
        if (*Mutant).OwnerThread != CurrentThread {
            KiReleaseDispatcherLock(OldIrql);

            ExRaiseStatus(if (*Mutant).Abandoned != 0 {
                STATUS_ABANDONED
            } else {
                STATUS_MUTANT_NOT_OWNED
            });
        }

        (*Mutant).Header.SignalState += 1;
    } else {
        (*Mutant).Header.SignalState = 1;
        (*Mutant).Abandoned = 1u8;
    }

    if (*Mutant).Header.SignalState == 1 {
        if PreviousState <= 0 {
            RemoveEntryList(core::ptr::addr_of_mut!((*Mutant).MutantListEntry));

            EnableApc = (*Mutant).ApcDisable;
        }

        (*Mutant).OwnerThread = core::ptr::null_mut();

        if !IsListEmpty(core::ptr::addr_of_mut!((*Mutant).Header.WaitListHead)) {
            KiWaitTest(core::ptr::addr_of_mut!((*Mutant).Header) as *mut core::ffi::c_void, Increment);
        }
    }

    if Wait == 0 {
        KiReleaseDispatcherLock(OldIrql);
    } else {
        (*CurrentThread).WaitNext = 1u8;
        *kt_WaitIrql(CurrentThread) = OldIrql;
    }

    if EnableApc != 0 {
        KeLeaveCriticalRegion();
    }

    PreviousState
}

#[no_mangle]
pub unsafe extern "system" fn KeReleaseMutex(
    Mutex: PKMUTEX,
    Wait: BOOLEAN,
) -> LONG {
    KeReleaseMutant(Mutex as PKMUTANT, MUTANT_INCREMENT, 0u8, Wait)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/mutex.c
//  * PURPOSE:         Implements the Mutant Dispatcher Object
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
// KeInitializeMutant(IN PKMUTANT Mutant,
//                    IN BOOLEAN InitialOwner)
// {
//     PKTHREAD CurrentThread;
//     KIRQL OldIrql;
//
//     /* Check if we have an initial owner */
//     if (InitialOwner)
//     {
//         /* We also need to associate a thread */
//         CurrentThread = PsGetCurrentThread();
//         Mutant->OwnerThread = CurrentThread;
//
//         /* We're about to touch the Thread, so lock the Dispatcher */
//         OldIrql = KiAcquireDispatcherLock();
//
//         /* And insert it into its list */
//         InsertTailList(&CurrentThread->MutantListHead,
//                        &Mutant->MutantListEntry);
//
//         /* Release Dispatcher Lock */
//         KiReleaseDispatcherLock(OldIrql);
//     }
//     else
//     {
//         /* In this case, we don't have an owner yet */
//         Mutant->OwnerThread = NULL;
//     }
//
//     /* Now we set up the Dispatcher Header */
//     Mutant->Header.Type = MutantObject;
//     Mutant->Header.Size = sizeof(KMUTANT) / sizeof(ULONG);
//     Mutant->Header.SignalState = InitialOwner ? 0 : 1;
//     InitializeListHead(&(Mutant->Header.WaitListHead));
//
//     /* Initialize the default data */
//     Mutant->Abandoned = FALSE;
//     Mutant->ApcDisable = 0;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeInitializeMutex(IN PKMUTEX Mutex,
//                   IN ULONG Level)
// {
//     /* Set up the Dispatcher Header */
//     Mutex->Header.Type = MutantObject;
//     Mutex->Header.Size = sizeof(KMUTEX) / sizeof(ULONG);
//     Mutex->Header.SignalState = 1;
//     InitializeListHead(&(Mutex->Header.WaitListHead));
//
//     /* Initialize the default data */
//     Mutex->OwnerThread = NULL;
//     Mutex->Abandoned = FALSE;
//     Mutex->ApcDisable = 1;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeReadStateMutant(IN PKMUTANT Mutant)
// {
//     /* Return the Signal State */
//     return Mutant->Header.SignalState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeReleaseMutant(IN PKMUTANT Mutant,
//                 IN KPRIORITY Increment,
//                 IN BOOLEAN Abandon,
//                 IN BOOLEAN Wait)
// {
//     KIRQL OldIrql;
//     LONG PreviousState;
//     PKTHREAD CurrentThread = PsGetCurrentThread();
//     BOOLEAN EnableApc = FALSE;
//     ASSERT_MUTANT(Mutant);
//     ASSERT_IRQL_LESS_OR_EQUAL(DISPATCH_LEVEL);
//
//     /* Lock the Dispatcher Database */
//     OldIrql = KiAcquireDispatcherLock();
//
//     /* Save the Previous State */
//     PreviousState = Mutant->Header.SignalState;
//
//     /* Check if it is to be abandonned */
//     if (Abandon == FALSE)
//     {
//         /* Make sure that the Owner Thread is the current Thread */
//         if (Mutant->OwnerThread != CurrentThread)
//         {
//             /* Release the lock */
//             KiReleaseDispatcherLock(OldIrql);
//
//             /* Raise an exception */
//             ExRaiseStatus(Mutant->Abandoned ? STATUS_ABANDONED :
//                                               STATUS_MUTANT_NOT_OWNED);
//         }
//
//         /* If the thread owns it, then increase the signal state */
//         Mutant->Header.SignalState++;
//     }
//     else
//     {
//         /* It's going to be abandonned */
//         Mutant->Header.SignalState = 1;
//         Mutant->Abandoned = TRUE;
//     }
//
//     /* Check if the signal state is only single */
//     if (Mutant->Header.SignalState == 1)
//     {
//         /* Check if it's below 0 now */
//         if (PreviousState <= 0)
//         {
//             /* Remove the mutant from the list */
//             RemoveEntryList(&Mutant->MutantListEntry);
//
//             /* Save if we need to re-enable APCs */
//             EnableApc = Mutant->ApcDisable;
//         }
//
//         /* Remove the Owning Thread and wake it */
//         Mutant->OwnerThread = NULL;
//
//         /* Check if the Wait List isn't empty */
//         if (!IsListEmpty(&Mutant->Header.WaitListHead))
//         {
//             /* Wake the Mutant */
//             KiWaitTest(&Mutant->Header, Increment);
//         }
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
//         CurrentThread->WaitNext = TRUE;
//         CurrentThread->WaitIrql = OldIrql;
//     }
//
//     /* Check if we need to re-enable APCs */
//     if (EnableApc) KeLeaveCriticalRegion();
//
//     /* Return the previous state */
//     return PreviousState;
// }
//
// /*
//  * @implemented
//  */
// LONG
// NTAPI
// KeReleaseMutex(IN PKMUTEX Mutex,
//                IN BOOLEAN Wait)
// {
//     ASSERT_MUTANT(Mutex);
//
//     /* There's no difference at this level between the two */
//     return KeReleaseMutant(Mutex, MUTANT_INCREMENT, FALSE, Wait);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
