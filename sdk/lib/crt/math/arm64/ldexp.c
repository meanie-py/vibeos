/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of ldexp for ARM64
 */

#include <math.h>
#include <float.h>

double ldexp(double x, int n)
{
    union { double d; unsigned long long u; } u;

    if (x == 0.0 || n == 0)
        return x;

    /* Handle large positive exponents in steps */
    if (n > 1023)
    {
        x *= 0x1p1023;
        n -= 1023;
        if (n > 1023)
        {
            x *= 0x1p1023;
            n -= 1023;
            if (n > 1023)
                n = 1023;
        }
    }
    /* Handle large negative exponents in steps */
    else if (n < -1022)
    {
        x *= 0x1p-1022;
        n += 1022;
        if (n < -1022)
        {
            x *= 0x1p-1022;
            n += 1022;
            if (n < -1022)
                n = -1022;
        }
    }

    u.u = (unsigned long long)(0x3FF + n) << 52;
    return x * u.d;
}
