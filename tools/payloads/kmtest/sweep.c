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
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define PER_TEST_TIMEOUT_MS (180u * 1000u)

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
        DWORD w;

        for (i = 0; i < 60 && !ev; ++i)
        {
            ev = OpenEventA(SYNCHRONIZE, FALSE, "SC_AutoStartComplete");
            if (!ev)
                Sleep(5000);
        }
        if (!ev)
        {
            DbgPrint("[KMTEST-SWEEP] FATAL: SC_AutoStartComplete event never appeared (services.exe wedged pre-autostart?)\n");
            fclose(f);
            return 1;
        }
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
        {
            DbgPrint("[KMTEST-SWEEP] FATAL: SCM auto-start did not complete in %ds — proceeding anyway to gather evidence\n", waited);
            /* deliberate fall-through: the per-test timeouts below attribute
             * the damage; a wedged SCM is itself a conformance finding */
        }
        else
        {
            DbgPrint("[KMTEST-SWEEP] SCM auto-start complete (waited ~%ds)\n", waited);
        }
    }

    DbgPrint("[KMTEST-SWEEP] start list=%s\n", listpath);

    while (fgets(line, sizeof(line), f))
    {
        char cmdline[MAX_PATH + 140];
        STARTUPINFOA si;
        PROCESS_INFORMATION pi;
        DWORD wait;
        char *nl = strpbrk(line, "\r\n");

        if (nl)
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
