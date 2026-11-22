/* regcopyprobe — RegCopyTreeW over SYSTEM\ControlSet001 (see payload.yaml), bisected on error.
 * Verdict: "[REGCOPYPROBE] done PASS|FAIL". */
#include <windows.h>
#include <stdio.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[REGCOPYPROBE] " __VA_ARGS__)

static const WCHAR SRC[] = L"SYSTEM\\ControlSet001";
static const WCHAR SCRATCH[] = L"SYSTEM\\RegCopyProbe";

/* Copy src (under HKLM) into a fresh scratch key; the status RegCopyTreeW returned. */
static LONG copy_once(const WCHAR *src)
{
    HKEY s, d;
    LONG rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, src, 0, KEY_READ, &s);
    if (rc != ERROR_SUCCESS) {
        SAY("open %ls -> %ld\n", src, rc);
        return rc;
    }
    RegDeleteTreeW(HKEY_LOCAL_MACHINE, SCRATCH);
    rc = RegCreateKeyExW(HKEY_LOCAL_MACHINE, SCRATCH, 0, NULL, REG_OPTION_NON_VOLATILE, KEY_WRITE, NULL, &d, NULL);
    if (rc != ERROR_SUCCESS) {
        SAY("create scratch -> %ld\n", rc);
        RegCloseKey(s);
        return rc;
    }
    rc = RegCopyTreeW(s, NULL, d);
    RegCloseKey(d);
    RegCloseKey(s);
    RegDeleteTreeW(HKEY_LOCAL_MACHINE, SCRATCH);
    return rc;
}

/* src fails as a whole: find a failing child and descend; if none fails, src itself (its
 * values, or a child's create) is the culprit. */
static void bisect(const WCHAR *src, LONG rc, int depth)
{
    HKEY k;
    if (depth > 24 || RegOpenKeyExW(HKEY_LOCAL_MACHINE, src, 0, KEY_READ, &k) != ERROR_SUCCESS) {
        SAY("culprit %ls (rc=%ld)\n", src, rc);
        return;
    }
    WCHAR name[256], child[1024];
    for (DWORD i = 0;; i++) {
        DWORD n = ARRAYSIZE(name);
        if (RegEnumKeyExW(k, i, name, &n, NULL, NULL, NULL, NULL) != ERROR_SUCCESS)
            break;
        _snwprintf(child, ARRAYSIZE(child), L"%ls\\%ls", src, name);
        child[ARRAYSIZE(child) - 1] = 0;
        LONG crc = copy_once(child);
        if (crc != ERROR_SUCCESS) {
            SAY("fails: %ls -> %ld\n", child, crc);
            RegCloseKey(k);
            bisect(child, crc, depth + 1);
            return;
        }
    }
    RegCloseKey(k);
    /* Every child copies alone: the key's own values, or creating a child key, fail. */
    DWORD values = 0, maxname = 0, maxdata = 0, subkeys = 0, maxclass = 0;
    if (RegOpenKeyExW(HKEY_LOCAL_MACHINE, src, 0, KEY_READ, &k) == ERROR_SUCCESS) {
        RegQueryInfoKeyW(k, NULL, NULL, NULL, &subkeys, NULL, &maxclass, &values, &maxname, &maxdata, NULL, NULL);
        RegCloseKey(k);
    }
    SAY("culprit %ls (rc=%ld): %lu subkeys (max class %lu), %lu values (max name %lu, max data %lu)\n",
        src, rc, subkeys, maxclass, values, maxname, maxdata);
}

int main(void)
{
    LONG rc = copy_once(SRC);
    SAY("RegCopyTreeW(%ls) -> %ld\n", SRC, rc);
    if (rc != ERROR_SUCCESS)
        bisect(SRC, rc, 0);
    SAY("done %s\n", rc == ERROR_SUCCESS ? "PASS" : "FAIL");
    return rc != ERROR_SUCCESS;
}
