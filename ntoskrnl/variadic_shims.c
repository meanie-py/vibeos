/*
 * variadic_shims.c — minimal trampoline for the surviving C-variadic kernel
 * exports (KdpDprintf, RtlStringCbPrintfA).
 *
 * Stable Rust can't define C-variadic exports, so the va_arg pull stays in
 * C. Format-string parsing and byte emission live in the Rust ntprintf
 * crate via `ntprintf_va_format_a`; this is just the va_list adapter.
 *
 * Headers limited to <ntdef.h> for basic types + NTAPI; <stdarg.h> for
 * va_list. NTSTATUS / SIZE_T / STRING come from <ntdef.h>. The two STATUS_*
 * constants and MAXUSHORT are hand-defined to avoid pulling <ntoskrnl.h>.
 *
 * Audit-target: 0/0/0/0 + a couple of forward decls. cc-rs in a future
 * cargo-driven build can compile this standalone.
 */

#include <ntdef.h>
#include <stdarg.h>

/* Hand-defined to avoid pulling <ntoskrnl.h> just for these. */
#define STATUS_SUCCESS                  ((NTSTATUS)0x00000000)
#define STATUS_BUFFER_OVERFLOW          ((NTSTATUS)0x80000005)
#define STATUS_INVALID_PARAMETER        ((NTSTATUS)0xC000000D)
#define MAXUSHORT                       0xffff
#define NTSTRSAFE_MAX_CCH               2147483647

/* Rust ntprintf core — packs args from va_list into a 4 KiB buffer. */
extern size_t ntprintf_va_format_a(char *dst, size_t cap,
                                   const char *fmt, va_list ap);

/* Provided by the C kd64 subsystem (kdprint.c on KD64_USE_C, kdbg shims
 * otherwise) — emits a STRING through the kernel debugger transport. */
NTAPI VOID KdpPrintString(IN PSTRING String);

#ifndef KD64_USE_C
VOID __cdecl
KdpDprintf(_In_ PCSTR Format, ...)
{
    STRING String;
    SIZE_T Length;
    va_list ap;
    CHAR Buffer[512];

    va_start(ap, Format);
    Length = ntprintf_va_format_a(Buffer, sizeof(Buffer), Format, ap);
    va_end(ap);

    if (Length > MAXUSHORT)
        Length = MAXUSHORT;

    String.Buffer = Buffer;
    String.Length = String.MaximumLength = (USHORT)Length;
    KdpPrintString(&String);
}
#endif

NTSTATUS __cdecl
RtlStringCbPrintfA(char *pszDest, size_t cbDest, const char *pszFormat, ...)
{
    va_list argList;
    SIZE_T written;

    if (cbDest > NTSTRSAFE_MAX_CCH)
    {
        if (cbDest > 0)
            *pszDest = '\0';
        return STATUS_INVALID_PARAMETER;
    }
    if (cbDest == 0)
        return STATUS_SUCCESS;

    va_start(argList, pszFormat);
    written = ntprintf_va_format_a(pszDest, cbDest, pszFormat, argList);
    va_end(argList);

    return (written + 1 > cbDest) ? STATUS_BUFFER_OVERFLOW : STATUS_SUCCESS;
}
