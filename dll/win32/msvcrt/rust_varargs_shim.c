/*
 * PROJECT:     ReactOS CRT (vibeos Rust port)
 * LICENSE:     LGPL-2.1-or-later (see wcs.c, from which these bodies are taken)
 * PURPOSE:     The irreducible C half of the printf family: the 32 variadic
 *              entry points, as va_start trampolines over the Rust engine.
 *
 * Everything printf in msvcrt is Rust now EXCEPT the `...` shells, and the
 * reason is a language limit, not a translation gap: stable Rust cannot
 * DEFINE a C-variadic function (`c_variadic` is nightly, and this project is
 * on pinned stable). Receiving an already-started cursor is fine — that is
 * what msvcrt-crt-rust/src/printf_shells.rs does for all 30 `va_list` entry
 * points, since a `va_list` is a plain `char *` on every Windows target we
 * build. Only `va_start` is out of reach. So these stay C, and each is the
 * same three lines the genuine wcs.c has: start the cursor, hand it to the
 * Rust twin, end it.
 *
 * WHY THIS IS ITS OWN TU, and not a block left behind in wcs.c:
 *
 *   Symbol replacement flips a TU's CALLERS, not its HOUSEMATES.  Wave 14b
 *   flipped the six engine symbols and expected the shells to follow; the
 *   stream family in file.c did follow (it calls across a TU boundary, so
 *   the linker gets a say), but wcs.c's own shells did not, because clang
 *   had already IPA-specialized the same-TU engine into them as
 *   `pf_printf_a.specialized.N`.  Intra-TU inlining never consults the
 *   linker.  Left in wcs.c these trampolines would keep inlining wcs.c's C
 *   engine and the flip would be silently undone — a green build over dead
 *   Rust.  From here the delegates are external symbols, so first-match
 *   resolves them to msvcrt-crt-rust and there is nothing to inline.
 *
 * Link order carries the same weight: this file is compiled into the msvcrt
 * MODULE target, not into msvcrt_static.  Module objects are resolved before
 * any archive member, so these definitions win first-match over wcs.o's
 * copies (--allow-multiple-definition), exactly as --whole-archive does for
 * libmsvcrt_crt_cargo.a.  Putting it in msvcrt_static instead would make the
 * outcome depend on member order within one archive.
 *
 * Delegates are named explicitly (`_vsnprintf`, never the `vsnprintf` that
 * stdio.h #defines to it) so the call target is visible in the source rather
 * than resolved by a macro two headers away.
 *
 * ONE deliberate deviation from a literal transcription: wcs.c routes four of
 * these through its file-static `vswprintf_p_l_opt`, which is not exported
 * and so is not reachable from here.  Its two exported wrappers are
 * definitionally equal to the calls being made —
 *   _vswprintf_p_l(b, l, f, loc, ap) == vswprintf_p_l_opt(b, l, f, 0, loc, ap)
 *   _vscwprintf_p_l(f, loc, ap)      == vswprintf_p_l_opt(NULL, INT_MAX, f, 0, loc, ap)
 * — both being one-line forwarders that pass options=0, so substituting them
 * changes nothing observable.  Every other body below is wcs.c verbatim.
 */

#define _NO_CRT_STDIO_INLINE
#include <limits.h>
#include <locale.h>
#include <wchar.h>

#include "msvcrt.h"

/*
 * The CRT headers gate several `_l` delegates behind `_MSVCR_VER>=80`, and
 * this DLL builds at `_MSVCR_VER=0` — so at this vintage they are exported
 * but not declared.  wcs.c gets away with calling them because it DEFINES
 * them earlier in the same file; from a separate TU they are undeclared, and
 * an implicit declaration would silently assume `int(...)` and mis-pass
 * `size_t`/pointer arguments.  Prototypes below are copied from the
 * definitions in wcs.c, so a drift in either becomes a compile error here.
 */
int CDECL _vsnprintf_l(char *str, size_t len, const char *format, _locale_t locale, va_list valist);
int CDECL _vsnprintf_c(char *str, size_t len, const char *format, va_list valist);
int CDECL _vsnprintf_c_l(char *str, size_t len, const char *format, _locale_t locale, va_list valist);
int CDECL _vsnprintf_s_l(char *str, size_t sizeOfBuffer, size_t count, const char *format, _locale_t locale, va_list valist);
int CDECL _vscprintf_l(const char *format, _locale_t locale, va_list valist);
int CDECL _vscprintf_p_l(const char *format, _locale_t locale, va_list args);
int CDECL _vswprintf_l(wchar_t *str, const wchar_t *format, _locale_t locale, va_list args);
int CDECL _vsnwprintf_l(wchar_t *str, size_t len, const wchar_t *format, _locale_t locale, va_list valist);
int CDECL _vswprintf_c(wchar_t *str, size_t len, const wchar_t *format, va_list valist);
int CDECL _vswprintf_c_l(wchar_t *str, size_t len, const wchar_t *format, _locale_t locale, va_list valist);
int CDECL _vsnwprintf_s_l(wchar_t *str, size_t sizeOfBuffer, size_t count, const wchar_t *format, _locale_t locale, va_list valist);

/* ---- narrow, plain sinks ------------------------------------------------ */

int WINAPIV sprintf(char *str, const char *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vsnprintf(str, INT_MAX, format, ap);
    va_end(ap);
    return r;
}

int WINAPIV _sprintf_l(char *str, const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnprintf_l(str, INT_MAX, format, locale, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf(char *str, size_t len, const char *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vsnprintf(str, len, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_l(char *str, size_t count, const char *format,
                        _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnprintf_l(str, count, format, locale, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_c(char *str, size_t count, const char *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vsnprintf_c(str, count, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_c_l(char *str, size_t count, const char *format,
                          _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnprintf_c_l(str, count, format, locale, valist);
    va_end(valist);
    return retval;
}

/* ---- narrow, bounded (`_s`) --------------------------------------------- */

int WINAPIV sprintf_s(char *str, size_t num, const char *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vsnprintf(str, num, format, ap);
    va_end(ap);
    return r;
}

int WINAPIV _sprintf_s_l(char *str, size_t count, const char *format,
                         _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnprintf_s_l(str, INT_MAX, count, format, locale, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_s(char *str, size_t len, size_t count,
                        const char *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vsnprintf_s_l(str, len, count, format, NULL, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snprintf_s_l(char *str, size_t len, size_t count,
                          const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnprintf_s_l(str, len, count, format, locale, valist);
    va_end(valist);
    return retval;
}

/* ---- narrow, positional (`_p`) ------------------------------------------ */

/*
 * `_sprintf_p` and `_scwprintf_p` (below) carry wcs.c's own `_MSVCR_VER>=80`
 * gate.  This DLL builds at `_MSVCR_VER=0`, so neither exists here and
 * neither is in msvcrt.spec — defining them unconditionally would have added
 * two symbols the C build never had.  Mirroring the gate rather than deleting
 * the bodies keeps this file a transcription of wcs.c, so a future vintage
 * bump turns both on at the same time on both sides.
 */
#if _MSVCR_VER >= 80
int WINAPIV _sprintf_p(char *buffer, size_t length, const char *format, ...)
{
    va_list valist;
    int r;

    va_start(valist, format);
    r = _vsprintf_p_l(buffer, length, format, NULL, valist);
    va_end(valist);
    return r;
}
#endif

int WINAPIV _sprintf_p_l(char *buffer, size_t length, const char *format,
                         _locale_t locale, ...)
{
    va_list valist;
    int r;

    va_start(valist, locale);
    r = _vsprintf_p_l(buffer, length, format, locale, valist);
    va_end(valist);
    return r;
}

/* ---- narrow, counting sinks (`_sc`) ------------------------------------- */

int WINAPIV _scprintf(const char *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vscprintf(format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _scprintf_l(const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vscprintf_l(format, locale, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _scprintf_p(const char *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vscprintf_p_l(format, NULL, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _scprintf_p_l(const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vscprintf_p_l(format, locale, valist);
    va_end(valist);
    return retval;
}

/* ---- wide, plain sinks --------------------------------------------------- */

int WINAPIV _swprintf(wchar_t *str, const wchar_t *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vsnwprintf(str, INT_MAX, format, ap);
    va_end(ap);
    return r;
}

int WINAPIV __swprintf_l(wchar_t *str, const wchar_t *format,
                         _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vswprintf_l(str, format, locale, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snwprintf(wchar_t *str, size_t len, const wchar_t *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vsnwprintf(str, len, format, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snwprintf_l(wchar_t *str, size_t len, const wchar_t *format,
                         _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnwprintf_l(str, len, format, locale, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _swprintf_c(wchar_t *str, size_t len, const wchar_t *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vswprintf_c(str, len, format, ap);
    va_end(ap);
    return r;
}

int WINAPIV _swprintf_c_l(wchar_t *str, size_t len, const wchar_t *format,
                          _locale_t locale, ...)
{
    va_list ap;
    int r;

    va_start(ap, locale);
    r = _vswprintf_c_l(str, len, format, locale, ap);
    va_end(ap);
    return r;
}

/* ---- wide, bounded (`_s`) ------------------------------------------------ */

int WINAPIV swprintf_s(wchar_t *str, size_t numberOfElements,
                       const wchar_t *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vsnwprintf_s(str, numberOfElements, INT_MAX, format, ap);
    va_end(ap);
    return r;
}

int WINAPIV _swprintf_s_l(wchar_t *str, size_t numberOfElements,
                          const wchar_t *format, _locale_t locale, ...)
{
    va_list ap;
    int r;

    va_start(ap, locale);
    r = _vsnwprintf_s_l(str, numberOfElements, INT_MAX, format, locale, ap);
    va_end(ap);
    return r;
}

int WINAPIV _snwprintf_s(wchar_t *str, size_t len, size_t count,
                         const wchar_t *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vsnwprintf_s_l(str, len, count, format, NULL, valist);
    va_end(valist);
    return retval;
}

int WINAPIV _snwprintf_s_l(wchar_t *str, size_t len, size_t count,
                           const wchar_t *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vsnwprintf_s_l(str, len, count, format, locale, valist);
    va_end(valist);
    return retval;
}

/* ---- wide, positional (`_p`) --------------------------------------------- */

int WINAPIV _swprintf_p(wchar_t *buffer, size_t length,
                        const wchar_t *format, ...)
{
    va_list valist;
    int r;

    va_start(valist, format);
    r = _vswprintf_p_l(buffer, length, format, NULL, valist);
    va_end(valist);
    return r;
}

int WINAPIV _swprintf_p_l(wchar_t *buffer, size_t length,
                          const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int r;

    va_start(valist, locale);
    r = _vswprintf_p_l(buffer, length, format, locale, valist);
    va_end(valist);
    return r;
}

/* ---- wide, counting sinks (`_sc`) ---------------------------------------- */

int WINAPIV _scwprintf(const wchar_t *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vsnwprintf(NULL, INT_MAX, format, ap);
    va_end(ap);
    return r;
}

int WINAPIV _scwprintf_l(const wchar_t *format, _locale_t locale, ...)
{
    va_list ap;
    int r;

    va_start(ap, locale);
    r = _vsnwprintf_l(NULL, INT_MAX, format, locale, ap);
    va_end(ap);
    return r;
}

/* `_MSVCR_VER>=80` in wcs.c — see the note on `_sprintf_p` above. */
#if _MSVCR_VER >= 80
int WINAPIV _scwprintf_p(const wchar_t *format, ...)
{
    va_list ap;
    int r;

    va_start(ap, format);
    r = _vscwprintf_p_l(format, NULL, ap);
    va_end(ap);
    return r;
}
#endif

int WINAPIV _scwprintf_p_l(const wchar_t *format, _locale_t locale, ...)
{
    va_list ap;
    int r;

    va_start(ap, locale);
    r = _vscwprintf_p_l(format, locale, ap);
    va_end(ap);
    return r;
}
