/* regstress -- concurrent registry writers and readers in one hive.
 *
 * WRITERS threads each own a key under HKCU\Software\VibeRegStress and keep resizing its
 * values (1 B .. 12 KB, so cells are reallocated and the hive keeps adding bins) and
 * creating/deleting subkeys; READERS threads enumerate and read every writer's values at
 * the same time. Each value carries its own length and a fill byte, so a reader can tell a
 * torn or misplaced cell from a legitimate concurrent update. With two CPUs this drives
 * HvAllocateCell/HvpAddBin on the same hive from both at once -- the race that bugchecked
 * arm64 -smp 2 in CmpCreateKeyControlBlock before cmlib took CMHIVE.HiveLock.
 */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[REGSTRESS] " __VA_ARGS__)

#define WRITERS 4
#define READERS 2
#define VALUES 48
#define MAXDATA 12288
#define RUN_MS 30000

int __cdecl atoi(const char *);

static volatile LONG g_stop, g_fail, g_writes, g_reads, g_subkeys;
static HKEY g_root;

static void fail(const char *what, LONG err, int w, int v)
{
    if (InterlockedIncrement(&g_fail) <= 20)
        SAY("FAIL %s err=%ld writer=%d value=%d\n", what, err, w, v);
}

/* data = [u32 length][u32 tag][fill...], fill = tag & 0xFF */
static DWORD fill_value(BYTE *buf, int w, int v, DWORD round)
{
    DWORD len = 8 + ((round * 2654435761u + v * 40503u) % (MAXDATA - 8));
    DWORD tag = ((DWORD)w << 24) | ((DWORD)v << 16) | (round & 0xFFFF);
    BYTE f = (BYTE)(tag ^ (tag >> 8) ^ (tag >> 16));
    ((DWORD *)buf)[0] = len;
    ((DWORD *)buf)[1] = tag;
    for (DWORD i = 8; i < len; i++) buf[i] = f;
    return len;
}

static int check_value(const BYTE *buf, DWORD got, int w, int v)
{
    DWORD len, tag;
    if (got < 8) return 0;
    len = ((const DWORD *)buf)[0];
    tag = ((const DWORD *)buf)[1];
    if (len != got || (int)(tag >> 24) != w || (int)((tag >> 16) & 0xFF) != v) return 0;
    BYTE f = (BYTE)(tag ^ (tag >> 8) ^ (tag >> 16));
    for (DWORD i = 8; i < len; i++)
        if (buf[i] != f) return 0;
    return 1;
}

static DWORD WINAPI writer(void *arg)
{
    int w = (int)(ULONG_PTR)arg;
    char name[32];
    HKEY k;
    BYTE *buf = HeapAlloc(GetProcessHeap(), 0, MAXDATA);
    DWORD round = 0;
    LONG e;

    wsprintfA(name, "W%d", w);
    e = RegCreateKeyExA(g_root, name, 0, NULL, 0, KEY_ALL_ACCESS, NULL, &k, NULL);
    if (e) { fail("create writer key", e, w, -1); return 0; }
    while (!g_stop) {
        for (int v = 0; v < VALUES && !g_stop; v++) {
            char vn[16];
            DWORD len = fill_value(buf, w, v, round);
            wsprintfA(vn, "v%d", v);
            e = RegSetValueExA(k, vn, 0, REG_BINARY, buf, len);
            if (e) fail("set", e, w, v);
            InterlockedIncrement(&g_writes);
            if ((round + v) % 7 == 0) {
                e = RegDeleteValueA(k, vn);
                if (e) fail("delete value", e, w, v);
            }
        }
        /* Subkeys grow the key's index cells and the KCB cache under the same hive. */
        {
            char sk[32];
            HKEY s;
            wsprintfA(sk, "S%lu", round % 16);
            if (round & 1) {
                e = RegCreateKeyExA(k, sk, 0, NULL, 0, KEY_ALL_ACCESS, NULL, &s, NULL);
                if (e) fail("create subkey", e, w, (int)round);
                else { RegSetValueExA(s, "x", 0, REG_BINARY, buf, 64); RegCloseKey(s); InterlockedIncrement(&g_subkeys); }
            } else {
                e = RegDeleteKeyA(k, sk);
                if (e && e != ERROR_FILE_NOT_FOUND) fail("delete subkey", e, w, (int)round);
            }
        }
        round++;
    }
    RegCloseKey(k);
    HeapFree(GetProcessHeap(), 0, buf);
    return 0;
}

static DWORD WINAPI reader(void *arg)
{
    BYTE *buf = HeapAlloc(GetProcessHeap(), 0, MAXDATA);
    (void)arg;
    while (!g_stop) {
        for (int w = 0; w < WRITERS && !g_stop; w++) {
            char name[32];
            HKEY k;
            wsprintfA(name, "W%d", w);
            if (RegOpenKeyExA(g_root, name, 0, KEY_READ, &k)) continue;
            for (DWORD i = 0;; i++) {
                char vn[32];
                DWORD vnl = sizeof(vn), got = MAXDATA, type;
                LONG e = RegEnumValueA(k, i, vn, &vnl, NULL, &type, buf, &got);
                if (e == ERROR_NO_MORE_ITEMS) break;
                /* A concurrent delete shifts the index; a concurrent grow can outrun buf. */
                if (e == ERROR_MORE_DATA || e == ERROR_FILE_NOT_FOUND) continue;
                if (e) { fail("enum", e, w, (int)i); break; }
                int v = atoi(vn + 1);
                if (type != REG_BINARY || !check_value(buf, got, w, v))
                    fail("torn value", (LONG)got, w, v);
                InterlockedIncrement(&g_reads);
            }
            RegCloseKey(k);
        }
    }
    HeapFree(GetProcessHeap(), 0, buf);
    return 0;
}

/* RegDeleteTreeA is a Vista export; this advapi32 is NT 5.2. */
static void del_tree(HKEY parent, const char *name)
{
    HKEY k;
    char sub[64];
    if (RegOpenKeyExA(parent, name, 0, KEY_ALL_ACCESS, &k)) return;
    for (;;) {
        DWORD n = sizeof(sub);
        if (RegEnumKeyExA(k, 0, sub, &n, NULL, NULL, NULL, NULL)) break;
        del_tree(k, sub);
    }
    RegCloseKey(k);
    RegDeleteKeyA(parent, name);
}

int main(void)
{
    HANDLE th[WRITERS + READERS];
    SYSTEM_INFO si;
    LONG e;

    GetSystemInfo(&si);
    SAY("start cpus=%lu writers=%d readers=%d run=%dms\n", si.dwNumberOfProcessors, WRITERS, READERS, RUN_MS);
    del_tree(HKEY_CURRENT_USER, "Software\\VibeRegStress");
    e = RegCreateKeyExA(HKEY_CURRENT_USER, "Software\\VibeRegStress", 0, NULL, 0, KEY_ALL_ACCESS, NULL, &g_root, NULL);
    if (e) { SAY("FAIL create root err=%ld\n", e); SAY("done FAIL\n"); return 1; }

    for (int i = 0; i < WRITERS; i++) th[i] = CreateThread(NULL, 0, writer, (void *)(ULONG_PTR)i, 0, NULL);
    for (int i = 0; i < READERS; i++) th[WRITERS + i] = CreateThread(NULL, 0, reader, NULL, 0, NULL);
    /* Heartbeat: a stall shows WHICH side stopped -- the workers (counts freeze) or
     * the whole process (the beat itself stops). */
    for (DWORD t0 = GetTickCount(), now; (now = GetTickCount()) - t0 < RUN_MS;) {
        Sleep(5000);
        SAY("beat t=%lums writes=%ld reads=%ld subkeys=%ld failures=%ld\n",
            GetTickCount() - t0, g_writes, g_reads, g_subkeys, g_fail);
    }
    g_stop = 1;
    if (WaitForMultipleObjects(WRITERS + READERS, th, TRUE, 60000) != WAIT_OBJECT_0)
        fail("workers did not stop within 60s", 0, -1, -1);
    SAY("workers joined\n");

    /* Every value still present must read back whole after the storm. */
    for (int w = 0; w < WRITERS; w++) {
        char name[32];
        HKEY k;
        BYTE *buf = HeapAlloc(GetProcessHeap(), 0, MAXDATA);
        wsprintfA(name, "W%d", w);
        if (RegOpenKeyExA(g_root, name, 0, KEY_READ, &k) == 0) {
            for (int v = 0; v < VALUES; v++) {
                char vn[16];
                DWORD got = MAXDATA, type;
                wsprintfA(vn, "v%d", v);
                e = RegQueryValueExA(k, vn, NULL, &type, buf, &got);
                if (e == ERROR_FILE_NOT_FOUND) continue;
                if (e || !check_value(buf, got, w, v)) fail("final readback", e, w, v);
            }
            RegCloseKey(k);
        } else fail("reopen", 0, w, -1);
        HeapFree(GetProcessHeap(), 0, buf);
    }
    RegCloseKey(g_root);
    del_tree(HKEY_CURRENT_USER, "Software\\VibeRegStress");

    SAY("writes=%ld reads=%ld subkeys=%ld failures=%ld\n", g_writes, g_reads, g_subkeys, g_fail);
    SAY("done %s\n", g_fail || g_writes < 100 ? "FAIL" : "PASS");
    return 0;
}
