/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Kernel Mutex
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeInitializeMutex / KeWaitForSingleObject / KeReleaseMutex.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestMutex - Exercise kernel mutex operations.
 *
 * Test 1: Init + acquire + release on single thread.
 * Test 2: Recursive acquisition (same thread can acquire multiple times).
 * Test 3: Mutex via Zw handle API.
 */
BOOLEAN
NTAPI
NtTestMutex(VOID)
{
    KMUTEX mutex;
    NTSTATUS status;
    LARGE_INTEGER zeroTimeout;

    zeroTimeout.QuadPart = 0;

    /* --- Test 1: Basic acquire/release --- */
    KeInitializeMutex(&mutex, 0);

    /* Mutex starts signaled (available) */
    if (mutex.Header.SignalState != 1)
    {
        DbgPrint("  FAIL: initial mutex state=%ld, expected 1 (signaled)\n",
                 mutex.Header.SignalState);
        return FALSE;
    }

    /* Acquire (should succeed immediately) */
    status = KeWaitForSingleObject(&mutex, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: KeWait on available mutex returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Mutex should now be non-signaled (held) */
    if (mutex.Header.SignalState >= 1)
    {
        DbgPrint("  FAIL: mutex state=%ld after acquire, expected <= 0\n",
                 mutex.Header.SignalState);
        KeReleaseMutex(&mutex, FALSE);
        return FALSE;
    }

    /* Release */
    KeReleaseMutex(&mutex, FALSE);

    /* Should be signaled again */
    if (mutex.Header.SignalState != 1)
    {
        DbgPrint("  FAIL: mutex state=%ld after release, expected 1\n",
                 mutex.Header.SignalState);
        return FALSE;
    }

    /* --- Test 2: Recursive acquisition --- */
    status = KeWaitForSingleObject(&mutex, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: first recursive acquire returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Same thread acquires again (recursive) */
    status = KeWaitForSingleObject(&mutex, Executive, KernelMode,
                                   FALSE, &zeroTimeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: second recursive acquire returned 0x%08lx\n", status);
        KeReleaseMutex(&mutex, FALSE);
        return FALSE;
    }

    /* Must release twice */
    KeReleaseMutex(&mutex, FALSE);
    KeReleaseMutex(&mutex, FALSE);

    if (mutex.Header.SignalState != 1)
    {
        DbgPrint("  FAIL: mutex state=%ld after double release, expected 1\n",
                 mutex.Header.SignalState);
        return FALSE;
    }

    /* --- Test 3: Mutex via handle --- */
    {
        HANDLE hMutant = NULL;
        OBJECT_ATTRIBUTES oa;

        InitializeObjectAttributes(&oa, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);
        status = ZwCreateMutant(&hMutant, MUTANT_ALL_ACCESS, &oa, FALSE);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwCreateMutant returned 0x%08lx\n", status);
            return FALSE;
        }

        /* Wait (acquire) */
        status = ZwWaitForSingleObject(hMutant, FALSE, &zeroTimeout);
        if (status != STATUS_SUCCESS)
        {
            DbgPrint("  FAIL: ZwWait on mutant returned 0x%08lx\n", status);
            ZwClose(hMutant);
            return FALSE;
        }

        /* Release */
        status = ZwReleaseMutant(hMutant, NULL);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwReleaseMutant returned 0x%08lx\n", status);
            ZwClose(hMutant);
            return FALSE;
        }

        ZwClose(hMutant);
    }

    DbgPrint("  Mutex: acquire/release + recursive + handle API passed\n");
    return TRUE;
}
