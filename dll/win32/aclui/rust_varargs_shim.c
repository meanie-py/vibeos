/* rust_varargs_shim.c — aclui-rust variadic entry trampoline.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` can only appear inside one —
 * c_variadic, rust-lang/rust#44930). misc.c's LoadAndFormatString is variadic, so its ENTRY
 * point lives here: it does the `va_start` Rust can't, then tail-calls the fixed-arity
 * `LoadAndFormatString_v` twin in aclui-rust/src/aclui/misc.rs, which holds the real translated
 * logic (AllocAndLoadString + FormatMessageW + LocalFree). This is the sanctioned varargs-rust
 * trampoline pattern (see AGENTS.md "c_variadic" and
 * sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template), NOT a translation reverted to C.
 *
 * WHY WE PASS THE va_list VALUE (not &ap): the `_v` twin forwards the va_list straight to
 * FormatMessageW's `Arguments` parameter, which is a `va_list *`. On all three targets
 * (x86_64/i686/aarch64 -pc-windows-gnu) `va_list` is `char*` — a single pointer — so passing it
 * through a `void*` and taking its address on the Rust side is exact.
 */

#include <stdarg.h>
#include <windef.h>
#include <winbase.h>

extern DWORD LoadAndFormatString_v(HINSTANCE hInstance,
                                   UINT uID,
                                   LPWSTR *lpTarget,
                                   void *ap);

DWORD LoadAndFormatString(HINSTANCE hInstance, UINT uID, LPWSTR *lpTarget, ...)
{
    va_list ap;
    DWORD r;

    va_start(ap, lpTarget);
    r = LoadAndFormatString_v(hInstance, uID, lpTarget, (void *)ap);
    va_end(ap);
    return r;
}
