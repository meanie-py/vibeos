/* rpmprobe — ReadProcessMemory/WriteProcessMemory over a range that runs off the end of the
 * committed memory must FAIL with a status, not bugcheck the machine.
 *
 * NtReadVirtualMemory -> MmCopyVirtualMemory -> MiDoMappedCopy locks the source with
 * MmProbeAndLockPages, which raises by design on a page it cannot lock. The C caught that in
 * _SEH2_TRY; the translation had no handler, so appstorm's stack dumper (8KB read from a thread's
 * SP, past the end of its stack) bugchecked amd64 with 0x1E. Large copies take the MDL path,
 * small ones the pool path, so both sizes are tried. Verdict: "[RPMPROBE] done VERDICT=PASS|FAIL". */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[RPMPROBE] " __VA_ARGS__)

static int fails;

static void check(const char *what, BOOL ok, DWORD err, BOOL want_ok)
{
    int good = want_ok ? ok : (!ok && err != 0);
    SAY("%s %s ok=%d err=%lu\n", good ? "PASS" : "FAIL", what, ok, ok ? 0 : err);
    fails += !good;
}

/* Race phase: the probe-then-copy strategy is only safe on MP if the COPY is fault-protected,
 * because another CPU can decommit the page between the range check and the copy. Here one
 * thread per CPU copies across a page boundary while another commits/decommits the page under it. */
static BYTE *race_base;
static SIZE_T race_page;
static volatile LONG race_stop, race_ok, race_err, race_bad;

static DWORD WINAPI race_copier(LPVOID arg)
{
    static BYTE tmp[8 * 4096];
    int write = (int)(ULONG_PTR)arg & 1;
    while (!race_stop) {
        SIZE_T got = 0;
        BOOL ok = write ? WriteProcessMemory(GetCurrentProcess(), race_base + race_page / 2, tmp, race_page, &got)
                        : ReadProcessMemory(GetCurrentProcess(), race_base + race_page / 2, tmp, race_page, &got);
        if (ok) InterlockedIncrement(&race_ok);
        else if (GetLastError()) InterlockedIncrement(&race_err);
        else InterlockedIncrement(&race_bad); /* failed without saying why */
    }
    return 0;
}

static DWORD WINAPI race_toggler(LPVOID unused)
{
    (void)unused;
    while (!race_stop) {
        VirtualAlloc(race_base + race_page, race_page, MEM_COMMIT, PAGE_READWRITE);
        VirtualFree(race_base + race_page, race_page, MEM_DECOMMIT);
    }
    return 0;
}

static void race_phase(SIZE_T page, DWORD cpus)
{
    HANDLE th[64];
    int n = 0;
    race_page = page;
    race_base = VirtualAlloc(NULL, 2 * page, MEM_RESERVE, PAGE_READWRITE);
    VirtualAlloc(race_base, page, MEM_COMMIT, PAGE_READWRITE);
    for (DWORD i = 0; i < cpus && n < 60; i++) {
        th[n++] = CreateThread(NULL, 0, race_copier, (LPVOID)(ULONG_PTR)i, 0, NULL);
        th[n++] = CreateThread(NULL, 0, race_toggler, NULL, 0, NULL);
    }
    Sleep(15000);
    race_stop = 1;
    WaitForMultipleObjects(n, th, TRUE, 30000);
    SAY("%s race cpus=%lu copies ok=%ld err=%ld silent=%ld\n",
        race_bad == 0 && race_ok + race_err > 0 ? "PASS" : "FAIL", cpus, race_ok, race_err, race_bad);
    fails += !(race_bad == 0 && race_ok + race_err > 0);
}

int main(void)
{
    SYSTEM_INFO si;
    GetSystemInfo(&si);
    SIZE_T page = si.dwPageSize;
    /* Three pages reserved, only the first committed: page 2 and 3 are not accessible. */
    BYTE *base = VirtualAlloc(NULL, 3 * page, MEM_RESERVE, PAGE_READWRITE);
    VirtualAlloc(base, page, MEM_COMMIT, PAGE_READWRITE);
    memset(base, 0x5a, page);
    static BYTE buf[3 * 4096 * 4];
    SIZE_T got = 0;
    HANDLE self = GetCurrentProcess();

    BOOL ok = ReadProcessMemory(self, base, buf, page, &got);
    check("read committed page", ok, GetLastError(), TRUE);
    for (SIZE_T sz = 64; sz <= 2 * page; sz *= 4) {
        char what[80];
        ok = ReadProcessMemory(self, base + page - sz / 2, buf, sz, &got);
        wsprintfA(what, "read %lu bytes across the commit boundary", (unsigned long)sz);
        check(what, ok, GetLastError(), FALSE);
        ok = WriteProcessMemory(self, base + page - sz / 2, buf, sz, &got);
        wsprintfA(what, "write %lu bytes across the commit boundary", (unsigned long)sz);
        check(what, ok, GetLastError(), FALSE);
    }
    ok = ReadProcessMemory(self, base + page, buf, 2 * page, &got);
    check("read entirely uncommitted range", ok, GetLastError(), FALSE);

    race_phase(page, si.dwNumberOfProcessors);
    SAY("done VERDICT=%s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
