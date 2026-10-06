/*
 * thread32.exe — the i386 THREADING guest for WoW64 task #64
 * (docs/engineering/active-lanes/wow64-plan.md). Built with the llvm-mingw
 * i686 target, no CRT (plain _start entry, same as Hello32.c/File32.c),
 * importing only kernel32, loaded through the genuine 32-bit loader.
 *
 * What it proves: a 32-bit guest can create real threads. Until #64 there was
 * no per-thread WoW64 entry at all — wow64.dll's thread state (the CPU area
 * holding the guest register file, TEB32's NtTib stack bounds, BTCpuThreadInit)
 * lived in function-local `static`s inside the process bootstrap, i.e. exactly
 * one thread's worth of storage. NtCreateThread was a loud refusal, and before
 * that a flat thunk that returned SUCCESS while the kernel built a thread with
 * a zeroed trap frame (osk.exe; BUGCHECK 0x1E via KiInitializeUserApc).
 *
 * The rungs are chosen so that each one FAILS LOUDLY for a specific missing
 * piece, rather than all passing off one lucky code path:
 *
 *   1-2 create + join         the thread exists and reaches its own exit
 *   3   parameter             CONTEXT32.Ebx survived (BaseInitializeContext puts
 *                             the thread parameter there, Eax = the routine)
 *   4   exit code             the return value propagated back out
 *   5   distinct TID          it is a real thread, not a callback on ours
 *   6   TLS                   LdrpInitializeThread ran on the new thread —
 *                             i.e. we routed it through the guest loader's
 *                             LdrInitializeThunk instead of jumping straight to
 *                             the entry. Without it TlsGetValue reads an array
 *                             that was never allocated.
 *   7   SEH on the new thread the TEB32 NtTib stack bounds were set. i386
 *                             RtlDispatchException validates every fs:[0]
 *                             record against them and SKIPS out-of-stack
 *                             records, so a thread whose bounds are still the
 *                             kernel's zeroes silently loses every handler.
 *   8-9 4 concurrent threads  the CPU area is genuinely per-thread. A shared
 *                             one cross-contaminates the guest register file;
 *                             distinct stack pointers prove distinct threads.
 *
 * Verdict contract (same as hello32/gui32/win32/file32): exit code 0x600D on
 * PASS; distinct 0xBADx codes name the failing rung so serial alone triages.
 */

#include <windows.h>

#define T32_FAIL_CREATE      0xBAD1
#define T32_FAIL_WAIT        0xBAD2
#define T32_FAIL_PARAM       0xBAD3
#define T32_FAIL_EXITCODE    0xBAD4
#define T32_FAIL_TID         0xBAD5
#define T32_FAIL_TLS         0xBAD6
#define T32_FAIL_SEH         0xBAD7
#define T32_FAIL_TLSALLOC    0xBAD8
#define T32_FAIL_CONCURRENT  0xBAD9
#define T32_FAIL_STACKS      0xBADA
#define T32_FAIL_MULTICREATE 0xBADB
#define T32_PASS             0x600D

#define T32_PARAM   0xC0FFEE01u
#define T32_RETVAL  0x5EED0001u
#define T32_TLSVAL  0x7175E500u
#define T32_NTHREAD 4

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

/* ---- rung 7: guest SEH on the WORKER thread ----------------------------- */

typedef struct _T32_EXCREG {
    struct _T32_EXCREG *Next;
    void *Handler;
} T32_EXCREG;

#ifdef _WIN64
/* thread64.exe: x64 has no fs:[0] chain; the handler is named by
 * T32FaultTarget's unwind info, so rung 7 proves the guest dispatcher walks
 * .pdata on the worker thread (and the TEB stack bounds it checks against). */
EXCEPTION_DISPOSITION T32Handler(EXCEPTION_RECORD *, void *, CONTEXT *, void *);
DWORD T32FaultTarget(void);
void T32FaultRecover(void);
__asm__(
    ".text\n"
    ".globl T32FaultTarget\n"
    ".def T32FaultTarget; .scl 2; .type 32; .endef\n"
    ".seh_proc T32FaultTarget\n"
    "T32FaultTarget:\n"
    "    subq $40, %rsp\n"
    "    .seh_stackalloc 40\n"
    "    .seh_endprologue\n"
    "    .seh_handler T32Handler, @except\n"
    "    movl 0x0, %eax\n"         /* AV: read absolute address 0 */
    "    xorl %eax, %eax\n"        /* reached only if resumed WITHOUT fixup */
    "    jmp 1f\n"
    ".globl T32FaultRecover\n"
    "T32FaultRecover:\n"
    "    movl $0x1234, %eax\n"
    "1:\n"
    "    addq $40, %rsp\n"
    "    ret\n"
    ".seh_endproc\n");
#else
/* Fault + recovery stubs with instruction bytes under our control, so the
 * handler can retarget Eip without decoding lengths. FaultTarget returns the
 * recovery stub's eax (0x1234) only if the handler redirected Eip there. */
void T32FaultTarget(void);
void T32FaultRecover(void);
__asm__(
    ".text\n"
    ".globl _T32FaultTarget\n"
    "_T32FaultTarget:\n"
    "    movl 0, %eax\n"            /* AV: read absolute address 0 */
    "    xorl %eax, %eax\n"         /* reached only if resumed WITHOUT fixup */
    "    ret\n"
    ".globl _T32FaultRecover\n"
    "_T32FaultRecover:\n"
    "    movl $0x1234, %eax\n"
    "    ret\n");

#endif

static volatile DWORD g_SehHits;
static volatile DWORD g_SehCode;
static DWORD g_SkipSeh; /* set by the "noseh" argument — see StrHasNoSeh */

/* An escape hatch for a host whose CPU layer cannot yet dispatch a hardware
 * fault into the guest's own 32-bit SEH: the launcher passes "noseh" and rung 7
 * reports itself SKIPPED, so the rest of the per-thread entry still gets
 * exercised instead of the whole guest being withheld. arm64 needed it until
 * E18 landed the redirect; NO launcher passes it today. Kept because the next
 * CPU layer to be brought up will want it. */
static DWORD StrHasNoSeh(const char *s)
{
    int i;
    for (i = 0; s[i]; i++)
        if (s[i] == 'n' && s[i+1] == 'o' && s[i+2] == 's' &&
            s[i+3] == 'e' && s[i+4] == 'h')
            return 1;
    return 0;
}

/* i386 SEH handler ABI: cdecl(ExceptionRecord, EstablisherFrame, Context,
 * DispatcherContext) -> EXCEPTION_DISPOSITION. */
#ifndef _WIN64
static
#endif
EXCEPTION_DISPOSITION __cdecl
T32Handler(EXCEPTION_RECORD *Rec, void *Frame, CONTEXT *Ctx, void *Disp)
{
    (void)Frame; (void)Disp;
    g_SehHits++;
    g_SehCode = Rec->ExceptionCode;
    if (Rec->ExceptionCode == 0xC0000005)
#ifdef _WIN64
        Ctx->Rip = (ULONG_PTR)T32FaultRecover; /* skip the faulting insn */
#else
        Ctx->Eip = (ULONG)(ULONG_PTR)T32FaultRecover; /* skip the faulting insn */
#endif
    return ExceptionContinueExecution;
}

#ifndef _WIN64
/* The registration record must live ON THIS THREAD'S STACK — that is exactly
 * what RtlDispatchException checks against TEB32's NtTib bounds. */
static void PushFrame(T32_EXCREG *Reg)
{
    T32_EXCREG *Prev;
    __asm__ __volatile__("movl %%fs:0, %0" : "=r"(Prev));
    Reg->Next = Prev;
    Reg->Handler = (void *)T32Handler;
    __asm__ __volatile__("movl %0, %%fs:0" : : "r"(Reg) : "memory");
}

static void PopFrame(T32_EXCREG *Reg)
{
    __asm__ __volatile__("movl %0, %%fs:0" : : "r"(Reg->Next) : "memory");
}

#endif

/* ---- the functional worker ---------------------------------------------- */

static volatile DWORD g_WorkerParam;
static volatile DWORD g_WorkerTid;
static volatile DWORD g_WorkerTlsOk;
static volatile DWORD g_WorkerSehOk;
static volatile DWORD g_WorkerRan;
static DWORD g_TlsIndex;

static DWORD WINAPI Worker(LPVOID Parameter)
{
#ifndef _WIN64
    T32_EXCREG Reg;
#endif
    DWORD Ret = 0;

    g_WorkerRan = 1;
    g_WorkerParam = (DWORD)(ULONG_PTR)Parameter;
    g_WorkerTid = GetCurrentThreadId();

    /* TLS on a thread the guest loader initialized. TlsGetValue indexes
     * TEB32->TlsSlots, which LdrpInitializeThread/LdrpAllocateTls set up. */
    if (TlsSetValue(g_TlsIndex, (LPVOID)(ULONG_PTR)T32_TLSVAL) &&
        TlsGetValue(g_TlsIndex) == (LPVOID)(ULONG_PTR)T32_TLSVAL)
        g_WorkerTlsOk = 1;

    /* Fault and recover through this thread's own fs:[0] chain. */
    if (g_SkipSeh)
    {
        g_WorkerSehOk = 2; /* skipped, reported as such by rung 7 */
    }
    else
    {
#ifdef _WIN64
        Ret = T32FaultTarget();
#else
        PushFrame(&Reg);
        __asm__ __volatile__("call _T32FaultTarget" : "=a"(Ret) : : "ecx", "edx", "memory");
        PopFrame(&Reg);
#endif
        if (g_SehHits == 1 && g_SehCode == 0xC0000005 && Ret == 0x1234)
            g_WorkerSehOk = 1;
    }

    return T32_RETVAL;
}

/* ---- rungs 8-9: concurrency --------------------------------------------- */

static volatile DWORD g_Sum[T32_NTHREAD];
static volatile ULONG_PTR g_Esp[T32_NTHREAD];
/* Workers that have recorded their stack pointer. Each waits here until all
 * have, so every stack is live when rung 9 compares them: a worker that exited
 * first could otherwise hand its freed stack to the next CreateThread, and the
 * rung would read a reused stack as a shared one (amd64 thread64, 2026-10-06,
 * four gate rejections on a correct WoW64). */
static volatile LONG g_Recorded;

#define T32_ROUNDS 8
#define T32_ITERS  5000

static DWORD MixRound(DWORD acc)
{
    DWORD i;
    for (i = 0; i < T32_ITERS; i++)
        acc = acc * 1664525u + 1013904223u;
    return acc;
}

/* The reference value, recomputed by the main thread. */
static DWORD Mix(DWORD seed)
{
    DWORD r, acc = seed;
    for (r = 0; r < T32_ROUNDS; r++)
        acc = MixRound(acc);
    return acc;
}

static DWORD WINAPI SpinWorker(LPVOID Parameter)
{
    DWORD Index = (DWORD)(ULONG_PTR)Parameter;
    ULONG_PTR Esp;
    DWORD r, acc = Index + 1;

#ifdef _WIN64
    __asm__ __volatile__("movq %%rsp, %0" : "=r"(Esp));
#else
    __asm__ __volatile__("movl %%esp, %0" : "=r"(Esp));
#endif
    g_Esp[Index] = Esp;
    InterlockedIncrement(&g_Recorded);
    while (g_Recorded < T32_NTHREAD)
        Sleep(0);

    /* Yield between rounds: every Sleep(0) is a syscall, i.e. a full
     * 32->64->32 mode switch through THIS thread's CPU area, with three other
     * guest threads doing the same concurrently. That is what would expose a
     * shared CPU area — a wrong accumulator here means one thread's register
     * file was clobbered by another's. Raw iteration count would not; the
     * switches are the test. */
    for (r = 0; r < T32_ROUNDS; r++)
    {
        acc = MixRound(acc);
        Sleep(0);
    }
    g_Sum[Index] = acc;
    return 0;
}

int start(void) /* i386 symbol _start (-e _start); x64 symbol start (-e start) */
{
    HANDLE Thread;
    HANDLE Threads[T32_NTHREAD];
    DWORD MainTid, Code, i, j;

    Say("[THREAD32] start\n");
    MainTid = GetCurrentThreadId();
    g_SkipSeh = StrHasNoSeh(GetCommandLineA());
    if (g_SkipSeh)
        Say("[THREAD32] rung7 (guest SEH) will be SKIPPED by request\n");

    g_TlsIndex = TlsAlloc();
    if (g_TlsIndex == TLS_OUT_OF_INDEXES)
    {
        Say("[THREAD32] FAIL: TlsAlloc\n");
        ExitProcess(T32_FAIL_TLSALLOC);
    }

    /* -- rung 1: create ---------------------------------------------------- */
    Thread = CreateThread(NULL, 0, Worker, (LPVOID)(ULONG_PTR)T32_PARAM, 0, NULL);
    if (!Thread)
    {
        SayHex("[THREAD32] FAIL rung1: CreateThread err", GetLastError());
        ExitProcess(T32_FAIL_CREATE);
    }
    Say("[THREAD32] rung1 PASS: CreateThread returned a handle\n");

    /* -- rung 2: it runs and exits ----------------------------------------- */
    if (WaitForSingleObject(Thread, 20000) != WAIT_OBJECT_0)
    {
        SayHex("[THREAD32] FAIL rung2: worker never exited; ran=", g_WorkerRan);
        ExitProcess(T32_FAIL_WAIT);
    }
    Say("[THREAD32] rung2 PASS: worker ran to completion\n");

    /* -- rung 3: the parameter survived (CONTEXT32.Ebx) -------------------- */
    if (g_WorkerParam != T32_PARAM)
    {
        SayHex("[THREAD32] FAIL rung3: param", g_WorkerParam);
        ExitProcess(T32_FAIL_PARAM);
    }
    Say("[THREAD32] rung3 PASS: thread parameter intact\n");

    /* -- rung 4: the exit code propagated ---------------------------------- */
    Code = 0;
    if (!GetExitCodeThread(Thread, &Code) || Code != T32_RETVAL)
    {
        SayHex("[THREAD32] FAIL rung4: exit code", Code);
        ExitProcess(T32_FAIL_EXITCODE);
    }
    Say("[THREAD32] rung4 PASS: exit code propagated\n");
    CloseHandle(Thread);

    /* -- rung 5: a genuinely distinct thread ------------------------------- */
    if (!g_WorkerTid || g_WorkerTid == MainTid)
    {
        SayHex("[THREAD32] FAIL rung5: worker tid", g_WorkerTid);
        ExitProcess(T32_FAIL_TID);
    }
    Say("[THREAD32] rung5 PASS: worker had its own thread id\n");

    /* -- rung 6: TLS (proves LdrpInitializeThread ran on it) --------------- */
    if (!g_WorkerTlsOk)
    {
        Say("[THREAD32] FAIL rung6: TLS unusable on the new thread — the guest "
            "loader's per-thread init did not run\n");
        ExitProcess(T32_FAIL_TLS);
    }
    Say("[THREAD32] rung6 PASS: TLS works on the new thread\n");

    /* -- rung 7: SEH on the new thread (TEB32 NtTib bounds) ---------------- */
    if (g_WorkerSehOk == 2)
    {
        Say("[THREAD32] rung7 SKIPPED: guest SEH redirect not available on this "
            "host arch (pre-existing, unrelated to threading)\n");
    }
    else if (!g_WorkerSehOk)
    {
        SayHex("[THREAD32] FAIL rung7: SEH on the new thread; hits", g_SehHits);
        SayHex("[THREAD32]        last exception code", g_SehCode);
        ExitProcess(T32_FAIL_SEH);
    }
    else
    {
        Say("[THREAD32] rung7 PASS: guest SEH dispatched on the new thread\n");
    }

    /* -- rungs 8-9: four concurrent threads -------------------------------- */
    for (i = 0; i < T32_NTHREAD; i++)
    {
        Threads[i] = CreateThread(NULL, 0, SpinWorker, (LPVOID)(ULONG_PTR)i, 0, NULL);
        if (!Threads[i])
        {
            SayHex("[THREAD32] FAIL rung8: CreateThread index", i);
            ExitProcess(T32_FAIL_MULTICREATE);
        }
    }
    /* Check the full return value, not just WAIT_TIMEOUT: WaitForMultipleObjects
     * passes an ARRAY of handles, 4-byte entries in a 32-bit guest against the
     * host's 8-byte ones, so a missing marshaller shows up as WAIT_FAILED and a
     * timeout-only test would sail past it and blame the results instead. */
    Code = WaitForMultipleObjects(T32_NTHREAD, Threads, TRUE, 30000);
    if (Code != WAIT_OBJECT_0)
    {
        SayHex("[THREAD32] FAIL rung8: WaitForMultipleObjects returned", Code);
        SayHex("[THREAD32]        last error", GetLastError());
        ExitProcess(T32_FAIL_CONCURRENT);
    }
    for (i = 0; i < T32_NTHREAD; i++)
    {
        DWORD Want = Mix(i + 1);
        CloseHandle(Threads[i]);
        if (g_Sum[i] != Want)
        {
            SayHex("[THREAD32] FAIL rung8: wrong result from thread", i);
            SayHex("[THREAD32]        got ", g_Sum[i]);
            SayHex("[THREAD32]        want", Want);
            ExitProcess(T32_FAIL_CONCURRENT);
        }
    }
    Say("[THREAD32] rung8 PASS: 4 concurrent threads each computed correctly\n");

    /* Distinct stacks: a shared CPU area would not give four distinct Esp. */
    for (i = 0; i < T32_NTHREAD; i++)
    {
        if (!g_Esp[i])
        {
            SayHex("[THREAD32] FAIL rung9: no stack recorded for thread", i);
            ExitProcess(T32_FAIL_STACKS);
        }
        for (j = i + 1; j < T32_NTHREAD; j++)
        {
            /* ~(ULONG_PTR)0xFFFF, not ~0xFFFFu: the latter is 0xFFFF0000
             * zero-extended, which drops bits 32-63 of an x64 stack address. */
            if ((g_Esp[i] & ~(ULONG_PTR)0xFFFF) == (g_Esp[j] & ~(ULONG_PTR)0xFFFF))
            {
                SayHex("[THREAD32] FAIL rung9: shared stack, thread", i);
                SayHex("[THREAD32]        collides with thread", j);
                SayHex("[THREAD32]        esp", (DWORD)g_Esp[i]);
                ExitProcess(T32_FAIL_STACKS);
            }
        }
    }
    Say("[THREAD32] rung9 PASS: every thread had its own stack\n");

    TlsFree(g_TlsIndex);
    Say("[THREAD32] ALL PASS\n");
    ExitProcess(T32_PASS);
    return 0;
}
