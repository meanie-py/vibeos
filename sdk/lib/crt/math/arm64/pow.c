/*
 * PROJECT:         ReactOS CRT library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            sdk/lib/crt/math/arm64/pow.c
 * PURPOSE:         ARM64 pow stub
 */

#include <math.h>

/*
 * Stub implementation of pow for ARM64 kernel CRT.
 * Only handles simple integer exponents for printf float formatting.
 */
double pow(double base, double exponent)
{
    double result = 1.0;
    int i, exp;

    /* Handle common cases for printf float formatting */
    if (exponent == 0.0)
        return 1.0;
    if (base == 0.0)
        return 0.0;
    if (base == 1.0)
        return 1.0;

    /* Integer exponent path (sufficient for printf) */
    exp = (int)exponent;
    if ((double)exp == exponent && exp >= 0)
    {
        for (i = 0; i < exp; i++)
            result *= base;
        return result;
    }
    if ((double)exp == exponent && exp < 0)
    {
        exp = -exp;
        for (i = 0; i < exp; i++)
            result *= base;
        return 1.0 / result;
    }

    /* FIXME: General case not yet implemented */
    return 0.0;
}
