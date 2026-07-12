/*
 * Minimal <msvcrt.h> for compiling the GENUINE dll/win32/msvcrt/string.c
 * on the host (linux x86_64, glibc).
 *
 * THE GUARD IS THE TRICK.  We deliberately reuse the include guard of the
 * real dll/win32/msvcrt/msvcrt.h (`__WINE_MSVCRT_H`).  string.c does
 * `#include "msvcrt.h"`, and a *quoted* include always searches the
 * directory of the including file first — i.e. it resolves to the REAL
 * 464-line header, which drags in windef.h/winbase.h/winnls.h and the
 * whole NT/PSDK chain.  Because build.rs force-includes THIS file first
 * (`-include ref_prelude.h` → `#include <msvcrt.h>` off the refstub
 * include path), the guard is already set by the time string.c gets
 * there, so the real header expands to nothing and we escape the chain
 * WITHOUT touching a single line of the reference source.
 *
 * The same does NOT apply to "bnum.h": that header is REAL LOGIC (the
 * base-10 bignum used by strtod/fpnum_parse), so we let string.c pull the
 * genuine dll/win32/msvcrt/bnum.h and merely supply the types it needs
 * (DWORD/BOOL/ULONGLONG/assert) from here.  Stubbing bnum.h would have
 * silently replaced the very algorithm under test.
 *
 * "winnls.h", "wine/asm.h" and "wine/debug.h" have no same-dir file, so
 * they fall through to the refstub include path and land on our stubs.
 */
#ifndef __WINE_MSVCRT_H
#define __WINE_MSVCRT_H

/* ------------------------------------------------------------------ *
 * Host libc.  Pull EVERY header string.c would pull (assert/stdlib/
 * stdio/math/limits/locale/float) plus the ones we need, BEFORE the
 * `wchar_t` remap below — no host header may ever be parsed under the
 * 16-bit wchar_t model.  string.c's own #includes then no-op on their
 * guards.
 * ------------------------------------------------------------------ */
#include <assert.h>
#include <ctype.h>
#include <errno.h>
#include <float.h>
#include <limits.h>
#include <locale.h>
#include <math.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ------------------------------------------------------------------ *
 * Windows type model
 * ------------------------------------------------------------------ */

/* Windows wchar_t is 16-bit; the host's is 32-bit.  Everything below
 * (and all of string.c) sees the Windows width, so `_itow`/`_ltow`/
 * `_i64tow` produce byte-identical buffers to a `*mut u16` Rust shim. */
#define wchar_t unsigned short

/* Calling conventions: no-ops on the SysV host. */
#define CDECL
#define __cdecl
#define WINAPI
#define __stdcall

#define __int64 long long   /* macro, not typedef, so `unsigned __int64` parses */

typedef int                 BOOL;
#ifndef TRUE
#define TRUE  1
#define FALSE 0
#endif
typedef unsigned char       UCHAR;
typedef unsigned char       BYTE;
typedef unsigned short      WORD;
typedef unsigned int        DWORD;   /* Windows DWORD is 32-bit */
typedef unsigned int        ULONG;
typedef int                 LONG;
typedef unsigned int        UINT;
typedef long long           LONGLONG;
typedef unsigned long long  ULONGLONG;
typedef unsigned int        LCID;
typedef unsigned short      WCHAR;
typedef WCHAR              *PWCHAR;
typedef void               *HANDLE;
typedef void               *HMODULE;

/* corecrt.h: "we need 32-bit longs even on 64-bit" — same rule here, the
 * host is LP64 so Windows `long` is `int`.  This is what makes
 * `_ltoa(-1, buf, 16)` produce "ffffffff" (32-bit unsigned reinterpret)
 * instead of the host's 64-bit "ffffffffffffffff". */
typedef int                 __msvcrt_long;
typedef unsigned int        __msvcrt_ulong;

/* …and the matching limit macros.  <limits.h> above gave us the LP64
 * values; string.c's clamps (_atol_l / _strtol_l / _strtoul_l) must see
 * the Windows 32-bit ones or they become no-ops.
 *
 * ULONG_MAX is deliberately a SIGNED 64-bit constant: `__int64 ret >
 * ULONG_MAX` has to stay a signed compare (on Win32, `unsigned long` is
 * 32-bit and promotes to __int64).  0xFFFFFFFFUL would be 64-bit
 * unsigned on the host and would wrongly clamp every negative value. */
#undef  LONG_MAX
#define LONG_MAX  0x7FFFFFFFL
#undef  LONG_MIN
#define LONG_MIN  (-0x7FFFFFFFL - 1)
#undef  ULONG_MAX
#define ULONG_MAX 0xFFFFFFFFLL

typedef unsigned long   ULONG_PTR;   /* pointer-sized, host LP64 */
typedef long            LONG_PTR;

/* ------------------------------------------------------------------ *
 * THE ANTI-CIRCULARITY NAMESPACE  (must precede every declaration below,
 * and every host header above, in that order — see build.rs)
 *
 * The finished test binary links the reference objects TOGETHER WITH
 * cnt-rust's `#[no_mangle]` shims, and cnt-rust exports `memmove`,
 * `strlen`, `tolower`, `floor`, `log10`, `_invalid_parameter`, … .  A
 * static definition beats the shared libc at link time, so:
 *
 *   - an un-namespaced `memmove` call inside `ref_memcpy` would silently
 *     bind to *the code under test* — the reference would be grading
 *     itself — and
 *   - a second `_invalid_parameter` definition is a hard duplicate-symbol
 *     link error.
 *
 * So every symbol the reference IMPORTS from "the CRT" is renamed into a
 * private `msvcrt_ref_*` namespace right here.  Both TUs (the string.c TU
 * and refimpl.c) see this header, so both agree on the renamed names, and
 * nothing called `_errno` / `memmove` / `get_locinfo` / … escapes the crate.
 *
 * It sits ABOVE the declarations on purpose: renaming a name that has
 * already been declared leaves the *call sites* renamed but unprototyped,
 * and C's implicit-declaration rule then invents `int f()` — which
 * truncates a returned pointer to 32 bits.  (That is exactly how
 * `get_locinfo()` first came back as a garbage pointer here.)
 *
 * Deliberately NOT namespaced — see the harness notes in src/lib.rs:
 *   floor, log10   only I10_OUTPUT uses them; sharing one libm between
 *                  reference and subject is the standard difftest move —
 *                  it factors libm exactness out of the comparison.
 *   sprintf        I10_OUTPUT's "%.*le"; the host printf engine.
 *   malloc         _strdup only needs *a* heap.
 * ------------------------------------------------------------------ */
#define memmove                 msvcrt_ref_memmove
#define _errno                  msvcrt_ref__errno
#define _invalid_parameter      msvcrt_ref__invalid_parameter
#define _isspace_l              msvcrt_ref__isspace_l
#define _tolower_l              msvcrt_ref__tolower_l
#define _toupper_l              msvcrt_ref__toupper_l
#define get_locinfo             msvcrt_ref_get_locinfo
#define msvcrt_get_thread_data  msvcrt_ref_get_thread_data
#define CompareStringA          msvcrt_ref_CompareStringA
#define LCMapStringA            msvcrt_ref_LCMapStringA
#define ros_dbg_log             msvcrt_ref_ros_dbg_log
#define wine_dbgstr_an          msvcrt_ref_wine_dbgstr_an

void *__cdecl memmove(void *dst, const void *src, size_t n);

/* winnt.h.  ARRAY_SIZE is load-bearing: strstr()'s KMP fallback sizes its
 * `lps` table with it.  Miss it and C's implicit-function-declaration rule
 * turns `ARRAY_SIZE(lps)` into a *call* to an undefined `int ARRAY_SIZE()`
 * — a silently wrong strstr that still links.  build.rs promotes
 * implicit-function-declaration to an error precisely so a future missing
 * macro can't do that again. */
#define ARRAY_SIZE(x)     (sizeof(x) / sizeof((x)[0]))
#define DECLSPEC_ALIGN(x) __attribute__((aligned(x)))
#define FIELD_OFFSET(t,f) ((LONG)__builtin_offsetof(t,f))

/* MSVC secure-CRT bits (crtdefs.h / errno.h) */
#define _TRUNCATE   ((size_t)-1)
#define STRUNCATE   80
#define _NLSCMPERROR ((unsigned int)0x7fffffff)

/* dll/win32/msvcrt/include/limits.h — the 64-bit limits, and the
 * un-underscored aliases string.c/bnum.h actually use. */
#define _I64_MAX    0x7fffffffffffffffLL
#define _I64_MIN    (-_I64_MAX - 1)
#define _UI64_MAX   0xffffffffffffffffULL
#define I64_MIN     _I64_MIN
#define I64_MAX     _I64_MAX
#define UI64_MAX    _UI64_MAX

/* dll/win32/msvcrt/include/math.h — _atodbl/_atoflt/_atoldbl return codes */
#define _OVERFLOW   3
#define _UNDERFLOW  4

/* dll/win32/msvcrt/include/wchar.h — MSVC's wint_t is 16-bit, so WEOF is
 * 0xFFFF (NOT glibc's 32-bit 0xFFFFFFFF).  fpnum_parse compares its
 * `wchar_t` getter result against it. */
typedef unsigned short wint_t;
#define WEOF ((wint_t)0xFFFF)

/* stdlib.h (MSVC): _atoflt/_atodbl/_atoldbl out-params */
typedef struct { float  f; } _CRT_FLOAT;
typedef struct { double x; } _CRT_DOUBLE;
typedef struct { unsigned char ld[10]; } _LDOUBLE;

/* msvcrt.h (verbatim decls, __REACTOS__ flavour) */
#define DBL80_MAX_10_EXP 4932
#define DBL80_MIN_10_EXP -4951
typedef struct { UCHAR ld[10]; } MSVCRT__LDOUBLE;      /* real x87 80-bit: 10 bytes */
typedef struct { ULONG x80[3]; } WINE_BROKEN_LDOUBLE;  /* what wine's code writes */

/* ------------------------------------------------------------------ *
 * Locale model — the DEFAULT "C" locale, faithfully.
 *
 * The real threadlocinfo is huge; string.c touches exactly three fields:
 *   locinfo->lc_handle[LC_CTYPE]     (_strlwr_l / _strupr_l family)
 *   locinfo->lc_handle[LC_COLLATE]   (_strcoll_l / _stricoll_l / _strxfrm_l)
 *   locinfo->lconv->decimal_point    (fpnum_parse / strtod)
 *
 * In the C locale every lc_handle[] is 0, and string.c *branches on that*:
 * `if(!locinfo->lc_handle[LC_COLLATE]) return strcmp(...)`.  So the C
 * locale exercises the genuine non-NLS code paths and CompareStringA /
 * LCMapStringA are dead code (they still have to link — see refimpl.c).
 *
 * lc_handle is oversized (16) because the LC_* indices in scope here come
 * from the HOST <locale.h> (LC_CTYPE=0 … LC_IDENTIFICATION=12), not from
 * MSVC's <locale.h> (LC_ALL=0 … LC_TIME=5).  The values are all zero
 * either way, so the index mapping is immaterial — the array just has to
 * be big enough not to read out of bounds.
 * ------------------------------------------------------------------ */
typedef struct threadlocaleinfostruct {
    int             refcount;
    unsigned int    lc_codepage;
    unsigned int    lc_collate_cp;
    LCID            lc_handle[16];
    struct lconv   *lconv;
    int             mb_cur_max;
} threadlocinfo, *pthreadlocinfo;

typedef struct threadmbcinfostruct {
    int             refcount;
    int             mbcodepage;
    int             ismbcodepage;
} threadmbcinfo, *pthreadmbcinfo;

typedef struct _locale_tstruct {
    pthreadlocinfo  locinfo;
    pthreadmbcinfo  mbcinfo;
} *_locale_t;

pthreadlocinfo CDECL get_locinfo(void);

/* Only ->strtok_next is ever touched (by strtok). */
typedef struct __thread_data {
    char           *strtok_next;
} thread_data_t;

extern thread_data_t * CDECL msvcrt_get_thread_data(void);

/* ------------------------------------------------------------------ *
 * Externals string.c calls (implemented in refimpl.c).  All of these are
 * already inside the msvcrt_ref_* import namespace declared far above.
 * ------------------------------------------------------------------ */
int * __cdecl _errno(void);
void  __cdecl _invalid_parameter(const wchar_t *expr, const wchar_t *func,
                                 const wchar_t *file, unsigned int line,
                                 uintptr_t arg);
int __cdecl _isspace_l(int c, _locale_t locale);
int __cdecl _tolower_l(int c, _locale_t locale);
int __cdecl _toupper_l(int c, _locale_t locale);

/* msvcrt.h's invalid-parameter macros, verbatim */
#define MSVCRT_INVALID_PMT(x,err)   (*_errno() = (err), _invalid_parameter(NULL, NULL, NULL, 0, 0))
#define MSVCRT_CHECK_PMT_ERR(x,err) ((x) || (MSVCRT_INVALID_PMT( 0, (err) ), FALSE))
#define MSVCRT_CHECK_PMT(x)         MSVCRT_CHECK_PMT_ERR((x), EINVAL)

/* msvcrt.h's float-parse plumbing (definitions live in string.c itself) */
enum fpmod {
    FP_ROUND_ZERO,
    FP_ROUND_DOWN,
    FP_ROUND_EVEN,
    FP_ROUND_UP,
    FP_VAL_INFINITY,
    FP_VAL_NAN
};

struct fpnum {
    int         sign;
    int         exp;
    ULONGLONG   m;
    enum fpmod  mod;
};

struct fpnum fpnum_parse(wchar_t (*)(void*), void (*)(void*),
        void*, pthreadlocinfo, BOOL);
int fpnum_double(struct fpnum*, double*);

#endif /* __WINE_MSVCRT_H */
