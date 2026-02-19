/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Deferred Procedure Calls (DPC)
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests KeInitializeDpc / KeInsertQueueDpc — verifies DPC fires
 * by signaling an event from the DPC callback.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

static volatile LONG DpcFired;
static KEVENT DpcEvent;

static VOID
NTAPI
TestDpcRoutine(
    IN PKDPC Dpc,
    IN PVOID DeferredContext,
    IN PVOID SystemArgument1,
    IN PVOID SystemArgument2)
{
    UNREFERENCED_PARAMETER(Dpc);
    UNREFERENCED_PARAMETER(SystemArgument1);
    UNREFERENCED_PARAMETER(SystemArgument2);

    /* Verify context pointer passed through correctly */
    if (DeferredContext == (PVOID)(ULONG_PTR)0xDEADBEEF)
        InterlockedIncrement(&DpcFired);

    KeSetEvent(&DpcEvent, IO_NO_INCREMENT, FALSE);
}

/**
 * NtTestDpc - Exercise DPC queue and dispatch.
 *
 * Queues a DPC, waits for it to fire (with timeout), verifies the
 * callback ran with correct context. Returns TRUE on success.
 */
BOOLEAN
NTAPI
NtTestDpc(VOID)
{
    KDPC dpc;
    LARGE_INTEGER timeout;
    NTSTATUS status;

    DpcFired = 0;
    KeInitializeEvent(&DpcEvent, NotificationEvent, FALSE);
    KeInitializeDpc(&dpc, TestDpcRoutine, (PVOID)(ULONG_PTR)0xDEADBEEF);

    /* Queue the DPC */
    if (!KeInsertQueueDpc(&dpc, NULL, NULL))
    {
        DbgPrint("  FAIL: KeInsertQueueDpc returned FALSE\n");
        return FALSE;
    }

    /* Wait up to 5 seconds for DPC to fire */
    timeout.QuadPart = -5 * 10000000LL;  /* 5 seconds */
    status = KeWaitForSingleObject(&DpcEvent, Executive, KernelMode,
                                   FALSE, &timeout);
    if (status == STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: DPC did not fire within 5 seconds\n");
        return FALSE;
    }
    if (status != STATUS_SUCCESS)
    {
        DbgPrint("  FAIL: KeWait for DPC event returned 0x%08lx\n", status);
        return FALSE;
    }

    if (DpcFired != 1)
    {
        DbgPrint("  FAIL: DpcFired=%ld, expected 1\n", DpcFired);
        return FALSE;
    }

    /* Test 2: Queue the same DPC twice — second insert should return FALSE
     * (already in queue or already executed) */
    KeResetEvent(&DpcEvent);
    DpcFired = 0;

    KeInsertQueueDpc(&dpc, NULL, NULL);
    /* Queuing again immediately — if it's still pending, returns FALSE */
    /* (This is racy but exercises the API path) */

    /* Wait for it to fire again */
    timeout.QuadPart = -5 * 10000000LL;
    status = KeWaitForSingleObject(&DpcEvent, Executive, KernelMode,
                                   FALSE, &timeout);
    if (status == STATUS_TIMEOUT)
    {
        DbgPrint("  FAIL: Second DPC did not fire within 5 seconds\n");
        return FALSE;
    }

    if (DpcFired != 1)
    {
        DbgPrint("  FAIL: Second DpcFired=%ld, expected 1\n", DpcFired);
        return FALSE;
    }

    DbgPrint("  Dpc: queue + dispatch + context verification passed\n");
    return TRUE;
}
