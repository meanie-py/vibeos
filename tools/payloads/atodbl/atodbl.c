/* atodbl -- why Wine's msvcrt string.c:3522 fails 5,535 times on i386: not _atodbl (exact
 * arithmetic on every logged pair says it returns the correctly rounded double), but the
 * CPU. The smoke backend's QEMU (11.0.50, a development snapshot) mis-executes SAHF after a
 * logic op: CF comes out wrong, so `fnstsw; sahf; jbe` -- how -march=pentium compiles every
 * x87 floating-point comparison -- branches the wrong way, and the test's compare_double()
 * calls two identical doubles 2^63 ulps apart. Three checks, each with its native result:
 *   1. SAHF after each integer op class, pure integer code (no x87, no system call);
 *   2. compare_double as the tree compiles it, run from its raw bytes;
 *   3. the test's own loop against msvcrt.dll's _atodbl.
 * tools/payloads/atodbl/build.sh uses the tree's i386 C flags, so (2) and (3) see the same
 * codegen as msvcrt_winetest. */
#include <windows.h>
#include <float.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
#include "sahf.h"

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[ATODBL] " __VA_ARGS__)
static int g_fail;

/* compare_double compiled by llvm-mingw with the tree's i386 flags (-march=pentium -O1
 * -mstackrealign ...): raw bytes, position-independent, no relocations. On the host CPU:
 * cmp(1,1,2)=1 cmp(1e-307,1e-307,2)=1 cmp(1,-1,2)=0. */
static const unsigned char cmp_blob[] = {
#include "cmp_blob.inc"
};

static inline BOOL compare_double(double f, double g, unsigned int ulps)
{
    ULONGLONG x = *(ULONGLONG *)&f;
    ULONGLONG y = *(ULONGLONG *)&g;

    if (f < 0)
        x = ~x + 1;
    else
        x |= ((ULONGLONG)1)<<63;
    if (g < 0)
        y = ~y + 1;
    else
        y |= ((ULONGLONG)1)<<63;

    return (x>y ? x-y : y-x) <= ulps;
}

static double mul_pow10(double x, double exp)
{
    int fpexcept = _EM_DENORMAL|_EM_INVALID|_EM_ZERODIVIDE|_EM_OVERFLOW|_EM_UNDERFLOW|_EM_INEXACT;
    BOOL negexp = (exp < 0);
    int fpcontrol;
    double ret;

    if(negexp)
        exp = -exp;
    fpcontrol = _control87(0, 0);
    _control87(fpexcept, 0xffffffff);
    ret = pow(10.0, exp);
    ret = (negexp ? x/ret : x*ret);
    _control87(fpcontrol, 0xffffffff);
    return ret;
}

int WINAPI WinMain(HINSTANCE a, HINSTANCE b, LPSTR c, int d)
{
    (void)a; (void)b; (void)c; (void)d;

    char line[256];
    int n = 0, bad = 0;
    for (int i = 0; i < (int)(sizeof sahfv / sizeof sahfv[0]); i++) {
        unsigned r = sahfv[i]();
        bad += r != 1;
        n += wsprintfA(line + n, "%s=%u ", sahfn[i], r);
    }
    SAY("1. jbe after sahf(AH=0x29) following: %s(native: all 1)\n", line);
    if (bad) { g_fail++; SAY("FAIL: SAHF set CF wrong after %d op classes -- the emulator, not vibeos\n", bad); }

    void *m = VirtualAlloc(NULL, 4096, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    memcpy(m, cmp_blob, sizeof cmp_blob);
    int (__cdecl *cmp)(double, double, unsigned) = (int (__cdecl *)(double, double, unsigned))m;
    int r1 = cmp(1.0, 1.0, 2), r2 = cmp(1e-307, 1e-307, 2), r3 = cmp(1.0, -1.0, 2);
    SAY("2. compiled compare_double: %d %d %d (native: 1 1 0)\n", r1, r2, r3);
    if (r1 != 1 || r2 != 1 || r3 != 0) g_fail++;

    /* Only the symptom is countable here: mul_pow10's own `exp < 0` is an x87 compare too,
     * so under the broken SAHF the EXPECTED side is sometimes x*10^j instead of x/10^j. That
     * _atodbl itself is right was shown off-guest: every one of the 5,535 pairs the winetest
     * logged parses to the correctly rounded double (2026-10-04).
     * Not 0 on a correct CPU: this binary's pow() is mingw's static x87 one, a few ulps off near
     * 1e-307, so 1,823 cases fail on real Windows with the same bits as in the guest under WHPX
     * and QEMU 11.1.0 (2026-10-18). The count is then a pure function of CPU semantics -- the
     * winetest itself calls msvcrt.dll's pow -- and the check is that it matches native. */
#define NATIVE_FAILS 1823
    int fails = 0, cases = 0;
    for (int j = DBL_MIN_10_EXP; j <= DBL_MAX_10_EXP; j++) {
        for (int i = 1; i <= 9; i++) {
            double expected = mul_pow10(i, j);
            if (expected < DBL_MIN || expected > DBL_MAX) continue;
            char num[32];
            _CRT_DOUBLE dd;
            snprintf(num, sizeof(num), "%de%d", i, j);
            _atodbl(&dd, num);
            cases++;
            fails += !compare_double(dd.x, expected, 2);
        }
    }
    SAY("3. string.c:3522 loop: %d cases, compare_double fails %d (native: %d)\n", cases, fails,
        NATIVE_FAILS);
    if (fails != NATIVE_FAILS) g_fail++;

    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
