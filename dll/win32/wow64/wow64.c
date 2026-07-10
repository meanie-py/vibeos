/*
 * PROJECT:     vibeos wow64.dll
 * PURPOSE:     Win10+-model WOW64 host layer — Phase A/B skeleton
 *              (docs/engineering/wow64-plan.md). This is OUR layer of the
 *              three-layer BTCpu architecture: the CPU layer below it is
 *              libwow64fex.dll (vendored FEX, contract-only), the kernel/
 *              loader layer above it is Phase C.
 *
 *              Phase A scope: exist, export the two symbols wow64fex imports,
 *              and make Wow64SystemServiceEx loudly report any syscall it is
 *              asked to thunk. The real 32→64 thunk table is Phase B.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>

ULONG
WINAPI
DbgPrint(_In_z_ _Printf_format_string_ PCSTR Format, ...);

LONG
WINAPI
NtTerminateProcess(_In_ HANDLE ProcessHandle, _In_ LONG ExitStatus);

LONG
WINAPI
NtQueryInformationProcess(_In_ HANDLE ProcessHandle,
                          _In_ ULONG ProcessInformationClass,
                          _Out_ PVOID ProcessInformation,
                          _In_ ULONG ProcessInformationLength,
                          _Out_opt_ PULONG ReturnLength);

/* ntdll-only DLL: no kernel32 import (a WoW64 process must not pull 64-bit
 * kernel32 through us — Win10 keeps it out of wow64 processes too). */
#define WOW64_CURRENT_PROCESS ((HANDLE)(LONG_PTR)-1)

/*
 * Phase-B probe services (wow64probe.exe / docs/engineering/wow64-plan.md).
 * Magic service numbers far above any real syscall id: ECHO proves 32-bit
 * guest-stack arguments marshal through the CPU layer and a 64-bit result
 * flows back into guest EAX; VERDICT receives that result back FROM the
 * guest, proving the full round-trip, then ends the guest process (the only
 * way out of BTCpuSimulate). Real Nt* thunks are Phase C/D.
 */
#define WOW64B_SVC_ECHO      0xF00Du
#define WOW64B_SVC_VERDICT   0xD00Eu
#define WOW64B_ECHO_MAGIC    0x5CA11EDu

/*
 * Wow64SystemServiceEx — the syscall thunk dispatcher. The CPU layer traps a
 * guest `syscall`/BOP, stores the guest context, and calls here with the
 * 32-bit service number and a pointer to the 32-bit argument array (guest
 * stack). Unknown services report and fail loud (STATUS_INVALID_SYSTEM_
 * SERVICE) so a premature guest syscall is visible on serial instead of
 * silently returning success.
 */
ULONG
WINAPI
Wow64SystemServiceEx(ULONG ServiceNumber, PULONG Arguments)
{
    switch (ServiceNumber)
    {
        case WOW64B_SVC_ECHO:
            DbgPrint("[FEX-WOW64B] ECHO service: args %p = %08lx %08lx %08lx\n",
                     Arguments, Arguments[0], Arguments[1], Arguments[2]);
            return WOW64B_ECHO_MAGIC;

        case WOW64B_SVC_VERDICT:
            if (Arguments[0] == WOW64B_ECHO_MAGIC)
            {
                DbgPrint("[FEX-WOW64B] guest syscall round-trip PASS (echo magic returned by guest)\n");
                NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
            }
            DbgPrint("[FEX-WOW64B] guest syscall round-trip FAIL (guest passed %08lx, want %08x)\n",
                     Arguments[0], WOW64B_ECHO_MAGIC);
            NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
            return 0; /* unreachable */

        default:
            DbgPrint("[WOW64] Wow64SystemServiceEx: service 0x%lx args %p — no thunk table yet (Phase B)\n",
                     ServiceNumber, Arguments);
            return (ULONG)0xC000001CL; /* STATUS_INVALID_SYSTEM_SERVICE */
    }
}

/*
 * Wow64LdrpInitialize — the thread handoff from the 64-bit ntdll loader.
 * For a WoW64 process, LdrpInitializeProcess (ntdll-rust ldrinit.rs) skips the
 * i386 image entirely, loads this DLL, and calls here with the initial thread
 * CONTEXT. This function owns the thread from here on and never returns: it
 * will bootstrap the 32-bit side (map syswow64 ntdll, init the CPU dll, build
 * the guest context, BTCpuSimulate).
 *
 * Phase C milestone 1 scope: prove the kernel+loader chain end-to-end —
 * EPROCESS marked WoW64 (PEB32 via ProcessWow64Information), TEB32 reachable
 * through the raw WowTebOffset slot (TEB+0x180c, kernel MmCreateTeb wrote it),
 * then exit 0x600D as the harness verdict. The CPU-layer bring-up (proven
 * standalone by wow64probe in Phase B) moves in here as milestone 2.
 */
VOID
WINAPI
Wow64LdrpInitialize(PVOID Context)
{
    ULONG_PTR Peb32 = 0;
    PBYTE Teb = (PBYTE)NtCurrentTeb();
    LONG WowTebOffset = *(LONG *)(Teb + 0x180c);
    PBYTE Teb32 = WowTebOffset ? Teb + WowTebOffset : NULL;
    LONG Status;

    DbgPrint("[FEX-WOW64C] Wow64LdrpInitialize: 64-bit loader handed the thread over (Context=%p)\n",
             Context);

    Status = NtQueryInformationProcess(WOW64_CURRENT_PROCESS,
                                       26 /* ProcessWow64Information */,
                                       &Peb32, sizeof(Peb32), NULL);
    DbgPrint("[FEX-WOW64C] PEB32=%p (query status %08lx), TEB32=%p (WowTebOffset=%ld)\n",
             (PVOID)Peb32, Status, Teb32, WowTebOffset);

    if (Status >= 0 && Peb32 && Teb32 &&
        *(ULONG *)(Teb32 + 0x30 /* TEB32.ProcessEnvironmentBlock */) == (ULONG)Peb32)
    {
        DbgPrint("[FEX-WOW64C] M1 PASS: WoW64 process bootstrap chain complete (image base %08lx)\n",
                 *(ULONG *)((PBYTE)Peb32 + 0x08 /* PEB32.ImageBaseAddress */));
        NtTerminateProcess(WOW64_CURRENT_PROCESS, 0x600D);
    }

    DbgPrint("[FEX-WOW64C] M1 FAIL: PEB32/TEB32 chain inconsistent\n");
    NtTerminateProcess(WOW64_CURRENT_PROCESS, 0xBAD);
}

/*
 * Wow64ProcessPendingCrossProcessItems — drains the cross-process work list
 * (writes into this process's guest memory by other processes, e.g.
 * CreateRemoteThread / WriteProcessMemory notifications). No cross-process
 * machinery exists yet; nothing to drain.
 */
VOID
WINAPI
Wow64ProcessPendingCrossProcessItems(VOID)
{
}

BOOL
WINAPI
DllMain(HINSTANCE hDll, DWORD dwReason, LPVOID lpReserved)
{
    UNREFERENCED_PARAMETER(lpReserved);

    UNREFERENCED_PARAMETER(hDll);

    if (dwReason == DLL_PROCESS_ATTACH)
    {
        /* No DisableThreadLibraryCalls: that's kernel32, and this DLL is
         * deliberately ntdll-only. Thread attach/detach no-op through DllMain. */
        DbgPrint("[WOW64] wow64.dll loaded\n");
    }
    return TRUE;
}
