/*
 * PROJECT:     vibeos kernel-mode tests
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     RtlUTF8ToUnicodeN / RtlUnicodeToUTF8N, the Windows 7 kernel's UTF-8
 *              conversions. Graded on Windows 11 first (`vibeos-cli winref kmtest`): a
 *              test Windows fails is a wrong test. Looked up with MmGetSystemRoutineAddress,
 *              never imported (see ntos_ex/ExPool2.c).
 */

#include <kmt_test.h>

#define NDEBUG
#include <debug.h>

typedef NTSTATUS (NTAPI *PFN_U8_TO_U16)(PWSTR, ULONG, PULONG, PCCH, ULONG);
typedef NTSTATUS (NTAPI *PFN_U16_TO_U8)(PCHAR, ULONG, PULONG, PCWCH, ULONG);

START_TEST(RtlUtf8)
{
    UNICODE_STRING Name;
    PFN_U8_TO_U16 ToUnicode;
    PFN_U16_TO_U8 ToUtf8;
    static const CHAR Utf8[] = "h\xC3\xA9llo \xE2\x82\xAC";     /* "hello" with e-acute, then the euro sign */
    static const WCHAR Wide[] = { L'h', 0xE9, L'l', L'l', L'o', L' ', 0x20AC };
    static const CHAR Bad[] = { 'a', (CHAR)0xFF, 'b' };
    WCHAR WBuf[16];
    CHAR CBuf[16];
    ULONG Size;
    NTSTATUS Status;

    RtlInitUnicodeString(&Name, L"RtlUTF8ToUnicodeN");
    ToUnicode = (PFN_U8_TO_U16)MmGetSystemRoutineAddress(&Name);
    RtlInitUnicodeString(&Name, L"RtlUnicodeToUTF8N");
    ToUtf8 = (PFN_U16_TO_U8)MmGetSystemRoutineAddress(&Name);
    ok(ToUnicode != NULL, "RtlUTF8ToUnicodeN is not exported\n");
    ok(ToUtf8 != NULL, "RtlUnicodeToUTF8N is not exported\n");
    if (!ToUnicode || !ToUtf8)
        return;

    /* UTF-8 -> UTF-16 */
    Size = 0xdead;
    RtlFillMemory(WBuf, sizeof(WBuf), 0x55);
    Status = ToUnicode(WBuf, sizeof(WBuf), &Size, Utf8, sizeof(Utf8) - 1);
    ok_eq_hex(Status, STATUS_SUCCESS);
    ok_eq_ulong(Size, (ULONG)sizeof(Wide));
    ok(RtlCompareMemory(WBuf, Wide, sizeof(Wide)) == sizeof(Wide), "wrong UTF-16\n");

    /* NULL destination: the size it would need */
    Size = 0;
    Status = ToUnicode(NULL, 0, &Size, Utf8, sizeof(Utf8) - 1);
    ok_eq_hex(Status, STATUS_SUCCESS);
    ok_eq_ulong(Size, (ULONG)sizeof(Wide));

    /* An invalid byte becomes U+FFFD, and the call says so */
    Size = 0;
    Status = ToUnicode(WBuf, sizeof(WBuf), &Size, Bad, sizeof(Bad));
    ok_eq_hex(Status, STATUS_SOME_NOT_MAPPED);
    ok_eq_ulong(Size, 3UL * sizeof(WCHAR));
    ok_eq_hex(WBuf[1], 0xFFFD);

    /* UTF-16 -> UTF-8 */
    Size = 0xdead;
    RtlFillMemory(CBuf, sizeof(CBuf), 0x55);
    Status = ToUtf8(CBuf, sizeof(CBuf), &Size, Wide, sizeof(Wide));
    ok_eq_hex(Status, STATUS_SUCCESS);
    ok_eq_ulong(Size, (ULONG)(sizeof(Utf8) - 1));
    ok(RtlCompareMemory(CBuf, Utf8, sizeof(Utf8) - 1) == sizeof(Utf8) - 1, "wrong UTF-8\n");

    Size = 0;
    Status = ToUtf8(NULL, 0, &Size, Wide, sizeof(Wide));
    ok_eq_hex(Status, STATUS_SUCCESS);
    ok_eq_ulong(Size, (ULONG)(sizeof(Utf8) - 1));

    /* Too small a destination */
    Size = 0;
    Status = ToUtf8(CBuf, 2, &Size, Wide, sizeof(Wide));
    ok_eq_hex(Status, STATUS_BUFFER_TOO_SMALL);
    trace("RtlUnicodeToUTF8N into 2 bytes: wrote %lu\n", Size);
}
