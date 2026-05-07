/*
 * mm_rust_shims.c — Non-inline ("outline") implementations of FORCEINLINE/macro
 * functions that the Rust ntoskrnl-rust MM crate calls via extern declarations.
 *
 * Strategy: Rename only the target FORCEINLINE functions so we can define
 * our own with the original name. Lower-level deps (KeEnterGuardedRegion,
 * ExAcquirePushLockExclusive, etc.) keep their names so the renamed originals
 * can call them normally.
 */

/* === Phase 1: Rename only the functions we're shimming === */

/* All `_orig_*` renames have been retired — every former trampoline that
 * delegated to `_orig_*` is now bare-name `extern "C"` exported from
 * shim_exports.rs (delegating to ntkrnl_sys::inline::mm). */
/* KiAcquireProcessLockRaiseToSynch, KiReleaseProcessLock are FORCEINLINE in
 * ke_x.h (PCH-covered) — can't rename, manually inlined below.
 * UpdateTotalCommittedPages, KeInvalidateTlbEntry, MmDereferenceSegment are
 * FORCEINLINE in PCH-covered headers — can't rename, manually inlined below */
/* KfRaiseIrql/KfLowerIrql: see Phase 4 for i386/amd64 wrappers */

/* MmLockSectionSegment / MmUnlockSectionSegment / MmGetPageEntrySectionSegment
 * renames retired — those outlines are canonical in kernel_c_helpers.c
 * (they need __FILE__/__LINE__ macro expansion). */

/* === Phase 2: Include headers === */
#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>
#include <mm/ARM3/miarm.h>
/* Include arch-specific mm.h for FORCEINLINE functions like MiIsPdeForAddressValid */
#include <internal/arch/mm.h>

/* All Phase 3 undefs retired alongside the Phase 1 renames. */

/* === Globals that were previously in section.c === */
/* Guarded: section.c provides these when compiled as C */
#if !defined(MM_SECTION_C_ACTIVE)
KEVENT MmWaitPageEvent;
ULONG_PTR MmSubsectionBase;
#endif

/* MmVerifyImageIsOkForMpUse — only needed in CONFIG_SMP (ntkrnlmp) build.
 * Was in sysldr.c under #ifdef CONFIG_SMP, not translated to Rust.
 * Guarded: sysldr.c provides this when compiled as C */
#if !defined(MM_SYSLDR_C_ACTIVE) && defined(CONFIG_SMP)
BOOLEAN
NTAPI
MmVerifyImageIsOkForMpUse(
    _In_ PVOID BaseAddress)
{
    PIMAGE_NT_HEADERS NtHeaders;
    PAGED_CODE();

    NtHeaders = RtlImageNtHeader(BaseAddress);
    if (!NtHeaders)
        return TRUE;

    /* Fail if we have more than one CPU, but the image is only safe for UP */
    if ((KeNumberProcessors > 1) &&
        (NtHeaders->FileHeader.Characteristics & IMAGE_FILE_UP_SYSTEM_ONLY))
    {
        return FALSE;
    }
    return TRUE;
}
#endif // !MM_SYSLDR_C_ACTIVE && CONFIG_SMP

/* === Phase 4: Export implementations === */

/*
 * Hardware PTE creation trampolines REMOVED — duplicated by
 * ntoskrnl-rust::shim_exports `extern "C"` exports (cdecl on all 3
 * arches), which delegate into mm-macros / ntkrnl_sys::inline::mm.
 * No C-side stdcall caller imports `_MI_MAKE_HARDWARE_PTE*@N` (verified
 * via nm scan of all .obj files), so the C trampolines were dead.
 */

#ifdef _M_ARM64
VOID
NTAPI
Arm64DumpPte(
    IN PCSTR Prefix,
    IN PVOID Address)
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
#endif

/*
 * MI_UPDATE_VALID_PTE trampoline REMOVED — duplicated by shim_exports.
 */

/* ARM64 direct UART for tracing (DbgPrint doesn't reach serial on ARM64).
 * Used by `MmDbgCheckIrql` (called from procsup.rs / ps/process.rs) — kept
 * even after the WS-lock trampolines were dropped. */
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
#endif

/*
 * Working-set lock family + dead debug instrumentation REMOVED —
 * 10 trampolines (MiLock/UnlockProcessWorkingSet{,Shared,Unsafe},
 * MiLock/UnlockWorkingSet{,Shared}) are now bare-name `extern "C"`
 * exports in `ntoskrnl-rust::shim_exports`, delegating into
 * `ntkrnl_sys::inline::mm::*`.
 *
 * The C trampolines carried two pieces of debug instrumentation:
 *   - `MmDbgLast{Safe,Excl}{Locker,Unlocker}` global pointer stores
 *     captured `__builtin_return_address(0)` of the last lock/unlock.
 *     Only consumer was `MmDebugWsFaultCheck`, which had no callers —
 *     the entire chain was dead code.
 *   - ARM64 IRQL UART traces flagged IRQL > APC_LEVEL at lock/unlock.
 *     These caught a fixed-since IRQL bug; if needed again they belong
 *     in `ntkrnl_sys::inline::mm` (with the per-arch UART helpers).
 *
 * `MmDebugTrace` (declared but never called from any caller) deleted
 * here too, plus its `extern fn` decl in `mm/mmfault.rs`.
 */

/* MiAcquireExpansionLock / MiReleaseExpansionLock trampolines REMOVED
 * — duplicated by ntoskrnl-rust::shim_exports `extern "C"` exports
 * (cdecl on all 3 arches, matching what was here). The C trampolines
 * also carried `MiExpansionLockOwner = PsGetCurrentThread()` tracking
 * + an ARM64 IRQL UART trace; these were already dead because the Rust
 * archive wins the link under --allow-multiple-definition. The miarm.h
 * FORCEINLINE ASSERTs that referenced MiExpansionLockOwner sat inside
 * the renamed `_orig_*` paths and were unreachable from the live link
 * order. Tracking is a planned follow-up. */

/*
 * Phase 1 batches M1/M2/M6 PTE-classification + bitfield-write +
 * atomic-exchange trampolines REMOVED. The C trampolines here were pure
 * 1-line delegations to mm-pte-class-inlines / mm-pte-mut-inlines /
 * mm-pte-atomic-inlines `_rs_*` impls, AND ntoskrnl-rust::shim_exports
 * already exports the same unprefixed `Mi*` names with identical bodies
 * (delegate to ntkrnl_sys::inline::mm::*). With
 * `--allow-multiple-definition` the Rust archive's exports win the link;
 * the C trampolines were unreachable. Names covered:
 *  MiIsMappedPte / MiDetermineUserGlobalPteMask / MM_ANY_WS_LOCK_HELD /
 *  MiIsPdeForAddressValid / MiMakeDirtyPage / MiMakeCleanPage /
 *  MiMakeAccessedPage / MiPageDisableCache / MiPageWriteThrough /
 *  MiPageWriteCombined / MiIsPageWriteable / MiIsPageDirty /
 *  MiMakeOwnerPage / MiMakeWritePage / MiInterlockedCompareExchangePte /
 *  MiInterlockedExchangePte.
 */

/*
 * InterlockedExchange / InterlockedCompareExchange — used by Rust balance.rs as
 * extern "C" (cdecl).
 *
 * On amd64/arm64: exintrin.c provides these as FASTCALL (which expands to cdecl
 * on non-i386) with an undecorated symbol. Rust's extern "C" also expects the
 * undecorated symbol. No shim needed.
 *
 * On i386: exintrin.c provides these as __fastcall (symbol @Name@N), but Rust's
 * extern "C" expects the cdecl symbol _Name. We provide cdecl wrappers here.
 * The headers define InterlockedExchange/_CompareExchange as macros, so undef first.
 */
/* InterlockedExchange / InterlockedCompareExchange (i386 cdecl wrappers)
 * are now canonical in kernel_c_helpers.c. */

/* InterlockedIncrementUL trampoline REMOVED — duplicated by shim_exports
 * (delegates into mm-util-inlines / ntkrnl_sys::inline::mm).
 * No C-side stdcall caller imports `_InterlockedIncrementUL@4`. */

/* UpdateTotalCommittedPages / KeInvalidateTlbEntry trampolines REMOVED
 * — duplicated by ntoskrnl-rust::shim_exports `extern "C"` exports
 * (cdecl on all 3 arches). */

/* MmLockSectionSegment / MmUnlockSectionSegment / MmGetPageEntrySectionSegment
 * are now canonical in kernel_c_helpers.c (need __FILE__/__LINE__ macros). */

/* MmDereferenceSegment trampoline REMOVED — duplicated by shim_exports. */

/*
 * ExAllocatePoolZero — bindgen declares this as `extern "stdcall"` on
 * i386, so callers want `_ExAllocatePoolZero@12`. shim_exports.rs only
 * provides the cdecl `_ExAllocatePoolZero`. The trampoline below
 * supplies the stdcall variant; body delegates to mm-util-inlines.
 * Safe to drop on amd64/arm64 (only one symbol per name there) but
 * keeping it uniform across arches keeps the .c smaller than wrapping
 * the body in an arch ifdef.
 */
extern PVOID _rs_ExAllocatePoolZero(POOL_TYPE PoolType, SIZE_T NumberOfBytes, ULONG Tag);
PVOID
NTAPI
ExAllocatePoolZero(
    IN POOL_TYPE PoolType,
    IN SIZE_T NumberOfBytes,
    IN ULONG Tag)
{
    return _rs_ExAllocatePoolZero(PoolType, NumberOfBytes, Tag);
}

/* MmGetAddressSpaceOwner / MmGetKernelAddressSpace / MmLockAddressSpace
 * / MmUnlockAddressSpace trampolines REMOVED — duplicated by
 * shim_exports `extern "C"` exports (cdecl on all 3 arches). */

/* InterlockedExchangeAddUL / RMAP_IS_SEGMENT / PAGE_ROUND_DOWN /
 * PFN_FROM_SSE / MM_IS_WAIT_PTE / MmSetDirtyPage / MmInitializeMdl
 * trampolines REMOVED — all duplicated by shim_exports `extern "C"`
 * exports (cdecl), delegating into mm-macros / mm-util-inlines /
 * ntkrnl_sys::inline::mm. No C-side stdcall callers in any .obj. */

/* ===================================================================
 * AVL Tree support for vadnode.rs — translated to Rust 2026-05-06.
 *
 * vadnode.c used to #include "miavl.h" + avlsupp.c to get MM-flavored
 * AVL routines (Mi* names). The C trampolines that re-exposed those
 * under the Rtl* / Rtlp* names that Rust vadnode.rs links against are
 * now thin delegations into the `mm-avl` sibling rlib (see
 * `sdk/lib/rust/mm-avl/src/lib.rs`).
 *
 * The 5 trivial accessors (Right/Left/Parent/IsLeft/IsRight) and the 3
 * heavyweight algorithms (Find/Insert/Delete) all live in the rlib now.
 * The original C avlsupp.c-include trick is preserved in the
 * `#if 0` block below for side-by-side debug.
 * =================================================================== */

/* All 8 mm-avl fns (RtlRightChildAvl / RtlLeftChildAvl / RtlParentAvl /
 * RtlIsRightChildAvl / RtlIsLeftChildAvl / RtlpFindAvlTableNodeOrParent /
 * RtlpInsertAvlTreeNode / RtlpDeleteAvlTreeNode) are bare-name exports
 * from the `mm-avl` rlib — no C trampoline. The duplicate-insertion
 * guard that lived here is now an early-return inside the rlib's
 * `RtlpInsertAvlTreeNode` body. */

/* ===================================================================
 * FORCEINLINE shims for Tier 3a Rust integration
 *
 * These functions are FORCEINLINE in miarm.h / ke_x.h — they have no
 * C symbol.  The Rust code calls them via extern "C" declarations.
 * All were renamed in Phase 1, so _orig_* versions are available.
 * =================================================================== */

/* MiIncrementPageTableReferences — now implemented in Rust (helpers.rs)
 * to ensure it uses the same UsedPageTableEntries accessor as
 * MiDecrementPageTableReferences and MiIsPageTablePresent. */

/* Tier 3a/4 FORCEINLINE trampolines REMOVED. The C `_orig_*` paths were
 * shadowed by ntoskrnl-rust::shim_exports `#[no_mangle] Mi*` exports
 * (delegating to ntkrnl_sys::inline::mm::*) — link-time
 * `--allow-multiple-definition` picks the Rust archive. Names covered:
 *  MiDropLockCount, MiReferenceProbedPageAndBumpLockCount,
 *  MiReferenceUnusedPageAndBumpLockCount, MiReferenceUsedPageAndBumpLockCount,
 *  MiIsUserPde, MiIsUserPte, MiRemoveZeroPageSafe, MiIsMemoryTypeFree,
 *  MiSynchronizeSystemPde. */

/* MiConvertSharedWorkingSetLockToExclusive trampoline REMOVED — bare-name
 * `extern "C"` export now in shim_exports.rs (delegates into
 * `ntkrnl_sys::inline::mm::MiConvertSharedWorkingSetLockToExclusive`). */

/* KiAcquireProcessLockRaiseToSynch / KiReleaseProcessLock trampolines
 * REMOVED — both are now bare-name `extern "C"` exports in
 * shim_exports.rs. The plain `KiReleaseProcessLock` calls
 * `KeReleaseInStackQueuedSpinLock` directly from Rust (the bindgen decl
 * is `extern "fastcall"` on i386, picked up automatically). */

/* ===================================================================
 * Tier 3b FORCEINLINE shims (ARM3/section.c, ARM3/virtual.c)
 * =================================================================== */

/* MiLockProcessWorkingSetForFault / MiUnlockProcessWorkingSetForFault
 * trampolines REMOVED — bare-name exports now in shim_exports.rs. */

/* MmGetCurrentAddressSpace / ProbeForWriteSize_t / ProbeForWriteIoStatusBlock
 * trampolines REMOVED — duplicated by shim_exports.
 * MiDereferencePfnAndDropLockCount: provided by Rust (section.rs).
 * MiIsRosSectionObject: provided by shim_exports. */

/*
 * KfRaiseIrql / KfLowerIrql wrappers for Rust (extern "C" / cdecl).
 *
 * - amd64: FORCEINLINE in SDK header (__readcr8/__writecr8) — no HAL symbol.
 *   Manually inline the implementation.
 * - i386: HAL exports as __fastcall (@KfRaiseIrql@4). Rust needs cdecl.
 *   Call the HAL fastcall function, wrap in cdecl.
 * - ARM64: HAL exports as cdecl. Rust's extern "C" works directly.
 *   But we provide a wrapper anyway for uniformity (compiler will inline it).
 */
/* KfRaiseIrql / KfLowerIrql / RustShimKfRaiseIrql / RustShimKfLowerIrql
 * are now canonical in kernel_c_helpers.c. */

/* RustShimRtlFindSetBits / RustShimKdbpGetHexNumber are now canonical
 * in kernel_c_helpers.c. */

/* MmFixupImageSectionSegments and its 4 helpers (MmspSortSegments,
 * MmspCheckSegmentBounds, MmspPageAlignSegments, MmspCompareSegments)
 * are now in mm/section.rs. Original C body deleted; git history before
 * b20f8cf1 has the full original. */

/* MmDebugTrace REMOVED — declared in mm/mmfault.rs but never actually
 * called from any caller. The extern decl in mmfault.rs was removed in
 * the same commit. */


#ifdef _M_ARM64
/* ARM64 debugging: check and report IRQL via UART */
VOID NTAPI MmDbgCheckIrql(PCSTR label)
{
    KIRQL irql = KeGetCurrentIrql();
    /* Always trace — unconditional */
    MmUartPuts("[IRQL-CHK] ");
    MmUartPuts(label);
    MmUartHex(" irql=", irql);
}
#endif

/* MiAcquirePfnLock / MiReleasePfnLock trampolines REMOVED — bare-name
 * exports now in shim_exports.rs (delegating into ntkrnl_sys::mm). */
