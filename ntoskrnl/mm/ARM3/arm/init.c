/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         BSD - See COPYING.ARM in the top level directory
 * FILE:            ntoskrnl/mm/ARM3/arm/init.c
 * PURPOSE:         ARM/ARM64 Memory Manager Initialization
 * PROGRAMMERS:     ReactOS Portable Systems Group
 *                  ReactOS ARM64 Port
 */

/* INCLUDES *******************************************************************/

#include <ntoskrnl.h>
#define NDEBUG
#include <debug.h>

#define MODULE_INVOLVED_IN_ARM3
#include <mm/ARM3/miarm.h>

#ifdef _M_ARM64

/* GLOBALS ********************************************************************/

/* Template PTE and PDE for a kernel page.
 * ARM64 requires bit 1 (NotLargePage) set for table descriptors (L0-L2)
 * and page descriptors (L3). Without it, the entry is block/reserved. */
MMPTE ValidKernelPde = {{PTE_VALID | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED}};
MMPTE ValidKernelPte = {{PTE_VALID | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED | PTE_ENABLE_CACHE}};

/* The same, but for local pages */
MMPTE ValidKernelPdeLocal = {{PTE_VALID | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED}};
MMPTE ValidKernelPteLocal = {{PTE_VALID | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED | PTE_ENABLE_CACHE}};

/* Template PDE for a demand-zero page */
MMPDE DemandZeroPde  = {{MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS}};
MMPTE DemandZeroPte  = {{MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS}};

/* Template PTE for prototype page */
MMPTE PrototypePte = {{(MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS) |
                      PTE_PROTOTYPE | (MI_PTE_LOOKUP_NEEDED << 32)}};

/* Template PTE for decommited page */
MMPTE MmDecommittedPte = {{MM_DECOMMIT << MM_PTE_SOFTWARE_PROTECTION_BITS}};

/* Address ranges unique to arch-specific init (not in mminit.c) */
PVOID MiSessionViewEnd;
PVOID MiSystemPteSpaceStart;
PVOID MiSystemPteSpaceEnd;

/* ARM64 UART trace for early MM debugging */
#define _MM_UART(s) do { \
    volatile ULONG *_u = (volatile ULONG *)0x09000000ULL; \
    const char *_m = (s); \
    while (*_m) { *_u = (ULONG)*_m++; } \
} while(0)

/* PRIVATE FUNCTIONS **********************************************************/

CODE_SEG("INIT")
VOID
NTAPI
MiInitializeSessionSpaceLayout(VOID)
{
    /* This is the entire size */
    MmSessionSize = MI_SESSION_SIZE;

    /* Session space ends at PTE_BASE (matches i386 convention) */
    MiSessionSpaceEnd = (PVOID)PTE_BASE;

    /* The highest range is the session image range */
    MmSessionImageSize = MI_SESSION_IMAGE_SIZE;
    MiSessionImageEnd = MiSessionSpaceEnd;
    MiSessionImageStart = (PUCHAR)MiSessionImageEnd - MmSessionImageSize;

    /* Session working set is below the session image range */
    MiSessionSpaceWs = (PUCHAR)MiSessionImageStart - MI_SESSION_WORKING_SET_SIZE;

    /* Session view is below the session working set */
    MmSessionViewSize = MI_SESSION_VIEW_SIZE;
    MiSessionViewEnd = MiSessionSpaceWs;
    MiSessionViewStart = (PUCHAR)MiSessionViewEnd - MmSessionViewSize;

    /* Session pool is below session view */
    MmSessionPoolSize = MI_SESSION_POOL_SIZE;
    MiSessionPoolEnd = MiSessionViewStart;
    MiSessionPoolStart = (PUCHAR)MiSessionPoolEnd - MmSessionPoolSize;

    /* And it all begins here */
    MmSessionBase = MiSessionPoolStart;

    /* Compute the PTE addresses for all the addresses we carved out */
    MiSessionImagePteStart = MiAddressToPte(MiSessionImageStart);
    MiSessionImagePteEnd = MiAddressToPte(MiSessionImageEnd);
    MiSessionBasePte = MiAddressToPte(MmSessionBase);
    MiSessionLastPte = MiAddressToPte(MiSessionSpaceEnd);

    /* Initialize the pointer to the session space structure */
    MmSessionSpace = (PMM_SESSION_SPACE)((ULONG_PTR)MiSessionImageStart + 0x10000);

    /* System view space ends at session space */
    MmSystemViewSize = MI_SYSTEM_VIEW_SIZE;
    MiSystemViewStart = (PUCHAR)MmSessionBase - MmSystemViewSize;
}

static
VOID
MiMapPXEs(
    PVOID StartAddress,
    PVOID EndAddress)
{
    PMMPTE PointerPxe;
    MMPTE TmplPte = ValidKernelPde;

    for (PointerPxe = MiAddressToPxe(StartAddress);
         PointerPxe <= MiAddressToPxe(EndAddress);
         PointerPxe++)
    {
        if (!PointerPxe->u.Hard.Valid)
        {
            TmplPte.u.Hard.PageFrameNumber = MxGetNextPage(1);
            MI_WRITE_VALID_PTE(PointerPxe, TmplPte);
            __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");
            RtlZeroMemory(MiPteToAddress(PointerPxe), PAGE_SIZE);
        }
    }
}

static
VOID
MiMapPPEs(
    PVOID StartAddress,
    PVOID EndAddress)
{
    PMMPTE PointerPpe;
    MMPTE TmplPte = ValidKernelPde;

    for (PointerPpe = MiAddressToPpe(StartAddress);
         PointerPpe <= MiAddressToPpe(EndAddress);
         PointerPpe++)
    {
        if (!PointerPpe->u.Hard.Valid)
        {
            TmplPte.u.Hard.PageFrameNumber = MxGetNextPage(1);
            MI_WRITE_VALID_PTE(PointerPpe, TmplPte);
            __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");
            RtlZeroMemory(MiPteToAddress(PointerPpe), PAGE_SIZE);
        }
    }
}

static
VOID
MiMapPDEs(
    PVOID StartAddress,
    PVOID EndAddress)
{
    PMMPDE PointerPde;
    MMPTE TmplPte = ValidKernelPde;

    for (PointerPde = MiAddressToPde(StartAddress);
         PointerPde <= MiAddressToPde(EndAddress);
         PointerPde++)
    {
        if (!PointerPde->u.Hard.Valid)
        {
            TmplPte.u.Hard.PageFrameNumber = MxGetNextPage(1);
            MI_WRITE_VALID_PTE(PointerPde, TmplPte);
            __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");
            RtlZeroMemory(MiPteToAddress(PointerPde), PAGE_SIZE);
        }
    }
}

static
VOID
MiMapPTEs(
    PVOID StartAddress,
    PVOID EndAddress)
{
    PMMPTE PointerPte;
    MMPTE TmplPte = ValidKernelPte;

    for (PointerPte = MiAddressToPte(StartAddress);
         PointerPte <= MiAddressToPte(EndAddress);
         PointerPte++)
    {
        if (!PointerPte->u.Hard.Valid)
        {
            TmplPte.u.Hard.PageFrameNumber = MxGetNextPage(1);
            MI_WRITE_VALID_PTE(PointerPte, TmplPte);
        }
    }
}

/*
 * Ensure page table hierarchy exists for a VA range at all 4 levels.
 * This calls MiMapPXEs, MiMapPPEs, MiMapPDEs in sequence.
 */
static
VOID
MiEnsurePageTableHierarchy(
    PVOID StartAddress,
    PVOID EndAddress)
{
    MiMapPXEs(StartAddress, EndAddress);
    MiMapPPEs(StartAddress, EndAddress);
    MiMapPDEs(StartAddress, EndAddress);
}

CODE_SEG("INIT")
static
VOID
MiInitializePageTable(VOID)
{
    ULONG64 PxePhysicalAddress;
    MMPTE TmplPte;
    PMMPTE PointerPxe;
    PFN_NUMBER PxePfn;

    /* Get current PGD from the self-map */
    PxePfn = ((PMMPTE)PXE_SELFMAP)->u.Hard.PageFrameNumber;
    PxePhysicalAddress = PxePfn << PAGE_SHIFT;

    /* Set directory base for the system process */
    PsGetCurrentProcess()->Pcb.DirectoryTableBase[0] = PxePhysicalAddress;

    /* On ARM64, skip zeroing user PXEs for now.
     * The identity mapping (TTBR0 range) is still needed for:
     *  - PL011 UART at 0x09000000 (debug output)
     *  - Any remaining identity-mapped accesses during boot
     * Once we have proper kernel MMIO mapping, we can zero these. */

    /* Flush the TLB */
    KeFlushCurrentTb();

    /* Set up a template PTE for hyperspace */
    TmplPte.u.Long = 0;
    TmplPte.u.Hard.Valid = 1;
    TmplPte.u.Hard.NotLargePage = 1;
    TmplPte.u.Hard.Writable = 1;
    TmplPte.u.Hard.Accessed = 1;
    HyperTemplatePte = TmplPte;

    /* Create PXEs for system address space.
     * Start from MiSystemViewStart (below session space / PTE_BASE) because
     * on ARM64 session space is at PXE 0x1EC, below HYPER_SPACE (PXE 0x1EE).
     * The self-map PXE (0x1ED) is already valid and will be skipped. */
    for (PointerPxe = MiAddressToPxe(MiSystemViewStart);
         PointerPxe <= MiAddressToPxe(MI_HIGHEST_SYSTEM_ADDRESS);
         PointerPxe++)
    {
        if (!PointerPxe->u.Hard.Valid)
        {
            TmplPte.u.Long = ValidKernelPde.u.Long;
            TmplPte.u.Hard.PageFrameNumber = MxGetNextPage(1);
            *PointerPxe = TmplPte;

            /* ARM64 weak ordering: ensure the PTE write is visible to the
             * hardware page table walker before accessing through it */
            __asm__ __volatile__("dsb ish\n\tisb" ::: "memory");

            RtlZeroMemory(MiPteToAddress(PointerPxe), PAGE_SIZE);
        }
    }

    /* Save hyperspace PXE PFN */
    PxePfn = PFN_FROM_PXE(MiAddressToPxe((PVOID)HYPER_SPACE));
    PsGetCurrentProcess()->Pcb.DirectoryTableBase[1] = PxePfn << PAGE_SHIFT;

    /* Map PPEs for paged pool */
    MiMapPPEs(MmPagedPoolStart, MmPagedPoolEnd);

    /* Setup PPEs for hyper space */
    MiMapPPEs((PVOID)HYPER_SPACE, (PVOID)HYPER_SPACE_END);

    /* Setup PPEs for system space view */
    MiMapPPEs(MiSystemViewStart, (PCHAR)MiSystemViewStart + MmSystemViewSize);

    /* Setup the mapping PDEs */
    MiEnsurePageTableHierarchy((PVOID)MI_MAPPING_RANGE_START,
                               (PVOID)MI_MAPPING_RANGE_END);

    /* Setup the mapping PTEs */
    MmFirstReservedMappingPte = MiAddressToPte((PVOID)MI_MAPPING_RANGE_START);
    MmLastReservedMappingPte = MiAddressToPte((PVOID)MI_MAPPING_RANGE_END);
    MmFirstReservedMappingPte->u.Hard.PageFrameNumber = MI_HYPERSPACE_PTES;

    /* Setup debug mapping PTE */
    MiEnsurePageTableHierarchy((PVOID)MI_DEBUG_MAPPING, (PVOID)MI_DEBUG_MAPPING);
    MmDebugPte = MiAddressToPte((PVOID)MI_DEBUG_MAPPING);

    /* Setup PDE and PTEs for VAD bitmap and working set list */
    MiEnsurePageTableHierarchy((PVOID)MI_VAD_BITMAP,
                               (PVOID)(MI_WORKING_SET_LIST + PAGE_SIZE - 1));
    MiMapPTEs((PVOID)MI_VAD_BITMAP, (PVOID)(MI_WORKING_SET_LIST + PAGE_SIZE - 1));
}

CODE_SEG("INIT")
static
VOID
MiBuildNonPagedPool(VOID)
{
    _MM_UART("[MM] MiBuildNonPagedPool\r\n");

    /* Check if this is a machine with less than 256MB of RAM, and no overide */
    if ((MmNumberOfPhysicalPages <= MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING) &&
        !(MmSizeOfNonPagedPoolInBytes))
    {
        MmSizeOfNonPagedPoolInBytes = 2 * 1024 * 1024;
    }

    /* Check if the user gave a ridiculously large nonpaged pool RAM size */
    if ((MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT) >
        (MmNumberOfPhysicalPages * 7 / 8))
    {
        MmSizeOfNonPagedPoolInBytes = 0;
    }

    /* Check if no registry setting was set, or if the setting was too low */
    if (MmSizeOfNonPagedPoolInBytes < MmMinimumNonPagedPoolSize)
    {
        MmSizeOfNonPagedPoolInBytes = MmMinimumNonPagedPoolSize;
        MmSizeOfNonPagedPoolInBytes += (MmNumberOfPhysicalPages - 1024) /
                                       256 * MmMinAdditionNonPagedPoolPerMb;
    }

    /* Cap at maximum */
    if (MmSizeOfNonPagedPoolInBytes > MI_MAX_INIT_NONPAGED_POOL_SIZE)
    {
        MmSizeOfNonPagedPoolInBytes = MI_MAX_INIT_NONPAGED_POOL_SIZE;
    }

    /* Check if a percentage cap was set through the registry */
    if (MmMaximumNonPagedPoolPercent) UNIMPLEMENTED;

    /* Page-align the nonpaged pool size */
    MmSizeOfNonPagedPoolInBytes &= ~(PAGE_SIZE - 1);

    /* Now, check if there was a registry size for the maximum size */
    if (!MmMaximumNonPagedPoolInBytes)
    {
        MmMaximumNonPagedPoolInBytes = MmDefaultMaximumNonPagedPool;
        MmMaximumNonPagedPoolInBytes += (MmNumberOfPhysicalPages - 1024) /
                                         256 * MmMaxAdditionNonPagedPoolPerMb;
    }

    /* Don't let the maximum go too high */
    if (MmMaximumNonPagedPoolInBytes > MI_MAX_NONPAGED_POOL_SIZE)
    {
        MmMaximumNonPagedPoolInBytes = MI_MAX_NONPAGED_POOL_SIZE;
    }

    /* Convert nonpaged pool size from bytes to pages */
    MmMaximumNonPagedPoolInPages = MmMaximumNonPagedPoolInBytes >> PAGE_SHIFT;

    /* Non paged pool starts after the PFN database */
    MmNonPagedPoolStart = (PUCHAR)MmPfnDatabase + MxPfnAllocation * PAGE_SIZE;

    /* Calculate the nonpaged pool expansion start region */
    MmNonPagedPoolExpansionStart = (PCHAR)MmNonPagedPoolStart +
                                          MmSizeOfNonPagedPoolInBytes;
    ASSERT(((ULONG_PTR)MmNonPagedPoolExpansionStart & (PAGE_SIZE - 1)) == 0);

    /* And this is where the none paged pool ends */
    MmNonPagedPoolEnd = (PCHAR)MmNonPagedPoolStart + MmMaximumNonPagedPoolInBytes;
    ASSERT(MmNonPagedPoolEnd < (PVOID)MM_HAL_VA_START);

    /* Map PXEs, PPEs and PDEs for non paged pool (including expansion) */
    MiEnsurePageTableHierarchy(MmNonPagedPoolStart, MmNonPagedPoolEnd);

    /* Map the nonpaged pool PTEs (without expansion) */
    MiMapPTEs(MmNonPagedPoolStart, (PCHAR)MmNonPagedPoolExpansionStart - 1);

    /* Initialize the ARM3 nonpaged pool */
    MiInitializeNonPagedPool();
    MiInitializeNonPagedPoolThresholds();
}

CODE_SEG("INIT")
static
VOID
MiBuildSystemPteSpace(VOID)
{
    PMMPTE PointerPte;
    SIZE_T NonPagedSystemSize;

    _MM_UART("[MM] MiBuildSystemPteSpace\r\n");

    /* Use the default number of system PTEs */
    MmNumberOfSystemPtes = MI_NUMBER_SYSTEM_PTES;
    NonPagedSystemSize = (MmNumberOfSystemPtes + 1) * PAGE_SIZE;

    /* Put system PTEs at the start of the system VA space */
    MiSystemPteSpaceStart = MmNonPagedSystemStart;
    MiSystemPteSpaceEnd = (PUCHAR)MiSystemPteSpaceStart + NonPagedSystemSize;

    /* Map the PXEs, PPEs and PDEs for the system PTEs */
    MiEnsurePageTableHierarchy(MiSystemPteSpaceStart, MiSystemPteSpaceEnd);

    /* Initialize the system PTE space */
    PointerPte = MiAddressToPte(MiSystemPteSpaceStart);
    MiInitializeSystemPtes(PointerPte, MmNumberOfSystemPtes, SystemPteSpace);

    /* Reserve system PTEs for zeroing PTEs and clear them */
    MiFirstReservedZeroingPte = MiReserveSystemPtes(MI_ZERO_PTES + 1,
                                                    SystemPteSpace);
    RtlZeroMemory(MiFirstReservedZeroingPte, (MI_ZERO_PTES + 1) * sizeof(MMPTE));

    /* Set the counter to maximum */
    MiFirstReservedZeroingPte->u.Hard.PageFrameNumber = MI_ZERO_PTES;

    _MM_UART("[MM] SysPTE space initialized\r\n");
}

static
VOID
MiSetupPfnForPageTable(
    PFN_NUMBER PageFrameIndex,
    PMMPTE PointerPte)
{
    PMMPFN Pfn;
    PMMPDE PointerPde;

    /* Get the pfn entry for this page */
    Pfn = MiGetPfnEntry(PageFrameIndex);

    /* Check if it's valid memory */
    if ((PageFrameIndex <= MmHighestPhysicalPage) &&
        (MmIsAddressValid(Pfn)) &&
        (Pfn->u3.e1.PageLocation == ActiveAndValid))
    {
        /* Setup the PFN entry */
        Pfn->u1.WsIndex = 0;
        Pfn->u2.ShareCount++;
        Pfn->PteAddress = PointerPte;
        Pfn->OriginalPte = *PointerPte;
        Pfn->u3.e1.PageLocation = ActiveAndValid;
        Pfn->u3.e1.CacheAttribute = MiNonCached;
        Pfn->u3.e2.ReferenceCount = 1;
        Pfn->u4.PteFrame = PFN_FROM_PTE(MiAddressToPte(PointerPte));
    }

    /* Increase the shared count of the PFN entry for the PDE */
    PointerPde = MiAddressToPde(MiPteToAddress(PointerPte));
    Pfn = MiGetPfnEntry(PFN_FROM_PTE(PointerPde));
    if ((PFN_FROM_PTE(PointerPde) <= MmHighestPhysicalPage) &&
        MmIsAddressValid(Pfn))
    {
        Pfn->u2.ShareCount++;
    }
}

CODE_SEG("INIT")
static
VOID
MiBuildPfnDatabaseFromPageTables(VOID)
{
    PVOID Address = NULL;
    PFN_NUMBER PageFrameIndex;
    PMMPDE PointerPde;
    PMMPTE PointerPte;
    ULONG i, j, k, l;
    PMMPFN Pfn;
    PMMPDE PointerPpe;
    PMMPDE PointerPxe;

    /* Manual setup of the top level page directory */
    PageFrameIndex = PFN_FROM_PTE(MiAddressToPte(PXE_BASE));
    Pfn = MiGetPfnEntry(PageFrameIndex);
    if (MmIsAddressValid(Pfn))
    {
        ASSERT(Pfn->u3.e1.PageLocation == ActiveAndValid);
        Pfn->u1.WsIndex = 0;
        Pfn->u2.ShareCount = 1;
        Pfn->PteAddress = NULL;
        Pfn->u3.e1.CacheAttribute = MiNonCached;
        Pfn->u3.e2.ReferenceCount = 1;
        Pfn->u4.PteFrame = 0;
    }

    /* Loop all PXEs */
    PointerPxe = MiAddressToPxe(Address);
    for (i = 0; i < PXE_PER_PAGE; i++, PointerPxe++)
    {
        if (!PointerPxe->u.Hard.Valid) continue;

        PageFrameIndex = PFN_FROM_PXE(PointerPxe);
        MiSetupPfnForPageTable(PageFrameIndex, (PMMPTE)PointerPxe);

        Address = MiPxeToAddress(PointerPxe);

        /* Loop all PPEs in this PDP */
        PointerPpe = MiAddressToPpe(Address);
        for (j = 0; j < PPE_PER_PAGE; j++, PointerPpe++)
        {
            if (!PointerPpe->u.Hard.Valid) continue;

            PageFrameIndex = PFN_FROM_PPE(PointerPpe);
            MiSetupPfnForPageTable(PageFrameIndex, (PMMPTE)PointerPpe);

            Address = MiPpeToAddress(PointerPpe);

            /* Loop all PDEs */
            PointerPde = MiAddressToPde(Address);
            for (k = 0; k < PDE_PER_PAGE; k++, PointerPde++)
            {
                if (!PointerPde->u.Hard.Valid) continue;

                PageFrameIndex = PFN_FROM_PDE(PointerPde);
                MiSetupPfnForPageTable(PageFrameIndex, (PMMPTE)PointerPde);

                Address = MiPdeToAddress(PointerPde);

                /* Loop all PTEs */
                PointerPte = MiAddressToPte(Address);
                for (l = 0; l < PTE_PER_PAGE; l++, PointerPte++)
                {
                    if (!PointerPte->u.Hard.Valid) continue;

                    PageFrameIndex = PFN_FROM_PTE(PointerPte);
                    if (PageFrameIndex <= MmHighestPhysicalPage)
                    {
                        Pfn = MiGetPfnEntry(PageFrameIndex);
                        if (MmIsAddressValid(Pfn) &&
                            Pfn->u3.e1.PageLocation != ActiveAndValid)
                        {
                            Pfn->u3.e1.PageLocation = ActiveAndValid;
                            Pfn->u3.e1.CacheAttribute = MiCached;
                        }
                    }
                }
            }
        }
    }
}

CODE_SEG("INIT")
static
VOID
MiBuildPfnDatabase(
    _In_ PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    _MM_UART("[MM] MiBuildPfnDatabase\r\n");

    /* Map PXEs, PPEs and PDEs for PFN database */
    MiEnsurePageTableHierarchy(MmPfnDatabase,
                               (PCHAR)MmPfnDatabase + (MxPfnAllocation << PAGE_SHIFT) - 1);

    /* Map PTEs for PFN database */
    MiMapPTEs(MmPfnDatabase,
              (PCHAR)MmPfnDatabase + (MxPfnAllocation << PAGE_SHIFT) - 1);

    /* Zero the PFN database */
    RtlZeroMemory(MmPfnDatabase, MxPfnAllocation << PAGE_SHIFT);

    /* Initialize color tables */
    MiInitializeColorTables();

    /* Build PFN database from existing page tables FIRST.
     * This marks all mapped pages as ActiveAndValid (but with ReferenceCount=0
     * for non-page-table pages). Must run before MiInitializePfnDatabase so
     * that MiBuildPfnDatabaseFromLoaderBlock can correctly add free pages to
     * the free list (overwriting the ActiveAndValid status with FreePageList). */
    MiBuildPfnDatabaseFromPageTables();

    /* Now scan memory descriptors to set up PFN entries for free pages */
    MiInitializePfnDatabase(LoaderBlock);
}

/* MAIN FUNCTION *************************************************************/

CODE_SEG("INIT")
NTSTATUS
NTAPI
MiInitMachineDependent(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    NTSTATUS Status;
    ULONG Flags;

    _MM_UART("[MM] MiInitMachineDependent ARM64\r\n");

    ASSERT(MxPfnAllocation != 0);

    /* Set some hardcoded addresses */
    MmHyperSpaceEnd = (PVOID)HYPER_SPACE_END;
    MmNonPagedSystemStart = (PVOID)MI_DEFAULT_SYSTEM_RANGE_START;
    MmPfnDatabase = (PVOID)MI_PFN_DATABASE;
    MmWorkingSetList = (PVOID)MI_WORKING_SET_LIST;

    /* Set the prototype PTE lookup field */
    PrototypePte.u.Soft.PageFileHigh = MI_PTE_LOOKUP_NEEDED;

    MiInitializePageTable();
    MiBuildNonPagedPool();
    MiBuildSystemPteSpace();
    MiBuildPfnDatabase(LoaderBlock);

    /* Reset ref/share counts for MmInitializeProcessAddressSpace */
    {
        PMMPFN Pfn;

        Pfn = MiGetPfnEntry(PFN_FROM_PTE((PMMPTE)PXE_SELFMAP));
        if (MmIsAddressValid(Pfn))
        {
            Pfn->u2.ShareCount = 0;
            Pfn->u3.e2.ReferenceCount = 0;
        }

        Pfn = MiGetPfnEntry(PFN_FROM_PDE(MiAddressToPde((PVOID)HYPER_SPACE)));
        if (MmIsAddressValid(Pfn))
        {
            Pfn->u2.ShareCount = 0;
            Pfn->u3.e2.ReferenceCount = 0;
        }

        Pfn = MiGetPfnEntry(PFN_FROM_PPE(MiAddressToPpe((PVOID)HYPER_SPACE)));
        if (MmIsAddressValid(Pfn))
        {
            Pfn->u2.ShareCount = 0;
            Pfn->u3.e2.ReferenceCount = 0;
        }

        Pfn = MiGetPfnEntry(PFN_FROM_PXE(MiAddressToPxe((PVOID)HYPER_SPACE)));
        if (MmIsAddressValid(Pfn))
        {
            Pfn->u2.ShareCount = 0;
            Pfn->u3.e2.ReferenceCount = 0;
        }

        Pfn = MiGetPfnEntry(PFN_FROM_PTE(MiAddressToPte(MmWorkingSetList)));
        if (MmIsAddressValid(Pfn))
        {
            Pfn->u2.ShareCount = 0;
            Pfn->u3.e2.ReferenceCount = 0;
        }
    }

    /* Initialize the nonpaged pool */
    InitializePool(NonPagedPool, 0);

    /* Initialize the bogus address space */
    Flags = 0;
    Status = MmInitializeProcessAddressSpace(PsGetCurrentProcess(), NULL, NULL, &Flags, NULL);
    if (!NT_SUCCESS(Status))
    {
        DPRINT1("MmInitializeProcessAddressSpace failed: 0x%lx\n", Status);
        return Status;
    }

    /* Initialize the balancer */
    MmInitializeBalancer((ULONG)MmAvailablePages, 0);

    _MM_UART("[MM] MiInitMachineDependent done\r\n");

    /* Sanity checks */
    ASSERT(MmPfnDatabase);
    ASSERT(MmNonPagedSystemStart);
    ASSERT(MmNonPagedPoolStart);
    ASSERT(MmSizeOfNonPagedPoolInBytes);
    ASSERT(MmMaximumNonPagedPoolInBytes);
    ASSERT(MmNonPagedPoolExpansionStart);
    ASSERT(MmHyperSpaceEnd);
    ASSERT(MmNumberOfSystemPtes);

    return STATUS_SUCCESS;
}

#else /* !_M_ARM64 — Original ARM32 stub */

/* GLOBALS ********************************************************************/

ULONG MmMaximumNonPagedPoolPercent;
ULONG MmSizeOfNonPagedPoolInBytes;
ULONG MmMaximumNonPagedPoolInBytes;
PVOID MmNonPagedSystemStart;
PVOID MmNonPagedPoolStart;
PVOID MmNonPagedPoolExpansionStart;
PVOID MmPagedPoolEnd;
PVOID MiSessionSpaceEnd;
PVOID MiSessionImageEnd;
PVOID MiSessionImageStart;
PVOID MiSessionViewStart;
PVOID MiSessionPoolEnd;
PVOID MiSessionPoolStart;
PVOID MmSessionBase;
ULONG MmSessionSize;
ULONG MmSessionViewSize;
ULONG MmSessionPoolSize;
ULONG MmSessionImageSize;
PVOID MiSystemViewStart;
ULONG MmSystemViewSize;
PFN_NUMBER MmSystemPageDirectory[PPE_PER_PAGE];
PMMPDE MmSystemPagePtes;
ULONG MmNumberOfSystemPtes;
ULONG MxPfnAllocation;
RTL_BITMAP MiPfnBitMap;
PPHYSICAL_MEMORY_DESCRIPTOR MmPhysicalMemoryBlock;
PMEMORY_ALLOCATION_DESCRIPTOR MxFreeDescriptor;
MEMORY_ALLOCATION_DESCRIPTOR MxOldFreeDescriptor;
ULONG MmNumberOfPhysicalPages, MmHighestPhysicalPage;
ULONG MmBootImageSize;
ULONG MmUserProbeAddress;
PVOID MmHighestUserAddress;
PVOID MmSystemRangeStart;
PVOID MmSystemCacheStart;
PVOID MmSystemCacheEnd;
MMSUPPORT MmSystemCacheWs;
PVOID MmHyperSpaceEnd;

/* PRIVATE FUNCTIONS **********************************************************/

CODE_SEG("INIT")
NTSTATUS
NTAPI
MiInitMachineDependent(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    //
    // Always return success for now
    //
    UNIMPLEMENTED_FATAL("NEVER TELL ME THE ODDS!\n");
    return STATUS_SUCCESS;
}

#endif /* _M_ARM64 */

/* EOF */
