/*
 * PROJECT:         ReactOS CRT library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            sdk/lib/crt/math/arm64/ceil.c
 * PURPOSE:         ARM64 ceil implementation using frintp instruction
 */

double ceil(double x)
{
    double result;
    __asm__ __volatile__("frintp %d0, %d1" : "=w" (result) : "w" (x));
    return result;
}
