/* modrun probe: zipfldr (the Compressed Folder shell extension), shipping Rust against the
 * pre-flip C.
 *
 * Staged twice: as zipfldr_c.exe it loads zipfldr_c.dll (the C oracle) from its directory,
 * as zipfldr_probe.exe the system zipfldr.dll. Values only -- HRESULTs, names, column text,
 * attributes, compare signs, counts -- so the two runs diff byte for byte.
 *
 * The archive is embedded (a stored entry, two deflated ones so zlib inflates, and a
 * directory) and written to the first writable of %TEMP%, C:\\, Documents, Desktop. The folder is driven the way explorer browses a .zip:
 * IPersistFile::Load, EnumObjects, GetDisplayNameOf, every GetDetailsOf column (name, type,
 * compressed size, password, size, ratio, date), GetAttributesOf, CompareIDs, BindToObject
 * into the subdirectory, and the item context menu's QueryContextMenu/GetCommandString.
 * Then the paths that sit behind UI, with a clicker thread answering the dialogs: "Extract
 * All" through its wizard, SendTo's drop target creating an archive from two files, and the
 * item menu's "delete" (confirmed) rewriting it. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <commctrl.h>
#include <stdio.h>

static const unsigned char kZip[] = {
    0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x83, 0x18, 0x22, 0x58, 0x3f, 0x48,
    0xd9, 0x05, 0x0b, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x72, 0x65,
    0x61, 0x64, 0x6d, 0x65, 0x2e, 0x74, 0x78, 0x74, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x7a, 0x69,
    0x70, 0x0d, 0x0a, 0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x83, 0x18, 0x22,
    0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
    0x00, 0x64, 0x69, 0x72, 0x2f, 0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x83,
    0x18, 0x22, 0x58, 0xf4, 0x6d, 0x1a, 0xc0, 0x0d, 0x00, 0x00, 0x00, 0xf2, 0x00, 0x00, 0x00, 0x0d,
    0x00, 0x00, 0x00, 0x64, 0x69, 0x72, 0x2f, 0x69, 0x6e, 0x6e, 0x65, 0x72, 0x2e, 0x74, 0x78, 0x74,
    0xcb, 0xcc, 0xcb, 0x4b, 0x2d, 0x52, 0xc8, 0x1c, 0x31, 0x24, 0x2f, 0x17, 0x00, 0x50, 0x4b, 0x03,
    0x04, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x83, 0x18, 0x22, 0x58, 0xe2, 0xd1, 0x3d, 0x0a, 0x0b,
    0x00, 0x00, 0x00, 0xb4, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x6e, 0x6f, 0x74, 0x65, 0x73,
    0x2e, 0x6c, 0x6f, 0x67, 0xcb, 0xc9, 0xcc, 0x4b, 0xe5, 0xe5, 0xca, 0x19, 0x52, 0x24, 0x00, 0x50,
    0x4b, 0x01, 0x02, 0x14, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x83, 0x18, 0x22, 0x58, 0x3f,
    0x48, 0xd9, 0x05, 0x0b, 0x00, 0x00, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72, 0x65, 0x61,
    0x64, 0x6d, 0x65, 0x2e, 0x74, 0x78, 0x74, 0x50, 0x4b, 0x01, 0x02, 0x14, 0x00, 0x14, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x83, 0x18, 0x22, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00,
    0x00, 0x33, 0x00, 0x00, 0x00, 0x64, 0x69, 0x72, 0x2f, 0x50, 0x4b, 0x01, 0x02, 0x14, 0x00, 0x14,
    0x00, 0x00, 0x00, 0x08, 0x00, 0x83, 0x18, 0x22, 0x58, 0xf4, 0x6d, 0x1a, 0xc0, 0x0d, 0x00, 0x00,
    0x00, 0xf2, 0x00, 0x00, 0x00, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20,
    0x00, 0x00, 0x00, 0x55, 0x00, 0x00, 0x00, 0x64, 0x69, 0x72, 0x2f, 0x69, 0x6e, 0x6e, 0x65, 0x72,
    0x2e, 0x74, 0x78, 0x74, 0x50, 0x4b, 0x01, 0x02, 0x14, 0x00, 0x14, 0x00, 0x00, 0x00, 0x08, 0x00,
    0x83, 0x18, 0x22, 0x58, 0xe2, 0xd1, 0x3d, 0x0a, 0x0b, 0x00, 0x00, 0x00, 0xb4, 0x00, 0x00, 0x00,
    0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x8d, 0x00,
    0x00, 0x00, 0x6e, 0x6f, 0x74, 0x65, 0x73, 0x2e, 0x6c, 0x6f, 0x67, 0x50, 0x4b, 0x05, 0x06, 0x00,
    0x00, 0x00, 0x00, 0x04, 0x00, 0x04, 0x00, 0xdc, 0x00, 0x00, 0x00, 0xbf, 0x00, 0x00, 0x00, 0x00,
    0x00
};

static const CLSID kZipStorage = {0xe88dcce0, 0xb7b3, 0x11d1, {0xa9, 0xf0, 0x00, 0xaa, 0x00, 0x60, 0xfa, 0x31}};
static const CLSID kZipContextMenu = {0xb8cdcb65, 0xb1bf, 0x4b42, {0x94, 0x28, 0x1d, 0xfd, 0xb7, 0xee, 0x92, 0xaf}};
static const CLSID kZipSendTo = {0x888dca60, 0xfc0a, 0x11cf, {0x8f, 0x0f, 0x00, 0xc0, 0x4f, 0xd7, 0xd0, 0x62}};

/* zipfldr's resource.h: the wizard's destination edit and "show extracted files" box. */
enum { kIdcDirectory = 1001, kIdcShowExtracted = 1102 };

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);

static FARPROC need(HMODULE dll, const char *name)
{
    FARPROC p = GetProcAddress(dll, name);
    if (!p)
    {
        printf("missing export %s\n", name);
        ExitProcess(2);
    }
    return p;
}

static void print_strret(const char *what, STRRET *sr, PCUITEMID_CHILD pidl)
{
    WCHAR buf[MAX_PATH];
    HRESULT hr = StrRetToBufW(sr, pidl, buf, MAX_PATH);
    if (FAILED(hr))
        printf("%s strret %08lx\n", what, (unsigned long)hr);
    else
        printf("%s '%ls'\n", what, buf);
}

static IShellFolder2 *create_folder(GCO gco, const WCHAR *path)
{
    IClassFactory *cf = NULL;
    HRESULT hr = gco(&kZipStorage, &IID_IClassFactory, (void **)&cf);
    printf("storage classobject %08lx\n", (unsigned long)hr);
    if (FAILED(hr))
        return NULL;
    IShellFolder2 *sf = NULL;
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IShellFolder2, (void **)&sf);
    printf("storage create %08lx\n", (unsigned long)hr);
    IClassFactory_Release(cf);
    if (FAILED(hr))
        return NULL;
    IPersistFile *pf = NULL;
    hr = IShellFolder2_QueryInterface(sf, &IID_IPersistFile, (void **)&pf);
    printf("storage qi persistfile %08lx\n", (unsigned long)hr);
    if (SUCCEEDED(hr))
    {
        hr = IPersistFile_Load(pf, path, STGM_READ);
        printf("storage load %08lx\n", (unsigned long)hr);
        IPersistFile_Release(pf);
    }
    return sf;
}

static int list_folder(const char *tag, IShellFolder2 *sf, LPITEMIDLIST *keep, int nkeep)
{
    IEnumIDList *en = NULL;
    HRESULT hr = IShellFolder2_EnumObjects(sf, NULL, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, &en);
    printf("%s enum %08lx\n", tag, (unsigned long)hr);
    if (hr != S_OK || !en)
        return 0;
    int n = 0;
    LPITEMIDLIST pidl;
    ULONG got;
    while (IEnumIDList_Next(en, 1, &pidl, &got) == S_OK && got == 1)
    {
        char what[64];
        STRRET sr;
        sprintf(what, "%s item %d name", tag, n);
        hr = IShellFolder2_GetDisplayNameOf(sf, pidl, SHGDN_NORMAL, &sr);
        if (SUCCEEDED(hr))
            print_strret(what, &sr, pidl);
        else
            printf("%s %08lx\n", what, (unsigned long)hr);
        sprintf(what, "%s item %d parsing", tag, n);
        hr = IShellFolder2_GetDisplayNameOf(sf, pidl, SHGDN_FORPARSING | SHGDN_INFOLDER, &sr);
        if (SUCCEEDED(hr))
            print_strret(what, &sr, pidl);
        else
            printf("%s %08lx\n", what, (unsigned long)hr);
        SFGAOF attrs = SFGAO_FOLDER | SFGAO_STREAM | SFGAO_CANCOPY | SFGAO_CANDELETE |
                       SFGAO_CANRENAME | SFGAO_HASPROPSHEET | SFGAO_FILESYSTEM | SFGAO_BROWSABLE;
        hr = IShellFolder2_GetAttributesOf(sf, 1, (PCUITEMID_CHILD_ARRAY)&pidl, &attrs);
        printf("%s item %d attrs %08lx %08lx\n", tag, n, (unsigned long)hr, (unsigned long)attrs);
        for (UINT col = 0; col < 9; col++)
        {
            SHELLDETAILS sd;
            ZeroMemory(&sd, sizeof(sd));
            sprintf(what, "%s item %d col %u", tag, n, col);
            hr = IShellFolder2_GetDetailsOf(sf, pidl, col, &sd);
            if (SUCCEEDED(hr))
                print_strret(what, &sd.str, pidl);
            else
                printf("%s %08lx\n", what, (unsigned long)hr);
        }
        if (n < nkeep)
            keep[n] = pidl;
        else
            CoTaskMemFree(pidl);
        n++;
    }
    IEnumIDList_Release(en);
    printf("%s items %d\n", tag, n);
    return n;
}

/* The UI-gated paths, driven without a user: a clicker thread answers each dialog that
 * appears in this process -- a message box with Yes (else OK), the extract wizard with Next
 * then Finish (unticking "show extracted files" so no explorer starts). It prints what it
 * answered, so a dialog only one side raises shows up in the diff. */
static volatile LONG g_clicker_stop;
static volatile LONG g_wizard_next, g_wizard_finish;

static BOOL CALLBACK clicker_enum(HWND hwnd, LPARAM lp)
{
    DWORD pid = 0;
    WCHAR cls[16];
    (void)lp;
    GetWindowThreadProcessId(hwnd, &pid);
    if (pid != GetCurrentProcessId() || !IsWindowVisible(hwnd) ||
        !GetClassNameW(hwnd, cls, 16) || lstrcmpW(cls, L"#32770"))
        return TRUE;
    HWND page = (HWND)SendMessageW(hwnd, PSM_GETCURRENTPAGEHWND, 0, 0);
    if (page)
    {
        if (GetDlgItem(page, kIdcDirectory) && !g_wizard_next && IsWindowEnabled(GetDlgItem(page, kIdcDirectory)))
        {
            InterlockedExchange(&g_wizard_next, 1);
            printf("wizard next\n");
            PostMessageW(hwnd, PSM_PRESSBUTTON, PSBTN_NEXT, 0);
        }
        else if (GetDlgItem(page, kIdcShowExtracted) && !g_wizard_finish)
        {
            InterlockedExchange(&g_wizard_finish, 1);
            SendMessageW(GetDlgItem(page, kIdcShowExtracted), BM_SETCHECK, BST_UNCHECKED, 0);
            printf("wizard finish\n");
            PostMessageW(hwnd, PSM_PRESSBUTTON, PSBTN_FINISH, 0);
        }
        return TRUE;
    }
    WCHAR text[512] = L"";
    GetDlgItemTextW(hwnd, 0xFFFF, text, 512);
    int button = GetDlgItem(hwnd, IDYES) ? IDYES : IDOK;
    printf("dialog %s '%ls'\n", button == IDYES ? "yes" : "ok", text);
    PostMessageW(hwnd, WM_COMMAND, MAKEWPARAM(button, BN_CLICKED), (LPARAM)GetDlgItem(hwnd, button));
    Sleep(300); /* let it close before the next scan sees it again */
    return TRUE;
}

static DWORD WINAPI clicker(LPVOID arg)
{
    (void)arg;
    while (!g_clicker_stop)
    {
        Sleep(100);
        EnumWindows(clicker_enum, 0);
    }
    return 0;
}

/* A minimal IDataObject offering CF_HDROP for a list of paths, which is all the drop target
 * and IShellExtInit read. */
typedef struct
{
    IDataObject iface;
    LONG ref;
    WCHAR files[4][MAX_PATH];
    int n;
} HdropObj;

static HRESULT WINAPI ho_qi(IDataObject *This, REFIID riid, void **ppv)
{
    if (IsEqualIID(riid, &IID_IUnknown) || IsEqualIID(riid, &IID_IDataObject))
    {
        *ppv = This;
        IDataObject_AddRef(This);
        return S_OK;
    }
    *ppv = NULL;
    return E_NOINTERFACE;
}
static ULONG WINAPI ho_addref(IDataObject *This) { return InterlockedIncrement(&((HdropObj *)This)->ref); }
static ULONG WINAPI ho_release(IDataObject *This) { return InterlockedDecrement(&((HdropObj *)This)->ref); }
static HRESULT WINAPI ho_querygetdata(IDataObject *This, FORMATETC *fe)
{
    (void)This;
    return fe->cfFormat == CF_HDROP && (fe->tymed & TYMED_HGLOBAL) ? S_OK : DV_E_FORMATETC;
}
static HRESULT WINAPI ho_getdata(IDataObject *This, FORMATETC *fe, STGMEDIUM *stg)
{
    HdropObj *o = (HdropObj *)This;
    if (ho_querygetdata(This, fe) != S_OK)
        return DV_E_FORMATETC;
    SIZE_T chars = 1;
    for (int i = 0; i < o->n; i++)
        chars += lstrlenW(o->files[i]) + 1;
    HGLOBAL h = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, sizeof(DROPFILES) + chars * sizeof(WCHAR));
    DROPFILES *df = (DROPFILES *)GlobalLock(h);
    df->pFiles = sizeof(DROPFILES);
    df->fWide = TRUE;
    WCHAR *p = (WCHAR *)(df + 1);
    for (int i = 0; i < o->n; i++)
    {
        lstrcpyW(p, o->files[i]);
        p += lstrlenW(p) + 1;
    }
    GlobalUnlock(h);
    stg->tymed = TYMED_HGLOBAL;
    stg->hGlobal = h;
    stg->pUnkForRelease = NULL;
    return S_OK;
}
/* Each stub keeps its real signature: on i386 a stdcall callee pops its own arguments. */
static HRESULT WINAPI ho_getdatahere(IDataObject *This, FORMATETC *fe, STGMEDIUM *stg)
{
    (void)This, (void)fe, (void)stg;
    return E_NOTIMPL;
}
static HRESULT WINAPI ho_canonical(IDataObject *This, FORMATETC *in, FORMATETC *out)
{
    (void)This, (void)in, (void)out;
    return E_NOTIMPL;
}
static HRESULT WINAPI ho_setdata(IDataObject *This, FORMATETC *fe, STGMEDIUM *stg, BOOL rel)
{
    (void)This, (void)fe, (void)stg, (void)rel;
    return E_NOTIMPL;
}
static HRESULT WINAPI ho_enumfmt(IDataObject *This, DWORD dir, IEnumFORMATETC **en)
{
    (void)This, (void)dir;
    *en = NULL;
    return E_NOTIMPL;
}
static HRESULT WINAPI ho_dadvise(IDataObject *This, FORMATETC *fe, DWORD f, IAdviseSink *s, DWORD *c)
{
    (void)This, (void)fe, (void)f, (void)s, (void)c;
    return OLE_E_ADVISENOTSUPPORTED;
}
static HRESULT WINAPI ho_dunadvise(IDataObject *This, DWORD c)
{
    (void)This, (void)c;
    return OLE_E_ADVISENOTSUPPORTED;
}
static HRESULT WINAPI ho_enumadvise(IDataObject *This, IEnumSTATDATA **en)
{
    (void)This;
    *en = NULL;
    return OLE_E_ADVISENOTSUPPORTED;
}

static IDataObjectVtbl ho_vtbl = {
    ho_qi, ho_addref, ho_release, ho_getdata, ho_getdatahere, ho_querygetdata,
    ho_canonical, ho_setdata, ho_enumfmt, ho_dadvise, ho_dunadvise, ho_enumadvise,
};

static void hdrop_init(HdropObj *o)
{
    ZeroMemory(o, sizeof(*o));
    o->iface.lpVtbl = &ho_vtbl;
    o->ref = 1;
}

/* Waits for a condition the probe cannot be signalled about: a file another thread of the
 * extension writes, or the wizard closing. */
static BOOL wait_until(BOOL (*cond)(const WCHAR *), const WCHAR *arg, int seconds)
{
    for (int i = 0; i < seconds * 10; i++)
    {
        if (cond(arg))
            return TRUE;
        Sleep(100);
    }
    return FALSE;
}

/* The creator writes the archive on its own thread; done once nobody holds it open. */
static BOOL zip_closed(const WCHAR *path)
{
    HANDLE h = CreateFileW(path, GENERIC_READ, 0, NULL, OPEN_EXISTING, 0, NULL);
    if (h == INVALID_HANDLE_VALUE)
        return FALSE;
    CloseHandle(h);
    return TRUE;
}


static BOOL CALLBACK find_wizard(HWND hwnd, LPARAM lp)
{
    DWORD pid = 0;
    GetWindowThreadProcessId(hwnd, &pid);
    if (pid == GetCurrentProcessId() && SendMessageW(hwnd, PSM_GETCURRENTPAGEHWND, 0, 0))
        *(BOOL *)lp = TRUE;
    return TRUE;
}

static BOOL wizard_open(const WCHAR *unused)
{
    BOOL found = FALSE;
    (void)unused;
    EnumWindows(find_wizard, (LPARAM)&found);
    return found;
}

static BOOL wizard_shut(const WCHAR *unused)
{
    return !wizard_open(unused);
}

/* Names, sizes and a byte sum, relative to the root, sorted by FindFirstFile's order. */
static void walk_tree(const WCHAR *root, const WCHAR *rel)
{
    WCHAR pat[MAX_PATH];
    lstrcpyW(pat, root);
    if (*rel)
        PathAppendW(pat, rel);
    PathAppendW(pat, L"*");
    WIN32_FIND_DATAW fd;
    HANDLE h = FindFirstFileW(pat, &fd);
    if (h == INVALID_HANDLE_VALUE)
        return;
    do
    {
        if (!lstrcmpW(fd.cFileName, L".") || !lstrcmpW(fd.cFileName, L".."))
            continue;
        WCHAR sub[MAX_PATH], full[MAX_PATH];
        lstrcpyW(sub, rel);
        PathAppendW(sub, fd.cFileName);
        lstrcpyW(full, root);
        PathAppendW(full, sub);
        if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
        {
            printf("extracted dir '%ls'\n", sub);
            walk_tree(root, sub);
            continue;
        }
        unsigned long sum = 0;
        HANDLE f = CreateFileW(full, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
        if (f != INVALID_HANDLE_VALUE)
        {
            BYTE buf[512];
            DWORD got;
            while (ReadFile(f, buf, sizeof(buf), &got, NULL) && got)
                for (DWORD k = 0; k < got; k++)
                    sum = sum * 31 + buf[k];
            CloseHandle(f);
        }
        printf("extracted file '%ls' size %lu sum %08lx\n", sub, fd.nFileSizeLow, sum);
    } while (FindNextFileW(h, &fd));
    FindClose(h);
}

static void remove_tree(const WCHAR *dir)
{
    WCHAR pat[MAX_PATH];
    lstrcpyW(pat, dir);
    PathAppendW(pat, L"*");
    WIN32_FIND_DATAW fd;
    HANDLE h = FindFirstFileW(pat, &fd);
    if (h != INVALID_HANDLE_VALUE)
    {
        do
        {
            if (!lstrcmpW(fd.cFileName, L".") || !lstrcmpW(fd.cFileName, L".."))
                continue;
            WCHAR full[MAX_PATH];
            lstrcpyW(full, dir);
            PathAppendW(full, fd.cFileName);
            if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)
                remove_tree(full);
            else
                DeleteFileW(full);
        } while (FindNextFileW(h, &fd));
        FindClose(h);
    }
    RemoveDirectoryW(dir);
}


/* Names and the size/compressed columns of an archive, by a fresh storage folder each time
 * (the date column is the creation time, so it is left out). */
static void brief_list(GCO gco, const char *tag, const WCHAR *zip)
{
    IShellFolder2 *sf = create_folder(gco, zip);
    if (!sf)
        return;
    IEnumIDList *en = NULL;
    HRESULT hr = IShellFolder2_EnumObjects(sf, NULL, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, &en);
    printf("%s enum %08lx\n", tag, (unsigned long)hr);
    int n = 0;
    LPITEMIDLIST pidl;
    ULONG got;
    while (en && IEnumIDList_Next(en, 1, &pidl, &got) == S_OK && got == 1)
    {
        for (UINT col = 0; col < 6; col++)
        {
            SHELLDETAILS sd;
            char what[48];
            ZeroMemory(&sd, sizeof(sd));
            sprintf(what, "%s item %d col %u", tag, n, col);
            hr = IShellFolder2_GetDetailsOf(sf, pidl, col, &sd);
            if (SUCCEEDED(hr))
                print_strret(what, &sd.str, pidl);
            else
                printf("%s %08lx\n", what, (unsigned long)hr);
        }
        CoTaskMemFree(pidl);
        n++;
    }
    if (en)
        IEnumIDList_Release(en);
    printf("%s items %d\n", tag, n);
    IShellFolder2_Release(sf);
}

static void write_file(const WCHAR *path, const char *body, int repeat)
{
    HANDLE h = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
    DWORD wrote;
    for (int i = 0; i < repeat; i++)
        WriteFile(h, body, lstrlenA(body), &wrote, NULL);
    CloseHandle(h);
}

/* SendTo -> Compressed folder: a drop of two files makes "<first>.zip" beside them. */
static BOOL create_archive(GCO gco, const WCHAR *src, WCHAR *zip)
{
    HdropObj files;
    hdrop_init(&files);
    CreateDirectoryW(src, NULL);
    lstrcpyW(files.files[0], src);
    PathAppendW(files.files[0], L"alpha.txt");
    write_file(files.files[0], "alpha alpha alpha alpha\r\n", 40);
    lstrcpyW(files.files[1], src);
    PathAppendW(files.files[1], L"beta.bin");
    write_file(files.files[1], "b", 3);
    files.n = 2;
    lstrcpyW(zip, src);
    PathAppendW(zip, L"alpha.zip");

    IClassFactory *cf = NULL;
    IDropTarget *dt = NULL;
    HRESULT hr = gco(&kZipSendTo, &IID_IClassFactory, (void **)&cf);
    if (SUCCEEDED(hr))
    {
        hr = IClassFactory_CreateInstance(cf, NULL, &IID_IDropTarget, (void **)&dt);
        IClassFactory_Release(cf);
    }
    printf("create droptarget %08lx\n", (unsigned long)hr);
    if (FAILED(hr))
        return FALSE;
    POINTL pt = {0, 0};
    DWORD effect = DROPEFFECT_COPY | DROPEFFECT_MOVE;
    hr = IDropTarget_DragEnter(dt, &files.iface, 0, pt, &effect);
    printf("create dragenter %08lx effect %lu\n", (unsigned long)hr, effect);
    effect = DROPEFFECT_COPY;
    hr = IDropTarget_Drop(dt, &files.iface, 0, pt, &effect);
    printf("create drop %08lx effect %lu\n", (unsigned long)hr, effect);
    IDropTarget_Release(dt);
    BOOL done = wait_until(zip_closed, zip, 60);
    printf("create archive written %d\n", done);
    printf("create dataobject refs %ld\n", files.ref);
    return done;
}

/* The item context menu's "delete" verb: confirm box (the clicker says yes), then the
 * archive is rewritten without the item. */
static void delete_item(GCO gco, const WCHAR *zip, const WCHAR *victim)
{
    IShellFolder2 *sf = create_folder(gco, zip);
    if (!sf)
        return;
    IEnumIDList *en = NULL;
    HRESULT hr = IShellFolder2_EnumObjects(sf, NULL, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, &en);
    LPITEMIDLIST pidl, hit = NULL;
    ULONG got;
    while (en && IEnumIDList_Next(en, 1, &pidl, &got) == S_OK && got == 1)
    {
        STRRET sr;
        WCHAR name[MAX_PATH] = L"";
        if (SUCCEEDED(IShellFolder2_GetDisplayNameOf(sf, pidl, SHGDN_FORPARSING | SHGDN_INFOLDER, &sr)))
            StrRetToBufW(&sr, pidl, name, MAX_PATH);
        if (!hit && !lstrcmpiW(name, victim))
            hit = pidl;
        else
            CoTaskMemFree(pidl);
    }
    if (en)
        IEnumIDList_Release(en);
    printf("delete found %d\n", hit != NULL);
    if (hit)
    {
        IContextMenu *cm = NULL;
        hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, (PCUITEMID_CHILD_ARRAY)&hit, &IID_IContextMenu,
                                         NULL, (void **)&cm);
        printf("delete menu %08lx\n", (unsigned long)hr);
        if (SUCCEEDED(hr))
        {
            CMINVOKECOMMANDINFO ici;
            ZeroMemory(&ici, sizeof(ici));
            ici.cbSize = sizeof(ici);
            ici.lpVerb = "delete";
            ici.nShow = SW_SHOWNORMAL;
            hr = IContextMenu_InvokeCommand(cm, &ici);
            printf("delete invoke %08lx\n", (unsigned long)hr);
            IContextMenu_Release(cm);
        }
        CoTaskMemFree(hit);
    }
    IShellFolder2_Release(sf);
}

/* "Extract All..." on the .zip itself: IShellExtInit + the "extract" verb start the wizard
 * on its own thread; the clicker presses Next (extraction) and Finish. */
static void extract_archive(GCO gco, const WCHAR *zip, const WCHAR *dest)
{
    remove_tree(dest);
    HdropObj files;
    hdrop_init(&files);
    lstrcpyW(files.files[0], zip);
    files.n = 1;
    IClassFactory *cf = NULL;
    IShellExtInit *ext = NULL;
    HRESULT hr = gco(&kZipContextMenu, &IID_IClassFactory, (void **)&cf);
    if (SUCCEEDED(hr))
    {
        hr = IClassFactory_CreateInstance(cf, NULL, &IID_IShellExtInit, (void **)&ext);
        IClassFactory_Release(cf);
    }
    printf("extract create %08lx\n", (unsigned long)hr);
    if (FAILED(hr))
        return;
    hr = IShellExtInit_Initialize(ext, NULL, &files.iface, NULL);
    printf("extract initialize %08lx\n", (unsigned long)hr);
    IContextMenu *cm = NULL;
    if (SUCCEEDED(hr))
        hr = IShellExtInit_QueryInterface(ext, &IID_IContextMenu, (void **)&cm);
    if (cm)
    {
        CMINVOKECOMMANDINFO ici;
        ZeroMemory(&ici, sizeof(ici));
        ici.cbSize = sizeof(ici);
        ici.lpVerb = "extract";
        ici.nShow = SW_SHOWNORMAL;
        hr = IContextMenu_InvokeCommand(cm, &ici);
        printf("extract invoke %08lx\n", (unsigned long)hr);
        IContextMenu_Release(cm);
        if (SUCCEEDED(hr))
        {
            BOOL opened = wait_until(wizard_open, NULL, 30);
            BOOL closed = opened && wait_until(wizard_shut, NULL, 120);
            printf("extract wizard opened %d closed %d next %ld finish %ld\n", opened, closed,
                   g_wizard_next, g_wizard_finish);
        }
    }
    IShellExtInit_Release(ext);
    walk_tree(dest, L"");
}

/* DllCanUnloadNow once the extension's worker threads have had time to drop their module
 * reference: a phase that leaks one shows here, not only in the final total. */
static void settle(VOIDFN pCanUnloadNow, const char *phase)
{
    HRESULT hr = S_FALSE;
    for (int i = 0; i < 50 && (hr = pCanUnloadNow()) != S_OK; i++)
        Sleep(100);
    printf("canunload after %s %08lx\n", phase, (unsigned long)hr);
}

int main(void)
{
    WCHAR self[MAX_PATH];
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    HMODULE dll;
    const BOOL oracle = len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe");
    if (oracle)
    {
        lstrcpyW(self + len - 4, L".dll");
        dll = LoadLibraryW(self);
    }
    else
        dll = LoadLibraryW(L"zipfldr.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GCO gco = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    need(dll, "RouteTheCallW");
    CoInitialize(NULL);

    /* The livecd's X:\ (and so the probe's own directory) is read-only, and %TEMP% may be
     * too; the first writable candidate wins. Reported by label only when none is, so the
     * two runs stay byte-identical whichever one an arch offers. */
    WCHAR path[MAX_PATH];
    WCHAR cands[4][MAX_PATH];
    GetTempPathW(MAX_PATH, cands[0]);
    lstrcpyW(cands[1], L"C:\\");
    SHGetSpecialFolderPathW(NULL, cands[2], CSIDL_PERSONAL, FALSE);
    SHGetSpecialFolderPathW(NULL, cands[3], CSIDL_DESKTOPDIRECTORY, FALSE);
    const char *labels[4] = {"temp", "c root", "personal", "desktop"};
    DWORD errs[4] = {0};
    BOOL written = FALSE;
    for (int c = 0; c < 4 && !written; c++)
    {
        lstrcpyW(path, cands[c]);
        PathAppendW(path, oracle ? L"modrun_zip_c.zip" : L"modrun_zip_rust.zip");
        HANDLE h = CreateFileW(path, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, NULL);
        DWORD wrote = 0;
        if (h == INVALID_HANDLE_VALUE)
        {
            errs[c] = GetLastError();
            continue;
        }
        written = WriteFile(h, kZip, sizeof(kZip), &wrote, NULL) && wrote == sizeof(kZip);
        errs[c] = written ? 0 : GetLastError();
        CloseHandle(h);
        if (!written)
            DeleteFileW(path);
    }
    if (!written)
    {
        for (int c = 0; c < 4; c++)
            printf("write archive %s failed %lu\n", labels[c], errs[c]);
        return 1;
    }

    IShellFolder2 *sf = create_folder(gco, path);
    if (sf)
    {
        for (UINT col = 0; col < 9; col++)
        {
            SHELLDETAILS sd;
            ZeroMemory(&sd, sizeof(sd));
            HRESULT hr = IShellFolder2_GetDetailsOf(sf, NULL, col, &sd);
            char what[32];
            sprintf(what, "header %u", col);
            if (SUCCEEDED(hr))
            {
                printf("header %u fmt=%d cx=%d ", col, sd.fmt, sd.cxChar);
                print_strret("", &sd.str, NULL);
            }
            else
                printf("%s %08lx\n", what, (unsigned long)hr);
            SHCOLSTATEF state = 0;
            hr = IShellFolder2_GetDefaultColumnState(sf, col, &state);
            printf("header %u state %08lx %08lx\n", col, (unsigned long)hr, (unsigned long)state);
        }
        LPITEMIDLIST items[8] = {0};
        int n = list_folder("root", sf, items, 8);
        for (int i = 0; i + 1 < n && i < 7; i++)
        {
            HRESULT hr = IShellFolder2_CompareIDs(sf, 0, items[i], items[i + 1]);
            short c = (short)HRESULT_CODE(hr);
            printf("compare %d %d: %08lx sign %d\n", i, i + 1, (unsigned long)(hr & 0xFFFF0000),
                   c > 0 ? 1 : c < 0 ? -1 : 0);
            hr = IShellFolder2_CompareIDs(sf, 0, items[i], items[i]);
            printf("compare %d self: %08lx\n", i, (unsigned long)hr);
        }
        for (int i = 0; i < n && i < 8; i++)
        {
            SFGAOF a = SFGAO_FOLDER;
            if (FAILED(IShellFolder2_GetAttributesOf(sf, 1, (PCUITEMID_CHILD_ARRAY)&items[i], &a)) ||
                !(a & SFGAO_FOLDER))
                continue;
            IShellFolder2 *sub = NULL;
            HRESULT hr = IShellFolder2_BindToObject(sf, items[i], NULL, &IID_IShellFolder,
                                                    (void **)&sub);
            printf("bind %d %08lx\n", i, (unsigned long)hr);
            if (SUCCEEDED(hr) && sub)
            {
                IShellFolder2 *sub2 = NULL;
                hr = IShellFolder2_QueryInterface(sub, &IID_IShellFolder2, (void **)&sub2);
                printf("bind %d qi folder2 %08lx\n", i, (unsigned long)hr);
                if (SUCCEEDED(hr))
                {
                    LPITEMIDLIST inner[4] = {0};
                    int m = list_folder("sub", sub2, inner, 4);
                    for (int k = 0; k < m && k < 4; k++)
                        CoTaskMemFree(inner[k]);
                    IShellFolder2_Release(sub2);
                }
                IShellFolder2_Release(sub);
            }
        }
        for (int i = 0; i < n && i < 8; i++)
        {
            SFGAOF a = SFGAO_FOLDER;
            if (FAILED(IShellFolder2_GetAttributesOf(sf, 1, (PCUITEMID_CHILD_ARRAY)&items[i], &a)) ||
                (a & SFGAO_FOLDER))
                continue;
            IContextMenu *cm = NULL;
            HRESULT hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, (PCUITEMID_CHILD_ARRAY)&items[i],
                                                     &IID_IContextMenu, NULL, (void **)&cm);
            printf("menu %d %08lx\n", i, (unsigned long)hr);
            if (SUCCEEDED(hr) && cm)
            {
                HMENU menu = CreatePopupMenu();
                hr = IContextMenu_QueryContextMenu(cm, menu, 0, 1, 0x7fff, CMF_NORMAL);
                printf("menu %d query %08lx items %d\n", i, (unsigned long)hr,
                       GetMenuItemCount(menu));
                DestroyMenu(menu);
                IContextMenu_Release(cm);
            }
            break;
        }
        for (int i = 0; i < 8; i++)
            CoTaskMemFree(items[i]);
        printf("storage release %lu\n", IShellFolder2_Release(sf));
    }

    IClassFactory *cf = NULL;
    HRESULT hr = gco(&kZipSendTo, &IID_IClassFactory, (void **)&cf);
    printf("sendto classobject %08lx\n", (unsigned long)hr);
    if (SUCCEEDED(hr))
    {
        IDropTarget *dt = NULL;
        hr = IClassFactory_CreateInstance(cf, NULL, &IID_IDropTarget, (void **)&dt);
        printf("sendto create %08lx\n", (unsigned long)hr);
        if (SUCCEEDED(hr))
            printf("sendto release %lu\n", IDropTarget_Release(dt));
        IClassFactory_Release(cf);
    }

    /* The paths behind UI: extract the embedded archive, then make one from two files and
     * delete an item from it. Each works under its own per-variant name beside the archive. */
    settle(pCanUnloadNow, "browse");
    HANDLE clk = CreateThread(NULL, 0, clicker, NULL, 0, NULL);
    /* Delete rewrites the archive through a GetTempFileName file, and this guest's %TEMP% can
     * be read-only: point TMP at the directory the archive was written to. */
    WCHAR tmp[MAX_PATH];
    lstrcpyW(tmp, path);
    PathRemoveFileSpecW(tmp);
    SetEnvironmentVariableW(L"TMP", tmp);
    WCHAR dest[MAX_PATH], src[MAX_PATH], made[MAX_PATH];
    lstrcpyW(dest, path);
    PathRemoveExtensionW(dest);
    extract_archive(gco, path, dest);
    settle(pCanUnloadNow, "extract");
    remove_tree(dest);
    lstrcpyW(src, dest);
    lstrcatW(src, L"_src");
    remove_tree(src);
    if (create_archive(gco, src, made))
    {
        brief_list(gco, "made", made);
        settle(pCanUnloadNow, "create");
        delete_item(gco, made, L"beta.bin");
        brief_list(gco, "deleted", made);
        settle(pCanUnloadNow, "delete");
    }
    remove_tree(src);
    InterlockedExchange(&g_clicker_stop, 1);
    WaitForSingleObject(clk, 5000);
    CloseHandle(clk);
    /* The extract and create threads drop their module reference as they exit. */
    for (int i = 0; i < 100 && pCanUnloadNow() != S_OK; i++)
        Sleep(100);
    DeleteFileW(path);
    printf("canunload %08lx\n", (unsigned long)pCanUnloadNow());
    printf("probe done\n");
    return 0;
}
