/* appstorm — do the stock GUI apps and control-panel applets open, draw and close?
 *
 * For each entry: snapshot the visible top-level windows, launch it, and take every NEW visible
 * window as its own (matching by pid would miss console windows, which csrss owns, and applets,
 * which run in rundll32). Then record title/class/size, probe responsiveness with
 * SendMessageTimeout(WM_NULL), ask the host for a screenshot ("[APPSTORM] shot <name>", see
 * shot_prefix in payload.yaml), and close it: WM_CLOSE, then TerminateProcess if it will not go.
 * An entry FAILS if no window appears, a window hangs, a process crashes, an error box or ANY
 * message box pops, or the shell's taskbar dies (a folder window runs in explorer's process).
 * After the launch sweep, SCENARIOS drive an app past its first window the way a user would --
 * menu commands, dialog text, button clicks -- and check the result on disk (see scenarios[]).
 * Verdict: "[APPSTORM] done VERDICT=PASS|FAIL pass=N fail=M absent=K". */
#include <windows.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[APPSTORM] " __VA_ARGS__)
#include "../stackdump.h"

/* expect_box: the text of a message box that is the RIGHT answer in the test machine, not a
 * bug -- the app telling the truth about the environment. Any other box still fails, so an
 * allowance cannot hide a regression that changes what the app says. */
/* wait_past: the title of a progress window the app opens BEFORE its real one. rapps
 * downloads its database first; closing that dialog makes it start the download again from
 * its main window (upstream gui.cpp CheckAvailable), so judging the dialog under TCG's slow
 * TLS measured the cancel path, not the app. The app is judged on the window after it. */
typedef struct {
    const char *name; const char *exe; const char *args; const char *expect_box; const char *wait_past;
} app_t;
static const app_t apps[] = {
    { "notepad", "notepad.exe", "" },       { "calc", "calc.exe", "" },
    { "regedit", "regedit.exe", "" },       { "taskmgr", "taskmgr.exe", "" },
    { "cmd", "cmd.exe", "" },               { "mspaint", "mspaint.exe", "" },
    { "wordpad", "wordpad.exe", "" },       { "charmap", "charmap.exe", "" },
    { "winmine", "winmine.exe", "" },       { "sol", "sol.exe", "" },
    { "spider", "spider.exe", "" },         { "winver", "winver.exe", "" },
    { "clipbrd", "clipbrd.exe", "" },
    { "sndvol32", "sndvol32.exe", "", "no active mixer devices" }, /* QEMU has no audio device */
    { "magnify", "magnify.exe", "" },       { "osk", "osk.exe", "" },
    /* The livecd's eventlog service has no log files ("EventLog service reports no log
     * files!", rpc.c), so opening the Application log fails ERROR_GEN_FAILURE. */
    { "eventvwr", "eventvwr.exe", "", "attached to the system is not functioning" },
    { "devmgmt", "devmgmt.exe", "" },
    { "servman", "servman.exe", "" },       { "msconfig", "msconfig.exe", "" },
    { "dxdiag", "dxdiag.exe", "" },         { "rapps", "rapps.exe", "", NULL, "Updating Database" },
    { "mplay32", "mplay32.exe", "" },       { "sndrec32", "sndrec32.exe", "" },
    { "explorer-folder", "explorer.exe", "%WINDIR%" },
    { "mycomputer", "explorer.exe", "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}" },
    { "iexplore", "iexplore.exe", "" },
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

/* The process appstorm launched for the current entry (0 between entries). A new window
 * counts only if its process is that one, a descendant (control.exe -> rundll32), or the
 * shell (a folder window lives in explorer). Anything else started concurrently -- the
 * amd64 image's WoW64 guest ladder opens 32-bit calc and sol mid-run -- was scored as the
 * app under test, and its close timeouts failed mspaint and wordpad. */
static DWORD launch_root;
/* Every process seen descending from launch_root, accumulated across polls: appwiz's
 * rundll32 starts rapps and exits, and a parent walk from rapps then dead-ends. */
static DWORD family[64];
static int nfamily;

static int in_family(DWORD pid)
{
    for (int i = 0; i < nfamily; i++)
        if (family[i] == pid)
            return 1;
    return 0;
}

static void grow_family(void)
{
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snap == INVALID_HANDLE_VALUE)
        return;
    for (int grew = 1; grew && nfamily < 64;) {
        PROCESSENTRY32W pe = { sizeof pe };
        grew = 0;
        for (BOOL k = Process32FirstW(snap, &pe); k && nfamily < 64; k = Process32NextW(snap, &pe))
            if (in_family(pe.th32ParentProcessID) && !in_family(pe.th32ProcessID)) {
                family[nfamily++] = pe.th32ProcessID;
                grew = 1;
            }
    }
    CloseHandle(snap);
}

static int from_launch(DWORD pid)
{
    DWORD shell = 0;
    if (!launch_root)
        return 1;
    GetWindowThreadProcessId(GetShellWindow(), &shell);
    if (pid == shell)
        return 1;
    grow_family();
    return in_family(pid);
}

static int new_windows(const wset_t *before, wset_t *out)
{
    wset_t now;
    snapshot(&now);
    out->n = 0;
    for (int i = 0; i < now.n; i++) {
        DWORD pid = 0;
        GetWindowThreadProcessId(now.h[i], &pid);
        if (!contains(before, now.h[i]) && pid != GetCurrentProcessId() && !prior_pid(pid)
            && from_launch(pid))
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

static int progress_showing(const wset_t *before, const char *title)
{
    wset_t got;
    new_windows(before, &got);
    for (int i = 0; i < got.n; i++) {
        WCHAR t[128] = L"";
        char ta[128];
        InternalGetWindowText(got.h[i], t, 128);
        ascii(t, ta, sizeof ta);
        if (!strncmp(ta, title, strlen(title)))
            return 1;
    }
    return 0;
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

/* A message box, whatever its title says: user32 gives the text static the id 0xFFFF
 * (MSGBOX_IDTEXT). Title matching alone missed iexplore's untitled, EMPTY box on the Pi. */
static int msgbox_text(HWND h, char *out, size_t n)
{
    WCHAR cls[16] = L"", t[256] = L"";
    HWND txt;
    GetClassNameW(h, cls, 16);
    if (wcscmp(cls, L"#32770") || !(txt = GetDlgItem(h, 0xFFFF)))
        return 0;
    InternalGetWindowText(txt, t, 256);
    ascii(t, out, n);
    return 1;
}

/* The shell must outlive every app: a crash in shell32 code hosted by explorer (a folder
 * window, My Computer) takes the taskbar with it, and the app's own verdict never sees it. */
static HWND shell_tray;
static void shell_note(void) { shell_tray = FindWindowW(L"Shell_TrayWnd", NULL); }
static int shell_check(const char *after)
{
    DWORD t0 = GetTickCount();
    if (shell_tray && IsWindow(shell_tray))
        return 1;
    SAY("  SHELL DIED after %s: taskbar window gone\n", after);
    /* Re-arm on a respawned shell so one death does not fail every later entry. */
    while (!(shell_tray = FindWindowW(L"Shell_TrayWnd", NULL)) && GetTickCount() - t0 < 20000)
        Sleep(500);
    SAY("  shell %s\n", shell_tray ? "came back" : "did not come back");
    return 0;
}

/* The first visible top-level window of `pid` with class `cls`, polling for up to `ms`. */
typedef struct { DWORD pid; const WCHAR *cls; HWND found; } wfind_t;
static BOOL CALLBACK wfind_cb(HWND h, LPARAM lp)
{
    wfind_t *f = (wfind_t *)lp;
    DWORD pid = 0;
    WCHAR c[64] = L"";
    GetWindowThreadProcessId(h, &pid);
    GetClassNameW(h, c, 64);
    if (pid == f->pid && IsWindowVisible(h) && !wcscmp(c, f->cls)) {
        f->found = h;
        return FALSE;
    }
    return TRUE;
}
static HWND wait_window(DWORD pid, const WCHAR *cls, HWND not, DWORD ms, HANDLE proc)
{
    DWORD t0 = GetTickCount();
    do {
        wfind_t f = { pid, cls, NULL };
        EnumWindows(wfind_cb, (LPARAM)&f);
        if (f.found && f.found != not)
            return f.found;
        if (proc && WaitForSingleObject(proc, 0) == WAIT_OBJECT_0)
            return NULL;
        Sleep(250);
    } while (GetTickCount() - t0 < ms);
    return NULL;
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
/* fastfat: FatNotifyChangeDirectory raises STATUS_DELETE_PENDING (FatRaiseStatus) on a
 * directory whose disposition is set. With dead PSEH that raise bugchecked 0x1E; with real
 * SEH, FatFsdDirectoryControl catches it and the request fails with ERROR_ACCESS_DENIED.
 * The follow-up proves the volume and its locks survived the unwind. */
static int fat_raise_check(const WCHAR *windir)
{
    typedef LONG (WINAPI *set_info_t)(HANDLE, void *, void *, ULONG, int);
    set_info_t set_info = (set_info_t)GetProcAddress(GetModuleHandleA("ntdll.dll"),
                                                     "NtSetInformationFile");
    WCHAR dir[MAX_PATH], pat[MAX_PATH];
    WIN32_FIND_DATAW fd;
    ULONG_PTR iosb[2];
    BOOLEAN del = TRUE;
    char buf[512];
    OVERLAPPED ov = { 0 };
    DWORD got = 0, err, wait;
    HANDLE h, h2;
    LONG st;
    int ok;

    wsprintfW(dir, L"%s\\Temp\\fsraise%lu", windir, GetCurrentProcessId());
    if (!CreateDirectoryW(dir, NULL)) {
        SAY("fsraise FAIL mkdir gle=%lu\n", GetLastError());
        return 0;
    }
    h = CreateFileW(dir, DELETE | FILE_LIST_DIRECTORY | SYNCHRONIZE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING,
                    FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED, NULL);
    if (h == INVALID_HANDLE_VALUE) {
        SAY("fsraise FAIL open gle=%lu\n", GetLastError());
        RemoveDirectoryW(dir);
        return 0;
    }
    /* FileDispositionInformation (13): sets FCB_STATE_DELETE_ON_CLOSE right away. */
    st = set_info(h, iosb, &del, sizeof del, 13);
    ov.hEvent = CreateEventW(NULL, TRUE, FALSE, NULL);
    SetLastError(0);
    if (ReadDirectoryChangesW(h, buf, sizeof buf, FALSE, FILE_NOTIFY_CHANGE_FILE_NAME, NULL, &ov,
                              NULL)) {
        /* Queued instead of raising: the check failed, but don't leave it pending. */
        wait = WaitForSingleObject(ov.hEvent, 2000);
        err = wait == WAIT_OBJECT_0 && !GetOverlappedResult(h, &ov, &got, FALSE) ? GetLastError() : 0;
        CancelIo(h);
    } else {
        err = GetLastError();
    }
    CloseHandle(ov.hEvent);
    CloseHandle(h); /* deletes the directory */
    wsprintfW(pat, L"%s\\*", windir);
    h2 = FindFirstFileW(pat, &fd);
    if (h2 != INVALID_HANDLE_VALUE)
        FindClose(h2);
    ok = st == 0 && err == ERROR_ACCESS_DENIED && h2 != INVALID_HANDLE_VALUE &&
         GetFileAttributesW(dir) == INVALID_FILE_ATTRIBUTES;
    SAY("fsraise %s fs=FAT disp=0x%lx err=%lu followup=%d\n", ok ? "PASS" : "FAIL",
        (unsigned long)st, err, h2 != INVALID_HANDLE_VALUE);
    return ok;
}

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
    if (!lstrcmpiW(fsw, L"FAT") || !lstrcmpiW(fsw, L"FAT32"))
        return fat_raise_check(windir);
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

/* FormatMessage(FROM_SYSTEM) for a plain Win32 error. Every app's error box goes through it
 * (notepad's ShowLastError falls back to the bare number when it fails: a box reading "5"),
 * so check the lookup directly, and RtlFindMessage under it, before blaming an app. */
NTSTATUS NTAPI RtlFindMessage(PVOID, ULONG, ULONG, ULONG, PVOID *);
static int fmtmsg_check(void)
{
    WCHAR buf[256] = L"", *alloc = NULL;
    char a[256];
    PVOID mre = NULL;
    DWORD n1, e1, n2, e2;
    NTSTATUS st;
    n1 = FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, NULL,
                        ERROR_ACCESS_DENIED, 0, buf, 256, NULL);
    e1 = GetLastError();
    n2 = FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ALLOCATE_BUFFER, NULL,
                        ERROR_ACCESS_DENIED, 0, (LPWSTR)&alloc, 0, NULL);
    e2 = GetLastError();
    st = RtlFindMessage(GetModuleHandleW(L"kernel32.dll"), 11 /* RT_MESSAGETABLE */, 0,
                        ERROR_ACCESS_DENIED, &mre);
    ascii(buf, a, sizeof a);
    for (char *c = a; *c; c++)
        if (*c == '\r' || *c == '\n') *c = ' ';
    SAY("fmtmsg %s buf=%lu(gle=%lu) alloc=%lu(gle=%lu) rtlfind=0x%lx text=\"%s\"\n",
        n1 && n2 ? "PASS" : "FAIL", n1, e1, n2, e2, (unsigned long)st, a);
    LocalFree(alloc);
    return n1 && n2;
}

/* APPSTORM_ONLY=a,b build.sh compiles in a subset of apps and scenarios, for iterating on one. */
static int wanted(const char *name)
{
#ifdef APPSTORM_ONLY
    char want[512], *tok;
    int hit = 0;
    snprintf(want, sizeof want, "%s", APPSTORM_ONLY);
    for (tok = strtok(want, ","); tok; tok = strtok(NULL, ","))
        hit |= !strcmp(tok, name);
    return hit;
#else
    (void)name;
    return 1;
#endif
}

/* --- Scenarios: drive an app past its first window, the way a user would. ---------------
 * Each one launches its app, pokes it with the messages its own UI would send (menu commands,
 * dialog text, button clicks), and checks the outcome on disk and in the process table. A
 * message box from the app at any step, the app dying, or the shell dying fails it. */
typedef struct { const char *name; int (*run)(const char *name); } scenario_t;

/* The machine a process's main image was built for: the Machine field of the PE header at
 * PEB->ImageBaseAddress, read across processes, or 0 if it cannot be read. Every app is
 * launched by name, so the machine it ran as is decided by where the loader found it: an
 * x86-64 appstorm on arm64 finds sysamd64's copy only through the guest's system32 redirect,
 * and without this check a missed redirect would run the NATIVE app and score it as an x86-64
 * pass. (Not ProcessImageInformation: the kernel answers that for the current process only.) */
typedef struct {
    LONG ExitStatus; PVOID PebBaseAddress; ULONG_PTR AffinityMask; LONG BasePriority;
    ULONG_PTR UniqueProcessId, InheritedFromUniqueProcessId;
} storm_basic_info_t;
LONG NTAPI NtQueryInformationProcess(HANDLE, ULONG, PVOID, ULONG, PULONG);
static USHORT image_machine(HANDLE process)
{
    storm_basic_info_t info;
    PVOID base = NULL;
    IMAGE_DOS_HEADER dos;
    USHORT machine = 0;
    SIZE_T got;
    if (NtQueryInformationProcess(process, 0 /* ProcessBasicInformation */, &info, sizeof info, NULL) < 0)
        return 0;
    /* PEB.ImageBaseAddress follows InheritedAddressSpace..Mutant: two pointers in. */
    if (!ReadProcessMemory(process, (BYTE *)info.PebBaseAddress + 2 * sizeof(PVOID), &base, sizeof base, &got)
        || !ReadProcessMemory(process, base, &dos, sizeof dos, &got) || dos.e_magic != IMAGE_DOS_SIGNATURE
        || !ReadProcessMemory(process, (BYTE *)base + dos.e_lfanew + 4, &machine, sizeof machine, &got))
        return 0;
    return machine;
}
static USHORT own_machine;

/* Exit code of a finished process, or -1 while it runs; a crash is scored like main()'s. */
static int crashed_code(DWORD rc)
{
    return ((rc & 0xC0000000u) == 0xC0000000u && rc != 0xC000013Au) || rc == 0x80;
}

/* Any message box the app has up now: log it and say so. */
static int app_msgbox(DWORD pid, const char *name, const char *step)
{
    HWND dlg = NULL;
    char mb[256];
    while ((dlg = FindWindowExW(NULL, dlg, L"#32770", NULL))) {
        DWORD p = 0;
        GetWindowThreadProcessId(dlg, &p);
        if (p == pid && IsWindowVisible(dlg) && msgbox_text(dlg, mb, sizeof mb)) {
            SAY("  %s: message box at %s: \"%s\"\n", name, step, mb);
            SAY("shot %s-msgbox\n", name);
            Sleep(3000);
            return 1;
        }
    }
    return 0;
}

/* App Paths, as ShellExecute resolves an exe that is not on PATH. syssetup.inf's [Finalize]
 * (RunOnceEx, first boot) moves iexplore.exe from %windir% into Program Files and registers it
 * here. The x86 livecd is read-only, so the move fails there and SearchPath still finds it in
 * %windir%; the arm64 ramdisk is writable and gets the real install, so only this finds it. */
static int app_path(const char *exe, char *out)
{
    char key[MAX_PATH], raw[MAX_PATH];
    DWORD len = sizeof raw, type;
    HKEY hk;
    int ok = 0;
    wsprintfA(key, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\%s", exe);
    if (RegOpenKeyExA(HKEY_LOCAL_MACHINE, key, 0, KEY_QUERY_VALUE, &hk))
        return 0;
    if (!RegQueryValueExA(hk, NULL, NULL, &type, (BYTE *)raw, &len) && (type == REG_SZ || type == REG_EXPAND_SZ)) {
        char *p = raw;
        if (*p == '"') { p++; p[strcspn(p, "\"")] = 0; }
        ExpandEnvironmentStringsA(p, out, MAX_PATH);
        ok = GetFileAttributesA(out) != INVALID_FILE_ATTRIBUTES;
    }
    RegCloseKey(hk);
    return ok;
}

/* Where to launch `exe` from: the system directory's copy first, then the search path, then
 * App Paths. notepad, regedit, explorer and iexplore also (or only) live in %windir%, and a
 * plain SearchPath picks that copy. It is the same binary natively, but for an x86-64 appstorm
 * on arm64 the system directory is the one an x86-64 process sees redirected to sysamd64:
 * the %windir% copy is the NATIVE build, and launching it scored native apps as x86-64 runs. */
static int find_app(const char *exe, char *out)
{
    char sys[MAX_PATH];
    if (GetSystemDirectoryA(sys, MAX_PATH) && snprintf(out, MAX_PATH, "%s\\%s", sys, exe) < MAX_PATH
        && GetFileAttributesA(out) != INVALID_FILE_ATTRIBUTES)
        return 1;
    return SearchPathA(NULL, exe, NULL, MAX_PATH, out, NULL) || app_path(exe, out);
}

/* notepad: type text, File > Save As, name a file on the scratch volume, OK. The Pi crashed
 * notepad here, and the dialog is shell32 end to end (DefView lists the folder, the combo
 * autocompletes against the filesystem), so it reaches code no launch-and-close does. */
#define NP_CMD_SAVE_AS 0x104 /* base/applications/notepad/notepad_res.h */
#define DLG_CMB13 0x47c      /* explorer-style file name combo (dlgs.h) */
#define DLG_EDT1 0x480       /* old-style file name edit */
static int scn_notepad_save(const char *name)
{
    char path[MAX_PATH], tmp[MAX_PATH], cmd[MAX_PATH + 16];
    WCHAR wfile[MAX_PATH];
    STARTUPINFOA si = { sizeof si };
    PROCESS_INFORMATION pi;
    HWND main, edit, dlg, fn, ok;
    const char *why = NULL;
    DWORD rc = 0, t0;
    WIN32_FILE_ATTRIBUTE_DATA fa;
    int saved = 0;

    if (!find_app("notepad.exe", path)) {
        SAY("scenario %s ABSENT\n", name);
        return -1;
    }
    /* The scratch FAT volume when the harness attached one (C:\TEMP, what [WT] points the
     * winetests at), else %TEMP% -- which on the livecd is X:, read-only, and a save there is
     * refused ERROR_ACCESS_DENIED for reasons that have nothing to do with notepad. */
    if ((GetFileAttributesA("C:\\TEMP") & (FILE_ATTRIBUTE_DIRECTORY | 0x80000000u)) == FILE_ATTRIBUTE_DIRECTORY)
        snprintf(tmp, sizeof tmp, "C:\\TEMP\\");
    else
        GetTempPathA(MAX_PATH, tmp);
    char target[MAX_PATH];
    snprintf(target, sizeof target, "%sstorm%lu.txt", tmp, GetCurrentProcessId());
    DeleteFileA(target);
    MultiByteToWideChar(CP_ACP, 0, target, -1, wfile, MAX_PATH);
    end_prior_instances("notepad.exe");
    snprintf(cmd, sizeof cmd, "\"%s\"", path);
    if (!CreateProcessA(NULL, cmd, NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi)) {
        SAY("scenario %s FAIL spawn gle=%lu\n", name, GetLastError());
        return 0;
    }
    WaitForInputIdle(pi.hProcess, 20000);
    if (!(main = wait_window(pi.dwProcessId, L"Notepad", NULL, 30000, pi.hProcess))) {
        why = "no notepad window";
        goto out;
    }
    if (!(edit = FindWindowExW(main, NULL, L"EDIT", NULL))) {
        why = "no edit control";
        goto out;
    }
    SendMessageW(edit, WM_SETTEXT, 0, (LPARAM)L"vibeos appstorm: saved by the notepad-save scenario\r\n");
    SendMessageW(edit, EM_SETMODIFY, TRUE, 0);
    SAY("  %s: typed, sending File > Save As\n", name);
    PostMessageW(main, WM_COMMAND, NP_CMD_SAVE_AS, 0);
    if (!(dlg = wait_window(pi.dwProcessId, L"#32770", NULL, 30000, pi.hProcess))) {
        why = WaitForSingleObject(pi.hProcess, 0) == WAIT_OBJECT_0 ? "died opening Save As" : "no Save As dialog";
        goto out;
    }
    if (app_msgbox(pi.dwProcessId, name, "Save As")) {
        why = "message box instead of Save As";
        goto out;
    }
    Sleep(3000); /* let DefView populate the folder */
    SAY("shot %s-dialog\n", name);
    Sleep(3000);
    if (!(fn = GetDlgItem(dlg, DLG_CMB13)) && !(fn = GetDlgItem(dlg, DLG_EDT1))) {
        why = "Save As has no file name box";
        goto out;
    }
    SendMessageW(fn, WM_SETTEXT, 0, (LPARAM)wfile);
    ok = GetDlgItem(dlg, IDOK);
    SAY("  %s: file name %s, pressing OK\n", name, target);
    PostMessageW(dlg, WM_COMMAND, MAKEWPARAM(IDOK, BN_CLICKED), (LPARAM)ok);
    t0 = GetTickCount();
    while (GetTickCount() - t0 < 30000 && WaitForSingleObject(pi.hProcess, 0) == WAIT_TIMEOUT) {
        if (!IsWindow(dlg) && GetFileAttributesExA(target, GetFileExInfoStandard, &fa)) {
            saved = 1;
            break;
        }
        if (app_msgbox(pi.dwProcessId, name, "after OK"))
            break;
        Sleep(250);
    }
    if (WaitForSingleObject(pi.hProcess, 0) == WAIT_OBJECT_0)
        why = "notepad died saving";
    else if (!saved)
        why = IsWindow(dlg) ? "Save As never closed" : "file not on disk";
    else if (!fa.nFileSizeLow)
        why = "saved file is empty";
    else
        SAY("  %s: saved %lu bytes\n", name, fa.nFileSizeLow);
out:
    if (why && WaitForSingleObject(pi.hProcess, 0) == WAIT_TIMEOUT) {
        SAY("shot %s-fail\n", name);
        Sleep(3000);
    }
    /* Close: after a good save there is no "save changes?" prompt; after a bad one, kill. */
    if (WaitForSingleObject(pi.hProcess, 0) == WAIT_TIMEOUT) {
        if (!why)
            PostMessageW(main, WM_CLOSE, 0, 0);
        if (why || WaitForSingleObject(pi.hProcess, 10000) == WAIT_TIMEOUT) {
            if (!why) {
                why = "would not close after saving";
                dump_process(pi.dwProcessId);
            }
            TerminateProcess(pi.hProcess, 1);
            WaitForSingleObject(pi.hProcess, 5000);
        }
    }
    GetExitCodeProcess(pi.hProcess, &rc);
    if (!why && crashed_code(rc))
        why = "crashed on exit";
    if (why && crashed_code(rc))
        SAY("  %s: notepad exit=0x%lx (crash)\n", name, rc);
    if (!shell_check(name) && !why)
        why = "shell died";
    SAY("scenario %s %s%s exit=0x%lx\n", name, why ? "FAIL: " : "PASS", why ? why : "", rc);
    DeleteFileA(target);
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    return !why;
}

static const scenario_t scenarios[] = {
    { "notepad-save", scn_notepad_save },
};

int main(void)
{
    /* An unhandled exception now reaches UnhandledExceptionFilter, whose fault box
     * would leave a crashing child waiting on a dialog nobody answers. This mode is
     * inherited by every process spawned from here, which then exits with its
     * exception code instead -- as an unattended run on Windows does. */
    SetErrorMode(SetErrorMode(0) | SEM_NOGPFAULTERRORBOX);
    int pass = 0, fail = 0, absent = 0;
    WCHAR sysdir[MAX_PATH], windir[MAX_PATH];
    GetSystemDirectoryW(sysdir, MAX_PATH);
    GetWindowsDirectoryW(windir, MAX_PATH);
    DWORD t0 = GetTickCount();
    while (setup_running() && GetTickCount() - t0 < 600000)
        Sleep(2000);
    SAY("setup %s after %lus\n", setup_running() ? "STILL RUNNING, proceeding" : "done",
        (GetTickCount() - t0) / 1000);
    own_machine = image_machine(GetCurrentProcess());
    SAY("start apps=%u machine=0x%x\n", (unsigned)(sizeof apps / sizeof apps[0]), own_machine);
    fs_raise_check(windir) ? pass++ : fail++;
    fmtmsg_check() ? pass++ : fail++;
    shell_note();
    SAY("shell tray=%p\n", (void *)shell_tray);
    Sleep(5000); /* let the desktop settle before the first snapshot */
    for (unsigned k = 0; k < sizeof apps / sizeof apps[0]; k++) {
        const app_t *a = &apps[k];
        char path[MAX_PATH], cmd[1024], args[MAX_PATH];
        if (!wanted(a->name))
            continue;
        if (!find_app(a->exe, path)) {
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
        DWORD app_t0 = GetTickCount();
        STARTUPINFOA si = { sizeof si };
        PROCESS_INFORMATION pi;
        if (!CreateProcessA(NULL, cmd, NULL, NULL, FALSE, 0, NULL, NULL, &si, &pi)) {
            SAY("app %s FAIL spawn gle=%lu\n", a->name, GetLastError());
            fail++;
            continue;
        }
        /* The same machine as appstorm itself, on every host: see image_machine. */
        USHORT machine = image_machine(pi.hProcess);
        int same_machine = machine && machine == own_machine;
        if (!same_machine)
            SAY("  launched as machine 0x%x, appstorm is 0x%x\n", machine, own_machine);
        launch_root = pi.dwProcessId;
        family[0] = launch_root;
        nfamily = 1;
        WaitForInputIdle(pi.hProcess, 20000);
        /* Wait for a window, then let it finish drawing. */
        DWORD t0 = GetTickCount();
        /* No early out on process exit: control.exe exits as soon as it has started the
         * applet's rundll32, so a gone launcher says nothing about the window. */
        /* Wait for a REAL window, not just the first new one: an applet's rundll32 shows its
         * hidden 166x24 host window at -32000,-32000 long before the dialog when the machine is
         * busy, and stopping there failed desk/sysdm/intl on amd64 with the dialog on its way. */
        /* grow_family on every poll: with no window yet nothing else does, and appwiz's
         * rundll32 exits within seconds, orphaning the rapps it started. */
        while (!has_real_window(&before, &got) && GetTickCount() - t0 < 30000) {
            grow_family();
            Sleep(250);
        }
        Sleep(3000);
        /* After the settle: rapps titles its dialog only once it has been created. */
        if (a->wait_past && progress_showing(&before, a->wait_past)) {
            DWORD w0 = GetTickCount();
            while (progress_showing(&before, a->wait_past) && GetTickCount() - w0 < 300000)
                Sleep(1000);
            SAY("  waited %lums past \"%s\"%s\n", GetTickCount() - w0, a->wait_past,
                progress_showing(&before, a->wait_past) ? " (STILL SHOWING)" : "");
            while (!has_real_window(&before, &got) && GetTickCount() - w0 < 330000)
                Sleep(250);
            Sleep(3000);
        }
        /* No real window: dump every launched process still alive. appwiz's rapps outlives
         * the rundll32 that started it, and nothing else here says what it is doing. */
        if (!has_real_window(&before, &got)) {
            grow_family();
            for (int f = 0; f < nfamily; f++) {
                HANDLE h = OpenProcess(SYNCHRONIZE, FALSE, family[f]);
                if (h && WaitForSingleObject(h, 0) == WAIT_TIMEOUT) {
                    SAY("  no window: pid=%lu alive, threads:\n", family[f]);
                    dump_process(family[f]);
                }
                if (h) CloseHandle(h);
            }
        }
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
            char mb[200];
            if (!strcmp(ca, "#32770") && (strstr(ta, "rror") || strstr(ta, "ailed") || strstr(ta, "xception")))
                errbox = 1;
            /* No stock app greets the user with a message box, whatever its title. */
            if (msgbox_text(got.h[i], mb, sizeof mb)) {
                int expected = a->expect_box && strstr(mb, a->expect_box);
                errbox |= !expected;
                SAY("  win %d is a message box%s: \"%s\"\n", i, expected ? " (expected here)" : "", mb);
            }
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
        for (int i = 0; i < got.n; i++)
            if (!PostMessageW(got.h[i], WM_CLOSE, 0, 0))
                SAY("  PostMessage(WM_CLOSE) to %p failed, error %lu\n", (void *)got.h[i], GetLastError());
        DWORD rc = 0;
        const char *close = "OK";
        for (int j = 0; j < npids; j++) {
            HANDLE h = j ? OpenProcess(SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION,
                                       FALSE, pids[j]) : pi.hProcess;
            if (!h) continue;
            if (WaitForSingleObject(h, 10000) == WAIT_TIMEOUT) {
                SAY("  close timeout pid=%lu, threads:\n", pids[j]);
                /* Every top-level window the process owns NOW: a modal box the close raised
                 * (a "save changes?" prompt) was created after the snapshot above. */
                {
                    wset_t now;
                    snapshot(&now);
                    for (int k = 0; k < now.n; k++) {
                        DWORD wp = 0;
                        WCHAR wc[64] = L"", wt[128] = L"";
                        char ac[64], at[128];
                        GetWindowThreadProcessId(now.h[k], &wp);
                        if (wp != pids[j]) continue;
                        GetClassNameW(now.h[k], wc, 64);
                        InternalGetWindowText(now.h[k], wt, 128);
                        ascii(wc, ac, sizeof ac);
                        ascii(wt, at, sizeof at);
                        SAY("  owns %p class=%s title=\"%s\" visible=%d\n", (void *)now.h[k], ac, at,
                            IsWindowVisible(now.h[k]));
                    }
                }
                /* Which half failed: a window still alive whose thread answers a sent message
                 * never got (or ignored) the posted WM_CLOSE; a destroyed window means the
                 * app closed and its exit path is what hung. */
                for (int i = 0; i < got.n; i++) {
                    DWORD wpid = 0;
                    DWORD_PTR r = 0;
                    GetWindowThreadProcessId(got.h[i], &wpid);
                    if (wpid != pids[j]) continue;
                    SAY("  window %p alive=%d visible=%d answers=%d\n", (void *)got.h[i],
                        IsWindow(got.h[i]), IsWindowVisible(got.h[i]),
                        IsWindow(got.h[i]) && SendMessageTimeoutW(got.h[i], WM_NULL, 0, 0,
                                                                  SMTO_ABORTIFHUNG, 2000, &r) != 0);
                    /* A SENT close runs the window procedure synchronously: if that destroys
                     * the window, the posted one was lost; if not, the close itself fails. */
                    if (IsWindow(got.h[i])) {
                        LRESULT sent = SendMessageTimeoutW(got.h[i], WM_CLOSE, 0, 0,
                                                           SMTO_ABORTIFHUNG, 3000, &r);
                        SAY("  sent WM_CLOSE: returned=%d window_after=%d\n", sent != 0,
                            IsWindow(got.h[i]));
                    }
                }
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
        int shell = shell_check(a->name);
        /* An app that ignores WM_CLOSE is a failure, not a pass: every app on every arch
         * timing out on close (a lost PostMessage) scored PASS here until this counted. */
        int good = ok && !hung && !errbox && !crashed && shell && same_machine && !strcmp(close, "OK");
        /* ms: launch to verdict. The serial log has no clock, and a run that hit the drain
         * deadline could not say which app ate the time. */
        /* availMB: free physical memory once the app has gone, so a sweep that runs the
         * machine out (an x86-64 sweep under FEX on a 1 GB arm64 VM did) shows the trend --
         * steady means pressure, falling app after app means each one leaks. */
        MEMORYSTATUSEX ms = { sizeof ms };
        GlobalMemoryStatusEx(&ms);
        SAY("app %s %s windows=%d first=\"%s\" hung=%d errbox=%d close=%s exit=0x%lx shell=%s machine=0x%x ms=%lu availMB=%lu\n",
            a->name, good ? "PASS" : "FAIL", got.n, first, hung, errbox, close, rc, shell ? "OK" : "DIED",
            machine, GetTickCount() - app_t0, (unsigned long)(ms.ullAvailPhys >> 20));
        good ? pass++ : fail++;
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
        launch_root = 0;
        Sleep(1500);
    }
    for (unsigned k = 0; k < sizeof scenarios / sizeof scenarios[0]; k++) {
        int r;
        if (!wanted(scenarios[k].name))
            continue;
        r = scenarios[k].run(scenarios[k].name);
        r < 0 ? absent++ : r ? pass++ : fail++;
        Sleep(1500);
    }
    SAY("done VERDICT=%s pass=%d fail=%d absent=%d\n", fail ? "FAIL" : "PASS", pass, fail, absent);
    return 0;
}
