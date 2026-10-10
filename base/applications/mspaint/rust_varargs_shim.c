/* rust_varargs_shim.c -- the one thing stable Rust cannot do for mspaint: DEFINE a
 * C-variadic function (`va_start` can only appear inside one). This trampoline does the
 * va_start and hands straight back to Rust; all of ShowError's logic is in
 * mspaint-rust's `ShowError_v`. Delete when `c_variadic` stabilizes (rust-lang/rust#44930).
 * Pattern: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template. A POINTER to the
 * va_list is passed, never the va_list itself. */

#include <stdarg.h>

typedef int INT;

extern void ShowError_v(INT stringID, void *ap); /* Rust */

void ShowError(INT stringID, ...)
{
    va_list ap;
    va_start(ap, stringID);
    ShowError_v(stringID, (void *)&ap);
    va_end(ap);
}
