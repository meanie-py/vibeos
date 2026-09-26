/* INERT since 2026-09-25: no longer in the build. Its varargs moved to Rust (strsafe-rust's
 * StringC*PrintfW! macros call msvcrt's variadic _snwprintf; the module's own variadics RECEIVE
 * their arguments the va-slots way). Kept on disk as the reference the Rust was checked against. */
/* rust_varargs_shim.c — sysdm-rust variadic entry trampolines.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` is only legal inside one —
 * c_variadic, rust-lang/rust#44930). Two of sysdm's entry points are variadic, so each one
 * lives here: it does the `va_start` Rust can't, then tail-calls the fixed-arity `*_v` twin
 * that holds the real ported body. This is the sanctioned varargs-rust trampoline pattern
 * (AGENTS.md "c_variadic", sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template), NOT a
 * translation reverted to C.
 *
 * ResourceMessageBox is sysdm's OWN function (sysdm.c:27) and the body is
 * ResourceMessageBox_v in sysdm-rust. StringCchPrintfW is strsafe.h's, `static __inline`, so
 * no object file in the tree defines it and flipping sysdm's .c takes the last inlined copy
 * with it; its body is in the shared strsafe-rust crate, same as desk's.
 *
 * WHY WE PASS `&ap` AND NOT `ap`: handing Rust a POINTER to the va_list avoids assuming a
 * va_list is itself a single pointer. It happens to be `char*` on all three of our Windows
 * targets, but it is an array-of-struct on SysV x86-64, and the by-pointer form is correct on
 * both. This is varargs-rust's documented convention, which strsafe-rust also follows.
 *
 * CDECL, not WINAPI: no ABI on any of our targets has a variadic stdcall, so a variadic
 * function IS cdecl however the header spells it. sysdm.c says `INT __cdecl` outright. The
 * Rust side declares both `extern "C"` for the same reason.
 */

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>
#include <windows.h>

extern INT ResourceMessageBox_v(HINSTANCE hInstance,
                                HWND hwnd,
                                UINT uType,
                                UINT uCaption,
                                UINT uText,
                                void *app);

INT __cdecl ResourceMessageBox(HINSTANCE hInstance,
                               HWND hwnd,
                               UINT uType,
                               UINT uCaption,
                               UINT uText,
                               ...)
{
    INT ret;
    va_list ap;

    va_start(ap, uText);
    ret = ResourceMessageBox_v(hInstance, hwnd, uType, uCaption, uText, (void *)&ap);
    va_end(ap);
    return ret;
}

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
