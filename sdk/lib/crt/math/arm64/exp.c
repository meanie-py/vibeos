/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of exp for ARM64
 *              Uses range reduction and polynomial approximation.
 */

#include <math.h>

static const double
LN2_HI  = 6.93147180369123816490e-01,   /* 0x3FE62E42, 0xFEE00000 */
LN2_LO  = 1.90821492927058500170e-10,   /* 0x3DEA39EF, 0x35793C76 */
INV_LN2 = 1.44269504088896338700e+00;   /* 0x3FF71547, 0x652B82FE */

/* Coefficients for the approximation of exp(r)-1 on [-ln2/2, ln2/2] */
static const double
P1 =  1.66666666666666019037e-01,  /* 0x3FC55555, 0x5555553E */
P2 = -2.77777777770155933842e-03,  /* 0xBF66C16C, 0x16BEBD93 */
P3 =  6.61375632143793436117e-05,  /* 0x3F11566A, 0xAF25DE2C */
P4 = -1.65339022054652515390e-06,  /* 0xBEBBBD41, 0xC5D26BF1 */
P5 =  4.13813679705723846039e-08;  /* 0x3E663769, 0x72BEA4D0 */

double exp(double x)
{
    double hi, lo, r, c, y;
    int k;
    union { double d; unsigned long long u; } u;
    unsigned hx;

    u.d = x;
    hx = (unsigned)(u.u >> 32) & 0x7FFFFFFF;

    /* Filter special cases */
    if (hx >= 0x40862E42) /* |x| >= 709.78... */
    {
        if (hx >= 0x7FF00000)
        {
            /* NaN or Inf */
            unsigned lx = (unsigned)(u.u & 0xFFFFFFFF);
            if (((hx & 0xFFFFF) | lx) != 0)
                return x; /* NaN */
            return (u.u >> 63) ? 0.0 : x; /* exp(-inf)=0, exp(inf)=inf */
        }
        if (x > 709.782712893383973096)
        {
            /* Overflow */
            u.u = 0x7FF0000000000000ULL;
            return u.d;
        }
        if (x < -745.13321910194110842)
            return 0.0; /* Underflow */
    }

    /* Argument reduction: r = x - k*ln2, |r| <= ln2/2 */
    if (hx > 0x3FD62E42) /* |x| > 0.5*ln2 */
    {
        if (hx < 0x3FF0A2B2) /* |x| < 1.5*ln2 */
        {
            if (!(u.u >> 63))
            {
                hi = x - LN2_HI;
                lo = LN2_LO;
                k = 1;
            }
            else
            {
                hi = x + LN2_HI;
                lo = -LN2_LO;
                k = -1;
            }
        }
        else
        {
            k = (int)(INV_LN2 * x + ((u.u >> 63) ? -0.5 : 0.5));
            hi = x - k * LN2_HI;
            lo = k * LN2_LO;
        }
        r = hi - lo;
    }
    else if (hx < 0x3E300000) /* |x| < 2^-28 */
    {
        return 1.0 + x;
    }
    else
    {
        k = 0;
        r = x;
    }

    /* Compute exp(r) using rational approximation */
    {
        double t = r * r;
        c = r - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
        y = 1.0 - ((r * c) / (c - 2.0) - r);
    }

    /* Scale by 2^k */
    if (k == 0)
        return y;

    if (k > 0 && k < 1024)
    {
        u.u = (unsigned long long)(0x3FF + k) << 52;
        return y * u.d;
    }
    else if (k < 0 && k > -1023)
    {
        u.u = (unsigned long long)(0x3FF + k) << 52;
        return y * u.d;
    }
    else
    {
        /* Handle extreme exponents */
        u.d = y;
        u.u += (unsigned long long)k << 52;
        return u.d;
    }
}
