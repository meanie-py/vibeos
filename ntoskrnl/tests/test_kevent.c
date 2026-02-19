/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — kernel event synchronization
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeInitializeEvent / KeSetEvent / KeWaitForSingleObject
 * using kernel-mode KEVENT objects (no handles).
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestKeEvent - Exercise kernel synchronization primitives.
 *
 * Initializes notification and synchronization events, sets them,
 * waits on them (zero timeout), and verifies state transitions.
 * Returns TRUE if all pass.
 */
BOOLEAN
NTAPI
NtTestKeEvent(VOID)
{
    KEVENT notifEvent, syncEvent;
    NTSTATUS status;
    LONG prevState;
    LARGE_INTEGER zeroTimeout;

    zeroTimeout.QuadPart = 0;

    /* --- Test 1: Notification event (manual-reset) --- */
    KeInitializeEvent(&notifEvent, NotificationEvent, FALSE);

    /* Initial state should be non-signaled */
    if (KeReadStateEvent(&notifEvent) != 0)
    {
        DbgPrint("  FAIL: notification event initial state != 0\n");
        return FALSE;
    }

    /* Set the event — returns previous state (0) */
    prevState = KeSetEvent(&notifEvent, IO_NO_INCREMENT, FALSE);
    if (prevState != 0)
    {
        DbgPrint("  FAIL: KeSetEvent returned prevState=%ld, expected 0\n", prevState);
        return FALSE;
    }

    /* Should be signaled now */
    if (KeReadStateEvent(&notifEvent) == 0)
    {
        DbgPrint("  FAIL: notification event not signaled after KeSetEvent\n");
        return FALSE;
    }

    /* Wait should succeed immediately (zero timeout) */
    status = KeWaitForSingleObject(&notifEvent, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: KeWait on signaled notification event returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Notification event stays signaled after wait */
    if (KeReadStateEvent(&notifEvent) == 0)
    {
        DbgPrint("  FAIL: notification event reset after wait (should stay signaled)\n");
        return FALSE;
    }

    /* Reset it manually */
    KeResetEvent(&notifEvent);
    if (KeReadStateEvent(&notifEvent) != 0)
    {
        DbgPrint("  FAIL: notification event still signaled after KeResetEvent\n");
        return FALSE;
    }

    /* --- Test 2: Synchronization event (auto-reset) --- */
    KeInitializeEvent(&syncEvent, SynchronizationEvent, FALSE);

    /* Set it */
    prevState = KeSetEvent(&syncEvent, IO_NO_INCREMENT, FALSE);
    if (prevState != 0)
    {
        DbgPrint("  FAIL: sync event KeSetEvent returned prevState=%ld, expected 0\n", prevState);
        return FALSE;
    }

    /* Wait should succeed and auto-reset */
    status = KeWaitForSingleObject(&syncEvent, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: KeWait on signaled sync event returned 0x%08lx\n", status);
        return FALSE;
    }

    /* After wait, synchronization event should be non-signaled (auto-reset) */
    if (KeReadStateEvent(&syncEvent) != 0)
    {
        DbgPrint("  FAIL: sync event still signaled after wait (should auto-reset)\n");
        return FALSE;
    }

    /* Wait on non-signaled event with zero timeout should return STATUS_TIMEOUT */
    status = KeWaitForSingleObject(&syncEvent, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: KeWait on non-signaled sync event returned 0x%08lx, expected TIMEOUT\n",
                 status);
        return FALSE;
    }

    DbgPrint("  KeEvent: notification + synchronization event tests passed\n");
    return TRUE;
}
