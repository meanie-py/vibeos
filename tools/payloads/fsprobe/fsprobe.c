/* fsprobe — the fastfat file operations that bugchecked the guest (0x23, an access violation
 * at FsRtlCheckOplock+0x2c, found by the ucrtdiff grader): renames of each flavour, and a run
 * of creates and deletes like _mktemp's grader. Every step announces itself BEFORE it runs, so
 * a bugcheck's last serial line names the operation. Verdict: "[FSPROBE] done VERDICT=PASS|FAIL". */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[FSPROBE] " __VA_ARGS__)

static LONG failures;
static WCHAR dir[MAX_PATH];

#define CHECK(cond, ...) do { if (!(cond)) { SAY("FAIL " __VA_ARGS__); failures++; } } while (0)

static const WCHAR *P(const WCHAR *leaf)
{
    static WCHAR buf[4][MAX_PATH];
    static int i;
    WCHAR *b = buf[i++ & 3];
    lstrcpyW(b, dir);
    lstrcatW(b, leaf);
    return b;
}

static BOOL make(const WCHAR *path)
{
    HANDLE h = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE) return FALSE;
    DWORD n;
    WriteFile(h, "fsprobe\n", 8, &n, NULL);
    CloseHandle(h);
    return TRUE;
}

static BOOL exists(const WCHAR *path) { return GetFileAttributesW(path) != INVALID_FILE_ATTRIBUTES; }

int WINAPI WinMain(HINSTANCE a, HINSTANCE b, LPSTR c, int d)
{
    (void)a; (void)b; (void)c; (void)d;
    /* x86 boots have %TEMP% on the read-only live CD; their scratch FAT disk carries an empty
     * \TEMP (tests/qemu/vm.py make_temp_disk). arm64's ramdisk %TEMP% is itself fastfat. */
    WCHAR tmp[MAX_PATH] = L"";
    static const WCHAR *const scratch[] = { L"C:\\TEMP\\", L"D:\\TEMP\\", L"E:\\TEMP\\" };
    for (int i = 0; i < 3 && !tmp[0]; i++)
        if (GetFileAttributesW(scratch[i]) != INVALID_FILE_ATTRIBUTES) lstrcpyW(tmp, scratch[i]);
    if (!tmp[0]) GetTempPathW(MAX_PATH - 32, tmp);
    wsprintfW(dir, L"%sfsprobe%lu\\", tmp, GetTickCount());
    SAY("start dir=%ls\n", dir);
    CHECK(CreateDirectoryW(dir, NULL), "mkdir gle=%lu\n", GetLastError());

    SAY("step create-delete run\n");
    for (int i = 0; i < 26; i++) {
        WCHAR leaf[32];
        wsprintfW(leaf, L"mk%c.tmp", L'a' + i);
        CHECK(make(P(leaf)), "create %ls gle=%lu\n", leaf, GetLastError());
        CHECK(DeleteFileW(P(leaf)), "delete %ls gle=%lu\n", leaf, GetLastError());
    }

    SAY("step rename fresh\n");
    CHECK(make(P(L"a.txt")), "create a gle=%lu\n", GetLastError());
    CHECK(MoveFileExW(P(L"a.txt"), P(L"b.txt"), 0), "rename fresh gle=%lu\n", GetLastError());
    CHECK(!exists(P(L"a.txt")) && exists(P(L"b.txt")), "rename fresh: wrong names after\n");

    SAY("step rename over existing (no replace)\n");
    CHECK(make(P(L"c.txt")), "create c gle=%lu\n", GetLastError());
    CHECK(!MoveFileExW(P(L"b.txt"), P(L"c.txt"), 0) && GetLastError() == ERROR_ALREADY_EXISTS,
          "rename noreplace gle=%lu\n", GetLastError());

    SAY("step rename over existing (replace)\n");
    CHECK(MoveFileExW(P(L"b.txt"), P(L"c.txt"), MOVEFILE_REPLACE_EXISTING),
          "rename replace gle=%lu\n", GetLastError());
    CHECK(!exists(P(L"b.txt")) && exists(P(L"c.txt")), "rename replace: wrong names after\n");

    SAY("step rename onto a just-deleted name\n");
    CHECK(make(P(L"d.txt")), "create d gle=%lu\n", GetLastError());
    CHECK(DeleteFileW(P(L"d.txt")), "delete d gle=%lu\n", GetLastError());
    CHECK(MoveFileExW(P(L"c.txt"), P(L"d.txt"), 0), "rename tunnel gle=%lu\n", GetLastError());

    SAY("step rename long name\n");
    CHECK(MoveFileExW(P(L"d.txt"), P(L"a much longer file name.text"), 0),
          "rename lfn gle=%lu\n", GetLastError());

    /* ucrtdiff's _mktemp grader: 26 letters x 2 sides of 9-character names, all live at
     * once, so every one needs a generated 8.3 alias and the directory grows. */
    SAY("step occupy run (52 live long names)\n");
    for (int pass = 0; pass < 2; pass++) {
        for (int i = 0; i < 26; i++) {
            for (int side = 0; side < 2; side++) {
                WCHAR leaf[32];
                wsprintfW(leaf, L"mkg%c%05d", L'a' + i, 4242 + side);
                if (!pass) CHECK(make(P(leaf)), "occupy %ls gle=%lu\n", leaf, GetLastError());
                else CHECK(DeleteFileW(P(leaf)), "release %ls gle=%lu\n", leaf, GetLastError());
            }
        }
        if (!pass) SAY("step occupy run: all 52 created\n");
    }

    SAY("step cleanup\n");
    DeleteFileW(P(L"a much longer file name.text"));
    CHECK(RemoveDirectoryW(dir), "rmdir gle=%lu\n", GetLastError());

    SAY("done VERDICT=%s failures=%ld\n", failures ? "FAIL" : "PASS", failures);
    return failures != 0;
}
