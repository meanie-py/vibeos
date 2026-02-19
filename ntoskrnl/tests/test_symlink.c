/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Symbolic Link Objects
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwCreateSymbolicLinkObject / ZwQuerySymbolicLinkObject.
 * Validates OB link creation and query on ARM64.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

BOOLEAN
NTAPI
NtTestSymLink(VOID)
{
    HANDLE hLink = NULL;
    OBJECT_ATTRIBUTES oa;
    UNICODE_STRING linkName;
    UNICODE_STRING targetName;
    NTSTATUS status;

    /* --- Test 1: Create a symbolic link and query it (same handle) --- */
    RtlInitUnicodeString(&linkName, L"\\??\\TestKernelLink");
    RtlInitUnicodeString(&targetName, L"\\Device");
    InitializeObjectAttributes(&oa, &linkName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwCreateSymbolicLinkObject(&hLink, SYMBOLIC_LINK_ALL_ACCESS,
                                         &oa, &targetName);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateSymbolicLinkObject returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Query the target using the create handle */
    {
        WCHAR queryBuf[128];
        UNICODE_STRING queryStr;
        ULONG returnLength;

        queryStr.Buffer = queryBuf;
        queryStr.Length = 0;
        queryStr.MaximumLength = sizeof(queryBuf);

        status = ZwQuerySymbolicLinkObject(hLink, &queryStr, &returnLength);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQuerySymbolicLinkObject returned 0x%08lx\n", status);
            ZwClose(hLink);
            return FALSE;
        }

        /* Verify target matches */
        if (!RtlEqualUnicodeString(&queryStr, &targetName, TRUE))
        {
            DbgPrint("  FAIL: symlink target = '%wZ', expected '%wZ'\n",
                     &queryStr, &targetName);
            ZwClose(hLink);
            return FALSE;
        }
    }

    ZwClose(hLink);
    hLink = NULL;

    /* --- Test 2: Create a permanent symlink, close, re-open, query --- */
    RtlInitUnicodeString(&linkName, L"\\??\\TestKernelLink2");
    RtlInitUnicodeString(&targetName, L"\\ObjectTypes");
    InitializeObjectAttributes(&oa, &linkName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE | OBJ_PERMANENT,
        NULL, NULL);

    status = ZwCreateSymbolicLinkObject(&hLink, SYMBOLIC_LINK_ALL_ACCESS,
                                         &oa, &targetName);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateSymbolicLinkObject(permanent) returned 0x%08lx\n",
                 status);
        return FALSE;
    }

    ZwClose(hLink);
    hLink = NULL;

    /* Re-open the permanent symlink */
    InitializeObjectAttributes(&oa, &linkName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwOpenSymbolicLinkObject(&hLink, SYMBOLIC_LINK_QUERY, &oa);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwOpenSymbolicLinkObject(permanent) returned 0x%08lx\n",
                 status);
        return FALSE;
    }

    /* Query and verify */
    {
        WCHAR queryBuf[128];
        UNICODE_STRING queryStr;
        ULONG returnLength;

        queryStr.Buffer = queryBuf;
        queryStr.Length = 0;
        queryStr.MaximumLength = sizeof(queryBuf);

        status = ZwQuerySymbolicLinkObject(hLink, &queryStr, &returnLength);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQuerySymbolicLinkObject(reopen) returned 0x%08lx\n",
                     status);
            ZwClose(hLink);
            return FALSE;
        }

        if (!RtlEqualUnicodeString(&queryStr, &targetName, TRUE))
        {
            DbgPrint("  FAIL: permanent symlink target = '%wZ', expected '%wZ'\n",
                     &queryStr, &targetName);
            ZwClose(hLink);
            return FALSE;
        }
    }

    /* Make non-permanent so it gets cleaned up */
    ZwMakeTemporaryObject(hLink);
    ZwClose(hLink);

    /* --- Test 3: Open an existing system symlink (\DosDevices) --- */
    RtlInitUnicodeString(&linkName, L"\\DosDevices");
    InitializeObjectAttributes(&oa, &linkName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwOpenSymbolicLinkObject(&hLink, SYMBOLIC_LINK_QUERY, &oa);
    if (NT_SUCCESS(status))
    {
        WCHAR queryBuf[128];
        UNICODE_STRING queryStr;
        ULONG returnLength;

        queryStr.Buffer = queryBuf;
        queryStr.Length = 0;
        queryStr.MaximumLength = sizeof(queryBuf);

        status = ZwQuerySymbolicLinkObject(hLink, &queryStr, &returnLength);
        if (NT_SUCCESS(status))
        {
            DbgPrint("  SymLink: \\DosDevices -> '%wZ'\n", &queryStr);
        }
        ZwClose(hLink);
    }
    else
    {
        DbgPrint("  INFO: \\DosDevices not a symlink (0x%08lx)\n", status);
    }

    DbgPrint("  SymLink: create + query + permanent + reopen + system passed\n");
    return TRUE;
}
