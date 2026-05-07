/*
 * mm_rust_shims.c — surviving C-side helpers for the Rust MM crate.
 *
 * Most of what used to live here (PTE accessors, working-set locks,
 * AVL bridges, macros, allocator wrappers) has been promoted to bare-
 * name `#[no_mangle] extern "C" fn` exports in
 * `sdk/lib/rust/ntoskrnl-rust/src/shim_exports.rs` or to sibling
 * `*-inlines` / `*-macros` rlibs. See `docs/engineering/shim-elimination.md`
 * for the audit trail; `git log --oneline -- ntoskrnl/mm/mm_rust_shims.c`
 * has the per-trampoline removal commits.
 *
 * What's left in this file:
 *   - `MmWaitPageEvent` / `MmSubsectionBase` globals (when section.c is
 *     compiled out).
 *   - `MmVerifyImageIsOkForMpUse` (ntkrnlmp/CONFIG_SMP build only).
 *   - ARM64 debug helpers: `Arm64DumpPte`, `MmUartPuts`/`MmUartHex`,
 *     `MmDbgCheckIrql`. Called from procsup.rs / ps/process.rs and the
 *     ke/arm64/stubs.rs APC + fault paths. `DbgPrint` doesn't reach
 *     serial on ARM64, hence the direct UART pokes.
 *   - `ExAllocatePoolZero` stdcall trampoline. bindgen declares this
 *     `extern "stdcall"` on i386, producing `_ExAllocatePoolZero@12`;
 *     `shim_exports.rs` only supplies the cdecl `_ExAllocatePoolZero`,
 *     so the stdcall variant has to come from a C trampoline.
 */

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>
#include <mm/ARM3/miarm.h>
#include <internal/arch/mm.h>

/* === Globals previously owned by section.c (when section.c is the
 * canonical owner — `MM_SECTION_C_ACTIVE` is set by the build) === */
#if !defined(MM_SECTION_C_ACTIVE)
KEVENT MmWaitPageEvent;
ULONG_PTR MmSubsectionBase;
#endif

/* === MmVerifyImageIsOkForMpUse — CONFIG_SMP-only path from sysldr.c === */
#if !defined(MM_SYSLDR_C_ACTIVE) && defined(CONFIG_SMP)
BOOLEAN
NTAPI
MmVerifyImageIsOkForMpUse(_In_ PVOID BaseAddress)
{
    PIMAGE_NT_HEADERS NtHeaders;
    PAGED_CODE();

    NtHeaders = RtlImageNtHeader(BaseAddress);
    if (!NtHeaders)
        return TRUE;

    /* Fail if we have more than one CPU but the image is UP-only. */
    if ((KeNumberProcessors > 1) &&
        (NtHeaders->FileHeader.Characteristics & IMAGE_FILE_UP_SYSTEM_ONLY))
    {
        return FALSE;
    }
    return TRUE;
}
#endif

/* === ARM64 debug helpers === */
#ifdef _M_ARM64
extern volatile PVOID Arm64UartKernelBase;

static void MmUartPuts(const char *s) {
    volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
    if (u) while (*s) *u = (ULONG)*s++;
}

static void MmUartHex(const char *prefix, ULONG_PTR val) {
    char buf[20];
    int i;
    MmUartPuts(prefix);
    for (i = 15; i >= 0; i--)
        buf[15-i] = "0123456789ABCDEF"[(val >> (i*4)) & 0xF];
    buf[16] = '\r'; buf[17] = '\n'; buf[18] = 0;
    MmUartPuts(buf);
}

VOID
NTAPI
Arm64DumpPte(IN PCSTR Prefix, IN PVOID Address)
{
    PMMPTE Pte = MiAddressToPte(Address);

    ARM64_UART_TRACE_HEX(Prefix, (ULONG_PTR)Address);
    ARM64_UART_TRACE_HEX("[PTE] Entry=", Pte->u.Long);
    ARM64_UART_TRACE_HEX("[PTE] Valid=", Pte->u.Hard.Valid);
    ARM64_UART_TRACE_HEX("[PTE] Owner=", Pte->u.Hard.Owner);
    ARM64_UART_TRACE_HEX("[PTE] Writable=", Pte->u.Hard.Writable);
    ARM64_UART_TRACE_HEX("[PTE] UserNoExec=", Pte->u.Hard.UserNoExecute);
    ARM64_UART_TRACE_HEX("[PTE] CopyOnWrite=", Pte->u.Hard.CopyOnWrite);
    ARM64_UART_TRACE_HEX("[PTE] NotDirty=", Pte->u.Hard.NotDirty);
    ARM64_UART_TRACE_HEX("[PTE] Accessed=", Pte->u.Hard.Accessed);
    ARM64_UART_TRACE_HEX("[PTE] NonGlobal=", Pte->u.Hard.NonGlobal);
    ARM64_UART_TRACE_HEX("[PTE] Pfn=", Pte->u.Hard.PageFrameNumber);
}

VOID NTAPI MmDbgCheckIrql(PCSTR label)
{
    KIRQL irql = KeGetCurrentIrql();
    MmUartPuts("[IRQL-CHK] ");
    MmUartPuts(label);
    MmUartHex(" irql=", irql);
}
#endif

/* === ExAllocatePoolZero stdcall variant for i386 ===
 * shim_exports.rs supplies cdecl `_ExAllocatePoolZero`; bindgen on
 * i386 wants stdcall `_ExAllocatePoolZero@12`. */
extern PVOID _rs_ExAllocatePoolZero(POOL_TYPE PoolType, SIZE_T NumberOfBytes, ULONG Tag);

PVOID
NTAPI
ExAllocatePoolZero(IN POOL_TYPE PoolType, IN SIZE_T NumberOfBytes, IN ULONG Tag)
{
    return _rs_ExAllocatePoolZero(PoolType, NumberOfBytes, Tag);
}
