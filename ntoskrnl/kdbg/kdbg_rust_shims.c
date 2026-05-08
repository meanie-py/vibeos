/*
 * kdbg_rust_shims.c — surviving C-side helpers for the Rust kdbg subsystem.
 *
 * Most of what used to live here has been promoted to Rust:
 *   - Globals (KdbCliCallbacks, KdbUseIntelSyntax, dmesg state, etc.) →
 *     `ntoskrnl-rust::kdbg::tables`.
 *   - Macro / FORCEINLINE wrappers (`Ke386Get{Global,Local}DescriptorTable`,
 *     `Ke386GetTr`, `KeGet/SetContextFrameRegister`, `KeGetTrapFrame*Register`)
 *     → `ke-macros` rlib.
 *   - Tables and enum-to-string helpers → `ntoskrnl-rust::kdbg::tables`.
 *
 * What's left in this file:
 *   - `KdbpPrint` C-variadic — Rust can't define C-variadic exports on
 *     stable. Format parsing and emission live in the Rust ntprintf crate
 *     via `ntprintf_va_format_a`; this is just the va_list adapter.
 *   - `KdbPromptStr` — a `CSTRING` initialised via `RTL_CONSTANT_STRING`.
 *     Trivially translatable to Rust but the const-init pattern is
 *     low-priority; kept here pending the same minimal-headers reduction
 *     that `mm_rust_shims.c` got on i386/amd64.
 */

#include <ntoskrnl.h>
#include "kdb.h"
#include "../kd/kdterminal.h"

/* Prompt string referenced from kdb_cli.rs. */
const CSTRING KdbPromptStr = RTL_CONSTANT_STRING("kdb:> ");

/* -------------------------------------------------------------------------
 * KdbpPrint — C-variadic. Stable Rust can't define C-variadic exports, so
 * the va_arg pull stays here; format parsing and byte emission live in the
 * Rust ntprintf crate via ntprintf_va_format_a.
 * ------------------------------------------------------------------------- */

VOID KdbpPagerInternal(IN PCHAR Buffer, IN ULONG BufLength, IN BOOLEAN DoPage);

extern size_t ntprintf_va_format_a(char *dst, size_t cap,
                                   const char *fmt, va_list ap);

extern BOOLEAN KdbOutputAborted;  /* defined in ntoskrnl-rust::kdbg::tables */

VOID
KdbpPrint(
    _In_ PSTR Format,
    _In_ ...)
{
    static CHAR Buffer[4096];
    SIZE_T Length;
    va_list ap;

    if (KdbOutputAborted)
        return;

    va_start(ap, Format);
    Length = ntprintf_va_format_a(Buffer, sizeof(Buffer), Format, ap);
    va_end(ap);

    if (Length >= sizeof(Buffer))
        Length = sizeof(Buffer) - 1;

    KdbpPagerInternal(Buffer, (ULONG)Length, FALSE);
}
