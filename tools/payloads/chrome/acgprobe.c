/* acgprobe: the dynamic-code policy (ACG) as Chrome's browser process uses it.
 * Chrome's sandbox turns on ProhibitDynamicCode + AllowThreadOptOut for its own browser
 * process at start; vibeos refused the call and Chrome died on a CHECK (sandbox quest, step 1).
 * This turns the policy on the same way and checks what it must and must not stop, one line
 * per fact. Windows' answers are the documented ones: ERROR_DYNAMIC_CODE_BLOCKED (1655). */
#include <windows.h>
#include <stdio.h>

#ifndef ERROR_DYNAMIC_CODE_BLOCKED
#define ERROR_DYNAMIC_CODE_BLOCKED 1655
#endif

typedef BOOL (WINAPI *set_mit_fn)(int, PVOID, SIZE_T);
typedef BOOL (WINAPI *get_mit_fn)(HANDLE, int, PVOID, SIZE_T);
typedef BOOL (WINAPI *set_thr_fn)(HANDLE, int, LPVOID, DWORD);

static set_mit_fn set_mit;
static get_mit_fn get_mit;
static set_thr_fn set_thr;
static int fails;

static void check(const char *what, BOOL ok, DWORD err, BOOL want_ok, DWORD want_err)
{
    BOOL pass = want_ok ? ok : (!ok && err == want_err);
    printf("[ACGPROBE] %-52s %s (ok=%d err=%lu)\n", what, pass ? "PASS" : "FAIL", ok, err);
    if (!pass)
        fails++;
}

static DWORD WINAPI opted_out(LPVOID unused)
{
    (void)unused;
    DWORD allow = 1;
    BOOL ok = set_thr(GetCurrentThread(), 2 /* ThreadDynamicCodePolicy */, &allow, sizeof allow);
    check("opt-out thread: SetThreadInformation(DynamicCode=1)", ok, GetLastError(), TRUE, 0);
    void *p = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    check("opt-out thread: VirtualAlloc(PAGE_EXECUTE_READWRITE)", p != NULL, GetLastError(), TRUE, 0);
    if (p)
        VirtualFree(p, 0, MEM_RELEASE);
    return 0;
}

int main(void)
{
    HMODULE k32 = GetModuleHandleA("kernel32.dll");
    set_mit = (set_mit_fn)GetProcAddress(k32, "SetProcessMitigationPolicy");
    get_mit = (get_mit_fn)GetProcAddress(k32, "GetProcessMitigationPolicy");
    set_thr = (set_thr_fn)GetProcAddress(k32, "SetThreadInformation");
    if (!set_mit || !get_mit || !set_thr) {
        printf("[ACGPROBE] missing exports\n[ACGPROBE] done VERDICT=FAIL\n");
        return 1;
    }

    /* Before: executable memory is allowed. */
    void *before = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    check("before ACG: VirtualAlloc(PAGE_EXECUTE_READWRITE)", before != NULL, GetLastError(), TRUE, 0);
    void *heap = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);

    DWORD flags = 0x1 | 0x2; /* ProhibitDynamicCode | AllowThreadOptOut */
    BOOL ok = set_mit(2 /* ProcessDynamicCodePolicy */, &flags, sizeof flags);
    check("SetProcessMitigationPolicy(DynamicCode, prohibit|optout)", ok, GetLastError(), TRUE, 0);

    DWORD got = 0;
    ok = get_mit(GetCurrentProcess(), 2, &got, sizeof got);
    printf("[ACGPROBE] GetProcessMitigationPolicy(DynamicCode) flags=0x%lx\n", got);
    check("GetProcessMitigationPolicy reports prohibit|optout", ok && (got & 3) == 3, GetLastError(), TRUE, 0);

    void *p = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    check("VirtualAlloc(PAGE_EXECUTE_READWRITE) blocked", p != NULL, GetLastError(), FALSE, ERROR_DYNAMIC_CODE_BLOCKED);
    p = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
    check("VirtualAlloc(PAGE_READWRITE) still allowed", p != NULL, GetLastError(), TRUE, 0);
    DWORD old;
    ok = heap ? VirtualProtect(heap, 4096, PAGE_EXECUTE_READ, &old) : FALSE;
    check("VirtualProtect(heap page -> PAGE_EXECUTE_READ) blocked", ok, GetLastError(), FALSE, ERROR_DYNAMIC_CODE_BLOCKED);
    ok = before ? VirtualProtect(before, 4096, PAGE_READWRITE, &old) : FALSE;
    check("VirtualProtect(exec page -> PAGE_READWRITE) allowed", ok, GetLastError(), TRUE, 0);
    HMODULE dll = LoadLibraryA("version.dll");
    check("LoadLibrary(version.dll) after ACG", dll != NULL, GetLastError(), TRUE, 0);

    flags = 0;
    ok = set_mit(2, &flags, sizeof flags);
    check("turning ACG back off is refused", ok, GetLastError(), FALSE, ERROR_ACCESS_DENIED);

    DWORD allow = 1;
    ok = set_thr(GetCurrentThread(), 2, &allow, sizeof allow);
    printf("[ACGPROBE] main thread opt-out ok=%d err=%lu\n", ok, GetLastError());
    allow = 0;
    set_thr(GetCurrentThread(), 2, &allow, sizeof allow);

    HANDLE t = CreateThread(NULL, 0, opted_out, NULL, 0, NULL);
    if (t) {
        WaitForSingleObject(t, 10000);
        CloseHandle(t);
    }
    p = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READ);
    check("main thread (not opted out) still blocked", p != NULL, GetLastError(), FALSE, ERROR_DYNAMIC_CODE_BLOCKED);

    printf("[ACGPROBE] done VERDICT=%s\n", fails ? "FAIL" : "PASS");
    return fails ? 1 : 0;
}
