/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of atan2 for ARM64
 */

#include <math.h>

static const double
pi     = 3.14159265358979311600e+00, /* 0x400921FB, 0x54442D18 */
pi_lo  = 1.22464679914735320717e-16, /* 0x3CA1A626, 0x33145C07 */
pi_2   = 1.57079632679489655800e+00, /* 0x3FF921FB, 0x54442D18 */
pi_4   = 7.85398163397448278999e-01; /* 0x3FE921FB, 0x54442D18 */

double atan2(double y, double x)
{
    double z;
    int k, m, hx, hy, ix, iy;
    unsigned lx, ly;
    union { double d; unsigned long long u; } ux, uy;

    ux.d = x;
    uy.d = y;
    hx = (int)(ux.u >> 32);
    lx = (unsigned)(ux.u & 0xFFFFFFFF);
    hy = (int)(uy.u >> 32);
    ly = (unsigned)(uy.u & 0xFFFFFFFF);

    ix = hx & 0x7FFFFFFF;
    iy = hy & 0x7FFFFFFF;

    /* x or y is NaN */
    if (((ix | ((lx | (unsigned)(-((int)lx))) >> 31)) > 0x7FF00000) ||
        ((iy | ((ly | (unsigned)(-((int)ly))) >> 31)) > 0x7FF00000))
        return x + y;

    /* x = 1.0 */
    if (hx == 0x3FF00000 && lx == 0)
        return atan(y);

    /* 2 * sign(x) + sign(y) */
    m = ((hy >> 31) & 1) | ((hx >> 31) & 2);

    /* y = 0 */
    if ((iy | ly) == 0)
    {
        switch (m)
        {
        case 0:
        case 1: return y;            /* atan(+-0, +anything) = +-0 */
        case 2: return  pi;          /* atan(+0, -anything) = pi */
        case 3: return -pi;          /* atan(-0, -anything) = -pi */
        }
    }

    /* x = 0 */
    if ((ix | lx) == 0)
        return (hy < 0) ? -pi_2 : pi_2;

    /* x = inf */
    if (ix == 0x7FF00000)
    {
        if (iy == 0x7FF00000)
        {
            switch (m)
            {
            case 0: return  pi_4;        /* atan(+inf, +inf) */
            case 1: return -pi_4;        /* atan(-inf, +inf) */
            case 2: return  3.0 * pi_4;  /* atan(+inf, -inf) */
            case 3: return -3.0 * pi_4;  /* atan(-inf, -inf) */
            }
        }
        else
        {
            switch (m)
            {
            case 0: return  0.0;  /* atan(+..., +inf) */
            case 1: return -0.0;  /* atan(-..., +inf) */
            case 2: return  pi;   /* atan(+..., -inf) */
            case 3: return -pi;   /* atan(-..., -inf) */
            }
        }
    }

    /* y = inf */
    if (iy == 0x7FF00000)
        return (hy < 0) ? -pi_2 : pi_2;

    /* Compute y/x */
    k = (iy - ix) >> 20;

    if (k > 60) /* |y/x| > 2^60 */
        z = pi_2 + 0.5 * pi_lo;
    else if (hx < 0 && k < -60) /* 0 > |y|/x > -2^-60 */
        z = 0.0;
    else
        z = atan(fabs(y / x));

    switch (m)
    {
    case 0: return  z;                           /* atan(+, +) */
    case 1: return -z;                           /* atan(-, +) */
    case 2: return  pi - (z - pi_lo);            /* atan(+, -) */
    default: return (z - pi_lo) - pi;            /* atan(-, -) */
    }
}
