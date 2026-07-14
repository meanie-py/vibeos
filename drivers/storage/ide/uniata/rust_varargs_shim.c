/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so uniata-rust's call sites are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: the debug-console
 * buffer sizing and the HalDisplayString call stay in uniata-rust/src/id_ata_part3.rs.
 * This is a shim for a language gap, NOT a Rust translation reverted to C (AGENTS.md
 * "Rust Is The Goal").
 *
 * We hand Rust a POINTER to the va_list (`&ap`), never the va_list itself — that
 * avoids assuming `va_list` is a single pointer (true on our three Windows targets,
 * false on SysV x86-64) and is what lets varargs-rust be unit-tested on the host.
 *
 * Rust side: sdk/lib/rust/varargs-rust (`VaArgs`) + the `*_v` twins in id_ata_part3.rs.
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

/* vibeos_vsnprintf — the one v-family forwarder uniata needs. Rust holds a POINTER to
 * the va_list (see above) but ntoskrnl's `_vsnprintf` wants the `va_list` ITSELF, and
 * stable Rust has no way to name that type. Deref here; the caller keeps ownership of
 * the buffer, its size, and the NUL termination (all still in Rust).
 */
extern int _vsnprintf(char *dst, size_t n, const char *fmt, va_list ap);

int vibeos_vsnprintf(char *dst, size_t n, const char *fmt, void *app)
{
    return _vsnprintf(dst, n, fmt, *(va_list *)app);
}

/* ---- PART 2: uniata's variadic entry points -----------------------------------
 * _PrintNtConsole — the cdecl varargs debug-console printer (id_ata.cpp). The Rust `_v`
 * twin owns the 512-byte buffer, the NUL termination and the HalDisplayString call; it
 * formats via vibeos_vsnprintf above.
 */

extern void _PrintNtConsole_v(const char *DebugMessage, void *ap);

void _PrintNtConsole(const char *DebugMessage, ...)
{
    va_list ap;
    va_start(ap, DebugMessage);
    _PrintNtConsole_v(DebugMessage, (void *)&ap);
    va_end(ap);
}
