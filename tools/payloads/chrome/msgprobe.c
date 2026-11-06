/* msgprobe: Chrome's UI-thread message pump (base/message_loop/message_pump_win.cc) against our
 * win32k. Chrome hangs with every thread blocked, and its UI thread sleeps in
 * MsgWaitForMultipleObjectsEx(QS_ALLINPUT, MWMO_INPUTAVAILABLE). Work is handed to that thread
 * by ONE posted kMsgHaveWork, guarded by a flag so nobody posts again until it is handled: a
 * wait that misses that message never wakes, and nothing ever retries. One line per fact, to
 * diff with msgprobe.win11.txt.
 *
 * The deterministic cases pin what a waiter must see of a message already in the queue; the
 * stress case races a poster against the pump exactly as ScheduleWork does and counts every
 * wait that timed out with the message still queued (a lost wake-up). */
#include <windows.h>
#include <stdio.h>

#define WM_HAVE_WORK (WM_USER + 1)
#define STRESS_N 3000

static void say(const char *fmt, ...)
{
    char b[400];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("[MSG] %s\n", b);
    fflush(stdout);
}

static volatile LONG g_scheduled, g_handled;

static LRESULT CALLBACK proc(HWND h, UINT m, WPARAM w, LPARAM l)
{
    if (m == WM_HAVE_WORK) {
        InterlockedExchange(&g_scheduled, 0);
        InterlockedIncrement(&g_handled);
        return 0;
    }
    return DefWindowProcA(h, m, w, l);
}

static const char *wr(DWORD r)
{
    static char b[32];
    if (r == WAIT_TIMEOUT)
        return "timeout";
    snprintf(b, sizeof b, "%lu", r);
    return b;
}

static void drain(void)
{
    MSG m;
    while (PeekMessageA(&m, NULL, 0, 0, PM_REMOVE))
        DispatchMessageA(&m);
}

static HWND g_hwnd;

static DWORD WINAPI poster(LPVOID unused)
{
    (void)unused;
    unsigned seed = 12345;
    for (int i = 0; i < STRESS_N; i++) {
        if (!InterlockedExchange(&g_scheduled, 1))
            PostMessageA(g_hwnd, WM_HAVE_WORK, 0, 0);
        seed = seed * 1103515245 + 12345;
        switch ((seed >> 16) % 4) {
        case 0: break;
        case 1: Sleep(0); break;
        case 2: SwitchToThread(); break;
        default: for (volatile int k = 0; k < 2000; k++); break;
        }
        /* Let the pump catch up before the next post, as real work is consumed. */
        while (g_scheduled && i % 64 == 63)
            Sleep(1);
    }
    return 0;
}

int main(void)
{
    WNDCLASSA wc = { 0 };
    MSG m;
    DWORD r;
    wc.lpfnWndProc = proc;
    wc.lpszClassName = "msgprobe";
    wc.hInstance = GetModuleHandleA(NULL);
    RegisterClassA(&wc);
    g_hwnd = CreateWindowA("msgprobe", "", 0, 0, 0, 0, 0, HWND_MESSAGE, NULL, wc.hInstance, NULL);
    say("message window: %s", g_hwnd ? "ok" : "FAIL");
    drain();

    /* 1: a message already peeked at (no longer "new") still satisfies MWMO_INPUTAVAILABLE. */
    PostMessageA(g_hwnd, WM_HAVE_WORK, 0, 0);
    PeekMessageA(&m, NULL, 0, 0, PM_NOREMOVE);
    r = MsgWaitForMultipleObjectsEx(0, NULL, 500, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    say("1 peeked, then wait INPUTAVAILABLE: %s", wr(r));
    r = MsgWaitForMultipleObjectsEx(0, NULL, 300, QS_ALLINPUT, 0);
    say("1 same message, wait without the flag: %s", wr(r));
    drain();

    /* 2: an unrelated wait that times out first (it must not forget the queued message). */
    PostMessageA(g_hwnd, WM_HAVE_WORK, 0, 0);
    r = MsgWaitForMultipleObjectsEx(0, NULL, 0, QS_TIMER, 0);
    say("2 posted, wait on QS_TIMER only: %s", wr(r));
    say("2 GetQueueStatus(QS_POSTMESSAGE) = %08lx", GetQueueStatus(QS_POSTMESSAGE));
    r = MsgWaitForMultipleObjectsEx(0, NULL, 500, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    say("2 then wait QS_ALLINPUT INPUTAVAILABLE: %s", wr(r));
    say("2 PeekMessage finds it: %d", PeekMessageA(&m, NULL, 0, 0, PM_NOREMOVE));
    drain();

    /* 3: two waits in a row with the message still queued and not peeked. */
    PostMessageA(g_hwnd, WM_HAVE_WORK, 0, 0);
    r = MsgWaitForMultipleObjectsEx(0, NULL, 500, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    say("3 first wait: %s", wr(r));
    r = MsgWaitForMultipleObjectsEx(0, NULL, 500, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    say("3 second wait, message still queued: %s", wr(r));
    say("3 GetQueueStatus(QS_ALLINPUT) = %08lx", GetQueueStatus(QS_ALLINPUT));
    drain();

    /* 4: Chrome's pump against a racing ScheduleWork. */
    {
        int stalls = 0, waits = 0, peek_found = 0;
        g_handled = 0;
        g_scheduled = 0;
        HANDLE t = CreateThread(NULL, 0, poster, NULL, 0, NULL);
        for (;;) {
            if (PeekMessageA(&m, NULL, 0, 0, PM_REMOVE)) {
                DispatchMessageA(&m);
                continue;
            }
            if (WaitForSingleObject(t, 0) == WAIT_OBJECT_0 && !g_scheduled)
                break;
            DWORD flags = MWMO_INPUTAVAILABLE;
            for (;;) {
                waits++;
                r = MsgWaitForMultipleObjectsEx(0, NULL, 2000, QS_ALLINPUT, flags);
                if (r == WAIT_OBJECT_0) {
                    if (PeekMessageA(&m, NULL, 0, 0, PM_NOREMOVE))
                        break;
                    flags = 0; /* Chrome: nothing there, so wait for a NEW message */
                    continue;
                }
                if (g_scheduled) {
                    stalls++;
                    peek_found += PeekMessageA(&m, NULL, 0, 0, PM_NOREMOVE) != 0;
                }
                break;
            }
            if (stalls > 20)
                break;
        }
        WaitForSingleObject(t, 10000);
        say("4 stress: lost wake-ups %d (message then found by Peek: %d)", stalls, peek_found);
        say("4 stress: all work handled: %s", g_handled > 0 && !g_scheduled ? "yes" : "NO");
        (void)waits;
    }
    say("done");
    return 0;
}
