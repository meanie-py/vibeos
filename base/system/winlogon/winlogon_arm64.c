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

void NtProcessStartup(PVOID Peb)
{
    LARGE_INTEGER Interval;

    /* Sleep forever - SMSS waits on our process handle */
    Interval.QuadPart = -(10000LL * 1000 * 60 * 60 * 24); /* 1 day */

    for (;;)
    {
        NtDelayExecution(FALSE, &Interval);
    }
}
