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
 * Types we must define BEFORE any host header can, because MSVC's width
 * differs from glibc's.  We claim glibc's own include guards so its
 * <wchar.h>/<wctype.h> (pulled below) skip their definitions and ours win.
 *
 *   wint_t    MSVC 16-bit (so WEOF == 0xFFFF); glibc 32-bit (0xFFFFFFFF).
 *             fpnum_parse compares its `wchar_t` getter result against WEOF.
 *   mbstate_t MSVC `int` (dll/win32/msvcrt/include/wchar.h); glibc a struct.
 * ------------------------------------------------------------------ */
#define __wint_t_defined 1
#define _WINT_T 1
typedef unsigned short wint_t;
#define WEOF ((wint_t)0xFFFF)

#define __mbstate_t_defined 1
typedef int mbstate_t;

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
#define WINAPIV       /* wcs.c's varargs entry points (_snwprintf, _scprintf, …) */
#define __stdcall

#define __int64 long long   /* macro, not typedef, so `unsigned __int64` parses */

typedef int                 BOOL;
#ifndef TRUE
#define TRUE  1
#define FALSE 0
#endif
typedef unsigned char       UCHAR;
typedef unsigned char       BOOLEAN;   /* NT's one-byte bool; RtlGenRandom returns it */
typedef unsigned int        UINT32;    /* math.c's float/bits unions */
typedef unsigned long long  UINT64;    /*   …and its double ones */
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

/* msvcrt's is 0x7FFF (dll/win32/msvcrt/include/stdlib.h:34, sdk/include/crt/stdlib.h:87);
 * glibc's is 0x7FFFFFFF, and <stdlib.h> above already defined it. `rand()` ends in
 * `(seed >> 16) & RAND_MAX`, so under the host's value the reference returns 16-bit numbers
 * where msvcrt returns 15-bit ones — and the first `rand` row caught exactly that, with the
 * REFERENCE wrong. A host constant leaking into the oracle is the same class as a hand-typed
 * one, and harder to see: nothing here typed it. */
#undef  RAND_MAX
#define RAND_MAX 0x7FFF

/* The floating-point CLASS codes, and this is the RAND_MAX trap at its worst: msvcrt and
 * glibc disagree on four of the five, and one of the two they share is the disagreement's
 * best camouflage.
 *
 *                 msvcrt (sdk/include/crt/math.h:76-85)   glibc <math.h>
 *   FP_NAN                2 (_NANCODE)                          0
 *   FP_INFINITE           1 (_INFCODE)                          1   <-- the only match
 *   FP_ZERO               0                                     2
 *   FP_SUBNORMAL         -2 (_DENORM)                           3
 *   FP_NORMAL            -1 (_FINITE)                           4
 *
 * math.c branches on these in six places and `_dclass`/`_fdclass` RETURN them, so under the
 * host's values the reference would take a different arm from msvcrt for every input that is
 * not an infinity — and report it as a Rust bug. Pre-empted rather than discovered, because
 * `rand()`'s RAND_MAX taught this exact lesson one file earlier: any macro the host header
 * and the Windows header both define, and define differently, is an oracle bug waiting for
 * its first row. */
#undef  FP_NAN
#undef  FP_INFINITE
#undef  FP_ZERO
#undef  FP_SUBNORMAL
#undef  FP_NORMAL
#define _DENORM      (-2)
#define _FINITE      (-1)
#define _INFCODE     1
#define _NANCODE     2
#define FP_INFINITE  _INFCODE
#define FP_NAN       _NANCODE
#define FP_NORMAL    _FINITE
#define FP_SUBNORMAL _DENORM
#define FP_ZERO      0

/* ------------------------------------------------------------------ *
 * glibc's WIDE headers — pulled here, and *deliberately* UNDER the 16-bit
 * wchar_t remap.  This is the exact opposite of the rule for the narrow
 * headers above, and the reason is worth stating.
 *
 * wcs.c does its own `#include <wchar.h>` / `<wctype.h>`, and it does so
 * AFTER build.rs's prelude has renamed wcs.c's public symbols to `ref_*`.
 * If glibc were left to declare `ref_wcslen`, `ref_wcscmp`, … under the HOST
 * 32-bit wchar_t, every one of wcs.c's 16-bit definitions would be a
 * conflicting-types error.  Parsed HERE, under the remap, glibc's wide
 * prototypes come out 16-bit — i.e. precisely the Windows ones wcs.c goes on
 * to define — and wcs.c's own includes then hit these guards and expand to
 * nothing.  That turns a 100-error collision into a no-op.
 *
 * Three do NOT line up even at 16-bit, because glibc's *shape* differs:
 * `wcstok` is 3-arg (msvcrt's is 2-arg here), and `wcstol`/`wcstoul` return an
 * LP64 `long`.  Those are hidden from glibc by the prelude's stage 0 and
 * re-declared with their Windows prototypes in stage 3 — the same two-step the
 * narrow `atol`/`strtol`/`strtoul` trio already needs (see build.rs).
 * ------------------------------------------------------------------ */
#include <wchar.h>
#include <wctype.h>

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
/* errno.c's Win32 -> errno store.  file.c calls it 54 times and so does file.rs, and it
 * writes through `_errno()` — so a SINGLE definition would resolve that call to whichever
 * side got the plain name and hand both sides one errno.  It did: the subject's failures
 * set the reference's slot.  Namespaced here so the reference's TUs and fileimpl.c's
 * definition meet at msvcrt_ref_msvcrt_set_errno, leaving the plain name for
 * subjectimpl.c.  The MAPPING TABLE stays shared and un-namespaced
 * (msvcrt_difftest_win32_to_errno) — one transcription, two stores. */
#define msvcrt_set_errno        msvcrt_ref_msvcrt_set_errno
/* _isspace_l / _tolower_l / _toupper_l USED to be here — they were things the reference
 * IMPORTED, satisfied by refimpl.c's hand-written ASCII stubs.  As of wave 10 ctype.c is
 * a reference TU and DEFINES them, so they are reference EXPORTS: the generated prelude
 * renames them to `ref__tolower_l` &c. like every other one.
 *
 * Leaving them here would silently win — the refstub is included AFTER the prelude, so
 * its #define overrides, ctype.c would emit `msvcrt_ref__tolower_l`, and every
 * `ref__tolower_l` reference would fail to link.  (Which is exactly how this was found.)
 * More importantly it would put the reference's own ctype implementation back in the
 * IMPORT namespace, where the harness cannot grade it. */
#define get_locinfo             msvcrt_ref_get_locinfo
/* Same story as get_locinfo, and for the same reason: mbcs.c IMPORTS this (locale.c
 * defines it, and locale.c is parsed here, not compiled), and so does the shipping
 * Rust — msvcrt-sys binds it as an extern. Namespaced, so the reference gets its OWN
 * copy of the pinned fixture and the subject gets a separate one. Two copies rather
 * than one shared: the fixture is read-only state, so a case that mutates it is a
 * translation bug, and only separate storage can catch that. */
#define get_mbcinfo             msvcrt_ref_get_mbcinfo
#define msvcrt_get_thread_data  msvcrt_ref_get_thread_data
/* Same reasoning as get_mbcinfo, one level up: `_setmbcp`'s non-thread arm installs into
 * `MSVCRT_locale->mbcinfo` and then copies the result into the exported `MSVCRT_mbctype`.
 * Shared, the two sides would install into the same slot and each read back what the
 * other wrote — a row comparing a table against itself, which is what renaming
 * `MSVCRT_mbctype` for the reference was already done to prevent. */
#define MSVCRT_locale           msvcrt_ref_locale
/* CompareStringA / LCMapStringA / …W / GetStringTypeW / WideCharToMultiByte USED to be
 * namespaced here.  As of wave 11 they are SHARED — one definition (refimpl.c), called by
 * both sides.
 *
 * They are kernel32, i.e. the ENVIRONMENT, and they are never code under test: this crate
 * does not flip kernel32.  The namespace exists to stop the reference calling into the
 * SUBJECT; it was never needed for these.  And modelling them TWICE — once for each side —
 * is the §9.73 trap in miniature: data (or behaviour) we hand to both sides can never be
 * caught by a differential test, because a mistake makes both sides wrong the same way and
 * classify still reports MATCH.  One copy, shared, so the comparison isolates the Rust's
 * LOGIC around the NLS call rather than our model of the NLS itself.
 *
 * (This is what makes the non-C-locale mode gradeable at all — see ref_set_nls().) */
#define ros_dbg_log             msvcrt_ref_ros_dbg_log
#define wine_dbgstr_an          msvcrt_ref_wine_dbgstr_an
/* …and the same for everything the WIDE reference TU (wcs.c) imports. */
#define wine_dbgstr_wn             msvcrt_ref_wine_dbgstr_wn
#define GetLastError               msvcrt_ref_GetLastError
#define GetProcessHeap             msvcrt_ref_GetProcessHeap
#define HeapAlloc                  msvcrt_ref_HeapAlloc
#define HeapFree                   msvcrt_ref_HeapFree
/* _mbtowc_l left this namespace when mbcs.c became a reference TU: mbcs.c DEFINES
 * it, so it is a reference EXPORT and the prelude renames it to ref__mbtowc_l.
 * Same move _isspace_l/_tolower_l/_toupper_l made for ctype.c above. */
/* `_get_output_format` left this namespace when misc.c became reference TU #9, and the
 * upgrade is the same one ctype.c and mbcs.c made: refimpl.c used to hand-stub it as a
 * constant 0, so wcs.c's `%e` exponent width was graded against a value THIS HARNESS chose.
 * misc.c defines the real one over the real `output_format` static that `_set_output_format`
 * writes, so it is now a reference EXPORT and the prelude renames it to
 * `ref__get_output_format`. Leaving the #define here would silently win — the refstub is
 * included AFTER the prelude — and misc.c would emit `msvcrt_ref__get_output_format` while
 * every `ref__get_output_format` reference failed to link. */
#define get_current_locale_noalloc msvcrt_ref_get_current_locale_noalloc
#define free_locale_noalloc        msvcrt_ref_free_locale_noalloc
#define MSVCRT__pwctype            msvcrt_ref__pwctype
/* An MSVC intrinsic (printf.h's %e exponent sizing).  The host has no such
 * builtin, so it becomes a real call — namespaced like every other import so
 * it can never collide with a kernel/CRT crate that exports the same name. */
#define _BitScanReverse            msvcrt_ref__BitScanReverse

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

/* ── errno: MSVCRT's numbering, not the host's ──────────────────────
 *
 * This file used to pin only STRUNCATE (the host has no such value) and let EINVAL /
 * ERANGE / EILSEQ fall through to glibc's <errno.h>.  That worked *by coincidence*:
 *
 *     EINVAL   msvcrt 22   glibc 22    same
 *     ERANGE   msvcrt 34   glibc 34    same
 *     EILSEQ   msvcrt 42   glibc 84    DIFFERENT
 *
 * Nothing exercised an EILSEQ path until ctype.c became a reference TU (wave 10), at
 * which point the reference started reporting `errno=84` for a value msvcrt.dll would
 * report as 42 — i.e. the oracle was modelling the HOST's errno, not msvcrt's.  A
 * numeric coincidence between two operating systems is not a model.  Pin all three. */
#undef EINVAL
#undef ERANGE
#undef EILSEQ
#define EINVAL  22
#define ERANGE  34
#define EILSEQ  42

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

/* (wint_t / WEOF are defined at the very top — they have to precede the
 * <wchar.h> pull whose guard we claim.) */

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
 * ── WAVE 10: THIS STRUCT IS NOW THE REAL LAYOUT, NOT A MODEL ────────
 *
 * It used to be a convenient cut-down (6 fields, `lc_handle[16]`).  That was fine
 * while only the C could see it.  It stopped being fine when Rust started reading
 * `locinfo->pctype/pclmap/pcumap`, because the SHIPPING Rust reads them through
 * `msvcrt-sys` — a bindgen snapshot of msvcrt's REAL layout (`pcumap` @ 336 on
 * amd64/arm64, @ 208 on i386).  The harness links that same Rust and points it at
 * *this* struct.  Two different layouts, one set of offsets: the Rust would have read
 * `pcumap` from beyond the end of the object, in the one component whose whole job is
 * to be a trustworthy oracle.  A wrong offset there does not crash — it makes the
 * oracle lie.
 *
 * So this is now a faithful transcription of msvcrt's `threadlocaleinfostruct`
 * (dll/win32/msvcrt/include/corecrt.h, with the `_MSVCR_VER=0` branches resolved).
 * The equivalence is not assumed — `ref_integrity` asserts, at run time, that C's
 * `offsetof` here equals Rust's `offset_of!` in msvcrt-sys, field by field.
 *
 * `__msvcrt_ulong` is 32-bit on BOTH the host (LP64: `unsigned int`) and Windows
 * (LLP64: `unsigned long`) — corecrt.h goes out of its way to guarantee that — so the
 * host layout and the amd64 layout coincide exactly.  Spelled `unsigned int` here so
 * it cannot depend on which one we are.
 *
 * lc_handle is now its REAL size (6), which means the LC_* indices must be MSVC's, not
 * the host's — see the LC_* block below.
 * ------------------------------------------------------------------ */
/* The layout itself lives in locale_layout.h, because subjectimpl.c needs it too and
 * must NOT see this file's anti-circularity namespace block.  See that header. */
#include "locale_layout.h"

typedef RL_LC_ID LC_ID, *LPLC_ID;
typedef struct threadlocaleinfostruct threadlocinfo, *pthreadlocinfo;

/* MSVC's locale categories.  The host <locale.h> is included FIRST by every reference
 * TU (`#include <locale.h>` then `#include "msvcrt.h"`), and glibc numbers these
 * completely differently (LC_CTYPE=0 … LC_ALL=6 … LC_IDENTIFICATION=12).  With
 * lc_handle[] at its real size of 6, a glibc `LC_ALL`(=6) would index one PAST the
 * end.  Pin them to MSVC's numbering, which is what msvcrt's own code means.
 * (Every handle is 0 in the "C" locale, so this changes no behaviour — only bounds.) */
#undef LC_ALL
#undef LC_COLLATE
#undef LC_CTYPE
#undef LC_MONETARY
#undef LC_NUMERIC
#undef LC_TIME
#define LC_ALL      RL_LC_ALL
#define LC_COLLATE  RL_LC_COLLATE
#define LC_CTYPE    RL_LC_CTYPE
#define LC_MONETARY RL_LC_MONETARY
#define LC_NUMERIC  RL_LC_NUMERIC
#define LC_TIME     RL_LC_TIME

/* The FULL struct, from locale_layout.h — see the note there.  This was a
 * three-field abbreviation until mbcs.c became a reference TU; the shipping Rust
 * has always read the real thing. */
typedef rl_threadmbcinfo threadmbcinfo, *pthreadmbcinfo;

/* Defined in locale_layout.h (both sides need the offset of `mbcinfo`).  The struct
 * tag is ALSO a typedef name here: wcs.c's _wcsicmp_l family declares
 * `_locale_tstruct tmp = {0};` on the stack. */
typedef struct _locale_tstruct _locale_tstruct, *_locale_t;

pthreadlocinfo CDECL get_locinfo(void);

/* One cursor per reference TU: strtok (string.c), wcstok (wcs.c), mbstok (mbcs.c).
 * `locale_flags` / `mbcinfo` are what mbcs.c's _setmbcp reaches for. This is the
 * REFERENCE's private thread data — refimpl.c returns it, and nothing else sees the
 * layout — so it carries only the fields the reference TUs actually touch. (The
 * SUBJECT's copy in subjectimpl.c is the one that has to match msvcrt-sys, and
 * ref_integrity asserts that.) */
/* The user-installable invalid-parameter handler.  errno.c declares a file-static of
 * this type; at _MSVCR_VER = 0 nothing can ever write it (its only setter,
 * _set_invalid_parameter_handler, is `#if _MSVCR_VER >= 80`), so the slot is
 * permanently NULL and the dispatch always takes the else arm.  The TYPE is still
 * needed to compile the declaration. */
typedef void (__cdecl *_invalid_parameter_handler)(const wchar_t *, const wchar_t *,
                                                   const wchar_t *, unsigned int,
                                                   uintptr_t);

/* The three handler slots cpp.c hands out and except.c calls through
 * (dll/win32/msvcrt/msvcrt.h:75-77).  `set_terminate`/`set_unexpected`/
 * `_set_se_translator` write them and return the previous value, so the TYPE is
 * part of the contract every one of those rows grades. */
struct _EXCEPTION_POINTERS;  /* completed by msvcrt_cpp_types.h, included after */
typedef void (__cdecl *terminate_function)(void);
typedef void (__cdecl *unexpected_function)(void);
typedef void (__cdecl *_se_translator_function)(unsigned int code,
                                                struct _EXCEPTION_POINTERS *info);

typedef struct __thread_data {
    char           *strtok_next;
    wchar_t        *wcstok_next;
    unsigned char  *mbstok_next;
    /* misc.c's `rand`/`srand` LCG state, added when misc.c became reference TU #9. The
     * subject's copy is msvcrt-sys's real `thread_data_t`; this one carries only what the
     * reference TUs touch, which is why the field arrives per-TU rather than up front. */
    unsigned int    random_seed;
    /* math.c's `__fpecode` slot (dll/win32/msvcrt/msvcrt.h:198). Arrives with math.c for
     * the same reason `random_seed` arrived with misc.c: this struct carries only what the
     * reference TUs actually touch. */
    int             fpecode;
    /* `_ecvt`/`_fcvt` hand out pointers INTO this buffer and it is lazily malloc'd, so it is
     * per-thread state the rows observe directly — the same aliasing contract errno.c's
     * strerror buffers carry (dll/win32/msvcrt/msvcrt.h:204). */
    char           *efcvt_buffer;
    int             locale_flags;
    pthreadmbcinfo  mbcinfo;
    /* file.c's tmpnam/_wtmpnam scratch — same slots as the real
     * dll/win32/msvcrt/msvcrt.h, kept in field order. */
    char                           *tmpnam_buffer;
    wchar_t                        *wtmpnam_buffer;
    /* errno.c's four slots, added when it became reference TU #7.  This is where
     * msvcrt actually keeps errno: `_errno()` is `&msvcrt_get_thread_data()->thread_errno`
     * and `__doserrno()` the doserrno one, so the reference no longer stores errno in
     * something the harness invented.  strerror/_wcserror hand out pointers INTO the two
     * buffers, which is why they are per-thread slots and not statics — that aliasing is
     * part of the contract those rows grade. */
    int                             thread_errno;
    __msvcrt_ulong                  thread_doserrno;
    char                           *strerror_buffer;
    wchar_t                        *wcserror_buffer;
    /* time.c's three lazily-malloc'd result buffers (dll/win32/msvcrt/msvcrt.h:191-193),
     * added when it became reference TU #11 — same per-TU arrival rule as everything
     * above.  They are exactly the aliasing contract the errno buffers carry, and time.c
     * leans on it harder: `_localtime64`, `_gmtime64` and `asctime` all hand out a pointer
     * INTO the caller's thread data and the next call overwrites it, which is why the
     * classic "print localtime and gmtime in one printf" bug prints one twice.  A row
     * that copied the result out before comparing would grade the value and lose the
     * aliasing; the rows keep the pointer.
     *
     * `struct tm` here is the HOST's, from <time.h>: glibc's is msvcrt's nine ints plus
     * `tm_gmtoff`/`tm_zone` at the END, so every field time.c touches sits at the same
     * offset and the reference simply allocates a larger box than the DLL would.  Tests
     * that hand a `struct tm` IN (mktime, asctime, strftime) must therefore allocate the
     * larger one — see tests/time.rs's CTm. */
    char                           *asctime_buffer;
    wchar_t                        *wasctime_buffer;
    struct tm                      *time_buffer;
    /* The C++ EH slots, arriving with cpp.c (reference TU #14) and except.c (#15)
     * under the same per-TU rule as everything above: this struct carries only
     * what the reference TUs actually touch.
     *
     * These are per-THREAD and that is the whole point of several rows.  The
     * terminate/unexpected/se_translator handlers are what `set_terminate` and
     * friends install and `terminate()` calls, and `processing_throw` is the
     * re-entrancy counter that decides whether a throw during a throw becomes a
     * `terminate`.  `frame_info_head` is the head of the linked list
     * `_CreateFrameInfo` pushes onto and `_FindAndUnlinkFrame` pops — a global
     * whose corruption stays invisible until a later unwind reads it, which is
     * exactly why it is graded rather than assumed.
     *
     * The pointer members are spelled `struct _X *` rather than by typedef
     * because msvcrt_cpp_types.h — where those structs are defined and proved —
     * is included AFTER this header, and an incomplete pointee is all a pointer
     * slot needs. */
    struct _EXCEPTION_POINTERS     *xcptinfo;
    terminate_function              terminate_handler;
    unexpected_function             unexpected_handler;
    _se_translator_function         se_translator;
    struct _EXCEPTION_RECORD       *exc_record;
    struct _CONTEXT                *ctx_record;
    int                             processing_throw;
    struct _frame_info             *frame_info_head;
} thread_data_t;

#define LOCALE_FREE    0x1
#define LOCALE_THREAD  0x2

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
/* undname.c's one ctype call. Declared HERE, i.e. after the prelude's stage-1 rename, so
 * this line becomes the prototype for `ref_isdigit` — ctype.c's own body. glibc's
 * declaration cannot serve: stage -1 pulls <ctype.h> and strips its macros BEFORE the
 * rename (it has to, or ctype.c's definitions would be macro-mangled), so glibc only ever
 * declares the PLAIN name. Without this the call degrades to an implicit `int ref_isdigit()`
 * — which -Werror=implicit-function-declaration is on to catch, and did. */
/* Parenthesised so glibc's FUNCTION-LIKE `#define isdigit(c) ...` cannot expand here:
 * refimpl.c includes this header WITHOUT the prelude, so that macro is live in it. The
 * prelude's own `#define isdigit ref_isdigit` is object-like and still applies. */
int __cdecl (isdigit)(int c);

/* ── what the math.c reference TU needs ────────────────────────────────
 *
 * `struct _exception` is the matherr record, VERBATIM from sdk/include/crt/math.h:20-26 —
 * the host's <math.h> has SVID's `struct exception`, which is a different type with a
 * different name, so nothing here can be borrowed. `_matherr` takes a pointer to this and
 * `math_error` builds one on the stack, so the field ORDER is part of the contract: a
 * harness copy with the members transposed would grade a struct msvcrt never builds. */
/* sdk/include/crt/math.h:31-33 — `_cabs`'s BY-VALUE parameter, so its layout is an ABI
 * contract and not merely a type: two doubles, x then y. */
struct _complex {
    double x, y;
};

struct _exception {
    int    type;
    char  *name;
    double arg1;
    double arg2;
    double retval;
};
int __cdecl _matherr(struct _exception *e);

/* wcs.c's `_scprintf`/`_snprintf`, which math.c's `_ecvt`/`_gcvt` family calls. Declared
 * HERE, i.e. AFTER the prelude's stage-1 rename, so these become the prototypes for
 * `ref__scprintf`/`ref__snprintf` — wcs.c's own bodies. Same move `isdigit` made for
 * undname.c, and for the same reason: the host header that would otherwise declare them
 * runs BEFORE the rename and so only ever declares the plain names, leaving the call to
 * degrade into an implicit `int f()` that returns garbage for a `size_t`. */
int WINAPIV _scprintf(const char *format, ...);
int WINAPIV _snprintf(char *str, size_t len, const char *format, ...);

/* xdk/ketypes.h:186. `msvcrt_init_math` indexes KUSER_SHARED_DATA's processor-feature array
 * with it — at the hard-coded address 0x7FFE0274, which is why that one function is
 * COMPILE-ONLY here and no row may call it. The constant still has to be right: a wrong
 * index would read a different feature bit, and on a host with no page there at all the
 * failure is a fault rather than a wrong answer. */
#define PF_XMMI64_INSTRUCTIONS_AVAILABLE 10

/* `_fpclass`'s return codes, LIFTED VERBATIM from ucrt/float.h:197-206. The
 * third constant family math.c needs and the third that the host cannot supply — glibc has
 * no `_FPCLASS_*` at all, so this one fails to compile rather than compiling wrong, which
 * makes it the friendly member of the set. Kept in the header's own order and spelling so a
 * diff against it is a diff, not a reading exercise. */
#define _FPCLASS_SNAN  0x0001
#define _FPCLASS_QNAN  0x0002
#define _FPCLASS_NINF  0x0004
#define _FPCLASS_NN    0x0008
#define _FPCLASS_ND    0x0010
#define _FPCLASS_NZ    0x0020
#define _FPCLASS_PZ    0x0040
#define _FPCLASS_PD    0x0080
#define _FPCLASS_PN    0x0100
#define _FPCLASS_PINF  0x0200

/* The x87/SSE CONTROL-WORD masks that `_control87`/`_controlfp`/`_controlfp_s` translate
 * between, lifted verbatim from ucrt/float.h:131-164. These are the whole
 * contract of those three functions -- each is a bit-shuffle between the hardware word and
 * msvcrt's abstract one -- so a wrong mask here would grade the shuffle against the wrong
 * layout and agree with nothing. Lifted rather than typed for the reason the FP_* table
 * above gives, and kept in the header's order so a diff against it stays a diff. */
#define _EM_AMBIGUIOUS  0x80000000
#define _EM_AMBIGUOUS   0x80000000
#define _MCW_EM         0x0008001f
#define _EM_INEXACT     0x00000001
#define _EM_UNDERFLOW   0x00000002
#define _EM_OVERFLOW    0x00000004
#define _EM_ZERODIVIDE  0x00000008
#define _EM_INVALID     0x00000010
#define _EM_DENORMAL    0x00080000
#define _MCW_RC         0x00000300
#define _RC_NEAR        0x00000000
#define _RC_DOWN        0x00000100
#define _RC_UP          0x00000200
#define _RC_CHOP        0x00000300
#define _MCW_PC         0x00030000
#define _PC_64          0x00000000
#define _PC_53          0x00010000
#define _PC_24          0x00020000
#define _MCW_IC         0x00040000
#define _IC_AFFINE      0x00040000
#define _IC_PROJECTIVE  0x00000000
#define _MCW_DN         0x03000000
#define _DN_SAVE        0x00000000
#define _DN_FLUSH       0x01000000
#define _DN_FLUSH_OPERANDS_SAVE_RESULTS 0x02000000
#define _DN_SAVE_OPERANDS_FLUSH_RESULTS 0x03000000

/* `_statusfp`/`_clearfp`'s status-word bits, same header, same reason
 * (ucrt/float.h). The host has FE_* for the same concepts at different values,
 * which is why gcc suggests `FE_INVALID` here -- taking that suggestion would compile and
 * grade the wrong bit. */
#define _SW_INEXACT     0x00000001
#define _SW_UNDERFLOW   0x00000002
#define _SW_OVERFLOW    0x00000004
#define _SW_ZERODIVIDE  0x00000008
#define _SW_INVALID     0x00000010
#define _SW_DENORMAL    0x00080000
#define _SW_UNEMULATED          0x0040
#define _SW_SQRTNEG             0x0080
#define _SW_STACKOVERFLOW       0x0200
#define _SW_STACKUNDERFLOW      0x0400

/* xmmintrin.h:54 -- the SSE function attribute, empty in the configuration msvcrt builds
 * under. math.c puts it on `_set_SSE2_enable`'s helpers; on the host it only has to parse. */
#define __ATTRIBUTE_SSE__

/* ── what the misc.c reference TU needs ────────────────────────────────
 *
 * Three kernel32 entry points and the two page-protection bits `_resetstkoflw` passes.
 * They were ALREADY THERE — fileimpl.c wrote them for the SUBJECT's misc.rs, under a
 * comment that names this exact wave ("The four misc.c needs … qsort, bsearch, _lsearch
 * and rand are pure algorithms with real oracles"). So the reference only needs the
 * prototypes; the definitions stay where they are, one each, SHARED, for the reason the
 * CompareStringA note above gives: they are the ENVIRONMENT and this crate does not flip kernel32, so modelling them
 * twice could only make both sides wrong the same way. What the rows grade is the contract
 * AROUND the call — `_beep`'s argument forwarding, `_sleep`'s zero-to-one adjustment,
 * `_resetstkoflw`'s return — not our model of kernel32. */
void  WINAPI Beep(DWORD dwFreq, DWORD dwDuration);
void  WINAPI Sleep(DWORD dwMilliseconds);
BOOL  WINAPI VirtualProtect(void *addr, ULONG_PTR size, DWORD prot, DWORD *old);
#define PAGE_READWRITE 0x04
#define PAGE_GUARD     0x100

/* ── what the undname.c reference TU needs ─────────────────────────────
 *
 * `lstrcpynA` is kernel32 — the ENVIRONMENT, never code under test — so it is SHARED, one
 * definition in timeimpl.c, like CompareStringA and for the same reason. It is also not
 * `strncpy`: it copies at most n-1 bytes and ALWAYS terminates, which is the whole of
 * `__unDNameEx`'s truncation behaviour when the caller supplies a buffer.
 *
 * Declared here rather than left implicit: `-Werror=implicit-function-declaration` is on
 * for exactly this, and an implicit `int lstrcpynA()` would truncate the returned pointer
 * to 32 bits.
 *
 * The demangler's ALLOCATOR, passed in by the caller. Copied from
 * dll/win32/msvcrt/msvcrt.h:263-264 verbatim -- these two are the reason undname.c is the
 * cheapest file in msvcrt to grade: the harness supplies memget/memfree per side at the
 * call, so there is no heap model to build and no allocation the test cannot see. */
typedef void* (__cdecl *malloc_func_t)(size_t);
typedef void  (__cdecl *free_func_t)(void*);
/* Spelled `char *` rather than LPSTR: those typedefs land further down this header. */
char * WINAPI lstrcpynA(char *dst, const char *src, int n);

/* ── what the mbcs.c reference TU imports ──────────────────────────────
 *
 * get_mbcinfo / free_mbcinfo / MSVCRT_locale live in locale.c, which this harness
 * PARSES (for the case-mapping tables) but does not compile — so they are environment,
 * supplied by refimpl.c and nlsimpl.c.  __crtLCMapStringA and locale_to_sname are
 * shared, one definition each, for the reason the NLS note above gives.
 *
 * The string.c entry points below are declared, not imported: string.c IS a reference
 * TU, so the prelude has already renamed them to ref_*, and these declarations exist
 * only so mbcs.c's calls are prototyped.  Without them C's implicit-declaration rule
 * invents `int f()` and truncates every returned pointer to 32 bits — the exact bug
 * the -Werror=implicit-function-declaration tripwire was added for. */
pthreadmbcinfo CDECL get_mbcinfo(void);
void                 free_mbcinfo(pthreadmbcinfo);
extern _locale_t     MSVCRT_locale;
int  __cdecl __crtLCMapStringA(LCID, DWORD, const char*, int, char*, int,
                               unsigned int, int);
BOOL locale_to_sname(const char*, unsigned short*, BOOL*, wchar_t*);
LONG WINAPI InterlockedIncrement(LONG volatile *addend);

int   __cdecl _stricmp(const char*, const char*);
int   __cdecl _strnicmp(const char*, const char*, size_t);
char* __cdecl _strrev(char*);
char* __cdecl _strset(char*, int);
char* __cdecl _strnset(char*, int, size_t);
int   __cdecl _strncoll_l(const char*, const char*, size_t, _locale_t);
int   __cdecl _strnicoll_l(const char*, const char*, size_t, _locale_t);
char* __cdecl strtok_s(char*, const char*, char**);

/* ctype.c's `_l` predicates — also a reference TU, also already renamed by the
 * prelude; these prototype mbcs.c's calls.  Note the leading underscore: the
 * HOST has `iswdigit_l(wint_t, locale_t)` and these are msvcrt's own
 * `_iswdigit_l(wint_t, _locale_t)`.  Different functions, and letting the host's
 * be picked up implicitly is how a `_locale_t` becomes a glibc `locale_t`. */
int __cdecl _iswalnum_l(wint_t, _locale_t);
int __cdecl _iswalpha_l(wint_t, _locale_t);
int __cdecl _iswdigit_l(wint_t, _locale_t);
int __cdecl _iswgraph_l(wint_t, _locale_t);
int __cdecl _iswlower_l(wint_t, _locale_t);
int __cdecl _iswprint_l(wint_t, _locale_t);
int __cdecl _iswpunct_l(wint_t, _locale_t);
int __cdecl _iswupper_l(wint_t, _locale_t);
int __cdecl _isprint_l(int, _locale_t);
int __cdecl _isleadbyte_l(int, _locale_t);
int __cdecl isleadbyte(int);

/* winbase.h byte accessors, as mbcs.c spells them.  `uintptr_t` stands in for
 * DWORD_PTR (refstub pulls <stdint.h> above); the cast only has to be wide enough
 * to hold whatever integer or pointer-ish value the macro is handed. */
#ifndef HIBYTE
#define LOBYTE(w) ((BYTE)((uintptr_t)(w) & 0xFF))
#define HIBYTE(w) ((BYTE)(((uintptr_t)(w) >> 8) & 0xFF))
#endif

/* minwindef.h's word accessors, as process.c spells them.  `_getdllprocaddr`'s whole
 * body is `if (HIWORD(ordinal)) return NULL;` followed by a cast of the low word to a
 * name pointer — so this macro IS the function's logic, not a convenience.  Same
 * uintptr_t stand-in for DWORD_PTR as the byte pair above. */
#ifndef HIWORD
#define LOWORD(l) ((WORD)((uintptr_t)(l) & 0xFFFF))
#define HIWORD(l) ((WORD)(((uintptr_t)(l) >> 16) & 0xFFFF))
#endif

/* msvcrt.h's own; the host has no such macro. */
#ifndef min
#define min(a,b) (((a) < (b)) ? (a) : (b))
#endif
#ifndef max
#define max(a,b) (((a) > (b)) ? (a) : (b))
#endif

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

/* ------------------------------------------------------------------ *
 * THE SECOND REFERENCE TU — dll/win32/msvcrt/wcs.c.
 *
 * wcs.c is 3100 lines, but only ~14 of its 176 public symbols are the pure
 * wide-string family under test (wcslen/wcscmp/wcschr/…).  The bulk is the
 * printf engine, which it instantiates twice by #including the REAL
 * "printf.h" (narrow, then again with PRINTF_WIDE).  That engine is compiled
 * as genuine, untranscribed code — and is inert: nothing under test calls it.
 * It just has to *link*, which is why its type surface is declared here.
 * ------------------------------------------------------------------ */
typedef char  *LPSTR;
typedef const char *LPCSTR;
typedef unsigned short *LPWSTR;
typedef const unsigned short *LPCWSTR;
typedef void  *LPVOID;
typedef int    INT;
typedef int    errno_t;

/* (glibc's internal ctype macros — __isascii/__toascii — are stripped in the generated
 * prelude's stage -1, BEFORE the ref_* renames.  Doing it here would be too late: it
 * would remove glibc's macro but leave the symbol un-renamed, and ctype.c would then
 * define a plain `__isascii` that collides with the code under test.  See build.rs.) */

/* locale.c's "setlocale() has never been called" flag.  ctype.c BRANCHES on it:
 * toupper/tolower take a hardcoded-ASCII fast path when it is TRUE, and only fall
 * through to _toupper_l/_tolower_l (the pcumap/pclmap lookup) when it is FALSE.
 * Defined by refimpl.c (locale.c is not a reference TU). */
extern BOOL initial_locale;

/* dll/win32/msvcrt/include/ctype.h — the _pctype/_pwctype classification bits.
 * Read by the isw* family (wcs.c) AND, since wave 10, by ctype.c — reference TU #3 —
 * which is the whole narrow isXXX/toXXX surface. */
#define _UPPER    RL_UPPER
#define _LOWER    RL_LOWER
#define _DIGIT    RL_DIGIT
#define _SPACE    RL_SPACE
#define _PUNCT    RL_PUNCT
#define _CONTROL  RL_CONTROL
#define _BLANK    RL_BLANK
#define _HEX      RL_HEX
#define _LEADBYTE RL_LEADBYTE
#define _ALPHA    RL_ALPHA

/* printf.h's template surface — verbatim from the real msvcrt.h. */
typedef int (*puts_clbk_a)(void*, int, const char*);
typedef int (*puts_clbk_w)(void*, int, const wchar_t*);
typedef union _printf_arg
{
    void *get_ptr;
    int get_int;
    LONGLONG get_longlong;
    double get_double;
} printf_arg;
typedef printf_arg (*args_clbk)(void*, int, int, va_list*);
int pf_printf_a(puts_clbk_a, void*, const char*, _locale_t,
        DWORD, args_clbk, void*, va_list*);
int pf_printf_w(puts_clbk_w, void*, const wchar_t*, _locale_t,
        DWORD, args_clbk, void*, va_list*);
int create_positional_ctx_a(void*, const char*, va_list);
int create_positional_ctx_w(void*, const wchar_t*, va_list);
printf_arg arg_clbk_valist(void*, int, int, va_list*);
printf_arg arg_clbk_positional(void*, int, int, va_list*);
#define MSVCRT_PRINTF_POSITIONAL_PARAMS             (0x0100)
#define MSVCRT_PRINTF_INVOKE_INVALID_PARAM_HANDLER  (0x0200)
#define _TWO_DIGIT_EXPONENT 0x1
#define _ARGMAX 100

/* msvcrt internals wcs.c imports (all implemented in refimpl.c). */
extern WORD *MSVCRT__pwctype;
unsigned int __cdecl _get_output_format(void);
int    __cdecl _mbtowc_l(wchar_t*, const char*, size_t, _locale_t);
_locale_t CDECL get_current_locale_noalloc(_locale_t);
void   CDECL free_locale_noalloc(_locale_t);
int    __cdecl _iswspace_l(wchar_t, _locale_t);
wint_t __cdecl _towlower_l(wint_t, _locale_t);
wint_t __cdecl _towupper_l(wint_t, _locale_t);
int    __cdecl _wctomb_l(char*, wchar_t, _locale_t);

/* Win32 that the wide NLS / heap paths import (all in refimpl.c). */
DWORD  WINAPI GetLastError(void);
HANDLE WINAPI GetProcessHeap(void);
LPVOID WINAPI HeapAlloc(HANDLE, DWORD, size_t);
BOOL   WINAPI HeapFree(HANDLE, DWORD, LPVOID);

/* MSVC intrinsic — printf.h's exponent sizing.  (refimpl.c) */
unsigned char _BitScanReverse(unsigned int *index, unsigned int mask);

#endif /* __WINE_MSVCRT_H */
