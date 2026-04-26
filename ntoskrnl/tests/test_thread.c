/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — System Thread Creation
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests PsCreateSystemThread — proves ARM64 thread stack setup,
 * context initialization, and scheduler context switch work correctly.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

static KEVENT ThreadEvent;
static volatile LONG ThreadRan;
static volatile LONG ThreadContext;
static volatile HANDLE ThreadPid;

static VOID
NTAPI
TestSystemThread(IN PVOID Context)
{
    if (Context == (PVOID)(ULONG_PTR)0xBAADF00D)
        InterlockedExchange(&ThreadContext, 1);

    ThreadPid = PsGetCurrentProcessId();
    InterlockedIncrement(&ThreadRan);
    KeSetEvent(&ThreadEvent, IO_NO_INCREMENT, FALSE);

    PsTerminateSystemThread(STATUS_SUCCESS);
}

/**
 * NtTestThread - Exercise system thread creation and scheduling.
 *
 * Test 1: Create thread, verify it runs, check context passing.
 * Test 2: Create multiple threads, verify all complete.
 */
BOOLEAN
NTAPI
NtTestThread(VOID)
{
    HANDLE hThread;
    OBJECT_ATTRIBUTES oa;
    NTSTATUS status;
    LARGE_INTEGER timeout;

    /* --- Test 1: Basic system thread --- */
    ThreadRan = 0;
    ThreadContext = 0;
    ThreadPid = 0;
    KeInitializeEvent(&ThreadEvent, NotificationEvent, FALSE);
    InitializeObjectAttributes(&oa, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);

    status = PsCreateSystemThread(&hThread, THREAD_ALL_ACCESS, &oa,
                                  NULL, NULL, TestSystemThread,
                                  (PVOID)(ULONG_PTR)0xBAADF00D);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: PsCreateSystemThread returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Wait for thread to complete */
    timeout.QuadPart = -10 * 10000000LL;  /* 10 sec */
    status = KeWaitForSingleObject(&ThreadEvent, Executive, KernelMode,
                                   FALSE, &timeout);
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: thread did not signal within timeout (0x%08lx)\n", status);
        ZwClose(hThread);
        return FALSE;
    }

    if (ThreadRan != 1)
    {
        DbgPrint("  FAIL: ThreadRan=%ld, expected 1\n", ThreadRan);
        ZwClose(hThread);
        return FALSE;
    }

    if (ThreadContext != 1)
    {
        DbgPrint("  FAIL: ThreadContext=%ld (context not passed)\n", ThreadContext);
        ZwClose(hThread);
        return FALSE;
    }

    /* System threads run in the System process (PID 4) */
    if (ThreadPid != (HANDLE)(ULONG_PTR)4)
    {
        DbgPrint("  INFO: thread PID=%p (expected 4 for System process)\n", ThreadPid);
        /* Non-fatal — PID might differ */
    }

    ZwClose(hThread);

    /* --- Test 2: Multiple threads --- */
    {
        #define NUM_THREADS 4
        HANDLE handles[NUM_THREADS];
        int i;

        /* We reuse the global event for simplicity — use a counter approach */
        ThreadRan = 0;
        KeInitializeEvent(&ThreadEvent, NotificationEvent, FALSE);

        for (i = 0; i < NUM_THREADS; i++)
        {
            status = PsCreateSystemThread(&handles[i], THREAD_ALL_ACCESS, &oa,
                                          NULL, NULL, TestSystemThread,
                                          (PVOID)(ULONG_PTR)0xBAADF00D);
            if (!NT_SUCCESS(status))
            {
                DbgPrint("  FAIL: PsCreateSystemThread[%d] returned 0x%08lx\n", i, status);
                /* Clean up previously created threads */
                while (--i >= 0) ZwClose(handles[i]);
                return FALSE;
            }
        }

        /* Wait for each thread to terminate (thread object becomes signaled) */
        for (i = 0; i < NUM_THREADS; i++)
        {
            timeout.QuadPart = -10 * 10000000LL;  /* 10 sec */
            status = ZwWaitForSingleObject(handles[i], FALSE, &timeout);
            if (status == STATUS_TIMEOUT)
            {
                DbgPrint("  FAIL: thread[%d] did not terminate within timeout\n", i);
                for (i = 0; i < NUM_THREADS; i++) ZwClose(handles[i]);
                return FALSE;
            }
        }

        if (ThreadRan < NUM_THREADS)
        {
            DbgPrint("  FAIL: only %ld of %d threads completed\n", ThreadRan, NUM_THREADS);
            for (i = 0; i < NUM_THREADS; i++) ZwClose(handles[i]);
            return FALSE;
        }

        for (i = 0; i < NUM_THREADS; i++)
            ZwClose(handles[i]);
        #undef NUM_THREADS
    }

    DbgPrint("  Thread: create + context + multi-thread all passed\n");
    return TRUE;
}
