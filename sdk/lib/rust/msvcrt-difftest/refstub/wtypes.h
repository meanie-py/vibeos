/*
 * Minimal <wtypes.h> for compiling the GENUINE dll/win32/msvcrt/wcs.c on the
 * host.  wcs.c pulls this for exactly two things:
 *
 *   BOOLEAN  — its `pf_flags` struct (the printf format-spec state).
 *   VARENUM  — printf.h tags every vararg fetch with a VT_* code
 *              (pf_args(ctx, pos, VT_I8, valist).get_longlong).
 *
 * Values are the real ones from sdk/include/psdk/wtypes.idl.  They only have
 * to be self-consistent — the printf engine is inert under test — but wrong
 * values would silently mis-fetch arguments if a future wave ever grades it,
 * so they are copied, not invented.
 */
#ifndef __WINE_WTYPES_H
#define __WINE_WTYPES_H

typedef unsigned char BOOLEAN;

enum VARENUM {
    VT_EMPTY = 0,
    VT_I4    = 3,
    VT_R8    = 5,
    VT_I8    = 20,
    VT_INT   = 22,
    VT_PTR   = 26
};

#endif /* __WINE_WTYPES_H */
