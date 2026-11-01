/* w11tok.c: the token pseudo-handles NtCurrentProcessToken (-4), NtCurrentThreadToken (-5)
 * and NtCurrentThreadEffectiveToken (-6), plus -3 and -7, through every call that takes a
 * handle, with and without impersonation. Windows 11's answers: w11tok.win11.txt (winref
 * 20261031-224050-vuti1dzc).
 */
#include <windows.h>
#include <stdio.h>
typedef LONG (NTAPI *NQIT)(HANDLE, int, PVOID, ULONG, PULONG);
typedef LONG (NTAPI *NQO)(HANDLE, int, PVOID, ULONG, PULONG);
typedef LONG (NTAPI *NC)(HANDLE);
typedef LONG (NTAPI *NDO)(HANDLE, HANDLE, HANDLE, PHANDLE, ACCESS_MASK, ULONG, ULONG);
typedef LONG (NTAPI *NSIT)(HANDLE, int, PVOID, ULONG);
NQIT nqit; NQO nqo; NC nc; NDO ndo; NSIT nsit;
static void probe(const char *tag) {
  HANDLE h[] = {(HANDLE)(LONG_PTR)-3, (HANDLE)(LONG_PTR)-4, (HANDLE)(LONG_PTR)-5, (HANDLE)(LONG_PTR)-6, (HANDLE)(LONG_PTR)-7};
  for (int i = 0; i < 5; i++) {
    BYTE b[512]; ULONG rl = 0; LONG st = nqit(h[i], 8 /*TokenType*/, b, sizeof b, &rl);
    LONG st2 = nqit(h[i], 7 /*TokenSource*/, b + 64, 64, &rl);
    LONG st3 = nqit(h[i], 1 /*TokenUser*/, b + 128, 256, &rl);
    ULONG oi[14] = {0}; LONG so = nqo(h[i], 0, oi, sizeof oi, &rl);
    HANDLE d = 0; LONG sd = ndo((HANDLE)-1, h[i], (HANDLE)-1, &d, TOKEN_QUERY, 0, 0); if (d) CloseHandle(d);
    HANDLE d2 = 0; LONG sd2 = ndo((HANDLE)-1, h[i], (HANDLE)-1, &d2, 0, 0, 2 /*SAME_ACCESS*/); if (d2) CloseHandle(d2);
    ULONG zero = 0; LONG ss = nsit(h[i], 12 /*TokenSessionId*/, &zero, 4);
    printf("%s h=%d type st=%lx v=%lu src=%lx user=%lx qobj=%lx acc=%lx attr=%lx dupQ=%lx dupSame=%lx set=%lx\n", tag, (int)(LONG_PTR)h[i], st, st ? 0 : *(ULONG*)b, st2, st3, so, oi[1], oi[0], sd, sd2, ss);
  }
  HANDLE dt = 0; SetLastError(0); BOOL r = DuplicateToken((HANDLE)-4, SecurityImpersonation, &dt); printf("%s DuplicateToken(-4) r=%d gle=%lu\n", tag, r, GetLastError()); if (dt) CloseHandle(dt);
  TOKEN_PRIVILEGES tp = {0}; SetLastError(0); r = AdjustTokenPrivileges((HANDLE)-4, FALSE, &tp, 0, NULL, NULL); printf("%s AdjustTokenPrivileges(-4) r=%d gle=%lu\n", tag, r, GetLastError());
}
int main(void) {
  HMODULE nt = GetModuleHandleA("ntdll.dll");
  nqit = (NQIT)GetProcAddress(nt, "NtQueryInformationToken"); nqo = (NQO)GetProcAddress(nt, "NtQueryObject");
  nc = (NC)GetProcAddress(nt, "NtClose"); ndo = (NDO)GetProcAddress(nt, "NtDuplicateObject"); nsit = (NSIT)GetProcAddress(nt, "NtSetInformationToken");
  probe("plain");
  printf("ImpersonateSelf=%d\n", ImpersonateSelf(SecurityIdentification));
  probe("imp");
  RevertToSelf();
  for (int i = 3; i <= 6; i++) printf("NtClose(-%d)=%lx\n", i, nc((HANDLE)(LONG_PTR)-i));
  printf("DONE\n");
  return 0;
}
