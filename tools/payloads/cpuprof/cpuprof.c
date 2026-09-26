/* cpuprof — what is burning CPU in the minutes after boot?
 *
 * Every INTERVAL seconds for SAMPLES rounds, reads SystemProcessInformation (process and thread
 * times) and prints the processes and threads whose kernel+user time grew the most since the
 * previous round, with each thread's start address and state. Idle is excluded.
 * Lines: "[CPUPROF] t=<s> proc <name> pid=<n> +<ms>ms (k=<ms> u=<ms>)"
 *        "[CPUPROF] t=<s>   thr tid=<n> +<ms>ms start=<addr> state=<n> wait=<n> prio=<n>"
 * Verdict: "[CPUPROF] done". */
#include <windows.h>
#include <winternl.h>
#include <stdio.h>
#include <stdlib.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[CPUPROF] " __VA_ARGS__)
#define INTERVAL 10
#define SAMPLES 24
#define MAXT 4096

typedef struct {
    LARGE_INTEGER KernelTime, UserTime, CreateTime;
    ULONG WaitTime;
    PVOID StartAddress;
    CLIENT_ID ClientId;
    LONG Priority, BasePriority;
    ULONG ContextSwitches, ThreadState, WaitReason;
} STI;  /* SYSTEM_THREAD_INFORMATION */
typedef struct {
    ULONG NextEntryOffset, NumberOfThreads;
    LARGE_INTEGER Reserved[3];
    LARGE_INTEGER CreateTime, UserTime, KernelTime;
    UNICODE_STRING ImageName;
    LONG BasePriority;
    HANDLE UniqueProcessId, InheritedFromUniqueProcessId;
    ULONG HandleCount, SessionId;
    ULONG_PTR PageDirectoryBase;
    SIZE_T PeakVirtualSize, VirtualSize;
    ULONG PageFaultCount;
    SIZE_T PeakWorkingSetSize, WorkingSetSize, QuotaPeakPagedPoolUsage, QuotaPagedPoolUsage,
           QuotaPeakNonPagedPoolUsage, QuotaNonPagedPoolUsage, PagefileUsage, PeakPagefileUsage,
           PrivatePageCount;
    LARGE_INTEGER ReadOperationCount, WriteOperationCount, OtherOperationCount,
                  ReadTransferCount, WriteTransferCount, OtherTransferCount;
    STI Threads[1];
} SPI;  /* SYSTEM_PROCESS_INFORMATION (NT5 layout) */

typedef struct { ULONG_PTR id; ULONGLONG t; int pidx; } ent;
/* SYSTEM_PROCESSOR_PERFORMANCE_INFORMATION: where the time goes that no process is charged for. */
typedef struct {
    LARGE_INTEGER IdleTime, KernelTime, UserTime, DpcTime, InterruptTime;
    ULONG InterruptCount;
} SPPI;
static SPPI prevcpu;
static ent prevp[512], prevt[MAXT];
static int nprevp, nprevt;

static ULONGLONG lookup(ent *a, int n, ULONG_PTR id)
{
    for (int i = 0; i < n; i++) if (a[i].id == id) return a[i].t;
    return (ULONGLONG)-1;
}

int main(void)
{
    static BYTE buf[1 << 20];
    NTSTATUS (NTAPI *qsi)(ULONG, PVOID, ULONG, PULONG) =
        (void *)GetProcAddress(GetModuleHandleA("ntdll.dll"), "NtQuerySystemInformation");
    SAY("start interval=%ds samples=%d\n", INTERVAL, SAMPLES);
    for (int s = 0; s <= SAMPLES; s++) {
        ULONG len;
        if (qsi(5 /* SystemProcessInformation */, buf, sizeof buf, &len) < 0) {
            SAY("query failed\n");
            break;
        }
        ent curp[512], *curt = malloc(sizeof(ent) * MAXT);
        int ncp = 0, nct = 0;
        struct { ULONGLONG d; int i; } topp[512];
        int ntopp = 0;
        for (SPI *p = (SPI *)buf;; p = (SPI *)((BYTE *)p + p->NextEntryOffset)) {
            ULONGLONG t = p->KernelTime.QuadPart + p->UserTime.QuadPart;
            ULONG_PTR pid = (ULONG_PTR)p->UniqueProcessId;
            if (ncp < 512) { curp[ncp].id = pid; curp[ncp].t = t; ncp++; }
            ULONGLONG was = lookup(prevp, nprevp, pid);
            if (s && pid && was != (ULONGLONG)-1 && t > was) {
                char name[64];
                int n = p->ImageName.Buffer ? WideCharToMultiByte(CP_ACP, 0, p->ImageName.Buffer,
                            p->ImageName.Length / 2, name, 63, NULL, NULL) : 0;
                name[n] = 0;
                if ((t - was) / 10000 >= 200)  /* >= 200ms in the interval */
                    SAY("t=%d proc %s pid=%lu +%llums (k=%llu u=%llu)\n", s * INTERVAL, n ? name : "?",
                        (unsigned long)pid, (t - was) / 10000, p->KernelTime.QuadPart / 10000,
                        p->UserTime.QuadPart / 10000);
            }
            for (ULONG k = 0; k < p->NumberOfThreads && nct < MAXT; k++) {
                STI *th = &p->Threads[k];
                ULONGLONG tt = th->KernelTime.QuadPart + th->UserTime.QuadPart;
                ULONG_PTR tid = (ULONG_PTR)th->ClientId.UniqueThread;
                curt[nct].id = tid; curt[nct].t = tt; nct++;
                ULONGLONG tw = lookup(prevt, nprevt, tid);
                if (s && pid && tw != (ULONGLONG)-1 && tt > tw && (tt - tw) / 10000 >= 200)
                    SAY("t=%d   thr tid=%lu pid=%lu +%llums start=%p state=%lu wait=%lu prio=%ld\n",
                        s * INTERVAL, (unsigned long)tid, (unsigned long)pid, (tt - tw) / 10000,
                        th->StartAddress, th->ThreadState, th->WaitReason, th->Priority);
            }
            if (!p->NextEntryOffset) break;
        }
        SPPI cpu;
        if (qsi(8 /* SystemProcessorPerformanceInformation */, &cpu, sizeof cpu, &len) >= 0) {
            if (s) {
                /* KernelTime includes IdleTime and DpcTime/InterruptTime on NT. */
                ULONGLONG idle = (cpu.IdleTime.QuadPart - prevcpu.IdleTime.QuadPart) / 10000;
                ULONGLONG kern = (cpu.KernelTime.QuadPart - prevcpu.KernelTime.QuadPart) / 10000;
                ULONGLONG user = (cpu.UserTime.QuadPart - prevcpu.UserTime.QuadPart) / 10000;
                ULONGLONG dpc = (cpu.DpcTime.QuadPart - prevcpu.DpcTime.QuadPart) / 10000;
                ULONGLONG intr = (cpu.InterruptTime.QuadPart - prevcpu.InterruptTime.QuadPart) / 10000;
                SAY("t=%d cpu0 idle=%llums kernel=%llums user=%llums dpc=%llums intr=%llums ints=%lu\n",
                    s * INTERVAL, idle, kern, user, dpc, intr, cpu.InterruptCount - prevcpu.InterruptCount);
            }
            prevcpu = cpu;
        }
        memcpy(prevp, curp, sizeof(ent) * ncp); nprevp = ncp;
        memcpy(prevt, curt, sizeof(ent) * nct); nprevt = nct;
        free(curt);
        if (s < SAMPLES) Sleep(INTERVAL * 1000);
    }
    SAY("done\n");
    return 0;
}
