/*
 * PROJECT:         ReactOS CRT library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            sdk/lib/crt/math/arm64/log10.c
 * PURPOSE:         ARM64 log10 stub
 */

/*
 * Stub implementation of log10 for ARM64 kernel CRT.
 * FIXME: Provide proper implementation.
 */
double log10(double x)
{
    /* Minimal implementation for printf float formatting */
    if (x <= 0.0)
        return -1.0e308; /* -infinity approximation */

    /* Rough integer log10 via repeated division */
    {
        double result = 0.0;
        double temp = x;

        if (temp >= 1.0)
        {
            while (temp >= 10.0)
            {
                temp /= 10.0;
                result += 1.0;
            }
            /* Linear interpolation for fractional part */
            result += (temp - 1.0) / 9.0;
        }
        else
        {
            while (temp < 1.0)
            {
                temp *= 10.0;
                result -= 1.0;
            }
            result += (temp - 1.0) / 9.0;
        }

        return result;
    }
}
