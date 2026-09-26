/*
 * reg32.exe — the i386 registry-view guest. Built with the llvm-mingw i686
 * target, no CRT (plain _start entry, same as Hello32.c/File32.c), importing
 * kernel32 + advapi32, loaded through the genuine 32-bit loader.
 *
 * What it proves: WoW64 REGISTRY REDIRECTION, from the only vantage point that
 * can see it — inside a real 32-bit process. Everything this guest checks is
 * invisible to the rest of the ladder, because the other nine guests never
 * touch the hive, and it is invisible to a host-side unit test, because the
 * whole feature keys off "is the CALLING process 32-bit"
 * (CmpWow64IsCurrentProcess32Bit, sdk/lib/rust/config-rust/src/cmwow64.rs).
 *
 * This is the class the ladder structurally could not see until now: a
 * redirection bug does not crash, hang or fail an API. It returns SUCCESS and
 * the wrong key. A 32-bit app writes its configuration into the 64-bit view and
 * the next reader gets someone else's answer — "runs x86 apps well" is false
 * while every gate is green.
 *
 * The rungs, and what each one would catch:
 *   1-2  a default-view create lands in Wow6432Node          (redirection happens at all)
 *   3    the same key is NOT in the native view              (isolation, the other direction)
 *   4    an explicit Wow6432Node path does not DOUBLE-splice (the one exemption, CmpWow64IsSharedKey)
 *   5    a 64-bit-only key still reads from the 32-bit view  (the miss-falls-back rule)
 *   6    RegDeleteKeyExW(KEY_WOW64_32KEY) deletes the 32-bit copy and LEAVES THE NATIVE ONE
 *
 * Rung 6 is the sharp one. Until 2026-09-13 RegDeleteKeyExW dropped the WoW64
 * view bits before NtOpenKey, so an explicit KEY_WOW64_32KEY delete opened
 * whatever the process's default view resolved to. It is the only rung here
 * whose failure mode is destructive rather than merely wrong, and it had no
 * test of any kind.
 *
 * Verdict contract (same as the other ladder guests): exit code 0x600D on PASS;
 * distinct 0xBADx codes name the failing rung so serial alone triages.
 */

#include <windows.h>

#define R32_FAIL_CREATE32     0xBAD1 /* default-view create failed outright */
#define R32_FAIL_SETVALUE     0xBAD2
#define R32_FAIL_NOT_IN_NODE  0xBAD3 /* create did NOT land in Wow6432Node -> no redirection */
#define R32_FAIL_NODE_VALUE   0xBAD4 /* found the key there but the value is wrong */
#define R32_FAIL_LEAKED_NATIVE 0xBAD5 /* key IS in the native view -> isolation broken */
#define R32_FAIL_DOUBLE_SPLICE 0xBAD6 /* explicit Wow6432Node path got spliced again */
#define R32_FAIL_NO_FALLBACK  0xBAD7 /* 64-bit-only key unreachable from the 32-bit view */
#define R32_FAIL_FALLBACK_VAL 0xBAD8
#define R32_FAIL_DEL_SETUP    0xBAD9 /* could not build the two-view delete fixture */
#define R32_FAIL_DELETE       0xBADA /* RegDeleteKeyExW(KEY_WOW64_32KEY) reported failure */
#define R32_FAIL_DEL_WRONGHIVE 0xBADB /* IT DELETED THE NATIVE KEY -- the 2026-09-13 bug */
#define R32_FAIL_DEL_NOTGONE  0xBADC /* 32-bit copy survived the delete */
#define R32_PASS              0x600D

/* KEY_WOW64_* are view selectors, not access bits (winnt.h). Spelled out here
 * because a -nostdlib i686 build against these headers does not always see
 * them, and a silently-zero selector would make every rung below pass for the
 * wrong reason -- the exact false-green this guest exists to prevent. */
#ifndef KEY_WOW64_64KEY
#define KEY_WOW64_64KEY 0x0100
#endif
#ifndef KEY_WOW64_32KEY
#define KEY_WOW64_32KEY 0x0200
#endif

#define PROBE_KEY    L"SOFTWARE\\VibeOSWow64Probe"
#define PROBE_IN_NODE L"SOFTWARE\\Wow6432Node\\VibeOSWow64Probe"
#define DEL_KEY      L"SOFTWARE\\VibeOSWow64DelProbe"
#define DEL_IN_NODE  L"SOFTWARE\\Wow6432Node\\VibeOSWow64DelProbe"
#define NATIVE_ONLY  L"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"

#define VIEW_MARK    L"View"
#define MARK_32      0x33320000u  /* written from the 32-bit view */
#define MARK_NATIVE  0x36340000u  /* written with KEY_WOW64_64KEY */

static void Say(const char *msg)
{
    /* OutputDebugStringA reaches serial via the guest DbgPrint forwarding
     * (int 2d DebugService -> [GUEST32] in the wow64 VEH). */
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

/* Create (or open) a key in a chosen view and stamp a DWORD into it. */
static LONG MakeKey(const WCHAR *path, REGSAM view, DWORD mark)
{
    HKEY h = NULL;
    DWORD disp = 0;
    LONG rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, path, 0, NULL, 0,
                              KEY_READ | KEY_WRITE | view, NULL, &h, &disp);
    if (rc != ERROR_SUCCESS)
        return rc;
    rc = RegSetValueExW(h, VIEW_MARK, 0, REG_DWORD,
                        (const BYTE *)&mark, sizeof(mark));
    RegCloseKey(h);
    return rc;
}

/* Read the stamp back from a chosen view. Returns ERROR_SUCCESS and *out only
 * when both the open and the query succeed. */
static LONG ReadMark(const WCHAR *path, REGSAM view, DWORD *out)
{
    HKEY h = NULL;
    DWORD type = 0, cb = sizeof(DWORD), val = 0;
    LONG rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, path, 0, KEY_READ | view, &h);
    if (rc != ERROR_SUCCESS)
        return rc;
    rc = RegQueryValueExW(h, VIEW_MARK, NULL, &type, (BYTE *)&val, &cb);
    RegCloseKey(h);
    if (rc == ERROR_SUCCESS && type != REG_DWORD)
        return ERROR_INVALID_DATA;
    if (rc == ERROR_SUCCESS)
        *out = val;
    return rc;
}

static BOOL KeyExists(const WCHAR *path, REGSAM view)
{
    HKEY h = NULL;
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, path, 0, KEY_READ | view, &h) != ERROR_SUCCESS)
        return FALSE;
    RegCloseKey(h);
    return TRUE;
}

static void Cleanup(void)
{
    /* Best effort, both views, both fixtures. A leftover key would make the
     * NEXT boot's rung 1 open an existing key instead of creating one, which
     * still passes -- but a leftover NATIVE DelProbe would make rung 6 pass
     * without proving anything, so leaving them behind is not harmless. */
    RegDeleteKeyExW(HKEY_LOCAL_MACHINE, PROBE_KEY, KEY_WOW64_32KEY, 0);
    RegDeleteKeyExW(HKEY_LOCAL_MACHINE, PROBE_KEY, KEY_WOW64_64KEY, 0);
    RegDeleteKeyExW(HKEY_LOCAL_MACHINE, DEL_KEY, KEY_WOW64_32KEY, 0);
    RegDeleteKeyExW(HKEY_LOCAL_MACHINE, DEL_KEY, KEY_WOW64_64KEY, 0);
}

static DWORD Run(void)
{
    LONG rc;
    DWORD mark = 0;

    Cleanup(); /* start from a known-empty hive regardless of a prior boot */

    /* ---- rungs 1-2: a default-view create must land in Wow6432Node ------- */
    rc = MakeKey(PROBE_KEY, 0, MARK_32);
    if (rc != ERROR_SUCCESS) {
        SayHex("[reg32] rung1 create(default view) failed rc", (DWORD)rc);
        return R32_FAIL_CREATE32;
    }
    Say("[reg32] rung1 default-view create OK\n");

    mark = 0;
    rc = ReadMark(PROBE_IN_NODE, KEY_WOW64_64KEY, &mark);
    if (rc != ERROR_SUCCESS) {
        SayHex("[reg32] rung2 key absent from Wow6432Node rc", (DWORD)rc);
        return R32_FAIL_NOT_IN_NODE;
    }
    if (mark != MARK_32) {
        SayHex("[reg32] rung2 wrong value in Wow6432Node", mark);
        return R32_FAIL_NODE_VALUE;
    }
    Say("[reg32] rung2 redirected into Wow6432Node OK\n");

    /* ---- rung 3: and it must NOT be in the native view ------------------- */
    if (KeyExists(PROBE_KEY, KEY_WOW64_64KEY)) {
        Say("[reg32] rung3 key ALSO in native view - isolation broken\n");
        return R32_FAIL_LEAKED_NATIVE;
    }
    Say("[reg32] rung3 native view clean OK\n");

    /* ---- rung 4: an explicit Wow6432Node path must not splice twice ------
     * Same read as rung 2 but in the DEFAULT view. Without the
     * CmpWow64IsSharedKey exemption this resolves to
     * SOFTWARE\Wow6432Node\Wow6432Node\... and misses. */
    mark = 0;
    rc = ReadMark(PROBE_IN_NODE, 0, &mark);
    if (rc != ERROR_SUCCESS || mark != MARK_32) {
        SayHex("[reg32] rung4 double-splice, rc", (DWORD)rc);
        return R32_FAIL_DOUBLE_SPLICE;
    }
    Say("[reg32] rung4 no double-splice OK\n");

    /* ---- rung 5: a 64-bit-only key still reads from the 32-bit view ------
     * NATIVE_ONLY exists only natively (hivesft.inf seeds SOFTWARE\Wow6432Node
     * EMPTY), so a default-view open must miss and fall back. This rung is the
     * one that fails if someone "fixes" redirection by making a miss an error. */
    {
        HKEY h = NULL;
        WCHAR name[64];
        DWORD type = 0, cb = sizeof(name);
        rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, NATIVE_ONLY, 0, KEY_READ, &h);
        if (rc != ERROR_SUCCESS) {
            SayHex("[reg32] rung5 no fallback to the 64-bit key, rc", (DWORD)rc);
            return R32_FAIL_NO_FALLBACK;
        }
        rc = RegQueryValueExW(h, L"ProductName", NULL, &type, (BYTE *)name, &cb);
        RegCloseKey(h);
        if (rc != ERROR_SUCCESS || type != REG_SZ) {
            SayHex("[reg32] rung5 fallback key has no ProductName, rc", (DWORD)rc);
            return R32_FAIL_FALLBACK_VAL;
        }
        Say("[reg32] rung5 64-bit fallback OK\n");
    }

    /* ---- rung 6: the wrong-hive DELETE oracle ---------------------------
     * Two keys, same path, different views. Delete the 32-bit one EXPLICITLY
     * and require that the native one survives. Before the RegDeleteKeyExW fix
     * the view bits were dropped before NtOpenKey, so this deleted whichever
     * key the process's default view resolved to. */
    if (MakeKey(DEL_KEY, KEY_WOW64_64KEY, MARK_NATIVE) != ERROR_SUCCESS ||
        MakeKey(DEL_KEY, KEY_WOW64_32KEY, MARK_32) != ERROR_SUCCESS) {
        Say("[reg32] rung6 could not build the two-view fixture\n");
        return R32_FAIL_DEL_SETUP;
    }
    /* Prove the fixture is really two distinct keys before trusting the delete:
     * if these collapsed to one key the rung would pass vacuously. */
    mark = 0;
    if (ReadMark(DEL_KEY, KEY_WOW64_64KEY, &mark) != ERROR_SUCCESS || mark != MARK_NATIVE) {
        SayHex("[reg32] rung6 fixture: native side wrong", mark);
        return R32_FAIL_DEL_SETUP;
    }
    mark = 0;
    if (ReadMark(DEL_IN_NODE, KEY_WOW64_64KEY, &mark) != ERROR_SUCCESS || mark != MARK_32) {
        SayHex("[reg32] rung6 fixture: 32-bit side wrong", mark);
        return R32_FAIL_DEL_SETUP;
    }

    rc = RegDeleteKeyExW(HKEY_LOCAL_MACHINE, DEL_KEY, KEY_WOW64_32KEY, 0);
    if (rc != ERROR_SUCCESS) {
        SayHex("[reg32] rung6 delete(KEY_WOW64_32KEY) failed rc", (DWORD)rc);
        return R32_FAIL_DELETE;
    }

    mark = 0;
    if (ReadMark(DEL_KEY, KEY_WOW64_64KEY, &mark) != ERROR_SUCCESS || mark != MARK_NATIVE) {
        Say("[reg32] rung6 THE NATIVE KEY WAS DELETED - wrong hive\n");
        return R32_FAIL_DEL_WRONGHIVE;
    }
    if (KeyExists(DEL_IN_NODE, KEY_WOW64_64KEY)) {
        Say("[reg32] rung6 32-bit copy survived its own delete\n");
        return R32_FAIL_DEL_NOTGONE;
    }
    Say("[reg32] rung6 delete hit the 32-bit view only OK\n");

    Cleanup();
    return R32_PASS;
}

int start(void) /* cdecl: i386 symbol is _start, matching -Wl,-e,_start */
{
    DWORD verdict = Run();
    SayHex("[reg32] verdict", verdict);
    ExitProcess(verdict);
    return (int)verdict;
}
