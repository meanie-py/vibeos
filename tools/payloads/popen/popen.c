/* popen -- why Wine's msvcrt misc.c:375 `_popen` fails with errno 13 (EACCES) on vibeos,
 * after which the test's fgets(NULL) kills the misc subtest (exit 128, no summary line).
 * Drives msvcrt.dll's _popen with the shapes misc.c uses and logs errno, _doserrno and the
 * Win32 error each step leaves, plus what cmd.exe resolution sees. */
#include <windows.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[POPEN] " __VA_ARGS__)
static int g_fail;

static int *(__cdecl *p_errno)(void);
static unsigned long *(__cdecl *p_doserrno)(void);
static FILE *(__cdecl *p_popen)(const char *, const char *);
static int (__cdecl *p_pclose)(FILE *);
static char *(__cdecl *p_fgets)(char *, int, FILE *);
static char *(__cdecl *p_getenv)(const char *);

static void try_popen(const char *what, const char *cmd, const char *mode, const char *expect)
{
    char buf[256] = "";
    *p_errno() = 0;
    *p_doserrno() = 0;
    SetLastError(0);
    FILE *f = p_popen(cmd, mode);
    DWORD gle = GetLastError();
    SAY("%s: _popen(%s, %s) = %p errno=%d doserrno=%lu gle=%lu\n", what, cmd, mode, f, *p_errno(), *p_doserrno(), gle);
    if (!f) { g_fail++; return; }
    if (expect) {
        p_fgets(buf, sizeof buf, f);
        SAY("%s: read \"%.60s\"\n", what, buf);
        if (strncmp(buf, expect, strlen(expect))) g_fail++;
    }
    SAY("%s: _pclose = %#x\n", what, p_pclose(f));
}


static void try_cp(const char *what, BOOL inherit, int stdmode)
{
    char cmdline[] = "cmd.exe /c echo hi";
    char app[MAX_PATH];
    GetSystemDirectoryA(app, sizeof app);
    lstrcatA(app, "\\cmd.exe");
    STARTUPINFOA si = { sizeof si };
    PROCESS_INFORMATION pi;
    HANDLE rd = NULL, wr = NULL;
    SECURITY_ATTRIBUTES sa = { sizeof sa, NULL, TRUE };
    if (stdmode == 1) {          /* a real inheritable pipe as stdout, like _popen */
        CreatePipe(&rd, &wr, &sa, 0);
        si.dwFlags = STARTF_USESTDHANDLES;
        si.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
        si.hStdOutput = wr;
        si.hStdError = GetStdHandle(STD_ERROR_HANDLE);
    }
    SetLastError(0);
    BOOL ok = CreateProcessA(app, cmdline, NULL, NULL, inherit, 0, NULL, NULL, &si, &pi);
    DWORD gle = GetLastError();
    SAY("%s: CreateProcessA(inherit=%d, std=%d) = %d gle=%lu stdin=%p stdout=%p stderr=%p\n", what, inherit, stdmode, ok, gle,
        GetStdHandle(STD_INPUT_HANDLE), GetStdHandle(STD_OUTPUT_HANDLE), GetStdHandle(STD_ERROR_HANDLE));
    if (ok) { WaitForSingleObject(pi.hProcess, 5000); CloseHandle(pi.hProcess); CloseHandle(pi.hThread); }
    else g_fail++;
    if (rd) CloseHandle(rd);
    if (wr) CloseHandle(wr);
}


static int (__cdecl *p_pipe)(int *, unsigned, int);
static int (__cdecl *p_dup)(int);
static int (__cdecl *p_close)(int);
static intptr_t (__cdecl *p_get_osfhandle)(int);

/* _popen's own steps, through msvcrt's exports, then its CreateProcessW call. */
static void replay(void)
{
    int fds[2] = { -1, -1 };
    *p_errno() = 0;
    int r = p_pipe(fds, 0, 0x8000 /*_O_BINARY*/ | 0x0080 /*_O_NOINHERIT*/);
    SAY("replay: _pipe = %d fds=%d,%d errno=%d\n", r, fds[0], fds[1], *p_errno());
    if (r) { g_fail++; return; }
    int child = p_dup(fds[1]);
    SAY("replay: _dup(%d) = %d errno=%d\n", fds[1], child, *p_errno());
    p_close(fds[1]);
    HANDLE hin = (HANDLE)p_get_osfhandle(0), hout = (HANDLE)p_get_osfhandle(child), herr = (HANDLE)p_get_osfhandle(2);
    DWORD fl = 0;
    BOOL got = GetHandleInformation(hout, &fl);
    SAY("replay: osfhandle stdin=%p child=%p stderr=%p; child inherit flag=%lu (got=%d)\n", hin, hout, herr, fl & 1, got);
    WCHAR app[MAX_PATH], cmd[MAX_PATH * 2];
    GetSystemDirectoryW(app, MAX_PATH);
    lstrcatW(app, L"\\cmd.exe");
    wsprintfW(cmd, L"%s /c echo hi", app);
    STARTUPINFOW si = { sizeof si };
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdInput = hin; si.hStdOutput = hout; si.hStdError = herr;
    PROCESS_INFORMATION pi;
    SetLastError(0);
    BOOL ok = CreateProcessW(app, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi);
    SAY("replay: CreateProcessW with msvcrt's handles = %d gle=%lu\n", ok, GetLastError());
    if (!ok) {
        /* which handle does it object to? */
        for (int k = 0; k < 3; k++) {
            STARTUPINFOW s2 = si;
            if (k == 0) s2.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
            if (k == 1) s2.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
            if (k == 2) s2.hStdError = GetStdHandle(STD_ERROR_HANDLE);
            SetLastError(0);
            BOOL o2 = CreateProcessW(app, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &s2, &pi);
            SAY("replay: ... with %s from GetStdHandle = %d gle=%lu\n", k == 0 ? "stdin" : k == 1 ? "stdout" : "stderr", o2, GetLastError());
            if (o2) { CloseHandle(pi.hProcess); CloseHandle(pi.hThread); }
        }
        g_fail++;
    } else { WaitForSingleObject(pi.hProcess, 5000); CloseHandle(pi.hProcess); CloseHandle(pi.hThread); }
    p_close(child);
    p_close(fds[0]);
}

int main(int argc, char **argv)
{
    if (argc > 1) {           /* child mode, as misc.c re-runs itself */
        printf("child output\n");
        ExitProcess(0x37);
    }
    HMODULE h = LoadLibraryA("msvcrt.dll");
#define G(v, n) (*(FARPROC *)&v = GetProcAddress(h, n))
    if (!G(p_errno, "_errno") || !G(p_doserrno, "__doserrno") || !G(p_popen, "_popen") || !G(p_pclose, "_pclose")
        || !G(p_fgets, "fgets") || !G(p_getenv, "getenv") || !G(p_pipe, "_pipe") || !G(p_dup, "_dup") || !G(p_close, "_close")
        || !G(p_get_osfhandle, "_get_osfhandle")) {
        SAY("done FAIL (load)\n");
        return 1;
    }
    char sysdir[MAX_PATH], self[MAX_PATH], cwd[MAX_PATH], cmd[2 * MAX_PATH];
    GetSystemDirectoryA(sysdir, sizeof sysdir);
    GetModuleFileNameA(NULL, self, sizeof self);
    GetCurrentDirectoryA(sizeof cwd, cwd);
    SAY("COMSPEC=%s sysdir=%s self=%s cwd=%s\n", p_getenv("COMSPEC") ? p_getenv("COMSPEC") : "(null)", sysdir, self, cwd);
    lstrcatA(sysdir, "\\cmd.exe");
    SAY("cmd.exe attrs=%#lx\n", GetFileAttributesA(sysdir));

    try_cp("cp-plain", FALSE, 0);
    try_cp("cp-inherit", TRUE, 0);
    try_cp("cp-pipe", TRUE, 1);
    replay();
    try_popen("echo", "echo hi", "r", "hi");
    wsprintfA(cmd, "\"%s\" child", self);
    try_popen("self-quoted", cmd, "r", "child output");

    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
