/*
 * PROJECT:     vibeos kernelbase.dll (minimal, Vista-scoped)
 * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
 * PURPOSE:     DLL entry point + WaitOnAddress (the one export that needs local
 *              code rather than a pure forwarder).
 *
 * See kernelbase.spec for the export surface and rationale. WaitOnAddress is a
 * thin DWORD-milliseconds -> LARGE_INTEGER shim over ntdll!RtlWaitOnAddress
 * (sdk/lib/rust/rtl-sync/src/waitaddr.rs); WakeByAddressSingle/All forward
 * straight to ntdll!RtlWakeAddress{Single,All} in the spec.
 */

#include <stdarg.h>
#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>
#include <ndk/rtlfuncs.h>

/* Not yet in the psdk rtl headers — declared here (implemented in ntdll, Rust). */
NTSYSAPI
NTSTATUS
NTAPI
RtlWaitOnAddress(
    _In_ volatile VOID *Address,
    _In_ PVOID CompareAddress,
    _In_ SIZE_T AddressSize,
    _In_opt_ const LARGE_INTEGER *Timeout);

BOOL
WINAPI
DllMain(
    _In_ HINSTANCE hDll,
    _In_ DWORD dwReason,
    _In_ LPVOID lpReserved)
{
    UNREFERENCED_PARAMETER(lpReserved);

    if (dwReason == DLL_PROCESS_ATTACH)
    {
        /* Pure synchronization shim — no per-thread work. */
        DisableThreadLibraryCalls(hDll);
    }
    return TRUE;
}

BOOL
WINAPI
WaitOnAddress(
    _In_ volatile VOID *Address,
    _In_ PVOID CompareAddress,
    _In_ SIZE_T AddressSize,
    _In_ DWORD dwMilliseconds)
{
    LARGE_INTEGER Timeout;
    PLARGE_INTEGER pTimeout = NULL;
    NTSTATUS Status;

    if (dwMilliseconds != INFINITE)
    {
        /* Relative timeout: negative 100ns units. */
        Timeout.QuadPart = (LONGLONG)dwMilliseconds * -10000LL;
        pTimeout = &Timeout;
    }

    Status = RtlWaitOnAddress(Address, CompareAddress, AddressSize, pTimeout);
    if (!NT_SUCCESS(Status) || Status == STATUS_TIMEOUT)
    {
        SetLastError(RtlNtStatusToDosError(Status));
        return FALSE;
    }
    return TRUE;
}
