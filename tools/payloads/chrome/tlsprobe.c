/* tlsprobe: implicit TLS in a DLL loaded after process start (Vista's rule; NT 5.2 gave such a
 * DLL no index, so it read slot 0, the EXE's block: how Chrome's chrome.dll faulted). Checks the
 * main thread, a thread that already existed at the LoadLibrary, and one made after it, each
 * seeing its own copy of the initialized value and a clean zero-fill tail. Windows' answers are
 * in tlsprobe.win11.txt. */
#include <windows.h>
#include <stdio.h>

typedef int (*get_t)(void); typedef void (*set_t)(int); typedef LONG (*cb_t)(void);
static get_t get; static set_t set; static get_t clean;
static HANDLE go, done;

static void report(const char *who)
{
    int v = get(), z = clean();
    set(v + 1);
    printf("[TLSPROBE] %-14s seed=%d zeros_clean=%d after_set=%d\n", who, v, z, get());
}

static DWORD WINAPI early(LPVOID p) { (void)p; WaitForSingleObject(go, INFINITE); report("early thread"); SetEvent(done); return 0; }
static DWORD WINAPI late(LPVOID p) { (void)p; report("late thread"); return 0; }

int main(int argc, char **argv)
{
    char path[MAX_PATH]; HMODULE m; HANDLE t;
    (void)argc;
    lstrcpynA(path, argv[0], MAX_PATH);
    *strrchr(path, '\\') = 0; lstrcatA(path, "\\tlsdll.dll");
    go = CreateEventA(NULL, TRUE, FALSE, NULL); done = CreateEventA(NULL, TRUE, FALSE, NULL);
    t = CreateThread(NULL, 0, early, NULL, 0, NULL);
    Sleep(200);
    m = LoadLibraryA(path);
    printf("[TLSPROBE] LoadLibrary %s\n", m ? "ok" : "FAILED");
    if (!m) return 1;
    get = (get_t)GetProcAddress(m, "tls_get"); set = (set_t)GetProcAddress(m, "tls_set");
    clean = (get_t)GetProcAddress(m, "tls_zeros_clean");
    printf("[TLSPROBE] tls callbacks run at load: %ld\n", ((cb_t)GetProcAddress(m, "tls_callbacks"))());
    report("main thread");
    SetEvent(go); WaitForSingleObject(done, INFINITE); CloseHandle(t);
    t = CreateThread(NULL, 0, late, NULL, 0, NULL); WaitForSingleObject(t, INFINITE); CloseHandle(t);
    report("main again");
    printf("[TLSPROBE] done\n");
    return 0;
}
