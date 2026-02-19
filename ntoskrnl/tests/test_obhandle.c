/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — object handle operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwCreateEvent / ZwSetEvent / ZwClose to exercise the object
 * manager handle table and event signaling.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestObHandle - Exercise object manager handle table.
 *
 * Creates a notification event, signals it, queries its state,
 * and closes the handle. Returns TRUE if all operations succeed.
 */
BOOLEAN
NTAPI
NtTestObHandle(VOID)
{
    HANDLE hEvent = NULL;
    OBJECT_ATTRIBUTES oa;
    NTSTATUS status;
    EVENT_BASIC_INFORMATION info;
    ULONG returnLength;

    /* Create an unnamed notification event (manual-reset) */
    InitializeObjectAttributes(&oa, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);
    status = ZwCreateEvent(&hEvent, EVENT_ALL_ACCESS, &oa,
                           NotificationEvent, FALSE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateEvent returned 0x%08lx\n", status);
        return FALSE;
    }

    if (hEvent == NULL)
    {
        DbgPrint("  FAIL: ZwCreateEvent succeeded but handle is NULL\n");
        return FALSE;
    }

    /* Query initial state — should be non-signaled (State == 0) */
    status = ZwQueryEvent(hEvent, EventBasicInformation,
                          &info, sizeof(info), &returnLength);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwQueryEvent returned 0x%08lx\n", status);
        ZwClose(hEvent);
        return FALSE;
    }

    if (info.EventState != 0)
    {
        DbgPrint("  FAIL: initial EventState=%ld, expected 0\n", info.EventState);
        ZwClose(hEvent);
        return FALSE;
    }

    /* Signal the event */
    status = ZwSetEvent(hEvent, NULL);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwSetEvent returned 0x%08lx\n", status);
        ZwClose(hEvent);
        return FALSE;
    }

    /* Query again — should be signaled now */
    status = ZwQueryEvent(hEvent, EventBasicInformation,
                          &info, sizeof(info), &returnLength);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwQueryEvent (post-set) returned 0x%08lx\n", status);
        ZwClose(hEvent);
        return FALSE;
    }

    if (info.EventState == 0)
    {
        DbgPrint("  FAIL: EventState still 0 after ZwSetEvent\n");
        ZwClose(hEvent);
        return FALSE;
    }

    /* Close the handle */
    status = ZwClose(hEvent);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwClose returned 0x%08lx\n", status);
        return FALSE;
    }

    DbgPrint("  ObHandle: create/signal/query/close all succeeded\n");
    return TRUE;
}
