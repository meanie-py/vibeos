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

    SAY("done VERDICT=%s fails=%d\n", fails ? "FAIL" : "PASS", fails);
    return fails != 0;
}
