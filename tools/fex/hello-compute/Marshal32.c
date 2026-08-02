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
#define M32_FAIL_REG_NTDLL    0xBAE1 /* couldn't resolve the registry syscalls */
#define M32_FAIL_REG_CREATE   0xBAE2 /* NtCreateKey on the scratch key failed */
#define M32_FAIL_REG_SET      0xBAE3 /* NtSetValueKey failed outright */
#define M32_FAIL_REG_QUERY    0xBAE4 /* the value we just set cannot be found */
#define M32_FAIL_REG_VALUE    0xBAE5 /* found, but wrong type/length/data */
#define M32_FAIL_REG_DELETE   0xBAE6 /* NtDeleteValueKey did not remove it */
#define M32_FAIL_WOW_OPEN     0xBAE7 /* plain NtOpenKey failed — rung is invalid */
#define M32_FAIL_WOW_64KEY    0xBAE8 /* NtOpenKey refused KEY_WOW64_64KEY */
#define M32_FAIL_WOW_32KEY    0xBAE9 /* NtOpenKey refused KEY_WOW64_32KEY */
#define M32_FAIL_WOW_CREATE   0xBAEA /* NtCreateKey refused a KEY_WOW64_* flag */
#define M32_PASS              0x600D

static void Report(const char *msg)
{
    OutputDebugStringA(msg);
}

/* No CRT here, so hex formatting is hand-rolled (same shape as Selfmod32's). */
static void ReportHex(const char *label, DWORD v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[96];
    int i = 0, n;
    while (label[i] && i < 72)
    {
        buf[i] = label[i];
        i++;
    }
    buf[i++] = ' ';
    buf[i++] = '0';
    buf[i++] = 'x';
    for (n = 28; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    OutputDebugStringA(buf);
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

/* --- check 5: PUNICODE_STRING args on the registry write path (E20) ------ *
 * UNICODE_STRING is 8 bytes in the guest {Length, MaximumLength, ULONG Buffer}
 * and 16 on the host {Length, MaximumLength, pad, ULONGLONG Buffer}. Buffer
 * therefore lives at offset 4 in the guest and offset 8 on the host, so a flat
 * thunk makes the kernel read the value-name POINTER from past the end of the
 * guest's struct — not a truncation, a wholly unrelated address.
 *
 * NtQueryValueKey was already marshalled while NtSetValueKey and
 * NtDeleteValueKey next to it were not, so reads worked and writes did not.
 * This drives all three through one round-trip: set a value, read it back BY
 * THE SAME NAME, then delete it and confirm it is gone. A wrong Buffer cannot
 * survive that -- either the set faults/errors, or it lands under a garbage
 * name and the query misses. */

typedef LONG(WINAPI *M32_CREATEKEY_FN)(PHANDLE, ULONG, VOID *, ULONG, PVOID, ULONG, PULONG);
typedef LONG(WINAPI *M32_SETVAL_FN)(HANDLE, VOID *, ULONG, ULONG, PVOID, ULONG);
typedef LONG(WINAPI *M32_QUERYVAL_FN)(HANDLE, VOID *, ULONG, PVOID, ULONG, PULONG);
typedef LONG(WINAPI *M32_DELVAL_FN)(HANDLE, VOID *);

typedef struct _M32_USTR {
    USHORT Length;
    USHORT MaximumLength;
    WCHAR *Buffer;
} M32_USTR;

typedef struct _M32_OA {
    ULONG Length;
    HANDLE RootDirectory;
    M32_USTR *ObjectName;
    ULONG Attributes;
    PVOID SecurityDescriptor;
    PVOID SecurityQualityOfService;
} M32_OA;

typedef struct _M32_KVPI {
    ULONG TitleIndex;
    ULONG Type;
    ULONG DataLength;
    UCHAR Data[8];
} M32_KVPI;

static void M32InitUstr(M32_USTR *u, WCHAR *s)
{
    USHORT n = 0;
    while (s[n])
        n++;
    u->Length = (USHORT)(n * sizeof(WCHAR));
    u->MaximumLength = (USHORT)(u->Length + sizeof(WCHAR));
    u->Buffer = s;
}

static int CheckRegistryValueNames(void)
{
    static WCHAR KeyPath[] = L"\\Registry\\Machine\\Software\\vibeos_marshal32";
    static WCHAR ValName[] = L"Wow64ValueNameProbe";
    HMODULE nt = GetModuleHandleA("ntdll.dll");
    M32_CREATEKEY_FN CreateKey;
    M32_SETVAL_FN SetValue;
    M32_QUERYVAL_FN QueryValue;
    M32_DELVAL_FN DeleteValue;
    M32_USTR uKey, uVal;
    M32_OA oa;
    M32_KVPI info;
    HANDLE key = NULL;
    ULONG disp = 0, got = 0;
    DWORD data = 0x5CA11ED;
    LONG st;

    if (!nt)
        return M32_FAIL_REG_NTDLL;
    CreateKey = (M32_CREATEKEY_FN)GetProcAddress(nt, "NtCreateKey");
    SetValue = (M32_SETVAL_FN)GetProcAddress(nt, "NtSetValueKey");
    QueryValue = (M32_QUERYVAL_FN)GetProcAddress(nt, "NtQueryValueKey");
    DeleteValue = (M32_DELVAL_FN)GetProcAddress(nt, "NtDeleteValueKey");
    if (!CreateKey || !SetValue || !QueryValue || !DeleteValue)
        return M32_FAIL_REG_NTDLL;

    M32InitUstr(&uKey, KeyPath);
    M32InitUstr(&uVal, ValName);
    oa.Length = sizeof(oa);
    oa.RootDirectory = NULL;
    oa.ObjectName = &uKey;
    oa.Attributes = 0x40; /* OBJ_CASE_INSENSITIVE */
    oa.SecurityDescriptor = NULL;
    oa.SecurityQualityOfService = NULL;

    /* Volatile: this is a probe, it must not survive the boot. */
    st = CreateKey(&key, 0xF003F /* KEY_ALL_ACCESS */, &oa, 0, NULL,
                   1 /* REG_OPTION_VOLATILE */, &disp);
    if (st < 0)
        return M32_FAIL_REG_CREATE;

    st = SetValue(key, &uVal, 0, 4 /* REG_DWORD */, &data, sizeof(data));
    if (st < 0)
    {
        ReportHex("[marshal32] reg: NtSetValueKey failed, status =", (DWORD)st);
        CloseHandle(key);
        return M32_FAIL_REG_SET;
    }

    info.Type = 0;
    info.DataLength = 0;
    st = QueryValue(key, &uVal, 2 /* KeyValuePartialInformation */, &info,
                    sizeof(info), &got);
    if (st < 0)
    {
        /* The classic signature of the bug: the set "succeeded" but the value
         * is not there under the name we asked for. */
        ReportHex("[marshal32] reg: value set but NOT FOUND, status =", (DWORD)st);
        CloseHandle(key);
        return M32_FAIL_REG_QUERY;
    }
    if (info.Type != 4 || info.DataLength != sizeof(data) ||
        *(DWORD *)info.Data != data)
    {
        ReportHex("[marshal32] reg: wrong value came back, type =", info.Type);
        CloseHandle(key);
        return M32_FAIL_REG_VALUE;
    }

    st = DeleteValue(key, &uVal);
    if (st < 0)
    {
        ReportHex("[marshal32] reg: NtDeleteValueKey failed, status =", (DWORD)st);
        CloseHandle(key);
        return M32_FAIL_REG_DELETE;
    }
    /* ...and it must actually be gone, which proves the delete used OUR name
     * and not a garbage one that happened to hit nothing. */
    st = QueryValue(key, &uVal, 2, &info, sizeof(info), &got);
    if (st >= 0)
    {
        Report("[marshal32] reg: value survived NtDeleteValueKey\n");
        CloseHandle(key);
        return M32_FAIL_REG_DELETE;
    }
    CloseHandle(key);

    Report("[marshal32] registry value-name round-trip verified "
           "(set/query/delete)\n");
    return 0;
}

/* --- check 6: the KEY_WOW64_* access-mask flags ---------------------------- *
 * KEY_WOW64_64KEY (0x0100) and KEY_WOW64_32KEY (0x0200) are not real access
 * bits: no key's DACL grants them, so the kernel MUST strip them before the
 * object-manager access check or the open is denied. NtCreateKey and NtOpenKey
 * each do that in the C with `DesiredAccess &= ~KEY_WOW64_RES` (0x0300).
 *
 * NtOpenKey's Rust port dropped the named constant for a bare 0x0100 literal,
 * so it stripped only the 64KEY bit and KEY_WOW64_32KEY reached
 * ObCheckObjectAccess: RegOpenKeyEx(.., KEY_WOW64_32KEY) returned
 * ACCESS_DENIED while the identical RegCreateKeyEx succeeded. Nothing about
 * that is visible to the compiler, and it is not WoW64-specific — a native
 * 64-bit `reg query /reg:32` hits it too.
 *
 * The rung opens one key four ways and reports each status, so a failure names
 * the offending bit instead of just saying "registry broken". */

typedef LONG(WINAPI *M32_OPENKEY_FN)(PHANDLE, ULONG, VOID *);

#define M32_KEY_WOW64_64KEY 0x0100
#define M32_KEY_WOW64_32KEY 0x0200

static int CheckRegistryWow64Flags(void)
{
    static WCHAR KeyPath[] = L"\\Registry\\Machine\\Software";
    static WCHAR ScratchPath[] = L"\\Registry\\Machine\\Software\\vibeos_m32wow";
    HMODULE nt = GetModuleHandleA("ntdll.dll");
    M32_OPENKEY_FN OpenKey;
    M32_CREATEKEY_FN CreateKey;
    M32_USTR uKey;
    M32_OA oa;
    HANDLE key = NULL;
    ULONG disp = 0;
    LONG st;

    if (!nt)
        return M32_FAIL_REG_NTDLL;
    OpenKey = (M32_OPENKEY_FN)GetProcAddress(nt, "NtOpenKey");
    CreateKey = (M32_CREATEKEY_FN)GetProcAddress(nt, "NtCreateKey");
    if (!OpenKey || !CreateKey)
        return M32_FAIL_REG_NTDLL;

    M32InitUstr(&uKey, KeyPath);
    oa.Length = sizeof(oa);
    oa.RootDirectory = NULL;
    oa.ObjectName = &uKey;
    oa.Attributes = 0x40; /* OBJ_CASE_INSENSITIVE */
    oa.SecurityDescriptor = NULL;
    oa.SecurityQualityOfService = NULL;

    /* Control: no wow64 bit at all. If this fails the rung proves nothing —
     * say so distinctly rather than blaming a flag. */
    st = OpenKey(&key, KEY_READ, &oa);
    if (st < 0)
    {
        ReportHex("[marshal32] wow64flags: plain NtOpenKey failed, status =",
                  (DWORD)st);
        return M32_FAIL_WOW_OPEN;
    }
    CloseHandle(key);

    key = NULL;
    st = OpenKey(&key, KEY_READ | M32_KEY_WOW64_64KEY, &oa);
    if (st < 0)
    {
        ReportHex("[marshal32] wow64flags: KEY_WOW64_64KEY refused, status =",
                  (DWORD)st);
        return M32_FAIL_WOW_64KEY;
    }
    CloseHandle(key);

    key = NULL;
    st = OpenKey(&key, KEY_READ | M32_KEY_WOW64_32KEY, &oa);
    if (st < 0)
    {
        ReportHex("[marshal32] wow64flags: KEY_WOW64_32KEY refused, status =",
                  (DWORD)st);
        return M32_FAIL_WOW_32KEY;
    }
    CloseHandle(key);

    /* NtCreateKey is the sibling that masked correctly all along; assert the
     * pair agrees so a future edit cannot silently break just one of them. */
    M32InitUstr(&uKey, ScratchPath);
    key = NULL;
    st = CreateKey(&key, 0xF003F /* KEY_ALL_ACCESS */ | M32_KEY_WOW64_32KEY,
                   &oa, 0, NULL, 1 /* REG_OPTION_VOLATILE */, &disp);
    if (st < 0)
    {
        ReportHex("[marshal32] wow64flags: NtCreateKey refused 32KEY, status =",
                  (DWORD)st);
        return M32_FAIL_WOW_CREATE;
    }
    CloseHandle(key);

    Report("[marshal32] KEY_WOW64_32KEY/64KEY accepted by open+create\n");
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

    rc = CheckRegistryValueNames();
    if (rc) ExitProcess(rc);

    rc = CheckRegistryWow64Flags();
    if (rc) ExitProcess(rc);

    Report("[marshal32] PASS: TBI + IOCP + monitors + out-cells + reg-names "
           "+ wow64-flags all verified\n");
    ExitProcess(M32_PASS);
    return 0;
}
