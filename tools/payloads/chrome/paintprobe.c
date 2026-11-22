/* paintprobe: does an untouched window keep getting WM_PAINT?
 * On the smoke's one vCPU, osk.exe (32-bit, under WOW64) used 27% of the CPU for a whole boot
 * with nobody typing, sampled drawing its buttons (comctl32 BUTTON_DrawTextCallback) and in
 * win32k syscalls. An idle window paints once; one whose update region is never validated
 * gets WM_PAINT forever. This makes a window with 30 child buttons, lets it settle for 1 s,
 * then pumps for 5 s counting WM_PAINT (parent and children) and the thread's CPU time.
 * Built 64-bit and 32-bit, so native and WOW64 can be compared in one boot.
 * One line per fact, to diff with paintprobe.win11.txt. */
#include <windows.h>
#include <stdio.h>

static HWND g_parent;

static LRESULT CALLBACK wndproc(HWND h, UINT m, WPARAM w, LPARAM l)
{
    if (m == WM_PAINT) {
        PAINTSTRUCT ps;
        HDC dc = BeginPaint(h, &ps);
        FillRect(dc, &ps.rcPaint, (HBRUSH)(COLOR_BTNFACE + 1));
        EndPaint(h, &ps);
        return 0;
    }
    return DefWindowProcA(h, m, w, l);
}

static LONGLONG thread_cpu_ms(void)
{
    FILETIME c, e, k, u;
    if (!GetThreadTimes(GetCurrentThread(), &c, &e, &k, &u))
        return -1;
    ULARGE_INTEGER kk = { .LowPart = k.dwLowDateTime, .HighPart = k.dwHighDateTime };
    ULARGE_INTEGER uu = { .LowPart = u.dwLowDateTime, .HighPart = u.dwHighDateTime };
    return (LONGLONG)((kk.QuadPart + uu.QuadPart) / 10000);
}

/* Pump for `ms`, counting what arrives. */
static void pump(DWORD ms, int *parent_paints, int *child_paints, int *other)
{
    DWORD end = GetTickCount() + ms;
    for (;;) {
        DWORD now = GetTickCount();
        if ((LONG)(end - now) <= 0)
            break;
        MsgWaitForMultipleObjects(0, NULL, FALSE, end - now, QS_ALLINPUT);
        MSG msg;
        while (PeekMessageA(&msg, NULL, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_PAINT) {
                if (msg.hwnd == g_parent)
                    (*parent_paints)++;
                else
                    (*child_paints)++;
            } else {
                (*other)++;
            }
            DispatchMessageA(&msg);
            if ((LONG)(end - GetTickCount()) <= 0)
                break;
        }
    }
}

int main(void)
{
    const int bits = (int)sizeof(void *) * 8;
    WNDCLASSA wc = { 0 };
    wc.lpfnWndProc = wndproc;
    wc.hInstance = GetModuleHandleA(NULL);
    wc.hCursor = LoadCursorA(NULL, (LPCSTR)IDC_ARROW);
    wc.lpszClassName = "paintprobe";
    if (!RegisterClassA(&wc)) {
        printf("[PAINTPROBE] bits=%d RegisterClass failed %lu\n", bits, GetLastError());
        return 1;
    }
    g_parent = CreateWindowExA(0, "paintprobe", "paintprobe", WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                               40, 40, 640, 220, NULL, NULL, wc.hInstance, NULL);
    if (!g_parent) {
        printf("[PAINTPROBE] bits=%d CreateWindow failed %lu\n", bits, GetLastError());
        return 1;
    }
    for (int i = 0; i < 30; i++) {
        char label[8];
        snprintf(label, sizeof label, "%c", 'A' + i % 26);
        CreateWindowExA(0, "BUTTON", label, WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                        8 + (i % 10) * 60, 8 + (i / 10) * 50, 56, 44, g_parent,
                        (HMENU)(INT_PTR)(100 + i), wc.hInstance, NULL);
    }
    ShowWindow(g_parent, SW_SHOW);
    UpdateWindow(g_parent);

    int pp = 0, cp = 0, other = 0;
    pump(1000, &pp, &cp, &other);
    printf("[PAINTPROBE] bits=%d settle 1 s: parent paints %d, child paints %d, other msgs %d\n",
           bits, pp, cp, other);
    fflush(stdout);

    pp = cp = other = 0;
    LONGLONG cpu0 = thread_cpu_ms();
    pump(5000, &pp, &cp, &other);
    LONGLONG cpu = thread_cpu_ms() - cpu0;
    RECT r;
    BOOL dirty = GetUpdateRect(g_parent, &r, FALSE);
    printf("[PAINTPROBE] bits=%d quiet 5 s: parent paints %d, child paints %d, other msgs %d, "
           "thread cpu %lld ms, parent update region %s\n",
           bits, pp, cp, other, cpu, dirty ? "NOT EMPTY" : "empty");
    printf("[PAINTPROBE] bits=%d done VERDICT=%s\n", bits,
           (pp + cp) <= 2 ? "PASS" : "FAIL (an untouched window keeps getting WM_PAINT)");
    fflush(stdout);
    DestroyWindow(g_parent);
    return 0;
}
