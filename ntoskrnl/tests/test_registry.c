/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Registry Operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwCreateKey / ZwSetValueKey / ZwQueryValueKey / ZwDeleteKey
 * to exercise the configuration manager.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/* Test key path under volatile CurrentControlSet (won't persist to hive) */
static const WCHAR TestKeyPath[] =
    L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\ReactOSTestKey";

/**
 * NtTestRegistry - Exercise configuration manager key/value operations.
 *
 * Creates a volatile key, writes REG_DWORD and REG_SZ values,
 * reads them back, deletes the key. Returns TRUE on success.
 */
BOOLEAN
NTAPI
NtTestRegistry(VOID)
{
    UNICODE_STRING keyName;
    OBJECT_ATTRIBUTES oa;
    HANDLE hKey = NULL;
    NTSTATUS status;
    ULONG disposition;
    UNICODE_STRING valueName;
    ULONG testDword = 0x42424242;
    WCHAR testString[] = L"ReactOS ARM64 Test";

    /* Buffer for query results */
    UCHAR queryBuf[256];
    PKEY_VALUE_PARTIAL_INFORMATION pInfo = (PKEY_VALUE_PARTIAL_INFORMATION)queryBuf;
    ULONG resultLength;

    /* --- Create test key (volatile = RAM-only, no hive write) --- */
    RtlInitUnicodeString(&keyName, TestKeyPath);
    InitializeObjectAttributes(&oa, &keyName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               NULL, NULL);

    status = ZwCreateKey(&hKey, KEY_ALL_ACCESS, &oa, 0, NULL,
                         REG_OPTION_VOLATILE, &disposition);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateKey returned 0x%08lx\n", status);
        return FALSE;
    }

    if (disposition != REG_CREATED_NEW_KEY && disposition != REG_OPENED_EXISTING_KEY)
    {
        DbgPrint("  FAIL: unexpected disposition %lu\n", disposition);
        ZwClose(hKey);
        return FALSE;
    }

    /* --- Write REG_DWORD value --- */
    RtlInitUnicodeString(&valueName, L"TestDword");
    status = ZwSetValueKey(hKey, &valueName, 0, REG_DWORD,
                           &testDword, sizeof(testDword));
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwSetValueKey(DWORD) returned 0x%08lx\n", status);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    /* --- Read REG_DWORD value back --- */
    status = ZwQueryValueKey(hKey, &valueName, KeyValuePartialInformation,
                             queryBuf, sizeof(queryBuf), &resultLength);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwQueryValueKey(DWORD) returned 0x%08lx\n", status);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    if (pInfo->Type != REG_DWORD || pInfo->DataLength != sizeof(ULONG))
    {
        DbgPrint("  FAIL: DWORD type=%lu len=%lu\n", pInfo->Type, pInfo->DataLength);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    if (*(PULONG)pInfo->Data != testDword)
    {
        DbgPrint("  FAIL: DWORD value=0x%08lx, expected 0x%08lx\n",
                 *(PULONG)pInfo->Data, testDword);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    /* --- Write REG_SZ value --- */
    RtlInitUnicodeString(&valueName, L"TestString");
    status = ZwSetValueKey(hKey, &valueName, 0, REG_SZ,
                           testString, sizeof(testString));
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwSetValueKey(SZ) returned 0x%08lx\n", status);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    /* --- Read REG_SZ value back --- */
    status = ZwQueryValueKey(hKey, &valueName, KeyValuePartialInformation,
                             queryBuf, sizeof(queryBuf), &resultLength);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwQueryValueKey(SZ) returned 0x%08lx\n", status);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    if (pInfo->Type != REG_SZ)
    {
        DbgPrint("  FAIL: SZ type=%lu\n", pInfo->Type);
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    if (RtlCompareMemory(pInfo->Data, testString, sizeof(testString)) != sizeof(testString))
    {
        DbgPrint("  FAIL: SZ value mismatch\n");
        ZwDeleteKey(hKey);
        ZwClose(hKey);
        return FALSE;
    }

    /* --- Delete the key --- */
    status = ZwDeleteKey(hKey);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwDeleteKey returned 0x%08lx\n", status);
        ZwClose(hKey);
        return FALSE;
    }

    ZwClose(hKey);

    /* --- Verify key is gone --- */
    status = ZwOpenKey(&hKey, KEY_READ, &oa);
    if (NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: key still exists after ZwDeleteKey\n");
        ZwClose(hKey);
        return FALSE;
    }

    DbgPrint("  Registry: create/set/query/delete key+values passed\n");
    return TRUE;
}
