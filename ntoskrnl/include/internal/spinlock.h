/*
* PROJECT:         ReactOS Kernel
* LICENSE:         GPL - See COPYING in the top level directory
* FILE:            ntoskrnl/include/internal/spinlock.h
* PURPOSE:         Internal Inlined Functions for spinlocks, shared with HAL
* PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
*/

#if defined(_M_IX86)
VOID
NTAPI
Kii386SpinOnSpinLock(PKSPIN_LOCK SpinLock, ULONG Flags);
#endif

//
// A checked build does TWO separate things to a spinlock, and this port needs
// them separated.
//
//   KI_TRACK_SPINLOCK_OWNER  — stamp `KeGetCurrentThread() | 1` into the lock
//     while it is held, and clear it on release. This is OBSERVABLE BEHAVIOUR:
//     `KeTestSpinLock` reads it, and every DDK caller that inspects a held lock
//     depends on it. Upstream gates this on plain `#if DBG`.
//
//   KI_VALIDATE_SPINLOCK_OWNER — the two KeBugCheckEx calls that turn a
//     mismatched acquire/release pair into SPIN_LOCK_ALREADY_OWNED /
//     SPIN_LOCK_NOT_OWNED.
//
// This tree narrowed BOTH to `DBG && !_USE_DUMMY_PSEH && !_USE_PSEH_CLANG`
// (5c8505479a3, then 15434cd8dea) as an i386 boot bring-up workaround, because
// the *bugchecks* were firing on inconsistent C-vs-Rust funnels. Suppressing
// the store as well was collateral: it left a build that reports
// KmtIsCheckedBuild = TRUE while behaving like a free build, so a held lock read
// as free. Restore the store; keep the bugchecks off until the mismatched pairs
// are found. The Rust half of the funnel (ntkrnl-sys inline/ke.rs KxAcquire/
// KxReleaseSpinLock) mirrors this split under `feature = "dbg"` — the two MUST
// agree, or a lock stamped by one side is never cleared by the other.
//
#if DBG
#define KI_TRACK_SPINLOCK_OWNER 1
#else
#define KI_TRACK_SPINLOCK_OWNER 0
#endif

#if DBG && !defined(_USE_DUMMY_PSEH) && !defined(_USE_PSEH_CLANG)
#define KI_VALIDATE_SPINLOCK_OWNER 1
#else
#define KI_VALIDATE_SPINLOCK_OWNER 0
#endif

//
// Spinlock Acquisition at IRQL >= DISPATCH_LEVEL
//
_Acquires_nonreentrant_lock_(SpinLock)
FORCEINLINE
VOID
KxAcquireSpinLock(
#if defined(CONFIG_SMP) || DBG
    _Inout_
#else
    _Unreferenced_parameter_
#endif
    PKSPIN_LOCK SpinLock)
{
#if KI_VALIDATE_SPINLOCK_OWNER
    /* Make sure that we don't own the lock already */
    if (((KSPIN_LOCK)KeGetCurrentThread() | 1) == *SpinLock)
    {
        /* We do, bugcheck! */
        KeBugCheckEx(SPIN_LOCK_ALREADY_OWNED, (ULONG_PTR)SpinLock, 0, 0, 0);
    }
#endif

#ifdef CONFIG_SMP
    /* Try to acquire the lock */
    while (InterlockedBitTestAndSet((PLONG)SpinLock, 0))
    {
#if defined(_M_IX86) && DBG
        /* On x86 debug builds, we use a much slower but useful routine */
        Kii386SpinOnSpinLock(SpinLock, 5);
#else
        /* It's locked... spin until it's unlocked */
        while (*(volatile KSPIN_LOCK *)SpinLock & 1)
        {
                /* Yield and keep looping */
                YieldProcessor();
        }
#endif
    }
#endif

    /* Add an explicit memory barrier to prevent the compiler from reordering
       memory accesses across the borders of spinlocks */
    KeMemoryBarrierWithoutFence();

#if KI_TRACK_SPINLOCK_OWNER
    /* On debug builds, we OR in the KTHREAD */
    *SpinLock = (KSPIN_LOCK)KeGetCurrentThread() | 1;
#endif
}

//
// Spinlock Release at IRQL >= DISPATCH_LEVEL
//
_Releases_nonreentrant_lock_(SpinLock)
FORCEINLINE
VOID
KxReleaseSpinLock(
#if defined(CONFIG_SMP) || DBG
    _Inout_
#else
    _Unreferenced_parameter_
#endif
    PKSPIN_LOCK SpinLock)
{
#if KI_VALIDATE_SPINLOCK_OWNER
    /* Make sure that the threads match */
    if (((KSPIN_LOCK)KeGetCurrentThread() | 1) != *SpinLock)
    {
        /* They don't, bugcheck */
        KeBugCheckEx(SPIN_LOCK_NOT_OWNED, (ULONG_PTR)SpinLock, 0, 0, 0);
    }
#endif

#if defined(CONFIG_SMP) || KI_TRACK_SPINLOCK_OWNER
    /* Clear the lock  */
#ifdef _WIN64
    InterlockedAnd64((PLONG64)SpinLock, 0);
#else
    InterlockedAnd((PLONG)SpinLock, 0);
#endif
#endif

    /* Add an explicit memory barrier to prevent the compiler from reordering
       memory accesses across the borders of spinlocks */
    KeMemoryBarrierWithoutFence();
}
