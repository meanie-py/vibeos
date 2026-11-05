/* dumpinfo: summarize every minidump in a directory to stdout, so a crash that Crashpad caught
 * (and so never reached the kernel's unhandled-exception print) is readable from the serial:
 * the exception record, the faulting thread's RIP/RSP, the module list, and every stack slot
 * of that thread pointing into a module (as module+offset). Raw MINIDUMP_* parsing, no
 * dbghelp: the guest's dbghelp is not the thing under test. Usage: dumpinfo <dir> */
#include <windows.h>
#include <stdio.h>
#include <string.h>

#pragma pack(push, 4)
typedef struct { DWORD Sig, Ver, NStreams, DirRva, CheckSum, Time; ULONG64 Flags; } Hdr;
typedef struct { DWORD Type, Size, Rva; } Dir;
typedef struct { ULONG64 Base; DWORD Size, CheckSum, Time, NameRva; BYTE rest[108 - 24]; } Mod;
typedef struct { DWORD ThreadId, align; DWORD Code, Flags; ULONG64 Rec, Addr; DWORD NParams, align2; ULONG64 Info[15]; DWORD CtxSize, CtxRva; } Exc;
typedef struct { DWORD ThreadId, Suspend, PriClass, Pri; ULONG64 Teb; ULONG64 StackStart; DWORD StackSize, StackRva; DWORD CtxSize, CtxRva; } Thr;
#pragma pack(pop)

static BYTE *g; static DWORD gn; static Mod *mods; static DWORD nmods;

static void modname(DWORD rva, char *out, int n)
{
    DWORD len = *(DWORD *)(g + rva) / 2, i; WCHAR *w = (WCHAR *)(g + rva + 4); const WCHAR *base = w;
    for (i = 0; i < len; i++) if (w[i] == L'\\') base = w + i + 1;
    WideCharToMultiByte(CP_ACP, 0, base, (int)(len - (base - w)), out, n - 1, NULL, NULL);
    out[n - 1] = 0; out[len - (base - w) < (DWORD)n ? len - (base - w) : (DWORD)n - 1] = 0;
}

static int where(ULONG64 a, char *out, int n)
{
    DWORD i; char nm[128];
    for (i = 0; i < nmods; i++)
        if (a >= mods[i].Base && a < mods[i].Base + mods[i].Size) {
            modname(mods[i].NameRva, nm, sizeof(nm));
            _snprintf(out, n, "%s+0x%llx", nm, (unsigned long long)(a - mods[i].Base));
            return 1;
        }
    return 0;
}

static void one(const char *path)
{
    HANDLE f = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING, 0, NULL);
    DWORD i, got; Hdr *h; Dir *d; Exc *e = NULL; Thr *thr = NULL; DWORD nthr = 0; char w[192];
    printf("[DUMPINFO] file %s\n", path);
    if (f == INVALID_HANDLE_VALUE) { printf("[DUMPINFO] cannot open (%lu)\n", GetLastError()); return; }
    gn = GetFileSize(f, NULL); g = HeapAlloc(GetProcessHeap(), 0, gn);
    ReadFile(f, g, gn, &got, NULL); CloseHandle(f);
    h = (Hdr *)g;
    if (got < sizeof(Hdr) || h->Sig != 0x504d444d) { printf("[DUMPINFO] not a minidump (%lu bytes)\n", got); return; }
    d = (Dir *)(g + h->DirRva); mods = NULL; nmods = 0;
    for (i = 0; i < h->NStreams; i++) {
        if (d[i].Type == 4) { nmods = *(DWORD *)(g + d[i].Rva); mods = (Mod *)(g + d[i].Rva + 4); }
        if (d[i].Type == 6) e = (Exc *)(g + d[i].Rva);
        if (d[i].Type == 3) { nthr = *(DWORD *)(g + d[i].Rva); thr = (Thr *)(g + d[i].Rva + 4); }
    }
    for (i = 0; i < nmods; i++) { modname(mods[i].NameRva, w, sizeof(w)); printf("[DUMPINFO] module %016llx %08lx %s\n", (unsigned long long)mods[i].Base, mods[i].Size, w); }
    if (!e) { printf("[DUMPINFO] no exception stream\n"); return; }
    w[0] = 0; where(e->Addr, w, sizeof(w));
    printf("[DUMPINFO] exception tid=%lu code=0x%08lx flags=0x%lx addr=%016llx %s nparams=%lu",
           e->ThreadId, e->Code, e->Flags, (unsigned long long)e->Addr, w, e->NParams);
    for (i = 0; i < e->NParams && i < 15; i++) printf(" p%lu=%llx", i, (unsigned long long)e->Info[i]);
    printf("\n");
    if (e->CtxSize >= 0x100) {
        CONTEXT *c = (CONTEXT *)(g + e->CtxRva);
        printf("[DUMPINFO] rip=%016llx rsp=%016llx rbp=%016llx rax=%llx rbx=%llx rcx=%llx rdx=%llx r8=%llx r9=%llx\n",
               c->Rip, c->Rsp, c->Rbp, c->Rax, c->Rbx, c->Rcx, c->Rdx, c->R8, c->R9);
        for (i = 0; i < nthr; i++) {
            ULONG64 *s, k, n;
            if (thr[i].ThreadId != e->ThreadId) continue;
            s = (ULONG64 *)(g + thr[i].StackRva); n = thr[i].StackSize / 8;
            /* The first words at rsp raw: a CHECK's operands (a result struct, an error code)
             * are usually locals of the faulting frame. */
            if (c->Rsp >= thr[i].StackStart && c->Rsp + 0x100 <= thr[i].StackStart + thr[i].StackSize) {
                ULONG64 *r = (ULONG64 *)(g + thr[i].StackRva + (c->Rsp - thr[i].StackStart)); int q;
                for (q = 0; q < 32; q += 4)
                    printf("[DUMPINFO] rsp+%03x %016llx %016llx %016llx %016llx\n", q * 8,
                           (unsigned long long)r[q], (unsigned long long)r[q + 1], (unsigned long long)r[q + 2], (unsigned long long)r[q + 3]);
            }
            /* Names on the stack: a delay-load failure hook keeps the DLL name in a local buffer. */
            {
                BYTE *sb = (BYTE *)s; DWORD j, run = 0;
                for (j = 0; j < thr[i].StackSize; j++) {
                    if (sb[j] >= 0x20 && sb[j] < 0x7f) { run++; continue; }
                    if (run >= 5 && (sb[j - 4] == '.' || run >= 12)) {
                        char str[256]; DWORD L = run < 255 ? run : 255;
                        memcpy(str, sb + j - run, L); str[L] = 0;
                        if (strstr(str, ".dll") || strstr(str, ".DLL")) printf("[DUMPINFO] stack string \"%s\"\n", str);
                    }
                    run = 0;
                }
            }
            for (k = c->Rsp >= thr[i].StackStart ? (c->Rsp - thr[i].StackStart) / 8 : 0; k < n; k++)
                if (where(s[k], w, sizeof(w)))
                    printf("[DUMPINFO] stack +%04llx %016llx %s\n", (unsigned long long)(k * 8 - (c->Rsp - thr[i].StackStart)), (unsigned long long)s[k], w);
        }
    }
}

int main(int argc, char **argv)
{
    WIN32_FIND_DATAA fd; HANDLE fh; char pat[MAX_PATH], p[MAX_PATH]; int n = 0;
    if (argc < 2) return 2;
    _snprintf(pat, sizeof(pat), "%s\\*.dmp", argv[1]);
    fh = FindFirstFileA(pat, &fd);
    if (fh != INVALID_HANDLE_VALUE) {
        do { _snprintf(p, sizeof(p), "%s\\%s", argv[1], fd.cFileName); one(p); n++; } while (FindNextFileA(fh, &fd));
        FindClose(fh);
    }
    printf("[DUMPINFO] %d dump(s) in %s\n", n, argv[1]);
    return 0;
}
