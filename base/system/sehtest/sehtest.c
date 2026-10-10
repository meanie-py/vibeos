/*
 * User-mode SEH boot self-test — the userland sibling of ntoskrnl/seh_poc.c
 * (crash-value lane; see docs/engineering/dummy-pseh-verdict.md).
 *
 * Compiled with -fms-extensions so __try/__except are the REAL clang SEH
 * keywords (this file deliberately includes no headers so no pseh macro
 * layer can interfere); on i386, retargeted to -windows-msvc, the only env
 * in which clang lowers frame-based fs:[0] SEH. Spawned fire-and-forget by userinit before the
 * shell starts; prints [SEH-USER] lines on the kernel debug serial.
 *
 * Exercises the full user-mode exception chain both ways:
 *   1. hardware fault: NULL write in a noinline callee (clang's mingw SEH
 *      is CALLS-ONLY — a direct deref in the __try body gets its handler
 *      DCE'd, so the fault must arrive through a call frame) →
 *      MmAccessFault → KiDispatchException → KiUserExceptionDispatcher →
 *      ntdll RtlDispatchException → .pdata walk → __C_specific_handler →
 *      filter → RtlUnwind → handler.
 *   2. software raise: kernel32 RaiseException → NtRaiseException →
 *      same dispatch chain.
 *
 * A FAIL (or this process dying silently) means user-mode exception
 * delivery regressed — check KiDispatchExceptionToUser, the dispatch.S
 * swallow branch, and that this module still ships real unwind tables.
 */

unsigned long __stdcall GetVersion(void);         /* anchor kernel32 import */
unsigned long DbgPrint(const char *Format, ...);  /* ntdll */
typedef __UINTPTR_TYPE__ SEH_ULONG_PTR;
void __stdcall RaiseException(unsigned long Code, unsigned long Flags,
                              unsigned long NumArgs, const SEH_ULONG_PTR *Args);

#define SEH_USER_RAISE_STATUS 0xE0DEAD02L
#define STATUS_ACCESS_VIOLATION 0xC0000005L

/* noinline + noipa-ish: keep the fault in a real callee frame */
__attribute__((noinline)) static void FaultingWrite(volatile int *p)
{
    *p = 42;
}

static int TestHardwareFault(void)
{
    volatile int caught = 0;
    volatile long code = 0;

    __try {
        FaultingWrite((volatile int *)0);
        caught = -1; /* unreachable if the fault dispatches */
    } __except (_exception_code() == STATUS_ACCESS_VIOLATION ? 1 : 0) {
        caught = 1;
        code = _exception_code();
    }

    DbgPrint("[SEH-USER] hw-fault: caught=%d code=0x%lx (%s)\n",
             caught, code,
             (caught == 1 && code == STATUS_ACCESS_VIOLATION) ? "PASS" : "FAIL");
    return caught == 1 && code == STATUS_ACCESS_VIOLATION;
}

static int TestSoftwareRaise(void)
{
    volatile int caught = 0;
    volatile long code = 0;

    __try {
        RaiseException(SEH_USER_RAISE_STATUS, 0, 0, 0);
        caught = -1;
    } __except (_exception_code() == SEH_USER_RAISE_STATUS ? 1 : 0) {
        caught = 1;
        code = _exception_code();
    }

    DbgPrint("[SEH-USER] sw-raise: caught=%d code=0x%lx (%s)\n",
             caught, code,
             (caught == 1 && code == SEH_USER_RAISE_STATUS) ? "PASS" : "FAIL");
    return caught == 1 && code == SEH_USER_RAISE_STATUS;
}

static int TestFinally(void)
{
    volatile int ran_finally = 0;
    volatile int caught = 0;

    __try {
        __try {
            RaiseException(SEH_USER_RAISE_STATUS, 0, 0, 0);
        } __finally {
            ran_finally = 1;
        }
    } __except (1) {
        caught = 1;
    }

    DbgPrint("[SEH-USER] finally: ran=%d caught=%d (%s)\n",
             ran_finally, caught,
             (ran_finally == 1 && caught == 1) ? "PASS" : "FAIL");
    return ran_finally == 1 && caught == 1;
}

/* spec2def's stubs are generated Rust. An export with no implementation must
 * raise EXCEPTION_WINE_STUB carrying the module and function names, as the C
 * stubs did; msvcrt's _CrtCheckMemory is a bare `@ stub`. */
#define EXCEPTION_WINE_STUB 0x80000100L
int _CrtCheckMemory(void);
typedef struct { unsigned long Code, Flags; void *Record, *Address;
                 unsigned long Count; SEH_ULONG_PTR Info[15]; } SEH_RECORD;
typedef struct { SEH_RECORD *Record; void *Context; } SEH_POINTERS;
static volatile int StubNamesMatch;

static int StubFilter(SEH_POINTERS *p)
{
    const char *module = (const char *)p->Record->Info[0];
    const char *function = (const char *)p->Record->Info[1];
    const char *want_m = "msvcrt.dll", *want_f = "_CrtCheckMemory";
    int i;

    if (p->Record->Code != EXCEPTION_WINE_STUB || p->Record->Count != 2)
        return 0;
    for (i = 0; want_m[i] && module[i] == want_m[i]; i++);
    StubNamesMatch = !want_m[i] && !module[i];
    for (i = 0; want_f[i] && function[i] == want_f[i]; i++);
    StubNamesMatch &= !want_f[i] && !function[i];
    return 1;
}

static int TestUnimplementedStub(void)
{
    volatile int caught = 0;

    __try {
        _CrtCheckMemory();
    } __except (StubFilter((SEH_POINTERS *)_exception_info())) {
        caught = 1;
    }

    DbgPrint("[SEH-USER] spec-stub: caught=%d names=%d (%s)\n", caught, StubNamesMatch,
             (caught && StubNamesMatch) ? "PASS" : "FAIL");
    return caught && StubNamesMatch;
}

/* kernel32's IsBad*Ptr. Their first Rust translation never probed: the C's
 * self-assignment probe was non-volatile and optimized away, and its __except was
 * dropped, so every non-NULL pointer was "good". The sdk delayimp apitests then
 * wrote through a read-only import name that IsBadWritePtr had cleared. */
int __stdcall IsBadReadPtr(const void *lp, SEH_ULONG_PTR ucb);
int __stdcall IsBadWritePtr(void *lp, SEH_ULONG_PTR ucb);
int __stdcall IsBadStringPtrA(const char *lpsz, SEH_ULONG_PTR ucchMax);
int __stdcall IsBadCodePtr(void *lpfn);
static const char ReadOnlyString[] = "read-only";

static int TestBadPtrProbes(void)
{
    volatile char writable[16] = { 'w', 0 };
    void *unmapped = (void *)0x1000; /* the first 64K are never mapped */
    int ok = 1;

    ok &= !IsBadReadPtr((const void *)writable, sizeof(writable));
    ok &= !IsBadWritePtr((void *)writable, sizeof(writable));
    ok &= !IsBadReadPtr(ReadOnlyString, sizeof(ReadOnlyString));
    ok &= IsBadWritePtr((void *)ReadOnlyString, 1);
    ok &= IsBadReadPtr(unmapped, 1);
    ok &= IsBadWritePtr(unmapped, 1);
    ok &= !IsBadStringPtrA(ReadOnlyString, 1000);
    ok &= IsBadStringPtrA((const char *)unmapped, 1);
    ok &= !IsBadCodePtr((void *)TestBadPtrProbes);

    DbgPrint("[SEH-USER] bad-ptr-probes: (%s)\n", ok ? "PASS" : "FAIL");
    return ok;
}

#ifdef __x86_64__
/* A debug print whose buffer the caller does not own. With no debugger
 * attached the kernel's KdpStub echoes BREAKPOINT_PRINT to the console UART,
 * and that echo once read the caller's buffer in place: a user-mode Rust
 * panic whose backtrace walked into garbage bugchecked the kernel (0x1E in
 * KdpStub). The print must fail with STATUS_ACCESS_VIOLATION, and the system
 * must still be up to say so. Same register convention as rtl's amd64
 * DebugService: eax = service, rcx = buffer, edx = length. */
#define BREAKPOINT_PRINT 1
static int TestBadPrintBuffer(void)
{
    long status;

    __asm__ volatile("int $0x2d\n\tint3"
                     : "=a"(status)
                     : "a"(BREAKPOINT_PRINT),
                       "c"((void *)0xFFFF800000001000ULL), /* kernel space */
                       "d"(64)
                     : "r8", "r9", "memory");

    DbgPrint("[SEH-USER] bad-print-buffer: status=0x%lx (%s)\n", status,
             status == STATUS_ACCESS_VIOLATION ? "PASS" : "FAIL");
    return status == STATUS_ACCESS_VIOLATION;
}
#endif

/* The top-level __except every Win32 thread starts under: kernel32's
 * BaseProcessStartup/BaseThreadStartup wrap the start routine in
 * __except(UnhandledExceptionFilter(...)), which calls the filter an app set
 * with SetUnhandledExceptionFilter and, on EXCEPTION_EXECUTE_HANDLER, exits
 * with the exception code. The first Rust translation kept only the __try, so
 * an unhandled fault skipped all of that and csrss ended the process as 0x80.
 * A child re-runs this exe with "crash", installs a filter, and faults with no
 * handler of its own; the parent wants the access-violation exit code. */
typedef long (__stdcall *TOP_LEVEL_FILTER)(SEH_POINTERS *);
TOP_LEVEL_FILTER __stdcall SetUnhandledExceptionFilter(TOP_LEVEL_FILTER Filter);
unsigned int __stdcall SetErrorMode(unsigned int Mode);
char *__stdcall GetCommandLineA(void);
unsigned long __stdcall GetModuleFileNameA(void *Module, char *Name, unsigned long Size);
typedef struct {
    unsigned long cb;
    char *lpReserved, *lpDesktop, *lpTitle;
    unsigned long dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars;
    unsigned long dwFillAttribute, dwFlags;
    unsigned short wShowWindow, cbReserved2;
    unsigned char *lpReserved2;
    void *hStdInput, *hStdOutput, *hStdError;
} SEH_STARTUPINFOA;
typedef struct {
    void *hProcess, *hThread;
    unsigned long dwProcessId, dwThreadId;
} SEH_PROCESS_INFORMATION;
int __stdcall CreateProcessA(const char *App, char *CmdLine, void *ProcAttr, void *ThreadAttr,
                             int Inherit, unsigned long Flags, void *Env, const char *Dir,
                             SEH_STARTUPINFOA *Si, SEH_PROCESS_INFORMATION *Pi);
unsigned long __stdcall WaitForSingleObject(void *Handle, unsigned long Ms);
int __stdcall GetExitCodeProcess(void *Process, unsigned long *Code);
int __stdcall CloseHandle(void *Handle);

#define SEM_NOGPFAULTERRORBOX 0x0002

static long __stdcall ChildTopLevelFilter(SEH_POINTERS *p)
{
    DbgPrint("[SEH-USER] top-level filter: child saw code=0x%lx\n", p->Record->Code);
    return 1; /* EXCEPTION_EXECUTE_HANDLER: exit with the exception code */
}

static int IsCrashChild(void)
{
    const char *c = GetCommandLineA();
    for (; *c; c++)
        if (c[0] == ' ' && c[1] == 'c' && c[2] == 'r' && c[3] == 'a' && c[4] == 's' &&
            c[5] == 'h' && (c[6] == 0 || c[6] == ' '))
            return 1;
    return 0;
}

static void RunCrashChild(void)
{
    SetErrorMode(SEM_NOGPFAULTERRORBOX); /* belt and braces: never a fault box */
    SetUnhandledExceptionFilter(ChildTopLevelFilter);
    FaultingWrite((volatile int *)0); /* no __try anywhere on this stack */
}

static int TestTopLevelFilter(void)
{
    char cmd[300];
    unsigned long n, code = 0, wait;
    SEH_STARTUPINFOA si = { 0 };
    SEH_PROCESS_INFORMATION pi = { 0 };
    const char *arg = " crash";
    int ok;

    n = GetModuleFileNameA(0, cmd + 1, sizeof(cmd) - 16);
    cmd[0] = '"';
    cmd[n + 1] = '"';
    for (n += 2; *arg; arg++)
        cmd[n++] = *arg;
    cmd[n] = 0;
    si.cb = sizeof(si);
    if (!CreateProcessA(0, cmd, 0, 0, 0, 0, 0, 0, &si, &pi)) {
        DbgPrint("[SEH-USER] top-level filter: CreateProcess failed (FAIL)\n");
        return 0;
    }
    /* The budget is GUEST time, and a starved host stretches the child's run without
     * stretching the guest clock: `vibeos-cli stress` failed this at 60 s with the
     * child's correct exit code already set (amd64 -smp 2 on one host core, and
     * -icount shift=7), i.e. a slow child read as a broken filter. A hung child still
     * fails, at 5 minutes, and a timeout now says so instead of looking like a FAIL. */
    wait = WaitForSingleObject(pi.hProcess, 300000);
    GetExitCodeProcess(pi.hProcess, &code);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    ok = wait == 0 && code == (unsigned long)STATUS_ACCESS_VIOLATION;
    DbgPrint("[SEH-USER] top-level filter: child exit=0x%lx%s (%s)\n", code,
             wait == 0 ? "" : " (wait TIMED OUT)", ok ? "PASS" : "FAIL");
    return ok;
}

int main(void)
{
    int pass = 1;

    if (IsCrashChild()) {
        RunCrashChild();
        return 2; /* only if the fault did not happen */
    }

    DbgPrint("[SEH-USER] user-mode SEH self-test starting (GetVersion=0x%lx)\n",
             GetVersion());

    pass &= TestSoftwareRaise();   /* least demanding first */
    pass &= TestFinally();
    pass &= TestHardwareFault();   /* the real hardware→userland roundtrip */
    pass &= TestUnimplementedStub();
    pass &= TestBadPtrProbes();
    pass &= TestTopLevelFilter();
#ifdef __x86_64__
    pass &= TestBadPrintBuffer();
#endif

    DbgPrint("[SEH-USER] RESULT: %s\n", pass ? "PASS" : "FAIL");
    return pass ? 0 : 1;
}
