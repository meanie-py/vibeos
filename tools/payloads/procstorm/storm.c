/*
 * procstorm — #39 amplifier (smp-bringup lane).
 *
 * The #39 wedge fires at most ONCE per boot, at the first user-mode service
 * start (`ScmStartUserModeService(DcomLaunch)`), at a rate near 2%. Hunting it
 * by soaking costs ~50 boots per sample, which is a tar pit. The evidence says
 * the rare event is PROCESS CREATION, not exception delivery: the offending
 * protection mask is 0 (MM_ZERO_ACCESS — no protection recorded at all, i.e. a
 * field nobody filled in), and the machinery that produces it is a freshly
 * built address space.
 *
 * So: build address spaces in bulk. This spawns itself as a do-nothing child N
 * times (default 1000) and reports progress to the kernel debug serial. If the
 * bug is generic to process creation, a healthy boot should now hit it many
 * times instead of once; if the storm runs clean, that is itself a real result
 * -- it says the bug needs something specific to the FIRST service start, and
 * the hunt moves there.
 *
 * A child is this same binary re-invoked with a mode argument:
 *
 *   "child"     — returns immediately. One address space, nothing else.
 *   "childexc"  — takes an access violation first (DEFAULT).
 *
 * The exception mode exists because plain creation was MEASURED not to be
 * enough: 1000/1000 spawns on the MP kernel produced zero guard reports, which
 * rules out address-space setup on its own. What the wedge capture actually
 * showed is a PC inside KiDispatchException writing an exception frame onto a
 * USER STACK -- so the interesting event is exception delivery on a freshly
 * built stack, which a child that just returns never performs. The dereference
 * is caught by an unhandled-exception filter that exits with a distinctive
 * code, so the parent can prove the exception really reached user-mode dispatch
 * rather than assuming it.
 *
 * GUI subsystem + classic msvcrt, per ../kmtest/build.sh: the explorer startup
 * walk cannot launch a script, and a console attach from that context hangs
 * the child before main().
 */
#include <windows.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define DEFAULT_ITERATIONS 1000u
#define CHILD_TIMEOUT_MS   (30u * 1000u)
#define PROGRESS_EVERY     50u

/* Distinctive so a child that died some OTHER way is not miscounted as a
 * successful exception dispatch. */
#define STORM_EXC_EXITCODE 0xE5E5E5E5u

static LONG CALLBACK storm_unhandled(EXCEPTION_POINTERS *ep)
{
    (void)ep;
    /* Reached only after the fault has gone kernel -> KiDispatchException ->
     * user-mode dispatch, which is the whole point of this mode. Leave via a
     * code the parent counts; returning EXECUTE_HANDLER here would let the CRT
     * pick the exit code and blur the signal. */
    ExitProcess(STORM_EXC_EXITCODE);
    return EXCEPTION_EXECUTE_HANDLER; /* not reached */
}

int main(int argc, char **argv)
{
    /* An unhandled exception now reaches UnhandledExceptionFilter, whose fault box
     * would leave a crashing child waiting on a dialog nobody answers. This mode is
     * inherited by every process spawned from here, which then exits with its
     * exception code instead -- as an unattended run on Windows does. */
    SetErrorMode(SetErrorMode(0) | SEM_NOGPFAULTERRORBOX);
    char self[MAX_PATH], cmdline[MAX_PATH + 16];
    unsigned iterations = DEFAULT_ITERATIONS, i;
    unsigned spawned = 0, failed = 0, timeouts = 0, excok = 0, excbad = 0;
    const char *childmode = "childexc";
    char *env;

    /* The child halves. */
    if (argc > 1 && strcmp(argv[1], "child") == 0)
        return 0;
    if (argc > 1 && strcmp(argv[1], "childexc") == 0)
    {
        SetUnhandledExceptionFilter(storm_unhandled);
        /* Fresh address space, fresh stack, first exception it ever takes --
         * the state the wedge was captured in. */
        *(volatile int *)0 = 1;
        return 0; /* not reached */
    }

    if (!GetModuleFileNameA(NULL, self, sizeof(self)))
    {
        DbgPrint("[PROCSTORM] FATAL: GetModuleFileName err=%lu\n", GetLastError());
        return 1;
    }

    /* "plain" reproduces the measured-negative baseline (creation only). */
    env = getenv("VIBEOS_PROCSTORM_MODE");
    if (env && strcmp(env, "plain") == 0)
        childmode = "child";

    env = getenv("VIBEOS_PROCSTORM_N");
    if (env && *env)
    {
        int n = atoi(env);
        if (n > 0)
            iterations = (unsigned)n;
    }

    /* Deliberately NOT gated on SC_AutoStartComplete, unlike the kmtest sweep.
     * That gate exists so CreateService cannot block on the service-database
     * lock; we never touch the SCM, and the autostart storm is the busiest the
     * memory manager ever gets -- exactly when we want to be creating address
     * spaces. */
    DbgPrint("[PROCSTORM] start n=%u mode=%s self=%s\n",
             iterations, childmode, self);

    for (i = 0; i < iterations; ++i)
    {
        STARTUPINFOA si;
        PROCESS_INFORMATION pi;
        DWORD w;

        ZeroMemory(&si, sizeof(si));
        si.cb = sizeof(si);
        ZeroMemory(&pi, sizeof(pi));

        _snprintf(cmdline, sizeof(cmdline), "\"%s\" %s", self, childmode);
        cmdline[sizeof(cmdline) - 1] = '\0';

        if (!CreateProcessA(self, cmdline, NULL, NULL, FALSE,
                            CREATE_NO_WINDOW, NULL, NULL, &si, &pi))
        {
            if (++failed <= 5)
                DbgPrint("[PROCSTORM] iter=%u CreateProcess err=%lu\n",
                         i, GetLastError());
            continue;
        }
        spawned++;

        w = WaitForSingleObject(pi.hProcess, CHILD_TIMEOUT_MS);
        if (w == WAIT_OBJECT_0 && strcmp(childmode, "childexc") == 0)
        {
            /* Verify the exception really reached user-mode dispatch instead of
             * assuming it: a child that died some other way must not be counted
             * as a successful dispatch. */
            DWORD code = 0;
            if (GetExitCodeProcess(pi.hProcess, &code) && code == STORM_EXC_EXITCODE)
                excok++;
            else if (++excbad <= 5)
                DbgPrint("[PROCSTORM] iter=%u child exit=0x%08lX (want 0x%08lX)\n",
                         i, code, (DWORD)STORM_EXC_EXITCODE);
        }
        if (w == WAIT_TIMEOUT)
        {
            /* A child that never exits is itself a finding -- say so and leave
             * it alone rather than papering over it with TerminateProcess. */
            timeouts++;
            DbgPrint("[PROCSTORM] iter=%u child HUNG (pid=%lu)\n",
                     i, pi.dwProcessId);
        }

        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);

        if (((i + 1) % PROGRESS_EVERY) == 0)
            DbgPrint("[PROCSTORM] progress %u/%u spawned=%u failed=%u hung=%u exc=%u\n",
                     i + 1, iterations, spawned, failed, timeouts, excok);
    }

    DbgPrint("[PROCSTORM] spawned=%u failed=%u hung=%u exc_dispatched=%u exc_wrong=%u of n=%u\n",
             spawned, failed, timeouts, excok, excbad, iterations);
    DbgPrint("[PROCSTORM] done\n");
    return 0;
}
