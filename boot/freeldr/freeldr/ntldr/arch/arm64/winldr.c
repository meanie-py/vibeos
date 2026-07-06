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

#define ARM64_PTE_ATTR_DEVICE   (1ULL << 2)     /* Device-nGnRnE (MAIR Attr1) */

/* Standard page entry for kernel memory */
#define ARM64_KERNEL_PAGE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_PAGE | ARM64_PTE_AF | \
                                 ARM64_PTE_SH_INNER | ARM64_PTE_AP_RW | ARM64_PTE_ATTR_NORMAL)

/* Device MMIO page entry (Device-nGnRnE via MAIR Attr1, no shareability bits) */
#define ARM64_DEVICE_PAGE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_PAGE | ARM64_PTE_AF | \
                                 ARM64_PTE_AP_RW | ARM64_PTE_ATTR_DEVICE)
/* Table descriptors (L0/L1/L2) include AF and SH_INNER even though ARM64 ignores
 * bits[11:2] for table descriptors. The kernel's recursive self-map reinterprets
 * table descriptors as L3 page descriptors where AF IS checked — without it, the
 * hardware raises an access flag fault when walking the self-map. */
#define ARM64_KERNEL_TABLE_FLAGS (ARM64_PTE_VALID | ARM64_PTE_TABLE | ARM64_PTE_AF | ARM64_PTE_SH_INNER)

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

    /* Self-mapping: PGD[0x1ED] points back to PGD itself.
     * This creates the recursive page table mapping at PTE_BASE
     * (0xFFFFF68000000000), mirroring the amd64 PXE self-map approach.
     *
     * IMPORTANT: Use PAGE flags (not just TABLE flags) because the self-map
     * entry is used as a table descriptor at L0/L1/L2 (where extra bits are
     * ignored) AND as a page descriptor at L3 (where AF must be set to
     * avoid an Access Flag fault). */
#define ARM64_PXE_SELFMAP_INDEX 0x1ED
    Arm64Pgd[ARM64_PXE_SELFMAP_INDEX] =
        ((ULONG64)Arm64Pgd & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;

    TRACE(">>> ARM64 PGD allocated at %p, self-map at index 0x%x\n",
          Arm64Pgd, ARM64_PXE_SELFMAP_INDEX);
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

    /* Check if already mapped — allow overwriting (may happen with
     * overlapping ranges or identity + kernel dual mapping) */
    if (Pte[Index] & ARM64_PTE_VALID)
    {
        Pte[Index] = (PhysicalAddress & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;
        return TRUE;
    }

    /* Create the page entry */
    Pte[Index] = (PhysicalAddress & ~0xFFFULL) | ARM64_KERNEL_PAGE_FLAGS;

    return TRUE;
}

static
BOOLEAN
MempMapDevicePage(ULONG64 PhysicalAddress)
{
    PARM64_PTE Pud, Pmd, Pte;

    /* Identity-map one MMIO page with Device-nGnRnE attributes */
    Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(PhysicalAddress));
    if (!Pud)
        return FALSE;

    Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(PhysicalAddress));
    if (!Pmd)
        return FALSE;

    Pte = MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(PhysicalAddress));
    if (!Pte)
        return FALSE;

    Pte[ARM64_PTE_INDEX(PhysicalAddress)] =
        (PhysicalAddress & ~0xFFFULL) | ARM64_DEVICE_PAGE_FLAGS;

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

    /* Kernel mapping: map the same physical pages at KSEG0_BASE offset */
    if (KernelMapping)
    {
        if (MempMapRangeOfPages(StartPage * PAGE_SIZE + KSEG0_BASE,
                                StartPage * PAGE_SIZE,
                                NumberOfPages) != NumberOfPages)
        {
            ERR("Failed to kernel-map pages\n");
            return FALSE;
        }
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
    ULONG64 tcr, mair;
    ULONG64 ttbr_val = (ULONG64)Arm64Pgd;

    ERR(">>> WinLdrSetProcessorContext for ARM64\n");
    ERR("  PGD at physical %p\n", Arm64Pgd);

    /* Mask all interrupts (IRQ, FIQ, SError, Debug) via DAIF */
    __asm__ __volatile__("msr daifset, #0xF");

    /* Configure MAIR_EL1: memory attribute indirection register
     * Attr0 = 0xFF (Normal WB-WA), Attr1 = 0x00 (Device-nGnRnE),
     * Attr2 = 0x44 (Normal NC) */
    mair = 0x00000000004400FFULL;
    __asm__ __volatile__("msr mair_el1, %0" :: "r"(mair));

    /* Configure TCR_EL1: translation control register
     * T0SZ=16, T1SZ=16 (48-bit VA), TG0=4KB, TG1=4KB,
     * Inner/Outer WB-WA cacheable, Inner Shareable */
    tcr = (16ULL << 0)  |  /* T0SZ = 16 */
          (16ULL << 16) |  /* T1SZ = 16 */
          (0ULL  << 14) |  /* TG0 = 4KB (0b00) */
          (2ULL  << 30) |  /* TG1 = 4KB (0b10) */
          (1ULL  << 8)  |  /* IRGN0 = WB-WA (0b01) */
          (1ULL  << 10) |  /* ORGN0 = WB-WA (0b01) */
          (3ULL  << 12) |  /* SH0 = Inner Shareable (0b11) */
          (1ULL  << 24) |  /* IRGN1 = WB-WA (0b01) */
          (1ULL  << 26) |  /* ORGN1 = WB-WA (0b01) */
          (3ULL  << 28) |  /* SH1 = Inner Shareable (0b11) */
          (2ULL  << 32);   /* IPS = 44-bit PA (0b010) */
    __asm__ __volatile__("msr tcr_el1, %0" :: "r"(tcr));

    /* Set TTBR0 and TTBR1 to our PGD (same table for both).
     * TTBR0 handles lower-half VAs (identity mapping at 0x40000000+).
     * TTBR1 handles upper-half VAs (kernel at KSEG0_BASE 0xFFFFF800+). */
    __asm__ __volatile__("msr ttbr0_el1, %0" :: "r"(ttbr_val));
    __asm__ __volatile__("msr ttbr1_el1, %0" :: "r"(ttbr_val));

    /* Full TLB invalidation + barriers */
    __asm__ __volatile__(
        "isb\n\t"
        "tlbi vmalle1\n\t"
        "dsb sy\n\t"
        "isb"
        ::: "memory"
    );

    ERR("<<< WinLdrSetProcessorContext done: TTBR0=TTBR1=%p TCR=0x%llx MAIR=0x%llx\n",
          Arm64Pgd, tcr, mair);
}

static
BOOLEAN
WinLdrMapSpecialPages(VOID)
{
    PARM64_PTE Pud, Pmd;

    /* Map KI_USER_SHARED_DATA */
    if (!MempMapSinglePage(KI_USER_SHARED_DATA, SharedUserDataPfn * PAGE_SIZE))
    {
        ERR("Could not map KI_USER_SHARED_DATA\n");
        return FALSE;
    }

    /* Pre-create page table hierarchy for HAL VA space (4MB at top of address space).
     * The HAL will fill in actual PTEs later, but the upper levels must exist. */
    Pud = MempGetOrCreatePageDir(Arm64Pgd, ARM64_PGD_INDEX(MM_HAL_VA_START));
    Pmd = MempGetOrCreatePageDir(Pud, ARM64_PUD_INDEX(MM_HAL_VA_START));
    MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(MM_HAL_VA_START));
    MempGetOrCreatePageDir(Pmd, ARM64_PMD_INDEX(MM_HAL_VA_START + 2 * 1024 * 1024));

    /* Identity-map the QEMU virt fw_cfg MMIO page (0x09020000, Device attrs)
     * so the kernel's Phase-0 tripwire-channel probe (ntoskrnl-rust
     * src/tripwire.rs) is safe regardless of what the UEFI memory map covered.
     * Non-fatal: without it the probe would fault, so only tripwire loses. */
    if (!MempMapDevicePage(0x09020000ULL))
        ERR("Could not map fw_cfg page (tripwire channel disabled)\n");

    return TRUE;
}

VOID
WinLdrSetupMachineDependent(PLOADER_PARAMETER_BLOCK LoaderBlock)
{
    PVOID SharedUserDataAddress = NULL;

    ERR(">>> WinLdrSetupMachineDependent for ARM64\n");

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

    ERR(">>> Allocating page tables...\n");

    /* Allocate page tables */
    if (MempAllocatePageTables() == FALSE)
    {
        ERR("Failed to allocate page tables\n");
        return;
    }

    ERR(">>> Mapping special pages...\n");

    /* Map special pages (KI_USER_SHARED_DATA, HAL VA space) */
    if (!WinLdrMapSpecialPages())
    {
        ERR("Failed to map special pages\n");
        return;
    }

    ERR("<<< WinLdrSetupMachineDependent done\n");
}

VOID
MempDump(VOID)
{
    ULONG i;

    TRACE("ARM64 Page Table Dump: PGD at %p\n", Arm64Pgd);
    if (!Arm64Pgd)
        return;

    for (i = 0; i < 512; i++)
    {
        if (Arm64Pgd[i] & ARM64_PTE_VALID)
        {
            TRACE("  PGD[0x%03x] = %016llx (PA=%016llx)\n",
                  i, Arm64Pgd[i], Arm64Pgd[i] & ~0xFFFULL);
        }
    }
}
