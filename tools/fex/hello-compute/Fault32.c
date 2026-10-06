/*
 * fault32.exe — the i386 GUEST-SEH verification guest for WoW64 E5
 * (docs/engineering/active-lanes/wow64-plan.md). Built like File32.c (no CRT,
 * plain _start), imports kernel32 only. Proves that exceptions raised in guest
 * code are dispatched into the guest's OWN 32-bit SEH with the exact faulting
 * context, and that the handler's continue path resumes execution:
 *
 *   1. Hardware AV, caught + resumed: install a raw fs:[0] frame, load marker
 *      values into ebx/esi/edi, fault on a read of absolute address 0. The
 *      handler must see ExceptionCode 0xC0000005 with AV parameters
 *      (read, address 0), the marker registers in the CONTEXT (proving the
 *      E5 context flush captured the LIVE fault state, not a stale syscall
 *      boundary), then redirect Eip to the recovery stub and continue.
 *      On amd64 this exercises: kernel trap -> 64-bit VEH ->
 *      BTCpuResetToConsistentState flush -> Wow64pDispatchFaultToGuest ->
 *      guest KiUserExceptionDispatcher -> handler -> NtContinue marshaller.
 *   2. Software raise, caught + continued: RaiseException(0xE0F00001,
 *      continuable, 2 params) must reach the same handler with both
 *      parameters intact and resume on ExceptionContinueExecution.
 *      Exercises the E5 NtRaiseException manual marshaller (the flat thunk
 *      fed the 64-bit kernel a 32-bit EXCEPTION_RECORD — garbage).
 *   3. OutputDebugStringA must return WITHOUT invoking our handler: the
 *      marshaller consumes DBG_PRINTEXCEPTION_C like the kernel's KD does
 *      natively (guest kernel32's __except is DUMMY_PSEH — if the print
 *      exception reached guest SEH it would land in OUR frame).
 *
 *   4. GetThreadContext(GetCurrentThread()) must return the GUEST's registers,
 *      into a buffer with a canary right behind it. While this syscall was
 *      flat-thunked the 64-bit kernel wrote its OWN 1232-byte CONTEXT into the
 *      guest's 716-byte one; the canary is the standing negative control for
 *      that overrun, and Esp/Eip are range-checked against the live guest stack
 *      so a host CONTEXT read at i386 offsets cannot pass either.
 *   5. SetThreadContext(GetCurrentThread()) with CONTEXT_CONTROL must redirect
 *      Eip — the classic self-directed context set. PASS is reaching the
 *      landing site; returning from the call at all is the failure.
 *      Rungs 4-5 exercise the NtGet/SetContextThread marshallers, which route
 *      through the CPU layer (BTCpuGet/SetContext) rather than the kernel.
 *
 * Verdict contract: exit 0x600D on PASS; distinct 0xBAFx names the failing
 * rung so serial alone triages ([WINLOGON] fault32.exe exit = 0x...).
 */

#include <windows.h>

static void Say(const char *Msg)
{
    HANDLE Out = GetStdHandle(STD_OUTPUT_HANDLE);
    DWORD N = 0, Len = 0;
    while (Msg[Len]) Len++;
    if (Out && Out != INVALID_HANDLE_VALUE)
        WriteFile(Out, Msg, Len, &N, NULL);
}

/* Raw i386 SEH registration record (fs:[0] chain). */
typedef struct _FAULT32_EXCREG {
    struct _FAULT32_EXCREG *Next;
    void *Handler;
} FAULT32_EXCREG;

static volatile ULONG g_Hits;       /* handler invocations */
static volatile ULONG g_Code;       /* last ExceptionCode seen */
static volatile ULONG g_Addr;       /* last ExceptionAddress */
static volatile ULONG g_NParams;
static volatile ULONG g_P0, g_P1;
static volatile ULONG g_Ebx, g_Esi, g_Edi;

#ifdef _WIN64
/* fault64.exe: the same rungs one width up. x64 has no fs:[0] chain — a frame
 * handler is named by the function's unwind info (.pdata/.xdata) and found by
 * the guest's RtlDispatchException walking those tables. So FaultTarget is a
 * real SEH function whose handler is Fault32Handler, and rung 1 additionally
 * proves the guest dispatcher unwinds .pdata correctly. rbx/rsi/rdi are
 * nonvolatile in the x64 ABI too, so the marker check carries over as is. */
EXCEPTION_DISPOSITION Fault32Handler(EXCEPTION_RECORD *, void *, CONTEXT *, void *);
ULONG FaultTarget(void);
void FaultRecover(void);
__asm__(
    ".text\n"
    ".globl FaultTarget\n"
    ".def FaultTarget; .scl 2; .type 32; .endef\n"
    ".seh_proc FaultTarget\n"
    "FaultTarget:\n"
    "    pushq %rbx\n"
    "    .seh_pushreg %rbx\n"
    "    pushq %rsi\n"
    "    .seh_pushreg %rsi\n"
    "    pushq %rdi\n"
    "    .seh_pushreg %rdi\n"
    "    subq $32, %rsp\n"
    "    .seh_stackalloc 32\n"
    "    .seh_endprologue\n"
    "    .seh_handler Fault32Handler, @except\n"
    "    movl $0xF00D0001, %ebx\n"
    "    movl $0xF00D0002, %esi\n"
    "    movl $0xF00D0003, %edi\n"
    "    movl 0x0, %eax\n"         /* AV: read absolute address 0 */
    "    xorl %eax, %eax\n"        /* reached only if resumed WITHOUT fixup */
    "    jmp 1f\n"
    ".globl FaultRecover\n"
    "FaultRecover:\n"
    "    movl $0x1234, %eax\n"
    "1:\n"
    "    addq $32, %rsp\n"
    "    popq %rdi\n"
    "    popq %rsi\n"
    "    popq %rbx\n"
    "    ret\n"
    ".seh_endproc\n"
    /* CallInFrame(Fn, a1, a2, a3, a4) = Fn(a1, a2, a3, a4) with Fault32Handler
     * as the frame handler — what the i386 build gets from the fs:[0] record
     * spanning start(). Rungs 2-3 call through it, so a raise or a debug print
     * has OUR handler on the stack exactly as it does on i386. */
    ".globl CallInFrame\n"
    ".def CallInFrame; .scl 2; .type 32; .endef\n"
    ".seh_proc CallInFrame\n"
    "CallInFrame:\n"
    "    subq $40, %rsp\n"
    "    .seh_stackalloc 40\n"
    "    .seh_endprologue\n"
    "    .seh_handler Fault32Handler, @except\n"
    "    movq %rcx, %r10\n"
    "    movq %rdx, %rcx\n"
    "    movq %r8, %rdx\n"
    "    movq %r9, %r8\n"
    "    movq 80(%rsp), %r9\n"     /* a4: 40 (ours) + 8 (ret) + 32 (home) */
    "    callq *%r10\n"
    "    nop\n"                    /* keep the return address inside the frame */
    "    addq $40, %rsp\n"
    "    ret\n"
    ".seh_endproc\n");
ULONG_PTR CallInFrame(void *Fn, ULONG_PTR A1, ULONG_PTR A2, ULONG_PTR A3, ULONG_PTR A4);
#define CTX_PC(c) ((c)->Rip)
#define CTX_SP(c) ((c)->Rsp)
#define CTX_BX(c) ((c)->Rbx)
#define CTX_SI(c) ((c)->Rsi)
#define CTX_DI(c) ((c)->Rdi)
#define GUEST_PC_LIMIT 0x00007FFFFFFF0000ull
#else
/* Fault + recovery stubs: exact instruction bytes under our control so the
 * handler can retarget Eip without decoding lengths. FaultTarget returns the
 * recovery stub's eax (0x1234) only if the handler redirected Eip there. */
void FaultTarget(void);
void FaultRecover(void);
__asm__(
    ".text\n"
    ".globl _FaultTarget\n"
    "_FaultTarget:\n"
    "    pushl %ebx\n"
    "    pushl %esi\n"
    "    pushl %edi\n"
    "    movl $0xF00D0001, %ebx\n"
    "    movl $0xF00D0002, %esi\n"
    "    movl $0xF00D0003, %edi\n"
    "    movl 0, %eax\n"            /* AV: read absolute address 0 */
    "    xorl %eax, %eax\n"         /* reached only if resumed WITHOUT fixup */
    "    popl %edi\n"
    "    popl %esi\n"
    "    popl %ebx\n"
    "    ret\n"
    ".globl _FaultRecover\n"
    "_FaultRecover:\n"
    "    movl $0x1234, %eax\n"
    "    popl %edi\n"
    "    popl %esi\n"
    "    popl %ebx\n"
    "    ret\n");

#define CTX_PC(c) ((c)->Eip)
#define CTX_SP(c) ((c)->Esp)
#define CTX_BX(c) ((c)->Ebx)
#define CTX_SI(c) ((c)->Esi)
#define CTX_DI(c) ((c)->Edi)
#define GUEST_PC_LIMIT 0x80000000u
#endif

/* i386 SEH handler ABI: cdecl(ExceptionRecord, EstablisherFrame, Context,
 * DispatcherContext) -> EXCEPTION_DISPOSITION. */
#ifndef _WIN64
static
#endif
EXCEPTION_DISPOSITION __cdecl
Fault32Handler(EXCEPTION_RECORD *Rec, void *Frame, CONTEXT *Ctx, void *Disp)
{
    (void)Frame; (void)Disp;
    g_Hits++;
    g_Code = Rec->ExceptionCode;
    g_Addr = (ULONG)(ULONG_PTR)Rec->ExceptionAddress;
    g_NParams = Rec->NumberParameters;
    g_P0 = Rec->NumberParameters > 0 ? (ULONG)Rec->ExceptionInformation[0] : 0xEE;
    g_P1 = Rec->NumberParameters > 1 ? (ULONG)Rec->ExceptionInformation[1] : 0xEE;
    g_Ebx = (ULONG)CTX_BX(Ctx);
    g_Esi = (ULONG)CTX_SI(Ctx);
    g_Edi = (ULONG)CTX_DI(Ctx);
    if (Rec->ExceptionCode == 0xC0000005)
        CTX_PC(Ctx) = (ULONG_PTR)FaultRecover; /* skip the faulting insn */
    return ExceptionContinueExecution;
}

#ifdef _WIN64
/* No chain to link: FaultTarget's unwind info names the handler. */
static void PushFrame(FAULT32_EXCREG *Reg) { (void)Reg; }
static void PopFrame(FAULT32_EXCREG *Reg) { (void)Reg; }
#else
/* NOTE: the registration record must live ON THE THREAD STACK —
 * RtlDispatchException validates fs:[0] entries against the TEB stack
 * limits and skips out-of-stack records. */
static void PushFrame(FAULT32_EXCREG *Reg)
{
    FAULT32_EXCREG *Prev;
    __asm__ __volatile__("movl %%fs:0, %0" : "=r"(Prev));
    Reg->Next = Prev;
    Reg->Handler = (void *)Fault32Handler;
    __asm__ __volatile__("movl %0, %%fs:0" : : "r"(Reg) : "memory");
}

static void PopFrame(FAULT32_EXCREG *Reg)
{
    __asm__ __volatile__("movl %0, %%fs:0" : : "r"(Reg->Next) : "memory");
}

#endif

/* Rung 5's landing site. Reached ONLY by SetThreadContext redirecting Eip here,
 * never by a call — so it must not return. Esp on arrival is whatever rung 4's
 * GetThreadContext reported, which is inside the ntdll32/kernel32 frames BELOW
 * ours: valid committed stack, so Say/ExitProcess can push there safely. */
static __attribute__((noinline)) void Fault32Landed(void)
{
    Say("[FAULT32] rung5 PASS: SetThreadContext(self) redirected Eip\n");
    Say("[FAULT32] ALL PASS\n");
    ExitProcess(0x600D);
}

/* A CONTEXT with a tripwire immediately behind it. The i386 CONTEXT is 0x2CC
 * bytes; the host's is 1232 (amd64) / 912 (arm64). While NtGetContextThread was
 * flat-thunked, the 64-bit kernel wrote ITS CONTEXT into this 716-byte buffer —
 * a >500-byte overrun straight past the end. This canary is the negative
 * control for that bug: it fails if the overrun ever comes back. */
typedef struct _FAULT32_CTXBUF
{
    CONTEXT Ctx;
    ULONG   Canary[8];
} FAULT32_CTXBUF;

#define FAULT32_CANARY(i) (0xC0DE0000u + (ULONG)(i))

int start(void) /* i386 symbol _start (-e _start); x64 symbol start (-e start) */
{
    FAULT32_EXCREG Reg; /* on-stack: see PushFrame note */
    FAULT32_CTXBUF B;
    ULONG_PTR MyEsp;
    ULONG i;
    ULONG Ret;

    Say("[FAULT32] start\n");
    PushFrame(&Reg);

    /* -- Rung 1: hardware AV -> guest SEH -> resume ----------------------- */
    Ret = 0;
#ifdef _WIN64
    Ret = FaultTarget();
#else
    __asm__ __volatile__("call _FaultTarget" : "=a"(Ret) : : "ecx", "edx", "memory");
#endif
    if (Ret != 0x1234)
        { PopFrame(&Reg); Say("[FAULT32] rung1 FAIL: no recovery\n"); ExitProcess(0xBAF1); }
    if (g_Hits != 1 || g_Code != 0xC0000005)
        { PopFrame(&Reg); Say("[FAULT32] rung1 FAIL: code/hits\n");   ExitProcess(0xBAF1); }
    if (g_NParams < 2 || g_P0 != 0 /* read */ || g_P1 != 0 /* address 0 */)
        { PopFrame(&Reg); Say("[FAULT32] rung1 FAIL: AV params\n");   ExitProcess(0xBAF2); }
    if (g_Ebx != 0xF00D0001 || g_Esi != 0xF00D0002 || g_Edi != 0xF00D0003)
        { PopFrame(&Reg); Say("[FAULT32] rung1 FAIL: stale context regs\n"); ExitProcess(0xBAF3); }
    Say("[FAULT32] rung1 PASS: AV caught, live context, resumed\n");

    /* -- Rung 2: software raise -> guest SEH -> continue ------------------ */
    {
        ULONG_PTR Args[2] = { 0xAB, 0xCD };
#ifdef _WIN64
        CallInFrame((void *)RaiseException, 0xE0F00001, 0 /* continuable */, 2, (ULONG_PTR)Args);
#else
        RaiseException(0xE0F00001, 0 /* continuable */, 2, Args);
#endif
        if (g_Hits != 2 || g_Code != 0xE0F00001 || g_P0 != 0xAB || g_P1 != 0xCD)
            { PopFrame(&Reg); Say("[FAULT32] rung2 FAIL: raise not seen\n"); ExitProcess(0xBAF4); }
    }
    Say("[FAULT32] rung2 PASS: RaiseException caught + continued\n");

    /* -- Rung 3: OutputDebugStringA consumed, NOT dispatched to our SEH --- */
#ifdef _WIN64
    CallInFrame((void *)OutputDebugStringA, (ULONG_PTR)"[FAULT32] rung3 ODS text\n", 0, 0, 0);
#else
    OutputDebugStringA("[FAULT32] rung3 ODS text\n");
#endif
    if (g_Hits != 2)
        { PopFrame(&Reg); Say("[FAULT32] rung3 FAIL: DBG_PRINTEXCEPTION hit guest SEH\n"); ExitProcess(0xBAF5); }
    Say("[FAULT32] rung3 PASS: ODS consumed\n");

    /* Rungs 4-5 need no handler, and rung 5 never returns here to pop it. */
    PopFrame(&Reg);

    /* -- Rung 4: GetThreadContext(self) -> real guest CONTEXT, no overrun ---- */
#ifdef _WIN64
    __asm__ __volatile__("movq %%rsp, %0" : "=r"(MyEsp));
#else
    __asm__ __volatile__("movl %%esp, %0" : "=r"(MyEsp));
#endif
    for (i = 0; i < 8; i++)
        B.Canary[i] = FAULT32_CANARY(i);
    B.Ctx.ContextFlags = CONTEXT_FULL; /* control | integer | segments */
    if (!GetThreadContext(GetCurrentThread(), &B.Ctx))
        { Say("[FAULT32] rung4 FAIL: GetThreadContext returned FALSE\n"); ExitProcess(0xBAF6); }
    for (i = 0; i < 8; i++)
        if (B.Canary[i] != FAULT32_CANARY(i))
            { Say("[FAULT32] rung4 FAIL: CONTEXT overrun (host-width write)\n"); ExitProcess(0xBAF6); }
    /* Esp must be a real guest stack address just below ours (the syscall is a
     * few frames deeper), and Eip a real guest code address. A host CONTEXT
     * reinterpreted at i386 offsets fails both. */
    if (CTX_SP(&B.Ctx) == 0 || CTX_SP(&B.Ctx) >= MyEsp || (MyEsp - CTX_SP(&B.Ctx)) > 0x4000)
        { Say("[FAULT32] rung4 FAIL: implausible Esp\n"); ExitProcess(0xBAF7); }
    if (CTX_PC(&B.Ctx) == 0 || CTX_PC(&B.Ctx) >= GUEST_PC_LIMIT)
        { Say("[FAULT32] rung4 FAIL: implausible Eip\n"); ExitProcess(0xBAF7); }
    if ((B.Ctx.ContextFlags & CONTEXT_CONTROL) != CONTEXT_CONTROL)
        { Say("[FAULT32] rung4 FAIL: control state not reported\n"); ExitProcess(0xBAF7); }
    Say("[FAULT32] rung4 PASS: guest CONTEXT read back, canary intact\n");

    /* -- Rung 5: SetThreadContext(self) redirects Eip -- never returns -------- */
    B.Ctx.ContextFlags = CONTEXT_CONTROL;
    CTX_PC(&B.Ctx) = (ULONG_PTR)&Fault32Landed;
    SetThreadContext(GetCurrentThread(), &B.Ctx);

    /* Reaching here at all means the redirect did not take effect. */
    Say("[FAULT32] rung5 FAIL: SetThreadContext(self) did not redirect Eip\n");
    ExitProcess(0xBAF8);
    return 0; /* not reached */
}
