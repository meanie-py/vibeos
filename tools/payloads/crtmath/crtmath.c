/* crtmath — the i386 math entry points every CRT DLL exports, called through each DLL's
 * export table with their real register conventions.
 *
 * `_ftol`/`_ftol2`/`_ftol2_sse` take st0 and return edx:eax; the sixteen `_CI*` take
 * their argument(s) on the x87 stack and must leave the rest of that stack intact; the
 * `__libm_sse2_*` wrappers take xmm0 (xmm1) and return xmm0. No C compiler emits those
 * calls portably, so normal tests never reach them — MSVC /arch:SSE2 and /Oi code does.
 * All five CRT DLLs export them (msvcrt, msvcrt20, msvcrt40, crtdll, ucrtbase), from one
 * Rust copy in crt-shared-rust's crtmath_i386 module.
 *
 * Expected values are the same DLL's own plain export (sin, pow, ...) where one exists,
 * so this grades the thunk, not the libm under it. Also checked: msvcrt40's `_HUGE` and
 * crtdll's `_HUGE_dll` (now MSVCRT__HUGE), and ucrtbase's _chgsign, _controlfp_s,
 * exp2f and nexttowardf, which moved from crtmath's C into ucrt-rust/cnt-rust.
 */
#include <windows.h>
#include <stdio.h>
#include <string.h>
#define sprintf_s _snprintf

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[CRTMATH] " __VA_ARGS__)

static int g_fail, g_checks;

static void check(const char *dll, const char *what, int ok)
{
    g_checks++;
    if (!ok)
        g_fail++;
    if (!ok)
        SAY("%s %s FAIL\n", dll, what);
}

static int same_bits(double a, double b) { return memcmp(&a, &b, sizeof a) == 0; }
static int same_bitsf(float a, float b) { return memcmp(&a, &b, sizeof a) == 0; }

/* st0 = x; call f; edx:eax */
static long long call_ftol(void *f, double x)
{
    long long r;
    __asm__ volatile("fldl %1\n\tcall *%2" : "=A"(r) : "m"(x), "r"(f) : "ecx", "memory", "st");
    return r;
}

/* x87 stack: sentinel, then the argument(s); call f; pop the result and the sentinel.
 * A thunk that loses or reorders the caller's x87 stack shows up in the sentinel. */
static double call_ci1(void *f, double x, double *sentinel_out)
{
    double r, s, sent = 12345.5;
    __asm__ volatile("fldl %2\n\tfldl %3\n\tcall *%4\n\tfstpl %0\n\tfstpl %1"
                     : "=m"(r), "=m"(s) : "m"(sent), "m"(x), "r"(f) : "eax", "ecx", "edx", "memory");
    *sentinel_out = s;
    return r;
}
static double call_ci2(void *f, double x, double y, double *sentinel_out)
{
    double r, s, sent = 12345.5;
    __asm__ volatile("fldl %2\n\tfldl %3\n\tfldl %4\n\tcall *%5\n\tfstpl %0\n\tfstpl %1"
                     : "=m"(r), "=m"(s) : "m"(sent), "m"(x), "m"(y), "r"(f) : "eax", "ecx", "edx", "memory");
    *sentinel_out = s;
    return r;
}

/* xmm0 (xmm1) in, xmm0 out */
static double call_sse_d1(void *f, double x)
{
    double r;
    __asm__ volatile("movsd %1, %%xmm0\n\tcall *%2\n\tmovsd %%xmm0, %0"
                     : "=m"(r) : "m"(x), "r"(f) : "eax", "ecx", "edx", "xmm0", "xmm1", "memory", "st");
    return r;
}
static double call_sse_d2(void *f, double x, double y)
{
    double r;
    __asm__ volatile("movsd %1, %%xmm0\n\tmovsd %2, %%xmm1\n\tcall *%3\n\tmovsd %%xmm0, %0"
                     : "=m"(r) : "m"(x), "m"(y), "r"(f) : "eax", "ecx", "edx", "xmm0", "xmm1", "memory", "st");
    return r;
}
static float call_sse_f1(void *f, float x)
{
    float r;
    __asm__ volatile("movss %1, %%xmm0\n\tcall *%2\n\tmovss %%xmm0, %0"
                     : "=m"(r) : "m"(x), "r"(f) : "eax", "ecx", "edx", "xmm0", "xmm1", "memory", "st");
    return r;
}
static float call_sse_f2(void *f, float x, float y)
{
    float r;
    __asm__ volatile("movss %1, %%xmm0\n\tmovss %2, %%xmm1\n\tcall *%3\n\tmovss %%xmm0, %0"
                     : "=m"(r) : "m"(x), "m"(y), "r"(f) : "eax", "ecx", "edx", "xmm0", "xmm1", "memory", "st");
    return r;
}

typedef double(__cdecl *d1_t)(double);
typedef double(__cdecl *d2_t)(double, double);
typedef float(__cdecl *f2_t)(float, float);

#define GET(name) ((void *)GetProcAddress(h, name))

static const char *const CI1[] = {"acos", "asin", "atan", "cos", "cosh", "exp", "log", "log10", "sin", "sinh", "sqrt", "tan", "tanh"};
static const char *const CI2[] = {"atan2", "fmod", "pow"};
static const char *const SSE1[] = {"acos", "asin", "atan", "cos", "exp", "log", "log10", "sin", "tan"};
static const char *const PREC1[] = {"acos", "asin", "atan", "cos", "exp", "log", "log10", "sin", "sqrt", "tan"};

static void probe(const char *dll)
{
    HMODULE h = LoadLibraryA(dll);
    char name[64], what[96];
    double s;
    unsigned i;
    int n0 = g_checks, f0 = g_fail;

    if (!h) {
        check(dll, "LoadLibrary", 0);
        return;
    }

    /* _ftol family: truncation toward zero, 64-bit result, rounding mode restored. */
    {
        static const char *const ftols[] = {"_ftol", "_ftol2", "_ftol2_sse"};
        for (i = 0; i < 3; i++) {
            void *f = GET(ftols[i]);
            if (!f) {
                /* msvcrt20/msvcrt40/crtdll export only _ftol, as the real ones do. */
                if (i == 0)
                    check(dll, ftols[i], 0);
                continue;
            }
            sprintf_s(what, sizeof what, "%s(2.9)", ftols[i]);
            check(dll, what, call_ftol(f, 2.9) == 2);
            sprintf_s(what, sizeof what, "%s(-2.9)", ftols[i]);
            check(dll, what, call_ftol(f, -2.9) == -2);
            sprintf_s(what, sizeof what, "%s(1e10)", ftols[i]);
            check(dll, what, call_ftol(f, 1e10) == 10000000000LL);
        }
        /* rounding mode survives: 2.5 rounds to 2 under the default nearest-even */
        {
            double half = 2.5, r;
            __asm__ volatile("fldl %1\n\tfrndint\n\tfstpl %0" : "=m"(r) : "m"(half));
            check(dll, "_ftol leaves RC at nearest", r == 2.0);
        }
    }

    /* _CI*: compare against the DLL's own plain export, and check the sentinel. */
    for (i = 0; i < sizeof CI1 / sizeof *CI1; i++) {
        void *f;
        d1_t plain = (d1_t)GET(CI1[i]);
        sprintf_s(name, sizeof name, "_CI%s", CI1[i]);
        f = GET(name);
        if (!f || !plain) {
            check(dll, name, 0);
            continue;
        }
        sprintf_s(what, sizeof what, "%s(0.5)", name);
        check(dll, what, same_bits(call_ci1(f, 0.5, &s), plain(0.5)));
        sprintf_s(what, sizeof what, "%s keeps the x87 stack", name);
        check(dll, what, s == 12345.5);
    }
    for (i = 0; i < sizeof CI2 / sizeof *CI2; i++) {
        void *f;
        d2_t plain = (d2_t)GET(CI2[i]);
        sprintf_s(name, sizeof name, "_CI%s", CI2[i]);
        f = GET(name);
        if (!f || !plain) {
            check(dll, name, 0);
            continue;
        }
        sprintf_s(what, sizeof what, "%s(7, 3) arg order", name);
        check(dll, what, same_bits(call_ci2(f, 7.0, 3.0, &s), plain(7.0, 3.0)));
        sprintf_s(what, sizeof what, "%s keeps the x87 stack", name);
        check(dll, what, s == 12345.5);
    }

    /* __libm_sse2_*: only msvcrt and ucrtbase export them. */
    if (!GET("__libm_sse2_sin"))
        goto done;
    for (i = 0; i < sizeof SSE1 / sizeof *SSE1; i++) {
        void *f, *ff;
        d1_t plain = (d1_t)GET(SSE1[i]);
        sprintf_s(name, sizeof name, "__libm_sse2_%s", SSE1[i]);
        f = GET(name);
        sprintf_s(name, sizeof name, "__libm_sse2_%sf", SSE1[i]);
        ff = GET(name);
        if (!f || !ff || !plain) {
            check(dll, name, 0);
            continue;
        }
        sprintf_s(what, sizeof what, "__libm_sse2_%s(0.5)", SSE1[i]);
        check(dll, what, same_bits(call_sse_d1(f, 0.5), plain(0.5)));
        sprintf_s(what, sizeof what, "__libm_sse2_%sf(0.5f)", SSE1[i]);
        check(dll, what, same_bitsf(call_sse_f1(ff, 0.5f), (float)plain(0.5)));
    }
    {
        d2_t patan2 = (d2_t)GET("atan2"), ppow = (d2_t)GET("pow");
        f2_t ppowf = (f2_t)GET("powf");
        void *f;
        if ((f = GET("__libm_sse2_atan2")) && patan2)
            check(dll, "__libm_sse2_atan2(1, 2)", same_bits(call_sse_d2(f, 1.0, 2.0), patan2(1.0, 2.0)));
        else
            check(dll, "__libm_sse2_atan2", 0);
        if ((f = GET("__libm_sse2_pow")) && ppow)
            check(dll, "__libm_sse2_pow(2, 10)", call_sse_d2(f, 2.0, 10.0) == 1024.0);
        else
            check(dll, "__libm_sse2_pow", 0);
        f = GET("__libm_sse2_powf");
        if (f)
            check(dll, "__libm_sse2_powf(2, 10)", call_sse_f2(f, 2.0f, 10.0f) == 1024.0f);
        else
            check(dll, "__libm_sse2_powf", 0);
        if (ppowf && f)
            check(dll, "__libm_sse2_powf(1.5, 2.5) == powf", same_bitsf(call_sse_f2(f, 1.5f, 2.5f), ppowf(1.5f, 2.5f)));
    }
    for (i = 0; i < sizeof PREC1 / sizeof *PREC1; i++) {
        void *f;
        d1_t plain = (d1_t)GET(PREC1[i]);
        sprintf_s(name, sizeof name, "_libm_sse2_%s_precise", PREC1[i]);
        f = GET(name);
        if (!f)
            continue; /* msvcrt.dll does not export the _precise set; ucrtbase does */
        sprintf_s(what, sizeof what, "%s(0.5)", name);
        check(dll, what, plain && same_bits(call_sse_d1(f, 0.5), plain(0.5)));
    }
    {
        void *f = GET("_libm_sse2_pow_precise");
        if (f)
            check(dll, "_libm_sse2_pow_precise(2, 10)", call_sse_d2(f, 2.0, 10.0) == 1024.0);
    }

done:
    SAY("%s: %d checks, %d failed\n", dll, g_checks - n0, g_fail - f0);
}

static void __cdecl quiet_handler(const wchar_t *e, const wchar_t *f, const wchar_t *file, unsigned line, uintptr_t r)
{
    (void)e; (void)f; (void)file; (void)line; (void)r;
}

static void probe_ucrt_leftovers(void)
{
    HMODULE h = LoadLibraryA("ucrtbase.dll");
    typedef double(__cdecl * chg_t)(double);
    typedef int(__cdecl * cfs_t)(unsigned *, unsigned, unsigned);
    typedef float(__cdecl * e2f_t)(float);
    typedef float(__cdecl * ntf_t)(float, double);
    typedef int *(__cdecl * errno_t_)(void);
    typedef void *(__cdecl * setiph_t)(void *);
    chg_t chg = (chg_t)GET("_chgsign");
    cfs_t cfs = (cfs_t)GET("_controlfp_s");
    e2f_t e2f = (e2f_t)GET("exp2f");
    ntf_t ntf = (ntf_t)GET("nexttowardf");
    errno_t_ perr = (errno_t_)GET("_errno");
    setiph_t setiph = (setiph_t)GET("_set_invalid_parameter_handler");
    const char *d = "ucrtbase.dll";
    int n0 = g_checks, f0 = g_fail;

    if (!chg || !cfs || !e2f || !ntf || !perr || !setiph) {
        check(d, "leftover exports resolve", 0);
        return;
    }
    check(d, "_chgsign(1.5)", chg(1.5) == -1.5);
    check(d, "_chgsign(-0.0) is +0", same_bits(chg(-0.0), 0.0));
    {
        unsigned cur = 0xdeadbeef, cur2 = 0;
        void *old = setiph((void *)quiet_handler);
        int r;
        *perr() = 0;
        r = cfs(&cur, 0x80000000u, 0xffffffffu);
        check(d, "_controlfp_s(bad) = EINVAL", r == 22 && *perr() == 22);
        check(d, "_controlfp_s(bad) still reports cur", cur != 0xdeadbeef);
        r = cfs(&cur2, 0, 0);
        check(d, "_controlfp_s(0, 0) = 0, same word", r == 0 && cur2 == cur);
        setiph(old);
    }
    check(d, "exp2f(10) = 1024", e2f(10.0f) == 1024.0f);
    check(d, "exp2f(-1) = 0.5", e2f(-1.0f) == 0.5f);
    *perr() = 0;
    {
        float r = e2f(200.0f);
        check(d, "exp2f(200) = inf, ERANGE", r > 3.4e38f && *perr() == 34);
    }
    {
        float r = ntf(1.0f, 2.0);
        unsigned bits;
        memcpy(&bits, &r, 4);
        check(d, "nexttowardf(1, 2.0) = 0x3f800001", bits == 0x3f800001u);
        r = ntf(1.0f, 0.0);
        memcpy(&bits, &r, 4);
        check(d, "nexttowardf(1, 0.0) = 0x3f7fffff", bits == 0x3f7fffffu);
    }
    SAY("ucrtbase.dll leftovers: %d checks, %d failed\n", g_checks - n0, g_fail - f0);
}

static void probe_huge(const char *dll, const char *sym)
{
    HMODULE h = LoadLibraryA(dll);
    double *p = h ? (double *)GET(sym) : NULL;
    char what[64];
    sprintf_s(what, sizeof what, "%s = +inf", sym);
    check(dll, what, p && *p > 1.7e308 && *p == *p);
}

int __cdecl main(void)
{
    static const char *const dlls[] = {"msvcrt.dll", "msvcrt20.dll", "msvcrt40.dll", "crtdll.dll", "ucrtbase.dll"};
    unsigned i;
    for (i = 0; i < 5; i++)
        probe(dlls[i]);
    probe_ucrt_leftovers();
    probe_huge("msvcrt.dll", "_HUGE");
    probe_huge("msvcrt40.dll", "_HUGE");
    probe_huge("crtdll.dll", "_HUGE_dll");
    SAY("%d checks, %d failed\n", g_checks, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return 0;
}
