/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so usbstor-rust's call sites (pdo.rs's extern
 * decl of RtlStringCchPrintfW) are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: the ntstrsafe
 * formatter and its length/NTSTATUS semantics stay in usbstor-rust/src/glue.rs. This
 * is a shim for a language gap, NOT a Rust translation reverted to C (AGENTS.md
 * "Rust Is The Goal").
 *
 * We hand Rust a POINTER to the va_list (`&ap`), never the va_list itself — that
 * avoids assuming `va_list` is a single pointer (true on our three Windows targets,
 * false on SysV x86-64) and is what lets varargs-rust be unit-tested on the host.
 *
 * Rust side: sdk/lib/rust/varargs-rust (`VaArgs`) + the `*_v` twins in glue.rs.
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template
 */

#include <stdarg.h>
#include <stddef.h>

typedef long NTSTATUS_C;   /* NTSTATUS is a 32-bit signed value on all three arches */
typedef unsigned short WCHAR_C;

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

/* ---- PART 2: usbstor's variadic entry points ----------------------------------- */

extern NTSTATUS_C RtlStringCbPrintfW_v(
    WCHAR_C *pszDest, size_t cbDest, const WCHAR_C *pszFormat, void *ap);

NTSTATUS_C RtlStringCbPrintfW(
    WCHAR_C *pszDest, size_t cbDest, const WCHAR_C *pszFormat, ...)
{
    va_list ap;
    NTSTATUS_C status;
    va_start(ap, pszFormat);
    status = RtlStringCbPrintfW_v(pszDest, cbDest, pszFormat, (void *)&ap);
    va_end(ap);
    return status;
}

/* RtlStringCchPrintfW — i386 resolves this from the import lib (ntoskrnl/usbd export it
 * there), so usbstor-rust's Rust twin is `#[cfg(not(target_arch = "x86"))]`. Mirror that
 * cfg here EXACTLY: on i386 there is no `_v` twin to call and defining the entry point
 * would duplicate the imported one. */
#if !defined(_M_IX86) && !defined(__i386__)

extern NTSTATUS_C RtlStringCchPrintfW_v(
    WCHAR_C *Dst, size_t CchDst, const WCHAR_C *Format, void *ap);

NTSTATUS_C RtlStringCchPrintfW(
    WCHAR_C *Dst, size_t CchDst, const WCHAR_C *Format, ...)
{
    va_list ap;
    NTSTATUS_C status;
    va_start(ap, Format);
    status = RtlStringCchPrintfW_v(Dst, CchDst, Format, (void *)&ap);
    va_end(ap);
    return status;
}

#endif /* !i386 */
