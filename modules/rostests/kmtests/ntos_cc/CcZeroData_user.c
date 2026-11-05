/*
 * PROJECT:         ReactOS kernel-mode tests
 * LICENSE:         GPLv2+ - See COPYING in the top level directory
 * PURPOSE:         Kernel-Mode Test Suite CcZeroData test user-mode part
 */

#include <kmt_test.h>

START_TEST(CcZeroData)
{
    HANDLE Handle;
    NTSTATUS Status;
    LARGE_INTEGER ByteOffset;
    IO_STATUS_BLOCK IoStatusBlock;
    OBJECT_ATTRIBUTES ObjectAttributes;
    UNICODE_STRING ZeroFile = RTL_CONSTANT_STRING(L"\\Device\\Kmtest-CcZeroData\\ZeroFile");
    CHAR Byte = 0;
    DWORD Error;

    Error = KmtLoadAndOpenDriver(L"CcZeroData", FALSE);
    ok_eq_int(Error, ERROR_SUCCESS);
    if (Error)
        return;

    InitializeObjectAttributes(&ObjectAttributes, &ZeroFile, OBJ_CASE_INSENSITIVE, NULL, NULL);
    Status = NtOpenFile(&Handle, FILE_ALL_ACCESS, &ObjectAttributes, &IoStatusBlock, 0,
                        FILE_NON_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT);
    ok_eq_hex(Status, STATUS_SUCCESS);
    if (!NT_SUCCESS(Status))
    {
        KmtCloseDriver();
        KmtUnloadDriver();
        return;
    }

    /* The driver runs the CcZeroData cases from inside this write */
    ByteOffset.QuadPart = 0;
    Status = NtWriteFile(Handle, NULL, NULL, NULL, &IoStatusBlock, &Byte, 1, &ByteOffset, NULL);
    ok_eq_hex(Status, STATUS_SUCCESS);

    NtClose(Handle);

    KmtCloseDriver();
    KmtUnloadDriver();
}
