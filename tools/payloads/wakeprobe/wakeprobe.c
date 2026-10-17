/* wakeprobe — how long does a READIED thread take to run?
 *
 * mfplat's winetest expects a work item on an idle pool to run within 100 ms; ours took
 * 0-469 ms, every sample near a multiple of the 15.6 ms clock tick. The cause was a
 * condition-variable wake made under the CV's spin bit (rtl-sync condvar.rs, InternalWake),
 * found with the RIP sampler below. Each stage times the same hand-off over
 * a different path, so the slow one names the layer:
 *   event    two plain threads, SetEvent -> WaitForSingleObject and back (kernel dispatch)
 *   pool     a private pool (1 thread) running a work item that sets an event
 *   blocked  the same, while a second pool's callback is blocked (mfplat's exact shape)
 *   spin     the event hand-off while another thread spins at normal priority
 *   poolspin the pool hand-off while that spinner runs: the unboosted wake
 *   timer    a 10 ms periodic pool timer for a second (mf's presentation-clock shape)
 * The process is pinned to one CPU, so a many-core host behaves like our one vCPU. It waits
 * for the LiveCD's setup.exe to exit (as the apitest sweep does), runs three rounds, and
 * reports how idle the CPU was across each stage: a slow stage on an idle CPU is a wake
 * path waiting for a tick, not a thread waiting its turn.
 * Times are QueryPerformanceCounter microseconds. Verdict PASS when no event or pool stage
 * has a sample over 50 ms; the spinner stages only characterize (Windows 11 also takes
 * about a tick for poolspin). The pool family is resolved by GetProcAddress (kernel32_vista
 * here, kernel32 on Windows). */
#define _WIN32_WINNT 0x0601
#include <windows.h>
#include <tlhelp32.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
static void say(const char *fmt, ...)
{
    char buf[256];
    va_list ap;
    va_start(ap, fmt);
    _vsnprintf(buf, sizeof(buf) - 1, fmt, ap);
    va_end(ap);
    buf[sizeof(buf) - 1] = 0;
    DbgPrint("[WAKEPROBE] %s", buf);
    fputs("[WAKEPROBE] ", stdout);
    fputs(buf, stdout);
    fflush(stdout);
}

#define N 64
static LARGE_INTEGER freq;
static HANDLE ping, pong, blocker, spin_stop;
static volatile LONG spinning;

static PTP_POOL (WINAPI *pCreateThreadpool)(PVOID);
static VOID (WINAPI *pCloseThreadpool)(PTP_POOL);
static BOOL (WINAPI *pSetThreadpoolThreadMinimum)(PTP_POOL, DWORD);
static VOID (WINAPI *pSetThreadpoolThreadMaximum)(PTP_POOL, DWORD);
static PTP_WORK (WINAPI *pCreateThreadpoolWork)(PTP_WORK_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static VOID (WINAPI *pSubmitThreadpoolWork)(PTP_WORK);
static VOID (WINAPI *pWaitForThreadpoolWorkCallbacks)(PTP_WORK, BOOL);
static VOID (WINAPI *pCloseThreadpoolWork)(PTP_WORK);
static LONG (NTAPI *pNtQuerySystemInformation)(ULONG, PVOID, ULONG, PULONG);
static BOOL (WINAPI *pSleepConditionVariableCS)(PCONDITION_VARIABLE, PCRITICAL_SECTION, DWORD);
static PTP_TIMER (WINAPI *pCreateThreadpoolTimer)(PTP_TIMER_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static VOID (WINAPI *pSetThreadpoolTimer)(PTP_TIMER, PFILETIME, DWORD, DWORD);
static VOID (WINAPI *pWaitForThreadpoolTimerCallbacks)(PTP_TIMER, BOOL);
static VOID (WINAPI *pCloseThreadpoolTimer)(PTP_TIMER);

/* SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION; KernelTime includes IdleTime. */
typedef struct { LARGE_INTEGER Idle, Kernel, User, Dpc, Intr; ULONG Ints; } SPPI;
static SPPI cpu_before;

static void cpu_mark(void)
{
    ULONG len;
    if (pNtQuerySystemInformation)
        pNtQuerySystemInformation(8, &cpu_before, sizeof(cpu_before), &len);
}

static int cpu_idle_pct(void)
{
    SPPI now;
    ULONG len;
    LONGLONG busy, idle;
    if (!pNtQuerySystemInformation || pNtQuerySystemInformation(8, &now, sizeof(now), &len) < 0)
        return -1;
    idle = now.Idle.QuadPart - cpu_before.Idle.QuadPart;
    busy = (now.Kernel.QuadPart - cpu_before.Kernel.QuadPart) + (now.User.QuadPart - cpu_before.User.QuadPart);
    return busy > 0 ? (int)(idle * 100 / busy) : -1;
}

static int setup_running(void)
{
    PROCESSENTRY32W pe;
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    int found = 0;
    if (snap == INVALID_HANDLE_VALUE)
        return 0;
    pe.dwSize = sizeof(pe);
    if (Process32FirstW(snap, &pe))
        do found = !lstrcmpiW(pe.szExeFile, L"setup.exe"); while (!found && Process32NextW(snap, &pe));
    CloseHandle(snap);
    return found;
}

/* SYSTEM_PROCESS_INFORMATION / SYSTEM_THREAD_INFORMATION, NT5 layout (as ../cpuprof). */
typedef struct {
    LARGE_INTEGER KernelTime, UserTime, CreateTime;
    ULONG WaitTime;
    PVOID StartAddress;
    struct { HANDLE Process, Thread; } ClientId;
    LONG Priority, BasePriority;
    ULONG ContextSwitches, ThreadState, WaitReason;
} STI;
typedef struct {
    ULONG NextEntryOffset, NumberOfThreads;
    LARGE_INTEGER Reserved[3];
    LARGE_INTEGER CreateTime, UserTime, KernelTime;
    struct { USHORT Length, MaximumLength; PWSTR Buffer; } ImageName;
    LONG BasePriority;
    HANDLE UniqueProcessId, InheritedFromUniqueProcessId;
    ULONG HandleCount, SessionId;
    ULONG_PTR PageDirectoryBase;
    SIZE_T PeakVirtualSize, VirtualSize;
    ULONG PageFaultCount;
    SIZE_T PeakWorkingSetSize, WorkingSetSize, QuotaPeakPagedPoolUsage, QuotaPagedPoolUsage,
           QuotaPeakNonPagedPoolUsage, QuotaNonPagedPoolUsage, PagefileUsage, PeakPagefileUsage,
           PrivatePageCount;
    LARGE_INTEGER ReadOperationCount, WriteOperationCount, OtherOperationCount,
                  ReadTransferCount, WriteTransferCount, OtherTransferCount;
    STI Threads[1];
} SPI;

/* Our own threads with their CPU so far: a pool worker that never sleeps shows here. */
static void dump_threads(void)
{
    static BYTE buf[1 << 20];
    ULONG len, k;
    SPI *p;
    if (!pNtQuerySystemInformation || pNtQuerySystemInformation(5, buf, sizeof(buf), &len) < 0)
        return;
    for (p = (SPI *)buf;; p = (SPI *)((BYTE *)p + p->NextEntryOffset)) {
        if ((ULONG_PTR)p->UniqueProcessId == GetCurrentProcessId()) {
            for (k = 0; k < p->NumberOfThreads; k++) {
                STI *t = &p->Threads[k];
                say("  thr %lu cpu=%llums switches=%lu state=%lu wait=%lu prio=%ld start=%p\n",
                    (ULONG)(ULONG_PTR)t->ClientId.Thread,
                    (ULONGLONG)(t->KernelTime.QuadPart + t->UserTime.QuadPart) / 10000, t->ContextSwitches,
                    t->ThreadState, t->WaitReason, t->Priority, t->StartAddress);
            }
            return;
        }
        if (!p->NextEntryOffset)
            return;
    }
}

static ULONGLONG thread_cpu(void)
{
    FILETIME c, e, k, u;
    GetThreadTimes(GetCurrentThread(), &c, &e, &k, &u);
    return (((ULONGLONG)k.dwHighDateTime << 32) | k.dwLowDateTime) + (((ULONGLONG)u.dwHighDateTime << 32) | u.dwLowDateTime);
}

/* RIP sampler for the main thread: where does it spend CPU inside the pool calls? */
#define MAXSAMP 8192
static HANDLE main_thread;
static volatile LONG sampling;
static ULONG_PTR samples[MAXSAMP];
static volatile LONG nsamp;

static DWORD WINAPI sampler(LPVOID unused)
{
    (void)unused;
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
    for (;;) {
        CONTEXT ctx;
        Sleep(2);
        if (!sampling || nsamp >= MAXSAMP)
            continue;
        if (SuspendThread(main_thread) == (DWORD)-1)
            continue;
        ctx.ContextFlags = CONTEXT_CONTROL;
        if (GetThreadContext(main_thread, &ctx))
#if defined(_M_AMD64) || defined(__x86_64__)
            samples[nsamp++] = ctx.Rip;
#elif defined(_M_ARM64) || defined(__aarch64__)
            samples[nsamp++] = ctx.Pc;
#else
            samples[nsamp++] = ctx.Eip;
#endif
        ResumeThread(main_thread);
    }
}

static void sample_report(void)
{
    int i, j, printed = 0;
    say("main-thread samples: %ld\n", nsamp);
    while (printed < 12) {
        ULONG_PTR best = 0;
        int bestn = 0;
        for (i = 0; i < nsamp; i++) {
            int n = 0;
            if (!samples[i]) continue;
            for (j = i; j < nsamp; j++) if (samples[j] == samples[i]) n++;
            if (n > bestn) { bestn = n; best = samples[i]; }
        }
        if (!bestn) break;
        {
            HMODULE m = NULL;
            char name[MAX_PATH] = "?";
            if (GetModuleHandleExA(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                                   (LPCSTR)best, &m))
                GetModuleFileNameA(m, name, sizeof(name));
            say("  %5d x %p  %s+0x%lx\n", bestn, (void *)best, strrchr(name, '\\') ? strrchr(name, '\\') + 1 : name,
                (ULONG)(best - (ULONG_PTR)m));
        }
        for (i = 0; i < nsamp; i++) if (samples[i] == best) samples[i] = 0;
        printed++;
    }
    nsamp = 0;
}

static double us(LONGLONG ticks) { return ticks * 1e6 / (double)freq.QuadPart; }

static int cmp(const void *a, const void *b)
{
    double x = *(const double *)a, y = *(const double *)b;
    return x < y ? -1 : x > y;
}

static int report(const char *name, double *s)
{
    int over_tick = 0, i;
    qsort(s, N, sizeof(*s), cmp);
    for (i = 0; i < N; i++)
        if (s[i] > 15600.0)
            over_tick++;
    say("%-8s min=%.0fus median=%.0fus max=%.0fus over-one-tick=%d/%d cpu-idle=%d%%\n", name, s[0], s[N / 2],
        s[N - 1], over_tick, N, cpu_idle_pct());
    return s[N - 1] > 50000.0;
}

static DWORD WINAPI echo(LPVOID unused)
{
    (void)unused;
    for (;;) {
        WaitForSingleObject(ping, INFINITE);
        SetEvent(pong);
    }
}

static DWORD WINAPI spinner(LPVOID unused)
{
    (void)unused;
    InterlockedIncrement(&spinning);
    while (WaitForSingleObject(spin_stop, 0) == WAIT_TIMEOUT) { }
    return 0;
}

static void CALLBACK set_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_WORK work)
{
    (void)inst; (void)work;
    SetEvent((HANDLE)ctx);
}

static void CALLBACK block_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_WORK work)
{
    (void)inst; (void)ctx; (void)work;
    WaitForSingleObject(blocker, 30000);
}

static int event_stage(const char *name)
{
    double s[N];
    LARGE_INTEGER a, b;
    int i;
    cpu_mark();
    for (i = 0; i < N; i++) {
        QueryPerformanceCounter(&a);
        SetEvent(ping);
        WaitForSingleObject(pong, INFINITE);
        QueryPerformanceCounter(&b);
        s[i] = us(b.QuadPart - a.QuadPart);
        Sleep(1);
    }
    return report(name, s);
}

static int pool_stage(const char *name, BOOL with_blocked)
{
    TP_CALLBACK_ENVIRON env, env2;
    PTP_POOL pool = pCreateThreadpool(NULL), pool2 = NULL;
    PTP_WORK work, blockwork = NULL;
    HANDLE done = CreateEventW(NULL, FALSE, FALSE, NULL);
    double s[N];
    LARGE_INTEGER a, b;
    int i, bad;
    ULONGLONG cpu_submit = 0, cpu_wait = 0, cpu_waitcb = 0;

    InitializeThreadpoolEnvironment(&env);
    SetThreadpoolCallbackPool(&env, pool);
    pSetThreadpoolThreadMinimum(pool, 1);
    pSetThreadpoolThreadMaximum(pool, 1);
    work = pCreateThreadpoolWork(set_cb, done, &env);
    if (with_blocked) {
        ResetEvent(blocker);
        pool2 = pCreateThreadpool(NULL);
        InitializeThreadpoolEnvironment(&env2);
        SetThreadpoolCallbackPool(&env2, pool2);
        pSetThreadpoolThreadMinimum(pool2, 1);
        pSetThreadpoolThreadMaximum(pool2, 4);
        blockwork = pCreateThreadpoolWork(block_cb, NULL, &env2);
        pSubmitThreadpoolWork(blockwork);
        Sleep(50);
    }
    cpu_mark();
    for (i = 0; i < N; i++) {
        ULONGLONG t0 = thread_cpu(), t1, t2, t3;
        QueryPerformanceCounter(&a);
        pSubmitThreadpoolWork(work);
        t1 = thread_cpu();
        WaitForSingleObject(done, INFINITE);
        t2 = thread_cpu();
        QueryPerformanceCounter(&b);
        s[i] = us(b.QuadPart - a.QuadPart);
        pWaitForThreadpoolWorkCallbacks(work, FALSE);
        t3 = thread_cpu();
        cpu_submit += t1 - t0; cpu_wait += t2 - t1; cpu_waitcb += t3 - t2;
        Sleep(1);
    }
    bad = report(name, s);
    say("  main-thread cpu: submit=%llums wait=%llums waitcallbacks=%llums\n", cpu_submit / 10000, cpu_wait / 10000,
        cpu_waitcb / 10000);
    if (with_blocked) {
        SetEvent(blocker);
        pWaitForThreadpoolWorkCallbacks(blockwork, FALSE);
        pCloseThreadpoolWork(blockwork);
        pCloseThreadpool(pool2);
    }
    pCloseThreadpoolWork(work);
    pCloseThreadpool(pool);
    CloseHandle(done);
    return bad;
}

/* A condition-variable sleep must SLEEP: 50 ms of wall time and ~0 CPU, then ERROR_TIMEOUT. */
static int cv_stage(void)
{
    CONDITION_VARIABLE cv = CONDITION_VARIABLE_INIT;
    CRITICAL_SECTION cs;
    FILETIME c, e, k0, u0, k1, u1;
    LARGE_INTEGER a, b;
    BOOL ret;
    DWORD err;
    int i, bad = 0;

    InitializeCriticalSection(&cs);
    for (i = 0; i < 4; i++) {
        EnterCriticalSection(&cs);
        GetThreadTimes(GetCurrentThread(), &c, &e, &k0, &u0);
        QueryPerformanceCounter(&a);
        ret = pSleepConditionVariableCS(&cv, &cs, 50);
        err = GetLastError();
        QueryPerformanceCounter(&b);
        GetThreadTimes(GetCurrentThread(), &c, &e, &k1, &u1);
        LeaveCriticalSection(&cs);
        say("cv       sleep(50ms) ret=%d err=%lu wall=%.0fus cpu=%llums\n", ret, err, us(b.QuadPart - a.QuadPart),
            ((((ULONGLONG)k1.dwHighDateTime << 32) | k1.dwLowDateTime) - (((ULONGLONG)k0.dwHighDateTime << 32) | k0.dwLowDateTime) +
             (((ULONGLONG)u1.dwHighDateTime << 32) | u1.dwLowDateTime) - (((ULONGLONG)u0.dwHighDateTime << 32) | u0.dwLowDateTime)) / 10000);
        bad |= ret || err != ERROR_TIMEOUT || us(b.QuadPart - a.QuadPart) < 40000.0;
    }
    DeleteCriticalSection(&cs);
    return bad;
}

/* A 10 ms periodic timer on a private 1-thread pool (rtworkq's periodic-callback shape, which
 * mf's presentation clock runs on) for one second: count, mean interval, largest gap. */
static volatile LONG timer_ticks;
static LARGE_INTEGER timer_last;
static volatile LONGLONG timer_maxgap;

static void CALLBACK tick_cb(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_TIMER timer)
{
    LARGE_INTEGER now;
    (void)inst; (void)ctx; (void)timer;
    QueryPerformanceCounter(&now);
    if (timer_ticks && now.QuadPart - timer_last.QuadPart > timer_maxgap)
        timer_maxgap = now.QuadPart - timer_last.QuadPart;
    timer_last = now;
    InterlockedIncrement(&timer_ticks);
}

static int timer_stage(void)
{
    TP_CALLBACK_ENVIRON env;
    PTP_POOL pool = pCreateThreadpool(NULL);
    PTP_TIMER timer;
    FILETIME due = { 0, 0 };
    LARGE_INTEGER a, b;
    LONG n;

    InitializeThreadpoolEnvironment(&env);
    SetThreadpoolCallbackPool(&env, pool);
    pSetThreadpoolThreadMinimum(pool, 1);
    pSetThreadpoolThreadMaximum(pool, 1);
    timer_ticks = 0;
    timer_maxgap = 0;
    timer = pCreateThreadpoolTimer(tick_cb, NULL, &env);
    cpu_mark();
    QueryPerformanceCounter(&a);
    pSetThreadpoolTimer(timer, &due, 10, 0);
    Sleep(1000);
    pSetThreadpoolTimer(timer, NULL, 0, 0);
    pWaitForThreadpoolTimerCallbacks(timer, TRUE);
    QueryPerformanceCounter(&b);
    n = timer_ticks;
    say("timer    10ms periodic for 1s: %ld callbacks, mean=%.0fus maxgap=%.0fus cpu-idle=%d%%\n", n,
        n ? us(b.QuadPart - a.QuadPart) / n : 0.0, us(timer_maxgap), cpu_idle_pct());
    pCloseThreadpoolTimer(timer);
    pCloseThreadpool(pool);
    /* Windows' 15.6 ms tick gives ~64; a stall shows as a gap of many ticks. */
    return n < 40 || us(timer_maxgap) > 100000.0;
}

#define RESOLVE(name) (*(FARPROC *)&p##name = GetProcAddress(k, #name))

int main(void)
{
    HMODULE k = LoadLibraryA("kernel32_vista.dll");
    int bad = 0, round, waited = 0;

    if (!k)
        k = GetModuleHandleA("kernel32.dll");
    if (!RESOLVE(CreateThreadpool) || !RESOLVE(CloseThreadpool) || !RESOLVE(SetThreadpoolThreadMinimum) ||
        !RESOLVE(SetThreadpoolThreadMaximum) || !RESOLVE(CreateThreadpoolWork) || !RESOLVE(SubmitThreadpoolWork) ||
        !RESOLVE(WaitForThreadpoolWorkCallbacks) || !RESOLVE(CloseThreadpoolWork) ||
        !RESOLVE(SleepConditionVariableCS) || !RESOLVE(CreateThreadpoolTimer) || !RESOLVE(SetThreadpoolTimer) ||
        !RESOLVE(WaitForThreadpoolTimerCallbacks) || !RESOLVE(CloseThreadpoolTimer)) {
        say("missing a threadpool export\n");
        say("done VERDICT=FAIL\n");
        return 1;
    }
    *(FARPROC *)&pNtQuerySystemInformation = GetProcAddress(GetModuleHandleA("ntdll.dll"), "NtQuerySystemInformation");
    QueryPerformanceFrequency(&freq);
    SetProcessAffinityMask(GetCurrentProcess(), 1);
    ping = CreateEventW(NULL, FALSE, FALSE, NULL);
    pong = CreateEventW(NULL, FALSE, FALSE, NULL);
    blocker = CreateEventW(NULL, TRUE, FALSE, NULL);
    spin_stop = CreateEventW(NULL, TRUE, FALSE, NULL);
    CloseHandle(CreateThread(NULL, 0, echo, NULL, 0, NULL));

    while (setup_running() && waited < 900) {
        Sleep(5000);
        waited += 5;
    }
    say("setup.exe gone after ~%ds\n", waited);

    bad |= cv_stage();
    DuplicateHandle(GetCurrentProcess(), GetCurrentThread(), GetCurrentProcess(), &main_thread, 0, FALSE,
                    DUPLICATE_SAME_ACCESS);
    CloseHandle(CreateThread(NULL, 0, sampler, NULL, 0, NULL));
    for (round = 1; round <= 3; round++) {
        say("round %d\n", round);
        bad |= event_stage("event");
        sampling = 1;
        bad |= pool_stage("pool", FALSE);
        bad |= pool_stage("blocked", TRUE);
        bad |= timer_stage();
        sampling = 0;
        if (round == 1)
            sample_report();
        Sleep(2000);
        dump_threads();
    }

    CloseHandle(CreateThread(NULL, 0, spinner, NULL, 0, NULL));
    while (!spinning)
        Sleep(1);
    event_stage("spin");
    pool_stage("poolspin", FALSE);
    SetEvent(spin_stop);

    say("done VERDICT=%s\n", bad ? "FAIL" : "PASS");
    return 0;
}
