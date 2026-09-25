/*
 * kmtest-sweep — vibeos conformance harness (conformance-ratchet quest M2/M3).
 *
 * GUI-subsystem launcher (startup-walk-safe — see the doom/httpd link recipe
 * in ../httpd/build.sh) that runs kmtest_.exe ONE PROCESS PER TEST so a
 * crashing or hanging test cannot eat the rest of the suite. The test list is
 * %SystemRoot%\bin\kmtest-tests.txt (one name per line, '#' comments), staged
 * by the kmtest payload next to the rostests binaries.
 *
 * All sweep progress goes to the kernel debug serial via ntdll DbgPrint;
 * kmtest_.exe itself emits [KMTEST:BEGIN/END ...] markers + the full result
 * log (VIBEOS_KMTEST_SERIAL tee). A [KMTEST-SWEEP] launch line without a
 * matching [KMTEST:END] attributes a guest crash/hang to the exact test.
 */
#include <windows.h>
#include <tlhelp32.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define PER_TEST_TIMEOUT_MS (180u * 1000u)

static int setup_running(void)
{
    PROCESSENTRY32 pe = { sizeof pe };
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    int found = 0;
    if (snap == INVALID_HANDLE_VALUE)
        return 0;
    if (Process32First(snap, &pe))
        do {
            if (!_stricmp(pe.szExeFile, "setup.exe"))
                found = 1;
        } while (!found && Process32Next(snap, &pe));
    CloseHandle(snap);
    return found;
}

static DWORD plugplay_state(void)
{
    SERVICE_STATUS st = { 0 };
    SC_HANDLE scm = OpenSCManagerA(NULL, NULL, SC_MANAGER_CONNECT);
    SC_HANDLE svc = scm ? OpenServiceA(scm, "PlugPlay", SERVICE_QUERY_STATUS) : NULL;
    if (svc) { QueryServiceStatus(svc, &st); CloseServiceHandle(svc); }
    if (scm) CloseServiceHandle(scm);
    return st.dwCurrentState;
}

/* The livecd's `setup -mini` (syssetup InstallLiveCD) starts PlugPlay, and umpnpmgr then installs
 * every device: ~30s of heavy user-mode work and interrupt traffic (measured with
 * tools/payloads/cpuprof). A sweep that starts inside that burst fails the load-sensitive tests
 * (ExFastMutex, ExResource, KeEvent) on every attempt, where they are otherwise only rarely
 * flaky. So wait it out, the way a test run on Windows would wait for setup to finish. Checking
 * "no pending installs" is not enough on its own: before setup has started PlugPlay, that is
 * trivially true, and the burst then lands mid-sweep. */
static void wait_for_pnp_quiet(void)
{
    DWORD (WINAPI *wait_pnp)(DWORD) = (void *)GetProcAddress(LoadLibraryA("setupapi.dll"),
                                                              "CMP_WaitNoPendingInstallEvents");
    DWORD t0 = GetTickCount();
    while (plugplay_state() != SERVICE_RUNNING && setup_running() && GetTickCount() - t0 < 180000)
        Sleep(2000);
    if (plugplay_state() != SERVICE_RUNNING)
    {
        DbgPrint("[KMTEST-SWEEP] PlugPlay not running (setup %s) - no PnP gate\n",
                 setup_running() ? "still running" : "not running");
        return;
    }
    if (!wait_pnp)
    {
        DbgPrint("[KMTEST-SWEEP] WARN: CMP_WaitNoPendingInstallEvents unavailable - no PnP gate\n");
        return;
    }
    DWORD w = wait_pnp(180000);
    DbgPrint("[KMTEST-SWEEP] PnP installs %s (waited ~%lus)\n",
             w == WAIT_OBJECT_0 ? "quiet" : "STILL PENDING, proceeding",
             (GetTickCount() - t0) / 1000);
}

int main(void)
{
    char windir[MAX_PATH], listpath[MAX_PATH], exepath[MAX_PATH];
    char line[128];
    FILE *f;
    int total = 0, launched = 0, failstart = 0, timeouts = 0;

    if (!GetWindowsDirectoryA(windir, sizeof(windir)))
    {
        DbgPrint("[KMTEST-SWEEP] FATAL: GetWindowsDirectory err=%lu\n", GetLastError());
        return 1;
    }
    _snprintf(listpath, sizeof(listpath), "%s\\bin\\kmtest-tests.txt", windir);
    _snprintf(exepath, sizeof(exepath), "%s\\bin\\kmtest_.exe", windir);

    f = fopen(listpath, "r");
    if (!f)
    {
        DbgPrint("[KMTEST-SWEEP] FATAL: cannot open %s\n", listpath);
        return 1;
    }

    /* Gate on SCM boot auto-start completion: services.exe holds the service
     * database lock for the whole auto-start storm, so an early CreateService
     * blocks indefinitely (first light 2026-07-10: 3/3 tests timed out this
     * way). services.exe signals the global SC_AutoStartComplete event when
     * the storm is over (base/system/services/services.c). */
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
            /* arm64 (2026-07-12): services.exe never publishes SC_AutoStartComplete
             * on arm64, so the event never appears (x86 signals it within the first
             * few tries). The sweep runs AFTER the desktop is up (explorer startup
             * walk), by which point boot auto-start is functionally complete — so
             * PROCEED rather than FATAL-return. If a test's CreateService then blocks,
             * the per-test timeout below attributes it and that is itself the arm64
             * SCM finding (services-rust: publish SC_AutoStartComplete on arm64). */
            DbgPrint("[KMTEST-SWEEP] WARN: SC_AutoStartComplete never appeared (arm64 services gap?) — proceeding without the gate\n");
        }
        else
        {
            DbgPrint("[KMTEST-SWEEP] waiting for SCM auto-start completion\n");
            while ((w = WaitForSingleObject(ev, 30000)) == WAIT_TIMEOUT)
            {
                waited += 30;
                DbgPrint("[KMTEST-SWEEP] still waiting for SC_AutoStartComplete (%ds)\n", waited);
                if (waited >= 900)
                    break;
            }
            CloseHandle(ev);
            if (w != WAIT_OBJECT_0)
                DbgPrint("[KMTEST-SWEEP] WARN: SCM auto-start did not complete in %ds — proceeding anyway to gather evidence\n", waited);
            else
                DbgPrint("[KMTEST-SWEEP] SCM auto-start complete (waited ~%ds)\n", waited);
        }
    }

    wait_for_pnp_quiet();

    DbgPrint("[KMTEST-SWEEP] start list=%s\n", listpath);

    while (fgets(line, sizeof(line), f))
    {
        char cmdline[MAX_PATH + 140];
        STARTUPINFOA si;
        PROCESS_INFORMATION pi;
        DWORD wait;
        char *nl = strpbrk(line, "\r\n");

        if (!nl)
        {
            /* Line longer than the buffer (in practice an over-long '#' comment
             * in the test list): fgets stopped WITHOUT a newline. Drain the rest
             * of the physical line so its tail is not read as a bogus test name
             * on the next iteration — a real test name is always short — then
             * skip it. (Bug 2026-07-12: a 155-char i386 quarantine comment split
             * here and the tail "...symbol-poor)." got launched as a fake test,
             * polluting the baseline and poisoning the result buffer.) */
            int ch;
            while ((ch = fgetc(f)) != EOF && ch != '\n') { }
            continue;
        }
        *nl = 0;
        if (!line[0] || line[0] == '#')
            continue;
        ++total;

        _snprintf(cmdline, sizeof(cmdline), "\"%s\" %s", exepath, line);
        ZeroMemory(&si, sizeof(si));
        si.cb = sizeof(si);
        ZeroMemory(&pi, sizeof(pi));

        DbgPrint("[KMTEST-SWEEP] launch %s\n", line);
        /* DETACHED_PROCESS: kmtest_.exe is a console app, and CSRSS console
         * attach from the startup-walk context hangs the child before main()
         * (second first-light finding, 2026-07-10 — same family as the doom/
         * httpd GUI-subsystem lesson). No console: stdout WriteFile fails
         * harmlessly; the VIBEOS_KMTEST_SERIAL tee carries the results. */
        if (!CreateProcessA(exepath, cmdline, NULL, NULL, FALSE, DETACHED_PROCESS, NULL, NULL, &si, &pi))
        {
            DbgPrint("[KMTEST-SWEEP] launch FAILED %s err=%lu\n", line, GetLastError());
            ++failstart;
            continue;
        }
        ++launched;

        wait = WaitForSingleObject(pi.hProcess, PER_TEST_TIMEOUT_MS);
        if (wait == WAIT_TIMEOUT)
        {
            ++timeouts;
            DbgPrint("[KMTEST-SWEEP] TIMEOUT %s (%us) - terminating\n",
                     line, PER_TEST_TIMEOUT_MS / 1000u);
            TerminateProcess(pi.hProcess, 0xDEAD);
            WaitForSingleObject(pi.hProcess, 10000);
        }
        else
        {
            DWORD code = 0;
            GetExitCodeProcess(pi.hProcess, &code);
            DbgPrint("[KMTEST-SWEEP] exit %s code=%lu\n", line, code);
        }
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
    }
    fclose(f);

    DbgPrint("[KMTEST-SWEEP] done total=%d launched=%d failstart=%d timeouts=%d\n",
             total, launched, failstart, timeouts);
    return 0;
}
