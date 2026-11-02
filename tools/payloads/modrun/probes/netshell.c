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

/* The manager's enumeration (CNetConnectionManager::EnumerateINetConnections) has a silent
 * `break` at every step, so 0 connections says nothing about which one failed. Replay its
 * steps against the guest and print each outcome; both variants see the same machine, so the
 * lines match and say where a connection is lost. iphlpapi and setupapi are loaded by name:
 * the probe links neither. */
typedef DWORD(WINAPI *GETIFTABLE)(void *, ULONG *, BOOL);
typedef DWORD(WINAPI *GETADAPTERSINFO)(void *, ULONG *);
typedef void *(WINAPI *GETCLASSDEVS)(const GUID *, PCWSTR, HWND, DWORD);
typedef BOOL(WINAPI *ENUMDEVINFO)(void *, DWORD, void *);
typedef BOOL(WINAPI *GETDEVREGPROP)(void *, void *, DWORD, DWORD *, BYTE *, DWORD, DWORD *);
typedef BOOL(WINAPI *DESTROYLIST)(void *);

/* Only the fields read here; IP_ADAPTER_INFO's prefix is the same on every arch. */
typedef struct AdapterInfo
{
    struct AdapterInfo *Next;
    DWORD ComboIndex;
    char AdapterName[260];
    char Description[132];
    UINT AddressLength;
    BYTE Address[8];
    DWORD Index;
} AdapterInfo;

typedef struct
{
    DWORD cbSize;
    GUID ClassGuid;
    DWORD DevInst;
    ULONG_PTR Reserved;
} DevInfoData;

static void replay_enumeration(void)
{
    static const GUID kNetClass = {0x4d36e972, 0xe325, 0x11ce, {0xbf, 0xc1, 0x08, 0x00, 0x2b, 0xe1, 0x03, 0x18}};
    HMODULE ip = LoadLibraryW(L"iphlpapi.dll"), sa = LoadLibraryW(L"setupapi.dll");
    GETIFTABLE pGetIfTable = (GETIFTABLE)GetProcAddress(ip, "GetIfTable");
    GETADAPTERSINFO pGetAdaptersInfo = (GETADAPTERSINFO)GetProcAddress(ip, "GetAdaptersInfo");
    GETCLASSDEVS pGetClassDevs = (GETCLASSDEVS)GetProcAddress(sa, "SetupDiGetClassDevsW");
    ENUMDEVINFO pEnum = (ENUMDEVINFO)GetProcAddress(sa, "SetupDiEnumDeviceInfo");
    GETDEVREGPROP pProp = (GETDEVREGPROP)GetProcAddress(sa, "SetupDiGetDeviceRegistryPropertyW");
    DESTROYLIST pDestroy = (DESTROYLIST)GetProcAddress(sa, "SetupDiDestroyDeviceInfoList");
    if (!pGetIfTable || !pGetAdaptersInfo || !pGetClassDevs || !pEnum || !pProp || !pDestroy)
    {
        printf("replay imports missing\n");
        return;
    }
    ULONG size = 0;
    DWORD rc = pGetIfTable(NULL, &size, TRUE);
    printf("replay iftable probe %lu\n", rc);
    BYTE *iftable = (BYTE *)HeapAlloc(GetProcessHeap(), 0, size ? size : 1);
    rc = pGetIfTable(iftable, &size, TRUE);
    printf("replay iftable %lu entries %lu\n", rc, rc ? 0 : *(DWORD *)iftable);
    HeapFree(GetProcessHeap(), 0, iftable);

    size = 0;
    rc = pGetAdaptersInfo(NULL, &size);
    printf("replay adaptersinfo probe %lu\n", rc);
    AdapterInfo *adapters = (AdapterInfo *)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size ? size : 1);
    rc = size ? pGetAdaptersInfo(adapters, &size) : rc;
    printf("replay adaptersinfo %lu\n", rc);
    for (AdapterInfo *a = rc ? NULL : adapters; a; a = a->Next)
        printf("replay adapter index %lu name %s\n", a->Index, a->AdapterName);

    void *info = pGetClassDevs(&kNetClass, NULL, NULL, 2 /* DIGCF_PRESENT */);
    printf("replay classdevs %s\n", info && info != INVALID_HANDLE_VALUE ? "ok" : "failed");
    if (!info || info == INVALID_HANDLE_VALUE)
        return;
    for (DWORD i = 0;; i++)
    {
        DevInfoData dev;
        ZeroMemory(&dev, sizeof(dev));
        dev.cbSize = sizeof(dev);
        if (!pEnum(info, i, &dev))
        {
            printf("replay devices %lu (stop %lu)\n", i, GetLastError());
            break;
        }
        WCHAR driver[200];
        DWORD need = 0;
        if (!pProp(info, &dev, 9 /* SPDRP_DRIVER */, NULL, (BYTE *)driver, sizeof(driver), &need))
        {
            printf("replay device %lu driver failed %lu\n", i, GetLastError());
            continue;
        }
        WCHAR key[260] = L"SYSTEM\\CurrentControlSet\\Control\\Class\\";
        lstrcatW(key, driver);
        HKEY hk;
        LONG lr = RegOpenKeyExW(HKEY_LOCAL_MACHINE, key, 0, KEY_READ, &hk);
        printf("replay device %lu driver %ls open %ld\n", i, driver, lr);
        if (lr)
            continue;
        WCHAR cfg[50] = L"";
        DWORD cb = sizeof(cfg);
        lr = RegQueryValueExW(hk, L"NetCfgInstanceId", NULL, NULL, (BYTE *)cfg, &cb);
        RegCloseKey(hk);
        printf("replay device %lu netcfg %ld %ls\n", i, lr, cfg);
        int matched = 0;
        for (AdapterInfo *a = rc ? NULL : adapters; a; a = a->Next)
        {
            WCHAR name[50];
            MultiByteToWideChar(CP_ACP, 0, a->AdapterName, -1, name, 50);
            if (!lstrcmpiW(name, cfg))
                matched = 1;
        }
        printf("replay device %lu adapter match %d\n", i, matched);
    }
    pDestroy(info);
    HeapFree(GetProcessHeap(), 0, adapters);

    /* Where the NIC is, if not in the Net class: every present device that is
     * a PCI network controller (class code 02, in its compatible ids), with its setup class
     * and service. */
    info = pGetClassDevs(NULL, NULL, NULL, 2 /* DIGCF_PRESENT */ | 4 /* DIGCF_ALLCLASSES */);
    if (!info || info == INVALID_HANDLE_VALUE)
    {
        printf("replay alldevs failed\n");
        return;
    }
    for (DWORD i = 0;; i++)
    {
        DevInfoData dev;
        ZeroMemory(&dev, sizeof(dev));
        dev.cbSize = sizeof(dev);
        if (!pEnum(info, i, &dev))
            break;
        WCHAR hwid[512] = L"", compat[1024] = L"", cls[64] = L"", svc[64] = L"";
        pProp(info, &dev, 1 /* SPDRP_HARDWAREID */, NULL, (BYTE *)hwid, sizeof(hwid), NULL);
        pProp(info, &dev, 2 /* SPDRP_COMPATIBLEIDS */, NULL, (BYTE *)compat, sizeof(compat), NULL);
        BOOL network = FALSE;
        for (WCHAR *id = compat; *id; id += lstrlenW(id) + 1)
            network |= wcsncmp(id, L"PCI\\CC_02", 9) == 0;
        if (!network)
            continue;
        pProp(info, &dev, 7 /* SPDRP_CLASS */, NULL, (BYTE *)cls, sizeof(cls), NULL);
        pProp(info, &dev, 4 /* SPDRP_SERVICE */, NULL, (BYTE *)svc, sizeof(svc), NULL);
        printf("replay nic %ls class '%ls' service '%ls' classguid %08lx\n", hwid, cls, svc,
               dev.ClassGuid.Data1);
    }
    pDestroy(info);
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
    replay_enumeration();

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
