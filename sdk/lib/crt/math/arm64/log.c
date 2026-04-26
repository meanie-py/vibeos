/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of log (natural logarithm) for ARM64
 *              Uses range reduction and polynomial approximation.
 */

#include <math.h>

static const double
LN2_HI = 6.93147180369123816490e-01,  /* 0x3FE62E42, 0xFEE00000 */
LN2_LO = 1.90821492927058500170e-10,  /* 0x3DEA39EF, 0x35793C76 */
/* Coefficients for log(1+s) - log(1-s) = 2s + 2/3 s^3 + 2/5 s^5 + ...
 * where s = (x-1)/(x+1) */
Lg1 = 6.666666666666735130e-01,  /* 3FE55555 55555593 */
Lg2 = 3.999999999940941908e-01,  /* 3FD99999 9997FA04 */
Lg3 = 2.857142874366239149e-01,  /* 3FD24924 94229359 */
Lg4 = 2.222219843214978396e-01,  /* 3FCC71C5 1D8E78AF */
Lg5 = 1.818357216161805012e-01,  /* 3FC74664 96CB03DE */
Lg6 = 1.531383769920937332e-01,  /* 3FC39A09 D078C69F */
Lg7 = 1.479819860511658591e-01;  /* 3FC2F112 DF3E5244 */

double log(double x)
{
    union { double d; unsigned long long u; } u;
    double hfsq, f, s, z, R, w, t1, t2, dk;
    int k, hx;
    unsigned lx;

    u.d = x;
    hx = (int)(u.u >> 32);
    lx = (unsigned)(u.u & 0xFFFFFFFF);

    k = 0;

    /* x < 2^-1022 (subnormal or negative) */
    if (hx < 0x00100000)
    {
        if (((hx & 0x7FFFFFFF) | lx) == 0)
            return -1.0 / 0.0; /* log(0) = -inf */
        if (hx < 0)
            return (x - x) / 0.0; /* log(negative) = NaN */
        /* Subnormal: normalize */
        k -= 54;
        x *= 0x1p54; /* 2^54 */
        u.d = x;
        hx = (int)(u.u >> 32);
    }

    /* Inf or NaN */
    if (hx >= 0x7FF00000)
        return x + x;

    /* log(1) = 0 exactly */
    if (hx == 0x3FF00000 && lx == 0)
        return 0.0;

    /* Reduce x to [sqrt(2)/2, sqrt(2)] */
    k += (hx >> 20) - 1023;
    hx &= 0x000FFFFF;
    /* Normalize: if x < sqrt(2)/2, multiply by 2 and decrement k */
    if (hx < 0x6A09E) /* < sqrt(2)/2 in high word */
    {
        u.u = ((unsigned long long)hx << 32) | lx;
        u.u |= 0x3FF0000000000000ULL;
    }
    else
    {
        k++;
        u.u = ((unsigned long long)hx << 32) | lx;
        u.u |= 0x3FE0000000000000ULL;
    }
    x = u.d;

    f = x - 1.0;

    /* Use log(1+f) = f - f^2/2 + ... for small f */
    if ((0x000FFFFF & (2 + hx)) < 3) /* |f| < 2^-20 */
    {
        if (f == 0.0)
        {
            dk = (double)k;
            return dk * LN2_HI + dk * LN2_LO;
        }
        R = f * f * (0.5 - 0.33333333333333333 * f);
        dk = (double)k;
        return dk * LN2_HI - ((R - dk * LN2_LO) - f);
    }

    s = f / (2.0 + f);
    dk = (double)k;
    z = s * s;
    w = z * z;
    t1 = w * (Lg2 + w * (Lg4 + w * Lg6));
    t2 = z * (Lg1 + w * (Lg3 + w * (Lg5 + w * Lg7)));
    R = t2 + t1;
    hfsq = 0.5 * f * f;
    return dk * LN2_HI - ((hfsq - (s * (hfsq + R) + dk * LN2_LO)) - f);
}
