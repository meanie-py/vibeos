/*
 * apitest-sweep — vibeos user-mode conformance harness.
 *
 * Sibling of ../kmtest/sweep.c, and the same shape: a GUI-subsystem launcher
 * dropped in the All Users Startup folder that runs ONE PROCESS PER SUITE so a
 * crashing or hanging suite cannot eat the rest of the run.
 *
 * ONE REAL DIFFERENCE, and it is the whole reason this file exists rather than
 * a flag on the kmtest sweep: kmtest_.exe reports through the kernel debug
 * serial itself (DbgPrint, VIBEOS_KMTEST_SERIAL tee), so that sweep can use
 * DETACHED_PROCESS and never look at the child's output. An apitest reports
 * through wine/test.h's winetest_vprintf, which is `fprintf(stdout, ...)` and
 * nothing else. So here we give each child a PIPE for stdout+stderr, pump it
 * on a reader thread, and re-emit every line to the serial with a prefix. A
 * detached apitest writes its results into a closed handle and the run looks
 * silently clean, which is exactly the failure this harness exists to stop.
 *
 * Suite list: %SystemRoot%\bin\apitest-suites.txt, one exe name per line
 * ('#' comments; `set NAME=VALUE` sets the environment of the children after it;
 * `timeout SUITE.exe SECONDS` replaces the per-subtest timeout for that suite).
 * DERIVED at build time from the staged binaries, never
 * hand-maintained -- ExPushLock sat unrun for the length of a whole merge
 * because the kmtest list is committed and nothing reconciles it against the
 * tests that exist.
 */
#include <windows.h>
#include <tlhelp32.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

/* Every sweep line goes to the kernel debug serial and, when this process HAS a stdout,
 * there too. In the guest (explorer's startup walk) it has none and this is DbgPrint
 * alone. On real Windows (`vibeos-cli winref apitest`, the reference column) DbgPrint
 * reaches nothing without a kernel debugger, so the same lines travel on stdout. */
static void emit(const char *Format, ...)
{
    static HANDLE out = INVALID_HANDLE_VALUE;
    char line[1024];
    va_list args;
    int n;
    DWORD written;

    va_start(args, Format);
    n = _vsnprintf(line, sizeof(line) - 1, Format, args);
    va_end(args);
    if (n < 0)
        n = sizeof(line) - 1;
    line[n] = 0;
    DbgPrint("%s", line);
    if (out == INVALID_HANDLE_VALUE)
        out = GetStdHandle(STD_OUTPUT_HANDLE);
    if (out && out != INVALID_HANDLE_VALUE)
        WriteFile(out, line, (DWORD)n, &written, NULL);
}

#define SAY(...) emit("[APITEST-SWEEP] " __VA_ARGS__)
#include "../stackdump.h"

/* A TIMEOUT used to arrive with nothing but its name: errorinfo and tmarshal hung for
 * weeks with no clue where. Dump every thread of the child before killing it; feed the
 * addresses to `vibeos-cli sym`. */
static void dump_child_threads(DWORD pid)
{
    THREADENTRY32 te = { sizeof te };
    HANDLE s = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if (s == INVALID_HANDLE_VALUE)
        return;
    for (BOOL k = Thread32First(s, &te); k; k = Thread32Next(s, &te))
        if (te.th32OwnerProcessID == pid)
            dump_thread(pid, te.th32ThreadID);
    CloseHandle(s);
}

#define PER_SUBTEST_TIMEOUT_MS (120u * 1000u)
#define MAX_TIMEOUTS           32
#define ENUMERATE_TIMEOUT_MS   (60u * 1000u)
#define PIPE_BUF_BYTES       4096

#define MAX_SUBTESTS      256
#define MAX_SUBTEST_NAME  64
#define SWEEP_LINE_MAX    512

struct pump_ctx
{
    HANDLE Read;
    const char *Suite;
    const char *Label;     /* [APITEST:<Suite> <Label>] prefix, "" for none */
    /* Capture mode: collect subtest names instead of echoing to the serial. A
     * wine/test.h binary run with no arguments prints
     *     Usage: <path> test_name
     *     Valid test names:
     *         <name>
     * and exits 1 -- it does NOT run anything. Discovering the names from the
     * binary keeps this self-maintaining, the same call this harness makes for
     * the suite list itself. */
    int Capture;
    char (*Names)[MAX_SUBTEST_NAME];
    int NameCount;
    int InList;
    /* First line seen during capture. If enumeration yields no names, this is
     * the only evidence of WHY -- a loader error, a missing import, a usage
     * banner in an unexpected shape. Discarding it would make every such suite
     * look identically and uninformatively empty. */
    char FirstLine[256];
};

static void take_line(struct pump_ctx *ctx, const char *line)
{
    if (!ctx->Capture)
    {
        if (ctx->Label[0])
            emit("[APITEST:%s %s] %s\n", ctx->Suite, ctx->Label, line);
        else
            emit("[APITEST:%s] %s\n", ctx->Suite, line);
        return;
    }

    if (!ctx->FirstLine[0] && line[0])
    {
        size_t n = strlen(line);
        if (n >= sizeof(ctx->FirstLine))
            n = sizeof(ctx->FirstLine) - 1;
        memcpy(ctx->FirstLine, line, n);
        ctx->FirstLine[n] = 0;
    }

    if (!ctx->InList)
    {
        if (strstr(line, "Valid test names:"))
            ctx->InList = 1;
        return;
    }
    /* Names are indented; anything flush-left ends the list. */
    if (line[0] != ' ' && line[0] != '\t')
    {
        ctx->InList = 0;
        return;
    }
    while (*line == ' ' || *line == '\t')
        ++line;
    if (*line && ctx->NameCount < MAX_SUBTESTS)
    {
        size_t n = strlen(line);
        if (n >= MAX_SUBTEST_NAME)
            n = MAX_SUBTEST_NAME - 1;
        memcpy(ctx->Names[ctx->NameCount], line, n);
        ctx->Names[ctx->NameCount][n] = 0;
        ++ctx->NameCount;
    }
}

/*
 * Re-emit the child's stdout to the serial, one line per DbgPrint so a crash
 * mid-suite still leaves every completed line on the wire. Partial reads are
 * normal on a pipe, so hold a line buffer across reads.
 */
static DWORD WINAPI pump(LPVOID param)
{
    struct pump_ctx *ctx = (struct pump_ctx *)param;
    char buf[PIPE_BUF_BYTES];
    char line[1024];
    size_t used = 0;
    DWORD got;

    while (ReadFile(ctx->Read, buf, sizeof(buf), &got, NULL) && got > 0)
    {
        DWORD i;
        for (i = 0; i < got; ++i)
        {
            char c = buf[i];
            if (c == '\r')
                continue;
            if (c == '\n' || used == sizeof(line) - 1)
            {
                line[used] = 0;
                if (used)
                    take_line(ctx, line);
                used = 0;
                if (c != '\n')
                    line[used++] = c;
                continue;
            }
            line[used++] = c;
        }
    }
    if (used)
    {
        line[used] = 0;
        take_line(ctx, line);
    }
    return 0;
}

/*
 * Run `<suite> [arg]` with stdout+stderr piped, bounded by a timeout.
 * Returns the exit code, or -1 if it could not start, or -2 on timeout.
 * With ctx->Capture set, output is parsed for subtest names instead of echoed.
 */
/* The children's working directory: bindir, unless bin\apitest-cwd.txt names
 * another. Opt-in, for suites that create files relative to it: the i386 media
 * boots from a CDFS image, so X:\ (bindir and %TEMP% included) refuses every
 * write with ERROR_ACCESS_DENIED, and such a suite reports its own environment
 * as failures. TEMP/TMP follow it (below). The regular apitest sweep stages no such file -- its suites may
 * read testdata relative to bindir. */
static char g_child_cwd[MAX_PATH];

static void load_child_cwd(const char *bindir)
{
    char path[MAX_PATH];
    FILE *f;
    size_t n;

    lstrcpynA(g_child_cwd, bindir, sizeof(g_child_cwd));
    _snprintf(path, sizeof(path), "%s\\apitest-cwd.txt", bindir);
    f = fopen(path, "r");
    if (!f)
        return;
    if (fgets(g_child_cwd, sizeof(g_child_cwd), f))
    {
        n = strlen(g_child_cwd);
        while (n && (g_child_cwd[n - 1] == '\n' || g_child_cwd[n - 1] == '\r' || g_child_cwd[n - 1] == ' '))
            g_child_cwd[--n] = 0;
    }
    fclose(f);
    if (!g_child_cwd[0])
    {
        lstrcpynA(g_child_cwd, bindir, sizeof(g_child_cwd));
        return;
    }
    /* %TEMP% is on the same read-only image (X:\reactos\TEMP), and suites put
     * their scratch files there (ucrtbase's thread test writes its helper DLL
     * to GetTempPath()), so the children inherit the writable directory as
     * TEMP/TMP too. */
    SetEnvironmentVariableA("TEMP", g_child_cwd);
    SetEnvironmentVariableA("TMP", g_child_cwd);
    emit("[APITEST-SWEEP] child cwd=TEMP=%s\n", g_child_cwd);
}

static int run_child(const char *bindir, const char *suite, const char *arg,
                     struct pump_ctx *ctx, DWORD timeout_ms)
{
    /* A list line is "<exe> <rest>", and <rest> may be a whole command line
     * (olewinetest's `reg.exe add "HKLM\...\Image File Execution Options\..."`),
     * not just a subtest name: size it for SWEEP_LINE_MAX. */
    char exepath[MAX_PATH], cmdline[MAX_PATH + SWEEP_LINE_MAX + 8];
    SECURITY_ATTRIBUTES sa;
    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    HANDLE rd = NULL, wr = NULL, nul = INVALID_HANDLE_VALUE, thread = NULL;
    DWORD wait, code = 0;

    _snprintf(exepath, sizeof(exepath), "%s\\%s", bindir, suite);
    if (arg && arg[0])
        _snprintf(cmdline, sizeof(cmdline), "\"%s\" %s", exepath, arg);
    else
        _snprintf(cmdline, sizeof(cmdline), "\"%s\"", exepath);

    ZeroMemory(&sa, sizeof(sa));
    sa.nLength = sizeof(sa);
    sa.bInheritHandle = TRUE;
    if (!CreatePipe(&rd, &wr, &sa, 0))
    {
        emit("[APITEST-SWEEP] launch FAILED %s (CreatePipe err=%lu)\n", suite, GetLastError());
        return -1;
    }
    /* The read end must NOT be inheritable, or the child holds it open and the
     * pump never sees EOF -- the classic redirect hang. */
    SetHandleInformation(rd, HANDLE_FLAG_INHERIT, 0);

    /* A real (empty) stdin, as every Windows test runner provides. Handed
     * INVALID_HANDLE_VALUE, the CRT parks fd 0 as a no-console placeholder that
     * fclose(stdin) cannot release, so fd 0 never comes free again: ucrtbase's
     * file.c then saw fopen/freopen land on fd 4 instead of 0 (file.c:91/97/170/175,
     * failed by Microsoft's own ucrtbase the same way). */
    nul = CreateFileA("NUL", GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, &sa,
                      OPEN_EXISTING, 0, NULL);

    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = wr;
    si.hStdError = wr;
    si.hStdInput = nul;
    ZeroMemory(&pi, sizeof(pi));

    /* DETACHED_PROCESS for the same reason as the kmtest sweep: these are
     * console apps and a CSRSS console attach from the explorer startup-walk
     * context hangs the child before main(). Our pipe carries the output, so
     * having no console costs nothing. */
    if (!CreateProcessA(exepath, cmdline, NULL, NULL, TRUE, DETACHED_PROCESS,
                        NULL, g_child_cwd, &si, &pi))
    {
        emit("[APITEST-SWEEP] launch FAILED %s err=%lu\n", suite, GetLastError());
        CloseHandle(rd);
        CloseHandle(wr);
        if (nul != INVALID_HANDLE_VALUE)
            CloseHandle(nul);
        return -1;
    }

    /*
     * Which side of CreateProcess did a wedge happen on?
     *
     * The wedge cluster (msvcrt system/_wsystem/atexit/static_construct, cmd
     * attrib/exit/pushd, kernel32, ntdll, user32 VirtualKey/WndProc) shows the
     * `launch` line and then nothing -- no `exit`, and critically no
     * `TIMEOUT ... (120s) - terminated` even though EVERY wait below is bounded
     * (120s child, +10s after the kill, 15s pump join). The sweep should advance
     * within ~145s whatever the child does, so it is not executing.
     *
     * FOUR candidates live in this span, and only the last is covered by a timer.
     * An earlier pass called the bounded wait the answer on the grounds that
     * "every wait below is bounded" -- true, but the span also holds two calls
     * that are not waits and are not bounded either, and that argument silently
     * skipped them:
     *   - CreateProcessA never returns.      Nothing bounds it.
     *   - CloseHandle(wr) never returns.     Nothing bounds it.
     *   - CreateThread never returns.        Nothing bounds it.
     *   - WaitForSingleObject's timeout never fires. This one was ACCUSED and
     *     ACQUITTED: an isolated boot of this suite with the quarantine disarmed
     *     printed `TIMEOUT ... log10 (120s) - terminated`, so the bounded wait
     *     does expire -- same call, same object type, same boot. It is listed
     *     because it can still fail INTERMITTENTLY, not because it is the answer.
     *
     * Three prints split all four for the cost of three DbgPrints. Read the last
     * line a wedged suite reached:
     *   (no spawned) -> CreateProcessA        (spawned) -> CloseHandle
     *   (closed)     -> CreateThread          (waiting) -> the wait itself
     * Only `waiting` with no following `exit`/`TIMEOUT` indicts the timeout, and
     * that is the only one of the four that is a kernel timer bug rather than a
     * call that is simply allowed to block. The other three are calls that are
     * ALLOWED to block, so hitting one of those is a lead, not a verdict.
     */
    emit("[APITEST-SWEEP] spawned %s pid=%lu\n", suite, (unsigned long)pi.dwProcessId);

    /* Drop OUR copy of the write end or the pump blocks forever after the child
     * exits: a pipe stays open as long as any writer handle lives. */
    CloseHandle(wr);
    if (nul != INVALID_HANDLE_VALUE)
        CloseHandle(nul);
    emit("[APITEST-SWEEP] closed  %s\n", suite);

    ctx->Read = rd;
    thread = CreateThread(NULL, 0, pump, ctx, 0, NULL);

    emit("[APITEST-SWEEP] waiting %s (%lums)\n", suite, (unsigned long)timeout_ms);
    wait = WaitForSingleObject(pi.hProcess, timeout_ms);
    if (wait == WAIT_TIMEOUT)
    {
        dump_child_threads(pi.dwProcessId);
        TerminateProcess(pi.hProcess, 0xDEAD);
        WaitForSingleObject(pi.hProcess, 10000);
    }
    else
    {
        GetExitCodeProcess(pi.hProcess, &code);
    }

    /* Killing the child closes its write end, so the pump now hits EOF. Bound
     * the join anyway: a grandchild that inherited the handle keeps it open. */
    if (thread)
    {
        WaitForSingleObject(thread, 15000);
        CloseHandle(thread);
    }
    CloseHandle(rd);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);

    return (wait == WAIT_TIMEOUT) ? -2 : (int)code;
}

/* `timeout <suite.exe> <seconds>` in the list: a suite that is slow, not wedged. dwrite's
 * font test runs 324K checks (on the Windows host) and was killed at 120s every time. */
static struct
{
    char Suite[MAX_SUBTEST_NAME];
    DWORD Ms;
} Timeouts[MAX_TIMEOUTS];
static int TimeoutCount;

static DWORD subtest_timeout_ms(const char *suite)
{
    int i;

    for (i = 0; i < TimeoutCount; ++i)
    {
        if (_stricmp(Timeouts[i].Suite, suite) == 0)
            return Timeouts[i].Ms;
    }
    return PER_SUBTEST_TIMEOUT_MS;
}

/* Returns 0 for a malformed directive; the caller says so out loud. */
static int set_timeout(const char *args)
{
    char suite[MAX_SUBTEST_NAME];
    unsigned long secs;
    int i;

    if (sscanf(args, "%63s %lu", suite, &secs) != 2 || secs == 0 || secs > 24ul * 3600ul)
        return 0;
    for (i = 0; i < TimeoutCount && _stricmp(Timeouts[i].Suite, suite); ++i) { }
    if (i == MAX_TIMEOUTS)
        return 0;
    if (i == TimeoutCount)
        ++TimeoutCount;
    _snprintf(Timeouts[i].Suite, sizeof(Timeouts[i].Suite), "%s", suite);
    Timeouts[i].Ms = (DWORD)(secs * 1000ul);
    return 1;
}

#define MAX_SKIPS 128

static char SkipList[MAX_SKIPS][MAX_SUBTEST_NAME * 2];
static int SkipCount;

/*
 * %SystemRoot%\\bin\\apitest-skip.txt: "<suite.exe> <subtest>" per line.
 * Subtest granularity matters because a single wedging subtest otherwise costs
 * its whole suite -- advapi32's RtlEncryptMemory crash-loops services.exe and
 * would have taken the other 18 advapi32 subtests with it.
 */
static void load_skips(const char *bindir)
{
    char path[MAX_PATH], line[MAX_SUBTEST_NAME * 2];
    FILE *f;

    _snprintf(path, sizeof(path), "%s\\apitest-skip.txt", bindir);
    f = fopen(path, "r");
    if (!f)
        return;   /* optional */

    while (fgets(line, sizeof(line), f) && SkipCount < MAX_SKIPS)
    {
        char *nl = strpbrk(line, "\r\n");
        if (nl)
            *nl = 0;
        if (!line[0] || line[0] == '#')
            continue;
        _snprintf(SkipList[SkipCount], sizeof(SkipList[0]), "%s", line);
        ++SkipCount;
    }
    fclose(f);
    emit("[APITEST-SWEEP] skip list: %d entr%s\n", SkipCount, SkipCount == 1 ? "y" : "ies");
}

static int is_skipped(const char *suite, const char *subtest)
{
    char key[MAX_SUBTEST_NAME * 2];
    int i;

    _snprintf(key, sizeof(key), "%s %s", suite, subtest);
    for (i = 0; i < SkipCount; ++i)
    {
        if (strcmp(SkipList[i], key) == 0)
            return 1;
    }
    return 0;
}

static void run_one_subtest(const char *bindir, const char *suite, const char *subtest,
                            int *subtests, int *failstart, int *timeouts)
{
    struct pump_ctx ctx;
    int rc;

    ZeroMemory(&ctx, sizeof(ctx));
    ctx.Suite = suite;
    ctx.Label = subtest;

    emit("[APITEST-SWEEP] launch %s %s\n", suite, subtest);
    rc = run_child(bindir, suite, subtest, &ctx, subtest_timeout_ms(suite));
    ++*subtests;

    if (rc == -1)
        ++*failstart;
    else if (rc == -2)
    {
        ++*timeouts;
        emit("[APITEST-SWEEP] TIMEOUT %s %s (%lus) - terminated\n",
                 suite, subtest, (unsigned long)(subtest_timeout_ms(suite) / 1000u));
    }
    else
        emit("[APITEST-SWEEP] exit %s %s code=%d\n", suite, subtest, rc);
}

static void run_suite(const char *bindir, const char *suite,
                      int *subtests, int *launched, int *failstart, int *timeouts)
{
    static char names[MAX_SUBTESTS][MAX_SUBTEST_NAME];
    struct pump_ctx ctx;
    int count, i, rc;

    /* Phase 1: ask the binary what it can run. */
    ZeroMemory(&ctx, sizeof(ctx));
    ctx.Suite = suite;
    ctx.Label = "";
    ctx.Capture = 1;
    ctx.Names = names;
    /* Announce BEFORE running the enumeration child: a crash in here used to
     * have nothing in flight to attribute it to, so the whole sweep was a blind
     * spot between suites. */
    emit("[APITEST-SWEEP] enumerate %s\n", suite);
    rc = run_child(bindir, suite, NULL, &ctx, ENUMERATE_TIMEOUT_MS);
    count = ctx.NameCount;

    if (rc == -1)
    {
        ++*failstart;
        return;
    }
    if (rc == -2)
    {
        /* Deliberately NOT "TIMEOUT <suite> during enumeration": that reads as
         * "<suite> <subtest>" to any reasonable parser, and ours duly invented a
         * subtest named "during" and banked it as a conformance identity. A
         * distinct verb costs nothing and cannot be misread. */
        emit("[APITEST-SWEEP] ENUM-TIMEOUT %s (%us)\n",
                 suite, ENUMERATE_TIMEOUT_MS / 1000u);
        ++*timeouts;
        return;
    }
    if (count == 0)
    {
        /* No "Valid test names:" block. Either not a wine/test.h binary, or it
         * failed to start at all -- say so, with whatever it did print, rather
         * than silently counting it clean. An exit 255 with no output at all is
         * almost always a loader failure (missing import). */
        emit("[APITEST-SWEEP] NOSUBTESTS %s (exit %d) first-line=%s\n",
                 suite, rc, ctx.FirstLine[0] ? ctx.FirstLine : "(no output)");
        /* A binary with ONE test (the sdk delayimp apitests: no testlist, so no
         * "Valid test names:" banner) does not list anything: it just RUNS, and
         * the enumeration pass above swallowed every line of its result but the
         * first. Run it once more with its output on the serial, so its
         * failures are readable rather than an exit code. A loader failure
         * printed nothing, and a usage banner is not a run. */
        if (ctx.FirstLine[0] && strncmp(ctx.FirstLine, "Usage:", 6) != 0)
        {
            ZeroMemory(&ctx, sizeof(ctx));
            ctx.Suite = suite;
            ctx.Label = "";
            ++*launched;
            rc = run_child(bindir, suite, NULL, &ctx, subtest_timeout_ms(suite));
            if (rc == -1)
                ++*failstart;
            else if (rc == -2)
            {
                ++*timeouts;
                emit("[APITEST-SWEEP] TIMEOUT %s (single test, %lus) - terminated\n",
                         suite, (unsigned long)(subtest_timeout_ms(suite) / 1000u));
            }
            else
                emit("[APITEST-SWEEP] exit %s (single test) code=%d\n", suite, rc);
        }
        return;
    }

    emit("[APITEST-SWEEP] suite %s subtests=%d\n", suite, count);

    /* Phase 2: one process per subtest, so a crash or a wedge costs exactly one
     * subtest rather than the rest of the suite. Same bargain the kmtest sweep
     * makes, one level finer because a suite here bundles many tests. */
    for (i = 0; i < count; ++i)
    {
        if (is_skipped(suite, names[i]))
        {
            /* Announced, never silent: a skip nobody can see is how a dead test
             * stays dead. */
            emit("[APITEST-SWEEP] SKIP %s %s\n", suite, names[i]);
            continue;
        }

        ZeroMemory(&ctx, sizeof(ctx));
        ctx.Suite = suite;
        ctx.Label = names[i];

        emit("[APITEST-SWEEP] launch %s %s\n", suite, names[i]);
        rc = run_child(bindir, suite, names[i], &ctx, subtest_timeout_ms(suite));
        ++*subtests;

        if (rc == -1)
            ++*failstart;
        else if (rc == -2)
        {
            ++*timeouts;
            emit("[APITEST-SWEEP] TIMEOUT %s %s (%lus) - terminated\n",
                     suite, names[i], (unsigned long)(subtest_timeout_ms(suite) / 1000u));
        }
        else
            emit("[APITEST-SWEEP] exit %s %s code=%d\n", suite, names[i], rc);
    }
    ++*launched;
}

/* The LiveCD's setup.exe (syssetup InstallLiveCD) registers COM components --
 * actxprxy's proxy/stubs among them -- in the background, AFTER auto-start, and on a
 * slow boot after the sweep has begun. A suite racing it sees REGDB_E_IIDNOTREG:
 * ole32's compobj test failed CoRegisterClassObject(LOCAL_SERVER) on exactly that
 * whenever it started before "InstallLiveCD: registering components" (2026-10-04).
 * So wait for setup.exe to exit. No setup.exe (an installed system) passes at once. */
static int setup_running(void)
{
    PROCESSENTRY32W pe;
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    int found = 0;

    if (snap == INVALID_HANDLE_VALUE)
        return 0;
    pe.dwSize = sizeof(pe);
    if (Process32FirstW(snap, &pe))
    {
        do
        {
            if (!lstrcmpiW(pe.szExeFile, L"setup.exe"))
            {
                found = 1;
                break;
            }
        } while (Process32NextW(snap, &pe));
    }
    CloseHandle(snap);
    return found;
}

static void wait_for_livecd_setup(void)
{
    int waited = 0;

    while (setup_running())
    {
        if (waited >= 900)
        {
            emit("[APITEST-SWEEP] WARN: setup.exe still running after %ds -- proceeding; "
                     "COM registration may be incomplete\n", waited);
            return;
        }
        Sleep(5000);
        waited += 5;
    }
    if (waited)
        emit("[APITEST-SWEEP] LiveCD setup finished (waited ~%ds)\n", waited);
}

int main(void)
{
    /* An unhandled exception now reaches UnhandledExceptionFilter, whose fault box
     * would leave a crashing child waiting on a dialog nobody answers. This mode is
     * inherited by every process spawned from here, which then exits with its
     * exception code instead -- as an unattended run on Windows does. */
    SetErrorMode(SetErrorMode(0) | SEM_NOGPFAULTERRORBOX);
    char windir[MAX_PATH], bindir[MAX_PATH], listpath[MAX_PATH];
    char line[SWEEP_LINE_MAX];
    FILE *f;
    int total = 0, launched = 0, failstart = 0, timeouts = 0, subtests = 0;

    if (!GetWindowsDirectoryA(windir, sizeof(windir)))
    {
        emit("[APITEST-SWEEP] FATAL: GetWindowsDirectory err=%lu\n", GetLastError());
        return 1;
    }
    _snprintf(bindir, sizeof(bindir), "%s\\bin", windir);
    _snprintf(listpath, sizeof(listpath), "%s\\apitest-suites.txt", bindir);

    f = fopen(listpath, "r");
    if (!f)
    {
        emit("[APITEST-SWEEP] FATAL: cannot open %s\n", listpath);
        return 1;
    }

    /* Same SCM gate as the kmtest sweep: several apitests drive services, and
     * an early CreateService blocks for the whole auto-start storm. arm64's
     * services.exe never publishes SC_AutoStartComplete, so a missing event is
     * a WARN and we proceed -- the startup walk already means the desktop is
     * up, and a per-suite timeout will attribute it if that is wrong. */
    {
        HANDLE ev = NULL;
        int i, waited = 0;
        DWORD w = WAIT_OBJECT_0;

        for (i = 0; i < 24 && !ev; ++i)
        {
            ev = OpenEventA(SYNCHRONIZE, FALSE, "SC_AutoStartComplete");
            if (!ev)
                Sleep(5000);
        }
        if (!ev)
        {
            emit("[APITEST-SWEEP] WARN: SC_AutoStartComplete never appeared (arm64 services gap?) — proceeding without the gate\n");
        }
        else
        {
            while ((w = WaitForSingleObject(ev, 30000)) == WAIT_TIMEOUT)
            {
                waited += 30;
                if (waited >= 900)
                    break;
            }
            CloseHandle(ev);
            if (w != WAIT_OBJECT_0)
                emit("[APITEST-SWEEP] WARN: SCM auto-start did not complete in %ds — proceeding anyway\n", waited);
            else
                emit("[APITEST-SWEEP] SCM auto-start complete (waited ~%ds)\n", waited);
        }
    }

    wait_for_livecd_setup();

    load_skips(bindir);
    load_child_cwd(bindir);
    emit("[APITEST-SWEEP] start list=%s\n", listpath);

    while (fgets(line, sizeof(line), f))
    {
        char *nl = strpbrk(line, "\r\n");

        if (!nl)
        {
            /* Over-long line: drain the rest of it rather than reading its tail
             * as a suite name on the next iteration. Cost the kmtest sweep a
             * fake test in the baseline once (2026-07-12). */
            int ch;
            while ((ch = fgetc(f)) != EOF && ch != '\n') { }
            /* Loud: a silently dropped line reads as a step nobody asked for
             * (olewinetest's 150-char reg.exe step vanished at the old 128). */
            emit("[APITEST-SWEEP] SKIP over-long list line (> %d chars): %.60s...\n",
                     SWEEP_LINE_MAX - 2, line);
            continue;
        }
        *nl = 0;
        if (!line[0] || line[0] == '#')
            continue;

        /* "set NAME=VALUE": an environment variable every LATER child inherits ("set
         * NAME=" clears it). WINETEST_REPORT_SUCCESS=1 makes a suite print each passing
         * ok() too, which is how a crash with no stack is placed: the last line before it. */
        if (!strncmp(line, "set ", 4))
        {
            char *eq = strchr(line + 4, '=');
            if (eq)
            {
                *eq = 0;
                SetEnvironmentVariableA(line + 4, eq[1] ? eq + 1 : NULL);
                emit("[APITEST-SWEEP] env %s=%s\n", line + 4, eq + 1);
                continue;
            }
        }
        if (!strncmp(line, "timeout ", 8))
        {
            if (set_timeout(line + 8))
                emit("[APITEST-SWEEP] timeout %s\n", line + 8);
            else
                emit("[APITEST-SWEEP] BAD timeout line (want: timeout SUITE.exe SECONDS): %s\n",
                         line);
            continue;
        }
        ++total;

        /* "<suite.exe> <subtest>": run just that subtest (gen_suites --only suite:subtest).
         * Reproducing one subtest otherwise meant running every subtest ahead of it, and a
         * suite that stalls early (user32's AttachThreadInput) hid the rest entirely. */
        {
            char *sp = strchr(line, ' ');
            if (sp)
            {
                *sp = 0;
                run_one_subtest(bindir, line, sp + 1, &subtests, &failstart, &timeouts);
                continue;
            }
        }

        run_suite(bindir, line, &subtests, &launched, &failstart, &timeouts);
    }
    fclose(f);

    emit("[APITEST-SWEEP] done total=%d launched=%d subtests=%d failstart=%d timeouts=%d\n",
             total, launched, subtests, failstart, timeouts);
    return 0;
}
