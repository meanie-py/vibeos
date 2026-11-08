/* gdiprobe: does a GDI object that is created and deleted again stay deleted?
 * Long Chrome boots wedged 15-60 min in: the stock 32-bit osk.exe piled up pens until the GDI
 * handle table was full, pool ran dry and win32k flooded "Not enough memory to allocate a
 * message". The cause was every WOW64 delete of a handle whose reuse byte had its top bit set
 * failing (win32k compared the sign-extended handle at pointer width). Each pair below runs N
 * times, past 128 reuses of a slot and past the whole table, and counts failed deletes as well
 * as the GDI object count. Built for amd64 and, through WOW64, i386. It runs before the Chrome
 * runs, while a guest that leaks still has handles to spare. One line per fact, to diff with
 * gdiprobe.win11.txt. */
#include <windows.h>
#include <stdio.h>
#include <string.h>

/* Past the whole GDI handle table (~16K) and the 10K per-process quota: a leak cannot pass. */
#define N 20000

/* GDIPROBE_TRACE: raw markers (WriteFile, no CRT formatting) between steps, to place a death
 * whose own output never arrives. Off unless built with -DGDIPROBE_TRACE. */
#ifdef GDIPROBE_TRACE
static void mark(const char *m)
{
    DWORD n;
    WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), m, (DWORD)strlen(m), &n, NULL);
}
#else
#define mark(m) ((void)0)
#endif

static void say(const char *fmt, ...)
{
    char b[300];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("[GDI] %s\n", b);
    fflush(stdout);
}

static DWORD objs(void)
{
    return GetGuiResources(GetCurrentProcess(), GR_GDIOBJECTS);
}

static const char *grew(DWORD before, DWORD after)
{
    /* Some slack: Windows' gdi32 keeps up to 10 deleted solid brushes cached for reuse. A
     * leak here is thousands. */
    return after <= before + 32 ? "flat" : "GREW";
}

int main(void)
{
    DWORD b, a;
    int fails;

    mark("[GDITRACE] main\n");
    say("%s", sizeof(void *) == 8 ? "64-bit" : "32-bit");
    mark("[GDITRACE] said\n");

    b = objs();
    mark("[GDITRACE] GetGuiResources returned\n");
    fails = 0;
    for (int i = 0; i < N; i++) {
        HDC dc = CreateDCA("DISPLAY", NULL, NULL, NULL);
        if (!dc) {
            fails++;
            continue;
        }
        if (!DeleteDC(dc))
            fails++;
    }
    a = objs();
    say("CreateDC(DISPLAY)+DeleteDC x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);

    b = objs();
    fails = 0;
    for (int i = 0; i < N; i++) {
        HDC dc = CreateICA("DISPLAY", NULL, NULL, NULL);
        if (!dc) {
            fails++;
            continue;
        }
        if (!DeleteDC(dc))
            fails++;
    }
    a = objs();
    say("CreateIC(DISPLAY)+DeleteDC x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);

    b = objs();
    fails = 0;
    for (int i = 0; i < N; i++) {
        HDC dc = GetDC(NULL);
        HDC mem = dc ? CreateCompatibleDC(dc) : NULL;
        HBITMAP bm = dc ? CreateCompatibleBitmap(dc, 16, 16) : NULL;
        if (!dc || !mem || !bm)
            fails++;
        if (bm && !DeleteObject(bm))
            fails++;
        if (mem && !DeleteDC(mem))
            fails++;
        if (dc)
            ReleaseDC(NULL, dc);
    }
    a = objs();
    say("GetDC+CreateCompatibleDC/Bitmap+free x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);

    /* What magnify does every tick: a cursor's bitmaps from GetIconInfo, then freed. */
    b = objs();
    fails = 0;
    for (int i = 0; i < N; i++) {
        /* The arrow, not GetCursorInfo's cursor: a process started outside the interactive
         * desktop has none, and the copies GetIconInfo makes are the same either way. */
        HCURSOR cur = LoadCursorA(NULL, (LPCSTR)IDC_ARROW);
        ICONINFO ii = { 0 };
        if (!cur || !GetIconInfo(cur, &ii)) {
            fails++;
            continue;
        }
        if (ii.hbmMask && !DeleteObject(ii.hbmMask))
            fails++;
        if (ii.hbmColor && !DeleteObject(ii.hbmColor))
            fails++;
    }
    a = objs();
    say("GetIconInfo(arrow)+DeleteObject x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);

    /* osk.exe's leaked objects are type 0x10 -- brushes AND pens (a pen is a brush object). */
    b = objs();
    fails = 0;
    for (int i = 0; i < N; i++) {
        HPEN p = CreatePen(PS_SOLID, 1, RGB(i & 255, 0, 0));
        if (!p || !DeleteObject(p))
            fails++;
    }
    a = objs();
    say("CreatePen+DeleteObject x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);

    b = objs();
    fails = 0;
    for (int i = 0; i < N; i++) {
        HBRUSH br = CreateSolidBrush(RGB(0, i & 255, 0));
        if (!br || !DeleteObject(br))
            fails++;
    }
    a = objs();
    say("CreateSolidBrush+DeleteObject x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);

    /* A key label with a prefix: DrawText underlines the character after '&' with a pen it
     * creates and deletes. */
    {
        HDC dc = GetDC(NULL);
        HDC mem = CreateCompatibleDC(dc);
        HBITMAP bm = CreateCompatibleBitmap(dc, 64, 32);
        HGDIOBJ old = SelectObject(mem, bm);
        RECT r = { 0, 0, 64, 32 };
        b = objs();
        fails = 0;
        for (int i = 0; i < N; i++)
            if (!DrawTextW(mem, L"&A", -1, &r, DT_SINGLELINE))
                fails++;
        a = objs();
        say("DrawText(\"&A\") x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);
        SelectObject(mem, old);
        DeleteObject(bm);
        DeleteDC(mem);
        ReleaseDC(NULL, dc);
    }

    /* What osk does ten times a second: set a key button's text, which repaints it. osk.exe's
     * GDI object count climbed by thousands in the guest while osk itself frees all it makes. */
    {
        HWND top = CreateWindowExA(0, "STATIC", "gdiprobe", WS_POPUP | WS_VISIBLE, 0, 0, 200, 60,
                                   NULL, NULL, NULL, NULL);
        HWND btn = CreateWindowExA(0, "BUTTON", "a", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 0, 0,
                                   60, 30, top, NULL, NULL, NULL);
        b = objs();
        fails = 0;
        for (int i = 0; i < N; i++) {
            if (!SetWindowTextW(btn, (i & 1) ? L"a" : L"A"))
                fails++;
            UpdateWindow(btn);
        }
        a = objs();
        say("button SetWindowText+repaint x%d: objects %s, failures %d (count %lu -> %lu)", N, grew(b, a), fails, b, a);
        DestroyWindow(top);
    }

    say("done");
    return 0;
}
