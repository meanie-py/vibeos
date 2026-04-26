/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — File I/O Operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests ZwCreateFile / ZwReadFile / ZwWriteFile / ZwQueryInformationFile
 * on the ramdisk filesystem. Proves the I/O manager, filesystem driver,
 * and ramdisk are all functioning on ARM64.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestFileIo - Exercise file I/O on the ramdisk.
 *
 * Test 1: Open an existing file (ntoskrnl.exe), read first bytes, verify MZ header.
 * Test 2: Create a new file, write data, read it back, delete.
 * Test 3: Query file size information.
 */
BOOLEAN
NTAPI
NtTestFileIo(VOID)
{
    UNICODE_STRING fileName;
    OBJECT_ATTRIBUTES oa;
    HANDLE hFile = NULL;
    IO_STATUS_BLOCK ioStatus;
    NTSTATUS status;
    UCHAR readBuf[64];

    /* --- Test 1: Read existing file (ntoskrnl.exe MZ header) --- */
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

    /* Read first 64 bytes */
    RtlZeroMemory(readBuf, sizeof(readBuf));
    status = ZwReadFile(hFile, NULL, NULL, NULL, &ioStatus,
                        readBuf, sizeof(readBuf), NULL, NULL);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwReadFile returned 0x%08lx\n", status);
        ZwClose(hFile);
        return FALSE;
    }

    /* Verify PE/MZ signature */
    if (readBuf[0] != 'M' || readBuf[1] != 'Z')
    {
        DbgPrint("  FAIL: expected MZ header, got 0x%02x 0x%02x\n",
                 readBuf[0], readBuf[1]);
        ZwClose(hFile);
        return FALSE;
    }

    /* --- Test 3: Query file size --- */
    {
        FILE_STANDARD_INFORMATION stdInfo;

        status = ZwQueryInformationFile(hFile, &ioStatus, &stdInfo,
                                        sizeof(stdInfo), FileStandardInformation);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwQueryInformationFile returned 0x%08lx\n", status);
            ZwClose(hFile);
            return FALSE;
        }

        /* ntoskrnl.exe should be at least 1MB */
        if (stdInfo.EndOfFile.QuadPart < 1024 * 1024)
        {
            DbgPrint("  FAIL: ntoskrnl.exe size=%lld, expected >= 1MB\n",
                     stdInfo.EndOfFile.QuadPart);
            ZwClose(hFile);
            return FALSE;
        }

        DbgPrint("  FileIo: ntoskrnl.exe size=%lld bytes\n", stdInfo.EndOfFile.QuadPart);
    }

    ZwClose(hFile);

    /* --- Test 2: Create, write, read back, delete --- */
    RtlInitUnicodeString(&fileName,
        L"\\SystemRoot\\system32\\test_kernel_io.tmp");
    InitializeObjectAttributes(&oa, &fileName,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, NULL);

    status = ZwCreateFile(&hFile,
                          FILE_GENERIC_READ | FILE_GENERIC_WRITE | DELETE | SYNCHRONIZE,
                          &oa, &ioStatus, NULL,
                          FILE_ATTRIBUTE_NORMAL, 0, FILE_OVERWRITE_IF,
                          FILE_SYNCHRONOUS_IO_NONALERT | FILE_DELETE_ON_CLOSE,
                          NULL, 0);
    if (!NT_SUCCESS(status))
    {
        DbgPrint("  FAIL: ZwCreateFile(write) returned 0x%08lx\n", status);
        return FALSE;
    }

    /* Write test data */
    {
        UCHAR writeBuf[128];
        LARGE_INTEGER offset;
        ULONG i;

        for (i = 0; i < 128; i++)
            writeBuf[i] = (UCHAR)(i ^ 0xAA);

        status = ZwWriteFile(hFile, NULL, NULL, NULL, &ioStatus,
                             writeBuf, sizeof(writeBuf), NULL, NULL);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwWriteFile returned 0x%08lx\n", status);
            ZwClose(hFile);
            return FALSE;
        }

        /* Seek back to start and read */
        offset.QuadPart = 0;
        status = ZwReadFile(hFile, NULL, NULL, NULL, &ioStatus,
                            readBuf, sizeof(readBuf), &offset, NULL);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: ZwReadFile(readback) returned 0x%08lx\n", status);
            ZwClose(hFile);
            return FALSE;
        }

        /* Verify first 64 bytes */
        {
            ULONG j;
            for (j = 0; j < 64; j++)
            {
                if (readBuf[j] != (UCHAR)(j ^ 0xAA))
                {
                    DbgPrint("  FAIL: readback[%lu] = 0x%02x, expected 0x%02x\n",
                             j, readBuf[j], (UCHAR)(j ^ 0xAA));
                    ZwClose(hFile);
                    return FALSE;
                }
            }
        }
    }

    /* Close (FILE_DELETE_ON_CLOSE removes the file) */
    ZwClose(hFile);

    DbgPrint("  FileIo: open+read+MZ + create+write+readback+delete passed\n");
    return TRUE;
}
