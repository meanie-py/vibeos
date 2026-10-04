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
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _EPROCESS,
    PEPROCESS,
    PLOADER_PARAMETER_BLOCK,
    PMM_SESSION_SPACE,
    KeFlushCurrentTb,
    MiInitializeColorTables,
    MiInitializeNonPagedPool,
    MiInitializeNonPagedPoolThresholds,
    MiInitializePfnDatabase,
    MiMapPfnDatabase,
    MmInitializeBalancer,
    MmInitializeProcessAddressSpace,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use crate::ARM3::helpers::*;
use crate::ARM3::mminit::{
    MiNumberOfFreePages,
    MiSessionImagePteEnd,
    MiSessionImagePteStart,
    MiSessionPoolEnd,
    MiSessionPoolStart,
    MmDefaultMaximumNonPagedPool,
    MmHyperSpaceEnd,
    MmMaxAdditionNonPagedPoolPerMb,
    MmMaximumNonPagedPoolPercent,
    MmMinAdditionNonPagedPoolPerMb,
    MmMinimumNonPagedPoolSize,
    MmSecondaryColors,
    MmSessionImageSize,
    MmSessionPoolSize,
    MmSessionSize,
    MmSessionViewSize,
    MmSystemCacheWorkingSetList,
    MxFreeDescriptor,
    MxGetNextPage,
    MxOldFreeDescriptor,
};
use crate::ARM3::hypermap::{MmFirstReservedMappingPte, MmLastReservedMappingPte, MiFirstReservedZeroingPte};

// ============================================================================
// i386-specific session layout constants (from ntoskrnl/include/internal/i386/mm.h)
// ============================================================================

const MI_SYSTEM_VIEW_SIZE: SIZE_T = 32 * _1MB;
const MI_SESSION_VIEW_SIZE: SIZE_T = 48 * _1MB;
const MI_SESSION_POOL_SIZE: SIZE_T = 16 * _1MB;
const MI_SESSION_IMAGE_SIZE: SIZE_T = 8 * _1MB;
const MI_SESSION_WORKING_SET_SIZE: SIZE_T = 4 * _1MB;
const MI_SESSION_SIZE: SIZE_T = MI_SESSION_VIEW_SIZE + MI_SESSION_POOL_SIZE + MI_SESSION_IMAGE_SIZE + MI_SESSION_WORKING_SET_SIZE;

// i386-specific nonpaged pool sizing constants
const MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING: PFN_NUMBER = (255 * _1MB) >> PAGE_SHIFT;
const MI_MAX_INIT_NONPAGED_POOL_SIZE: SIZE_T = 128 * _1MB;
const MI_MAX_NONPAGED_POOL_SIZE: SIZE_T = 128 * _1MB;

// PTE_PROTOTYPE bit (bit 10, same on x86 as miarm.h line 93)
const PTE_PROTOTYPE_BIT: ULONG_PTR = 0x400;

// WriteThrough bit in i386 PTE hardware entry (bit 3)
const PTE_WRITE_THROUGH_BIT: ULONG_PTR = 1 << 3;

// Misc address constants for i386 (from i386/mm.h)
// MI_MAPPING_RANGE_START = HYPER_SPACE (0xC0400000)
// MI_MAPPING_RANGE_END   = HYPER_SPACE + MI_HYPERSPACE_PTES * PAGE_SIZE
const MI_MAPPING_RANGE_START: ULONG_PTR = HYPER_SPACE;
const MI_MAPPING_RANGE_END: ULONG_PTR = MI_MAPPING_RANGE_START + MI_HYPERSPACE_PTES * PAGE_SIZE;
// MI_WORKING_SET_LIST = (MI_VAD_BITMAP + PAGE_SIZE)
// MI_VAD_BITMAP = (MI_DUMMY_PTE + PAGE_SIZE)
// MI_DUMMY_PTE = (MI_MAPPING_RANGE_END + PAGE_SIZE)
const MI_WORKING_SET_LIST: ULONG_PTR = MI_MAPPING_RANGE_END + 3 * PAGE_SIZE;

// MM_ALLOCATION_GRANULARITY — 64KB, same on all platforms
const MM_ALLOCATION_GRANULARITY: SIZE_T = 0x10000;

// ============================================================================
// Extern globals
// ============================================================================

// Minimal i386 KIPCR view — we only need the IDT field (at offset 0x38)
// KIPCR layout on i386:
//   NT_TIB union (28 bytes) + SelfPcr(4) + Prcb(4) + Irql(1)+pad(3)
//   + IRR(4) + IrrActive(4) + IDR(4) + KdVersionBlock(4) + IDT(4)
// IDT is at byte offset 28+4+4+4+4+4+4+4 = 56 = 0x38
#[repr(C)]
struct KIPCR_i386 {
    _nt_tib: [u8; 28],      // NT_TIB union: 7 × 4-byte ptrs
    _self_pcr: *mut u8,     // SelfPcr
    _prcb: *mut u8,         // Prcb
    _irql: u8,              // Irql
    _pad: [u8; 3],
    _irr: u32,              // IRR
    _irr_active: u32,       // IrrActive
    _idr: u32,              // IDR
    _kd_version: *mut u8,   // KdVersionBlock
    pub IDT: *mut u8,       // PKIDTENTRY IDT
}

/// Get PCR on i386 by casting the result of helpers::KeGetPcr() to KIPCR_i386.
/// helpers::KeGetPcr() reads FS:[0] on i386 and returns PVOID (the KIPCR address).
#[inline]
unsafe fn i386_get_pcr() -> *mut KIPCR_i386 {
    KeGetPcr() as *mut KIPCR_i386
}

extern "C" {
    static KiI386PentiumLockErrataPresent: BOOLEAN; // cpu.rs/cpu.c define it BOOLEAN; a ULONG read takes 3 bytes past it

    /// MmSubsectionBase — defined in mm/section.rs (extern static in C)
    static mut MmSubsectionBase: ULONG_PTR;
}

// ============================================================================
// Globals — template PTEs and PDEs
// ============================================================================

#[no_mangle]
pub static mut ValidKernelPde: MMPTE = MMPTE { Long: 0 };
#[no_mangle]
pub static mut ValidKernelPte: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut ValidKernelPdeLocal: MMPTE = MMPTE { Long: 0 };
#[no_mangle]
pub static mut ValidKernelPteLocal: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut DemandZeroPde: MMPTE = MMPTE { Long: 0 };
#[no_mangle]
pub static mut DemandZeroPte: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut MmDecommittedPte: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut PrototypePte: MMPTE = MMPTE { Long: 0 };

pub unsafe fn init_globals() {
    let kernel_pte_val: ULONG_PTR = PTE_VALID | PTE_READWRITE | PTE_DIRTY | PTE_ACCESSED;
    ValidKernelPde.Long = kernel_pte_val;
    ValidKernelPte.Long = kernel_pte_val;
    ValidKernelPdeLocal.Long = kernel_pte_val;
    ValidKernelPteLocal.Long = kernel_pte_val;
    DemandZeroPde.Long = (MM_READWRITE as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS;
    DemandZeroPte.Long = (MM_READWRITE as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS;
    PrototypePte.Long = ((MM_READWRITE as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS)
        | PTE_PROTOTYPE
        | (MI_PTE_LOOKUP_NEEDED << PAGE_SHIFT);
    MmDecommittedPte.Long = (MM_DECOMMIT as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS;
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitializeSessionSpaceLayout() {
    MmSessionSize = MI_SESSION_SIZE;
    MmSessionViewSize = MI_SESSION_VIEW_SIZE;
    MmSessionPoolSize = MI_SESSION_POOL_SIZE;
    MmSessionImageSize = MI_SESSION_IMAGE_SIZE;

    MmSystemViewSize = MI_SYSTEM_VIEW_SIZE;

    MiSessionImageEnd = PTE_BASE as PVOID;

    MiSessionImageStart = ((MiSessionImageEnd as ULONG_PTR) - MmSessionImageSize) as PVOID;

    MiSessionViewStart = ((MiSessionImageEnd as ULONG_PTR)
        - MmSessionImageSize
        - MI_SESSION_WORKING_SET_SIZE
        - MmSessionViewSize) as PVOID;

    MiSessionPoolEnd = MiSessionViewStart;
    MiSessionPoolStart = ((MiSessionPoolEnd as ULONG_PTR) - MmSessionPoolSize) as PVOID;

    MmSessionBase = MiSessionPoolStart;

    ntkrnl_sys::ASSERT!((MmSessionBase as ULONG_PTR) + MmSessionSize == PTE_BASE);

    MiSessionSpaceEnd = MiSessionImageEnd;

    MiSystemViewStart = ((MmSessionBase as ULONG_PTR) - MmSystemViewSize) as PVOID;

    MiSessionImagePteStart = MiAddressToPte(MiSessionImageStart);
    MiSessionImagePteEnd = MiAddressToPte(MiSessionImageEnd);
    MiSessionBasePte = MiAddressToPte(MmSessionBase);
    MiSessionSpaceWs = ((MiSessionViewStart as ULONG_PTR) + MmSessionViewSize) as PVOID;
    MiSessionLastPte = MiAddressToPte(MiSessionSpaceEnd);

    MmSessionSpace = ((MmSessionBase as ULONG_PTR)
        + MmSessionSize
        - MmSessionImageSize
        - MM_ALLOCATION_GRANULARITY) as PVOID;
}

// CODE_SEG("INIT")
unsafe fn MiComputeNonPagedPoolVa(free_pages: ULONG) {
    let pool_pages: PFN_NUMBER;

    if (MmNumberOfPhysicalPages <= MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING)
        && (MmSizeOfNonPagedPoolInBytes == 0)
    {
        MmSizeOfNonPagedPoolInBytes = 2 * _1MB;
    }

    MmHyperSpaceEnd = ((MmSystemCacheWorkingSetList as ULONG_PTR) - 1) as PVOID;

    if (MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT) > (free_pages * 7 / 8) as SIZE_T {
        MmSizeOfNonPagedPoolInBytes = 0;
    }

    if MmSizeOfNonPagedPoolInBytes < MmMinimumNonPagedPoolSize {
        MmSizeOfNonPagedPoolInBytes = MmMinimumNonPagedPoolSize;
        MmSizeOfNonPagedPoolInBytes +=
            ((free_pages as SIZE_T) - 1024) / 256 * (MmMinAdditionNonPagedPoolPerMb as SIZE_T);
    }

    if MmSizeOfNonPagedPoolInBytes > MI_MAX_INIT_NONPAGED_POOL_SIZE {
        MmSizeOfNonPagedPoolInBytes = MI_MAX_INIT_NONPAGED_POOL_SIZE;
    }

    if MmMaximumNonPagedPoolPercent != 0 {
        // UNIMPLEMENTED
    }

    MmSizeOfNonPagedPoolInBytes &= !(PAGE_SIZE - 1);

    if MmMaximumNonPagedPoolInBytes == 0 {
        MmMaximumNonPagedPoolInBytes = MmDefaultMaximumNonPagedPool;

        MmMaximumNonPagedPoolInBytes += PAGE_ALIGN(
            ((MmHighestPhysicalPage + 1) as SIZE_T) * core::mem::size_of::<ntkrnl_sys::mm::MMPFN>(),
        ) as SIZE_T;

        if (free_pages as SIZE_T) >= 0x1F000 {
            MmMaximumNonPagedPoolInBytes += ((free_pages as SIZE_T) - 1024) / 256
                * (MmMaxAdditionNonPagedPoolPerMb as SIZE_T / 2);
            if MmMaximumNonPagedPoolInBytes < MI_MAX_NONPAGED_POOL_SIZE {
                MmMaximumNonPagedPoolInBytes = MI_MAX_NONPAGED_POOL_SIZE;
            }
        } else {
            MmMaximumNonPagedPoolInBytes +=
                ((free_pages as SIZE_T) - 1024) / 256 * (MmMaxAdditionNonPagedPoolPerMb as SIZE_T);
        }
    }

    pool_pages = MmSizeOfNonPagedPoolInBytes
        + (PAGE_SIZE * 16)
        + PAGE_ALIGN(
            ((MmHighestPhysicalPage + 1) as SIZE_T) * core::mem::size_of::<ntkrnl_sys::mm::MMPFN>(),
        ) as SIZE_T;
    if MmMaximumNonPagedPoolInBytes < pool_pages {
        MmMaximumNonPagedPoolInBytes = pool_pages;
    }

    let pool_pages_max = MI_MAX_NONPAGED_POOL_SIZE * 2;

    if MmMaximumNonPagedPoolInBytes > pool_pages_max {
        MmMaximumNonPagedPoolInBytes = pool_pages_max;
    }

    if MmMaximumNonPagedPoolInBytes > MI_MAX_NONPAGED_POOL_SIZE {
        if MmSizeOfNonPagedPoolInBytes
            < (MmMaximumNonPagedPoolInBytes - MI_MAX_NONPAGED_POOL_SIZE)
        {
            if MmMaximumNonPagedPoolInBytes
                > (MmSizeOfNonPagedPoolInBytes + MI_MAX_NONPAGED_POOL_SIZE)
            {
                MmMaximumNonPagedPoolInBytes =
                    MmSizeOfNonPagedPoolInBytes + MI_MAX_NONPAGED_POOL_SIZE;
            }
        }
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitMachineDependent(
    loader_block: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let loader_block = opaque_ptr(loader_block);
    // Initialize template PTEs that have zero static initializers in Rust
    // (in C these are compile-time initialized, but Rust requires runtime init)
    init_globals();

    let mut page_frame_index: PFN_NUMBER;
    let mut start_pde: PMMPTE;
    let mut end_pde: PMMPTE;
    let mut pointer_pte: PMMPTE;
    let mut last_pte: PMMPTE;
    let mut temp_pde: MMPTE;
    let mut temp_pte: MMPTE;
    let mut old_irql: KIRQL;
    let mut pfn1: PMMPFN;
    let mut flags: ULONG;

    temp_pte = ValidKernelPte;
    temp_pde = ValidKernelPde;

    pointer_pte = MiAddressToPde(PDE_BASE as PVOID);
    page_frame_index = PFN_FROM_PTE(pointer_pte) << PAGE_SHIFT;
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] = page_frame_index as u32;

    start_pde = MiAddressToPde(core::ptr::null_mut());
    end_pde = MiAddressToPde(KSEG0_BASE as PVOID);
    RtlZeroMemory(
        start_pde as PVOID,
        ((end_pde as usize) - (start_pde as usize)) as SIZE_T,
    );

    MiComputeNonPagedPoolVa(MiNumberOfFreePages as ULONG);

    MmNonPagedPoolStart = ((MmNonPagedPoolEnd as ULONG_PTR)
        - MmMaximumNonPagedPoolInBytes
        + MmSizeOfNonPagedPoolInBytes) as PVOID;
    MmNonPagedPoolStart = PAGE_ALIGN(MmNonPagedPoolStart as ULONG_PTR) as PVOID;
    let non_paged_pool_expansion_va = MmNonPagedPoolStart;

    let non_paged_system_size = (MmNumberOfSystemPtes as SIZE_T + 1) * PAGE_SIZE;
    MmNonPagedSystemStart =
        ((MmNonPagedPoolStart as ULONG_PTR) - non_paged_system_size) as PVOID;
    MmNonPagedSystemStart =
        ((MmNonPagedSystemStart as ULONG_PTR) & !(PDE_MAPPED_VA - 1)) as PVOID;

    if (MmNonPagedSystemStart as ULONG_PTR) < 0xEB000000 {
        MmNonPagedSystemStart = 0xEB000000 as PVOID;

        MmNumberOfSystemPtes = (((MmNonPagedPoolStart as ULONG_PTR)
            - (MmNonPagedSystemStart as ULONG_PTR))
            >> PAGE_SHIFT) as ULONG;
        MmNumberOfSystemPtes -= 1;
        ntkrnl_sys::ASSERT!(MmNumberOfSystemPtes > 1000);
    }

    if MmSizeOfPagedPoolInBytes
        > ((MmNonPagedSystemStart as ULONG_PTR) - (MmPagedPoolStart as ULONG_PTR))
    {
        // Paged pool is too big!
    }

    MmPfnDatabase = 0xB0000000 as PMMPFN;
    ntkrnl_sys::ASSERT!(((MmPfnDatabase as ULONG_PTR) & (PDE_MAPPED_VA - 1)) == 0);

    MmNonPagedPoolStart =
        ((MmPfnDatabase as ULONG_PTR) + (MxPfnAllocation << PAGE_SHIFT)) as PVOID;

    page_frame_index = MxGetNextPage(
        MxPfnAllocation + (MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT),
    );
    ntkrnl_sys::ASSERT!(page_frame_index != 0);

    MmMaximumNonPagedPoolInPages = MmMaximumNonPagedPoolInBytes >> PAGE_SHIFT;

    // Trace 'A' = about to enter first PDE loop (NonPagedSystem)

    start_pde = MiAddressToPde(MmNonPagedSystemStart);
    end_pde = MiAddressToPde(((MmNonPagedPoolEnd as ULONG_PTR) - 1) as PVOID);

    // Dump start_pde, end_pde
    {
        let sp = start_pde as u32;
        let ep = end_pde as u32;
        for &byte in &sp.to_le_bytes() {
            let nibbles = [byte >> 4, byte & 0xf];
            for n in nibbles {
                let c = if n < 10 { b'0' + n } else { b'a' + n - 10 };
            }
        }
        for &byte in &ep.to_le_bytes() {
            let nibbles = [byte >> 4, byte & 0xf];
            for n in nibbles {
                let c = if n < 10 { b'0' + n } else { b'a' + n - 10 };
            }
        }
    }

    while start_pde <= end_pde {
        temp_pde.Long = ValidKernelPde.Long;
        temp_pde.set_page_frame_number(MxGetNextPage(1));
        MI_WRITE_VALID_PTE(start_pde, temp_pde);

        pointer_pte = MiPteToAddress(start_pde) as PMMPTE;
        RtlZeroMemory(pointer_pte as PVOID, PAGE_SIZE);

        start_pde = (start_pde as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }

    // Trace 'B' = first PDE loop done, entering second PDE loop (PfnDatabase)

    start_pde = MiAddressToPde(MmPfnDatabase as PVOID);
    end_pde = MiAddressToPde(
        ((MmNonPagedPoolStart as ULONG_PTR) + MmSizeOfNonPagedPoolInBytes - 1) as PVOID,
    );
    while start_pde <= end_pde {
        temp_pde.Long = ValidKernelPde.Long;
        temp_pde.set_page_frame_number(MxGetNextPage(1));
        MI_WRITE_VALID_PTE(start_pde, temp_pde);

        pointer_pte = MiPteToAddress(start_pde) as PMMPTE;
        RtlZeroMemory(pointer_pte as PVOID, PAGE_SIZE);

        start_pde = (start_pde as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }

    // Trace 'C' = second PDE loop done, entering PTE mapping loop

    MmSubsectionBase = MmNonPagedPoolStart as ULONG_PTR;

    MmNonPagedPoolExpansionStart = non_paged_pool_expansion_va;

    pointer_pte = MiAddressToPte(MmNonPagedPoolStart);
    last_pte = MiAddressToPte(
        ((MmNonPagedPoolStart as ULONG_PTR) + MmSizeOfNonPagedPoolInBytes - 1) as PVOID,
    );
    while pointer_pte <= last_pte {
        temp_pte.Long = ValidKernelPte.Long;
        temp_pte.set_page_frame_number(page_frame_index);
        page_frame_index += 1;
        MI_WRITE_VALID_PTE(pointer_pte, temp_pte);
        pointer_pte = (pointer_pte as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }

    ntkrnl_sys::ASSERT!(MiAddressToPte(MmNonPagedSystemStart) < MiAddressToPte(MmNonPagedPoolExpansionStart));

    // Trace 'D' = PTE mapping done
    MiInitializeNonPagedPool();
    MiInitializeNonPagedPoolThresholds();

    MiMapPfnDatabase(loader_block);

    MiInitializeColorTables();

    MiInitializePfnDatabase(loader_block);
    MmInitializeBalancer(MmAvailablePages as ULONG, 0);

    *MxFreeDescriptor = MxOldFreeDescriptor;

    InitializePool(NonPagedPool, 0);

    pointer_pte = MiAddressToPte(MmNonPagedSystemStart);
    MmNumberOfSystemPtes = (((MiAddressToPte(MmNonPagedPoolExpansionStart) as usize)
        - (pointer_pte as usize))
        / core::mem::size_of::<MMPTE>()) as ULONG;
    MmNumberOfSystemPtes -= 1;

    MiInitializeSystemPtes(pointer_pte, MmNumberOfSystemPtes, SystemPteSpace);

    // DBG: before MiAddressToPde(HYPER_SPACE)
    start_pde = MiAddressToPde(HYPER_SPACE as PVOID);
    // DBG: after MiAddressToPde, before MiAcquirePfnLock
    old_irql = MiAcquirePfnLock();
    // DBG: after MiAcquirePfnLock

    MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
    crate::ARM3::helpers::MI_SET_PROCESS2(b"Kernel\0".as_ptr() as PVOID);
    page_frame_index = MiRemoveAnyPage(0);
    temp_pde.Long = ValidKernelPdeLocal.Long;
    temp_pde.set_page_frame_number(page_frame_index);
    MI_WRITE_VALID_PTE(start_pde, temp_pde);
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[1] =
        (page_frame_index << PAGE_SHIFT) as u32;

    KeFlushCurrentTb();

    MiReleasePfnLock(old_irql);

    pointer_pte = MiAddressToPte(HYPER_SPACE as PVOID);
    RtlZeroMemory(pointer_pte as PVOID, PAGE_SIZE);
    // DBG: K+4 after RtlZeroMemory(hyperspace)

    MmFirstReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_START as PVOID);
    MmLastReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_END as PVOID);
    // Set PageFrameNumber field in the first reserved mapping PTE to MI_HYPERSPACE_PTES
    (*MmFirstReservedMappingPte).set_page_frame_number(MI_HYPERSPACE_PTES as PFN_NUMBER);

    MmWorkingSetList = MI_WORKING_SET_LIST as PVOID;
    // DBG: K+5 before MiReserveSystemPtes

    MiFirstReservedZeroingPte =
        MiReserveSystemPtes(MI_ZERO_PTES as ULONG + 1, SystemPteSpace);
    // DBG: K+6 after MiReserveSystemPtes
    RtlZeroMemory(
        MiFirstReservedZeroingPte as PVOID,
        (MI_ZERO_PTES as usize + 1) * core::mem::size_of::<MMPTE>(),
    );

    // Set PageFrameNumber field in the first zeroing PTE to MI_ZERO_PTES
    (*MiFirstReservedZeroingPte).set_page_frame_number(MI_ZERO_PTES as PFN_NUMBER);

    old_irql = MiAcquirePfnLock();

    pfn1 = MiGetPfnEntry(PFN_FROM_PTE(MiAddressToPde(PDE_BASE as PVOID)));
    (*pfn1).set_share_count(0);
    (*pfn1).set_reference_count(0);

    MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
    crate::ARM3::helpers::MI_SET_PROCESS2(b"Kernel WS List\0".as_ptr() as PVOID);
    page_frame_index = MiRemoveAnyPage(0);
    temp_pte.Long = ValidKernelPteLocal.Long;
    temp_pte.set_page_frame_number(page_frame_index);

    pointer_pte = MiAddressToPte(MmWorkingSetList);
    MI_WRITE_VALID_PTE(pointer_pte, temp_pte);

    RtlZeroMemory(MiPteToAddress(pointer_pte) as PVOID, PAGE_SIZE);
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).WorkingSetPage = page_frame_index as u32;

    if KiI386PentiumLockErrataPresent != 0 {
        pointer_pte = MiAddressToPte((*i386_get_pcr()).IDT as PVOID);
        // Set WriteThrough bit (bit 3) directly on the PTE's Long value
        (*pointer_pte).Long |= PTE_WRITE_THROUGH_BIT;
    }

    MiReleasePfnLock(old_irql);

    flags = 0;
    MmInitializeProcessAddressSpace(
        PsGetCurrentProcess() as PEPROCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(flags),
        core::ptr::null_mut(),
    );

    ntkrnl_sys::ASSERT!((MmFreePagesByColor[0] as ULONG_PTR) < PTE_BASE);
    start_pde = MiAddressToPde(MmFreePagesByColor[0] as PVOID);
    ntkrnl_sys::ASSERT!((*start_pde).valid());
    pointer_pte = MiAddressToPte(MmFreePagesByColor[0] as PVOID);
    ntkrnl_sys::ASSERT!((*pointer_pte).valid());
    last_pte = MiAddressToPte(
        ((MmFreePagesByColor[1].add(MmSecondaryColors as usize) as ULONG_PTR) - 1) as PVOID,
    );
    ntkrnl_sys::ASSERT!((*last_pte).valid());

    while pointer_pte <= last_pte {
        pfn1 = MiGetPfnEntry(PFN_FROM_PTE(pointer_pte));
        if (*pfn1).reference_count() == 0 {
            (*pfn1).set_pte_frame(PFN_FROM_PTE(start_pde));
            (*pfn1).set_pte_address(pointer_pte);
            (*pfn1).set_share_count((*pfn1).share_count() + 1);
            (*pfn1).set_reference_count(1);
            (*pfn1).set_page_location(ActiveAndValid);
            (*pfn1).set_cache_attribute(MiCached);
        }

        pointer_pte = (pointer_pte as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }

    STATUS_SUCCESS
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/i386/init.c
//  * PURPOSE:         ARM Memory Manager Initialization for x86
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
// /* Template PTE and PDE for a kernel page */
//  /* FIXME: These should be PTE_GLOBAL */
// MMPTE ValidKernelPde = {{PTE_VALID|PTE_READWRITE|PTE_DIRTY|PTE_ACCESSED}};
// MMPTE ValidKernelPte = {{PTE_VALID|PTE_READWRITE|PTE_DIRTY|PTE_ACCESSED}};
//
// /* The same, but for local pages */
// MMPTE ValidKernelPdeLocal = {{PTE_VALID|PTE_READWRITE|PTE_DIRTY|PTE_ACCESSED}};
// MMPTE ValidKernelPteLocal = {{PTE_VALID|PTE_READWRITE|PTE_DIRTY|PTE_ACCESSED}};
//
// /* Template PDE for a demand-zero page */
// MMPDE DemandZeroPde  = {{MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS}};
// MMPTE DemandZeroPte  = {{MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS}};
//
// /* Template PTE for prototype page */
// MMPTE PrototypePte = {{(MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS) |
//                       PTE_PROTOTYPE | (MI_PTE_LOOKUP_NEEDED << PAGE_SHIFT)}};
//
// /* Template PTE for decommited page */
// MMPTE MmDecommittedPte = {{MM_DECOMMIT << MM_PTE_SOFTWARE_PROTECTION_BITS}};
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiInitializeSessionSpaceLayout(VOID)
// {
//     //
//     // Set the size of session view, pool, and image
//     //
//     MmSessionSize = MI_SESSION_SIZE;
//     MmSessionViewSize = MI_SESSION_VIEW_SIZE;
//     MmSessionPoolSize = MI_SESSION_POOL_SIZE;
//     MmSessionImageSize = MI_SESSION_IMAGE_SIZE;
//
//     //
//     // Set the size of system view
//     //
//     MmSystemViewSize = MI_SYSTEM_VIEW_SIZE;
//
//     //
//     // This is where it all ends
//     //
//     MiSessionImageEnd = (PVOID)PTE_BASE;
//
//     //
//     // This is where we will load Win32k.sys and the video driver
//     //
//     MiSessionImageStart = (PVOID)((ULONG_PTR)MiSessionImageEnd -
//                                   MmSessionImageSize);
//
//     //
//     // So the view starts right below the session working set (itself below
//     // the image area)
//     //
//     MiSessionViewStart = (PVOID)((ULONG_PTR)MiSessionImageEnd -
//                                  MmSessionImageSize -
//                                  MI_SESSION_WORKING_SET_SIZE -
//                                  MmSessionViewSize);
//
//     //
//     // Session pool follows
//     //
//     MiSessionPoolEnd = MiSessionViewStart;
//     MiSessionPoolStart = (PVOID)((ULONG_PTR)MiSessionPoolEnd -
//                                  MmSessionPoolSize);
//
//     //
//     // And it all begins here
//     //
//     MmSessionBase = MiSessionPoolStart;
//
//     //
//     // Sanity check that our math is correct
//     //
//     ASSERT((ULONG_PTR)MmSessionBase + MmSessionSize == PTE_BASE);
//
//     //
//     // Session space ends wherever image session space ends
//     //
//     MiSessionSpaceEnd = MiSessionImageEnd;
//
//     //
//     // System view space ends at session space, so now that we know where
//     // this is, we can compute the base address of system view space itself.
//     //
//     MiSystemViewStart = (PVOID)((ULONG_PTR)MmSessionBase -
//                                 MmSystemViewSize);
//
//     /* Compute the PTE addresses for all the addresses we carved out */
//     MiSessionImagePteStart = MiAddressToPte(MiSessionImageStart);
//     MiSessionImagePteEnd = MiAddressToPte(MiSessionImageEnd);
//     MiSessionBasePte = MiAddressToPte(MmSessionBase);
//     MiSessionSpaceWs = (PVOID)((ULONG_PTR)MiSessionViewStart + MmSessionViewSize);
//     MiSessionLastPte = MiAddressToPte(MiSessionSpaceEnd);
//
//     /* Initialize session space */
//     MmSessionSpace = (PMM_SESSION_SPACE)((ULONG_PTR)MmSessionBase +
//                                          MmSessionSize -
//                                          MmSessionImageSize -
//                                          MM_ALLOCATION_GRANULARITY);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiComputeNonPagedPoolVa(IN ULONG FreePages)
// {
//     IN PFN_NUMBER PoolPages;
//
//     /* Check if this is a machine with less than 256MB of RAM, and no overide */
//     if ((MmNumberOfPhysicalPages <= MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING) &&
//         !(MmSizeOfNonPagedPoolInBytes))
//     {
//         /* Force the non paged pool to be 2MB so we can reduce RAM usage */
//         MmSizeOfNonPagedPoolInBytes = 2 * _1MB;
//     }
//
//     /* Hyperspace ends here */
//     MmHyperSpaceEnd = (PVOID)((ULONG_PTR)MmSystemCacheWorkingSetList - 1);
//
//     /* Check if the user gave a ridicuously large nonpaged pool RAM size */
//     if ((MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT) > (FreePages * 7 / 8))
//     {
//         /* More than 7/8ths of RAM was dedicated to nonpaged pool, ignore! */
//         MmSizeOfNonPagedPoolInBytes = 0;
//     }
//
//     /* Check if no registry setting was set, or if the setting was too low */
//     if (MmSizeOfNonPagedPoolInBytes < MmMinimumNonPagedPoolSize)
//     {
//         /* Start with the minimum (256 KB) and add 32 KB for each MB above 4 */
//         MmSizeOfNonPagedPoolInBytes = MmMinimumNonPagedPoolSize;
//         MmSizeOfNonPagedPoolInBytes += (FreePages - 1024) / 256 * MmMinAdditionNonPagedPoolPerMb;
//     }
//
//     /* Check if the registy setting or our dynamic calculation was too high */
//     if (MmSizeOfNonPagedPoolInBytes > MI_MAX_INIT_NONPAGED_POOL_SIZE)
//     {
//         /* Set it to the maximum */
//         MmSizeOfNonPagedPoolInBytes = MI_MAX_INIT_NONPAGED_POOL_SIZE;
//     }
//
//     /* Check if a percentage cap was set through the registry */
//     if (MmMaximumNonPagedPoolPercent) UNIMPLEMENTED;
//
//     /* Page-align the nonpaged pool size */
//     MmSizeOfNonPagedPoolInBytes &= ~(PAGE_SIZE - 1);
//
//     /* Now, check if there was a registry size for the maximum size */
//     if (!MmMaximumNonPagedPoolInBytes)
//     {
//         /* Start with the default (1MB) */
//         MmMaximumNonPagedPoolInBytes = MmDefaultMaximumNonPagedPool;
//
//         /* Add space for PFN database */
//         MmMaximumNonPagedPoolInBytes += (ULONG)
//             PAGE_ALIGN((MmHighestPhysicalPage +  1) * sizeof(MMPFN));
//
//         /* Check if the machine has more than 512MB of free RAM */
//         if (FreePages >= 0x1F000)
//         {
//             /* Add 200KB for each MB above 4 */
//             MmMaximumNonPagedPoolInBytes += (FreePages - 1024) / 256 *
//                                             (MmMaxAdditionNonPagedPoolPerMb / 2);
//             if (MmMaximumNonPagedPoolInBytes < MI_MAX_NONPAGED_POOL_SIZE)
//             {
//                 /* Make it at least 128MB since this machine has a lot of RAM */
//                 MmMaximumNonPagedPoolInBytes = MI_MAX_NONPAGED_POOL_SIZE;
//             }
//         }
//         else
//         {
//             /* Add 400KB for each MB above 4 */
//             MmMaximumNonPagedPoolInBytes += (FreePages - 1024) / 256 *
//                                             MmMaxAdditionNonPagedPoolPerMb;
//         }
//     }
//
//     /* Make sure there's at least 16 pages + the PFN available for expansion */
//     PoolPages = MmSizeOfNonPagedPoolInBytes + (PAGE_SIZE * 16) +
//                 ((ULONG)PAGE_ALIGN(MmHighestPhysicalPage + 1) * sizeof(MMPFN));
//     if (MmMaximumNonPagedPoolInBytes < PoolPages)
//     {
//         /* The maximum should be at least high enough to cover all the above */
//         MmMaximumNonPagedPoolInBytes = PoolPages;
//     }
//
//     /* Systems with 2GB of kernel address space get double the size */
//     PoolPages = MI_MAX_NONPAGED_POOL_SIZE * 2;
//
//     /* On the other hand, make sure that PFN + nonpaged pool doesn't get too big */
//     if (MmMaximumNonPagedPoolInBytes > PoolPages)
//     {
//         /* Trim it down to the maximum architectural limit (256MB) */
//         MmMaximumNonPagedPoolInBytes = PoolPages;
//     }
//
//     /* Check if this is a system with > 128MB of non paged pool */
//     if (MmMaximumNonPagedPoolInBytes > MI_MAX_NONPAGED_POOL_SIZE)
//     {
//         /* Check if the initial size is less than the extra 128MB boost */
//         if (MmSizeOfNonPagedPoolInBytes < (MmMaximumNonPagedPoolInBytes -
//                                            MI_MAX_NONPAGED_POOL_SIZE))
//         {
//             /* FIXME: Should check if the initial pool can be expanded */
//
//             /* Assume no expansion possible, check ift he maximum is too large */
//             if (MmMaximumNonPagedPoolInBytes > (MmSizeOfNonPagedPoolInBytes +
//                                                 MI_MAX_NONPAGED_POOL_SIZE))
//             {
//                 /* Set it to the initial value plus the boost */
//                 MmMaximumNonPagedPoolInBytes = MmSizeOfNonPagedPoolInBytes +
//                                                MI_MAX_NONPAGED_POOL_SIZE;
//             }
//         }
//     }
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// MiInitMachineDependent(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PFN_NUMBER PageFrameIndex;
//     PMMPTE StartPde, EndPde, PointerPte, LastPte;
//     MMPTE TempPde, TempPte;
//     PVOID NonPagedPoolExpansionVa;
//     SIZE_T NonPagedSystemSize;
//     KIRQL OldIrql;
//     PMMPFN Pfn1;
//     ULONG Flags;
//
// #if defined(_GLOBAL_PAGES_ARE_AWESOME_)
//
//     /* Check for global bit */
//     if (KeFeatureBits & KF_GLOBAL_PAGE)
//     {
//         /* Set it on the template PTE and PDE */
//         ValidKernelPte.u.Hard.Global = TRUE;
//         ValidKernelPde.u.Hard.Global = TRUE;
//     }
//
// #endif
//
//     /* Now templates are ready */
//     TempPte = ValidKernelPte;
//     TempPde = ValidKernelPde;
//
//     //
//     // Set CR3 for the system process
//     //
//     PointerPte = MiAddressToPde(PDE_BASE);
//     PageFrameIndex = PFN_FROM_PTE(PointerPte) << PAGE_SHIFT;
//     PsGetCurrentProcess()->Pcb.DirectoryTableBase[0] = PageFrameIndex;
//
//     //
//     // Blow away user-mode
//     //
//     StartPde = MiAddressToPde(0);
//     EndPde = MiAddressToPde(KSEG0_BASE);
//     RtlZeroMemory(StartPde, (EndPde - StartPde) * sizeof(MMPTE));
//
//     /* Compute non paged pool limits and size */
//     MiComputeNonPagedPoolVa(MiNumberOfFreePages);
//
//     //
//     // Now calculate the nonpaged pool expansion VA region
//     //
//     MmNonPagedPoolStart = (PVOID)((ULONG_PTR)MmNonPagedPoolEnd -
//                                   MmMaximumNonPagedPoolInBytes +
//                                   MmSizeOfNonPagedPoolInBytes);
//     MmNonPagedPoolStart = (PVOID)PAGE_ALIGN(MmNonPagedPoolStart);
//     NonPagedPoolExpansionVa = MmNonPagedPoolStart;
//     DPRINT("NP Pool has been tuned to: %lu bytes and %lu bytes\n",
//            MmSizeOfNonPagedPoolInBytes, MmMaximumNonPagedPoolInBytes);
//
//     //
//     // Now calculate the nonpaged system VA region, which includes the
//     // nonpaged pool expansion (above) and the system PTEs. Note that it is
//     // then aligned to a PDE boundary (4MB).
//     //
//     NonPagedSystemSize = (MmNumberOfSystemPtes + 1) * PAGE_SIZE;
//     MmNonPagedSystemStart = (PVOID)((ULONG_PTR)MmNonPagedPoolStart -
//                                     NonPagedSystemSize);
//     MmNonPagedSystemStart = (PVOID)((ULONG_PTR)MmNonPagedSystemStart &
//                                     ~(PDE_MAPPED_VA - 1));
//
//     //
//     // Don't let it go below the minimum
//     //
//     if (MmNonPagedSystemStart < (PVOID)0xEB000000)
//     {
//         //
//         // This is a hard-coded limit in the Windows NT address space
//         //
//         MmNonPagedSystemStart = (PVOID)0xEB000000;
//
//         //
//         // Reduce the amount of system PTEs to reach this point
//         //
//         MmNumberOfSystemPtes = ((ULONG_PTR)MmNonPagedPoolStart -
//                                 (ULONG_PTR)MmNonPagedSystemStart) >>
//                                 PAGE_SHIFT;
//         MmNumberOfSystemPtes--;
//         ASSERT(MmNumberOfSystemPtes > 1000);
//     }
//
//     //
//     // Check if we are in a situation where the size of the paged pool
//     // is so large that it overflows into nonpaged pool
//     //
//     if (MmSizeOfPagedPoolInBytes >
//         ((ULONG_PTR)MmNonPagedSystemStart - (ULONG_PTR)MmPagedPoolStart))
//     {
//         //
//         // We need some recalculations here
//         //
//         DPRINT1("Paged pool is too big!\n");
//     }
//
//     //
//     // Normally, the PFN database should start after the loader images.
//     // This is already the case in ReactOS, but for now we want to co-exist
//     // with the old memory manager, so we'll create a "Shadow PFN Database"
//     // instead, and arbitrarly start it at 0xB0000000.
//     //
//     MmPfnDatabase = (PVOID)0xB0000000;
//     ASSERT(((ULONG_PTR)MmPfnDatabase & (PDE_MAPPED_VA - 1)) == 0);
//
//     //
//     // Non paged pool comes after the PFN database
//     //
//     MmNonPagedPoolStart = (PVOID)((ULONG_PTR)MmPfnDatabase +
//                                   (MxPfnAllocation << PAGE_SHIFT));
//
//     //
//     // Now we actually need to get these many physical pages. Nonpaged pool
//     // is actually also physically contiguous (but not the expansion)
//     //
//     PageFrameIndex = MxGetNextPage(MxPfnAllocation +
//                                    (MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT));
//     ASSERT(PageFrameIndex != 0);
//     DPRINT("PFN DB PA PFN begins at: %lx\n", PageFrameIndex);
//     DPRINT("NP PA PFN begins at: %lx\n", PageFrameIndex + MxPfnAllocation);
//
//     /* Convert nonpaged pool size from bytes to pages */
//     MmMaximumNonPagedPoolInPages = MmMaximumNonPagedPoolInBytes >> PAGE_SHIFT;
//
//     //
//     // Now we need some pages to create the page tables for the NP system VA
//     // which includes system PTEs and expansion NP
//     //
//     StartPde = MiAddressToPde(MmNonPagedSystemStart);
//     EndPde = MiAddressToPde((PVOID)((ULONG_PTR)MmNonPagedPoolEnd - 1));
//     while (StartPde <= EndPde)
//     {
//         //
//         // Get a page
//         //
//         TempPde.u.Hard.PageFrameNumber = MxGetNextPage(1);
//         MI_WRITE_VALID_PTE(StartPde, TempPde);
//
//         //
//         // Zero out the page table
//         //
//         PointerPte = MiPteToAddress(StartPde);
//         RtlZeroMemory(PointerPte, PAGE_SIZE);
//
//         //
//         // Next
//         //
//         StartPde++;
//     }
//
//     //
//     // Now we need pages for the page tables which will map initial NP
//     //
//     StartPde = MiAddressToPde(MmPfnDatabase);
//     EndPde = MiAddressToPde((PVOID)((ULONG_PTR)MmNonPagedPoolStart +
//                                     MmSizeOfNonPagedPoolInBytes - 1));
//     while (StartPde <= EndPde)
//     {
//         //
//         // Get a page
//         //
//         TempPde.u.Hard.PageFrameNumber = MxGetNextPage(1);
//         MI_WRITE_VALID_PTE(StartPde, TempPde);
//
//         //
//         // Zero out the page table
//         //
//         PointerPte = MiPteToAddress(StartPde);
//         RtlZeroMemory(PointerPte, PAGE_SIZE);
//
//         //
//         // Next
//         //
//         StartPde++;
//     }
//
//     MmSubsectionBase = (ULONG_PTR)MmNonPagedPoolStart;
//
//     //
//     // Now remember where the expansion starts
//     //
//     MmNonPagedPoolExpansionStart = NonPagedPoolExpansionVa;
//
//     //
//     // Last step is to actually map the nonpaged pool
//     //
//     PointerPte = MiAddressToPte(MmNonPagedPoolStart);
//     LastPte = MiAddressToPte((PVOID)((ULONG_PTR)MmNonPagedPoolStart +
//                                      MmSizeOfNonPagedPoolInBytes - 1));
//     while (PointerPte <= LastPte)
//     {
//         //
//         // Use one of our contigous pages
//         //
//         TempPte.u.Hard.PageFrameNumber = PageFrameIndex++;
//         MI_WRITE_VALID_PTE(PointerPte++, TempPte);
//     }
//
//     //
//     // Sanity check: make sure we have properly defined the system PTE space
//     //
//     ASSERT(MiAddressToPte(MmNonPagedSystemStart) <
//            MiAddressToPte(MmNonPagedPoolExpansionStart));
//
//     /* Now go ahead and initialize the nonpaged pool */
//     MiInitializeNonPagedPool();
//     MiInitializeNonPagedPoolThresholds();
//
//     /* Map the PFN database pages */
//     MiMapPfnDatabase(LoaderBlock);
//
//     /* Initialize the color tables */
//     MiInitializeColorTables();
//
//     /* Build the PFN Database */
//     MiInitializePfnDatabase(LoaderBlock);
//     MmInitializeBalancer(MmAvailablePages, 0);
//
//     //
//     // Reset the descriptor back so we can create the correct memory blocks
//     //
//     *MxFreeDescriptor = MxOldFreeDescriptor;
//
//     //
//     // Initialize the nonpaged pool
//     //
//     InitializePool(NonPagedPool, 0);
//
//     //
//     // We PDE-aligned the nonpaged system start VA, so haul some extra PTEs!
//     //
//     PointerPte = MiAddressToPte(MmNonPagedSystemStart);
//     MmNumberOfSystemPtes = MiAddressToPte(MmNonPagedPoolExpansionStart) -
//                            PointerPte;
//     MmNumberOfSystemPtes--;
//     DPRINT("Final System PTE count: %lu (%lu bytes)\n",
//            MmNumberOfSystemPtes, MmNumberOfSystemPtes * PAGE_SIZE);
//
//     //
//     // Create the system PTE space
//     //
//     MiInitializeSystemPtes(PointerPte, MmNumberOfSystemPtes, SystemPteSpace);
//
//     /* Get the PDE For hyperspace */
//     StartPde = MiAddressToPde(HYPER_SPACE);
//
//     /* Lock PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Allocate a page for hyperspace and create it */
//     MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//     MI_SET_PROCESS2("Kernel");
//     PageFrameIndex = MiRemoveAnyPage(0);
//     TempPde = ValidKernelPdeLocal;
//     TempPde.u.Hard.PageFrameNumber = PageFrameIndex;
//     MI_WRITE_VALID_PTE(StartPde, TempPde);
//     PsGetCurrentProcess()->Pcb.DirectoryTableBase[1] = PageFrameIndex << PAGE_SHIFT;
//
//     /* Flush the TLB */
//     KeFlushCurrentTb();
//
//     /* Release the lock */
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // Zero out the page table now
//     //
//     PointerPte = MiAddressToPte(HYPER_SPACE);
//     RtlZeroMemory(PointerPte, PAGE_SIZE);
//
//     //
//     // Setup the mapping PTEs
//     //
//     MmFirstReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_START);
//     MmLastReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_END);
//     MmFirstReservedMappingPte->u.Hard.PageFrameNumber = MI_HYPERSPACE_PTES;
//
//     /* Set the working set address */
//     MmWorkingSetList = (PVOID)MI_WORKING_SET_LIST;
//
//     //
//     // Reserve system PTEs for zeroing PTEs and clear them
//     //
//     MiFirstReservedZeroingPte = MiReserveSystemPtes(MI_ZERO_PTES + 1,
//                                                     SystemPteSpace);
//     RtlZeroMemory(MiFirstReservedZeroingPte, (MI_ZERO_PTES + 1) * sizeof(MMPTE));
//
//     //
//     // Set the counter to maximum to boot with
//     //
//     MiFirstReservedZeroingPte->u.Hard.PageFrameNumber = MI_ZERO_PTES;
//
//     /* Lock PFN database */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Reset the ref/share count so that MmInitializeProcessAddressSpace works */
//     Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(MiAddressToPde(PDE_BASE)));
//     Pfn1->u2.ShareCount = 0;
//     Pfn1->u3.e2.ReferenceCount = 0;
//
//     /* Get a page for the working set list */
//     MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//     MI_SET_PROCESS2("Kernel WS List");
//     PageFrameIndex = MiRemoveAnyPage(0);
//     TempPte = ValidKernelPteLocal;
//     TempPte.u.Hard.PageFrameNumber = PageFrameIndex;
//
//     /* Map the working set list */
//     PointerPte = MiAddressToPte(MmWorkingSetList);
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//     /* Zero it out, and save the frame index */
//     RtlZeroMemory(MiPteToAddress(PointerPte), PAGE_SIZE);
//     PsGetCurrentProcess()->WorkingSetPage = PageFrameIndex;
//
//     /* Check for Pentium LOCK errata */
//     if (KiI386PentiumLockErrataPresent)
//     {
//         /* Mark the 1st IDT page as Write-Through to prevent a lockup
//            on a F00F instruction.
//            See https://www.rcollins.org/Errata/Dec97/F00FBug.html */
//         PointerPte = MiAddressToPte(KeGetPcr()->IDT);
//         PointerPte->u.Hard.WriteThrough = 1;
//     }
//
//     /* Release the lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Initialize the bogus address space */
//     Flags = 0;
//     MmInitializeProcessAddressSpace(PsGetCurrentProcess(), NULL, NULL, &Flags, NULL);
//
//     /* Make sure the color lists are valid */
//     ASSERT(MmFreePagesByColor[0] < (PMMCOLOR_TABLES)PTE_BASE);
//     StartPde = MiAddressToPde(MmFreePagesByColor[0]);
//     ASSERT(StartPde->u.Hard.Valid == 1);
//     PointerPte = MiAddressToPte(MmFreePagesByColor[0]);
//     ASSERT(PointerPte->u.Hard.Valid == 1);
//     LastPte = MiAddressToPte((ULONG_PTR)&MmFreePagesByColor[1][MmSecondaryColors] - 1);
//     ASSERT(LastPte->u.Hard.Valid == 1);
//
//     /* Loop the color list PTEs */
//     while (PointerPte <= LastPte)
//     {
//         /* Get the PFN entry */
//         Pfn1 = MiGetPfnEntry(PFN_FROM_PTE(PointerPte));
//         if (!Pfn1->u3.e2.ReferenceCount)
//         {
//             /* Fill it out */
//             Pfn1->u4.PteFrame = PFN_FROM_PTE(StartPde);
//             Pfn1->PteAddress = PointerPte;
//             Pfn1->u2.ShareCount++;
//             Pfn1->u3.e2.ReferenceCount = 1;
//             Pfn1->u3.e1.PageLocation = ActiveAndValid;
//             Pfn1->u3.e1.CacheAttribute = MiCached;
//         }
//
//         /* Keep going */
//         PointerPte++;
//     }
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
