/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — KeWaitForMultipleObjects
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests WaitAll and WaitAny modes with events.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestMultiWait - Exercise KeWaitForMultipleObjects.
 *
 * Test 1: WaitAll with all events signaled — should succeed immediately.
 * Test 2: WaitAll with one event not signaled — should timeout.
 * Test 3: WaitAny with one event signaled — should return its index.
 * Test 4: WaitAny with no events signaled — should timeout.
 */
BOOLEAN
NTAPI
NtTestMultiWait(VOID)
{
    #define NUM_EVENTS 3
    KEVENT events[NUM_EVENTS];
    PVOID waitObjects[NUM_EVENTS];
    KWAIT_BLOCK waitBlocks[NUM_EVENTS];
    NTSTATUS status;
    LARGE_INTEGER zeroTimeout;
    int i;

    zeroTimeout.QuadPart = 0;

    /* Initialize events */
    for (i = 0; i < NUM_EVENTS; i++)
    {
        KeInitializeEvent(&events[i], NotificationEvent, FALSE);
        waitObjects[i] = &events[i];
    }

    /* --- Test 1: WaitAll, all signaled --- */
    for (i = 0; i < NUM_EVENTS; i++)
        KeSetEvent(&events[i], IO_NO_INCREMENT, FALSE);

    status = KeWaitForMultipleObjects(NUM_EVENTS, waitObjects, WaitAll,
                                      Executive, KernelMode, FALSE,
                                      &zeroTimeout, waitBlocks);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: WaitAll (all signaled) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* --- Test 2: WaitAll, one not signaled --- */
    KeResetEvent(&events[1]);  /* events[0] and [2] still signaled */

    status = KeWaitForMultipleObjects(NUM_EVENTS, waitObjects, WaitAll,
                                      Executive, KernelMode, FALSE,
                                      &zeroTimeout, waitBlocks);
    if (status != STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: WaitAll (one reset) returned 0x%08lx, expected TIMEOUT\n", status);
        return FALSE;
    }

    /* --- Test 3: WaitAny, events[2] signaled --- */
    KeResetEvent(&events[0]);
    /* events[1] already reset, events[2] still signaled */

    status = KeWaitForMultipleObjects(NUM_EVENTS, waitObjects, WaitAny,
                                      Executive, KernelMode, FALSE,
                                      &zeroTimeout, waitBlocks);
    if (status != STATUS_WAIT_0 + 2)
    {
        DbgPrint("  FAIL: WaitAny returned 0x%08lx, expected STATUS_WAIT_0+2 (0x%08lx)\n",
                 status, (NTSTATUS)(STATUS_WAIT_0 + 2));
        return FALSE;
    }

    /* --- Test 4: WaitAny, none signaled --- */
    KeResetEvent(&events[2]);

    status = KeWaitForMultipleObjects(NUM_EVENTS, waitObjects, WaitAny,
                                      Executive, KernelMode, FALSE,
                                      &zeroTimeout, waitBlocks);
    if (status != STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: WaitAny (none signaled) returned 0x%08lx, expected TIMEOUT\n", status);
        return FALSE;
    }

    /* --- Test 5: WaitAny with sync event (auto-reset) --- */
    {
        KEVENT syncEvents[2];
        PVOID syncWait[2];
        KWAIT_BLOCK syncBlocks[2];

        KeInitializeEvent(&syncEvents[0], SynchronizationEvent, FALSE);
        KeInitializeEvent(&syncEvents[1], SynchronizationEvent, TRUE);  /* signaled */
        syncWait[0] = &syncEvents[0];
        syncWait[1] = &syncEvents[1];

        status = KeWaitForMultipleObjects(2, syncWait, WaitAny,
                                          Executive, KernelMode, FALSE,
                                          &zeroTimeout, syncBlocks);
        if (status != STATUS_WAIT_0 + 1)
        {
            DbgPrint("  FAIL: WaitAny sync returned 0x%08lx, expected WAIT_0+1\n", status);
            return FALSE;
        }

        /* Sync event should auto-reset after wait */
        if (KeReadStateEvent(&syncEvents[1]) != 0)
        {
            DbgPrint("  FAIL: sync event[1] not auto-reset after WaitAny\n");
            return FALSE;
        }
    }

    DbgPrint("  MultiWait: WaitAll + WaitAny + auto-reset passed\n");
    return TRUE;
    #undef NUM_EVENTS
}
