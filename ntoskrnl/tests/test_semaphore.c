/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Kernel Semaphore
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeInitializeSemaphore / KeReleaseSemaphore / KeReadStateSemaphore
 * and ZwCreateSemaphore / ZwReleaseSemaphore handle API.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestSemaphore - Exercise kernel semaphore operations.
 *
 * Test 1: Init with count=2, acquire twice, third acquire should timeout.
 * Test 2: Release increments count.
 * Test 3: Handle-based API via Zw functions.
 */
BOOLEAN
NTAPI
NtTestSemaphore(VOID)
{
    KSEMAPHORE sem;
    NTSTATUS status;
    LARGE_INTEGER zeroTimeout;

    zeroTimeout.QuadPart = 0;

    /* --- Test 1: Basic counting semaphore --- */
    KeInitializeSemaphore(&sem, 2, 5);  /* count=2, limit=5 */

    if (KeReadStateSemaphore(&sem) != 2)
    {
        DbgPrint("  FAIL: initial semaphore count=%ld, expected 2\n",
                 KeReadStateSemaphore(&sem));
        return FALSE;
    }

    /* First acquire (count 2->1) */
    status = KeWaitForSingleObject(&sem, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: first semaphore acquire returned 0x%08lx\n", status);
        return FALSE;
    }

    if (KeReadStateSemaphore(&sem) != 1)
    {
        DbgPrint("  FAIL: count=%ld after first acquire, expected 1\n",
                 KeReadStateSemaphore(&sem));
        return FALSE;
    }

    /* Second acquire (count 1->0) */
    status = KeWaitForSingleObject(&sem, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: second semaphore acquire returned 0x%08lx\n", status);
        return FALSE;
    }

    if (KeReadStateSemaphore(&sem) != 0)
    {
        DbgPrint("  FAIL: count=%ld after second acquire, expected 0\n",
                 KeReadStateSemaphore(&sem));
        return FALSE;
    }

    /* Third acquire should timeout (count is 0) */
    status = KeWaitForSingleObject(&sem, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: third acquire returned 0x%08lx, expected STATUS_TIMEOUT\n", status);
        return FALSE;
    }

    /* --- Test 2: Release increments count --- */
    KeReleaseSemaphore(&sem, IO_NO_INCREMENT, 1, FALSE);  /* count 0->1 */

    if (KeReadStateSemaphore(&sem) != 1)
    {
        DbgPrint("  FAIL: count=%ld after release(1), expected 1\n",
                 KeReadStateSemaphore(&sem));
        return FALSE;
    }

    /* Release by 2 (count 1->3) */
    KeReleaseSemaphore(&sem, IO_NO_INCREMENT, 2, FALSE);

    if (KeReadStateSemaphore(&sem) != 3)
    {
        DbgPrint("  FAIL: count=%ld after release(2), expected 3\n",
                 KeReadStateSemaphore(&sem));
        return FALSE;
    }

    /* --- Test 3: Handle-based semaphore --- */
    {
        HANDLE hSem = NULL;
        OBJECT_ATTRIBUTES oa;
        LONG prevCount;

        InitializeObjectAttributes(&oa, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);
        status = ZwCreateSemaphore(&hSem, SEMAPHORE_ALL_ACCESS, &oa, 1, 10);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwCreateSemaphore returned 0x%08lx\n", status);
            return FALSE;
        }

        /* Acquire (count 1->0) */
        status = ZwWaitForSingleObject(hSem, FALSE, &zeroTimeout);
        if (status != STATUS_SUCCESS)
        {
            DbgPrint("  FAIL: ZwWait on semaphore returned 0x%08lx\n", status);
            ZwClose(hSem);
            return FALSE;
        }

        /* Release (count 0->1), get previous count */
        status = ZwReleaseSemaphore(hSem, 1, &prevCount);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwReleaseSemaphore returned 0x%08lx\n", status);
            ZwClose(hSem);
            return FALSE;
        }

        if (prevCount != 0)
        {
            DbgPrint("  FAIL: prevCount=%ld, expected 0\n", prevCount);
            ZwClose(hSem);
            return FALSE;
        }

        ZwClose(hSem);
    }

    DbgPrint("  Semaphore: counting + release + handle API passed\n");
    return TRUE;
}
