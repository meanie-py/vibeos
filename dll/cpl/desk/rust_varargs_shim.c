/* rust_varargs_shim.c — desk-rust variadic entry trampolines.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` is only legal inside one —
 * c_variadic, rust-lang/rust#44930). desk reaches three of strsafe.h's variadic entry points,
 * so each one lives here: it does the `va_start` Rust can't, then tail-calls the fixed-arity
 * `*_v` twin in desk-rust (src/desk/strsafe.rs), which holds the real ported body. This is the
 * sanctioned varargs-rust trampoline pattern (AGENTS.md "c_variadic",
 * sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template), NOT a translation reverted to C.
 *
 * WHY THESE THREE ARE HERE AT ALL: `STRSAFEAPI` is `static __inline`, so no object file in the
 * tree defines any of strsafe.h's functions — every C caller inlined its own copy, and flipping
 * desk's .c took the last one with it. The non-variadic ones are ported into strsafe.rs; only
 * the `...` entry points need a C hop.
 *
 * WHY WE PASS THE va_list VALUE (not &ap): each `_v` twin forwards the va_list straight on to
 * _vsnwprintf, whose last parameter IS a va_list. On all three targets (i686/x86_64/aarch64
 * -pc-windows-gnu) `va_list` is `char*` — a single pointer — so handing it through a `void*`
 * is exact.
 *
 * Deliberately NOT #include <strsafe.h>: the header's own `static __inline` definitions have
 * internal linkage and would be a redefinition of the external ones this file provides.
 *
 * CDECL, not WINAPI, even though strsafe.h spells them STRSAFEAPI: no ABI on any of our targets
 * has a variadic stdcall, so a variadic WINAPI function IS cdecl. The Rust side declares them
 * `extern "C"` for the same reason.
 */

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>
#include <windows.h>

extern HRESULT StringCchPrintfW_v(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, void *ap);

HRESULT StringCchPrintfW(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, ...)
{
    HRESULT hr;
    va_list ap;

    va_start(ap, pszFormat);
    hr = StringCchPrintfW_v(pszDest, cchDest, pszFormat, (void *)ap);
    va_end(ap);
    return hr;
}

extern HRESULT StringCbPrintfW_v(WCHAR *pszDest, size_t cbDest, const WCHAR *pszFormat, void *ap);

HRESULT StringCbPrintfW(WCHAR *pszDest, size_t cbDest, const WCHAR *pszFormat, ...)
{
    HRESULT hr;
    va_list ap;

    va_start(ap, pszFormat);
    hr = StringCbPrintfW_v(pszDest, cbDest, pszFormat, (void *)ap);
    va_end(ap);
    return hr;
}

extern HRESULT StringCbPrintfExW_v(WCHAR *pszDest,
                                   size_t cbDest,
                                   WCHAR **ppszDestEnd,
                                   size_t *pcbRemaining,
                                   DWORD dwFlags,
                                   const WCHAR *pszFormat,
                                   void *ap);

HRESULT StringCbPrintfExW(WCHAR *pszDest,
                          size_t cbDest,
                          WCHAR **ppszDestEnd,
                          size_t *pcbRemaining,
                          DWORD dwFlags,
                          const WCHAR *pszFormat,
                          ...)
{
    HRESULT hr;
    va_list ap;

    va_start(ap, pszFormat);
    hr = StringCbPrintfExW_v(pszDest, cbDest, ppszDestEnd, pcbRemaining, dwFlags, pszFormat, (void *)ap);
    va_end(ap);
    return hr;
}
