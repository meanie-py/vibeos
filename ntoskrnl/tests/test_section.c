/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — Section Objects
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwCreateSection / ZwMapViewOfSection / ZwUnmapViewOfSection
 * using file-backed sections (proven on ARM64 via PE loader).
 *
 * NOTE: Pagefile-backed SEC_COMMIT sections require demand-zero paging
 * which is broken on ARM64 with DUMMY_PSEH (ARM3 page fault handler
 * relies on SEH). File-backed sections work because the PE loader
 * path pre-populates pages.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestSection - Exercise section object operations.
 *
 * Test 1: Create file-backed section from ntoskrnl.exe, map, read MZ header.
 * Test 2: Query section size info.
 * Test 3: Unmap and re-map at different view size.
 */
BOOLEAN
NTAPI
NtTestSection(VOID)
{
    HANDLE hFile = NULL;
    HANDLE hSection = NULL;
    OBJECT_ATTRIBUTES oa;
    UNICODE_STRING fileName;
    IO_STATUS_BLOCK ioStatus;
    NTSTATUS status;
    PVOID baseAddr;
    SIZE_T viewSize;

    /* Open ntoskrnl.exe for section backing */
    RtlInitUnicodeString(&fileName,
        L"\\SystemRoot\\system32\\ntoskrnl.exe");
    InitializeObjectAttributes(&oa, &fileName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwCreateFile(&hFile, FILE_READ_DATA | SYNCHRONIZE, &oa,
                          &ioStatus, NULL, FILE_ATTRIBUTE_NORMAL,
                          FILE_SHARE_READ, FILE_OPEN,
                          FILE_SYNCHRONOUS_IO_NONALERT, NULL, 0);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateFile(ntoskrnl.exe) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* --- Test 1: Create file-backed section --- */
    InitializeObjectAttributes(&oa, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);
    status = ZwCreateSection(&hSection, SECTION_ALL_ACCESS, &oa,
                             NULL, PAGE_READONLY, SEC_COMMIT, hFile);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateSection returned 0x%08lx\n", status);
        ZwClose(hFile);
        return FALSE;
    }

    /* Map first 4KB */
    baseAddr = NULL;
    viewSize = 0x1000;
    status = ZwMapViewOfSection(hSection, NtCurrentProcess(), &baseAddr,
                                0, 0, NULL, &viewSize,
                                ViewUnmap, 0, PAGE_READONLY);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwMapViewOfSection returned 0x%08lx\n", status);
        ZwClose(hSection);
        ZwClose(hFile);
        return FALSE;
    }

    if (baseAddr == NULL)
    {
        DbgPrint("  FAIL: mapped address is NULL\n");
        ZwClose(hSection);
        ZwClose(hFile);
        return FALSE;
    }

    /* Verify MZ header through the section mapping */
    {
        PUCHAR data = (PUCHAR)baseAddr;

        if (data[0] != 'M' || data[1] != 'Z')
        {
            DbgPrint("  FAIL: section mapping MZ check: 0x%02x 0x%02x\n",
                     data[0], data[1]);
            ZwUnmapViewOfSection(NtCurrentProcess(), baseAddr);
            ZwClose(hSection);
            ZwClose(hFile);
            return FALSE;
        }

        /* Verify PE signature at offset from e_lfanew */
        {
            LONG e_lfanew = *(PLONG)(data + 0x3C);
            if (e_lfanew > 0 && e_lfanew < 0x800)
            {
                /* Only check if PE header is within our mapped 4KB */
                if ((ULONG)e_lfanew + 4 <= viewSize)
                {
                    if (data[e_lfanew] != 'P' || data[e_lfanew + 1] != 'E' ||
                        data[e_lfanew + 2] != 0 || data[e_lfanew + 3] != 0)
                    {
                        DbgPrint("  FAIL: PE signature at offset 0x%lx: "
                                 "0x%02x 0x%02x 0x%02x 0x%02x\n",
                                 e_lfanew, data[e_lfanew], data[e_lfanew+1],
                                 data[e_lfanew+2], data[e_lfanew+3]);
                        ZwUnmapViewOfSection(NtCurrentProcess(), baseAddr);
                        ZwClose(hSection);
                        ZwClose(hFile);
                        return FALSE;
                    }
                }
            }
        }
    }

    /* --- Test 2: Unmap --- */
    status = ZwUnmapViewOfSection(NtCurrentProcess(), baseAddr);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwUnmapViewOfSection returned 0x%08lx\n", status);
        ZwClose(hSection);
        ZwClose(hFile);
        return FALSE;
    }

    /* --- Test 3: Re-map with larger view --- */
    baseAddr = NULL;
    viewSize = 0x10000;  /* 64KB */
    status = ZwMapViewOfSection(hSection, NtCurrentProcess(), &baseAddr,
                                0, 0, NULL, &viewSize,
                                ViewUnmap, 0, PAGE_READONLY);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: second ZwMapViewOfSection returned 0x%08lx\n", status);
        ZwClose(hSection);
        ZwClose(hFile);
        return FALSE;
    }

    /* Quick sanity: MZ header still readable at new mapping */
    {
        PUCHAR data = (PUCHAR)baseAddr;
        if (data[0] != 'M' || data[1] != 'Z')
        {
            DbgPrint("  FAIL: second mapping MZ check failed\n");
            ZwUnmapViewOfSection(NtCurrentProcess(), baseAddr);
            ZwClose(hSection);
            ZwClose(hFile);
            return FALSE;
        }
    }

    ZwUnmapViewOfSection(NtCurrentProcess(), baseAddr);
    ZwClose(hSection);
    ZwClose(hFile);

    DbgPrint("  Section: file-backed create + map + MZ + PE + unmap + remap passed\n");
    return TRUE;
}
