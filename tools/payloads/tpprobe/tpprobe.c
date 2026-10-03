/* tpprobe — does the Vista threadpool actually run callbacks?
 *
 * kernel32_vista's CreateThreadpool* wrappers sit on ntdll_vista's Tp* exports, whose bodies are
 * the rtl-threadpool crate: ntdll_vista links its OWN copy of that pool, which needs its own
 * RtlpInitializeThreadPooling (upstream's DllMain never calls it, leaving the timer and wait
 * queues zeroed). Work objects never touch those queues, so each object type gets its own stage.
 * Everything is resolved with GetProcAddress: our kernel32.dll does not export the family at
 * 0x502, and an import would fail the whole process instead of naming the missing function.
 * Verdict: "[TPPROBE] done VERDICT=PASS|FAIL". */
#define _WIN32_WINNT 0x0601
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[TPPROBE] " __VA_ARGS__)
#include "../stackdump.h"

static PTP_WORK (WINAPI *pCreateThreadpoolWork)(PTP_WORK_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static VOID (WINAPI *pSubmitThreadpoolWork)(PTP_WORK);
static VOID (WINAPI *pWaitForThreadpoolWorkCallbacks)(PTP_WORK, BOOL);
static VOID (WINAPI *pCloseThreadpoolWork)(PTP_WORK);
static BOOL (WINAPI *pTrySubmitThreadpoolCallback)(PTP_SIMPLE_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static PTP_TIMER (WINAPI *pCreateThreadpoolTimer)(PTP_TIMER_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static VOID (WINAPI *pSetThreadpoolTimer)(PTP_TIMER, PFILETIME, DWORD, DWORD);
static VOID (WINAPI *pWaitForThreadpoolTimerCallbacks)(PTP_TIMER, BOOL);
static VOID (WINAPI *pCloseThreadpoolTimer)(PTP_TIMER);
static PTP_WAIT (WINAPI *pCreateThreadpoolWait)(PTP_WAIT_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static VOID (WINAPI *pSetThreadpoolWait)(PTP_WAIT, HANDLE, PFILETIME);
static VOID (WINAPI *pWaitForThreadpoolWaitCallbacks)(PTP_WAIT, BOOL);
static VOID (WINAPI *pCloseThreadpoolWait)(PTP_WAIT);
static PTP_POOL (WINAPI *pCreateThreadpool)(PVOID);
static VOID (WINAPI *pCloseThreadpool)(PTP_POOL);
static BOOL (WINAPI *pSetThreadpoolThreadMinimum)(PTP_POOL, DWORD);
static VOID (WINAPI *pSetThreadpoolThreadMaximum)(PTP_POOL, DWORD);
static PTP_CLEANUP_GROUP (WINAPI *pCreateThreadpoolCleanupGroup)(VOID);
static VOID (WINAPI *pCloseThreadpoolCleanupGroupMembers)(PTP_CLEANUP_GROUP, BOOL, PVOID);
static VOID (WINAPI *pCloseThreadpoolCleanupGroup)(PTP_CLEANUP_GROUP);

static volatile LONG stage, finished, failures, work_runs, simple_runs, timer_runs, wait_runs,
    group_runs;
static const char *const stage_names[] = { "resolve", "work", "simple", "timer", "wait", "pool+group" };
static HANDLE simple_done, timer_done, wait_event, wait_done;
static DWORD main_tid;

#define CHECK(cond, ...) do { if (!(cond)) { SAY("FAIL " __VA_ARGS__); InterlockedIncrement(&failures); } } while (0)

static DWORD WINAPI watchdog(LPVOID unused)
{
    LONG last = -1, same = 0;
    (void)unused;
    while (!finished) {
        Sleep(5000);
        LONG now = stage;
        if (now == last && ++same == 4) {
            SAY("STUCK in stage %s for 20s\n", stage_names[now]);
            dump_thread(GetCurrentProcessId(), main_tid);
            SAY("done VERDICT=FAIL\n");
            ExitProcess(1);
        }
        if (now != last) same = 0;
        last = now;
    }
    return 0;
}

static void CALLBACK work_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_WORK work)
{
    (void)inst; (void)work;
    InterlockedIncrement((LONG *)ctx);
}

static void CALLBACK simple_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx)
{
    (void)inst;
    InterlockedIncrement(&simple_runs);
    SetEvent((HANDLE)ctx);
}

static void CALLBACK timer_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_TIMER timer)
{
    (void)inst; (void)timer;
    InterlockedIncrement(&timer_runs);
    SetEvent((HANDLE)ctx);
}

static void CALLBACK wait_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_WAIT wait, TP_WAIT_RESULT result)
{
    (void)inst; (void)wait;
    if (result == WAIT_OBJECT_0)
        InterlockedIncrement(&wait_runs);
    SetEvent((HANDLE)ctx);
}

#define RESOLVE(mod, name) \
    do { *(FARPROC *)&p##name = GetProcAddress(mod, #name); CHECK(p##name, "missing export %s\n", #name); } while (0)

int main(void)
{
    main_tid = GetCurrentThreadId();
    CreateThread(NULL, 0, watchdog, NULL, 0, NULL);

    HMODULE k = LoadLibraryA("kernel32_vista.dll");
    SAY("kernel32_vista.dll -> %p (err %lu)\n", (void *)k, k ? 0 : GetLastError());
    if (!k) {
        SAY("done VERDICT=FAIL\n");
        return 1;
    }
    RESOLVE(k, CreateThreadpoolWork); RESOLVE(k, SubmitThreadpoolWork);
    RESOLVE(k, WaitForThreadpoolWorkCallbacks); RESOLVE(k, CloseThreadpoolWork);
    RESOLVE(k, TrySubmitThreadpoolCallback);
    RESOLVE(k, CreateThreadpoolTimer); RESOLVE(k, SetThreadpoolTimer);
    RESOLVE(k, WaitForThreadpoolTimerCallbacks); RESOLVE(k, CloseThreadpoolTimer);
    RESOLVE(k, CreateThreadpoolWait); RESOLVE(k, SetThreadpoolWait);
    RESOLVE(k, WaitForThreadpoolWaitCallbacks); RESOLVE(k, CloseThreadpoolWait);
    RESOLVE(k, CreateThreadpool); RESOLVE(k, CloseThreadpool);
    RESOLVE(k, SetThreadpoolThreadMinimum); RESOLVE(k, SetThreadpoolThreadMaximum);
    RESOLVE(k, CreateThreadpoolCleanupGroup); RESOLVE(k, CloseThreadpoolCleanupGroupMembers);
    RESOLVE(k, CloseThreadpoolCleanupGroup);
    if (failures) {
        SAY("done VERDICT=FAIL\n");
        return 1;
    }

    /* work: one object submitted 16 times; the wait must see every callback. */
    stage = 1;
    PTP_WORK work = pCreateThreadpoolWork(work_cb, (PVOID)&work_runs, NULL);
    CHECK(work, "CreateThreadpoolWork err %lu\n", GetLastError());
    if (work) {
        for (int i = 0; i < 16; i++)
            pSubmitThreadpoolWork(work);
        pWaitForThreadpoolWorkCallbacks(work, FALSE);
        CHECK(work_runs == 16, "work ran %ld of 16\n", work_runs);
        pCloseThreadpoolWork(work);
    }
    SAY("work: %ld callbacks\n", work_runs);

    /* simple: fire-and-forget. */
    stage = 2;
    simple_done = CreateEventA(NULL, TRUE, FALSE, NULL);
    CHECK(pTrySubmitThreadpoolCallback(simple_cb, simple_done, NULL), "TrySubmit err %lu\n", GetLastError());
    CHECK(WaitForSingleObject(simple_done, 10000) == WAIT_OBJECT_0, "simple callback never ran\n");
    SAY("simple: %ld callbacks\n", simple_runs);

    /* timer: 50ms relative, one shot. Needs the timer queue initialised. */
    stage = 3;
    timer_done = CreateEventA(NULL, TRUE, FALSE, NULL);
    PTP_TIMER timer = pCreateThreadpoolTimer(timer_cb, timer_done, NULL);
    CHECK(timer, "CreateThreadpoolTimer err %lu\n", GetLastError());
    if (timer) {
        ULARGE_INTEGER due;
        FILETIME ft;
        due.QuadPart = (ULONGLONG)-(50 * 10000LL);
        ft.dwLowDateTime = due.LowPart;
        ft.dwHighDateTime = due.HighPart;
        pSetThreadpoolTimer(timer, &ft, 0, 0);
        CHECK(WaitForSingleObject(timer_done, 10000) == WAIT_OBJECT_0, "timer never fired\n");
        pWaitForThreadpoolTimerCallbacks(timer, FALSE);
        pCloseThreadpoolTimer(timer);
    }
    SAY("timer: %ld callbacks\n", timer_runs);

    /* wait: a callback when an event we signal becomes signalled. Needs the wait queue. */
    stage = 4;
    wait_event = CreateEventA(NULL, TRUE, FALSE, NULL);
    wait_done = CreateEventA(NULL, TRUE, FALSE, NULL);
    PTP_WAIT wait = pCreateThreadpoolWait(wait_cb, wait_done, NULL);
    CHECK(wait, "CreateThreadpoolWait err %lu\n", GetLastError());
    if (wait) {
        pSetThreadpoolWait(wait, wait_event, NULL);
        SetEvent(wait_event);
        CHECK(WaitForSingleObject(wait_done, 10000) == WAIT_OBJECT_0, "wait callback never ran\n");
        CHECK(wait_runs == 1, "wait callback saw a non-signal result\n");
        pWaitForThreadpoolWaitCallbacks(wait, FALSE);
        pCloseThreadpoolWait(wait);
    }
    SAY("wait: %ld callbacks\n", wait_runs);

    /* pool+group: work on a private 1..2-thread pool, torn down through its cleanup group. */
    stage = 5;
    PTP_POOL pool = pCreateThreadpool(NULL);
    PTP_CLEANUP_GROUP group = pCreateThreadpoolCleanupGroup();
    CHECK(pool && group, "CreateThreadpool/CleanupGroup err %lu\n", GetLastError());
    if (pool && group) {
        TP_CALLBACK_ENVIRON env;
        pSetThreadpoolThreadMaximum(pool, 2);
        CHECK(pSetThreadpoolThreadMinimum(pool, 1), "SetThreadpoolThreadMinimum err %lu\n", GetLastError());
        InitializeThreadpoolEnvironment(&env);
        SetThreadpoolCallbackPool(&env, pool);
        SetThreadpoolCallbackCleanupGroup(&env, group, NULL);
        PTP_WORK gw = pCreateThreadpoolWork(work_cb, (PVOID)&group_runs, &env);
        CHECK(gw, "CreateThreadpoolWork(env) err %lu\n", GetLastError());
        if (gw) {
            for (int i = 0; i < 8; i++)
                pSubmitThreadpoolWork(gw);
            /* Releases gw too: the group owns it now. */
            pCloseThreadpoolCleanupGroupMembers(group, FALSE, NULL);
            CHECK(group_runs == 8, "group work ran %ld of 8\n", group_runs);
        }
        pCloseThreadpoolCleanupGroup(group);
        pCloseThreadpool(pool);
    }
    SAY("pool+group: %ld callbacks\n", group_runs);

    finished = 1;
    SAY("done VERDICT=%s\n", failures ? "FAIL" : "PASS");
    return failures ? 1 : 0;
}
