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
#define CTX_ESP(c) (*(volatile ULONG *)((PBYTE)(c) + 0xC4))

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
