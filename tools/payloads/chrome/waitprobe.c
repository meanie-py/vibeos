/* waitprobe: RegisterWaitForSingleObject / UnregisterWaitEx as base::win::ObjectWatcher uses them.
 * Chrome's browser UI thread hung in ObjectWatcher::StopWatching: it passes its own event as the
 * completion event and then waits on it, so an UnregisterWaitEx that never signals that event
 * blocks the thread for good. One line per fact, to diff with waitprobe.win11.txt. */
#include <windows.h>
#include <stdio.h>

static void say(const char *fmt, ...)
{
    char b[400];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(b, sizeof b, fmt, ap);
    va_end(ap);
    printf("[WAIT] %s\n", b);
    fflush(stdout);
}

static volatile LONG g_calls, g_done;
static DWORD g_sleep;

static VOID CALLBACK cb(PVOID ctx, BOOLEAN fired)
{
    (void)ctx;
    (void)fired;
    InterlockedIncrement(&g_calls);
    if (g_sleep)
        Sleep(g_sleep);
    InterlockedIncrement(&g_done);
}

static const char *sig(HANDLE ev, DWORD ms)
{
    DWORD r = WaitForSingleObject(ev, ms);
    return r == WAIT_OBJECT_0 ? "signaled" : r == WAIT_TIMEOUT ? "NOT signaled" : "wait failed";
}

static HANDLE reg(HANDLE obj, DWORD ms, ULONG flags)
{
    HANDLE w = NULL;
    g_calls = g_done = 0;
    if (!RegisterWaitForSingleObject(&w, obj, cb, NULL, ms, flags))
        say("RegisterWaitForSingleObject failed %lu", GetLastError());
    return w;
}

static HANDLE g_taken, g_release;

static DWORD WINAPI take_mutex(LPVOID m)
{
    WaitForSingleObject((HANDLE)m, INFINITE);
    SetEvent(g_taken);
    WaitForSingleObject(g_release, 5000);
    ReleaseMutex((HANDLE)m);
    return 0;
}

/* Wait until a callback has started. */
static void until_called(void)
{
    for (int i = 0; i < 400 && !g_calls; i++)
        Sleep(5);
}

int main(void)
{
    HANDLE obj = CreateEventA(NULL, TRUE, FALSE, NULL);
    HANDLE ce = CreateEventA(NULL, TRUE, FALSE, NULL);
    const ULONG chrome = WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE;
    g_taken = CreateEventA(NULL, FALSE, FALSE, NULL);
    g_release = CreateEventA(NULL, FALSE, FALSE, NULL);
    HANDLE w;
    BOOL ok;

    /* A: ObjectWatcher::StopWatching on a watch that never fired. */
    w = reg(obj, INFINITE, chrome);
    Sleep(50);
    SetLastError(0);
    ok = UnregisterWaitEx(w, ce);
    say("A never fired: UnregisterWaitEx(event) = %d err %lu, event %s", ok, GetLastError(), sig(ce, 2000));
    ResetEvent(ce);

    /* B: the watch fired once (WT_EXECUTEONLYONCE) and its callback returned. */
    w = reg(obj, INFINITE, chrome);
    SetEvent(obj);
    until_called();
    Sleep(50);
    ResetEvent(obj);
    SetLastError(0);
    ok = UnregisterWaitEx(w, ce);
    say("B fired once: calls %ld, UnregisterWaitEx(event) = %d err %lu, event %s", g_calls, ok,
        GetLastError(), sig(ce, 2000));
    ResetEvent(ce);

    /* C: a callback is running when the wait is unregistered with an event. */
    g_sleep = 400;
    w = reg(obj, INFINITE, WT_EXECUTEONLYONCE);
    SetEvent(obj);
    until_called();
    SetLastError(0);
    ok = UnregisterWaitEx(w, ce);
    say("C in callback: UnregisterWaitEx(event) = %d err %lu", ok, GetLastError());
    say("C event right away: %s", sig(ce, 0));
    say("C event once the callback returns: %s, callback done %ld", sig(ce, 3000), g_done);
    ResetEvent(obj);
    ResetEvent(ce);

    /* D: INVALID_HANDLE_VALUE blocks until the running callback returns. */
    w = reg(obj, INFINITE, WT_EXECUTEONLYONCE);
    SetEvent(obj);
    until_called();
    SetLastError(0);
    ok = UnregisterWaitEx(w, INVALID_HANDLE_VALUE);
    say("D in callback: UnregisterWaitEx(INVALID) = %d err %lu, callback done at return %ld", ok,
        GetLastError(), g_done);
    ResetEvent(obj);

    /* E: UnregisterWait (no event) while a callback runs. */
    w = reg(obj, INFINITE, WT_EXECUTEONLYONCE);
    SetEvent(obj);
    until_called();
    SetLastError(0);
    ok = UnregisterWait(w);
    say("E in callback: UnregisterWait = %d err %lu", ok, GetLastError());
    Sleep(600);
    ResetEvent(obj);
    g_sleep = 0;

    /* F: UnregisterWait with nothing running. */
    w = reg(obj, INFINITE, chrome);
    Sleep(50);
    SetLastError(0);
    ok = UnregisterWait(w);
    say("F never fired: UnregisterWait = %d err %lu", ok, GetLastError());

    /* G: a repeating timed wait stops calling back once UnregisterWaitEx(INVALID) returns. */
    w = reg(obj, 20, 0);
    Sleep(200);
    ok = UnregisterWaitEx(w, INVALID_HANDLE_VALUE);
    {
        LONG at = g_calls;
        Sleep(200);
        say("G repeating: UnregisterWaitEx(INVALID) = %d, called back %s, none after: %s", ok,
            at > 2 ? "several times" : "too few", g_calls == at ? "yes" : "NO");
    }

    /* H: register then unregister at once, with the object signaled, racing the wait thread. */
    {
        int unsignaled = 0, failed = 0;
        SetEvent(obj);
        for (int i = 0; i < 300; i++) {
            w = reg(obj, INFINITE, (i & 1) ? chrome : WT_EXECUTEONLYONCE);
            if (i % 3 == 1)
                Sleep(0);
            ok = UnregisterWaitEx(w, ce);
            if (!ok && GetLastError() != ERROR_IO_PENDING)
                failed++;
            if (WaitForSingleObject(ce, 3000) != WAIT_OBJECT_0)
                unsignaled++;
            ResetEvent(ce);
        }
        ResetEvent(obj);
        say("H race x300: completion event never signaled %d, unexpected failures %d", unsignaled, failed);
    }
    /* M: releasing a mutex the caller does not own is an error, not a crash. Our kernel
     * bugchecked (0x1E, STATUS_MUTANT_NOT_OWNED raised and never caught) on the first. */
    {
        HANDLE m = CreateMutexA(NULL, FALSE, NULL);
        SetLastError(0);
        ok = ReleaseMutex(m);
        say("M unowned: ReleaseMutex = %d err %lu", ok, GetLastError());
        HANDLE t = CreateThread(NULL, 0, take_mutex, m, 0, NULL);
        WaitForSingleObject(g_taken, 5000);
        SetLastError(0);
        ok = ReleaseMutex(m);
        say("M owned by another thread: ReleaseMutex = %d err %lu", ok, GetLastError());
        SetEvent(g_release);
        WaitForSingleObject(t, 5000);
        SetLastError(0);
        DWORD r = SignalObjectAndWait(m, ce, 0, FALSE);
        say("M SignalObjectAndWait(unowned mutex) = %#lx err %lu", r, GetLastError());
        WaitForSingleObject(m, 0);
        say("M owned: ReleaseMutex = %d, again = %d err %lu", ReleaseMutex(m), ReleaseMutex(m), GetLastError());
    }
    say("done");
    return 0;
}
