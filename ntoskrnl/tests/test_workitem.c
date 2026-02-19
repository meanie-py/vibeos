/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Work Items (kernel worker thread pool)
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ExInitializeWorkItem / ExQueueWorkItem
 * to verify the kernel worker thread pool dispatches correctly.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

static KEVENT WorkItemEvent;
static volatile LONG WorkItemRan;
static volatile LONG WorkItemContext;

/* PWORKER_THREAD_ROUTINE signature: VOID (NTAPI *)(PVOID Parameter) */
static VOID
NTAPI
TestWorkItemRoutine(
    IN PVOID Context)
{
    /* Verify context */
    if (Context == (PVOID)(ULONG_PTR)0xCAFEBABE)
        InterlockedExchange(&WorkItemContext, 1);

    InterlockedIncrement(&WorkItemRan);
    KeSetEvent(&WorkItemEvent, IO_NO_INCREMENT, FALSE);
}

/**
 * NtTestWorkItem - Exercise kernel worker thread pool.
 *
 * Queues work items to DelayedWorkQueue and CriticalWorkQueue,
 * verifies callbacks run with correct context.
 */
BOOLEAN
NTAPI
NtTestWorkItem(VOID)
{
    WORK_QUEUE_ITEM workItem;
    LARGE_INTEGER timeout;
    NTSTATUS status;

    /* --- Test 1: DelayedWorkQueue --- */
    WorkItemRan = 0;
    WorkItemContext = 0;
    KeInitializeEvent(&WorkItemEvent, NotificationEvent, FALSE);

    ExInitializeWorkItem(&workItem,
                         TestWorkItemRoutine,
                         (PVOID)(ULONG_PTR)0xCAFEBABE);

    ExQueueWorkItem(&workItem, DelayedWorkQueue);

    /* Wait up to 10 seconds for work item to execute */
    timeout.QuadPart = -10 * 10000000LL;
    status = KeWaitForSingleObject(&WorkItemEvent, Executive, KernelMode,
                                   FALSE, &timeout);
    if (status == STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: work item did not execute within 10 seconds\n");
        return FALSE;
    }
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: KeWait for work item returned 0x%08lx\n", status);
        return FALSE;
    }

    if (WorkItemRan != 1)
    {
        DbgPrint("  FAIL: WorkItemRan=%ld, expected 1\n", WorkItemRan);
        return FALSE;
    }

    if (WorkItemContext != 1)
    {
        DbgPrint("  FAIL: WorkItemContext=%ld (context not passed correctly)\n",
                 WorkItemContext);
        return FALSE;
    }

    /* --- Test 2: CriticalWorkQueue --- */
    WorkItemRan = 0;
    WorkItemContext = 0;
    KeResetEvent(&WorkItemEvent);

    ExInitializeWorkItem(&workItem,
                         TestWorkItemRoutine,
                         (PVOID)(ULONG_PTR)0xCAFEBABE);

    ExQueueWorkItem(&workItem, CriticalWorkQueue);

    timeout.QuadPart = -10 * 10000000LL;
    status = KeWaitForSingleObject(&WorkItemEvent, Executive, KernelMode,
                                   FALSE, &timeout);
    if (status == STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: critical work item did not execute within 10 seconds\n");
        return FALSE;
    }

    if (WorkItemRan != 1)
    {
        DbgPrint("  FAIL: CriticalWorkQueue WorkItemRan=%ld\n", WorkItemRan);
        return FALSE;
    }

    DbgPrint("  WorkItem: DelayedWorkQueue + CriticalWorkQueue passed\n");
    return TRUE;
}
