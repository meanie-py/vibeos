/* modrun probe: ntobjshex (the NT object namespace and registry shell folders), shipping Rust
 * against the pre-flip C.
 *
 * Staged twice: as ntobjshex_c.exe it loads ntobjshex_c.dll (the C oracle) from its
 * directory, as ntobjshex_probe.exe the system ntobjshex.dll. Values only -- HRESULTs, names,
 * column text, attributes, counts -- so the runs diff byte for byte; a non-ASCII character
 * is printed as \uXXXX.
 *
 * Nothing at boot browses these folders, so smoke reaches none of it. Covered: the ATL server
 * (both object-map classes, the COM map their CRTP base CCommonFolder<..> declares, and
 * aggregation, which both refuse), IPersistFolder2, EnumObjects
 * over the NT root and the registry root (CEnumNTDirectory, CEnumRegRoot), BindToObject into
 * an NT directory and into HKEY_LOCAL_MACHINE (CEnumRegKey) and their enumerations, every
 * display-name flavour, the column headers and each column's details (object types, symbolic
 * link targets, registry value types), CompareIDs, GetAttributesOf, the extract-icon and
 * context-menu objects, and DllCanUnloadNow. Read-only: nothing is created or written. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <stdio.h>

static const CLSID CLSID_NtObjectFolder = {0x845b0fb2, 0x66e0, 0x416b, {0x8f, 0x91, 0x31, 0x4e, 0x23, 0xf7, 0xc1, 0x2d}};
static const CLSID CLSID_RegistryFolder = {0x1c6d6e08, 0x2332, 0x4a7b, {0xa9, 0x4d, 0x64, 0x32, 0xdb, 0x2b, 0x5a, 0xe6}};

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);

#define MAX_ITEMS 48
#define DETAIL_ITEMS 6

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

static void details(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
{
    for (UINT c = 0; c < 10; c++)
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

static void names(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
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
        HRESULT hr = IShellFolder2_GetDisplayNameOf(sf, pidl, kinds[k].f, &sr);
        printf("  name %s %08lx '", kinds[k].label, hr);
        if (SUCCEEDED(hr))
            put_strret(&sr, pidl);
        printf("'\n");
    }
}

/* Enumerate `sf`, print every item's name, details for the first few, and return the index
 * of the item named `want` (or -1). The items are left in `items`. */
static int walk(IShellFolder2 *sf, const char *label, SHCONTF flags, LPITEMIDLIST *items,
                ULONG *count, const WCHAR *want)
{
    IEnumIDList *en = NULL;
    HRESULT hr = IShellFolder2_EnumObjects(sf, NULL, flags, &en);
    printf("%s enum %08lx\n", label, hr);
    *count = 0;
    if (!en)
        return -1;
    LPITEMIDLIST p;
    ULONG got;
    while (*count < MAX_ITEMS && IEnumIDList_Next(en, 1, &p, &got) == S_OK && got == 1)
        items[(*count)++] = p;
    IEnumIDList_Release(en);
    printf("%s items %lu\n", label, *count);
    int found = -1;
    for (ULONG i = 0; i < *count; i++)
    {
        STRRET sr;
        WCHAR buf[MAX_PATH] = L"";
        if (SUCCEEDED(IShellFolder2_GetDisplayNameOf(sf, items[i], SHGDN_INFOLDER, &sr)))
            StrRetToBufW(&sr, items[i], buf, MAX_PATH);
        printf("%s item %lu '", label, i);
        put_wide(buf);
        SFGAOF attrs = SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_LINK | SFGAO_BROWSABLE |
                       SFGAO_CANRENAME | SFGAO_CANDELETE;
        hr = IShellFolder2_GetAttributesOf(sf, 1, (PCUITEMID_CHILD_ARRAY)&items[i], &attrs);
        printf("' attrs %08lx %08lx\n", hr, attrs);
        if (want && found < 0 && !lstrcmpiW(buf, want))
            found = (int)i;
    }
    for (ULONG i = 0; i < *count && i < DETAIL_ITEMS; i++)
    {
        printf("%s detail %lu\n", label, i);
        names(sf, items[i]);
        details(sf, items[i]);
    }
    if (*count >= 2)
    {
        for (int c = 0; c < 4; c++)
            printf("%s compare col %d %d\n", label, c,
                   (short)HRESULT_CODE(IShellFolder2_CompareIDs(sf, c, items[0], items[1])));
        printf("%s compare self %d\n", label,
               (short)HRESULT_CODE(IShellFolder2_CompareIDs(sf, 0, items[1], items[1])));
    }
    return found;
}

/* BindToObject answers only IID_IShellFolder (foldercommon.h); the child is then QI'd. */
static IShellFolder2 *bind_child(IShellFolder2 *sf, const char *label, LPITEMIDLIST item)
{
    IShellFolder *child = NULL;
    HRESULT hr = IShellFolder2_BindToObject(sf, item, NULL, &IID_IShellFolder, (void **)&child);
    printf("%s bind %08lx\n", label, hr);
    if (!child)
        return NULL;
    IShellFolder2 *child2 = NULL;
    hr = IShellFolder_QueryInterface(child, &IID_IShellFolder2, (void **)&child2);
    printf("%s bind qi %08lx\n", label, hr);
    IShellFolder_Release(child);
    return child2;
}

static void free_items(LPITEMIDLIST *items, ULONG count)
{
    for (ULONG i = 0; i < count; i++)
        CoTaskMemFree(items[i]);
}

static IShellFolder2 *create(GCO gco, const CLSID *clsid, const char *label)
{
    IClassFactory *cf = NULL;
    HRESULT hr = gco(clsid, &IID_IClassFactory, (void **)&cf);
    printf("%s classobject %08lx\n", label, hr);
    if (!cf)
        return NULL;
    IShellFolder2 *sf = NULL;
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IShellFolder2, (void **)&sf);
    printf("%s create %08lx\n", label, hr);
    IUnknown *agg = NULL;
    hr = IClassFactory_CreateInstance(cf, (IUnknown *)cf, &IID_IUnknown, (void **)&agg);
    printf("%s create aggregated %08lx\n", label, hr);
    if (agg)
        IUnknown_Release(agg);
    IClassFactory_Release(cf);
    if (!sf)
        return NULL;
    IPersistFolder2 *pf = NULL;
    hr = IShellFolder2_QueryInterface(sf, &IID_IPersistFolder2, (void **)&pf);
    printf("%s qi persistfolder2 %08lx\n", label, hr);
    IShellFolder *sf1 = NULL;
    hr = IShellFolder2_QueryInterface(sf, &IID_IShellFolder, (void **)&sf1);
    printf("%s qi shellfolder %08lx same=%d\n", label, hr, (void *)sf1 == (void *)sf);
    if (sf1)
        IShellFolder_Release(sf1);
    IDropTarget *dt = NULL;
    hr = IShellFolder2_QueryInterface(sf, &IID_IDropTarget, (void **)&dt);
    printf("%s qi droptarget %08lx\n", label, hr);
    if (pf)
    {
        CLSID got;
        hr = IPersistFolder2_GetClassID(pf, &got);
        printf("%s classid %08lx %d\n", label, hr, IsEqualCLSID(&got, clsid));
        LPITEMIDLIST desk = NULL;
        SHGetSpecialFolderLocation(NULL, CSIDL_DESKTOP, &desk);
        hr = IPersistFolder2_Initialize(pf, desk);
        printf("%s initialize %08lx\n", label, hr);
        LPITEMIDLIST cur = NULL;
        hr = IPersistFolder2_GetCurFolder(pf, &cur);
        printf("%s curfolder %08lx same=%d\n", label, hr, cur && desk && ILIsEqual(cur, desk));
        CoTaskMemFree(cur);
        CoTaskMemFree(desk);
        IPersistFolder2_Release(pf);
    }
    printf("%s header\n", label);
    details(sf, NULL);
    return sf;
}

static void ui_objects(IShellFolder2 *sf, const char *label, LPITEMIDLIST item)
{
    IExtractIconW *ei = NULL;
    HRESULT hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, (PCUITEMID_CHILD_ARRAY)&item,
                                             &IID_IExtractIconW, NULL, (void **)&ei);
    printf("%s extracticon %08lx\n", label, hr);
    /* GetIconLocation is not called: the pre-flip C faults on it too (address 2, the
     * GIL_FORSHELL flag read as a pointer -- owed, cpp-cluster-plan.md), so it measures
     * nothing about the port. */
    if (ei)
        IUnknown_Release((IUnknown *)ei);
    IContextMenu *cm = NULL;
    hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, (PCUITEMID_CHILD_ARRAY)&item, &IID_IContextMenu,
                                     NULL, (void **)&cm);
    printf("%s contextmenu %08lx\n", label, hr);
    if (cm)
        IContextMenu_Release(cm);
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
        dll = LoadLibraryW(L"ntobjshex.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GCO gco = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    need(dll, "DllRegisterServer");
    need(dll, "DllUnregisterServer");
    CoInitialize(NULL);
    LPITEMIDLIST items[MAX_ITEMS], sub[MAX_ITEMS];
    ULONG count = 0, subcount = 0;

    IShellFolder2 *nt = create(gco, &CLSID_NtObjectFolder, "nt");
    if (nt)
    {
        int at = walk(nt, "nt", SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, items, &count, L"ObjectTypes");
        printf("nt objecttypes at %d\n", at >= 0);
        if (count)
            ui_objects(nt, "nt", items[0]);
        if (at >= 0)
        {
            IShellFolder2 *child = bind_child(nt, "nt", items[at]);
            if (child)
            {
                walk(child, "nt/ObjectTypes", SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, sub,
                     &subcount, NULL);
                free_items(sub, subcount);
                printf("nt/ObjectTypes release %lu\n", IShellFolder2_Release(child));
            }
        }
        free_items(items, count);
        printf("nt release %lu\n", IShellFolder2_Release(nt));
    }

    IShellFolder2 *reg = create(gco, &CLSID_RegistryFolder, "reg");
    if (reg)
    {
        int at = walk(reg, "reg", SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, items, &count,
                      L"HKEY_LOCAL_MACHINE");
        printf("reg hklm at %d\n", at >= 0);
        if (count)
            ui_objects(reg, "reg", items[0]);
        if (at >= 0)
        {
            IShellFolder2 *child = bind_child(reg, "reg", items[at]);
            if (child)
            {
                walk(child, "reg/HKLM", SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, sub, &subcount,
                     NULL);
                free_items(sub, subcount);
                printf("reg/HKLM release %lu\n", IShellFolder2_Release(child));
            }
        }
        free_items(items, count);
        printf("reg release %lu\n", IShellFolder2_Release(reg));
    }
    printf("canunload %08lx\n", pCanUnloadNow());
    CoUninitialize();
    printf("probe done\n");
    return 0;
}
