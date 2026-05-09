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
 *   - `KdbPromptStr` — a `CSTRING` initialised via the equivalent of
 *     `RTL_CONSTANT_STRING("kdb:> ")`.
 *
 * Header-free since the cargo-driven build PoC: no `<ntoskrnl.h>`, no
 * `<kdb.h>`, no `<../kd/kdterminal.h>`. Local typedefs + extern decls of
 * the few functions/globals we need cover everything.
 */

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

/* --- Local NT type aliases (replaces <ntoskrnl.h>) ------------------ */
typedef void              VOID;
typedef char              CHAR;
typedef unsigned char     UCHAR;
typedef UCHAR             BOOLEAN;
typedef CHAR             *PCHAR;
typedef CHAR             *PSTR;
typedef unsigned short    USHORT;
typedef unsigned long     ULONG;
typedef size_t            SIZE_T;

#define IN
#ifndef _In_
# define _In_
#endif
#define FALSE 0

/* Mirror of CSTRING from sdk/include/psdk/ntdef.h. */
typedef struct _CSTRING
{
    USHORT Length;
    USHORT MaximumLength;
    const CHAR *Buffer;
} CSTRING;

/* Open-coded RTL_CONSTANT_STRING("kdb:> "): the macro expands to
 * { sizeof(s) - sizeof(s[0]), sizeof(s), s }, i.e. (Length, MaximumLength,
 * Buffer) where the literal "kdb:> " is 7 bytes including the terminator. */
const CSTRING KdbPromptStr = { 6, 7, "kdb:> " };

/* -------------------------------------------------------------------------
 * KdbpPrint — C-variadic. Stable Rust can't define C-variadic exports, so
 * the va_arg pull stays here; format parsing and byte emission live in the
 * Rust ntprintf crate via ntprintf_va_format_a.
 * ------------------------------------------------------------------------- */

extern VOID KdbpPagerInternal(IN PCHAR Buffer, IN ULONG BufLength, IN BOOLEAN DoPage);

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
