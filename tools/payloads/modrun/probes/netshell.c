/* modrun probe: netshell (Network Connections), shipping Rust against the pre-flip C.
 *
 * Staged twice: as netshell_c.exe it loads netshell_c.dll (the C oracle) from its directory,
 * as netshell_probe.exe the system netshell.dll. Values only -- HRESULTs, names, statuses,
 * column text, counts; GUIDs only as "equal to" checks -- so the runs diff byte for byte.
 *
 * Explorer creates netshell's ConnectionTray at every boot (through stobject), so smoke
 * reaches its startup; this drives the rest: the ATL server's four classes, the connection
 * manager's enumeration over the guest's adapters and each connection's properties
 * (NcFreeNetconProperties frees them), the Network Connections shell folder (enumeration,
 * names, columns, compares, attributes), NcIsValidConnectionName, the wizard-page count of
 * NetSetupRequestWizardPages, and DllCanUnloadNow. Nothing is renamed, connected or shown.
 * netcon.h's interfaces are declared here: the probe builds against mingw's headers. */
#define COBJMACROS
#include <windows.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <stdio.h>

static const CLSID kNetworkConnections = {0x7007ACC7, 0x3202, 0x11D1, {0xAA, 0xD2, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E}};
static const CLSID kLanConnectionUi = {0x7007ACC5, 0x3202, 0x11D1, {0xAA, 0xD2, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E}};
static const CLSID kConnectionTray = {0x7007ACCF, 0x3202, 0x11D1, {0xAA, 0xD2, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E}};
static const CLSID kConnectionManager = {0xBA126AD1, 0x2166, 0x11D1, {0xB1, 0xD0, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E}};
static const IID IID_INetConnectionManager_ = {0xC08956A2, 0x1CD3, 0x11D1, {0xB1, 0xC5, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E}};
static const IID IID_IOleCommandTarget_ = {0xb722bccb, 0x4e68, 0x101b, {0xa2, 0xbc, 0x00, 0xaa, 0x00, 0x40, 0x47, 0x70}};

typedef struct
{
    GUID guidId;
    LPWSTR pszwName;
    LPWSTR pszwDeviceName;
    int Status;
    int MediaType;
    DWORD dwCharacter;
    CLSID clsidThisObject;
    CLSID clsidUiObject;
} NETCON_PROPERTIES;

typedef struct NetConnection NetConnection;
typedef struct
{
    HRESULT(STDMETHODCALLTYPE *QueryInterface)(NetConnection *, REFIID, void **);
    ULONG(STDMETHODCALLTYPE *AddRef)(NetConnection *);
    ULONG(STDMETHODCALLTYPE *Release)(NetConnection *);
    HRESULT(STDMETHODCALLTYPE *Connect)(NetConnection *);
    HRESULT(STDMETHODCALLTYPE *Disconnect)(NetConnection *);
    HRESULT(STDMETHODCALLTYPE *Delete)(NetConnection *);
    HRESULT(STDMETHODCALLTYPE *Duplicate)(NetConnection *, LPCWSTR, NetConnection **);
    HRESULT(STDMETHODCALLTYPE *GetProperties)(NetConnection *, NETCON_PROPERTIES **);
    HRESULT(STDMETHODCALLTYPE *GetUiObjectClassId)(NetConnection *, CLSID *);
    HRESULT(STDMETHODCALLTYPE *Rename)(NetConnection *, LPCWSTR);
} NetConnectionVtbl;
struct NetConnection { const NetConnectionVtbl *lpVtbl; };

typedef struct EnumNetConnection EnumNetConnection;
typedef struct
{
    HRESULT(STDMETHODCALLTYPE *QueryInterface)(EnumNetConnection *, REFIID, void **);
    ULONG(STDMETHODCALLTYPE *AddRef)(EnumNetConnection *);
    ULONG(STDMETHODCALLTYPE *Release)(EnumNetConnection *);
    HRESULT(STDMETHODCALLTYPE *Next)(EnumNetConnection *, ULONG, NetConnection **, ULONG *);
    HRESULT(STDMETHODCALLTYPE *Skip)(EnumNetConnection *, ULONG);
    HRESULT(STDMETHODCALLTYPE *Reset)(EnumNetConnection *);
    HRESULT(STDMETHODCALLTYPE *Clone)(EnumNetConnection *, EnumNetConnection **);
} EnumNetConnectionVtbl;
struct EnumNetConnection { const EnumNetConnectionVtbl *lpVtbl; };

typedef struct NetConnectionManager NetConnectionManager;
typedef struct
{
    HRESULT(STDMETHODCALLTYPE *QueryInterface)(NetConnectionManager *, REFIID, void **);
    ULONG(STDMETHODCALLTYPE *AddRef)(NetConnectionManager *);
    ULONG(STDMETHODCALLTYPE *Release)(NetConnectionManager *);
    HRESULT(STDMETHODCALLTYPE *EnumConnections)(NetConnectionManager *, int, EnumNetConnection **);
} NetConnectionManagerVtbl;
struct NetConnectionManager { const NetConnectionManagerVtbl *lpVtbl; };

typedef HRESULT(WINAPI *GCO)(REFCLSID, REFIID, void **);
typedef HRESULT(WINAPI *VOIDFN)(void);
typedef VOID(WINAPI *FREEPROPS)(NETCON_PROPERTIES *);
typedef BOOL(WINAPI *VALIDNAME)(PCWSTR);
typedef DWORD(WINAPI *WIZPAGES)(PDWORD, HPROPSHEETPAGE *, void *);

#define MAX_ITEMS 16

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
    if (FAILED(StrRetToBufW(sr, pidl, buf, MAX_PATH)))
        printf("<strret>");
    else
        put_wide(buf);
}

static IUnknown *create(GCO gco, const CLSID *clsid, const IID *iid, const char *label)
{
    IClassFactory *cf = NULL;
    HRESULT hr = gco(clsid, &IID_IClassFactory, (void **)&cf);
    printf("%s classobject %08lx\n", label, hr);
    if (!cf)
        return NULL;
    IUnknown *obj = NULL;
    hr = IClassFactory_CreateInstance(cf, NULL, iid, (void **)&obj);
    printf("%s create %08lx\n", label, hr);
    IClassFactory_Release(cf);
    return obj;
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
        dll = LoadLibraryW(L"netshell.dll");
    if (!dll)
    {
        printf("LoadLibrary failed %lu\n", GetLastError());
        return 1;
    }
    GCO gco = (GCO)need(dll, "DllGetClassObject");
    VOIDFN pCanUnloadNow = (VOIDFN)need(dll, "DllCanUnloadNow");
    FREEPROPS pFree = (FREEPROPS)need(dll, "NcFreeNetconProperties");
    VALIDNAME pValid = (VALIDNAME)need(dll, "NcIsValidConnectionName");
    WIZPAGES pPages = (WIZPAGES)need(dll, "NetSetupRequestWizardPages");
    need(dll, "DllRegisterServer");
    need(dll, "DllUnregisterServer");
    CoInitialize(NULL);

    static const WCHAR *names[] = {L"Local Area Connection", L"a/b", L"   ", L"x:y", L"ok name", L""};
    for (int i = 0; i < 6; i++)
    {
        printf("validname %d '", i);
        put_wide(names[i]);
        printf("' %d\n", pValid(names[i]));
    }
    printf("validname null %d\n", pValid(NULL));
    DWORD pages = 0;
    printf("wizardpages %lu count %lu\n", pPages(&pages, NULL, NULL), pages);
    printf("wizardpages nullcount %lu\n", pPages(NULL, NULL, NULL));

    /* The connection manager over the guest's adapters. */
    NetConnectionManager *mgr =
        (NetConnectionManager *)create(gco, &kConnectionManager, &IID_INetConnectionManager_,
                                       "manager");
    if (mgr)
    {
        EnumNetConnection *en = NULL;
        HRESULT hr = mgr->lpVtbl->EnumConnections(mgr, 0 /* NCME_DEFAULT */, &en);
        printf("manager enum %08lx\n", hr);
        if (en)
        {
            NetConnection *c;
            ULONG got, n = 0;
            while (n < MAX_ITEMS && en->lpVtbl->Next(en, 1, &c, &got) == S_OK && got == 1)
            {
                NETCON_PROPERTIES *p = NULL;
                hr = c->lpVtbl->GetProperties(c, &p);
                printf("connection %lu props %08lx", n, hr);
                if (p)
                {
                    printf(" name '");
                    put_wide(p->pszwName);
                    printf("' device '");
                    put_wide(p->pszwDeviceName);
                    printf("' status %d media %d character %08lx thisclsid-tray %d\n", p->Status,
                           p->MediaType, p->dwCharacter,
                           IsEqualCLSID(&p->clsidUiObject, &kLanConnectionUi));
                    pFree(p);
                }
                else
                    printf("\n");
                CLSID ui;
                hr = c->lpVtbl->GetUiObjectClassId(c, &ui);
                printf("connection %lu uiclass %08lx lanui=%d\n", n, hr,
                       SUCCEEDED(hr) && IsEqualCLSID(&ui, &kLanConnectionUi));
                printf("connection %lu release %lu\n", n, c->lpVtbl->Release(c));
                n++;
            }
            printf("manager connections %lu\n", n);
            printf("manager reset %08lx\n", en->lpVtbl->Reset(en));
            printf("manager skip %08lx\n", en->lpVtbl->Skip(en, 1));
            en->lpVtbl->Release(en);
        }
        printf("manager release %lu\n", mgr->lpVtbl->Release(mgr));
    }

    /* The Network Connections folder. */
    IShellFolder2 *sf = (IShellFolder2 *)create(gco, &kNetworkConnections,
                                                &IID_IShellFolder2, "folder");
    if (sf)
    {
        IPersistFolder2 *pf = NULL;
        HRESULT hr = IShellFolder2_QueryInterface(sf, &IID_IPersistFolder2, (void **)&pf);
        printf("folder qi persistfolder2 %08lx\n", hr);
        if (pf)
        {
            LPITEMIDLIST cp = NULL;
            SHGetSpecialFolderLocation(NULL, CSIDL_CONTROLS, &cp);
            printf("folder initialize %08lx\n", IPersistFolder2_Initialize(pf, cp));
            CoTaskMemFree(cp);
            IPersistFolder2_Release(pf);
        }
        for (UINT col = 0; col < 8; col++)
        {
            SHELLDETAILS sd;
            hr = IShellFolder2_GetDetailsOf(sf, NULL, col, &sd);
            printf("folder header %u %08lx", col, hr);
            if (FAILED(hr))
            {
                printf("\n");
                break;
            }
            printf(" fmt=%d cx=%d '", sd.fmt, sd.cxChar);
            put_strret(&sd.str, NULL);
            printf("'\n");
        }
        IEnumIDList *en = NULL;
        hr = IShellFolder2_EnumObjects(sf, NULL, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, &en);
        printf("folder enum %08lx\n", hr);
        LPITEMIDLIST items[MAX_ITEMS];
        ULONG count = 0, got;
        if (en)
        {
            while (count < MAX_ITEMS && IEnumIDList_Next(en, 1, &items[count], &got) == S_OK)
                count++;
            IEnumIDList_Release(en);
        }
        printf("folder items %lu\n", count);
        for (ULONG i = 0; i < count; i++)
        {
            STRRET sr;
            hr = IShellFolder2_GetDisplayNameOf(sf, items[i], SHGDN_NORMAL, &sr);
            printf("folder item %lu name %08lx '", i, hr);
            if (SUCCEEDED(hr))
                put_strret(&sr, items[i]);
            SFGAOF attrs = SFGAO_CANRENAME | SFGAO_CANDELETE | SFGAO_HASPROPSHEET |
                           SFGAO_CANLINK | SFGAO_FOLDER;
            hr = IShellFolder2_GetAttributesOf(sf, 1, (PCUITEMID_CHILD_ARRAY)&items[i], &attrs);
            printf("' attrs %08lx %08lx\n", hr, attrs);
            for (UINT col = 0; col < 8; col++)
            {
                SHELLDETAILS sd;
                hr = IShellFolder2_GetDetailsOf(sf, items[i], col, &sd);
                if (FAILED(hr))
                    break;
                printf("  col %u '", col);
                put_strret(&sd.str, items[i]);
                printf("'\n");
            }
        }
        if (count)
        {
            printf("folder compare self %d\n",
                   (short)HRESULT_CODE(IShellFolder2_CompareIDs(sf, 0, items[0], items[0])));
            printf("folder compare allfields self %d\n",
                   (short)HRESULT_CODE(
                       IShellFolder2_CompareIDs(sf, SHCIDS_ALLFIELDS, items[0], items[0])));
            IContextMenu *cm = NULL;
            hr = IShellFolder2_GetUIObjectOf(sf, NULL, 1, (PCUITEMID_CHILD_ARRAY)&items[0],
                                             &IID_IContextMenu, NULL, (void **)&cm);
            printf("folder contextmenu %08lx\n", hr);
            if (cm)
                IContextMenu_Release(cm);
        }
        for (ULONG i = 0; i < count; i++)
            CoTaskMemFree(items[i]);
        printf("folder release %lu\n", IShellFolder2_Release(sf));
    }

    /* The other two classes: created and released, nothing shown or started. */
    IUnknown *tray = create(gco, &kConnectionTray, &IID_IOleCommandTarget_, "tray");
    if (tray)
        printf("tray release %lu\n", IUnknown_Release(tray));
    IUnknown *ui = create(gco, &kLanConnectionUi, &IID_IUnknown, "lanui");
    if (ui)
        printf("lanui release %lu\n", IUnknown_Release(ui));

    printf("canunload %08lx\n", pCanUnloadNow());
    CoUninitialize();
    printf("probe done\n");
    return 0;
}
