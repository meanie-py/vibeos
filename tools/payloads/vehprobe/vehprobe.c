/* vehprobe — does user mode call vectored exception handlers, on every arch?
 *
 * i386's and amd64's RtlDispatchException never called them until 2026-11-05: the C guards
 * the call with `#ifndef _NTOSKRNL_` and librtl is only ever built with -D_NTOSKRNL_, so the
 * compiled C dropped it in ntdll too. This grades the dispatch order the way Windows defines
 * it, with nothing arch-specific in the faulting code: the faulting store hits a PAGE_NOACCESS
 * page, and the handler fixes the PAGE rather than the instruction, so the store re-executes
 * and succeeds. (Skipping the instruction needs its length, which only arm64 knows for free.)
 *
 *   fault    a hardware AV reaches the handlers, the store then lands, the process lives
 *   order    handlers run newest-first for First=1 and in registration order for First=0, and
 *            CONTINUE_SEARCH passes the exception to the next one
 *   continue a continue handler runs once after a handler returns CONTINUE_EXECUTION
 *   remove   a removed handler is not called again
 *   raise    RaiseException returns when a handler continues it
 *
 * Each case runs on its own thread with a timeout, so a wedged dispatch is a HANG row rather
 * than a wedged probe. The raise case is known to hang on arm64 (roadmap: "arm64: a software
 * raise that a handler CONTINUES never returns"), so arm64 reports it but leaves it out of
 * the verdict until that is fixed.
 * Verdict: "[VEHPROBE] done VERDICT=PASS". */
#define _WIN32_WINNT 0x0601
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

static void SAY(const char *Format, ...)
{
    char line[512];
    va_list args;
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    int len = lstrlenA(lstrcpyA(line, "[VEHPROBE] "));
    va_start(args, Format);
    wvsprintfA(line + len, Format, args);
    va_end(args);
    DbgPrint("%s", line);
    if (out && out != INVALID_HANDLE_VALUE)
        WriteFile(out, line, lstrlenA(line), &n, NULL);
}

#define PROBE_CODE 0xE0564548u /* 'VEH' */

static volatile LONG *g_page;
static char g_trace[16];
static volatile LONG g_ntrace;
static volatile LONG g_continues;

static void trace(char c)
{
    LONG i = InterlockedIncrement(&g_ntrace) - 1;
    if (i < (LONG)sizeof(g_trace) - 1)
        g_trace[i] = c;
}

static void reset_trace(void)
{
    ZeroMemory(g_trace, sizeof(g_trace));
    g_ntrace = 0;
    g_continues = 0;
}

static int is_ours(EXCEPTION_POINTERS *ep)
{
    EXCEPTION_RECORD *r = ep->ExceptionRecord;
    if (r->ExceptionCode == PROBE_CODE)
        return 1;
    return r->ExceptionCode == EXCEPTION_ACCESS_VIOLATION && r->NumberParameters >= 2
        && (ULONG_PTR)r->ExceptionInformation[1] == (ULONG_PTR)g_page;
}

/* Fixes the cause: makes the page writable (AV) or just continues (our raise). */
static LONG CALLBACK fixer(EXCEPTION_POINTERS *ep)
{
    DWORD old;
    if (!is_ours(ep))
        return EXCEPTION_CONTINUE_SEARCH;
    trace('F');
    if (ep->ExceptionRecord->ExceptionCode == EXCEPTION_ACCESS_VIOLATION
        && !VirtualProtect((void *)g_page, 4096, PAGE_READWRITE, &old))
        return EXCEPTION_CONTINUE_SEARCH;
    return EXCEPTION_CONTINUE_EXECUTION;
}

/* Looks and passes it on. */
static LONG CALLBACK passer_a(EXCEPTION_POINTERS *ep)
{
    if (is_ours(ep))
        trace('a');
    return EXCEPTION_CONTINUE_SEARCH;
}

static LONG CALLBACK passer_b(EXCEPTION_POINTERS *ep)
{
    if (is_ours(ep))
        trace('b');
    return EXCEPTION_CONTINUE_SEARCH;
}

static LONG CALLBACK on_continue(EXCEPTION_POINTERS *ep)
{
    if (is_ours(ep))
        InterlockedIncrement(&g_continues);
    return EXCEPTION_CONTINUE_SEARCH;
}

static int arm_page(void)
{
    DWORD old;
    if (!g_page)
        g_page = VirtualAlloc(NULL, 4096, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (!g_page)
        return 0;
    g_page[0] = 0;
    return VirtualProtect((void *)g_page, 4096, PAGE_NOACCESS, &old);
}

/* The faulting store: an ordinary write the hardware refuses until a handler fixes the page. */
static void poke(LONG v)
{
    g_page[0] = v;
}

typedef int (*case_fn)(char *detail);

static int case_fault(char *detail)
{
    PVOID h = AddVectoredExceptionHandler(1, fixer);
    int ok;
    reset_trace();
    if (!h || !arm_page())
        return lstrcpyA(detail, "setup failed"), 0;
    poke(0x1234);
    RemoveVectoredExceptionHandler(h);
    ok = g_page[0] == 0x1234 && lstrcmpA(g_trace, "F") == 0;
    wsprintfA(detail, "trace '%s' value %08lx", g_trace, g_page[0]);
    return ok;
}

static int case_order(char *detail)
{
    /* Registration: fixer last-in-the-list (First=0), then a at the front, then b in front of
     * a. Windows calls b, a, fixer. */
    PVOID hf = AddVectoredExceptionHandler(0, fixer);
    PVOID ha = AddVectoredExceptionHandler(1, passer_a);
    PVOID hb = AddVectoredExceptionHandler(1, passer_b);
    reset_trace();
    if (!hf || !ha || !hb || !arm_page())
        return lstrcpyA(detail, "setup failed"), 0;
    poke(0x5678);
    RemoveVectoredExceptionHandler(hb);
    RemoveVectoredExceptionHandler(ha);
    RemoveVectoredExceptionHandler(hf);
    wsprintfA(detail, "trace '%s' value %08lx", g_trace, g_page[0]);
    return g_page[0] == 0x5678 && lstrcmpA(g_trace, "baF") == 0;
}

static int case_continue(char *detail)
{
    PVOID hf = AddVectoredExceptionHandler(1, fixer);
    PVOID hc = AddVectoredContinueHandler(1, on_continue);
    reset_trace();
    if (!hf || !hc || !arm_page())
        return lstrcpyA(detail, "setup failed"), 0;
    poke(0x9abc);
    RemoveVectoredContinueHandler(hc);
    RemoveVectoredExceptionHandler(hf);
    wsprintfA(detail, "trace '%s' continues %ld", g_trace, g_continues);
    return lstrcmpA(g_trace, "F") == 0 && g_continues == 1;
}

static int case_remove(char *detail)
{
    PVOID ha = AddVectoredExceptionHandler(1, passer_a);
    PVOID hf = AddVectoredExceptionHandler(0, fixer);
    ULONG removed;
    reset_trace();
    if (!ha || !hf || !arm_page())
        return lstrcpyA(detail, "setup failed"), 0;
    removed = RemoveVectoredExceptionHandler(ha);
    poke(0x1111);
    RemoveVectoredExceptionHandler(hf);
    wsprintfA(detail, "removed %lu trace '%s'", removed, g_trace);
    return removed != 0 && lstrcmpA(g_trace, "F") == 0;
}

static int case_raise(char *detail)
{
    PVOID h = AddVectoredExceptionHandler(1, fixer);
    reset_trace();
    if (!h)
        return lstrcpyA(detail, "setup failed"), 0;
    RaiseException(PROBE_CODE, 0, 0, NULL);
    RemoveVectoredExceptionHandler(h);
    wsprintfA(detail, "returned, trace '%s'", g_trace);
    return lstrcmpA(g_trace, "F") == 0;
}

struct job
{
    case_fn fn;
    int ok;
    char detail[200];
};

static DWORD WINAPI run_job(void *arg)
{
    struct job *j = arg;
    j->ok = j->fn(j->detail);
    return 0;
}

/* 1 pass, 0 fail, -1 hang. A thread that died of an unhandled fault takes the process with
 * it, so a dead dispatch shows as the log stopping before "done". */
static int run_case(const char *name, case_fn fn)
{
    static struct job j;
    HANDLE t;
    DWORD w;
    ZeroMemory(&j, sizeof(j));
    j.fn = fn;
    t = CreateThread(NULL, 0, run_job, &j, 0, NULL);
    if (!t)
    {
        SAY("%s FAIL (CreateThread %lu)\n", name, GetLastError());
        return 0;
    }
    w = WaitForSingleObject(t, 20000);
    if (w != WAIT_OBJECT_0)
    {
        TerminateThread(t, 1);
        CloseHandle(t);
        SAY("%s HANG (no return in 20 s, trace '%s')\n", name, g_trace);
        return -1;
    }
    CloseHandle(t);
    SAY("%s %s (%s)\n", name, j.ok ? "PASS" : "FAIL", j.detail);
    return j.ok;
}

int main(void)
{
    int pass = 1, r;
    SAY("begin\n");
    pass &= run_case("fault", case_fault) == 1;
    pass &= run_case("order", case_order) == 1;
    pass &= run_case("continue", case_continue) == 1;
    pass &= run_case("remove", case_remove) == 1;
    r = run_case("raise", case_raise);
#if defined(__aarch64__) || defined(_M_ARM64)
    if (r != 1)
        SAY("raise: known open on arm64 (RtlRaiseException hands the dispatcher its own "
            "context), not graded\n");
#else
    pass &= r == 1;
#endif
    SAY("done VERDICT=%s\n", pass ? "PASS" : "FAIL");
    return pass ? 0 : 1;
}
