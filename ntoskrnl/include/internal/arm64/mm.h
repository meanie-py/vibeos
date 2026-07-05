/*
 * kernel internal memory management definitions for arm64
 */
#pragma once

/* ARM64 uses 4-level page tables with 48-bit virtual addresses */
#define _MI_PAGING_LEVELS 4
#define _MI_HAS_NO_EXECUTE 1

/* Memory layout base addresses - ARM64 Windows-compatible */
#define MI_USER_PROBE_ADDRESS                   (PVOID)0x7FFFFFFEFFFF
#define MI_DEFAULT_SYSTEM_RANGE_START           (PVOID)0xFFFF800000000000ULL
#define HYPER_SPACE                                    0xFFFFF70000000000ULL
#define HYPER_SPACE_END                                0xFFFFF77FFFFFFFFFULL
#define MI_SYSTEM_CACHE_WS_START                (PVOID)0xFFFFF78000000000ULL
#define MI_SYSTEM_CACHE_START                          0xFFFFF98000000000ULL
#define MI_PAGED_POOL_START                     (PVOID)0xFFFFFA8000000000ULL
#define MI_NONPAGED_POOL_END                    (PVOID)0xFFFFFAFFFFFFFFFFULL
#define MI_DEBUG_MAPPING                        (PVOID)0xFFFFFB0000000000ULL
#define MI_HIGHEST_SYSTEM_ADDRESS               (PVOID)0xFFFFFFFFFFFFFFFFULL

/* Page table entry counts - ARM64 with 4KB pages */
#define PTE_PER_PAGE 512
#define PDE_PER_PAGE 512
#define PPE_PER_PAGE 512
#define PXE_PER_PAGE 512

/* The size of the virtual memory area that is mapped using a single PDE */
#define PDE_MAPPED_VA (PTE_PER_PAGE * PAGE_SIZE)

/* Misc address definitions */
#define MI_SYSTEM_PTE_BASE                      (PVOID)MiAddressToPte(NULL)
#define MM_HIGHEST_VAD_ADDRESS \
    (PVOID)((ULONG_PTR)MM_HIGHEST_USER_ADDRESS - (16 * PAGE_SIZE))
#define MI_MAPPING_RANGE_START              (HYPER_SPACE)
#define MI_MAPPING_RANGE_END                (MI_MAPPING_RANGE_START + \
                                                  MI_HYPERSPACE_PTES * PAGE_SIZE)
#define MI_DUMMY_PTE                        (PMMPTE)(MI_MAPPING_RANGE_END + \
                                                  PAGE_SIZE)
#define MI_VAD_BITMAP                       (PMMPTE)(((ULONG_PTR)MI_DUMMY_PTE) + \
                                                  PAGE_SIZE)
#define MI_WORKING_SET_LIST                 (PMMPTE)(((ULONG_PTR)MI_VAD_BITMAP) + \
                                                  PAGE_SIZE)

/* PFN database virtual address — room for PFN DB + nonpaged pool up to MI_NONPAGED_POOL_END */
#define MI_PFN_DATABASE                         0xFFFFFA0000000000ULL

/* Number of system PTEs (same as amd64) */
#define MI_NUMBER_SYSTEM_PTES                   22000

/* Memory sizes */
#define MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING   ((255 * _1MB) >> PAGE_SHIFT)
#define MI_MIN_PAGES_FOR_SYSPTE_TUNING          ((19 * _1MB) >> PAGE_SHIFT)
#define MI_MIN_PAGES_FOR_SYSPTE_BOOST           ((32 * _1MB) >> PAGE_SHIFT)
#define MI_MIN_PAGES_FOR_SYSPTE_BOOST_BOOST     ((256 * _1MB) >> PAGE_SHIFT)
#define MI_MIN_INIT_PAGED_POOLSIZE              (32 * _1MB)
#define MI_MAX_INIT_NONPAGED_POOL_SIZE          (128 * _1MB)
#define MI_MAX_NONPAGED_POOL_SIZE               (128 * _1MB)
#define MI_SYSTEM_VIEW_SIZE                     (32 * _1MB)
#define MI_SESSION_VIEW_SIZE                    (48 * _1MB)
#define MI_SESSION_POOL_SIZE                    (16 * _1MB)
#define MI_SESSION_IMAGE_SIZE                   (8 * _1MB)
#define MI_SESSION_WORKING_SET_SIZE             (4 * _1MB)
#define MI_SESSION_SIZE                         (MI_SESSION_VIEW_SIZE + \
                                                 MI_SESSION_POOL_SIZE + \
                                                 MI_SESSION_IMAGE_SIZE + \
                                                 MI_SESSION_WORKING_SET_SIZE)
#define MI_MIN_ALLOCATION_FRAGMENT              (4 * _1KB)
#define MI_ALLOCATION_FRAGMENT                  (64 * _1KB)
#define MI_MAX_ALLOCATION_FRAGMENT              (2  * _1MB)

/* Misc constants */
#define MM_PTE_SOFTWARE_PROTECTION_BITS         6
#define MI_MIN_SECONDARY_COLORS                 8
#define MI_SECONDARY_COLORS                     64
#define MI_MAX_SECONDARY_COLORS                 1024
#define MI_MAX_FREE_PAGE_LISTS                  4
#define MI_HYPERSPACE_PTES                     (256 - 1)
#define MI_ZERO_PTES                           (32)
#define MI_MAX_ZERO_BITS                        21
#define SESSION_POOL_LOOKASIDES                 26

/* PPE/PXE write macros (same as PTE/PDE) */
#define MI_WRITE_VALID_PPE MI_WRITE_VALID_PTE
#define MI_WRITE_VALID_PXE MI_WRITE_VALID_PTE
#define ValidKernelPpe ValidKernelPde
#define ValidKernelPxe ValidKernelPde

/* MMPTE related defines */
/* Must fit in MMPTE_LIST.NextEntry (36-bit bitfield) — matches AMD64 */
#define MM_EMPTY_PTE_LIST  ((ULONG64)0xFFFFFFFF)
#define MM_EMPTY_LIST  ((ULONG_PTR)-1)

/* Easy accessing PFN in PTE/PDE */
#define PFN_FROM_PTE(v) ((v)->u.Hard.PageFrameNumber)
#define PFN_FROM_PDE(v) ((v)->u.Hard.PageFrameNumber)
#define PFN_FROM_PPE(v) ((v)->u.Hard.PageFrameNumber)
#define PFN_FROM_PXE(v) ((v)->u.Hard.PageFrameNumber)

/* Macros for portable PTE modification */
#define MI_MAKE_DIRTY_PAGE(x)      ((x)->u.Hard.NotDirty = 0)
#define MI_MAKE_CLEAN_PAGE(x)      ((x)->u.Hard.NotDirty = 1)
#define MI_MAKE_ACCESSED_PAGE(x)   ((x)->u.Hard.Accessed = 1)
#define MI_PAGE_DISABLE_CACHE(x)   ((x)->u.Hard.CacheType = 0)
#define MI_PAGE_WRITE_THROUGH(x)   ((x)->u.Hard.CacheType = 1)
#define MI_PAGE_WRITE_COMBINED(x)  ((x)->u.Hard.CacheType = 3)
#define MI_IS_PAGE_LARGE(x)        ((x)->u.Hard.NotLargePage == 0)
#define MI_IS_PAGE_WRITEABLE(x)    ((x)->u.Hard.Writable == 1)
#define MI_IS_PAGE_COPY_ON_WRITE(x)((x)->u.Hard.CopyOnWrite == 1)
#define MI_IS_PAGE_EXECUTABLE(x)   ((x)->u.Hard.UserNoExecute == 0)
#define MI_IS_PAGE_DIRTY(x)        ((x)->u.Hard.NotDirty == 0)
#define MI_MAKE_OWNER_PAGE(x)      ((x)->u.Hard.Owner = 1)
#define MI_MAKE_WRITE_PAGE(x)      ((x)->u.Hard.Writable = 1)

/*
 * Macros to identify the page fault reason from the error code.
 * ARM64 encodes FaultCode identically to x86 for compatibility with
 * the arch-independent MM code in ARM3/pagfault.c and mmfault.c:
 *   Bit 0: 1 = page was present (permission fault), 0 = not present (translation fault)
 *   Bit 1: 1 = write access, 0 = read access
 *   Bit 4: 1 = instruction fetch
 * See KiSynchronousExceptionHandler in trap.c for the encoding.
 */
#define MI_IS_NOT_PRESENT_FAULT(FaultCode) !BooleanFlagOn(FaultCode, 0x00000001)
#define MI_IS_WRITE_ACCESS(FaultCode)       BooleanFlagOn(FaultCode, 0x00000002)
#define MI_IS_INSTRUCTION_FETCH(FaultCode)  BooleanFlagOn(FaultCode, 0x00000010)

/* ARM64 page table structure: 4KB pages, 4 levels (PXE/PPE/PDE/PTE).
 * AddressToP*e functions use masked offsets to stay within the self-map
 * page ranges — same approach as AMD64. Without masking, kernel VAs
 * (with upper bits all 1s) produce offsets that escape the self-map. */

FORCEINLINE PMMPTE
_MiAddressToPte(PVOID Address)
{
    ULONG64 Offset = (ULONG64)Address >> (12 - 3);
    Offset &= 0xFFFFFFFFFULL << 3;
    return (PMMPTE)(PTE_BASE + Offset);
}
#define MiAddressToPte(x) _MiAddressToPte((PVOID)(x))

FORCEINLINE PMMPDE
_MiAddressToPde(PVOID Address)
{
    ULONG64 Offset = (ULONG64)Address >> (21 - 3);
    Offset &= 0x7FFFFFFULL << 3;
    return (PMMPDE)(PDE_BASE + Offset);
}
#define MiAddressToPde(x) _MiAddressToPde((PVOID)(x))

FORCEINLINE PMMPTE
MiAddressToPpe(PVOID Address)
{
    ULONG64 Offset = (ULONG64)Address >> (30 - 3);
    Offset &= 0x3FFFFULL << 3;
    return (PMMPTE)(PPE_BASE + Offset);
}

FORCEINLINE PMMPTE
MiAddressToPxe(PVOID Address)
{
    ULONG64 Offset = (ULONG64)Address >> (39 - 3);
    Offset &= 0x1FFULL << 3;
    return (PMMPTE)(PXE_BASE + Offset);
}

/* Check if the PDE hierarchy for a given address is valid (all levels present).
 * Same as AMD64 version but with 4 levels. */
FORCEINLINE
BOOLEAN
MiIsPdeForAddressValid(PVOID Address)
{
    return ((MiAddressToPxe(Address)->u.Hard.Valid) &&
            (MiAddressToPpe(Address)->u.Hard.Valid) &&
            (MiAddressToPde(Address)->u.Hard.Valid));
}

/* Convert an address to a corresponding PTE offset/index */
#define MiAddressToPteOffset(x) \
    ((((ULONG_PTR)(x)) >> 12) & 0x1FF)

/* Convert an address to a corresponding PDE offset/index */
#define MiAddressToPdeOffset(x) \
    ((((ULONG_PTR)(x)) >> 21) & 0x1FF)
#define MiGetPdeOffset MiAddressToPdeOffset

/* Convert an address to a corresponding PPE offset/index */
#define MiAddressToPpeOffset(x) \
    ((((ULONG_PTR)(x)) >> 30) & 0x1FF)

/* Convert an address to a corresponding PXE offset/index */
#define MiAddressToPxeOffset(x) \
    ((((ULONG_PTR)(x)) >> 39) & 0x1FF)

/* Convert a PTE/PDE/PPE/PXE into a corresponding address.
 * Use signed math so kernel VAs (bit 47=1) are properly sign-extended
 * to canonical form (bits 63:48 all 1s). Same approach as AMD64. */
FORCEINLINE PVOID MiPteToAddress(PMMPTE PointerPte)
{
    return (PVOID)(((LONG64)PointerPte << 25) >> 16);
}

FORCEINLINE PVOID MiPdeToAddress(PMMPTE PointerPde)
{
    return (PVOID)(((LONG64)PointerPde << 34) >> 16);
}

FORCEINLINE PVOID MiPpeToAddress(PMMPTE PointerPpe)
{
    return (PVOID)(((LONG64)PointerPpe << 43) >> 16);
}

FORCEINLINE PVOID MiPxeToAddress(PMMPTE PointerPxe)
{
    return (PVOID)(((LONG64)PointerPxe << 52) >> 16);
}

/* Translate between P*Es.
 * NB: PDE-of-X is PTE-of-(PTE-of-X), i.e. ONE self-map recursion step, so
 * MiPteToPde is MiAddressToPte (NOT MiAddressToPde, which lands one level
 * too high on the PPE and once corrupted PT-page share counts at teardown —
 * BUGCHECK 0x4E/0x99, see field-manual §9.16). Matches amd64/mm.h. */
#define MiPdeToPte(_Pde)   ((PMMPTE)MiPteToAddress(_Pde))
#define MiPteToPde(_Pte)   ((PMMPDE)MiAddressToPte(_Pte))
#define MiPpeToPde(_Ppe)   ((PMMPDE)MiPdeToAddress(_Ppe))

/* Convert a PPE into its lowest PTE */
FORCEINLINE
PMMPTE
MiPpeToPte(PMMPPE PointerPpe)
{
    return (PMMPTE)MiPdeToAddress(PointerPpe);
}

/* Convert a PXE into its lowest PTE */
FORCEINLINE
PMMPTE
MiPxeToPte(PMMPXE PointerPxe)
{
    return (PMMPTE)MiPpeToAddress(PointerPxe);
}

/* Convert a PTE to a corresponding PPE (two recursion steps — matches amd64) */
FORCEINLINE
PMMPPE
MiPteToPpe(PMMPTE PointerPte)
{
    return (PMMPPE)MiAddressToPde(PointerPte);
}

/* Convert a PTE to a corresponding PXE (three recursion steps — matches amd64) */
FORCEINLINE
PMMPXE
MiPteToPxe(PMMPTE PointerPte)
{
    return (PMMPXE)MiAddressToPpe(PointerPte);
}

/* Convert a PDE to a corresponding PPE */
FORCEINLINE
PMMPPE
MiPdeToPpe(PMMPDE PointerPde)
{
    return (PMMPPE)MiAddressToPte(PointerPde);
}

/* Convert a PDE to a corresponding PXE */
FORCEINLINE
PMMPXE
MiPdeToPxe(PMMPDE PointerPde)
{
    return (PMMPXE)MiAddressToPde(PointerPde);
}

/* Check P*E boundaries */
#define MiIsPteOnPdeBoundary(PointerPte) \
    ((((ULONG_PTR)PointerPte) & (PAGE_SIZE - 1)) == 0)
#define MiIsPteOnPpeBoundary(PointerPte) \
    ((((ULONG_PTR)PointerPte) & ((PTE_PER_PAGE * PAGE_SIZE) - 1)) == 0)
#define MiIsPteOnPxeBoundary(PointerPte) \
    ((((ULONG_PTR)PointerPte) & ((PTE_PER_PAGE * PTE_PER_PAGE * PAGE_SIZE) - 1)) == 0)

//
// Decodes a Prototype PTE into the underlying PTE
// Arithmetic right shift sign-extends the 48-bit address (same as AMD64)
//
#define MiProtoPteToPte(x)                  \
    (PMMPTE)(((LONG64)(x)->u.Long) >> 16)

//
// Decodes a Subsection PTE into the Subsection pointer
// Arithmetic right shift sign-extends the 48-bit address (same as AMD64)
//
#define MiSubsectionPteToSubsection(x)      \
    (PMMPTE)(((LONG64)(x)->u.Long) >> 16)

/* See PDR definition */
#define MI_ZERO_PTE                         (PMMPTE)(MI_MAPPING_RANGE_END + \
                                             PAGE_SIZE)

//
// Builds a Prototype PTE for the address of the PTE
// ARM64 uses a single 48-bit ProtoAddress field (same as AMD64)
//
FORCEINLINE
VOID
MI_MAKE_PROTOTYPE_PTE(IN PMMPTE NewPte,
                      IN PMMPTE PointerPte)
{
    /* Store the Address */
    NewPte->u.Long = (ULONG64)PointerPte << 16;

    /* Mark this as a prototype PTE */
    NewPte->u.Proto.Prototype = 1;

    ASSERT(MiProtoPteToPte(NewPte) == PointerPte);
}

//
// Builds a Subsection PTE for the address of the Segment
// ARM64 uses a single 48-bit SubsectionAddress field
//
FORCEINLINE
VOID
MI_MAKE_SUBSECTION_PTE(
    _Out_ PMMPTE NewPte,
    _In_ PVOID Segment)
{
    /* Mark this as a prototype */
    NewPte->u.Long = 0;
    NewPte->u.Subsect.Prototype = 1;

    /* Store the lower 48 bits of the Segment address */
    NewPte->u.Subsect.SubsectionAddress = ((ULONG_PTR)Segment & 0x0000FFFFFFFFFFFF);
}

FORCEINLINE
BOOLEAN
MI_IS_MAPPED_PTE(PMMPTE PointerPte)
{
    return ((PointerPte->u.Hard.Valid != 0) ||
            (PointerPte->u.Proto.Prototype != 0) ||
            (PointerPte->u.Trans.Transition != 0) ||
            (PointerPte->u.Hard.PageFrameNumber != 0));
}
