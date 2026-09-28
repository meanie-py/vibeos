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
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _MEMORY_ALLOCATION_DESCRIPTOR,
    _EPROCESS,
    _TYPE_OF_MEMORY,
    _TYPE_OF_MEMORY_LoaderMaximum,
    _TYPE_OF_MEMORY_LoaderXIPRom,
    _TYPE_OF_MEMORY_LoaderBad,
    _TYPE_OF_MEMORY_LoaderFree,
    _TYPE_OF_MEMORY_LoaderMemoryData,
    PMEMORY_ALLOCATION_DESCRIPTOR,
    PLOADER_PARAMETER_BLOCK,
    PMM_SESSION_SPACE,
    KPROCESSOR_MODE,
    KeFlushCurrentTb,
    MiInitializeColorTables,
    MiInitializeNonPagedPool,
    MiInitializeNonPagedPoolThresholds,
    MiIsMemoryTypeFree,
    MiIsMemoryTypeInvisible,
    MmInitializeBalancer,
    MmInitializeProcessAddressSpace,
    MmIsAddressValid,
};

use crate::ARM3::helpers::*;
use crate::ARM3::mminit::{
    MiSessionImagePteEnd,
    MiSessionImagePteStart,
    MiSessionPoolEnd,
    MiSessionPoolStart,
    MiSessionSpaceWs,
    MiSessionViewStart,
    MmDefaultMaximumNonPagedPool,
    MmHyperSpaceEnd,
    MmMaxAdditionNonPagedPoolPerMb,
    MmMaximumNonPagedPoolPercent,
    MmMinAdditionNonPagedPoolPerMb,
    MmMinimumNonPagedPoolSize,
    MmSessionImageSize,
    MmSessionPoolSize,
    MmSessionSize,
    MmSessionViewSize,
    MxFreeDescriptor,
    MxGetNextPage,
    MxOldFreeDescriptor,
};
use crate::ARM3::hypermap::{MmFirstReservedMappingPte, MmLastReservedMappingPte, MiFirstReservedZeroingPte, HyperTemplatePte};

extern "C" {
    fn CONTAINING_RECORD( address: PVOID, type_size: usize, field_offset: usize, ) -> PVOID;
}

// ============================================================================
// amd64-specific constants (from ntoskrnl/include/internal/amd64/mm.h)
// ============================================================================

const MI_SESSION_VIEW_SIZE: SIZE_T = 512 * _1MB;
const MI_SESSION_POOL_SIZE: SIZE_T = 64 * _1MB;
const MI_SESSION_IMAGE_SIZE: SIZE_T = 16 * _1MB;
const MI_SESSION_WORKING_SET_SIZE: SIZE_T = 16 * _1MB;
const MI_SESSION_SIZE: SIZE_T = MI_SESSION_VIEW_SIZE + MI_SESSION_POOL_SIZE + MI_SESSION_IMAGE_SIZE + MI_SESSION_WORKING_SET_SIZE;
const MI_SYSTEM_VIEW_SIZE: SIZE_T = 512 * _1MB;
const MI_SESSION_SPACE_END: ULONG_PTR = 0xFFFFF98000000000;

// Pool sizing constants (amd64 — 128 GB, not 128 MB like i386)
const MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING: PFN_NUMBER = (255 * _1MB) >> PAGE_SHIFT;
const MI_MAX_INIT_NONPAGED_POOL_SIZE: SIZE_T = 128 * 1024 * 1024 * 1024; // 128 GB
const MI_MAX_NONPAGED_POOL_SIZE: SIZE_T = 128 * 1024 * 1024 * 1024; // 128 GB
// internal/amd64/mm.h:79 — raised upstream from 22000 in 4cba65d760a (#8613).
const MI_NUMBER_SYSTEM_PTES: ULONG = 22000 * 22;

// Address layout constants
const PXE_SELFMAP: ULONG_PTR = 0xFFFFF6FB7DBEDF68;
const MM_SYSTEM_SPACE_START: ULONG_PTR = 0xFFFFF88000000000;
const MI_PFN_DATABASE: ULONG_PTR = 0xFFFFFA8000000000;
const MI_HIGHEST_SYSTEM_ADDRESS: ULONG_PTR = 0xFFFFFFFFFFFFFFFF;

// Hyperspace/working set constants — derived from the C defines
const MI_MAPPING_RANGE_START: ULONG_PTR = HYPER_SPACE;
const MI_MAPPING_RANGE_END: ULONG_PTR = MI_MAPPING_RANGE_START + MI_HYPERSPACE_PTES * PAGE_SIZE;
const MI_DUMMY_PTE: ULONG_PTR = MI_MAPPING_RANGE_END + PAGE_SIZE;
const MI_VAD_BITMAP: ULONG_PTR = MI_DUMMY_PTE + PAGE_SIZE;
const MI_WORKING_SET_LIST: ULONG_PTR = MI_VAD_BITMAP + PAGE_SIZE;

// MM_ALLOCATION_GRANULARITY — 64KB, same on all platforms
const MM_ALLOCATION_GRANULARITY: SIZE_T = 0x10000;

// ============================================================================
// Globals — template PTEs
// ============================================================================

#[no_mangle]
pub static mut ValidKernelPde: MMPTE = MMPTE {
    Long: PTE_VALID | PTE_EXECUTE_READWRITE | PTE_DIRTY | PTE_ACCESSED,
};

#[no_mangle]
pub static mut ValidKernelPte: MMPTE = MMPTE {
    Long: PTE_VALID | PTE_EXECUTE_READWRITE | PTE_DIRTY | PTE_ACCESSED,
};

#[no_mangle]
pub static mut ValidKernelPdeLocal: MMPTE = MMPTE {
    Long: PTE_VALID | PTE_EXECUTE_READWRITE | PTE_DIRTY | PTE_ACCESSED,
};

#[no_mangle]
pub static mut ValidKernelPteLocal: MMPTE = MMPTE {
    Long: PTE_VALID | PTE_EXECUTE_READWRITE | PTE_DIRTY | PTE_ACCESSED,
};

// DemandZeroPde — on amd64 this is MMPTE (not a separate MMPDE type)
#[no_mangle]
pub static mut DemandZeroPde: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut DemandZeroPte: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut MmDecommittedPte: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut PrototypePte: MMPTE = MMPTE { Long: 0 };

#[no_mangle]
pub static mut MiSessionViewEnd: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MiSystemPteSpaceStart: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MiSystemPteSpaceEnd: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MxPfnSizeInBytes: ULONG64 = 0;

#[no_mangle]
pub static mut MiIncludeType: [BOOLEAN; _TYPE_OF_MEMORY_LoaderMaximum as usize] = [0; _TYPE_OF_MEMORY_LoaderMaximum as usize];

#[no_mangle]
pub static mut MxFreePageBase: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MxFreePageCount: ULONG64 = 0;

#[no_mangle]
pub static mut MiPfnsInitialized: BOOLEAN = 0;

pub unsafe fn init_globals() {
    let kernel_pte_val: ULONG_PTR = PTE_VALID | PTE_EXECUTE_READWRITE | PTE_DIRTY | PTE_ACCESSED;
    ValidKernelPde.Long = kernel_pte_val;
    ValidKernelPte.Long = kernel_pte_val;
    ValidKernelPdeLocal.Long = kernel_pte_val;
    ValidKernelPteLocal.Long = kernel_pte_val;
    DemandZeroPde.Long = (MM_EXECUTE_READWRITE as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS;
    DemandZeroPte.Long = (MM_READWRITE as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS;
    PrototypePte.Long = ((MM_EXECUTE_READWRITE as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS)
        | PTE_PROTOTYPE
        | ((MI_PTE_LOOKUP_NEEDED as ULONG_PTR) << 32);
    MmDecommittedPte.Long = (MM_DECOMMIT as ULONG_PTR) << MM_PTE_SOFTWARE_PROTECTION_BITS;
}

// ============================================================================
// Helper: read/write CR4 via inline asm (not available in stable core::arch)
// ============================================================================

#[cfg(target_arch = "x86_64")]
unsafe fn read_cr4() -> u64 {
    let val: u64;
    core::arch::asm!("mov {}, cr4", out(reg) val, options(nostack, preserves_flags));
    val
}

#[cfg(target_arch = "x86_64")]
unsafe fn write_cr4(val: u64) {
    core::arch::asm!("mov cr4, {}", in(reg) val, options(nostack, preserves_flags));
}

// ============================================================================
// PFN_FROM_PXE / PFN_FROM_PPE / PFN_FROM_PDE — same as PFN_FROM_PTE
// ============================================================================

#[inline]
fn PFN_FROM_PXE(pte: PMMPTE) -> PFN_NUMBER {
    PFN_FROM_PTE(pte)
}

#[inline]
fn PFN_FROM_PPE(pte: PMMPTE) -> PFN_NUMBER {
    PFN_FROM_PTE(pte)
}

#[inline]
fn PFN_FROM_PDE(pte: PMMPTE) -> PFN_NUMBER {
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

// ============================================================================
// Session space layout
// ============================================================================

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitializeSessionSpaceLayout() {
    // Set the entire size
    MmSessionSize = MI_SESSION_SIZE;

    // Start with session space end
    MiSessionSpaceEnd = MI_SESSION_SPACE_END as PVOID;

    // The highest range is the session image range
    MmSessionImageSize = MI_SESSION_IMAGE_SIZE;
    MiSessionImageEnd = MiSessionSpaceEnd;
    MiSessionImageStart = ((MiSessionImageEnd as ULONG_PTR) - MmSessionImageSize) as PVOID;

    // Session working set is below the session image range
    MiSessionSpaceWs = ((MiSessionImageStart as ULONG_PTR) - MI_SESSION_WORKING_SET_SIZE) as PVOID;

    // Session view is below the session working set
    MmSessionViewSize = MI_SESSION_VIEW_SIZE;
    MiSessionViewEnd = MiSessionSpaceWs;
    MiSessionViewStart = ((MiSessionViewEnd as ULONG_PTR) - MmSessionViewSize) as PVOID;

    // Session pool is below session view
    MmSessionPoolSize = MI_SESSION_POOL_SIZE;
    MiSessionPoolEnd = MiSessionViewStart;
    MiSessionPoolStart = ((MiSessionPoolEnd as ULONG_PTR) - MmSessionPoolSize) as PVOID;

    // And it all begins here
    MmSessionBase = MiSessionPoolStart;

    // System view space
    MmSystemViewSize = MI_SYSTEM_VIEW_SIZE;
    MiSystemViewStart = ((MmSessionBase as ULONG_PTR) - MmSystemViewSize) as PVOID;

    // Compute the PTE addresses for all the addresses we carved out
    MiSessionImagePteStart = MiAddressToPte(MiSessionImageStart);
    MiSessionImagePteEnd = MiAddressToPte(MiSessionImageEnd);
    MiSessionBasePte = MiAddressToPte(MmSessionBase);
    MiSessionLastPte = MiAddressToPte(MiSessionSpaceEnd);

    // Initialize the pointer to the session space structure
    MmSessionSpace = ((MiSessionImageStart as ULONG_PTR) + 0x10000) as PVOID;
}

// ============================================================================
// Page table mapping functions
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiMapPPEs(start_address: PVOID, end_address: PVOID) {
    let start_address = opaque_ptr(start_address);
    let end_address = opaque_ptr(end_address);
    let mut pointer_ppe = MiAddressToPpe(start_address);
    let end_ppe = MiAddressToPpe(end_address);
    let mut tmpl_pde = ValidKernelPde;

    while pointer_ppe <= end_ppe {
        if !(*pointer_ppe).valid() {
            tmpl_pde.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(pointer_ppe, tmpl_pde);
            RtlZeroMemory(MiPteToAddress(pointer_ppe), PAGE_SIZE);
        }
        pointer_ppe = (pointer_ppe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiMapPDEs(start_address: PVOID, end_address: PVOID) {
    let start_address = opaque_ptr(start_address);
    let end_address = opaque_ptr(end_address);
    let mut pointer_pde = MiAddressToPde(start_address);
    let end_pde = MiAddressToPde(end_address);
    let mut tmpl_pde = ValidKernelPde;

    while pointer_pde <= end_pde {
        if !(*pointer_pde).valid() {
            tmpl_pde.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(pointer_pde, tmpl_pde);
            RtlZeroMemory(MiPteToAddress(pointer_pde), PAGE_SIZE);
        }
        pointer_pde = (pointer_pde as usize + core::mem::size_of::<MMPTE>()) as PMMPDE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiMapPTEs(start_address: PVOID, end_address: PVOID) {
    let start_address = opaque_ptr(start_address);
    let end_address = opaque_ptr(end_address);
    let mut pointer_pte = MiAddressToPte(start_address);
    let end_pte = MiAddressToPte(end_address);
    let mut tmpl_pte = ValidKernelPte;

    while pointer_pte <= end_pte {
        if !(*pointer_pte).valid() {
            tmpl_pte.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(pointer_pte, tmpl_pte);
            RtlZeroMemory(MiPteToAddress(pointer_pte), PAGE_SIZE);
        }
        pointer_pte = (pointer_pte as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }
}

// ============================================================================
// Page table initialization
// ============================================================================

unsafe fn MiInitializePageTable() {
    let pxe_pfn = (*(PXE_SELFMAP as PMMPTE)).page_frame_number();
    let pxe_physical_address = (pxe_pfn as ULONG64) << PAGE_SHIFT;

    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] = pxe_physical_address as u64;

    // Enable global pages (CR4.PGE = bit 7)
    #[cfg(target_arch = "x86_64")]
    {
        let cr4 = read_cr4();
        write_cr4(cr4 | 0x80);
    }

    // Zero out user-mode PXEs
    let mut pointer_pxe = MiAddressToPxe(core::ptr::null_mut());
    let end_pxe = MiAddressToPxe(MmHighestUserAddress);

    while pointer_pxe <= end_pxe {
        (*pointer_pxe).Long = 0;
        pointer_pxe = (pointer_pxe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }

    KeFlushCurrentTb();

    // Set up a template PTE for hyperspace
    let mut tmpl_pte = MMPTE { Long: 0 };
    tmpl_pte.set_valid(true);
    tmpl_pte.Long |= PTE_READWRITE; // Write bit
    HyperTemplatePte = tmpl_pte;

    // Create PDPTs for shared system address space
    pointer_pxe = MiAddressToPxe(HYPER_SPACE as PVOID);
    let end_system_pxe = MiAddressToPxe(MI_HIGHEST_SYSTEM_ADDRESS as PVOID);

    while pointer_pxe <= end_system_pxe {
        if !(*pointer_pxe).valid() {
            tmpl_pte.set_page_frame_number(MxGetNextPage(1));
            *pointer_pxe = tmpl_pte;
            RtlZeroMemory(MiPteToAddress(pointer_pxe), PAGE_SIZE);
        }
        pointer_pxe = (pointer_pxe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
    }

    let pxe_pfn_hyper = PFN_FROM_PXE(MiAddressToPxe(HYPER_SPACE as PVOID));
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[1] = (pxe_pfn_hyper << PAGE_SHIFT) as u64;

    // Map PPEs for paged pool
    MiMapPPEs(MmPagedPoolStart, MmPagedPoolEnd);

    // Setup 1 PPE for hyper space
    MiMapPPEs(HYPER_SPACE as PVOID, HYPER_SPACE_END as PVOID);

    // Setup PPEs for system space view
    MiMapPPEs(MiSystemViewStart, (MiSystemViewStart as usize + MmSystemViewSize) as PVOID);

    // Setup the mapping PDEs
    MiMapPDEs(MI_MAPPING_RANGE_START as PVOID, MI_MAPPING_RANGE_END as PVOID);

    // Setup the mapping PTEs
    MmFirstReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_START as PVOID);
    MmLastReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_END as PVOID);
    (*MmFirstReservedMappingPte).set_page_frame_number(MI_HYPERSPACE_PTES);

    // Setup debug mapping PTE
    MiMapPPEs(MI_DEBUG_MAPPING as PVOID, MI_DEBUG_MAPPING as PVOID);
    MiMapPDEs(MI_DEBUG_MAPPING as PVOID, MI_DEBUG_MAPPING as PVOID);
    MmDebugPte = MiAddressToPte(MI_DEBUG_MAPPING as PVOID);

    // Setup PDE and PTEs for VAD bitmap and working set list
    MiMapPDEs(MI_VAD_BITMAP as PVOID, (MI_WORKING_SET_LIST + PAGE_SIZE - 1) as PVOID);
    MiMapPTEs(MI_VAD_BITMAP as PVOID, (MI_WORKING_SET_LIST + PAGE_SIZE - 1) as PVOID);
}

// ============================================================================
// Non-paged pool
// ============================================================================

unsafe fn MiBuildNonPagedPool() {
    if MmNumberOfPhysicalPages <= MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING && MmSizeOfNonPagedPoolInBytes == 0 {
        MmSizeOfNonPagedPoolInBytes = 2 * 1024 * 1024;
    }

    if (MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT) > (MmNumberOfPhysicalPages * 7 / 8) {
        MmSizeOfNonPagedPoolInBytes = 0;
    }

    if MmSizeOfNonPagedPoolInBytes < MmMinimumNonPagedPoolSize {
        MmSizeOfNonPagedPoolInBytes = MmMinimumNonPagedPoolSize;
        MmSizeOfNonPagedPoolInBytes += (MmNumberOfPhysicalPages - 1024) / 256 * (MmMinAdditionNonPagedPoolPerMb as SIZE_T);
    }

    if MmSizeOfNonPagedPoolInBytes > MI_MAX_INIT_NONPAGED_POOL_SIZE {
        MmSizeOfNonPagedPoolInBytes = MI_MAX_INIT_NONPAGED_POOL_SIZE;
    }

    if MmMaximumNonPagedPoolPercent != 0 {
        panic!("UNIMPLEMENTED: MmMaximumNonPagedPoolPercent");
    }

    MmSizeOfNonPagedPoolInBytes &= !(PAGE_SIZE - 1);

    if MmMaximumNonPagedPoolInBytes == 0 {
        MmMaximumNonPagedPoolInBytes = MmDefaultMaximumNonPagedPool;
        MmMaximumNonPagedPoolInBytes += (MmNumberOfPhysicalPages - 1024) / 256 * (MmMaxAdditionNonPagedPoolPerMb as SIZE_T);
    }

    if MmMaximumNonPagedPoolInBytes > MI_MAX_NONPAGED_POOL_SIZE {
        MmMaximumNonPagedPoolInBytes = MI_MAX_NONPAGED_POOL_SIZE;
    }

    MmMaximumNonPagedPoolInPages = MmMaximumNonPagedPoolInBytes >> PAGE_SHIFT;

    MmNonPagedPoolStart = (MmPfnDatabase as usize + MxPfnAllocation * PAGE_SIZE) as PVOID;

    MmNonPagedPoolExpansionStart = (MmNonPagedPoolStart as usize + MmSizeOfNonPagedPoolInBytes) as PVOID;

    MmNonPagedPoolEnd = (MmNonPagedPoolStart as usize + MmMaximumNonPagedPoolInBytes) as PVOID;

    MiMapPPEs(MmNonPagedPoolStart, MmNonPagedPoolEnd);
    MiMapPDEs(MmNonPagedPoolStart, MmNonPagedPoolEnd);

    MiMapPTEs(MmNonPagedPoolStart, (MmNonPagedPoolExpansionStart as usize - 1) as PVOID);

    MiInitializeNonPagedPool();
    MiInitializeNonPagedPoolThresholds();
}

// ============================================================================
// System PTE space
// ============================================================================

unsafe fn MiBuildSystemPteSpace() {
    MmNumberOfSystemPtes = MI_NUMBER_SYSTEM_PTES;
    let non_paged_system_size = ((MmNumberOfSystemPtes as SIZE_T) + 1) * PAGE_SIZE;

    MiSystemPteSpaceStart = MmNonPagedSystemStart;
    MiSystemPteSpaceEnd = (MiSystemPteSpaceStart as usize + non_paged_system_size) as PVOID;

    MiMapPPEs(MiSystemPteSpaceStart, MiSystemPteSpaceEnd);
    MiMapPDEs(MiSystemPteSpaceStart, MiSystemPteSpaceEnd);

    let pointer_pte = MiAddressToPte(MiSystemPteSpaceStart);
    MiInitializeSystemPtes(pointer_pte, MmNumberOfSystemPtes, SystemPteSpace);

    MiFirstReservedZeroingPte = MiReserveSystemPtes(MI_ZERO_PTES + 1, SystemPteSpace);
    RtlZeroMemory(
        MiFirstReservedZeroingPte as PVOID,
        ((MI_ZERO_PTES as usize) + 1) * core::mem::size_of::<MMPTE>(),
    );

    (*MiFirstReservedZeroingPte).set_page_frame_number(MI_ZERO_PTES as PFN_NUMBER);
}

// ============================================================================
// PFN database setup
// ============================================================================

unsafe fn MiSetupPfnForPageTable(page_frame_index: PFN_NUMBER, pointer_pte: PMMPTE) {
    let pfn = MiGetPfnEntry(page_frame_index);

    if page_frame_index <= MmHighestPhysicalPage
        && MmIsAddressValid(pfn as PVOID) != 0
        && (*pfn).page_location() == ActiveAndValid
    {
        (*pfn).set_flink(0); // u1.WsIndex = 0
        (*pfn).set_share_count((*pfn).share_count() + 1);
        (*pfn).set_pte_address(pointer_pte);
        // OriginalPte = *pointer_pte — write MMPTE value into OriginalPte field
        // OriginalPte is at offset 32 on 64-bit (after PteAddress + u2 + u3)
        {
            let orig_pte_offset = 32usize; // offset of OriginalPte in MMPFN on 64-bit
            let dst = (pfn as *mut u8).add(orig_pte_offset) as *mut MMPTE;
            *dst = *pointer_pte;
        }
        (*pfn).set_page_location(ActiveAndValid);
        (*pfn).set_cache_attribute(MiNonCached as u16);
        (*pfn).set_reference_count(1);
        (*pfn).set_pte_frame(PFN_FROM_PTE(MiAddressToPte(pointer_pte as PVOID)));
    }

    let pointer_pde = MiAddressToPde(MiPteToAddress(pointer_pte));
    let pfn_pde = MiGetPfnEntry(PFN_FROM_PTE(pointer_pde));
    (*pfn_pde).set_share_count((*pfn_pde).share_count() + 1);
}

unsafe fn MiBuildPfnDatabaseFromPageTables() {
    let mut address = core::ptr::null_mut::<u8>();

    let page_frame_index = PFN_FROM_PTE(MiAddressToPte(PXE_BASE as PVOID));

    let pfn = MiGetPfnEntry(page_frame_index);
    (*pfn).set_flink(0); // u1.WsIndex = 0
    (*pfn).set_share_count(1);
    (*pfn).set_pte_address(core::ptr::null_mut());
    (*pfn).set_cache_attribute(MiNonCached as u16);
    (*pfn).set_reference_count(1);
    (*pfn).set_pte_frame(0);

    {
        let mut pointer_pxe = MiAddressToPxe(address as PVOID);
        for _i in 0..PXE_PER_PAGE {
            if !(*pointer_pxe).valid() {
                pointer_pxe = (pointer_pxe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
                continue;
            }

            let page_frame_index = PFN_FROM_PXE(pointer_pxe);
            MiSetupPfnForPageTable(page_frame_index, pointer_pxe);

            address = MiPxeToAddress(pointer_pxe) as *mut u8;

            let mut pointer_ppe = MiAddressToPpe(address as PVOID);
            for _j in 0..PPE_PER_PAGE {
                if !(*pointer_ppe).valid() {
                    pointer_ppe = (pointer_ppe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
                    continue;
                }

                let page_frame_index = PFN_FROM_PPE(pointer_ppe);
                MiSetupPfnForPageTable(page_frame_index, pointer_ppe);

                address = MiPpeToAddress(pointer_ppe) as *mut u8;

                let mut pointer_pde = MiAddressToPde(address as PVOID);
                for _k in 0..PDE_PER_PAGE {
                    if !(*pointer_pde).valid() {
                        pointer_pde = (pointer_pde as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
                        continue;
                    }

                    let page_frame_index = PFN_FROM_PDE(pointer_pde);
                    MiSetupPfnForPageTable(page_frame_index, pointer_pde);

                    address = MiPdeToAddress(pointer_pde) as *mut u8;

                    let mut pointer_pte = MiAddressToPte(address as PVOID);
                    for _l in 0..PTE_PER_PAGE {
                        if !(*pointer_pte).valid() {
                            pointer_pte = (pointer_pte as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
                            continue;
                        }

                        let page_frame_index = PFN_FROM_PTE(pointer_pte);
                        MiSetupPfnForPageTable(page_frame_index, pointer_pte);

                        pointer_pte = (pointer_pte as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
                    }

                    pointer_pde = (pointer_pde as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
                }

                pointer_ppe = (pointer_ppe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
            }

            pointer_pxe = (pointer_pxe as usize + core::mem::size_of::<MMPTE>()) as PMMPTE;
        }
    }
}

unsafe fn MiAddDescriptorToDatabase(
    base_page: PFN_NUMBER,
    page_count: PFN_NUMBER,
    memory_type: _TYPE_OF_MEMORY,
) {
    if MiIsMemoryTypeFree(memory_type) != 0 {
        let mut pfn = (MmPfnDatabase as usize
            + (base_page + page_count - 1) * core::mem::size_of::<MMPFN>())
            as PMMPFN;
        let mut count = page_count;

        while count > 0 {
            count -= 1;
            (*pfn).set_cache_attribute(MiNonCached as u16);
            MiInsertPageInFreeList(base_page + count);
            pfn = (pfn as usize - core::mem::size_of::<MMPFN>()) as PMMPFN;
        }
    } else if memory_type == _TYPE_OF_MEMORY_LoaderXIPRom {
        let mut pfn = (MmPfnDatabase as usize + base_page * core::mem::size_of::<MMPFN>()) as PMMPFN;
        let mut count = page_count;

        while count > 0 {
            count -= 1;
            (*pfn).set_pte_address(core::ptr::null_mut());
            (*pfn).set_flink(0); // u1.Flink = 0
            (*pfn).set_share_count(0); // u2.ShareCount = 0
            (*pfn).set_page_location(0); // u3.e1.PageLocation = 0
            (*pfn).set_cache_attribute(MiNonCached as u16);
            // u3.e1.Rom = 1, u3.e1.PrototypePte = 1 (C: amd64/init.c ROM-page loop).
            // Was raw `pfn+26 |= 1<<4` — bit 4 is PageColor bit 0, NOT Rom (bit 14);
            // use the typed MMPFNENTRY accessors (found by scan_magic_offsets audit).
            (*pfn).set_rom(true);
            (*pfn).set_prototype_pte(true);
            (*pfn).set_reference_count(0);
            (*pfn).set_pte_frame(0); // u4.PteFrame = 0 (also clears InPageError)

            pfn = (pfn as usize + core::mem::size_of::<MMPFN>()) as PMMPFN;
        }
    } else if memory_type == _TYPE_OF_MEMORY_LoaderBad {
        panic!("FIXME: LoaderBad");
    } else {
        let mut pfn = (MmPfnDatabase as usize + base_page * core::mem::size_of::<MMPFN>()) as PMMPFN;
        let mut count = page_count;

        while count > 0 {
            count -= 1;
            (*pfn).set_page_location(ActiveAndValid);
            pfn = (pfn as usize + core::mem::size_of::<MMPFN>()) as PMMPFN;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiBuildPfnDatabase(loader_block: PLOADER_PARAMETER_BLOCK) {
    let loader_block = opaque_ptr(loader_block);
    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

    MiMapPPEs(
        MmPfnDatabase as PVOID,
        (MmPfnDatabase as usize + MxPfnAllocation * PAGE_SIZE - 1) as PVOID,
    );

    MiMapPDEs(
        MmPfnDatabase as PVOID,
        (MmPfnDatabase as usize + MxPfnAllocation * PAGE_SIZE - 1) as PVOID,
    );

    MiInitializeColorTables();

    let mut list_entry = (*loader_block).MemoryDescriptorListHead.Flink;
    let list_head = core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY;

    while list_entry != list_head {
        let descriptor = CONTAINING_RECORD(
            list_entry as PVOID,
            core::mem::size_of::<_MEMORY_ALLOCATION_DESCRIPTOR>(),
            0,
        ) as PMEMORY_ALLOCATION_DESCRIPTOR;

        if MiIsMemoryTypeInvisible((*descriptor).MemoryType) == 0 {
            let mut descriptor_ptr = descriptor;
            if descriptor == MxFreeDescriptor {
                descriptor_ptr = core::ptr::addr_of_mut!(MxOldFreeDescriptor);
            }

            let base_page = (*descriptor_ptr).BasePage;
            let page_count = (*descriptor_ptr).PageCount;

            MiMapPTEs(
                (MmPfnDatabase as usize + (base_page as usize) * core::mem::size_of::<MMPFN>()) as PVOID,
                (MmPfnDatabase as usize + ((base_page + page_count) as usize) * core::mem::size_of::<MMPFN>() - 1) as PVOID,
            );

            if descriptor_ptr != core::ptr::addr_of_mut!(MxOldFreeDescriptor) as PMEMORY_ALLOCATION_DESCRIPTOR {
                MiAddDescriptorToDatabase(base_page as usize, page_count as usize, (*descriptor_ptr).MemoryType);
            }
        }

        list_entry = (*list_entry).Flink;
    }

    let base_page = (*MxFreeDescriptor).BasePage;
    let page_count = (*MxFreeDescriptor).PageCount;
    MiAddDescriptorToDatabase(base_page as usize, page_count as usize, _TYPE_OF_MEMORY_LoaderFree);

    let base_page = MxOldFreeDescriptor.BasePage;
    let page_count = (*MxFreeDescriptor).BasePage - base_page;
    MiAddDescriptorToDatabase(base_page as usize, page_count as usize, _TYPE_OF_MEMORY_LoaderMemoryData);

    *MxFreeDescriptor = MxOldFreeDescriptor;

    MiBuildPfnDatabaseFromPageTables();

    MiPfnsInitialized = 1;

    drop(pfn_guard);
}

// ============================================================================
// Main init entry point
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiInitMachineDependent(loader_block: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let loader_block = opaque_ptr(loader_block);
    // Initialize template PTEs that have zero static initializers in Rust
    // (in C these are compile-time initialized, but Rust requires runtime init)
    init_globals();

    MmHyperSpaceEnd = HYPER_SPACE_END as PVOID;
    MmNonPagedSystemStart = MM_SYSTEM_SPACE_START as PVOID;
    MmPfnDatabase = MI_PFN_DATABASE as PMMPFN;
    MmWorkingSetList = MI_WORKING_SET_LIST as PVOID;

    MiInitializePageTable();

    MiBuildNonPagedPool();

    MiBuildSystemPteSpace();

    MiBuildPfnDatabase(loader_block);

    let mut pfn = MiGetPfnEntry(PFN_FROM_PTE(PXE_SELFMAP as PMMPTE));
    (*pfn).set_share_count(0);
    (*pfn).set_reference_count(0);

    pfn = MiGetPfnEntry(PFN_FROM_PDE(MiAddressToPde(HYPER_SPACE as PVOID)));
    (*pfn).set_share_count(0);
    (*pfn).set_reference_count(0);

    pfn = MiGetPfnEntry(PFN_FROM_PPE(MiAddressToPpe(HYPER_SPACE as PVOID)));
    (*pfn).set_share_count(0);
    (*pfn).set_reference_count(0);

    pfn = MiGetPfnEntry(PFN_FROM_PXE(MiAddressToPxe(HYPER_SPACE as PVOID)));
    (*pfn).set_share_count(0);
    (*pfn).set_reference_count(0);

    pfn = MiGetPfnEntry(PFN_FROM_PTE(MiAddressToPte(MmWorkingSetList)));
    (*pfn).set_share_count(0);
    (*pfn).set_reference_count(0);

    InitializePool(NonPagedPool, 0);

    let mut flags = 0u32;
    let status = MmInitializeProcessAddressSpace(
        PsGetCurrentProcess() as *mut _EPROCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(flags),
        core::ptr::null_mut(),
    );

    if status < 0 {
        return status;
    }

    MmInitializeBalancer(MmAvailablePages as ULONG, 0);

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, See COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            ntoskrnl/mm/amd64/init.c
//  * PURPOSE:         Memory Manager Initialization for amd64
//  *
//  * PROGRAMMERS:     Timo kreuzer (timo.kreuzer@reactos.org)
//  *                  ReactOS Portable Systems Group
//  */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
