/* Minimal host stub for the AMD-libm reference compile (cnt-difftest).
 * libm.h pulls Windows <fpieee.h> for the OP_* opcodes and relies on the
 * MS <math.h> _DOMAIN/_OVERFLOW/... error-type constants. Both are passed
 * only to _handle_error, whose value-effect (return from_bits(value)) is
 * reproduced by the stub in build.rs — so the exact integer values here are
 * irrelevant to the diffed result; they only need to exist. */
#ifndef CNT_DIFFTEST_FPIEEE_STUB_H
#define CNT_DIFFTEST_FPIEEE_STUB_H

/* fpieee.h opcode constants used by the libm_sse2 sources we reference. */
#define OP_EXP   0
#define OP_LOG   0
#define OP_SINH  0
#define OP_COSH  0
#define OP_TANH  0
#define OP_ASIN  0
#define OP_ACOS  0
#define OP_ATAN  0
#define OP_ATAN2 0
#define OP_TAN   0
#define OP_SIN   0
#define OP_COS   0
#define OP_HYPOT 0
#define OP_POW   0
#define OP_LDEXP 0
#define OP_LOGB  0

/* MS <math.h> _exception .type values (glibc math.h does not define them). */
#ifndef _DOMAIN
#define _DOMAIN    1
#endif
#ifndef _SING
#define _SING      2
#endif
#ifndef _OVERFLOW
#define _OVERFLOW  3
#endif
#ifndef _UNDERFLOW
#define _UNDERFLOW 4
#endif
#ifndef _PLOSS
#define _PLOSS     5
#endif
#ifndef _TLOSS
#define _TLOSS     6
#endif

#endif
