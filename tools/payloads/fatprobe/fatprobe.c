/* fatprobe — the fastfat operations that bugchecked arm64 (0x23 from FatBugCheckExceptionFilter,
 * faulting in FsRtlCheckOplock) under the ucrtbase Tier-2 grader, without the grader:
 *   1. MoveFileExW of an existing file to a fresh name in the temp dir, then back;
 *   2. a run of create/close/delete, the shape of _mktemp's grader;
 *   3. the same with a second handle open while deleting and renaming (oplock break paths);
 *   4. listing a directory of 60 long-named files, through FindFirstFileW and through one
 *      16 KB NtQueryDirectoryFile, each name exactly once (arm64's Fonts directory came back
 *      with duplicates, 8.3 names and most files missing, so Marlett never loaded);
 *   5. every name listed in %windir% (and system32) opens by its path, so a lookup and a listing
 *      of the same directory agree (appstorm's SearchPath could not find X:\reactos\iexplore.exe).
 * Each step is announced before it runs, so a bugcheck names the step that took the guest down.
 * Verdict: "[FATPROBE] done VERDICT=PASS|FAIL". */
#include <windows.h>
#include <winternl.h>
#include <string.h>

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

#define ENUM_FILES 60

typedef struct {
    ULONG NextEntryOffset, FileIndex;
    LARGE_INTEGER CreationTime, LastAccessTime, LastWriteTime, ChangeTime, EndOfFile, AllocationSize;
    ULONG FileAttributes, FileNameLength;
    WCHAR FileName[1];
} DIRINFO;

NTSTATUS NTAPI NtQueryDirectoryFile(HANDLE, HANDLE, PVOID, PVOID, PIO_STATUS_BLOCK, PVOID, ULONG,
                                    ULONG, BOOLEAN, PUNICODE_STRING, BOOLEAN);

/* seen[i] counts how often enum-<i> was listed; anything else that is not . or .. is wrong. */
static int count_name(const WCHAR *name, int len, int *seen)
{
    int n;
    if ((len == 1 && name[0] == L'.') || (len == 2 && name[0] == L'.' && name[1] == L'.')) return 1;
    if (len > 26 && !wcsncmp(name, L"fatprobe-enum-long-name-", 24)) {
        n = (name[24] - L'0') * 10 + (name[25] - L'0');
        if (n >= 0 && n < ENUM_FILES) { seen[n]++; return 1; }
    }
    SAY("FAIL enum: unexpected name %.*ls\n", len, name);
    failures++;
    return 0;
}

static void check_seen(const char *how, const int *seen)
{
    int i;
    for (i = 0; i < ENUM_FILES; i++)
        CHECK(seen[i] == 1, "enum %s: fatprobe-enum-long-name-%02d listed %d times\n", how, i, seen[i]);
}

static void step_enum(void)
{
    WCHAR sub[MAX_PATH], p[MAX_PATH], pat[MAX_PATH];
    WIN32_FIND_DATAW fd;
    int seen[ENUM_FILES] = {0}, i, records = 0;
    HANDLE h;
    static BYTE buf[16384];
    IO_STATUS_BLOCK iosb;
    NTSTATUS st;
    BOOLEAN restart = TRUE;

    SAY("step enumerate\n");
    wsprintfW(sub, L"%sfatprobe-enum", dir);
    CreateDirectoryW(sub, NULL);
    for (i = 0; i < ENUM_FILES; i++) {
        wsprintfW(p, L"%s\\fatprobe-enum-long-name-%02d.data", sub, i);
        CHECK(make(p), "create enum %d err=%lu\n", i, GetLastError());
    }

    wsprintfW(pat, L"%s\\*", sub);
    h = FindFirstFileW(pat, &fd);
    CHECK(h != INVALID_HANDLE_VALUE, "FindFirstFileW err=%lu\n", GetLastError());
    if (h != INVALID_HANDLE_VALUE) {
        do count_name(fd.cFileName, lstrlenW(fd.cFileName), seen); while (FindNextFileW(h, &fd));
        FindClose(h);
    }
    check_seen("FindFirstFileW", seen);

    for (i = 0; i < ENUM_FILES; i++) seen[i] = 0;
    h = CreateFileW(sub, FILE_LIST_DIRECTORY | SYNCHRONIZE, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    NULL, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, NULL);
    CHECK(h != INVALID_HANDLE_VALUE, "open dir err=%lu\n", GetLastError());
    while (h != INVALID_HANDLE_VALUE) {
        DIRINFO *d = (DIRINFO *)buf;
        int batch = 0;
        st = NtQueryDirectoryFile(h, NULL, NULL, NULL, &iosb, buf, sizeof(buf), 1 /* FileDirectoryInformation */,
                                  FALSE, NULL, restart);
        restart = FALSE;
        if (st < 0 || st == 0x80000006 /* STATUS_NO_MORE_FILES */) break;
        for (;;) {
            batch++;
            count_name(d->FileName, d->FileNameLength / 2, seen);
            if (!d->NextEntryOffset) break;
            d = (DIRINFO *)((BYTE *)d + d->NextEntryOffset);
        }
        SAY("enum batch of %d (Information=%lu)\n", batch, (ULONG)iosb.Information);
        records += batch;
        if (records > 4 * ENUM_FILES) { SAY("FAIL enum: no end in sight\n"); failures++; break; }
    }
    if (h != INVALID_HANDLE_VALUE) CloseHandle(h);
    check_seen("NtQueryDirectoryFile", seen);

    for (i = 0; i < ENUM_FILES; i++) {
        wsprintfW(p, L"%s\\fatprobe-enum-long-name-%02d.data", sub, i);
        DeleteFileW(p);
    }
    RemoveDirectoryW(sub);
}

static void lookup_dir(const WCHAR *d)
{
    WCHAR pat[MAX_PATH], p[MAX_PATH];
    WIN32_FIND_DATAW fd;
    HANDLE h;
    int listed = 0, bad = 0;

    wsprintfW(pat, L"%s\\*", d);
    h = FindFirstFileW(pat, &fd);
    CHECK(h != INVALID_HANDLE_VALUE, "lookup: list %ls err=%lu\n", d, GetLastError());
    if (h == INVALID_HANDLE_VALUE) return;
    do {
        if (!lstrcmpW(fd.cFileName, L".") || !lstrcmpW(fd.cFileName, L"..")) continue;
        listed++;
        wsprintfW(p, L"%s\\%s", d, fd.cFileName);
        if (GetFileAttributesW(p) == INVALID_FILE_ATTRIBUTES) {
            SAY("FAIL lookup: %ls is listed but does not open (err=%lu, 8.3=%ls)\n", p, GetLastError(),
                fd.cAlternateFileName);
            failures++;
            bad++;
        }
    } while (FindNextFileW(h, &fd));
    FindClose(h);
    SAY("lookup %ls: %d listed, %d do not open\n", d, listed, bad);
}

static void step_lookup(void)
{
    WCHAR w[MAX_PATH], s32[MAX_PATH];
    SAY("step lookup\n");
    GetWindowsDirectoryW(w, MAX_PATH);
    GetSystemDirectoryW(s32, MAX_PATH);
    lookup_dir(w);
    lookup_dir(s32);
    {
        /* appstorm's SearchPathA(NULL, "iexplore.exe") failed while the file opened by path. */
        static const char *const names[] = { "iexplore.exe", "regedit.exe", "notepad.exe", "hh.exe" };
        char found[MAX_PATH];
        int k;
        {
            static char env[2048];
            char probe[MAX_PATH], me[MAX_PATH], *slash;
            const char *dirs[3];
            DWORD a;
            GetModuleFileNameA(NULL, me, MAX_PATH);
            if ((slash = strrchr(me, '\\'))) *slash = 0;
            dirs[0] = me;
            GetSystemDirectoryA(probe, MAX_PATH);
            dirs[1] = _strdup(probe);
            GetWindowsDirectoryA(probe, MAX_PATH);
            dirs[2] = _strdup(probe);
            for (k = 0; k < 3; k++) {
                wsprintfA(probe, "%s\\iexplore.exe", dirs[k]);
                SetLastError(0);
                a = GetFileAttributesA(probe);
                SAY("probe %s -> attrs=%lx err=%lu\n", probe, a, GetLastError());
            }
            GetEnvironmentVariableA("PATH", env, sizeof env);
            SAY("PATH=%s\n", env);
        }
        for (k = 0; k < 4; k++) {
            DWORD n = SearchPathA(NULL, names[k], NULL, MAX_PATH, found, NULL);
            CHECK(n != 0, "SearchPathA %s err=%lu\n", names[k], GetLastError());
            if (n) SAY("SearchPathA %s -> %s\n", names[k], found);
        }
    }

    /* Grow and shrink %windir% itself with long names, then look again: a write that
     * clobbered a neighbouring dirent shows up as a listed name that no longer opens. */
    {
        WCHAR p[MAX_PATH];
        int i;
        SAY("step lookup-after-writes\n");
        for (i = 0; i < 40; i++) {
            wsprintfW(p, L"%s\\fatprobe-windir-long-name-%02d.data", w, i);
            CHECK(make(p), "create windir %d err=%lu\n", i, GetLastError());
        }
        lookup_dir(w);
        for (i = 0; i < 40; i += 2) {
            wsprintfW(p, L"%s\\fatprobe-windir-long-name-%02d.data", w, i);
            CHECK(DeleteFileW(p), "delete windir %d err=%lu\n", i, GetLastError());
        }
        lookup_dir(w);
        for (i = 1; i < 40; i += 2) {
            wsprintfW(p, L"%s\\fatprobe-windir-long-name-%02d.data", w, i);
            CHECK(DeleteFileW(p), "delete windir %d err=%lu\n", i, GetLastError());
        }
        lookup_dir(w);
    }
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

    step_enum();
    step_lookup();

    SAY("done VERDICT=%s failures=%ld\n", failures ? "FAIL" : "PASS", failures);
    return failures != 0;
}
