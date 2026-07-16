/*
 * async32.exe — the i386 ASYNC file-I/O guest for WoW64 M15b
 * (docs/engineering/active-lanes/wow64-plan.md). Built like File32.c (no CRT,
 * plain _start, kernel32 only), loaded through the genuine 32-bit loader
 * under FEX.
 *
 * What it proves: kernel-side IO_STATUS_BLOCK32 completion. Before M15b the
 * wow64 thunk REFUSED Event-based/overlapped I/O outright (the host IOSB
 * lived on the thunk's dying stack frame — field manual §9.36). Now the
 * kernel writes the guest's 8-byte IOSB32 itself at I/O completion, so
 * overlapped ReadFile must work end-to-end:
 *
 *   - OVERLAPPED.Internal      IS IOSB32.Status      (offset 0)
 *   - OVERLAPPED.InternalHigh  IS IOSB32.Information (offset 4)
 *   - OVERLAPPED.Offset        (offset 8) doubles as the CLOBBER DETECTOR:
 *     if the kernel ever writes the 16-byte 64-bit IOSB layout again (latch
 *     missed / TEB offset drift), Information lands at +8 and stomps Offset.
 *
 * Verdict contract (same as hello32/file32): exit 0x600D on PASS; distinct
 * 0xBADx names the failing rung so serial alone triages.
 */

#include <windows.h>

#define ASYNC32_FAIL_EVENT      0xBAD1 /* CreateEventA failed */
#define ASYNC32_FAIL_OPEN       0xBAD2 /* CreateFileA(FILE_FLAG_OVERLAPPED) failed */
#define ASYNC32_FAIL_ISSUE      0xBAD3 /* ReadFile neither succeeded nor pended */
#define ASYNC32_FAIL_WAIT       0xBAD4 /* event never signaled -> completion lost */
#define ASYNC32_FAIL_STATUS     0xBAD5 /* IOSB32.Status (ov.Internal) != SUCCESS */
#define ASYNC32_FAIL_COUNT      0xBAD6 /* IOSB32.Information (ov.InternalHigh) wrong */
#define ASYNC32_FAIL_CLOBBER    0xBAD7 /* ov.Offset stomped -> 64-bit IOSB written */
#define ASYNC32_FAIL_OVRESULT   0xBAD8 /* GetOverlappedResult disagrees */
#define ASYNC32_FAIL_OPEN2      0xBAD9 /* sync control handle failed to open */
#define ASYNC32_FAIL_CONTENT    0xBADA /* async bytes != sync bytes of same region */
#define ASYNC32_FAIL_POLL       0xBADB /* eventless read: Internal never left PENDING */
#define ASYNC32_FAIL_POLLDATA   0xBADC /* eventless read: bad status/count after poll */
#define ASYNC32_PASS            0x600D

#define TARGET_PATH  "C:\\ReactOS\\syswow64\\hello32.exe"
#define READ_OFFSET  64
#define READ_LEN     512
#define STATUS_PENDING_32 ((DWORD)0x00000103)

static void Report(const char *msg)
{
    OutputDebugStringA(msg);
}

static void ReportHex(const char *label, DWORD v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[80];
    int i = 0, n;
    while (label[i] && i < 60) { buf[i] = label[i]; i++; }
    buf[i++] = ' '; buf[i++] = '0'; buf[i++] = 'x';
    for (n = 28; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    Report(buf);
}

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    HANDLE ev, h, h2;
    OVERLAPPED ov, ov2;
    BYTE abuf[READ_LEN], sbuf[READ_LEN];
    DWORD got, i;
    BOOL ok;

    Report("[async32] start: overlapped ReadFile (M15b kernel-side IOSB32)\n");

    ev = CreateEventA(NULL, TRUE, FALSE, NULL);
    if (!ev || ev == INVALID_HANDLE_VALUE)
        ExitProcess(ASYNC32_FAIL_EVENT);

    h = CreateFileA(TARGET_PATH, GENERIC_READ,
                    FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        ReportHex("[async32] FAIL open err", GetLastError());
        ExitProcess(ASYNC32_FAIL_OPEN);
    }

    /* Rung 1: event-based overlapped read. */
    for (i = 0; i < sizeof(ov); i++) ((BYTE *)&ov)[i] = 0;
    ov.Offset = READ_OFFSET;
    ov.hEvent = ev;
    ok = ReadFile(h, abuf, READ_LEN, NULL, &ov);
    if (!ok && GetLastError() != ERROR_IO_PENDING)
    {
        ReportHex("[async32] FAIL issue err", GetLastError());
        ExitProcess(ASYNC32_FAIL_ISSUE);
    }
    Report(ok ? "[async32] read completed synchronously\n"
              : "[async32] read PENDED (true async)\n");

    if (WaitForSingleObject(ev, 10000) != WAIT_OBJECT_0)
        ExitProcess(ASYNC32_FAIL_WAIT);

    ReportHex("[async32] ov.Internal (IOSB32.Status)     ", (DWORD)ov.Internal);
    ReportHex("[async32] ov.InternalHigh (IOSB32.Info)   ", (DWORD)ov.InternalHigh);
    ReportHex("[async32] ov.Offset (clobber detector)    ", ov.Offset);

    if ((DWORD)ov.Internal != 0)
        ExitProcess(ASYNC32_FAIL_STATUS);
    if ((DWORD)ov.InternalHigh != READ_LEN)
        ExitProcess(ASYNC32_FAIL_COUNT);
    if (ov.Offset != READ_OFFSET) /* 16-byte 64-bit IOSB write stomps this */
        ExitProcess(ASYNC32_FAIL_CLOBBER);

    if (!GetOverlappedResult(h, &ov, &got, FALSE) || got != READ_LEN)
        ExitProcess(ASYNC32_FAIL_OVRESULT);

    /* Control: same region through a plain synchronous handle. */
    h2 = CreateFileA(TARGET_PATH, GENERIC_READ,
                     FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_EXISTING,
                     0, NULL);
    if (h2 == INVALID_HANDLE_VALUE)
        ExitProcess(ASYNC32_FAIL_OPEN2);
    if (SetFilePointer(h2, READ_OFFSET, NULL, FILE_BEGIN) != READ_OFFSET ||
        !ReadFile(h2, sbuf, READ_LEN, &got, NULL) || got != READ_LEN)
        ExitProcess(ASYNC32_FAIL_OPEN2);
    for (i = 0; i < READ_LEN; i++)
        if (abuf[i] != sbuf[i])
            ExitProcess(ASYNC32_FAIL_CONTENT);
    Report("[async32] async bytes match sync control read\n");

    /* Rung 2: eventless overlapped read — the guest POLLS its IOSB32. Only
     * the kernel-side completion write can ever flip Internal off PENDING. */
    for (i = 0; i < sizeof(ov2); i++) ((BYTE *)&ov2)[i] = 0;
    ov2.Offset = 0;
    ov2.Internal = STATUS_PENDING_32;
    ok = ReadFile(h, abuf, READ_LEN, NULL, &ov2);
    if (!ok && GetLastError() != ERROR_IO_PENDING)
        ExitProcess(ASYNC32_FAIL_ISSUE);
    for (i = 0; i < 1000 && (DWORD)ov2.Internal == STATUS_PENDING_32; i++)
        Sleep(10);
    if ((DWORD)ov2.Internal == STATUS_PENDING_32)
        ExitProcess(ASYNC32_FAIL_POLL);
    ReportHex("[async32] poll: Internal", (DWORD)ov2.Internal);
    ReportHex("[async32] poll: InternalHigh", (DWORD)ov2.InternalHigh);
    if ((DWORD)ov2.Internal != 0 || (DWORD)ov2.InternalHigh != READ_LEN)
        ExitProcess(ASYNC32_FAIL_POLLDATA);

    CloseHandle(h);
    CloseHandle(h2);
    CloseHandle(ev);

    Report("[async32] PASS: overlapped I/O completed with kernel-written IOSB32\n");
    ExitProcess(ASYNC32_PASS);
    return 0;
}
