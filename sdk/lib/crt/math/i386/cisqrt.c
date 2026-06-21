/*
 * SELF-CONTAINED / HEADERLESS (cmake-off WS1, 2026-06-21): was
 * `#include <precomp.h>` (the whole rtl/crt header chain) — now compiled
 * directly by cc-rs via reactos-link's `.c_shims()`, NOT replayed from cmake's
 * compile_commands.json. The two macros precomp.h provided (CDECL, FPU_DOUBLE)
 * are hand-defined below, identical to sdk/lib/crt/include/internal/math.h.
 * Verified symbol-identical (`_CIsqrt` T, `sqrt` U) vs the cmake object via nm.
 *
 * _CIsqrt is the MSVC x87 math intrinsic: the argument arrives on the x87 FPU
 * stack (not as a C parameter), so FPU_DOUBLE pops it into a C double, then we
 * forward to the (now-Rust) libm sqrt.
 */
#define CDECL
#define FPU_DOUBLE(var) double var; \
    __asm__ __volatile__( "fstpl %0;fwait" : "=m" (var) : )

double sqrt(double);

/*
 * @implemented
 */
double	CDECL	_CIsqrt(void)
{
	FPU_DOUBLE(x);
	return sqrt(x);
}
