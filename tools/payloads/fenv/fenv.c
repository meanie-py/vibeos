/* fenv — ucrtbase's fenv.h family, graded by Wine's own test_fenv.
 *
 * Until 2026-10-03 every fe* export of ucrtbase was a spec stub. ucrt-rust's
 * fenv.rs ports Wine's implementation; this is Wine's conformance test for it
 * (modules/rostests/winetests/ucrtbase/misc.c test_fenv, whose expected values
 * are native Windows'), lifted out because the winetest's misc subtest hangs
 * before reaching it. Every function comes from ucrtbase.dll by GetProcAddress,
 * so no static mingw fenv can answer in its place. Also __fpe_flt_rounds.
 */
#include <windows.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[FENV] " __VA_ARGS__)

#define FE_TONEAREST  0x000
#define FE_DOWNWARD   0x100
#define FE_UPWARD     0x200
#define FE_TOWARDZERO 0x300
#define FE_INEXACT    0x01
#define FE_UNDERFLOW  0x02
#define FE_OVERFLOW   0x04
#define FE_DIVBYZERO  0x08
#define FE_INVALID    0x10
#define FE_ALL_EXCEPT 0x1f

typedef struct { unsigned long _Fe_ctl, _Fe_stat; } fenv_t;
typedef unsigned long fexcept_t;

static int (__cdecl *p_fegetenv)(fenv_t *);
static int (__cdecl *p_fesetenv)(const fenv_t *);
static int (__cdecl *p_fesetround)(int);
static int (__cdecl *p_fegetround)(void);
static int (__cdecl *p_fesetexceptflag)(const fexcept_t *, int);
static int (__cdecl *p_fegetexceptflag)(fexcept_t *, int);
static int (__cdecl *p_fetestexcept)(int);
static int (__cdecl *p_feclearexcept)(int);
static int (__cdecl *p_feholdexcept)(fenv_t *);
static int (__cdecl *p___fpe_flt_rounds)(void);
static unsigned (__cdecl *p__clearfp)(void);

static int g_fail, g_ok;
#define ok(c, ...) do { if (c) g_ok++; else { g_fail++; SAY("FAIL line %d: ", __LINE__); DbgPrint(__VA_ARGS__); } } while (0)
#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))

static unsigned long fenv_encode(unsigned int e)
{
#if defined(__i386__)
    return e << 24 | e << 16 | e;
#elif defined(__x86_64__)
    return e << 24 | e;
#else
    return e;
#endif
}

static void test_fenv(void)
{
    static const int tests[] = { 0, FE_INEXACT, FE_UNDERFLOW, FE_OVERFLOW, FE_DIVBYZERO, FE_INVALID, FE_ALL_EXCEPT };
    static const struct { fexcept_t except; unsigned int flag; unsigned int get; fexcept_t expect; } tests2[] = {
        { 0,                        0,                       0,              0 },
        { FE_ALL_EXCEPT,            FE_INEXACT,              0,              0 },
        { FE_ALL_EXCEPT,            FE_INEXACT,              FE_ALL_EXCEPT,  FE_INEXACT },
        { FE_ALL_EXCEPT,            FE_INEXACT,              FE_INEXACT,     FE_INEXACT },
        { FE_ALL_EXCEPT,            FE_INEXACT,              FE_OVERFLOW,    0 },
        { FE_ALL_EXCEPT,            FE_ALL_EXCEPT,           FE_ALL_EXCEPT,  FE_ALL_EXCEPT },
        { FE_ALL_EXCEPT,            FE_ALL_EXCEPT,           FE_INEXACT,     FE_INEXACT },
        { FE_ALL_EXCEPT,            FE_ALL_EXCEPT,           0,              0 },
        { FE_ALL_EXCEPT,            FE_ALL_EXCEPT,           ~0,             FE_ALL_EXCEPT },
        { FE_ALL_EXCEPT,            FE_ALL_EXCEPT,           ~FE_ALL_EXCEPT, 0 },
        { FE_INEXACT,               FE_ALL_EXCEPT,           FE_ALL_EXCEPT,  FE_INEXACT },
        { FE_INEXACT,               FE_UNDERFLOW,            FE_ALL_EXCEPT,  0 },
        { FE_UNDERFLOW,             FE_INEXACT,              FE_ALL_EXCEPT,  0 },
        { FE_INEXACT|FE_UNDERFLOW,  FE_UNDERFLOW,            FE_ALL_EXCEPT,  FE_UNDERFLOW },
        { FE_UNDERFLOW,             FE_INEXACT|FE_UNDERFLOW, FE_ALL_EXCEPT,  FE_UNDERFLOW },
    };
    fenv_t env, env2;
    fexcept_t except;
    int i, ret, flags;

    p__clearfp();
    ret = p_fegetenv(&env);
    ok(!ret, "fegetenv returned %x\n", ret);
#if defined(__i386__) || defined(__x86_64__)
    ok(env._Fe_ctl >> 24 == (env._Fe_ctl & 0xff), "fenv_t format: _Fe_ctl %lx\n", env._Fe_ctl);
#endif
    p_fesetround(FE_UPWARD);
    ok(!env._Fe_stat, "env._Fe_stat = %lx\n", env._Fe_stat);
    ret = p_fegetenv(&env2);
    ok(!ret, "fegetenv returned %x\n", ret);
    ok(env._Fe_ctl != env2._Fe_ctl, "fesetround didn't change _Fe_ctl (%lx).\n", env._Fe_ctl);
    ret = p_fesetenv(&env);
    ok(!ret, "fesetenv returned %x\n", ret);
    ret = p_fegetround();
    ok(ret == FE_TONEAREST, "Got unexpected round mode %#x.\n", ret);

    except = fenv_encode(FE_ALL_EXCEPT);
    ret = p_fesetexceptflag(&except, FE_INEXACT|FE_UNDERFLOW);
    ok(!ret, "fesetexceptflag returned %x\n", ret);
    except = p_fetestexcept(FE_ALL_EXCEPT);
    ok(except == (FE_INEXACT|FE_UNDERFLOW), "expected %x, got %lx\n", FE_INEXACT|FE_UNDERFLOW, except);

    ret = p_feclearexcept(~FE_ALL_EXCEPT);
    ok(!ret, "feclearexceptflag returned %x\n", ret);
    except = p_fetestexcept(FE_ALL_EXCEPT);
    ok(except == (FE_INEXACT|FE_UNDERFLOW), "expected %x, got %lx\n", FE_INEXACT|FE_UNDERFLOW, except);

    ret = p_fesetexceptflag(NULL, 0);
    ok(!ret, "fesetexceptflag returned %x\n", ret);
    except = p_fetestexcept(FE_ALL_EXCEPT);
    ok(except == (FE_INEXACT|FE_UNDERFLOW), "expected %x, got %lx\n", FE_INEXACT|FE_UNDERFLOW, except);

    except = 0;
    ret = p_fesetexceptflag(&except, FE_ALL_EXCEPT);
    ok(!ret, "fesetexceptflag returned %x\n", ret);
    except = p_fetestexcept(FE_ALL_EXCEPT);
    ok(!except, "expected 0, got %lx\n", except);

    ret = p_fetestexcept(FE_ALL_EXCEPT);
    ok(!ret, "fetestexcept returned %x\n", ret);

    flags = 0;
    for (i = 0; i < ARRAY_SIZE(tests); i++) {
        except = fenv_encode(FE_ALL_EXCEPT);
        ret = p_fesetexceptflag(&except, tests[i]);
        ok(!ret, "Test %d: fesetexceptflag returned %x\n", i, ret);
        ret = p_fetestexcept(tests[i]);
        ok(ret == tests[i], "Test %d: expected %x, got %x\n", i, tests[i], ret);
        flags |= tests[i];
        ret = p_fetestexcept(FE_ALL_EXCEPT);
        ok(ret == flags, "Test %d: expected %x, got %x\n", i, flags, ret);
        except = ~0;
        ret = p_fegetexceptflag(&except, ~0);
        ok(!ret, "Test %d: fegetexceptflag returned %x.\n", i, ret);
        ok(except == fenv_encode(flags), "Test %d: expected %lx, got %lx\n", i, fenv_encode(flags), except);
        except = ~0;
        ret = p_fegetexceptflag(&except, tests[i]);
        ok(!ret, "Test %d: fegetexceptflag returned %x.\n", i, ret);
        ok(except == fenv_encode(tests[i]), "Test %d: expected %lx, got %lx\n", i, fenv_encode(tests[i]), except);
    }

    for (i = 0; i < ARRAY_SIZE(tests); i++) {
        ret = p_feclearexcept(tests[i]);
        ok(!ret, "Test %d: feclearexceptflag returned %x\n", i, ret);
        flags &= ~tests[i];
        except = p_fetestexcept(tests[i]);
        ok(!except, "Test %d: expected %x, got %lx\n", i, flags, except);
    }

    except = p_fetestexcept(FE_ALL_EXCEPT);
    ok(!except, "expected 0, got %lx\n", except);

    for (i = 0; i < ARRAY_SIZE(tests); i++) {
        except = fenv_encode(tests[i]);
        ret = p_fesetexceptflag(&except, FE_ALL_EXCEPT);
        ok(!ret, "Test %d: fesetexceptflag returned %x\n", i, ret);
        ret = p_fetestexcept(tests[i]);
        ok(ret == tests[i], "Test %d: expected %x, got %x\n", i, tests[i], ret);
        ret = p_fetestexcept(FE_ALL_EXCEPT);
        ok(ret == tests[i], "Test %d: expected %x, got %x\n", i, tests[i], ret);
    }

    for (i = 0; i < ARRAY_SIZE(tests2); i++) {
        p__clearfp();
        except = fenv_encode(tests2[i].except);
        ret = p_fesetexceptflag(&except, tests2[i].flag);
        ok(!ret, "Test %d: fesetexceptflag returned %x\n", i, ret);
        ret = p_fetestexcept(tests2[i].get);
        ok(ret == tests2[i].expect, "Test %d: expected %lx, got %x\n", i, tests2[i].expect, ret);
    }

    ret = p_feclearexcept(FE_ALL_EXCEPT);
    ok(!ret, "feclearexceptflag returned %x\n", ret);
    except = p_fetestexcept(FE_ALL_EXCEPT);
    ok(!except, "expected 0, got %lx\n", except);

    /* feholdexcept: saves, then clears every flag */
    except = fenv_encode(FE_OVERFLOW);
    p_fesetexceptflag(&except, FE_ALL_EXCEPT);
    ret = p_feholdexcept(&env);
    ok(!ret, "feholdexcept returned %x\n", ret);
    ok(!p_fetestexcept(FE_ALL_EXCEPT), "feholdexcept left flags %x\n", p_fetestexcept(FE_ALL_EXCEPT));
    p_fesetenv(&env);
    ok(p_fetestexcept(FE_ALL_EXCEPT) == FE_OVERFLOW, "fesetenv restored %x\n", p_fetestexcept(FE_ALL_EXCEPT));
    p_feclearexcept(FE_ALL_EXCEPT);
}

static void test___fpe_flt_rounds(void)
{
    /* Wine misc.c test___fpe_flt_rounds: FLT_ROUNDS per mode. */
    static const struct { int mode, expect; } t[] = {
        { FE_TONEAREST, 1 }, { FE_TOWARDZERO, 0 }, { FE_UPWARD, 2 }, { FE_DOWNWARD, 3 },
    };
    int i;
    for (i = 0; i < ARRAY_SIZE(t); i++) {
        ok(!p_fesetround(t[i].mode), "fesetround(%x) failed\n", t[i].mode);
        ok(p_fegetround() == t[i].mode, "fegetround = %x, expected %x\n", p_fegetround(), t[i].mode);
        ok(p___fpe_flt_rounds() == t[i].expect, "__fpe_flt_rounds = %d in mode %x\n", p___fpe_flt_rounds(), t[i].mode);
    }
    ok(p_fesetround(0x123) != 0, "fesetround(0x123) accepted\n");
    p_fesetround(FE_TONEAREST);
}

int __cdecl main(void)
{
    HMODULE h = LoadLibraryA("ucrtbase.dll");
#define G(v, n) (*(FARPROC *)&v = GetProcAddress(h, n))
    if (!h || !G(p_fegetenv, "fegetenv") || !G(p_fesetenv, "fesetenv") || !G(p_fesetround, "fesetround")
        || !G(p_fegetround, "fegetround") || !G(p_fesetexceptflag, "fesetexceptflag")
        || !G(p_fegetexceptflag, "fegetexceptflag") || !G(p_fetestexcept, "fetestexcept")
        || !G(p_feclearexcept, "feclearexcept") || !G(p_feholdexcept, "feholdexcept")
        || !G(p___fpe_flt_rounds, "__fpe_flt_rounds") || !G(p__clearfp, "_clearfp")) {
        SAY("done FAIL (load)\n");
        return 1;
    }
    test_fenv();
    test___fpe_flt_rounds();
    SAY("%d checks, %d failed\n", g_ok + g_fail, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
