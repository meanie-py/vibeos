/*
 * win32.exe — the i386 WINDOW-CREATION guest for WoW64 M8
 * (docs/engineering/active-lanes/wow64-plan.md). Built like Gui32.c (no CRT,
 * plain _start), but imports user32: RegisterClassA + CreateWindowExA + a
 * REAL message pump. This forces the whole user32 client-connect path
 * (NtUserProcessConnect, SERVERINFO/gSharedInfo, desktop heap) AND
 * kernel→user callbacks (WM_NCCREATE/WM_CREATE arrive via KeUserModeCallback
 * → wow64 callback table → guest 32-bit wndproc under FEX).
 *
 * Verdict contract: exit 0x600D on PASS; distinct 0xBADx names the failing
 * rung so serial alone triages ([WINLOGON] win32.exe exit = 0x...).
 *
 * Visual side effect: a 200x150 window at (40,40) filled magenta by its own
 * WM_PAINT — a 32-bit x86 app running a real window on the arm64 desktop.
 */

#include <windows.h>

#define W32_FAIL_REGCLASS    0xBAD5
#define W32_FAIL_CREATEWND   0xBAD6
#define W32_FAIL_NO_CREATE   0xBAD7 /* window made but WM_CREATE never hit wndproc */
#define W32_WARN_NO_PAINT    0x600E /* window + callbacks + pump OK, WM_PAINT not yet
                                     * generated (M9 win32k paint-invalidation) */
#define W32_PASS             0x600D /* window created, WM_PAINT painted magenta */

/* Soft cap on pump iterations: WM_PAINT delivery for a WoW64 window is the M9
 * frontier; until then the window+callback capability (M8) is the PASS line. */
#define W32_PUMP_LIMIT       200

static void Report(const char *msg)
{
    OutputDebugStringA(msg);
}

static volatile LONG g_SawCreate;
static volatile LONG g_SawPaint;

static LRESULT CALLBACK WndProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp)
{
    switch (msg)
    {
        case WM_NCCREATE:
            /* Answer TRUE directly: the guest DefWindowProc NCCREATE path
             * pulls in dependencies (window-text alloc) not yet wired for
             * WoW64; TRUE keeps creation proceeding. */
            Report("[win32] WM_NCCREATE in guest wndproc -> TRUE\n");
            return 1;

        case WM_CREATE:
            g_SawCreate = 1;
            Report("[win32] WM_CREATE in guest wndproc\n");
            return 0;

        case WM_PAINT:
        {
            PAINTSTRUCT ps;
            HDC dc = BeginPaint(hwnd, &ps);
            if (dc)
            {
                HBRUSH b = CreateSolidBrush(RGB(255, 0, 255));
                RECT rc;
                GetClientRect(hwnd, &rc);
                FillRect(dc, &rc, b);
                DeleteObject(b);
                EndPaint(hwnd, &ps);
            }
            if (!g_SawPaint)
            {
                g_SawPaint = 1;
                Report("[win32] WM_PAINT painted magenta; posting quit\n");
                PostQuitMessage(0);
            }
            return 0;
        }

        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;
    }
    return DefWindowProcA(hwnd, msg, wp, lp);
}

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    WNDCLASSA wc;
    HWND hwnd;
    MSG m;
    int pumped = 0;

    Report("[win32] start: RegisterClassA\n");
    /* no CRT: no memset — field-init the class explicitly */
    wc.style = 0;
    wc.cbClsExtra = 0;
    wc.cbWndExtra = 0;
    wc.hIcon = NULL;
    wc.hCursor = NULL;
    wc.lpszMenuName = NULL;
    wc.lpfnWndProc = WndProc;
    wc.hInstance = GetModuleHandleA(NULL);
    wc.hbrBackground = (HBRUSH)GetStockObject(WHITE_BRUSH);
    wc.lpszClassName = "W32M8CLS";
    if (!RegisterClassA(&wc))
    {
        Report("[win32] FAIL: RegisterClassA\n");
        ExitProcess(W32_FAIL_REGCLASS);
    }

    Report("[win32] CreateWindowExA\n");
    hwnd = CreateWindowExA(0, "W32M8CLS", "wow64 m8", WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                           40, 40, 200, 150, NULL, NULL, wc.hInstance, NULL);
    if (!hwnd)
    {
        Report("[win32] FAIL: CreateWindowExA returned NULL\n");
        ExitProcess(W32_FAIL_CREATEWND);
    }
    if (!g_SawCreate)
    {
        Report("[win32] FAIL: window exists but WM_CREATE never dispatched\n");
        ExitProcess(W32_FAIL_NO_CREATE);
    }

    Report("[win32] window up; pumping messages\n");
    UpdateWindow(hwnd); /* request a synchronous first WM_PAINT */

    /* PeekMessage pump so it self-terminates when the queue drains (a blocking
     * GetMessage would hang with no WM_QUIT). WM_PAINT is the goal; if it never
     * arrives within the cap, window+callback capability (M8) still passed as a
     * soft-warn rung, like gui32's GetPixel readback. */
    while (pumped < W32_PUMP_LIMIT && !g_SawPaint)
    {
        if (PeekMessageA(&m, NULL, 0, 0, PM_REMOVE))
        {
            TranslateMessage(&m);
            DispatchMessageA(&m);
        }
        pumped++;
    }

    if (!g_SawPaint)
    {
        Report("[win32] WARN: window+callbacks+pump OK, no WM_PAINT (M9 owed)\n");
        ExitProcess(W32_WARN_NO_PAINT);
    }

    Report("[win32] PASS: 32-bit window created, painted through native win32k\n");
    ExitProcess(W32_PASS);
    return 0;
}
