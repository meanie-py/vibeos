/* Minimal <internal/wine/msvcrt.h> stub for isolating wtoi64.c on the
 * host. Everything wtoi64.c actually uses is supplied by refstub's
 * <precomp.h> (MSVCRT_CHECK_PMT, _I64_MAX/_MIN, _locale_t, BOOL) or by
 * the per-unit prelude (LONGLONG, _UI64_MAX, towlower, iswctype). The
 * TRACE/_errno/ERANGE references are all under `#ifndef _LIBCNT_` and
 * _LIBCNT_ is set, so they never compile. This stub just lets the
 * `#include <internal/wine/msvcrt.h>` line resolve. */
#ifndef CNT_REFSTUB_WINE_MSVCRT_H
#define CNT_REFSTUB_WINE_MSVCRT_H
#endif
