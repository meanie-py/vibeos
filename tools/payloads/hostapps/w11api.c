/* w11api.c: Windows 10/11 user-mode APIs as Windows 11 answers them, to diff against winref.
 * AppPolicyGet* (all eight, five token values), FlsGetValue2, GetFileInformationByName (each
 * class, buffer sizes, missing/relative/\\?\ paths) and NtQueryInformationByName's class set.
 * Usage: w11api.exe [file] [dir]; Windows' answers for notepad.exe / C:\Windows are in
 * w11api.win11.txt (winref 20261031-224050-vuti1dzc). Hex dumps differ by file; compare shape.
 */
#include <windows.h>
#include <stdio.h>
typedef LONG (WINAPI *AP)(HANDLE, int *);
typedef PVOID (WINAPI *FG2)(DWORD);
typedef BOOL (WINAPI *GFIBN)(PCWSTR, int, PVOID, ULONG);
typedef LONG (NTAPI *NQIBN)(PVOID, PVOID, PVOID, ULONG, int);
typedef struct { USHORT Length, MaximumLength; PWSTR Buffer; } US;
typedef struct { ULONG Length; HANDLE Root; US *Name; ULONG Attr; PVOID Sd, Sqos; } OA;
typedef struct { LONG Status; ULONG_PTR Info; } IOSB;
static void dumpbuf(const char *tag, BYTE *b, ULONG n) { printf("%s", tag); for (ULONG i = 0; i < n; i++) printf("%02x", b[i]); printf("\n"); }
int wmain(int argc, wchar_t **argv) {
  PCWSTR file = argc > 1 ? argv[1] : L"C:\\Windows\\notepad.exe", dir = argc > 2 ? argv[2] : L"C:\\Windows";
  HMODULE k32 = GetModuleHandleA("kernel32.dll"), kb = LoadLibraryA("kernelbase.dll"), nt = GetModuleHandleA("ntdll.dll");
  const char *ap[] = {"AppPolicyGetClrCompat","AppPolicyGetCreateFileAccess","AppPolicyGetLifecycleManagement",
    "AppPolicyGetMediaFoundationCodecLoading","AppPolicyGetProcessTerminationMethod","AppPolicyGetShowDeveloperDiagnostic",
    "AppPolicyGetThreadInitializationType","AppPolicyGetWindowingModel"};
  HANDLE toks[] = {(HANDLE)(LONG_PTR)-6, (HANDLE)(LONG_PTR)-4, (HANDLE)(LONG_PTR)-5, NULL, (HANDLE)(LONG_PTR)0x1234};
  for (int i = 0; i < 8; i++) {
    AP f = (AP)GetProcAddress(kb, ap[i]); AP g = (AP)GetProcAddress(k32, ap[i]);
    printf("%s kb=%d k32=%d\n", ap[i], f != NULL, g != NULL);
    if (!f) continue;
    for (int t = 0; t < 5; t++) { int v = 0x5555; SetLastError(0xdead); LONG r = f(toks[t], &v); printf("  tok=%Id rc=%ld v=%d gle=%lx\n", (LONG_PTR)toks[t], r, v, GetLastError()); }
  }
  /* FlsGetValue2 */
  FG2 fg2 = (FG2)GetProcAddress(k32, "FlsGetValue2"); FG2 fg2b = (FG2)GetProcAddress(kb, "FlsGetValue2");
  printf("FlsGetValue2 k32=%d kb=%d RtlFlsGetValue2=%d RtlFlsGetValue=%d\n", fg2 != NULL, fg2b != NULL,
    GetProcAddress(nt, "RtlFlsGetValue2") != NULL, GetProcAddress(nt, "RtlFlsGetValue") != NULL);
  if (fg2) {
    DWORD i = FlsAlloc(NULL); FlsSetValue(i, (PVOID)0x1234);
    SetLastError(0xdead); PVOID v = fg2(i); printf("  idx=%lu v=%p gle=%lx\n", i, v, GetLastError());
    DWORD j = FlsAlloc(NULL); SetLastError(0xdead); v = fg2(j); printf("  unset idx=%lu v=%p gle=%lx\n", j, v, GetLastError());
    FlsFree(j); SetLastError(0xdead); v = fg2(j); printf("  freed v=%p gle=%lx\n", v, GetLastError());
    SetLastError(0xdead); v = FlsGetValue(j); printf("  FlsGetValue(freed) v=%p gle=%lx\n", v, GetLastError());
    fflush(stdout);
  }
  /* GetFileInformationByName */
  GFIBN gf = (GFIBN)GetProcAddress(k32, "GetFileInformationByName"); GFIBN gfb = (GFIBN)GetProcAddress(kb, "GetFileInformationByName");
  printf("GetFileInformationByName k32=%d kb=%d\n", gf != NULL, gfb != NULL);
  if (!gf) gf = gfb;
  if (gf) {
    WCHAR longp[MAX_PATH + 8]; _snwprintf(longp, MAX_PATH + 8, L"\\\\?\\%ls", file);
    PCWSTR names[] = {file, dir, L"C:\\nonexistent.xyz", L"no-such-relative.exe", longp};
    for (int n = 0; n < 5; n++) for (int c = 0; c < 6; c++) {
      BYTE b[256]; memset(b, 0xcc, sizeof b); SetLastError(0xdead); BOOL r = gf(names[n], c, b, sizeof b);
      printf("  n=%d c=%d r=%d gle=%lu\n", n, c, r, GetLastError()); if (r && n < 2) dumpbuf("    ", b, c == 2 ? 8 : 112);
    }
    ULONG sizes[] = {0, 4, 47, 48, 72, 111, 112, 120};
    for (int s = 0; s < 8; s++) { BYTE b[256]; SetLastError(0xdead); BOOL r = gf(names[0], 0, b, sizes[s]); printf("  size=%lu r=%d gle=%lu\n", sizes[s], r, GetLastError()); }
    for (int c = 0; c < 4; c++) { SetLastError(0xdead); BOOL r = gf(names[0], c, NULL, 0); printf("  null c=%d r=%d gle=%lu\n", c, r, GetLastError()); }
  }
  NQIBN nq = (NQIBN)GetProcAddress(nt, "NtQueryInformationByName");
  printf("NtQueryInformationByName=%d\n", nq != NULL);
  if (nq) {
    WCHAR p[MAX_PATH + 8]; _snwprintf(p, MAX_PATH + 8, L"\\??\\%ls", file); US u = {(USHORT)(wcslen(p) * 2), (USHORT)(wcslen(p) * 2 + 2), p}; OA oa = {sizeof oa, 0, &u, 0x40, 0, 0};
    for (int c = 1; c < 80; c++) { BYTE b[1024]; IOSB io = {0x5555, 0x5555}; LONG st = nq(&oa, &io, b, sizeof b, c);
      if (st != (LONG)0xC0000003) printf("  class=%d st=%lx info=%Iu\n", c, st, io.Info); }
  }
  printf("DONE\n");
  return 0;
}
