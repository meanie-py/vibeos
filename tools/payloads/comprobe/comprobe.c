/* comprobe — where does cross-thread COM marshalling break, and does SHObjectProperties hang?
 *
 * 1. For a list of interfaces: the HKCR ProxyStubClsid32 value, CoGetPSClsid, and
 *    CoGetClassObject(that clsid, IID_IPSFactoryBuffer) — the two halves of ole32's
 *    get_facbuf_for_iid.
 * 2. CoMarshalInterThreadInterfaceInStream of a minimal IDataObject, then unmarshal on a
 *    second STA thread and call through the proxy.
 * 3. SHObjectProperties(SHOP_FILEPATH) on explorer.exe with a watchdog that dumps this
 *    process's top-level windows after 20s and closes any property sheet it finds.
 * Verdict: "[COMPROBE] done VERDICT=PASS|FAIL". */
#define COBJMACROS
#include <windows.h>
#include <objbase.h>
#include <ole2.h>
#include <shlobj.h>
#include <stdio.h>
#include <tlhelp32.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[COMPROBE] " __VA_ARGS__)

static int failures;

/* ---- a minimal IDataObject ---- */
typedef struct { IDataObject iface; LONG ref; } DObj;
static HRESULT STDMETHODCALLTYPE d_qi(IDataObject *t, REFIID r, void **p)
{
    if (IsEqualIID(r, &IID_IUnknown) || IsEqualIID(r, &IID_IDataObject)) {
        *p = t; IDataObject_AddRef(t); return S_OK;
    }
    *p = NULL; return E_NOINTERFACE;
}
static ULONG STDMETHODCALLTYPE d_addref(IDataObject *t) { return InterlockedIncrement(&((DObj *)t)->ref); }
static ULONG STDMETHODCALLTYPE d_release(IDataObject *t) { return InterlockedDecrement(&((DObj *)t)->ref); }
static HRESULT STDMETHODCALLTYPE d_getdata(IDataObject *t, FORMATETC *f, STGMEDIUM *m) { return DV_E_FORMATETC; }
static HRESULT STDMETHODCALLTYPE d_getdatahere(IDataObject *t, FORMATETC *f, STGMEDIUM *m) { return E_NOTIMPL; }
static HRESULT STDMETHODCALLTYPE d_querygetdata(IDataObject *t, FORMATETC *f) { return 0x4242; /* marker */ }
static HRESULT STDMETHODCALLTYPE d_canon(IDataObject *t, FORMATETC *a, FORMATETC *b) { return E_NOTIMPL; }
static HRESULT STDMETHODCALLTYPE d_setdata(IDataObject *t, FORMATETC *f, STGMEDIUM *m, BOOL r) { return E_NOTIMPL; }
static HRESULT STDMETHODCALLTYPE d_enum(IDataObject *t, DWORD d, IEnumFORMATETC **e) { return E_NOTIMPL; }
static HRESULT STDMETHODCALLTYPE d_dadvise(IDataObject *t, FORMATETC *f, DWORD a, IAdviseSink *s, DWORD *c) { return E_NOTIMPL; }
static HRESULT STDMETHODCALLTYPE d_dunadvise(IDataObject *t, DWORD c) { return E_NOTIMPL; }
static HRESULT STDMETHODCALLTYPE d_enumadvise(IDataObject *t, IEnumSTATDATA **e) { return E_NOTIMPL; }
static IDataObjectVtbl dvt = { d_qi, d_addref, d_release, d_getdata, d_getdatahere, d_querygetdata,
                               d_canon, d_setdata, d_enum, d_dadvise, d_dunadvise, d_enumadvise };
static DObj g_obj = { { &dvt }, 1 };

static void probe_iid(const char *name, REFIID iid)
{
    WCHAR key[80], val[64] = L"";
    DWORD cb = sizeof val;
    CLSID ps;
    IUnknown *fac = NULL;
    HRESULT hps, hco = E_FAIL;
    LONG rr;
    StringFromGUID2(iid, key + 10, 40);
    memcpy(key, L"Interface\\", 20);
    wcscat(key, L"\\ProxyStubClsid32");
    rr = RegGetValueW(HKEY_CLASSES_ROOT, key, NULL, RRF_RT_REG_SZ, NULL, val, &cb);
    hps = CoGetPSClsid(iid, &ps);
    if (hps == S_OK)
        hco = CoGetClassObject(&ps, CLSCTX_INPROC_SERVER, NULL, &IID_IPSFactoryBuffer, (void **)&fac);
    if (fac) IUnknown_Release(fac);
    SAY("psfactory %-16s reg=%ld(%ls) CoGetPSClsid=%08lx CoGetClassObject=%08lx\n",
        name, rr, rr == 0 ? val : L"-", hps, hco);
    if (hps != S_OK || hco != S_OK) failures++;
}

static IStream *g_stm;
static HRESULT g_thread_hr = E_PENDING, g_call_hr;
static DWORD WINAPI unmarshal_thread(void *arg)
{
    IDataObject *p = NULL;
    CoInitialize(NULL);
    g_thread_hr = CoGetInterfaceAndReleaseStream(g_stm, &IID_IDataObject, (void **)&p);
    if (p) { g_call_hr = IDataObject_QueryGetData(p, NULL); IDataObject_Release(p); }
    CoUninitialize();
    return 0;
}

static void probe_marshal(void)
{
    HRESULT hr = CoMarshalInterThreadInterfaceInStream(&IID_IDataObject, (IUnknown *)&g_obj.iface, &g_stm);
    SAY("marshal IDataObject hr=%08lx\n", hr);
    if (hr != S_OK) { failures++; return; }
    HANDLE t = CreateThread(NULL, 0, unmarshal_thread, NULL, 0, NULL);
    DWORD w, deadline = GetTickCount() + 20000;
    /* this STA must pump for the cross-apartment call to be served */
    while ((w = MsgWaitForMultipleObjects(1, &t, FALSE, 500, QS_ALLINPUT)) != WAIT_OBJECT_0
           && GetTickCount() < deadline) {
        MSG m;
        while (PeekMessageW(&m, NULL, 0, 0, PM_REMOVE)) DispatchMessageW(&m);
    }
    SAY("unmarshal hr=%08lx proxied QueryGetData=%08lx (want 00004242) %s\n", g_thread_hr, g_call_hr,
        w == WAIT_OBJECT_0 ? "" : "TIMEOUT");
    if (g_thread_hr != S_OK || g_call_hr != 0x4242) failures++;
}

/* The data object shell32 itself would marshal for SHObjectProperties: does it answer QIs it
 * should not (IMarshal would switch ole32 to custom marshalling), and does it round-trip? */
static IStream *g_stm2;
static HRESULT g_thr2 = E_PENDING, g_call2 = E_PENDING;
static DWORD WINAPI unmarshal_thread2(void *arg)
{
    IDataObject *p = NULL;
    CoInitialize(NULL);
    g_thr2 = CoGetInterfaceAndReleaseStream(g_stm2, &IID_IDataObject, (void **)&p);
    if (p) {
        FORMATETC fe = { CF_HDROP, NULL, DVASPECT_CONTENT, -1, TYMED_HGLOBAL };
        g_call2 = IDataObject_QueryGetData(p, &fe);
        IDataObject_Release(p);
    }
    CoUninitialize();
    return 0;
}

static void probe_shell_dataobject(void)
{
    WCHAR path[MAX_PATH];
    PIDLIST_ABSOLUTE pidl = NULL;
    IShellFolder *parent = NULL;
    PCUITEMID_CHILD child;
    IDataObject *pdo = NULL;
    GetWindowsDirectoryW(path, MAX_PATH);
    wcscat(path, L"\\explorer.exe");
    HRESULT hr = SHParseDisplayName(path, NULL, &pidl, 0, NULL);
    if (hr == S_OK) hr = SHBindToParent(pidl, &IID_IShellFolder, (void **)&parent, &child);
    if (hr == S_OK) hr = IShellFolder_GetUIObjectOf(parent, NULL, 1, &child, &IID_IDataObject, NULL, (void **)&pdo);
    SAY("shell dataobject hr=%08lx\n", hr);
    if (!pdo) { failures++; return; }
    static const GUID IID_Bogus = { 0x12345678, 0x1234, 0x1234, { 1, 2, 3, 4, 5, 6, 7, 8 } };
    IUnknown *u;
    SAY("  QI IMarshal=%08lx IStdMarshalInfo=%08lx bogus=%08lx\n",
        IDataObject_QueryInterface(pdo, &IID_IMarshal, (void **)&u) == S_OK ? (IUnknown_Release(u), S_OK) : E_NOINTERFACE,
        IDataObject_QueryInterface(pdo, &IID_IStdMarshalInfo, (void **)&u) == S_OK ? (IUnknown_Release(u), S_OK) : E_NOINTERFACE,
        IDataObject_QueryInterface(pdo, &IID_Bogus, (void **)&u) == S_OK ? (IUnknown_Release(u), S_OK) : E_NOINTERFACE);
    hr = CoMarshalInterThreadInterfaceInStream(&IID_IDataObject, (IUnknown *)pdo, &g_stm2);
    SAY("  marshal hr=%08lx\n", hr);
    if (hr == S_OK) {
        HANDLE t = CreateThread(NULL, 0, unmarshal_thread2, NULL, 0, NULL);
        DWORD w, deadline = GetTickCount() + 20000;
        while ((w = MsgWaitForMultipleObjects(1, &t, FALSE, 500, QS_ALLINPUT)) != WAIT_OBJECT_0
               && GetTickCount() < deadline) {
            MSG m;
            while (PeekMessageW(&m, NULL, 0, 0, PM_REMOVE)) DispatchMessageW(&m);
        }
        SAY("  unmarshal hr=%08lx proxied QueryGetData(CF_HDROP)=%08lx %s\n", g_thr2, g_call2,
            w == WAIT_OBJECT_0 ? "" : "TIMEOUT");
        if (g_thr2 != S_OK || g_call2 != S_OK) failures++;
    } else failures++;
    IDataObject_Release(pdo);
}

static BOOL CALLBACK dump_win(HWND h, LPARAM lp)
{
    DWORD pid; WCHAR cls[64], title[128];
    GetWindowThreadProcessId(h, &pid);
    if (pid != GetCurrentProcessId()) return TRUE;
    /* InternalGetWindowText: no WM_GETTEXT, so a window whose thread is stuck cannot stall us. */
    GetClassNameW(h, cls, 64); InternalGetWindowText(h, title, 128);
    SAY("  window %p class=%ls title=\"%ls\" visible=%d\n", h, cls, title, IsWindowVisible(h));
    if (!wcscmp(cls, L"#32770")) { *(int *)lp += 1; PostMessageW(h, WM_CLOSE, 0, 0); }
    return TRUE;
}

static volatile LONG props_returned;
static HRESULT props_hr;
static DWORD WINAPI props_thread(void *arg)
{
    WCHAR path[MAX_PATH];
    CoInitialize(NULL);
    GetWindowsDirectoryW(path, MAX_PATH);
    wcscat(path, L"\\explorer.exe");
    SAY("SHObjectProperties(%ls)\n", path);
    props_hr = SHObjectProperties(NULL, SHOP_FILEPATH, path, NULL);
    props_returned = 1;
    MSG m;  /* SHObjectProperties returns before the sheet closes; keep this STA alive */
    DWORD end = GetTickCount() + 30000;
    while (GetTickCount() < end) {
        while (PeekMessageW(&m, NULL, 0, 0, PM_REMOVE)) DispatchMessageW(&m);
        Sleep(50);
    }
    return 0;
}

static void probe_props(void)
{
    HANDLE t = CreateThread(NULL, 0, props_thread, NULL, 0, NULL);
    Sleep(20000);
    int sheets = 0;
    SAY("SHObjectProperties returned=%ld hr=%08lx; windows after 20s:\n", props_returned, props_hr);
    SAY("shot properties\n");
    Sleep(4000);
    EnumWindows(dump_win, (LPARAM)&sheets);
    SAY("property sheets seen=%d\n", sheets);
    if (!props_returned || props_hr != S_OK || sheets == 0) failures++;
    WaitForSingleObject(t, 15000);
}

/* How far did the livecd's "setup -mini" (syssetup!InstallLiveCD) get? It is what registers
 * COM and starts PlugPlay on a livecd, as setup does on an installed system. */
static DWORD WINAPI profile_thread(void *arg)
{
    WCHAR path[MAX_PATH], v[MAX_PATH] = L"";
    GetWindowsDirectoryW(path, MAX_PATH);
    wcscat(path, L"\\unattend.inf");
    DWORD r = GetPrivateProfileStringW(L"Shell", L"DefaultThemesOff", L"no", v, MAX_PATH, path);
    SAY("GetPrivateProfileStringW(%ls) = %lu \"%ls\"\n", path, r, v);
    return 0;
}

static void probe_livecd_setup(void)
{
    HANDLE pt = CreateThread(NULL, 0, profile_thread, NULL, 0, NULL);
    if (WaitForSingleObject(pt, 15000) == WAIT_TIMEOUT)
        SAY("GetPrivateProfileStringW(unattend.inf) HUNG for 15s\n");
    PROCESSENTRY32W pe = { sizeof pe };
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    int n = 0, all = 0;
    if (snap != INVALID_HANDLE_VALUE && Process32FirstW(snap, &pe)) {
        do {
            all++;
            if (all <= 40) SAY("  proc %ls\n", pe.szExeFile);
            if (!_wcsicmp(pe.szExeFile, L"setup.exe") || !_wcsicmp(pe.szExeFile, L"userinit.exe") ||
                !_wcsicmp(pe.szExeFile, L"svchost.exe") || !_wcsicmp(pe.szExeFile, L"rundll32.exe"))
                SAY("proc %ls pid=%lu parent=%lu threads=%lu\n", pe.szExeFile, pe.th32ProcessID,
                    pe.th32ParentProcessID, pe.cntThreads), n++;
        } while (Process32NextW(snap, &pe));
        CloseHandle(snap);
    }
    SAY("interesting processes=%d of %d (snapshot=%p gle=%lu)\n", n, all, snap, GetLastError());
    SC_HANDLE scm = OpenSCManagerW(NULL, NULL, SC_MANAGER_CONNECT);
    SC_HANDLE svc = scm ? OpenServiceW(scm, L"PlugPlay", SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG) : NULL;
    if (svc) {
        SERVICE_STATUS st;
        BYTE buf[4096];
        DWORD need;
        QueryServiceStatus(svc, &st);
        QueryServiceConfigW(svc, (QUERY_SERVICE_CONFIGW *)buf, sizeof buf, &need);
        SAY("PlugPlay state=%lu (4=running) start=%lu (2=auto 3=demand)\n", st.dwCurrentState,
            ((QUERY_SERVICE_CONFIGW *)buf)->dwStartType);
        CloseServiceHandle(svc);
    } else SAY("PlugPlay: cannot open (gle=%lu)\n", GetLastError());
    if (scm) CloseServiceHandle(scm);
    DWORD v, cb = sizeof v;
    SAY("Setup\\SetupType=%ld\n", RegGetValueW(HKEY_LOCAL_MACHINE, L"SYSTEM\\Setup", L"SetupType", RRF_RT_REG_DWORD, NULL, &v, &cb) ? -1L : (long)v);
    cb = sizeof v;
    SAY("Setup\\SystemSetupInProgress=%ld\n", RegGetValueW(HKEY_LOCAL_MACHINE, L"SYSTEM\\Setup", L"SystemSetupInProgress", RRF_RT_REG_DWORD, NULL, &v, &cb) ? -1L : (long)v);
}

int main(void)
{
    CoInitialize(NULL);
    SAY("start\n");
    probe_livecd_setup();
    {   /* The livecd registers COM from setup -mini, which runs alongside the shell: wait for
         * ole32's proxy keys (EarlyRegisterDlls) rather than racing it. */
        DWORD t0 = GetTickCount(), cb;
        WCHAR v[64];
        while (GetTickCount() - t0 < 300000) {
            cb = sizeof v;
            if (!RegGetValueW(HKEY_CLASSES_ROOT,
                    L"Interface\\{0000010E-0000-0000-C000-000000000046}\\ProxyStubClsid32",
                    NULL, RRF_RT_REG_SZ, NULL, v, &cb))
                break;
            Sleep(2000);
        }
        SAY("waited %lus for ole32's interface registration (%s)\n", (GetTickCount() - t0) / 1000,
            cb == sizeof v ? "gave up" : "present");
    }
    probe_iid("IUnknown", &IID_IUnknown);
    probe_iid("IDataObject", &IID_IDataObject);
    probe_iid("IStream", &IID_IStream);
    probe_iid("IDropTarget", &IID_IDropTarget);
    probe_iid("IClassFactory", &IID_IClassFactory);
    probe_iid("IEnumFORMATETC", &IID_IEnumFORMATETC);
    probe_iid("IShellFolder", &IID_IShellFolder);
    probe_marshal();
    probe_shell_dataobject();
    probe_props();
    probe_livecd_setup();   /* again, a minute later: setup -mini runs asynchronously */
    SAY("failures=%d\n", failures);
    SAY("done VERDICT=%s\n", failures ? "FAIL" : "PASS");
    CoUninitialize();
    return 0;
}
