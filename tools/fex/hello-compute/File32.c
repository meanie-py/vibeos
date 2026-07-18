/*
 * file32.exe — the i386 file-I/O guest for WoW64 M10
 * (docs/engineering/active-lanes/wow64-plan.md). Built with the llvm-mingw i686
 * target, no CRT (plain _start entry, same as Hello32.c/Gui32.c), importing only
 * kernel32, loaded through the genuine 32-bit loader under FEX.
 *
 * What it proves: the IO_STATUS_BLOCK family marshals correctly 32<->64. That
 * struct's `ULONG_PTR Information` is 4 bytes in the guest and 8 on the host, so
 * before M10 (when NtReadFile & friends were AUTO-thunked) the host kernel wrote
 * 16 bytes into the guest's 8-byte IOSB and the guest read Information from the
 * wrong offset. `ReadFile`'s returned byte count IS that field, which makes it a
 * sharp oracle: a wrong count means the marshalling is broken, full stop.
 *
 * Verdict contract (same as hello32/gui32/win32): exit code 0x600D on PASS;
 * distinct 0xBADx codes name the failing rung so serial alone triages. The
 * winlogon harness (TryStartWow64File) logs the exit code.
 */

#include <windows.h>

/* Distinct exit codes per rung */
#define FILE32_FAIL_OPEN        0xBAD1
#define FILE32_FAIL_SIZE        0xBAD2
#define FILE32_FAIL_READ        0xBAD3
#define FILE32_FAIL_COUNT       0xBAD4 /* ReadFile lied about the byte count -> IOSB.Information */
#define FILE32_FAIL_CONTENT     0xBAD5
#define FILE32_FAIL_SEEK        0xBAD6
#define FILE32_FAIL_WRITE       0xBAD7
#define FILE32_FAIL_ROUNDTRIP   0xBAD8
#define FILE32_FAIL_FIND        0xBAD9
#define FILE32_PASS             0x600D

/* Paths are derived from GetWindowsDirectory at runtime: the amd64 livecd's
 * system volume is X:, the arm64 ramdisk's is C: — a hardcoded drive letter
 * fails on one of them (it cost an 0xBAD1 on the first amd64 livecd run). */
static char SELF_PATH[MAX_PATH], TEMP_PATH[MAX_PATH], FIND_GLOB[MAX_PATH];
static WCHAR FIND_GLOB_W[MAX_PATH];
#define FIND_WANT  "hello32.exe"

static void PathJoinA(char *dst, const char *dir, const char *rest)
{
    int i = 0, j = 0;
    while (dir[j]) dst[i++] = dir[j++];
    dst[i++] = '\\';
    j = 0;
    while (rest[j]) dst[i++] = rest[j++];
    dst[i] = 0;
}

static void BuildPaths(void)
{
    char win[MAX_PATH];
    WCHAR winw[MAX_PATH];
    int i, j;
    win[0] = 0; winw[0] = 0;
    GetWindowsDirectoryA(win, MAX_PATH - 32);
    GetWindowsDirectoryW(winw, MAX_PATH - 32);
    PathJoinA(SELF_PATH, win, "syswow64\\hello32.exe");
    PathJoinA(TEMP_PATH, win, "syswow64\\file32.tmp");
    PathJoinA(FIND_GLOB, win, "syswow64\\*");
    i = 0; while (winw[i]) { FIND_GLOB_W[i] = winw[i]; i++; }
    {
        static const WCHAR Rest[] = L"\\syswow64\\*";
        j = 0;
        while (Rest[j]) FIND_GLOB_W[i++] = Rest[j++];
        FIND_GLOB_W[i] = 0;
    }
}

static void Report(const char *msg)
{
    /* OutputDebugStringA reaches serial via the guest DbgPrint forwarding
     * (int 2d DebugService -> [GUEST32] in the wow64 VEH). */
    OutputDebugStringA(msg);
}

/* No CRT: hand-rolled. */
static int StrEqI(const char *a, const char *b)
{
    for (;;)
    {
        char ca = *a++, cb = *b++;
        if (ca >= 'A' && ca <= 'Z') ca += 32;
        if (cb >= 'A' && cb <= 'Z') cb += 32;
        if (ca != cb) return 0;
        if (!ca) return 1;
    }
}

static void ReportHex(const char *label, DWORD v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[80];
    int i = 0, n;
    while (label[i] && i < 60) { buf[i] = label[i]; i++; }
    buf[i++] = ' '; buf[i++] = '0'; buf[i++] = 'x';
    for (n = 28; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    Report(buf);
}

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    HANDLE h;
    DWORD size, got, put;
    BYTE buf[512];
    BYTE pattern[256];
    WIN32_FIND_DATAA fd;
    HANDLE find;
    int i, found, nfound;

    /* ---- rung 1: open + size (NtOpenFile, NtQueryInformationFile) --------- */
    BuildPaths();
    Report("[file32] start: opening ");
    Report(SELF_PATH);
    Report("\n");
    h = CreateFileA(SELF_PATH, GENERIC_READ, FILE_SHARE_READ, NULL,
                    OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        Report("[file32] FAIL: CreateFileA(read) returned INVALID_HANDLE_VALUE\n");
        ExitProcess(FILE32_FAIL_OPEN);
    }

    /* GetFileSize -> NtQueryInformationFile(FileStandardInformation) */
    size = GetFileSize(h, NULL);
    ReportHex("[file32] GetFileSize =", size);
    if (size == INVALID_FILE_SIZE || size < sizeof(buf))
    {
        Report("[file32] FAIL: implausible file size\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_SIZE);
    }

    /* ---- rung 2: read (NtReadFile) — the IOSB.Information oracle ---------- */
    got = 0xdeadbeef;
    if (!ReadFile(h, buf, sizeof(buf), &got, NULL))
    {
        Report("[file32] FAIL: ReadFile returned FALSE\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_READ);
    }

    /* THE assertion. `got` is IO_STATUS_BLOCK.Information marshalled down from
     * the host. If the IOSB marshalling is wrong this is garbage (or 0), even
     * though the read itself "succeeded". */
    ReportHex("[file32] ReadFile bytes =", got);
    if (got != sizeof(buf))
    {
        Report("[file32] FAIL: ReadFile byte count != requested "
               "(IO_STATUS_BLOCK.Information marshalling is broken)\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_COUNT);
    }

    if (buf[0] != 'M' || buf[1] != 'Z')
    {
        Report("[file32] FAIL: no MZ header — read returned the wrong bytes\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_CONTENT);
    }
    Report("[file32] read 512 bytes, MZ header intact\n");

    /* ---- rung 3: seek + short read (NtSetInformationFile/FilePosition) ---- */
    if (SetFilePointer(h, 256, NULL, FILE_BEGIN) != 256)
    {
        Report("[file32] FAIL: SetFilePointer(256) did not land at 256\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_SEEK);
    }
    got = 0xdeadbeef;
    if (!ReadFile(h, pattern, 64, &got, NULL) || got != 64)
    {
        Report("[file32] FAIL: 64-byte read at offset 256 returned the wrong count\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_COUNT);
    }
    /* Those 64 bytes must equal what the first read already gave us at [256..320). */
    for (i = 0; i < 64; i++)
    {
        if (pattern[i] != buf[256 + i])
        {
            Report("[file32] FAIL: seek+read content != same range from the full read\n");
            CloseHandle(h);
            ExitProcess(FILE32_FAIL_CONTENT);
        }
    }
    CloseHandle(h);
    Report("[file32] seek+read at offset 256 agrees with the full read\n");

    /* ---- rung 4: write + read-back round trip (NtWriteFile) --------------- */
    for (i = 0; i < (int)sizeof(pattern); i++)
        pattern[i] = (BYTE)(i * 7 + 3);

    h = CreateFileA(TEMP_PATH, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS,
                    FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        DWORD e = GetLastError();
        if (e == 19 /* ERROR_WRITE_PROTECT */ || e == 5 /* ERROR_ACCESS_DENIED */)
        {
            /* The amd64 livecd's system volume is a CD: the write/round-trip
             * rungs cannot run there. The arm64 ramdisk (writable) still
             * exercises them, so skipping here loses no coverage. */
            ReportHex("[file32] write rung SKIPPED (read-only volume), err =", e);
            goto find_rung;
        }
        Report("[file32] FAIL: CreateFileA(write) returned INVALID_HANDLE_VALUE\n");
        ExitProcess(FILE32_FAIL_WRITE);
    }
    put = 0xdeadbeef;
    if (!WriteFile(h, pattern, sizeof(pattern), &put, NULL) || put != sizeof(pattern))
    {
        Report("[file32] FAIL: WriteFile byte count wrong (IOSB.Information)\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_WRITE);
    }
    CloseHandle(h);
    ReportHex("[file32] WriteFile bytes =", put);

    h = CreateFileA(TEMP_PATH, GENERIC_READ, FILE_SHARE_READ, NULL,
                    OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE)
    {
        Report("[file32] FAIL: cannot reopen the file we just wrote\n");
        ExitProcess(FILE32_FAIL_ROUNDTRIP);
    }
    got = 0xdeadbeef;
    if (!ReadFile(h, buf, sizeof(pattern), &got, NULL) || got != sizeof(pattern))
    {
        Report("[file32] FAIL: read-back byte count wrong\n");
        CloseHandle(h);
        ExitProcess(FILE32_FAIL_ROUNDTRIP);
    }
    for (i = 0; i < (int)sizeof(pattern); i++)
    {
        if (buf[i] != pattern[i])
        {
            Report("[file32] FAIL: read-back content != what we wrote\n");
            CloseHandle(h);
            ExitProcess(FILE32_FAIL_ROUNDTRIP);
        }
    }
    CloseHandle(h);
    DeleteFileA(TEMP_PATH);
    Report("[file32] write+read-back round trip of 256 bytes matched\n");

    /* ---- rung 5: directory enumeration (NtQueryDirectoryFile) ------------- */
find_rung:
    found = 0;
    find = FindFirstFileA(FIND_GLOB, &fd);
    if (find == INVALID_HANDLE_VALUE)
    {
        Report("[file32] FAIL: FindFirstFileA over syswow64 found nothing\n");
        ExitProcess(FILE32_FAIL_FIND);
    }
    nfound = 0;
    do
    {
        nfound++;
        if (StrEqI(fd.cFileName, FIND_WANT))
            found = 1;
    } while (!found && FindNextFileA(find, &fd));
    FindClose(find);

    if (!found)
    {
        /* Self-triage: re-run the SAME scan through the WIDE API. It shares the
         * directory scan and skips only the wide->ANSI down-conversion, so it
         * says which half is broken — and it prints the names, which is what you
         * want when the ANSI ones came back blank.
         *
         * As of 2026-07-12 this is a LIVE BUG: with a large syswow64 (the M11
         * 53-module import closure) every ANSI cFileName comes back EMPTY —
         * even "." — while the wide names are perfect. With the old 9-file
         * syswow64 the ANSI names were fine, so it is exposed by, not caused by,
         * the bigger directory. See wow64-plan.md "OWED: FindFirstFileA". */
        WIN32_FIND_DATAW fdw;
        HANDLE findw;
        ReportHex("[file32] entries enumerated before the scan ended =", nfound);
        Report("[file32] the ANSI scan never yielded " FIND_WANT
               " — retrying through the WIDE API to localise it\n");
        findw = FindFirstFileW(FIND_GLOB_W, &fdw);
        if (findw != INVALID_HANDLE_VALUE)
        {
            int wfound = 0;
            do
            {
                char nameA[MAX_PATH];
                int k = 0;
                while (k < MAX_PATH - 1 && fdw.cFileName[k])
                {
                    nameA[k] = (char)fdw.cFileName[k]; /* ASCII-only, deliberately crude */
                    k++;
                }
                nameA[k] = 0;
                Report("[file32]   direntW: '");
                Report(nameA);
                Report("'\n");
                if (StrEqI(nameA, FIND_WANT))
                    wfound = 1;
            } while (!wfound && FindNextFileW(findw, &fdw));
            FindClose(findw);
            if (wfound)
                Report("[file32] DIAGNOSIS: the WIDE enumeration DID find it. The directory "
                       "scan (NtQueryDirectoryFile) is fine; FindFirstFileA's wide->ANSI "
                       "down-conversion is what's broken.\n");
            else
                Report("[file32] DIAGNOSIS: the WIDE enumeration missed it too — the "
                       "directory scan itself is wrong, not the ANSI conversion.\n");
        }
    }

    if (!found)
    {
        ReportHex("[file32] entries enumerated before the scan ended =", nfound);
        Report("[file32] FAIL: directory enumeration never yielded " FIND_WANT "\n");
        ExitProcess(FILE32_FAIL_FIND);
    }
    Report("[file32] directory enumeration found " FIND_WANT "\n");

    Report("[file32] PASS: 32-bit file I/O through the native arm64 kernel\n");
    ExitProcess(FILE32_PASS);
    return 0;
}
