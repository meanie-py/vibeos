/*
 * PROJECT:     ReactOS Kernel
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Kernel test — RTL String Operations
 * COPYRIGHT:   Copyright 2025 ReactOS ARM64 Port
 *
 * Tests RtlInitUnicodeString, RtlCompareUnicodeString, RtlCopyUnicodeString,
 * RtlAnsiStringToUnicodeString, RtlUnicodeStringToAnsiString,
 * RtlAppendUnicodeStringToString, RtlIntegerToUnicodeString, etc.
 */

#include <ntoskrnl.h>

#define NDEBUG
#include <debug.h>

/**
 * NtTestRtlStr - Exercise RTL string functions.
 */
BOOLEAN
NTAPI
NtTestRtlStr(VOID)
{
    /* --- Test 1: RtlInitUnicodeString --- */
    {
        UNICODE_STRING us;
        RtlInitUnicodeString(&us, L"Hello");

        if (us.Length != 10)  /* 5 wchars * 2 bytes */
        {
            DbgPrint("  FAIL: RtlInitUnicodeString Length=%u, expected 10\n", us.Length);
            return FALSE;
        }
        if (us.MaximumLength != 12)  /* Length + sizeof(WCHAR) */
        {
            DbgPrint("  FAIL: RtlInitUnicodeString MaxLen=%u, expected 12\n", us.MaximumLength);
            return FALSE;
        }
    }

    /* --- Test 2: RtlCompareUnicodeString --- */
    {
        UNICODE_STRING a, b, c;
        RtlInitUnicodeString(&a, L"Alpha");
        RtlInitUnicodeString(&b, L"Alpha");
        RtlInitUnicodeString(&c, L"Beta");

        if (RtlCompareUnicodeString(&a, &b, FALSE) != 0)
        {
            DbgPrint("  FAIL: 'Alpha' != 'Alpha'\n");
            return FALSE;
        }

        if (RtlCompareUnicodeString(&a, &c, FALSE) >= 0)
        {
            DbgPrint("  FAIL: 'Alpha' should be < 'Beta'\n");
            return FALSE;
        }

        /* Case-insensitive */
        {
            UNICODE_STRING upper, lower;
            RtlInitUnicodeString(&upper, L"HELLO");
            RtlInitUnicodeString(&lower, L"hello");

            if (RtlCompareUnicodeString(&upper, &lower, TRUE) != 0)
            {
                DbgPrint("  FAIL: case-insensitive 'HELLO' != 'hello'\n");
                return FALSE;
            }

            if (RtlCompareUnicodeString(&upper, &lower, FALSE) == 0)
            {
                DbgPrint("  FAIL: case-sensitive 'HELLO' == 'hello'\n");
                return FALSE;
            }
        }
    }

    /* --- Test 3: RtlCopyUnicodeString --- */
    {
        UNICODE_STRING src, dst;
        WCHAR dstBuf[32];

        RtlInitUnicodeString(&src, L"CopyMe");
        dst.Buffer = dstBuf;
        dst.Length = 0;
        dst.MaximumLength = sizeof(dstBuf);

        RtlCopyUnicodeString(&dst, &src);

        if (dst.Length != src.Length)
        {
            DbgPrint("  FAIL: copy Length=%u, expected %u\n", dst.Length, src.Length);
            return FALSE;
        }

        if (RtlCompareUnicodeString(&dst, &src, FALSE) != 0)
        {
            DbgPrint("  FAIL: copied string doesn't match source\n");
            return FALSE;
        }
    }

    /* --- Test 4: RtlAppendUnicodeStringToString --- */
    {
        UNICODE_STRING base, append;
        WCHAR baseBuf[64];
        NTSTATUS status;

        RtlInitUnicodeString(&append, L" World");

        /* Initialize base with "Hello" */
        baseBuf[0] = L'H'; baseBuf[1] = L'e'; baseBuf[2] = L'l';
        baseBuf[3] = L'l'; baseBuf[4] = L'o';
        base.Buffer = baseBuf;
        base.Length = 10;  /* 5 wchars */
        base.MaximumLength = sizeof(baseBuf);

        status = RtlAppendUnicodeStringToString(&base, &append);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: RtlAppendUnicodeStringToString returned 0x%08lx\n", status);
            return FALSE;
        }

        {
            UNICODE_STRING expected;
            RtlInitUnicodeString(&expected, L"Hello World");
            if (RtlCompareUnicodeString(&base, &expected, FALSE) != 0)
            {
                DbgPrint("  FAIL: append result mismatch\n");
                return FALSE;
            }
        }
    }

    /* --- Test 5: RtlAnsiStringToUnicodeString --- */
    {
        ANSI_STRING ansi;
        UNICODE_STRING unicode;
        UNICODE_STRING expected;
        NTSTATUS status;

        RtlInitAnsiString(&ansi, "TestAnsi");
        status = RtlAnsiStringToUnicodeString(&unicode, &ansi, TRUE);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: RtlAnsiStringToUnicodeString returned 0x%08lx\n", status);
            return FALSE;
        }

        RtlInitUnicodeString(&expected, L"TestAnsi");
        if (RtlCompareUnicodeString(&unicode, &expected, FALSE) != 0)
        {
            DbgPrint("  FAIL: ANSI→Unicode conversion mismatch\n");
            RtlFreeUnicodeString(&unicode);
            return FALSE;
        }

        RtlFreeUnicodeString(&unicode);
    }

    /* --- Test 6: RtlUnicodeStringToAnsiString --- */
    {
        UNICODE_STRING unicode;
        ANSI_STRING ansi;
        NTSTATUS status;

        RtlInitUnicodeString(&unicode, L"TestUnicode");
        status = RtlUnicodeStringToAnsiString(&ansi, &unicode, TRUE);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: RtlUnicodeStringToAnsiString returned 0x%08lx\n", status);
            return FALSE;
        }

        if (strcmp(ansi.Buffer, "TestUnicode") != 0)
        {
            DbgPrint("  FAIL: Unicode→ANSI result='%s'\n", ansi.Buffer);
            RtlFreeAnsiString(&ansi);
            return FALSE;
        }

        RtlFreeAnsiString(&ansi);
    }

    /* --- Test 7: RtlIntegerToUnicodeString --- */
    {
        UNICODE_STRING numStr;
        WCHAR numBuf[32];
        UNICODE_STRING expected;
        NTSTATUS status;

        numStr.Buffer = numBuf;
        numStr.Length = 0;
        numStr.MaximumLength = sizeof(numBuf);

        status = RtlIntegerToUnicodeString(12345, 10, &numStr);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: RtlIntegerToUnicodeString returned 0x%08lx\n", status);
            return FALSE;
        }

        RtlInitUnicodeString(&expected, L"12345");
        if (RtlCompareUnicodeString(&numStr, &expected, FALSE) != 0)
        {
            DbgPrint("  FAIL: IntegerToUnicode(12345) mismatch\n");
            return FALSE;
        }

        /* Hex */
        numStr.Length = 0;
        status = RtlIntegerToUnicodeString(0xFF, 16, &numStr);
        if (!NT_SUCCESS(status))
        {
            DbgPrint("  FAIL: RtlIntegerToUnicodeString(hex) returned 0x%08lx\n", status);
            return FALSE;
        }

        RtlInitUnicodeString(&expected, L"ff");
        if (RtlCompareUnicodeString(&numStr, &expected, TRUE) != 0)
        {
            DbgPrint("  FAIL: IntegerToUnicode(0xFF, hex) mismatch\n");
            return FALSE;
        }
    }

    DbgPrint("  RtlStr: init/compare/copy/append/ansi<->unicode/integer all passed\n");
    return TRUE;
}
