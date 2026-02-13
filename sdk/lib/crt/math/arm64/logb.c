/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of logb/logbf for ARM64
 */

#include <math.h>
#include <float.h>

/* _logb - MSVC-style name used by msvcrt */
double _logb(double x)
{
    union { double d; unsigned long long u; } u;
    int exp;

    u.d = x;
    /* Clear sign bit */
    u.u &= 0x7FFFFFFFFFFFFFFFULL;

    /* Zero */
    if (u.u == 0)
        return -1.0 / 0.0; /* -infinity */

    /* Extract exponent field */
    exp = (int)(u.u >> 52);

    /* Infinity or NaN */
    if (exp == 0x7FF)
        return x * x; /* +inf for inf, NaN for NaN */

    /* Subnormal: normalize */
    if (exp == 0)
    {
        u.d *= 0x1p64;
        exp = (int)(u.u >> 52) - 64;
    }

    return (double)(exp - 0x3FF);
}

/* _logbf - MSVC-style name used by msvcrt */
float _logbf(float x)
{
    union { float f; unsigned int u; } u;
    int exp;

    u.f = x;
    u.u &= 0x7FFFFFFFU;

    if (u.u == 0)
        return -1.0f / 0.0f;

    exp = (int)(u.u >> 23);

    if (exp == 0xFF)
        return x * x;

    if (exp == 0)
    {
        u.f *= 0x1p25f;
        exp = (int)(u.u >> 23) - 25;
    }

    return (float)(exp - 0x7F);
}
