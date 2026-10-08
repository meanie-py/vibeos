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

static void dump_thread(DWORD pid, DWORD tid)
{
    MODULEENTRY32W me[128];
    int nm = 0;
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
        SuspendThread(th);
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
        ResumeThread(th);
        Sleep(100);
    }
    SAY("  pc samples:%s\n", pcs);
    ctx.ContextFlags = CONTEXT_CONTROL;
    SuspendThread(th);
    BOOL got = GetThreadContext(th, &ctx);
#if defined(_M_X64) || defined(__x86_64__)
    ULONG_PTR pc = ctx.Rip, sp = ctx.Rsp;
#elif defined(_M_ARM64) || defined(__aarch64__)
    ULONG_PTR pc = ctx.Pc, sp = ctx.Sp;
#else
    ULONG_PTR pc = ctx.Eip, sp = ctx.Esp;
#endif
    static ULONG_PTR stack[1024];
    SIZE_T got_bytes = 0;
    /* In 512-byte steps, keeping what reads: one read of the whole buffer fails outright when
     * the stack's top is nearer than sizeof stack, which hid every frame of a thread parked
     * near the top of a small stack (an x86-64 guest thread's, under wowx64). */
    while (got && got_bytes < sizeof stack) {
        SIZE_T n = 0;
        if (!ReadProcessMemory(ph, (LPCVOID)(sp + got_bytes), (char *)stack + got_bytes, 512, &n) || n != 512)
            break;
        got_bytes += 512;
    }
    ResumeThread(th);
    CloseHandle(th);
    CloseHandle(ph);
    if (!got) {
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
}
#endif
