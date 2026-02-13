/*
 * PROJECT:         ReactOS Boot Loader
 * LICENSE:         GPL - See COPYING in the top level directory
 * FILE:            boot/freeldr/freeldr/ntldr/arch/arm64/winldr.c
 * PURPOSE:         ARM64 specific Windows Loader routines
 * PROGRAMMERS:     ReactOS ARM64 Port
 */

/* INCLUDES ***************************************************************/

#include <freeldr.h>
#include "../../winldr.h"

#include <debug.h>
DBG_DEFAULT_CHANNEL(WINDOWS);

/* GLOBALS ***************************************************************/

/* ARM64 uses 4KB pages with 4-level page tables (like x86-64) */
#define ARM64_PAGE_SHIFT    12
#define ARM64_PAGE_SIZE     (1 << ARM64_PAGE_SHIFT)

/* Page table entry indices */
#define ARM64_PGD_SHIFT     39
#define ARM64_PUD_SHIFT     30
#define ARM64_PMD_SHIFT     21
#define ARM64_PTE_SHIFT     12

#define ARM64_PTRS_PER_LEVEL    512

/* Get index for each level */
#define ARM64_PGD_INDEX(va) (((va) >> ARM64_PGD_SHIFT) & 0x1FF)
#define ARM64_PUD_INDEX(va) (((va) >> ARM64_PUD_SHIFT) & 0x1FF)
#define ARM64_PMD_INDEX(va) (((va) >> ARM64_PMD_SHIFT) & 0x1FF)
#define ARM64_PTE_INDEX(va) (((va) >> ARM64_PTE_SHIFT) & 0x1FF)

/* Page table entry attributes */
#define ARM64_PTE_VALID         (1ULL << 0)
#define ARM64_PTE_TABLE         (1ULL << 1)
#define ARM64_PTE_PAGE          (1ULL << 1)
#define ARM64_PTE_AF            (1ULL << 10)    /* Access Flag */
#define ARM64_PTE_SH_INNER      (3ULL << 8)     /* Inner Shareable */
#define ARM64_PTE_AP_RW         (0ULL << 6)     /* Read-Write */
#define ARM64_PTE_AP_RO         (1ULL << 7)     /* Read-Only */
#define ARM64_PTE_ATTR_NORMAL   (0ULL << 2)     /* Normal memory */

/* Standard page entry for kernel memory */
#define ARM64_KERNEL_PAGE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_PAGE | ARM64_PTE_AF | \
                                 ARM64_PTE_SH_INNER | ARM64_PTE_AP_RW | ARM64_PTE_ATTR_NORMAL)
#define ARM64_KERNEL_TABLE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_TABLE)

typedef ULONG64 ARM64_PTE;
typedef ARM64_PTE *PARM64_PTE;

/* Level 0 page table (PGD) */
PARM64_PTE Arm64Pgd;

/* Shared user data page frame number */
PFN_NUMBER SharedUserDataPfn;

/* FUNCTIONS **************************************************************/

static
BOOLEAN
MempAllocatePageTables(VOID)
{
    TRACE(">>> MempAllocatePageTables\n");

    /* Allocate a page for the PGD (Level 0) */
    Arm64Pgd = MmAllocateMemoryWithType(PAGE_SIZE, LoaderMemoryData);
    if (!Arm64Pgd)
    {
        ERR("Failed to allocate ARM64 PGD\n");
        return FALSE;
    }

    /* Zero the PGD */
    RtlZeroMemory(Arm64Pgd, PAGE_SIZE);

    TRACE(">>> ARM64 PGD allocated at %p\n", Arm64Pgd);
    return TRUE;
}

static
PARM64_PTE
MempGetOrCreatePageDir(PARM64_PTE ParentTable, ULONG Index)
{
    PARM64_PTE SubDir;
    ARM64_PTE Entry;

    if (!ParentTable)
        return NULL;

    Entry = ParentTable[Index];

    if (!(Entry & ARM64_PTE_VALID))
    {
        /* Allocate a new table */
        SubDir = MmAllocateMemoryWithType(PAGE_SIZE, LoaderMemoryData);
        if (!SubDir)
            return NULL;

        RtlZeroMemory(SubDir, PAGE_SIZE);

        /* Create table entry */
        ParentTable[Index] = ((ULONG64)SubDir & ~0xFFFULL) | ARM64_KERNEL_TABLE_FLAGS;
    }
    else
    {
        /* Extract physical address from entry */
        SubDir = (PARM64_PTE)(Entry & ~0xFFFULL);
    }

    return SubDir;
}

static
BOOLEAN
MempMapSinglePage(ULONG64 VirtualAddress, ULONG64 PhysicalAddress)
{
    PARM64_PTE Pud, Pmd, Pte;
    ULONG Index;

    /* Walk the page table hierarchy, creating tables as needed */
    Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(VirtualAddress));
    if (!Pud)
        return FALSE;

    Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(VirtualAddress));
    if (!Pmd)
        return FALSE;

    Pte = MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(VirtualAddress));
    if (!Pte)
        return FALSE;

    Index = ARM64_PTE_INDEX(VirtualAddress);

    /* Check if already mapped */
    if (Pte[Index] & ARM64_PTE_VALID)
    {
        WARN("Page at %p already mapped\n", (PVOID)VirtualAddress);
        return FALSE;
    }

    /* Create the page entry */
    Pte[Index] = (PhysicalAddress & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;

    return TRUE;
}

BOOLEAN
MempIsPageMapped(PVOID VirtualAddress)
{
    PARM64_PTE Pud, Pmd, Pte;
    ULONG64 Va = (ULONG64)VirtualAddress;

    if (!(Arm64Pgd[ARM64_PGD_INDEX(Va)] & ARM64_PTE_VALID))
        return FALSE;

    Pud = (PARM64_PTE)(Arm64Pgd[ARM64_PGD_INDEX(Va)] & ~0xFFFULL);
    if (!(Pud[ARM64_PUD_INDEX(Va)] & ARM64_PTE_VALID))
        return FALSE;

    Pmd = (PARM64_PTE)(Pud[ARM64_PUD_INDEX(Va)] & ~0xFFFULL);
    if (!(Pmd[ARM64_PMD_INDEX(Va)] & ARM64_PTE_VALID))
        return FALSE;

    Pte = (PARM64_PTE)(Pmd[ARM64_PMD_INDEX(Va)] & ~0xFFFULL);
    if (!(Pte[ARM64_PTE_INDEX(Va)] & ARM64_PTE_VALID))
        return FALSE;

    return TRUE;
}

static
PFN_NUMBER
MempMapRangeOfPages(ULONG64 VirtualAddress, ULONG64 PhysicalAddress, PFN_NUMBER cPages)
{
    PFN_NUMBER i;

    for (i = 0; i < cPages; i++)
    {
        if (!MempMapSinglePage(VirtualAddress, PhysicalAddress))
        {
            ERR("Failed to map page %ld from %p to %p\n",
                i, (PVOID)VirtualAddress, (PVOID)PhysicalAddress);
            return i;
        }
        VirtualAddress += PAGE_SIZE;
        PhysicalAddress += PAGE_SIZE;
    }
    return i;
}

BOOLEAN
MempSetupPaging(IN PFN_NUMBER StartPage,
                IN PFN_NUMBER NumberOfPages,
                IN BOOLEAN KernelMapping)
{
    TRACE(">>> MempSetupPaging(0x%lx, %ld, KernelMapping=%d)\n",
          StartPage, NumberOfPages, KernelMapping);

    /* Identity mapping */
    if (MempMapRangeOfPages(StartPage * PAGE_SIZE,
                            StartPage * PAGE_SIZE,
                            NumberOfPages) != NumberOfPages)
    {
        ERR("Failed to identity map pages %ld, %ld\n", StartPage, NumberOfPages);
        return FALSE;
    }

    /* Kernel mapping (if requested) */
    if (KernelMapping)
    {
        /* On ARM64, kernel addresses are in the upper half (TTBR1) */
        /* For now, we just do identity mapping */
        /* TODO: Implement proper KSEG0 equivalent for ARM64 */
    }

    return TRUE;
}

VOID
MempUnmapPage(PFN_NUMBER Page)
{
    /* TODO: Implement page unmapping */
    TRACE(">>> MempUnmapPage(%ld)\n", Page);
}

VOID
WinLdrSetProcessorContext(
    _In_ USHORT OperatingSystemVersion)
{
    TRACE("WinLdrSetProcessorContext for ARM64\n");

    /* Disable interrupts */
    /* On ARM64, this is done by masking IRQ/FIQ in DAIF */
    /* The actual implementation would use: __asm__ volatile("msr daifset, #3"); */

    /* Set the page table base register (TTBR0) */
    /* __asm__ volatile("msr ttbr0_el1, %0" :: "r"(Arm64Pgd)); */

    /* Invalidate TLBs */
    /* __asm__ volatile("tlbi vmalle1" ::: "memory"); */
    /* __asm__ volatile("dsb sy" ::: "memory"); */
    /* __asm__ volatile("isb" ::: "memory"); */

    /* Note: The actual implementation requires assembly intrinsics
     * that need to be added to the ARM64 headers */

    TRACE("Leave WinLdrSetProcessorContext\n");
}

VOID
WinLdrSetupMachineDependent(PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    PVOID SharedUserDataAddress = NULL;

    TRACE("WinLdrSetupMachineDependent for ARM64\n");

    /* Set machine type - ARM64 doesn't use ISA/EISA, just set a placeholder */
    LoaderBlock->u.Arm.MachineType = 0;

    /* Allocate 1 page for SharedUserData */
    SharedUserDataAddress = MmAllocateMemoryWithType(MM_PAGE_SIZE, LoaderStartupPcrPage);
    if (SharedUserDataAddress == NULL)
    {
        ERR("Can't allocate SharedUserData page.\n");
        return;
    }
    SharedUserDataPfn = (ULONG_PTR)SharedUserDataAddress >> MM_PAGE_SHIFT;
    RtlZeroMemory(SharedUserDataAddress, MM_PAGE_SIZE);

    /* Allocate page tables */
    if (MempAllocatePageTables() == FALSE)
    {
        ERR("Failed to allocate page tables\n");
        return;
    }

    /* Map special pages */
    /* TODO: Map KI_USER_SHARED_DATA and other special ARM64 pages */

    TRACE("Leave WinLdrSetupMachineDependent\n");
}

VOID
MempDump(VOID)
{
    /* Debug function to dump page table state */
}
