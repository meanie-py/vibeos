/*
 * shelltail — does shell32's former C/C++ glue behave the same now that it is Rust?
 *
 * On 2026-09-24 shell32's last five non-Rust files moved to Rust:
 *   - three ATL CWindowImpl<> classes (CStubWindow32, CMSGlobalFolderOptionsStub,
 *     CChangeNotifyBroker), now on shell32-rust's atlwin substrate;
 *   - the ShellMessageBoxA variadic entry point, now a va-slots receiver;
 *   - the StringCchPrintfW trampoline, now strsafe_rust::StringCchPrintfW!.
 * No boot reaches most of that: nothing in the tree registers for old-style change
 * notification (the broker), and nothing opens Folder Options or a property sheet. So this
 * measures it, the way ../regdiff measured the registrar: the parent runs this binary twice
 * as a child, once against a BASELINE shell32 (the C/C++ glue, staged as
 * system32\shelltail-base.dll) and once against this tree's shell32.dll, and diffs what
 * they observe. Each child calls only its own DLL's exports, and reports a class's instance
 * as SELF when it is that DLL, since the two are loaded under different names.
 *
 * The parent ALSO checks absolute expectations on the new DLL's report, so "identical and
 * both broken" cannot pass.
 *
 *   shelltail.exe old|new <dll> <check>   (child: one check)
 *   shelltail.exe                   (parent)
 * Per check: "[SHELLTAIL] check <c>: old=.. new=.. -> SAME|FIXED|DIFFERENT|BROKEN"; FIXED means the
 * baseline crashed or hung on it and the new DLL passes. Verdict: "[SHELLTAIL] done VERDICT=...".
 */
#include <windows.h>
#include <shlobj.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

#define CHILD_TIMEOUT_MS (60u * 1000u)

static HMODULE g_dll;

static void emit(const char *fmt, ...)
{
    char line[1024];
    va_list ap;
    DWORD n;
    va_start(ap, fmt);
    vsnprintf(line, sizeof(line), fmt, ap);
    va_end(ap);
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    WriteFile(out, line, (DWORD)strlen(line), &n, NULL);
    WriteFile(out, "\n", 1, &n, NULL);
}

static void pump(DWORD ms)
{
    DWORD end = GetTickCount() + ms;
    MSG msg;
    while ((LONG)(end - GetTickCount()) > 0) {
        while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(10);
    }
}

/* A registered class, as the fields ATL's DECLARE_WND_CLASS_EX + Register decide. */
static void report_class(const char *tag, const WCHAR *cls)
{
    WNDCLASSEXW wc = { sizeof(wc) };
    if (!GetClassInfoExW(g_dll, cls, &wc)) {
        emit("%s.class=ABSENT gle=%lu", tag, GetLastError());
        return;
    }
    emit("%s.class.style=0x%lx", tag, (unsigned long)wc.style);
    emit("%s.class.cbClsExtra=%d cbWndExtra=%d", tag, wc.cbClsExtra, wc.cbWndExtra);
    emit("%s.class.hbrBackground=%lu", tag, (unsigned long)(ULONG_PTR)wc.hbrBackground);
    emit("%s.class.hCursor=%s", tag,
         wc.hCursor == LoadCursorW(NULL, (LPCWSTR)IDC_ARROW) ? "IDC_ARROW" : "OTHER");
    emit("%s.class.hIcon=%s hIconSm=%s", tag, wc.hIcon ? "SET" : "NULL", wc.hIconSm ? "SET" : "NULL");
    emit("%s.class.hInstance=%s", tag, wc.hInstance == (HINSTANCE)g_dll ? "SELF" : "OTHER");
    emit("%s.class.menu=%s", tag, wc.lpszMenuName ? "SET" : "NULL");
}

static void report_window(const char *tag, HWND h)
{
    WCHAR cls[64] = { 0 };
    GetClassNameW(h, cls, 64);
    emit("%s.wnd.style=0x%08lx exstyle=0x%08lx", tag, (unsigned long)GetWindowLongW(h, GWL_STYLE),
         (unsigned long)GetWindowLongW(h, GWL_EXSTYLE));
    emit("%s.wnd.visible=%d parent=%s", tag, IsWindowVisible(h) ? 1 : 0, GetParent(h) ? "SET" : "NULL");
}

/* ---- 1. ShellMessageBoxA: varargs received and formatted ---------------------------- */

static char g_msgtext[512];

/* The message boxes these checks raise are closed by a WATCHER THREAD rather than a timer in
 * their modal loop: on arm64 the timer route stalled for the baseline and the new DLL alike,
 * so it measured the harness, not shell32. The watcher polls for this process's box, runs the
 * check's inspector while the box (and whatever owns it) is up, then dismisses it. */
typedef void (*inspector_fn)(HWND dlg);
static inspector_fn g_inspect;
static DWORD g_main_tid;

static BOOL CALLBACK find_proc_msgbox(HWND h, LPARAM lp)
{
    WCHAR cls[16];
    GetClassNameW(h, cls, 16);
    if (lstrcmpW(cls, L"#32770") == 0) {
        *(HWND *)lp = h;
        return FALSE;
    }
    return TRUE;
}

static DWORD WINAPI box_watcher(LPVOID unused)
{
    (void)unused;
    for (int i = 0; i < 400; i++) { /* 20 s */
        HWND dlg = NULL;
        /* The box belongs to the thread that called into shell32. No visibility filter: that
         * is what missed it on i386 and arm64. */
        EnumThreadWindows(g_main_tid, find_proc_msgbox, (LPARAM)&dlg);
        if (dlg) {
            Sleep(200); /* let the box finish initialising its text */
            if (g_inspect)
                g_inspect(dlg);
            PostMessageW(dlg, WM_COMMAND, IDOK, 0);
            return 0;
        }
        Sleep(50);
    }
    emit("watcher=NO_BOX_APPEARED");
    return 1;
}

static HANDLE start_watcher(inspector_fn fn)
{
    g_inspect = fn;
    g_main_tid = GetCurrentThreadId();
    return CreateThread(NULL, 0, box_watcher, NULL, 0, NULL);
}

static void msgbox_inspect(HWND dlg)
{
    GetDlgItemTextA(dlg, 0xFFFF, g_msgtext, sizeof(g_msgtext));
}

typedef int (__cdecl *PSHELLMESSAGEBOXA)(HINSTANCE, HWND, LPCSTR, LPCSTR, UINT, ...);

static void check_messagebox(void)
{
    PSHELLMESSAGEBOXA pfn = (PSHELLMESSAGEBOXA)GetProcAddress(g_dll, "ShellMessageBoxA");
    if (!pfn) {
        emit("msgbox=NOEXPORT");
        return;
    }
    /* Five inserts: on arm64 three arrive in x5-x7 and two on the stack; on win64 all are on
     * the stack; on i386 everything is. %2 and %4 are ints, so a mis-staged slot shows. */
    HANDLE w = start_watcher(msgbox_inspect);
    int r = pfn(NULL, NULL, "A=%1!s! B=%2!d! C=%3!s! D=%4!x! E=%5!s!", "shelltail", MB_OK,
                "alpha", 42, "gamma", 0xbeef, "epsilon");
    WaitForSingleObject(w, 5000);
    CloseHandle(w);
    emit("msgbox.ret=%d", r);
    emit("msgbox.text=%s", g_msgtext);
}

/* ---- 2. Old-style change notification: the broker ---------------------------------- */

#define WM_SHELLTAIL_NOTIFY (WM_APP + 7)
static int g_notified, g_last_event;

static LRESULT CALLBACK client_proc(HWND h, UINT m, WPARAM w, LPARAM l)
{
    if (m == WM_SHELLTAIL_NOTIFY) {
        g_notified++;
        g_last_event = (int)l;
        return 0;
    }
    return DefWindowProcW(h, m, w, l);
}

static int count_thread_windows_of(const WCHAR *cls)
{
    int n = 0;
    HWND h = NULL;
    while ((h = FindWindowExW(HWND_MESSAGE, h, cls, NULL)) != NULL)
        if (GetWindowThreadProcessId(h, NULL) == GetCurrentThreadId())
            n++;
    h = NULL;
    while ((h = FindWindowExW(NULL, h, cls, NULL)) != NULL)
        if (GetWindowThreadProcessId(h, NULL) == GetCurrentThreadId())
            n++;
    return n;
}

typedef ULONG (WINAPI *PREG)(HWND, int, LONG, UINT, int, const SHChangeNotifyEntry *);
typedef BOOL (WINAPI *PDEREG)(ULONG);
typedef void (WINAPI *PNOTIFY)(LONG, UINT, LPCVOID, LPCVOID);

static void check_broker(void)
{
    PREG reg = (PREG)GetProcAddress(g_dll, "SHChangeNotifyRegister");
    PDEREG dereg = (PDEREG)GetProcAddress(g_dll, "SHChangeNotifyDeregister");
    PNOTIFY notify = (PNOTIFY)GetProcAddress(g_dll, "SHChangeNotify");
    if (!reg || !dereg || !notify) {
        emit("broker=NOEXPORT");
        return;
    }
    WNDCLASSW wc = { 0 };
    wc.lpfnWndProc = client_proc;
    wc.hInstance = GetModuleHandleW(NULL);
    wc.lpszClassName = L"ShellTailClient";
    RegisterClassW(&wc);
    HWND client = CreateWindowW(L"ShellTailClient", L"", 0, 0, 0, 0, 0, NULL, NULL, wc.hInstance, NULL);

    int workers_before = count_thread_windows_of(L"WorkerW");
    SHChangeNotifyEntry e = { NULL, TRUE };
    /* No SHCNRF_NewDelivery: the old delivery path, which goes through the broker. */
    ULONG id = reg(client, SHCNRF_ShellLevel, SHCNE_ALLEVENTS, WM_SHELLTAIL_NOTIFY, 1, &e);
    int workers_registered = count_thread_windows_of(L"WorkerW");
    emit("broker.registered=%s", id ? "YES" : "NO");
    emit("broker.workerw_added=%d", workers_registered - workers_before);

    notify(SHCNE_MKDIR, SHCNF_PATHW | SHCNF_FLUSH, L"X:\\shelltail-probe", NULL);
    pump(1500);
    emit("broker.delivered=%d", g_notified);
    emit("broker.event=0x%x", g_last_event);

    emit("broker.deregistered=%d", id ? dereg(id) : -1);
    pump(500);
    emit("broker.workerw_after_dereg=%d", count_thread_windows_of(L"WorkerW") - workers_before);
    DestroyWindow(client);
}

/* ---- 3. CStubWindow32: a property sheet's owner ------------------------------------ */

/* SHOpenPropSheetW with no keys and no data object: shell32 creates the StubWindow32 owner on
 * THIS thread, finds no pages, shows its "cannot show properties" box, and destroys the stub
 * when that box closes. So the watcher can inspect the live stub while the box is up.
 * (SHObjectProperties would reach the same owner, but it opens the sheet on another thread and
 * hangs in ole32 marshalling today -- a separate problem, not this one.) */
static HWND g_stub;

static void stub_inspect(HWND dlg)
{
    HWND stub = FindWindowW(L"StubWindow32", NULL);
    emit("stub.found=%s", stub ? "YES" : "NO");
    if (stub) {
        g_stub = stub;
        emit("stub.same_thread=%d", GetWindowThreadProcessId(stub, NULL) == g_main_tid);
        emit("stub.StubType=%lu", (unsigned long)(ULONG_PTR)GetPropW(stub, L"StubType"));
        emit("stub.msgbox_owner_is_stub=%d", GetWindow(dlg, GW_OWNER) == stub);
        report_window("stub", stub);
        report_class("stub", L"StubWindow32");
    }
}

typedef int (WINAPI *POPENPROPSHEET)(LPCWSTR, HKEY *, UINT, const CLSID *, IDataObject *,
                                     IShellBrowser *, LPCWSTR);

static void check_stubwindow(void)
{
    POPENPROPSHEET open = (POPENPROPSHEET)GetProcAddress(g_dll, "SHOpenPropSheetW");
    if (!open) {
        emit("stub=NOEXPORT");
        return;
    }
    HANDLE w = start_watcher(stub_inspect);
    int r = open(L"shelltail", NULL, 0, NULL, NULL, NULL, NULL);
    WaitForSingleObject(w, 5000);
    CloseHandle(w);
    emit("stub.SHOpenPropSheetW=%d", r);
    emit("stub.destroyed_after=%d", g_stub && !IsWindow(g_stub) ? 1 : 0);
    emit("stub.class_left_registered=%d", FindWindowW(L"StubWindow32", NULL) ? 1 : 0);
}

static HWND wait_for(const WCHAR *cls, DWORD ms)
{
    DWORD end = GetTickCount() + ms;
    HWND h;
    while (!(h = FindWindowW(cls, NULL)) && (LONG)(end - GetTickCount()) > 0)
        pump(50);
    return h;
}

static void close_owned_sheet(HWND owner)
{
    HWND popup = GetLastActivePopup(owner);
    if (popup && popup != owner)
        PostMessageW(popup, WM_COMMAND, IDCANCEL, 0);
}

/* ---- 4. CMSGlobalFolderOptionsStub: Folder Options' owner ------------------------ */

typedef void (WINAPI *PRUNDLL)(HWND, HINSTANCE, LPCWSTR, int);

static DWORD WINAPI folder_options_thread(LPVOID p)
{
    ((PRUNDLL)p)(NULL, NULL, L"0", SW_SHOWNORMAL);
    return 0;
}

static void check_folder_options(void)
{
    PRUNDLL run = (PRUNDLL)GetProcAddress(g_dll, "Options_RunDLLW");
    if (!run) {
        emit("fopt=NOEXPORT");
        return;
    }
    HANDLE th = CreateThread(NULL, 0, folder_options_thread, (LPVOID)run, 0, NULL);
    HWND stub = wait_for(L"MSGlobalFolderOptionsStub", 10000);
    emit("fopt.found=%s", stub ? "YES" : "NO");
    if (stub) {
        report_window("fopt", stub);
        report_class("fopt", L"MSGlobalFolderOptionsStub");
        close_owned_sheet(stub);
    }
    DWORD w = WaitForSingleObject(th, 10000);
    emit("fopt.thread_exited=%d", w == WAIT_OBJECT_0);
    emit("fopt.stub_gone=%d", FindWindowW(L"MSGlobalFolderOptionsStub", NULL) ? 0 : 1);
    CloseHandle(th);
}

/* ---- 5. Reg-folder items: strsafe_rust::StringCchPrintfW! in FormatGUIDKey ---------- */

/* The printf macro that replaced the StringCchPrintfW trampoline is not exported anywhere, but
 * FormatGUIDKey builds every registry path a reg-folder item's name and icon come from
 * ("CLSID\\%s", "CLSID\\%s\\DefaultIcon"), and SHGetFileInfoW on a "::{clsid}" path reaches
 * both through public exports. A broken format shows up as an empty name or icon location. */
typedef DWORD_PTR (WINAPI *PSHGETFILEINFOW)(LPCWSTR, DWORD, SHFILEINFOW *, UINT, UINT);

/* Wide to narrow, with this DLL's own path replaced by %MODULE%: the baseline is loaded as
 * shelltail-base.dll and the new one as shell32.dll, so the path itself must not count. */
static void w2a(const WCHAR *w, char *a, int n)
{
    WCHAR self[MAX_PATH];
    if (!WideCharToMultiByte(CP_ACP, 0, w, -1, a, n, NULL, NULL)) {
        a[0] = 0;
        return;
    }
    if (GetModuleFileNameW(g_dll, self, MAX_PATH) && lstrcmpiW(w, self) == 0)
        lstrcpynA(a, "%MODULE%", n);
}

static void regitem(PSHGETFILEINFOW gfi, const char *tag, const WCHAR *path)
{
    SHFILEINFOW sfi;
    char a[MAX_PATH * 2];
    ZeroMemory(&sfi, sizeof(sfi));
    DWORD_PTR r = gfi(path, 0, &sfi, sizeof(sfi), SHGFI_DISPLAYNAME | SHGFI_TYPENAME);
    w2a(sfi.szDisplayName, a, sizeof(a));
    emit("regitem.%s.name.ret=%d name=%s", tag, r != 0, a);
    w2a(sfi.szTypeName, a, sizeof(a));
    emit("regitem.%s.type=%s", tag, a);
    ZeroMemory(&sfi, sizeof(sfi));
    r = gfi(path, 0, &sfi, sizeof(sfi), SHGFI_ICONLOCATION);
    w2a(sfi.szDisplayName, a, sizeof(a));
    emit("regitem.%s.icon.ret=%d set=%d location=%s index=%d", tag, r != 0, sfi.szDisplayName[0] != 0,
         a, sfi.iIcon);
}

static void check_regitem(void)
{
    PSHGETFILEINFOW gfi = (PSHGETFILEINFOW)GetProcAddress(g_dll, "SHGetFileInfoW");
    if (!gfi) {
        emit("regitem=NOEXPORT");
        return;
    }
    regitem(gfi, "mycomputer", L"::{20D04FE0-3AEA-1069-A2D8-08002B30309D}");
    regitem(gfi, "recyclebin", L"::{645FF040-5081-101B-9F08-00AA002F954E}");
}

static const struct { const char *name; void (*fn)(void); } g_checks[] = {
    { "msgbox", check_messagebox },
    { "broker", check_broker },
    { "stub", check_stubwindow },
    { "fopt", check_folder_options },
    { "regitem", check_regitem },
};
#define N_CHECKS (sizeof(g_checks) / sizeof(g_checks[0]))

/* One check per child, so a crash or hang in one (the baseline shell32 panics on the
 * property-sheet path) is recorded against that check instead of ending the comparison. */
static int child(const char *mode, const char *dll, const char *check)
{
    g_dll = LoadLibraryA(dll);
    if (!g_dll) {
        DbgPrint("[SHELLTAIL] FAIL %s: LoadLibrary(%s) gle=%lu\n", mode, dll, GetLastError());
        return 2;
    }
    for (unsigned i = 0; i < N_CHECKS; i++) {
        if (strcmp(g_checks[i].name, check) == 0) {
            DbgPrint("[SHELLTAIL] %s: %s begin\n", mode, check);
            g_checks[i].fn();
            DbgPrint("[SHELLTAIL] %s: %s end\n", mode, check);
            return 0;
        }
    }
    return 3;
}

/* ---------------------------------- parent ----------------------------------------- */

typedef struct { char **v; unsigned n; } lines_t;

static void split(char *text, lines_t *l)
{
    char *save = NULL;
    for (char *s = strtok_r(text, "\r\n", &save); s; s = strtok_r(NULL, "\r\n", &save)) {
        l->v = (char **)realloc(l->v, (l->n + 1) * sizeof(char *));
        l->v[l->n++] = s;
    }
}

/* Run one child; returns its stdout, and sets *outcome to "OK", "CRASH(0x...)" or "TIMEOUT".
 * The read polls against a deadline: a hung child keeps the pipe open, so a blocking read
 * would never let the timeout fire. */
static char *run_child(const char *self, const char *mode, const char *dll, const char *check,
                       char *outcome, size_t outsz)
{
    char cmd[2 * MAX_PATH + 64];
    snprintf(cmd, sizeof(cmd), "\"%s\" %s \"%s\" %s", self, mode, dll, check);
    SECURITY_ATTRIBUTES sa = { sizeof(sa), NULL, TRUE };
    HANDLE rd, wr;
    snprintf(outcome, outsz, "ERROR");
    if (!CreatePipe(&rd, &wr, &sa, 0))
        return NULL;
    SetHandleInformation(rd, HANDLE_FLAG_INHERIT, 0);
    STARTUPINFOA si = { sizeof(si) };
    si.dwFlags = STARTF_USESTDHANDLES;
    si.hStdOutput = wr;
    si.hStdError = wr;
    PROCESS_INFORMATION pi;
    if (!CreateProcessA(NULL, cmd, NULL, NULL, TRUE, 0, NULL, NULL, &si, &pi)) {
        DbgPrint("[SHELLTAIL] FAIL cannot spawn %s child gle=%lu\n", mode, GetLastError());
        return NULL;
    }
    CloseHandle(wr);
    size_t cap = 1 << 14, len = 0;
    char *text = (char *)malloc(cap);
    DWORD deadline = GetTickCount() + CHILD_TIMEOUT_MS;
    BOOL timed_out = FALSE;
    for (;;) {
        DWORD avail = 0;
        if (!PeekNamedPipe(rd, NULL, 0, NULL, &avail, NULL))
            break; /* broken pipe: the child exited and everything was read */
        if (avail) {
            DWORD got = 0;
            if (len + avail + 1 > cap)
                text = (char *)realloc(text, cap = (len + avail + 1) * 2);
            if (!ReadFile(rd, text + len, avail, &got, NULL))
                break;
            len += got;
            continue;
        }
        if ((LONG)(deadline - GetTickCount()) <= 0) {
            timed_out = TRUE;
            TerminateProcess(pi.hProcess, 0xDEAD);
            break;
        }
        Sleep(50);
    }
    text[len] = 0;
    CloseHandle(rd);
    WaitForSingleObject(pi.hProcess, 5000);
    DWORD code = 0xFFFFFFFF;
    GetExitCodeProcess(pi.hProcess, &code);
    CloseHandle(pi.hThread);
    CloseHandle(pi.hProcess);
    if (timed_out)
        snprintf(outcome, outsz, "TIMEOUT");
    else if (code != 0)
        snprintf(outcome, outsz, "CRASH(0x%08lx)", (unsigned long)code);
    else
        snprintf(outcome, outsz, "OK");
    return text;
}

static const char *find(lines_t *l, const char *prefix)
{
    size_t n = strlen(prefix);
    for (unsigned i = 0; i < l->n; i++)
        if (strncmp(l->v[i], prefix, n) == 0)
            return l->v[i] + n;
    return NULL;
}

static unsigned expect(lines_t *l, const char *key, const char *want)
{
    const char *got = find(l, key);
    if (got && strcmp(got, want) == 0)
        return 0;
    DbgPrint("[SHELLTAIL] EXPECT %s%s, got %s\n", key, want, got ? got : "(missing)");
    return 1;
}

/* Absolute expectations on the NEW DLL's report, per check. */
static unsigned expectations(const char *check, lines_t *n)
{
    unsigned bad = 0;
    if (!strcmp(check, "msgbox")) {
        bad += expect(n, "msgbox.text=", "A=alpha B=42 C=gamma D=beef E=epsilon");
        bad += expect(n, "msgbox.ret=", "1");
    } else if (!strcmp(check, "broker")) {
        bad += expect(n, "broker.registered=", "YES");
        bad += expect(n, "broker.workerw_added=", "1");
        bad += expect(n, "broker.delivered=", "1");
        bad += expect(n, "broker.event=", "0x8"); /* SHCNE_MKDIR */
        /* No expectation on workerw_after_dereg: the broker lives on after deregistration in
         * the baseline too (the server in explorer cannot DestroyWindow our thread's window);
         * the A/B covers it. */
    } else if (!strcmp(check, "stub")) {
        bad += expect(n, "stub.found=", "YES");
        bad += expect(n, "stub.class.hbrBackground=", "9"); /* COLOR_WINDOWTEXT + 1 */
        bad += expect(n, "stub.class.hInstance=", "SELF");
        bad += expect(n, "stub.StubType=", "2");       /* TYPE_PROPERTYSHEET */
        bad += expect(n, "stub.msgbox_owner_is_stub=", "0");
        bad += expect(n, "stub.destroyed_after=", "1");
    } else if (!strcmp(check, "regitem")) {
        bad += expect(n, "regitem.mycomputer.name.ret=", "1 name=My Computer");
        const char *ic = find(n, "regitem.mycomputer.icon.ret=");
        if (!ic || strncmp(ic, "1 set=1", 7) != 0) {
            DbgPrint("[SHELLTAIL] EXPECT regitem.mycomputer.icon.ret=1 set=1..., got %s\n",
                     ic ? ic : "(missing)");
            bad++;
        }
    } else if (!strcmp(check, "fopt")) {
        bad += expect(n, "fopt.found=", "YES");
        bad += expect(n, "fopt.class.hbrBackground=", "9");
        bad += expect(n, "fopt.class.hInstance=", "SELF");
        bad += expect(n, "fopt.stub_gone=", "1");
    }
    return bad;
}

int main(int argc, char **argv)
{
    /* An unhandled exception now reaches UnhandledExceptionFilter, whose fault box
     * would leave a crashing child waiting on a dialog nobody answers. This mode is
     * inherited by every process spawned from here, which then exits with its
     * exception code instead -- as an unattended run on Windows does. */
    SetErrorMode(SetErrorMode(0) | SEM_NOGPFAULTERRORBOX);
    if (argc == 4)
        return child(argv[1], argv[2], argv[3]);

    char self[MAX_PATH], sysdir[MAX_PATH], olddll[MAX_PATH], newdll[MAX_PATH];
    GetModuleFileNameA(NULL, self, MAX_PATH);
    GetSystemDirectoryA(sysdir, MAX_PATH);
    snprintf(olddll, sizeof(olddll), "%s\\shelltail-base.dll", sysdir);
    snprintf(newdll, sizeof(newdll), "%s\\shell32.dll", sysdir);
    DbgPrint("[SHELLTAIL] start: old=%s new=%s\n", olddll, newdll);

    unsigned same = 0, differ = 0, fixed = 0, broken = 0;
    for (unsigned c = 0; c < N_CHECKS; c++) {
        const char *check = g_checks[c].name;
        char oo[32], no[32];
        char *to = run_child(self, "old", olddll, check, oo, sizeof(oo));
        char *tn = run_child(self, "new", newdll, check, no, sizeof(no));
        lines_t o = { 0 }, n = { 0 };
        if (to) split(to, &o);
        if (tn) split(tn, &n);
        for (unsigned i = 0; i < n.n; i++)
            DbgPrint("[SHELLTAIL] new %s\n", n.v[i]);
        unsigned bad = strcmp(no, "OK") == 0 ? expectations(check, &n) : 1;
        unsigned diffs = 0;
        if (strcmp(oo, "OK") == 0) {
            unsigned max = o.n > n.n ? o.n : n.n;
            for (unsigned i = 0; i < max; i++) {
                const char *a = i < o.n ? o.v[i] : "(none)";
                const char *b = i < n.n ? n.v[i] : "(none)";
                if (strcmp(a, b) != 0) {
                    DbgPrint("[SHELLTAIL] - %s\n[SHELLTAIL] + %s\n", a, b);
                    diffs++;
                }
            }
        }
        const char *verdict;
        if (bad)
            verdict = "BROKEN", broken++;
        else if (strcmp(oo, "OK") != 0)
            verdict = "FIXED", fixed++; /* the baseline could not complete it; the new one passes */
        else if (diffs)
            verdict = "DIFFERENT", differ++;
        else
            verdict = "SAME", same++;
        DbgPrint("[SHELLTAIL] check %s: old=%s new=%s diffs=%u expectation_failures=%u -> %s\n",
                 check, oo, no, diffs, bad, verdict);
    }
    DbgPrint("[SHELLTAIL] done VERDICT=%s same=%u fixed=%u different=%u broken=%u\n",
             broken ? "BROKEN" : differ ? "DIFFERENT" : "SAME", same, fixed, differ, broken);
    return 0;
}
