/* sysbench — what does one system call cost, native vs an x86-64 guest under FEX?
 *
 * Times a ladder of operations, each repeated until it has run for at least MINMS of wall
 * time, and prints the cost per operation. The arm64 build runs it natively, then launches the
 * amd64 build (an x86-64 guest under wowx64/FEX) with "-child" and waits for it, so both rows
 * come from the same machine, minutes apart. The ladder separates what a guest pays:
 *   loop        guest code alone, no syscall (FEX's JIT speed against native)
 *   nt_cpu      the cheapest syscall there is (NtGetCurrentProcessorNumber, no arguments)
 *   nt_qip      a syscall with a stack argument (NtQueryInformationProcess, 5 arguments)
 *   nt_vm       NtAllocate + NtFreeVirtualMemory (wowx64 reports both to the CPU layer)
 *   nt_protect  two NtProtectVirtualMemory calls (reported too; can invalidate code)
 *   gdi_pixel   GetPixel on a memory DC (sysdm builds its logo two of these per pixel)
 *   user_peek   PeekMessage on an empty queue
 *   user_wnd    CreateWindowEx + DestroyWindow (each runs window-procedure callbacks)
 *   memset4k    ntdll's memset of 4 KB, the CRT core the kernel shares (no syscall)
 *   memcpy4k    ntdll's memcpy of 4 KB
 * Before timing anything, ntdll's memset/memcpy/memmove are checked against a byte loop over
 * every destination offset x source offset x length up to 16 x 16 x 300, overlapping both
 * ways: they are asm (cnt-rust memops.rs), and on arm64 only the guest can run them.
 * Lines: "[SYSBENCH] machine=<arm64|amd64> <name> n=<ops> ns_per_op=<ns>"
 *        "[SYSBENCH] machine=<arch> memops OK cases=<n>" (or "memops FAIL ...")
 * Verdict: "[SYSBENCH] done VERDICT=PASS" (the launcher, after the child exits and only when
 * both memops checks passed). */
#include <windows.h>
#include <winternl.h>
#include <stdio.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
NTSTATUS NTAPI NtAllocateVirtualMemory(HANDLE, PVOID *, ULONG_PTR, PSIZE_T, ULONG, ULONG);
NTSTATUS NTAPI NtFreeVirtualMemory(HANDLE, PVOID *, PSIZE_T, ULONG);
NTSTATUS NTAPI NtProtectVirtualMemory(HANDLE, PVOID *, PSIZE_T, ULONG, PULONG);
ULONG NTAPI NtGetCurrentProcessorNumber(void);

#define SAY(...) DbgPrint("[SYSBENCH] " __VA_ARGS__)
#define MINMS 1500

#if defined(__aarch64__)
static const char *Machine = "arm64";
#elif defined(__x86_64__)
static const char *Machine = "amd64";
#else
static const char *Machine = "i386";
#endif

typedef void *(__cdecl *MEMFN)(void *, const void *, size_t);
typedef void *(__cdecl *SETFN)(void *, int, size_t);
static MEMFN NtMemcpy, NtMemmove;
static SETFN NtMemset;
static unsigned char Buf4kA[4096], Buf4kB[4096];

static volatile ULONG Sink;
static HDC MemDc;
static HWND Owner;
static void *Page;

static __attribute__((noinline)) void op_loop(void)
{
    ULONG x = Sink;
    for (int i = 0; i < 100; i++) x = x * 1103515245u + 12345u;
    Sink = x;
}
static void op_nt_cpu(void) { Sink += NtGetCurrentProcessorNumber(); }
static void op_nt_qip(void)
{
    PROCESS_BASIC_INFORMATION Pbi;
    ULONG Len;
    Sink += NtQueryInformationProcess(GetCurrentProcess(), ProcessBasicInformation, &Pbi, sizeof(Pbi), &Len);
}
static void op_nt_vm(void)
{
    PVOID Base = NULL;
    SIZE_T Size = 0x10000;
    Sink += NtAllocateVirtualMemory(GetCurrentProcess(), &Base, 0, &Size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    Size = 0;
    Sink += NtFreeVirtualMemory(GetCurrentProcess(), &Base, &Size, MEM_RELEASE);
}
static void op_nt_protect(void)
{
    PVOID Base = Page;
    SIZE_T Size = 0x1000;
    ULONG Old;
    Sink += NtProtectVirtualMemory(GetCurrentProcess(), &Base, &Size, PAGE_READONLY, &Old);
    Base = Page;
    Size = 0x1000;
    Sink += NtProtectVirtualMemory(GetCurrentProcess(), &Base, &Size, PAGE_READWRITE, &Old);
}
static void op_gdi_pixel(void) { Sink += GetPixel(MemDc, Sink & 31, 7); }
static void op_user_peek(void)
{
    MSG Msg;
    Sink += PeekMessageW(&Msg, NULL, 0, 0, PM_REMOVE);
}
static void op_user_wnd(void)
{
    HWND W = CreateWindowExW(0, L"SysbenchWnd", L"", WS_CHILD, 0, 0, 10, 10, Owner, NULL, NULL, NULL);
    Sink += (ULONG)(ULONG_PTR)W;
    DestroyWindow(W);
}

static void op_memset4k(void) { NtMemset(Buf4kA, (int)Sink, sizeof(Buf4kA)); Sink += Buf4kA[7]; }
static void op_memcpy4k(void) { NtMemcpy(Buf4kB, Buf4kA, sizeof(Buf4kA)); Sink += Buf4kB[9]; }

/* The reference side never calls (or lets clang turn a loop into) the CRT it is grading:
 * every byte goes through a volatile pointer. */
typedef volatile unsigned char VB;
static void ref_move(VB *d, VB *s, size_t n)
{
    static VB Tmp[512];
    for (size_t i = 0; i < n; i++) Tmp[i] = s[i];
    for (size_t i = 0; i < n; i++) d[i] = Tmp[i];
}
static void ref_copy(VB *d, VB *s, size_t n) { for (size_t i = 0; i < n; i++) d[i] = s[i]; }
static int ref_differ(VB *a, VB *b, size_t n)
{
    for (size_t i = 0; i < n; i++)
        if (a[i] != b[i]) return 1;
    return 0;
}

/* 0 on success, else prints the first mismatch. */
static int memops_check(void)
{
    static unsigned char Base[1024], A[1024], B[1024];
    HMODULE Nt = GetModuleHandleA("ntdll.dll");
    NtMemcpy = (MEMFN)GetProcAddress(Nt, "memcpy");
    NtMemmove = (MEMFN)GetProcAddress(Nt, "memmove");
    NtMemset = (SETFN)GetProcAddress(Nt, "memset");
    if (!NtMemcpy || !NtMemmove || !NtMemset) {
        SAY("machine=%s memops FAIL ntdll lacks memcpy/memmove/memset\n", Machine);
        return 1;
    }
    ULONG Seed = 0x12345678, Cases = 0;
    for (int i = 0; i < 1024; i++) {
        Seed ^= Seed << 13; Seed ^= Seed >> 17; Seed ^= Seed << 5;
        Base[i] = (unsigned char)Seed;
    }
    for (size_t n = 0; n < 300; n++) {
        for (size_t da = 0; da < 16; da++) {
            for (size_t sa = 0; sa < 16; sa++) {
                /* disjoint memcpy, then overlapping memmove both ways */
                const size_t Pairs[3][2] = {{512 + da, sa}, {64 + da, 64 + sa + 3}, {64 + da + 3, 64 + sa}};
                for (int k = 0; k < 3; k++) {
                    size_t d = Pairs[k][0], s = Pairs[k][1];
                    ref_copy(A, Base, sizeof(A));
                    ref_copy(B, Base, sizeof(B));
                    (k == 0 ? NtMemcpy : NtMemmove)(A + d, A + s, n);
                    ref_move(B + d, B + s, n);
                    if (ref_differ(A, B, sizeof(A))) {
                        SAY("machine=%s memops FAIL %s n=%u dst=%u src=%u\n", Machine, k ? "memmove" : "memcpy",
                            (unsigned)n, (unsigned)d, (unsigned)s);
                        return 1;
                    }
                    Cases++;
                }
            }
            ref_copy(A, Base, sizeof(A));
            ref_copy(B, Base, sizeof(B));
            int v = (int)(Seed ^= Seed << 13) | 0x100; /* memset takes the low byte */
            NtMemset(A + da, v, n);
            for (size_t i = 0; i < n; i++) ((VB *)B)[da + i] = (unsigned char)v;
            if (ref_differ(A, B, sizeof(A))) {
                SAY("machine=%s memops FAIL memset n=%u dst=%u\n", Machine, (unsigned)n, (unsigned)da);
                return 1;
            }
            Cases++;
        }
    }
    SAY("machine=%s memops OK cases=%lu\n", Machine, Cases);
    return 0;
}

static void run(const char *Name, void (*Op)(void))
{
    LARGE_INTEGER F, T0, T1;
    QueryPerformanceFrequency(&F);
    Op(); /* first call: page faults, JIT compilation, lazy binds */
    ULONGLONG N = 0, Batch = 1;
    QueryPerformanceCounter(&T0);
    for (;;) {
        for (ULONGLONG i = 0; i < Batch; i++) Op();
        N += Batch;
        QueryPerformanceCounter(&T1);
        if ((T1.QuadPart - T0.QuadPart) * 1000 >= MINMS * F.QuadPart) break;
        if (Batch < 4096) Batch *= 2;
    }
    ULONGLONG Ns = (ULONGLONG)((double)(T1.QuadPart - T0.QuadPart) * 1e9 / (double)F.QuadPart / (double)N);
    SAY("machine=%s %s n=%llu ns_per_op=%llu\n", Machine, Name, N, Ns);
}

static int bench(void)
{
    if (memops_check())
        return 1;

    WNDCLASSW Wc = {0};
    Wc.lpfnWndProc = DefWindowProcW;
    Wc.lpszClassName = L"SysbenchWnd";
    RegisterClassW(&Wc);
    Owner = CreateWindowExW(0, L"SysbenchWnd", L"sysbench", WS_OVERLAPPED, 0, 0, 50, 50, NULL, NULL, NULL, NULL);
    HDC Screen = GetDC(NULL);
    MemDc = CreateCompatibleDC(Screen);
    SelectObject(MemDc, CreateCompatibleBitmap(Screen, 32, 32));
    ReleaseDC(NULL, Screen);
    Page = VirtualAlloc(NULL, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);

    run("loop", op_loop);
    run("nt_cpu", op_nt_cpu);
    run("nt_qip", op_nt_qip);
    run("nt_vm", op_nt_vm);
    run("nt_protect", op_nt_protect);
    run("gdi_pixel", op_gdi_pixel);
    run("user_peek", op_user_peek);
    run("user_wnd", op_user_wnd);
    run("memset4k", op_memset4k);
    run("memcpy4k", op_memcpy4k);
    return 0;
}

int main(int argc, char **argv)
{
    if (argc > 1 && !strcmp(argv[1], "-child"))
        return bench();
    /* Let the boot settle: the shell and its startup items are still loading. */
    Sleep(20000);
    SAY("start machine=%s\n", Machine);
    int Failed = bench();

    char Path[MAX_PATH], Cmd[MAX_PATH + 16];
    GetWindowsDirectoryA(Path, MAX_PATH);
    strcat(Path, "\\sysbench64.exe");
    snprintf(Cmd, sizeof(Cmd), "\"%s\" -child", Path);
    STARTUPINFOA Si = {sizeof(Si)};
    PROCESS_INFORMATION Pi;
    if (!CreateProcessA(Path, Cmd, NULL, NULL, FALSE, 0, NULL, NULL, &Si, &Pi)) {
        SAY("done VERDICT=FAIL cannot launch %s (error %lu)\n", Path, GetLastError());
        return 1;
    }
    WaitForSingleObject(Pi.hProcess, INFINITE);
    DWORD Code = 1;
    GetExitCodeProcess(Pi.hProcess, &Code);
    if (Failed)
        SAY("done VERDICT=FAIL native memops check\n");
    else if (Code)
        SAY("done VERDICT=FAIL child exited 0x%lx\n", Code);
    else
        SAY("done VERDICT=PASS\n");
    return 0;
}
