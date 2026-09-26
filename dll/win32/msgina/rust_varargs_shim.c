/* INERT since 2026-09-25: no longer in the build. Its varargs moved to Rust (strsafe-rust's
 * StringC*PrintfW! macros call msvcrt's variadic _snwprintf; the module's own variadics RECEIVE
 * their arguments the va-slots way). Kept on disk as the reference the Rust was checked against. */
/* rust_varargs_shim.c — msgina-rust variadic entry trampolines.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` is only legal inside one —
 * c_variadic, rust-lang/rust#44930). msgina reaches two of strsafe.h's variadic entry points,
 * so each one lives here: it does the `va_start` Rust can't, then tail-calls the fixed-arity
 * `*_v` twin in the shared strsafe-rust crate, which holds the real ported body. This is the
 * sanctioned varargs-rust trampoline pattern
 * (sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template), NOT a translation reverted to C.
 *
 * WHY msgina NEEDS THIS AT ALL — it did not have one, and that was the bug. The two entry
 * points were Rust MACROS that discarded the count argument and forwarded to user32's
 * `wsprintfW`, which bounds to 1024 wchars internally. shutdown.rs formats a resource-loaded
 * format string plus the user name into a `WCHAR[256]` on the stack, so a long user name wrote
 * past the end of it; the C truncated and returned STRSAFE_E_INSUFFICIENT_BUFFER. See the
 * roadmap's bug registry.
 *
 * `STRSAFEAPI` is `static __inline`, so no object file in the tree defines any of strsafe.h's
 * functions — which is why the bodies had to be ported at all. They now live in ONE place
 * (sdk/lib/rust/strsafe-rust) rather than once per flipped module.
 *
 * WHY WE PASS `&ap` AND NOT `ap`: handing Rust a POINTER to the va_list avoids assuming a
 * va_list is itself a single pointer. It happens to be `char*` on all three of our Windows
 * targets, but it is an array-of-struct on SysV x86-64, and the by-pointer form is correct on
 * both — which is also what lets the mechanism be exercised natively on the host.
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

extern HRESULT StringCchPrintfW_v(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, void *app);

HRESULT StringCchPrintfW(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, ...)
{
    HRESULT hr;
    va_list ap;

    va_start(ap, pszFormat);
    hr = StringCchPrintfW_v(pszDest, cchDest, pszFormat, (void *)&ap);
    va_end(ap);
    return hr;
}

extern HRESULT StringCbPrintfW_v(WCHAR *pszDest, size_t cbDest, const WCHAR *pszFormat, void *app);

HRESULT StringCbPrintfW(WCHAR *pszDest, size_t cbDest, const WCHAR *pszFormat, ...)
{
    HRESULT hr;
    va_list ap;

    va_start(ap, pszFormat);
    hr = StringCbPrintfW_v(pszDest, cbDest, pszFormat, (void *)&ap);
    va_end(ap);
    return hr;
}
