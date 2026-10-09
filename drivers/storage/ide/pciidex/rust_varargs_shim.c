/* rust_varargs_shim.c — the one thing stable Rust cannot express for pciidex.sys.
 *
 * DEFINING a C-variadic function needs `va_start`, which is legal only inside one: that is
 * the `c_variadic` feature (rust-lang#44930), so it cannot be written in Rust on stable.
 * pciidex exports exactly one such function, PciIdeXDebugPrint (pciidex.spec:
 * `@ varargs PciIdeXDebugPrint(long str)`), so it is a trampoline here with the exact
 * symbol and cdecl variadic ABI the spec/importlib expect. It hands Rust a POINTER to its
 * va_list; the level filter (the only policy) stays in pciidex-rust's PciIdeXDebugPrint_v
 * (sdk/lib/rust/pciidex-rust/hand/PciIdeXDebugPrint.rs). This is a shim for a language
 * gap, NOT a Rust translation reverted to C (AGENTS.md).
 *
 * vDbgPrintEx takes its va_list BY VALUE, which Rust holding an opaque `void*` cannot pass,
 * so vibeos_vDbgPrintEx dereferences the pointer for it and does nothing else.
 *
 * The ntstrsafe printf trampolines the pre-rewrite pciidex needed are gone: the rewrite's
 * Rust uses strsafe-rust's RtlStringCbPrintfW!/RtlStringCchPrintfExW! macros, which format
 * with the arguments in place.
 */

#include <stdarg.h>

/* NTAPI/stdcall decorates only on i386; it is ignored (and warned about) elsewhere. */
#if defined(__i386__)
#define STDCALL_ __stdcall
#else
#define STDCALL_
#endif

extern unsigned long STDCALL_ vDbgPrintEx(unsigned long ComponentId, unsigned long Level,
                                          const char *Format, va_list arglist);

unsigned long vibeos_vDbgPrintEx(unsigned long ComponentId, unsigned long Level,
                                 const char *Format, void *app)
{
    return vDbgPrintEx(ComponentId, Level, Format, *(va_list *)app);
}

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
