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

/* i386 SEH handler ABI: cdecl(ExceptionRecord, EstablisherFrame, Context,
 * DispatcherContext) -> EXCEPTION_DISPOSITION. */
static EXCEPTION_DISPOSITION __cdecl
Fault32Handler(EXCEPTION_RECORD *Rec, void *Frame, CONTEXT *Ctx, void *Disp)
{
    (void)Frame; (void)Disp;
    g_Hits++;
    g_Code = Rec->ExceptionCode;
    g_Addr = (ULONG)(ULONG_PTR)Rec->ExceptionAddress;
    g_NParams = Rec->NumberParameters;
    g_P0 = Rec->NumberParameters > 0 ? (ULONG)Rec->ExceptionInformation[0] : 0xEE;
    g_P1 = Rec->NumberParameters > 1 ? (ULONG)Rec->ExceptionInformation[1] : 0xEE;
    g_Ebx = Ctx->Ebx;
    g_Esi = Ctx->Esi;
    g_Edi = Ctx->Edi;
    if (Rec->ExceptionCode == 0xC0000005)
        Ctx->Eip = (ULONG)(ULONG_PTR)FaultRecover; /* skip the faulting insn */
    return ExceptionContinueExecution;
}

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

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    FAULT32_EXCREG Reg; /* on-stack: see PushFrame note */
    ULONG Ret;

    Say("[FAULT32] start\n");
    PushFrame(&Reg);

    /* -- Rung 1: hardware AV -> guest SEH -> resume ----------------------- */
    Ret = 0;
    __asm__ __volatile__("call _FaultTarget" : "=a"(Ret) : : "ecx", "edx", "memory");
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
        RaiseException(0xE0F00001, 0 /* continuable */, 2, Args);
        if (g_Hits != 2 || g_Code != 0xE0F00001 || g_P0 != 0xAB || g_P1 != 0xCD)
            { PopFrame(&Reg); Say("[FAULT32] rung2 FAIL: raise not seen\n"); ExitProcess(0xBAF4); }
    }
    Say("[FAULT32] rung2 PASS: RaiseException caught + continued\n");

    /* -- Rung 3: OutputDebugStringA consumed, NOT dispatched to our SEH --- */
    OutputDebugStringA("[FAULT32] rung3 ODS text\n");
    if (g_Hits != 2)
        { PopFrame(&Reg); Say("[FAULT32] rung3 FAIL: DBG_PRINTEXCEPTION hit guest SEH\n"); ExitProcess(0xBAF5); }
    Say("[FAULT32] rung3 PASS: ODS consumed\n");

    PopFrame(&Reg);
    Say("[FAULT32] ALL PASS\n");
    ExitProcess(0x600D);
    return 0; /* not reached */
}
