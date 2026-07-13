/* rust_varargs_shim.c — the ONLY two things stable Rust cannot express.
 *
 *   1. DEFINING a C-variadic function (`va_start` is legal only inside one), and
 *   2. va_arg.
 *
 * Both are the `c_variadic` feature (rust-lang/rust#44930 — FCP-complete with a merge
 * disposition, so this file is expected to be DELETED once it stabilizes; see
 * docs/engineering/active-lanes/stable-toolchain-plan.md). *Calling* and *declaring*
 * a variadic C fn are already stable, so pciidex-rust's call sites are untouched.
 *
 * Every entry below is a 3-line trampoline straight back into Rust: the ntstrsafe
 * wrappers' length/NTSTATUS semantics and PciIdeXDebugPrint's level filter stay in
 * pciidex-rust (src/inlines.rs, src/miniport.rs). This is a shim for a language gap,
 * NOT a Rust translation reverted to C (AGENTS.md "Rust Is The Goal").
 *
 * We hand Rust a POINTER to the va_list (`&ap`), never the va_list itself — that
 * avoids assuming `va_list` is a single pointer (true on our three Windows targets,
 * false on SysV x86-64) and is what lets varargs-rust be unit-tested on the host.
 *
 * PART 1b is pciidex-specific: all three of its variadic functions FORWARD their
 * va_list to a C vararg consumer (`_vsnwprintf` / `vDbgPrintEx`) rather than pulling
 * args themselves. Rust holds only an opaque `void*`, so it cannot pass a `va_list`
 * BY VALUE; these one-line forwarders dereference the pointer and make the real call.
 * They do nothing else — all policy (buffer sizing, NUL-termination, NTSTATUS mapping,
 * the DebugPrintLevel filter) stays in Rust.
 *
 * Rust side: sdk/lib/rust/varargs-rust (`VaArgs`) + the `*_v` twins in pciidex-rust.
 * Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template
 */

#include <stdarg.h>
#include <stddef.h>

typedef long NTSTATUS_C;   /* NTSTATUS is a 32-bit signed value on all three arches */
typedef unsigned short WCHAR_C;

/* NTAPI/stdcall decorates only on i386; it is ignored (and warned about) elsewhere. */
#if defined(__i386__)
#define STDCALL_ __stdcall
#else
#define STDCALL_
#endif

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

/* ---- PART 1b: va_list forwarders to the C vararg consumers ---------------------
 * `_vsnwprintf` (cdecl) and `vDbgPrintEx` (stdcall) are real ntoskrnl exports that take
 * a `va_list` BY VALUE. Declared here rather than via <stdio.h>: mingw's stdio.h maps
 * them onto __mingw_* inline wrappers that do not exist in ntoskrnl.
 */
extern int _vsnwprintf(WCHAR_C *buffer, size_t count, const WCHAR_C *format, va_list argptr);

int vibeos_vsnwprintf(WCHAR_C *buffer, size_t count, const WCHAR_C *format, void *app)
{
    return _vsnwprintf(buffer, count, format, *(va_list *)app);
}

extern unsigned long STDCALL_ vDbgPrintEx(unsigned long ComponentId, unsigned long Level,
                                          const char *Format, va_list arglist);

unsigned long vibeos_vDbgPrintEx(unsigned long ComponentId, unsigned long Level,
                                 const char *Format, void *app)
{
    return vDbgPrintEx(ComponentId, Level, Format, *(va_list *)app);
}

/* ---- PART 2: pciidex's variadic entry points ----------------------------------- */

/* ntstrsafe.h FORCEINLINE (pciidex-rust/src/inlines.rs). Rust-internal callers only. */
extern NTSTATUS_C RtlStringCbPrintfW_v(
    WCHAR_C *dest, size_t cbDest, const WCHAR_C *format, void *ap);

NTSTATUS_C RtlStringCbPrintfW(
    WCHAR_C *dest, size_t cbDest, const WCHAR_C *format, ...)
{
    va_list ap;
    NTSTATUS_C status;
    va_start(ap, format);
    status = RtlStringCbPrintfW_v(dest, cbDest, format, (void *)&ap);
    va_end(ap);
    return status;
}

extern NTSTATUS_C RtlStringCchPrintfExW_v(
    WCHAR_C *dest, size_t cchDest, WCHAR_C **ppszDestEnd, size_t *pcchRemaining,
    unsigned long flags, const WCHAR_C *format, void *ap);

NTSTATUS_C RtlStringCchPrintfExW(
    WCHAR_C *dest, size_t cchDest, WCHAR_C **ppszDestEnd, size_t *pcchRemaining,
    unsigned long flags, const WCHAR_C *format, ...)
{
    va_list ap;
    NTSTATUS_C status;
    va_start(ap, format);
    status = RtlStringCchPrintfExW_v(dest, cchDest, ppszDestEnd, pcchRemaining,
                                     flags, format, (void *)&ap);
    va_end(ap);
    return status;
}

/* PciIdeXDebugPrint is an EXPORT (pciidex.spec: `@ varargs PciIdeXDebugPrint(long str)`)
 * — uniata.sys / pciide.sys import it. This trampoline therefore keeps the exact symbol
 * name and the variadic cdecl ABI the spec/importlib expects; the level filter and the
 * vDbgPrintEx dispatch stay in pciidex-rust/src/miniport.rs.
 */
extern void PciIdeXDebugPrint_v(
    unsigned long DebugPrintLevel, const char *DebugMessage, void *ap);

void PciIdeXDebugPrint(
    unsigned long DebugPrintLevel, const char *DebugMessage, ...)
{
    va_list ap;
    va_start(ap, DebugMessage);
    PciIdeXDebugPrint_v(DebugPrintLevel, DebugMessage, (void *)&ap);
    va_end(ap);
}
