/* rust_varargs_shim.c — syssetup-rust variadic entry trampolines.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` is only legal inside one —
 * c_variadic, rust-lang/rust#44930). syssetup has two, so their ENTRY points live here: each
 * does the `va_start` Rust can't, then tail-calls the fixed-arity `*_v` twin in syssetup-rust,
 * which holds the real translated logic. This is the sanctioned varargs-rust trampoline pattern
 * (AGENTS.md "c_variadic", sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template), NOT a
 * translation reverted to C.
 *
 * WHY WE PASS THE va_list VALUE (not &ap): each `_v` twin forwards the va_list straight on to
 * vsprintf / vsnwprintf, whose last parameter IS a va_list. On all three targets
 * (i686/x86_64/aarch64 -pc-windows-gnu) `va_list` is `char*` — a single pointer — so handing it
 * through a `void*` is exact.
 *
 * pSetupDebugPrint is `@ varargs` in syssetup.spec, so this file owns a real DLL export;
 * FatalError is file-local to install.c and is only reached from the translated bodies there.
 */

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>

extern void pSetupDebugPrint_v(PCWSTR pszFileName,
                               INT nLineNumber,
                               PCWSTR pszTag,
                               PCWSTR pszMessage,
                               void *ap);

void CDECL pSetupDebugPrint(PCWSTR pszFileName,
                            INT nLineNumber,
                            PCWSTR pszTag,
                            PCWSTR pszMessage,
                            ...)
{
    va_list ap;

    va_start(ap, pszMessage);
    pSetupDebugPrint_v(pszFileName, nLineNumber, pszTag, pszMessage, (void *)ap);
    va_end(ap);
}

extern void FatalError_v(char *pszFmt, void *ap);

void FatalError(char *pszFmt, ...)
{
    va_list ap;

    va_start(ap, pszFmt);
    FatalError_v(pszFmt, (void *)ap);
    va_end(ap);
}

/* strsafe.h's StringCchPrintfW. The other bounded string ops the module uses (StringCchCopyW,
 * StringCchCatW, StringCbCopyW) are ported into syssetup-rust (src/syssetup/strsafe.rs), because
 * nothing in the tree defines a `static __inline` -- but this one is variadic, so its ENTRY has
 * to be here for the same c_variadic reason as the two above. The body is in
 * StringCchPrintfW_v; this is only the va_start.
 *
 * Deliberately NOT #include <strsafe.h>: the header's own `static __inline` definition has
 * internal linkage and would be a redefinition of the external one this file provides. */
#include <windows.h>

extern HRESULT StringCchPrintfW_v(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, void *ap);

HRESULT WINAPI StringCchPrintfW(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, ...)
{
    HRESULT hr;
    va_list ap;

    va_start(ap, pszFormat);
    hr = StringCchPrintfW_v(pszDest, cchDest, pszFormat, (void *)ap);
    va_end(ap);
    return hr;
}
