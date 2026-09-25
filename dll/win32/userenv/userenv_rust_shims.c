/* INERT since 2026-09-25: no longer in the build. Its varargs moved to Rust (strsafe-rust's
 * StringC*PrintfW! macros call msvcrt's variadic _snwprintf; the module's own variadics RECEIVE
 * their arguments the va-slots way). Kept on disk as the reference the Rust was checked against. */
/*
 * userenv_rust_shims.c — C shims for the userenv Rust cohabitation flip.
 *
 * Two things the Rust cohab (sdk/lib/rust/userenv-rust) references but stable Rust / the
 * translation cannot provide as linkable symbols:
 *
 *  1. Data globals `hInstance` (userenv.c) and `LocalSystemAuthority` (misc.c) — defined in the
 *     now-Rust .c files, but the translation dropped the file-scope definitions (it emitted them
 *     as bindgen `extern` imports). Provide the storage here; the Rust reads/writes them via the
 *     import (Ws2HelpDllMain-style: the Rust DllMain sets hInstance through the same symbol).
 *
 *  2. StringCchPrintfW — strsafe.h ships it only as a `static inline` (no linkable symbol), and
 *     it is VARIADIC, which stable Rust cannot define. Provide a real out-of-line __cdecl symbol
 *     via _vsnwprintf. (Do NOT #include <strsafe.h> here — that would redeclare the inline.)
 */

#define WIN32_NO_STATUS
#define _INC_WINDOWS
#define COM_NO_WINDOWS_H

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>
#include <winnt.h>

/* userenv.c:32  HINSTANCE hInstance = NULL;  (DllMain sets it via this same symbol) */
HINSTANCE hInstance = NULL;

/* misc.c:35  SID_IDENTIFIER_AUTHORITY LocalSystemAuthority = {SECURITY_NT_AUTHORITY}; */
SID_IDENTIFIER_AUTHORITY LocalSystemAuthority = { SECURITY_NT_AUTHORITY };

int __cdecl _vsnwprintf(wchar_t *buffer, size_t count, const wchar_t *format, va_list argptr);

HRESULT __cdecl StringCchPrintfW(wchar_t *pszDest, size_t cchDest, const wchar_t *pszFormat, ...)
{
    va_list args;
    int r;

    if (cchDest == 0)
        return (HRESULT)0x80070057L; /* STRSAFE_E_INVALID_PARAMETER */

    va_start(args, pszFormat);
    r = _vsnwprintf(pszDest, cchDest, pszFormat, args);
    va_end(args);

    if (r < 0 || (size_t)r >= cchDest)
    {
        pszDest[cchDest - 1] = 0;
        return (HRESULT)0x8007007AL; /* STRSAFE_E_INSUFFICIENT_BUFFER */
    }
    return (HRESULT)0L; /* S_OK */
}
