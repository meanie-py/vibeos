/*
 * C variadic front-ends for conutils-rust.
 *
 * Stable Rust cannot define C variadic functions, so the Rust translation owns
 * the V-style implementations and this shim preserves the public C ABI.
 */

/* FIXME: Temporary HACK before we cleanly support UNICODE functions */
#define UNICODE
#define _UNICODE

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>

#include "conutils.h"

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
    IN UINT uID,
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
    IN DWORD dwFlags,
    IN LPCVOID lpSource OPTIONAL,
    IN DWORD dwMessageId,
    IN DWORD dwLanguageId,
    ...)
{
    INT Len;
    va_list args;

    dwFlags &= ~FORMAT_MESSAGE_ARGUMENT_ARRAY;
    va_start(args, dwLanguageId);
    Len = ConMsgPrintfV(Stream, dwFlags, lpSource, dwMessageId, dwLanguageId, &args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConResMsgPrintfEx(
    IN PCON_STREAM Stream,
    IN HINSTANCE hInstance OPTIONAL,
    IN DWORD dwFlags,
    IN UINT uID,
    IN LANGID LanguageId,
    ...)
{
    INT Len;
    va_list args;

    dwFlags &= ~FORMAT_MESSAGE_ARGUMENT_ARRAY;
    va_start(args, LanguageId);
    Len = ConResMsgPrintfExV(Stream, hInstance, dwFlags, uID, LanguageId, &args);
    va_end(args);

    return Len;
}

INT
__cdecl
ConResMsgPrintf(
    IN PCON_STREAM Stream,
    IN DWORD dwFlags,
    IN UINT uID,
    ...)
{
    INT Len;
    va_list args;

    dwFlags &= ~FORMAT_MESSAGE_ARGUMENT_ARRAY;
    va_start(args, uID);
    Len = ConResMsgPrintfV(Stream, dwFlags, uID, &args);
    va_end(args);

    return Len;
}
