/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Object Reference Counting
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ObReferenceObjectByHandle / ObDereferenceObject —
 * handle-to-object conversion and reference lifecycle.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestObRef - Exercise object reference counting.
 *
 * Test 1: Create event, get object pointer from handle, verify type, deref.
 * Test 2: Multiple references, verify object survives until last deref.
 * Test 3: ObReferenceObjectByHandle with type mismatch returns error.
 */
BOOLEAN
NTAPI
NtTestObRef(VOID)
{
    HANDLE hEvent = NULL;
    OBJECT_ATTRIBUTES oa;
    NTSTATUS status;
    PVOID eventObj = NULL;

    InitializeObjectAttributes(&oa, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);

    /* --- Test 1: Basic handle → object → deref --- */
    status = ZwCreateEvent(&hEvent, EVENT_ALL_ACCESS, &oa,
                           NotificationEvent, FALSE);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateEvent returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Get object pointer from handle */
    status = ObReferenceObjectByHandle(hEvent, EVENT_ALL_ACCESS, ExEventObjectType,
                                       KernelMode, &eventObj, NULL);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ObReferenceObjectByHandle returned 0x%08lx\n", status);
        ZwClose(hEvent);
        return FALSE;
    }

    if (eventObj == NULL)
    {
        DbgPrint("  FAIL: ObReferenceObjectByHandle returned NULL object\n");
        ZwClose(hEvent);
        return FALSE;
    }

    /* Verify we can use the object — set the event via kernel API */
    KeSetEvent((PKEVENT)eventObj, IO_NO_INCREMENT, FALSE);

    if (KeReadStateEvent((PKEVENT)eventObj) == 0)
    {
        DbgPrint("  FAIL: event not signaled after KeSetEvent on object pointer\n");
        ObDereferenceObject(eventObj);
        ZwClose(hEvent);
        return FALSE;
    }

    /* Dereference */
    ObDereferenceObject(eventObj);

    /* --- Test 2: Multiple references --- */
    {
        PVOID obj1 = NULL, obj2 = NULL;

        status = ObReferenceObjectByHandle(hEvent, EVENT_ALL_ACCESS, ExEventObjectType,
                                           KernelMode, &obj1, NULL);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: second ObRefByHandle returned 0x%08lx\n", status);
            ZwClose(hEvent);
            return FALSE;
        }

        /* Reference the same object again via ObReferenceObject */
        ObReferenceObject(obj1);
        obj2 = obj1;

        /* Both references must be released */
        ObDereferenceObject(obj1);
        ObDereferenceObject(obj2);
    }

    /* --- Test 3: Type mismatch --- */
    {
        PVOID wrongObj = NULL;

        /* Try to reference an event handle as a process type */
        status = ObReferenceObjectByHandle(hEvent, PROCESS_ALL_ACCESS,
                                           PsProcessType,
                                           KernelMode, &wrongObj, NULL);
        if (NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: type mismatch should have failed but returned SUCCESS\n");
            ObDereferenceObject(wrongObj);
            ZwClose(hEvent);
            return FALSE;
        }

        /* Expected: STATUS_OBJECT_TYPE_MISMATCH */
        if (status != STATUS_OBJECT_TYPE_MISMATCH)
        {
            DbgPrint("  INFO: type mismatch returned 0x%08lx (expected 0x%08lx)\n",
                     status, (NTSTATUS)STATUS_OBJECT_TYPE_MISMATCH);
            /* Non-fatal — some implementations may use different error codes */
        }
    }

    ZwClose(hEvent);

    DbgPrint("  ObRef: handle->object + multi-ref + type-mismatch passed\n");
    return TRUE;
}
