/* INERT since 2026-09-24: not in CMake. Replaced by Rust: ShellMessageBoxA receives its
 * varargs via va-slots-rust (shell32-rust/src/wine/shellord.rs), and StringCchPrintfW is
 * strsafe_rust::StringCchPrintfW!. Kept on disk as the reference. */

/* rust_strsafe_shim.c — linkage for the strsafe.h INLINES that Rust calls.
 *
 * `StringCchPrintfW` is a PHANTOM SYMBOL: strsafe.h:515 declares it `STRSAFEAPI`, which is
 * `static __inline HRESULT WINAPI`, so it exists only inside each C translation unit that
 * includes the header and there is nothing for a Rust `extern` block to bind to. An `extern`
 * declaration for it compiles fine and fails at LINK (`undefined symbol: _StringCchPrintfW`),
 * which is how shell32-rust/src/shell32/shlexec.rs found it.
 *
 * This is NOT the same gap as rust_varargs_shim.c. That file exists because stable Rust cannot
 * DEFINE a C-variadic; here Rust only needs to CALL one, which is already stable — the missing
 * piece is purely a symbol to call. So this is an ordinary forwarding trampoline into the
 * header's own `StringCchVPrintfW`, which keeps strsafe's semantics exactly: bounded, always
 * NUL-terminating, `STRSAFE_E_INSUFFICIENT_BUFFER` on truncation. Do NOT reimplement it in Rust
 * on top of `swprintf` — that truncation contract is the whole reason callers use strsafe.
 *
 * Its own TU rather than an append to rust_varargs_shim.c: that file deliberately includes no
 * Windows headers (it hand-declares `ShellMessageBoxA` so it can spell the variadic itself), and
 * pulling in <windows.h> beside it makes shellapi.h's real prototype conflict with that one.
 *
 * Rust side: the `StringCchPrintfW` declaration in shell32-rust/src/prelude.rs, which binds to
 * this name via `#[link_name]`.
 */

#include <windows.h>
#include <strsafe.h>

HRESULT StringCchPrintfW_RustBridge(WCHAR *pszDest, size_t cchDest, const WCHAR *pszFormat, ...)
{
    HRESULT hr;
    va_list ap;
    va_start(ap, pszFormat);
    hr = StringCchVPrintfW(pszDest, cchDest, pszFormat, ap);
    va_end(ap);
    return hr;
}
