/*
 * rust_variadic_shim.c — C trampolines for classpnp's exported/internal C-variadic
 * functions, forwarding to the Rust workers in classpnp-rust.
 *
 * Stable Rust cannot define a `extern "C" fn(...)` with C varargs (the c_variadic
 * feature is nightly-only), and `va_start` is a compiler macro that must live in the
 * function that actually receives the varargs. So the public variadic entry points
 * stay in C: each does va_start and forwards the resulting va_list (a plain pointer on
 * all three Windows targets — i386/amd64/arm64) to its non-variadic Rust worker, which
 * holds the real logic. This is the project's sanctioned C-variadic shim pattern
 * (cf. sdk/lib/rtl/dbgprint_variadic.c).
 *
 * The two va_arg_* helpers exist because va_arg, like va_start, is a macro: the Rust
 * workers call back into C to pull each typed argument out of the va_list.
 */

#include <ntddk.h>

/* ---- Rust workers (defined in classpnp-rust, #[no_mangle]) ---------------- */
NTSTATUS pInitializeStorageRequestBlock(PVOID Srb, USHORT AddressType, ULONG ByteSize,
                                        ULONG NumSrbExData, va_list ap);
NTSTATUS classpnp_CreateStorageRequestBlockWorker(PVOID Srb, USHORT AddressType,
                                                  PVOID AllocateRoutine, PVOID ByteSize,
                                                  ULONG NumSrbExData, va_list ap);
VOID classpnp_ClassDebugPrintWorker(ULONG DebugPrintLevel, PCCHAR DebugMessage, va_list ap);

/* ---- va_arg helpers the Rust workers call (va_arg is a macro) ------------- */
ULONG classpnp_va_arg_ulong(va_list *ap)              { return va_arg(*ap, ULONG); }
ULONG classpnp_va_arg_srbexdatatype(va_list *ap)      { return va_arg(*ap, ULONG); }

/* ---- public variadic entry points (the real exported symbols) ------------ */

NTSTATUS
InitializeStorageRequestBlock(PSTORAGE_REQUEST_BLOCK Srb, USHORT AddressType,
                              ULONG ByteSize, ULONG NumSrbExData, ...)
{
    va_list ap;
    NTSTATUS status;
    va_start(ap, NumSrbExData);
    status = pInitializeStorageRequestBlock((PVOID)Srb, AddressType, ByteSize, NumSrbExData, ap);
    va_end(ap);
    return status;
}

NTSTATUS
CreateStorageRequestBlock(PSTORAGE_REQUEST_BLOCK *Srb, USHORT AddressType,
                          PVOID AllocateRoutine, PULONG ByteSize, ULONG NumSrbExData, ...)
{
    va_list ap;
    NTSTATUS status;
    va_start(ap, NumSrbExData);
    status = classpnp_CreateStorageRequestBlockWorker((PVOID)Srb, AddressType,
                                                      AllocateRoutine, (PVOID)ByteSize,
                                                      NumSrbExData, ap);
    va_end(ap);
    return status;
}

VOID
ClassDebugPrint(ULONG DebugPrintLevel, PCCHAR DebugMessage, ...)
{
    va_list ap;
    va_start(ap, DebugMessage);
    classpnp_ClassDebugPrintWorker(DebugPrintLevel, DebugMessage, ap);
    va_end(ap);
}
