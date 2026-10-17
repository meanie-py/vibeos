use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
// mm::* import removed to avoid KeGetCurrentThread ambiguity
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    PKINTERRUPT,
    PKSPIN_LOCK_QUEUE,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use ntkrnl_sys::inline::ke::{set_spin_lock_word, spin_lock_word};

// FASTCALL on i386 (matching Windows API / bindgen), "system" on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KeAcquireInStackQueuedSpinLock(SpinLock: PKSPIN_LOCK, LockHandle: PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLock(LockHandle: PKLOCK_QUEUE_HANDLE);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KeAcquireInStackQueuedSpinLock(SpinLock: PKSPIN_LOCK, LockHandle: PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLock(LockHandle: PKLOCK_QUEUE_HANDLE);
}

const LQ_WAIT: usize = 1;
const LQ_OWN: usize = 2;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireQueuedSpinLockAtDpcLevel(LockHandle: PKSPIN_LOCK_QUEUE) {
    KeAcquireQueuedSpinLockAtDpcLevelInner(LockHandle)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireQueuedSpinLockAtDpcLevel(LockHandle: PKSPIN_LOCK_QUEUE) {
    KeAcquireQueuedSpinLockAtDpcLevelInner(LockHandle)
}

#[inline(always)]
unsafe fn KeAcquireQueuedSpinLockAtDpcLevelInner(LockHandle: PKSPIN_LOCK_QUEUE) {
    #[cfg(any(debug_assertions, feature = "smp"))]
    {
        if KeGetCurrentIrql() < DISPATCH_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_GREATER_OR_EQUAL,
                (*LockHandle).Lock as ULONG_PTR,
                KeGetCurrentIrql() as ULONG_PTR,
                0,
                0,
            );
        }
    }

    KxAcquireSpinLock((*LockHandle).Lock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseQueuedSpinLockFromDpcLevel(LockHandle: PKSPIN_LOCK_QUEUE) {
    KeReleaseQueuedSpinLockFromDpcLevelInner(LockHandle)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseQueuedSpinLockFromDpcLevel(LockHandle: PKSPIN_LOCK_QUEUE) {
    KeReleaseQueuedSpinLockFromDpcLevelInner(LockHandle)
}

#[inline(always)]
unsafe fn KeReleaseQueuedSpinLockFromDpcLevelInner(LockHandle: PKSPIN_LOCK_QUEUE) {
    #[cfg(any(debug_assertions, feature = "smp"))]
    {
        if KeGetCurrentIrql() < DISPATCH_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_GREATER_OR_EQUAL,
                (*LockHandle).Lock as ULONG_PTR,
                KeGetCurrentIrql() as ULONG_PTR,
                0,
                0,
            );
        }
    }

    KxReleaseSpinLock((*LockHandle).Lock as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn KeAcquireInterruptSpinLock(Interrupt: PKINTERRUPT) -> KIRQL {
    let Interrupt = opaque_ptr(Interrupt);
    let mut OldIrql: KIRQL = 0;

    KeRaiseIrql((*Interrupt).SynchronizeIrql, core::ptr::addr_of_mut!(OldIrql));

    KeAcquireSpinLockAtDpcLevel((*Interrupt).ActualLock as PKSPIN_LOCK);
    OldIrql
}

#[no_mangle]
pub unsafe extern "system" fn KeReleaseInterruptSpinLock(
    Interrupt: PKINTERRUPT,
    OldIrql: KIRQL,
) {
    let Interrupt = opaque_ptr(Interrupt);
    KeReleaseSpinLockFromDpcLevel((*Interrupt).ActualLock as PKSPIN_LOCK);

    KeLowerIrql(OldIrql);
}

#[no_mangle]
pub unsafe extern "system" fn _KeInitializeSpinLock(SpinLock: PKSPIN_LOCK) {
    // Any alignment, as Windows takes it: see ntkrnl_sys::inline::ke::spin_lock_word.
    set_spin_lock_word(SpinLock as _, 0);
}

#[no_mangle]
pub unsafe extern "system" fn KeAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) {
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    KxAcquireSpinLock(SpinLock as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn KeReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK) {
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    KxReleaseSpinLock(SpinLock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KefAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) {
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    KxAcquireSpinLock(SpinLock as *mut _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KefAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) {
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    KxAcquireSpinLock(SpinLock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KefReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK) {
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    KxReleaseSpinLock(SpinLock as *mut _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KefReleaseSpinLockFromDpcLevel(SpinLock: PKSPIN_LOCK) {
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    KxReleaseSpinLock(SpinLock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiAcquireSpinLock(SpinLock: PKSPIN_LOCK) {
    KxAcquireSpinLock(SpinLock as *mut _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiAcquireSpinLock(SpinLock: PKSPIN_LOCK) {
    KxAcquireSpinLock(SpinLock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiReleaseSpinLock(SpinLock: PKSPIN_LOCK) {
    KxReleaseSpinLock(SpinLock as *mut _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KiReleaseSpinLock(SpinLock: PKSPIN_LOCK) {
    KxReleaseSpinLock(SpinLock as *mut _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeTryToAcquireSpinLockAtDpcLevel(
    SpinLock: PKSPIN_LOCK,
) -> BOOLEAN {
    // Upstream gates all three of these on plain `#if DBG`. They were translated
    // to cargo's `debug_assertions`, which is a DIFFERENT switch and one this
    // workspace's release profile leaves OFF (sdk/lib/rust/Cargo.toml) — so on
    // every kernel we have ever shipped, this function acquired the lock and
    // never stamped it, while KxAcquireSpinLock (gated on `feature = "dbg"`)
    // did. spinlock.h says in as many words that the two halves of this funnel
    // must agree; a lock taken here read as FREE to KeTestSpinLock and to every
    // DDK caller that inspects it, and on the UP kernel — where the interlocked
    // body below compiles away entirely — the whole function was a no-op that
    // returned TRUE. `feature = "dbg"` is the mirror of DBG=1; use it.
    #[cfg(feature = "dbg")]
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    // Paired with `smp` for the same reason KxAcquireSpinLock's copy is: only on
    // an MP kernel is finding our own thread already stamped a guaranteed
    // deadlock. On UP the acquire below is compiled out, so a recursive try is
    // harmless and bugchecking on it would invent a failure the UP boot lacks.
    #[cfg(all(feature = "smp", feature = "dbg"))]
    if ((PsGetCurrentThread() as KSPIN_LOCK) | 1) == spin_lock_word(SpinLock as _) as KSPIN_LOCK {
        KeBugCheckEx(SPIN_LOCK_ALREADY_OWNED, SpinLock as ULONG_PTR, 0, 0, 0);
    }

    #[cfg(feature = "smp")]
    {
        if spin_lock_word(SpinLock as _) == 0 {
            if InterlockedBitTestAndSet(SpinLock as *mut LONG, 0) != 0 {
                return 0u8;
            }
        } else {
            return 0u8;
        }
    }

    // KI_TRACK_SPINLOCK_OWNER. Observable behaviour, not a diagnostic.
    #[cfg(feature = "dbg")]
    {
        set_spin_lock_word(SpinLock as _, (PsGetCurrentThread() as ULONG_PTR) | 1);
    }

    // cswitchdep P2: the UP try-acquire success path never touches Kx, so it
    // taps the held-counter directly; the matching release DOES route through
    // KxReleaseSpinLock, keeping the pair balanced.
    #[cfg(feature = "cswitchdep")]
    super::cswitchdep::spin_acquired(SpinLock as usize);

    1u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeTryToAcquireSpinLockAtDpcLevel(SpinLock: PKSPIN_LOCK) -> BOOLEAN {
    // See the x86 copy above for why these are `feature = "dbg"` and not
    // `debug_assertions`: they are DBG=1 behaviour, and the profile flag they
    // were translated to is off in every kernel this tree ships.
    #[cfg(feature = "dbg")]
    if KeGetCurrentIrql() < DISPATCH_LEVEL {
        KeBugCheckEx(
            IRQL_NOT_GREATER_OR_EQUAL,
            SpinLock as ULONG_PTR,
            KeGetCurrentIrql() as ULONG_PTR,
            0,
            0,
        );
    }

    #[cfg(all(feature = "smp", feature = "dbg"))]
    if ((PsGetCurrentThread() as KSPIN_LOCK) | 1) == spin_lock_word(SpinLock as _) as KSPIN_LOCK {
        KeBugCheckEx(SPIN_LOCK_ALREADY_OWNED, SpinLock as ULONG_PTR, 0, 0, 0);
    }

    #[cfg(feature = "smp")]
    {
        if spin_lock_word(SpinLock as _) == 0 {
            if InterlockedBitTestAndSet(SpinLock as *mut LONG, 0) != 0 {
                return 0u8;
            }
        } else {
            return 0u8;
        }
    }

    // KI_TRACK_SPINLOCK_OWNER. Observable behaviour, not a diagnostic.
    #[cfg(feature = "dbg")]
    {
        set_spin_lock_word(SpinLock as _, (PsGetCurrentThread() as ULONG_PTR) | 1);
    }

    // cswitchdep P2: the UP try-acquire success path never touches Kx, so it
    // taps the held-counter directly; the matching release DOES route through
    // KxReleaseSpinLock, keeping the pair balanced.
    #[cfg(feature = "cswitchdep")]
    super::cswitchdep::spin_acquired(SpinLock as usize);

    1u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireInStackQueuedSpinLockAtDpcLevel(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).LockQueue.Next = core::ptr::null_mut();
    (*LockHandle).LockQueue.Lock = SpinLock as *mut _;
    #[cfg(feature = "smp")]
    {
        if KeGetCurrentIrql() < DISPATCH_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_GREATER_OR_EQUAL,
                (*LockHandle).LockQueue.Lock as ULONG_PTR,
                KeGetCurrentIrql() as ULONG_PTR,
                0,
                0,
            );
        }
    }

    ntkrnl_sys::inline::ke::KxAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireInStackQueuedSpinLockAtDpcLevel(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).LockQueue.Next = core::ptr::null_mut();
    (*LockHandle).LockQueue.Lock = SpinLock as *mut _;
    #[cfg(feature = "smp")]
    {
        if KeGetCurrentIrql() < DISPATCH_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_GREATER_OR_EQUAL,
                (*LockHandle).LockQueue.Lock as ULONG_PTR,
                KeGetCurrentIrql() as ULONG_PTR,
                0,
                0,
            );
        }
    }

    ntkrnl_sys::inline::ke::KxAcquireInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseInStackQueuedSpinLockFromDpcLevel(
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    #[cfg(feature = "smp")]
    {
        if KeGetCurrentIrql() < DISPATCH_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_GREATER_OR_EQUAL,
                (*LockHandle).LockQueue.Lock as ULONG_PTR,
                KeGetCurrentIrql() as ULONG_PTR,
                0,
                0,
            );
        }
    }

    ntkrnl_sys::inline::ke::KxReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseInStackQueuedSpinLockFromDpcLevel(
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    #[cfg(feature = "smp")]
    {
        if KeGetCurrentIrql() < DISPATCH_LEVEL {
            KeBugCheckEx(
                IRQL_NOT_GREATER_OR_EQUAL,
                (*LockHandle).LockQueue.Lock as ULONG_PTR,
                KeGetCurrentIrql() as ULONG_PTR,
                0,
                0,
            );
        }
    }

    ntkrnl_sys::inline::ke::KxReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!((*LockHandle).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireSpinLockForDpc(SpinLock: PKSPIN_LOCK) -> KIRQL {
    KeGetCurrentIrql()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireSpinLockForDpc(SpinLock: PKSPIN_LOCK) -> KIRQL {
    KeGetCurrentIrql()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseSpinLockForDpc(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL) {
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseSpinLockForDpc(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL) {
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireInStackQueuedSpinLockForDpc(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).OldIrql = KeGetCurrentIrql();
    if (*LockHandle).OldIrql >= DISPATCH_LEVEL {
        KeAcquireInStackQueuedSpinLockAtDpcLevel(SpinLock, LockHandle);
    } else {
        KeAcquireInStackQueuedSpinLock(SpinLock, LockHandle);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireInStackQueuedSpinLockForDpc(
    SpinLock: PKSPIN_LOCK,
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    (*LockHandle).OldIrql = KeGetCurrentIrql();
    if (*LockHandle).OldIrql >= DISPATCH_LEVEL {
        KeAcquireInStackQueuedSpinLockAtDpcLevel(SpinLock, LockHandle);
    } else {
        KeAcquireInStackQueuedSpinLock(SpinLock, LockHandle);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseInStackQueuedSpinLockForDpc(
    LockHandle: PKLOCK_QUEUE_HANDLE,
) {
    if (*LockHandle).OldIrql >= DISPATCH_LEVEL {
        KeReleaseInStackQueuedSpinLockFromDpcLevel(LockHandle);
    } else {
        KeReleaseInStackQueuedSpinLock(LockHandle);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseInStackQueuedSpinLockForDpc(LockHandle: PKLOCK_QUEUE_HANDLE) {
    if (*LockHandle).OldIrql >= DISPATCH_LEVEL {
        KeReleaseInStackQueuedSpinLockFromDpcLevel(LockHandle);
    } else {
        KeReleaseInStackQueuedSpinLock(LockHandle);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeTestSpinLock(SpinLock: PKSPIN_LOCK) -> BOOLEAN {
    if spin_lock_word(SpinLock as _) != 0 {
        YieldProcessor();
        return 0u8;
    }

    1u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeTestSpinLock(SpinLock: PKSPIN_LOCK) -> BOOLEAN {
    if spin_lock_word(SpinLock as _) != 0 {
        YieldProcessor();
        return 0u8;
    }

    1u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn Kii386SpinOnSpinLock(SpinLock: PKSPIN_LOCK, Flags: ULONG) {
    while (*(SpinLock as *const KSPIN_LOCK) & 1) != 0 {
        YieldProcessor();
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/spinlock.c
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
// #define LQ_WAIT     1
// #define LQ_OWN      2
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// #if 0
// //
// // FIXME: The queued spinlock routines are broken.
// //
//
// VOID
// FASTCALL
// KeAcquireQueuedSpinLockAtDpcLevel(IN PKSPIN_LOCK_QUEUE LockHandle)
// {
// #ifdef CONFIG_SMP
//     PKSPIN_LOCK_QUEUE Prev;
//
//     /* Set the new lock */
//     Prev = (PKSPIN_LOCK_QUEUE)
//            InterlockedExchange((PLONG)LockHandle->Next,
//                                (LONG)LockHandle);
//     if (!Prev)
//     {
//         /* There was nothing there before. We now own it */
//          *LockHandle->Lock |= LQ_OWN;
//         return;
//     }
//
//     /* Set the wait flag */
//      *LockHandle->Lock |= LQ_WAIT;
//
//     /* Link us */
//     Prev->Next = (PKSPIN_LOCK_QUEUE)LockHandle;
//
//     /* Loop and wait */
//     while (*LockHandle->Lock & LQ_WAIT)
//         YieldProcessor();
// #endif
// }
//
// VOID
// FASTCALL
// KeReleaseQueuedSpinLockFromDpcLevel(IN PKSPIN_LOCK_QUEUE LockHandle)
// {
// #ifdef CONFIG_SMP
//     KSPIN_LOCK LockVal;
//     PKSPIN_LOCK_QUEUE Waiter;
//
//     /* Remove own and wait flags */
//     *LockHandle->Lock &= ~(LQ_OWN | LQ_WAIT);
//     LockVal = *LockHandle->Lock;
//
//     /* Check if we already own it */
//     if (LockVal == (KSPIN_LOCK)LockHandle)
//     {
//         /* Disown it */
//         LockVal = (KSPIN_LOCK)
//                   InterlockedCompareExchangePointer(LockHandle->Lock,
//                                                     NULL,
//                                                     LockHandle);
//     }
//     if (LockVal == (KSPIN_LOCK)LockHandle) return;
//
//     /* Need to wait for it */
//     Waiter = LockHandle->Next;
//     while (!Waiter)
//     {
//         YieldProcessor();
//         Waiter = LockHandle->Next;
//     }
//
//     /* It's gone */
//     *(ULONG_PTR*)&Waiter->Lock ^= (LQ_OWN | LQ_WAIT);
//     LockHandle->Next = NULL;
// #endif
// }
//
// #else
// //
// // HACK: Hacked to work like normal spinlocks
// //
//
// _IRQL_requires_min_(DISPATCH_LEVEL)
// _Acquires_nonreentrant_lock_(*LockHandle->Lock)
// _Acquires_exclusive_lock_(*LockHandle->Lock)
// VOID
// FASTCALL
// KeAcquireQueuedSpinLockAtDpcLevel(_Inout_ PKSPIN_LOCK_QUEUE LockHandle)
// {
// #if defined(CONFIG_SMP) || DBG
//     /* Make sure we are at DPC or above! */
//     if (KeGetCurrentIrql() < DISPATCH_LEVEL)
//     {
//         /* We aren't -- bugcheck */
//         KeBugCheckEx(IRQL_NOT_GREATER_OR_EQUAL,
//                      (ULONG_PTR)LockHandle->Lock,
//                      KeGetCurrentIrql(),
//                      0,
//                      0);
//     }
// #endif
//
//     /* Do the inlined function */
//     KxAcquireSpinLock(LockHandle->Lock);
// }
//
// _IRQL_requires_min_(DISPATCH_LEVEL)
// _Releases_nonreentrant_lock_(*LockHandle->Lock)
// _Releases_exclusive_lock_(*LockHandle->Lock)
// VOID
// FASTCALL
// KeReleaseQueuedSpinLockFromDpcLevel(_Inout_ PKSPIN_LOCK_QUEUE LockHandle)
// {
// #if defined(CONFIG_SMP) || DBG
//     /* Make sure we are at DPC or above! */
//     if (KeGetCurrentIrql() < DISPATCH_LEVEL)
//     {
//         /* We aren't -- bugcheck */
//         KeBugCheckEx(IRQL_NOT_GREATER_OR_EQUAL,
//                      (ULONG_PTR)LockHandle->Lock,
//                      KeGetCurrentIrql(),
//                      0,
//                      0);
//     }
// #endif
//
//     /* Do the inlined function */
//     KxReleaseSpinLock(LockHandle->Lock);
// }
//
// #endif
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// KIRQL NTAPI KeAcquireInterruptSpinLock(IN PKINTERRUPT Interrupt) { ... }
// VOID NTAPI KeReleaseInterruptSpinLock(IN PKINTERRUPT Interrupt, IN KIRQL OldIrql) { ... }
// VOID NTAPI _KeInitializeSpinLock(IN PKSPIN_LOCK SpinLock) { ... }
// VOID NTAPI KeAcquireSpinLockAtDpcLevel(IN PKSPIN_LOCK SpinLock) { ... }
// VOID NTAPI KeReleaseSpinLockFromDpcLevel(IN PKSPIN_LOCK SpinLock) { ... }
// VOID FASTCALL KefAcquireSpinLockAtDpcLevel(IN PKSPIN_LOCK SpinLock) { ... }
// VOID FASTCALL KefReleaseSpinLockFromDpcLevel(IN PKSPIN_LOCK SpinLock) { ... }
// VOID FASTCALL KiAcquireSpinLock(IN PKSPIN_LOCK SpinLock) { ... }
// VOID FASTCALL KiReleaseSpinLock(IN PKSPIN_LOCK SpinLock) { ... }
// BOOLEAN FASTCALL KeTryToAcquireSpinLockAtDpcLevel(IN OUT PKSPIN_LOCK SpinLock) { ... }
// VOID FASTCALL KeAcquireInStackQueuedSpinLockAtDpcLevel(...) { ... }
// VOID FASTCALL KeReleaseInStackQueuedSpinLockFromDpcLevel(...) { ... }
// KIRQL FASTCALL KeAcquireSpinLockForDpc(IN PKSPIN_LOCK SpinLock) { ... }
// VOID FASTCALL KeReleaseSpinLockForDpc(IN PKSPIN_LOCK SpinLock, IN KIRQL OldIrql) { ... }
// VOID FASTCALL KeAcquireInStackQueuedSpinLockForDpc(...) { ... }
// VOID FASTCALL KeReleaseInStackQueuedSpinLockForDpc(...) { ... }
// BOOLEAN FASTCALL KeTestSpinLock(IN PKSPIN_LOCK SpinLock) { ... }
// #ifdef _M_IX86
// VOID NTAPI Kii386SpinOnSpinLock(PKSPIN_LOCK SpinLock, ULONG Flags) { ... }
// #endif
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
