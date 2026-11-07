/* gdiprobe: does a DC that is created and freed again stay freed?
 * Long Chrome boots wedged after about an hour: a 32-bit app polling once a second opened a
 * display DC each time (NtGdiOpenDCW), the GDI handle index climbed to the ~16K table limit and
 * never came back, and once NtGdiOpenDCW returned 0 the pool behind it ran dry too (win32k
 * "Not enough memory to allocate a message"). Each pair below runs N times, more than the
 * handle table holds, so a pair that leaks starts failing; the GDI object count is printed too.
 * Built for amd64 and, to go through WOW64 like the leaking app, i386. It runs before the Chrome
 * runs, while a guest that leaks still has handles to spare; once the leak has run for a while
 * nothing after it gets to run. One line per fact, to diff with gdiprobe.win11.txt. */
#include <windows.h>
#include <stdio.h>

/* Past the whole GDI handle table (~16K) and the 10K per-process quota: a leak cannot pass. */
#define N 20000

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
    return after <= before + 4 ? "flat" : "GREW";
}

int main(void)
{
    DWORD b, a;
    int fails;

    say("%s", sizeof(void *) == 8 ? "64-bit" : "32-bit");

    b = objs();
    fails = 0;
    for (int i = 0; i < N; i++) {
        HDC dc = CreateDCA("DISPLAY", NULL, NULL, NULL);
        if (!dc) {
            fails++;
            continue;
        }
        DeleteDC(dc);
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
        DeleteDC(dc);
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
        if (bm)
            DeleteObject(bm);
        if (mem)
            DeleteDC(mem);
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
