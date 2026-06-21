/*
 * COPYRIGHT:       GNU GPL, see COPYING in the top level directory
 * PROJECT:         ReactOS crt library
 * FILE:            lib/sdk/crt/printf/_vscwprintf.c
 * PURPOSE:         Implementation of _vscwprintf
 */

#ifdef CRT_PRINTF_HEADERLESS
/*
 * SELF-CONTAINED / HEADERLESS (cmake-off WS1, 2026-06-21): compiled via
 * reactos-link .c_shims() with -DCRT_PRINTF_HEADERLESS instead of a cmake
 * compile_commands.json replay. Hand-defined FILE/va_list/flags layout-faithful
 * to sdk/include/crt stdio.h; verified disassembly-identical vs the header build
 * (WS1 harness). The normal cmake crt.dll build takes the #else branch.
 */
typedef __builtin_va_list va_list;
typedef __WCHAR_TYPE__ wchar_t;
#define NULL    ((void *)0)
#define _IOWRT  0x0002
#define _IOSTRG 0x0040
typedef struct _iobuf
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
#else
#include <stdio.h>
#include <stdarg.h>
#endif

int __cdecl wstreamout(FILE *stream, const wchar_t *format, va_list argptr);

int
__cdecl
_vscwprintf(
    const wchar_t *format,
    va_list argptr)
{
    FILE nulfile;
    nulfile._tmpfname = nulfile._ptr = nulfile._base = NULL;
    nulfile._bufsiz = nulfile._charbuf = nulfile._cnt = 0;
    nulfile._flag = _IOSTRG | _IOWRT;
    return wstreamout(&nulfile, format, argptr);
}
