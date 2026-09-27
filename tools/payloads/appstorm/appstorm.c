/* appstorm — do the stock GUI apps and control-panel applets open, draw and close?
 *
 * For each entry: snapshot the visible top-level windows, launch it, and take every NEW visible
 * window as its own (matching by pid would miss console windows, which csrss owns, and applets,
 * which run in rundll32). Then record title/class/size, probe responsiveness with
 * SendMessageTimeout(WM_NULL), ask the host for a screenshot ("[APPSTORM] shot <name>", see
 * shot_prefix in payload.yaml), and close it: WM_CLOSE, then TerminateProcess if it will not go.
 * An entry FAILS if no window appears, a window hangs, a process crashes, or an error box pops.
 * Verdict: "[APPSTORM] done VERDICT=PASS|FAIL pass=N fail=M absent=K". */
#include <windows.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[APPSTORM] " __VA_ARGS__)
#include "../stackdump.h"

typedef struct { const char *name; const char *exe; const char *args; } app_t;
static const app_t apps[] = {
    { "notepad", "notepad.exe", "" },       { "calc", "calc.exe", "" },
    { "regedit", "regedit.exe", "" },       { "taskmgr", "taskmgr.exe", "" },
    { "cmd", "cmd.exe", "" },               { "mspaint", "mspaint.exe", "" },
    { "wordpad", "wordpad.exe", "" },       { "charmap", "charmap.exe", "" },
    { "winmine", "winmine.exe", "" },       { "sol", "sol.exe", "" },
    { "spider", "spider.exe", "" },         { "winver", "winver.exe", "" },
    { "clipbrd", "clipbrd.exe", "" },       { "sndvol32", "sndvol32.exe", "" },
    { "magnify", "magnify.exe", "" },       { "osk", "osk.exe", "" },
    { "eventvwr", "eventvwr.exe", "" },     { "devmgmt", "devmgmt.exe", "" },
    { "servman", "servman.exe", "" },       { "msconfig", "msconfig.exe", "" },
    { "dxdiag", "dxdiag.exe", "" },         { "rapps", "rapps.exe", "" },
    { "mplay32", "mplay32.exe", "" },       { "sndrec32", "sndrec32.exe", "" },
    { "explorer-folder", "explorer.exe", "%WINDIR%" },
    { "control", "control.exe", "" },
    { "cpl-desk", "control.exe", "desk.cpl" },        { "cpl-sysdm", "control.exe", "sysdm.cpl" },
    { "cpl-timedate", "control.exe", "timedate.cpl" }, { "cpl-intl", "control.exe", "intl.cpl" },
    { "cpl-main", "control.exe", "main.cpl" },        { "cpl-mmsys", "control.exe", "mmsys.cpl" },
    { "cpl-appwiz", "control.exe", "appwiz.cpl" },    { "cpl-ncpa", "control.exe", "ncpa.cpl" },
    { "cpl-access", "control.exe", "access.cpl" },    { "cpl-powercfg", "control.exe", "powercfg.cpl" },
    { "cpl-inetcpl", "control.exe", "inetcpl.cpl" },  { "cpl-joy", "control.exe", "joy.cpl" },
};

#define MAXW 64
typedef struct { HWND h[MAXW]; int n; } wset_t;

static BOOL CALLBACK collect(HWND h, LPARAM lp)
{
    wset_t *s = (wset_t *)lp;
    if (IsWindowVisible(h) && s->n < MAXW) s->h[s->n++] = h;
    return TRUE;
}
static int contains(const wset_t *s, HWND h)
{
    for (int i = 0; i < s->n; i++) if (s->h[i] == h) return 1;
    return 0;
}
static void snapshot(wset_t *s) { s->n = 0; EnumWindows(collect, (LPARAM)s); }

/* New visible windows that are not ours and not the shell's. */
/* Processes alive before the launch. Their new windows are not the app's -- on amd64 the
 * background WoW64 user32_winetest pops console windows at any moment, and one was scored as
 * dxdiag's window. explorer.exe is the exception: a folder window opens INSIDE the shell. */
static DWORD prior_pids[512];
static int n_prior;

static void snapshot_pids(void)
{
    PROCESSENTRY32W pe = { sizeof pe };
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    n_prior = 0;
    if (snap == INVALID_HANDLE_VALUE)
        return;
    if (Process32FirstW(snap, &pe))
        do {
            if (lstrcmpiW(pe.szExeFile, L"explorer.exe") && n_prior < 512)
                prior_pids[n_prior++] = pe.th32ProcessID;
        } while (Process32NextW(snap, &pe));
    CloseHandle(snap);
}

/* Single-instance apps (osk) exit at once when a copy is already up, and winlogon's stock-app
 * check leaves one running. Whether it was still alive decided osk's verdict, so every run was
 * a coin flip. Give each app a fresh start instead. explorer.exe is the shell: never touch it. */
static void end_prior_instances(const char *exe)
{
    PROCESSENTRY32W pe = { sizeof pe };
    WCHAR wexe[MAX_PATH];
    DWORD parent = 0;
    HANDLE snap;
    if (!lstrcmpiA(exe, "explorer.exe"))
        return;
    MultiByteToWideChar(CP_ACP, 0, exe, -1, wexe, MAX_PATH);
    snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snap == INVALID_HANDLE_VALUE)
        return;
    /* Never our own launcher: arm64 starts appstorm from a cmd.exe, and cmd is on the list. */
    if (Process32FirstW(snap, &pe))
        do {
            if (pe.th32ProcessID == GetCurrentProcessId())
                parent = pe.th32ParentProcessID;
        } while (Process32NextW(snap, &pe));
    pe.dwSize = sizeof pe;
    if (Process32FirstW(snap, &pe))
        do {
            HANDLE h;
            if (lstrcmpiW(pe.szExeFile, wexe) || pe.th32ProcessID == parent
                || pe.th32ProcessID == GetCurrentProcessId())
                continue;
            h = OpenProcess(PROCESS_TERMINATE | SYNCHRONIZE, FALSE, pe.th32ProcessID);
            if (!h)
                continue;
            SAY("  ended prior %s pid=%lu\n", exe, pe.th32ProcessID);
            TerminateProcess(h, 0);
            WaitForSingleObject(h, 5000);
            CloseHandle(h);
        } while (Process32NextW(snap, &pe));
    CloseHandle(snap);
}

static int prior_pid(DWORD pid)
{
    for (int i = 0; i < n_prior; i++)
        if (prior_pids[i] == pid)
            return 1;
    return 0;
}

static int new_windows(const wset_t *before, wset_t *out)
{
    wset_t now;
    snapshot(&now);
    out->n = 0;
    for (int i = 0; i < now.n; i++) {
        DWORD pid = 0;
        GetWindowThreadProcessId(now.h[i], &pid);
        if (!contains(before, now.h[i]) && pid != GetCurrentProcessId() && !prior_pid(pid))
            out->h[out->n++] = now.h[i];
    }
    return out->n;
}

static int is_real(HWND h)
{
    RECT r;
    return GetWindowRect(h, &r) && r.right - r.left >= 50 && r.bottom - r.top >= 30 && r.left > -30000;
}

static int has_real_window(const wset_t *before, wset_t *got)
{
    new_windows(before, got);
    for (int i = 0; i < got->n; i++)
        if (is_real(got->h[i]))
            return 1;
    return 0;
}

static void ascii(const WCHAR *w, char *o, size_t n)
{
    size_t i = 0;
    for (; w[i] && i + 1 < n; i++) o[i] = (w[i] >= 32 && w[i] < 127) ? (char)w[i] : '?';
    o[i] = 0;
}

static void dump_hung_thread(HWND h)
{
    DWORD pid = 0, tid = GetWindowThreadProcessId(h, &pid);
    dump_thread(pid, tid);
}

/* A process that ignores WM_CLOSE is stuck somewhere no single window's thread tells us:
 * the UI thread may be fine while a worker holds up ExitProcess. Dump every thread. */
static void dump_process(DWORD pid)
{
    THREADENTRY32 te = { sizeof te };
    HANDLE s = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if (s == INVALID_HANDLE_VALUE) return;
    for (BOOL k = Thread32First(s, &te); k; k = Thread32Next(s, &te))
        if (te.th32OwnerProcessID == pid) dump_thread(pid, te.th32ThreadID);
    CloseHandle(s);
}

/* Boot-time `setup -mini` installs devices and registers components for minutes after the
 * desktop appears, with its own "Please wait" dialog. Apps launched meanwhile compete with it
 * (a window takes past any sane timeout to answer) and its dialog sits among theirs, so wait. */
static int setup_running(void)
{
    PROCESSENTRY32W pe = { sizeof pe };
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    int found = 0;
    if (snap == INVALID_HANDLE_VALUE)
        return 0;
    if (Process32FirstW(snap, &pe))
        do {
            if (!lstrcmpiW(pe.szExeFile, L"setup.exe"))
                found = 1;
        } while (!found && Process32NextW(snap, &pe));
    CloseHandle(snap);
    return found;
}

/* A cdfs raise, end to end. An empty name part (";1") makes CdInitializeEnumeration raise
 * STATUS_INVALID_PARAMETER inside CdQueryDirectory, which holds the directory's FCB. With a
 * real handler (ntkrnl_sys::seh) the query fails ERROR_INVALID_PARAMETER and the finally has
 * released the FCB; kernel32 then closes the directory, and cdfs cleanup takes that FCB
 * exclusively, so a leaked lock hangs right here. Under DUMMY_PSEH the raise was a 0x1E.
 * Only meaningful where the Windows directory is on cdfs (the livecd). */
static int fs_raise_check(const WCHAR *windir)
{
    WCHAR vol[4] = { windir[0], L':', L'\\', 0 }, fsw[16] = L"", pat[MAX_PATH];
    char fs[16];
    WIN32_FIND_DATAW fd;
    HANDLE h, h2;
    DWORD err;
    int ok;

    GetVolumeInformationW(vol, NULL, 0, NULL, NULL, NULL, fsw, 16);
    ascii(fsw, fs, sizeof fs);
    if (lstrcmpiW(fsw, L"CDFS")) {
        SAY("fsraise SKIP fs=%s\n", fs);
        return 1;
    }
    wsprintfW(pat, L"%s\\;1", windir);
    h = FindFirstFileW(pat, &fd);
    err = GetLastError();
    if (h != INVALID_HANDLE_VALUE)
        FindClose(h);
    wsprintfW(pat, L"%s\\*", windir);
    h2 = FindFirstFileW(pat, &fd);
    if (h2 != INVALID_HANDLE_VALUE)
        FindClose(h2);
    ok = h == INVALID_HANDLE_VALUE && err == ERROR_INVALID_PARAMETER && h2 != INVALID_HANDLE_VALUE;
    SAY("fsraise %s err=%lu followup=%d\n", ok ? "PASS" : "FAIL", err, h2 != INVALID_HANDLE_VALUE);
    return ok;
}

int main(void)
{
    int pass = 0, fail = 0, absent = 0;
    WCHAR sysdir[MAX_PATH], windir[MAX_PATH];
    GetSystemDirectoryW(sysdir, MAX_PATH);
    GetWindowsDirectoryW(windir, MAX_PATH);
    DWORD t0 = GetTickCount();
    while (setup_running() && GetTickCount() - t0 < 600000)
        Sleep(2000);
    SAY("setup %s after %lus\n", setup_running() ? "STILL RUNNING, proceeding" : "done",
        (GetTickCount() - t0) / 1000);
    SAY("start apps=%u\n", (unsigned)(sizeof apps / sizeof apps[0]));
    fs_raise_check(windir) ? pass++ : fail++;
    Sleep(5000); /* let the desktop settle before the first snapshot */
    for (unsigned k = 0; k < sizeof apps / sizeof apps[0]; k++) {
        const app_t *a = &apps[k];
        char path[MAX_PATH], cmd[1024], args[MAX_PATH];
#ifdef APPSTORM_ONLY
        {   /* build-time subset, for iterating on one failure: APPSTORM_ONLY=a,b build.sh */
            char want[512], *tok;
            int hit = 0;
            snprintf(want, sizeof want, "%s", APPSTORM_ONLY);
            for (tok = strtok(want, ","); tok; tok = strtok(NULL, ","))
                hit |= !strcmp(tok, a->name);
            if (!hit) continue;
        }
#endif
        if (!SearchPathA(NULL, a->exe, NULL, MAX_PATH, path, NULL)) {
            SAY("app %s ABSENT (%s)\n", a->name, a->exe);
            absent++;
            continue;
        }
#ifdef APPSTORM_PAGEHEAP
        /* Debug page heap for the app (IFEO GlobalFlag = FLG_HEAP_PAGE_ALLOCS): a heap overrun
         * then faults AT the corrupting store instead of in a later RtlFreeHeap. */
        {
            char key[300];
            HKEY hk;
            DWORD flag = 0x02000000;
            wsprintfA(key, "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options\\%s", a->exe);
            if (!RegCreateKeyExA(HKEY_LOCAL_MACHINE, key, 0, NULL, 0, KEY_SET_VALUE, NULL, &hk, NULL)) {
                RegSetValueExA(hk, "GlobalFlag", 0, REG_DWORD, (const BYTE *)&flag, sizeof flag);
                RegCloseKey(hk);
                SAY("pageheap on for %s\n", a->exe);
            }
        }
#endif
        ExpandEnvironmentStringsA(a->args, args, MAX_PATH);
        snprintf(cmd, sizeof cmd, "\"%s\" %s", path, args);
        wset_t before, got;
        end_prior_instances(a->exe);
        snapshot(&before);
        snapshot_pids();
        STARTUPINFOA si = { sizeof si };
        PROCESS_INFORMATION pi;
        if (!CreateProcessA(NULL, cmd, NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi)) {
            SAY("app %s FAIL spawn gle=%lu\n", a->name, GetLastError());
            fail++;
            continue;
        }
        WaitForInputIdle(pi.hProcess, 20000);
        /* Wait for a window, then let it finish drawing. */
        DWORD t0 = GetTickCount();
        /* No early out on process exit: control.exe exits as soon as it has started the
         * applet's rundll32, so a gone launcher says nothing about the window. */
        /* Wait for a REAL window, not just the first new one: an applet's rundll32 shows its
         * hidden 166x24 host window at -32000,-32000 long before the dialog when the machine is
         * busy, and stopping there failed desk/sysdm/intl on amd64 with the dialog on its way. */
        while (!has_real_window(&before, &got) && GetTickCount() - t0 < 30000)
            Sleep(250);
        Sleep(3000);
        new_windows(&before, &got);
        /* At least one real window: an applet host also owns tiny helper windows. */
        int ok = 0, hung = 0, errbox = 0;
        char first[200] = "";
        for (int i = 0; i < got.n; i++) {
            WCHAR t[128] = L"", c[64] = L"";
            char ta[128], ca[64];
            RECT r = { 0 };
            DWORD_PTR res;
            /* A window that populates synchronously (Control Panel loads every applet on its UI
             * thread) can take well past 5s under TCG, which is BUSY, not hung. Keep asking for up
             * to 60s and report the wait; only a window silent that long is HUNG. */
            DWORD asked = GetTickCount();
            int resp = SendMessageTimeoutW(got.h[i], WM_NULL, 0, 0, SMTO_ABORTIFHUNG, 5000, &res) != 0;
            if (!resp) {
                SAY("  win %d slow to answer, sampling:\n", i);
                dump_hung_thread(got.h[i]);
                while (!resp && GetTickCount() - asked < 60000)
                    resp = SendMessageTimeoutW(got.h[i], WM_NULL, 0, 0, SMTO_BLOCK, 5000, &res) != 0;
                SAY("  win %d %s after %lums\n", i, resp ? "answered" : "still silent", GetTickCount() - asked);
            }
            /* Read these once it has answered: a window mid-load has no title yet, and
             * InternalGetWindowText reads it without sending the window a message. */
            InternalGetWindowText(got.h[i], t, 128);
            GetClassNameW(got.h[i], c, 64);
            GetWindowRect(got.h[i], &r);
            ascii(t, ta, sizeof ta);
            ascii(c, ca, sizeof ca);
            int small = (r.right - r.left) < 50 || (r.bottom - r.top) < 30;
            /* An error box is a dialog whose title says so, from any process. */
            if (!strcmp(ca, "#32770") && (strstr(ta, "rror") || strstr(ta, "ailed") || strstr(ta, "xception")))
                errbox = 1;
            hung |= !resp;
            if (!small) ok = 1;
            SAY("  win %d title=\"%s\" class=%s rect=%ldx%ld@%ld,%ld resp=%s\n", i, ta, ca,
                r.right - r.left, r.bottom - r.top, r.left, r.top, resp ? "OK" : "HUNG");
            if (!i) snprintf(first, sizeof first, "%s", ta);
        }
        SAY("shot %s\n", a->name);
        Sleep(4000);
        /* Close every new window, then wait for EVERY owning process: an applet runs in a
         * rundll32 that control.exe merely started, so control.exe exiting says nothing. Only
         * a process still alive after the wait is killed, and only that counts as a TIMEOUT. */
        DWORD pids[MAXW + 1], shellpid = 0;
        int npids = 0;
        GetWindowThreadProcessId(GetShellWindow(), &shellpid);
        pids[npids++] = pi.dwProcessId;
        for (int i = 0; i < got.n; i++) {
            DWORD pid = 0;
            int seen = 0;
            WCHAR cls[64] = L"";
            /* Never the shell (a folder window lives in explorer) and never a console's owner
             * (csrss): those must survive, and killing csrss takes the system down. */
            GetClassNameW(got.h[i], cls, 64);
            if (!wcscmp(cls, L"ConsoleWindowClass")) continue;
            GetWindowThreadProcessId(got.h[i], &pid);
            if (pid == shellpid) continue;
            for (int j = 0; j < npids; j++) seen |= pids[j] == pid;
            if (pid && !seen) pids[npids++] = pid;
        }
        for (int i = 0; i < got.n; i++) PostMessageW(got.h[i], WM_CLOSE, 0, 0);
        DWORD rc = 0;
        const char *close = "OK";
        for (int j = 0; j < npids; j++) {
            HANDLE h = j ? OpenProcess(SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION,
                                       FALSE, pids[j]) : pi.hProcess;
            if (!h) continue;
            if (WaitForSingleObject(h, 10000) == WAIT_TIMEOUT) {
                SAY("  close timeout pid=%lu, threads:\n", pids[j]);
                dump_process(pids[j]);
                TerminateProcess(h, 1);
                close = "TIMEOUT";
            }
            if (!j) GetExitCodeProcess(h, &rc);
            if (j) CloseHandle(h);
        }
        /* 0x80 as well: a user-mode exception reaching second chance is terminated by csrss's
         * exception port with TerminateProcess(client, 0x80), not with the exception code
         * (ke/amd64/except.rs). dxdiag died that way and was scored PASS. */
        int crashed = ((rc & 0xC0000000u) == 0xC0000000u && rc != 0xC000013Au) || rc == 0x80;
        int good = ok && !hung && !errbox && !crashed;
        SAY("app %s %s windows=%d first=\"%s\" hung=%d errbox=%d close=%s exit=0x%lx\n", a->name,
            good ? "PASS" : "FAIL", got.n, first, hung, errbox, close, rc);
        good ? pass++ : fail++;
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        Sleep(1500);
    }
    SAY("done VERDICT=%s pass=%d fail=%d absent=%d\n", fail ? "FAIL" : "PASS", pass, fail, absent);
    return 0;
}
