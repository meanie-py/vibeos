/* modrun probe: acppage (the Compatibility property page), shipping Rust against the pre-flip C.
 *
 * Staged twice: as acppage_c.exe it loads acppage_c.dll (the C oracle) from its directory,
 * as acppage_probe.exe the system acppage.dll. Values only -- HRESULTs, control states, the
 * layer string the page writes -- never a pointer or a profile path, so the runs diff byte
 * for byte. Each run removes what it created.
 *
 * Covered, none of which smoke reaches (explorer shows the page only in a file's
 * Properties): the ATL server (atl_rust::dll), the COM map, IShellExtInit::Initialize over a
 * CF_HDROP data object on each of InitFile's paths (one file / two, an extension it refuses,
 * System32, a .lnk it follows, the AppCompat DisableEngine policy), the GetExeFromLnk export,
 * and the page itself in a modeless property sheet: OnInitDialog's mode list (the table's
 * Display text), OnSetActive reading the per-file layers back from the registry (its Name
 * column), UpdateControls, and OnApply writing them through apphelp. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <prsht.h>
#include <stdio.h>

static const CLSID CLSID_Page = {0x513D916F, 0x2A8E, 0x4F51, {0xAE, 0xAB, 0x0C, 0xBC, 0x76, 0xFB, 0x1A, 0xF8}};

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);
typedef BOOL(WINAPI *EXEFROMLNK)(PCWSTR, PWSTR, size_t);

#define LAYERS L"Software\\Microsoft\\Windows NT\\CurrentVersion\\AppCompatFlags\\Layers"
#define POLICY L"SOFTWARE\\Policies\\Microsoft\\Windows\\AppCompat"

static FARPROC need(HMODULE dll, const char *name)
{
    FARPROC f = GetProcAddress(dll, name);
    if (!f)
    {
        printf("missing export %s\n", name);
        ExitProcess(2);
    }
    return f;
}

/* ---- a minimal IDataObject: CF_HDROP over a fixed list of paths ---------------------- */
typedef struct
{
    IDataObject iface;
    LONG ref;
    const WCHAR **files;
    UINT n;
} DataObj;

static HRESULT STDMETHODCALLTYPE do_qi(IDataObject *t, REFIID riid, void **ppv)
{
    if (IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IDataObject))
    {
        *ppv = t;
        IDataObject_AddRef(t);
        return S_OK;
    }
    *ppv = NULL;
    return E_NOINTERFACE;
}
static ULONG STDMETHODCALLTYPE do_addref(IDataObject *t) { return InterlockedIncrement(&((DataObj *)t)->ref); }
static ULONG STDMETHODCALLTYPE do_release(IDataObject *t) { return InterlockedDecrement(&((DataObj *)t)->ref); }
static HRESULT STDMETHODCALLTYPE do_getdata(IDataObject *t, FORMATETC *fmt, STGMEDIUM *med)
{
    DataObj *d = (DataObj *)t;
    if (fmt->cfFormat != CF_HDROP || !(fmt->tymed & TYMED_HGLOBAL))
        return DV_E_FORMATETC;
    SIZE_T chars = 1;
    for (UINT i = 0; i < d->n; i++)
        chars += lstrlenW(d->files[i]) + 1;
    HGLOBAL h = GlobalAlloc(GHND, sizeof(DROPFILES) + chars * sizeof(WCHAR));
    DROPFILES *df = (DROPFILES *)GlobalLock(h);
    df->pFiles = sizeof(DROPFILES);
    df->fWide = TRUE;
    WCHAR *p = (WCHAR *)(df + 1);
    for (UINT i = 0; i < d->n; i++)
    {
        lstrcpyW(p, d->files[i]);
        p += lstrlenW(p) + 1;
    }
    GlobalUnlock(h);
    med->tymed = TYMED_HGLOBAL;
    med->hGlobal = h;
    med->pUnkForRelease = NULL;
    return S_OK;
}
static HRESULT STDMETHODCALLTYPE do_notimpl() { return E_NOTIMPL; }
static IDataObjectVtbl do_vtbl = {
    do_qi, do_addref, do_release, do_getdata, (void *)do_notimpl, (void *)do_notimpl, (void *)do_notimpl,
    (void *)do_notimpl, (void *)do_notimpl, (void *)do_notimpl, (void *)do_notimpl, (void *)do_notimpl,
};

/* ---- helpers ---------------------------------------------------------------------------- */
static GCO pGetClassObject;

static IShellExtInit *new_page(void)
{
    IClassFactory *cf = NULL;
    IShellExtInit *init = NULL;
    if (SUCCEEDED(pGetClassObject(&CLSID_Page, &IID_IClassFactory, (void **)&cf)))
    {
        IClassFactory_CreateInstance(cf, NULL, &IID_IShellExtInit, (void **)&init);
        IClassFactory_Release(cf);
    }
    return init;
}

static HRESULT init_with(IShellExtInit *init, const WCHAR **files, UINT n)
{
    DataObj d = {{&do_vtbl}, 1, files, n};
    return IShellExtInit_Initialize(init, NULL, &d.iface, NULL);
}

static void initialize_case(const char *label, const WCHAR **files, UINT n)
{
    IShellExtInit *init = new_page();
    if (!init)
    {
        printf("%s: no object\n", label);
        return;
    }
    printf("%s: %08lx\n", label, init_with(init, files, n));
    IShellExtInit_Release(init);
}

static void make_link(const WCHAR *lnk, const WCHAR *target)
{
    IShellLinkW *psl = NULL;
    IPersistFile *ppf = NULL;
    if (SUCCEEDED(CoCreateInstance(&CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, &IID_IShellLinkW, (void **)&psl)))
    {
        IShellLinkW_SetPath(psl, target);
        if (SUCCEEDED(IShellLinkW_QueryInterface(psl, &IID_IPersistFile, (void **)&ppf)))
        {
            IPersistFile_Save(ppf, lnk, TRUE);
            IPersistFile_Release(ppf);
        }
        IShellLinkW_Release(psl);
    }
}

static void show_layers(const char *label, const WCHAR *exe)
{
    WCHAR v[512] = L"";
    DWORD cb = sizeof(v) - sizeof(WCHAR);
    LONG r = RegGetValueW(HKEY_CURRENT_USER, LAYERS, exe, RRF_RT_REG_SZ, NULL, v, &cb);
    printf("%s: %s '%ls'\n", label, r ? "absent" : "present", r ? L"" : v);
}

static void pump(DWORD ms)
{
    DWORD t0 = GetTickCount();
    MSG msg;
    while (GetTickCount() - t0 < ms)
    {
        while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE))
        {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Sleep(10);
    }
}

static HPROPSHEETPAGE g_page;
static BOOL CALLBACK add_page(HPROPSHEETPAGE page, LPARAM lParam)
{
    (*(int *)lParam)++;
    g_page = page;
    return TRUE;
}

static void show_check(HWND page, int id, const char *label)
{
    printf("%s: checked=%u enabled=%d\n", label, IsDlgButtonChecked(page, id), IsWindowEnabled(GetDlgItem(page, id)));
}

int main(void)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    WCHAR self[MAX_PATH], dir[MAX_PATH], exe[MAX_PATH], txt[MAX_PATH], lnk[MAX_PATH], sys[MAX_PATH];
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    HMODULE dll;
    if (len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe"))
    {
        WCHAR p[MAX_PATH];
        lstrcpyW(p, self);
        lstrcpyW(p + len - 4, L".dll");
        dll = LoadLibraryW(p);
    }
    else
        dll = LoadLibraryW(L"acppage.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    pGetClassObject = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    VOIDFN pRegister = (VOIDFN)need(dll, "DllRegisterServer");
    VOIDFN pUnregister = (VOIDFN)need(dll, "DllUnregisterServer");
    EXEFROMLNK pGetExeFromLnk = (EXEFROMLNK)need(dll, "GetExeFromLnk");
    CoInitialize(NULL);
    InitCommonControls();

    /* The subject: a copy of this probe on a local fixed disk (AllowPermLayer refuses the CD). */
    lstrcpyW(dir, L"C:\\modrun_acp");
    CreateDirectoryW(dir, NULL);
    lstrcpyW(exe, dir);
    PathAppendW(exe, L"app.exe");
    lstrcpyW(txt, dir);
    PathAppendW(txt, L"notes.txt");
    lstrcpyW(lnk, dir);
    PathAppendW(lnk, L"app.lnk");
    printf("subject copied %d\n", CopyFileW(self, exe, FALSE));
    CloseHandle(CreateFileW(txt, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL));
    make_link(lnk, exe);
    GetSystemDirectoryW(sys, MAX_PATH);
    PathAppendW(sys, L"notepad.exe");

    printf("unload at start %08lx\n", pCanUnloadNow());
    IShellExtInit *init = new_page();
    printf("create %d\n", init != NULL);
    if (!init)
        return 1;
    IShellPropSheetExt *sheet = NULL;
    IUnknown *unk = NULL;
    printf("qi propsheetext %08lx\n", IShellExtInit_QueryInterface(init, &IID_IShellPropSheetExt, (void **)&sheet));
    printf("qi unknown %08lx\n", IShellExtInit_QueryInterface(init, &IID_IUnknown, (void **)&unk));
    if (unk)
        IUnknown_Release(unk);
    printf("unload with object %08lx\n", pCanUnloadNow());

    const WCHAR *two[2] = {exe, txt};
    const WCHAR *one_txt[1] = {txt};
    const WCHAR *one_sys[1] = {sys};
    const WCHAR *one_lnk[1] = {lnk};
    const WCHAR *one_exe[1] = {exe};
    initialize_case("initialize two files", two, 2);
    initialize_case("initialize txt", one_txt, 1);
    initialize_case("initialize system32", one_sys, 1);
    initialize_case("initialize lnk", one_lnk, 1);
    HKEY pol;
    if (!RegCreateKeyExW(HKEY_LOCAL_MACHINE, POLICY, 0, NULL, 0, KEY_SET_VALUE, NULL, &pol, NULL))
    {
        DWORD one = 1;
        RegSetValueExW(pol, L"DisableEngine", 0, REG_DWORD, (BYTE *)&one, sizeof(one));
        initialize_case("initialize policy-disabled", one_exe, 1);
        RegDeleteValueW(pol, L"DisableEngine");
        RegCloseKey(pol);
    }

    WCHAR target[MAX_PATH] = L"(untouched)";
    BOOL got = pGetExeFromLnk(lnk, target, MAX_PATH);
    printf("getexefromlnk %d '%ls'\n", got, got ? PathFindFileNameW(target) : L"");
    got = pGetExeFromLnk(txt, target, MAX_PATH);
    printf("getexefromlnk not a link %d\n", got);

    /* The page: layers set as a previous Apply would have left them. */
    HKEY lk;
    if (!RegCreateKeyExW(HKEY_CURRENT_USER, LAYERS, 0, NULL, 0, KEY_SET_VALUE, NULL, &lk, NULL))
    {
        const WCHAR *v = L"WIN98 256COLOR MYCUSTOM";
        RegSetValueExW(lk, exe, 0, REG_SZ, (BYTE *)v, (lstrlenW(v) + 1) * sizeof(WCHAR));
        RegCloseKey(lk);
    }
    printf("initialize exe %08lx\n", init_with(init, one_exe, 1));
    int added = 0;
    HRESULT hr = sheet ? IShellPropSheetExt_AddPages(sheet, add_page, (LPARAM)&added) : E_NOINTERFACE;
    printf("addpages %08lx pages=%d\n", hr, added);
    printf("unload with page %08lx\n", pCanUnloadNow());
    if (added == 1)
    {
        PROPSHEETHEADERW psh = {sizeof(psh)};
        psh.dwFlags = PSH_MODELESS;
        psh.pszCaption = L"probe";
        psh.nPages = 1;
        psh.phpage = &g_page;
        HWND sh = (HWND)PropertySheetW(&psh);
        printf("sheet %d\n", sh != NULL);
        pump(500);
        HWND page = sh ? (HWND)SendMessageW(sh, PSM_GETCURRENTPAGEHWND, 0, 0) : NULL;
        printf("page window %d\n", page != NULL);
        if (page)
        {
            HWND combo = GetDlgItem(page, 1002);
            int count = (int)SendMessageW(combo, CB_GETCOUNT, 0, 0);
            int cur = (int)SendMessageW(combo, CB_GETCURSEL, 0, 0);
            WCHAR item[128] = L"";
            SendMessageW(combo, CB_GETLBTEXT, cur, (LPARAM)item);
            printf("modes %d selected %d '%ls' enabled=%d\n", count, cur, item, IsWindowEnabled(combo));
            show_check(page, 1001, "run in compatibility mode");
            show_check(page, 1003, "256 colors");
            show_check(page, 1004, "640x480");
            show_check(page, 1005, "no visual themes");
            WCHAR custom[256] = L"";
            GetDlgItemTextW(page, 1013, custom, 256);
            printf("custom layers '%ls'\n", custom);
            /* A user's edit: 640x480 on, then Apply. */
            CheckDlgButton(page, 1004, BST_CHECKED);
            SendMessageW(page, WM_COMMAND, MAKEWPARAM(1004, BN_CLICKED), (LPARAM)GetDlgItem(page, 1004));
            pump(100);
            SendMessageW(sh, PSM_APPLY, 0, 0);
            pump(300);
            show_layers("layers after apply", exe);
        }
        if (sh)
            DestroyWindow(sh);
        pump(300);
    }
    if (sheet)
        IShellPropSheetExt_Release(sheet);
    IShellExtInit_Release(init);
    printf("unload at end %08lx\n", pCanUnloadNow());

    if (!RegOpenKeyExW(HKEY_CURRENT_USER, LAYERS, 0, KEY_SET_VALUE, &lk))
    {
        RegDeleteValueW(lk, exe);
        RegCloseKey(lk);
    }
    DeleteFileW(lnk);
    DeleteFileW(txt);
    DeleteFileW(exe);
    RemoveDirectoryW(dir);

    printf("register %08lx\n", pRegister());
    WCHAR key[96], g[64];
    StringFromGUID2(&CLSID_Page, g, 64);
    wsprintfW(key, L"CLSID\\%ls\\InProcServer32", g);
    HKEY k;
    LONG r = RegOpenKeyExW(HKEY_CLASSES_ROOT, key, 0, KEY_READ, &k);
    printf("inprocserver32 %s\n", r ? "absent" : "present");
    if (!r)
        RegCloseKey(k);
    printf("unregister %08lx\n", pUnregister());
    printf("probe done\n");
    return 0;
}
