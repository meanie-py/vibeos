/* clockprobe — do the guest's clocks agree, and does a 10 ms periodic pool timer keep up?
 *
 * mf.c:4553 measures the presentation clock's 10 ms periodic callback by MFGetSystemTime
 * (QueryPerformanceCounter) across a Sleep(500), and reads 20-40 ms per callback in the guest.
 * Pool-latency logging showed no callback holding rtworkq's one-thread timer pool, so this
 * asks the clocks themselves: across each Sleep(500), how far did QPC, GetTickCount and the
 * interrupt time (KUSER_SHARED_DATA) move, and how many 10 ms timer callbacks ran, on a
 * private pool with max 1 thread (rtworkq's TIMER queue shape).
 * Verdict: "[CLOCKPROBE] done". */
#define _WIN32_WINNT 0x0601
#include <windows.h>
#include <tlhelp32.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

/* DbgPrint for the guest's serial; stdout too, for `vibeos-cli winref run` on Windows, where
 * DbgPrint reaches nothing without a kernel debugger (the apitest sweep's emit()). */
static void SAY(const char *Format, ...)
{
    char line[512];
    va_list args;
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    int len = lstrlenA(lstrcpyA(line, "[CLOCKPROBE] "));
    va_start(args, Format);
    wvsprintfA(line + len, Format, args);
    va_end(args);
    DbgPrint("%s", line);
    if (out && out != INVALID_HANDLE_VALUE)
        WriteFile(out, line, lstrlenA(line), &n, NULL);
}

/* Rounds 10-19 run beside a thread that spins at normal priority: does a periodic timer's
 * callback still get the CPU when someone else is running? */
static volatile LONG hog_on;
static DWORD WINAPI hog(void *arg)
{
    volatile ULONG x = 0;
    for (;;)
    {
        while (hog_on)
            ++x;
        Sleep(10);
    }
    return 0;
}

static PTP_TIMER (WINAPI *pCreateThreadpoolTimer)(PTP_TIMER_CALLBACK, PVOID, PTP_CALLBACK_ENVIRON);
static VOID (WINAPI *pSetThreadpoolTimer)(PTP_TIMER, PFILETIME, DWORD, DWORD);
static PTP_POOL (WINAPI *pCreateThreadpool)(PVOID);
static BOOL (WINAPI *pSetThreadpoolThreadMinimum)(PTP_POOL, DWORD);
static VOID (WINAPI *pSetThreadpoolThreadMaximum)(PTP_POOL, DWORD);

static volatile LONG calls;
static LARGE_INTEGER freq;

static ULONGLONG interrupt_time(void)
{
    /* KUSER_SHARED_DATA.InterruptTime at 0x7FFE0008: High1Time/LowPart/High2Time. */
    volatile ULONG *p = (volatile ULONG *)0x7FFE0008;
    ULONG lo, hi1, hi2;
    do
    {
        hi1 = p[1];
        lo = p[0];
        hi2 = p[2];
    } while (hi1 != hi2);
    return ((ULONGLONG)hi1 << 32) | lo;
}

/* The LiveCD's setup.exe registers components after the desktop is up; the apitest sweep
 * waits for it, so a probe that does not measures a contended CPU (the first run did). */
static void wait_for_setup(void)
{
    int waited = 0;
    for (;;)
    {
        PROCESSENTRY32W pe = { sizeof pe };
        HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        int found = 0;
        if (snap != INVALID_HANDLE_VALUE)
        {
            for (BOOL k = Process32FirstW(snap, &pe); k; k = Process32NextW(snap, &pe))
                if (!lstrcmpiW(pe.szExeFile, L"setup.exe"))
                    found = 1;
            CloseHandle(snap);
        }
        if (!found || waited >= 900)
            break;
        Sleep(5000);
        waited += 5;
    }
    SAY("setup.exe gone after ~%ds\n", waited);
}

static ULONGLONG ft(FILETIME f)
{
    return ((ULONGLONG)f.dwHighDateTime << 32) | f.dwLowDateTime;
}

static void CALLBACK on_timer(PTP_CALLBACK_INSTANCE inst, PVOID ctx, PTP_TIMER timer)
{
    InterlockedIncrement(&calls);
}

int main(void)
{
    HMODULE k32 = GetModuleHandleA("kernel32.dll");
    TP_CALLBACK_ENVIRON env;
    PTP_POOL pool;
    PTP_TIMER timer;
    FILETIME due = { 0, 0 };
    int i;

    pCreateThreadpoolTimer = (void *)GetProcAddress(k32, "CreateThreadpoolTimer");
    pSetThreadpoolTimer = (void *)GetProcAddress(k32, "SetThreadpoolTimer");
    pCreateThreadpool = (void *)GetProcAddress(k32, "CreateThreadpool");
    pSetThreadpoolThreadMinimum = (void *)GetProcAddress(k32, "SetThreadpoolThreadMinimum");
    pSetThreadpoolThreadMaximum = (void *)GetProcAddress(k32, "SetThreadpoolThreadMaximum");
    if (!pCreateThreadpoolTimer || !pSetThreadpoolTimer || !pCreateThreadpool)
    {
        HMODULE k32v = LoadLibraryA("kernel32_vista.dll");
        pCreateThreadpoolTimer = (void *)GetProcAddress(k32v, "CreateThreadpoolTimer");
        pSetThreadpoolTimer = (void *)GetProcAddress(k32v, "SetThreadpoolTimer");
        pCreateThreadpool = (void *)GetProcAddress(k32v, "CreateThreadpool");
        pSetThreadpoolThreadMinimum = (void *)GetProcAddress(k32v, "SetThreadpoolThreadMinimum");
        pSetThreadpoolThreadMaximum = (void *)GetProcAddress(k32v, "SetThreadpoolThreadMaximum");
    }
    if (!pCreateThreadpoolTimer || !pSetThreadpoolTimer || !pCreateThreadpool)
    {
        SAY("done VERDICT=FAIL (threadpool API not found)\n");
        return 1;
    }
    wait_for_setup();
    Sleep(10000);
    QueryPerformanceFrequency(&freq);
    SAY("qpc frequency %lu kHz\n", (ULONG)(freq.QuadPart / 1000));

    pool = pCreateThreadpool(NULL);
    pSetThreadpoolThreadMinimum(pool, 1);
    pSetThreadpoolThreadMaximum(pool, 1);
    InitializeThreadpoolEnvironment(&env);
    SetThreadpoolCallbackPool(&env, pool);
    timer = pCreateThreadpoolTimer(on_timer, NULL, &env);
    pSetThreadpoolTimer(timer, &due, 10, 0);
    Sleep(200);

    CloseHandle(CreateThread(NULL, 0, hog, NULL, 0, NULL));
    for (i = 0; i < 20; ++i)
    {
        hog_on = (i >= 10);
        LARGE_INTEGER q0, q1;
        ULONGLONG it0, it1;
        DWORD t0, t1;
        LONG c0, c1;

        FILETIME i0, k0, u0, i1, k1, u1;
        GetSystemTimes(&i0, &k0, &u0);
        c0 = calls;
        QueryPerformanceCounter(&q0);
        t0 = GetTickCount();
        it0 = interrupt_time();
        Sleep(500);
        c1 = calls;
        QueryPerformanceCounter(&q1);
        t1 = GetTickCount();
        it1 = interrupt_time();
        GetSystemTimes(&i1, &k1, &u1);
        {
            ULONGLONG idle = ft(i1) - ft(i0), busy = (ft(k1) - ft(k0)) + (ft(u1) - ft(u0));
            ULONGLONG qpc_us = (ULONGLONG)(q1.QuadPart - q0.QuadPart) * 1000000 / freq.QuadPart;
            ULONGLONG it_us = (it1 - it0) / 10;
            LONG n = c1 - c0;
            SAY("round %d%s: qpc %lu us, ticks %lu ms, interrupt %lu us, callbacks %ld, "
                "per-callback qpc %lu us, idle %lu%%\n",
                i, hog_on ? " (hog)" : "", (ULONG)qpc_us, t1 - t0, (ULONG)it_us, n,
                (ULONG)(n ? qpc_us / n : 0), (ULONG)(busy ? idle * 100 / busy : 0));
        }
    }
    hog_on = 0;
    SAY("done VERDICT=PASS\n");
    return 0;
}
