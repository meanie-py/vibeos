/*
 * Stub for msvcrt's <corecrt.h>, so the GENUINE <mbctype.h> and <mbstring.h>
 * can be included as-is.
 *
 * THE GUARD IS THE TRICK — same as refstub/msvcrt.h.  We claim
 * `__WINE_CORECRT_H`, so when mbctype.h does `#include <corecrt.h>` it gets
 * this file and the real one (which drags in sal.h and the whole PSDK chain)
 * is never reached.
 *
 * Why stub corecrt and NOT mbctype/mbstring: mbctype.h is where `_M1`, `_M2`,
 * `_SBUP`, `_SBLOW`, `_MP`, `_MS` and the `_MB_CP_*` codepage selectors are
 * DEFINED, and those flag bits are the entire content of a `threadmbcinfo`.
 * Transcribing them here would put the harness's idea of "lead byte" in a
 * different file from msvcrt's, free to drift — and a drifted flag is
 * invisible, because the reference and the subject share one fixture and
 * would both read the same wrong bit.  So the real header is included and
 * only its dependency is stubbed.  Everything mbctype.h/mbstring.h actually
 * need from corecrt.h is the two decorations below; `_locale_t` and `size_t`
 * arrive earlier, from refstub/msvcrt.h.
 */
#ifndef __WINE_CORECRT_H
#define __WINE_CORECRT_H

/* refstub/msvcrt.h is force-included ahead of every reference TU and already
 * defines __cdecl (a no-op on the SysV host).  Guard anyway so this header is
 * not order-dependent. */
#ifndef __cdecl
#define __cdecl
#endif

/* dllimport/dllexport decoration: meaningless for a host object. */
#define _ACRTIMP

#endif /* __WINE_CORECRT_H */
