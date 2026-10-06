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
 *   5. Registry value names (E20): set a value, read it back BY THE SAME NAME,
 *      delete it, confirm it is gone. A `PUNICODE_STRING` read from past the
 *      end of the guest struct cannot survive that round trip.
 *   6. The KEY_WOW64_* access-mask flags (E21): open one key four ways so a
 *      failure names the offending bit instead of "registry broken".
 *   7. The SELF-DESCRIBING SIZE family (E22): TrackMouseEvent / GetCursorInfo
 *      / GetGUIThreadInfo / SendInput each state their own sizeof, which
 *      win32k version-checks against the HOST sizeof. Every one of them
 *      returned FALSE for every 32-bit process. Each check asserts three
 *      things at once: the call SUCCEEDS (the size reached the kernel as the
 *      host size), the size field comes BACK as the guest size (the copy-out
 *      undid the rewrite, so the guest is not left holding a size that does
 *      not describe its own struct), and a canary parked immediately after
 *      the struct SURVIVES (the kernel's larger write landed in the bounce
 *      buffer, not on the guest's stack).
 *   8. The win32k LAST-ERROR channel: win32k's EngSetLastError writes the
 *      *64-bit* TEB and a 32-bit guest reads TEB32, so every error code the
 *      whole USER/GDI surface produced used to be invisible here. Drives one
 *      call (TrackMouseEvent) three ways so a code has to travel rather than
 *      merely appear: bad size -> ERROR_INVALID_PARAMETER, bad flag -> a
 *      DIFFERENT code, and a SUCCEEDING call must leave the guest's own
 *      last-error alone.
 *   9. Wow6432Node registry redirection (E24): a 32-bit process gets its own
 *      view of HKLM\Software. Asserts the two views DISAGREE — the default
 *      view's own KeyNameInformation must say Wow6432Node, and the identical
 *      path opened with KEY_WOW64_64KEY must not see the value just written —
 *      plus that a 64-bit-only key still resolves from the 32-bit view, which
 *      is the miss-fallback the whole design rests on.
 *
 * Verdict contract: exit 0x600D on PASS; distinct 0xBADx names the failing
 * rung so serial alone triages ([WINLOGON] marshal32.exe exit = 0x...).
 */

#include <winsock2.h>
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
#define M32_FAIL_TME_CALL     0xBAEB /* TrackMouseEvent(TME_QUERY) returned FALSE */
#define M32_FAIL_TME_SIZE     0xBAEC /* ...succeeded but cbSize came back host-sized */
#define M32_FAIL_TME_CANARY   0xBAED /* ...and the kernel's 24-byte write overran us */
#define M32_FAIL_CI_CALL      0xBAEE /* GetCursorInfo returned FALSE */
#define M32_FAIL_CI_SIZE      0xBAEF /* ...cbSize came back host-sized */
#define M32_FAIL_CI_CANARY    0xBAF0 /* ...24-byte write overran the 20-byte struct */
#define M32_FAIL_GTI_CALL     0xBAF1 /* GetGUIThreadInfo returned FALSE */
#define M32_FAIL_GTI_SIZE     0xBAF2 /* ...cbSize came back host-sized */
#define M32_FAIL_GTI_CANARY   0xBAF3 /* ...72-byte write overran the 48-byte struct */
#define M32_FAIL_SENDINPUT    0xBAF4 /* SendInput rejected the guest's cbSize */
#define M32_FAIL_LERR_CALL    0xBAF5 /* a call that must be REJECTED succeeded */
#define M32_FAIL_LERR_PARAM   0xBAF6 /* bad cbSize: not ERROR_INVALID_PARAMETER */
#define M32_FAIL_LERR_FLAGS   0xBAF7 /* bad flag bit: not ERROR_INVALID_FLAGS */
#define M32_FAIL_LERR_QUERY   0xBAF8 /* the succeeding control call failed */
#define M32_FAIL_LERR_CLOBBER 0xBAF9 /* ...and ate the guest's own last-error */
#define M32_FAIL_RDIR_CREATE  0xBAFA /* couldn't create the scratch key at all */
#define M32_FAIL_RDIR_NAME    0xBAFB /* default view did NOT land in Wow6432Node */
#define M32_FAIL_RDIR_LEAK    0xBAFC /* the 64-bit view can see the 32-bit value */
#define M32_FAIL_RDIR_64VIEW  0xBAFD /* KEY_WOW64_64KEY did not reach the 64-bit view */
#define M32_FAIL_RDIR_SHARED  0xBAFE /* a 64-bit-only key stopped resolving */
/* Socket rungs get their own block: the 0xBAFx range is full, and these are the
 * first checks that reach a DRIVER payload (AFD) rather than a syscall argument. */
#define M32_FAIL_WSA_STARTUP  0xBB01 /* WSAStartup failed — no Winsock at all */
#define M32_FAIL_WSA_SOCKET   0xBB02 /* socket() failed — provider never loaded */
#define M32_FAIL_SELECT_CALL  0xBB03 /* select() returned SOCKET_ERROR */
#define M32_FAIL_SELECT_READY 0xBB04 /* select() claimed a fresh socket was ready */
#define M32_FAIL_SELECT_GUARD 0xBB05 /* something wrote past the guest's fd_set */
#define M32_FAIL_SELECT_COUNT 0xBB06 /* fd_count came back altered */
#define M32_FAIL_SENDTO_COUNT 0xBB08 /* ...succeeded but sent the wrong length */
#define M32_FAIL_SENDTO_BIND  0xBB09 /* the implicit bind never happened */
#define M32_FAIL_SENDTO_GUARD 0xBB0A /* something wrote past the guest's sockaddr */
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

/* Report a COUNTED wide string (registry paths are not NUL-terminated when
 * they come back in a KEY_NAME_INFORMATION). Non-ASCII is not expected on
 * these paths and is shown as '?' rather than silently dropped.
 *
 * This exists because rung 9 once failed with "default view is not in
 * Wow6432Node" and did not say what the name WAS — which cost a full
 * build-and-boot cycle to answer. An assertion should name what it found,
 * not only what it wanted. */
static void ReportWide(const char *label, const WCHAR *s, ULONG chars)
{
    char buf[256];
    ULONG i = 0, j = 0;

    while (label[i] && j < sizeof(buf) - 4)
        buf[j++] = label[i++];
    for (i = 0; i < chars && j < sizeof(buf) - 3; i++)
        buf[j++] = (s[i] > 0 && s[i] < 0x80) ? (char)s[i] : '?';
    buf[j++] = '\n';
    buf[j] = 0;
    OutputDebugStringA(buf);
}

/* --- check 1: NtQueryInformationThread(ThreadBasicInformation) ----------- */

/* THREAD_BASIC_INFORMATION at the CALLER's width (ULONG_PTR: 4 bytes in
 * marshal32, 8 in marshal64), spelled out so the i386 field offsets are
 * visible rather than inherited from a header. */
typedef struct _M32_TBI32 {
    ULONG ExitStatus;
    ULONG_PTR TebBaseAddress;
    ULONG_PTR UniqueProcess;
    ULONG_PTR UniqueThread;
    ULONG_PTR AffinityMask;
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
    if (Tbi.TebBaseAddress != (ULONG_PTR)NtCurrentTeb())
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

/* --- check 7: the SELF-DESCRIBING SIZE family (E22) ------------------------ *
 * A struct that carries its own sizeof is the one shape a field-perfect
 * marshaller still gets wrong: the guest writes ITS sizeof and win32k compares
 * against the HOST's, so the call is rejected before a single field is read.
 * TRACKMOUSEEVENT is 16 vs 24, CURSORINFO 20 vs 24, GUITHREADINFO 48 vs 72,
 * and INPUT 28 vs 40 (SendInput states its size in an ARGUMENT instead of a
 * field, which is why that one is hand-marshalled).
 *
 * Each check is three assertions, and they fail in different directions:
 *   - the call must SUCCEED         -> the size reached the kernel host-sized
 *   - the size must come back GUEST-sized -> the copy-out undid the rewrite
 *   - the canary after the struct must survive -> the kernel's larger write
 *     landed in the bounce buffer and not on our stack
 * Only the first fires on unmarshalled media (the kernel rejects the call and
 * writes nothing at all, so the canary survives by doing nothing). The other
 * two are the guards against "fixed the size check, corrupted the caller",
 * which is what a naive size rewrite with no bounce buffer would do. */

#define M32_TME_QUERY 0x40000000
#define M32_CANARY    0xC0DEFACEu

static int CheckSelfDescribingSizes(void)
{
    struct { TRACKMOUSEEVENT s; DWORD canary[2]; } tme;
    struct { CURSORINFO s;      DWORD canary[2]; } ci;
    struct { GUITHREADINFO s;   DWORD canary[2]; } gti;
    INPUT in;
    UINT sent;

    /* --- TRACKMOUSEEVENT: 16 -> 24 ------------------------------------- */
    tme.s.cbSize = sizeof(TRACKMOUSEEVENT);
    tme.s.dwFlags = M32_TME_QUERY;
    tme.s.hwndTrack = NULL;
    tme.s.dwHoverTime = 0;
    tme.canary[0] = M32_CANARY;
    tme.canary[1] = M32_CANARY;

    if (!TrackMouseEvent(&tme.s))
    {
        ReportHex("[marshal32] cbSize: TrackMouseEvent(TME_QUERY) failed, err =",
                  GetLastError());
        return M32_FAIL_TME_CALL;
    }
    /* The kernel restamps cbSize with ITS sizeof (24) on the query path, so
     * this is a direct read-out of whether the copy-back narrowed it again. */
    if (tme.s.cbSize != sizeof(TRACKMOUSEEVENT))
    {
        ReportHex("[marshal32] cbSize: TRACKMOUSEEVENT.cbSize came back as",
                  tme.s.cbSize);
        return M32_FAIL_TME_SIZE;
    }
    if (tme.canary[0] != M32_CANARY || tme.canary[1] != M32_CANARY)
    {
        Report("[marshal32] cbSize: TRACKMOUSEEVENT canary overwritten\n");
        return M32_FAIL_TME_CANARY;
    }

    /* --- CURSORINFO: 20 -> 24 ------------------------------------------ */
    ci.s.cbSize = sizeof(CURSORINFO);
    ci.canary[0] = M32_CANARY;
    ci.canary[1] = M32_CANARY;

    if (!GetCursorInfo(&ci.s))
    {
        ReportHex("[marshal32] cbSize: GetCursorInfo failed, err =", GetLastError());
        return M32_FAIL_CI_CALL;
    }
    if (ci.s.cbSize != sizeof(CURSORINFO))
    {
        ReportHex("[marshal32] cbSize: CURSORINFO.cbSize came back as", ci.s.cbSize);
        return M32_FAIL_CI_SIZE;
    }
    if (ci.canary[0] != M32_CANARY || ci.canary[1] != M32_CANARY)
    {
        Report("[marshal32] cbSize: CURSORINFO canary overwritten\n");
        return M32_FAIL_CI_CANARY;
    }

    /* --- GUITHREADINFO: 48 -> 72 (the widest overrun of the family) ----- */
    gti.s.cbSize = sizeof(GUITHREADINFO);
    gti.canary[0] = M32_CANARY;
    gti.canary[1] = M32_CANARY;

    if (!GetGUIThreadInfo(0, &gti.s))
    {
        ReportHex("[marshal32] cbSize: GetGUIThreadInfo failed, err =", GetLastError());
        return M32_FAIL_GTI_CALL;
    }
    if (gti.s.cbSize != sizeof(GUITHREADINFO))
    {
        ReportHex("[marshal32] cbSize: GUITHREADINFO.cbSize came back as", gti.s.cbSize);
        return M32_FAIL_GTI_SIZE;
    }
    if (gti.canary[0] != M32_CANARY || gti.canary[1] != M32_CANARY)
    {
        Report("[marshal32] cbSize: GUITHREADINFO canary overwritten\n");
        return M32_FAIL_GTI_CANARY;
    }

    /* --- INPUT: 28 -> 40, with the size in an ARGUMENT ------------------ *
     * A relative mouse move of (0,0): reaches UserSendMouseInput, moves
     * nothing, clicks nothing.
     *
     * This check used to excuse a failure unless GetLastError() was
     * ERROR_INVALID_PARAMETER, on the theory that SendInput can also fail for
     * environmental reasons (no input access, no active desktop). The control
     * run killed that idea: win32k's EngSetLastError writes
     * NtCurrentTeb()->LastErrorValue — the *64-bit* TEB — while a 32-bit guest
     * reads TEB32, so NO error code win32k sets is visible here and the
     * discriminator could never fire. (Separately tracked; it is a gap across
     * the whole win32k surface, not just this call.) An assertion that cannot
     * fail is worse than no assertion, so this is a hard failure now:
     * winlogon runs the ladder on the active desktop, the same one every other
     * guest creates windows on, and a genuine loss of input access there is
     * itself worth a red. */
    in.type = INPUT_MOUSE;
    in.mi.dx = 0;
    in.mi.dy = 0;
    in.mi.mouseData = 0;
    in.mi.dwFlags = MOUSEEVENTF_MOVE;
    in.mi.time = 0;
    in.mi.dwExtraInfo = 0;

    sent = SendInput(1, &in, sizeof(INPUT));
    if (sent != 1)
    {
        ReportHex("[marshal32] cbSize: SendInput accepted events =", sent);
        /* Meaningful at x86-64 (one TEB); at i386 see the note above. */
        ReportHex("[marshal32] cbSize: SendInput GetLastError =", GetLastError());
        return M32_FAIL_SENDINPUT;
    }

    Report("[marshal32] self-describing sizes verified "
           "(TrackMouseEvent/GetCursorInfo/GetGUIThreadInfo/SendInput)\n");
    return 0;
}

/* --- check 8: the win32k LAST-ERROR channel ------------------------------- *
 * win32k reports failure by setting last-error and returning FALSE, and it
 * sets it with EngSetLastError -> NtCurrentTeb()->LastErrorValue, which in
 * kernel mode is the thread's 64-BIT TEB. GetLastError() here reads TEB32.
 * Every error code the USER/GDI surface produced therefore landed in a TEB no
 * 32-bit process ever reads: the app got the FALSE and never the reason.
 *
 * TrackMouseEvent is the probe because user32.spec forwards it STRAIGHT to
 * NtUserTrackMouseEvent with no client-side validation at all, so every code
 * asserted below can only have come from the kernel. It is driven three ways:
 *
 *   (a) a cbSize the kernel rejects -> ERROR_INVALID_PARAMETER. The E22 size
 *       rewrite is value-guarded (it rewrites only the exact 32-bit sizeof),
 *       so a cbSize of 1 passes through untouched and really does reach the
 *       kernel's version check.
 *   (b) a legal cbSize with an illegal FLAG bit -> ERROR_INVALID_FLAGS, from
 *       a different branch of the same function. Two distinct codes out of
 *       one call is what makes this "the code travelled" rather than "an
 *       error appeared" — a bridge that hardcoded, guessed, or merely zeroed
 *       would satisfy one of these and not both.
 *   (c) a call that SUCCEEDS must leave the guest's own last-error alone.
 *       win32k does not touch last-error on the TME_QUERY success path, so a
 *       sentinel set here must survive the syscall. This is the assertion
 *       that rejects the naive bridge — an unconditional copy-down of the
 *       64-bit TEB would hand the guest whatever some earlier syscall left
 *       there, turning a fix for failures into a corruption of successes.
 *
 * Only (a) and (b) can see the original bug; on unfixed media both report the
 * sentinel, which is the bug printing its own name. (c) passes before and
 * after — it is a guard on the fix, not a detector of the bug. */

#define M32_LERR_SENTINEL 0x5A5A0082u
#define M32_TME_BOGUS_FLAG 0x00000004u /* not TME_{CANCEL,QUERY,NONCLIENT,LEAVE,HOVER} */

static int CheckWin32kLastError(void)
{
    TRACKMOUSEEVENT tme;
    DWORD err;

    /* (a) a size the kernel's version check rejects. */
    tme.cbSize = 1;
    tme.dwFlags = M32_TME_QUERY;
    tme.hwndTrack = NULL;
    tme.dwHoverTime = 0;
    SetLastError(M32_LERR_SENTINEL);
    if (TrackMouseEvent(&tme))
    {
        Report("[marshal32] lasterr: TrackMouseEvent ACCEPTED cbSize 1 — rung is invalid\n");
        return M32_FAIL_LERR_CALL;
    }
    err = GetLastError();
    if (err != ERROR_INVALID_PARAMETER)
    {
        ReportHex("[marshal32] lasterr: bad cbSize should be 0x57, guest sees", err);
        return M32_FAIL_LERR_PARAM;
    }

    /* (b) the same call, a different kernel branch, a different code. */
    tme.cbSize = sizeof(TRACKMOUSEEVENT);
    tme.dwFlags = M32_TME_QUERY | M32_TME_BOGUS_FLAG;
    SetLastError(M32_LERR_SENTINEL);
    if (TrackMouseEvent(&tme))
    {
        Report("[marshal32] lasterr: TrackMouseEvent ACCEPTED a bogus flag — rung is invalid\n");
        return M32_FAIL_LERR_CALL;
    }
    err = GetLastError();
    if (err != ERROR_INVALID_FLAGS)
    {
        ReportHex("[marshal32] lasterr: bad flag should be 0x3EC, guest sees", err);
        return M32_FAIL_LERR_FLAGS;
    }

    /* (c) a success must not disturb what the guest itself last set. */
    tme.cbSize = sizeof(TRACKMOUSEEVENT);
    tme.dwFlags = M32_TME_QUERY;
    tme.hwndTrack = NULL;
    tme.dwHoverTime = 0;
    SetLastError(M32_LERR_SENTINEL);
    if (!TrackMouseEvent(&tme))
    {
        ReportHex("[marshal32] lasterr: the control TME_QUERY failed, err =", GetLastError());
        return M32_FAIL_LERR_QUERY;
    }
    err = GetLastError();
    if (err != M32_LERR_SENTINEL)
    {
        ReportHex("[marshal32] lasterr: a SUCCEEDING win32k call overwrote last-error with",
                  err);
        return M32_FAIL_LERR_CLOBBER;
    }

    Report("[marshal32] win32k last-error reaches the guest "
           "(invalid-param, invalid-flags, preserved across success)\n");
    return 0;
}

/* --- check 9: Wow6432Node registry redirection (E24) ----------------------- *
 * A 32-bit process must get its own view of HKLM\Software. The kernel splices
 * WOW6432NODE in at the SOFTWARE boundary (CmpParseKey -> cmwow64.rs), so the
 * SAME path names two different keys depending on who is asking.
 *
 * "The write went somewhere" is not evidence — with no redirection at all a
 * create still succeeds and a read still finds it. So this rung asserts the
 * two views DISAGREE, by two independent routes:
 *   (a) the default-view handle's OWN NAME (NtQueryKey/KeyNameInformation)
 *       must contain Wow6432Node. That says WHERE it went, from the kernel's
 *       mouth, not from ours.
 *   (b) the identical path opened with KEY_WOW64_64KEY must NOT see the value
 *       we just wrote. That says the two views are genuinely distinct
 *       objects. Unredirected, (a) and (b) both fail; a bridge that faked
 *       either one would still have to satisfy the other.
 *
 * (c) then pins the property the whole design rests on: a key that exists ONLY
 * in the 64-bit hive must still resolve from the 32-bit view, via the
 * miss-fallback. Nothing populates Wow6432Node on this system, so without that
 * fallback every OS key a 32-bit app reads would vanish — redirection without
 * population is a regression, and this is the assertion that would catch it.
 * It passes before AND after the feature: it is a guard, not a detector. */

typedef LONG(WINAPI *M32_QUERYKEY_FN)(HANDLE, ULONG, VOID *, ULONG, PULONG);

#define M32_KEY_NAME_INFORMATION 3
#define M32_RDIR_SENTINEL 0x5EDC0024u

/* Case-insensitive search for a wide needle in a counted wide haystack. */
static int M32WideContains(const WCHAR *hay, ULONG hayChars, const WCHAR *needle)
{
    ULONG n = 0, i, j;

    while (needle[n])
        n++;
    if (hayChars < n)
        return 0;

    for (i = 0; i + n <= hayChars; i++)
    {
        for (j = 0; j < n; j++)
        {
            WCHAR a = hay[i + j], b = needle[j];
            if (a >= L'a' && a <= L'z') a = (WCHAR)(a - 32);
            if (b >= L'a' && b <= L'z') b = (WCHAR)(b - 32);
            if (a != b)
                break;
        }
        if (j == n)
            return 1;
    }
    return 0;
}

static int CheckRegistryRedirection(void)
{
    static WCHAR ScratchPath[] = L"\\Registry\\Machine\\Software\\vibeos_m32redir";
    static WCHAR SharedPath[] = L"\\Registry\\Machine\\Software\\Microsoft";
    static WCHAR ValName[] = L"Wow64RedirectProbe";
    static WCHAR NodeName[] = L"Wow6432Node";
    HMODULE nt = GetModuleHandleA("ntdll.dll");
    M32_CREATEKEY_FN CreateKey;
    M32_OPENKEY_FN OpenKey;
    M32_SETVAL_FN SetValue;
    M32_QUERYVAL_FN QueryValue;
    M32_QUERYKEY_FN QueryKey;
    M32_USTR uKey, uVal;
    M32_OA oa;
    M32_KVPI info;
    /* KEY_NAME_INFORMATION: ULONG NameLength; WCHAR Name[]. */
    struct { ULONG NameLength; WCHAR Name[256]; } nameInfo;
    HANDLE key = NULL, key64 = NULL;
    ULONG disp = 0, got = 0;
    DWORD data = M32_RDIR_SENTINEL;
    LONG st;

    if (!nt)
        return M32_FAIL_REG_NTDLL;
    CreateKey = (M32_CREATEKEY_FN)GetProcAddress(nt, "NtCreateKey");
    OpenKey = (M32_OPENKEY_FN)GetProcAddress(nt, "NtOpenKey");
    SetValue = (M32_SETVAL_FN)GetProcAddress(nt, "NtSetValueKey");
    QueryValue = (M32_QUERYVAL_FN)GetProcAddress(nt, "NtQueryValueKey");
    QueryKey = (M32_QUERYKEY_FN)GetProcAddress(nt, "NtQueryKey");
    if (!CreateKey || !OpenKey || !SetValue || !QueryValue || !QueryKey)
        return M32_FAIL_REG_NTDLL;

    M32InitUstr(&uKey, ScratchPath);
    M32InitUstr(&uVal, ValName);
    oa.Length = sizeof(oa);
    oa.RootDirectory = NULL;
    oa.ObjectName = &uKey;
    oa.Attributes = 0x40; /* OBJ_CASE_INSENSITIVE */
    oa.SecurityDescriptor = NULL;
    oa.SecurityQualityOfService = NULL;

    /* Default view. For a 32-bit process this must resolve into Wow6432Node.
     * Volatile: a probe must not survive the boot. */
    st = CreateKey(&key, 0xF003F /* KEY_ALL_ACCESS */, &oa, 0, NULL,
                   1 /* REG_OPTION_VOLATILE */, &disp);
    if (st < 0)
    {
        ReportHex("[marshal32] redirect: scratch create failed, status =", (DWORD)st);
        return M32_FAIL_RDIR_CREATE;
    }
    st = SetValue(key, &uVal, 0, 4 /* REG_DWORD */, &data, sizeof(data));
    if (st < 0)
    {
        CloseHandle(key);
        ReportHex("[marshal32] redirect: scratch set failed, status =", (DWORD)st);
        return M32_FAIL_RDIR_CREATE;
    }

    /* (a) ask the KERNEL where that handle actually points. */
    nameInfo.NameLength = 0;
    st = QueryKey(key, M32_KEY_NAME_INFORMATION, &nameInfo, sizeof(nameInfo), &got);
    if (st < 0 ||
        !M32WideContains(nameInfo.Name, nameInfo.NameLength / sizeof(WCHAR), NodeName))
    {
        CloseHandle(key);
        ReportHex("[marshal32] redirect: default view is not in Wow6432Node, status =",
                  (DWORD)st);
        if (st >= 0)
            ReportWide("[marshal32] redirect: the kernel calls that handle ",
                       nameInfo.Name, nameInfo.NameLength / sizeof(WCHAR));
        return M32_FAIL_RDIR_NAME;
    }
    CloseHandle(key);

    /* (b) the SAME path in the 64-bit view must be a different key, so the
     * value we just wrote must not be visible through it. */
    key64 = NULL;
    st = OpenKey(&key64, KEY_READ | M32_KEY_WOW64_64KEY, &oa);
    if (st >= 0)
    {
        got = 0;
        st = QueryValue(key64, &uVal, 2 /* KeyValuePartialInformation */,
                        &info, sizeof(info), &got);
        CloseHandle(key64);
        if (st >= 0)
        {
            Report("[marshal32] redirect: 64-bit view can see the 32-bit value\n");
            return M32_FAIL_RDIR_LEAK;
        }
    }
    /* NOT finding the key at all is the expected outcome here: nothing created
     * it in the 64-bit hive. Only a successful VALUE read is a failure. */

    /* (c) a key that exists only 64-bit must still resolve from the 32-bit
     * view, or every OS key a 32-bit app reads would disappear. */
    M32InitUstr(&uKey, SharedPath);
    key = NULL;
    st = OpenKey(&key, KEY_READ, &oa);
    if (st < 0)
    {
        ReportHex("[marshal32] redirect: 64-bit-only key stopped resolving, status =",
                  (DWORD)st);
        return M32_FAIL_RDIR_SHARED;
    }
    CloseHandle(key);

    /* And the explicit 64-bit selector must still work on it. */
    key = NULL;
    st = OpenKey(&key, KEY_READ | M32_KEY_WOW64_64KEY, &oa);
    if (st < 0)
    {
        ReportHex("[marshal32] redirect: KEY_WOW64_64KEY failed on a shared key, status =",
                  (DWORD)st);
        return M32_FAIL_RDIR_64VIEW;
    }
    CloseHandle(key);

    Report("[marshal32] Wow6432Node redirection: private 32-bit view, "
           "64-bit view distinct, miss falls back\n");
    return 0;
}

/* ---------------------------------------------------------------------------
 * select() on a fresh socket — the first check that reaches an AFD payload.
 *
 * IOCTL_AFD_SELECT carries AFD_POLL_INFO, which the E25 audit classified
 * SEMANTIC rather than merely WIDENING: the header grows 16 -> 24 AND the
 * Handles[] element stride grows 12 -> 16, so the array cannot be copied
 * wholesale at either end. It is also the only IN/OUT payload in the audited
 * set — the driver writes Events and Status back per handle — which makes it
 * the one that fails in both directions if the marshaller is wrong.
 *
 * Deliberately NETWORK-FREE. No bind, no connect, no packet: a fresh UDP
 * socket with a zero timeout still builds a full AFD_POLL_INFO, issues the
 * IOCTL, and takes the write-back. Sending anything would drag in the netstack
 * intermittent tracked as #83 and make a red run ambiguous, which is exactly
 * what a first-exercise rung must not be.
 * ------------------------------------------------------------------------ */
static int CheckSocketSelect(void)
{
    WSADATA wsa;
    SOCKET s;
    /* The fd_set is followed by a canary in the same object. If a marshaller
     * ever sizes the write-back from the guest's HandleCount without measuring
     * the guest's buffer, it lands here instead of silently past the end. */
    struct
    {
        fd_set set;
        DWORD  guard;
    } fds;
    struct timeval tv;
    int rc;

    /* Breadcrumbs, one per Winsock call. The first boot of this rung faulted
     * with rip == the faulting address and no output at all, which narrows to
     * "somewhere in here" and no further. Each line below costs one serial
     * write and turns that into an exact call. */
    Report("[marshal32] select: calling WSAStartup\n");
    if (WSAStartup(0x0202, &wsa) != 0)
    {
        Report("[marshal32] FAIL: WSAStartup\n");
        return M32_FAIL_WSA_STARTUP;
    }

    /* SOCK_DGRAM needs no peer, so this cannot block or touch the wire. If the
     * Winsock provider (msafd.dll) is not staged into SysWOW64 this is where it
     * shows up, and it shows up as a plain failure rather than a hang. */
    Report("[marshal32] select: WSAStartup returned, calling socket()\n");
    s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (s == INVALID_SOCKET)
    {
        ReportHex("[marshal32] FAIL: socket() failed, WSAGetLastError =",
                  (DWORD)WSAGetLastError());
        WSACleanup();
        return M32_FAIL_WSA_SOCKET;
    }

    fds.guard = 0x5A5AA5A5;
    FD_ZERO(&fds.set);
    FD_SET(s, &fds.set);
    tv.tv_sec = 0;
    tv.tv_usec = 0;

    Report("[marshal32] select: socket created, calling select()\n");
    rc = select(0, &fds.set, NULL, NULL, &tv);
    Report("[marshal32] select: select() returned\n");

    if (rc == SOCKET_ERROR)
    {
        ReportHex("[marshal32] FAIL: select() errored, WSAGetLastError =",
                  (DWORD)WSAGetLastError());
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SELECT_CALL;
    }
    if (fds.guard != 0x5A5AA5A5)
    {
        ReportHex("[marshal32] FAIL: select() overwrote past the fd_set, guard =",
                  fds.guard);
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SELECT_GUARD;
    }
    /* Nothing was ever sent to it, so it must not be readable. A marshaller
     * that mis-shapes the write-back tends to report garbage readiness here
     * rather than fail outright, so this is the check with real teeth. */
    if (rc != 0)
    {
        ReportHex("[marshal32] FAIL: fresh socket reported ready, select =", (DWORD)rc);
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SELECT_READY;
    }
    if (fds.set.fd_count != 0)
    {
        ReportHex("[marshal32] FAIL: select() left fd_count wrong, count =",
                  (DWORD)fds.set.fd_count);
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SELECT_COUNT;
    }

    closesocket(s);
    WSACleanup();
    Report("[marshal32] select: AFD_POLL_INFO round-tripped, fd_set intact\n");
    return 0;
}

/* --------------------------------------------------------------------------
 * sendto() on a fresh UDP socket — the deep-marshaller rung.
 *
 * IOCTL_AFD_SEND_DATAGRAM carries AFD_SEND_INFO_UDP, 56 -> 104, the only AFD
 * payload with nested structs. Two independent things can be marshalled wrong
 * and this rung separates them:
 *
 *   BufferArray/BufferCount — the WSABUF array widens 8 -> 16 per element, so
 *   a wrong stride sends the wrong bytes or the wrong length. The byte count
 *   check below catches it, because IOSB.Information IS that length.
 *
 *   TdiConnection.RemoteAddress — a guest pointer to a TRANSPORT_ADDRESS.
 *   Checking that sendto merely succeeded would NOT prove this one: the strong
 *   evidence is that the socket comes back IMPLICITLY BOUND. AFD only reaches
 *   WarmSocketForBind by dereferencing RemoteAddress for Address[0].
 *   AddressType, so a bound AF_INET socket with a real port means the driver
 *   read our address family through the pointer we widened. A mis-widened
 *   pointer yields garbage there and the bind fails or binds the wrong family.
 *
 * The datagram goes to the discard port on loopback, so nothing has to be
 * listening and no peer is involved; a send that the stack itself declines is
 * reported with its WSA error so it reads as an environment failure rather
 * than a marshalling one. (The select rung above stays network-free on
 * purpose; this one cannot be — sending IS the thing under test.)
 * ------------------------------------------------------------------------ */
static int CheckSocketSendTo(void)
{
    WSADATA wsa;
    SOCKET s;
    /* Same canary discipline as the select rung: the sockaddr the driver reads
     * through our widened pointer is followed by a guard in the same object, so
     * an over-read that turns into an over-write lands here. */
    struct
    {
        struct sockaddr_in addr;
        DWORD              guard;
    } to;
    struct sockaddr_in local;
    int locallen;
    static const char Payload[] = "marshal32-udp";
    const int PayloadLen = (int)(sizeof(Payload) - 1);
    int rc;

    Report("[marshal32] sendto: calling WSAStartup\n");
    if (WSAStartup(0x0202, &wsa) != 0)
    {
        Report("[marshal32] FAIL: WSAStartup\n");
        return M32_FAIL_WSA_STARTUP;
    }

    Report("[marshal32] sendto: calling socket()\n");
    s = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (s == INVALID_SOCKET)
    {
        ReportHex("[marshal32] FAIL: socket() failed, WSAGetLastError =",
                  (DWORD)WSAGetLastError());
        WSACleanup();
        return M32_FAIL_WSA_SOCKET;
    }

    /* No CRT, so the sockaddr is filled field by field. 0x0100007F is
     * 127.0.0.1 already in network order on a little-endian guest; port 9 is
     * discard, which needs no listener. */
    to.guard                 = 0x5A5AA5A5;
    to.addr.sin_family       = AF_INET;
    to.addr.sin_port         = htons(9);
    to.addr.sin_addr.s_addr  = 0x0100007F;

    /* Deliberately NOT bound first: an unbound socket is SOCKET_STATE_CREATED,
     * which is the state that forces AfdPacketSocketWriteData down the
     * TaBuildNullTransportAddress(RemoteAddress->Address[0].AddressType) path.
     * Binding first would skip the very dereference this rung exists to prove. */
    Report("[marshal32] sendto: calling sendto()\n");
    rc = sendto(s, Payload, PayloadLen, 0,
                (const struct sockaddr *)&to.addr, (int)sizeof(to.addr));
    Report("[marshal32] sendto: sendto() returned\n");

    if (to.guard != 0x5A5AA5A5)
    {
        ReportHex("[marshal32] FAIL: sendto() overwrote past the sockaddr, guard =",
                  to.guard);
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SENDTO_GUARD;
    }

    /* A send that the TRANSPORT declined is not a marshalling failure, and this
     * rung must not turn one into a red ladder. A board without a NIC netstack
     * drives (the Pi 4) gets WSAENETUNREACH, and the netstack has no loopback
     * path, so "no route" is an EXPECTED environment here rather than a defect.
     * (QEMU smoke always has QEMU's default NIC, so there the send succeeds.) The datagram leaving the box was never the
     * claim; what is being tested is whether AFD could read the payload we
     * widened. So the error is reported and execution continues to the bind
     * check, which is the assertion that actually distinguishes the two. */
    if (rc == SOCKET_ERROR)
        ReportHex("[marshal32] sendto: transport declined the datagram (not fatal), "
                  "WSAGetLastError =", (DWORD)WSAGetLastError());
    else if (rc != PayloadLen)
    {
        /* It DID go through, so the byte count is meaningful and exact: this is
         * IOSB.Information, which catches a wrong WSABUF stride directly. */
        ReportHex("[marshal32] FAIL: sendto() sent the wrong length, sent =", (DWORD)rc);
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SENDTO_COUNT;
    }

    /* The teeth, and they need no network. AfdPacketSocketWriteData reaches
     * WarmSocketForBind only by dereferencing RemoteAddress for
     * Address[0].AddressType, and that happens BEFORE the datagram is handed to
     * the transport. So the FAMILY coming back AF_INET is the proof: it is
     * copied straight from the address we pointed at, through the pointer this
     * marshaller widened. A mis-widened pointer yields a garbage AddressType,
     * TaLengthOfTransportAddressByType returns 0, TaBuildNullTransportAddress
     * returns NULL, and the send bails STATUS_NO_MEMORY with the socket still
     * unbound — which lands here, red, network or no network.
     *
     * The PORT is deliberately not checked, and the first cut of this rung was
     * wrong to check it. The implicit bind is to a NULL transport address by
     * construction: TdiBuildNullTransportAddressInPlace (tdiconn.c) sets
     * AddressType and then RtlZeroMemory's the address bytes, port included. So
     * port 0 is the correct and only possible answer here, and asserting a
     * non-zero one demanded something the driver never produces — it failed the
     * rung while every marshalling property under test had actually passed. */
    local.sin_family = 0;
    local.sin_port   = 0;
    locallen = (int)sizeof(local);
    if (getsockname(s, (struct sockaddr *)&local, &locallen) == SOCKET_ERROR ||
        local.sin_family != AF_INET)
    {
        ReportHex("[marshal32] FAIL: no implicit bind after sendto, family =",
                  (DWORD)local.sin_family);
        closesocket(s);
        WSACleanup();
        return M32_FAIL_SENDTO_BIND;
    }

    closesocket(s);
    WSACleanup();
    Report("[marshal32] sendto: AFD_SEND_INFO_UDP marshalled, "
           "socket implicitly bound through the widened RemoteAddress\n");
    return 0;
}

int start(void) /* i386 symbol _start (-e _start); x64 symbol start (-e start) */
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

    rc = CheckSelfDescribingSizes();
    if (rc) ExitProcess(rc);

    rc = CheckWin32kLastError();
    if (rc) ExitProcess(rc);

    rc = CheckRegistryRedirection();
    if (rc) ExitProcess(rc);

    rc = CheckSocketSelect();
    if (rc) ExitProcess(rc);

    rc = CheckSocketSendTo();
    if (rc) ExitProcess(rc);

    Report("[marshal32] PASS: TBI + IOCP + monitors + out-cells + reg-names "
           "+ wow64-flags + cbSize + last-error + reg-redirect + select "
           "+ sendto all verified\n");
    ExitProcess(M32_PASS);
    return 0;
}
