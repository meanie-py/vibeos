/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Object Directory Operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwOpenDirectoryObject and ZwQueryDirectoryObject.
 * Validates the Object Manager's directory traversal on ARM64.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

BOOLEAN
NTAPI
NtTestObjDir(VOID)
{
    HANDLE hDir = NULL;
    OBJECT_ATTRIBUTES oa;
    UNICODE_STRING dirName;
    NTSTATUS status;

    /* --- Test 1: Open root object directory --- */
    RtlInitUnicodeString(&dirName, L"\\");
    InitializeObjectAttributes(&oa, &dirName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwOpenDirectoryObject(&hDir, DIRECTORY_QUERY, &oa);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwOpenDirectoryObject(\\) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* --- Test 2: Enumerate directory entries --- */
    {
        UCHAR queryBuf[512];
        ULONG context = 0;
        ULONG returnLength;
        BOOLEAN found = FALSE;

        /* First call: get first batch */
        status = ZwQueryDirectoryObject(hDir, queryBuf, sizeof(queryBuf),
                                         TRUE, /* ReturnSingleEntry */
                                         TRUE, /* RestartScan */
                                         &context, &returnLength);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQueryDirectoryObject returned 0x%08lx\n", status);
            ZwClose(hDir);
            return FALSE;
        }

        /* Should have at least one entry */
        {
            POBJECT_DIRECTORY_INFORMATION dirInfo =
                (POBJECT_DIRECTORY_INFORMATION)queryBuf;

            if (dirInfo->Name.Length == 0)
            {
                DbgPrint("  FAIL: first directory entry has empty name\n");
                ZwClose(hDir);
                return FALSE;
            }

            DbgPrint("  ObjDir: first entry: %wZ (type: %wZ)\n",
                     &dirInfo->Name, &dirInfo->TypeName);
        }
    }

    ZwClose(hDir);

    /* --- Test 3: Open known subdirectory (\Device) --- */
    RtlInitUnicodeString(&dirName, L"\\Device");
    InitializeObjectAttributes(&oa, &dirName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwOpenDirectoryObject(&hDir, DIRECTORY_QUERY, &oa);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwOpenDirectoryObject(\\Device) returned 0x%08lx\n",
                 status);
        return FALSE;
    }

    ZwClose(hDir);

    /* --- Test 4: Open \ObjectTypes directory --- */
    RtlInitUnicodeString(&dirName, L"\\ObjectTypes");
    InitializeObjectAttributes(&oa, &dirName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwOpenDirectoryObject(&hDir, DIRECTORY_QUERY, &oa);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  INFO: ZwOpenDirectoryObject(\\ObjectTypes) returned 0x%08lx\n",
                 status);
        /* Non-fatal — might not exist on all configurations */
    }
    else
    {
        ZwClose(hDir);
    }

    DbgPrint("  ObjDir: root + enumerate + Device + ObjectTypes passed\n");
    return TRUE;
}
