/*
 * kernel_c_helpers.c — residual C helpers that cannot be implemented in
 * Rust. See `docs/engineering/shim-elimination.md` and
 * `git log --oneline -- ntoskrnl/kernel_c_helpers.c` for audit trail.
 *
 * Survivors:
 *   - C-variadic functions (Rust can't define C-variadic exports on
 *     stable): KdpDprintf, RtlStringCbPrintfA, the KDBG ExpKdbgExt*
 *     extensions that call KdbpPrint.
 *   - SepInitTokensShim: documented LLVM i386 fat-LTO ud2 workaround.
 *     Re-probed 2026-05-07; opaque_ptr/compiler_fence/black_box don't
 *     defeat it. Putting token init outside Rust LTO scope is required.
 *   - PS-subsystem C globals (PspSystemDllBase, PsInitialSystemProcess,
 *     Psp{Process,Thread}Mapping, SharedUserData, ...). Rust imports
 *     them as `extern "C" { static mut ... }`.
 *   - ARM64 UART debug helpers (UART pokes from bringup-path Rust).
 *   - MM section-segment lock outlines that need C `__FILE__`/`__LINE__`.
 *   - Header-rename machinery (Phase 1/3) for FORCEINLINE outlines and
 *     file-scope-static array re-export.
 */

/* Phase 1: Rename FORCEINLINE/macro functions BEFORE including headers.
 * This causes the header to create _saved_* inline versions that we can
 * call from our Phase 5 outline function definitions below. */
#define RtlStringCbPrintfA _saved_RtlStringCbPrintfA

/* Rename static const arrays from ps_i.h so we can re-include with
 * external linkage later. The first inclusion (via ntoskrnl.h) creates
 * _static_* copies; the second creates the real names. */
#define PsProcessInfoClass _static_PsProcessInfoClass
#define PsThreadInfoClass  _static_PsThreadInfoClass

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

/* Undo the ntstrsafe.h "poison" macros */
#undef RtlStringVPrintfWorkerA

/* Phase 3: Undef the rename so we can define the real function */
#undef RtlStringCbPrintfA
#undef PsProcessInfoClass
#undef PsThreadInfoClass
#undef SharedUserData
#undef ARM64_UART_TRACE
#undef ARM64_UART_TRACE_HEX


/*
 * SepInitTokensShim — C helper to work around LLVM i386 LTO bug.
 *
 * During fat LTO on i386, LLVM incorrectly inserts unreachable/ud2 after the
 * `@ObInitializeFastReference@8` call that sets up the system process token.
 * This drops all subsequent code (anonymous logon token creation, return TRUE).
 * The bug does not affect amd64 or arm64.
 *
 * Re-probed 2026-05-07 by inlining the body into se/semgr.rs. The bug is
 * confirmed real — `test_no_unexpected_ud2` catches it after each rebuild.
 * None of `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box`
 * defeat the miscompilation. Performing token init in C (outside Rust LTO
 * scope) is the only working workaround.
 */
BOOLEAN
SepInitTokensShim(VOID)
{
    PTOKEN tok;

    /* Initialize process token fast reference to NULL */
    ExInitializeFastReference(&PsGetCurrentProcess()->Token, NULL);

    /* Create the system process token */
    tok = SepCreateSystemProcessToken();
    ObInitializeFastReference(&PsGetCurrentProcess()->Token, tok);

    /* Create anonymous logon tokens */
    SeAnonymousLogonToken = SepCreateSystemAnonymousLogonToken();
    if (!SeAnonymousLogonToken) return FALSE;

    SeAnonymousLogonTokenNoEveryone = SepCreateSystemAnonymousLogonTokenNoEveryone();
    if (!SeAnonymousLogonTokenNoEveryone) return FALSE;

    return TRUE;
}

/*
 * KdpDprintf — variadic debug printf for KD64 subsystem.
 *
 * The va_arg pull stays in C; format-string parsing + emission live in the
 * Rust ntprintf crate via ntprintf_va_format_a.
 * Only needed when KD64 is Rust (C kdprint.c provides its own when KD64_USE_C).
 */
extern size_t ntprintf_va_format_a(char *dst, size_t cap,
                                   const char *fmt, va_list ap);

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

/*
 * RtlStringCbPrintfA — variadic safe string printf.
 *
 * va_arg pull stays in C; format-string parsing + emission via Rust ntprintf.
 * Returns STATUS_BUFFER_OVERFLOW on truncation, STATUS_SUCCESS otherwise,
 * matching the contract of the original RtlStringVPrintfWorkerA helper.
 */
NTSTATUS __cdecl RtlStringCbPrintfA(
    char *pszDest,
    size_t cbDest,
    const char *pszFormat,
    ...)
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

    /* ntprintf_va_format_a returns the byte count that would have been
     * written excluding NUL. Truncation when written + 1 > cbDest. */
    return (written + 1 > cbDest) ? STATUS_BUFFER_OVERFLOW : STATUS_SUCCESS;
}

/* ExpKdbgExtFileCache / ExpKdbgExtDefWrites / ExpKdbgExtHandle moved to
 * ntoskrnl-rust::ex::handle. The Rust `kdbp_print!` macro replaces variadic
 * KdbpPrint and routes through ntprintf which already handles `%wZ` / `%S`. */

/* ===================================================================
 * PsProcessInfoClass / PsThreadInfoClass — re-include ps_i.h with
 * external linkage (override 'static' to nothing).
 * The first inclusion (via ntoskrnl.h) created _static_* copies.
 * This second inclusion creates the real names with external linkage.
 * Previously in ps_rust_shims.c.
 * =================================================================== */
#pragma push_macro("static")
#define static
#include "internal/ps_i.h"
#pragma pop_macro("static")

/* PS-subsystem globals + SharedUserData — moved to Rust
 * (ntoskrnl-rust::ps::ps_globals). */

/* ===================================================================
 * ARM64_UART_TRACE / ARM64_UART_TRACE_HEX — UART debug output functions.
 * Called from Rust via extern "C" on ARM64 only.
 * Previously in ps_rust_shims.c.
 * =================================================================== */

VOID NTAPI ARM64_UART_TRACE(PCSTR msg)
{
#ifdef _M_ARM64
    extern volatile PVOID Arm64UartKernelBase;
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (u) { while (*msg) *u = (ULONG)*msg++; }
#else
    UNREFERENCED_PARAMETER(msg);
#endif
}

VOID NTAPI ARM64_UART_TRACE_HEX(PCSTR prefix, ULONG_PTR val)
{
#ifdef _M_ARM64
    static const char hex[] = "0123456789ABCDEF";
    int i;
    extern volatile PVOID Arm64UartKernelBase;
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (!u) return;
    /* Print prefix */
    while (*prefix) *u = (ULONG)*prefix++;
    /* Print hex value */
    *u = '0'; *u = 'x';
    for (i = (sizeof(val)*2 - 1); i >= 0; i--)
        *u = hex[(val >> (i*4)) & 0xF];
#else
    UNREFERENCED_PARAMETER(prefix);
    UNREFERENCED_PARAMETER(val);
#endif
}

/* ===================================================================
 * MM section-segment lock/unlock — uses __FILE__/__LINE__ C macros so
 * the underlying _Mm* primitives can record where the lock was taken.
 * Rust can't expand these macros, hence the C outlines.
 * =================================================================== */

#include <mm/ARM3/miarm.h>

#undef MmLockSectionSegment
#undef MmUnlockSectionSegment
#undef MmGetPageEntrySectionSegment

void NTAPI MmLockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmLockSectionSegment(Segment, __FILE__, __LINE__); }

void NTAPI MmUnlockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmUnlockSectionSegment(Segment, __FILE__, __LINE__); }

ULONG_PTR NTAPI MmGetPageEntrySectionSegment(
    IN PMM_SECTION_SEGMENT Segment, IN PLARGE_INTEGER Offset)
{ return _MmGetPageEntrySectionSegment(Segment, Offset, __FILE__, __LINE__); }

