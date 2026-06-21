/*
 * SELF-CONTAINED / HEADERLESS (cmake-off WS1, 2026-06-21): was
 * `#include <precomp.h>`; now compiled directly by cc-rs via `.c_shims()`.
 * CDECL/FPU_DOUBLE hand-defined (identical to internal/math.h). _CIsin pops
 * its x87-stack arg into a C double and forwards to the (now-Rust) libm sin.
 * Verified symbol-identical (`_CIsin` T, `sin` U) vs the cmake object via nm.
 */
#define CDECL
#define FPU_DOUBLE(var) double var; \
    __asm__ __volatile__( "fstpl %0;fwait" : "=m" (var) : )

double sin(double);

/*
 * @implemented
 */
double	CDECL	_CIsin(void)
{
	FPU_DOUBLE(x);
	return sin(x);
}
