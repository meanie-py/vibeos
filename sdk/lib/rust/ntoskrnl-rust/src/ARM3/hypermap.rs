use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    KeFlushProcessTb,
    PKIRQL,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// ValidKernelPteLocal is a copy of ValidKernelPte used by hyperspace code
extern "C" {
    static ValidKernelPteLocal: MMPTE;
}

const MI_HYPERSPACE_PTES: PFN_NUMBER = 0xFF; // internal/<arch>/mm.h (256 - 1)

#[no_mangle]
pub static mut MmFirstReservedMappingPte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MmLastReservedMappingPte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MiFirstReservedZeroingPte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut HyperTemplatePte: MMPTE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub unsafe extern "system" fn MiMapPageInHyperSpace(
    Process: PEPROCESS,
    Page: PFN_NUMBER,
    OldIrql: PKIRQL,
) -> PVOID {
    let Process = opaque_ptr(Process);
    let mut TempPte: MMPTE;
    let mut PointerPte: PMMPTE;
    let mut Offset: PFN_NUMBER;

    ntkrnl_sys::ASSERT!(Page != 0);
    ntkrnl_sys::ASSERT!(MiGetPfnEntry(Page) != core::ptr::null_mut());

    TempPte = ValidKernelPteLocal;
    TempPte.set_page_frame_number(Page);

    PointerPte = MmFirstReservedMappingPte;

    ntkrnl_sys::ASSERT!(Process == PsGetCurrentProcess());
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).HyperSpaceLock) as *mut usize,
        OldIrql,
    );

    Offset = PFN_FROM_PTE(PointerPte);
    if Offset == 0 {
        Offset = MI_HYPERSPACE_PTES;
        KeFlushProcessTb();
    }

    (*PointerPte).set_page_frame_number(Offset - 1);

    PointerPte = PointerPte.add(Offset as usize);
    MI_WRITE_VALID_PTE(PointerPte, TempPte);

    MiPteToAddress(PointerPte)
}

#[no_mangle]
pub unsafe extern "system" fn MiUnmapPageInHyperSpace(
    Process: PEPROCESS,
    Address: PVOID,
    OldIrql: KIRQL,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    ntkrnl_sys::ASSERT!(Process == PsGetCurrentProcess());

    (*MiAddressToPte(Address)).Long = 0;

    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == DISPATCH_LEVEL);
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*(Process as *mut ntkrnl_sys::ddk::_EPROCESS)).HyperSpaceLock) as *mut usize,
        OldIrql,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MiMapPagesInZeroSpace(
    mut Pfn1: PMMPFN,
    NumberOfPages: PFN_NUMBER,
) -> PVOID {
    let mut Pfn1 = opaque_ptr(Pfn1);
    let mut TempPte: MMPTE;
    let mut PointerPte: PMMPTE;
    let mut Offset: PFN_NUMBER;
    let PageFrameIndex: PFN_NUMBER;

    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);
    ntkrnl_sys::ASSERT!(NumberOfPages != 0);
    ntkrnl_sys::ASSERT!(NumberOfPages <= MI_ZERO_PTES as PFN_NUMBER);

    PointerPte = MiFirstReservedZeroingPte;

    Offset = PFN_FROM_PTE(PointerPte);
    if NumberOfPages > Offset {
        Offset = MI_ZERO_PTES as PFN_NUMBER;
        (*PointerPte).set_page_frame_number(Offset);
        KeFlushProcessTb();
    }

    (*PointerPte).set_page_frame_number(Offset - NumberOfPages);

    PointerPte = PointerPte.add((Offset + 1) as usize);
    TempPte = ValidKernelPte;

    // x86 zeroes through an uncached mapping to spare the cache; PCD/PWT keep
    // ordinary memory semantics. On arm64 MI_PAGE_DISABLE_CACHE selects DEVICE
    // memory, which is not that: RAM mapped Device while the same page is Normal
    // write-back elsewhere is a mismatched-attribute alias, and Device faults
    // `dc zva` (KeZeroPages) with an alignment fault, which a Raspberry Pi 4
    // raised on every boot (QEMU TCG does not check). Zero through Normal WB.
    #[cfg(not(target_arch = "aarch64"))]
    {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_THROUGH(&mut TempPte);
    }

    ntkrnl_sys::ASSERT!(Pfn1 as ULONG_PTR != LIST_HEAD);
    while Pfn1 as ULONG_PTR != LIST_HEAD {
        let PageFrameIndex = MiGetPfnEntryIndex(Pfn1);

        TempPte.set_page_frame_number(PageFrameIndex);

        PointerPte = PointerPte.sub(1);
        MI_WRITE_VALID_PTE(PointerPte, TempPte);

        Pfn1 = (*Pfn1).flink() as PMMPFN;
    }

    MiPteToAddress(PointerPte)
}

#[no_mangle]
pub unsafe extern "system" fn MiUnmapPagesInZeroSpace(
    VirtualAddress: PVOID,
    NumberOfPages: PFN_NUMBER,
) {
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let PointerPte: PMMPTE;

    ntkrnl_sys::ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);
    ntkrnl_sys::ASSERT!(NumberOfPages != 0);
    ntkrnl_sys::ASSERT!(NumberOfPages <= MI_ZERO_PTES as PFN_NUMBER);

    PointerPte = MiAddressToPte(VirtualAddress);

    RtlZeroMemory(
        PointerPte as PVOID,
        (NumberOfPages as usize) * core::mem::size_of::<MMPTE>(),
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/hypermap.c
//  * PURPOSE:         ARM Memory Manager Hyperspace Mapping Functionality
//  * PROGRAMMERS:     ReactOS Portable Systems Group
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
// /* GLOBALS ********************************************************************/
//
// PMMPTE MmFirstReservedMappingPte, MmLastReservedMappingPte;
// PMMPTE MiFirstReservedZeroingPte;
// MMPTE HyperTemplatePte;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// PVOID
// NTAPI
// MiMapPageInHyperSpace(IN PEPROCESS Process,
//                       IN PFN_NUMBER Page,
//                       IN PKIRQL OldIrql)
// {
//     MMPTE TempPte;
//     PMMPTE PointerPte;
//     PFN_NUMBER Offset;
//
//     //
//     // Never accept page 0 or non-physical pages
//     //
//     ASSERT(Page != 0);
//     ASSERT(MiGetPfnEntry(Page) != NULL);
//
//     //
//     // Build the PTE
//     //
//     TempPte = ValidKernelPteLocal;
//     TempPte.set_page_frame_number(Page);
//
//     //
//     // Pick the first hyperspace PTE
//     //
//     PointerPte = MmFirstReservedMappingPte;
//
//     //
//     // Acquire the hyperlock
//     //
//     ASSERT(Process == PsGetCurrentProcess());
//     KeAcquireSpinLock(&Process->HyperSpaceLock, OldIrql);
//
//     //
//     // Now get the first free PTE
//     //
//     Offset = PFN_FROM_PTE(PointerPte);
//     if (!Offset)
//     {
//         //
//         // Reset the PTEs
//         //
//         Offset = MI_HYPERSPACE_PTES;
//         KeFlushProcessTb();
//     }
//
//     //
//     // Prepare the next PTE
//     //
//     PointerPte->u.Hard.PageFrameNumber = Offset - 1;
//
//     //
//     // Write the current PTE
//     //
//     PointerPte += Offset;
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//     //
//     // Return the address
//     //
//     return MiPteToAddress(PointerPte);
// }
//
// VOID
// NTAPI
// MiUnmapPageInHyperSpace(IN PEPROCESS Process,
//                         IN PVOID Address,
//                         IN KIRQL OldIrql)
// {
//     ASSERT(Process == PsGetCurrentProcess());
//
//     //
//     // Blow away the mapping
//     //
//     MiAddressToPte(Address)->u.Long = 0;
//
//     //
//     // Release the hyperlock
//     //
//     ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL);
//     KeReleaseSpinLock(&Process->HyperSpaceLock, OldIrql);
// }
//
// PVOID
// NTAPI
// MiMapPagesInZeroSpace(IN PMMPFN Pfn1,
//                       IN PFN_NUMBER NumberOfPages)
// {
//     MMPTE TempPte;
//     PMMPTE PointerPte;
//     PFN_NUMBER Offset, PageFrameIndex;
//
//     //
//     // Sanity checks
//     //
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//     ASSERT(NumberOfPages != 0);
//     ASSERT(NumberOfPages <= MI_ZERO_PTES);
//
//     //
//     // Pick the first zeroing PTE
//     //
//     PointerPte = MiFirstReservedZeroingPte;
//
//     //
//     // Now get the first free PTE
//     //
//     Offset = PFN_FROM_PTE(PointerPte);
//     if (NumberOfPages > Offset)
//     {
//         //
//         // Reset the PTEs
//         //
//         Offset = MI_ZERO_PTES;
//         PointerPte->u.Hard.PageFrameNumber = Offset;
//         KeFlushProcessTb();
//     }
//
//     //
//     // Prepare the next PTE
//     //
//     PointerPte->u.Hard.PageFrameNumber = Offset - NumberOfPages;
//
//     /* Choose the correct PTE to use, and which template */
//     PointerPte += (Offset + 1);
//     TempPte = ValidKernelPte;
//
//     /* Disable cache. Write through */
//     MI_PAGE_DISABLE_CACHE(&TempPte);
//     MI_PAGE_WRITE_THROUGH(&TempPte);
//
//     /* Make sure the list isn't empty and loop it */
//     ASSERT(Pfn1 != (PVOID)LIST_HEAD);
//     while (Pfn1 != (PVOID)LIST_HEAD)
//     {
//         /* Get the page index for this PFN */
//         PageFrameIndex = MiGetPfnEntryIndex(Pfn1);
//
//         //
//         // Write the PFN
//         //
//         TempPte.set_page_frame_number(PageFrameIndex);
//
//         //
//         // Set the correct PTE to write to, and set its new value
//         //
//         PointerPte--;
//         MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//         /* Move to the next PFN */
//         Pfn1 = (PMMPFN)Pfn1->u1.Flink;
//     }
//
//     //
//     // Return the address
//     //
//     return MiPteToAddress(PointerPte);
// }
//
// VOID
// NTAPI
// MiUnmapPagesInZeroSpace(IN PVOID VirtualAddress,
//                         IN PFN_NUMBER NumberOfPages)
// {
//     PMMPTE PointerPte;
//
//     //
//     // Sanity checks
//     //
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//     ASSERT (NumberOfPages != 0);
//     ASSERT(NumberOfPages <= MI_ZERO_PTES);
//
//     //
//     // Get the first PTE for the mapped zero VA
//     //
//     PointerPte = MiAddressToPte(VirtualAddress);
//
//     //
//     // Blow away the mapped zero PTEs
//     //
//     RtlZeroMemory(PointerPte, NumberOfPages * sizeof(MMPTE));
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
