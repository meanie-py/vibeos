/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of fmod for ARM64
 *              Based on fdlibm algorithm.
 */

#include <math.h>

double fmod(double x, double y)
{
    union { double d; unsigned long long u; } ux, uy;
    unsigned long long xi, yi;
    int ex, ey;
    unsigned long long sx;

    ux.d = x;
    uy.d = y;
    xi = ux.u;
    yi = uy.u;

    /* Extract sign of x */
    sx = xi & 0x8000000000000000ULL;

    /* Clear signs */
    xi &= 0x7FFFFFFFFFFFFFFFULL;
    yi &= 0x7FFFFFFFFFFFFFFFULL;

    /* Special cases */
    if (yi == 0 || xi >= 0x7FF0000000000000ULL || yi > 0x7FF0000000000000ULL)
        return (x * y) / (x * y); /* NaN */

    if (xi < yi)
        return x;

    /* Extract exponents */
    ex = (int)(xi >> 52);
    ey = (int)(yi >> 52);

    /* Normalize subnormals */
    if (ex == 0)
    {
        unsigned long long i = xi << 12;
        ex -= 11;
        while (!(i >> 63)) { i <<= 1; ex--; }
        xi <<= (1 - ex);
        ex = 0;
    }
    else
    {
        xi &= 0x000FFFFFFFFFFFFFULL;
        xi |= 0x0010000000000000ULL;
    }

    if (ey == 0)
    {
        unsigned long long i = yi << 12;
        ey -= 11;
        while (!(i >> 63)) { i <<= 1; ey--; }
        yi <<= (1 - ey);
        ey = 0;
    }
    else
    {
        yi &= 0x000FFFFFFFFFFFFFULL;
        yi |= 0x0010000000000000ULL;
    }

    /* Modular reduction */
    while (ex > ey)
    {
        if (xi >= yi)
        {
            xi -= yi;
            if (xi == 0)
            {
                ux.u = sx;
                return ux.d;
            }
        }
        xi <<= 1;
        ex--;
    }

    if (xi >= yi)
    {
        xi -= yi;
        if (xi == 0)
        {
            ux.u = sx;
            return ux.d;
        }
    }

    /* Normalize result */
    while (!(xi >> 52))
    {
        xi <<= 1;
        ex--;
    }

    /* Construct result */
    if (ex > 0)
    {
        xi -= 0x0010000000000000ULL;
        xi |= (unsigned long long)ex << 52;
    }
    else
    {
        xi >>= (1 - ex);
    }

    xi |= sx;
    ux.u = xi;
    return ux.d;
}
