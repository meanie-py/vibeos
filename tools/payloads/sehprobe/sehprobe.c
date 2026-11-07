/* sehprobe — can user mode crash the kernel through a dropped __except?
 *
 * The kernel was translated from C built with DUMMY_PSEH, where `_SEH2_EXCEPT` never runs, so
 * a port could keep a try body and drop its handler without any test noticing. In the kernel
 * the exception then has nobody to catch it and the machine bugchecks. scan_dead_seh_handlers
 * lists every such function; this payload calls the ones a user process can reach, the way a
 * buggy or hostile program would: a bad pointer where the C caught the fault, or a count past
 * a limit where the callee RAISES by contract (KeSuspendThread, KeReleaseSemaphore,
 * KeReleaseMutant). Before 2026-11-06 the first case, one ReleaseSemaphore past the maximum,
 * bugchecked all three arches (0x1E, code C0000047).
 *
 * Each case prints "begin" before the call, so if the kernel dies the serial log ends on the
 * case that killed it. A case passes when the call returns the status Windows returns.
 * Verdict: "[SEHPROBE] done VERDICT=PASS". */
#define _WIN32_WINNT 0x0601
#include <windows.h>
#include <winternl.h>

ULONG __cdecl DbgPrint(const char *Format, ...);

static void SAY(const char *Format, ...)
{
    char line[512];
    va_list args;
    DWORD n;
    HANDLE out = GetStdHandle(STD_OUTPUT_HANDLE);
    int len = lstrlenA(lstrcpyA(line, "[SEHPROBE] "));
    va_start(args, Format);
    wvsprintfA(line + len, Format, args);
    va_end(args);
    DbgPrint("%s", line);
    if (out && out != INVALID_HANDLE_VALUE)
        WriteFile(out, line, lstrlenA(line), &n, NULL);
}

/* A user address nothing maps: the first page is reserved on NT. */
#define BAD_PTR ((void *)(ULONG_PTR)0x10)

#ifndef STATUS_ACCESS_VIOLATION
#define STATUS_ACCESS_VIOLATION ((NTSTATUS)0xC0000005L)
#endif
#define STATUS_SUSPEND_COUNT_EXCEEDED ((NTSTATUS)0xC000004AL)
#define STATUS_SEMAPHORE_LIMIT_EXCEEDED ((NTSTATUS)0xC0000047L)
#define STATUS_MUTANT_NOT_OWNED ((NTSTATUS)0xC0000046L)

typedef NTSTATUS(NTAPI *NtCreateNamedPipeFile_t)(PHANDLE, ULONG, POBJECT_ATTRIBUTES, PIO_STATUS_BLOCK,
                                                 ULONG, ULONG, ULONG, ULONG, ULONG, ULONG, ULONG,
                                                 ULONG, ULONG, PLARGE_INTEGER);
typedef NTSTATUS(NTAPI *NtCreateMailslotFile_t)(PHANDLE, ULONG, POBJECT_ATTRIBUTES, PIO_STATUS_BLOCK,
                                                ULONG, ULONG, ULONG, PLARGE_INTEGER);
typedef NTSTATUS(NTAPI *NtQueryKey_t)(HANDLE, int, PVOID, ULONG, PULONG);
typedef NTSTATUS(NTAPI *NtQueryObject_t)(HANDLE, int, PVOID, ULONG, PULONG);
typedef NTSTATUS(NTAPI *NtSuspendThread_t)(HANDLE, PULONG);
typedef NTSTATUS(NTAPI *NtResumeThread_t)(HANDLE, PULONG);
typedef NTSTATUS(NTAPI *NtReleaseSemaphore_t)(HANDLE, LONG, PLONG);
typedef NTSTATUS(NTAPI *NtReleaseMutant_t)(HANDLE, PLONG);
typedef NTSTATUS(NTAPI *NtSignalAndWaitForSingleObject_t)(HANDLE, HANDLE, BOOLEAN, PLARGE_INTEGER);
typedef VOID(NTAPI *RtlInitUnicodeString_t)(PUNICODE_STRING, PCWSTR);

static HMODULE g_ntdll;
#define NT(name) ((name##_t)GetProcAddress(g_ntdll, #name))

static int g_pass = 1;

/* `want2` is an alternative Windows also returns (0 for none). */
static void grade(const char *name, NTSTATUS got, NTSTATUS want, NTSTATUS want2)
{
    int ok = got == want || (want2 && got == want2);
    SAY("%s %s (status %08lx, want %08lx)\n", name, ok ? "PASS" : "FAIL", (ULONG)got, (ULONG)want);
    g_pass &= ok;
}

static void init_oa(OBJECT_ATTRIBUTES *oa, UNICODE_STRING *us, PCWSTR path)
{
    NT(RtlInitUnicodeString)(us, path);
    InitializeObjectAttributes(oa, us, OBJ_CASE_INSENSITIVE, NULL, NULL);
}

static DWORD WINAPI idle(void *arg)
{
    (void)arg;
    for (;;)
        Sleep(1000);
}

/* KeSuspendThread raises STATUS_SUSPEND_COUNT_EXCEEDED past MAXIMUM_SUSPEND_COUNT (127);
 * PsSuspendThread catches it and NtSuspendThread returns it. */
static void case_suspend(void)
{
    HANDLE t = CreateThread(NULL, 0, idle, NULL, 0, NULL);
    NTSTATUS st = 0;
    ULONG prev = 0;
    int i;
    SAY("suspend begin\n");
    for (i = 0; i < 130 && st >= 0; i++)
        st = NT(NtSuspendThread)(t, &prev);
    SAY("suspend stopped after %d calls, previous count %lu\n", i, prev);
    grade("suspend", st, STATUS_SUSPEND_COUNT_EXCEEDED, 0);
    for (i = 0; i < 130; i++)
        NT(NtResumeThread)(t, NULL);
    TerminateThread(t, 0);
    CloseHandle(t);
}

/* KeReleaseSemaphore raises STATUS_SEMAPHORE_LIMIT_EXCEEDED; NtReleaseSemaphore catches it. */
static void case_semaphore(void)
{
    HANDLE s = CreateSemaphoreW(NULL, 1, 1, NULL);
    SAY("semaphore begin\n");
    grade("semaphore", NT(NtReleaseSemaphore)(s, 5, NULL), STATUS_SEMAPHORE_LIMIT_EXCEEDED, 0);
    CloseHandle(s);
}

/* KeReleaseMutant raises STATUS_MUTANT_NOT_OWNED; NtReleaseMutant catches it. */
static DWORD WINAPI release_foreign(void *m)
{
    grade("mutant", NT(NtReleaseMutant)((HANDLE)m, NULL), STATUS_MUTANT_NOT_OWNED, 0);
    return 0;
}

static void case_mutant(void)
{
    HANDLE m = CreateMutexW(NULL, TRUE, NULL); /* owned by THIS thread */
    HANDLE t;
    SAY("mutant begin\n");
    t = CreateThread(NULL, 0, release_foreign, m, 0, NULL);
    WaitForSingleObject(t, INFINITE);
    CloseHandle(t);
    ReleaseMutex(m);
    CloseHandle(m);
}

/* NtSignalAndWaitForSingleObject releases the signal object under the same catches. */
static DWORD WINAPI signal_foreign(void *m)
{
    HANDLE e = CreateEventW(NULL, TRUE, TRUE, NULL);
    grade("signal_mutant", NT(NtSignalAndWaitForSingleObject)((HANDLE)m, e, FALSE, NULL),
          STATUS_MUTANT_NOT_OWNED, 0);
    CloseHandle(e);
    return 0;
}

static void case_signal(void)
{
    HANDLE s = CreateSemaphoreW(NULL, 1, 1, NULL); /* already full */
    HANDLE e = CreateEventW(NULL, TRUE, TRUE, NULL);
    HANDLE m = CreateMutexW(NULL, TRUE, NULL);
    HANDLE t;
    SAY("signal begin\n");
    grade("signal_semaphore", NT(NtSignalAndWaitForSingleObject)(s, e, FALSE, NULL),
          STATUS_SEMAPHORE_LIMIT_EXCEEDED, 0);
    t = CreateThread(NULL, 0, signal_foreign, m, 0, NULL);
    WaitForSingleObject(t, INFINITE);
    CloseHandle(t);
    ReleaseMutex(m);
    CloseHandle(m);
    CloseHandle(e);
    CloseHandle(s);
}

/* NtCreateNamedPipeFile reads the caller's DefaultTimeout under SEH. */
static void case_pipe_timeout(void)
{
    OBJECT_ATTRIBUTES oa;
    UNICODE_STRING us;
    IO_STATUS_BLOCK iosb;
    HANDLE h = NULL;
    NTSTATUS st;
    SAY("pipe_timeout begin\n");
    init_oa(&oa, &us, L"\\Device\\NamedPipe\\sehprobe");
    st = NT(NtCreateNamedPipeFile)(&h, GENERIC_READ | SYNCHRONIZE, &oa, &iosb,
                                   FILE_SHARE_READ | FILE_SHARE_WRITE, FILE_CREATE, 0, 0, 0, 0, 1,
                                   4096, 4096, (PLARGE_INTEGER)BAD_PTR);
    grade("pipe_timeout", st, STATUS_ACCESS_VIOLATION, 0);
    if (st >= 0)
        CloseHandle(h);
}

/* NtCreateMailslotFile reads the caller's ReadTimeout under SEH. */
static void case_mailslot_timeout(void)
{
    OBJECT_ATTRIBUTES oa;
    UNICODE_STRING us;
    IO_STATUS_BLOCK iosb;
    HANDLE h = NULL;
    NTSTATUS st;
    SAY("mailslot_timeout begin\n");
    init_oa(&oa, &us, L"\\Device\\Mailslot\\sehprobe");
    st = NT(NtCreateMailslotFile)(&h, GENERIC_READ | SYNCHRONIZE, &oa, &iosb, 0, 0, 512,
                                  (PLARGE_INTEGER)BAD_PTR);
    grade("mailslot_timeout", st, STATUS_ACCESS_VIOLATION, 0);
    if (st >= 0)
        CloseHandle(h);
}

/* CmQueryKey writes the caller's buffer under SEH. */
static void case_query_key(void)
{
    HKEY k;
    ULONG len = 0;
    SAY("query_key begin\n");
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SOFTWARE", 0, KEY_READ, &k) != ERROR_SUCCESS)
    {
        SAY("query_key FAIL (RegOpenKeyExW)\n");
        g_pass = 0;
        return;
    }
    grade("query_key", NT(NtQueryKey)((HANDLE)k, 0 /* KeyBasicInformation */, BAD_PTR, 256, &len),
          STATUS_ACCESS_VIOLATION, 0);
    RegCloseKey(k);
}

/* ObQueryNameString writes the caller's buffer under SEH. */
static void case_query_name(void)
{
    HKEY k;
    ULONG len = 0;
    SAY("query_name begin\n");
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SOFTWARE", 0, KEY_READ, &k) != ERROR_SUCCESS)
    {
        SAY("query_name FAIL (RegOpenKeyExW)\n");
        g_pass = 0;
        return;
    }
    grade("query_name", NT(NtQueryObject)((HANDLE)k, 1 /* ObjectNameInformation */, BAD_PTR, 512, &len),
          STATUS_ACCESS_VIOLATION, STATUS_DATATYPE_MISALIGNMENT);
    RegCloseKey(k);
}

/* CcCopyRead copies the cached file into the caller's buffer; a fault there must come back
 * as an error (the C turns it into STATUS_INVALID_USER_BUFFER), never take the kernel down.
 * NtReadFile probes the buffer first, so the fault needs a race: one thread reads a cached
 * system DLL in 1 MB chunks while another keeps flipping the buffer's protection. Graded on
 * surviving; the status counts say whether the race was actually hit. */
#define RACE_BYTES (1024 * 1024)
static volatile LONG g_race_stop;
static BYTE *g_race_buf;

static DWORD WINAPI flipper(void *arg)
{
    DWORD old;
    (void)arg;
    while (!g_race_stop)
    {
        /* Only the LAST page: NtReadFile's probe then usually passes, and the 1 MB copy is
         * well under way when it reaches the page, so the fault lands inside CcCopyRead. */
        VirtualProtect(g_race_buf + RACE_BYTES - 4096, 4096, PAGE_NOACCESS, &old);
        VirtualProtect(g_race_buf + RACE_BYTES - 4096, 4096, PAGE_READWRITE, &old);
    }
    return 0;
}

static void case_cached_read_race(void)
{
    WCHAR path[MAX_PATH];
    HANDLE f, t;
    DWORD got, start, reads = 0, ok = 0, failed = 0, last_error = 0, in_copy = 0, at_probe = 0;
    SAY("cached_read_race begin\n");
    GetSystemDirectoryW(path, MAX_PATH);
    lstrcatW(path, L"\\shell32.dll");
    f = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, 0, NULL);
    g_race_buf = VirtualAlloc(NULL, RACE_BYTES, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (f == INVALID_HANDLE_VALUE || !g_race_buf)
    {
        SAY("cached_read_race FAIL (setup: file %lu)\n", f == INVALID_HANDLE_VALUE ? GetLastError() : 0);
        g_pass = 0;
        return;
    }
    ReadFile(f, g_race_buf, RACE_BYTES, &got, NULL); /* warm the cache */
    g_race_stop = 0;
    t = CreateThread(NULL, 0, flipper, NULL, 0, NULL);
    start = GetTickCount();
    while (GetTickCount() - start < 5000)
    {
        SetFilePointer(f, 0, NULL, FILE_BEGIN);
        reads++;
        if (ReadFile(f, g_race_buf, RACE_BYTES, &got, NULL))
            ok++;
        else
        {
            failed++;
            last_error = GetLastError();
            if (last_error == ERROR_INVALID_USER_BUFFER)
                in_copy++; /* the fault landed inside the cache manager's copy */
            else if (last_error == ERROR_NOACCESS)
                at_probe++;
        }
    }
    g_race_stop = 1;
    WaitForSingleObject(t, INFINITE);
    CloseHandle(t);
    CloseHandle(f);
    VirtualFree(g_race_buf, 0, MEM_RELEASE);
    SAY("cached_read_race PASS (survived %lu reads: %lu ok, %lu failed: %lu in the Cc copy, %lu at the "
        "probe, last error %lu)\n", reads, ok, failed, in_copy, at_probe, last_error);
}

int main(void)
{
    g_ntdll = GetModuleHandleW(L"ntdll.dll");
    SAY("begin\n");
    /* Raise-by-design first: each is one call, and each is a kernel crash if dropped. */
    case_semaphore();
    case_mutant();
    case_suspend();
    case_signal();
    /* Then the bad-pointer class. */
    case_pipe_timeout();
    case_mailslot_timeout();
    case_query_key();
    case_query_name();
    /* Last: a race, so it can only end the run by crashing the kernel, never mid-sequence. */
    case_cached_read_race();
    SAY("done VERDICT=%s\n", g_pass ? "PASS" : "FAIL");
    return g_pass ? 0 : 1;
}
