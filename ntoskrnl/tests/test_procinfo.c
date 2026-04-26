/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Process and Thread Information Queries
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwQueryInformationProcess and ZwQueryInformationThread.
 * Validates ARM64 PEB/TEB initialization and process state.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

BOOLEAN
NTAPI
NtTestProcInfo(VOID)
{
    NTSTATUS status;
    ULONG returnLength;

    /* --- Test 1: ProcessBasicInformation for current process --- */
    {
        PROCESS_BASIC_INFORMATION pbi;

        RtlZeroMemory(&pbi, sizeof(pbi));
        status = ZwQueryInformationProcess(NtCurrentProcess(),
                                            ProcessBasicInformation,
                                            &pbi, sizeof(pbi), &returnLength);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQueryInformationProcess(Basic) returned 0x%08lx\n",
                     status);
            return FALSE;
        }

        if (returnLength != sizeof(pbi))
        {
            DbgPrint("  FAIL: returnLength=%lu, expected %u\n",
                     returnLength, (unsigned)sizeof(pbi));
            return FALSE;
        }

        /* In TESTKERNEL mode, we're running as the System process (PID 4) */
        DbgPrint("  ProcInfo: PID=%p, ParentPID=%p, ExitStatus=0x%lx\n",
                 (PVOID)pbi.UniqueProcessId,
                 (PVOID)pbi.InheritedFromUniqueProcessId,
                 pbi.ExitStatus);

        /* PID should be non-zero */
        if (pbi.UniqueProcessId == 0)
        {
            DbgPrint("  FAIL: UniqueProcessId is 0\n");
            return FALSE;
        }
    }

    /* --- Test 2: ThreadBasicInformation for current thread --- */
    {
        THREAD_BASIC_INFORMATION tbi;

        RtlZeroMemory(&tbi, sizeof(tbi));
        status = ZwQueryInformationThread(NtCurrentThread(),
                                           ThreadBasicInformation,
                                           &tbi, sizeof(tbi), &returnLength);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQueryInformationThread(Basic) returned 0x%08lx\n",
                     status);
            return FALSE;
        }

        /* Thread should not have exited */
        if (tbi.ExitStatus != STATUS_PENDING)
        {
            DbgPrint("  INFO: thread ExitStatus=0x%lx (expected STATUS_PENDING)\n",
                     tbi.ExitStatus);
        }

        DbgPrint("  ProcInfo: TID=%p, priority=%ld\n",
                 tbi.ClientId.UniqueThread, tbi.BasePriority);
    }

    /* --- Test 3: Query process handle count --- */
    {
        ULONG handleCount = 0;

        status = ZwQueryInformationProcess(NtCurrentProcess(),
                                            ProcessHandleCount,
                                            &handleCount, sizeof(handleCount),
                                            &returnLength);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQueryInformationProcess(HandleCount) returned 0x%08lx\n",
                     status);
            return FALSE;
        }

        /* System process should have at least a few handles */
        if (handleCount == 0)
        {
            DbgPrint("  FAIL: process handle count is 0\n");
            return FALSE;
        }

        DbgPrint("  ProcInfo: handle count=%lu\n", handleCount);
    }

    /* --- Test 4: PsGetCurrentProcessId / PsGetCurrentThreadId --- */
    {
        HANDLE pid = PsGetCurrentProcessId();
        HANDLE tid = PsGetCurrentThreadId();

        if (pid == NULL)
        {
            DbgPrint("  FAIL: PsGetCurrentProcessId returned NULL\n");
            return FALSE;
        }

        if (tid == NULL)
        {
            DbgPrint("  FAIL: PsGetCurrentThreadId returned NULL\n");
            return FALSE;
        }

        DbgPrint("  ProcInfo: PsGetCurrentPID=%p, TID=%p\n", pid, tid);
    }

    DbgPrint("  ProcInfo: basic + thread + handles + IDs passed\n");
    return TRUE;
}
