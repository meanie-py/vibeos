/* modrun probe: the shell's drop-handler extensions (sendmail's DeskLink, mydocs), shipping
 * Rust against the pre-flip C.
 *
 * One probe, staged four times; its own name says which module and which build:
 * sendmail_c.exe / mydocs_c.exe load <module>_c.dll (the C oracle) from their directory,
 * sendmail_probe.exe / mydocs_probe.exe the system <module>.dll. Values only -- HRESULTs,
 * DROPEFFECT masks, file names -- never a pointer or a profile path, so the runs diff byte
 * for byte. Each run deletes what it created, so the second variant starts from the same
 * state as the first.
 *
 * Covered, none of which smoke reaches (the handlers run when a file is dropped on a
 * .DeskLink or .mydocs SendTo target): ATL's in-process server (atl_rust::dll), the COM map
 * -- including `COM_INTERFACE_ENTRY_IID(IID_IPersist, IPersist)`, which must land on the
 * IPersistFile subobject -- the DragEnter/DragOver effect masks, Drop through a real
 * two-item HIDA (CDataObjectHIDA, HIDA_GetPIDLFolder/HIDA_GetPIDLItem past index 0), and
 * DllRegisterServer's .rgs (a REG_EXPAND_SZ with `%%` escapes) plus its SendTo file. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <stdio.h>

static const CLSID CLSID_DeskLink = {0x9E56BE61, 0xC50F, 0x11CF, {0x9A, 0x2C, 0x00, 0xA0, 0xC9, 0x0A, 0x90, 0xCE}};
static const CLSID CLSID_MyDocsDrop = {0xECF03A32, 0x103D, 0x11d2, {0x85, 0x4D, 0x00, 0x60, 0x08, 0x05, 0x93, 0x67}};

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);
typedef HRESULT(WINAPI *CIDLDATA)(PCIDLIST_ABSOLUTE, UINT, PCUIDLIST_RELATIVE_ARRAY, IDataObject **);

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

static const char *exists(const WCHAR *dir, const WCHAR *name)
{
    WCHAR p[MAX_PATH];
    lstrcpyW(p, dir);
    PathAppendW(p, name);
    return GetFileAttributesW(p) == INVALID_FILE_ATTRIBUTES ? "absent" : "present";
}

static void remove_in(const WCHAR *dir, const WCHAR *name)
{
    WCHAR p[MAX_PATH];
    lstrcpyW(p, dir);
    PathAppendW(p, name);
    DeleteFileW(p);
}

/* The target of a .lnk, as a file name: what CreateShellLink stored. */
static void show_link(const WCHAR *dir, const WCHAR *name)
{
    WCHAR p[MAX_PATH], target[MAX_PATH] = L"(none)", workdir[MAX_PATH] = L"(none)";
    IShellLinkW *psl = NULL;
    IPersistFile *ppf = NULL;
    lstrcpyW(p, dir);
    PathAppendW(p, name);
    HRESULT hr = CoCreateInstance(&CLSID_ShellLink, NULL, CLSCTX_INPROC_SERVER, &IID_IShellLinkW, (void **)&psl);
    if (SUCCEEDED(hr))
        hr = IShellLinkW_QueryInterface(psl, &IID_IPersistFile, (void **)&ppf);
    if (SUCCEEDED(hr))
        hr = IPersistFile_Load(ppf, p, STGM_READ);
    if (SUCCEEDED(hr))
    {
        IShellLinkW_GetPath(psl, target, MAX_PATH, NULL, 0);
        IShellLinkW_GetWorkingDirectory(psl, workdir, MAX_PATH);
    }
    printf("link '%ls': load %08lx target '%ls' workdir '%ls'\n", name, hr, PathFindFileNameW(target),
           workdir[0] ? PathFindFileNameW(workdir) : L"");
    if (ppf)
        IPersistFile_Release(ppf);
    if (psl)
        IShellLinkW_Release(psl);
}

/* The HIDA the handler will read, read back the way shellutils.h does: each item must be
 * the file it was made from. */
static BOOL hida_resolves(IDataObject *pdo, UINT n)
{
    FORMATETC fmt = {(CLIPFORMAT)RegisterClipboardFormatW(L"Shell IDList Array"), NULL, DVASPECT_CONTENT, -1, TYMED_HGLOBAL};
    STGMEDIUM medium = {0};
    HRESULT hr = IDataObject_GetData(pdo, &fmt, &medium);
    printf("hida getdata %08lx\n", hr);
    if (FAILED(hr))
        return FALSE;
    CIDA *cida = (CIDA *)GlobalLock(medium.hGlobal);
    BOOL ok = cida && cida->cidl == n;
    printf("hida cidl %u\n", cida ? cida->cidl : 0);
    for (UINT i = 0; cida && i < cida->cidl; i++)
    {
        PCIDLIST_ABSOLUTE parent = (PCIDLIST_ABSOLUTE)((BYTE *)cida + cida->aoffset[0]);
        PCUIDLIST_RELATIVE child = (PCUIDLIST_RELATIVE)((BYTE *)cida + cida->aoffset[i + 1]);
        PIDLIST_ABSOLUTE full = ILCombine(parent, child);
        WCHAR path[MAX_PATH] = L"";
        BOOL got = full && SHGetPathFromIDListW(full, path);
        printf("hida item %u '%ls'\n", i, got ? PathFindFileNameW(path) : L"(unresolved)");
        ok = ok && got;
        ILFree(full);
    }
    if (cida)
        GlobalUnlock(medium.hGlobal);
    ReleaseStgMedium(&medium);
    return ok;
}

/* A data object over items of `dir` -- what explorer hands the handler on a drop. */
static IDataObject *data_object(const WCHAR *dir, const WCHAR **names, UINT n)
{
    static CIDLDATA create;
    if (!create)
        create = (CIDLDATA)GetProcAddress(GetModuleHandleW(L"shell32.dll"), (LPCSTR)83);
    PIDLIST_ABSOLUTE folder = ILCreateFromPathW(dir);
    PITEMID_CHILD items[4];
    for (UINT i = 0; i < n; i++)
    {
        WCHAR p[MAX_PATH];
        lstrcpyW(p, dir);
        PathAppendW(p, names[i]);
        PIDLIST_ABSOLUTE full = ILCreateFromPathW(p);
        items[i] = ILClone(ILFindLastID(full));
        ILFree(full);
    }
    IDataObject *pdo = NULL;
    BOOL all = folder != NULL;
    for (UINT i = 0; i < n; i++)
    {
        printf("item %u pidl %s\n", i, items[i] ? "ok" : "null");
        all = all && items[i];
    }
    if (!all)
    {
        printf("data object skipped\n");
        for (UINT i = 0; i < n; i++)
            ILFree(items[i]);
        ILFree(folder);
        return NULL;
    }
    HRESULT hr = create ? create(folder, n, (PCUIDLIST_RELATIVE_ARRAY)items, &pdo) : E_NOTIMPL;
    printf("data object %08lx\n", hr);
    if (pdo && !hida_resolves(pdo, n))
    {
        /* A HIDA whose items do not resolve sends mydocs into a MessageBox (no one to close
         * it); the handlers are not what is under test then, the fixture is. */
        IDataObject_Release(pdo);
        pdo = NULL;
    }
    for (UINT i = 0; i < n; i++)
        ILFree(items[i]);
    ILFree(folder);
    return pdo;
}

static void touch(const WCHAR *dir, const WCHAR *name, const char *text)
{
    WCHAR p[MAX_PATH];
    DWORD n;
    lstrcpyW(p, dir);
    PathAppendW(p, name);
    HANDLE h = CreateFileW(p, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, 0, NULL);
    WriteFile(h, text, lstrlenA(text), &n, NULL);
    CloseHandle(h);
}

static const WCHAR *usf = L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders";
#define PROBE_KEY L"Software\\vibeos\\modrun_droptarget"

/* Put back any shell folder a killed run left redirected. */
static void restore_stale(void)
{
    const WCHAR *values[2] = {L"Desktop", L"Personal"};
    for (int i = 0; i < 2; i++)
    {
        WCHAR v[MAX_PATH];
        DWORD type = 0, cb = sizeof(v);
        if (RegGetValueW(HKEY_CURRENT_USER, PROBE_KEY, values[i], RRF_RT_ANY | RRF_NOEXPAND, &type, v, &cb))
            continue;
        RegSetKeyValueW(HKEY_CURRENT_USER, usf, values[i], type, v, cb);
        RegDeleteKeyValueW(HKEY_CURRENT_USER, PROBE_KEY, values[i]);
    }
}

int main(void)
{
    /* Unbuffered: a run that hangs and is killed must still show how far it got. */
    setvbuf(stdout, NULL, _IONBF, 0);
    WCHAR self[MAX_PATH], module[64];
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    BOOL oracle = len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe");
    lstrcpynW(module, PathFindFileNameW(self), 64);
    *wcschr(module, L'_') = 0;
    BOOL desklink = !lstrcmpiW(module, L"sendmail");
    HMODULE dll;
    if (oracle)
    {
        lstrcpyW(self + len - 4, L".dll");
        dll = LoadLibraryW(self);
    }
    else
    {
        WCHAR name[64];
        wsprintfW(name, L"%ls.dll", module);
        dll = LoadLibraryW(name);
    }
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GCO pGetClassObject = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    VOIDFN pRegister = (VOIDFN)need(dll, "DllRegisterServer");
    VOIDFN pUnregister = (VOIDFN)GetProcAddress(dll, "DllUnregisterServer");
    const CLSID *clsid = desklink ? &CLSID_DeskLink : &CLSID_MyDocsDrop;
    CoInitialize(NULL);

    restore_stale();
    printf("module %ls\n", module);
    printf("unload at start %08lx\n", pCanUnloadNow());
    IClassFactory *cf = NULL;
    HRESULT hr = pGetClassObject(clsid, &IID_IClassFactory, (void **)&cf);
    printf("class object %08lx\n", hr);
    if (FAILED(hr))
        return 1;
    IDropTarget *pdt = NULL;
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IDropTarget, (void **)&pdt);
    printf("create %08lx\n", hr);
    IClassFactory_Release(cf);
    if (FAILED(hr))
        return 1;
    printf("unload with object %08lx\n", pCanUnloadNow());

    IPersistFile *ppf = NULL;
    IPersist *pp = NULL;
    IUnknown *punk = NULL;
    printf("qi persistfile %08lx\n", IDropTarget_QueryInterface(pdt, &IID_IPersistFile, (void **)&ppf));
    printf("qi persist %08lx\n", IDropTarget_QueryInterface(pdt, &IID_IPersist, (void **)&pp));
    printf("qi unknown %08lx\n", IDropTarget_QueryInterface(pdt, &IID_IUnknown, (void **)&punk));
    printf("persist is persistfile %d\n", pp != NULL && (void *)pp == (void *)ppf);
    printf("unknown is droptarget %d\n", punk != NULL && (void *)punk == (void *)pdt);
    CLSID got = {0};
    if (pp)
    {
        hr = IPersist_GetClassID(pp, &got);
        printf("class id %08lx match=%d\n", hr, IsEqualCLSID(&got, clsid));
        printf("class id null %08lx\n", IPersist_GetClassID(pp, NULL));
    }
    if (ppf)
    {
        printf("load %08lx\n", IPersistFile_Load(ppf, L"x", 0));
        printf("isdirty %08lx\n", IPersistFile_IsDirty(ppf));
        printf("save %08lx\n", IPersistFile_Save(ppf, L"x", TRUE));
    }

    POINTL pt = {0, 0};
    DWORD effect = DROPEFFECT_COPY | DROPEFFECT_MOVE | DROPEFFECT_LINK;
    hr = IDropTarget_DragEnter(pdt, NULL, 0, pt, &effect);
    printf("dragenter %08lx effect=%lx\n", hr, effect);
    effect = DROPEFFECT_COPY | DROPEFFECT_MOVE | DROPEFFECT_LINK;
    hr = IDropTarget_DragOver(pdt, 0, pt, &effect);
    printf("dragover %08lx effect=%lx\n", hr, effect);
    printf("dragleave %08lx\n", IDropTarget_DragLeave(pdt));
    effect = DROPEFFECT_COPY;
    hr = IDropTarget_Drop(pdt, NULL, 0, pt, &effect);
    printf("drop null %08lx effect=%lx\n", hr, effect);

    /* A real drop: two files from a scratch directory (a folder too, for DeskLink). */
    /* The sources go in the first directory that takes a write: most of the live CD is
     * read-only, and a missing source would hand the handlers null PIDLs. Each candidate is
     * reported (by label, never by path) so a run that finds none says why. */
    WCHAR src[MAX_PATH], dest[MAX_PATH];
    WCHAR cands[4][MAX_PATH];
    const char *labels[4] = {"temp", "c root", "desktop", "personal"};
    DWORD errs[4];
    GetTempPathW(MAX_PATH, cands[0]);
    lstrcpyW(cands[1], L"C:\\");
    SHGetSpecialFolderPathW(NULL, cands[2], CSIDL_DESKTOPDIRECTORY, FALSE);
    SHGetSpecialFolderPathW(NULL, cands[3], CSIDL_PERSONAL, FALSE);
    src[0] = 0;
    for (int c = 0; c < 4; c++)
    {
        WCHAR t[MAX_PATH];
        lstrcpyW(t, cands[c]);
        PathAppendW(t, L"modrun_drop");
        BOOL ok = CreateDirectoryW(t, NULL) || GetLastError() == ERROR_ALREADY_EXISTS;
        errs[c] = ok ? 0 : GetLastError();
        if (ok && !src[0])
            lstrcpyW(src, t);
        else if (ok)
            RemoveDirectoryW(t);
    }
    /* Which candidate was writable differs by arch and by what an earlier run left, so it is
     * reported only when none was (by label, never by path). */
    if (!src[0])
        for (int c = 0; c < 4; c++)
            printf("not writable: %s (%lu)\n", labels[c], errs[c]);
    printf("scratch dir %s\n", src[0] && GetFileAttributesW(src) != INVALID_FILE_ATTRIBUTES ? "present" : "absent");
    touch(src, L"alpha.txt", "alpha");
    touch(src, L"beta.txt", "beta");
    WCHAR sub[MAX_PATH];
    lstrcpyW(sub, src);
    PathAppendW(sub, L"gamma");
    CreateDirectoryW(sub, NULL);
    const WCHAR *names[3] = {L"alpha.txt", L"beta.txt", L"gamma"};
    UINT n = desklink ? 3 : 2;
    /* The live CD's profile is read-only, so the handler's destination (Desktop for DeskLink,
     * My Documents for mydocs) would end in shell32's modal error box. Point that one shell
     * folder at a directory on C: for the drop, and put it back after. */
    const WCHAR *folder_value = desklink ? L"Desktop" : L"Personal";
    WCHAR saved[MAX_PATH] = L"";
    DWORD saved_type = 0, saved_cb = sizeof(saved);
    BOOL had = !RegGetValueW(HKEY_CURRENT_USER, usf, folder_value, RRF_RT_ANY | RRF_NOEXPAND, &saved_type, saved, &saved_cb);
    /* Kept under the probe's own key too, so a run killed mid-drop is undone by the next. */
    if (had)
        RegSetKeyValueW(HKEY_CURRENT_USER, PROBE_KEY, folder_value, saved_type, saved, saved_cb);
    /* Beside the sources: C: is writable on the amd64 and i386 media but not on arm64's. */
    WCHAR redirect[MAX_PATH];
    lstrcpyW(redirect, src);
    PathRemoveFileSpecW(redirect);
    PathAppendW(redirect, L"modrun_dest");
    CreateDirectoryW(redirect, NULL);
    HKEY hk;
    BOOL moved = !RegCreateKeyExW(HKEY_CURRENT_USER, usf, 0, NULL, 0, KEY_SET_VALUE, NULL, &hk, NULL);
    if (moved)
    {
        moved = !RegSetValueExW(hk, folder_value, 0, REG_EXPAND_SZ, (BYTE *)redirect, (lstrlenW(redirect) + 1) * sizeof(WCHAR));
        RegCloseKey(hk);
    }
    SHGetSpecialFolderPathW(NULL, dest, desklink ? CSIDL_DESKTOPDIRECTORY : CSIDL_PERSONAL, FALSE);
    printf("destination redirected %d\n", moved && !lstrcmpiW(dest, redirect));
    IDataObject *pdo = !lstrcmpiW(dest, redirect) ? data_object(src, names, n) : NULL;
    /* A file left by an aborted run would make the copy ask to overwrite (a dialog). */
    const WCHAR *links[3] = {L"Shortcut to alpha.lnk", L"Shortcut to beta.lnk", L"Shortcut to gamma.lnk"};
    for (UINT i = 0; i < n; i++)
        remove_in(dest, desklink ? links[i] : names[i]);
    if (pdo)
    {
        effect = DROPEFFECT_COPY | DROPEFFECT_LINK;
        hr = IDropTarget_Drop(pdt, pdo, 0, pt, &effect);
        printf("drop %08lx effect=%lx\n", hr, effect);
        IDataObject_Release(pdo);
        if (desklink)
        {
            for (UINT i = 0; i < 3; i++)
            {
                printf("%ls: %s\n", links[i], exists(dest, links[i]));
                show_link(dest, links[i]);
                remove_in(dest, links[i]);
            }
        }
        else
        {
            for (UINT i = 0; i < 2; i++)
            {
                printf("%ls: %s\n", names[i], exists(dest, names[i]));
                remove_in(dest, names[i]);
            }
        }
    }
    if (!RegOpenKeyExW(HKEY_CURRENT_USER, usf, 0, KEY_SET_VALUE, &hk))
    {
        if (had)
            RegSetValueExW(hk, folder_value, 0, saved_type, (BYTE *)saved, saved_cb);
        else
            RegDeleteValueW(hk, folder_value);
        RegCloseKey(hk);
    }
    RegDeleteKeyValueW(HKEY_CURRENT_USER, PROBE_KEY, folder_value);
    RemoveDirectoryW(redirect);
    remove_in(src, L"alpha.txt");
    remove_in(src, L"beta.txt");
    RemoveDirectoryW(sub);
    RemoveDirectoryW(src);

    if (ppf)
        IPersistFile_Release(ppf);
    if (pp)
        IPersist_Release(pp);
    if (punk)
        IUnknown_Release(punk);
    printf("release %lu\n", IDropTarget_Release(pdt));
    printf("unload at end %08lx\n", pCanUnloadNow());

    /* DllRegisterServer: the .rgs, and the SendTo file it drops into the default profile. */
    WCHAR sendto[MAX_PATH];
    hr = SHGetFolderPathW(NULL, CSIDL_SENDTO, INVALID_HANDLE_VALUE, SHGFP_TYPE_DEFAULT, sendto);
    printf("register %08lx\n", pRegister());
    WCHAR key[80], expand[MAX_PATH];
    StringFromGUID2(clsid, expand, 64);
    wsprintfW(key, L"CLSID\\%ls\\DefaultIcon", expand);
    DWORD type = 0, cb = sizeof(expand);
    LONG r = RegGetValueW(HKEY_CLASSES_ROOT, key, NULL, RRF_RT_ANY | RRF_NOEXPAND, &type, expand, &cb);
    printf("defaulticon %ld type=%lu '%ls'\n", r, type, r ? L"" : expand);
    if (SUCCEEDED(hr))
    {
        WIN32_FIND_DATAW fd;
        WCHAR pattern[MAX_PATH];
        lstrcpyW(pattern, sendto);
        PathAppendW(pattern, desklink ? L"*.DeskLink" : L"*.mydocs");
        HANDLE f = FindFirstFileW(pattern, &fd);
        printf("sendto file %s\n", f == INVALID_HANDLE_VALUE ? "absent" : "present");
        if (f != INVALID_HANDLE_VALUE)
        {
            FindClose(f);
            remove_in(sendto, fd.cFileName);
        }
    }
    if (pUnregister)
        printf("unregister %08lx\n", pUnregister());
    else
        printf("unregister not exported\n");
    printf("probe done\n");
    fflush(stdout);
    return 0;
}
