/*
 * kdbg_rust_shims.c — surviving C-side helpers for the Rust kdbg subsystem.
 *
 * Provides:
 *   - File-private statics from the original kdb_cli.c that Rust references
 *     via `extern "C" { static mut X: T; }` (KdbCliCallbacks, output mode
 *     flags, dmesg ring buffer state, KdbPromptStr).
 *   - Macro / FORCEINLINE wrappers that Rust can't expand directly:
 *     `Ke386GetGlobalDescriptorTable`, `Ke386GetLocalDescriptorTable`,
 *     `Ke386GetTr`, `KeGet*ContextFrameRegister`, `KeGet*TrapFrame*Register`.
 *   - The `KdbpPrint` C-variadic function (Rust can't define C-variadic
 *     exports on stable; the format-parsing core lives in the Rust ntprintf
 *     crate, this is just the va_list adapter).
 *
 * Translated to Rust 2026-05-07 (kdbg/tables.rs):
 *   - `ComponentTable` (156 entries) and `ComponentTableCount`.
 *   - `KdbDebuggerCommands` (45 entries) and `KdbDebuggerCommandsCount`.
 *   - `KDB_ACCESS_TYPE_TO_STRING` / `KDB_ENTER_CONDITION_TO_STRING` /
 *     `NPX_STATE_TO_STRING` enum-to-string helpers.
 * The C originals were silently link-dead: `kdb_cli.rs` declared the arrays
 * as `extern "C" { static mut FOO: [T; 0]; }` so any indexing folded to a
 * panic. Properly sized Rust arrays unblock filter/help/dispatch.
 *
 * The original C kdb_cli.c is excluded from the build (see ntos.cmake).
 */

#include <ntoskrnl.h>
#include "kdb.h"
#include "../kd/kdterminal.h"

/* PKDBG_CLI_ROUTINE was a file-private typedef in kdb_cli.c. Re-declare it
 * here. PCH is plain `char *` from CRT. */
typedef
BOOLEAN
(NTAPI *PKDBG_CLI_ROUTINE)(
    IN PCHAR Command,
    IN ULONG Argc,
    IN PCH Argv[]);

/* -------------------------------------------------------------------------
 * Statics that Rust kdb_cli.rs references via `extern "C" { static mut X: T; }`
 * Definitions intentionally drop `static` so the symbols are externally
 * linkable. Volatile qualifiers are dropped (Rust has its own access pattern).
 * ------------------------------------------------------------------------- */

/* CLI callback slots — fixed-size array, as in the original. */
PKDBG_CLI_ROUTINE KdbCliCallbacks[10];

/* Output state / mode flags. */
BOOLEAN KdbUseIntelSyntax = TRUE;
BOOLEAN KdbBreakOnModuleLoad = FALSE;
ULONG   KdbNumberOfRowsPrinted = 0;
ULONG   KdbNumberOfColsPrinted = 0;
BOOLEAN KdbOutputAborted = FALSE;
BOOLEAN KdbRepeatLastCommand = FALSE;

/* KdbInitFileBuffer / KdbpBugCheckRequested are NOT static in C —
 * they're defined in kdb_cli.c with external linkage and referenced
 * from kdb.c and kdb_cli.c. Define them here since C kdb_cli.c is gone. */
volatile PCHAR KdbInitFileBuffer = NULL;
BOOLEAN KdbpBugCheckRequested = FALSE;

/* Dmesg ring buffer state. */
const ULONG     KdpDmesgBufferSize = 128 * 1024;
PCHAR           KdpDmesgBuffer = NULL;
volatile ULONG  KdpDmesgCurrentPosition = 0;
volatile ULONG  KdpDmesgFreeBytes = 0;
volatile ULONG  KdbDmesgTotalWritten = 0;
volatile BOOLEAN KdbpIsInDmesgMode = FALSE;
KSPIN_LOCK      KdpDmesgLogSpinLock;

/* Prompt string. */
const CSTRING KdbPromptStr = RTL_CONSTANT_STRING("kdb:> ");

/* -------------------------------------------------------------------------
 * Macro / FORCEINLINE wrappers from intrin_i.h and internal/i386/ke.h.
 * Rust calls these via `extern "C" { fn ...; }` — provide non-inline
 * definitions here. The arch-specific FRAME/STACK register helpers wrap
 * a single struct-field access each.
 * ------------------------------------------------------------------------- */

/* Ke386GetGlobalDescriptorTable / Ke386GetLocalDescriptorTable are macros
 * for __sgdt / __sldt in intrin_i.h. */
#undef Ke386GetGlobalDescriptorTable
#undef Ke386GetLocalDescriptorTable

VOID
__cdecl
Ke386GetGlobalDescriptorTable(_Out_ PVOID Descriptor)
{
    __sgdt(Descriptor);
}

VOID
__cdecl
Ke386GetLocalDescriptorTable(_Out_ PUSHORT Limit)
{
    __sldt(Limit);
}

#ifdef _M_IX86

/* Ke386GetTr() — FORCEINLINE in intrin_i.h, but Rust links by name. */
USHORT
__cdecl
Ke386GetTr(VOID)
{
    USHORT Tr;
    __asm__("str %0\n\t" : "=m"(Tr));
    return Tr;
}

ULONG_PTR __cdecl KeGetContextFrameRegister(PCONTEXT Context)         { return Context->Ebp; }
VOID      __cdecl KeSetContextFrameRegister(PCONTEXT Context, ULONG_PTR Frame) { Context->Ebp = Frame; }
ULONG_PTR __cdecl KeGetTrapFrameFrameRegister(PKTRAP_FRAME TrapFrame) { return TrapFrame->Ebp; }
ULONG_PTR __cdecl KeGetTrapFrameStackRegister(PKTRAP_FRAME TrapFrame)
{
    if (TrapFrame->PreviousPreviousMode == KernelMode)
        return TrapFrame->TempEsp;
    return TrapFrame->HardwareEsp;
}
#endif /* _M_IX86 */

#ifdef _M_AMD64
ULONG_PTR __cdecl KeGetContextFrameRegister(PCONTEXT Context)         { return Context->Rbp; }
VOID      __cdecl KeSetContextFrameRegister(PCONTEXT Context, ULONG_PTR Frame) { Context->Rbp = Frame; }
ULONG_PTR __cdecl KeGetTrapFrameFrameRegister(PKTRAP_FRAME TrapFrame) { return TrapFrame->Rbp; }
ULONG_PTR __cdecl KeGetTrapFrameStackRegister(PKTRAP_FRAME TrapFrame) { return TrapFrame->Rsp; }
#endif /* _M_AMD64 */

/* -------------------------------------------------------------------------
 * KdbpPrint — C-variadic. Stable Rust can't define C-variadic exports, so
 * the va_arg pull stays here; format parsing and byte emission live in the
 * Rust ntprintf crate via ntprintf_va_format_a.
 * ------------------------------------------------------------------------- */

VOID KdbpPagerInternal(IN PCHAR Buffer, IN ULONG BufLength, IN BOOLEAN DoPage);

extern size_t ntprintf_va_format_a(char *dst, size_t cap,
                                   const char *fmt, va_list ap);

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
