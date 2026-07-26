/* rust_varargs_shim.c — va_start/va_arg trampolines for wldap32's two cdecl-varargs
 * exports, ber_printf and ber_scanf.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` is only legal inside one)
 * nor do `va_arg` — both are `c_variadic`, rust-lang/rust#44930. This file supplies those
 * two primitives and nothing else: each entry point is a 3-line trampoline that hands
 * straight back to Rust, where the whole ber_printf/ber_scanf format walk lives. It is a
 * shim for a language gap, NOT a translation reverted to C.
 *
 * Template + rationale: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template.
 * Rust side: sdk/lib/rust/wldap32-rust/src/wldap32/ber.rs (`*_v`).
 *
 * We pass `&ap`, never `ap`: that avoids assuming `va_list` is itself a single pointer.
 */

#include <stdarg.h>

/* ---- PART 1: the va_arg pullers (identical in every module) ---------------------- */
int                vibeos_va_arg_i32(void *app) { return va_arg(*(va_list *)app, int); }
unsigned int       vibeos_va_arg_u32(void *app) { return va_arg(*(va_list *)app, unsigned int); }
long long          vibeos_va_arg_i64(void *app) { return va_arg(*(va_list *)app, long long); }
unsigned long long vibeos_va_arg_u64(void *app) { return va_arg(*(va_list *)app, unsigned long long); }
double             vibeos_va_arg_f64(void *app) { return va_arg(*(va_list *)app, double); }
void *             vibeos_va_arg_ptr(void *app) { return va_arg(*(va_list *)app, void *); }

/* ---- PART 2: wldap32's variadic entry points ------------------------------------- */

extern int WLDAP32_ber_printf_v(void *berelement, char *fmt, void *ap);
extern int WLDAP32_ber_scanf_v(void *berelement, char *fmt, void *ap);

int WLDAP32_ber_printf(void *berelement, char *fmt, ...)
{
    va_list ap;
    int r;
    va_start(ap, fmt);
    r = WLDAP32_ber_printf_v(berelement, fmt, (void *)&ap);
    va_end(ap);
    return r;
}

int WLDAP32_ber_scanf(void *berelement, char *fmt, ...)
{
    va_list ap;
    int r;
    va_start(ap, fmt);
    r = WLDAP32_ber_scanf_v(berelement, fmt, (void *)&ap);
    va_end(ap);
    return r;
}
