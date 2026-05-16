/* Minimal <precomp.h> for isolating libcntpr .c reference logic on the
 * host (strstr.c, atoi64.c). Pulls only what those need from the host
 * libc; the actual algorithm under test is the ReactOS .c body. */
#ifndef CNT_REFSTUB_PRECOMP_H
#define CNT_REFSTUB_PRECOMP_H
#include <stddef.h>
#include <string.h>   /* host strlen/strncmp — used by strstr.c */
#include <ctype.h>    /* host isspace/isdigit — used by atoi64.c */
#define CDECL
#define __cdecl
typedef long long __int64;
#endif
