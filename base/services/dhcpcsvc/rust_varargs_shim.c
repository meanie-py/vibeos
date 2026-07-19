/* rust_varargs_shim.c — dhcpcsvc-rust variadic entry trampolines.
 *
 * Stable Rust cannot DEFINE a C-variadic function (`va_start` can only appear inside one —
 * c_variadic, rust-lang/rust#44930). The ISC dhclient log helpers `note`/`warn`/`warning`/
 * `debug`/`error` (base/services/dhcpcsvc/dhcp/util.c) are variadic, so their ENTRY points live
 * here: each does the `va_start` Rust can't, then tail-calls the fixed-arity `*_v` twin in
 * dhcpcsvc-rust/src/dhcp/util.rs, which holds the real translated logic (the `_vsnprintf` format +
 * the DPRINT). This is the sanctioned varargs-rust trampoline pattern (see AGENTS.md "c_variadic"
 * and sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template), NOT a translation reverted to C.
 *
 * INTEGRATION: when the util.c cohab flip lands, copy this file to
 *   base/services/dhcpcsvc/rust_varargs_shim.c
 * and add it to that CMakeLists.txt SOURCE list (the driver-crate pattern, e.g.
 * drivers/usb/usbport/CMakeLists.txt). The dhcpcsvc-rust staticlib itself builds WITHOUT this file
 * (the entry symbols are just undefined externals resolved at DLL-link time).
 *
 * WHY WE PASS THE va_list VALUE (not &ap): the `*_v` twins forward the va_list straight to
 * `_vsnprintf`, which takes a `va_list`. On ALL THREE targets (x86_64/i686/aarch64 -pc-windows-gnu)
 * `va_list` is `char*` — a single pointer — so passing it through a `void*` is exact. (This crate is
 * windows-only; the by-&ap form in the template is for ABIs where va_list is an array, e.g. SysV.)
 */

#include <stdarg.h>

extern int  note_v   (const char *format, void *ap);
extern int  warn_v   (const char *format, void *ap);
extern int  warning_v(const char *format, void *ap);
extern int  debug_v  (const char *format, void *ap);
extern void error_v  (const char *format, void *ap);

int note(const char *format, ...) {
    va_list ap; va_start(ap, format);
    int r = note_v(format, (void *)ap);
    va_end(ap);
    return r;
}

int warn(const char *format, ...) {
    va_list ap; va_start(ap, format);
    int r = warn_v(format, (void *)ap);
    va_end(ap);
    return r;
}

int warning(const char *format, ...) {
    va_list ap; va_start(ap, format);
    int r = warning_v(format, (void *)ap);
    va_end(ap);
    return r;
}

int debug(const char *format, ...) {
    va_list ap; va_start(ap, format);
    int r = debug_v(format, (void *)ap);
    va_end(ap);
    return r;
}

void error(const char *format, ...) {
    va_list ap; va_start(ap, format);
    error_v(format, (void *)ap);
    va_end(ap);
}
