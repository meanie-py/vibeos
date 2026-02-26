#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    PEPROCESS,
    KIRQL,
    PULONG_PTR,
    MiRemoveZeroPageSafe,
};

use crate::ARM3::helpers::*;
use crate::ARM3::mminit::MmHyperSpaceEnd;

// amd64-specific constants
const PXE_SELFMAP: ULONG_PTR = 0xFFFFF6FB7DBEDF68;

// ============================================================================
// Helper: PFN_FROM_PXE — same as PFN_FROM_PTE on hand-written MMPTE
// ============================================================================

#[inline]
fn PFN_FROM_PXE(pte: PMMPTE) -> PFN_NUMBER {
    PFN_FROM_PTE(pte)
}

// ============================================================================
// MiAddressToPxi / MiAddressToPti — page table index functions
// ============================================================================

#[inline]
fn MiAddressToPxi(address: PVOID) -> ULONG {
    (((address as u64) >> 39) & 0x1FF) as ULONG
}

#[inline]
fn MiAddressToPti(address: PVOID) -> ULONG {
    (((address as u64) >> 12) & 0x1FF) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn MiArchCreateProcessAddressSpace(
    Process: PEPROCESS,
    DirectoryTableBase: PULONG_PTR,
) -> BOOLEAN {
    let mut OldIrql: KIRQL;
    let TableBasePfn: PFN_NUMBER;
    let HyperPfn: PFN_NUMBER;
    let mut HyperPdPfn: PFN_NUMBER;
    let mut HyperPtPfn: PFN_NUMBER;
    let SystemPte: PMMPTE;
    let mut TempPte: MMPTE;
    let mut PdePte: MMPTE = MMPTE { Long: 0 };
    let mut TableIndex: ULONG;
    let PageTablePointer: PMMPTE;
    let mut PageColor: ULONG;

    TableBasePfn = (*DirectoryTableBase.offset(0) >> PAGE_SHIFT) as PFN_NUMBER;
    HyperPfn = (*DirectoryTableBase.offset(1) >> PAGE_SHIFT) as PFN_NUMBER;

    OldIrql = MiAcquirePfnLock();
    PageColor = MI_GET_NEXT_PROCESS_COLOR(Process as PVOID);
    HyperPdPfn = MiRemoveZeroPageSafe(PageColor) as PFN_NUMBER;
    if HyperPdPfn == 0 {
        HyperPdPfn = MiRemoveAnyPage(PageColor);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(HyperPdPfn);
        OldIrql = MiAcquirePfnLock();
    }
    PageColor = MI_GET_NEXT_PROCESS_COLOR(Process as PVOID);
    HyperPtPfn = MiRemoveZeroPageSafe(PageColor) as PFN_NUMBER;
    if HyperPtPfn == 0 {
        HyperPtPfn = MiRemoveAnyPage(PageColor);
        MiReleasePfnLock(OldIrql);
        MiZeroPhysicalPage(HyperPtPfn);
    } else {
        MiReleasePfnLock(OldIrql);
    }

    SystemPte = MiReserveSystemPtes(1, SystemPteSpace);
    if SystemPte.is_null() {
        return 0u8;
    }

    PageTablePointer = MiPteToAddress(SystemPte) as PMMPTE;

    MI_MAKE_HARDWARE_PTE_KERNEL(
        &mut PdePte,
        SystemPte,
        MM_READWRITE,
        TableBasePfn,
    );
    MI_WRITE_VALID_PTE(SystemPte, PdePte);

    TableIndex = (PXE_PER_PAGE / 2) as ULONG;
    core::ptr::write_bytes(
        PageTablePointer as *mut u8,
        0,
        (TableIndex as usize) * core::mem::size_of::<MMPTE>(),
    );
    core::ptr::copy_nonoverlapping(
        MiAddressToPxe(0 as PVOID).offset(TableIndex as isize) as *const u8,
        PageTablePointer.offset(TableIndex as isize) as *mut u8,
        PAGE_SIZE - (TableIndex as usize) * core::mem::size_of::<MMPTE>(),
    );

    TempPte = super::init::ValidKernelPte;

    TableIndex = MiAddressToPxi(PXE_SELFMAP as PVOID);
    TempPte.set_page_frame_number(TableBasePfn);
    *PageTablePointer.offset(TableIndex as isize) = TempPte;

    TableIndex = MiAddressToPxi(HYPER_SPACE as PVOID);
    TempPte.set_page_frame_number(HyperPfn);
    *PageTablePointer.offset(TableIndex as isize) = TempPte;

    PdePte.set_page_frame_number(HyperPfn);
    *SystemPte = PdePte;
    core::arch::asm!("invlpg [{}]", in(reg) PageTablePointer, options(nostack, preserves_flags));

    TempPte.set_page_frame_number(HyperPdPfn);
    *PageTablePointer.offset(0) = TempPte;

    PdePte.set_page_frame_number(HyperPdPfn);
    *SystemPte = PdePte;
    core::arch::asm!("invlpg [{}]", in(reg) PageTablePointer, options(nostack, preserves_flags));

    TempPte.set_page_frame_number(HyperPtPfn);
    *PageTablePointer.offset(0) = TempPte;

    PdePte.set_page_frame_number(HyperPtPfn);
    *SystemPte = PdePte;
    core::arch::asm!("invlpg [{}]", in(reg) PageTablePointer, options(nostack, preserves_flags));

    TempPte.set_page_frame_number((*Process).WorkingSetPage as usize);
    TableIndex = MiAddressToPti(MmWorkingSetList);
    *PageTablePointer.offset(TableIndex as isize) = TempPte;

    MiReleaseSystemPtes(SystemPte, 1, SystemPteSpace);

    return 1u8;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/mm/amd64/procsup.c
//  * PURPOSE:         Low level memory management manipulation
//  *
//  * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                  ReactOS Portable Systems Group
//  */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
