/*
 * PROJECT:     ReactOS CRT
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     Generic C implementation of tan for ARM64
 */

#include <math.h>

#ifdef _MSC_VER
#pragma function(tan)
#endif

double tan(double x)
{
    double s = sin(x);
    double c = cos(x);
    return s / c;
}
