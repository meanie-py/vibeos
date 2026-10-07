/*
 * apc64.exe — x86-64 guest: user APCs into guest code, and what a guest thread
 * leaves behind when it ends (docs/engineering/wow64-x64-guest-design.md,
 * "Owed" 1 and 4). x86-64 only: the 32-bit ladder has no counterpart yet.
 * Built like the other x64 guests (no CRT, plain `start`, kernel32 only); it
 * runs under FEX on arm64 and natively on amd64, the native run being the
 * oracle that the guest itself is right.
 *
 * Under FEX every user APC is delivered through the NATIVE KiUserApcDispatcher,
 * and its routine here is x86-64 code. The rungs, and what each would catch:
 *
 *   1  QueueUserAPC to self + SleepEx       a guest APC runs at all, with its
 *                                           parameter, and the wait reports
 *                                           WAIT_IO_COMPLETION afterwards
 *   2  two APCs, one alertable wait         the second is delivered by the
 *                                           first's NtContinue(..., TRUE)
 *   3  ReadFileEx completion routine        an I/O APC (kernel-queued, not
 *                                           QueueUserAPC) reaches the guest
 *   4  APC to a guest thread in SleepEx     cross-thread delivery into a thread
 *                                           parked in a forwarded syscall
 *   5  APC queued to a SUSPENDED new thread runs before its thread routine —
 *                                           i.e. it survives the window where
 *                                           the thread has no guest state yet
 *   6  64 threads created and joined        private commit stays flat: each
 *                                           guest thread's CPU-layer state and
 *                                           native stack are freed at exit
 *   7  16 threads killed by TerminateThread the same, for the other way out;
 *                                           and the exit code is the killer's
 *
 * Verdict contract: exit 0x600D on PASS; distinct 0xBAxx names the rung.
 */

#include <windows.h>

#define A64_FAIL_SELF_WAIT    0xBA11 /* SleepEx did not report WAIT_IO_COMPLETION */
#define A64_FAIL_SELF_RAN     0xBA12 /* the APC did not run, or saw a wrong parameter/thread */
#define A64_FAIL_TWO          0xBA21 /* two queued APCs did not both run, in order */
#define A64_FAIL_FILE_OPEN    0xBA31
#define A64_FAIL_FILE_ISSUE   0xBA32 /* ReadFileEx refused */
#define A64_FAIL_FILE_WAIT    0xBA33 /* no WAIT_IO_COMPLETION */
#define A64_FAIL_FILE_RESULT  0xBA34 /* completion routine never ran, or wrong error/count/data */
#define A64_FAIL_CROSS        0xBA41 /* APC to a waiting guest thread lost */
#define A64_FAIL_SUSPENDED    0xBA51 /* APC to a suspended new thread lost or ran late */
#define A64_FAIL_JOIN         0xBA61 /* a thread failed to create or exit */
#define A64_FAIL_JOIN_LEAK    0xBA62 /* private commit grew with thread count */
#define A64_FAIL_KILL         0xBA71 /* TerminateThread failed or wrong exit code */
#define A64_FAIL_KILL_LEAK    0xBA72
#define A64_PASS              0x600D

/* rung 6 — threads that EXIT cleanly must give everything back: their JIT
 * state, native stack (~1 MB each) and thread-local CRT heap. 64 of them
 * leaking anything shows up far below this; a real per-thread leak would blow
 * past 64 MB. */
#define A64_LEAK_LIMIT (24u << 20)

/* rung 7 — threads FORCE-TERMINATED by another thread are different, and so is
 * Windows here: TerminateThread cannot run the victim's own cleanup, so its
 * thread-local CRT/allocator heap (a few MB) is unreclaimable by the
 * terminator — only the kernel-owned stack, our native stack and the CPU
 * layer's JIT state come back (BTCpu64ThreadTerm). So the budget is per-thread
 * and generous: it catches the JIT/native-stack NOT being freed (that was
 * ~9 MB/thread before BTCpu64ThreadTerm wired up; ~4.5 MB with it) without
 * failing on the heap the platform structurally keeps. */
#define A64_KILL_LEAK_PER_THREAD (6u << 20)

static void Say(const char *msg)
{
    OutputDebugStringA(msg);
}

static void SayHex(const char *label, ULONG_PTR v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[112];
    int i = 0, n;
    while (label[i] && i < 80) { buf[i] = label[i]; i++; }
    buf[i++] = ' '; buf[i++] = '0'; buf[i++] = 'x';
    for (n = 60; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    Say(buf);
}

static void Fail(const char *what, DWORD code)
{
    Say(what);
    SayHex("[apc64] FAIL, exiting with", code);
    ExitProcess(code);
}

/* ---- APC routines ------------------------------------------------------ */

static volatile LONG ApcCount;
static volatile ULONG_PTR ApcSeen[4];
static volatile DWORD ApcTid[4];

static VOID CALLBACK RecordApc(ULONG_PTR Param)
{
    LONG n = InterlockedIncrement(&ApcCount) - 1;
    if (n < 4)
    {
        ApcSeen[n] = Param;
        ApcTid[n] = GetCurrentThreadId();
    }
}

static volatile DWORD IoError = 0xFFFFFFFF, IoBytes;
static VOID CALLBACK ReadDone(DWORD Error, DWORD Bytes, LPOVERLAPPED Ov)
{
    (void)Ov;
    IoError = Error;
    IoBytes = Bytes;
}

static void ResetApcs(void)
{
    int i;
    ApcCount = 0;
    for (i = 0; i < 4; i++) { ApcSeen[i] = 0; ApcTid[i] = 0; }
}

/* ---- thread bodies ----------------------------------------------------- */

static DWORD WINAPI AlertableSleeper(LPVOID Param)
{
    (void)Param;
    return SleepEx(30000, TRUE);
}

/* rung 5: the order things happened in on the new thread. */
static volatile LONG Order;
static volatile LONG ApcAt = -1, RoutineAt = -1;
static VOID CALLBACK EarlyApc(ULONG_PTR Param)
{
    (void)Param;
    ApcAt = InterlockedIncrement(&Order);
}
static DWORD WINAPI NoteStart(LPVOID Param)
{
    (void)Param;
    RoutineAt = InterlockedIncrement(&Order);
    return 0;
}

static DWORD WINAPI Quick(LPVOID Param)
{
    return (DWORD)(ULONG_PTR)Param ^ 0x5A5A;
}

static HANDLE Running;
static DWORD WINAPI Forever(LPVOID Param)
{
    (void)Param;
    SetEvent(Running);
    for (;;)
        Sleep(INFINITE);
}

/* Private committed bytes of this process, by walking its address space. */
static SIZE_T PrivateCommit(void)
{
    MEMORY_BASIC_INFORMATION Mbi;
    BYTE *p = 0;
    SIZE_T Total = 0;
    while (VirtualQuery(p, &Mbi, sizeof(Mbi)) == sizeof(Mbi))
    {
        if (Mbi.State == MEM_COMMIT && Mbi.Type == MEM_PRIVATE)
            Total += Mbi.RegionSize;
        p = (BYTE *)Mbi.BaseAddress + Mbi.RegionSize;
        if ((ULONG_PTR)p >= (ULONG_PTR)0x7FFFFFFF0000ull || Mbi.RegionSize == 0)
            break;
    }
    return Total;
}

static void JoinMany(int Count)
{
    int i;
    for (i = 0; i < Count; i++)
    {
        DWORD Code = 0;
        HANDLE h = CreateThread(NULL, 0, Quick, (LPVOID)(ULONG_PTR)i, 0, NULL);
        if (!h)
            Fail("[apc64] rung6 CreateThread failed\n", A64_FAIL_JOIN);
        if (WaitForSingleObject(h, 30000) != WAIT_OBJECT_0 || !GetExitCodeThread(h, &Code) ||
            Code != ((DWORD)i ^ 0x5A5A))
            Fail("[apc64] rung6 a thread did not exit with its code\n", A64_FAIL_JOIN);
        CloseHandle(h);
    }
}

static void KillMany(int Count)
{
    int i;
    for (i = 0; i < Count; i++)
    {
        DWORD Code = 0;
        HANDLE h = CreateThread(NULL, 0, Forever, NULL, 0, NULL);
        if (!h)
            Fail("[apc64] rung7 CreateThread failed\n", A64_FAIL_KILL);
        /* Into its wait: it then has guest state to free. */
        if (WaitForSingleObject(Running, 30000) != WAIT_OBJECT_0)
            Fail("[apc64] rung7 thread never started\n", A64_FAIL_KILL);
        if (!TerminateThread(h, 0x7E57 + i) || WaitForSingleObject(h, 30000) != WAIT_OBJECT_0 ||
            !GetExitCodeThread(h, &Code) || Code != (DWORD)(0x7E57 + i))
            Fail("[apc64] rung7 TerminateThread failed or wrong exit code\n", A64_FAIL_KILL);
        CloseHandle(h);
    }
}

void start(void)
{
    DWORD Me = GetCurrentThreadId();
    DWORD Wait;
    HANDLE h;
    SIZE_T Before, After;

    Say("[apc64] start\n");

    /* rung 1 */
    ResetApcs();
    if (!QueueUserAPC(RecordApc, GetCurrentThread(), 0x1111))
        Fail("[apc64] rung1 QueueUserAPC refused\n", A64_FAIL_SELF_RAN);
    Wait = SleepEx(5000, TRUE);
    if (Wait != WAIT_IO_COMPLETION)
    {
        SayHex("[apc64] rung1 SleepEx returned", Wait);
        Fail("[apc64] rung1 no WAIT_IO_COMPLETION\n", A64_FAIL_SELF_WAIT);
    }
    if (ApcCount != 1 || ApcSeen[0] != 0x1111 || ApcTid[0] != Me)
        Fail("[apc64] rung1 APC did not run as queued\n", A64_FAIL_SELF_RAN);
    Say("[apc64] rung1 APC to self OK\n");

    /* rung 2 */
    ResetApcs();
    QueueUserAPC(RecordApc, GetCurrentThread(), 0x2222);
    QueueUserAPC(RecordApc, GetCurrentThread(), 0x3333);
    Wait = SleepEx(5000, TRUE);
    if (Wait != WAIT_IO_COMPLETION || ApcCount != 2 || ApcSeen[0] != 0x2222 || ApcSeen[1] != 0x3333)
    {
        SayHex("[apc64] rung2 APCs run", (ULONG_PTR)ApcCount);
        Fail("[apc64] rung2 two APCs not both delivered in order\n", A64_FAIL_TWO);
    }
    Say("[apc64] rung2 two APCs in one wait OK\n");

    /* rung 3 */
    {
        char Path[MAX_PATH];
        static char Buf[512];
        OVERLAPPED Ov;
        HANDLE f;
        Ov.Internal = 0; Ov.InternalHigh = 0; Ov.Offset = 0; Ov.OffsetHigh = 0; Ov.hEvent = NULL;
        if (!GetModuleFileNameA(NULL, Path, MAX_PATH))
            Fail("[apc64] rung3 GetModuleFileNameA failed\n", A64_FAIL_FILE_OPEN);
        f = CreateFileA(Path, GENERIC_READ, FILE_SHARE_READ, NULL, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, NULL);
        if (f == INVALID_HANDLE_VALUE)
        {
            SayHex("[apc64] rung3 CreateFileA, GetLastError", GetLastError());
            Fail("[apc64] rung3 cannot open own image\n", A64_FAIL_FILE_OPEN);
        }
        if (!ReadFileEx(f, Buf, sizeof(Buf), &Ov, ReadDone))
        {
            SayHex("[apc64] rung3 ReadFileEx, GetLastError", GetLastError());
            Fail("[apc64] rung3 ReadFileEx refused\n", A64_FAIL_FILE_ISSUE);
        }
        Wait = SleepEx(10000, TRUE);
        if (Wait != WAIT_IO_COMPLETION)
        {
            SayHex("[apc64] rung3 SleepEx returned", Wait);
            Fail("[apc64] rung3 no WAIT_IO_COMPLETION\n", A64_FAIL_FILE_WAIT);
        }
        if (IoError != 0 || IoBytes != sizeof(Buf) || Buf[0] != 'M' || Buf[1] != 'Z')
        {
            SayHex("[apc64] rung3 completion error", IoError);
            SayHex("[apc64] rung3 completion bytes", IoBytes);
            Fail("[apc64] rung3 completion routine wrong or missing\n", A64_FAIL_FILE_RESULT);
        }
        CloseHandle(f);
        Say("[apc64] rung3 ReadFileEx completion routine OK\n");
    }

    /* rung 4 */
    {
        DWORD Code = 0, Tid = 0;
        ResetApcs();
        h = CreateThread(NULL, 0, AlertableSleeper, NULL, 0, &Tid);
        if (!h)
            Fail("[apc64] rung4 CreateThread failed\n", A64_FAIL_CROSS);
        Sleep(200); /* into its SleepEx */
        QueueUserAPC(RecordApc, h, 0x4444);
        if (WaitForSingleObject(h, 10000) != WAIT_OBJECT_0 || !GetExitCodeThread(h, &Code) ||
            Code != WAIT_IO_COMPLETION || ApcCount != 1 || ApcSeen[0] != 0x4444 || ApcTid[0] != Tid)
        {
            SayHex("[apc64] rung4 sleeper exit", Code);
            Fail("[apc64] rung4 APC to a waiting thread lost\n", A64_FAIL_CROSS);
        }
        CloseHandle(h);
        Say("[apc64] rung4 APC into another thread's wait OK\n");
    }

    /* rung 5 */
    h = CreateThread(NULL, 0, NoteStart, NULL, CREATE_SUSPENDED, NULL);
    if (!h || !QueueUserAPC(EarlyApc, h, 0) || ResumeThread(h) == (DWORD)-1 ||
        WaitForSingleObject(h, 10000) != WAIT_OBJECT_0)
        Fail("[apc64] rung5 suspended thread setup failed\n", A64_FAIL_SUSPENDED);
    CloseHandle(h);
    if (ApcAt != 1 || RoutineAt != 2)
    {
        SayHex("[apc64] rung5 APC ran at", (ULONG_PTR)ApcAt);
        SayHex("[apc64] rung5 routine ran at", (ULONG_PTR)RoutineAt);
        Fail("[apc64] rung5 APC to a suspended thread lost or late\n", A64_FAIL_SUSPENDED);
    }
    Say("[apc64] rung5 APC before a new thread's routine OK\n");

    /* rung 6 — warm up first, so one-time growth is not counted */
    JoinMany(4);
    Before = PrivateCommit();
    JoinMany(64);
    After = PrivateCommit();
    SayHex("[apc64] rung6 private commit before", Before);
    SayHex("[apc64] rung6 private commit after 64 threads", After);
    if (After > Before && After - Before > A64_LEAK_LIMIT)
        Fail("[apc64] rung6 per-thread state leaked\n", A64_FAIL_JOIN_LEAK);
    Say("[apc64] rung6 64 threads joined, commit flat OK\n");

    /* rung 7 */
    Running = CreateEventA(NULL, FALSE, FALSE, NULL);
    if (!Running)
        Fail("[apc64] rung7 CreateEventA failed\n", A64_FAIL_KILL);
    KillMany(2);
    Before = PrivateCommit();
    KillMany(16);
    After = PrivateCommit();
    SayHex("[apc64] rung7 private commit before", Before);
    SayHex("[apc64] rung7 private commit after 16 kills", After);
    if (After > Before && After - Before > 16u * A64_KILL_LEAK_PER_THREAD)
        Fail("[apc64] rung7 terminated threads leaked JIT/stack\n", A64_FAIL_KILL_LEAK);
    Say("[apc64] rung7 16 threads terminated, reclaimable state freed OK\n");

    Say("[apc64] PASS\n");
    ExitProcess(A64_PASS);
}
