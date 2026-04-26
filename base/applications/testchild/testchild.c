/*
 * testchild.c - Minimal Win32 CUI process for ARM64 CreateProcess testing
 *
 * This binary is spawned by winlogon_arm64.c to verify the full
 * CreateProcessW code path works on ARM64.
 */

#include <windows.h>

int main(void)
{
    /* 5x Sleep(0) = distinctive NtDelayExecution pattern in SVC trace */
    Sleep(0);
    Sleep(0);
    Sleep(0);
    Sleep(0);
    Sleep(0);

    /* Stay alive briefly so winlogon can observe us */
    Sleep(3000);

    ExitProcess(42);
    return 0;
}
