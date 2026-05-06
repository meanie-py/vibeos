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

/* FORCEINLINE functions (from miarm.h, mm.h, ke.h) */
#define MI_MAKE_HARDWARE_PTE_KERNEL _orig_MI_MAKE_HARDWARE_PTE_KERNEL
#define MI_MAKE_HARDWARE_PTE_USER   _orig_MI_MAKE_HARDWARE_PTE_USER
#define MI_MAKE_HARDWARE_PTE        _orig_MI_MAKE_HARDWARE_PTE
#define MI_UPDATE_VALID_PTE         _orig_MI_UPDATE_VALID_PTE
#define MiLockProcessWorkingSet     _orig_MiLockProcessWorkingSet
#define MiUnlockProcessWorkingSet   _orig_MiUnlockProcessWorkingSet
#define MiLockProcessWorkingSetShared   _orig_MiLockProcessWorkingSetShared
#define MiUnlockProcessWorkingSetShared _orig_MiUnlockProcessWorkingSetShared
#define MiLockProcessWorkingSetUnsafe   _orig_MiLockProcessWorkingSetUnsafe
#define MiUnlockProcessWorkingSetUnsafe _orig_MiUnlockProcessWorkingSetUnsafe
#define MiLockWorkingSet            _orig_MiLockWorkingSet
#define MiUnlockWorkingSet          _orig_MiUnlockWorkingSet
#define MiLockWorkingSetShared      _orig_MiLockWorkingSetShared
#define MiUnlockWorkingSetShared    _orig_MiUnlockWorkingSetShared
#define MiAcquireExpansionLock      _orig_MiAcquireExpansionLock
#define MiReleaseExpansionLock      _orig_MiReleaseExpansionLock
#define MiDetermineUserGlobalPteMask _orig_MiDetermineUserGlobalPteMask
#define MM_ANY_WS_LOCK_HELD         _orig_MM_ANY_WS_LOCK_HELD
/* Tier 3b FORCEINLINE shims (miarm.h, mm.h) */
#define MiLockProcessWorkingSetForFault  _orig_MiLockProcessWorkingSetForFault
#define MiUnlockProcessWorkingSetForFault _orig_MiUnlockProcessWorkingSetForFault
/* MmGetCurrentAddressSpace: PCH-covered FORCEINLINE, can't rename — manually inlined below */
/* MiDereferencePfnAndDropLockCount: provided by Rust (section.rs) — no C shim needed */
#define MiIsRosSectionObject _orig_MiIsRosSectionObject
/* Tier 3a FORCEINLINE shims (miarm.h, ke_x.h) */
#define MiIncrementPageTableReferences  _orig_MiIncrementPageTableReferences
#define MiDropLockCount                 _orig_MiDropLockCount
#define MiReferenceProbedPageAndBumpLockCount  _orig_MiReferenceProbedPageAndBumpLockCount
#define MiReferenceUnusedPageAndBumpLockCount  _orig_MiReferenceUnusedPageAndBumpLockCount
#define MiReferenceUsedPageAndBumpLockCount    _orig_MiReferenceUsedPageAndBumpLockCount
#define MiIsUserPde                     _orig_MiIsUserPde
#define MiIsUserPte                     _orig_MiIsUserPte
#define MiRemoveZeroPageSafe            _orig_MiRemoveZeroPageSafe
#define MiIsMemoryTypeFree              _orig_MiIsMemoryTypeFree
/* Tier 4 FORCEINLINE shims (miarm.h) */
#define MiSynchronizeSystemPde          _orig_MiSynchronizeSystemPde
#define MiConvertSharedWorkingSetLockToExclusive _orig_MiConvertSharedWorkingSetLockToExclusive
#define MiIsPdeForAddressValid          _orig_MiIsPdeForAddressValid
/* KiAcquireProcessLockRaiseToSynch, KiReleaseProcessLock are FORCEINLINE in
 * ke_x.h (PCH-covered) — can't rename, manually inlined below.
 * UpdateTotalCommittedPages, KeInvalidateTlbEntry, MmDereferenceSegment are
 * FORCEINLINE in PCH-covered headers — can't rename, manually inlined below */
/* KfRaiseIrql/KfLowerIrql: see Phase 4 for i386/amd64 wrappers */

/* #define macros (from ex.h, mm.h) — rename to prevent expansion */
/* InterlockedIncrementUL/DecrementUL already shimmed in ps_rust_shims.c */
#define MmLockSectionSegment        _orig_MmLockSectionSegment
#define MmUnlockSectionSegment      _orig_MmUnlockSectionSegment
#define MmGetPageEntrySectionSegment _orig_MmGetPageEntrySectionSegment

/* === Phase 2: Include headers === */
#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>
#include <mm/ARM3/miarm.h>
/* Include arch-specific mm.h for FORCEINLINE functions like MiIsPdeForAddressValid */
#include <internal/arch/mm.h>

/* === Phase 3: Undef renamed functions === */
#undef MI_MAKE_HARDWARE_PTE_KERNEL
#undef MI_MAKE_HARDWARE_PTE_USER
#undef MI_MAKE_HARDWARE_PTE
#undef MI_UPDATE_VALID_PTE
#undef MiLockProcessWorkingSet
#undef MiUnlockProcessWorkingSet
#undef MiLockProcessWorkingSetShared
#undef MiUnlockProcessWorkingSetShared
#undef MiLockProcessWorkingSetUnsafe
#undef MiUnlockProcessWorkingSetUnsafe
#undef MiLockWorkingSet
#undef MiUnlockWorkingSet
#undef MiLockWorkingSetShared
#undef MiUnlockWorkingSetShared
#undef MiAcquireExpansionLock
#undef MiReleaseExpansionLock
#undef MiDetermineUserGlobalPteMask
#undef MM_ANY_WS_LOCK_HELD
/* Tier 3b FORCEINLINE undefs */
#undef MiLockProcessWorkingSetForFault
#undef MiUnlockProcessWorkingSetForFault
/* MmGetCurrentAddressSpace: PCH-covered, no rename — undef before Phase 4 implementation */
/* MiDereferencePfnAndDropLockCount: provided by Rust — no undef needed */
#undef MiIsRosSectionObject
/* Tier 3a FORCEINLINE undefs */
#undef MiIncrementPageTableReferences
#undef MiDropLockCount
#undef MiReferenceProbedPageAndBumpLockCount
#undef MiReferenceUnusedPageAndBumpLockCount
#undef MiReferenceUsedPageAndBumpLockCount
#undef MiIsUserPde
#undef MiIsUserPte
#undef MiRemoveZeroPageSafe
#undef MiIsMemoryTypeFree
/* Tier 4 FORCEINLINE undefs */
#undef MiSynchronizeSystemPde
#undef MiConvertSharedWorkingSetLockToExclusive
#undef MiIsPdeForAddressValid
/* KiAcquireProcessLockRaiseToSynch, KiReleaseProcessLock: PCH-covered, no rename */
/* (UpdateTotalCommittedPages, KeInvalidateTlbEntry, MmDereferenceSegment: no rename) */
/* KfRaiseIrql/KfLowerIrql: see Phase 4 for i386/amd64 wrappers */
/* MmInitializeMdl is a #define macro — undef so we can define an outline function */
#undef MmInitializeMdl
/* Additional macros that need outline wrappers */
#undef RMAP_IS_SEGMENT
#undef PAGE_ROUND_DOWN
#undef InterlockedExchangeAddUL
#undef MmSetDirtyPage
#undef PFN_FROM_SSE
#undef MM_IS_WAIT_PTE
#undef IS_SWAP_FROM_SSE
#undef SWAPENTRY_FROM_SSE
#undef MmLockSectionSegment
#undef MmUnlockSectionSegment
#undef MmGetPageEntrySectionSegment

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
 * Hardware PTE creation — bodies in Rust at sdk/lib/rust/mm-macros/src/lib.rs.
 */
extern void _rs_MI_MAKE_HARDWARE_PTE_KERNEL(PMMPTE, PMMPTE, ULONG_PTR, ULONG_PTR);
extern void _rs_MI_MAKE_HARDWARE_PTE_USER  (PMMPTE, PMMPTE, ULONG_PTR, ULONG_PTR);
extern void _rs_MI_MAKE_HARDWARE_PTE       (PMMPTE, PMMPTE, ULONG_PTR, ULONG_PTR);

void
NTAPI
MI_MAKE_HARDWARE_PTE_KERNEL(
    IN PMMPTE NewPte,
    IN PMMPTE MappingPte,
    IN ULONG_PTR ProtectionMask,
    IN PFN_NUMBER PageFrameNumber)
{
    _rs_MI_MAKE_HARDWARE_PTE_KERNEL(NewPte, MappingPte, ProtectionMask, (ULONG_PTR)PageFrameNumber);
}

void
NTAPI
MI_MAKE_HARDWARE_PTE_USER(
    IN PMMPTE NewPte,
    IN PMMPTE MappingPte,
    IN ULONG_PTR ProtectionMask,
    IN PFN_NUMBER PageFrameNumber)
{
    _rs_MI_MAKE_HARDWARE_PTE_USER(NewPte, MappingPte, ProtectionMask, (ULONG_PTR)PageFrameNumber);
}

void
NTAPI
MI_MAKE_HARDWARE_PTE(
    IN PMMPTE NewPte,
    IN PMMPTE MappingPte,
    IN ULONG_PTR ProtectionMask,
    IN PFN_NUMBER PageFrameNumber)
{
    _rs_MI_MAKE_HARDWARE_PTE(NewPte, MappingPte, ProtectionMask, (ULONG_PTR)PageFrameNumber);
}

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
 * PTE update (volatile write with assertion checks) — body in Rust at
 * sdk/lib/rust/mm-macros/src/lib.rs::_rs_MI_UPDATE_VALID_PTE.
 */
extern void _rs_MI_UPDATE_VALID_PTE(PMMPTE PointerPte, MMPTE TempPte);

void
NTAPI
MI_UPDATE_VALID_PTE(
    IN PMMPTE PointerPte,
    IN MMPTE TempPte)
{
    _rs_MI_UPDATE_VALID_PTE(PointerPte, TempPte);
}

/*
 * Working set locks
 * Bindgen says extern "C" (FORCEINLINE with no calling convention) — no NTAPI.
 */
/* Lightweight caller tracking — just a pointer store, no serial I/O overhead.
 * When MiLockProcessWorkingSetShared fires with OwnsExcl==1, we print these. */
volatile PVOID MmDbgLastExclLocker = NULL;   /* last MiLockProcessWorkingSetUnsafe caller */
volatile PVOID MmDbgLastExclUnlocker = NULL; /* last MiUnlockProcessWorkingSetUnsafe caller */
/* Called from MmAccessFault entry when OwnsExcl is set — print faulting address + EIP */
void MmDebugWsFaultCheck(PVOID Address, PVOID TrapInfo)
{
    PETHREAD Thread = PsGetCurrentThread();
    if (Thread->OwnsProcessWorkingSetExclusive) {
        ULONG_PTR Eip = 0;
#ifdef _M_IX86
        if (TrapInfo) {
            /* KTRAP_FRAME.Eip is at offset 0x68 on i386 */
            Eip = *(ULONG_PTR *)((ULONG_PTR)TrapInfo + 0x68);
        }
#elif defined(_M_AMD64)
        if (TrapInfo) {
            /* KTRAP_FRAME.Rip is at offset 0x168 on amd64 */
            Eip = *(ULONG_PTR *)((ULONG_PTR)TrapInfo + 0x168);
        }
#endif
        DPRINT1("[WS-FAULT] MmAccessFault with OwnsExcl=1! FaultAddr=%p, EIP=%p\n"
                "  LastExclLocker=%p, LastExclUnlocker=%p\n",
                Address, (PVOID)Eip,
                MmDbgLastExclLocker, MmDbgLastExclUnlocker);
    }
}
volatile PVOID MmDbgLastSafeLocker = NULL;   /* last MiLockProcessWorkingSet caller */
volatile PVOID MmDbgLastSafeUnlocker = NULL; /* last MiUnlockProcessWorkingSet caller */

/* ARM64 direct UART for tracing (DbgPrint doesn't reach serial on ARM64) */
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
 * Working-set lock family (Phase 1 batch M4 — bodies delegate to Rust
 * impls in the `mm-ws-lock-inlines` rlib).
 * The trampoline retains the ARM64 IRQL UART tracing and the
 * MmDbgLast{Safe,Excl}{Locker,Unlocker} debug instrumentation —
 * `__builtin_return_address(0)` is meaningful only here in the C frame.
 * The lock-acquisition itself is `_rs_Mi*WorkingSet*`.
 */
extern void _rs_MiLockProcessWorkingSet(PEPROCESS Process, PETHREAD Thread);
void MiLockProcessWorkingSet(IN PEPROCESS Process, IN PETHREAD Thread)
{
    MmDbgLastSafeLocker = __builtin_return_address(0);
#ifdef _M_ARM64
    {
        KIRQL irql = KeGetCurrentIrql();
        if (irql > APC_LEVEL) {
            MmUartPuts("[MM] BUG: MiLockProcessWorkingSet IRQL=");
            MmUartHex("", irql);
            MmUartHex("  caller=", (ULONG_PTR)__builtin_return_address(0));
        }
    }
#endif
    _rs_MiLockProcessWorkingSet(Process, Thread);
}

extern void _rs_MiUnlockProcessWorkingSet(PEPROCESS Process, PETHREAD Thread);
void MiUnlockProcessWorkingSet(IN PEPROCESS Process, IN PETHREAD Thread)
{
    MmDbgLastSafeUnlocker = __builtin_return_address(0);
    _rs_MiUnlockProcessWorkingSet(Process, Thread);
#ifdef _M_ARM64
    {
        KIRQL irql = KeGetCurrentIrql();
        if (irql > APC_LEVEL) {
            MmUartPuts("[MM] IRQL after MiUnlockProcessWorkingSet=");
            MmUartHex("", irql);
            MmUartHex("  caller=", (ULONG_PTR)__builtin_return_address(0));
        }
    }
#endif
}

extern void _rs_MiLockProcessWorkingSetUnsafe(PEPROCESS Process, PETHREAD Thread);
void MiLockProcessWorkingSetUnsafe(IN PEPROCESS Process, IN PETHREAD Thread)
{
    MmDbgLastExclLocker = __builtin_return_address(0);
    _rs_MiLockProcessWorkingSetUnsafe(Process, Thread);
}

extern void _rs_MiUnlockProcessWorkingSetUnsafe(PEPROCESS Process, PETHREAD Thread);
void MiUnlockProcessWorkingSetUnsafe(IN PEPROCESS Process, IN PETHREAD Thread)
{
    MmDbgLastExclUnlocker = __builtin_return_address(0);
    _rs_MiUnlockProcessWorkingSetUnsafe(Process, Thread);
}

extern void _rs_MiLockWorkingSet(PETHREAD Thread, PMMSUPPORT WorkingSet);
void MiLockWorkingSet(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _rs_MiLockWorkingSet(Thread, WorkingSet); }

extern void _rs_MiUnlockWorkingSet(PETHREAD Thread, PMMSUPPORT WorkingSet);
void MiUnlockWorkingSet(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _rs_MiUnlockWorkingSet(Thread, WorkingSet); }

extern void _rs_MiLockProcessWorkingSetShared(PEPROCESS Process, PETHREAD Thread);
void MiLockProcessWorkingSetShared(IN PEPROCESS Process, IN PETHREAD Thread)
{
#ifdef _M_ARM64
    {
        KIRQL irql = KeGetCurrentIrql();
        if (irql > APC_LEVEL) {
            MmUartPuts("[MM] BUG: MiLockProcessWorkingSetShared IRQL=");
            MmUartHex("", irql);
            MmUartHex("  caller=", (ULONG_PTR)__builtin_return_address(0));
            MmUartHex("  caller2=", (ULONG_PTR)__builtin_return_address(1));
            MmUartHex("  caller3=", (ULONG_PTR)__builtin_return_address(2));
        }
    }
#endif
    if (Thread->OwnsProcessWorkingSetExclusive) {
        DPRINT1("[WS-BUG] MiLockProcessWorkingSetShared: OwnsExcl=%u, Flags=0x%x, caller=%p\n"
                "  LastExclLocker=%p, LastExclUnlocker=%p\n"
                "  LastSafeLocker=%p, LastSafeUnlocker=%p\n",
                Thread->OwnsProcessWorkingSetExclusive,
                Thread->SameThreadApcFlags,
                __builtin_return_address(0),
                MmDbgLastExclLocker, MmDbgLastExclUnlocker,
                MmDbgLastSafeLocker, MmDbgLastSafeUnlocker);
    }
    _rs_MiLockProcessWorkingSetShared(Process, Thread);
}

extern void _rs_MiUnlockProcessWorkingSetShared(PEPROCESS Process, PETHREAD Thread);
void MiUnlockProcessWorkingSetShared(IN PEPROCESS Process, IN PETHREAD Thread)
{
    _rs_MiUnlockProcessWorkingSetShared(Process, Thread);
}

extern void _rs_MiLockWorkingSetShared(PETHREAD Thread, PMMSUPPORT WorkingSet);
void MiLockWorkingSetShared(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _rs_MiLockWorkingSetShared(Thread, WorkingSet); }

extern void _rs_MiUnlockWorkingSetShared(PETHREAD Thread, PMMSUPPORT WorkingSet);
void MiUnlockWorkingSetShared(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _rs_MiUnlockWorkingSetShared(Thread, WorkingSet); }

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

/* InterlockedIncrementUL — body delegates to Rust impl in
 * `mm-util-inlines` rlib (Phase 1 batch M3). */
#undef InterlockedIncrementUL
extern ULONG _rs_InterlockedIncrementUL(PULONG Addend);
ULONG NTAPI InterlockedIncrementUL(IN PULONG Addend)
{ return _rs_InterlockedIncrementUL(Addend); }

/* UpdateTotalCommittedPages / KeInvalidateTlbEntry trampolines REMOVED
 * — duplicated by ntoskrnl-rust::shim_exports `extern "C"` exports
 * (cdecl on all 3 arches). */

/* MmLockSectionSegment / MmUnlockSectionSegment / MmGetPageEntrySectionSegment
 * are now canonical in kernel_c_helpers.c (need __FILE__/__LINE__ macros). */

/* MmDereferenceSegment trampoline REMOVED — duplicated by shim_exports. */

/*
 * ExAllocatePoolZero — body delegates to Rust impl in `mm-util-inlines`
 * (Phase 1 batch M3). The Rust side calls ExAllocatePoolWithTag +
 * RtlZeroMemory just like the previous C implementation.
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

/*
 * InterlockedExchangeAddUL — body delegates to Rust impl in
 * `mm-util-inlines` (Phase 1 batch M3).
 */
extern ULONG _rs_InterlockedExchangeAddUL(PULONG Addend, ULONG Value);
ULONG
NTAPI
InterlockedExchangeAddUL(IN PULONG Addend, IN ULONG Value)
{
    return _rs_InterlockedExchangeAddUL(Addend, Value);
}

/*
 * RMAP_IS_SEGMENT — macro in mm.h. Body in Rust at
 * sdk/lib/rust/mm-macros/src/lib.rs::_rs_RMAP_IS_SEGMENT.
 */
extern BOOLEAN _rs_RMAP_IS_SEGMENT(PVOID Address);
BOOLEAN
NTAPI
RMAP_IS_SEGMENT(IN PVOID Address)
{
    return _rs_RMAP_IS_SEGMENT(Address);
}

/*
 * PAGE_ROUND_DOWN — macro in mmtypes.h. Body in Rust at
 * sdk/lib/rust/mm-macros/src/lib.rs::_rs_PAGE_ROUND_DOWN.
 */
extern ULONG_PTR _rs_PAGE_ROUND_DOWN(PVOID Address);
ULONG_PTR
NTAPI
PAGE_ROUND_DOWN(IN PVOID Address)
{
    return _rs_PAGE_ROUND_DOWN(Address);
}

/*
 * PFN_FROM_SSE — macro in mm.h. Body in Rust at
 * sdk/lib/rust/mm-macros/src/lib.rs::_rs_PFN_FROM_SSE.
 */
extern ULONG_PTR _rs_PFN_FROM_SSE(ULONG_PTR Entry);
PFN_NUMBER
NTAPI
PFN_FROM_SSE(IN ULONG_PTR Entry)
{
    return (PFN_NUMBER)_rs_PFN_FROM_SSE(Entry);
}

/*
 * MM_IS_WAIT_PTE — macro in mm.h. Body in Rust at
 * sdk/lib/rust/mm-macros/src/lib.rs::_rs_MM_IS_WAIT_PTE.
 */
extern BOOLEAN _rs_MM_IS_WAIT_PTE(ULONG_PTR Entry);
BOOLEAN
NTAPI
MM_IS_WAIT_PTE(IN ULONG_PTR Entry)
{
    return _rs_MM_IS_WAIT_PTE(Entry);
}

/*
 * MmSetDirtyPage — body delegates to Rust impl in `mm-util-inlines`
 * (Phase 1 batch M3).
 */
extern void _rs_MmSetDirtyPage(PEPROCESS Process, PVOID Address);
VOID
NTAPI
MmSetDirtyPage(IN PEPROCESS Process, IN PVOID Address)
{
    _rs_MmSetDirtyPage(Process, Address);
}

/* KeFlushProcessTb trampoline REMOVED — duplicated by shim_exports. */

/*
 * MmInitializeMdl — macro in xdk/mmfuncs.h (covered by PCH).
 * Body in Rust at sdk/lib/rust/mm-macros/src/lib.rs::_rs_MmInitializeMdl.
 */
extern void _rs_MmInitializeMdl(PMDL Mdl, PVOID BaseVa, ULONG Length);

void
NTAPI
MmInitializeMdl(
    IN PMDL Mdl,
    IN PVOID BaseVa,
    IN ULONG Length)
{
    _rs_MmInitializeMdl(Mdl, BaseVa, Length);
}

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

/* The 7 pure-delegation AVL fns (RtlRightChildAvl / RtlLeftChildAvl /
 * RtlParentAvl / RtlIsRightChildAvl / RtlIsLeftChildAvl /
 * RtlpFindAvlTableNodeOrParent / RtlpDeleteAvlTreeNode) are exported
 * directly by the mm-avl rlib (sdk/lib/rust/mm-avl/) under the bare
 * Rtl* / Rtlp* names — no C trampoline.
 *
 * RtlpInsertAvlTreeNode keeps a C trampoline for the duplicate-insertion
 * DPRINT1 guard; the Rust impl has only a debug_assert which is a no-op
 * in release builds. */
extern void NTAPI _rs_RtlpInsertAvlTreeNode(
    IN PMM_AVL_TABLE Table, IN PMMADDRESS_NODE NewNode,
    IN PMMADDRESS_NODE Parent, IN TABLE_SEARCH_RESULT SearchResult);

void
NTAPI
RtlpInsertAvlTreeNode(
    IN PMM_AVL_TABLE Table,
    IN PMMADDRESS_NODE NewNode,
    IN PMMADDRESS_NODE Parent,
    IN TABLE_SEARCH_RESULT SearchResult)
{
    /* Guard: duplicate insertion would corrupt the AVL tree and cause deadlocks.
     * This stays in C — the DPRINT1 with detailed VPN context is more useful
     * here than at the Rust layer (vadnode.rs callers pass typed pointers, but
     * the diagnostic value is in seeing the field values). */
    if (SearchResult == TableFoundNode)
    {
        DPRINT1("RtlpInsertAvlTreeNode: BLOCKED duplicate insertion! NewNode=%p Parent=%p "
                "VPN=[%lx-%lx] ParentVPN=[%lx-%lx]\n",
                NewNode, Parent,
                (ULONG)NewNode->StartingVpn, (ULONG)NewNode->EndingVpn,
                (ULONG)Parent->StartingVpn, (ULONG)Parent->EndingVpn);
        return;
    }
    _rs_RtlpInsertAvlTreeNode(Table, NewNode, Parent, SearchResult);
}

#if 0  /* Original C body — preserved for side-by-side debug. */
/* --- Simple FORCEINLINE helpers (miavl.h MiXxx equivalents) --- */

PMMADDRESS_NODE
NTAPI
RtlRightChildAvl_C(IN PMMADDRESS_NODE Node)
{
    return Node->RightChild;
}

PMMADDRESS_NODE
NTAPI
RtlLeftChildAvl_C(IN PMMADDRESS_NODE Node)
{
    return Node->LeftChild;
}

PMMADDRESS_NODE
NTAPI
RtlParentAvl_C(IN PMMADDRESS_NODE Node)
{
    return (PMMADDRESS_NODE)((ULONG_PTR)Node->u1.Parent & ~(ULONG_PTR)3);
}

BOOLEAN
NTAPI
RtlIsRightChildAvl_C(IN PMMADDRESS_NODE Node)
{
    return (RtlRightChildAvl(RtlParentAvl(Node)) == Node);
}

BOOLEAN
NTAPI
RtlIsLeftChildAvl_C(IN PMMADDRESS_NODE Node)
{
    return (RtlLeftChildAvl(RtlParentAvl(Node)) == Node);
}

/* --- Heavyweight AVL algorithms from avlsupp.c (Mi-flavored) ---
 *
 * Include miavl.h to get the FORCEINLINE helpers (MiSetParent, MiSetBalance,
 * MiBalance, MiInsertAsLeftChildAvl, etc.) and then #include avlsupp.c which
 * compiles the algorithms using those helpers.
 *
 * miavl.h renames Rtlp* → Mi* via #define, so avlsupp.c produces:
 *   MiFindAvlTableNodeOrParent, MiPromoteAvlTreeNode,
 *   MiRebalanceAvlTreeNode, MiInsertAvlTreeNode, MiDeleteAvlTreeNode
 *
 * After the include, we #undef and write thin Rtlp* wrappers.
 */
#include "ARM3/miavl.h"
#include <sdk/lib/rtl/avlsupp.c>

/* (#undef block + Rtlp* trampoline wrappers — ~50 lines elided in this debug
 * snapshot, see git history before commit b20f8cf1 for the full original.) */
#endif /* 0 — AVL bodies now in mm-avl rlib */

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

BOOLEAN MiConvertSharedWorkingSetLockToExclusive(PETHREAD Thread, PMMSUPPORT Vm)
{ return _orig_MiConvertSharedWorkingSetLockToExclusive(Thread, Vm); }

/* KiAcquireProcessLockRaiseToSynch trampoline REMOVED — duplicated by
 * shim_exports.rs.
 *
 * KiReleaseProcessLock keeps its C trampoline — shim_exports.rs only
 * has the FromSynchLevel variant; the bindgen-generated cdecl decl
 * for plain KiReleaseProcessLock has no Rust definition yet. */
#ifdef _X86_
VOID FASTCALL KeReleaseInStackQueuedSpinLock(IN PKLOCK_QUEUE_HANDLE LockHandle);
#endif

void KiReleaseProcessLock(IN PKLOCK_QUEUE_HANDLE Handle)
{
    KeReleaseInStackQueuedSpinLock(Handle);
}

/* ===================================================================
 * Tier 3b FORCEINLINE shims (ARM3/section.c, ARM3/virtual.c)
 * =================================================================== */

extern void _rs_MiLockProcessWorkingSetForFault(PEPROCESS Process, PETHREAD Thread,
                                                 BOOLEAN Safe, BOOLEAN Shared);
void MiLockProcessWorkingSetForFault(IN PEPROCESS Process,
                                      IN PETHREAD Thread,
                                      IN BOOLEAN Safe,
                                      IN BOOLEAN Shared)
{
    _rs_MiLockProcessWorkingSetForFault(Process, Thread, Safe, Shared);
}

extern void _rs_MiUnlockProcessWorkingSetForFault(PEPROCESS Process, PETHREAD Thread,
                                                   PBOOLEAN Safe, PBOOLEAN Shared);
void MiUnlockProcessWorkingSetForFault(IN PEPROCESS Process,
                                        IN PETHREAD Thread,
                                        OUT PBOOLEAN Safe,
                                        OUT PBOOLEAN Shared)
{
    _rs_MiUnlockProcessWorkingSetForFault(Process, Thread, Safe, Shared);
}

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

/* =====================================================================
 * MmFixupImageSectionSegments — called from Rust ExeFmtpCreateImageSection
 * to sort, bounds-check, page-align, trim, and initialize image section
 * segments. This entire block was missing from the Rust translation of
 * section.c, causing uninitialized FAST_MUTEX in segment locks.
 * ===================================================================== */

#include <reactos/exeformat.h>

/* ========================================================================
 * MmFixupImageSectionSegments and its 4 helpers — translated to Rust 2026-05-06.
 *
 * Direct port to `mm/section.rs::{MmspSortSegments, MmspCheckSegmentBounds,
 * MmspPageAlignSegments, MmFixupImageSectionSegments}`. Pure-logic image
 * segment fixup: sort by VA, validate bounds, page-align with merge, trim,
 * initialize per-segment locks/refcount/flags/pagetable/FileObject.
 *
 * The original C body is preserved below in `#if 0` so future debug sessions
 * can compare the two side-by-side without a git checkout.
 * ======================================================================== */

#if 0
static
int
__cdecl
MmspCompareSegments_Shim(const void * x, const void * y)
{
    const MM_SECTION_SEGMENT *Segment1 = (const MM_SECTION_SEGMENT *)x;
    const MM_SECTION_SEGMENT *Segment2 = (const MM_SECTION_SEGMENT *)y;

    if (Segment1->Image.VirtualAddress > Segment2->Image.VirtualAddress)
        return 1;
    else if (Segment1->Image.VirtualAddress < Segment2->Image.VirtualAddress)
        return -1;
    else
        return 0;
}

static
VOID
NTAPI
MmspSortSegments_Shim(IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
                       IN ULONG Flags)
{
    if (!(Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_SORTED))
    {
        qsort(ImageSectionObject->Segments,
              ImageSectionObject->NrSegments,
              sizeof(ImageSectionObject->Segments[0]),
              MmspCompareSegments_Shim);
    }
}

static
BOOLEAN
NTAPI
MmspCheckSegmentBounds_Shim(
    IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    IN ULONG Flags)
{
    ULONG i;

    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_NO_OVERLAP)
        return TRUE;

    ASSERT(ImageSectionObject->NrSegments >= 1);

    for (i = 0; i < ImageSectionObject->NrSegments; ++i)
    {
        if (ImageSectionObject->Segments[i].Length.QuadPart == 0)
            return FALSE;

        if (i > 0)
        {
            if ((ImageSectionObject->Segments[i - 1].Image.VirtualAddress +
                    ImageSectionObject->Segments[i - 1].Length.QuadPart) !=
                    ImageSectionObject->Segments[i].Image.VirtualAddress)
            {
                return FALSE;
            }
        }
    }

    return TRUE;
}

static
BOOLEAN
NTAPI
MmspPageAlignSegments_Shim(
    IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    IN ULONG Flags)
{
    ULONG i;
    ULONG LastSegment;
    PMM_SECTION_SEGMENT EffectiveSegment;

    if (Flags & EXEFMT_LOAD_ASSUME_SEGMENTS_PAGE_ALIGNED)
        return TRUE;

    LastSegment = 0;
    EffectiveSegment = &ImageSectionObject->Segments[LastSegment];

    for (i = 0; i < ImageSectionObject->NrSegments; ++i)
    {
        if (i == 0)
        {
            ULONG_PTR VirtualAddress;
            ULONG_PTR VirtualOffset;

            VirtualAddress = EffectiveSegment->Image.VirtualAddress;
            EffectiveSegment->Image.VirtualAddress = (ULONG_PTR)PAGE_ROUND_DOWN((PVOID)VirtualAddress);
            EffectiveSegment->Length.QuadPart = PAGE_ROUND_UP(VirtualAddress + EffectiveSegment->Length.QuadPart) -
                                                EffectiveSegment->Image.VirtualAddress;

            VirtualOffset = VirtualAddress - EffectiveSegment->Image.VirtualAddress;

            if (EffectiveSegment->Image.FileOffset < VirtualOffset)
                return FALSE;

            EffectiveSegment->Image.FileOffset -= VirtualOffset;
            EffectiveSegment->RawLength.QuadPart += VirtualOffset;
        }
        else
        {
            PMM_SECTION_SEGMENT Segment = &ImageSectionObject->Segments[i];
            ULONG_PTR EndOfEffectiveSegment;

            EndOfEffectiveSegment = (ULONG_PTR)(EffectiveSegment->Image.VirtualAddress + EffectiveSegment->Length.QuadPart);
            ASSERT((EndOfEffectiveSegment % PAGE_SIZE) == 0);

            if (EndOfEffectiveSegment == Segment->Image.VirtualAddress)
            {
                LastSegment++;
                ASSERT(LastSegment <= i);
                ASSERT(LastSegment < ImageSectionObject->NrSegments);

                EffectiveSegment = &ImageSectionObject->Segments[LastSegment];

                if (LastSegment != i)
                    *EffectiveSegment = *Segment;

                ASSERT((EffectiveSegment->Image.VirtualAddress % PAGE_SIZE) == 0);
                EffectiveSegment->Length.QuadPart = PAGE_ROUND_UP(EffectiveSegment->Length.QuadPart);
            }
            else if (EndOfEffectiveSegment > Segment->Image.VirtualAddress)
            {
                static const ULONG FlagsToProtection[16] =
                {
                    PAGE_NOACCESS,
                    PAGE_READONLY,
                    PAGE_READWRITE,
                    PAGE_READWRITE,
                    PAGE_EXECUTE_READ,
                    PAGE_EXECUTE_READ,
                    PAGE_EXECUTE_READWRITE,
                    PAGE_EXECUTE_READWRITE,
                    PAGE_WRITECOPY,
                    PAGE_WRITECOPY,
                    PAGE_WRITECOPY,
                    PAGE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY,
                    PAGE_EXECUTE_WRITECOPY
                };

                unsigned ProtectionFlags;

                if (Segment->Image.FileOffset != (EffectiveSegment->Image.FileOffset +
                                                  EffectiveSegment->RawLength.QuadPart))
                {
                    return FALSE;
                }

                EffectiveSegment->RawLength.QuadPart += Segment->RawLength.QuadPart;

                ASSERT(PAGE_ROUND_UP(Segment->Image.VirtualAddress + Segment->Length.QuadPart) >= EndOfEffectiveSegment);

                EffectiveSegment->Length.QuadPart = PAGE_ROUND_UP(Segment->Image.VirtualAddress + Segment->Length.QuadPart) -
                                                    EffectiveSegment->Image.VirtualAddress;

                EffectiveSegment->Protection |= Segment->Protection;

                ProtectionFlags = 0;

                if (EffectiveSegment->Protection & PAGE_IS_READABLE)
                    ProtectionFlags |= 1 << 0;

                if (EffectiveSegment->Protection & PAGE_IS_WRITABLE)
                    ProtectionFlags |= 1 << 1;

                if (EffectiveSegment->Protection & PAGE_IS_EXECUTABLE)
                    ProtectionFlags |= 1 << 2;

                if (EffectiveSegment->Protection & PAGE_IS_WRITECOPY)
                    ProtectionFlags |= 1 << 3;

                ASSERT(ProtectionFlags < 16);
                EffectiveSegment->Protection = FlagsToProtection[ProtectionFlags];

                if (!(Segment->Protection & PAGE_IS_WRITECOPY) &&
                        EffectiveSegment->Protection & PAGE_IS_WRITECOPY)
                {
                    return FALSE;
                }
            }
            else
            {
                KeBugCheck(MEMORY_MANAGEMENT);
            }
        }
    }
    ImageSectionObject->NrSegments = LastSegment + 1;

    return TRUE;
}

NTSTATUS
NTAPI
MmFixupImageSectionSegments(
    IN OUT PMM_IMAGE_SECTION_OBJECT ImageSectionObject,
    IN ULONG Flags,
    IN PFILE_OBJECT FileObject)
{
    ULONG i;
    ULONG OldNrSegments;

    /* Sort them by virtual address */
    MmspSortSegments_Shim(ImageSectionObject, Flags);

    /* Ensure they don't overlap in memory */
    if (!MmspCheckSegmentBounds_Shim(ImageSectionObject, Flags))
        return STATUS_INVALID_IMAGE_FORMAT;

    /* Ensure they are aligned */
    OldNrSegments = ImageSectionObject->NrSegments;

    if (!MmspPageAlignSegments_Shim(ImageSectionObject, Flags))
        return STATUS_INVALID_IMAGE_FORMAT;

    /* Trim them if the alignment phase merged some of them */
    if (ImageSectionObject->NrSegments < OldNrSegments)
    {
        PMM_SECTION_SEGMENT Segments;
        SIZE_T SizeOfSegments;

        SizeOfSegments = sizeof(MM_SECTION_SEGMENT) * ImageSectionObject->NrSegments;

        Segments = ExAllocatePoolWithTag(PagedPool,
                                         SizeOfSegments,
                                         TAG_MM_SECTION_SEGMENT);

        if (Segments == NULL)
            return STATUS_INSUFFICIENT_RESOURCES;

        RtlCopyMemory(Segments, ImageSectionObject->Segments, SizeOfSegments);
        ExFreePool(ImageSectionObject->Segments);
        ImageSectionObject->Segments = Segments;
    }

    DPRINT("MmFixupImageSectionSegments: Flags=0x%x, OldNrSegments=%u, NrSegments=%u\n",
           Flags, OldNrSegments, ImageSectionObject->NrSegments);

    /* And finish their initialization */
    for (i = 0; i < ImageSectionObject->NrSegments; ++i)
    {
        DPRINT("  Segment[%u]: VA=0x%lx Len=0x%llx RawLen=0x%llx FileOff=0x%llx Prot=0x%x\n",
               i,
               (ULONG)ImageSectionObject->Segments[i].Image.VirtualAddress,
               ImageSectionObject->Segments[i].Length.QuadPart,
               ImageSectionObject->Segments[i].RawLength.QuadPart,
               (ULONGLONG)ImageSectionObject->Segments[i].Image.FileOffset,
               ImageSectionObject->Segments[i].Protection);
        ExInitializeFastMutex(&ImageSectionObject->Segments[i].Lock);
        ImageSectionObject->Segments[i].ReferenceCount = &ImageSectionObject->RefCount;
        ImageSectionObject->Segments[i].Flags = &ImageSectionObject->SegFlags;
        MiInitializeSectionPageTable(&ImageSectionObject->Segments[i]);
        ImageSectionObject->Segments[i].FileObject = FileObject;
    }

    ASSERT(ImageSectionObject->RefCount > 0);

    ImageSectionObject->FileObject = FileObject;

    return STATUS_SUCCESS;
}
#endif /* 0 — MmFixupImageSectionSegments + helpers now in mm/section.rs */

/* Debug trace helper callable from Rust (stdcall on i386) */
VOID NTAPI MmDebugTrace(ULONG Code, ULONG_PTR Param1, ULONG_PTR Param2)
{
    DPRINT1("MmDebugTrace: code=%lu p1=0x%lx p2=0x%lx IRQL=%u\n",
            Code, Param1, Param2, (ULONG)KeGetCurrentIrql());

    /* When code==100, Param1 is FileObject — dump resource state */
    if (Code == 100 && Param1 != 0)
    {
        PFILE_OBJECT fo = (PFILE_OBJECT)Param1;
        PFSRTL_COMMON_FCB_HEADER fcb = (PFSRTL_COMMON_FCB_HEADER)(fo->FsContext);
        if (fcb && fcb->Resource) {
            PERESOURCE res = fcb->Resource;
            DPRINT1("  FileObj=%p FsContext=%p Resource=%p ActiveEntries=%d "
                    "NumberOfExclusiveWaiters=%u NumberOfSharedWaiters=%u "
                    "OwnerEntry.OwnerThread=%p Flag=0x%x\n",
                    fo, fcb, res, res->ActiveEntries,
                    res->NumberOfExclusiveWaiters, res->NumberOfSharedWaiters,
                    (PVOID)res->OwnerEntry.OwnerThread, res->Flag);
        } else {
            DPRINT1("  FileObj=%p FsContext=%p Resource=%p (NULL resource)\n",
                    fo, fcb, fcb ? fcb->Resource : NULL);
        }
    }
}

/* Removed: Temporary C NtCreateSection — Rust version re-enabled */
#if 0
NTSTATUS
NTAPI
NtCreateSection_removed(OUT PHANDLE SectionHandle,
                IN ACCESS_MASK DesiredAccess,
                IN POBJECT_ATTRIBUTES ObjectAttributes OPTIONAL,
                IN PLARGE_INTEGER MaximumSize OPTIONAL,
                IN ULONG SectionPageProtection OPTIONAL,
                IN ULONG AllocationAttributes,
                IN HANDLE FileHandle OPTIONAL)
{
    LARGE_INTEGER SafeMaximumSize;
    PVOID SectionObject;
    HANDLE Handle;
    KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
    NTSTATUS Status;
    PAGED_CODE();

    DbgPrint("C-NtCreateSection: ENTER SH=%p DA=0x%lx OA=%p FH=%p Alloc=0x%lx\n",
             SectionHandle, DesiredAccess, ObjectAttributes, FileHandle, AllocationAttributes);

    /* Check for non-existing flags */
    if ((AllocationAttributes & ~(SEC_COMMIT | SEC_RESERVE | SEC_BASED |
                                  SEC_LARGE_PAGES | SEC_IMAGE | SEC_NOCACHE |
                                  SEC_NO_CHANGE)))
    {
        if (!(AllocationAttributes & 1))
        {
            return STATUS_INVALID_PARAMETER_6;
        }
    }

    /* Check for no allocation type */
    if (!(AllocationAttributes & (SEC_COMMIT | SEC_RESERVE | SEC_IMAGE)))
    {
        return STATUS_INVALID_PARAMETER_6;
    }

    /* Check for image allocation with invalid attributes */
    if ((AllocationAttributes & SEC_IMAGE) &&
        (AllocationAttributes & (SEC_COMMIT | SEC_RESERVE | SEC_LARGE_PAGES |
                                 SEC_NOCACHE | SEC_NO_CHANGE)))
    {
        return STATUS_INVALID_PARAMETER_6;
    }

    /* Check for allocation type is both commit and reserve */
    if ((AllocationAttributes & SEC_COMMIT) && (AllocationAttributes & SEC_RESERVE))
    {
        return STATUS_INVALID_PARAMETER_6;
    }

    /* Now check for valid protection */
    if ((SectionPageProtection & PAGE_NOCACHE) ||
        (SectionPageProtection & PAGE_WRITECOMBINE) ||
        (SectionPageProtection & PAGE_GUARD) ||
        (SectionPageProtection & PAGE_NOACCESS))
    {
        return STATUS_INVALID_PAGE_PROTECTION;
    }

    /* Use a maximum size of zero, if none was specified */
    SafeMaximumSize.QuadPart = 0;

    /* Check for user-mode caller */
    if (PreviousMode != KernelMode)
    {
        /* Enter SEH */
        _SEH2_TRY
        {
            /* Safely check user-mode parameters */
            if (MaximumSize) SafeMaximumSize = ProbeForReadLargeInteger(MaximumSize);
            MaximumSize = &SafeMaximumSize;
            ProbeForWriteHandle(SectionHandle);
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            /* Return the exception code */
            _SEH2_YIELD(return _SEH2_GetExceptionCode());
        }
        _SEH2_END;
    }
    else if (!MaximumSize) MaximumSize = &SafeMaximumSize;

    /* Check that MaximumSize is valid if backed by paging file */
    if ((!FileHandle) && (!MaximumSize->QuadPart))
        return STATUS_INVALID_PARAMETER_4;

    /* Create the section */
    DbgPrint("C-NtCreateSection: calling MmCreateSection FH=%p\n", FileHandle);
    Status = MmCreateSection(&SectionObject,
                             DesiredAccess,
                             ObjectAttributes,
                             MaximumSize,
                             SectionPageProtection,
                             AllocationAttributes,
                             FileHandle,
                             NULL);
    if (!NT_SUCCESS(Status)) return Status;

    /* Now insert the object */
    Status = ObInsertObject(SectionObject,
                            NULL,
                            DesiredAccess,
                            0,
                            NULL,
                            &Handle);
    if (NT_SUCCESS(Status))
    {
        /* Enter SEH */
        _SEH2_TRY
        {
            /* Return the handle safely */
            *SectionHandle = Handle;
        }
        _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
        {
            /* Nothing here */
        }
        _SEH2_END;
    }

    return Status;
}
#endif

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

/* MiAcquirePfnLock / MiReleasePfnLock — FORCEINLINE wrappers in mm.h
 * (KeAcquireQueuedSpinLock/KeReleaseQueuedSpinLock with LockQueuePfnLock).
 * Phase 1 batch M7 — bodies delegate to Rust impls in `mm-lock-inlines`. */
extern KIRQL _rs_MiAcquirePfnLock(VOID);
KIRQL MiAcquirePfnLock(VOID)
{ return _rs_MiAcquirePfnLock(); }

extern VOID _rs_MiReleasePfnLock(KIRQL OldIrql);
VOID MiReleasePfnLock(KIRQL OldIrql)
{ _rs_MiReleasePfnLock(OldIrql); }
