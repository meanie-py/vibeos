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
 * ('#' comments). DERIVED at build time from the staged binaries, never
 * hand-maintained -- ExPushLock sat unrun for the length of a whole merge
 * because the kmtest list is committed and nothing reconciles it against the
 * tests that exist.
 */
#include <windows.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define PER_SUBTEST_TIMEOUT_MS (120u * 1000u)
#define ENUMERATE_TIMEOUT_MS   (60u * 1000u)
#define PIPE_BUF_BYTES       4096

#define MAX_SUBTESTS      256
#define MAX_SUBTEST_NAME  64

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
            DbgPrint("[APITEST:%s %s] %s\n", ctx->Suite, ctx->Label, line);
        else
            DbgPrint("[APITEST:%s] %s\n", ctx->Suite, line);
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
 * as failures. The regular apitest sweep stages no such file -- its suites may
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
        lstrcpynA(g_child_cwd, bindir, sizeof(g_child_cwd));
    DbgPrint("[APITEST-SWEEP] child cwd=%s\n", g_child_cwd);
}

static int run_child(const char *bindir, const char *suite, const char *arg,
                     struct pump_ctx *ctx, DWORD timeout_ms)
{
    char exepath[MAX_PATH], cmdline[MAX_PATH + MAX_SUBTEST_NAME + 8];
    SECURITY_ATTRIBUTES sa;
    STARTUPINFOA si;
    PROCESS_INFORMATION pi;
    HANDLE rd = NULL, wr = NULL, thread = NULL;
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
        DbgPrint("[APITEST-SWEEP] launch FAILED %s (CreatePipe err=%lu)\n", suite, GetLastError());
        return -1;
    }
    /* The read end must NOT be inheritable, or the child holds it open and the
     * pump never sees EOF -- the classic redirect hang. */
    SetHandleInformation(rd, HANDLE_FLAG_INHERIT, 0);

    ZeroMemory(&si, sizeof(si));
    si.cb = sizeof(si);
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = wr;
    si.hStdError = wr;
    si.hStdInput = INVALID_HANDLE_VALUE;
    ZeroMemory(&pi, sizeof(pi));

    /* DETACHED_PROCESS for the same reason as the kmtest sweep: these are
     * console apps and a CSRSS console attach from the explorer startup-walk
     * context hangs the child before main(). Our pipe carries the output, so
     * having no console costs nothing. */
    if (!CreateProcessA(exepath, cmdline, NULL, NULL, TRUE, DETACHED_PROCESS,
                        NULL, g_child_cwd, &si, &pi))
    {
        DbgPrint("[APITEST-SWEEP] launch FAILED %s err=%lu\n", suite, GetLastError());
        CloseHandle(rd);
        CloseHandle(wr);
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
    DbgPrint("[APITEST-SWEEP] spawned %s pid=%lu\n", suite, (unsigned long)pi.dwProcessId);

    /* Drop OUR copy of the write end or the pump blocks forever after the child
     * exits: a pipe stays open as long as any writer handle lives. */
    CloseHandle(wr);
    DbgPrint("[APITEST-SWEEP] closed  %s\n", suite);

    ctx->Read = rd;
    thread = CreateThread(NULL, 0, pump, ctx, 0, NULL);

    DbgPrint("[APITEST-SWEEP] waiting %s (%lums)\n", suite, (unsigned long)timeout_ms);
    wait = WaitForSingleObject(pi.hProcess, timeout_ms);
    if (wait == WAIT_TIMEOUT)
    {
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
    DbgPrint("[APITEST-SWEEP] skip list: %d entr%s\n", SkipCount, SkipCount == 1 ? "y" : "ies");
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
    DbgPrint("[APITEST-SWEEP] enumerate %s\n", suite);
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
        DbgPrint("[APITEST-SWEEP] ENUM-TIMEOUT %s (%us)\n",
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
        DbgPrint("[APITEST-SWEEP] NOSUBTESTS %s (exit %d) first-line=%s\n",
                 suite, rc, ctx.FirstLine[0] ? ctx.FirstLine : "(no output)");
        return;
    }

    DbgPrint("[APITEST-SWEEP] suite %s subtests=%d\n", suite, count);

    /* Phase 2: one process per subtest, so a crash or a wedge costs exactly one
     * subtest rather than the rest of the suite. Same bargain the kmtest sweep
     * makes, one level finer because a suite here bundles many tests. */
    for (i = 0; i < count; ++i)
    {
        if (is_skipped(suite, names[i]))
        {
            /* Announced, never silent: a skip nobody can see is how a dead test
             * stays dead. */
            DbgPrint("[APITEST-SWEEP] SKIP %s %s\n", suite, names[i]);
            continue;
        }

        ZeroMemory(&ctx, sizeof(ctx));
        ctx.Suite = suite;
        ctx.Label = names[i];

        DbgPrint("[APITEST-SWEEP] launch %s %s\n", suite, names[i]);
        rc = run_child(bindir, suite, names[i], &ctx, PER_SUBTEST_TIMEOUT_MS);
        ++*subtests;

        if (rc == -1)
            ++*failstart;
        else if (rc == -2)
        {
            ++*timeouts;
            DbgPrint("[APITEST-SWEEP] TIMEOUT %s %s (%us) - terminated\n",
                     suite, names[i], PER_SUBTEST_TIMEOUT_MS / 1000u);
        }
        else
            DbgPrint("[APITEST-SWEEP] exit %s %s code=%d\n", suite, names[i], rc);
    }
    ++*launched;
}

int main(void)
{
    char windir[MAX_PATH], bindir[MAX_PATH], listpath[MAX_PATH];
    char line[128];
    FILE *f;
    int total = 0, launched = 0, failstart = 0, timeouts = 0, subtests = 0;

    if (!GetWindowsDirectoryA(windir, sizeof(windir)))
    {
        DbgPrint("[APITEST-SWEEP] FATAL: GetWindowsDirectory err=%lu\n", GetLastError());
        return 1;
    }
    _snprintf(bindir, sizeof(bindir), "%s\\bin", windir);
    _snprintf(listpath, sizeof(listpath), "%s\\apitest-suites.txt", bindir);

    f = fopen(listpath, "r");
    if (!f)
    {
        DbgPrint("[APITEST-SWEEP] FATAL: cannot open %s\n", listpath);
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
            DbgPrint("[APITEST-SWEEP] WARN: SC_AutoStartComplete never appeared (arm64 services gap?) — proceeding without the gate\n");
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
                DbgPrint("[APITEST-SWEEP] WARN: SCM auto-start did not complete in %ds — proceeding anyway\n", waited);
            else
                DbgPrint("[APITEST-SWEEP] SCM auto-start complete (waited ~%ds)\n", waited);
        }
    }

    load_skips(bindir);
    load_child_cwd(bindir);
    DbgPrint("[APITEST-SWEEP] start list=%s\n", listpath);

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
            continue;
        }
        *nl = 0;
        if (!line[0] || line[0] == '#')
            continue;
        ++total;

        run_suite(bindir, line, &subtests, &launched, &failstart, &timeouts);
    }
    fclose(f);

    DbgPrint("[APITEST-SWEEP] done total=%d launched=%d subtests=%d failstart=%d timeouts=%d\n",
             total, launched, subtests, failstart, timeouts);
    return 0;
}
