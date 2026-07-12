/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so pcix-rust's call sites are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: id.c's ANSI_STRING
 * bookkeeping (RtlInitAnsiString, StringSize/TotalLength/Count, the CharBuffer bump, the
 * RtlAsserts) stays in pcix-rust/src/pci/id.rs. This is a shim for a language gap, NOT a
 * Rust translation reverted to C (AGENTS.md "Rust Is The Goal").
 *
 * We hand Rust a POINTER to the va_list (`&ap`), never the va_list itself — that
 * avoids assuming `va_list` is a single pointer (true on our three Windows targets,
 * false on SysV x86-64) and is what lets varargs-rust be unit-tested on the host.
 *
 * PART 1b is pcix-specific: both variadic functions FORWARD their va_list to C's
 * `vsprintf` rather than pulling args themselves. Rust holds only an opaque `void*`, so
 * it cannot pass a `va_list` BY VALUE; this one-line forwarder dereferences the pointer
 * and makes the real call. It does nothing else — the destination pointer, the length
 * accounting and every ANSI_STRING field update stay in Rust.
 *
 * Rust side: sdk/lib/rust/varargs-rust (`VaArgs`) + the `*_v` twins in pcix-rust.
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template
 */

#include <stdarg.h>
#include <stddef.h>

/* ---- PART 1: the va_arg pullers (mirrored by varargs-rust's extern block) ------
 * Each takes a POINTER to the caller's va_list and ADVANCES it, so successive pulls
 * from Rust walk the argument list exactly as C's va_arg would.
 */
int                vibeos_va_arg_i32(void *app) { return va_arg(*(va_list *)app, int); }
unsigned int       vibeos_va_arg_u32(void *app) { return va_arg(*(va_list *)app, unsigned int); }
long long          vibeos_va_arg_i64(void *app) { return va_arg(*(va_list *)app, long long); }
unsigned long long vibeos_va_arg_u64(void *app) { return va_arg(*(va_list *)app, unsigned long long); }
double             vibeos_va_arg_f64(void *app) { return va_arg(*(va_list *)app, double); }
void *             vibeos_va_arg_ptr(void *app) { return va_arg(*(va_list *)app, void *); }

/* ---- PART 1b: va_list forwarder to the C vararg consumer -----------------------
 * `vsprintf` is a real ntoskrnl export (cdecl) taking a `va_list` BY VALUE. Declared
 * here rather than via <stdio.h>: mingw's stdio.h maps it onto a __mingw_vsprintf
 * inline wrapper that does not exist in ntoskrnl.
 */
extern int vsprintf(char *buffer, const char *format, va_list ap);

int vibeos_vsprintf(char *buffer, const char *format, void *app)
{
    return vsprintf(buffer, format, *(va_list *)app);
}

/* ---- PART 2: pcix's variadic entry points -------------------------------------- */

/* pci/id.c PciIdPrintf — driver-internal (pcix has no .spec exports); all callers are
 * in pcix-rust/src/pci/id.rs, which declares this entry and calls it unchanged.
 */
extern unsigned long PciIdPrintf_v(void *IdBuffer, const char *Format, void *ap);

unsigned long PciIdPrintf(void *IdBuffer, const char *Format, ...)
{
    va_list ap;
    unsigned long r;
    va_start(ap, Format);
    r = PciIdPrintf_v(IdBuffer, Format, (void *)&ap);
    va_end(ap);
    return r;
}

extern unsigned long PciIdPrintfAppend_v(void *IdBuffer, const char *Format, void *ap);

unsigned long PciIdPrintfAppend(void *IdBuffer, const char *Format, ...)
{
    va_list ap;
    unsigned long r;
    va_start(ap, Format);
    r = PciIdPrintfAppend_v(IdBuffer, Format, (void *)&ap);
    va_end(ap);
    return r;
}
