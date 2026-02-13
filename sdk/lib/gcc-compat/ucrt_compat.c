/*
 * Compatibility shims for UCRT functions expected by libc++.
 * llvm-mingw's libc++ is compiled against UCRT (Universal CRT) and calls
 * internal __stdio_common_* functions. ReactOS uses msvcrt, so we provide
 * simplified implementations that forward to standard C functions.
 */

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <wchar.h>
#include <errno.h>

/* mingw provides __mingw_vsscanf in libmingwex */
int __cdecl __mingw_vsscanf(const char *str, const char *format, va_list argptr);

/* __stdio_common_vsscanf - used by libc++ for sscanf-like operations */
int __cdecl __stdio_common_vsscanf(
    unsigned __int64 options,
    const char *input,
    size_t length,
    const char *format,
    void *locale,
    va_list valist)
{
    (void)options;
    (void)length;
    (void)locale;
    return __mingw_vsscanf(input, format, valist);
}

/* __stdio_common_vswprintf - used by libc++ for swprintf-like operations */
int __cdecl __stdio_common_vswprintf(
    unsigned __int64 options,
    wchar_t *str,
    size_t len,
    const wchar_t *format,
    void *locale,
    va_list valist)
{
    (void)options;
    (void)locale;
    return _vswprintf_c(str, len, format, valist);
}

/* strtof - C99 function, forward to strtod */
float __cdecl strtof(const char *nptr, char **endptr)
{
    return (float)strtod(nptr, endptr);
}

/* wcstoll - C99 function, forward to _wcstoi64 */
long long __cdecl wcstoll(const wchar_t *nptr, wchar_t **endptr, int base)
{
    return _wcstoi64(nptr, endptr, base);
}

/* wcstoull - C99 function, forward to _wcstoui64 */
unsigned long long __cdecl wcstoull(const wchar_t *nptr, wchar_t **endptr, int base)
{
    return _wcstoui64(nptr, endptr, base);
}

/* _configthreadlocale - needed by libc++ for locale support */
int __cdecl _configthreadlocale(int per_thread_locale_type)
{
    (void)per_thread_locale_type;
    /* ReactOS msvcrt doesn't support per-thread locale; return previous setting (0 = global) */
    return 0;
}

