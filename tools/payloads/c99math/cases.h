/* cases.h — the c99math payload's case table, shared by the guest test (c99math.c)
 * and the host generator of its expected values (gen_expected.c).
 *
 *   D(fn, x, err, ulps)        double fn(double)      value = host glibc fn(x)
 *   D2(fn, x, y, err, ulps)    double fn(double, double)
 *   F(fn, x, err, ulps)        float fn(float)
 *   F2(fn, x, y, err, ulps)    float fn(float, float)
 *   DX(fn, x, v, err)          double fn(double)      value = v exactly (Windows
 *   D2X(fn, x, y, v, err)                             contract where glibc differs)
 *   FX(fn, x, v, err)
 *   I32(fn, x, v, err)         int/long fn(double)    value = v
 *   I64(fn, x, v, err)         __int64 fn(double)
 *   IF32 / IF64                the same from float
 *
 * `err` is the ucrtbase errno after the call, which is set to NOERR (-1) before it:
 * the C99 functions set EDOM/ERANGE and never reach _matherr. The errno column is
 * the CONTRACT (Wine's musl patch points; native-verified rows say so), not glibc's.
 * A NaN expectation matches any NaN. ulps 0 also requires the sign of a zero.
 */
#define NOERR (-1)
#define EDOM_ 33
#define ERANGE_ 34
#define INF_ __builtin_inf()
#define NAN_ __builtin_nan("")
#define INFF_ __builtin_inff()
#define NANF_ __builtin_nanf("")

/* ── inverse hyperbolics (Wine test_math_errors rows marked W) ── */
D(acosh, 1.0, NOERR, 0)                 /* W */
D(acosh, 1.5, NOERR, 1)
D(acosh, 10.0, NOERR, 1)
D(acosh, 1e300, NOERR, 1)
D(acosh, INF_, NOERR, 0)                /* W */
DX(acosh, 0.0, NAN_, EDOM_)             /* W */
DX(acosh, 0.5, NAN_, EDOM_)
DX(acosh, -INF_, NAN_, EDOM_)           /* W */
F(acoshf, 2.0f, NOERR, 1)
FX(acoshf, 0.0f, NANF_, EDOM_)
D(asinh, 0.0, NOERR, 0)
D(asinh, -0.0, NOERR, 0)
D(asinh, 0.5, NOERR, 1)
D(asinh, -2.0, NOERR, 1)
D(asinh, 1e-10, NOERR, 1)
D(asinh, 1e300, NOERR, 1)
D(asinh, INF_, NOERR, 0)                /* W */
D(asinh, -INF_, NOERR, 0)               /* W */
F(asinhf, -3.0f, NOERR, 1)
D(atanh, 0.0, NOERR, 0)
D(atanh, 0.5, NOERR, 1)
D(atanh, -0.9, NOERR, 1)
D(atanh, 1e-10, NOERR, 1)
DX(atanh, 1.0, INF_, ERANGE_)           /* W */
DX(atanh, -1.0, -INF_, ERANGE_)         /* W */
DX(atanh, 2.0, NAN_, EDOM_)             /* W */
DX(atanh, -2.0, NAN_, EDOM_)            /* W */
DX(atanh, INF_, NAN_, EDOM_)            /* W */
DX(atanh, -INF_, NAN_, EDOM_)           /* W */
F(atanhf, 0.25f, NOERR, 1)
FX(atanhf, 1.0f, INFF_, ERANGE_)
FX(atanhf, 3.0f, NANF_, EDOM_)

/* ── cbrt, erf, erfc ── */
DX(cbrt, 27.0, 3.0, NOERR)              /* exact; glibc says 3 + 1ulp */
DX(cbrt, -8.0, -2.0, NOERR)
D(cbrt, 0.001, NOERR, 1)
D(cbrt, 3.0, NOERR, 1)
D(cbrt, 1e-310, NOERR, 1)
D(cbrt, -0.0, NOERR, 0)
D(cbrt, INF_, NOERR, 0)
FX(cbrtf, 64.0f, 4.0f, NOERR)
F(cbrtf, 2.0f, NOERR, 1)
D(erf, 0.0, NOERR, 0)
D(erf, 1e-20, NOERR, 1)
D(erf, 0.5, NOERR, 1)
D(erf, -1.0, NOERR, 1)
D(erf, 3.0, NOERR, 1)
D(erf, 6.0, NOERR, 0)
D(erf, -INF_, NOERR, 0)
F(erff, 0.75f, NOERR, 1)
D(erfc, 0.0, NOERR, 0)
D(erfc, 0.5, NOERR, 1)
D(erfc, -1.0, NOERR, 1)
D(erfc, 3.0, NOERR, 2)
D(erfc, 10.0, NOERR, 2)
D(erfc, 26.0, NOERR, 2)
DX(erfc, 28.0, 0.0, ERANGE_)
DX(erfc, 30.0, 0.0, ERANGE_)
D(erfc, -30.0, NOERR, 0)
D(erfc, INF_, NOERR, 0)
D(erfc, -INF_, NOERR, 0)
F(erfcf, 1.5f, NOERR, 2)
FX(erfcf, 30.0f, 0.0f, ERANGE_)

/* ── expm1, log1p ── */
D(expm1, 0.0, NOERR, 0)
D(expm1, 1e-10, NOERR, 1)
D(expm1, 1.0, NOERR, 1)
D(expm1, -1.0, NOERR, 1)
D(expm1, 50.0, NOERR, 1)
D(expm1, -50.0, NOERR, 1)
D(expm1, INF_, NOERR, 0)                /* W */
D(expm1, -INF_, NOERR, 0)               /* W */
F(expm1f, 0.5f, NOERR, 1)
D(log1p, 0.0, NOERR, 0)
D(log1p, 1e-10, NOERR, 1)
D(log1p, 1.0, NOERR, 1)
D(log1p, -0.5, NOERR, 1)
D(log1p, 1e300, NOERR, 1)
D(log1p, INF_, NOERR, 0)                /* W */
DX(log1p, -1.0, -INF_, ERANGE_)         /* W */
DX(log1p, -2.0, NAN_, EDOM_)            /* W */
DX(log1p, -INF_, NAN_, EDOM_)           /* W */
F(log1pf, 3.0f, NOERR, 1)
FX(log1pf, -1.0f, -INFF_, ERANGE_)
FX(log1pf, -5.0f, NANF_, EDOM_)

/* ── log2 (was log * M_LOG2E in stubs.c) ── */
DX(log2, 8.0, 3.0, NOERR)
DX(log2, 0.125, -3.0, NOERR)
DX(log2, 1024.0, 10.0, NOERR)
D(log2, 3.0, NOERR, 1)
D(log2, 1e-310, NOERR, 1)
D(log2, INF_, NOERR, 0)                 /* W */
DX(log2, 0.0, -INF_, ERANGE_)
DX(log2, -1.0, NAN_, EDOM_)
DX(log2, -INF_, NAN_, EDOM_)
FX(log2f, 32.0f, 5.0f, NOERR)
FX(log2f, 0.0f, -INFF_, ERANGE_)
FX(log2f, -2.0f, NANF_, EDOM_)

/* ── gamma ── */
D(lgamma, 1.0, NOERR, 0)
D(lgamma, 2.0, NOERR, 0)
D(lgamma, 0.5, NOERR, 2)
D(lgamma, 3.5, NOERR, 2)
D(lgamma, 100.0, NOERR, 2)
D(lgamma, -0.5, NOERR, 2)
D(lgamma, -2.5, NOERR, 16)              /* near a zero of lgamma: musl is ~10 ulp from exact, glibc 2.5 */
D(lgamma, 1e-300, NOERR, 2)
D(lgamma, 1e305, NOERR, 2)
DX(lgamma, 1e308, INF_, ERANGE_)
DX(lgamma, 0.0, INF_, ERANGE_)
DX(lgamma, -1.0, INF_, ERANGE_)
DX(lgamma, -3.0, INF_, ERANGE_)
D(lgamma, INF_, NOERR, 0)
D(lgamma, -INF_, NOERR, 0)
F(lgammaf, 4.5f, NOERR, 2)
FX(lgammaf, -2.0f, INFF_, ERANGE_)
D(tgamma, 1.0, NOERR, 0)
D(tgamma, 5.0, NOERR, 0)
D(tgamma, 0.5, NOERR, 2)
D(tgamma, 10.5, NOERR, 4)
D(tgamma, 170.0, NOERR, 4)
D(tgamma, -0.5, NOERR, 4)
D(tgamma, -1.5, NOERR, 4)
DX(tgamma, 172.0, INF_, ERANGE_)
DX(tgamma, 200.0, INF_, ERANGE_)
DX(tgamma, 0.0, INF_, ERANGE_)
DX(tgamma, -0.0, -INF_, ERANGE_)
DX(tgamma, -1.0, NAN_, EDOM_)
DX(tgamma, -INF_, NAN_, EDOM_)
D(tgamma, INF_, NOERR, 0)
FX(tgammaf, 4.0f, 6.0f, NOERR)           /* exact; glibc says 6 + 1ulp */
FX(tgammaf, 2.5f, 0x1.544fa6p+0f, NOERR)  /* correctly rounded (sqrt(pi)*3/4); glibc is 2 ulp off */
FX(tgammaf, 40.0f, INFF_, ERANGE_)
FX(tgammaf, -3.0f, NANF_, EDOM_)

/* ── exponent extraction: Microsoft's FP_ILOGB0 = INT_MIN, FP_ILOGBNAN = INT_MAX ── */
I32(ilogb, 1.0, 0, NOERR)
I32(ilogb, 8.0, 3, NOERR)
I32(ilogb, 0.1, -4, NOERR)
I32(ilogb, 1e-310, -1030, NOERR)
I32(ilogb, 0.0, -0x7fffffff - 1, NOERR)
I32(ilogb, INF_, 0x7fffffff, NOERR)
I32(ilogb, NAN_, 0x7fffffff, NOERR)
IF32(ilogbf, 0.1f, -4, NOERR)
IF32(ilogbf, NANF_, 0x7fffffff, NOERR)
DX(logb, 8.0, 3.0, NOERR)
DX(logb, 0.1, -4.0, NOERR)
DX(logb, 1e-310, -1030.0, NOERR)
DX(logb, 0.0, -INF_, ERANGE_)
DX(logb, INF_, INF_, NOERR)
FX(logbf, 0.1f, -4.0f, NOERR)
FX(logbf, 0.0f, -INFF_, ERANGE_)

/* ── rounding (default mode; the directed modes are tested separately) ── */
DX(rint, 1.5, 2.0, NOERR)
DX(rint, 2.5, 2.0, NOERR)
DX(rint, -0.5, -0.0, NOERR)
DX(rint, 0.2, 0.0, NOERR)
DX(rint, -3.7, -4.0, NOERR)
DX(rint, 1e300, 1e300, NOERR)
DX(rint, 4503599627370495.5, 4503599627370496.0, NOERR)
FX(rintf, 2.5f, 2.0f, NOERR)
FX(rintf, 8388607.5f, 8388608.0f, NOERR)
DX(nearbyint, 2.5, 2.0, NOERR)
DX(nearbyint, -1.5, -2.0, NOERR)
FX(nearbyintf, 3.5f, 4.0f, NOERR)
DX(trunc, 1.9, 1.0, NOERR)
DX(trunc, -1.9, -1.0, NOERR)
DX(trunc, -0.2, -0.0, NOERR)
DX(trunc, 1e300, 1e300, NOERR)
FX(truncf, -2.7f, -2.0f, NOERR)
I32(lround, 2.5, 3, NOERR)
I32(lround, -2.5, -3, NOERR)
I32(lround, 0.49999999999999994, 0, NOERR)
I32(lround, 2147483647.4, 2147483647, NOERR)
I32(lround, 2147483647.5, 0, EDOM_)
I32(lround, -2147483648.4, -0x7fffffff - 1, NOERR)
I32(lround, -2147483648.5, 0, EDOM_)
I32(lround, NAN_, 0, EDOM_)
I32(lround, INF_, 0, EDOM_)
IF32(lroundf, 2147483520.0f, 2147483520, NOERR)
IF32(lroundf, 2147483648.0f, 0, EDOM_)  /* a saturating cast would say INT_MAX */
IF32(lroundf, -2147483648.0f, -0x7fffffff - 1, NOERR)
I64(llround, 2.5, 3, NOERR)
I64(llround, 9.2e18, 9200000000000000000LL, NOERR)
I64(llround, 9223372036854775808.0, 0, EDOM_)
I64(llround, -9223372036854775808.0, -0x7fffffffffffffffLL - 1, NOERR)
I64(llround, NAN_, 0, EDOM_)
IF64(llroundf, -3.5f, -4, NOERR)
IF64(llroundf, 9223372036854775808.0f, 0, EDOM_)
I32(lrint, 2.5, 2, NOERR)               /* was __debugbreak() in stubs.c */
I32(lrint, -3.5, -4, NOERR)
I32(lrint, 2147483647.0, 2147483647, NOERR)
I32(lrint, -2147483648.0, -0x7fffffff - 1, NOERR)
I32(lrint, 4294967296.0, 0, EDOM_)
I32(lrint, -2147483649.0, 0, EDOM_)
I32(lrint, NAN_, 0, EDOM_)
IF32(lrintf, 1.5f, 2, NOERR)
I64(llrint, 2.5, 2, NOERR)
I64(llrint, -9223372036854775808.0, -0x7fffffffffffffffLL - 1, NOERR)
I64(llrint, 2e19, 0, EDOM_)
I64(llrint, -1e19, 0, EDOM_)
I64(llrint, NAN_, 0, EDOM_)
IF64(llrintf, 3.5f, 4, NOERR)

/* ── remainder: Wine's msvcr120 test_remainder, native-verified (64-bit) ── */
D2X(remainder, 3.0, 2.0, -1.0, NOERR)
D2X(remainder, 1.0, 1.0, 0.0, NOERR)
D2X(remainder, INF_, 0.0, NAN_, EDOM_)
D2X(remainder, INF_, 42.0, NAN_, EDOM_)
D2X(remainder, NAN_, 0.0, NAN_, EDOM_)
D2X(remainder, NAN_, 42.0, NAN_, EDOM_)
D2X(remainder, 0.0, INF_, 0.0, NOERR)
D2X(remainder, 42.0, INF_, 42.0, NOERR)
D2X(remainder, 0.0, NAN_, NAN_, EDOM_)
D2X(remainder, 42.0, NAN_, NAN_, EDOM_)
D2X(remainder, 1.0, 0.0, NAN_, EDOM_)
D2X(remainder, INF_, INF_, NAN_, EDOM_)
D2(remainder, 10.3, 3.1, NOERR, 0)
F2(remainderf, 7.0f, 2.0f, NOERR, 0)

/* ── fmin/fmax/fdim (C99 F.9.9: NaN loses, -0 < +0), copysign ── */
D2X(fmin, 1.0, 2.0, 1.0, NOERR)
D2X(fmin, NAN_, 1.0, 1.0, NOERR)
D2X(fmin, 1.0, NAN_, 1.0, NOERR)
D2X(fmin, -0.0, 0.0, -0.0, NOERR)
D2X(fmin, 0.0, -0.0, -0.0, NOERR)
D2X(fmax, -0.0, 0.0, 0.0, NOERR)
D2X(fmax, 0.0, -0.0, 0.0, NOERR)
D2X(fmax, -INF_, NAN_, -INF_, NOERR)
D2X(fmax, 3.0, -7.0, 3.0, NOERR)
D2X(fdim, 5.0, 3.0, 2.0, NOERR)
D2X(fdim, 3.0, 5.0, 0.0, NOERR)
D2X(fdim, NAN_, 1.0, NAN_, NOERR)
D2X(fdim, INF_, INF_, 0.0, NOERR)
D2X(copysign, 1.0, -0.0, -1.0, NOERR)
D2X(copysign, -2.0, 3.0, 2.0, NOERR)
D2X(copysign, INF_, -1.0, -INF_, NOERR)

/* ── C89 pow / log10 / fmod: these were upstream placeholders in cnt-rust (pow(2, 0.5) == 0,
 * log10 a linear guess that never returned for inf, fmod looping on a subnormal divisor).
 * Value rows only: their errno/_matherr contract is still owed (roadmap). Also run against
 * msvcrt.dll, which shipped the same bodies. ── */
D2(pow, 2.0, 0.5, NOERR, 1)
D2(pow, 8.0, 0.3333333333333333, NOERR, 1)
D2(pow, 1.5, 2.5, NOERR, 1)
D2(pow, 10.0, -0.5, NOERR, 1)
D2(pow, 2.0, 10.0, NOERR, 0)
D2(pow, -2.0, 3.0, NOERR, 0)
D2(pow, 7.0, 0.1, NOERR, 1)
D2(pow, 100.0, 1.5, NOERR, 0)
F2(powf, 2.0f, 0.5f, NOERR, 1)
D(log10, 2.0, NOERR, 1)
D(log10, 50.0, NOERR, 1)
D(log10, 1000.0, NOERR, 0)
D(log10, 0.3, NOERR, 1)
D(log10, 1e-300, NOERR, 1)
D(log10, INF_, NOERR, 0)
D2(fmod, 10.3, 3.0, NOERR, 0)
D2(fmod, -7.5, 2.0, NOERR, 0)
D2(fmod, 1.0, 1.5e-323, NOERR, 0)
D2(fmod, 1e300, 3.0, NOERR, 0)
F2(fmodf, 7.5f, 2.0f, NOERR, 0)
