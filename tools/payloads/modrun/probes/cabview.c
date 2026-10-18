/* modrun probe: cabview (the .cab shell folder), shipping Rust against the pre-flip C.
 *
 * Staged twice: as cabview_c.exe it loads cabview_c.dll (the C oracle) from its directory,
 * as cabview_probe.exe the system cabview.dll. Values only -- HRESULTs, names, attributes,
 * column text -- never a pointer or a path outside the cabinet, so the runs diff byte for
 * byte. A non-ASCII character is printed as \uXXXX so the serial log cannot mangle it.
 *
 * Covered, none of which smoke reaches (explorer opens a .cab only when someone browses
 * one): the ATL server and its COM map, IPersistFolder2 over the fixture (test.cab, see
 * fixtures/cabview/make_cab.py), EnumObjects -- which walks the cabinet through cabinet.dll's
 * FDI and extract.cpp's callbacks -- and IEnumIDList's Next/Skip/Reset, every item's display
 * names, attributes and column details (CABITEM's packed layout, the DOS date, the UTF-8
 * name flag), the column headers and their SCIDs, GetDetailsEx, CompareIDs per column,
 * ParseDisplayName, GetUIObjectOf's context menu (ItemMenuCallback and InsertMenuItem),
 * IShellIcon, BindToObject, and DllRegisterServer/DllUnregisterServer. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <stdio.h>
#include <string.h>

static const CLSID CLSID_Cab = {0x0CD7A5C0, 0x9F37, 0x11CE, {0xAE, 0x65, 0x08, 0x00, 0x2B, 0x2E, 0x12, 0x62}};
static const GUID FMTID_Stg = {0xB725F130, 0x47EF, 0x101A, {0xA5, 0xF1, 0x02, 0x60, 0x8C, 0x9E, 0xEB, 0xAC}};
#define PID_NAME 10
#define PID_SIZE 12
#define PID_ATTRIBUTES 13
#define PID_WRITETIME 14

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);

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

static void put_strret(IShellFolder2 *sf, PCUITEMID_CHILD pidl, STRRET *sr)
{
    WCHAR buf[MAX_PATH];
    HRESULT hr = StrRetToBufW(sr, pidl, buf, MAX_PATH);
    if (FAILED(hr))
        printf("<strret %08lx>", hr);
    else
        put_wide(buf);
    (void)sf;
}

static void display_names(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
{
    static const struct { SHGDNF f; const char *label; } kinds[] = {
        {SHGDN_NORMAL, "normal"},
        {SHGDN_INFOLDER, "infolder"},
        {SHGDN_INFOLDER | SHGDN_FORPARSING, "infolder-parsing"},
        {SHGDN_INFOLDER | SHGDN_FOREDITING, "infolder-editing"},
    };
    for (int k = 0; k < 4; k++)
    {
        STRRET sr;
        HRESULT hr = IShellFolder2_GetDisplayNameOf(sf, pidl, kinds[k].f, &sr);
        printf("  name %s %08lx '", kinds[k].label, hr);
        if (SUCCEEDED(hr))
            put_strret(sf, pidl, &sr);
        printf("'\n");
    }
}

static void details(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
{
    for (UINT c = 0; c < 16; c++)
    {
        SHELLDETAILS sd;
        HRESULT hr = IShellFolder2_GetDetailsOf(sf, pidl, c, &sd);
        if (hr == E_INVALIDARG)
        {
            printf("  columns %u\n", c);
            break;
        }
        printf("  col %u %08lx", c, hr);
        if (SUCCEEDED(hr))
        {
            if (!pidl)
                printf(" fmt=%d cx=%d", sd.fmt, sd.cxChar);
            printf(" '");
            put_strret(sf, pidl, &sd.str);
            printf("'");
        }
        printf("\n");
    }
}

static void details_ex(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
{
    static const DWORD pids[] = {PID_NAME, PID_SIZE, PID_ATTRIBUTES, PID_WRITETIME};
    for (int i = 0; i < 4; i++)
    {
        SHCOLUMNID scid = {FMTID_Stg, pids[i]};
        VARIANT v;
        VariantInit(&v);
        HRESULT hr = IShellFolder2_GetDetailsEx(sf, pidl, &scid, &v);
        printf("  ex pid %lu %08lx vt=%u", pids[i], hr, V_VT(&v));
        if (SUCCEEDED(hr) && V_VT(&v) == VT_BSTR)
        {
            printf(" '");
            put_wide(V_BSTR(&v));
            printf("'");
        }
        else if (SUCCEEDED(hr) && (V_VT(&v) == VT_UI4 || V_VT(&v) == VT_I4))
            printf(" %lu", V_UI4(&v));
        else if (SUCCEEDED(hr) && V_VT(&v) == VT_UI8)
            printf(" %llu", (unsigned long long)V_UI8(&v));
        else if (SUCCEEDED(hr) && V_VT(&v) == VT_DATE)
            printf(" %.6f", V_DATE(&v));
        printf("\n");
        VariantClear(&v);
    }
}

static void menu(IShellFolder2 *sf, PCUITEMID_CHILD pidl)
{
    IContextMenu *cm = NULL;
    HRESULT hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, &pidl, &IID_IContextMenu, NULL, (void **)&cm);
    printf("  contextmenu %08lx\n", hr);
    if (FAILED(hr) || !cm)
        return;
    HMENU h = CreatePopupMenu();
    hr = IContextMenu_QueryContextMenu(cm, h, 0, 1, 0x7fff, CMF_NORMAL);
    int n = GetMenuItemCount(h);
    printf("  query %08lx items=%d\n", hr, n);
    for (int i = 0; i < n; i++)
    {
        WCHAR text[128] = L"";
        MENUITEMINFOW mii = {sizeof(mii)};
        mii.fMask = MIIM_FTYPE | MIIM_ID | MIIM_STATE;
        GetMenuItemInfoW(h, i, TRUE, &mii);
        GetMenuStringW(h, i, text, 128, MF_BYPOSITION);
        printf("  item %d type=%x state=%x id=%u '", i, (unsigned)mii.fType, (unsigned)(mii.fState & MFS_DEFAULT), mii.wID);
        put_wide(text);
        printf("'\n");
    }
    DestroyMenu(h);
    IContextMenu_Release(cm);
}

int main(void)
{
    WCHAR self[MAX_PATH], cab[MAX_PATH];
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
        dll = LoadLibraryW(L"cabview.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GCO pGetClassObject = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    VOIDFN pRegister = (VOIDFN)need(dll, "DllRegisterServer");
    VOIDFN pUnregister = (VOIDFN)need(dll, "DllUnregisterServer");
    CoInitialize(NULL);

    /* The fixture is staged beside the probe. */
    lstrcpyW(cab, self);
    PathRemoveFileSpecW(cab);
    PathAppendW(cab, L"test.cab");
    printf("fixture %d\n", GetFileAttributesW(cab) != INVALID_FILE_ATTRIBUTES);

    printf("unload at start %08lx\n", pCanUnloadNow());
    IClassFactory *cf = NULL;
    printf("classobject %08lx\n", pGetClassObject(&CLSID_Cab, &IID_IClassFactory, (void **)&cf));
    if (!cf)
        return 1;
    IShellFolder2 *sf = NULL;
    printf("create %08lx\n", IClassFactory_CreateInstance(cf, NULL, &IID_IShellFolder2, (void **)&sf));
    IClassFactory_Release(cf);
    if (!sf)
        return 1;
    printf("unload with object %08lx\n", pCanUnloadNow());

    IPersistFolder2 *pf = NULL;
    printf("qi persistfolder2 %08lx\n", IShellFolder2_QueryInterface(sf, &IID_IPersistFolder2, (void **)&pf));
    IShellIcon *si = NULL;
    printf("qi shellicon %08lx\n", IShellFolder2_QueryInterface(sf, &IID_IShellIcon, (void **)&si));
    IUnknown *junk = NULL;
    printf("qi persistfile %08lx\n", IShellFolder2_QueryInterface(sf, &IID_IPersistFile, (void **)&junk));
    if (junk)
        IUnknown_Release(junk);
    if (!pf)
        return 1;
    CLSID clsid;
    HRESULT hr = IPersistFolder2_GetClassID(pf, &clsid);
    printf("classid %08lx %d\n", hr, SUCCEEDED(hr) && IsEqualCLSID(&clsid, &CLSID_Cab));

    PIDLIST_ABSOLUTE root = ILCreateFromPathW(cab);
    printf("initialize %08lx\n", IPersistFolder2_Initialize(pf, root));
    PIDLIST_ABSOLUTE cur = NULL;
    hr = IPersistFolder2_GetCurFolder(pf, &cur);
    printf("curfolder %08lx same=%d\n", hr, cur && ILIsEqual(cur, root));
    ILFree(cur);

    /* Column headers: GetDetailsOf with no item. */
    printf("headers\n");
    details(sf, NULL);
    ULONG sort = 99, disp = 99;
    printf("defaultcolumn %08lx sort=%lu display=%lu\n", IShellFolder2_GetDefaultColumn(sf, 0, &sort, &disp), sort, disp);
    for (UINT c = 0; c < 8; c++)
    {
        SHCOLSTATEF st = 0;
        SHCOLUMNID scid = {{0}};
        HRESULT h1 = IShellFolder2_GetDefaultColumnState(sf, c, &st);
        HRESULT h2 = IShellFolder2_MapColumnToSCID(sf, c, &scid);
        printf("colstate %u %08lx %lx scid %08lx stg=%d pid=%lu\n", c, h1, st, h2,
               IsEqualGUID(&scid.fmtid, &FMTID_Stg), scid.pid);
    }

    IEnumIDList *en = NULL;
    hr = IShellFolder2_EnumObjects(sf, NULL, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS | SHCONTF_INCLUDEHIDDEN, &en);
    printf("enum %08lx\n", hr);
    PITEMID_CHILD items[16];
    ULONG count = 0;
    if (en)
    {
        ULONG got = 0;
        PITEMID_CHILD two[2] = {NULL, NULL};
        hr = IEnumIDList_Next(en, 2, two, &got);
        printf("next2 %08lx got=%lu\n", hr, got);
        for (ULONG i = 0; i < got; i++)
            ILFree(two[i]);
        printf("skip1 %08lx\n", IEnumIDList_Skip(en, 1));
        printf("reset %08lx\n", IEnumIDList_Reset(en));
        while (count < 16 && IEnumIDList_Next(en, 1, &items[count], &got) == S_OK && got == 1)
            count++;
        printf("skip past end %08lx\n", IEnumIDList_Skip(en, 5));
        IEnumIDList_Release(en);
    }
    printf("items %lu\n", count);
    for (ULONG i = 0; i < count; i++)
    {
        printf("item %lu\n", i);
        display_names(sf, items[i]);
        SFGAOF att = SFGAO_FOLDER | SFGAO_STREAM | SFGAO_READONLY | SFGAO_HIDDEN | SFGAO_SYSTEM |
                     SFGAO_CANCOPY | SFGAO_CANMOVE | SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET;
        PCUITEMID_CHILD one = items[i];
        hr = IShellFolder2_GetAttributesOf(sf, 1, &one, &att);
        printf("  attributes %08lx %lx\n", hr, att);
        details(sf, items[i]);
        details_ex(sf, items[i]);
        if (si)
        {
            int index = -2;
            hr = IShellIcon_GetIconOf(si, items[i], GIL_FORSHELL, &index);
            printf("  iconof %08lx valid=%d\n", hr, index >= 0);
        }
    }
    if (count >= 2)
    {
        for (int c = 0; c < 7; c++)
        {
            HRESULT a = IShellFolder2_CompareIDs(sf, c, items[0], items[1]);
            HRESULT b = IShellFolder2_CompareIDs(sf, c, items[1], items[0]);
            HRESULT s = IShellFolder2_CompareIDs(sf, c, items[0], items[0]);
            printf("compare col %d %d %d %d\n", c, (short)HRESULT_CODE(a), (short)HRESULT_CODE(b),
                   (short)HRESULT_CODE(s));
        }
        /* SHCIDS_ALLFIELDS: the C reports two different items EQUAL (folder.cpp's loop zeroes
           its result at COL_NAME and stops); the Rust orders them (a declared divergence). */
        HRESULT all = IShellFolder2_CompareIDs(sf, SHCIDS_ALLFIELDS, items[0], items[1]);
        HRESULT self_ = IShellFolder2_CompareIDs(sf, SHCIDS_ALLFIELDS, items[0], items[0]);
        printf("compare allfields %d\n", (short)HRESULT_CODE(all));
        printf("compare allfields self %d\n", (short)HRESULT_CODE(self_));
        menu(sf, items[0]);
    }
    if (count >= 1)
    {
        PCUITEMID_CHILD both[2] = {items[0], count >= 2 ? items[1] : items[0]};
        SFGAOF att = SFGAO_FOLDER | SFGAO_STREAM | SFGAO_READONLY;
        printf("attributes of two %08lx %lx\n", IShellFolder2_GetAttributesOf(sf, 2, both, &att), att);
        IDataObject *dobj = NULL;
        hr = IShellFolder2_GetUIObjectOf(sf, NULL, 2, both, &IID_IDataObject, NULL, (void **)&dobj);
        printf("dataobject %08lx\n", hr);
        if (dobj)
            IDataObject_Release(dobj);
        IExtractIconW *ei = NULL;
        hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, both, &IID_IExtractIconW, NULL, (void **)&ei);
        printf("extracticon %08lx\n", hr);
        if (ei)
            IUnknown_Release((IUnknown *)ei);
        void *bound = NULL;
        printf("bindtoobject %08lx\n", IShellFolder2_BindToObject(sf, items[0], NULL, &IID_IShellFolder, &bound));
    }

    static const WCHAR *parse[] = {L"readme.txt", L"docs\\notes.txt", L"nope.txt"};
    for (int i = 0; i < 3; i++)
    {
        ULONG eaten = 0;
        PIDLIST_RELATIVE p = NULL;
        SFGAOF att = SFGAO_FOLDER | SFGAO_READONLY;
        hr = IShellFolder2_ParseDisplayName(sf, NULL, NULL, (LPWSTR)parse[i], &eaten, &p, &att);
        printf("parse '");
        put_wide(parse[i]);
        printf("' %08lx found=%d att=%lx\n", hr, p != NULL, SUCCEEDED(hr) ? att : 0);
        if (p)
        {
            display_names(sf, (PCUITEMID_CHILD)p);
            ILFree(p);
        }
    }

    for (ULONG i = 0; i < count; i++)
        ILFree(items[i]);
    if (si)
        IShellIcon_Release(si);
    IPersistFolder2_Release(pf);
    printf("release %lu\n", IShellFolder2_Release(sf));
    ILFree(root);
    printf("unload at end %08lx\n", pCanUnloadNow());

    printf("register %08lx\n", pRegister());
    HKEY k;
    LONG e = RegOpenKeyExW(HKEY_CLASSES_ROOT, L"CLSID\\{0CD7A5C0-9F37-11CE-AE65-08002B2E1262}\\InprocServer32", 0, KEY_READ, &k);
    printf("inprocserver32 %s\n", e == ERROR_SUCCESS ? "present" : "absent");
    if (e == ERROR_SUCCESS)
        RegCloseKey(k);
    printf("unregister %08lx\n", pUnregister());
    /* Put the registration back: the Rust run is last, so the system DLL ends registered. */
    printf("register again %08lx\n", pRegister());
    CoUninitialize();
    printf("probe done\n");
    return 0;
}
