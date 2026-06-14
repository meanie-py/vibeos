/*
 * conutils Rust cohabitation — C variadic forwarders.
 *
 * The console formatting entry points (ConPrintf, ConResPrintf, ConResPrintfEx,
 * ConMsgPrintf, ConResMsgPrintf, ConResMsgPrintfEx) are variadic. Rust (stable)
 * cannot define C variadic functions, so conutils-rust implements only the
 * va_list-taking *V cores (ConPrintfV, ConResPrintfExV, ...) and these thin
 * __cdecl forwarders bridge `...` -> va_list. The original outstream.c carried
 * them, but it is no longer built (replaced by sdk/lib/rust/conutils-rust), so
 * they live here as part of the cohab C TU.
 *
 * Why this file exists: on x86 the linker silently pulled these forwarders from a
 * STALE pre-flip libconutils.a; a clean arm64 tree (correctly stub-only) had no
 * provider, so every console app (cmd.exe et al.) failed to link with undefined
 * ConResPrintf/ConPrintf/&c. Bodies are copied verbatim from outstream.c.
 */
#define UNICODE
#define _UNICODE

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>
#include <winnls.h>
#include <winuser.h>
#include <wincon.h>

#include "conutils.h"
#include "stream.h"

INT
__cdecl
ConPrintf(
    IN PCON_STREAM Stream,
    IN PCWSTR szStr,
    ...)
{
    INT Len;
    va_list args;

    va_start(args, szStr);
    Len = ConPrintfV(Stream, szStr, args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConResPrintfEx(
    IN PCON_STREAM Stream,
    IN HINSTANCE hInstance OPTIONAL,
    IN UINT   uID,
    IN LANGID LanguageId,
    ...)
{
    INT Len;
    va_list args;

    va_start(args, LanguageId);
    Len = ConResPrintfExV(Stream, hInstance, uID, LanguageId, args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConResPrintf(
    IN PCON_STREAM Stream,
    IN UINT uID,
    ...)
{
    INT Len;
    va_list args;

    va_start(args, uID);
    Len = ConResPrintfV(Stream, uID, args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConMsgPrintf(
    IN PCON_STREAM Stream,
    IN DWORD   dwFlags,
    IN LPCVOID lpSource OPTIONAL,
    IN DWORD   dwMessageId,
    IN DWORD   dwLanguageId,
    ...)
{
    INT Len;
    va_list args;

    /* Sanitize dwFlags */
    dwFlags &= ~FORMAT_MESSAGE_ARGUMENT_ARRAY;

    va_start(args, dwLanguageId);
    Len = ConMsgPrintfV(Stream,
                        dwFlags,
                        lpSource,
                        dwMessageId,
                        dwLanguageId,
                        &args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConResMsgPrintfEx(
    IN PCON_STREAM Stream,
    IN HINSTANCE hInstance OPTIONAL,
    IN DWORD  dwFlags,
    IN UINT   uID,
    IN LANGID LanguageId,
    ...)
{
    INT Len;
    va_list args;

    /* Sanitize dwFlags */
    dwFlags &= ~FORMAT_MESSAGE_ARGUMENT_ARRAY;

    va_start(args, LanguageId);
    Len = ConResMsgPrintfExV(Stream,
                             hInstance,
                             dwFlags,
                             uID,
                             LanguageId,
                             &args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConResMsgPrintf(
    IN PCON_STREAM Stream,
    IN DWORD dwFlags,
    IN UINT  uID,
    ...)
{
    INT Len;
    va_list args;

    /* Sanitize dwFlags */
    dwFlags &= ~FORMAT_MESSAGE_ARGUMENT_ARRAY;

    va_start(args, uID);
    Len = ConResMsgPrintfV(Stream, dwFlags, uID, &args);
    va_end(args);

    return Len;
}
