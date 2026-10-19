/* modrun probe: stobject (the systray service object), shipping Rust against the pre-flip C.
 *
 * Staged twice: as stobject_c.exe it loads stobject_c.dll (the C oracle) from its directory,
 * as stobject_probe.exe the system stobject.dll. Values only -- HRESULTs, message results,
 * window styles, the registry's service mask -- never a handle or a pointer, so the runs diff
 * byte for byte.
 *
 * Explorer starts its own instance at every boot, so smoke reaches CreateSysTrayThread and
 * WM_CREATE; it never reaches the rest. Covered here: the ATL server and its COM map,
 * IOleCommandTarget::Exec's command group check and its OLECMDID_NEW / OLECMDID_SAVE pair
 * (the tray thread, the window CMessageWndClass styles, and the cross-thread WM_CLOSE
 * shutdown), the WM_USER+220/221 service protocol that the Power, Volume and Hotplug
 * handlers answer through the g_IconHandlers table -- including IsServiceEnabled's
 * unnormalised BOOL and EnableService's write of the Services value -- WM_TIMER's poll, a
 * message no handler takes, and DllCanUnloadNow. The Services value is restored on exit. */
#define COBJMACROS
#include <windows.h>
#include <ole2.h>
#include <docobj.h>
#include <stdio.h>

static const CLSID CLSID_SysTray = {0x35CEC8A3, 0x2BE6, 0x11D2, {0x87, 0x73, 0x92, 0xE2, 0x20, 0x52, 0x41, 0x53}};
static const GUID CGID_ShellServiceObject = {0x000214D2, 0, 0, {0xC0, 0, 0, 0, 0, 0, 0, 0x46}};
static const WCHAR SYSTRAY_KEY[] = L"Software\\Microsoft\\Windows\\CurrentVersion\\Applets\\SysTray";

#define POWER_SERVICE_FLAG 0x00000001
#define HOTPLUG_SERVICE_FLAG 0x00000002
#define VOLUME_SERVICE_FLAG 0x00000004
#define MOUSE_SERVICE_FLAG 0x80000000
#define POLL_TIMER_ID 1

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

static HWND g_found;

static BOOL CALLBACK find_tray(HWND hwnd, LPARAM lp)
{
    DWORD pid = 0;
    WCHAR cls[64];
    GetWindowThreadProcessId(hwnd, &pid);
    if (pid == (DWORD)lp && GetClassNameW(hwnd, cls, 64) && !lstrcmpW(cls, L"SystemTray_Main"))
    {
        g_found = hwnd;
        return FALSE;
    }
    return TRUE;
}

static HWND wait_tray(void)
{
    for (int i = 0; i < 100; i++)
    {
        g_found = NULL;
        EnumWindows(find_tray, (LPARAM)GetCurrentProcessId());
        if (g_found)
            return g_found;
        Sleep(50);
    }
    return NULL;
}

/* The Services value, or -1 when absent. */
static LONGLONG read_services(void)
{
    HKEY key;
    DWORD v = 0, size = sizeof(v), type = 0;
    if (RegOpenKeyExW(HKEY_CURRENT_USER, SYSTRAY_KEY, 0, KEY_READ, &key))
        return -1;
    LONG rc = RegQueryValueExW(key, L"Services", NULL, &type, (BYTE *)&v, &size);
    RegCloseKey(key);
    return rc || type != REG_DWORD ? -1 : (LONGLONG)v;
}

static void write_services(LONGLONG v)
{
    HKEY key;
    if (RegCreateKeyExW(HKEY_CURRENT_USER, SYSTRAY_KEY, 0, NULL, 0, KEY_WRITE, NULL, &key, NULL))
        return;
    if (v < 0)
        RegDeleteValueW(key, L"Services");
    else
    {
        DWORD d = (DWORD)v;
        RegSetValueExW(key, L"Services", 0, REG_DWORD, (const BYTE *)&d, sizeof(d));
    }
    RegCloseKey(key);
}

static void query(HWND tray, const char *label, DWORD flag)
{
    LRESULT r = SendMessageW(tray, WM_USER + 221, flag, 0);
    printf("query %s %ld\n", label, (long)r);
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
        dll = LoadLibraryW(L"stobject.dll");
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

    LONGLONG saved = read_services();
    /* A known starting mask, so both runs see the same one: GetServicesEnabled reads it. */
    write_services(POWER_SERVICE_FLAG | VOLUME_SERVICE_FLAG | HOTPLUG_SERVICE_FLAG);

    IClassFactory *cf = NULL;
    HRESULT hr = pGetClassObject(&CLSID_SysTray, &IID_IClassFactory, (void **)&cf);
    printf("classobject %08lx\n", hr);
    if (FAILED(hr))
        return 1;
    IOleCommandTarget *ct = NULL;
    hr = IClassFactory_CreateInstance(cf, NULL, &IID_IOleCommandTarget, (void **)&ct);
    printf("create %08lx\n", hr);
    IUnknown *outer = NULL;
    hr = IClassFactory_CreateInstance(cf, (IUnknown *)cf, &IID_IUnknown, (void **)&outer);
    printf("create aggregated %08lx null=%d\n", hr, outer == NULL);
    IClassFactory_Release(cf);
    if (!ct)
        return 1;

    IUnknown *unk = NULL;
    hr = IOleCommandTarget_QueryInterface(ct, &IID_IUnknown, (void **)&unk);
    printf("qi unknown %08lx\n", hr);
    if (unk)
        IUnknown_Release(unk);
    IOleWindow *ow = NULL;
    hr = IOleCommandTarget_QueryInterface(ct, &IID_IOleWindow, (void **)&ow);
    printf("qi olewindow %08lx null=%d\n", hr, ow == NULL);

    OLECMD cmd = {OLECMDID_NEW, 0};
    hr = IOleCommandTarget_QueryStatus(ct, &CGID_ShellServiceObject, 1, &cmd, NULL);
    printf("querystatus %08lx\n", hr);
    hr = IOleCommandTarget_Exec(ct, &GUID_NULL, OLECMDID_NEW, 0, NULL, NULL);
    printf("exec wrong group %08lx\n", hr);
    hr = IOleCommandTarget_Exec(ct, &CGID_ShellServiceObject, 99, 0, NULL, NULL);
    printf("exec unknown cmd %08lx\n", hr);
    printf("tray before new %d\n", wait_tray() != NULL);

    hr = IOleCommandTarget_Exec(ct, &CGID_ShellServiceObject, OLECMDID_NEW, 0, NULL, NULL);
    printf("exec new %08lx\n", hr);
    HWND tray = wait_tray();
    printf("tray %d\n", tray != NULL);
    if (tray)
    {
        printf("style %08lx exstyle %08lx\n", GetWindowLongW(tray, GWL_STYLE),
               GetWindowLongW(tray, GWL_EXSTYLE));
        printf("class style %08lx visible %d\n", GetClassLongW(tray, GCL_STYLE),
               IsWindowVisible(tray));
        printf("same thread %d\n", GetWindowThreadProcessId(tray, NULL) == GetCurrentThreadId());

        query(tray, "power", POWER_SERVICE_FLAG);
        query(tray, "hotplug", HOTPLUG_SERVICE_FLAG);
        query(tray, "volume", VOLUME_SERVICE_FLAG);
        query(tray, "mouse", MOUSE_SERVICE_FLAG);
        query(tray, "unknown", 0x10);

        static const struct { const char *label; DWORD flag; } svc[] = {
            {"power", POWER_SERVICE_FLAG},
            {"hotplug", HOTPLUG_SERVICE_FLAG},
            {"volume", VOLUME_SERVICE_FLAG},
        };
        for (int i = 0; i < 3; i++)
        {
            LRESULT r = SendMessageW(tray, WM_USER + 220, svc[i].flag, FALSE);
            printf("disable %s %ld services %lld\n", svc[i].label, (long)r, read_services());
            query(tray, svc[i].label, svc[i].flag);
        }
        for (int i = 0; i < 3; i++)
        {
            LRESULT r = SendMessageW(tray, WM_USER + 220, svc[i].flag, TRUE);
            printf("enable %s %ld services %lld\n", svc[i].label, (long)r, read_services());
            query(tray, svc[i].label, svc[i].flag);
        }
        printf("timer poll %ld\n", (long)SendMessageW(tray, WM_TIMER, POLL_TIMER_ID, 0));
        printf("unhandled %ld\n", (long)SendMessageW(tray, WM_USER + 500, 7, 9));

        hr = IOleCommandTarget_Exec(ct, &CGID_ShellServiceObject, OLECMDID_SAVE, 0, NULL, NULL);
        printf("exec save %08lx\n", hr);
        int gone = 0;
        for (int i = 0; i < 100 && !gone; i++)
        {
            gone = !IsWindow(tray);
            if (!gone)
                Sleep(50);
        }
        printf("tray gone %d\n", gone);
        /* The tray thread's Release follows its message loop; let it run. */
        Sleep(500);
    }

    ULONG refs = IOleCommandTarget_Release(ct);
    printf("release %lu\n", refs);
    printf("canunload %08lx\n", pCanUnloadNow());
    write_services(saved);
    CoUninitialize();
    printf("probe done\n");
    return 0;
}
