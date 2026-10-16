/* cnt-rust scanf trampoline — the irreducible C half.
 *
 * Rust cannot DEFINE a C-variadic function on stable (the c_variadic
 * feature is nightly; this project's discipline is "declare, never
 * define" variadics). The genuine ReactOS scanf.c is itself a thin
 * `va_start; vsscanf_l(...); va_end` shell over the engine — this file
 * is exactly that shell. It owns va_start/va_end and hands the live
 * `va_list*` to the Rust engine half (cnt_v*scanf in cabi/manual.rs).
 *
 * `cnt_va_next_ptr` performs one `va_arg(*ap, void*)`: every scanf
 * assignment target is a pointer, so the variadic ABI slot is uniform
 * regardless of the pointee type — the Rust side does the typed store.
 * The engine invokes it at the exact per-directive point the genuine
 * scanf.h calls va_arg, keeping the va_list in lockstep.
 *
 * Phase 1 provides sscanf/swscanf; Phase 3 adds _snscanf/_snwscanf.
 * Kernel cmake wiring of this .c is the (deferred, like every other
 * cnt-rust symbol) cohabitation step; today cnt-difftest's build.rs
 * compiles it for the host differential.
 */
#include <stdarg.h>
#include <stddef.h> /* size_t */

extern int cnt_vsscanf(const char *file, const char *fmt, va_list *ap);
extern int cnt_vsscanf_s(const char *file, const char *fmt, va_list *ap);
extern int cnt_vswscanf(const unsigned short *file,
                        const unsigned short *fmt, va_list *ap);
extern int cnt_vsnscanf(const char *file, size_t length,
                        const char *fmt, va_list *ap);
extern int cnt_vsnwscanf(const unsigned short *file, size_t length,
                         const unsigned short *fmt, va_list *ap);

void *cnt_va_next_ptr(va_list *ap)
{
    return va_arg(*ap, void *);
}

int sscanf(const char *str, const char *format, ...)
{
    va_list ap;
    int r;
    va_start(ap, format);
    r = cnt_vsscanf(str, format, &ap);
    va_end(ap);
    return r;
}

int sscanf_s(const char *str, const char *format, ...)
{
    va_list ap;
    int r;
    va_start(ap, format);
    r = cnt_vsscanf_s(str, format, &ap);
    va_end(ap);
    return r;
}

int swscanf(const unsigned short *str, const unsigned short *format, ...)
{
    va_list ap;
    int r;
    va_start(ap, format);
    r = cnt_vswscanf(str, format, &ap);
    va_end(ap);
    return r;
}

int _snscanf(const char *input, size_t length, const char *format, ...)
{
    va_list ap;
    int r;
    va_start(ap, format);
    r = cnt_vsnscanf(input, length, format, &ap);
    va_end(ap);
    return r;
}

int _snwscanf(const unsigned short *input, size_t length,
              const unsigned short *format, ...)
{
    va_list ap;
    int r;
    va_start(ap, format);
    r = cnt_vsnwscanf(input, length, format, &ap);
    va_end(ap);
    return r;
}
