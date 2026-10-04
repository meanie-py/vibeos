/* locking -- _locking's retry contract in ucrtbase.dll and msvcrt.dll, graded against the
 * documented behaviour rather than a reference build (Microsoft's locking.cpp is gone, so
 * ucrtdiff has nothing to compare _locking against):
 *   _LK_NBLCK / _LK_NBRLCK  one attempt; a held region fails with EACCES.
 *   _LK_LOCK  / _LK_RLCK    "tries again after 1 second ... after 10 attempts" fails with
 *                           EDEADLOCK -- so ~9 s of waiting, and a region released during
 *                           the wait is then taken.
 * The waits are why this is its own payload: ucrtdiff's per-export budget is 8 s.
 */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[LOCKING] " __VA_ARGS__)
static int g_fail, g_ok;
#define CHECK(c, ...) do { if (c) g_ok++; else { g_fail++; SAY("FAIL %s line %d: ", g_dll, __LINE__); DbgPrint(__VA_ARGS__); } } while (0)

#define O_RDWR 0x0002
#define O_CREAT 0x0100
#define O_BINARY 0x8000
#define S_IREAD 0x0100
#define S_IWRITE 0x0080
#define LK_UNLCK 0
#define LK_LOCK 1
#define LK_NBLCK 2
#define LK_RLCK 3
#define LK_NBRLCK 4
#define EACCES 13
#define EDEADLOCK 36

static const char *g_dll;
static int *(__cdecl *p_errno)(void);
static int (__cdecl *p_open)(const char *, int, ...);
static int (__cdecl *p_close)(int);
static int (__cdecl *p_write)(int, const void *, unsigned);
static long (__cdecl *p_lseek)(int, long, int);
static int (__cdecl *p_locking)(int, int, long);
static int (__cdecl *p_unlink)(const char *);

static int g_holder;

static DWORD WINAPI release_later(void *ms)
{
    Sleep((DWORD)(ULONG_PTR)ms);
    p_lseek(g_holder, 0, 0);
    p_locking(g_holder, LK_UNLCK, 4);
    return 0;
}

/* Run a held-region _LK_LOCK-style call and time it. */
static int timed_lock(int fd, int mode, DWORD *ms)
{
    DWORD t0 = GetTickCount();
    int r = p_locking(fd, mode, 4);
    *ms = GetTickCount() - t0;
    return r;
}

static void run(const char *dll, const char *path)
{
    HMODULE h = LoadLibraryA(dll);
    g_dll = dll;
#define G(v, n) (*(FARPROC *)&v = GetProcAddress(h, n))
    if (!h || !G(p_errno, "_errno") || !G(p_open, "_open") || !G(p_close, "_close") || !G(p_write, "_write")
        || !G(p_lseek, "_lseek") || !G(p_locking, "_locking") || !G(p_unlink, "_unlink")) {
        CHECK(0, "load\n");
        return;
    }
    int a = p_open(path, O_RDWR | O_CREAT | O_BINARY, S_IREAD | S_IWRITE);
    int b = p_open(path, O_RDWR | O_BINARY);
    CHECK(a >= 0 && b >= 0, "open a=%d b=%d errno=%d\n", a, b, *p_errno());
    if (a < 0 || b < 0) return;
    CHECK(p_write(a, "0123456789", 10) == 10, "write\n");
    p_lseek(a, 0, 0);
    p_lseek(b, 0, 0);
    g_holder = a;
    DWORD ms;

    SAY("%s: nonblocking\n", dll);
    CHECK(p_locking(a, LK_NBLCK, 4) == 0, "holder NBLCK errno=%d\n", *p_errno());
    *p_errno() = 0;
    CHECK(p_locking(b, LK_NBLCK, 4) == -1 && *p_errno() == EACCES, "contender NBLCK errno=%d\n", *p_errno());
    *p_errno() = 0;
    CHECK(timed_lock(b, LK_NBRLCK, &ms) == -1 && *p_errno() == EACCES && ms < 500,
          "contender NBRLCK errno=%d %lums\n", *p_errno(), ms);
    /* A region that does not overlap the held one is free. */
    p_lseek(b, 4, 0);
    CHECK(p_locking(b, LK_NBLCK, 4) == 0, "disjoint NBLCK errno=%d\n", *p_errno());
    CHECK(p_locking(b, LK_UNLCK, 4) == 0, "disjoint UNLCK errno=%d\n", *p_errno());
    p_lseek(b, 0, 0);

    SAY("%s: _LK_LOCK against a held region (ten tries, ~9 s)\n", dll);
    *p_errno() = 0;
    int r = timed_lock(b, LK_LOCK, &ms);
    CHECK(r == -1 && *p_errno() == EDEADLOCK, "LOCK held r=%d errno=%d\n", r, *p_errno());
    CHECK(ms >= 8500 && ms <= 13000, "LOCK held waited %lums, want ~9000\n", ms);

    SAY("%s: _LK_RLCK, holder releases after 2.5 s\n", dll);
    HANDLE t = CreateThread(NULL, 0, release_later, (void *)(ULONG_PTR)2500, 0, NULL);
    r = timed_lock(b, LK_RLCK, &ms);
    WaitForSingleObject(t, INFINITE);
    CloseHandle(t);
    CHECK(r == 0, "RLCK after release r=%d errno=%d\n", r, *p_errno());
    CHECK(ms >= 2400 && ms <= 5000, "RLCK after release took %lums, want ~3000\n", ms);
    /* b holds it now: a cannot. */
    p_lseek(a, 0, 0);
    *p_errno() = 0;
    CHECK(p_locking(a, LK_NBLCK, 4) == -1 && *p_errno() == EACCES, "reverse NBLCK errno=%d\n", *p_errno());
    p_lseek(b, 0, 0);
    CHECK(p_locking(b, LK_UNLCK, 4) == 0, "UNLCK errno=%d\n", *p_errno());
    p_lseek(a, 0, 0);
    CHECK(timed_lock(a, LK_LOCK, &ms) == 0 && ms < 500, "free LOCK took %lums\n", ms);
    CHECK(p_locking(a, LK_UNLCK, 4) == 0, "free UNLCK\n");

    p_close(b);
    p_close(a);
    p_unlink(path);
}

int WINAPI WinMain(HINSTANCE x, HINSTANCE y, LPSTR z, int w)
{
    (void)x; (void)y; (void)z; (void)w;
    /* x86 boots have %TEMP% on the read-only live CD; their scratch FAT disk carries an
     * empty \TEMP (tests/qemu/vm.py make_temp_disk). arm64's ramdisk %TEMP% is fastfat. */
    char tmp[MAX_PATH] = "";
    static const char *const scratch[] = { "C:\\TEMP\\", "D:\\TEMP\\", "E:\\TEMP\\" };
    for (int i = 0; i < 3 && !tmp[0]; i++)
        if (GetFileAttributesA(scratch[i]) != INVALID_FILE_ATTRIBUTES) lstrcpyA(tmp, scratch[i]);
    if (!tmp[0]) GetTempPathA(MAX_PATH - 32, tmp);
    char p1[MAX_PATH], p2[MAX_PATH];
    wsprintfA(p1, "%slk_u%lu.tmp", tmp, GetTickCount());
    wsprintfA(p2, "%slk_m%lu.tmp", tmp, GetTickCount());
    SAY("start %s\n", p1);
    run("ucrtbase.dll", p1);
    run("msvcrt.dll", p2);
    SAY("%d checks, %d failed\n", g_ok + g_fail, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
