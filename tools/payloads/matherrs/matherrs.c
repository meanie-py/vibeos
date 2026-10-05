/* matherrs — Wine's ucrtbase test_math_errors (modules/rostests/winetests/ucrtbase/misc.c),
 * lifted out VERBATIM apart from the plumbing below, with a DbgPrint before every call.
 * The winetest's misc subtest stops printing between log10(0) and sin(-inf) and is
 * killed at 120 s; the trace names the call that never returns.
 */
#include <windows.h>
#include <math.h>
#include <string.h>

ULONG __cdecl DbgPrint(const char *Format, ...);
#define SAY(...) DbgPrint("[MATHERRS] " __VA_ARGS__)
static int g_fail, g_ok;
#define ok(c, ...) do { if (c) g_ok++; else { g_fail++; SAY("FAIL line %d: ", __LINE__); DbgPrint(__VA_ARGS__); } } while (0)
#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))
#define EDOM 33
#define ERANGE 34
#define FE_TONEAREST 0
#ifndef _DOMAIN
#define _DOMAIN 1
#define _SING 2
#define _OVERFLOW 3
#define _UNDERFLOW 4
#endif

static HMODULE h;
static int *(__cdecl *p_errno)(void);
static void (__cdecl *p___setusermatherr)(int (__cdecl *)(struct _exception *));
static int (__cdecl *p_fesetround)(int);
static double (__cdecl *p_acos)(double);
#undef errno
#define errno (*p_errno())

static inline double __port_min_pos_double(void)
{
    static const UINT64 __min_pos_double = 0x10000000000000;
    return *(const double *)&__min_pos_double;
}

static inline double __port_max_double(void)
{
    static const UINT64 __max_double = 0x7FEFFFFFFFFFFFFF;
    return *(const double *)&__max_double;
}

static struct _exception exception;

static int CDECL matherr_callback(struct _exception *e)
{
    exception = *e;

    if (!strcmp(e->name, "acos") && e->arg1 == 2)
        e->retval = -1;
    return 0;
}

static void test_math_errors(void)
{
    const struct {
        char func[16];
        double x;
        int error;
        int exception;
    } testsd[] = {
        {"_logb", -INFINITY, -1, -1},
        {"_logb", -1, -1, -1},
        {"_logb", 0, ERANGE, _SING},
        {"_logb", INFINITY, -1, -1},
        {"acos", -INFINITY, EDOM, _DOMAIN},
        {"acos", -2, EDOM, _DOMAIN},
        {"acos", -1, -1, -1},
        {"acos", 1, -1, -1},
        {"acos", 2, EDOM, _DOMAIN},
        {"acos", INFINITY, EDOM, _DOMAIN},
        {"acosh", -INFINITY, EDOM, -1},
        {"acosh", 0, EDOM, -1},
        {"acosh", 1, -1, -1},
        {"acosh", INFINITY, -1, -1},
        {"asin", -INFINITY, EDOM, _DOMAIN},
        {"asin", -2, EDOM, _DOMAIN},
        {"asin", -1, -1, -1},
        {"asin", 1, -1, -1},
        {"asin", 2, EDOM, _DOMAIN},
        {"asin", INFINITY, EDOM, _DOMAIN},
        {"asinh", -INFINITY, -1, -1},
        {"asinh", INFINITY, -1, -1},
        {"atan", -INFINITY, -1, -1},
        {"atan", 0, -1, -1},
        {"atan", INFINITY, -1, -1},
        {"atanh", -INFINITY, EDOM, -1},
        {"atanh", -2, EDOM, -1},
        {"atanh", -1, ERANGE, -1},
        {"atanh", 1, ERANGE, -1},
        {"atanh", 2, EDOM, -1},
        {"atanh", INFINITY, EDOM, -1},
        {"cos", -INFINITY, EDOM, _DOMAIN},
        {"cos", INFINITY, EDOM, _DOMAIN},
        {"cosh", -INFINITY, -1, -1},
        {"cosh", 0, -1, -1},
        {"cosh", INFINITY, -1, -1},
        {"exp", -INFINITY, -1, -1},
        {"exp", -1e100, -1, _UNDERFLOW},
        {"exp", 1e100, ERANGE, _OVERFLOW},
        {"exp", INFINITY, -1, -1},
        {"exp2", -INFINITY, -1, -1},
        {"exp2", -1e100, -1, -1},
        {"exp2", 1e100, ERANGE, -1},
        {"exp2", INFINITY, -1, -1},
        {"expm1", -INFINITY, -1, -1},
        {"expm1", INFINITY, -1, -1},
        {"log", -INFINITY, EDOM, _DOMAIN},
        {"log", -1, EDOM, _DOMAIN},
        {"log", 0, ERANGE, _SING},
        {"log", INFINITY, -1, -1},
        {"log10", -INFINITY, EDOM, _DOMAIN},
        {"log10", -1, EDOM, _DOMAIN},
        {"log10", 0, ERANGE, _SING},
        {"log10", INFINITY, -1, -1},
        {"log1p", -INFINITY, EDOM, -1},
        {"log1p", -2, EDOM, -1},
        {"log1p", -1, ERANGE, -1},
        {"log1p", INFINITY, -1, -1},
        {"log2", INFINITY, -1, -1},
        {"sin", -INFINITY, EDOM, _DOMAIN},
        {"sin", INFINITY, EDOM, _DOMAIN},
        {"sinh", -INFINITY, -1, -1},
        {"sinh", 0, -1, -1},
        {"sinh", INFINITY, -1, -1},
        {"sqrt", -INFINITY, EDOM, _DOMAIN},
        {"sqrt", -1, EDOM, _DOMAIN},
        {"sqrt", 0, -1, -1},
        {"sqrt", INFINITY, -1, -1},
        {"tan", -INFINITY, EDOM, _DOMAIN},
        {"tan", -M_PI_2, -1, -1},
        {"tan", M_PI_2, -1, -1},
        {"tan", INFINITY, EDOM, _DOMAIN},
        {"tanh", -INFINITY, -1, -1},
        {"tanh", 0, -1, -1},
        {"tanh", INFINITY, -1, -1},
    };
    const struct {
        char func[16];
        double a;
        double b;
        int error;
        int exception;
    } tests2d[] = {
        {"atan2", -INFINITY, 0, -1, -1},
        {"atan2", 0, 0, -1, -1},
        {"atan2", INFINITY, 0, -1, -1},
        {"atan2", 0, -INFINITY, -1, -1},
        {"atan2", 0, INFINITY, -1, -1},
        {"pow", -INFINITY, -2, -1, -1},
        {"pow", -INFINITY, -1, -1, -1},
        {"pow", -INFINITY, 0, -1, -1},
        {"pow", -INFINITY, 1, -1, -1},
        {"pow", -INFINITY, 2, -1, -1},
        {"pow", -1e100, -10, -1, _UNDERFLOW},
        {"pow", -1e100, 10, ERANGE, _OVERFLOW},
        {"pow", -1, 1.5, EDOM, _DOMAIN},
        {"pow", 0, -2, ERANGE, _SING},
        {"pow", 0, -1, ERANGE, _SING},
        {"pow", 0.5, -INFINITY, -1, -1},
        {"pow", 0.5, INFINITY, -1, -1},
        {"pow", 2, -INFINITY, -1, -1},
        {"pow", 2, -1e100, -1, _UNDERFLOW},
        {"pow", 2, 1e100, ERANGE, _OVERFLOW},
        {"pow", 2, INFINITY, -1, -1},
        {"pow", 1e100, -10, -1, _UNDERFLOW},
        {"pow", 1e100, 10, ERANGE, _OVERFLOW},
        {"pow", INFINITY, -2, -1, -1},
        {"pow", INFINITY, -1, -1, -1},
        {"pow", INFINITY, 0, -1, -1},
        {"pow", INFINITY, 1, -1, -1},
        {"pow", INFINITY, 2, -1, -1},
    };
    const struct {
        char func[16];
        double a;
        double b;
        double c;
        int error;
        int exception;
    } tests3d[] = {
        /* 0 * inf --> EDOM */
        {"fma", INFINITY, 0, 0, EDOM, -1},
        {"fma", 0, INFINITY, 0, EDOM, -1},
        /* inf - inf -> EDOM */
        {"fma", INFINITY, 1, -INFINITY, EDOM, -1},
        {"fma", -INFINITY, 1, INFINITY, EDOM, -1},
        {"fma", 1, INFINITY, -INFINITY, EDOM, -1},
        {"fma", 1, -INFINITY, INFINITY, EDOM, -1},
        /* NaN */
        {"fma", NAN, 0, 0, -1, -1},
        {"fma", 0, NAN, 0, -1, -1},
        {"fma", 0, 0, NAN, -1, -1},
        /* over/underflow */
        {"fma", __port_max_double(), __port_max_double(), __port_max_double(), -1, -1},
        {"fma", __port_min_pos_double(), __port_min_pos_double(), 1, -1, -1},
    };
    const struct {
        char func[16];
        double a;
        long b;
        int error;
        int exception;
    } testsdl[] = {
        {"_scalb", -INFINITY, 1, -1, -1},
        {"_scalb", -1e100, 1, -1, -1},
        {"_scalb", 0, 1, -1, -1},
        {"_scalb", 1e100, 1, -1, -1},
        {"_scalb", INFINITY, 1, -1, -1},
        {"_scalb", 1, 1e9, ERANGE, _OVERFLOW},
        {"ldexp", -INFINITY, 1, -1, -1},
        {"ldexp", -1e100, 1, -1, -1},
        {"ldexp", 0, 1, -1, -1},
        {"ldexp", 1e100, 1, -1, -1},
        {"ldexp", INFINITY, 1, -1, -1},
        {"ldexp", 1, -1e9, -1, _UNDERFLOW},
        {"ldexp", 1, 1e9, ERANGE, _OVERFLOW},
    };
    double (CDECL *p_funcd)(double);
    double (CDECL *p_func2d)(double, double);
    double (CDECL *p_func3d)(double, double, double);
    double (CDECL *p_funcdl)(double, long);
    HMODULE module;
    double d;
    int i;

    p___setusermatherr(matherr_callback);
    module = h;

    /* necessary so that exp(1e100)==INFINITY on glibc, we can remove this if we change our implementation */
    p_fesetround(FE_TONEAREST);

    for(i = 0; i < ARRAY_SIZE(testsd); i++) {
        p_funcd = (void*)GetProcAddress(module, testsd[i].func);
        errno = -1;
        exception.type = -1;
        SAY("call %s row %d\n", testsd[i].func, i);
        p_funcd(testsd[i].x);
        ok(errno == testsd[i].error,
           "%s(%f) got errno %d\n", testsd[i].func, testsd[i].x, errno);
        ok(exception.type == testsd[i].exception,
           "%s(%f) got exception type %d\n", testsd[i].func, testsd[i].x, exception.type);
        if(exception.type == -1) continue;
        ok(exception.arg1 == testsd[i].x,
           "%s(%f) got exception arg1 %f\n", testsd[i].func, testsd[i].x, exception.arg1);
    }

    for(i = 0; i < ARRAY_SIZE(tests2d); i++) {
        p_func2d = (void*)GetProcAddress(module, tests2d[i].func);
        errno = -1;
        exception.type = -1;
        SAY("call %s row %d\n", tests2d[i].func, i);
        p_func2d(tests2d[i].a, tests2d[i].b);
        ok(errno == tests2d[i].error,
           "%s(%f, %f) got errno %d\n", tests2d[i].func, tests2d[i].a, tests2d[i].b, errno);
        ok(exception.type == tests2d[i].exception,
           "%s(%f, %f) got exception type %d\n", tests2d[i].func, tests2d[i].a, tests2d[i].b, exception.type);
        if(exception.type == -1) continue;
        ok(exception.arg1 == tests2d[i].a,
           "%s(%f, %f) got exception arg1 %f\n", tests2d[i].func, tests2d[i].a, tests2d[i].b, exception.arg1);
        ok(exception.arg2 == tests2d[i].b,
           "%s(%f, %f) got exception arg2 %f\n", tests2d[i].func, tests2d[i].a, tests2d[i].b, exception.arg2);
    }

    for(i = 0; i < ARRAY_SIZE(tests3d); i++) {
        p_func3d = (void*)GetProcAddress(module, tests3d[i].func);
        errno = -1;
        exception.type = -1;
        SAY("call %s row %d\n", tests3d[i].func, i);
        p_func3d(tests3d[i].a, tests3d[i].b, tests3d[i].c);
        ok(errno == tests3d[i].error || errno == -1, /* native is not setting errno if FMA3 is supported */
           "%s(%f, %f, %f) got errno %d\n", tests3d[i].func, tests3d[i].a, tests3d[i].b, tests3d[i].c, errno);
        ok(exception.type == tests3d[i].exception,
           "%s(%f, %f, %f) got exception type %d\n", tests3d[i].func, tests3d[i].a, tests3d[i].b, tests3d[i].c, exception.type);
        if(exception.type == -1) continue;
        ok(exception.arg1 == tests3d[i].a,
           "%s(%f, %f, %f) got exception arg1 %f\n", tests3d[i].func, tests3d[i].a, tests3d[i].b, tests3d[i].c, exception.arg1);
        ok(exception.arg2 == tests3d[i].b,
           "%s(%f, %f, %f) got exception arg2 %f\n", tests3d[i].func, tests3d[i].a, tests3d[i].b, tests3d[i].c, exception.arg2);
    }

    for(i = 0; i < ARRAY_SIZE(testsdl); i++) {
        p_funcdl = (void*)GetProcAddress(module, testsdl[i].func);
        errno = -1;
        exception.type = -1;
        SAY("call %s row %d\n", testsdl[i].func, i);
        p_funcdl(testsdl[i].a, testsdl[i].b);
        ok(errno == testsdl[i].error,
           "%s(%f, %ld) got errno %d\n", testsdl[i].func, testsdl[i].a, testsdl[i].b, errno);
        ok(exception.type == testsdl[i].exception,
           "%s(%f, %ld) got exception type %d\n", testsdl[i].func, testsdl[i].a, testsdl[i].b, exception.type);
        if(exception.type == -1) continue;
        ok(exception.arg1 == testsdl[i].a,
           "%s(%f, %ld) got exception arg1 %f\n", testsdl[i].func, testsdl[i].a, testsdl[i].b, exception.arg1);
        ok(exception.arg2 == testsdl[i].b,
           "%s(%f, %ld) got exception arg2 %f\n", testsdl[i].func, testsdl[i].a, testsdl[i].b, exception.arg2);
    }

    d = p_acos(2.0);
    ok(d == -1.0, "failed to change log10 return value: %e\n", d);
}

int __cdecl main(void)
{
    h = LoadLibraryA("ucrtbase.dll");
#define G(v, n) (*(FARPROC *)&v = GetProcAddress(h, n))
    if (!h || !G(p_errno, "_errno") || !G(p___setusermatherr, "__setusermatherr")
        || !G(p_fesetround, "fesetround") || !G(p_acos, "acos")) {
        SAY("done FAIL (load)\n");
        return 1;
    }
    test_math_errors();
    SAY("%d checks, %d failed\n", g_ok + g_fail, g_fail);
    SAY("done %s\n", g_fail ? "FAIL" : "PASS");
    return g_fail != 0;
}
