/*
 * modrun — run a translated module in the guest, Rust and its pre-flip C side by side.
 *
 * GUI-subsystem launcher in the All Users Startup folder (a .cmd there never runs:
 * explorer starts Startup items with ShellExecuteExW). Reads %SystemRoot%\bin\modrun.txt,
 * GENERATED from tools/payloads/modrun/suites/*.yaml by gen.py:
 *
 *     case diskpart.list-disk
 *     variant rust %SystemRoot%\system32\diskpart.exe
 *     variant c %SystemRoot%\bin\modrun\diskpart_c.exe
 *     stdin list disk
 *     timeout 60
 *     end
 *
 * Every variant gets the same command line and stdin, its stdout+stderr piped back and
 * re-emitted on the debug serial as `[MODRUN:<case>:<variant>] <line>`, then
 * `[MODRUN] exit <case> <variant> rc=0x.. ms=..`. This program judges NOTHING: the
 * host-side grader (grade.py) diffs the variants and checks the suite's expectations, so
 * a new check is a YAML edit, never a rebuild of the guest image.
 */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define SAY(...) DbgPrint("[MODRUN] " __VA_ARGS__)
#include "../stackdump.h"
#include <tlhelp32.h>

#define MAX_VARIANTS 4
#define MAX_STDIN    16384
#define MAX_OUTPUT   (1u << 20)
#define CHUNK        400 /* DbgPrint truncates near 512 bytes */

struct variant
{
    char name[32];
    char cmd[1024];
};

struct capture
{
    HANDLE Read;
    char *Buf;
    DWORD Used;
    BOOL Truncated;
};

static DWORD WINAPI reader(LPVOID p)
{
    struct capture *c = (struct capture *)p;
    char tmp[4096];
    DWORD got;
    while (ReadFile(c->Read, tmp, sizeof tmp, &got, NULL) && got)
    {
        DWORD room = MAX_OUTPUT - c->Used;
        if (got > room)
        {
            got = room;
            c->Truncated = TRUE;
        }
        memcpy(c->Buf + c->Used, tmp, got);
        c->Used += got;
    }
    return 0;
}

static void dump_threads(DWORD pid)
{
    THREADENTRY32 te = { sizeof te };
    HANDLE s = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if (s == INVALID_HANDLE_VALUE)
        return;
    for (BOOL k = Thread32First(s, &te); k; k = Thread32Next(s, &te))
        if (te.th32OwnerProcessID == pid)
            dump_thread(pid, te.th32ThreadID);
    CloseHandle(s);
}

/* UTF-16 output (a console program writing wide text into a pipe) -> UTF-8, so both
 * variants reach the grader in one encoding whatever their CRT chose. Narrow output that is
 * not UTF-8 is in the console's OEM codepage (conutils converts to it): fc printed `thr\xE9e`,
 * the host's serial reader decodes UTF-8 and replaced the byte, and both copies of the record
 * failed their checksum -- reported as transport loss, never graded. */
static char *as_utf8(char *buf, DWORD *len)
{
    DWORD n = *len, zeros = 0, i;
    int wide;
    for (i = 1; i < n; i += 2)
        zeros += buf[i] == 0;
    wide = (n >= 2 && (BYTE)buf[0] == 0xFF && (BYTE)buf[1] == 0xFE) || (n >= 4 && zeros * 3 > n / 2 * 2);
    if (!wide)
    {
        int wn;
        WCHAR *w;
        char *u;
        int out;
        if (!n || MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, buf, (int)n, NULL, 0) > 0)
            return buf;
        wn = MultiByteToWideChar(CP_OEMCP, 0, buf, (int)n, NULL, 0);
        w = (WCHAR *)malloc(wn * sizeof(WCHAR));
        MultiByteToWideChar(CP_OEMCP, 0, buf, (int)n, w, wn);
        out = WideCharToMultiByte(CP_UTF8, 0, w, wn, NULL, 0, NULL, NULL);
        u = (char *)malloc(out + 1);
        WideCharToMultiByte(CP_UTF8, 0, w, wn, u, out, NULL, NULL);
        free(w);
        *len = (DWORD)out;
        return u;
    }
    {
        const WCHAR *w = (const WCHAR *)buf;
        int wn = (int)(n / 2), out;
        char *u;
        if (wn && w[0] == 0xFEFF)
            w++, wn--;
        out = WideCharToMultiByte(CP_UTF8, 0, w, wn, NULL, 0, NULL, NULL);
        u = (char *)malloc(out + 1);
        WideCharToMultiByte(CP_UTF8, 0, w, wn, u, out, NULL, NULL);
        *len = (DWORD)out;
        return u;
    }
}

/* The debug serial is not line-atomic: on i386 another thread's DbgPrint lands in the middle
 * of ours ("ptype 0x09 AIX dat[CENSUS] wait profile..."), and a record can be lost whole. So
 * every record carries its sequence number and a checksum, and goes out TWICE; the grader keeps
 * the first intact copy of each and reports a gap as transport loss, never as a diff. A `+`
 * after the number means the next record continues this line (a line longer than CHUNK). */
static unsigned checksum(const char *s, size_t n, unsigned seq)
{
    unsigned h = 0x811c9dc5u ^ seq;
    size_t i;
    for (i = 0; i < n; ++i)
        h = (h ^ (unsigned char)s[i]) * 0x01000193u;
    return (h ^ (h >> 16)) & 0xFFFFu;
}

static unsigned emit(const char *id, const char *variant, const char *line, size_t n, unsigned seq)
{
    size_t at = 0;
    do
    {
        size_t k = n - at > CHUNK ? CHUNK : n - at;
        int more = at + k < n;
        unsigned sum = checksum(line + at, k, seq);
        int copy;
        for (copy = 0; copy < 2; ++copy)
            DbgPrint("[MODRUN:%s:%s:%u%s:%04x] %.*s\n", id, variant, seq, more ? "+" : "", sum, (int)k,
                     line + at);
        at += k;
        ++seq;
    } while (at < n);
    return seq;
}

static void run(const char *id, const struct variant *v, const char *in, DWORD in_len, DWORD timeout_ms)
{
    SECURITY_ATTRIBUTES sa = { sizeof sa, NULL, TRUE };
    HANDLE out_rd, out_wr, in_rd, in_wr, th;
    STARTUPINFOA si = { sizeof si };
    PROCESS_INFORMATION pi;
    struct capture cap = { 0 };
    char cmd[2048];
    DWORD rc = 0, t0, ms, wrote, i, start;
    unsigned seq = 0;
    BOOL timed_out = FALSE;
    char *text;

    ExpandEnvironmentStringsA(v->cmd, cmd, sizeof cmd);
    if (!CreatePipe(&out_rd, &out_wr, &sa, 0) || !CreatePipe(&in_rd, &in_wr, &sa, MAX_STDIN))
    {
        SAY("exit %s %s rc=LAUNCH ms=0 CreatePipe err=%lu\n", id, v->name, GetLastError());
        return;
    }
    SetHandleInformation(out_rd, HANDLE_FLAG_INHERIT, 0);
    SetHandleInformation(in_wr, HANDLE_FLAG_INHERIT, 0);
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdInput = in_rd;
    si.hStdOutput = out_wr;
    si.hStdError = out_wr;
    SAY("start %s %s %s\n", id, v->name, cmd);
    t0 = GetTickCount();
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, DETACHED_PROCESS, NULL, NULL, &si, &pi))
    {
        SAY("exit %s %s rc=LAUNCH ms=0 CreateProcess err=%lu\n", id, v->name, GetLastError());
        CloseHandle(out_rd), CloseHandle(out_wr), CloseHandle(in_rd), CloseHandle(in_wr);
        return;
    }
    CloseHandle(out_wr);
    CloseHandle(in_rd);
    cap.Read = out_rd;
    cap.Buf = (char *)malloc(MAX_OUTPUT);
    th = CreateThread(NULL, 0, reader, &cap, 0, NULL);
    if (in_len)
        WriteFile(in_wr, in, in_len, &wrote, NULL);
    CloseHandle(in_wr); /* EOF: a program still reading stdin sees the script end */
    if (WaitForSingleObject(pi.hProcess, timeout_ms) == WAIT_TIMEOUT)
    {
        timed_out = TRUE;
        SAY("TIMEOUT %s %s after %lums, threads:\n", id, v->name, timeout_ms);
        dump_threads(pi.dwProcessId);
        TerminateProcess(pi.hProcess, 0xDEAD);
        WaitForSingleObject(pi.hProcess, 5000);
    }
    ms = GetTickCount() - t0;
    GetExitCodeProcess(pi.hProcess, &rc);
    WaitForSingleObject(th, 10000);
    CloseHandle(th);
    CloseHandle(out_rd);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);

    text = as_utf8(cap.Buf, &cap.Used);
    for (start = 0, i = 0; i <= cap.Used; ++i)
    {
        if (i == cap.Used || text[i] == '\n')
        {
            DWORD end = i;
            if (end > start && text[end - 1] == '\r')
                --end;
            if (i < cap.Used || end > start)
                seq = emit(id, v->name, text + start, end - start, seq);
            start = i + 1;
        }
    }
    SAY("exit %s %s rc=0x%08lx ms=%lu records=%u%s%s\n", id, v->name, rc, ms, seq,
        timed_out ? " TIMEOUT" : "", cap.Truncated ? " TRUNCATED" : "");
    if (text != cap.Buf)
        free(text);
    free(cap.Buf);
}

/* The LiveCD registers its COM classes (syssetup's RegistrationPhase2) while the shell is
 * already up, and modrun is a shell startup program: the first case of a boot saw
 * CLSID_ProgressDialog unregistered and the next one found it. syssetup sets
 * HKLM\SYSTEM\Setup\VibeosRegistrationDone when it is through; wait for it on the LiveCD
 * (an installed system registered everything at setup, and never sets it). */
static void wait_for_registration(void)
{
    char root[MAX_PATH];
    DWORD done = 0, cb = sizeof done, t0 = GetTickCount();
    GetWindowsDirectoryA(root, sizeof root);
    if (root[0] != 'X' && root[0] != 'x')
        return;
    while (RegGetValueA(HKEY_LOCAL_MACHINE, "SYSTEM\\Setup", "VibeosRegistrationDone", RRF_RT_REG_DWORD, NULL,
                        &done, &cb) != ERROR_SUCCESS || !done)
    {
        if (GetTickCount() - t0 > 600000)
        {
            SAY("setup registration NOT signalled after 600000ms: COM-dependent cases may fail\n");
            return;
        }
        cb = sizeof done;
        Sleep(500);
    }
    SAY("setup registration done after %lums\n", GetTickCount() - t0);
}

int main(void)
{
    char path[MAX_PATH], line[2048], id[128] = "";
    static char in[MAX_STDIN];
    struct variant vars[MAX_VARIANTS];
    int nvars = 0, cases = 0;
    DWORD in_len = 0, timeout_ms = 60000;
    FILE *f;

    ExpandEnvironmentStringsA("%SystemRoot%\\bin\\modrun.txt", path, sizeof path);
    f = fopen(path, "rb");
    if (!f)
    {
        SAY("FATAL: cannot open %s\n", path);
        SAY("done cases=0 FATAL\n");
        return 1;
    }
    SAY("begin %s\n", path);
    wait_for_registration();
    while (fgets(line, sizeof line, f))
    {
        size_t n = strlen(line);
        while (n && (line[n - 1] == '\n' || line[n - 1] == '\r'))
            line[--n] = 0;
        if (!strncmp(line, "case ", 5))
        {
            lstrcpynA(id, line + 5, sizeof id);
            nvars = 0, in_len = 0, timeout_ms = 60000;
        }
        else if (!strncmp(line, "variant ", 8) && nvars < MAX_VARIANTS)
        {
            const char *sp = strchr(line + 8, ' ');
            if (!sp)
                continue;
            lstrcpynA(vars[nvars].name, line + 8, (int)(sp - (line + 8)) + 1);
            lstrcpynA(vars[nvars].cmd, sp + 1, sizeof vars[nvars].cmd);
            ++nvars;
        }
        else if (!strncmp(line, "stdin ", 6) || !strcmp(line, "stdin"))
        {
            const char *s = line[5] ? line + 6 : "";
            size_t k = strlen(s);
            if (in_len + k + 2 <= sizeof in)
            {
                memcpy(in + in_len, s, k);
                in_len += (DWORD)k;
                in[in_len++] = '\r';
                in[in_len++] = '\n';
            }
        }
        else if (!strncmp(line, "timeout ", 8))
            timeout_ms = (DWORD)atoi(line + 8) * 1000u;
        else if (!strcmp(line, "end"))
        {
            int k;
            ++cases;
            SAY("case %s variants=%d\n", id, nvars);
            for (k = 0; k < nvars; ++k)
                run(id, &vars[k], in, in_len, timeout_ms);
        }
    }
    fclose(f);
    SAY("done cases=%d\n", cases);
    return 0;
}
