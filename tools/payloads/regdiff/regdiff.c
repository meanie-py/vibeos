/*
 * regdiff — what does a DLL's DllRegisterServer / DllUnregisterServer WRITE, and does a
 * replacement write the same thing?
 *
 * Built for retiring shell32.cpp: its registration used to run ATL's STATIC registrar
 * (statreg.h, compiled into shell32) and now asks atl100.dll's registrar to parse the same
 * .rgs scripts. Two parsers of one script language can disagree quietly — a key left behind
 * on unregister, a value typed differently — and nothing at boot would notice, because the
 * media's hives are prebuilt. So this measures it.
 *
 * The parent runs this binary twice as a child, once per DLL:
 *
 *   regdiff.exe old  <path of the baseline DLL>
 *   regdiff.exe new  <path of the DLL under test>
 *
 * Each child loads its DLL, then points HKEY_CLASSES_ROOT / HKEY_LOCAL_MACHINE /
 * HKEY_CURRENT_USER / HKEY_USERS at empty sandbox keys with RegOverridePredefKey (in-process
 * only; the real registry is untouched), calls DllRegisterServer, dumps the sandbox, calls
 * DllUnregisterServer into the same sandbox, and dumps again. A dump is one line per key and
 * per value, sorted, with this DLL's own path replaced by %MODULE% (the baseline DLL is staged
 * under a different file name). The parent then diffs old against new, phase by phase.
 *
 * The children are separate processes so two copies of one DLL never share a process.
 * GUI subsystem + classic msvcrt, like ../procstorm: the explorer startup walk launches it.
 */
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <objbase.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define MAX_PRINTED_DIFFS 300u
#define CHILD_TIMEOUT_MS  (5u * 60u * 1000u)

static const HKEY g_roots[4] = { HKEY_CLASSES_ROOT, HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, HKEY_USERS };
static const char *g_root_names[4] = { "HKCR", "HKLM", "HKCU", "HKU" };

/* ---- growable, sortable line list ---------------------------------------------------- */

typedef struct { char **v; size_t n, cap; } lines_t;

static void lines_add(lines_t *l, const char *s)
{
    if (l->n == l->cap) {
        l->cap = l->cap ? l->cap * 2 : 256;
        l->v = (char **)realloc(l->v, l->cap * sizeof(char *));
    }
    l->v[l->n++] = _strdup(s);
}

static int cmp_str(const void *a, const void *b)
{
    return strcmp(*(char *const *)a, *(char *const *)b);
}

/* ---- dump ------------------------------------------------------------------------------ */

static WCHAR g_module[MAX_PATH], g_moddir[MAX_PATH];

/* UTF-8 of `w`, with every case-insensitive occurrence of this DLL's path replaced by %MODULE%
 * and of its directory by %MODULEDIR%. The directory needs its own rule: its CASE depends on
 * how the loader came to name the module (X:\reactos\System32 for the baseline, loaded by the
 * path this payload built; system32 for shell32.dll, already loaded by name), and values such
 * as a typelib's HELPDIR hold the directory without the file name. */
static void to_utf8_normalized(const WCHAR *w, size_t cch, char *out, size_t outsz)
{
    static WCHAR tmp[8192];
    size_t mlen = wcslen(g_module), dlen = wcslen(g_moddir), o = 0, i = 0;
    while (i < cch && o + 16 < sizeof(tmp) / sizeof(tmp[0])) {
        if (mlen && i + mlen <= cch && _wcsnicmp(w + i, g_module, mlen) == 0) {
            memcpy(tmp + o, L"%MODULE%", 8 * sizeof(WCHAR));
            o += 8;
            i += mlen;
        } else if (dlen && i + dlen <= cch && _wcsnicmp(w + i, g_moddir, dlen) == 0) {
            memcpy(tmp + o, L"%MODULEDIR%", 11 * sizeof(WCHAR));
            o += 11;
            i += dlen;
        } else {
            WCHAR c = w[i++];
            tmp[o++] = c ? c : L'|'; /* MULTI_SZ separators */
        }
    }
    tmp[o] = 0;
    int n = WideCharToMultiByte(CP_UTF8, 0, tmp, (int)o, out, (int)outsz - 1, NULL, NULL);
    out[n > 0 ? n : 0] = 0;
}

static void format_value(DWORD type, const BYTE *data, DWORD cb, char *out, size_t outsz)
{
    switch (type) {
    case REG_SZ:
    case REG_EXPAND_SZ:
    case REG_MULTI_SZ: {
        size_t cch = cb / sizeof(WCHAR);
        const WCHAR *w = (const WCHAR *)data;
        /* Trailing NULs are storage, not content — two writers may count them differently,
         * and that difference is not a registration difference. The count is still shown. */
        size_t n = cch;
        while (n && w[n - 1] == 0)
            n--;
        char body[6000];
        to_utf8_normalized(w, n, body, sizeof(body));
        snprintf(out, outsz, "\"%s\" (+%u nul)", body, (unsigned)(cch - n));
        break;
    }
    case REG_DWORD:
        snprintf(out, outsz, "0x%08lx", cb >= 4 ? *(const DWORD *)data : 0ul);
        break;
    default: {
        size_t o = 0;
        o += snprintf(out + o, outsz - o, "len=%lu:", (unsigned long)cb);
        for (DWORD i = 0; i < cb && i < 64 && o + 4 < outsz; i++)
            o += snprintf(out + o, outsz - o, "%02x", data[i]);
        break;
    }
    }
}

static void walk(HKEY key, const char *root, const WCHAR *path, lines_t *out)
{
    char pathu8[4096], line[8192];
    to_utf8_normalized(path, wcslen(path), pathu8, sizeof(pathu8));
    snprintf(line, sizeof(line), "%s\\%s", root, pathu8);
    lines_add(out, line);

    DWORD nsub = 0, maxsub = 0, nval = 0, maxname = 0, maxdata = 0;
    if (RegQueryInfoKeyW(key, NULL, NULL, NULL, &nsub, &maxsub, NULL, &nval, &maxname, &maxdata,
                         NULL, NULL) != ERROR_SUCCESS)
        return;

    WCHAR *name = (WCHAR *)malloc((maxname + 2) * sizeof(WCHAR));
    BYTE *data = (BYTE *)malloc(maxdata + 8);
    for (DWORD i = 0; i < nval; i++) {
        DWORD cchName = maxname + 1, cb = maxdata + 4, type = 0;
        if (RegEnumValueW(key, i, name, &cchName, NULL, &type, data, &cb) != ERROR_SUCCESS)
            continue;
        char nameu8[1024], val[7000];
        to_utf8_normalized(name, cchName, nameu8, sizeof(nameu8));
        format_value(type, data, cb, val, sizeof(val));
        snprintf(line, sizeof(line), "%s\\%s\t%s\ttype=%lu\t%s", root, pathu8,
                 cchName ? nameu8 : "@", (unsigned long)type, val);
        lines_add(out, line);
    }
    free(data);
    free(name);

    WCHAR *sub = (WCHAR *)malloc((maxsub + 2) * sizeof(WCHAR));
    for (DWORD i = 0; i < nsub; i++) {
        DWORD cch = maxsub + 1;
        if (RegEnumKeyExW(key, i, sub, &cch, NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
            continue;
        HKEY child;
        if (RegOpenKeyExW(key, sub, 0, KEY_READ, &child) != ERROR_SUCCESS)
            continue;
        size_t plen = wcslen(path);
        WCHAR *cpath = (WCHAR *)malloc((plen + cch + 2) * sizeof(WCHAR));
        wcscpy(cpath, path);
        if (plen)
            wcscat(cpath, L"\\");
        wcscat(cpath, sub);
        walk(child, root, cpath, out);
        free(cpath);
        RegCloseKey(child);
    }
    free(sub);
}

/* ---- child ------------------------------------------------------------------------------ */

typedef BOOL (WINAPI *expand_fn)(HANDLE, LPCWSTR, LPWSTR, DWORD);
typedef HRESULT (WINAPI *folder_fn)(HWND, int, HANDLE, DWORD, LPWSTR);

/* Every folder SHELL_RegisterShellFolders registers, asked for (without CSIDL_FLAG_CREATE,
 * so nothing is written) with the tokens it uses: -1 (Default User) and NULL for the user
 * set, NULL for the common set; plus the environment expansion behind them. Only failures
 * are printed. `where` says which registry it ran against. */
static void probe_folders(const char *where, folder_fn folder)
{
    static const int user_ids[] = { 0x02, 0x05, 0x06, 0x1a, 0x07, 0x08, 0x09, 0x0b, 0x0d, 0x0e,
                                    0x10, 0x13, 0x15, 0x1b, 0x1c, 0x20, 0x21, 0x22, 0x27, 0x14,
                                    0x30 };
    static const int common_ids[] = { 0x16, 0x17, 0x18, 0x19, 0x1f, 0x23, 0x2d, 0x2e, 0x2f,
                                      0x35, 0x36, 0x37 };
    WCHAR buf[MAX_PATH];
    HMODULE ue = LoadLibraryA("userenv.dll");
    expand_fn expand = ue ? (expand_fn)GetProcAddress(ue, "ExpandEnvironmentStringsForUserW") : NULL;
    if (!expand || !folder) {
        DbgPrint("[REGDIFF] probe %s FAIL: no userenv/SHGetFolderPathW\n", where);
        return;
    }
    for (int t = 0; t < 2; t++) {
        HANDLE tok = t ? (HANDLE)(LONG_PTR)-1 : NULL;
        SetLastError(0);
        BOOL ok = expand(tok, L"%USERPROFILE%", buf, MAX_PATH);
        if (!ok)
            DbgPrint("[REGDIFF] probe %s ExpandEnvironmentStringsForUserW(token=%d) gle=%lu\n",
                     where, t ? -1 : 0, GetLastError());
        for (size_t k = 0; k < sizeof(user_ids) / sizeof(user_ids[0]); k++) {
            HRESULT hr = folder(NULL, user_ids[k], tok, 0, buf);
            if (FAILED(hr))
                DbgPrint("[REGDIFF] probe %s user csidl=0x%02x token=%d hr=0x%08lx\n", where,
                         user_ids[k], t ? -1 : 0, (unsigned long)hr);
        }
    }
    for (size_t k = 0; k < sizeof(common_ids) / sizeof(common_ids[0]); k++) {
        HRESULT hr = folder(NULL, common_ids[k], NULL, 0, buf);
        if (FAILED(hr))
            DbgPrint("[REGDIFF] probe %s common csidl=0x%02x hr=0x%08lx\n", where, common_ids[k],
                     (unsigned long)hr);
    }
    DbgPrint("[REGDIFF] probe %s done\n", where);
}


/* Copies `src`'s values, and with `subkeys` its whole subtree, into `dst`. */
static LONG copy_tree(HKEY src, HKEY dst, BOOL subkeys)
{
    DWORD nsub = 0, maxsub = 0, nval = 0, maxname = 0, maxdata = 0;
    LONG err = RegQueryInfoKeyW(src, NULL, NULL, NULL, &nsub, &maxsub, NULL, &nval, &maxname,
                                &maxdata, NULL, NULL);
    if (err != ERROR_SUCCESS)
        return err;
    WCHAR *name = (WCHAR *)malloc((maxname + maxsub + 2) * sizeof(WCHAR));
    BYTE *data = (BYTE *)malloc(maxdata + 8);
    for (DWORD i = 0; i < nval && err == ERROR_SUCCESS; i++) {
        DWORD cchName = maxname + 1, cb = maxdata + 4, type = 0;
        err = RegEnumValueW(src, i, name, &cchName, NULL, &type, data, &cb);
        if (err == ERROR_SUCCESS)
            err = RegSetValueExW(dst, name, 0, type, data, cb);
    }
    for (DWORD i = 0; subkeys && i < nsub && err == ERROR_SUCCESS; i++) {
        DWORD cch = maxname + maxsub + 1;
        HKEY s, d;
        err = RegEnumKeyExW(src, i, name, &cch, NULL, NULL, NULL, NULL);
        if (err == ERROR_SUCCESS)
            err = RegOpenKeyExW(src, name, 0, KEY_READ, &s);
        if (err == ERROR_SUCCESS) {
            err = RegCreateKeyExW(dst, name, 0, NULL, 0, KEY_ALL_ACCESS, NULL, &d, NULL);
            if (err == ERROR_SUCCESS) {
                err = copy_tree(s, d, TRUE);
                RegCloseKey(d);
            }
            RegCloseKey(s);
        }
    }
    free(data);
    free(name);
    return err;
}

/* The sandbox must still hold what the registrar READS. SHELL_RegisterShellFolders resolves
 * the profile folders through userenv, which reads HKLM\...\ProfileList; against an empty
 * HKLM that is ERROR_FILE_NOT_FOUND, and DllRegisterServer returned 0x80070002 here on every
 * arch, for the C++ shell32 and the Rust alike -- the harness's failure, not shell32's. The
 * environment expansion behind it (userenv's CreateEnvironmentBlock) also reads the Session
 * Manager environment and CurrentVersion's ProgramFilesDir/CommonFilesDir. Those two are
 * copied VALUES ONLY: their subkeys (Explorer\Shell Folders among them) are what
 * registration writes, and seeding them would hide its writes from the dump. Both children
 * get the same seed, so it cancels out of the diff. Fails loud: an unseeded run would diff
 * as SAME while measuring nothing past the first profile lookup. */
static const struct { const WCHAR *path; BOOL subkeys; } g_seeds[] = {
    { L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\ProfileList", TRUE },
    { L"SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment", FALSE },
    { L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion", FALSE },
};

static BOOL seed_sandbox(HKEY sandbox_hklm, const char *mode)
{
    for (size_t i = 0; i < sizeof(g_seeds) / sizeof(g_seeds[0]); i++) {
        HKEY src, dst;
        LONG err = RegOpenKeyExW(HKEY_LOCAL_MACHINE, g_seeds[i].path, 0, KEY_READ, &src);
        if (err == ERROR_SUCCESS) {
            err = RegCreateKeyExW(sandbox_hklm, g_seeds[i].path, 0, NULL, 0, KEY_ALL_ACCESS,
                                  NULL, &dst, NULL);
            if (err == ERROR_SUCCESS) {
                err = copy_tree(src, dst, g_seeds[i].subkeys);
                RegCloseKey(dst);
            }
            RegCloseKey(src);
        }
        if (err != ERROR_SUCCESS) {
            char u8[256];
            WideCharToMultiByte(CP_UTF8, 0, g_seeds[i].path, -1, u8, sizeof(u8), NULL, NULL);
            DbgPrint("[REGDIFF] FAIL %s: cannot seed the sandbox's HKLM\\%s = %ld\n", mode, u8, err);
            return FALSE;
        }
    }
    return TRUE;
}

/* Fails loud: if an override does not take, DllRegisterServer writes the REAL registry and
 * the dump comes back empty — which would diff as SAME. That is how this harness first met
 * advapi32's zero-extended HKEY_* constants on amd64. */
static BOOL override_all(HKEY sandbox[4])
{
    for (int i = 0; i < 4; i++) {
        /* RegOverridePredefKey takes ownership of the handle it is given (it closes it on the
         * next override), so hand it a fresh one each time. */
        HKEY dup;
        LONG err = RegOpenKeyExW(sandbox[i], NULL, 0, KEY_ALL_ACCESS, &dup);
        if (err == ERROR_SUCCESS)
            err = RegOverridePredefKey(g_roots[i], dup);
        if (err != ERROR_SUCCESS) {
            DbgPrint("[REGDIFF] FAIL RegOverridePredefKey(%s) = %ld\n", g_root_names[i], err);
            return FALSE;
        }
    }
    return TRUE;
}

static void restore_all(void)
{
    for (int i = 0; i < 4; i++)
        RegOverridePredefKey(g_roots[i], NULL);
}

/* The children write their dumps to stdout, a pipe the parent reads: the livecd's system
 * drive is read-only, so there is nowhere to put a file. */
static void emit(const char *line)
{
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    WriteFile(out, line, (DWORD)strlen(line), &n, NULL);
    WriteFile(out, "\n", 1, &n, NULL);
}

static void dump_to(HKEY sandbox[4], const char *phase, HRESULT hr)
{
    char head[64];
    lines_t l = { 0 };
    for (int i = 0; i < 4; i++)
        walk(sandbox[i], g_root_names[i], L"", &l);
    qsort(l.v, l.n, sizeof(char *), cmp_str);
    snprintf(head, sizeof(head), "#phase=%s", phase);
    emit(head);
    snprintf(head, sizeof(head), "#hr=0x%08lx", (unsigned long)hr);
    emit(head);
    for (size_t i = 0; i < l.n; i++)
        emit(l.v[i]);
}

typedef HRESULT (WINAPI *reg_fn)(void);

static int child(const char *mode, const char *dllA)
{
    WCHAR dll[MAX_PATH], keypath[256];
    MultiByteToWideChar(CP_ACP, 0, dllA, -1, dll, MAX_PATH);
    SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOOPENFILEERRORBOX);
    CoInitialize(NULL);

    /* Load BEFORE overriding, so the DLL's own initialisation sees the real registry. */
    HMODULE m = LoadLibraryW(dll);
    if (!m) {
        DbgPrint("[REGDIFF] FAIL %s: LoadLibrary(%s) gle=%lu\n", mode, dllA, GetLastError());
        return 2;
    }
    GetModuleFileNameW(m, g_module, MAX_PATH);
    wcscpy(g_moddir, g_module);
    WCHAR *slash = wcsrchr(g_moddir, L'\\');
    if (slash)
        *slash = 0;
    reg_fn doreg = (reg_fn)GetProcAddress(m, "DllRegisterServer");
    reg_fn dounreg = (reg_fn)GetProcAddress(m, "DllUnregisterServer");
    if (!doreg || !dounreg) {
        DbgPrint("[REGDIFF] FAIL %s: missing Dll(Un)RegisterServer export\n", mode);
        return 3;
    }

    HKEY sandbox[4];
    for (int i = 0; i < 4; i++) {
        /* narrow snprintf + convert: llvm-mingw's swprintf is the UCRT one, absent here */
        char keyA[256];
        snprintf(keyA, sizeof(keyA), "Software\\vibeos-regdiff\\%s\\%s", mode, g_root_names[i]);
        MultiByteToWideChar(CP_ACP, 0, keyA, -1, keypath, 256);
        if (RegCreateKeyExW(HKEY_CURRENT_USER, keypath, 0, NULL, 0, KEY_ALL_ACCESS, NULL,
                            &sandbox[i], NULL) != ERROR_SUCCESS) {
            DbgPrint("[REGDIFF] FAIL %s: cannot create sandbox %ls\n", mode, keypath);
            return 4;
        }
    }

    if (!seed_sandbox(sandbox[1], mode))
        return 6;
    if (!override_all(sandbox))
        return 5;
    char where[32];
    snprintf(where, sizeof(where), "%s-sandbox", mode);
    probe_folders(where, (folder_fn)GetProcAddress(m, "SHGetFolderPathW"));
    HRESULT hr = doreg();
    restore_all();
    dump_to(sandbox, "reg", hr);

    if (!override_all(sandbox))
        return 5;
    HRESULT hr2 = dounreg();
    restore_all();
    dump_to(sandbox, "unreg", hr2);

    DbgPrint("[REGDIFF] %s: DllRegisterServer=0x%08lx DllUnregisterServer=0x%08lx\n", mode,
             (unsigned long)hr, (unsigned long)hr2);
    return 0;
}

/* ---- parent ----------------------------------------------------------------------------- */

/* One child's output, split by its "#phase=" markers. Each phase starts with "#hr=...". */
typedef struct { lines_t reg, unreg; } dumps_t;

static void split_output(char *text, dumps_t *d)
{
    lines_t *cur = NULL;
    for (char *line = strtok(text, "\n"); line; line = strtok(NULL, "\n")) {
        if (!strcmp(line, "#phase=reg"))
            cur = &d->reg;
        else if (!strcmp(line, "#phase=unreg"))
            cur = &d->unreg;
        else if (cur)
            lines_add(cur, line);
    }
}

/* Both inputs are sorted; prints the symmetric difference and returns its size. */
static unsigned diff_phase(const char *phase, lines_t *a, lines_t *b)
{
    if (a->n == 0 || b->n == 0) {
        DbgPrint("[REGDIFF] FAIL %s: a child produced no dump (old %u, new %u lines)\n", phase,
                 (unsigned)a->n, (unsigned)b->n);
        return 0xFFFFFFFFu;
    }
    /* line 0 is "#hr=..."; compare it on its own, then the sorted body. */
    unsigned diffs = 0, printed = 0;
    if (strcmp(a->v[0], b->v[0]) != 0) {
        DbgPrint("[REGDIFF] %s: result differs: old %s, new %s\n", phase, a->v[0], b->v[0]);
        diffs++;
    }
    size_t i = 1, j = 1;
    while (i < a->n || j < b->n) {
        int c = i >= a->n ? 1 : j >= b->n ? -1 : strcmp(a->v[i], b->v[j]);
        if (c == 0) {
            i++, j++;
            continue;
        }
        const char *s = c < 0 ? a->v[i++] : b->v[j++];
        diffs++;
        if (printed++ < MAX_PRINTED_DIFFS)
            DbgPrint("[REGDIFF] %s %c %.400s\n", phase, c < 0 ? '-' : '+', s);
    }
    DbgPrint("[REGDIFF] %s: old %s %u lines, new %s %u lines, %u differ\n", phase, a->v[0],
             (unsigned)(a->n - 1), b->v[0], (unsigned)(b->n - 1), diffs);
    return diffs;
}

static int run_child(const char *self, const char *mode, const char *dll, dumps_t *d)
{
    char cmd[2 * MAX_PATH + 32];
    snprintf(cmd, sizeof(cmd), "\"%s\" %s \"%s\"", self, mode, dll);
    SECURITY_ATTRIBUTES sa = { sizeof(sa), NULL, TRUE };
    HANDLE rd, wr;
    if (!CreatePipe(&rd, &wr, &sa, 0)) {
        DbgPrint("[REGDIFF] FAIL CreatePipe gle=%lu\n", GetLastError());
        return -1;
    }
    SetHandleInformation(rd, HANDLE_FLAG_INHERIT, 0);
    STARTUPINFOA si = { sizeof(si) };
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = wr;
    si.hStdError = wr;
    si.hStdInput = NULL;
    PROCESS_INFORMATION pi;
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi)) {
        DbgPrint("[REGDIFF] FAIL cannot spawn %s child gle=%lu\n", mode, GetLastError());
        return -1;
    }
    CloseHandle(wr); /* so ReadFile sees EOF when the child exits */

    size_t cap = 1 << 16, len = 0;
    char *text = (char *)malloc(cap);
    for (;;) {
        DWORD got = 0;
        if (len + 4096 + 1 > cap)
            text = (char *)realloc(text, cap *= 2);
        if (!ReadFile(rd, text + len, 4096, &got, NULL) || got == 0)
            break;
        len += got;
    }
    text[len] = 0;
    CloseHandle(rd);

    DWORD w = WaitForSingleObject(pi.hProcess, CHILD_TIMEOUT_MS);
    DWORD code = 0xFFFFFFFF;
    GetExitCodeProcess(pi.hProcess, &code);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    if (w != WAIT_OBJECT_0) {
        DbgPrint("[REGDIFF] FAIL %s child timed out\n", mode);
        return -1;
    }
    if (code != 0)
        DbgPrint("[REGDIFF] FAIL %s child exit=0x%08lx\n", mode, (unsigned long)code);
    split_output(text, d);
    free(text);
    return (int)code;
}

/* SHELL_RegisterShellFolders registers the Default User's folders, which SHGetFolderPath
 * spells as the token (HANDLE)-1. Asked against the REAL registry (no sandbox), so a
 * failure here is the system's, not this harness's. */
int main(int argc, char **argv)
{
    /* An unhandled exception now reaches UnhandledExceptionFilter, whose fault box
     * would leave a crashing child waiting on a dialog nobody answers. This mode is
     * inherited by every process spawned from here, which then exits with its
     * exception code instead -- as an unattended run on Windows does. */
    SetErrorMode(SetErrorMode(0) | SEM_NOGPFAULTERRORBOX);
    if (argc == 3)
        return child(argv[1], argv[2]);

    char self[MAX_PATH], sysdir[MAX_PATH], olddll[MAX_PATH], newdll[MAX_PATH];
    GetModuleFileNameA(NULL, self, MAX_PATH);
    GetSystemDirectoryA(sysdir, MAX_PATH);
    snprintf(olddll, sizeof(olddll), "%s\\regdiff-base.dll", sysdir);
    snprintf(newdll, sizeof(newdll), "%s\\shell32.dll", sysdir);
    DbgPrint("[REGDIFF] start: old=%s new=%s\n", olddll, newdll);
    HMODULE sh = LoadLibraryA("shell32.dll");
    probe_folders("real", sh ? (folder_fn)GetProcAddress(sh, "SHGetFolderPathW") : NULL);

    dumps_t o = { 0 }, n = { 0 };
    int ro = run_child(self, "old", olddll, &o);
    int rn = run_child(self, "new", newdll, &n);
    if (ro != 0 || rn != 0) {
        DbgPrint("[REGDIFF] done VERDICT=ERROR\n");
        return 1;
    }
    unsigned dr = diff_phase("reg", &o.reg, &n.reg);
    unsigned du = diff_phase("unreg", &o.unreg, &n.unreg);
    DbgPrint("[REGDIFF] done VERDICT=%s reg_diffs=%u unreg_diffs=%u\n",
             dr == 0 && du == 0 ? "SAME" : "DIFFERENT", dr, du);
    return 0;
}
