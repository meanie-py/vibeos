/*
 * gui32.exe — the i386 GUI-syscall guest for WoW64 M4/wow64win
 * (docs/engineering/wow64-plan.md). Built with the llvm-mingw i686 target,
 * no CRT (plain _start entry, same as Hello32.c), but with REAL imports:
 * kernel32 + gdi32, loaded through the genuine 32-bit loader under FEX.
 * It exercises the win32k thunk table WITHOUT user-mode callbacks:
 * a direct display DC (NtGdiOpenDCW via CreateDCA) + PatBlt + GetPixel.
 *
 * Verdict contract (same as hello32): exit code 0x600D on PASS; distinct
 * 0xBADx codes name the failing rung so serial alone triages. The winlogon
 * harness (TryStartWow64Gui) logs the exit code.
 *
 * Visual side effect: a magenta rectangle at the top-left of the arm64
 * desktop — a 32-bit x86 app drawing through the 64-bit arm64 win32k.
 */

#include <windows.h>

/* Distinct exit codes per rung */
#define GUI32_FAIL_CREATEDC   0xBAD1
#define GUI32_FAIL_PATBLT     0xBAD2
#define GUI32_WARN_GETPIXEL   0x600E /* drew fine, readback mismatched (fb readback support) */
#define GUI32_PASS            0x600D

static void Report(const char *msg)
{
    /* OutputDebugStringA reaches serial via the guest DbgPrint forwarding
     * (int 2d DebugService -> [GUEST32] in the wow64 VEH). */
    OutputDebugStringA(msg);
}

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    HDC dc;
    HBRUSH brush, old;
    COLORREF px;

    Report("[gui32] start: CreateDCA(DISPLAY)\n");
    dc = CreateDCA("DISPLAY", NULL, NULL, NULL);
    if (!dc)
    {
        Report("[gui32] FAIL: CreateDCA returned NULL\n");
        ExitProcess(GUI32_FAIL_CREATEDC);
    }

    Report("[gui32] display DC ok; PatBlt 160x120 magenta\n");
    brush = CreateSolidBrush(RGB(255, 0, 255));
    old = (HBRUSH)SelectObject(dc, brush);
    if (!PatBlt(dc, 8, 8, 160, 120, PATCOPY))
    {
        Report("[gui32] FAIL: PatBlt returned FALSE\n");
        ExitProcess(GUI32_FAIL_PATBLT);
    }
    SelectObject(dc, old);
    DeleteObject(brush);

    /* Readback is a soft rung: some display drivers don't support GetPixel
     * on the screen DC; the draw already proves the win32k path. */
    px = GetPixel(dc, 20, 20);
    DeleteDC(dc);
    if (px != RGB(255, 0, 255))
    {
        Report("[gui32] PASS(draw) but GetPixel readback mismatched\n");
        ExitProcess(GUI32_WARN_GETPIXEL);
    }

    Report("[gui32] PASS: 32-bit GDI drew on the host desktop and read it back\n");
    ExitProcess(GUI32_PASS);
    return 0; /* unreachable */
}
