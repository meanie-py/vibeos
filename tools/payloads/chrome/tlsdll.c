/* tlsdll: a DLL with implicit TLS (a PE TLS directory), for tlsprobe. `seed` is initialized
 * data in the template; `zeros` sits in the zero-fill tail, which the loader must allocate too. */
#include <windows.h>
static _Thread_local int seed = 42;
static _Thread_local char zeros[4096];
static volatile LONG callbacks;
__declspec(dllexport) int tls_get(void) { return seed; }
__declspec(dllexport) void tls_set(int v) { seed = v; }
__declspec(dllexport) int tls_zeros_clean(void)
{
    int i;
    for (i = 0; i < (int)sizeof(zeros); i++) if (zeros[i]) return 0;
    zeros[0] = zeros[sizeof(zeros) - 1] = 1; /* dirty THIS thread's copy only */
    return 1;
}
__declspec(dllexport) LONG tls_callbacks(void) { return callbacks; }
static void NTAPI on_tls(PVOID h, DWORD reason, PVOID r) { (void)h; (void)r; if (reason == DLL_PROCESS_ATTACH) InterlockedIncrement(&callbacks); }
#ifdef __cplusplus
extern "C"
#endif
__attribute__((section(".CRT$XLB"), used)) PIMAGE_TLS_CALLBACK tlsdll_callback = on_tls;
BOOL WINAPI DllMain(HINSTANCE h, DWORD r, LPVOID p) { (void)h; (void)r; (void)p; return TRUE; }
