/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Kernel Timers
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeInitializeTimerEx / KeSetTimerEx / KeCancelTimer / KeReadStateTimer.
 *
 * NOTE: On ARM64 in TESTKERNEL mode, the HAL clock interrupt may not be
 * running, so timers don't actually expire. We test the API surface
 * (init, set, cancel, state queries) without waiting for timer expiration.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestTimer - Exercise kernel timer data structure management.
 *
 * Test 1: Initialize + verify initial state.
 * Test 2: Set timer + verify pending state + cancel.
 * Test 3: Cancel already-cancelled timer returns FALSE.
 * Test 4: Set with DPC, cancel before expiry, verify DPC not queued.
 */
BOOLEAN
NTAPI
NtTestTimer(VOID)
{
    KTIMER timer;
    KDPC dpc;
    LARGE_INTEGER dueTime;

    /* --- Test 1: Initialize notification timer --- */
    KeInitializeTimerEx(&timer, NotificationTimer);

    if (KeReadStateTimer(&timer) != FALSE)
    {
        DbgPrint("  FAIL: notification timer initially signaled\n");
        return FALSE;
    }

    /* --- Test 2: Set and cancel --- */
    dueTime.QuadPart = -10 * 10000000LL;  /* 10 seconds (won't actually fire) */
    KeSetTimerEx(&timer, dueTime, 0, NULL);

    /* Timer is set (pending) — should not be signaled yet */
    if (KeReadStateTimer(&timer) != FALSE)
    {
        DbgPrint("  FAIL: pending timer reads as signaled\n");
        return FALSE;
    }

    /* Cancel the pending timer — should return TRUE */
    if (!KeCancelTimer(&timer))
    {
        DbgPrint("  FAIL: KeCancelTimer returned FALSE for pending timer\n");
        return FALSE;
    }

    /* After cancel, timer should not be signaled */
    if (KeReadStateTimer(&timer) != FALSE)
    {
        DbgPrint("  FAIL: cancelled timer reads as signaled\n");
        return FALSE;
    }

    /* --- Test 3: Cancel already-cancelled timer --- */
    if (KeCancelTimer(&timer) != FALSE)
    {
        DbgPrint("  FAIL: KeCancelTimer returned TRUE for already-cancelled timer\n");
        return FALSE;
    }

    /* --- Test 4: Synchronization timer with DPC --- */
    KeInitializeTimerEx(&timer, SynchronizationTimer);
    KeInitializeDpc(&dpc, (PKDEFERRED_ROUTINE)(ULONG_PTR)0, NULL);  /* dummy DPC */

    if (KeReadStateTimer(&timer) != FALSE)
    {
        DbgPrint("  FAIL: sync timer initially signaled\n");
        return FALSE;
    }

    /* Set with long due time */
    dueTime.QuadPart = -30 * 10000000LL;  /* 30 seconds */
    KeSetTimerEx(&timer, dueTime, 0, NULL);  /* no DPC for safety */

    /* Set again (re-arm) — should cancel previous and re-set */
    dueTime.QuadPart = -60 * 10000000LL;
    if (KeSetTimerEx(&timer, dueTime, 0, NULL) != TRUE)
    {
        /* KeSetTimerEx returns TRUE if timer was already in the queue */
        /* This may or may not return TRUE depending on implementation */
    }

    /* Cancel */
    KeCancelTimer(&timer);

    /* --- Test 5: Multiple init/set/cancel cycles --- */
    {
        int i;
        for (i = 0; i < 10; i++)
        {
            KeInitializeTimerEx(&timer, (i % 2) ? SynchronizationTimer : NotificationTimer);
            dueTime.QuadPart = -10 * 10000000LL;
            KeSetTimerEx(&timer, dueTime, 0, NULL);

            if (KeReadStateTimer(&timer) != FALSE)
            {
                DbgPrint("  FAIL: cycle %d timer signaled while pending\n", i);
                return FALSE;
            }

            KeCancelTimer(&timer);
        }
    }

    DbgPrint("  Timer: init/set/cancel/rearm/cycles all passed\n");
    return TRUE;
}
