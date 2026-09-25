/* INERT since 2026-09-25: no longer in the build. Its varargs moved to Rust (strsafe-rust's
 * StringC*PrintfW! macros call msvcrt's variadic _snwprintf; the module's own variadics RECEIVE
 * their arguments the va-slots way). Kept on disk as the reference the Rust was checked against. */
/*
 * netcfgx — out-of-line home for the one strsafe entry point the Rust translation calls.
 *
 * This is NOT a translation left in C. `StringCchPrintfW` is `STRSAFEAPI` (a `static inline`)
 * in <strsafe.h>, so every C TU that used it inlined its own copy and no object file in the
 * tree carries the symbol — there is nothing for the Rust `extern` to bind to. Stable Rust
 * cannot supply one either: DEFINING a C-variadic function, and `va_start` itself, are
 * nightly-only (rust-lang/rust#44930). So the variadic frame is built here and handed
 * straight to strsafe's own va_list twin; every line of logic stays in netcfgx-rust.
 *
 * Same shape as sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template, and the same reason
 * wldap32 keeps rust_varargs_shim.c for ber_printf/ber_scanf.
 *
 * The name is prefixed because <strsafe.h> is included here too, and its inline definition of
 * `StringCchPrintfW` would collide with a same-named out-of-line one. netcfgx-rust binds to
 * this symbol via `#[link_name]`, so the call sites keep the C spelling.
 */

#define WIN32_NO_STATUS
#include <windef.h>
#include <winbase.h>
#include <stdarg.h>
#include <strsafe.h>

HRESULT WINAPI
rust_StringCchPrintfW(
    STRSAFE_LPWSTR pszDest,
    size_t cchDest,
    STRSAFE_LPCWSTR pszFormat,
    ...)
{
    HRESULT hr;
    va_list argList;

    va_start(argList, pszFormat);
    hr = StringCchVPrintfW(pszDest, cchDest, pszFormat, argList);
    va_end(argList);

    return hr;
}
