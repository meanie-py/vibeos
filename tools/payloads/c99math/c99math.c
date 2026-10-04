/* c99math — ucrtbase's C99 math functions (ucrt-rust c99math.rs over the `libm`
 * crate), graded value + errno in the guest.
 *
 * Until 2026-10-03 every one of these was a ucrtbase spec stub returning garbage.
 * cases.h is the table: values from the host's glibc libm within a stated ulp
 * budget (two independent libms agreeing), or an exact value where the Windows
 * contract differs from glibc's; errno is the UCRT contract (Wine's musl patch
 * points, and Wine's native-verified msvcr120 remainder/_dpcomp/_Cbuild tables).
 * Everything comes from ucrtbase.dll by GetProcAddress -- including _errno, which
 * is NOT msvcrt.dll's errno -- so no static mingw libm can answer in its place.
 */
#include <windows.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[C99MATH] " __VA_ARGS__)

#include "expected.h"

enum kind { KD, KD2, KF, KF2, KI32D, KI64D, KI32F, KI64F };
struct row { const char *fn; enum kind k; double x, y; int err, ulps; };

static const struct row rows[] = {
#undef D
#undef D2
#undef F
#undef F2
#define D(fn, x, e, u) { #fn, KD, x, 0, e, u },
#define D2(fn, x, y, e, u) { #fn, KD2, x, y, e, u },
#define F(fn, x, e, u) { #fn, KF, x, 0, e, u },
#define F2(fn, x, y, e, u) { #fn, KF2, x, y, e, u },
#define DX(fn, x, v, e) { #fn, KD, x, 0, e, 0 },
#define D2X(fn, x, y, v, e) { #fn, KD2, x, y, e, 0 },
#define FX(fn, x, v, e) { #fn, KF, x, 0, e, 0 },
#define I32(fn, x, v, e) { #fn, KI32D, x, 0, e, 0 },
#define I64(fn, x, v, e) { #fn, KI64D, x, 0, e, 0 },
#define IF32(fn, x, v, e) { #fn, KI32F, x, 0, e, 0 },
#define IF64(fn, x, v, e) { #fn, KI64F, x, 0, e, 0 },
#include "cases.h"
};

static HMODULE h;
static int *(__cdecl *p_errno)(void);
static unsigned (__cdecl *p__clearfp)(void);
static unsigned (__cdecl *p__statusfp)(void);
static int (__cdecl *p_fesetround)(int);

static int g_fail, g_ok;
#define ok(c, ...) do { if (c) g_ok++; else { g_fail++; SAY("FAIL line %d: ", __LINE__); DbgPrint(__VA_ARGS__); } } while (0)
#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))

static FARPROC need(const char *n)
{
    FARPROC p = GetProcAddress(h, n);
    if (!p) { g_fail++; SAY("FAIL: ucrtbase exports no %s\n", n); }
    return p;
}

/* DbgPrint has no %a/%f we can trust everywhere; print bit patterns. */
static unsigned long long dbits(double d) { union { double d; unsigned long long u; } x = { d }; return x.u; }
static unsigned fbits(float f) { union { float f; unsigned u; } x = { f }; return x.u; }

static long long ord64(unsigned long long u) { return (u >> 63) ? -(long long)(u & 0x7fffffffffffffffULL) : (long long)u; }
static long long ord32(unsigned u) { return (u >> 31) ? -(long long)(u & 0x7fffffffu) : (long long)u; }

static int match64(unsigned long long got, unsigned long long exp, int ulps)
{
    long long d;
    if ((exp & 0x7ff0000000000000ULL) == 0x7ff0000000000000ULL && (exp & 0xfffffffffffffULL))
        return (got & 0x7ff0000000000000ULL) == 0x7ff0000000000000ULL && (got & 0xfffffffffffffULL);
    if (!ulps) return got == exp;
    d = ord64(got) - ord64(exp);
    return (d < 0 ? -d : d) <= ulps;
}

static int match32(unsigned got, unsigned exp, int ulps)
{
    long long d;
    if ((exp & 0x7f800000u) == 0x7f800000u && (exp & 0x7fffffu))
        return (got & 0x7f800000u) == 0x7f800000u && (got & 0x7fffffu);
    if (!ulps) return got == exp;
    d = ord32(got) - ord32(exp);
    return (d < 0 ? -d : d) <= ulps;
}

static void test_table(void)
{
    int i;
    for (i = 0; i < ARRAY_SIZE(rows); i++) {
        const struct row *r = &rows[i];
        FARPROC p;
        unsigned long long exp = expected[i];
#if defined(__i386__)
        /* x86 ucrtbase has no powf/fmodf export (the spec says -arch=!i386, as native). */
        if (!strcmp(r->fn, "powf") || !strcmp(r->fn, "fmodf")) continue;
#endif
        p = need(r->fn);
        int e;
        if (!p) continue;
        *p_errno() = -1;
        switch (r->k) {
        case KD: {
            unsigned long long got = dbits(((double (__cdecl *)(double))p)(r->x));
            e = *p_errno();
            ok(match64(got, exp, r->ulps), "%s(%I64x) = %I64x, expected %I64x (+-%d ulp)\n", r->fn, dbits(r->x), got, exp, r->ulps);
            break;
        }
        case KD2: {
            unsigned long long got = dbits(((double (__cdecl *)(double, double))p)(r->x, r->y));
            e = *p_errno();
            ok(match64(got, exp, r->ulps), "%s(%I64x, %I64x) = %I64x, expected %I64x\n", r->fn, dbits(r->x), dbits(r->y), got, exp);
            break;
        }
        case KF: {
            unsigned got = fbits(((float (__cdecl *)(float))p)((float)r->x));
            e = *p_errno();
            ok(match32(got, (unsigned)exp, r->ulps), "%s(%x) = %x, expected %x (+-%d ulp)\n", r->fn, fbits((float)r->x), got, (unsigned)exp, r->ulps);
            break;
        }
        case KF2: {
            unsigned got = fbits(((float (__cdecl *)(float, float))p)((float)r->x, (float)r->y));
            e = *p_errno();
            ok(match32(got, (unsigned)exp, r->ulps), "%s(%x, %x) = %x, expected %x\n", r->fn, fbits((float)r->x), fbits((float)r->y), got, (unsigned)exp);
            break;
        }
        case KI32D: {
            long got = ((long (__cdecl *)(double))p)(r->x);
            e = *p_errno();
            ok(got == (long)exp, "%s(%I64x) = %ld, expected %ld\n", r->fn, dbits(r->x), got, (long)exp);
            break;
        }
        case KI64D: {
            long long got = ((long long (__cdecl *)(double))p)(r->x);
            e = *p_errno();
            ok(got == (long long)exp, "%s(%I64x) = %I64d, expected %I64d\n", r->fn, dbits(r->x), got, (long long)exp);
            break;
        }
        case KI32F: {
            long got = ((long (__cdecl *)(float))p)((float)r->x);
            e = *p_errno();
            ok(got == (long)exp, "%s(%x) = %ld, expected %ld\n", r->fn, fbits((float)r->x), got, (long)exp);
            break;
        }
        default: {
            long long got = ((long long (__cdecl *)(float))p)((float)r->x);
            e = *p_errno();
            ok(got == (long long)exp, "%s(%x) = %I64d, expected %I64d\n", r->fn, fbits((float)r->x), got, (long long)exp);
            break;
        }
        }
        ok(e == r->err, "%s(%I64x): errno %d, expected %d\n", r->fn, dbits(r->x), e, r->err);
    }
}

/* msvcrt.dll shipped cnt-rust's pow/log10/fmod too: the same value rows, against it. */
static void test_msvcrt_values(void)
{
    static const char *const names[] = { "pow", "powf", "log10", "fmod", "fmodf" };
    HMODULE m = LoadLibraryA("msvcrt.dll");
    int i, n;
    if (!m) { g_fail++; SAY("FAIL: no msvcrt.dll\n"); return; }
    for (i = 0; i < ARRAY_SIZE(rows); i++) {
        const struct row *r = &rows[i];
        FARPROC p = NULL;
        for (n = 0; n < ARRAY_SIZE(names); n++)
            if (!strcmp(r->fn, names[n])) p = GetProcAddress(m, r->fn);
        if (!p) continue;
        if (r->k == KD)
            ok(match64(dbits(((double (__cdecl *)(double))p)(r->x)), expected[i], r->ulps),
               "msvcrt %s(%I64x) = %I64x\n", r->fn, dbits(r->x), dbits(((double (__cdecl *)(double))p)(r->x)));
        else if (r->k == KD2)
            ok(match64(dbits(((double (__cdecl *)(double, double))p)(r->x, r->y)), expected[i], r->ulps),
               "msvcrt %s(%I64x, %I64x) = %I64x\n", r->fn, dbits(r->x), dbits(r->y),
               dbits(((double (__cdecl *)(double, double))p)(r->x, r->y)));
        else if (r->k == KF2)
            ok(match32(fbits(((float (__cdecl *)(float, float))p)((float)r->x, (float)r->y)), (unsigned)expected[i], r->ulps),
               "msvcrt %s(%x, %x)\n", r->fn, fbits((float)r->x), fbits((float)r->y));
    }
}

#define FE_TONEAREST  0x000
#define FE_DOWNWARD   0x100
#define FE_UPWARD     0x200
#define FE_TOWARDZERO 0x300
#define _SW_INEXACT   0x1

static void test_rounding(void)
{
    double (__cdecl *p_rint)(double) = (void *)need("rint");
    double (__cdecl *p_nearbyint)(double) = (void *)need("nearbyint");
    float (__cdecl *p_rintf)(float) = (void *)need("rintf");
    long long (__cdecl *p_llrint)(double) = (void *)need("llrint");
    static const struct { int mode; double x, r; } t[] = {
        { FE_UPWARD, 1.2, 2.0 }, { FE_UPWARD, -1.8, -1.0 },
        { FE_DOWNWARD, 1.8, 1.0 }, { FE_DOWNWARD, -1.2, -2.0 },
        { FE_TOWARDZERO, -1.8, -1.0 }, { FE_TOWARDZERO, 1.8, 1.0 },
        { FE_TONEAREST, 0.5, 0.0 }, { FE_TONEAREST, 1.5, 2.0 },
    };
    int i;
    long long ll;
    if (!p_rint || !p_nearbyint || !p_rintf || !p_llrint) return;

    /* rint and nearbyint follow the CURRENT rounding mode. */
    for (i = 0; i < ARRAY_SIZE(t); i++) {
        p_fesetround(t[i].mode);
        ok(p_rint(t[i].x) == t[i].r, "rint(%I64x) in mode %x = %I64x\n", dbits(t[i].x), t[i].mode, dbits(p_rint(t[i].x)));
        ok(p_nearbyint(t[i].x) == t[i].r, "nearbyint(%I64x) in mode %x = %I64x\n", dbits(t[i].x), t[i].mode, dbits(p_nearbyint(t[i].x)));
        ok(p_rintf((float)t[i].x) == (float)t[i].r, "rintf(%I64x) in mode %x\n", dbits(t[i].x), t[i].mode);
    }
    p_fesetround(FE_TONEAREST);

    /* rint raises inexact; nearbyint must not, and must not clear one already set. */
    p__clearfp();
    p_nearbyint(1.5);
    ok(!(p__statusfp() & _SW_INEXACT), "nearbyint raised inexact, status %x\n", p__statusfp());
    p__clearfp();
    p_rint(1.5);
    ok(p__statusfp() & _SW_INEXACT, "rint did not raise inexact, status %x\n", p__statusfp());
    p_nearbyint(2.0);
    ok(p__statusfp() & _SW_INEXACT, "nearbyint cleared an inexact it did not raise\n");
    p__clearfp();

    /* llrint between INT64_MAX and UINT64_MAX: Wine accepts it with no errno and
     * converts it the way the hardware does -- x86's indefinite INT64_MIN, arm64's
     * saturation. */
    *p_errno() = -1;
    ll = p_llrint(1e19);
#if defined(__aarch64__)
    ok(ll == 0x7fffffffffffffffLL, "llrint(1e19) = %I64d\n", ll);
#else
    ok(ll == -0x7fffffffffffffffLL - 1, "llrint(1e19) = %I64d\n", ll);
#endif
    ok(*p_errno() == -1, "llrint(1e19) errno %d\n", *p_errno());
}

static void test_misc(void)
{
    double (__cdecl *p_remquo)(double, double, int *) = (void *)need("remquo");
    float (__cdecl *p_remquof)(float, float, int *) = (void *)need("remquof");
    double (__cdecl *p_nan)(const char *) = (void *)need("nan");
    float (__cdecl *p_nanf)(const char *) = (void *)need("nanf");
    int (__cdecl *p__dpcomp)(double, double) = (void *)need("_dpcomp");
    int (__cdecl *p__fdpcomp)(float, float) = (void *)need("_fdpcomp");
    typedef struct { double v[2]; } dcomplex;
    dcomplex (__cdecl *p__Cbuild)(double, double) = (void *)need("_Cbuild");
    double (__cdecl *p_creal)(dcomplex) = (void *)need("creal");
    static const struct { double x, y; int ret; } dp[] = {   /* Wine msvcr120 test__dpcomp */
        {0, 0, 2}, {1, 1, 2}, {-1, -1, 2},
        {-2, -1, 1}, {-1, 1, 1}, {1, 2, 1},
        {1, -1, 4}, {2, 1, 4}, {-1, -2, 4},
        {NAN_, NAN_, 0}, {NAN_, 1, 0}, {1, NAN_, 0},
        {INF_, INF_, 2}, {-1, INF_, 1}, {1, INF_, 1},
    };
    int i, q;
    double r;
    dcomplex c;

    if (p_remquo) {
        r = p_remquo(10.0, 3.0, &q);
        ok(r == 1.0 && q == 3, "remquo(10, 3) = %I64x, quo %d\n", dbits(r), q);
        r = p_remquo(-10.0, 3.0, &q);
        ok(r == -1.0 && q == -3, "remquo(-10, 3) = %I64x, quo %d\n", dbits(r), q);
        r = p_remquo(7.0, 2.0, &q);
        ok(r == -1.0 && (q & 7) == 4, "remquo(7, 2) = %I64x, quo %d\n", dbits(r), q);
        *p_errno() = -1;
        p_remquo(1.0, 0.0, &q);
        ok(*p_errno() == EDOM_, "remquo(1, 0) errno %d\n", *p_errno());
    }
    if (p_remquof) {
        float f = p_remquof(10.0f, 3.0f, &q);
        ok(f == 1.0f && q == 3, "remquof(10, 3) = %x, quo %d\n", fbits(f), q);
    }
    {
        /* fma: Wine's test_math_errors tests3d (invalid -> EDOM, NaN or overflow -> none),
         * plus the single rounding stubs.c's x*y+z could not give. */
        double (__cdecl *p_fma)(double, double, double) = (void *)need("fma");
        float (__cdecl *p_fmaf)(float, float, float) = (void *)need("fmaf");
        static const struct { double a, b, c; int e; } t3[] = {
            { INF_, 0, 0, EDOM_ }, { 0, INF_, 0, EDOM_ },
            { INF_, 1, -INF_, EDOM_ }, { -INF_, 1, INF_, EDOM_ },
            { 1, INF_, -INF_, EDOM_ }, { 1, -INF_, INF_, EDOM_ },
            { NAN_, 0, 0, NOERR }, { 0, NAN_, 0, NOERR }, { 0, 0, NAN_, NOERR },
            { 1.7976931348623157e308, 1.7976931348623157e308, 1.7976931348623157e308, NOERR },
        };
        int j;
        for (j = 0; p_fma && j < ARRAY_SIZE(t3); j++) {
            *p_errno() = -1;
            p_fma(t3[j].a, t3[j].b, t3[j].c);
            ok(*p_errno() == t3[j].e, "fma row %d errno %d, expected %d\n", j, *p_errno(), t3[j].e);
        }
        if (p_fma) {
            /* (1 + 2^-30)^2 - 1 = 2^-29 + 2^-60: fused keeps the 2^-60 that x*y+z rounds away. */
            double a = 1.0 + 0x1p-30;
            r = p_fma(a, a, -1.0);
            ok(r == 0x1p-29 + 0x1p-60, "fma not fused: %I64x\n", dbits(r));
        }
        if (p_fmaf) {
            float a = 1.0f + 0x1p-13f;
            float f = p_fmaf(a, a, -1.0f);
            ok(f == 0x1p-12f + 0x1p-26f, "fmaf not fused: %x\n", fbits(f));
        }
    }
    if (p_nan) ok(dbits(p_nan("")) == 0x7ff8000000000000ULL, "nan(\"\") = %I64x\n", dbits(p_nan("")));
    if (p_nanf) ok(fbits(p_nanf("")) == 0x7fc00000u, "nanf(\"\") = %x\n", fbits(p_nanf("")));
    for (i = 0; p__dpcomp && i < ARRAY_SIZE(dp); i++) {
        ok(p__dpcomp(dp[i].x, dp[i].y) == dp[i].ret, "_dpcomp row %d = %d\n", i, p__dpcomp(dp[i].x, dp[i].y));
        ok(p__fdpcomp((float)dp[i].x, (float)dp[i].y) == dp[i].ret, "_fdpcomp row %d\n", i);
    }
    if (p__Cbuild && p_creal) {                              /* Wine msvcr120 test__Cbuild */
        c = p__Cbuild(1.0, 2.0);
        ok(c.v[0] == 1.0 && c.v[1] == 2.0, "_Cbuild(1, 2) = {%I64x, %I64x}\n", dbits(c.v[0]), dbits(c.v[1]));
        ok(p_creal(c) == 1.0, "creal = %I64x\n", dbits(p_creal(c)));
        c = p__Cbuild(3.0, NAN_);
        ok(c.v[0] == 3.0 && c.v[1] != c.v[1], "_Cbuild(3, nan)\n");
        ok(p_creal(c) == 3.0, "creal(3, nan)\n");
    }

#if !defined(__i386__)
    {
        /* _fpclassf: not exported on i386 (and an sNaN would not survive x87 there). */
        int (__cdecl *p__fpclassf)(float) = (void *)need("_fpclassf");
        static const struct { unsigned bits; int cls; } fc[] = {
            { 0x3f800000, 0x100 }, { 0xbf800000, 0x008 }, { 0x00000000, 0x040 }, { 0x80000000, 0x020 },
            { 0x00000001, 0x080 }, { 0x80000001, 0x010 }, { 0x7f800000, 0x200 }, { 0xff800000, 0x004 },
            { 0x7fc00000, 0x002 }, { 0x7fa00000, 0x001 },
        };
        for (i = 0; p__fpclassf && i < ARRAY_SIZE(fc); i++) {
            union { unsigned u; float f; } v = { fc[i].bits };
            ok(p__fpclassf(v.f) == fc[i].cls, "_fpclassf(%x) = %x, expected %x\n", fc[i].bits, p__fpclassf(v.f), fc[i].cls);
        }
    }
#endif
}

int __cdecl main(void)
{
    h = LoadLibraryA("ucrtbase.dll");
#define G(v, n) (*(FARPROC *)&v = GetProcAddress(h, n))
    if (!h || !G(p_errno, "_errno") || !G(p__clearfp, "_clearfp") || !G(p__statusfp, "_statusfp")
        || !G(p_fesetround, "fesetround")) {
        SAY("done FAIL (load)\n");
        return 1;
    }
    test_table();
    test_rounding();
    test_misc();
    test_msvcrt_values();
    SAY("%d checks, %d failed\n", g_ok + g_fail, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
