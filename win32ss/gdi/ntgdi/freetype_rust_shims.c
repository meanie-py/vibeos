/*
 * PROJECT:     ReactOS win32 kernel mode subsystem
 * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
 * PURPOSE:     cdecl shims for ntstrsafe header-only inlines that freetype.rs calls.
 *
 * ntstrsafe functions (RtlStringCb, RtlStringCch, RtlULongMult) are FORCEINLINE/static
 * in ntstrsafe.h — they have no exported symbol.  Re-export them through plain-cdecl
 * w32r_* wrappers so the Rust translation can link against them.
 */

#include <win32k.h>
#include <ntstrsafe.h>

NTSTATUS
w32r_RtlStringCbCopyW(WCHAR *d, size_t cb, const WCHAR *s)
{
    return RtlStringCbCopyW(d, cb, s);
}

NTSTATUS
w32r_RtlStringCchCatW(WCHAR *d, size_t cch, const WCHAR *s)
{
    return RtlStringCchCatW(d, cch, s);
}

NTSTATUS
w32r_RtlStringCchCopyNW(WCHAR *d, size_t cch, const WCHAR *s, size_t n)
{
    return RtlStringCchCopyNW(d, cch, s, n);
}

NTSTATUS
w32r_RtlStringCchCopyW(WCHAR *d, size_t cch, const WCHAR *s)
{
    return RtlStringCchCopyW(d, cch, s);
}

NTSTATUS
w32r_RtlULongMult(ULONG a, ULONG b, ULONG *out)
{
    return RtlULongMult(a, b, out);
}

/* RtlStringCbPrintfW is variadic; freetype.rs has exactly one call site with
 * one format argument (a WCHAR* path string).  Use a fixed-arity shim. */
NTSTATUS
w32r_RtlStringCbPrintfW_2(WCHAR *d, size_t cb, const WCHAR *fmt, WCHAR *a1)
{
    return RtlStringCbPrintfW(d, cb, fmt, a1);
}
