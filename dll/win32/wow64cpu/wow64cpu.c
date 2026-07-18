/*
 * PROJECT:     vibeos wow64cpu.dll
 * PURPOSE:     Native (heaven's-gate) WOW64 CPU layer for amd64 — Phase E
 *              (docs/engineering/wow64-plan.md). This is the amd64 sibling of
 *              arm64's libwow64fex.dll: it exports the SAME BTCpu* contract the
 *              shared wow64.dll host layer already drives, but instead of a JIT
 *              it runs the 32-bit guest NATIVELY in x86-64 compatibility mode.
 *
 *              The syscall bridge is the classic mode switch: the guest's
 *              i386 ntdll stub (`mov eax,N; call [Wow64Transition]`) calls into
 *              a below-2GB trampoline that far-jumps CS 0x23 -> 0x33 into
 *              64-bit code, marshals through OUR wow64.dll's
 *              Wow64SystemServiceEx, and iretq's back to compat mode. No
 *              emulator: the silicon executes the 32-bit instructions.
 *
 *              The mode-switch trampolines live in wow64cpu.S; this file owns
 *              the BTCpu* entry points and the C half of the syscall dispatch.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>
#include <stdint.h>

ULONG WINAPI DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

LONG WINAPI NtAllocateVirtualMemory(_In_ HANDLE ProcessHandle,
                                    _Inout_ PVOID *BaseAddress,
                                    _In_ ULONG_PTR ZeroBits,
                                    _Inout_ PSIZE_T RegionSize,
                                    _In_ ULONG AllocationType,
                                    _In_ ULONG Protect);

/* Provided by OUR wow64.dll (the shared host layer). wow64cpu imports exactly
 * the two symbols the arm64 CPU layer imports too. */
ULONG WINAPI Wow64SystemServiceEx(ULONG ServiceNumber, PULONG Arguments);

#define WOW64_CURRENT_PROCESS   ((HANDLE)(LONG_PTR)-1)
#define WOW64_ZEROBITS_BELOW_2G ((ULONG_PTR)0x7FFFFFFF) /* NtAVM mask form */

/* i386 CONTEXT field offsets — the CPU-area WOW64_CONTEXT the host installs.
 * Kept in sync with wow64cpu.S (WCTX_*). */
#define CTX_EAX(c) (*(volatile ULONG *)((PBYTE)(c) + 0xB0))
#define CTX_EIP(c) (*(volatile ULONG *)((PBYTE)(c) + 0xB8))
#define CTX_EFL(c) (*(volatile ULONG *)((PBYTE)(c) + 0xC0))
#define CTX_ESP(c) (*(volatile ULONG *)((PBYTE)(c) + 0xC4))

/* CPU-area header (wow64.c: 16-byte header, WOW64_CONTEXT at +16; +2 = Machine,
 * +8 = the saved host rsp slot wow64cpu.S owns). +4 is ours: set when
 * BTCpuSetContext replaced the guest context mid-service, so the dispatcher
 * must resume the NEW state instead of doing the return-to-stub fixup. The
 * shared bytes are: [0..1] flags, [2..3] Machine, [4..7] REPLACED, [8..15]
 * host rsp. wow64.dll's callback bridge also knows the +4 flag (documented
 * contract) so it can clear it around the callback-return path. */
#define CPU_CTX_REPLACED(ctx) (*(volatile ULONG *)((PBYTE)(ctx) - 12))

/* Maintained-field window copied by Get/SetContext: SegGs..SegSs (0x8C..0xC8).
 * The FP/extended-register tail is live silicon state a native guest never
 * round-trips through the CPU layer, so it is untouched. */
#define CTX_COPY_LO 0x8C
#define CTX_COPY_HI 0xC8

#define WOW64_TEB_TLSSLOTS_OFFSET 0x1480 /* amd64 TEB.TlsSlots */
#define WOW64_TLS_CPURESERVED     1

static PBYTE Wow64CpuArea(VOID)
{
    return (PBYTE)*(PVOID *)((PBYTE)NtCurrentTeb() +
                             WOW64_TEB_TLSSLOTS_OFFSET +
                             WOW64_TLS_CPURESERVED * sizeof(PVOID));
}

/* The below-2GB trampoline page. BTCpuGetBopCode hands its 32-bit entry to the
 * host, which patches the guest ntdll32's Wow64Transition slot to point here. */
static PVOID Wow64CpuBopPage;

/* The 64-bit syscall landing pad (wow64cpu.S). The low trampoline far-jumps to
 * 64-bit mode and `jmp`s here with r10d = guest EAX, rsp = guest ESP, r15 =
 * CPU-area context. Declared as data so we can embed its absolute address in
 * the runtime-emitted `movabs` — it is NEVER called as a C function. */
extern BYTE Wow64CpuSyscall64[];

/*
 * Wow64CpuDispatch — the C half of the 32->64 syscall bridge. wow64cpu.S has
 * already saved the guest GP registers into the CPU-area context and switched
 * to a 64-bit stack; on return it re-enters the guest from that same context.
 *
 *   Ctx      = the CPU-area WOW64_CONTEXT (Eax already = guest syscall number)
 *   GuestEsp = the guest ESP at the `call [Wow64Transition]` site, i.e.
 *              [GuestEsp]   = return-into-stub, [GuestEsp+4] = return-to-caller,
 *              [GuestEsp+8] = first syscall argument (i386 stdcall stack shape).
 */
VOID WINAPI Wow64CpuDispatch(PVOID Ctx, ULONG GuestEsp)
{
    ULONG Service = CTX_EAX(Ctx);
    PULONG Arguments = (PULONG)(ULONG_PTR)(GuestEsp + 8);
    ULONG Result = Wow64SystemServiceEx(Service, Arguments);

    /* A guest NtContinue routed through BTCpuSetContext during this service:
     * the CPU-area context now IS the state to resume (new Eip/Esp/Eax) —
     * the normal return-to-stub fixup below would clobber it. */
    if (CPU_CTX_REPLACED(Ctx))
    {
        CPU_CTX_REPLACED(Ctx) = 0;
        return;
    }

    /* Return to the guest: EAX = result, pop the stub return address. */
    CTX_EAX(Ctx) = Result;
    CTX_EIP(Ctx) = *(volatile ULONG *)(ULONG_PTR)GuestEsp; /* return-into-stub */
    CTX_ESP(Ctx) = GuestEsp + 4;
}

/*
 * BTCpuProcessInit — allocate the below-2GB trampoline and emit the heaven's-
 * gate byte sequence. Called once per process by the host before BTCpuSimulate.
 */
VOID WINAPI BTCpuProcessInit(VOID)
{
    PVOID Page = NULL;
    SIZE_T Size = 0x1000;
    LONG Status = NtAllocateVirtualMemory(WOW64_CURRENT_PROCESS, &Page,
                                          WOW64_ZEROBITS_BELOW_2G, &Size,
                                          MEM_RESERVE | MEM_COMMIT,
                                          PAGE_EXECUTE_READWRITE);
    PBYTE P;

    if (Status < 0 || !Page || (ULONG_PTR)Page >= 0x80000000u)
    {
        DbgPrint("[WOW64CPU] FAIL: below-2GB trampoline alloc %08lx base=%p\n",
                 Status, Page);
        return;
    }

    P = (PBYTE)Page;
    /* +0  (32-bit, compat mode): far-jump CS 0x33 to the 64-bit landing @ +7.
     *     EA <off32> <sel16>  =  jmp far 0x33:(Page+7) */
    P[0] = 0xEA;
    *(ULONG *)(P + 1) = (ULONG)(ULONG_PTR)Page + 7;
    *(USHORT *)(P + 5) = 0x0033;
    /* +7  (64-bit): stash guest EAX, then jump to the real 64-bit dispatcher.
     *     41 89 C2              mov  r10d, eax
     *     48 B8 <imm64>         movabs rax, Wow64CpuSyscall64
     *     FF E0                 jmp  rax */
    P[7] = 0x41; P[8] = 0x89; P[9] = 0xC2;
    P[10] = 0x48; P[11] = 0xB8;
    *(ULONGLONG *)(P + 12) = (ULONGLONG)(ULONG_PTR)&Wow64CpuSyscall64[0];
    P[20] = 0xFF; P[21] = 0xE0;

    Wow64CpuBopPage = Page;
    DbgPrint("[WOW64CPU] BTCpuProcessInit: heaven's-gate trampoline @ %p (64-bit landing %p)\n",
             Page, &Wow64CpuSyscall64[0]);
}

/* Return the 32-bit trampoline entry the host patches into Wow64Transition. */
PVOID WINAPI BTCpuGetBopCode(VOID)
{
    return Wow64CpuBopPage;
}

/* Native execution keeps no per-thread JIT state; the CPU-area context is the
 * whole story. */
VOID WINAPI BTCpuThreadInit(VOID)
{
}

/*
 * BTCpuSetContext — replace the current thread's guest context. Reached from
 * the guest's own NtContinue (wow64_manual.c marshaller) DURING a service
 * dispatch: copy the CONTEXT32 into the CPU-area WOW64_CONTEXT and flag it so
 * Wow64CpuDispatch resumes the new state instead of returning to the stub.
 * Self-thread only (the marshaller always passes the current thread) — a
 * cross-thread SetThreadContext path would need a suspend handshake.
 */
LONG WINAPI BTCpuSetContext(HANDLE Thread, HANDLE Process, PVOID Unknown,
                            PVOID Context32)
{
    PBYTE CpuArea = Wow64CpuArea();
    PBYTE Ctx;
    PBYTE Src = (PBYTE)Context32;
    ULONG i;

    (VOID)Thread; (VOID)Process; (VOID)Unknown;
    if (!Src || !CpuArea)
        return (LONG)0xC000000DL; /* STATUS_INVALID_PARAMETER */
    Ctx = CpuArea + 16;

    /* Segments + integer + control state (SegGs..SegSs). Segments in the
     * source are ignored on entry — EnterGuest hardcodes 0x23/0x2B/0x53 — but
     * copying them keeps the CPU-area a faithful CONTEXT32 for a later
     * GetContext. EFlags gets IF + the reserved bit 1 forced on. */
    for (i = CTX_COPY_LO; i <= CTX_COPY_HI; i += 4)
        *(volatile ULONG *)(Ctx + i) = *(ULONG *)(Src + i);
    CTX_EFL(Ctx) |= 0x202;
    CPU_CTX_REPLACED(Ctx) = 1; /* dispatcher must resume THIS state, not fixup */
    return 0; /* STATUS_SUCCESS */
}

/*
 * BTCpuGetContext — snapshot the current thread's guest context out of the
 * CPU-area WOW64_CONTEXT. The inverse of BTCpuSetContext; the callback bridge
 * (wow64.dll) calls it to save the guest state before entering a nested guest
 * callback, then restores it via BTCpuSetContext afterwards. Self-thread only.
 */
LONG WINAPI BTCpuGetContext(HANDLE Thread, HANDLE Process, PVOID Unknown,
                            PVOID Context32)
{
    PBYTE CpuArea = Wow64CpuArea();
    PBYTE Ctx;
    PBYTE Dst = (PBYTE)Context32;
    ULONG i, Flags;

    (VOID)Thread; (VOID)Process; (VOID)Unknown;
    if (!Dst || !CpuArea)
        return (LONG)0xC000000DL;
    Ctx = CpuArea + 16;

    Flags = *(ULONG *)Dst; /* caller's requested ContextFlags — preserve */
    for (i = CTX_COPY_LO; i <= CTX_COPY_HI; i += 4)
        *(volatile ULONG *)(Dst + i) = *(ULONG *)(Ctx + i);
    *(ULONG *)Dst = Flags;
    return 0;
}

/*
 * BTCpuResetToConsistentState — flush the LIVE guest state into the CPU-area
 * WOW64_CONTEXT at a hardware fault (E5). Called first thing by wow64.dll's
 * VEH for every process exception.
 *
 * The heaven's-gate property makes this trivial: when compat-mode guest code
 * faults, the kernel captures the full register file in the 64-bit trap
 * CONTEXT — the low 32 bits of the captured GPRs (and Rsp/Rip) ARE the guest
 * registers, and SegCs == 0x23 identifies the frame as guest code. Without
 * this flush the CPU area holds the state of the PREVIOUS syscall boundary,
 * so everything downstream (the [FEX-WOW64C] fault diagnostics, GetContext,
 * and the guest-SEH dispatch wow64.dll builds from this context) reads stale
 * values. FEX does the equivalent flush internally on arm64.
 *
 * Never NtContinues — dispatch policy belongs to the host VEH.
 */
LONG WINAPI BTCpuResetToConsistentState(EXCEPTION_POINTERS *Info)
{
    CONTEXT *H = Info->ContextRecord;
    PBYTE CpuArea = Wow64CpuArea();
    PBYTE Ctx;

    if (!CpuArea || H->SegCs != 0x23)
        return 0; /* fault was in host 64-bit code — nothing of the guest's to flush */
    Ctx = CpuArea + 16;

    *(volatile ULONG *)(Ctx + 0x8C) = H->SegGs;    /* SegGs  */
    *(volatile ULONG *)(Ctx + 0x90) = H->SegFs;    /* SegFs  */
    *(volatile ULONG *)(Ctx + 0x94) = H->SegEs;    /* SegEs  */
    *(volatile ULONG *)(Ctx + 0x98) = H->SegDs;    /* SegDs  */
    *(volatile ULONG *)(Ctx + 0x9C) = (ULONG)H->Rdi;
    *(volatile ULONG *)(Ctx + 0xA0) = (ULONG)H->Rsi;
    *(volatile ULONG *)(Ctx + 0xA4) = (ULONG)H->Rbx;
    *(volatile ULONG *)(Ctx + 0xA8) = (ULONG)H->Rdx;
    *(volatile ULONG *)(Ctx + 0xAC) = (ULONG)H->Rcx;
    *(volatile ULONG *)(Ctx + 0xB0) = (ULONG)H->Rax;
    *(volatile ULONG *)(Ctx + 0xB4) = (ULONG)H->Rbp;
    *(volatile ULONG *)(Ctx + 0xB8) = (ULONG)H->Rip; /* Eip */
    *(volatile ULONG *)(Ctx + 0xBC) = H->SegCs;    /* SegCs  */
    *(volatile ULONG *)(Ctx + 0xC0) = H->EFlags;
    *(volatile ULONG *)(Ctx + 0xC4) = (ULONG)H->Rsp; /* Esp */
    *(volatile ULONG *)(Ctx + 0xC8) = H->SegSs;    /* SegSs  */
    return 0;
}

/* No JIT code cache to invalidate — 32-bit code executes natively. */
LONG WINAPI BTCpuNotifyMapViewOfSection(PVOID a, PVOID Base, PVOID c,
                                        SIZE_T d, ULONG e, ULONG f)
{
    (VOID)a; (VOID)Base; (VOID)c; (VOID)d; (VOID)e; (VOID)f;
    return 0; /* STATUS_SUCCESS */
}

BOOL WINAPI DllMain(HINSTANCE hDll, DWORD dwReason, LPVOID lpReserved)
{
    (VOID)hDll; (VOID)dwReason; (VOID)lpReserved;
    return TRUE;
}
