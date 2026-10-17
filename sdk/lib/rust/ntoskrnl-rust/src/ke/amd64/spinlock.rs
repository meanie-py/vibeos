use crate::ke::ke_defs::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{
    PKIRQL, KSPIN_LOCK_QUEUE_NUMBER, LOGICAL,
    KeTryToAcquireSpinLockAtDpcLevel,
};

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[no_mangle]
pub unsafe extern "system" fn KeAcquireSpinLockRaiseToSynch(SpinLock: PKSPIN_LOCK) -> KIRQL {
    let mut OldIrql: KIRQL = 0;
    KeRaiseIrql(SYNCH_LEVEL, &mut OldIrql);
    KxAcquireSpinLock(SpinLock as *mut _);
    OldIrql
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[no_mangle]
pub unsafe extern "system" fn KeAcquireSpinLockRaiseToDpc(SpinLock: PKSPIN_LOCK) -> KIRQL {
    let mut OldIrql: KIRQL = 0;
    KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql);
    KxAcquireSpinLock(SpinLock as *mut _);
    OldIrql
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn KeReleaseSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL) {
    KxReleaseSpinLock(SpinLock as *mut _);
    KeLowerIrql(OldIrql);
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireQueuedSpinLock(LockNumber: KSPIN_LOCK_QUEUE_NUMBER) -> KIRQL {
    let mut OldIrql: KIRQL = 0;
    KeRaiseIrql(DISPATCH_LEVEL, &mut OldIrql);
    let prcb = KeGetCurrentPrcb();
    KxAcquireSpinLock((*prcb).LockQueue[LockNumber as usize].Lock as *mut _);
    OldIrql
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
) -> KIRQL {
    let mut OldIrql: KIRQL = 0;
    KeRaiseIrql(SYNCH_LEVEL, &mut OldIrql);
    let prcb = KeGetCurrentPrcb();
    KxAcquireSpinLock((*prcb).LockQueue[LockNumber as usize].Lock as *mut _);
    OldIrql
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireInStackQueuedSpinLock(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).LockQueue.Next = core::ptr::null_mut();
    (*LockHandle).LockQueue.Lock = SpinLock as *mut u64;
    KeRaiseIrql(DISPATCH_LEVEL, &mut (*LockHandle).OldIrql);
    ntkrnl_sys::inline::ke::KxAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).LockQueue.Next = core::ptr::null_mut();
    (*LockHandle).LockQueue.Lock = SpinLock as *mut u64;
    KeRaiseIrql(SYNCH_LEVEL, &mut (*LockHandle).OldIrql);
    ntkrnl_sys::inline::ke::KxAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseQueuedSpinLock(
    LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: KIRQL,
) {
    let prcb = KeGetCurrentPrcb();
    KxReleaseSpinLock((*prcb).LockQueue[LockNumber as usize].Lock as *mut _);
    KeLowerIrql(OldIrql);
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseInStackQueuedSpinLock(LockHandle: PKLOCK_QUEUE_HANDLE) {
    ntkrnl_sys::inline::ke::KxReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
    KeLowerIrql((*LockHandle).OldIrql);
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeTryToAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: PKIRQL,
) -> BOOLEAN {
    KeRaiseIrql(SYNCH_LEVEL, OldIrql);

    #[cfg(feature = "smp")]
    {
        let prcb = KeGetCurrentPrcb();
        KeTryToAcquireSpinLockAtDpcLevel((*prcb).LockQueue[LockNumber as usize].Lock)
    }

    #[cfg(not(feature = "smp"))]
    {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        1u8
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeTryToAcquireQueuedSpinLock(
    LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: PKIRQL,
) -> LOGICAL {
    KeRaiseIrql(DISPATCH_LEVEL, OldIrql);

    #[cfg(feature = "smp")]
    {
        let prcb = KeGetCurrentPrcb();
        KeTryToAcquireSpinLockAtDpcLevel((*prcb).LockQueue[LockNumber as usize].Lock) as LOGICAL
    }

    #[cfg(not(feature = "smp"))]
    {
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        1u32
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS HAL
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/amd64/spinlock.c
//  * PURPOSE:         Spinlock and Queued Spinlock Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #undef KeAcquireSpinLock
// #undef KeReleaseSpinLock
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// KIRQL
// KeAcquireSpinLockRaiseToSynch(PKSPIN_LOCK SpinLock)
// {
//     KIRQL OldIrql;
//
//     /* Raise to sync */
//     KeRaiseIrql(SYNCH_LEVEL, &OldIrql);
//
//     /* Acquire the lock and return */
//     KxAcquireSpinLock(SpinLock as *mut _);
//     return OldIrql;
// }
//
// /*
//  * @implemented
//  */
// KIRQL
// NTAPI
// KeAcquireSpinLockRaiseToDpc(PKSPIN_LOCK SpinLock)
// {
//     KIRQL OldIrql;
//
//     /* Raise to dispatch */
//     KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//     /* Acquire the lock and return */
//     KxAcquireSpinLock(SpinLock as *mut _);
//     return OldIrql;
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeReleaseSpinLock(PKSPIN_LOCK SpinLock,
//                   KIRQL OldIrql)
// {
//     /* Release the lock and lower IRQL back */
//     KxReleaseSpinLock(SpinLock as *mut _);
//     KeLowerIrql(OldIrql);
// }
//
// /*
//  * @implemented
//  */
// KIRQL
// KeAcquireQueuedSpinLock(IN KSPIN_LOCK_QUEUE_NUMBER LockNumber)
// {
//     KIRQL OldIrql;
//
//     /* Raise to dispatch */
//     KeRaiseIrql(DISPATCH_LEVEL, &OldIrql);
//
//     /* Acquire the lock */
//     KxAcquireSpinLock(KeGetCurrentPrcb()->LockQueue[LockNumber].Lock); // HACK
//     return OldIrql;
// }
//
// /*
//  * @implemented
//  */
// KIRQL
// KeAcquireQueuedSpinLockRaiseToSynch(IN KSPIN_LOCK_QUEUE_NUMBER LockNumber)
// {
//     KIRQL OldIrql;
//
//     /* Raise to synch */
//     KeRaiseIrql(SYNCH_LEVEL, &OldIrql);
//
//     /* Acquire the lock */
//     KxAcquireSpinLock(KeGetCurrentPrcb()->LockQueue[LockNumber].Lock); // HACK
//     return OldIrql;
// }
//
// /*
//  * @implemented
//  */
// VOID
// KeAcquireInStackQueuedSpinLock(IN PKSPIN_LOCK SpinLock,
//                                IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     /* Set up the lock */
//     LockHandle->LockQueue.Next = NULL;
//     LockHandle->LockQueue.Lock = SpinLock;
//
//     /* Raise to dispatch */
//     KeRaiseIrql(DISPATCH_LEVEL, &LockHandle->OldIrql);
//
//     /* Acquire the lock */
//     KxAcquireSpinLock(LockHandle->LockQueue.Lock); // HACK
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// KeAcquireInStackQueuedSpinLockRaiseToSynch(IN PKSPIN_LOCK SpinLock,
//                                            IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     /* Set up the lock */
//     LockHandle->LockQueue.Next = NULL;
//     LockHandle->LockQueue.Lock = SpinLock;
//
//     /* Raise to synch */
//     KeRaiseIrql(SYNCH_LEVEL, &LockHandle->OldIrql);
//
//     /* Acquire the lock */
//     KxAcquireSpinLock(LockHandle->LockQueue.Lock); // HACK
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// KeReleaseQueuedSpinLock(IN KSPIN_LOCK_QUEUE_NUMBER LockNumber,
//                         IN KIRQL OldIrql)
// {
//     /* Release the lock */
//     KxReleaseSpinLock(KeGetCurrentPrcb()->LockQueue[LockNumber].Lock); // HACK
//
//     /* Lower IRQL back */
//     KeLowerIrql(OldIrql);
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// KeReleaseInStackQueuedSpinLock(IN PKLOCK_QUEUE_HANDLE LockHandle)
// {
//     /* Simply lower IRQL back */
//     KxReleaseSpinLock(LockHandle->LockQueue.Lock); // HACK
//     KeLowerIrql(LockHandle->OldIrql);
// }
//
//
// /*
//  * @implemented
//  */
// BOOLEAN
// KeTryToAcquireQueuedSpinLockRaiseToSynch(IN KSPIN_LOCK_QUEUE_NUMBER LockNumber,
//                                          IN PKIRQL OldIrql)
// {
//     /* Raise to synch level */
//     KeRaiseIrql(SYNCH_LEVEL, OldIrql);
//
// #ifdef CONFIG_SMP
//     // HACK
//     return KeTryToAcquireSpinLockAtDpcLevel(KeGetCurrentPrcb()->LockQueue[LockNumber].Lock);
// #else
//     /* Add an explicit memory barrier to prevent the compiler from reordering
//        memory accesses across the borders of spinlocks */
//     KeMemoryBarrierWithoutFence();
//
//     /* Always return true on UP Machines */
//     return TRUE;
// #endif
// }
//
// /*
//  * @implemented
//  */
// LOGICAL
// KeTryToAcquireQueuedSpinLock(IN KSPIN_LOCK_QUEUE_NUMBER LockNumber,
//                              OUT PKIRQL OldIrql)
// {
//     /* Raise to dispatch level */
//     KeRaiseIrql(DISPATCH_LEVEL, OldIrql);
//
// #ifdef CONFIG_SMP
//     // HACK
//     return KeTryToAcquireSpinLockAtDpcLevel(KeGetCurrentPrcb()->LockQueue[LockNumber].Lock);
// #else
//
//     /* Add an explicit memory barrier to prevent the compiler from reordering
//        memory accesses across the borders of spinlocks */
//     KeMemoryBarrierWithoutFence();
//
//     /* Always return true on UP Machines */
//     return TRUE;
// #endif
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
