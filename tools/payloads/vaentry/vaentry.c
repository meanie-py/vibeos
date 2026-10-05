/* vaentry — the CRT's C `...` exports, now varargs-rust va_entry! thunks, called the way a
 * C program calls them, on all three arches.
 *
 * The thunk spills the register arguments so the whole list is contiguous, and the Rust
 * body reads its fixed arguments from slots and hands the rest on as the va_list. What can
 * go wrong is therefore where the list crosses from registers to the stack: amd64 passes
 * four in rcx/rdx/r8/r9 (doubles too, for variadics), arm64 eight in x0-x7. The checks put
 * ints, doubles, 64-bit ints and strings on both sides of those boundaries, use exports
 * with up to five FIXED arguments (one of which is then on the stack on amd64), write back
 * through scanf's pointers, read _sopen's optional pmode (proven by the read-only
 * attribute it produces) and walk _spawnl's argument list into a real child process.
 * Every export is resolved from the named DLL, so msvcrt's and ucrtbase's are each tested.
 */
#include <windows.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[VAENTRY] " __VA_ARGS__)

static int g_fail, g_checks;
static void check(const char *dll, const char *what, int ok)
{
    g_checks++;
    if (!ok) {
        g_fail++;
        SAY("%s %s FAIL\n", dll, what);
    }
}

typedef int(__cdecl *sprintf_t)(char *, const char *, ...);
typedef int(__cdecl *snprintf_t)(char *, size_t, const char *, ...);
typedef int(__cdecl *snprintf_s_l_t)(char *, size_t, size_t, const char *, void *, ...);
typedef int(__cdecl *snwprintf_s_l_t)(wchar_t *, size_t, size_t, const wchar_t *, void *, ...);
typedef int(__cdecl *swprintf_s_t)(wchar_t *, size_t, const wchar_t *, ...);
typedef int(__cdecl *scprintf_t)(const char *, ...);
typedef int(__cdecl *sscanf_t)(const char *, const char *, ...);
typedef int(__cdecl *snscanf_t)(const char *, size_t, const char *, ...);
typedef int(__cdecl *sopen_t)(const char *, int, int, ...);
typedef int(__cdecl *open_t)(const char *, int, ...);
typedef int(__cdecl *close_t)(int);
typedef void *(__cdecl *fopen_t)(const char *, const char *);
typedef int(__cdecl *fprintf_t)(void *, const char *, ...);
typedef int(__cdecl *fclose_t)(void *);
typedef intptr_t(__cdecl *spawnl_t)(int, const char *, const char *, ...);

#define GET(T, n) ((T)(void *)GetProcAddress(h, n))
#define O_RDONLY_ 0x0000
#define O_RDWR_ 0x0002
#define O_CREAT_ 0x0100
#define O_TRUNC_ 0x0200
#define SH_DENYNO_ 0x40
#define S_IREAD_ 0x0100
#define S_IWRITE_ 0x0080

static void tmpname(char *out, const char *tag)
{
    /* x86 boots have %TEMP% on the read-only live CD; their scratch FAT disk carries an
     * empty \TEMP (tests/qemu/vm.py make_temp_disk). arm64's ramdisk %TEMP% is fastfat. */
    static const char *const scratch[] = {"C:\\TEMP\\", "D:\\TEMP\\", "E:\\TEMP\\"};
    char dir[MAX_PATH] = "";
    for (int k = 0; k < 3 && !dir[0]; k++)
        if (GetFileAttributesA(scratch[k]) != INVALID_FILE_ATTRIBUTES)
            lstrcpyA(dir, scratch[k]);
    if (!dir[0])
        GetTempPathA(sizeof dir, dir);
    snprintf(out, MAX_PATH, "%svaentry-%s-%lu.tmp", dir, tag, GetCurrentProcessId());
}

static void probe(const char *dll)
{
    HMODULE h = LoadLibraryA(dll);
    char buf[128], s[32], path[MAX_PATH];
    wchar_t wbuf[64];
    int n0 = g_checks, f0 = g_fail, r, i, j;
    double d;

    if (!h) {
        check(dll, "LoadLibrary", 0);
        return;
    }

    sprintf_t psprintf = GET(sprintf_t, "sprintf");
    if (psprintf) {
        /* 2 fixed + 9 variadic, mixed: crosses rcx..r9 and x0..x7 with a double and an int64 */
        r = psprintf(buf, "%d|%s|%.3f|%I64d|%c|%x|%d|%d|%.1f", 1, "two", 3.25, 0x123456789LL, 'Z', 0xbeef, 7, 8, 9.5);
        check(dll, "sprintf mixed", r == 37 && !strcmp(buf, "1|two|3.250|4886718345|Z|beef|7|8|9.5"));
        if (strcmp(buf, "1|two|3.250|4886718345|Z|beef|7|8|9.5"))
            SAY("%s sprintf got \"%s\" (%d)\n", dll, buf, r);
    } else if (!strcmp(dll, "msvcrt.dll"))
        check(dll, "sprintf exported", 0); /* ucrtbase exports no sprintf (it is inline in UCRT headers) */

    snprintf_t psnprintf = GET(snprintf_t, "_snprintf");
    if (psnprintf) {
        memset(buf, 'x', sizeof buf);
        r = psnprintf(buf, 5, "%s", "abcdefgh");
        check(dll, "_snprintf truncates", r == -1 && !memcmp(buf, "abcde", 5) && buf[5] == 'x');
    }

    snprintf_s_l_t psl = GET(snprintf_s_l_t, "_snprintf_s_l");
    if (psl) {
        /* 5 FIXED arguments: on amd64 the locale is already on the stack */
        r = psl(buf, sizeof buf, (size_t)-1, "%d-%d-%d-%.2f-%s", NULL, 10, 20, 30, 0.5, "end");
        check(dll, "_snprintf_s_l 5 fixed", r == 17 && !strcmp(buf, "10-20-30-0.50-end"));
        if (strcmp(buf, "10-20-30-0.50-end"))
            SAY("%s _snprintf_s_l got \"%s\" (%d)\n", dll, buf, r);
    }
    snwprintf_s_l_t pwsl = GET(snwprintf_s_l_t, "_snwprintf_s_l");
    if (pwsl) {
        r = pwsl(wbuf, 64, (size_t)-1, L"%d:%ls:%d", NULL, 5, L"w", 6);
        check(dll, "_snwprintf_s_l", r == 5 && !wcscmp(wbuf, L"5:w:6"));
    }
    swprintf_s_t pswp = GET(swprintf_s_t, "swprintf_s");
    if (pswp) {
        r = pswp(wbuf, 64, L"%d %ls %d", 42, L"wide", -1);
        check(dll, "swprintf_s", r == 10 && !wcscmp(wbuf, L"42 wide -1"));
    }
    scprintf_t pscp = GET(scprintf_t, "_scprintf");
    if (pscp)
        check(dll, "_scprintf 10 ints", pscp("%d%d%d%d%d%d%d%d%d%d", 1, 2, 3, 4, 5, 6, 7, 8, 9, 10) == 11);

    sscanf_t psscanf = GET(sscanf_t, "sscanf");
    if (psscanf) {
        i = j = 0; d = 0; s[0] = 0;
        r = psscanf("12 3.5 hello 99", "%d %lf %31s %d", &i, &d, s, &j);
        check(dll, "sscanf", r == 4 && i == 12 && d == 3.5 && !strcmp(s, "hello") && j == 99);
    }
    snscanf_t psnscanf = GET(snscanf_t, "_snscanf");
    if (psnscanf) {
        i = j = 0;
        r = psnscanf("77 88 99", 5, "%d %d", &i, &j);
        check(dll, "_snscanf length", r == 2 && i == 77 && j == 88);
    }

    /* _sopen's pmode is variadic: a file created _S_IREAD only comes out read-only. */
    sopen_t psopen = GET(sopen_t, "_sopen");
    open_t popen = GET(open_t, "_open");
    close_t pclose = GET(close_t, "_close");
    if (psopen && popen && pclose) {
        tmpname(path, dll);
        DeleteFileA(path);
        int fd = psopen(path, O_CREAT_ | O_RDWR_ | O_TRUNC_, SH_DENYNO_, S_IREAD_);
        check(dll, "_sopen O_CREAT", fd >= 0);
        if (fd >= 0)
            pclose(fd);
        DWORD at = GetFileAttributesA(path);
        check(dll, "_sopen pmode _S_IREAD -> read-only", at != INVALID_FILE_ATTRIBUTES && (at & FILE_ATTRIBUTE_READONLY));
        fd = popen(path, O_RDONLY_);
        check(dll, "_open without pmode", fd >= 0);
        if (fd >= 0)
            pclose(fd);
        SetFileAttributesA(path, FILE_ATTRIBUTE_NORMAL);
        DeleteFileA(path);
        fd = popen(path, O_CREAT_ | O_RDWR_, S_IREAD_ | S_IWRITE_);
        check(dll, "_open O_CREAT pmode rw", fd >= 0);
        if (fd >= 0)
            pclose(fd);
        at = GetFileAttributesA(path);
        check(dll, "_open pmode rw -> writable", at != INVALID_FILE_ATTRIBUTES && !(at & FILE_ATTRIBUTE_READONLY));
        DeleteFileA(path);
    } else
        check(dll, "_sopen/_open/_close exported", 0);

    /* fprintf through a FILE of the same DLL, read back raw */
    fopen_t pfopen = GET(fopen_t, "fopen");
    fprintf_t pfprintf = GET(fprintf_t, "fprintf");
    fclose_t pfclose = GET(fclose_t, "fclose");
    if (pfopen && pfprintf && pfclose) {
        tmpname(path, "fp");
        void *f = pfopen(path, "wb");
        check(dll, "fopen", f != NULL);
        if (f) {
            r = pfprintf(f, "%s=%d,%.2f,%I64u,%d,%d,%d", "k", 1, 2.5, 18446744073709551615ULL, 4, 5, 6);
            pfclose(f);
            HANDLE fh = CreateFileA(path, GENERIC_READ, 0, NULL, OPEN_EXISTING, 0, NULL);
            DWORD got = 0;
            memset(buf, 0, sizeof buf);
            if (fh != INVALID_HANDLE_VALUE) {
                ReadFile(fh, buf, sizeof buf - 1, &got, NULL);
                CloseHandle(fh);
            }
            check(dll, "fprintf to FILE", r == 35 && !strcmp(buf, "k=1,2.50,18446744073709551615,4,5,6"));
            if (strcmp(buf, "k=1,2.50,18446744073709551615,4,5,6"))
                SAY("%s fprintf got \"%s\" (%d)\n", dll, buf, r);
        }
        DeleteFileA(path);
    }

    /* _spawnl walks its argument list into the child's command line */
    spawnl_t pspawnl = GET(spawnl_t, "_spawnl");
    if (pspawnl) {
        char cmd[MAX_PATH];
        GetSystemDirectoryA(cmd, sizeof cmd);
        strcat(cmd, "\\cmd.exe");
        intptr_t rc = pspawnl(0 /* _P_WAIT */, cmd, "cmd", "/c", "exit", "7", NULL);
        check(dll, "_spawnl P_WAIT exit 7", rc == 7);
        if (rc != 7)
            SAY("%s _spawnl returned %ld\n", dll, (long)rc);
    } else
        check(dll, "_spawnl exported", 0);

    SAY("%s: %d checks, %d failed\n", dll, g_checks - n0, g_fail - f0);
}

/* _CrtDbgReport[W] (msvcrt only): `...` exports too, and the whole report family is Rust
 * (msvcrt-crt-rust dbgrpt.rs, was sdk/lib/crt/misc/dbgrpt.cpp). File mode only -- the
 * window mode would block on a message box. Expected text is the C's formatting. */
typedef int(__cdecl *crtdbgreport_t)(int, const char *, int, const char *, const char *, ...);
typedef int(__cdecl *crtdbgreportw_t)(int, const wchar_t *, int, const wchar_t *, const wchar_t *, ...);
typedef int(__cdecl *setmode_t)(int, int);
typedef HANDLE(__cdecl *setfile_t)(int, HANDLE);

static void read_back(const char *path, char *buf, DWORD cap, DWORD *got)
{
    HANDLE fh = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_EXISTING, 0, NULL);
    *got = 0;
    memset(buf, 0, cap);
    if (fh != INVALID_HANDLE_VALUE) {
        ReadFile(fh, buf, cap - 2, got, NULL);
        CloseHandle(fh);
    }
}

static void probe_dbgrpt(void)
{
    HMODULE h = LoadLibraryA("msvcrt.dll");
    crtdbgreport_t rep = GET(crtdbgreport_t, "_CrtDbgReport");
    crtdbgreportw_t repw = GET(crtdbgreportw_t, "_CrtDbgReportW");
    setmode_t setmode = GET(setmode_t, "_CrtSetReportMode");
    setfile_t setfile = GET(setfile_t, "_CrtSetReportFile");
    const char *d = "msvcrt.dll";
    char path[MAX_PATH], buf[256];
    DWORD got;
    int n0 = g_checks, f0 = g_fail;

    if (!rep || !repw || !setmode || !setfile) {
        check(d, "_CrtDbgReport family exported", 0);
        return;
    }
    tmpname(path, "dbgrpt");
    HANDLE fh = CreateFileA(path, GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, CREATE_ALWAYS, 0, NULL);
    check(d, "report file", fh != INVALID_HANDLE_VALUE);
    if (fh == INVALID_HANDLE_VALUE)
        return;

    /* _CRT_WARN 0, _CRT_ERROR 1, _CRT_ASSERT 2; _CRTDBG_MODE_FILE 1, _CRTDBG_MODE_DEBUG 2 */
    int old_warn = setmode(0, 1);
    check(d, "_CrtSetReportMode(WARN) old = DEBUG", old_warn == 2);
    check(d, "_CrtSetReportMode(WARN, REPORT_MODE) reads FILE", setmode(0, -1) == 1);
    HANDLE old_file = setfile(0, fh);
    check(d, "_CrtSetReportFile(WARN) old = INVALID", old_file == (HANDLE)(intptr_t)-1);
    check(d, "_CrtSetReportFile(WARN, REPORT_FILE) reads it back", setfile(0, (HANDLE)(intptr_t)-6) == fh);
    check(d, "_CrtSetReportMode(bad type) = 0", setmode(3, 1) == 0);

    int r = rep(0, "f.c", 12, NULL, "hello %d %s %.1f", 42, "x", 2.5);
    read_back(path, buf, sizeof buf, &got);
    check(d, "_CrtDbgReport(WARN) returns 0", r == 0);
    check(d, "_CrtDbgReport(WARN) text", !strcmp(buf, "f.c(12) : hello 42 x 2.5"));
    if (strcmp(buf, "f.c(12) : hello 42 x 2.5"))
        SAY("dbgrpt warn got \"%s\"\n", buf);

    setmode(2, 1);
    setfile(2, fh);
    r = rep(2, NULL, 0, NULL, "boom %d", 7);
    read_back(path, buf, sizeof buf, &got);
    check(d, "_CrtDbgReport(ASSERT) text", !strcmp(buf, "f.c(12) : hello 42 x 2.5Assertion failed: boom 7\r\n"));
    if (strcmp(buf, "f.c(12) : hello 42 x 2.5Assertion failed: boom 7\r\n"))
        SAY("dbgrpt assert got \"%s\"\n", buf);

    setmode(1, 1);
    setfile(1, fh);
    SetFilePointer(fh, 0, NULL, FILE_BEGIN);
    SetEndOfFile(fh);
    r = repw(1, L"w.c", 3, NULL, L"%d-%d", 1, 2);
    read_back(path, buf, sizeof buf, &got);
    check(d, "_CrtDbgReportW(ERROR) text", got == 24 && !memcmp(buf, L"w.c(3) : 1-2", 24));

    setmode(0, old_warn);
    setfile(0, old_file);
    setmode(1, 4);
    setmode(2, 4);
    CloseHandle(fh);
    DeleteFileA(path);
    SAY("%s dbgrpt: %d checks, %d failed\n", d, g_checks - n0, g_fail - f0);
}

int __cdecl main(void)
{
    probe("msvcrt.dll");
    probe("ucrtbase.dll");
    probe_dbgrpt();
    SAY("%d checks, %d failed\n", g_checks, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return 0;
}
