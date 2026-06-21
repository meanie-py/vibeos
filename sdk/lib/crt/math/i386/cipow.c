/*
 * SELF-CONTAINED / HEADERLESS (cmake-off WS1, 2026-06-21): was
 * `#include <precomp.h>`; now compiled directly by cc-rs via `.c_shims()`.
 * CDECL/FPU_DOUBLES hand-defined (identical to internal/math.h). _CIpow pops
 * its TWO x87-stack args into C doubles and forwards to the (now-Rust) libm
 * pow. Verified symbol-identical (`_CIpow` T, `pow` U) vs the cmake object.
 */
#define CDECL
#define FPU_DOUBLES(var1,var2) double var1,var2; \
    __asm__ __volatile__( "fstpl %0;fwait" : "=m" (var2) : ); \
    __asm__ __volatile__( "fstpl %0;fwait" : "=m" (var1) : )

double pow(double, double);

/*
 * @implemented
 */
double	CDECL	_CIpow(void)
{
	FPU_DOUBLES(x, y);
	return pow(x, y);
}
