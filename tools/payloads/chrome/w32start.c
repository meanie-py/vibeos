/* w32start: which part of an llvm-mingw i686 image kills it in WOW64 start-up?
 * gdiprobe's i386 build died before main (64-bit ntdll, activation context: add_entity ->
 * RtlReAllocateHeap raising STATUS_NO_MEMORY for 0x2d0 bytes, exit 128) while the stock i386
 * apps, built by the ReactOS toolchain, start. Built three ways (build.sh): W32START_BARE has no
 * CRT and imports only kernel32 like the guests that work; the default uses llvm-mingw's CRT
 * (msvcrt, a .tls section); W32START_USER32 adds a user32 import, W32START_GDI32 gdi32 too.
 * Each that starts prints one line. All four started in every boot since they were added (and
 * the GUI-subsystem gdiprobe32 with them), so they stay as a start-up check for that failure,
 * which is not explained yet (registry: "llvm-mingw i686 console exe dies in WOW64 start-up"). */
#include <windows.h>
#include <stdio.h>

#ifdef W32START_BARE
void __stdcall entry(void)
{
    static const char msg[] = "[W32START] bare: started\n";
    DWORD n;
    WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), msg, sizeof msg - 1, &n, NULL);
    ExitProcess(0);
}
#else
int main(void)
{
#ifdef W32START_GDI32
    printf("[W32START] crt+user32+gdi32: started (desktop %s, stock %s)\n", GetDesktopWindow() ? "ok" : "none",
           GetStockObject(BLACK_BRUSH) ? "ok" : "none");
#elif defined(W32START_USER32)
    printf("[W32START] crt+user32: started (desktop %s)\n", GetDesktopWindow() ? "ok" : "none");
#else
    printf("[W32START] crt: started\n");
#endif
    fflush(stdout);
    return 0;
}
#endif
