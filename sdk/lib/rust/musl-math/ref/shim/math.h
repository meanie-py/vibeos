/* shim math.h: the reference build must not see the host glibc's <math.h>, whose
 * internal __sin/__cos/__tan prototypes conflict with musl's libm.h. Only what the
 * exp/log/pow sources use, from compiler builtins. */
#ifndef MUSL_REF_SHIM_MATH_H
#define MUSL_REF_SHIM_MATH_H
#include <float.h>
typedef double double_t;
typedef float float_t;
#define INFINITY __builtin_inff()
#define NAN __builtin_nanf("")
#define isnan(x) __builtin_isnan(x)
#define isinf(x) __builtin_isinf(x)
#define signbit(x) __builtin_signbit(x)
#define fabs(x) __builtin_fabs(x)
#define fabsf(x) __builtin_fabsf(x)
#endif
