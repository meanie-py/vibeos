/* wow64_pfnmap.c: host user32 client procs -> the guest's. Header-light on
 * purpose, so callers need no win32k headers. */
#pragma once

VOID Wow64pPfnMapSetPsi(ULONGLONG Psi64);
VOID Wow64pPfnMapSetGuest(ULONG PfnA32, ULONG PfnW32);
ULONG Wow64pMapClientProc(ULONGLONG Proc64);
ULONGLONG Wow64pUnmapClientProc(ULONG Proc32);
