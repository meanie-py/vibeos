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
typedef long long __int64;
typedef int BOOL;
#define TRUE 1
#define FALSE 0
/* Compile the reference's _LIBCNT_ path — that is the libcntpr
 * behavior cnt-rust models (clamp w/o errno, no TRACE, CHECK_PMT is
 * just the predicate). Matching it is required for parity. */
#define _LIBCNT_
#define MSVCRT_CHECK_PMT(x) (x)
#define _I64_MAX  0x7fffffffffffffffLL
#define _I64_MIN  (-_I64_MAX - 1)
typedef void *_locale_t;
#endif
