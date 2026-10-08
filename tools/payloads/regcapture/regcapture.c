/*
 * regcapture — what would setup's COM registration phase write? Emitted as INF AddReg lines.
 *
 * An installed system runs syssetup.inf's registration sections once, at setup: EarlyPhaseSetup's
 * [EarlyRegisterDlls] (ole32's proxies among them), RegistrationPhase2's [OleControlDlls], and
 * [TypeLibraries]. The livecd never runs setup, so none of it is in its hives, and every
 * cross-apartment COM call fails with REGDB_E_IIDNOTREG (upstream's livecd.inf hand-registers two
 * interfaces under "FIXME: Registration"). This payload runs those same sections in-guest, each
 * entry in its own child process against an in-process sandbox (RegOverridePredefKey, as
 * ../regdiff does), and streams what each wrote as INF lines over serial: a diagnostic for
 * what registration produces, per DLL.
 *
 * The sections are read from the image's own inf\syssetup.inf through SetupAPI, with setupapi's
 * register_dlls_callback field layout (3 file, 4 flags, 6 args) and its flag meanings
 * (1 DllRegisterServer, 2 DllInstall(TRUE, args)). Typelibs: LoadTypeLibEx(REGKIND_REGISTER),
 * which is what __wine_register_resources does for a .tlb.
 *
 * Protocol (DbgPrint caps a line, INF lines can be long):
 *   [REGCAP] begin <file>
 *   [REGCAP]+<up to 200 chars>       continuation chunk of the current line
 *   [REGCAP]=<up to 200 chars>       last chunk of the current line
 *   [REGCAP] end <file> rc=<child exit> hr=<hr> lines=<n>
 *   [REGCAP] done entries=<n> failed=<n>
 */
#include <windows.h>
#include <setupapi.h>
#include <oleauto.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define CHILD_TIMEOUT_MS (5u * 60u * 1000u)
static const HKEY g_roots[4] = { HKEY_CLASSES_ROOT, HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, HKEY_USERS };
static const char *g_root_names[4] = { "HKCR", "HKLM", "HKCU", "HKU" };

/* ---------------------------------------------------------------- child: emit INF lines */

static void emit(const char *line)
{
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    WriteFile(out, line, (DWORD)strlen(line), &n, NULL);
    WriteFile(out, "\n", 1, &n, NULL);
}

/* A quoted INF string: `"` doubles, `%` doubles (inflib substitutes %name%, and %% is a
 * literal percent). UTF-8 out; the generator refuses non-ASCII rather than guess an encoding. */
static size_t quote(const WCHAR *w, size_t cch, char *out, size_t outsz)
{
    size_t o = 0;
    out[o++] = '"';
    for (size_t i = 0; i < cch && o + 8 < outsz; i++) {
        WCHAR c = w[i];
        if (c == L'"') { out[o++] = '"'; out[o++] = '"'; }
        else if (c == L'%') { out[o++] = '%'; out[o++] = '%'; }
        else if (c < 0x80) out[o++] = (char)c;
        else o += WideCharToMultiByte(CP_UTF8, 0, &c, 1, out + o, (int)(outsz - o - 2), NULL, NULL);
    }
    out[o++] = '"';
    out[o] = 0;
    return o;
}

static char g_line[65536];

static void emit_value(const char *root, const char *qpath, const WCHAR *name, DWORD cchName,
                       DWORD type, const BYTE *data, DWORD cb)
{
    char qname[2048];
    size_t o;
    quote(name, cchName, qname, sizeof qname);
    const WCHAR *w = (const WCHAR *)data;
    size_t cch = cb / sizeof(WCHAR);
    if (type == REG_SZ || type == REG_EXPAND_SZ) {
        while (cch && w[cch - 1] == 0) cch--;
        o = snprintf(g_line, sizeof g_line, "%s,%s,%s,0x%08x,", root, qpath, qname,
                     type == REG_SZ ? 0x00000000 : 0x00020000);
        quote(w, cch, g_line + o, sizeof g_line - o);
    } else if (type == REG_MULTI_SZ) {
        o = snprintf(g_line, sizeof g_line, "%s,%s,%s,0x00010000", root, qpath, qname);
        for (size_t i = 0; i < cch && w[i];) {
            size_t len = wcsnlen(w + i, cch - i);
            g_line[o++] = ',';
            o += quote(w + i, len, g_line + o, sizeof g_line - o);
            i += len + 1;
        }
        g_line[o] = 0;
    } else if (type == REG_DWORD && cb == 4) {
        snprintf(g_line, sizeof g_line, "%s,%s,%s,0x00010001,0x%08lx", root, qpath, qname,
                 *(const DWORD *)data);
    } else {
        /* REG_BINARY is 0x00000001; any other type rides in the high word (mkhive's default
         * branch). 1 and 2 would collide with the DWORD/NONE encodings, but strings never
         * arrive here. */
        DWORD flags = type == REG_BINARY ? 0x00000001 : ((type << 16) | 1);
        o = snprintf(g_line, sizeof g_line, "%s,%s,%s,0x%08lx", root, qpath, qname, flags);
        for (DWORD i = 0; i < cb && o + 8 < sizeof g_line; i++)
            o += snprintf(g_line + o, sizeof g_line - o, ",%02x", data[i]);
    }
    emit(g_line);
}

static unsigned g_emitted;

static void walk(HKEY key, const char *root, const WCHAR *path)
{
    char qpath[4096];
    quote(path, wcslen(path), qpath, sizeof qpath);
    DWORD nsub = 0, maxsub = 0, nval = 0, maxname = 0, maxdata = 0;
    if (RegQueryInfoKeyW(key, NULL, NULL, NULL, &nsub, &maxsub, NULL, &nval, &maxname, &maxdata,
                         NULL, NULL) != ERROR_SUCCESS)
        return;
    if (*path && nval == 0 && nsub == 0) {        /* an empty key still has to exist */
        snprintf(g_line, sizeof g_line, "%s,%s,,0x00000010", root, qpath);
        emit(g_line);
        g_emitted++;
    }
    WCHAR *name = (WCHAR *)malloc((maxname + 2) * sizeof(WCHAR));
    BYTE *data = (BYTE *)malloc(maxdata + 8);
    for (DWORD i = 0; i < nval; i++) {
        DWORD cchName = maxname + 1, cb = maxdata + 4, type = 0;
        if (RegEnumValueW(key, i, name, &cchName, NULL, &type, data, &cb) != ERROR_SUCCESS)
            continue;
        emit_value(root, qpath, name, cchName, type, data, cb);
        g_emitted++;
    }
    free(data);
    free(name);
    WCHAR *sub = (WCHAR *)malloc((maxsub + 2) * sizeof(WCHAR));
    for (DWORD i = 0; i < nsub; i++) {
        DWORD cch = maxsub + 1;
        HKEY child;
        if (RegEnumKeyExW(key, i, sub, &cch, NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
            continue;
        if (RegOpenKeyExW(key, sub, 0, KEY_READ, &child) != ERROR_SUCCESS)
            continue;
        size_t plen = wcslen(path);
        WCHAR *cpath = (WCHAR *)malloc((plen + cch + 2) * sizeof(WCHAR));
        wcscpy(cpath, path);
        if (plen) wcscat(cpath, L"\\");
        wcscat(cpath, sub);
        walk(child, root, cpath);
        free(cpath);
        RegCloseKey(child);
    }
    free(sub);
}

/* Fails loud: an override that does not take sends the writes to the REAL registry, and the
 * capture would come back empty (see ../regdiff). */
static BOOL override_all(HKEY sandbox[4])
{
    for (int i = 0; i < 4; i++) {
        HKEY dup;
        LONG err = RegOpenKeyExW(sandbox[i], NULL, 0, KEY_ALL_ACCESS, &dup);
        if (err == ERROR_SUCCESS)
            err = RegOverridePredefKey(g_roots[i], dup);
        if (err != ERROR_SUCCESS) {
            DbgPrint("[REGCAP] FAIL RegOverridePredefKey(%s) = %ld\n", g_root_names[i], err);
            return FALSE;
        }
    }
    return TRUE;
}

static int child(const char *kind, const char *fileA, int flags, const char *argsA)
{
    WCHAR file[MAX_PATH], args[512], keypath[256];
    MultiByteToWideChar(CP_ACP, 0, fileA, -1, file, MAX_PATH);
    MultiByteToWideChar(CP_ACP, 0, argsA, -1, args, 512);
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX);
    CoInitialize(NULL);

    HKEY sandbox[4];
    DWORD pid = GetCurrentProcessId();
    for (int i = 0; i < 4; i++) {
        char keyA[256];
        snprintf(keyA, sizeof keyA, "Software\\vibeos-regcapture\\%lu\\%s", pid, g_root_names[i]);
        MultiByteToWideChar(CP_ACP, 0, keyA, -1, keypath, 256);
        if (RegCreateKeyExW(HKEY_CURRENT_USER, keypath, 0, NULL, 0, KEY_ALL_ACCESS, NULL,
                            &sandbox[i], NULL) != ERROR_SUCCESS)
            return 4;
    }

    HRESULT hr = S_OK;
    HMODULE m = NULL;
    if (!strcmp(kind, "tlb")) {
        ITypeLib *tl = NULL;
        if (!override_all(sandbox)) return 5;
        hr = LoadTypeLibEx(file, REGKIND_REGISTER, &tl);
        if (tl) tl->lpVtbl->Release(tl);
    } else {
        /* Loaded BEFORE the override, so the DLL's own initialisation sees the real registry. */
        m = LoadLibraryW(file);
        if (!m) { printf("#hr=0x%08lx\n", (unsigned long)HRESULT_FROM_WIN32(GetLastError())); return 2; }
        if (!override_all(sandbox)) return 5;
        if (flags & 1) {
            HRESULT (WINAPI *reg)(void) = (void *)GetProcAddress(m, "DllRegisterServer");
            hr = reg ? reg() : HRESULT_FROM_WIN32(ERROR_PROC_NOT_FOUND);
        }
        if (SUCCEEDED(hr) && (flags & 2)) {
            HRESULT (WINAPI *inst)(BOOL, LPCWSTR) = (void *)GetProcAddress(m, "DllInstall");
            hr = inst ? inst(TRUE, *args ? args : NULL) : HRESULT_FROM_WIN32(ERROR_PROC_NOT_FOUND);
        }
    }
    for (int i = 0; i < 4; i++)
        RegOverridePredefKey(g_roots[i], NULL);
    char head[64];
    snprintf(head, sizeof head, "#hr=0x%08lx", (unsigned long)hr);
    emit(head);
    for (int i = 0; i < 4; i++)
        walk(sandbox[i], g_root_names[i], L"");
    return 0;
}

/* ---------------------------------------------------------------- parent */

static unsigned g_entries, g_failed;

static void forward(const char *line)
{
    size_t len = strlen(line), off = 0;
    do {
        size_t n = len - off > 200 ? 200 : len - off;
        DbgPrint("[REGCAP]%c%.*s\n", off + n < len ? '+' : '=', (int)n, line + off);
        off += n;
    } while (off < len);
}

static void run_entry(const char *self, const char *kind, const WCHAR *path, int flags, const WCHAR *args)
{
    char cmd[2048], pathA[MAX_PATH], argsA[512];
    WideCharToMultiByte(CP_ACP, 0, path, -1, pathA, MAX_PATH, NULL, NULL);
    WideCharToMultiByte(CP_ACP, 0, args, -1, argsA, 512, NULL, NULL);
    snprintf(cmd, sizeof cmd, "\"%s\" child %s \"%s\" %d \"%s\"", self, kind, pathA, flags, argsA);
    g_entries++;
    DbgPrint("[REGCAP] begin %s\n", pathA);

    SECURITY_ATTRIBUTES sa = { sizeof sa, NULL, TRUE };
    HANDLE rd, wr;
    CreatePipe(&rd, &wr, &sa, 0);
    SetHandleInformation(rd, HANDLE_FLAG_INHERIT, 0);
    STARTUPINFOA si = { sizeof si };
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = wr;
    si.hStdError = wr;
    PROCESS_INFORMATION pi;
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi)) {
        DbgPrint("[REGCAP] end %s rc=spawn-failed gle=%lu\n", pathA, GetLastError());
        g_failed++;
        CloseHandle(rd); CloseHandle(wr);
        return;
    }
    CloseHandle(wr);

    /* Read with a deadline: a child wedged in DllRegisterServer must not stall the sweep. */
    static char buf[1 << 20];
    size_t have = 0;
    unsigned lines = 0;
    char hr[32] = "?";
    DWORD deadline = GetTickCount() + CHILD_TIMEOUT_MS;
    for (;;) {
        DWORD avail = 0, got = 0;
        if (!PeekNamedPipe(rd, NULL, 0, NULL, &avail, NULL)) break;   /* child closed it */
        if (!avail) {
            if (WaitForSingleObject(pi.hProcess, 0) == WAIT_OBJECT_0) {
                if (!PeekNamedPipe(rd, NULL, 0, NULL, &avail, NULL) || !avail) break;
            } else if (GetTickCount() > deadline) {
                TerminateProcess(pi.hProcess, 99);
                DbgPrint("[REGCAP] TIMEOUT %s\n", pathA);
                break;
            } else { Sleep(20); continue; }
        }
        if (have + avail >= sizeof buf - 1) avail = (DWORD)(sizeof buf - 1 - have);
        if (!ReadFile(rd, buf + have, avail, &got, NULL) || !got) break;
        have += got;
        buf[have] = 0;
        char *nl, *start = buf;
        while ((nl = strchr(start, '\n'))) {
            *nl = 0;
            if (!strncmp(start, "#hr=", 4)) snprintf(hr, sizeof hr, "%s", start + 4);
            else { forward(start); lines++; }
            start = nl + 1;
        }
        have = strlen(start);
        memmove(buf, start, have + 1);
    }
    WaitForSingleObject(pi.hProcess, 5000);
    DWORD rc = 0;
    GetExitCodeProcess(pi.hProcess, &rc);
    CloseHandle(pi.hProcess); CloseHandle(pi.hThread); CloseHandle(rd);
    if (rc != 0 || (LONG)strtoul(hr, NULL, 16) < 0) g_failed++;
    DbgPrint("[REGCAP] end %s rc=%lu hr=%s lines=%u\n", pathA, rc, hr, lines);
}

static void run_dll_section(const char *self, HINF inf, const WCHAR *section)
{
    INFCONTEXT ctx;
    WCHAR sysdir[MAX_PATH];
    GetSystemDirectoryW(sysdir, MAX_PATH);
    for (BOOL ok = SetupFindFirstLineW(inf, section, NULL, &ctx); ok; ok = SetupFindNextLine(&ctx, &ctx)) {
        WCHAR sub[MAX_PATH] = L"", file[MAX_PATH], args[512] = L"", path[MAX_PATH];
        INT dirid = 0, flags = 0;
        SetupGetIntField(&ctx, 1, &dirid);
        SetupGetStringFieldW(&ctx, 2, sub, MAX_PATH, NULL);
        if (!SetupGetStringFieldW(&ctx, 3, file, MAX_PATH, NULL)) continue;
        if (!SetupGetIntField(&ctx, 4, &flags)) flags = 0;
        SetupGetStringFieldW(&ctx, 6, args, 512, NULL);
        if (dirid != 11) { DbgPrint("[REGCAP] SKIP dirid %d %ls\n", dirid, file); continue; }
        wcscpy(path, sysdir);
        if (*sub) { wcscat(path, L"\\"); wcscat(path, sub); }
        wcscat(path, L"\\");
        wcscat(path, file);
        if (GetFileAttributesW(path) == INVALID_FILE_ATTRIBUTES) {
            DbgPrint("[REGCAP] ABSENT %ls\n", path);  /* not in this image: nothing to register */
            continue;
        }
        run_entry(self, "dll", path, flags, args);
    }
}

int main(int argc, char **argv)
{
    /* An unhandled exception now reaches UnhandledExceptionFilter, whose fault box
     * would leave a crashing child waiting on a dialog nobody answers. This mode is
     * inherited by every process spawned from here, which then exits with its
     * exception code instead -- as an unattended run on Windows does. */
    SetErrorMode(SetErrorMode(0) | SEM_NOGPFAULTERRORBOX);
    if (argc >= 6 && !strcmp(argv[1], "child"))
        return child(argv[2], argv[3], atoi(argv[4]), argv[5]);

    char self[MAX_PATH];
    WCHAR infpath[MAX_PATH];
    GetModuleFileNameA(NULL, self, MAX_PATH);
    GetWindowsDirectoryW(infpath, MAX_PATH);
    wcscat(infpath, L"\\inf\\syssetup.inf");
    HINF inf = SetupOpenInfFileW(infpath, NULL, INF_STYLE_WIN4, NULL);
    DbgPrint("[REGCAP] start inf=%ls %s\n", infpath, inf == INVALID_HANDLE_VALUE ? "OPEN-FAILED" : "");
    if (inf == INVALID_HANDLE_VALUE) {
        DbgPrint("[REGCAP] done entries=0 failed=1\n");
        return 1;
    }
    /* Setup's order: EarlyPhaseSetup, then RegistrationPhase2, then the typelibs. */
    run_dll_section(self, inf, L"EarlyRegisterDlls");
    run_dll_section(self, inf, L"OleControlDlls");
    INFCONTEXT ctx;
    WCHAR sysdir[MAX_PATH];
    GetSystemDirectoryW(sysdir, MAX_PATH);
    for (BOOL ok = SetupFindFirstLineW(inf, L"TypeLibraries", NULL, &ctx); ok; ok = SetupFindNextLine(&ctx, &ctx)) {
        WCHAR file[MAX_PATH], path[MAX_PATH];
        if (!SetupGetStringFieldW(&ctx, 1, file, MAX_PATH, NULL)) continue;
        wcscpy(path, sysdir); wcscat(path, L"\\"); wcscat(path, file);
        if (GetFileAttributesW(path) == INVALID_FILE_ATTRIBUTES) {
            DbgPrint("[REGCAP] ABSENT %ls\n", path);
            continue;
        }
        run_entry(self, "tlb", path, 0, L"");
    }
    SetupCloseInfFile(inf);
    DbgPrint("[REGCAP] done entries=%u failed=%u\n", g_entries, g_failed);
    return 0;
}
