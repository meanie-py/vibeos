/*
 * PROJECT:     ReactOS ARM64 Port
 * FILE:        base/system/winlogon/winlogon_arm64.c
 * PURPOSE:     Minimal winlogon stub for ARM64 (no Win32k/CSRSS dependency)
 *
 * DESCRIPTION: This is a placeholder winlogon that SMSS launches as the
 *              initial command. It simply waits forever via NtDelayExecution.
 *              Replace with real winlogon once Win32k session support is available.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>

#define NTOS_MODE_USER
#include <ndk/kefuncs.h>
#include <ndk/exfuncs.h>

/*
 * Force DLL imports — the ntdll loader loads these when the process starts.
 * user32 and gdi32 DllMain make Win32k SSDT syscalls (NtUserProcessConnect,
 * NtGdiInit), proving the GUI thread auto-conversion path works.
 * Without an actual symbol reference, lld dead-strips the import entry.
 */
__declspec(dllimport) int __stdcall GetSystemMetrics(int);
__declspec(dllimport) int __stdcall GetDeviceCaps(void *, int);
__declspec(dllimport) long __stdcall RegCloseKey(void *);

volatile void *_force_user32 = (void *)&GetSystemMetrics;
volatile void *_force_gdi32 = (void *)&GetDeviceCaps;
volatile void *_force_advapi32 = (void *)&RegCloseKey;

void NtProcessStartup(PVOID Peb)
{
    LARGE_INTEGER Interval;

    /* Marker: 3 consecutive NtYieldExecution calls to confirm this binary runs.
     * NtYieldExecution = SVC 0x52. Three in a row is distinctive in the trace. */
    NtYieldExecution();
    NtYieldExecution();
    NtYieldExecution();

    /* Sleep forever - SMSS waits on our process handle */
    Interval.QuadPart = -(10000LL * 1000 * 60 * 60 * 24); /* 1 day */

    for (;;)
    {
        NtDelayExecution(FALSE, &Interval);
    }
}
