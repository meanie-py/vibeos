/*
 * PROJECT:     ReactOS CRT (vibeos Rust port)
 * LICENSE:     LGPL-2.1-or-later (see console.c, from which these bodies are taken)
 * PURPOSE:     The irreducible C half of console.c: its four `_cprintf`/`_cwprintf`
 *              WINAPIV variadic entry points, as va_start trampolines over the Rust
 *              bodies in msvcrt-crt-rust/src/msvcrt/console.rs.
 *
 * The console.c counterpart of rust_varargs_shim.c (wcs.c, printf), rust_scanf_shim.c
 * (scanf.c), rust_file_varargs_shim.c (file.c) and rust_process_varargs_shim.c
 * (process.c), for the same language limit and not for any translation gap: stable Rust
 * cannot DEFINE a C-variadic function (`c_variadic` is nightly, and this project is on
 * pinned stable). RECEIVING an already-started `va_list` is fine — it is a plain pointer
 * on every Windows target we build — so the four `_v` twins these call are Rust.
 *
 * Unlike process.c's shim, no twin had to be invented here: `_vcprintf`, `_vcprintf_l`,
 * `_vcwprintf` and `_vcwprintf_l` are console.c's OWN va_list forms and are real
 * msvcrt.spec exports, so the trampolines below are the C verbatim.
 *
 * WHY ITS OWN TU, and not a block left behind in console.c: symbol replacement flips a
 * TU's CALLERS, not its HOUSEMATES. Left in console.c these trampolines would keep
 * calling console.c's own C bodies — intra-TU inlining never consults the linker — and
 * the flip would be a green build over dead Rust.
 *
 * Link order carries the same weight: this file is compiled into the msvcrt MODULE
 * target, not into msvcrt_static. Module objects are resolved before any archive member,
 * so these definitions win first-match over console.o's copies
 * (--allow-multiple-definition), exactly as --whole-archive does for
 * libmsvcrt_crt_cargo.a.
 */

#include <stdarg.h>

#include "msvcrt.h"

/*
 * The four Rust twins. These ARE exported (msvcrt.spec) and declared in the CRT headers,
 * but declaring them here keeps this TU self-contained and makes an accidental prototype
 * drift a compile error rather than a silently mis-passed `va_list`.
 */
int CDECL _vcprintf(const char *format, va_list valist);
int CDECL _vcprintf_l(const char *format, _locale_t locale, va_list valist);
int CDECL _vcwprintf(const wchar_t *format, va_list valist);
int CDECL _vcwprintf_l(const wchar_t *format, _locale_t locale, va_list valist);

/*********************************************************************
 *		_cprintf_l (MSVCRT.@)
 */
int WINAPIV _cprintf_l(const char *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vcprintf_l(format, locale, valist);
    va_end(valist);

    return retval;
}

/*********************************************************************
 *		_cprintf (MSVCRT.@)
 */
int WINAPIV _cprintf(const char *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vcprintf(format, valist);
    va_end(valist);

    return retval;
}

/*********************************************************************
 *		_cwprintf_l (MSVCRT.@)
 */
int WINAPIV _cwprintf_l(const wchar_t *format, _locale_t locale, ...)
{
    int retval;
    va_list valist;

    va_start(valist, locale);
    retval = _vcwprintf_l(format, locale, valist);
    va_end(valist);

    return retval;
}

/*********************************************************************
 *		_cwprintf (MSVCRT.@)
 */
int WINAPIV _cwprintf(const wchar_t *format, ...)
{
    int retval;
    va_list valist;

    va_start(valist, format);
    retval = _vcwprintf(format, valist);
    va_end(valist);

    return retval;
}
