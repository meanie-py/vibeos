/*
 * spawn32.exe — the i386 process-creation guest. Built with the llvm-mingw
 * i686 target, no CRT (plain _start entry, same as Hello32.c/Reg32.c),
 * importing kernel32 only, loaded through the genuine 32-bit loader.
 *
 * What it proves: a 32-bit guest can START ANOTHER 32-BIT PROCESS. The lane
 * doc ranks this the #1 real gap ("every installer and every app that shells
 * out hits this") and says its status is genuinely UNKNOWN rather than
 * known-broken, so measuring it on today's tree is the first move. This guest
 * is that measurement, and it is expected to FAIL when first landed. That is
 * the point: a red rung with a GetLastError beside it is a diagnosis, and it
 * is the thing no host-side test can produce, because the whole question is
 * what the 32-bit syscall thunks do with a handle to a DIFFERENT process.
 *
 * Traced prediction, recorded here so the boot can refute it. ReactOS builds a
 * child in two steps -- NtCreateProcessEx, then NtCreateThread for the initial
 * thread, passing the CHILD's ProcessHandle (kernel32-rust proc_create.rs, at
 * the "Create the Kernel Thread Object" call). wow64_manual.c's NtCreateThread
 * thunk refuses any ProcessHandle that is not -1, deliberately and with a
 * reason -- a cross-process thread has no per-thread entry in the target and
 * nobody would initialize it. So rung 1 should fail with the child's main
 * thread never created, and CreateProcess should report the failure its own
 * source comments as "a process is not allowed to exist without a main
 * thread". If rung 1 PASSES, that prediction was wrong and the interesting
 * question moves to rungs 2-5.
 *
 * Self-spawning on purpose: staging is all-or-nothing (`_wow64_guest_ok`), so
 * a guest that needed a second binary on the media would be one more way for
 * the whole ladder to go unstaged. The child is this same .exe with one extra
 * argument, which also makes "the child is a 32-bit process" trivially true by
 * construction -- so rung 2 tests the SYSTEM's view of it, not the file's.
 *
 * The rungs, and what each one would catch:
 *   1  CreateProcessW(CREATE_SUSPENDED) succeeds     (the initial thread exists at all)
 *   2  IsWow64Process(child) is TRUE                 (the child came up as a guest, not native)
 *   3  ResumeThread releases it                      (cross-process suspend count is real)
 *   4  the child reaches its own exit                (it ran, rather than dying on entry)
 *   5  its exit code arrives intact                  (the parent can actually collect a result)
 *   6  CreateRemoteThread into the RUNNING child      (the other half of task #64)
 *   7  that injected thread runs to completion        (it was initialised, not just created)
 *
 * Rung 3 is worth its own code because it is a SECOND cross-process operation
 * on a different syscall (NtResumeThread, not NtCreateThread): if rung 1 is
 * ever fixed and rung 3 then fails, the two refusals were independent and the
 * fix was partial.
 *
 * Verdict contract (same as the other ladder guests): exit code 0x600D on PASS;
 * distinct 0xBAEx codes name the failing rung so serial alone triages. The
 * child half exits 0x0C41, which is neither PASS nor a 0xBAEx, so a child
 * verdict can never be mistaken for the parent's.
 */

#include <windows.h>

#define S32_FAIL_MODPATH   0xBAE0 /* could not learn our own path -- fixture fault, not a verdict */
#define S32_FAIL_CREATE    0xBAE1 /* CreateProcessW failed -- the predicted failure */
#define S32_FAIL_NOT_WOW64 0xBAE2 /* child is not a 32-bit process under WoW64 */
#define S32_FAIL_RESUME    0xBAE3 /* ResumeThread failed -- cross-process suspend broken */
#define S32_FAIL_WAIT      0xBAE4 /* child never terminated */
#define S32_FAIL_EXITCODE  0xBAE5 /* child ran but its exit code did not survive */
#define S32_FAIL_READY     0xBAE6 /* child never reached its own code */
#define S32_FAIL_REMOTE    0xBAE7 /* CreateRemoteThread failed */
#define S32_FAIL_REMOTE_RUN 0xBAE8 /* remote thread created but never ran its body */
#define S32_CHILD_MAGIC    0x0C41 /* child saw the remote thread's signal */
#define S32_CHILD_NOSIGNAL 0x0C42 /* child ran but nothing ever signalled it */
#define S32_REMOTE_MAGIC   0x0DEF /* what the injected thread returns */
#define S32_PASS           0x600D

/* Named so the two halves can find each other without inheriting handles --
 * the point is to test the SYSTEM's cross-process machinery, not handle
 * inheritance. READY is set by the child once it is fully loaded and running
 * its own code; GO is set by the thread the parent injects INTO the child. */
#define S32_EVENT_READY  L"VibeOSSpawn32ChildReady"
#define S32_EVENT_GO     L"VibeOSSpawn32RemoteRan"
/* Why a THIRD event. A thread's exit code only exists while its process does:
 * the child used to exit the moment the injected thread signalled it, and
 * ExitProcess overwrote that thread's 0x0DEF with the process code before the
 * parent could read it -- rung 7 reported the CHILD's verdict back to itself.
 * So the child holds still until the parent has collected rung 7. */
#define S32_EVENT_DONE   L"VibeOSSpawn32ParentDone"

/* How long to give a child that should exit immediately. Long enough that a
 * loaded host is not the reason, short enough that a hung child cannot eat the
 * ladder's drain budget -- one stalled guest has cost four later guests their
 * verdicts before now. */
#define CHILD_WAIT_MS 30000

#define CHILD_ARG L" child"

static void Say(const char *msg)
{
    /* OutputDebugStringA reaches serial via the guest DbgPrint forwarding
     * (int 2d DebugService -> [GUEST32] in the wow64 VEH). */
    OutputDebugStringA(msg);
}

static void SayHex(const char *label, DWORD v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[96];
    int i = 0, n;
    while (label[i] && i < 72) { buf[i] = label[i]; i++; }
    buf[i++] = ' '; buf[i++] = '0'; buf[i++] = 'x';
    for (n = 28; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    Say(buf);
}

/* Does the command line carry our marker? Searched as a substring rather than
 * parsed as argv: there is no CRT here to split it, and the marker is one we
 * chose that nothing else on the media uses. */
static BOOL HasChildMarker(void)
{
    const WCHAR *p = GetCommandLineW();
    const WCHAR *needle = CHILD_ARG;
    int i;

    if (!p)
        return FALSE;
    for (; *p; p++) {
        for (i = 0; needle[i] && p[i] == needle[i]; i++)
            ;
        if (!needle[i])
            return TRUE;
    }
    return FALSE;
}

/* A SECOND, independent child signal, and the reason is a hazard rather than
 * tidiness: this guest self-spawns, so if the child fails to recognise itself
 * it runs the PARENT path and spawns its own child, and that is a fork bomb
 * inside the guest. The first signal is the command line -- which is precisely
 * one of the things a WoW64 bug could mangle, and mangling it is the exact
 * condition that triggers the bomb. A guest whose safety guard fails in the
 * same failure it exists to measure is not guarded.
 *
 * So: a named mutex, which does not travel through argument marshalling at
 * all. Whoever creates it first is the parent; anyone who finds it already
 * there is a child. Recursion is capped at one level even if the command line
 * is destroyed entirely, and the two signals are OR'd so a fork bomb now needs
 * BOTH mechanisms to fail at once. The handle is deliberately leaked for the
 * process lifetime -- releasing it would let a grandchild claim the role. */
static HANDLE g_Guard;

static BOOL AnotherInstanceIsParent(void)
{
    g_Guard = CreateMutexW(NULL, FALSE, L"VibeOSSpawn32GuestGuard");
    if (!g_Guard)
        return FALSE; /* no verdict from this signal -- the marker still stands */
    return GetLastError() == ERROR_ALREADY_EXISTS;
}

static BOOL IsChildInvocation(void)
{
    /* Both are consulted, and AnotherInstanceIsParent() must run either way so
     * the parent actually CLAIMS the guard before it spawns anything. */
    BOOL guardSaysChild = AnotherInstanceIsParent();
    return HasChildMarker() || guardSaysChild;
}

/* "<our full path>" + CHILD_ARG, quoted because the path contains spaces
 * nowhere today and will the first time someone stages from Program Files. */
static BOOL BuildChildCmdline(WCHAR *buf, DWORD cch)
{
    DWORD n;
    int i = 0;

    /* Leave room for the two quotes, CHILD_ARG and the NUL. */
    if (cch < 32)
        return FALSE;
    buf[i++] = L'"';
    n = GetModuleFileNameW(NULL, buf + i, cch - 24);
    /* Truncation returns the buffer size on this path rather than failing, so
     * a full buffer is indistinguishable from a fit -- treat it as failure. */
    if (n == 0 || n >= cch - 24)
        return FALSE;
    i += (int)n;
    buf[i++] = L'"';
    for (n = 0; CHILD_ARG[n]; n++)
        buf[i++] = CHILD_ARG[n];
    buf[i] = 0;
    return TRUE;
}

/* The body the PARENT injects into the CHILD with CreateRemoteThread.
 *
 * Its address is taken in the parent and executed in the child, which is only
 * sound because the child is this very same .exe mapped at the same base -- the
 * self-spawn that rung 2 already relies on. Everything it calls goes through
 * spawn32's own IAT, which the child's loader resolved at the same addresses
 * for the same reason. A test that injected into a DIFFERENT image could not
 * do this and would have to write code into the target.
 *
 * It signals a NAMED event rather than touching memory, so the proof that it
 * ran is something the child observes for itself rather than something the
 * parent infers. */
static DWORD WINAPI RemoteBody(LPVOID Unused)
{
    HANDLE Go;
    (void)Unused;
    Say("[spawn32] remote thread: running INSIDE the child\n");
    Go = CreateEventW(NULL, TRUE, FALSE, S32_EVENT_GO);
    if (Go) {
        SetEvent(Go);
        CloseHandle(Go);
    }
    return S32_REMOTE_MAGIC;
}

static DWORD RunChild(void)
{
    HANDLE Ready, Go, Done;
    DWORD waited;

    Say("[spawn32] child: running\n");

    /* Announce that the child's own code is executing. Until this fires the
     * parent must not inject: a target whose loader has not finished is the
     * INITIAL-thread case, a different path entirely, and racing the two would
     * make this guest test whichever one won. */
    Ready = CreateEventW(NULL, TRUE, FALSE, S32_EVENT_READY);
    Go = CreateEventW(NULL, TRUE, FALSE, S32_EVENT_GO);
    if (!Ready || !Go) {
        Say("[spawn32] child: could not create its events\n");
        return S32_CHILD_NOSIGNAL;
    }
    SetEvent(Ready);

    waited = WaitForSingleObject(Go, CHILD_WAIT_MS);
    if (waited != WAIT_OBJECT_0) {
        SayHex("[spawn32] child: nothing signalled it, wait", waited);
        return S32_CHILD_NOSIGNAL;
    }
    Say("[spawn32] child: saw the injected thread's signal\n");

    /* Hold the process open until the parent has read the injected thread's
     * exit code -- see S32_EVENT_DONE. A timeout here is not a failure of the
     * thing under test, so it still exits CHILD_MAGIC and lets the parent's
     * own rungs decide. */
    Done = CreateEventW(NULL, TRUE, FALSE, S32_EVENT_DONE);
    if (Done) {
        WaitForSingleObject(Done, CHILD_WAIT_MS);
        CloseHandle(Done);
    }
    return S32_CHILD_MAGIC;
}

static DWORD RunParent(void)
{
    WCHAR cmdline[560];
    STARTUPINFOW si;
    PROCESS_INFORMATION pi;
    DWORD code = 0, waited, remoteTid = 0, remoteCode = 0;
    HANDLE hReady = NULL, hRemote = NULL, hDone = NULL;
    BOOL isWow = FALSE;
    int i;

    for (i = 0; i < (int)sizeof(si); i++)
        ((BYTE *)&si)[i] = 0;
    for (i = 0; i < (int)sizeof(pi); i++)
        ((BYTE *)&pi)[i] = 0;
    si.cb = sizeof(si);

    if (!BuildChildCmdline(cmdline, 560)) {
        Say("[spawn32] could not build the child command line\n");
        return S32_FAIL_MODPATH;
    }

    /* CREATE_SUSPENDED so rungs 2 and 3 have something to look at. A child
     * that ran to completion first would make IsWow64Process a race against a
     * dead process rather than a question about a live one. */
    if (!CreateProcessW(NULL, cmdline, NULL, NULL, FALSE, CREATE_SUSPENDED,
                        NULL, NULL, &si, &pi)) {
        SayHex("[spawn32] rung1 CreateProcessW FAILED, GetLastError", GetLastError());
        Say("[spawn32] rung1 a 32-bit guest cannot start a process\n");
        return S32_FAIL_CREATE;
    }
    Say("[spawn32] rung1 CreateProcessW OK\n");
    SayHex("[spawn32] child pid", pi.dwProcessId);

#ifdef _WIN64
    /* spawn64.exe: the child is this same x86-64 image, so it must NOT be a
     * WoW64 process -- an x86-64 process is not one, on Windows ARM64 any more
     * than on amd64. Under FEX this checks that the guest's CreateProcess made
     * an x86-64 process (wowx64), not a 32-bit one. */
    if (!IsWow64Process(pi.hProcess, &isWow) || isWow) {
        Say("[spawn32] rung2 x86-64 child reports as a WoW64 process\n");
#else
    if (!IsWow64Process(pi.hProcess, &isWow) || !isWow) {
        Say("[spawn32] rung2 child is NOT a WoW64 process\n");
#endif
        TerminateProcess(pi.hProcess, 1);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        return S32_FAIL_NOT_WOW64;
    }
    Say("[spawn32] rung2 child is a 32-bit guest OK\n");

    /* (DWORD)-1 is the documented failure return; 0 would mean it was not
     * suspended, which after CREATE_SUSPENDED is just as wrong. */
    if (ResumeThread(pi.hThread) == (DWORD)-1) {
        SayHex("[spawn32] rung3 ResumeThread FAILED, GetLastError", GetLastError());
        TerminateProcess(pi.hProcess, 1);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        return S32_FAIL_RESUME;
    }
    Say("[spawn32] rung3 ResumeThread OK\n");

    /* rung 6: CreateRemoteThread into a RUNNING 32-bit process. Sequenced here
     * rather than at the end because the child is waiting on it -- and gated on
     * the child's READY event, because injecting into a target whose loader has
     * not finished is the initial-thread path instead, and a race between the
     * two would silently test the wrong one. */
    hReady = CreateEventW(NULL, TRUE, FALSE, S32_EVENT_READY);
    if (!hReady || WaitForSingleObject(hReady, CHILD_WAIT_MS) != WAIT_OBJECT_0) {
        Say("[spawn32] rung6 child never signalled READY\n");
        TerminateProcess(pi.hProcess, 1);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        return S32_FAIL_READY;
    }

    /* DONE must exist BEFORE the child can look for it, and must outlive the
     * child's wait. A named event dies with its last handle: creating it after
     * rung 7, setting it and closing it at once let a child that reached its own
     * CreateEventW a moment later get a FRESH, unsignalled event -- it then sat
     * out its whole 30s wait while ours on its exit ran out too, and the ladder
     * reported 0xBAE4 for a race in this file (four gates on 2026-10-04). */
    hDone = CreateEventW(NULL, TRUE, FALSE, S32_EVENT_DONE);

    hRemote = CreateRemoteThread(pi.hProcess, NULL, 0, RemoteBody, NULL, 0, &remoteTid);
    if (!hRemote) {
        SayHex("[spawn32] rung6 CreateRemoteThread FAILED, GetLastError", GetLastError());
        TerminateProcess(pi.hProcess, 1);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        if (hDone)
            CloseHandle(hDone);
        return S32_FAIL_REMOTE;
    }
    SayHex("[spawn32] rung6 CreateRemoteThread OK, tid", remoteTid);

    /* rung 7: it RAN, not merely got created. The exit code proves the body
     * executed to its return; the child's own verdict proves the signal
     * arrived. Two independent observers of the same event, which is what
     * separates "a thread object exists" from "a thread ran guest code". */
    if (WaitForSingleObject(hRemote, CHILD_WAIT_MS) != WAIT_OBJECT_0 ||
        !GetExitCodeThread(hRemote, &remoteCode) ||
        remoteCode != S32_REMOTE_MAGIC) {
        SayHex("[spawn32] rung7 remote thread did not complete, code", remoteCode);
        TerminateProcess(pi.hProcess, 1);
        CloseHandle(hRemote);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        if (hDone)
            CloseHandle(hDone);
        return S32_FAIL_REMOTE_RUN;
    }
    Say("[spawn32] rung7 remote thread ran to completion OK\n");
    CloseHandle(hRemote);

    /* Release the child now that rung 7 has its answer. */
    if (hDone)
        SetEvent(hDone);

    waited = WaitForSingleObject(pi.hProcess, CHILD_WAIT_MS);
    if (waited != WAIT_OBJECT_0) {
        SayHex("[spawn32] rung4 child did not exit, wait", waited);
        TerminateProcess(pi.hProcess, 1);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        if (hDone)
            CloseHandle(hDone);
        return S32_FAIL_WAIT;
    }
    Say("[spawn32] rung4 child exited OK\n");

    if (!GetExitCodeProcess(pi.hProcess, &code) || code != S32_CHILD_MAGIC) {
        SayHex("[spawn32] rung5 wrong child exit code", code);
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
        if (hDone)
            CloseHandle(hDone);
        return S32_FAIL_EXITCODE;
    }
    Say("[spawn32] rung5 child exit code intact OK\n");

    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    if (hDone)
        CloseHandle(hDone);
    return S32_PASS;
}

int start(void) /* i386 symbol _start (-e _start); x64 symbol start (-e start) */
{
    DWORD verdict = IsChildInvocation() ? RunChild() : RunParent();
    SayHex("[spawn32] verdict", verdict);
    ExitProcess(verdict);
    return (int)verdict;
}
