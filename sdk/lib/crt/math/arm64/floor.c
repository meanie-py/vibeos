/*
 * PROJECT:         ReactOS CRT library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            sdk/lib/crt/math/arm64/floor.c
 * PURPOSE:         ARM64 floor implementation using frintm instruction
 */

double floor(double x)
{
    double result;
    __asm__ __volatile__("frintm %d0, %d1" : "=w" (result) : "w" (x));
    return result;
}
