use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PEPROCESS;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    _EPROCESS,
    MiAcquireExpansionLock,
    MiReleaseExpansionLock,
};
use crate::ARM3::helpers::{
    MI_MAKE_DIRTY_PAGE,
    MmWorkingSetList,
    ValidKernelPteLocal,
    PDE_MAPPED_VA,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use crate::ARM3::largepag::MmProcessList;

/// i386-specific: MiAddressToPteOffset — within-page-table offset (0..1023)
/// C: `((ULONG_PTR)(Va) & (PDE_MAPPED_VA - 1)) / PAGE_SIZE`
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn MiAddressToPteOffset(va: PVOID) -> ULONG {
    (((va as ULONG_PTR) & (PDE_MAPPED_VA - 1)) / PAGE_SIZE) as ULONG
}

/// i386-specific: MiGetPdeOffset — page directory index (0..1023)
/// C: `MiAddressToPdeOffset(Va) = (ULONG_PTR)(Va) / PDE_MAPPED_VA`
#[cfg(target_arch = "x86")]
#[inline]
fn MiGetPdeOffset(va: ULONG_PTR) -> ULONG {
    (va / PDE_MAPPED_VA) as ULONG
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn MiArchCreateProcessAddressSpace(
    Process: PEPROCESS,
    DirectoryTableBase: PULONG_PTR,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let DirectoryTableBase = opaque_ptr(DirectoryTableBase);
    let PdeIndex = (*DirectoryTableBase.offset(0)) >> PAGE_SHIFT;
    let HyperIndex = (*DirectoryTableBase.offset(1)) >> PAGE_SHIFT;
    let mut PointerPte: PMMPTE;
    let mut PdePte: MMPTE = MMPTE { Long: 0 };
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let mut PteTable: PMMPTE = core::ptr::null_mut();
    let mut PdeOffset: ULONG = 0;
    let OldIrql: KIRQL;

    PointerPte = MiReserveSystemPtes(1, SystemPteSpace);
    if PointerPte.is_null() {
        return 0u8;
    }
    PteTable = MiPteToAddress(PointerPte) as PMMPTE;

    MI_MAKE_HARDWARE_PTE_KERNEL(&mut PdePte,
                                PointerPte,
                                MM_READWRITE,
                                HyperIndex);

    MI_MAKE_DIRTY_PAGE(&mut PdePte);
    MI_WRITE_VALID_PTE(PointerPte, PdePte);

    TempPte = ValidKernelPteLocal;
    TempPte.set_page_frame_number((*(Process as *mut _EPROCESS)).WorkingSetPage as ULONG_PTR);
    PdeOffset = MiAddressToPteOffset(MmWorkingSetList);
    *PteTable.offset(PdeOffset as isize) = TempPte;

    MI_MAKE_HARDWARE_PTE_KERNEL(&mut PdePte,
                                PointerPte,
                                MM_READWRITE,
                                PdeIndex);

    MI_MAKE_DIRTY_PAGE(&mut PdePte);
    *PointerPte = PdePte;
    core::arch::asm!("invlpg [{0}]", in(reg) PteTable, options(nostack));

    PteTable = MiPteToAddress(PointerPte) as PMMPTE;

    PdeOffset = MiGetPdeOffset(MmSystemRangeStart as ULONG_PTR);
    RtlCopyMemory(
        PteTable.offset(PdeOffset as isize) as PVOID,
        MiAddressToPde(MmSystemRangeStart) as PVOID,
        PAGE_SIZE - PdeOffset as usize * core::mem::size_of::<MMPTE>(),
    );

    TempPte = ValidKernelPteLocal;
    TempPte.set_page_frame_number(HyperIndex);
    PdeOffset = MiGetPdeOffset(HYPER_SPACE);
    *PteTable.offset(PdeOffset as isize) = TempPte;

    PdeOffset += 1;

    PdeOffset = MiGetPdeOffset(PTE_BASE as ULONG_PTR);
    TempPte.set_page_frame_number(PdeIndex);
    *PteTable.offset(PdeOffset as isize) = TempPte;

    MiReleaseSystemPtes(PointerPte, 1, SystemPteSpace);

    OldIrql = MiAcquireExpansionLock();
    InsertTailList(
        core::ptr::addr_of_mut!(MmProcessList) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).MmProcessLinks) as PLIST_ENTRY,
    );
    MiReleaseExpansionLock(OldIrql);

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD-3-Clause (https://spdx.org/licenses/BSD-3-Clause)
//  * FILE:            ntoskrnl/mm/i386/procsup.c
//  * PURPOSE:         Process handling for i386 architecture
//  * PROGRAMMERS:     Jérôme Gardou
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// BOOLEAN
// MiArchCreateProcessAddressSpace(
//     _In_ PEPROCESS Process,
//     _In_ PULONG_PTR DirectoryTableBase)
// {
//     PFN_NUMBER PdeIndex = DirectoryTableBase[0] >> PAGE_SHIFT;
//     PFN_NUMBER HyperIndex = DirectoryTableBase[1] >> PAGE_SHIFT;
//     PMMPTE PointerPte;
//     MMPTE PdePte, TempPte;
//     PMMPTE PteTable;
//     ULONG PdeOffset;
//     KIRQL OldIrql;
//
//     /* Get a PTE  */
//     PointerPte = MiReserveSystemPtes(1, SystemPteSpace);
//     if (!PointerPte)
//         return FALSE;
//     PteTable = MiPteToAddress(PointerPte);
//
//     /* Build a page table for hyper space */
//     MI_MAKE_HARDWARE_PTE_KERNEL(&PdePte,
//                                 PointerPte,
//                                 MM_READWRITE,
//                                 HyperIndex);
//
//     /* Set it dirty and map it */
//     MI_MAKE_DIRTY_PAGE(&PdePte);
//     MI_WRITE_VALID_PTE(PointerPte, PdePte);
//
//     /* Now write the PTE/PDE entry for the working set list index itself */
//     TempPte = ValidKernelPteLocal;
//     TempPte.u.Hard.PageFrameNumber = Process->WorkingSetPage;
//     PdeOffset = MiAddressToPteOffset(MmWorkingSetList);
//     PteTable[PdeOffset] = TempPte;
//
//     /* Now we map the page directory */
//     MI_MAKE_HARDWARE_PTE_KERNEL(&PdePte,
//                                 PointerPte,
//                                 MM_READWRITE,
//                                 PdeIndex);
//
//     /* Set it dirty and map it */
//     MI_MAKE_DIRTY_PAGE(&PdePte);
//     *PointerPte = PdePte;
//     /* We changed the page! */
//     __invlpg(PteTable);
//
//     /* Now get the page directory (which we'll double map, so call it a page table) */
//     PteTable = MiPteToAddress(PointerPte);
//
//     /* Copy all the kernel mappings */
//     PdeOffset = MiGetPdeOffset(MmSystemRangeStart);
//     RtlCopyMemory(&PteTable[PdeOffset],
//                   MiAddressToPde(MmSystemRangeStart),
//                   PAGE_SIZE - PdeOffset * sizeof(MMPTE));
//
//     /* Now write the PTE/PDE entry for hyperspace itself */
//     TempPte = ValidKernelPteLocal;
//     TempPte.u.Hard.PageFrameNumber = HyperIndex;
//     PdeOffset = MiGetPdeOffset(HYPER_SPACE);
//     PteTable[PdeOffset] = TempPte;
//
//     /* Sanity check */
//     PdeOffset++;
//     ASSERT(MiGetPdeOffset(MmHyperSpaceEnd) >= PdeOffset);
//
//     /* Now do the x86 trick of making the PDE a page table itself */
//     PdeOffset = MiGetPdeOffset(PTE_BASE);
//     TempPte.u.Hard.PageFrameNumber = PdeIndex;
//     PteTable[PdeOffset] = TempPte;
//
//     /* Let go of the system PTE */
//     MiReleaseSystemPtes(PointerPte, 1, SystemPteSpace);
//
//     /* Insert us into the Mm process list */
//     OldIrql = MiAcquireExpansionLock();
//     InsertTailList(&MmProcessList, &Process->MmProcessLinks);
//     MiReleaseExpansionLock(OldIrql);
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
