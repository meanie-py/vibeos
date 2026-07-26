/*
 * PROJECT:     ReactOS CRT (vibeos Rust port)
 * LICENSE:     LGPL-2.1-or-later (see scanf.c, from which these bodies are taken)
 * PURPOSE:     The irreducible C half of the scanf family: the 40 variadic entry
 *              points, as va_start trampolines over the Rust engine.
 *
 * Everything scanf in msvcrt is Rust now EXCEPT the `...` shells, and the reason is a
 * language limit, not a translation gap: stable Rust cannot DEFINE a C-variadic
 * function (`c_variadic` is nightly, and this project is on pinned stable). Receiving
 * an already-started cursor is fine — that is what msvcrt-crt-rust/src/scanf.rs does
 * for all sixteen `v*scanf*_l` engines and for the four `__stdio_common_v*scanf`
 * entry points, since a `va_list` is a plain `char *` on every Windows target we
 * build. Only `va_start` is out of reach. So these stay C, and each is the same four
 * lines the genuine scanf.c has: start the cursor, hand it to the Rust engine, end it.
 *
 * Note the two shapes, which are the only thing that varies: the `_l` forms take the
 * locale as their last NAMED parameter and so say `va_start(valist, locale)`; the
 * non-`_l` forms say `va_start(valist, format)` — except `_snscanf`/`_snwscanf`, whose
 * last named parameter is also `format` because `length` precedes it. Getting one of
 * these wrong shifts every argument by one slot and is invisible to the compiler.
 *
 * WHY THIS IS ITS OWN TU, and not a block left behind in scanf.c:
 *
 *   Symbol replacement flips a TU's CALLERS, not its HOUSEMATES.  The sixteen engines
 *   are `static` in scanf.c, so scanf.c's own shells do not even go through the
 *   linker to reach them — there is no symbol to replace and no first-match to win.
 *   Left in scanf.c these trampolines would call the C engines unconditionally and
 *   the flip would be silently undone: a green build over dead Rust.  (The printf
 *   lane learned the same lesson the harder way, through clang's intra-TU IPA
 *   specialization — see rust_varargs_shim.c.)  From here the delegates are external
 *   symbols in msvcrt-crt-rust and there is nothing to inline.
 *
 * Link order carries the same weight: this file is compiled into the msvcrt MODULE
 * target, not into msvcrt_static.  Module objects are resolved before any archive
 * member, so these definitions win first-match over scanf.o's copies
 * (--allow-multiple-definition), exactly as --whole-archive does for
 * libmsvcrt_crt_cargo.a.  Putting it in msvcrt_static instead would make the outcome
 * depend on member order within one archive.
 *
 * Every body below is scanf.c verbatim.  `__stdio_common_vsscanf`,
 * `__stdio_common_vswscanf`, `__stdio_common_vfscanf` and `__stdio_common_vfwscanf`
 * are NOT here: they take an already-started `va_list`, so they are pure Rust
 * (msvcrt-crt-rust/src/scanf.rs).  Neither are `vsscanf`/`vswscanf`, which scanf.c
 * gates behind `#if _MSVCR_VER==120` and this DLL builds at `_MSVCR_VER=0`.
 */

#define _NO_CRT_STDIO_INLINE
#include <stdarg.h>
#include <stdio.h>
#include <wchar.h>

#include "msvcrt.h"

/*
 * The sixteen engines are `static int` in scanf.c and appear in no header, so there is
 * nothing to include.  These prototypes are copied from the `_FUNCTION_` expansions in
 * scanf.h:53-146 — one per WIDE_SCANF/SECURE/STRING/STRING_LEN/CONSOLE combination —
 * and they are what msvcrt-crt-rust/src/scanf.rs defines.  An implicit declaration
 * would silently assume `int(...)` and mis-pass the `size_t` and pointer arguments.
 */
int CDECL vfscanf_l(FILE *file, const char *format, _locale_t locale, va_list ap);
int CDECL vfscanf_s_l(FILE *file, const char *format, _locale_t locale, va_list ap);
int CDECL vfwscanf_l(FILE *file, const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vfwscanf_s_l(FILE *file, const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vsscanf_l(const char *file, const char *format, _locale_t locale, va_list ap);
int CDECL vsscanf_s_l(const char *file, const char *format, _locale_t locale, va_list ap);
int CDECL vswscanf_l(const wchar_t *file, const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vswscanf_s_l(const wchar_t *file, const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vsnscanf_l(const char *file, size_t length, const char *format, _locale_t locale, va_list ap);
int CDECL vsnscanf_s_l(const char *file, size_t length, const char *format, _locale_t locale, va_list ap);
int CDECL vsnwscanf_l(const wchar_t *file, size_t length, const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vsnwscanf_s_l(const wchar_t *file, size_t length, const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vcscanf_l(const char *format, _locale_t locale, va_list ap);
int CDECL vcscanf_s_l(const char *format, _locale_t locale, va_list ap);
int CDECL vcwscanf_l(const wchar_t *format, _locale_t locale, va_list ap);
int CDECL vcwscanf_s_l(const wchar_t *format, _locale_t locale, va_list ap);

/* ---- narrow FILE (scanf.c:147-203) --------------------------------------- */

/*********************************************************************
 *		fscanf (MSVCRT.@)
 */
int WINAPIV fscanf(FILE *file, const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfscanf_l(file, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_fscanf_l (MSVCRT.@)
 */
int WINAPIV _fscanf_l(FILE *file, const char *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfscanf_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		fscanf_s (MSVCRT.@)
 */
int WINAPIV fscanf_s(FILE *file, const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfscanf_s_l(file, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_fscanf_s_l (MSVCRT.@)
 */
int WINAPIV _fscanf_s_l(FILE *file, const char *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfscanf_s_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- narrow stdin (scanf.c:205-259) -------------------------------------- */

/*********************************************************************
 *		scanf (MSVCRT.@)
 */
int WINAPIV scanf(const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfscanf_l(stdin, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_scanf_l (MSVCRT.@)
 */
int WINAPIV _scanf_l(const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfscanf_l(stdin, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		scanf_s (MSVCRT.@)
 */
int WINAPIV scanf_s(const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfscanf_s_l(stdin, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_scanf_s_l (MSVCRT.@)
 */
int WINAPIV _scanf_s_l(const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfscanf_s_l(stdin, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- wide FILE (scanf.c:261-317) ----------------------------------------- */

/*********************************************************************
 *		fwscanf (MSVCRT.@)
 */
int WINAPIV fwscanf(FILE *file, const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfwscanf_l(file, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_fwscanf_l (MSVCRT.@)
 */
int WINAPIV _fwscanf_l(FILE *file, const wchar_t *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfwscanf_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		fwscanf_s (MSVCRT.@)
 */
int WINAPIV fwscanf_s(FILE *file, const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfwscanf_s_l(file, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_fwscanf_s_l (MSVCRT.@)
 */
int WINAPIV _fwscanf_s_l(FILE *file, const wchar_t *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfwscanf_s_l(file, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- wide stdin (scanf.c:319-375) ---------------------------------------- */

/*********************************************************************
 *		wscanf (MSVCRT.@)
 */
int WINAPIV wscanf(const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfwscanf_l(stdin, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_wscanf_l (MSVCRT.@)
 */
int WINAPIV _wscanf_l(const wchar_t *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfwscanf_l(stdin, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		wscanf_s (MSVCRT.@)
 */
int WINAPIV wscanf_s(const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vfwscanf_s_l(stdin, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_wscanf_s_l (MSVCRT.@)
 */
int WINAPIV _wscanf_s_l(const wchar_t *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vfwscanf_s_l(stdin, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- narrow string (scanf.c:377-433) ------------------------------------- */

/*********************************************************************
 *		sscanf (MSVCRT.@)
 */
int WINAPIV sscanf(const char *str, const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vsscanf_l(str, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_sscanf_l (MSVCRT.@)
 */
int WINAPIV _sscanf_l(const char *str, const char *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vsscanf_l(str, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		sscanf_s (MSVCRT.@)
 */
int WINAPIV sscanf_s(const char *str, const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vsscanf_s_l(str, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_sscanf_s_l (MSVCRT.@)
 */
int WINAPIV _sscanf_s_l(const char *str, const char *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vsscanf_s_l(str, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- wide string (scanf.c:435-491) --------------------------------------- */

/*********************************************************************
 *		swscanf (MSVCRT.@)
 */
int WINAPIV swscanf(const wchar_t *str, const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vswscanf_l(str, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_swscanf_l (MSVCRT.@)
 */
int WINAPIV _swscanf_l(const wchar_t *str, const wchar_t *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vswscanf_l(str, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		swscanf_s (MSVCRT.@)
 */
int WINAPIV swscanf_s(const wchar_t *str, const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vswscanf_s_l(str, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_swscanf_s_l (MSVCRT.@)
 */
int WINAPIV _swscanf_s_l(const wchar_t *str, const wchar_t *format,
        _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vswscanf_s_l(str, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- narrow console (scanf.c:493-547) ------------------------------------ */

/*********************************************************************
 *		_cscanf (MSVCRT.@)
 */
int WINAPIV _cscanf(const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vcscanf_l(format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_cscanf_l (MSVCRT.@)
 */
int WINAPIV _cscanf_l(const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vcscanf_l(format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_cscanf_s (MSVCRT.@)
 */
int WINAPIV _cscanf_s(const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vcscanf_s_l(format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_cscanf_s_l (MSVCRT.@)
 */
int WINAPIV _cscanf_s_l(const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vcscanf_s_l(format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- wide console (scanf.c:549-603) -------------------------------------- */

/*********************************************************************
 *		_cwscanf (MSVCRT.@)
 */
int WINAPIV _cwscanf(const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vcwscanf_l(format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_cwscanf_l (MSVCRT.@)
 */
int WINAPIV _cwscanf_l(const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vcwscanf_l(format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_cwscanf_s (MSVCRT.@)
 */
int WINAPIV _cwscanf_s(const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vcwscanf_s_l(format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_cwscanf_s_l (MSVCRT.@)
 */
int WINAPIV _cwscanf_s_l(const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vcwscanf_s_l(format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- narrow counted string (scanf.c:605-661) ----------------------------- */

/*********************************************************************
 *		_snscanf (MSVCRT.@)
 */
int WINAPIV _snscanf(const char *input, size_t length, const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vsnscanf_l(input, length, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_snscanf_l (MSVCRT.@)
 */
int WINAPIV _snscanf_l(const char *input, size_t length,
        const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vsnscanf_l(input, length, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_snscanf_s (MSVCRT.@)
 */
int WINAPIV _snscanf_s(const char *input, size_t length, const char *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vsnscanf_s_l(input, length, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_snscanf_s_l (MSVCRT.@)
 */
int WINAPIV _snscanf_s_l(const char *input, size_t length,
        const char *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vsnscanf_s_l(input, length, format, locale, valist);
    va_end(valist);
    return res;
}

/* ---- wide counted string (scanf.c:738-796) ------------------------------- */

/*
 * `_snwscanf` and `_snwscanf_s` take a NON-const `wchar_t *input` in scanf.c while
 * their narrow twins take `const char *`.  That asymmetry is upstream's and is
 * transcribed, not tidied: the spelling is part of the exported prototype.
 */

/*********************************************************************
 *		_snwscanf (MSVCRT.@)
 */
int WINAPIV _snwscanf(wchar_t *input, size_t length,
        const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vsnwscanf_l(input, length, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_snwscanf_l (MSVCRT.@)
 */
int WINAPIV _snwscanf_l(wchar_t *input, size_t length,
        const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vsnwscanf_l(input, length, format, locale, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_snwscanf_s (MSVCRT.@)
 */
int WINAPIV _snwscanf_s(wchar_t *input, size_t length,
        const wchar_t *format, ...)
{
    va_list valist;
    int res;

    va_start(valist, format);
    res = vsnwscanf_s_l(input, length, format, NULL, valist);
    va_end(valist);
    return res;
}

/*********************************************************************
 *		_snscanf_s_l (MSVCRT.@)
 */
int WINAPIV _snwscanf_s_l(wchar_t *input, size_t length,
        const wchar_t *format, _locale_t locale, ...)
{
    va_list valist;
    int res;

    va_start(valist, locale);
    res = vsnwscanf_s_l(input, length, format, locale, valist);
    va_end(valist);
    return res;
}
