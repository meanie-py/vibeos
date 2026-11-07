/* memwatch.h: one line of where the machine's memory is, for the sweep's `memwatch SECS`.
 * A Chrome run wedged the guest out of memory (win32k "Not enough memory to allocate a
 * message", NtQuerySystemInformation failing with STATUS_INSUFFICIENT_RESOURCES) with nothing
 * in the log saying what had grown. Printed every SECS while a child runs, these lines put the
 * growth on the timeline: physical and commit, the pool tags holding the most, and the
 * processes holding the most private memory. Included by sweep.c, and compiled alone by
 * memwatch_test.c to check its numbers against Windows. */
#include <windows.h>
#include <winternl.h>
#include <stdio.h>
#include <string.h>

/* SYSTEM_POOLTAG_INFORMATION (class 22): not in the SDK headers. */
typedef struct {
    union {
        UCHAR Tag[4];
        ULONG TagUlong;
    };
    ULONG PagedAllocs;
    ULONG PagedFrees;
    SIZE_T PagedUsed;
    ULONG NonPagedAllocs;
    ULONG NonPagedFrees;
    SIZE_T NonPagedUsed;
} MW_POOLTAG;

#define MW_TOP 6

static unsigned char *g_mw_buf;
static ULONG g_mw_cap;

/* NtQuerySystemInformation into g_mw_buf, growing it (to 16M) while the answer does not fit. */
static NTSTATUS mw_query(ULONG cls)
{
    NTSTATUS st;
    ULONG got = 0;
    for (;;) {
        if (!g_mw_buf) {
            g_mw_cap = g_mw_cap ? g_mw_cap : 256 * 1024;
            g_mw_buf = VirtualAlloc(NULL, g_mw_cap, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
            if (!g_mw_buf)
                return (NTSTATUS)0xC0000017; /* STATUS_NO_MEMORY */
        }
        st = NtQuerySystemInformation((SYSTEM_INFORMATION_CLASS)cls, g_mw_buf, g_mw_cap, &got);
        if (st != (NTSTATUS)0xC0000004 || g_mw_cap >= 16 * 1024 * 1024)
            return st;
        VirtualFree(g_mw_buf, 0, MEM_RELEASE);
        g_mw_buf = NULL;
        g_mw_cap *= 2;
    }
}

static void mw_tag(char out[5], const UCHAR *t)
{
    for (int i = 0; i < 4; i++)
        out[i] = (t[i] >= 32 && t[i] < 127) ? (char)t[i] : '.';
    out[4] = 0;
}

/* Append to `line` (cap bytes), never past its end. */
static void mw_cat(char *line, size_t cap, const char *fmt, ...)
{
    size_t n = strlen(line);
    va_list ap;
    if (n + 1 >= cap)
        return;
    va_start(ap, fmt);
    vsnprintf(line + n, cap - n, fmt, ap);
    va_end(ap);
}

static void memwatch_line(char *line, size_t cap)
{
    MEMORYSTATUSEX ms = { sizeof ms };
    NTSTATUS st;
    line[0] = 0;
    if (GlobalMemoryStatusEx(&ms))
        mw_cat(line, cap, "phys %lluM free of %lluM, commit %lluM of %lluM",
               ms.ullAvailPhys >> 20, ms.ullTotalPhys >> 20,
               (ms.ullTotalPageFile - ms.ullAvailPageFile) >> 20, ms.ullTotalPageFile >> 20);
    else
        mw_cat(line, cap, "GlobalMemoryStatusEx failed %lu", GetLastError());

    /* Pool tags by bytes held (paged + nonpaged). */
    st = mw_query(22);
    if (st >= 0) {
        ULONG count = *(ULONG *)g_mw_buf;
        MW_POOLTAG *tags = (MW_POOLTAG *)(g_mw_buf + sizeof(SIZE_T));
        int top[MW_TOP];
        int ntop = 0;
        SIZE_T total = 0;
        for (ULONG i = 0; i < count; i++) {
            SIZE_T used = tags[i].PagedUsed + tags[i].NonPagedUsed;
            total += used;
            int at = ntop;
            while (at > 0 && used > tags[top[at - 1]].PagedUsed + tags[top[at - 1]].NonPagedUsed)
                at--;
            if (at < MW_TOP) {
                if (ntop < MW_TOP)
                    ntop++;
                memmove(&top[at + 1], &top[at], (ntop - 1 - at) * sizeof top[0]);
                top[at] = (int)i;
            }
        }
        mw_cat(line, cap, " | pool %lluK:", (unsigned long long)(total >> 10));
        for (int i = 0; i < ntop; i++) {
            char t[5];
            MW_POOLTAG *p = &tags[top[i]];
            mw_tag(t, p->Tag);
            mw_cat(line, cap, " %s %lluK/%lluK(np)", t,
                   (unsigned long long)((p->PagedUsed + p->NonPagedUsed) >> 10),
                   (unsigned long long)(p->NonPagedUsed >> 10));
        }
    } else {
        mw_cat(line, cap, " | pool tags: %08lx", (unsigned long)st);
    }

    /* Processes by private bytes. */
    st = mw_query(SystemProcessInformation);
    if (st >= 0) {
        SYSTEM_PROCESS_INFORMATION *top[MW_TOP];
        int ntop = 0;
        SYSTEM_PROCESS_INFORMATION *p = (SYSTEM_PROCESS_INFORMATION *)g_mw_buf;
        for (;;) {
            int at = ntop;
            while (at > 0 && p->PrivatePageCount > top[at - 1]->PrivatePageCount)
                at--;
            if (at < MW_TOP) {
                if (ntop < MW_TOP)
                    ntop++;
                memmove(&top[at + 1], &top[at], (ntop - 1 - at) * sizeof top[0]);
                top[at] = p;
            }
            if (!p->NextEntryOffset)
                break;
            p = (SYSTEM_PROCESS_INFORMATION *)((char *)p + p->NextEntryOffset);
        }
        mw_cat(line, cap, " | private:");
        for (int i = 0; i < ntop; i++)
            mw_cat(line, cap, " %.*ls/%lu %lluM ws %lluM", (int)(top[i]->ImageName.Length / 2),
                   top[i]->ImageName.Buffer ? top[i]->ImageName.Buffer : L"idle",
                   (unsigned long)(ULONG_PTR)top[i]->UniqueProcessId,
                   (unsigned long long)(top[i]->PrivatePageCount >> 20),
                   (unsigned long long)(top[i]->VirtualMemoryCounters.WorkingSetSize >> 20));
        /* GDI objects per process: a process leaking DCs or bitmaps climbs here. Looked up at
         * run time, so the sweep does not load user32 unless memwatch is on. */
        static DWORD(WINAPI * gui_res)(HANDLE, DWORD);
        if (!gui_res) {
            HMODULE u = LoadLibraryA("user32.dll");
            gui_res = u ? (DWORD(WINAPI *)(HANDLE, DWORD))(void *)GetProcAddress(u, "GetGuiResources") : NULL;
        }
        if (gui_res) {
            DWORD best[3] = { 0 }, bestpid[3] = { 0 };
            const WCHAR *bestname[3] = { 0 };
            int bestlen[3] = { 0 };
            p = (SYSTEM_PROCESS_INFORMATION *)g_mw_buf;
            for (;;) {
                DWORD pid = (DWORD)(ULONG_PTR)p->UniqueProcessId;
                HANDLE h = pid ? OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pid) : NULL;
                if (h) {
                    DWORD n = gui_res(h, 0 /* GR_GDIOBJECTS */);
                    CloseHandle(h);
                    for (int i = 0; i < 3; i++) {
                        if (n > best[i]) {
                            for (int j = 2; j > i; j--) {
                                best[j] = best[j - 1];
                                bestpid[j] = bestpid[j - 1];
                                bestname[j] = bestname[j - 1];
                                bestlen[j] = bestlen[j - 1];
                            }
                            best[i] = n;
                            bestpid[i] = pid;
                            bestname[i] = p->ImageName.Buffer;
                            bestlen[i] = p->ImageName.Length / 2;
                            break;
                        }
                    }
                }
                if (!p->NextEntryOffset)
                    break;
                p = (SYSTEM_PROCESS_INFORMATION *)((char *)p + p->NextEntryOffset);
            }
            mw_cat(line, cap, " | gdi:");
            for (int i = 0; i < 3 && best[i]; i++)
                mw_cat(line, cap, " %.*ls/%lu %lu", bestlen[i], bestname[i] ? bestname[i] : L"?",
                       (unsigned long)bestpid[i], (unsigned long)best[i]);
        }
    } else {
        mw_cat(line, cap, " | processes: %08lx", (unsigned long)st);
    }
}
