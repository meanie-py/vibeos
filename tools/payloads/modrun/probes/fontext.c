/* modrun probe: fontext (the Fonts shell folder), shipping Rust against the pre-flip C.
 *
 * Staged twice: as fontext_c.exe it loads fontext_c.dll (the C oracle) from its directory, as
 * fontext_probe.exe the system fontext.dll. Values only -- HRESULTs, names, column text,
 * attributes, counts -- never a pointer, so the runs diff byte for byte. A non-ASCII
 * character is printed as \uXXXX so the serial log cannot mangle it.
 *
 * Explorer loads this folder only when someone opens Fonts, so smoke reaches none of it.
 * Covered: the ATL server (and CFontExtModule's Init, which builds the font cache before
 * anything can enumerate), IPersistFolder2 over CSIDL_FONTS, EnumObjects over the cache
 * (CFontCache's sorted insert, the font PIDLs), every display-name flavour, the column
 * headers and each column's details (file size, modified date, attributes), CompareIDs per
 * column and for SHCIDS_ALLFIELDS, GetAttributesOf, ParseDisplayName, the item and
 * background context menus, the CF_HDROP data object, and DllCanUnloadNow. Nothing is
 * installed or registered: DllRegisterServer writes desktop.ini into the Fonts folder. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <stdio.h>

static const CLSID CLSID_CFontExt = {0xbd84b380, 0x8ca2, 0x1069, {0xab, 0x1d, 0x08, 0x00, 0x09, 0x48, 0xf5, 0x34}};

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);

#define MAX_ITEMS 64
#define DETAIL_ITEMS 4

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

static void put_wide(const WCHAR *s)
{
    for (; s && *s; s++)
    {
        if (*s >= 0x20 && *s < 0x7F)
            putchar((char)*s);
        else
            printf("\\u%04x", *s);
    }
}

static void put_strret(STRRET *sr, PCUITEMID_CHILD pidl)
{
    WCHAR buf[MAX_PATH];
    HRESULT hr = StrRetToBufW(sr, pidl, buf, MAX_PATH);
    if (FAILED(hr))
        printf("<strret %08lx>", hr);
    else
        put_wide(buf);
}

/* A display name with the Fonts folder's own path replaced: the probe must not print where
 * %SystemRoot% is, only what follows it. */
static void put_path(const WCHAR *s, const WCHAR *fonts)
{
    size_t n = lstrlenW(fonts);
    if (n && !StrCmpNIW(s, fonts, (int)n))
    {
        printf("<fonts>");
        s += n;
    }
    put_wide(s);
}

static void names(IShellFolder2 *sf, PCUITEMID_CHILD pidl, const WCHAR *fonts)
{
    static const struct { SHGDNF f; const char *label; } kinds[] = {
        {SHGDN_NORMAL, "normal"},
        {SHGDN_INFOLDER, "infolder"},
        {SHGDN_FORPARSING, "parsing"},
        {SHGDN_INFOLDER | SHGDN_FORPARSING, "infolder-parsing"},
    };
    for (int k = 0; k < 4; k++)
    {
        STRRET sr;
        WCHAR buf[MAX_PATH];
        HRESULT hr = IShellFolder2_GetDisplayNameOf(sf, pidl, kinds[k].f, &sr);
        printf("  name %s %08lx '", kinds[k].label, hr);
        if (SUCCEEDED(hr) && SUCCEEDED(StrRetToBufW(&sr, pidl, buf, MAX_PATH)))
            put_path(buf, fonts);
        printf("'\n");
    }
}

static void details(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
{
    for (UINT c = 0; c < 12; c++)
    {
        SHELLDETAILS sd;
        HRESULT hr = IShellFolder2_GetDetailsOf(sf, pidl, c, &sd);
        if (FAILED(hr) && c > 0)
        {
            printf("  columns %u (%08lx)\n", c, hr);
            break;
        }
        printf("  col %u %08lx", c, hr);
        if (SUCCEEDED(hr))
        {
            if (!pidl)
                printf(" fmt=%d cx=%d", sd.fmt, sd.cxChar);
            printf(" '");
            put_strret(&sd.str, pidl);
            printf("'");
        }
        printf("\n");
    }
}

int main(void)
{
    WCHAR self[MAX_PATH], fonts[MAX_PATH] = L"";
    DWORD len = GetModuleFileNameW(NULL, self, MAX_PATH);
    HMODULE dll;
    if (len > 6 && !lstrcmpiW(self + len - 6, L"_c.exe"))
    {
        lstrcpyW(self + len - 4, L".dll");
        dll = LoadLibraryW(self);
    }
    else
        dll = LoadLibraryW(L"fontext.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GCO pGetClassObject = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    need(dll, "DllRegisterServer");
    need(dll, "DllUnregisterServer");
    CoInitialize(NULL);
    SHGetFolderPathW(NULL, CSIDL_FONTS, NULL, 0, fonts);
    if (fonts[0])
        PathAddBackslashW(fonts);

    IClassFactory *cf = NULL;
    HRESULT hr = pGetClassObject(&CLSID_CFontExt, &IID_IClassFactory, (void **)&cf);
    printf("classobject %08lx\n", hr);
    if (FAILED(hr))
        return 1;
    IShellFolder2 *sf = NULL;
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IShellFolder2, (void **)&sf);
    printf("create %08lx\n", hr);
    IClassFactory_Release(cf);
    if (!sf)
        return 1;

    IPersistFolder2 *pf = NULL;
    hr = IShellFolder2_QueryInterface(sf, &IID_IPersistFolder2, (void **)&pf);
    printf("qi persistfolder2 %08lx\n", hr);
    CLSID clsid;
    if (pf)
    {
        hr = IPersistFolder2_GetClassID(pf, &clsid);
        printf("classid %08lx %d\n", hr, IsEqualCLSID(&clsid, &CLSID_CFontExt));
        LPITEMIDLIST root = NULL;
        hr = SHGetSpecialFolderLocation(NULL, CSIDL_FONTS, &root);
        printf("fonts pidl %08lx\n", hr);
        hr = IPersistFolder2_Initialize(pf, root);
        printf("initialize %08lx\n", hr);
        LPITEMIDLIST cur = NULL;
        hr = IPersistFolder2_GetCurFolder(pf, &cur);
        printf("curfolder %08lx same=%d\n", hr, cur && root && ILIsEqual(cur, root));
        CoTaskMemFree(cur);
        CoTaskMemFree(root);
        IPersistFolder2_Release(pf);
    }
    IDropTarget *dt = NULL;
    hr = IShellFolder2_QueryInterface(sf, &IID_IDropTarget, (void **)&dt);
    printf("qi droptarget %08lx\n", hr);
    if (dt)
        IDropTarget_Release(dt);

    printf("header\n");
    details(sf, NULL);

    IEnumIDList *en = NULL;
    hr = IShellFolder2_EnumObjects(sf, NULL, SHCONTF_NONFOLDERS, &en);
    printf("enum %08lx\n", hr);
    LPITEMIDLIST items[MAX_ITEMS];
    ULONG count = 0;
    if (en)
    {
        LPITEMIDLIST p;
        ULONG got;
        while (count < MAX_ITEMS && IEnumIDList_Next(en, 1, &p, &got) == S_OK && got == 1)
            items[count++] = p;
        printf("items %lu\n", count);
        hr = IEnumIDList_Reset(en);
        printf("reset %08lx\n", hr);
        hr = IEnumIDList_Skip(en, 1);
        printf("skip %08lx\n", hr);
        IEnumIDList_Release(en);
    }
    for (ULONG i = 0; i < count; i++)
    {
        STRRET sr;
        printf("item %lu '", i);
        if (SUCCEEDED(IShellFolder2_GetDisplayNameOf(sf, items[i], SHGDN_NORMAL, &sr)))
            put_strret(&sr, items[i]);
        printf("'\n");
    }
    for (ULONG i = 0; i < count && i < DETAIL_ITEMS; i++)
    {
        printf("detail %lu\n", i);
        names(sf, items[i], fonts);
        details(sf, items[i]);
        SFGAOF attrs = SFGAO_CANCOPY | SFGAO_CANDELETE | SFGAO_CANRENAME | SFGAO_HASPROPSHEET |
                       SFGAO_FOLDER | SFGAO_FILESYSTEM | SFGAO_CANLINK;
        hr = IShellFolder2_GetAttributesOf(sf, 1, (PCUITEMID_CHILD_ARRAY)&items[i], &attrs);
        printf("  attrs %08lx %08lx\n", hr, attrs);
    }
    if (count >= 2)
    {
        for (int c = 0; c < 5; c++)
        {
            hr = IShellFolder2_CompareIDs(sf, c, items[0], items[1]);
            printf("compare col %d %d\n", c, (short)HRESULT_CODE(hr));
        }
        hr = IShellFolder2_CompareIDs(sf, SHCIDS_ALLFIELDS, items[0], items[0]);
        printf("compare allfields self %d\n", (short)HRESULT_CODE(hr));
        hr = IShellFolder2_CompareIDs(sf, SHCIDS_ALLFIELDS, items[1], items[0]);
        printf("compare allfields %d\n", (short)HRESULT_CODE(hr));
    }
    if (count > 0)
    {
        /* ParseDisplayName takes the font's file path. */
        STRRET sr;
        WCHAR path[MAX_PATH];
        if (SUCCEEDED(IShellFolder2_GetDisplayNameOf(sf, items[0], SHGDN_FORPARSING, &sr)) &&
            SUCCEEDED(StrRetToBufW(&sr, items[0], path, MAX_PATH)))
        {
            LPITEMIDLIST parsed = NULL;
            ULONG eaten = 0;
            hr = IShellFolder2_ParseDisplayName(sf, NULL, NULL, path, &eaten, &parsed, NULL);
            printf("parse %08lx", hr);
            if (parsed)
                printf(" same=%d", IShellFolder2_CompareIDs(sf, 0, parsed, items[0]) == S_OK);
            printf("\n");
            CoTaskMemFree(parsed);
        }
        LPITEMIDLIST bogus = NULL;
        hr = IShellFolder2_ParseDisplayName(sf, NULL, NULL, L"C:\\no\\such\\font.ttf", NULL, &bogus,
                                            NULL);
        printf("parse missing %08lx null=%d\n", hr, bogus == NULL);
        CoTaskMemFree(bogus);

        IContextMenu *cm = NULL;
        hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, (PCUITEMID_CHILD_ARRAY)&items[0],
                                         &IID_IContextMenu, NULL, (void **)&cm);
        printf("item contextmenu %08lx\n", hr);
        if (cm)
        {
            HMENU menu = CreatePopupMenu();
            hr = IContextMenu_QueryContextMenu(cm, menu, 0, 1, 0x7FFF, CMF_NORMAL);
            printf("  query %08lx entries %d\n", hr, GetMenuItemCount(menu));
            DestroyMenu(menu);
            IContextMenu_Release(cm);
        }
        IDataObject *dob = NULL;
        ULONG n = count < 3 ? count : 3;
        hr = IShellFolder2_GetUIObjectOf(sf, NULL, n, (PCUITEMID_CHILD_ARRAY)items,
                                         &IID_IDataObject, NULL, (void **)&dob);
        printf("dataobject %08lx\n", hr);
        if (dob)
        {
            FORMATETC fe = {CF_HDROP, NULL, DVASPECT_CONTENT, -1, TYMED_HGLOBAL};
            STGMEDIUM stg;
            hr = IDataObject_GetData(dob, &fe, &stg);
            printf("  hdrop %08lx", hr);
            if (SUCCEEDED(hr))
            {
                UINT files = DragQueryFileW((HDROP)stg.hGlobal, 0xFFFFFFFF, NULL, 0);
                printf(" files %u '", files);
                WCHAR f[MAX_PATH];
                if (files && DragQueryFileW((HDROP)stg.hGlobal, 0, f, MAX_PATH))
                    put_path(f, fonts);
                printf("'");
                ReleaseStgMedium(&stg);
            }
            printf("\n");
            IDataObject_Release(dob);
        }
    }
    IContextMenu *bg = NULL;
    hr = IShellFolder2_CreateViewObject(sf, NULL, &IID_IContextMenu, (void **)&bg);
    printf("background contextmenu %08lx\n", hr);
    if (bg)
    {
        HMENU menu = CreatePopupMenu();
        hr = IContextMenu_QueryContextMenu(bg, menu, 0, 1, 0x7FFF, CMF_NORMAL);
        printf("  query %08lx entries %d\n", hr, GetMenuItemCount(menu));
        DestroyMenu(menu);
        IContextMenu_Release(bg);
    }

    for (ULONG i = 0; i < count; i++)
        CoTaskMemFree(items[i]);
    printf("release %lu\n", IShellFolder2_Release(sf));
    printf("canunload %08lx\n", pCanUnloadNow());
    CoUninitialize();
    printf("probe done\n");
    return 0;
}
