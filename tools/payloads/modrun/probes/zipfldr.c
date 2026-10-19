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
 * Nothing is extracted, created, shown or run. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
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
static const CLSID kZipSendTo = {0x888dca60, 0xfc0a, 0x11cf, {0x8f, 0x0f, 0x00, 0xc0, 0x4f, 0xd7, 0xd0, 0x62}};

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
    DeleteFileW(path);
    printf("canunload %08lx\n", (unsigned long)pCanUnloadNow());
    printf("probe done\n");
    return 0;
}
