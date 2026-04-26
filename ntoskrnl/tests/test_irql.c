/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — IRQL and Spinlocks
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeRaiseIrql / KeLowerIrql / KeAcquireSpinLock / KeReleaseSpinLock
 * and Interlocked operations on ARM64.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestIrql - Exercise IRQL manipulation and spinlocks.
 *
 * Test 1: Raise/lower IRQL at various levels.
 * Test 2: Spinlock acquire/release with IRQL verification.
 * Test 3: Interlocked operations (Increment, Decrement, CompareExchange).
 */
BOOLEAN
NTAPI
NtTestIrql(VOID)
{
    KIRQL oldIrql, currentIrql;
    KSPIN_LOCK spinLock;
    volatile LONG counter;

    /* --- Test 1: IRQL manipulation --- */
    currentIrql = KeGetCurrentIrql();
    if (currentIrql != PASSIVE_LEVEL)
    {
        DbgPrint("  FAIL: initial IRQL=%u, expected PASSIVE_LEVEL(%u)\n",
                 currentIrql, PASSIVE_LEVEL);
        return FALSE;
    }

    /* Raise to APC_LEVEL */
    KeRaiseIrql(APC_LEVEL, &oldIrql);
    if (oldIrql != PASSIVE_LEVEL)
    {
        DbgPrint("  FAIL: oldIrql from raise to APC=%u, expected PASSIVE\n", oldIrql);
        KeLowerIrql(PASSIVE_LEVEL);
        return FALSE;
    }
    if (KeGetCurrentIrql() != APC_LEVEL)
    {
        DbgPrint("  FAIL: current IRQL=%u after raise, expected APC_LEVEL(%u)\n",
                 KeGetCurrentIrql(), APC_LEVEL);
        KeLowerIrql(PASSIVE_LEVEL);
        return FALSE;
    }

    /* Raise to DISPATCH_LEVEL */
    KeRaiseIrql(DISPATCH_LEVEL, &oldIrql);
    if (oldIrql != APC_LEVEL)
    {
        DbgPrint("  FAIL: oldIrql from raise to DISPATCH=%u, expected APC\n", oldIrql);
        KeLowerIrql(PASSIVE_LEVEL);
        return FALSE;
    }
    if (KeGetCurrentIrql() != DISPATCH_LEVEL)
    {
        DbgPrint("  FAIL: current IRQL=%u after raise, expected DISPATCH_LEVEL(%u)\n",
                 KeGetCurrentIrql(), DISPATCH_LEVEL);
        KeLowerIrql(PASSIVE_LEVEL);
        return FALSE;
    }

    /* Lower back to PASSIVE */
    KeLowerIrql(APC_LEVEL);
    if (KeGetCurrentIrql() != APC_LEVEL)
    {
        DbgPrint("  FAIL: IRQL=%u after lower to APC\n", KeGetCurrentIrql());
        KeLowerIrql(PASSIVE_LEVEL);
        return FALSE;
    }
    KeLowerIrql(PASSIVE_LEVEL);
    if (KeGetCurrentIrql() != PASSIVE_LEVEL)
    {
        DbgPrint("  FAIL: IRQL=%u after lower to PASSIVE\n", KeGetCurrentIrql());
        return FALSE;
    }

    /* --- Test 2: Spinlock --- */
    KeInitializeSpinLock(&spinLock);

    KeAcquireSpinLock(&spinLock, &oldIrql);

    /* IRQL should be at DISPATCH_LEVEL while holding spinlock */
    if (KeGetCurrentIrql() != DISPATCH_LEVEL)
    {
        DbgPrint("  FAIL: IRQL=%u while holding spinlock, expected DISPATCH\n",
                 KeGetCurrentIrql());
        KeReleaseSpinLock(&spinLock, oldIrql);
        return FALSE;
    }

    KeReleaseSpinLock(&spinLock, oldIrql);

    /* Should be back to PASSIVE */
    if (KeGetCurrentIrql() != PASSIVE_LEVEL)
    {
        DbgPrint("  FAIL: IRQL=%u after spinlock release, expected PASSIVE\n",
                 KeGetCurrentIrql());
        return FALSE;
    }

    /* --- Test 3: Interlocked operations --- */
    counter = 0;

    if (InterlockedIncrement(&counter) != 1)
    {
        DbgPrint("  FAIL: InterlockedIncrement returned != 1\n");
        return FALSE;
    }
    if (counter != 1)
    {
        DbgPrint("  FAIL: counter=%ld after InterlockedIncrement\n", counter);
        return FALSE;
    }

    if (InterlockedIncrement(&counter) != 2)
    {
        DbgPrint("  FAIL: second InterlockedIncrement returned != 2\n");
        return FALSE;
    }

    if (InterlockedDecrement(&counter) != 1)
    {
        DbgPrint("  FAIL: InterlockedDecrement returned != 1\n");
        return FALSE;
    }

    /* CompareExchange: should succeed (counter == 1, exchange to 99) */
    if (InterlockedCompareExchange(&counter, 99, 1) != 1)
    {
        DbgPrint("  FAIL: InterlockedCompareExchange old != 1\n");
        return FALSE;
    }
    if (counter != 99)
    {
        DbgPrint("  FAIL: counter=%ld after CmpXchg, expected 99\n", counter);
        return FALSE;
    }

    /* CompareExchange: should fail (counter == 99, comparand == 1) */
    if (InterlockedCompareExchange(&counter, 0, 1) != 99)
    {
        DbgPrint("  FAIL: InterlockedCompareExchange should have returned 99\n");
        return FALSE;
    }
    if (counter != 99)
    {
        DbgPrint("  FAIL: counter changed on failed CmpXchg\n");
        return FALSE;
    }

    /* InterlockedExchange */
    if (InterlockedExchange(&counter, 42) != 99)
    {
        DbgPrint("  FAIL: InterlockedExchange old != 99\n");
        return FALSE;
    }
    if (counter != 42)
    {
        DbgPrint("  FAIL: counter=%ld after InterlockedExchange, expected 42\n", counter);
        return FALSE;
    }

    DbgPrint("  Irql: raise/lower + spinlock + interlocked ops passed\n");
    return TRUE;
}
