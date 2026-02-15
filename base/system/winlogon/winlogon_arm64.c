/*
 * PROJECT:     ReactOS ARM64 Port
 * FILE:        base/system/winlogon/winlogon_arm64.c
 * PURPOSE:     Minimal winlogon stub for ARM64 — tests CreateProcessW
 *
 * DESCRIPTION: SMSS launches this as the initial command. It calls
 *              CreateProcessW to spawn testchild.exe, waits for it,
 *              then sleeps forever.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>

#define NTOS_MODE_USER
#include <ndk/kefuncs.h>
#include <ndk/exfuncs.h>
#include <ndk/obfuncs.h>

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
    STARTUPINFOW si;
    PROCESS_INFORMATION pi;
    LARGE_INTEGER Interval;
    BOOL ok;

    /* Zero init */
    for (int i = 0; i < sizeof(si); i++) ((char*)&si)[i] = 0;
    for (int i = 0; i < sizeof(pi); i++) ((char*)&pi)[i] = 0;
    si.cb = sizeof(si);

    /* Marker: 3 consecutive NtYieldExecution calls to confirm this binary runs.
     * NtYieldExecution = SVC 0x52. Three in a row is distinctive in the trace. */
    NtYieldExecution();
    NtYieldExecution();
    NtYieldExecution();

    /* Spawn testchild.exe via full Win32 CreateProcessW path.
     * kernel32 SearchPathW will find it in BaseWindowsSystemDirectory. */
    ok = CreateProcessW(L"testchild.exe",
                        NULL,
                        NULL,
                        NULL,
                        FALSE,
                        DETACHED_PROCESS,
                        NULL,
                        NULL,
                        &si,
                        &pi);

    if (ok)
    {
        /* Wait for child to exit */
        NtWaitForSingleObject(pi.hProcess, FALSE, NULL);
        NtClose(pi.hProcess);
        NtClose(pi.hThread);
    }

    /* Sleep forever — SMSS waits on our process handle */
    Interval.QuadPart = -(10000LL * 1000 * 60 * 60 * 24); /* 1 day */

    for (;;)
    {
        NtDelayExecution(FALSE, &Interval);
    }
}
