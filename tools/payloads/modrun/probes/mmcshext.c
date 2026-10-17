/* modrun probe: mmcshext (the .msc icon handler), shipping Rust against the pre-flip C.
 *
 * Staged twice, like the comctl32 probe: named mmcshext_c.exe it loads mmcshext_c.dll (the C
 * oracle) from its own directory, under any other name the system mmcshext.dll. It prints
 * values only -- HRESULTs, reference counts, strings -- never a pointer, so the runs diff byte
 * for byte. The one path that differs by construction (InprocServer32 names the DLL that was
 * registered) is printed as its file name with the oracle's `_c` dropped.
 *
 * Covered, none of which smoke reaches (setup registers mmcshext; the live CD never does):
 * ATL's in-process server (atl_rust::dll) -- DllCanUnloadNow tracking the factory and object
 * locks, DllGetClassObject for a known and an unknown CLSID, the class factory refusing
 * aggregation -- the COM map's QueryInterface, IExtractIconW::GetIconLocation on its three
 * paths (async, default icon, a real .msc read through msxml3), IPersistFile, and
 * DllRegisterServer/DllUnregisterServer through the .rgs registrar (NoRemove keys kept). */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <stdio.h>

static const CLSID CLSID_Msc = {0x7A80E4A8, 0x8005, 0x11D2, {0xBC, 0xF8, 0x00, 0xC0, 0x4F, 0x72, 0xC7, 0x17}};
static const CLSID CLSID_Nobody = {0x0BADF00D, 0x1234, 0x5678, {1, 2, 3, 4, 5, 6, 7, 8}};

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);
static GCO pGetClassObject;
static VOIDFN pCanUnloadNow, pRegister, pUnregister;

static FARPROC need(HMODULE dll, const char *name)
{
    FARPROC f = GetProcAddress(dll, name);
    if (!f)
    {
        printf("missing export %s\n", name);
        fflush(stdout);
        ExitProcess(2);
    }
    return f;
}

static void show_value(HKEY root, const WCHAR *path, const WCHAR *value, const char *label)
{
    WCHAR buf[MAX_PATH * 2];
    DWORD cb = sizeof(buf) - sizeof(WCHAR), type = 0;
    HKEY k;
    LONG r = RegOpenKeyExW(root, path, 0, KEY_READ, &k);
    if (r)
    {
        printf("%s: no key (%ld)\n", label, r);
        return;
    }
    r = RegQueryValueExW(k, value, NULL, &type, (BYTE *)buf, &cb);
    RegCloseKey(k);
    if (r)
    {
        printf("%s: no value (%ld)\n", label, r);
        return;
    }
    buf[cb / sizeof(WCHAR)] = 0;
    WCHAR *shown = buf;
    if (wcschr(buf, L'\\'))
    {
        /* A module path: its file name, the oracle's mmcshext_c.dll as mmcshext.dll. */
        shown = wcsrchr(buf, L'\\') + 1;
        CharLowerW(shown);
        WCHAR *c = wcsstr(shown, L"_c.dll");
        if (c)
            lstrcpyW(c, L".dll");
    }
    printf("%s: type=%lu '%ls'\n", label, type, shown);
}

static void show_key(HKEY root, const WCHAR *path, const char *label)
{
    HKEY k;
    LONG r = RegOpenKeyExW(root, path, 0, KEY_READ, &k);
    printf("%s: %s\n", label, r ? "absent" : "present");
    if (!r)
        RegCloseKey(k);
}

#define CLSID_KEY L"CLSID\\{7A80E4A8-8005-11D2-BCF8-00C04F72C717}"
#define APPROVED L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Shell Extensions\\Approved"

static void icon(IExtractIconW *pei, UINT in, const char *label)
{
    WCHAR file[MAX_PATH] = L"(untouched)";
    int index = -7;
    UINT out = 0xdead;
    /* By slot: mingw's C IExtractIconWVtbl leaves out the three IUnknown slots, so its
     * `GetIconLocation` member is slot 0 (QueryInterface). In the ABI it is slot 3. */
    typedef HRESULT(STDMETHODCALLTYPE * GIL)(IExtractIconW *, UINT, LPWSTR, UINT, int *, UINT *);
    GIL get = (GIL)((void **)pei->lpVtbl)[3];
    HRESULT hr = get(pei, in, file, MAX_PATH, &index, &out);
    WCHAR *name = wcsrchr(file, L'\\') ? wcsrchr(file, L'\\') + 1 : file;
    printf("%s: hr=%08lx out=%x index=%d file='%ls'\n", label, hr, out, index, name);
}

static void write_msc(const WCHAR *path, const char *xml)
{
    HANDLE h = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
    DWORD n;
    WriteFile(h, xml, lstrlenA(xml), &n, NULL);
    CloseHandle(h);
}

int main(void)
{
    WCHAR self[MAX_PATH];
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    HMODULE dll;
    if (len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe"))
    {
        lstrcpyW(self + len - 4, L".dll");
        dll = LoadLibraryW(self);
    }
    else
        dll = LoadLibraryW(L"mmcshext.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    pGetClassObject = (GCO)need(dll, "DllGetClassObject");
    pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    pRegister = (VOIDFN)need(dll, "DllRegisterServer");
    pUnregister = (VOIDFN)need(dll, "DllUnregisterServer");
    CoInitialize(NULL);

    printf("unload at start %08lx\n", pCanUnloadNow());
    IClassFactory *cf = NULL;
    HRESULT hr = pGetClassObject(&CLSID_Nobody, &IID_IClassFactory, (void **)&cf);
    printf("unknown clsid %08lx null=%d\n", hr, cf == NULL);
    hr = pGetClassObject(&CLSID_Msc, &IID_IClassFactory, (void **)&cf);
    printf("class object %08lx\n", hr);
    if (FAILED(hr))
        return 1;
    printf("unload with factory %08lx\n", pCanUnloadNow());
    IUnknown *again = NULL;
    hr = pGetClassObject(&CLSID_Msc, &IID_IUnknown, (void **)&again);
    printf("class object again %08lx same=%d\n", hr, (void *)again == (void *)cf);
    if (again)
        IUnknown_Release(again);

    IExtractIconW *pei = NULL;
    IUnknown *agg = (IUnknown *)cf;
    hr = IClassFactory_CreateInstance(cf, agg, &IID_IExtractIconW, (void **)&pei);
    printf("aggregate iface %08lx null=%d\n", hr, pei == NULL);
    hr = IClassFactory_CreateInstance(cf, agg, &IID_IUnknown, (void **)&pei);
    printf("aggregate unknown %08lx null=%d\n", hr, pei == NULL);
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IDispatch, (void **)&pei);
    printf("unsupported iface %08lx null=%d\n", hr, pei == NULL);
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IExtractIconW, (void **)&pei);
    printf("create %08lx\n", hr);
    if (FAILED(hr))
        return 1;
    hr = IClassFactory_LockServer(cf, TRUE);
    printf("lockserver %08lx unload while locked %08lx\n", hr, pCanUnloadNow());
    printf("unlockserver %08lx\n", IClassFactory_LockServer(cf, FALSE));
    printf("factory release %lu\n", IClassFactory_Release(cf));
    printf("unload with object %08lx\n", pCanUnloadNow());

    IPersistFile *ppf = NULL;
    IUnknown *unk = NULL;
    void *none = (void *)1;
    printf("qi persistfile %08lx\n", IUnknown_QueryInterface((IUnknown *)pei, &IID_IPersistFile, (void **)&ppf));
    printf("qi unknown %08lx is_first=%d\n",
           IUnknown_QueryInterface((IUnknown *)pei, &IID_IUnknown, (void **)&unk), (void *)unk == (void *)pei);
    printf("qi dispatch %08lx null=%d\n", IUnknown_QueryInterface((IUnknown *)pei, &IID_IDispatch, &none), none == NULL);
    ULONG added = IUnknown_AddRef(unk);
    printf("refs %lu %lu\n", added, IUnknown_Release(unk));
    IUnknown_Release(unk);

    icon(pei, GIL_ASYNC, "async");
    icon(pei, GIL_DEFAULTICON, "default");
    icon(pei, 0, "no file");
    CLSID clsid;
    printf("getclassid %08lx isdirty %08lx\n", IPersistFile_GetClassID(ppf, &clsid), IPersistFile_IsDirty(ppf));

    WCHAR tmp[MAX_PATH], msc[MAX_PATH];
    GetTempPathW(MAX_PATH, tmp);
    wsprintfW(msc, L"%smodrun_icon.msc", tmp);
    write_msc(msc, "<?xml version=\"1.0\"?><MMC_ConsoleFile><VisualAttributes>"
                   "<Icon Index=\"3\" File=\"%SystemRoot%\\system32\\shell32.dll\"/>"
                   "</VisualAttributes></MMC_ConsoleFile>");
    printf("load %08lx\n", IPersistFile_Load(ppf, msc, STGM_READ));
    icon(pei, 0, "msc icon");
    write_msc(msc, "<?xml version=\"1.0\"?><MMC_ConsoleFile><VisualAttributes/></MMC_ConsoleFile>");
    printf("load %08lx\n", IPersistFile_Load(ppf, msc, STGM_READ));
    icon(pei, 0, "msc empty");
    write_msc(msc, "not xml at all");
    printf("load %08lx\n", IPersistFile_Load(ppf, msc, STGM_READ));
    icon(pei, 0, "msc broken");
    DeleteFileW(msc);

    IPersistFile_Release(ppf);
    printf("last release %lu\n", IUnknown_Release((IUnknown *)pei));
    printf("unload at end %08lx\n", pCanUnloadNow());

    printf("register %08lx\n", pRegister());
    show_value(HKEY_CLASSES_ROOT, CLSID_KEY, NULL, "clsid");
    show_value(HKEY_CLASSES_ROOT, CLSID_KEY L"\\InprocServer32", NULL, "inproc");
    show_value(HKEY_CLASSES_ROOT, CLSID_KEY L"\\InprocServer32", L"ThreadingModel", "threading");
    show_value(HKEY_CLASSES_ROOT, L"mscfile\\DefaultIcon", NULL, "defaulticon");
    show_value(HKEY_CLASSES_ROOT, L"mscfile\\shellex\\IconHandler", NULL, "iconhandler");
    show_value(HKEY_LOCAL_MACHINE, APPROVED, L"{7A80E4A8-8005-11D2-BCF8-00C04F72C717}", "approved");
    printf("unregister %08lx\n", pUnregister());
    show_key(HKEY_CLASSES_ROOT, CLSID_KEY, "clsid after");
    show_key(HKEY_CLASSES_ROOT, L"mscfile\\DefaultIcon", "defaulticon after");
    show_key(HKEY_CLASSES_ROOT, L"mscfile\\shellex", "shellex after");
    show_key(HKEY_LOCAL_MACHINE, APPROVED, "approved after");
    /* Put the registration back, as setup leaves it. */
    printf("re-register %08lx\n", pRegister());
    printf("probe done\n");
    fflush(stdout);
    return 0;
}
