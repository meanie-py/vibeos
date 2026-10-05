/* Stand-in for sdk/lib/crt/math/ieee754/ieee754.h, deleted with the vendored FDLIBM it
 * declared. The Bessel wrappers include it for these six prototypes only; tests/diff.rs
 * defines them over the `libm` crate. */
#pragma once
double __ieee754_j0(double);
double __ieee754_j1(double);
double __ieee754_jn(int, double);
double __ieee754_y0(double);
double __ieee754_y1(double);
double __ieee754_yn(int, double);
