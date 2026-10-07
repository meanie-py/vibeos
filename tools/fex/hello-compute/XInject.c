/*
 * xinject.exe — spawn64's NATIVE helper: the other end of rung 13. Built for
 * the host's own machine (aarch64 on arm64, x86-64 on amd64) and staged as
 * system32\xinject.exe. No CRT, kernel32 + ntdll only.
 *
 *   xinject.exe <parent pid, hex> <routine, hex>
 *
 * On arm64 it is two proofs at once. spawn64, an x86-64 guest, STARTS it -- a
 * native arm64 child of an emulated process, whose initial thread wowx64 must
 * hand the native kernel32's process start. And it then creates threads BACK
 * in spawn64, running <routine>, an x86-64 function there: once through
 * CreateRemoteThread (kernel32's Win32 shape, starting in the native
 * kernel32's BaseThreadStartup, which an x86-64 guest process does not have)
 * and once through RtlCreateUserThread (the bare routine). spawn64's routine
 * ends with ExitThread(parameter + 1), so each thread's exit code says the
 * guest ran it with the parameter it was given. On amd64 the same calls are
 * plain native-to-native, the oracle that the rung itself is right.
 *
 * Exit code: 0x600D when both threads ran, else a distinct 0xBAFn per step.
 */
#include <windows.h>

typedef LONG XNTSTATUS;
typedef struct { HANDLE UniqueProcess, UniqueThread; } XCLIENT_ID;
__declspec(dllimport) XNTSTATUS NTAPI RtlCreateUserThread(HANDLE Process, PVOID Sd, BOOLEAN Suspended,
    ULONG StackZeroBits, SIZE_T StackReserve, SIZE_T StackCommit, PVOID Start, PVOID Param,
    PHANDLE Thread, XCLIENT_ID *Cid);

#define XI_PASS            0x600D
#define XI_FAIL_ARGS       0xBAF0 /* command line did not carry pid + routine */
#define XI_FAIL_OPEN       0xBAF1 /* OpenProcess(parent) */
#define XI_FAIL_WIN32      0xBAF2 /* CreateRemoteThread failed */
#define XI_FAIL_WIN32_RUN  0xBAF3 /* ... its thread did not end with parameter + 1 */
#define XI_FAIL_RTL        0xBAF4 /* RtlCreateUserThread failed */
#define XI_FAIL_RTL_RUN    0xBAF5 /* ... its thread did not end with parameter + 1 */

#define WIN32_PARAM 0x1000
#define RTL_PARAM   0x2000
#define WAIT_MS     30000

static void Say(const char *msg)
{
    OutputDebugStringA(msg);
}

static void SayHex(const char *label, DWORD v)
{
    static const char Digits[] = "0123456789abcdef";
    char buf[96];
    int i = 0, n;
    while (label[i] && i < 72) { buf[i] = label[i]; i++; }
    buf[i++] = ' '; buf[i++] = '0'; buf[i++] = 'x';
    for (n = 28; n >= 0; n -= 4)
        buf[i++] = Digits[(v >> n) & 0xf];
    buf[i++] = '\n';
    buf[i] = 0;
    Say(buf);
}

/* The next hex number in *p, skipping leading blanks. FALSE if there is none. */
static BOOL NextHex(const WCHAR **p, ULONG_PTR *Out)
{
    const WCHAR *s = *p;
    ULONG_PTR v = 0;
    int digits = 0;
    while (*s == L' ' || *s == L'\t')
        s++;
    for (;; s++, digits++) {
        WCHAR c = *s;
        if (c >= L'0' && c <= L'9') v = v * 16 + (c - L'0');
        else if (c >= L'a' && c <= L'f') v = v * 16 + (c - L'a' + 10);
        else if (c >= L'A' && c <= L'F') v = v * 16 + (c - L'A' + 10);
        else break;
    }
    *p = s;
    *Out = v;
    return digits > 0;
}

/* The thread's exit code, or 0xFFFFFFFF if it did not end in time. */
static DWORD Finish(HANDLE Thread)
{
    DWORD Code = 0xFFFFFFFF;
    if (WaitForSingleObject(Thread, WAIT_MS) != WAIT_OBJECT_0 || !GetExitCodeThread(Thread, &Code))
        Code = 0xFFFFFFFF;
    CloseHandle(Thread);
    return Code;
}

static DWORD Run(void)
{
    const WCHAR *p = GetCommandLineW();
    ULONG_PTR Pid, Routine;
    HANDLE Process, Thread = NULL;
    XCLIENT_ID Cid;
    DWORD Code;
    XNTSTATUS Status;

    /* Past argv[0], quoted or not. */
    if (*p == L'"') {
        for (p++; *p && *p != L'"'; p++)
            ;
        if (*p)
            p++;
    } else {
        while (*p && *p != L' ' && *p != L'\t')
            p++;
    }
    if (!NextHex(&p, &Pid) || !NextHex(&p, &Routine) || !Routine)
        return XI_FAIL_ARGS;

    Process = OpenProcess(PROCESS_ALL_ACCESS, FALSE, (DWORD)Pid);
    if (!Process) {
        SayHex("[xinject] OpenProcess(parent) FAILED, GetLastError", GetLastError());
        return XI_FAIL_OPEN;
    }

    Thread = CreateRemoteThread(Process, NULL, 0, (LPTHREAD_START_ROUTINE)Routine,
                                (LPVOID)(ULONG_PTR)WIN32_PARAM, 0, NULL);
    if (!Thread) {
        SayHex("[xinject] CreateRemoteThread FAILED, GetLastError", GetLastError());
        CloseHandle(Process);
        return XI_FAIL_WIN32;
    }
    Code = Finish(Thread);
    if (Code != WIN32_PARAM + 1) {
        SayHex("[xinject] CreateRemoteThread's thread ended with", Code);
        CloseHandle(Process);
        return XI_FAIL_WIN32_RUN;
    }
    Say("[xinject] CreateRemoteThread into the parent OK\n");

    Status = RtlCreateUserThread(Process, NULL, FALSE, 0, 0, 0, (PVOID)Routine,
                                 (PVOID)(ULONG_PTR)RTL_PARAM, &Thread, &Cid);
    if (Status < 0) {
        SayHex("[xinject] RtlCreateUserThread FAILED, status", (DWORD)Status);
        CloseHandle(Process);
        return XI_FAIL_RTL;
    }
    Code = Finish(Thread);
    CloseHandle(Process);
    if (Code != RTL_PARAM + 1) {
        SayHex("[xinject] RtlCreateUserThread's thread ended with", Code);
        return XI_FAIL_RTL_RUN;
    }
    Say("[xinject] RtlCreateUserThread into the parent OK\n");
    return XI_PASS;
}

int start(void)
{
    DWORD Verdict = Run();
    SayHex("[xinject] verdict", Verdict);
    ExitProcess(Verdict);
    return (int)Verdict;
}
