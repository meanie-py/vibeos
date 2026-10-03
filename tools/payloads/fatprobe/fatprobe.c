/* fatprobe — the fastfat operations that bugchecked arm64 (0x23 from FatBugCheckExceptionFilter,
 * faulting in FsRtlCheckOplock) under the ucrtbase Tier-2 grader, without the grader:
 *   1. MoveFileExW of an existing file to a fresh name in the temp dir, then back;
 *   2. a run of create/close/delete, the shape of _mktemp's grader;
 *   3. the same with a second handle open while deleting and renaming (oplock break paths).
 * Each step is announced before it runs, so a bugcheck names the step that took the guest down.
 * Verdict: "[FATPROBE] done VERDICT=PASS|FAIL". */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[FATPROBE] " __VA_ARGS__)

static LONG failures;
#define CHECK(cond, ...) do { if (!(cond)) { SAY("FAIL " __VA_ARGS__); failures++; } } while (0)

static WCHAR dir[MAX_PATH];

static void path(WCHAR *out, const WCHAR *name, int n)
{
    wsprintfW(out, L"%sfatprobe-%s%d.tmp", dir, name, n);
}

static BOOL make(const WCHAR *p)
{
    HANDLE h = CreateFileW(p, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    DWORD w;
    if (h == INVALID_HANDLE_VALUE) return FALSE;
    WriteFile(h, "fatprobe", 8, &w, NULL);
    CloseHandle(h);
    return TRUE;
}

int main(void)
{
    WCHAR a[MAX_PATH], b[MAX_PATH];
    HANDLE h;
    int i;

    GetTempPathW(MAX_PATH, dir);
    SAY("start dir=%ls\n", dir);

    SAY("step rename\n");
    for (i = 0; i < 8; i++) {
        path(a, L"src", i);
        path(b, L"dst", i);
        CHECK(make(a), "create %ls err=%lu\n", a, GetLastError());
        CHECK(MoveFileExW(a, b, 0), "rename %d err=%lu\n", i, GetLastError());
        CHECK(MoveFileExW(b, a, MOVEFILE_REPLACE_EXISTING), "rename back %d err=%lu\n", i, GetLastError());
        CHECK(DeleteFileW(a), "delete %d err=%lu\n", i, GetLastError());
    }

    SAY("step create-delete\n");
    for (i = 0; i < 64; i++) {
        path(a, L"run", i);
        CHECK(make(a), "create run %d err=%lu\n", i, GetLastError());
    }
    for (i = 0; i < 64; i++) {
        path(a, L"run", i);
        CHECK(DeleteFileW(a), "delete run %d err=%lu\n", i, GetLastError());
    }

    SAY("step shared-handle\n");
    for (i = 0; i < 8; i++) {
        path(a, L"open", i);
        path(b, L"moved", i);
        CHECK(make(a), "create open %d err=%lu\n", i, GetLastError());
        h = CreateFileW(a, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                        NULL, OPEN_EXISTING, 0, NULL);
        CHECK(h != INVALID_HANDLE_VALUE, "open %d err=%lu\n", i, GetLastError());
        CHECK(MoveFileExW(a, b, 0), "rename open %d err=%lu\n", i, GetLastError());
        CHECK(DeleteFileW(b), "delete open %d err=%lu\n", i, GetLastError());
        if (h != INVALID_HANDLE_VALUE) CloseHandle(h);
    }

    SAY("done VERDICT=%s failures=%ld\n", failures ? "FAIL" : "PASS", failures);
    return failures != 0;
}
