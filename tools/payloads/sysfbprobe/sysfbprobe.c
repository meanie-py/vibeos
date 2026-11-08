/* sysfbprobe — sysfb as a PnP display driver (docs/engineering/sysfb-pnp-design.md).
 *
 * Runs from the Startup folder, i.e. with win32k drawing through sysfb, and checks:
 *   step 1  every sysfb device node is started; a PCI one sits in the Display class (what
 *           Device Manager groups by); \\.\DISPLAY1 is attached to the desktop.
 *   step 3  the instance driving the display refuses removal and disable (the desktop has
 *           it open), naming itself as the veto; the idle root instance, when the PCI one
 *           owns the display, goes through Device Manager's own disable and enable
 *           (SetupDiCallClassInstaller DIF_PROPERTYCHANGE): it stops, carries
 *           CM_PROB_DISABLED and the persisted CONFIGFLAG_DISABLED, then starts again.
 *   step 4  a sleep (S3) and a hibernate (S4) request are refused by the display, not
 *           answered "not implemented" (nobody refused) or, as before, a silent success.
 * After each step the screen must still round-trip a pixel, so a refusal that loses the
 * framebuffer mapping fails here too.
 * Verdict: "[SYSFBPROBE] done VERDICT=PASS|FAIL". */
#include <windows.h>
#include <setupapi.h>
#include <cfgmgr32.h>
#include <regstr.h>
#include <stdio.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[SYSFBPROBE] " __VA_ARGS__)

typedef LONG NTSTATUS;
NTSTATUS NTAPI NtSetSystemPowerState(POWER_ACTION SystemAction, SYSTEM_POWER_STATE MinSystemState,
                                     ULONG Flags);
#define STATUS_NOT_SUPPORTED ((NTSTATUS)0xC00000BB)

static int failures;
#define CHECK(cond, ...)                                                                       \
    do {                                                                                       \
        if (!(cond)) {                                                                         \
            SAY("FAIL " __VA_ARGS__);                                                          \
            failures++;                                                                        \
        }                                                                                      \
    } while (0)

static const char DISPLAY_CLASS[] = "{4d36e968-e325-11ce-bfc1-08002be10318}";

static int node_started(DEVINST dev, ULONG *problem)
{
    ULONG status = 0, prob = 0;
    if (CM_Get_DevNode_Status(&status, &prob, dev, 0) != CR_SUCCESS)
        return 0;
    if (problem)
        *problem = prob;
    return (status & DN_STARTED) != 0;
}

/* The display still works: a 1x1 solid fill reads back. A display DC, not GetDC(NULL),
 * which is the desktop window's and is clipped by the windows over it; PatBlt, not SetPixel,
 * because SetPixel on the display surface writes black here (a GDI bug, not sysfb's: the
 * same DC reads the wallpaper's real colour). */
static void check_screen(const char *when)
{
    HDC dc = CreateDCA("DISPLAY", NULL, NULL, NULL);
    if (!dc) {
        SAY("FAIL %s: CreateDC(DISPLAY) failed\n", when);
        failures++;
        return;
    }
    COLORREF old = GetPixel(dc, 0, 0);
    HBRUSH probe = CreateSolidBrush(RGB(0x12, 0x34, 0x56));
    HGDIOBJ prev = SelectObject(dc, probe);
    PatBlt(dc, 0, 0, 1, 1, PATCOPY);
    COLORREF got = GetPixel(dc, 0, 0);
    HBRUSH restore = CreateSolidBrush(old);
    SelectObject(dc, restore);
    PatBlt(dc, 0, 0, 1, 1, PATCOPY);
    SelectObject(dc, prev);
    DeleteObject(probe);
    DeleteObject(restore);
    DeleteDC(dc);
    CHECK(got == RGB(0x12, 0x34, 0x56), "%s: pixel round trip read 0x%06lx (before 0x%06lx)\n", when,
          (ULONG)got, (ULONG)old);
}

static void enable_shutdown_privilege(void)
{
    HANDLE token;
    TOKEN_PRIVILEGES tp = { 1 };
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &token)) {
        SAY("FAIL OpenProcessToken: %lu\n", GetLastError());
        failures++;
        return;
    }
    LookupPrivilegeValueA(NULL, SE_SHUTDOWN_NAME, &tp.Privileges[0].Luid);
    tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    if (!AdjustTokenPrivileges(token, FALSE, &tp, 0, NULL, NULL) || GetLastError() != ERROR_SUCCESS) {
        SAY("FAIL enabling SeShutdownPrivilege: %lu\n", GetLastError());
        failures++;
    }
    CloseHandle(token);
}

static DEVINST root, pci;
static char root_id[MAX_DEVICE_ID_LEN];

/* Every device node whose service is sysfb, found by walking the tree with CM_Get_Child /
 * CM_Get_Sibling. (SetupDiGetClassDevs(DIGCF_ALLCLASSES) failed with error 31 here.) */
static void visit(DEVINST dev);
static void walk(DEVINST parent)
{
    DEVINST child;
    if (CM_Get_Child(&child, parent, 0) != CR_SUCCESS)
        return;
    do {
        visit(child);
        walk(child);
    } while (CM_Get_Sibling(&child, child, 0) == CR_SUCCESS);
}

static void visit(DEVINST dev)
{
    char id[MAX_DEVICE_ID_LEN], service[64] = "", klass[64] = "";
    ULONG len = sizeof(service);
    if (CM_Get_DevNode_Registry_PropertyA(dev, CM_DRP_SERVICE, NULL, service, &len, 0) != CR_SUCCESS ||
        _stricmp(service, "sysfb") != 0)
        return;
    len = sizeof(klass);
    CM_Get_DevNode_Registry_PropertyA(dev, CM_DRP_CLASSGUID, NULL, klass, &len, 0);
    CM_Get_Device_IDA(dev, id, sizeof(id), 0);
    {
        ULONG problem = 0;
        int up = node_started(dev, &problem);
        SAY("node %s class=%s started=%d problem=%lu\n", id, klass[0] ? klass : "-", up, problem);
        if (_strnicmp(id, "ROOT\\SYSFB\\", 11) == 0) {
            root = dev;
            strcpy(root_id, id);
            CHECK(up, "root instance %s not started (problem %lu)\n", id, problem);
        } else if (_strnicmp(id, "PCI\\", 4) == 0) {
            /* A second display controller fails start by design; the probe expects the one
             * QEMU has, which holds the GOP framebuffer. */
            pci = dev;
            CHECK(up, "PCI instance %s not started (problem %lu)\n", id, problem);
            CHECK(_stricmp(klass, DISPLAY_CLASS) == 0, "PCI instance %s is in class %s, not Display\n",
                  id, klass[0] ? klass : "(none)");
        }
    }
}

/* The CM_* calls are RPCs to umpnpmgr: wait for the PlugPlay service, which the service
 * manager starts after the Startup folder has already run. */
static void wait_plugplay(void)
{
    SC_HANDLE scm = OpenSCManagerA(NULL, NULL, SC_MANAGER_CONNECT);
    SC_HANDLE svc = scm ? OpenServiceA(scm, "PlugPlay", SERVICE_QUERY_STATUS) : NULL;
    SERVICE_STATUS st = { 0 };
    int i;
    for (i = 0; svc && i < 600; i++) {
        if (QueryServiceStatus(svc, &st) && st.dwCurrentState == SERVICE_RUNNING)
            break;
        Sleep(500);
    }
    SAY("PlugPlay service state %lu after %d.%d s\n", st.dwCurrentState, i / 2, (i % 2) * 5);
    CHECK(st.dwCurrentState == SERVICE_RUNNING, "PlugPlay service not running\n");
    if (svc)
        CloseServiceHandle(svc);
    if (scm)
        CloseServiceHandle(scm);
}

/* The owner refuses both ways in: a removal names it as the veto, a disable is vetoed. */
static void check_owner_veto(DEVINST owner, const char *what)
{
    char id[MAX_DEVICE_ID_LEN], veto[MAX_DEVICE_ID_LEN] = "";
    PNP_VETO_TYPE type = PNP_VetoTypeUnknown;
    CM_Get_Device_IDA(owner, id, sizeof(id), 0);
    CONFIGRET cr = CM_Query_And_Remove_SubTreeA(owner, &type, veto, sizeof(veto), CM_REMOVE_NO_RESTART);
    SAY("remove %s (%s) -> cr=%lu veto type %d \"%s\"\n", what, id, cr, (int)type, veto);
    CHECK(cr == CR_REMOVE_VETOED, "removing the %s answered cr=%lu, expected CR_REMOVE_VETOED\n", what, cr);
    CHECK(type == PNP_VetoDevice && _stricmp(veto, id) == 0, "the %s veto is type %d \"%s\", expected the device itself\n",
          what, (int)type, veto);
    cr = CM_Disable_DevNode(owner, CM_DISABLE_UI_NOT_OK);
    SAY("disable %s -> cr=%lu\n", what, cr);
    CHECK(cr == CR_REMOVE_VETOED, "disabling the %s answered cr=%lu, expected CR_REMOVE_VETOED\n", what, cr);
    CHECK(node_started(owner, NULL), "the %s stopped after refusing\n", what);
}

/* Device Manager's Disable/Enable: the class installer's DIF_PROPERTYCHANGE, global scope,
 * through the W entry points devmgr uses (SetupDiSetClassInstallParamsA is a stub, as in
 * Wine and upstream). */
static BOOL change_state(const char *id, DWORD state)
{
    WCHAR wid[MAX_DEVICE_ID_LEN];
    MultiByteToWideChar(CP_ACP, 0, id, -1, wid, MAX_DEVICE_ID_LEN);
    HDEVINFO set = SetupDiCreateDeviceInfoList(NULL, NULL);
    SP_DEVINFO_DATA data = { sizeof(data) };
    SP_PROPCHANGE_PARAMS pc = { { sizeof(SP_CLASSINSTALL_HEADER), DIF_PROPERTYCHANGE }, state, DICS_FLAG_GLOBAL, 0 };
    const char *step = "SetupDiCreateDeviceInfoList";
    BOOL ok = set != INVALID_HANDLE_VALUE;
    if (ok && !(step = "SetupDiOpenDeviceInfo", SetupDiOpenDeviceInfoW(set, wid, NULL, 0, &data)))
        ok = FALSE;
    if (ok && !(step = "SetupDiSetClassInstallParams",
                SetupDiSetClassInstallParamsW(set, &data, &pc.ClassInstallHeader, sizeof(pc))))
        ok = FALSE;
    if (ok && !(step = "SetupDiCallClassInstaller", SetupDiCallClassInstaller(DIF_PROPERTYCHANGE, set, &data)))
        ok = FALSE;
    if (!ok)
        SAY("FAIL %s %s: %s error %lu\n", state == DICS_DISABLE ? "disabling" : "enabling", id, step,
            GetLastError());
    if (set != INVALID_HANDLE_VALUE)
        SetupDiDestroyDeviceInfoList(set);
    return ok;
}

static DWORD config_flags(const char *id)
{
    char path[400];
    DWORD flags = 0, size = sizeof(flags);
    snprintf(path, sizeof(path), "SYSTEM\\CurrentControlSet\\Enum\\%s", id);
    RegGetValueA(HKEY_LOCAL_MACHINE, path, "ConfigFlags", RRF_RT_REG_DWORD, NULL, &flags, &size);
    return flags;
}

static void cycle_root(void)
{
    ULONG problem = 0;
    CHECK(change_state(root_id, DICS_DISABLE), "Device Manager disable of %s failed\n", root_id);
    int up = node_started(root, &problem);
    DWORD flags = config_flags(root_id);
    SAY("root disabled: started=%d problem=%lu ConfigFlags=0x%lx\n", up, problem, flags);
    CHECK(!up && problem == CM_PROB_DISABLED, "disabled root: started=%d problem=%lu\n", up, problem);
    CHECK(flags & CONFIGFLAG_DISABLED, "disable did not persist: ConfigFlags=0x%lx\n", flags);
    check_screen("root disabled");

    CHECK(change_state(root_id, DICS_ENABLE), "Device Manager enable of %s failed\n", root_id);
    up = node_started(root, &problem);
    flags = config_flags(root_id);
    SAY("root enabled: started=%d problem=%lu ConfigFlags=0x%lx\n", up, problem, flags);
    CHECK(up && problem == 0, "re-enabled root: started=%d problem=%lu\n", up, problem);
    CHECK(!(flags & CONFIGFLAG_DISABLED), "enable left ConfigFlags=0x%lx\n", flags);
    check_screen("root enabled");
}

int main(void)
{
    wait_plugplay();

    /* ---- step 1: the device nodes ---- */
    DEVINST top;
    CONFIGRET cr0 = CM_Locate_DevNodeA(&top, NULL, CM_LOCATE_DEVNODE_NORMAL);
    CHECK(cr0 == CR_SUCCESS, "no root devnode (cr=%lu)\n", cr0);
    walk(top);
    CHECK(root != 0, "no ROOT\\SYSFB instance\n");
    SAY("owner should be the %s instance\n", pci ? "PCI" : "root");

    DISPLAY_DEVICEA dd = { sizeof(dd) };
    CHECK(EnumDisplayDevicesA(NULL, 0, &dd, 0), "EnumDisplayDevices(0) failed\n");
    SAY("display %s \"%s\" id=%s flags=0x%lx\n", dd.DeviceName, dd.DeviceString, dd.DeviceID,
        dd.StateFlags);
    CHECK(strcmp(dd.DeviceName, "\\\\.\\DISPLAY1") == 0, "first display is %s\n", dd.DeviceName);
    CHECK(dd.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, "DISPLAY1 not attached\n");
    check_screen("step 1");

    /* ---- step 3: the owner refuses removal; the idle root cycles ---- */
    if (pci) {
        check_owner_veto(pci, "PCI owner");
        if (root)
            cycle_root();
    } else if (root) {
        check_owner_veto(root, "root owner");
    }
    check_screen("step 3");

    /* ---- step 4: sleep and hibernate are refused by the display ---- */
    enable_shutdown_privilege();
    NTSTATUS st = NtSetSystemPowerState(PowerActionSleep, PowerSystemSleeping3, 0);
    SAY("sleep S3 -> 0x%08lx\n", (ULONG)st);
    CHECK(st == STATUS_NOT_SUPPORTED, "S3 answered 0x%08lx, expected sysfb's 0xC00000BB\n", (ULONG)st);
    st = NtSetSystemPowerState(PowerActionHibernate, PowerSystemHibernate, 0);
    SAY("hibernate S4 -> 0x%08lx\n", (ULONG)st);
    CHECK(st == STATUS_NOT_SUPPORTED, "S4 answered 0x%08lx, expected sysfb's 0xC00000BB\n", (ULONG)st);
    check_screen("after the power queries");

    SAY("done VERDICT=%s (%d failures)\n", failures ? "FAIL" : "PASS", failures);
    return failures ? 1 : 0;
}
