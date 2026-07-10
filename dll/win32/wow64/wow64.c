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

/*
 * Wow64SystemServiceEx — the syscall thunk dispatcher. The CPU layer traps a
 * guest `syscall`/BOP, stores the guest context, and calls here with the
 * 32-bit service number and a pointer to the 32-bit argument array (guest
 * stack). Phase B populates a thunk table; until then, report and fail loud
 * (STATUS_INVALID_SYSTEM_SERVICE) so a premature guest syscall is visible on
 * serial instead of silently returning success.
 */
ULONG
WINAPI
Wow64SystemServiceEx(ULONG ServiceNumber, PULONG Arguments)
{
    DbgPrint("[WOW64] Wow64SystemServiceEx: service 0x%lx args %p — no thunk table yet (Phase B)\n",
             ServiceNumber, Arguments);
    return (ULONG)0xC000001CL; /* STATUS_INVALID_SYSTEM_SERVICE */
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

    if (dwReason == DLL_PROCESS_ATTACH)
    {
        DisableThreadLibraryCalls(hDll);
        DbgPrint("[WOW64] wow64.dll loaded (Phase A skeleton)\n");
    }
    return TRUE;
}
