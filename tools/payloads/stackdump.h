/* stackdump.h — print where a thread is: its PC and every stack slot that points into a loaded
 * module, as absolute addresses (feed them to `vibeos-cli sym --arch <a>`) plus module+offset.
 * Scanning the stack for return-address-shaped values over-reports (stale frames too), but the
 * innermost real frames are always among them. Shared by the payloads that detect a hang.
 * Needs DbgPrint and a SAY(...) macro from the includer; link -lkernel32. */
#ifndef PAYLOAD_STACKDUMP_H
#define PAYLOAD_STACKDUMP_H
#include <windows.h>
#include <tlhelp32.h>

static void stackdump_ascii(const WCHAR *w, char *o, size_t n)
{
    size_t i = 0;
    for (; w[i] && i + 1 < n; i++)
        o[i] = w[i] < 0x80 ? (char)w[i] : '?';
    o[i] = 0;
}

/* Which call a probe is in, for the deadline report: a probe that never returns names its stage. */
static volatile LONG g_sd_stage;
static const char *const g_sd_stages[] = {
    "snapshot", "open", "SuspendThread", "GetThreadContext", "ResumeThread", "ReadProcessMemory",
};
#define SD_STAGE(n) InterlockedExchange(&g_sd_stage, (n))

/* One thread's scheduler view, from NtQuerySystemInformation(SystemProcessInformation)
 * (layouts as ndk/extypes.h; resolved at run time so includers keep their link line). Sampled
 * before a probe and again at its deadline, the deltas say what a stuck target is doing: kernel
 * time growing by the whole wait means it is spinning in the kernel; no context switches means
 * it is blocked, and ThreadState/WaitReason (KTHREAD_STATE, KWAIT_REASON) say on what. */
struct sd_thread_view {
    LONGLONG ktime, utime;
    ULONG switches, state, wait;
    PVOID start;
    BOOL found;
};
struct sd_thread_info {
    LARGE_INTEGER KernelTime, UserTime, CreateTime;
    ULONG WaitTime;
    PVOID StartAddress;
    HANDLE Pid, Tid;
    LONG Priority, BasePriority;
    ULONG ContextSwitches, ThreadState, WaitReason, Pad;
};
static void sd_sample_thread(DWORD pid, DWORD tid, struct sd_thread_view *v)
{
    typedef LONG (NTAPI *qsi_fn)(ULONG, PVOID, ULONG, PULONG);
    static qsi_fn qsi;
    v->found = FALSE;
    if (!qsi)
        qsi = (qsi_fn)GetProcAddress(GetModuleHandleA("ntdll.dll"), "NtQuerySystemInformation");
    if (!qsi)
        return;
    ULONG size = 256 * 1024, need = 0;
    BYTE *buf = NULL;
    for (int tries = 0; tries < 4; tries++) {
        buf = HeapAlloc(GetProcessHeap(), 0, size);
        if (!buf)
            return;
        LONG st = qsi(5 /* SystemProcessInformation */, buf, size, &need);
        if (st >= 0)
            break;
        HeapFree(GetProcessHeap(), 0, buf);
        buf = NULL;
        if (st != (LONG)0xC0000004 /* STATUS_INFO_LENGTH_MISMATCH */)
            return;
        size = need > size ? need + 64 * 1024 : size * 2;
    }
    if (!buf)
        return;
    /* SYSTEM_PROCESS_INFORMATION: NextEntryOffset, NumberOfThreads, ... UniqueProcessId at
     * 0x50 (x64) / 0x44 (x86); the thread array follows the 0x100 (x64) / 0xB8 (x86) header. */
    const size_t pid_off = sizeof(void *) == 8 ? 0x50 : 0x44;
    const size_t hdr = sizeof(void *) == 8 ? 0x100 : 0xB8;
    for (BYTE *p = buf;;) {
        ULONG next = *(ULONG *)p, nthreads = *(ULONG *)(p + 4);
        if ((DWORD)(ULONG_PTR)*(HANDLE *)(p + pid_off) == pid) {
            struct sd_thread_info *t = (struct sd_thread_info *)(p + hdr);
            for (ULONG i = 0; i < nthreads; i++) {
                if ((DWORD)(ULONG_PTR)t[i].Tid != tid)
                    continue;
                v->ktime = t[i].KernelTime.QuadPart;
                v->utime = t[i].UserTime.QuadPart;
                v->switches = t[i].ContextSwitches;
                v->state = t[i].ThreadState;
                v->wait = t[i].WaitReason;
                v->start = t[i].StartAddress;
                v->found = TRUE;
            }
            break;
        }
        if (!next)
            break;
        p += next;
    }
    HeapFree(GetProcessHeap(), 0, buf);
}

/* Every process's CPU time so far against its age, most CPU first: says which processes
 * kept the (by default single) vCPU busy during a run. A process that should idle (osk,
 * an x64 test window) showing CPU close to its age is spinning. Same SystemProcessInformation
 * walk as above: UserTime 0x28, KernelTime 0x30, CreateTime 0x20, ImageName UNICODE_STRING
 * at 0x38 on both, UniqueProcessId 0x50 (x64) / 0x44 (x86). Returns the busiest pid that
 * is not idle, System or the caller, so the caller can sample its threads (0 if none). */
static __inline DWORD sd_report_process_cpu(const char *tag, int top)
{
    typedef LONG (NTAPI *qsi_fn)(ULONG, PVOID, ULONG, PULONG);
    qsi_fn qsi = (qsi_fn)GetProcAddress(GetModuleHandleA("ntdll.dll"), "NtQuerySystemInformation");
    if (!qsi)
        return 0;
    ULONG size = 256 * 1024, need = 0;
    BYTE *buf = NULL;
    for (int tries = 0; tries < 4; tries++) {
        buf = HeapAlloc(GetProcessHeap(), 0, size);
        if (!buf)
            return 0;
        LONG st = qsi(5, buf, size, &need);
        if (st >= 0)
            break;
        HeapFree(GetProcessHeap(), 0, buf);
        buf = NULL;
        if (st != (LONG)0xC0000004)
            return 0;
        size = need > size ? need + 64 * 1024 : size * 2;
    }
    if (!buf)
        return 0;
    const size_t pid_off = sizeof(void *) == 8 ? 0x50 : 0x44;
    const size_t name_buf_off = sizeof(void *) == 8 ? 0x40 : 0x3C;
    FILETIME now_ft;
    GetSystemTimeAsFileTime(&now_ft);
    LONGLONG now = ((LONGLONG)now_ft.dwHighDateTime << 32) | now_ft.dwLowDateTime;
    DWORD hog = 0;
    struct { BYTE *p; LONGLONG cpu; } rows[512];
    int n = 0;
    for (BYTE *p = buf; n < 512;) {
        ULONG next = *(ULONG *)p;
        rows[n].p = p;
        rows[n].cpu = *(LONGLONG *)(p + 0x28) + *(LONGLONG *)(p + 0x30);
        n++;
        if (!next)
            break;
        p += next;
    }
    for (int i = 0; i < n && i < top; i++) {
        int best = i;
        for (int j = i + 1; j < n; j++)
            if (rows[j].cpu > rows[best].cpu)
                best = j;
        if (best != i) {
            BYTE *tp = rows[i].p; LONGLONG tc = rows[i].cpu;
            rows[i] = rows[best];
            rows[best].p = tp; rows[best].cpu = tc;
        }
        BYTE *p = rows[i].p;
        USHORT len = *(USHORT *)(p + 0x38);
        const WCHAR *name = *(const WCHAR **)(p + name_buf_off);
        DWORD pid = (DWORD)(ULONG_PTR)*(HANDLE *)(p + pid_off);
        if (!hog && pid > 4 && pid != GetCurrentProcessId())
            hog = pid;
        LONGLONG created = *(LONGLONG *)(p + 0x20);
        LONGLONG age_ms = created ? (now - created) / 10000 : 0;
        SAY("cputime %s pid=%lu %-.*ls kernel=%lld ms user=%lld ms age=%lld ms\n", tag,
            (unsigned long)pid, name ? (int)(len / 2) : 6,
            name ? name : L"(idle)", *(LONGLONG *)(p + 0x30) / 10000,
            *(LONGLONG *)(p + 0x28) / 10000, age_ms);
    }
    HeapFree(GetProcessHeap(), 0, buf);
    return hog;
}

#if defined(_M_X64) || defined(__x86_64__)
/* A 32-bit thread under WOW64: its 64-bit context is the WOW64 layer (wow64.dll, win32u
 * thunks), which says only "in a syscall". Its x86 context (Wow64GetThreadContext, current
 * at a syscall boundary) and 32-bit stack say which guest code made the call. Modules come
 * from the guest loader's own list: ProcessWow64Information is the PEB32; PEB32.Ldr 0x0c,
 * PEB_LDR_DATA32.InLoadOrderModuleList 0x0c, LDR_DATA_TABLE_ENTRY32 DllBase 0x18,
 * SizeOfImage 0x20, BaseDllName 0x2c (Buffer at 0x30). */
struct sd_mod32 { DWORD base, size; char name[32]; };
static int sd_wow64_modules(HANDLE ph, struct sd_mod32 *mods, int max)
{
    typedef LONG (NTAPI *qip_fn)(HANDLE, ULONG, PVOID, ULONG, PULONG);
    qip_fn qip = (qip_fn)GetProcAddress(GetModuleHandleA("ntdll.dll"), "NtQueryInformationProcess");
    ULONG_PTR peb32 = 0;
    if (!qip || qip(ph, 26 /* ProcessWow64Information */, &peb32, sizeof peb32, NULL) < 0 || !peb32)
        return 0;
    DWORD ldr = 0, head, link;
    SIZE_T n;
    if (!ReadProcessMemory(ph, (LPCVOID)(peb32 + 0x0c), &ldr, 4, &n) || !ldr)
        return 0;
    head = ldr + 0x0c;
    if (!ReadProcessMemory(ph, (LPCVOID)(ULONG_PTR)head, &link, 4, &n))
        return 0;
    int count = 0;
    while (link && link != head && count < max) {
        BYTE e[0x34];
        if (!ReadProcessMemory(ph, (LPCVOID)(ULONG_PTR)link, e, sizeof e, &n))
            break;
        WCHAR w[32] = { 0 };
        USHORT len = *(USHORT *)(e + 0x2c);
        DWORD buf = *(DWORD *)(e + 0x30);
        if (len > sizeof w - 2)
            len = sizeof w - 2;
        if (buf)
            ReadProcessMemory(ph, (LPCVOID)(ULONG_PTR)buf, w, len, &n);
        mods[count].base = *(DWORD *)(e + 0x18);
        mods[count].size = *(DWORD *)(e + 0x20);
        stackdump_ascii(w, mods[count].name, sizeof mods[count].name);
        count++;
        link = *(DWORD *)e;
    }
    return count;
}
static void sd_wow64_where(const struct sd_mod32 *mods, int nm, DWORD v, char *out, size_t cap)
{
    for (int m = 0; m < nm; m++)
        if (v >= mods[m].base && v - mods[m].base < mods[m].size) {
            wsprintfA(out, "%s+0x%lx", mods[m].name, (unsigned long)(v - mods[m].base));
            return;
        }
    (void)cap;
    out[0] = 0;
}
static void sd_wow64_dump(HANDLE ph, HANDLE th, DWORD pid, DWORD tid)
{
    typedef BOOL (WINAPI *w64ctx_fn)(HANDLE, PWOW64_CONTEXT);
    w64ctx_fn get = (w64ctx_fn)GetProcAddress(GetModuleHandleA("kernel32.dll"), "Wow64GetThreadContext");
    BOOL wow = FALSE;
    if (!get || !IsWow64Process(ph, &wow) || !wow)
        return;
    static struct sd_mod32 mods[96];
    int nm = sd_wow64_modules(ph, mods, 96);
    WOW64_CONTEXT c;
    char pcs[8 * 40] = "", where[64];
    for (int k = 0; k < 8; k++) {
        memset(&c, 0, sizeof c);
        c.ContextFlags = WOW64_CONTEXT_CONTROL;
        SuspendThread(th);
        BOOL ok = get(th, &c);
        ResumeThread(th);
        if (ok) {
            sd_wow64_where(mods, nm, c.Eip, where, sizeof where);
            wsprintfA(pcs + lstrlenA(pcs), " %08lx%s%s", (unsigned long)c.Eip, where[0] ? ":" : "", where);
        } else {
            wsprintfA(pcs + lstrlenA(pcs), " (err %lu)", GetLastError());
        }
        Sleep(100);
    }
    SAY("  x86 pc samples pid=%lu tid=%lu (%d guest modules):%s\n", pid, tid, nm, pcs);
    memset(&c, 0, sizeof c);
    c.ContextFlags = WOW64_CONTEXT_CONTROL;
    SuspendThread(th);
    BOOL ok = get(th, &c);
    DWORD stack[256];
    SIZE_T got = 0;
    if (ok)
        for (SIZE_T off = 0; off < sizeof stack; off += 256) {
            SIZE_T n = 0;
            if (!ReadProcessMemory(ph, (LPCVOID)(ULONG_PTR)(c.Esp + off), (char *)stack + off, 256, &n) || n != 256)
                break;
            got += 256;
        }
    ResumeThread(th);
    int shown = 0;
    for (SIZE_T i = 0; i < got / 4 && shown < 30; i++) {
        sd_wow64_where(mods, nm, stack[i], where, sizeof where);
        if (where[0]) {
            SAY("  x86 stack +0x%x %08lx %s\n", (unsigned)(i * 4), (unsigned long)stack[i], where);
            shown++;
        }
    }
}
#endif

static void dump_thread_body(DWORD pid, DWORD tid)
{
    MODULEENTRY32W me[128];
    int nm = 0;
    SD_STAGE(0);
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid);
    if (snap != INVALID_HANDLE_VALUE) {
        me[0].dwSize = sizeof me[0];
        for (BOOL more = Module32FirstW(snap, &me[0]); more && nm < 127; ) {
            nm++;
            me[nm].dwSize = sizeof me[nm];
            more = Module32NextW(snap, &me[nm]);
        }
        CloseHandle(snap);
    }
    SD_STAGE(1);
    HANDLE th = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT, FALSE, tid);
    HANDLE ph = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, FALSE, pid);
    if (!th || !ph) {
        SAY("  stuck pid=%lu tid=%lu: cannot open (thread=%p process=%p)\n", pid, tid, th, ph);
        if (th) CloseHandle(th);
        if (ph) CloseHandle(ph);
        return;
    }
    CONTEXT ctx = { 0 };
    /* Eight PC samples first: one snapshot cannot tell a thread that is BLOCKED (same PC, every
     * time) from one that is LOOPING (PCs wander through the same few functions). */
    char pcs[8 * 20] = "";
    for (int k = 0; k < 8; k++) {
        ctx.ContextFlags = CONTEXT_CONTROL;
        SD_STAGE(2);
        SuspendThread(th);
        SD_STAGE(3);
        if (GetThreadContext(th, &ctx)) {
#if defined(_M_X64) || defined(__x86_64__)
            ULONG_PTR p = ctx.Rip;
#elif defined(_M_ARM64) || defined(__aarch64__)
            ULONG_PTR p = ctx.Pc;
#else
            ULONG_PTR p = ctx.Eip;
#endif
            wsprintfA(pcs + lstrlenA(pcs), " %p", (void *)p);
        }
        SD_STAGE(4);
        ResumeThread(th);
        Sleep(100);
    }
    SAY("  pc samples:%s\n", pcs);
#if defined(_M_X64) || defined(__x86_64__)
    sd_wow64_dump(ph, th, pid, tid);
#endif
    ctx.ContextFlags = CONTEXT_CONTROL;
    SD_STAGE(2);
    SuspendThread(th);
    SD_STAGE(3);
    BOOL got = GetThreadContext(th, &ctx);
#if defined(_M_X64) || defined(__x86_64__)
    ULONG_PTR pc = ctx.Rip, sp = ctx.Rsp;
#elif defined(_M_ARM64) || defined(__aarch64__)
    ULONG_PTR pc = ctx.Pc, sp = ctx.Sp;
#else
    ULONG_PTR pc = ctx.Eip, sp = ctx.Esp;
#endif
    /* Not static: a probe abandoned at its deadline may still write here later. */
    ULONG_PTR *stack = HeapAlloc(GetProcessHeap(), 0, 1024 * sizeof(ULONG_PTR));
    const SIZE_T stack_bytes = stack ? 1024 * sizeof(ULONG_PTR) : 0;
    SIZE_T got_bytes = 0;
    /* In 512-byte steps, keeping what reads: one read of the whole buffer fails outright when
     * the stack's top is nearer than sizeof stack, which hid every frame of a thread parked
     * near the top of a small stack (an x86-64 guest thread's, under wowx64). */
    SD_STAGE(5);
    while (got && got_bytes < stack_bytes) {
        SIZE_T n = 0;
        if (!ReadProcessMemory(ph, (LPCVOID)(sp + got_bytes), (char *)stack + got_bytes, 512, &n) || n != 512)
            break;
        got_bytes += 512;
    }
    SD_STAGE(4);
    ResumeThread(th);
    CloseHandle(th);
    CloseHandle(ph);
    if (!got) {
        HeapFree(GetProcessHeap(), 0, stack);
        SAY("  stuck pid=%lu tid=%lu: GetThreadContext failed %lu\n", pid, tid, GetLastError());
        return;
    }
    int shown = 0;
    for (int i = -1; i < (int)(got_bytes / sizeof(ULONG_PTR)) && shown < 40; i++) {
        ULONG_PTR v = i < 0 ? pc : stack[i];
        for (int m = 0; m < nm; m++) {
            ULONG_PTR b = (ULONG_PTR)me[m].modBaseAddr;
            if (v >= b && v < b + me[m].modBaseSize) {
                char name[64];
                stackdump_ascii(me[m].szModule, name, sizeof name);
                if (i < 0)
                    SAY("  stuck pid=%lu tid=%lu pc=%p %s+0x%lx\n", pid, tid, (void *)v, name, (unsigned long)(v - b));
                else
                    SAY("  stack +0x%x %p %s+0x%lx\n", i * (int)sizeof(ULONG_PTR), (void *)v, name, (unsigned long)(v - b));
                shown++;
                break;
            }
        }
    }
    HeapFree(GetProcessHeap(), 0, stack);
}

struct sd_args {
    DWORD pid, tid;
};

static DWORD WINAPI dump_thread_worker(LPVOID p)
{
    struct sd_args *a = p;
    dump_thread_body(a->pid, a->tid);
    HeapFree(GetProcessHeap(), 0, a);
    return 0;
}

/* The probe runs on its own thread with a deadline. Getting another thread's context makes the
 * kernel queue an APC to it and wait, so a target that never takes the APC used to block the
 * whole sweep for good: the payload then stalled silently for an hour (registry: "amd64 Chrome
 * payload: the guest goes silent"). Now it says which call did not return and moves on; the
 * stuck worker is abandoned. */
static void dump_thread(DWORD pid, DWORD tid)
{
    struct sd_args *a = HeapAlloc(GetProcessHeap(), 0, sizeof *a);
    if (!a)
        return;
    a->pid = pid;
    a->tid = tid;
    SD_STAGE(0);
    struct sd_thread_view before, after;
    sd_sample_thread(pid, tid, &before);
    HANDLE w = CreateThread(NULL, 0, dump_thread_worker, a, 0, NULL);
    if (!w) {
        HeapFree(GetProcessHeap(), 0, a);
        return;
    }
    if (WaitForSingleObject(w, 20000) == WAIT_TIMEOUT) {
        LONG st = g_sd_stage;
        SAY("  stuck pid=%lu tid=%lu: PROBE-TIMEOUT in %s (20s), abandoned\n", pid, tid,
            st >= 0 && st < (LONG)(sizeof g_sd_stages / sizeof g_sd_stages[0]) ? g_sd_stages[st] : "?");
        sd_sample_thread(pid, tid, &after);
        if (before.found && after.found)
            SAY("  stuck pid=%lu tid=%lu: state=%lu wait=%lu start=%p over 20s: kernel +%lu ms, "
                "user +%lu ms, switches +%lu\n", pid, tid, after.state, after.wait, after.start,
                (unsigned long)((after.ktime - before.ktime) / 10000),
                (unsigned long)((after.utime - before.utime) / 10000),
                after.switches - before.switches);
        else
            SAY("  stuck pid=%lu tid=%lu: no thread view (before=%d after=%d)\n", pid, tid,
                before.found, after.found);
        /* The probe stuck in GetThreadContext had suspended the target first. Left suspended,
         * the target's process could not be killed at its timeout and the sweep waited on it
         * for good. Undo that suspend here; when the late probe resumes too, a resume of an
         * unsuspended thread is a no-op. */
        if (st == 3) {
            HANDLE th = OpenThread(THREAD_SUSPEND_RESUME, FALSE, tid);
            if (th) {
                ResumeThread(th);
                CloseHandle(th);
            }
        }
    }
    CloseHandle(w);
}
#endif
