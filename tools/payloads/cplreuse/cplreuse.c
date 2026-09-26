/* cplreuse — does a DLL load hang when it reuses the VA another DLL just vacated?
 *
 * Every stock .cpl links at 0x10000000, so Control Panel's enumeration loads one, frees it and
 * maps the next at the same address. On i386 the second load (appwiz after access) parked for
 * good on its first .data access. This replays that sequence without the shell: load and free
 * each applet in turn, and a watchdog names the load that stops making progress.
 * Verdict: "[CPLREUSE] done VERDICT=PASS|FAIL". */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[CPLREUSE] " __VA_ARGS__)
#include "../stackdump.h"

static DWORD main_tid;

static const char *const cpls[] = {
    "access.cpl", "appwiz.cpl", "desk.cpl", "intl.cpl", "main.cpl", "mmsys.cpl",
    "ncpa.cpl", "powercfg.cpl", "sysdm.cpl", "timedate.cpl", "access.cpl", "appwiz.cpl",
};
static volatile LONG current = -1, finished;

static DWORD WINAPI watchdog(LPVOID unused)
{
    LONG last = -2, same = 0;
    (void)unused;
    while (!finished) {
        Sleep(5000);
        LONG now = current;
        if (now == last && now >= 0 && ++same == 4) {
            SAY("STUCK loading %s (#%ld) for 20s\n", cpls[now], now);
            dump_thread(GetCurrentProcessId(), main_tid);
            SAY("done VERDICT=FAIL\n");
            ExitProcess(1);
        }
        if (now != last) same = 0;
        last = now;
    }
    return 0;
}

int main(void)
{
    main_tid = GetCurrentThreadId();
    CreateThread(NULL, 0, watchdog, NULL, 0, NULL);
    for (LONG i = 0; i < (LONG)(sizeof cpls / sizeof cpls[0]); i++) {
        current = i;
        HMODULE h = LoadLibraryA(cpls[i]);
        SAY("load %s -> %p (err %lu)\n", cpls[i], (void *)h, h ? 0 : GetLastError());
        if (h) {
            FreeLibrary(h);
            SAY("freed %s\n", cpls[i]);
        }
    }
    finished = 1;
    SAY("done VERDICT=PASS\n");
    return 0;
}
