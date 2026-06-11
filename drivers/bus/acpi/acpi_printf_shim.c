/*
 * acpi_printf_shim.c — C-variadic trampoline for the Rust OSL (acpi-rust).
 *
 * osl.c was translated to Rust (sdk/lib/rust/acpi-rust/, ACPICA FFI-to-blob
 * Phase 2). All 45 AcpiOs* callbacks now come from libacpi_rust.a — except
 * AcpiOsPrintf, which is C-variadic and so cannot be a Rust #[no_mangle] fn.
 * This thin shim forwards to the Rust AcpiOsVprintf (a no-op under the driver's
 * NDEBUG build, exactly like the original). osl.c is kept on disk per the
 * never-delete rule.
 */

#include "precomp.h"
#include <stdarg.h>

void ACPI_INTERNAL_VAR_XFACE
AcpiOsPrintf(const char *Fmt, ...)
{
    va_list Args;
    va_start(Args, Fmt);
    AcpiOsVprintf(Fmt, Args);
    va_end(Args);
}
