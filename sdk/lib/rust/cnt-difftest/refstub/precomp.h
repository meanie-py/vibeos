/* Minimal <precomp.h> for isolating libcntpr .c reference logic on the
 * host (strstr.c, atoi64.c). Pulls only what those need from the host
 * libc; the actual algorithm under test is the ReactOS .c body. */
#ifndef CNT_REFSTUB_PRECOMP_H
#define CNT_REFSTUB_PRECOMP_H
#include <stddef.h>
#include <string.h>   /* host strlen/strncmp — used by strstr.c */
#include <ctype.h>    /* host isspace/isdigit/tolower — strtoi64.c   */
#include <limits.h>   /* ULONG_MAX — strtoul.c                       */
#define CDECL
#define __cdecl
#define __int64 long long   /* macro (not typedef) so `unsigned __int64` parses — itoa.c */
typedef int BOOL;
#define TRUE 1
#define FALSE 0
/* Compile the reference's _LIBCNT_ path — that is the libcntpr
 * behavior cnt-rust models (clamp w/o errno, no TRACE, CHECK_PMT is
 * just the predicate). Matching it is required for parity. */
#define _LIBCNT_
#define MSVCRT_CHECK_PMT(x) (x)
#define MSVCRT_INVALID_PMT(x,err)   /* _LIBCNT_: secure-crt checks disabled */
#define _I64_MAX  0x7fffffffffffffffLL
#define _I64_MIN  (-_I64_MAX - 1)
#define EINVAL 22   /* MSVCRT errno.h */
#define ERANGE 34
typedef void *_locale_t;
typedef unsigned long long ULONGLONG;
/* Windows wchar_t is 16-bit; itow.c uses WCHAR/PWCHAR internally and
 * `wchar_t` in its public sigs. The itow reference TU additionally
 * `#define wchar_t unsigned short` so sizeof matches the cnt-rust
 * `*mut u16` model (host wchar_t is 32-bit — would break parity). */
typedef unsigned short WCHAR;
typedef WCHAR *PWCHAR;
#endif
