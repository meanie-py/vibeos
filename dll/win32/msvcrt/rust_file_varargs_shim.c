/*
 * PROJECT:     ReactOS CRT (vibeos Rust port)
 * LICENSE:     LGPL-2.1-or-later (see file.c, from which these bodies are taken)
 * PURPOSE:     The irreducible C half of file.c: its 20 variadic entry points,
 *              as va_start trampolines over the Rust bodies.
 *
 * This is the file.c counterpart of rust_varargs_shim.c (wcs.c, printf) and
 * rust_scanf_shim.c (scanf.c), and it exists for the same language limit, not
 * for any translation gap: stable Rust cannot DEFINE a C-variadic function
 * (`c_variadic` is nightly, and this project is on pinned stable). RECEIVING an
 * already-started cursor is fine — a `va_list` is a plain `char *` on every
 * Windows target we build — which is why all 16 `v*` delegates below are Rust,
 * in msvcrt-crt-rust/src/msvcrt/file.rs. Only `va_start` is out of reach.
 *
 * Two families live here, and they reach Rust by different routes:
 *
 *   * The 16 printf shells delegate to their `va_list` twin (`vfprintf`,
 *     `_vfprintf_p_l`, ...). Those are external symbols defined in file.rs, so
 *     first-match resolves the call to Rust.
 *
 *   * `_sopen`/`_wsopen` delegate to `_sopen_dispatch`/`_wsopen_dispatch`.
 *     Those are external too — NOT `static`, unlike scanf.c's engines — so the
 *     same first-match rule applies and no separate mechanism is needed.
 *
 *   * `_open`/`_wopen` delegate to `_sopen`/`_wsopen`, which are defined right
 *     here, so those two calls stay inside this TU. That is intentional and
 *     safe: the callee is itself only a trampoline, and its real work is the
 *     external `*_dispatch` call. Even if clang inlines `_sopen` into `_open`,
 *     the inlined body still calls `_sopen_dispatch` by symbol, so the flip
 *     cannot be silently undone the way wcs.c's same-TU engine undid wave 14b.
 *
 * WHY ITS OWN TU, and not a block left behind in file.c: symbol replacement
 * flips a TU's CALLERS, not its HOUSEMATES. Left in file.c these trampolines
 * would keep calling file.c's own C bodies — intra-TU inlining never consults
 * the linker — and the flip would be a green build over dead Rust. From here
 * every delegate is an external symbol.
 *
 * Link order carries the same weight: this file is compiled into the msvcrt
 * MODULE target, not into msvcrt_static. Module objects are resolved before any
 * archive member, so these definitions win first-match over file.o's copies
 * (--allow-multiple-definition), exactly as --whole-archive does for
 * libmsvcrt_crt_cargo.a.
 *
 * Every body below is file.c verbatim. There are no rewrites in this TU — note
 * in particular that `_fprintf_p` delegates to `_vfprintf_p_l(..., NULL, ...)`
 * and NOT to `_vfprintf_p`, and that the `_l` shells anchor `va_start` on
 * `locale` rather than `format`. Both are load-bearing and both are checked by
 * tools/scan_varargs_shim_parity.py, which diffs this file against file.c.
 */

#define _NO_CRT_STDIO_INLINE
#include <fcntl.h>   /* _O_CREAT, for the four _open-family bodies */
#include <share.h>   /* _SH_DENYNO, likewise — msvcrt.h pulls in neither */
#include <stdarg.h>
#include <wchar.h>

#include "msvcrt.h"

/*
 * The CRT headers gate most of these delegates behind `_MSVCR_VER>=80`, and this
 * DLL builds at `_MSVCR_VER=0` — so at this vintage they are exported but not
 * declared. file.c gets away with calling them because it DEFINES them earlier in
 * the same file; from a separate TU they are undeclared, and an implicit
 * declaration would silently assume `int(...)`, mis-passing the `_locale_t` and
 * `va_list` arguments. Prototypes below are copied from the definitions in file.c,
 * so a drift in either becomes a compile error here.
 */
int CDECL vfprintf_s(FILE* file, const char *format, va_list valist);
int CDECL vfwprintf_s(FILE* file, const wchar_t *format, va_list valist);
int CDECL vprintf_s(const char *format, va_list valist);
int CDECL vwprintf_s(const wchar_t *format, va_list valist);
int CDECL _vfprintf_l(FILE* file, const char *format, _locale_t locale, va_list valist);
int CDECL _vfwprintf_l(FILE* file, const wchar_t *format, _locale_t locale, va_list valist);
int CDECL _vfprintf_p_l(FILE* file, const char *format, _locale_t locale, va_list valist);
int CDECL _vfwprintf_p_l(FILE* file, const wchar_t *format, _locale_t locale, va_list valist);
int CDECL _vfprintf_s_l(FILE* file, const char *format, _locale_t locale, va_list valist);
int CDECL _vfwprintf_s_l(FILE* file, const wchar_t *format, _locale_t locale, va_list valist);

/*
 * The two open dispatchers, likewise: file.c declares them at its own top, and
 * they are not in any public header at this vintage.
 */
int CDECL _sopen_dispatch(const char *path, int oflags, int shflags, int pmode,
                          int *fd, int secure);
int CDECL _wsopen_dispatch(const wchar_t *path, int oflags, int shflags, int pmode,
                           int *fd, int secure);

/*********************************************************************
 *              _wsopen (MSVCRT.@)
 */
int WINAPIV _wsopen( const wchar_t *path, int oflags, int shflags, ... )
{
  int pmode;
  int fd;

  if (oflags & _O_CREAT)
  {
    va_list ap;

    va_start(ap, shflags);
    pmode = va_arg(ap, int);
    va_end(ap);
  }
  else
    pmode = 0;

  return _wsopen_dispatch(path, oflags, shflags, pmode, &fd, 0) ? -1 : fd;
}

/*********************************************************************
 *              _sopen (MSVCRT.@)
 */
int WINAPIV _sopen( const char *path, int oflags, int shflags, ... )
{
  int pmode;
  int fd;

  if (oflags & _O_CREAT)
  {
    va_list ap;

    va_start(ap, shflags);
    pmode = va_arg(ap, int);
    va_end(ap);
  }
  else
    pmode = 0;

  return _sopen_dispatch(path, oflags, shflags, pmode, &fd, 0) ? -1 : fd;
}

/*********************************************************************
 *              _open (MSVCRT.@)
 */
int WINAPIV _open( const char *path, int flags, ... )
{
  va_list ap;

  if (flags & _O_CREAT)
  {
    int pmode;
    va_start(ap, flags);
    pmode = va_arg(ap, int);
    va_end(ap);
    return _sopen( path, flags, _SH_DENYNO, pmode );
  }
  else
    return _sopen( path, flags, _SH_DENYNO);
}

/*********************************************************************
 *              _wopen (MSVCRT.@)
 */
int WINAPIV _wopen(const wchar_t *path,int flags,...)
{
  va_list ap;

  if (flags & _O_CREAT)
  {
    int pmode;
    va_start(ap, flags);
    pmode = va_arg(ap, int);
    va_end(ap);
    return _wsopen( path, flags, _SH_DENYNO, pmode );
  }
  else
    return _wsopen( path, flags, _SH_DENYNO);
}

/*********************************************************************
 *		fprintf (MSVCRT.@)
 */
int WINAPIV fprintf(FILE* file, const char *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vfprintf(file, format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		fprintf_s (MSVCRT.@)
 */
int WINAPIV fprintf_s(FILE* file, const char *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vfprintf_s(file, format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *    _fprintf_l (MSVCRT.@)
 */
int WINAPIV _fprintf_l(FILE* file, const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;
    va_start(valist, locale);
    res = _vfprintf_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *    _fprintf_p (MSVCRT.@)
 */
int WINAPIV _fprintf_p(FILE* file, const char *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = _vfprintf_p_l(file, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *    _fprintf_p_l (MSVCRT.@)
 */
int WINAPIV _fprintf_p_l(FILE* file, const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;
    va_start(valist, locale);
    res = _vfprintf_p_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *    _fprintf_s_l (MSVCRT.@)
 */
int WINAPIV _fprintf_s_l(FILE* file, const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;
    va_start(valist, locale);
    res = _vfprintf_s_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		fwprintf (MSVCRT.@)
 */
int WINAPIV fwprintf(FILE* file, const wchar_t *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vfwprintf(file, format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		fwprintf_s (MSVCRT.@)
 */
int WINAPIV fwprintf_s(FILE* file, const wchar_t *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vfwprintf_s(file, format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *              _fwprintf_l (MSVCRT.@)
 */
int WINAPIV _fwprintf_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;
    va_start(valist, locale);
    res = _vfwprintf_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *              _fwprintf_p (MSVCRT.@)
 */
int WINAPIV _fwprintf_p(FILE* file, const wchar_t *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = _vfwprintf_p_l(file, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *              _fwprintf_p_l (MSVCRT.@)
 */
int WINAPIV _fwprintf_p_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;
    va_start(valist, locale);
    res = _vfwprintf_p_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *              _fwprintf_s_l (MSVCRT.@)
 */
int WINAPIV _fwprintf_s_l(FILE* file, const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;
    va_start(valist, locale);
    res = _vfwprintf_s_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		printf (MSVCRT.@)
 */
int WINAPIV printf(const char *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vfprintf(stdout, format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		printf_s (MSVCRT.@)
 */
int WINAPIV printf_s(const char *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vprintf_s(format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		wprintf (MSVCRT.@)
 */
int WINAPIV wprintf(const wchar_t *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vwprintf(format, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		wprintf_s (MSVCRT.@)
 */
int WINAPIV wprintf_s(const wchar_t *format, ...)
{
    va_list valist;
    int res;
    va_start(valist, format);
    res = vwprintf_s(format, valist);
    va_end(valist);
    return res;
}
