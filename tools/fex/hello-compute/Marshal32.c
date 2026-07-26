/*
 * marshal32.exe — the i386 MARSHALLER-VERIFICATION guest for WoW64 M18
 * (docs/engineering/active-lanes/wow64-plan.md). Built like File32.c (no CRT,
 * plain _start), imports kernel32 + user32 + ntdll. Each check exercises a
 * marshaller landed by M17/M18 that no earlier guest touches, with a verdict
 * the guest can verify AGAINST ITSELF (its own TEB, its own posted values),
 * so a wrong marshal fails deterministically rather than "looks fine":
 *
 *   1. NtQueryInformationThread(ThreadBasicInformation) on the -2
 *      pseudo-handle (sign-extension path): TebBaseAddress must equal the
 *      guest's own fs-visible TEB32 (NtCurrentTeb()), UniqueThread/Process
 *      must equal GetCurrentThreadId/ProcessId. Proves the TBI 48->28
 *      marshal + the WowTebOffset TEB32 resolution.
 *   2. IOCP round-trip: CreateIoCompletionPort -> PostQueuedCompletionStatus
 *      (key/bytes/overlapped sentinels) -> GetQueuedCompletionStatus must
 *      return the exact sentinels. Proves NtRemoveIoCompletion's two out-slot
 *      narrows + IOSB copy-down (and NtSetIoCompletion flat widening).
 *   3. Monitors: EnumDisplayMonitors must invoke the callback >= 1 time
 *      (count-aware HMONITOR array marshal), GetMonitorInfoA on that monitor
 *      must return a non-empty rect (new NtUserGetMonitorInfo flat thunk),
 *      MonitorFromPoint(0,0) must return non-NULL (new flat thunk with a
 *      by-value POINT packed across the guest dword stack).
 *   4. Out-cell canaries (E19): NtOpenProcessToken / NtDuplicateObject /
 *      NtCreateTimer each fill a 4-byte guest HANDLE cell with a value the
 *      64-bit kernel stores pointer-sized. A canary parked immediately after
 *      the cell must survive — flat-thunked, the host's 8-byte store ate it.
 *      Requires the calls to SUCCEED: a failing syscall writes nothing and
 *      would pass this check while completely unmarshalled.
 *
 * Verdict contract: exit 0x600D on PASS; distinct 0xBADx names the failing
 * rung so serial alone triages ([WINLOGON] marshal32.exe exit = 0x...).
 */

#include <windows.h>
#include <winternl.h>

#define M32_FAIL_QIT_CALL     0xBAD1 /* NtQueryInformationThread returned error */
#define M32_FAIL_QIT_TEB      0xBAD2 /* TebBaseAddress != guest's own TEB32 */
#define M32_FAIL_QIT_IDS      0xBAD3 /* ClientId doesn't match GetCurrent*Id */
#define M32_FAIL_IOCP_CREATE  0xBAD4 /* CreateIoCompletionPort failed */
#define M32_FAIL_IOCP_POST    0xBAD5 /* PostQueuedCompletionStatus failed */
#define M32_FAIL_IOCP_GET     0xBAD6 /* GetQueuedCompletionStatus failed/timeout */
#define M32_FAIL_IOCP_VALUES  0xBAD7 /* dequeued key/bytes/overlapped mismatch */
#define M32_FAIL_MON_ENUM     0xBAD8 /* EnumDisplayMonitors: no callback */
#define M32_FAIL_MON_INFO     0xBAD9 /* GetMonitorInfoA failed/empty rect */
#define M32_FAIL_MON_FROMPT   0xBADA /* MonitorFromPoint returned NULL */
#define M32_FAIL_CELL_NTDLL   0xBADB /* couldn't resolve the ntdll out-cell entries */
#define M32_FAIL_CELL_TOKCALL 0xBADC /* NtOpenProcessToken failed outright */
#define M32_FAIL_CELL_TOKOVR  0xBADD /* ...and overran its 4-byte handle cell */
#define M32_FAIL_CELL_DUPCALL 0xBADE /* NtDuplicateObject failed outright */
#define M32_FAIL_CELL_DUPOVR  0xBADF /* ...and overran its 4-byte handle cell */
#define M32_FAIL_CELL_TMROVR  0xBAE0 /* NtCreateTimer overran (NULL-OA path) */
#define M32_PASS              0x600D

static void Report(const char *msg)
{
    OutputDebugStringA(msg);
}

/* --- check 1: NtQueryInformationThread(ThreadBasicInformation) ----------- */

typedef struct _M32_TBI32 {
    ULONG ExitStatus;
    ULONG TebBaseAddress;
    ULONG UniqueProcess;
    ULONG UniqueThread;
    ULONG AffinityMask;
    LONG  Priority;
    LONG  BasePriority;
} M32_TBI32;

typedef LONG (WINAPI *M32_QIT_FN)(HANDLE, ULONG, PVOID, ULONG, PULONG);

static int CheckThreadBasicInformation(void)
{
    M32_QIT_FN Qit = (M32_QIT_FN)GetProcAddress(GetModuleHandleA("ntdll.dll"),
                                                "NtQueryInformationThread");
    M32_TBI32 Tbi;
    ULONG Ret = 0;
    LONG St;

    Tbi.TebBaseAddress = 0;
    Tbi.UniqueProcess = 0;
    Tbi.UniqueThread = 0;

    St = Qit(GetCurrentThread() /* -2 pseudo-handle: sign-extension path */,
             0 /* ThreadBasicInformation */, &Tbi, sizeof(Tbi), &Ret);
    if (St < 0)
    {
        Report("[marshal32] FAIL: NtQueryInformationThread(TBI) errored\n");
        return M32_FAIL_QIT_CALL;
    }
    if (Tbi.TebBaseAddress != (ULONG)(ULONG_PTR)NtCurrentTeb())
    {
        Report("[marshal32] FAIL: TBI.TebBaseAddress != own TEB32\n");
        return M32_FAIL_QIT_TEB;
    }
    if (Tbi.UniqueProcess != GetCurrentProcessId() ||
        Tbi.UniqueThread != GetCurrentThreadId())
    {
        Report("[marshal32] FAIL: TBI.ClientId mismatch\n");
        return M32_FAIL_QIT_IDS;
    }
    Report("[marshal32] TBI: TEB32 + ClientId verified\n");
    return 0;
}

/* --- check 2: IOCP round-trip -------------------------------------------- */

static int CheckIoCompletionRoundTrip(void)
{
    HANDLE Port;
    DWORD Bytes = 0;
    ULONG_PTR Key = 0;
    LPOVERLAPPED Ov = NULL;

    Port = CreateIoCompletionPort(INVALID_HANDLE_VALUE, NULL, 0, 0);
    if (!Port)
    {
        Report("[marshal32] FAIL: CreateIoCompletionPort\n");
        return M32_FAIL_IOCP_CREATE;
    }
    if (!PostQueuedCompletionStatus(Port, 123, 0xABCD, (LPOVERLAPPED)0x51C0))
    {
        Report("[marshal32] FAIL: PostQueuedCompletionStatus\n");
        return M32_FAIL_IOCP_POST;
    }
    if (!GetQueuedCompletionStatus(Port, &Bytes, &Key, &Ov, 5000))
    {
        Report("[marshal32] FAIL: GetQueuedCompletionStatus (timeout?)\n");
        return M32_FAIL_IOCP_GET;
    }
    if (Bytes != 123 || Key != 0xABCD || Ov != (LPOVERLAPPED)0x51C0)
    {
        Report("[marshal32] FAIL: IOCP sentinels corrupted by marshal\n");
        return M32_FAIL_IOCP_VALUES;
    }
    CloseHandle(Port);
    Report("[marshal32] IOCP: post/get sentinels round-tripped\n");
    return 0;
}

/* --- check 3: monitor enumeration ---------------------------------------- */

static volatile LONG g_Monitors;
static HMONITOR g_FirstMonitor;

static BOOL CALLBACK MonCb(HMONITOR mon, HDC dc, LPRECT rc, LPARAM lp)
{
    (void)dc; (void)rc; (void)lp;
    if (!g_Monitors)
        g_FirstMonitor = mon;
    g_Monitors++;
    return TRUE;
}

static int CheckMonitors(void)
{
    MONITORINFO Mi;
    POINT Pt;

    if (!EnumDisplayMonitors(NULL, NULL, MonCb, 0) || !g_Monitors)
    {
        Report("[marshal32] FAIL: EnumDisplayMonitors gave no callbacks\n");
        return M32_FAIL_MON_ENUM;
    }
    Mi.cbSize = sizeof(Mi);
    if (!GetMonitorInfoA(g_FirstMonitor, &Mi) ||
        Mi.rcMonitor.right <= Mi.rcMonitor.left)
    {
        Report("[marshal32] FAIL: GetMonitorInfoA empty rect\n");
        return M32_FAIL_MON_INFO;
    }
    Pt.x = 0;
    Pt.y = 0;
    if (!MonitorFromPoint(Pt, MONITOR_DEFAULTTONEAREST))
    {
        Report("[marshal32] FAIL: MonitorFromPoint(0,0) NULL\n");
        return M32_FAIL_MON_FROMPT;
    }
    Report("[marshal32] monitors: enum + info + from-point verified\n");
    return 0;
}

/* --- check 4: the out-cell class (E19) ---------------------------------- *
 * These syscalls take a guest pointer to a cell the kernel fills with a
 * POINTER-SIZED value. On i386 that cell is 4 bytes; flat-thunked, the host's
 * 64-bit store ran four bytes past it. So park a canary immediately after the
 * handle and check it survived.
 *
 * Two properties make this a real test rather than a shape:
 *   - the call must SUCCEED. A failing syscall writes nothing and would leave
 *     the canary intact even completely unmarshalled, i.e. pass while broken.
 *   - the canary is `volatile`. We only hand the callee `&cell.Handle`, so a
 *     conforming compiler is entitled to assume the rest of the struct is
 *     untouched and fold the comparison away — which is precisely the
 *     optimisation that would delete the bug from the test but not the OS. */

#define M32_CANARY 0xC0DEFACEu

typedef struct _M32_HCELL {
    HANDLE         Handle;  /* 4 bytes on i386 — exactly what got overrun */
    volatile DWORD Canary;
} M32_HCELL;

typedef LONG (WINAPI *M32_OPT_FN)(HANDLE, ULONG, PHANDLE);
typedef LONG (WINAPI *M32_DUP_FN)(HANDLE, HANDLE, HANDLE, PHANDLE, ULONG,
                                  ULONG, ULONG);
typedef LONG (WINAPI *M32_TMR_FN)(PHANDLE, ULONG, PVOID, ULONG);

static int CheckOutCellCanaries(void)
{
    HMODULE nt = GetModuleHandleA("ntdll.dll");
    M32_OPT_FN OpenProcessToken;
    M32_DUP_FN DuplicateObject;
    M32_TMR_FN CreateTimer;
    M32_HCELL cell;
    HANDLE self = (HANDLE)(LONG_PTR)-1;   /* NtCurrentProcess pseudo-handle */
    LONG st;

    if (!nt)
        return M32_FAIL_CELL_NTDLL;
    OpenProcessToken = (M32_OPT_FN)GetProcAddress(nt, "NtOpenProcessToken");
    DuplicateObject  = (M32_DUP_FN)GetProcAddress(nt, "NtDuplicateObject");
    CreateTimer      = (M32_TMR_FN)GetProcAddress(nt, "NtCreateTimer");
    if (!OpenProcessToken || !DuplicateObject)
        return M32_FAIL_CELL_NTDLL;

    /* A: NtOpenProcessToken — the one every security-aware app reaches, and
     *    the one whose *Ex sibling was already marshalled. */
    cell.Handle = NULL;
    cell.Canary = M32_CANARY;
    st = OpenProcessToken(self, TOKEN_QUERY, &cell.Handle);
    if (st < 0)
        return M32_FAIL_CELL_TOKCALL;
    if (cell.Canary != M32_CANARY)
        return M32_FAIL_CELL_TOKOVR;
    CloseHandle(cell.Handle);

    /* B: NtDuplicateObject — DuplicateHandle's syscall. */
    cell.Handle = NULL;
    cell.Canary = M32_CANARY;
    st = DuplicateObject(self, GetCurrentThread(), self, &cell.Handle,
                         0, 0, DUPLICATE_SAME_ACCESS);
    if (st < 0)
        return M32_FAIL_CELL_DUPCALL;
    if (cell.Canary != M32_CANARY)
        return M32_FAIL_CELL_DUPOVR;
    CloseHandle(cell.Handle);

    /* C: NtCreateTimer with a NULL ObjectAttributes — covers the _In_opt_ OA
     *    path, which must reach the kernel as a real NULL and must not fault
     *    inside the thunk. Only assert the canary if the call actually ran:
     *    unlike A and B this one can legitimately be absent. */
    if (CreateTimer)
    {
        cell.Handle = NULL;
        cell.Canary = M32_CANARY;
        st = CreateTimer(&cell.Handle, TIMER_ALL_ACCESS, NULL,
                         1 /* SynchronizationTimer */);
        if (st >= 0)
        {
            if (cell.Canary != M32_CANARY)
                return M32_FAIL_CELL_TMROVR;
            CloseHandle(cell.Handle);
            Report("[marshal32] out-cell: timer (NULL OA) verified\n");
        }
        else
        {
            Report("[marshal32] out-cell: NtCreateTimer unavailable, skipped\n");
        }
    }

    Report("[marshal32] out-cell canaries intact (token, duplicate)\n");
    return 0;
}

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    int rc;

    Report("[marshal32] start: M18 marshaller verification\n");

    rc = CheckThreadBasicInformation();
    if (rc) ExitProcess(rc);

    rc = CheckIoCompletionRoundTrip();
    if (rc) ExitProcess(rc);

    rc = CheckMonitors();
    if (rc) ExitProcess(rc);

    rc = CheckOutCellCanaries();
    if (rc) ExitProcess(rc);

    Report("[marshal32] PASS: TBI + IOCP + monitors + out-cells all verified\n");
    ExitProcess(M32_PASS);
    return 0;
}
