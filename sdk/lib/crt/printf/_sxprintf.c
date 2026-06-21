/*
 * COPYRIGHT:       GNU GPL, see COPYING in the top level directory
 * PROJECT:         ReactOS crt library
 * FILE:            lib/sdk/crt/printf/_sxprintf.c
 * PURPOSE:         Implementation of swprintf
 * PROGRAMMER:      Timo Kreuzer
 */

#ifdef CRT_PRINTF_HEADERLESS
/*
 * SELF-CONTAINED / HEADERLESS (cmake-off WS1, 2026-06-21): when the consumer
 * compiles this engine via reactos-link's `.c_shims()` (cc-rs direct, NOT a
 * cmake compile_commands.json replay), it passes -DCRT_PRINTF_HEADERLESS and we
 * hand-define the handful of types/macros stdio.h/stdarg.h/limits.h/tchar.h
 * provided, layout-faithful to sdk/include/crt stdio.h + tchar.h. Only the
 * non-secapi path runs here (IS_SECAPI is undefined, so 0, for the sprintf /
 * swprintf / vsprintf / snprintf / vsnprintf shells the cargo cells use), so the
 * safecrt.h surface (errno/MSVCRT_*) is dead and intentionally not provided.
 * Each shell is verified disassembly-identical vs the header build (WS1 harness:
 * cmake command with vs without -DCRT_PRINTF_HEADERLESS). The normal cmake
 * crt.dll build (and the secapi _s shells) take the #else branch unchanged,
 * so the conversion has zero blast radius.
 */
typedef __SIZE_TYPE__     size_t;
typedef __builtin_va_list va_list;
#define va_start(ap, p) __builtin_va_start(ap, p)
#define va_end(ap)      __builtin_va_end(ap)
#define NULL    ((void *)0)
#define INT_MAX 0x7fffffff
#define _IOWRT  0x0002          /* sdk/include/crt/stdio.h */
#define _IOSTRG 0x0040
typedef struct _iobuf            /* sdk/include/crt/stdio.h FILE, layout-faithful */
{
    char *_ptr;
    int   _cnt;
    char *_base;
    int   _flag;
    int   _file;
    int   _charbuf;
    int   _bufsiz;
    char *_tmpfname;
} FILE;
#ifdef _UNICODE
typedef __WCHAR_TYPE__ _TCHAR;   /* unsigned short on Windows targets */
#define _T(x) L ## x
#else
typedef char _TCHAR;
#define _T(x) x
#endif
#else
#include <stdio.h>
#include <stdarg.h>
#include <limits.h>
#include <tchar.h>
#if IS_SECAPI
#include <internal/safecrt.h>
#endif
#endif

#ifdef _UNICODE
#define _tstreamout wstreamout
#else
#define _tstreamout streamout
#endif

#define min(a,b) (((a) < (b)) ? (a) : (b))

int __cdecl _tstreamout(FILE *stream, const _TCHAR *format, va_list argptr);

int
#if defined(USER32_WSPRINTF) && defined(_M_IX86)
__stdcall
#else
__cdecl
#endif
_sxprintf(
    _TCHAR *buffer,
#if IS_SECAPI
    size_t sizeOfBuffer,
#endif
#if USE_COUNT
   size_t count,
#endif
    const _TCHAR *format,
#if USE_VARARGS
    va_list argptr)
#else
    ...)
#endif
{
#if !USE_COUNT
    const size_t count = INT_MAX;
#endif
#if !IS_SECAPI
    const size_t sizeOfBuffer = count;
#endif
#if !USE_VARARGS
    va_list argptr;
#endif
    int result;
    FILE stream;

#if IS_SECAPI
    /* Validate parameters */
    if (MSVCRT_CHECK_PMT(((buffer == NULL) || (format == NULL) || (sizeOfBuffer <= 0))))
    {
        errno = EINVAL;
        return -1;
    }

    /* Limit output to count + 1 characters */
    if (count != -1)
        sizeOfBuffer = min(sizeOfBuffer, count + 1);
#endif

    /* Setup the FILE structure */
    stream._base = (char*)buffer;
    stream._ptr = stream._base;
    stream._charbuf = 0;
    stream._cnt = (int)(sizeOfBuffer * sizeof(_TCHAR));
    stream._bufsiz = 0;
    stream._flag = _IOSTRG | _IOWRT;
    stream._tmpfname = 0;

#if !USE_VARARGS
    va_start(argptr, format);
#endif
    result = _tstreamout(&stream, format, argptr);
#if !USE_VARARGS
    va_end(argptr);
#endif

#if IS_SECAPI
    /* Check for failure or unterminated string */
    if ((result < 0) || (result == sizeOfBuffer))
    {
        /* Null-terminate the buffer at the end */
        buffer[sizeOfBuffer-1] = _T('\0');

        /* Check if we can truncate */
        if (count != _TRUNCATE)
        {
            /* We can't, invoke invalid parameter handler */
            MSVCRT_INVALID_PMT("Buffer is too small", ERANGE);

            /* If we came back, set the buffer to an empty string */
            *buffer = 0;
        }

        /* Return failure */
        return -1;
    }

    /* Null-terminate the buffer after the string */
    buffer[result] = _T('\0');
#else
    /* Only zero terminate if there is enough space left */
    if ((stream._cnt >= sizeof(_TCHAR)) && (stream._ptr))
        *(_TCHAR*)stream._ptr = _T('\0');
#endif

    return result;
}


