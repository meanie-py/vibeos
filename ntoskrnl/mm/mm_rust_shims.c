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
#ifndef NEWCC
KEVENT MmWaitPageEvent;
#endif
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
 * Hardware PTE creation
 */
void
NTAPI
MI_MAKE_HARDWARE_PTE_KERNEL(
    IN PMMPTE NewPte,
    IN PMMPTE MappingPte,
    IN ULONG_PTR ProtectionMask,
    IN PFN_NUMBER PageFrameNumber)
{
    _orig_MI_MAKE_HARDWARE_PTE_KERNEL(NewPte, MappingPte, ProtectionMask, PageFrameNumber);
}

void
NTAPI
MI_MAKE_HARDWARE_PTE_USER(
    IN PMMPTE NewPte,
    IN PMMPTE MappingPte,
    IN ULONG_PTR ProtectionMask,
    IN PFN_NUMBER PageFrameNumber)
{
    _orig_MI_MAKE_HARDWARE_PTE_USER(NewPte, MappingPte, ProtectionMask, PageFrameNumber);
}

void
NTAPI
MI_MAKE_HARDWARE_PTE(
    IN PMMPTE NewPte,
    IN PMMPTE MappingPte,
    IN ULONG_PTR ProtectionMask,
    IN PFN_NUMBER PageFrameNumber)
{
    _orig_MI_MAKE_HARDWARE_PTE(NewPte, MappingPte, ProtectionMask, PageFrameNumber);
}

/*
 * PTE update (volatile write with assertion checks)
 */
void
NTAPI
MI_UPDATE_VALID_PTE(
    IN PMMPTE PointerPte,
    IN MMPTE TempPte)
{
    _orig_MI_UPDATE_VALID_PTE(PointerPte, TempPte);
}

/*
 * Working set locks
 * Bindgen says extern "C" (FORCEINLINE with no calling convention) — no NTAPI.
 */
void MiLockProcessWorkingSet(IN PEPROCESS Process, IN PETHREAD Thread)
{ _orig_MiLockProcessWorkingSet(Process, Thread); }

void MiUnlockProcessWorkingSet(IN PEPROCESS Process, IN PETHREAD Thread)
{ _orig_MiUnlockProcessWorkingSet(Process, Thread); }

void MiLockProcessWorkingSetUnsafe(IN PEPROCESS Process, IN PETHREAD Thread)
{ _orig_MiLockProcessWorkingSetUnsafe(Process, Thread); }

void MiUnlockProcessWorkingSetUnsafe(IN PEPROCESS Process, IN PETHREAD Thread)
{ _orig_MiUnlockProcessWorkingSetUnsafe(Process, Thread); }

void MiLockWorkingSet(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _orig_MiLockWorkingSet(Thread, WorkingSet); }

void MiUnlockWorkingSet(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _orig_MiUnlockWorkingSet(Thread, WorkingSet); }

void MiLockProcessWorkingSetShared(IN PEPROCESS Process, IN PETHREAD Thread)
{ _orig_MiLockProcessWorkingSetShared(Process, Thread); }

void MiUnlockProcessWorkingSetShared(IN PEPROCESS Process, IN PETHREAD Thread)
{ _orig_MiUnlockProcessWorkingSetShared(Process, Thread); }

void MiLockWorkingSetShared(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _orig_MiLockWorkingSetShared(Thread, WorkingSet); }

void MiUnlockWorkingSetShared(IN PETHREAD Thread, IN PMMSUPPORT WorkingSet)
{ _orig_MiUnlockWorkingSetShared(Thread, WorkingSet); }

/*
 * Expansion lock
 * Bindgen says extern "C" (FORCEINLINE with no calling convention) — no NTAPI.
 */
KIRQL MiAcquireExpansionLock(VOID)
{ return _orig_MiAcquireExpansionLock(); }

void MiReleaseExpansionLock(IN KIRQL OldIrql)
{ _orig_MiReleaseExpansionLock(OldIrql); }

/*
 * PTE mapping check — arch-specific, accesses hardware PTE bitfields
 * Bindgen says extern "C" — no NTAPI.
 */
BOOLEAN MiIsMappedPte(IN PMMPTE PointerPte)
{ return MI_IS_MAPPED_PTE(PointerPte); }

/*
 * User/global PTE mask determination
 * Bindgen says extern "C" — no NTAPI.
 */
ULONG_PTR MiDetermineUserGlobalPteMask(IN PMMPTE PointerPte)
{ return _orig_MiDetermineUserGlobalPteMask(PointerPte); }

/*
 * Working set lock check (FORCEINLINE in miarm.h — no NTAPI, no calling convention)
 */
BOOLEAN MM_ANY_WS_LOCK_HELD(IN PETHREAD Thread)
{ return _orig_MM_ANY_WS_LOCK_HELD(Thread); }

/*
 * PDE hierarchy validity check (4-level paging on amd64/arm64)
 * No NTAPI — bindgen would say extern "C" for FORCEINLINE.
 */
#if defined(_M_AMD64) || defined(_M_ARM64)
BOOLEAN MiIsPdeForAddressValid(IN PVOID Address)
{
    return ((MiAddressToPxe(Address)->u.Hard.Valid) &&
            (MiAddressToPpe(Address)->u.Hard.Valid) &&
            (MiAddressToPde(Address)->u.Hard.Valid));
}
#endif

/*
 * PTE portable macros — access hardware PTE bitfields that differ per arch.
 * These wrap C macros (no calling convention) — no NTAPI.
 */
void MiMakeDirtyPage(IN PMMPTE Pte)    { MI_MAKE_DIRTY_PAGE(Pte); }
void MiMakeCleanPage(IN PMMPTE Pte)    { MI_MAKE_CLEAN_PAGE(Pte); }
void MiMakeAccessedPage(IN PMMPTE Pte) { MI_MAKE_ACCESSED_PAGE(Pte); }
void MiPageDisableCache(IN PMMPTE Pte) { MI_PAGE_DISABLE_CACHE(Pte); }
void MiPageWriteThrough(IN PMMPTE Pte) { MI_PAGE_WRITE_THROUGH(Pte); }
void MiPageWriteCombined(IN PMMPTE Pte){ MI_PAGE_WRITE_COMBINED(Pte); }
BOOLEAN MiIsPageWriteable(IN PMMPTE Pte) { return MI_IS_PAGE_WRITEABLE(Pte); }
BOOLEAN MiIsPageDirty(IN PMMPTE Pte)     { return MI_IS_PAGE_DIRTY(Pte); }
void MiMakeOwnerPage(IN PMMPTE Pte)    { MI_MAKE_OWNER_PAGE(Pte); }
void MiMakeWritePage(IN PMMPTE Pte)    { MI_MAKE_WRITE_PAGE(Pte); }

/*
 * InterlockedCompareExchangePte / InterlockedExchangePte — arch-specific PTE atomic update.
 * Not NTAPI, no calling convention in C — no NTAPI.
 */
LONG_PTR MiInterlockedCompareExchangePte(
    IN volatile PMMPTE PointerPte, IN LONG_PTR Exchange, IN LONG_PTR Comperand)
{ return InterlockedCompareExchangePte(PointerPte, Exchange, Comperand); }

LONG_PTR MiInterlockedExchangePte(
    IN volatile PMMPTE PointerPte, IN LONG_PTR Value)
{ return InterlockedExchangePte(PointerPte, Value); }

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
#if defined(_M_IX86)
#undef InterlockedExchange
#undef InterlockedCompareExchange

LONG InterlockedExchange(volatile LONG *Target, LONG Value)
{ return _InterlockedExchange(Target, Value); }

LONG InterlockedCompareExchange(
    volatile LONG *Destination, LONG Exchange, LONG Comparand)
{ return _InterlockedCompareExchange(Destination, Exchange, Comparand); }
#endif /* _M_IX86 */

/* InterlockedDecrementUL — already in ps_rust_shims.c */
/* InterlockedIncrementUL — not in any other shim file, provide here */
#undef InterlockedIncrementUL
ULONG NTAPI InterlockedIncrementUL(IN PULONG Addend)
{ return (ULONG)_InterlockedIncrement((volatile LONG *)Addend); }

/*
 * UpdateTotalCommittedPages — FORCEINLINE in mm.h (covered by PCH, can't rename)
 * Manually inline the implementation.
 */
void UpdateTotalCommittedPages(IN LONG Delta)
{
    SIZE_T TotalCommittedPages = InterlockedExchangeAddSizeT(&MmTotalCommittedPages, Delta) + Delta;
    SIZE_T PeakCommitment = MmPeakCommitment;
    while (TotalCommittedPages > PeakCommitment &&
           InterlockedCompareExchangeSizeT(&MmPeakCommitment, TotalCommittedPages, PeakCommitment) != PeakCommitment)
    {
        PeakCommitment = MmPeakCommitment;
    }
}

/*
 * KeInvalidateTlbEntry — FORCEINLINE per-arch in ke.h (covered by PCH, can't rename)
 * Manually inline the arch-specific implementation.
 */
void KeInvalidateTlbEntry(IN PVOID Address)
{
#if defined(_M_AMD64) || defined(_M_IX86)
    __invlpg(Address);
#elif defined(_M_ARM64)
    ULONG64 TlbiArg = (ULONG64)Address >> 12;
    __asm__ __volatile__("tlbi vale1is, %0" : : "r"(TlbiArg) : "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
#else
#error "Unsupported architecture"
#endif
}

/*
 * Section segment lock/unlock — macros wrapping underscored versions
 */
void NTAPI MmLockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmLockSectionSegment(Segment, __FILE__, __LINE__); }

void NTAPI MmUnlockSectionSegment(IN PMM_SECTION_SEGMENT Segment)
{ _MmUnlockSectionSegment(Segment, __FILE__, __LINE__); }

ULONG_PTR NTAPI MmGetPageEntrySectionSegment(
    IN PMM_SECTION_SEGMENT Segment, IN PLARGE_INTEGER Offset)
{ return _MmGetPageEntrySectionSegment(Segment, Offset, __FILE__, __LINE__); }

/*
 * MmDereferenceSegment — FORCEINLINE in mm.h (covered by PCH, can't rename)
 * Calls MmDereferenceSegmentWithLock with MM_NOIRQL.
 */
void MmDereferenceSegment(IN PMM_SECTION_SEGMENT Segment)
{ MmDereferenceSegmentWithLock(Segment, MM_NOIRQL); }

/*
 * ExAllocatePoolZero — FORCEINLINE in exfuncs.h (covered by PCH, can't rename)
 * Calls ExAllocatePoolWithTag + RtlZeroMemory.
 */
PVOID
NTAPI
ExAllocatePoolZero(
    IN POOL_TYPE PoolType,
    IN SIZE_T NumberOfBytes,
    IN ULONG Tag)
{
    PVOID Allocation = ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
    if (Allocation != NULL) {
        RtlZeroMemory(Allocation, NumberOfBytes);
    }
    return Allocation;
}

/*
 * MmGetAddressSpaceOwner — FORCEINLINE in mm.h (covered by PCH, can't rename)
 */
PEPROCESS MmGetAddressSpaceOwner(IN PMMSUPPORT AddressSpace)
{
    if (AddressSpace == MmKernelAddressSpace) return NULL;
    return CONTAINING_RECORD(AddressSpace, EPROCESS, Vm);
}

/*
 * MmGetKernelAddressSpace — FORCEINLINE in mm.h (covered by PCH, can't rename)
 */
PMMSUPPORT MmGetKernelAddressSpace(VOID)
{
    return MmKernelAddressSpace;
}

/*
 * MmLockAddressSpace — FORCEINLINE in mm.h (covered by PCH, can't rename)
 */
void MmLockAddressSpace(IN PMMSUPPORT AddressSpace)
{
    KeAcquireGuardedMutex(&CONTAINING_RECORD(AddressSpace, EPROCESS, Vm)->AddressCreationLock);
}

/*
 * MmUnlockAddressSpace — FORCEINLINE in mm.h (covered by PCH, can't rename)
 */
void MmUnlockAddressSpace(IN PMMSUPPORT AddressSpace)
{
    KeReleaseGuardedMutex(&CONTAINING_RECORD(AddressSpace, EPROCESS, Vm)->AddressCreationLock);
}

/*
 * InterlockedExchangeAddUL — macro in ex.h (undeffed in Phase 3)
 * Wraps InterlockedExchangeAdd.
 */
ULONG
NTAPI
InterlockedExchangeAddUL(IN PULONG Addend, IN ULONG Value)
{
    return (ULONG)InterlockedExchangeAdd((volatile LONG *)Addend, (LONG)Value);
}

/*
 * RMAP_IS_SEGMENT — macro in mm.h (undeffed in Phase 3)
 */
BOOLEAN
NTAPI
RMAP_IS_SEGMENT(IN PVOID Address)
{
    return (((ULONG_PTR)(Address) & RMAP_SEGMENT_MASK) == RMAP_SEGMENT_MASK);
}

/*
 * PAGE_ROUND_DOWN — macro in mmtypes.h (undeffed in Phase 3)
 */
ULONG_PTR
NTAPI
PAGE_ROUND_DOWN(IN PVOID Address)
{
    return (((ULONG_PTR)(Address)) & ~((ULONG_PTR)(PAGE_SIZE - 1)));
}

/*
 * PFN_FROM_SSE — macro in mm.h (undeffed in Phase 3)
 */
PFN_NUMBER
NTAPI
PFN_FROM_SSE(IN ULONG_PTR Entry)
{
    return (PFN_NUMBER)(Entry >> PAGE_SHIFT);
}

/*
 * MM_IS_WAIT_PTE — macro in mm.h (undeffed in Phase 3)
 */
BOOLEAN
NTAPI
MM_IS_WAIT_PTE(IN ULONG_PTR Entry)
{
    return ((Entry & 0x1) && ((Entry >> 1) == MM_WAIT_ENTRY));
}

/*
 * MmSetDirtyPage — macro in mm.h: MmSetDirtyBit(P, A, TRUE)
 */
VOID
NTAPI
MmSetDirtyPage(IN PEPROCESS Process, IN PVOID Address)
{
    MmSetDirtyBit(Process, Address, TRUE);
}

/*
 * KeFlushProcessTb — FORCEINLINE per-arch in ke.h (covered by PCH, can't rename)
 * Manually inline the arch-specific TLB flush.
 */
void KeFlushProcessTb(VOID)
{
#if defined(_M_AMD64) || defined(_M_IX86)
    __writecr3(__readcr3());
#elif defined(_M_ARM64)
    __asm__ __volatile__("tlbi vmalle1is" ::: "memory");
    __asm__ __volatile__("dsb ish" ::: "memory");
    __asm__ __volatile__("isb" ::: "memory");
#else
#error "Unsupported architecture"
#endif
}

/*
 * MmInitializeMdl — macro in xdk/mmfuncs.h (covered by PCH).
 * The Rust side declares this as extern "system" (stdcall on i386).
 */
void
NTAPI
MmInitializeMdl(
    IN PMDL Mdl,
    IN PVOID BaseVa,
    IN ULONG Length)
{
    Mdl->Next = (PMDL)NULL;
    Mdl->Size = (CSHORT)(sizeof(MDL) +
        (sizeof(PFN_NUMBER) * ADDRESS_AND_SIZE_TO_SPAN_PAGES(BaseVa, Length)));
    Mdl->MdlFlags = 0;
    Mdl->StartVa = (PVOID)PAGE_ALIGN(BaseVa);
    Mdl->ByteOffset = BYTE_OFFSET(BaseVa);
    Mdl->ByteCount = Length;
}

/* ===================================================================
 * AVL Tree support for vadnode.rs
 *
 * vadnode.c used to #include "miavl.h" + avlsupp.c to get MM-flavored
 * AVL routines (Mi* names).  The Rust vadnode.rs calls the Rtl* / Rtlp*
 * names from extern "system".  We provide outline wrappers here.
 * =================================================================== */

/* --- Simple FORCEINLINE helpers (miavl.h MiXxx equivalents) --- */

PMMADDRESS_NODE
NTAPI
RtlRightChildAvl(IN PMMADDRESS_NODE Node)
{
    return Node->RightChild;
}

PMMADDRESS_NODE
NTAPI
RtlLeftChildAvl(IN PMMADDRESS_NODE Node)
{
    return Node->LeftChild;
}

PMMADDRESS_NODE
NTAPI
RtlParentAvl(IN PMMADDRESS_NODE Node)
{
    return (PMMADDRESS_NODE)((ULONG_PTR)Node->u1.Parent & ~(ULONG_PTR)3);
}

BOOLEAN
NTAPI
RtlIsRightChildAvl(IN PMMADDRESS_NODE Node)
{
    return (RtlRightChildAvl(RtlParentAvl(Node)) == Node);
}

BOOLEAN
NTAPI
RtlIsLeftChildAvl(IN PMMADDRESS_NODE Node)
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

/* Undef miavl.h renames so we can define our own Rtlp* symbols */
#undef RtlpFindAvlTableNodeOrParent
#undef RtlpPromoteAvlTreeNode
#undef RtlpRebalanceAvlTreeNode
#undef RtlpInsertAvlTreeNode
#undef RtlpDeleteAvlTreeNode
#undef RtlpCopyAvlNodeData
#undef RtlpAvlCompareRoutine
#undef RtlSetParent
#undef RtlSetBalance
#undef RtlBalance
#undef RtlRightChildAvl
#undef RtlLeftChildAvl
#undef RtlParentAvl
#undef RtlIsLeftChildAvl
#undef RtlIsRightChildAvl
#undef RtlInsertAsLeftChildAvl
#undef RtlInsertAsRightChildAvl
#undef PRTL_AVL_TABLE
#undef PRTL_BALANCED_LINKS
#undef MI_ASSERT

TABLE_SEARCH_RESULT
NTAPI
RtlpFindAvlTableNodeOrParent(
    IN PMM_AVL_TABLE Table,
    IN PVOID Buffer,
    OUT PMMADDRESS_NODE *NodeOrParent)
{
    return MiFindAvlTableNodeOrParent(Table, Buffer, NodeOrParent);
}

void
NTAPI
RtlpInsertAvlTreeNode(
    IN PMM_AVL_TABLE Table,
    IN PMMADDRESS_NODE NewNode,
    IN PMMADDRESS_NODE Parent,
    IN TABLE_SEARCH_RESULT SearchResult)
{
    MiInsertAvlTreeNode(Table, NewNode, (PVOID)Parent, SearchResult);
}

void
NTAPI
RtlpDeleteAvlTreeNode(
    IN PMM_AVL_TABLE Table,
    IN PMMADDRESS_NODE Node)
{
    MiDeleteAvlTreeNode(Table, Node);
}

/* ===================================================================
 * FORCEINLINE shims for Tier 3a Rust integration
 *
 * These functions are FORCEINLINE in miarm.h / ke_x.h — they have no
 * C symbol.  The Rust code calls them via extern "C" declarations.
 * All were renamed in Phase 1, so _orig_* versions are available.
 * =================================================================== */

USHORT MiIncrementPageTableReferences(IN PVOID Address)
{ return _orig_MiIncrementPageTableReferences(Address); }

void MiDropLockCount(IN PMMPFN Pfn1)
{ _orig_MiDropLockCount(Pfn1); }

void MiReferenceProbedPageAndBumpLockCount(IN PMMPFN Pfn1)
{ _orig_MiReferenceProbedPageAndBumpLockCount(Pfn1); }

void MiReferenceUnusedPageAndBumpLockCount(IN PMMPFN Pfn1)
{ _orig_MiReferenceUnusedPageAndBumpLockCount(Pfn1); }

void MiReferenceUsedPageAndBumpLockCount(IN PMMPFN Pfn1)
{ _orig_MiReferenceUsedPageAndBumpLockCount(Pfn1); }

BOOLEAN MiIsUserPde(PVOID Address)
{ return _orig_MiIsUserPde(Address); }

BOOLEAN MiIsUserPte(PVOID Address)
{ return _orig_MiIsUserPte(Address); }

PFN_NUMBER MiRemoveZeroPageSafe(IN ULONG Color)
{ return _orig_MiRemoveZeroPageSafe(Color); }

BOOLEAN MiIsMemoryTypeFree(TYPE_OF_MEMORY MemoryType)
{ return _orig_MiIsMemoryTypeFree(MemoryType); }

/* Tier 4 FORCEINLINE shim implementations */
#if (_MI_PAGING_LEVELS == 2)
BOOLEAN MiSynchronizeSystemPde(PMMPDE PointerPde)
{ return _orig_MiSynchronizeSystemPde(PointerPde); }
#endif

BOOLEAN MiConvertSharedWorkingSetLockToExclusive(PETHREAD Thread, PMMSUPPORT Vm)
{ return _orig_MiConvertSharedWorkingSetLockToExclusive(Thread, Vm); }

/*
 * KiAcquireProcessLockRaiseToSynch / KiReleaseProcessLock —
 * FORCEINLINE in ke_x.h (covered by PCH, can't rename).
 * Manually inline the implementation.
 *
 * On i386, KeReleaseInStackQueuedSpinLock is declared DECLSPEC_IMPORT (HAL import)
 * in the SDK headers, so the compiler generates an indirect call through the IAT.
 * However, the Rust shim_exports.rs provides its own local UP implementation of
 * KeAcquireInStackQueuedSpinLock* / KeReleaseInStackQueuedSpinLock with fastcall
 * convention.  The Rust acquire does NOT store (KeGetCurrentThread()|1) into the
 * spinlock, but the HAL's release checks for that value and bugchecks on mismatch.
 * To avoid the acquire/release mismatch, re-declare without __declspec(dllimport)
 * so the linker resolves to the Rust-provided version (matching acquire semantics).
 */
#ifdef _X86_
VOID FASTCALL KeReleaseInStackQueuedSpinLock(IN PKLOCK_QUEUE_HANDLE LockHandle);
#endif

void KiAcquireProcessLockRaiseToSynch(IN PKPROCESS Process,
                                       IN PKLOCK_QUEUE_HANDLE Handle)
{
    KeAcquireInStackQueuedSpinLockRaiseToSynch(&Process->ProcessLock, Handle);
}

void KiReleaseProcessLock(IN PKLOCK_QUEUE_HANDLE Handle)
{
    KeReleaseInStackQueuedSpinLock(Handle);
}

/* ===================================================================
 * Tier 3b FORCEINLINE shims (ARM3/section.c, ARM3/virtual.c)
 * =================================================================== */

void MiLockProcessWorkingSetForFault(IN PEPROCESS Process,
                                      IN PETHREAD Thread,
                                      IN BOOLEAN Safe,
                                      IN BOOLEAN Shared)
{
    _orig_MiLockProcessWorkingSetForFault(Process, Thread, Safe, Shared);
}

void MiUnlockProcessWorkingSetForFault(IN PEPROCESS Process,
                                        IN PETHREAD Thread,
                                        OUT PBOOLEAN Safe,
                                        OUT PBOOLEAN Shared)
{
    _orig_MiUnlockProcessWorkingSetForFault(Process, Thread, Safe, Shared);
}

#undef MmGetCurrentAddressSpace
PMMSUPPORT MmGetCurrentAddressSpace(VOID)
{
    return &((PEPROCESS)KeGetCurrentThread()->ApcState.Process)->Vm;
}

/* MiDereferencePfnAndDropLockCount: provided by Rust (section.rs) — no C shim */

BOOLEAN MiIsRosSectionObject(IN PSECTION Section)
{
    return _orig_MiIsRosSectionObject(Section);
}

/*
 * ProbeForWriteSize_t — macro in probe.h
 * ProbeForWriteIoStatusBlock — macro in probe.h
 * Must #undef the macros before defining outline functions.
 * These are cdecl (no NTAPI) — Rust imports via extern "C".
 *
 * ProbeForWriteUlong / ProbeForWritePointer are already provided as NTAPI
 * in tier1_rust_shims.c — do not duplicate here.
 */
#undef ProbeForWriteSize_t
void ProbeForWriteSize_t(IN SIZE_T *Ptr)
{
    volatile SIZE_T *p = (volatile SIZE_T *)Ptr;
    *p = *p;
}

#undef ProbeForWriteIoStatusBlock
void ProbeForWriteIoStatusBlock(IN PIO_STATUS_BLOCK Ptr)
{
    volatile IO_STATUS_BLOCK *p = (volatile IO_STATUS_BLOCK *)Ptr;
    *p = *p;
}

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
#if defined(_M_AMD64)
KIRQL KfRaiseIrql(IN KIRQL NewIrql)
{
    KIRQL OldIrql = (KIRQL)__readcr8();
    __writecr8(NewIrql);
    return OldIrql;
}

void KfLowerIrql(IN KIRQL NewIrql)
{
    __writecr8(NewIrql);
}
#elif defined(_M_IX86)
/* On i386, KfRaiseIrql/KfLowerIrql are __fastcall HAL imports (declared in
 * xdk/x86/ke.h). The Rust side calls cdecl wrappers named RustShimKf*.
 * We can't redefine the same name with a different calling convention,
 * so provide differently-named cdecl wrappers. */
KIRQL RustShimKfRaiseIrql(IN KIRQL NewIrql)
{
    return KfRaiseIrql(NewIrql);
}

void RustShimKfLowerIrql(IN KIRQL NewIrql)
{
    KfLowerIrql(NewIrql);
}
#endif

/* RtlFindSetBits cdecl wrapper for Rust (NTAPI→cdecl calling convention) */
ULONG RustShimRtlFindSetBits(PRTL_BITMAP BitMapHeader, ULONG NumberToFind, ULONG HintIndex)
{
    return RtlFindSetBits(BitMapHeader, NumberToFind, HintIndex);
}

/* KdbpGetHexNumber cdecl wrapper for Rust (NTAPI→cdecl calling convention) */
BOOLEAN NTAPI KdbpGetHexNumber(IN PCHAR pszNum, OUT ULONG_PTR *pulValue);
BOOLEAN RustShimKdbpGetHexNumber(PCHAR Arg, ULONG_PTR *Number)
{
    return KdbpGetHexNumber(Arg, Number);
}

