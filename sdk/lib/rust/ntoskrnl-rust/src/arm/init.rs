use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _EPROCESS,
    PEPROCESS,
    PLOADER_PARAMETER_BLOCK,
    PMEMORY_ALLOCATION_DESCRIPTOR,
    PMM_SESSION_SPACE,
    MEMORY_ALLOCATION_DESCRIPTOR,
    MiInitMachineDependent as MiInitMachineDependent_extern,
    MiInitializeColorTables,
    MiInitializeNonPagedPool,
    MiInitializeNonPagedPoolThresholds,
    MiInitializePfnDatabase,
    MmInitializeBalancer,
    MmInitializeProcessAddressSpace,
    MmIsAddressValid,
    KeFlushCurrentTb,
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
    MxFreeDescriptor,
    MxGetNextPage,
    MxOldFreeDescriptor,
};
use crate::ARM3::hypermap::{MmFirstReservedMappingPte, MmLastReservedMappingPte, MiFirstReservedZeroingPte, HyperTemplatePte};

// Extern "C" declaration for ARM64_UART_TRACE (no-op in C, provided by C shim)
extern "C" {
    fn ARM64_UART_TRACE(msg: PCHAR);
}

// ============================================================================
// ARM64-specific session layout constants (from ntoskrnl/include/internal/arm64/mm.h)
// ============================================================================

#[cfg(target_arch = "aarch64")]
const MI_SYSTEM_VIEW_SIZE: SIZE_T = 32 * _1MB;
#[cfg(target_arch = "aarch64")]
const MI_SESSION_VIEW_SIZE: SIZE_T = 48 * _1MB;
#[cfg(target_arch = "aarch64")]
const MI_SESSION_POOL_SIZE: SIZE_T = 16 * _1MB;
#[cfg(target_arch = "aarch64")]
const MI_SESSION_IMAGE_SIZE: SIZE_T = 8 * _1MB;
#[cfg(target_arch = "aarch64")]
const MI_SESSION_WORKING_SET_SIZE: SIZE_T = 4 * _1MB;
#[cfg(target_arch = "aarch64")]
const MI_SESSION_SIZE: SIZE_T = MI_SESSION_VIEW_SIZE + MI_SESSION_POOL_SIZE + MI_SESSION_IMAGE_SIZE + MI_SESSION_WORKING_SET_SIZE;

// ARM64-specific nonpaged pool sizing constants
#[cfg(target_arch = "aarch64")]
const MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING: PFN_NUMBER = (255 * _1MB) >> PAGE_SHIFT;
#[cfg(target_arch = "aarch64")]
const MI_MAX_INIT_NONPAGED_POOL_SIZE: SIZE_T = 128 * _1MB;
#[cfg(target_arch = "aarch64")]
const MI_MAX_NONPAGED_POOL_SIZE: SIZE_T = 128 * _1MB;

// ARM64 MI_NUMBER_SYSTEM_PTES = 22000
#[cfg(target_arch = "aarch64")]
const MI_NUMBER_SYSTEM_PTES: usize = 22000;

// PTE_NOT_LARGE_PAGE — bit 1 on ARM64 (Table/Page descriptor bit)
#[cfg(target_arch = "aarch64")]
const PTE_NOT_LARGE_PAGE: ULONG_PTR = 0x2;

// MI_PTE_LOOKUP_NEEDED — value that goes in high 32 bits of PrototypePte
#[cfg(target_arch = "aarch64")]
const MI_PTE_LOOKUP_NEEDED: ULONG_PTR = 0xFFFF;

// Misc address constants for ARM64 (from arm64/mm.h)
#[cfg(target_arch = "aarch64")]
const MI_MAPPING_RANGE_START: ULONG_PTR = HYPER_SPACE;
#[cfg(target_arch = "aarch64")]
const MI_MAPPING_RANGE_END: ULONG_PTR = MI_MAPPING_RANGE_START + MI_HYPERSPACE_PTES * PAGE_SIZE;
#[cfg(target_arch = "aarch64")]
const MI_DUMMY_PTE: ULONG_PTR = MI_MAPPING_RANGE_END + PAGE_SIZE;
#[cfg(target_arch = "aarch64")]
const MI_VAD_BITMAP: ULONG_PTR = MI_DUMMY_PTE + PAGE_SIZE;
#[cfg(target_arch = "aarch64")]
const MI_WORKING_SET_LIST: ULONG_PTR = MI_VAD_BITMAP + PAGE_SIZE;
#[cfg(target_arch = "aarch64")]
const MI_DEBUG_MAPPING: ULONG_PTR = 0xFFFFFB0000000000;
#[cfg(target_arch = "aarch64")]
const MI_HIGHEST_SYSTEM_ADDRESS: ULONG_PTR = 0xFFFFFFFFFFFFFFFF;

// PXE_SELFMAP: self-referencing PXE entry
#[cfg(target_arch = "aarch64")]
const PXE_SELFMAP: usize = PXE_BASE + ((PXE_BASE >> 39) & 0x1FF) * 8;

/// MiAddressToPxeOffset — (VA >> 39) & 0x1FF
#[cfg(target_arch = "aarch64")]
#[inline]
fn MiAddressToPxeOffset(va: PVOID) -> ULONG {
    (((va as ULONG_PTR) >> 39) & 0x1FF) as ULONG
}

// ============================================================================
// ARM64 Template PTE / PDE globals
// ============================================================================

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut ValidKernelPde: MMPTE = unsafe { core::mem::transmute(
    PTE_VALID as ULONG_PTR | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED
) };
// Inner Shareable (SH = 0b11, bits [9:8]): without it every page the memory
// manager maps is Non-shareable, and cross-core atomics on it are not atomic on
// real hardware (see MmProtectToPteMask in ke/arm64/stubs.rs).
#[cfg(target_arch = "aarch64")]
const PTE_SH_INNER: ULONG_PTR = 0x300;
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut ValidKernelPte: MMPTE = unsafe { core::mem::transmute(
    PTE_VALID as ULONG_PTR | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED | PTE_ENABLE_CACHE | PTE_SH_INNER
) };

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut ValidKernelPdeLocal: MMPTE = unsafe { core::mem::transmute(
    PTE_VALID as ULONG_PTR | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED
) };
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut ValidKernelPteLocal: MMPTE = unsafe { core::mem::transmute(
    PTE_VALID as ULONG_PTR | PTE_NOT_LARGE_PAGE | PTE_READWRITE | PTE_ACCESSED | PTE_ENABLE_CACHE | PTE_SH_INNER
) };

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut DemandZeroPde: MMPTE = unsafe { core::mem::transmute(
    (MM_READWRITE as ULONG_PTR) << (MM_PTE_SOFTWARE_PROTECTION_BITS as ULONG_PTR)
) };
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut DemandZeroPte: MMPTE = unsafe { core::mem::transmute(
    (MM_READWRITE as ULONG_PTR) << (MM_PTE_SOFTWARE_PROTECTION_BITS as ULONG_PTR)
) };

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut PrototypePte: MMPTE = unsafe { core::mem::transmute(
    ((MM_READWRITE as ULONG_PTR) << (MM_PTE_SOFTWARE_PROTECTION_BITS as ULONG_PTR))
    | PTE_PROTOTYPE
    | (MI_PTE_LOOKUP_NEEDED << 32)
) };

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut MmDecommittedPte: MMPTE = unsafe { core::mem::transmute(
    (MM_DECOMMIT as ULONG_PTR) << (MM_PTE_SOFTWARE_PROTECTION_BITS as ULONG_PTR)
) };

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut MiSessionViewEnd: PVOID = core::ptr::null_mut();
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut MiSystemPteSpaceStart: PVOID = core::ptr::null_mut();
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub static mut MiSystemPteSpaceEnd: PVOID = core::ptr::null_mut();

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitializeSessionSpaceLayout() {
    MmSessionSize = MI_SESSION_SIZE;
    MiSessionSpaceEnd = PTE_BASE as PVOID;

    MmSessionImageSize = MI_SESSION_IMAGE_SIZE;
    MiSessionImageEnd = MiSessionSpaceEnd;
    MiSessionImageStart = (MiSessionImageEnd as PUCHAR).sub(MmSessionImageSize) as PVOID;

    MiSessionSpaceWs = (MiSessionImageStart as PUCHAR).sub(MI_SESSION_WORKING_SET_SIZE) as PVOID;

    MmSessionViewSize = MI_SESSION_VIEW_SIZE;
    MiSessionViewEnd = MiSessionSpaceWs as PVOID;
    MiSessionViewStart = (MiSessionViewEnd as PUCHAR).sub(MmSessionViewSize) as PVOID;

    MmSessionPoolSize = MI_SESSION_POOL_SIZE;
    MiSessionPoolEnd = MiSessionViewStart;
    MiSessionPoolStart = (MiSessionPoolEnd as PUCHAR).sub(MmSessionPoolSize) as PVOID;

    MmSessionBase = MiSessionPoolStart;

    MiSessionImagePteStart = MiAddressToPte(MiSessionImageStart);
    MiSessionImagePteEnd = MiAddressToPte(MiSessionImageEnd);
    MiSessionBasePte = MiAddressToPte(MmSessionBase);
    MiSessionLastPte = MiAddressToPte(MiSessionSpaceEnd);

    MmSessionSpace = ((MiSessionImageStart as ULONG_PTR) + 0x10000) as PVOID;

    MmSystemViewSize = MI_SYSTEM_VIEW_SIZE;
    MiSystemViewStart = (MmSessionBase as PUCHAR).sub(MmSystemViewSize) as PVOID;
}

#[cfg(target_arch = "aarch64")]
unsafe fn MiMapPXEs(
    StartAddress: PVOID,
    EndAddress: PVOID
) {
    let mut TmplPte = ValidKernelPde;
    let mut PointerPxe = MiAddressToPxe(StartAddress);
    let EndPxe = MiAddressToPxe(EndAddress);

    while PointerPxe <= EndPxe {
        if !(*PointerPxe).valid() {
            TmplPte.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(PointerPxe, TmplPte);
            core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));
            RtlZeroMemory(MiPteToAddress(PointerPxe), PAGE_SIZE);
        }
        PointerPxe = PointerPxe.add(1);
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn MiMapPPEs(
    StartAddress: PVOID,
    EndAddress: PVOID
) {
    let mut TmplPte = ValidKernelPde;
    let mut PointerPpe = MiAddressToPpe(StartAddress);
    let EndPpe = MiAddressToPpe(EndAddress);

    while PointerPpe <= EndPpe {
        if !(*PointerPpe).valid() {
            TmplPte.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(PointerPpe, TmplPte);
            core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));
            RtlZeroMemory(MiPteToAddress(PointerPpe), PAGE_SIZE);
        }
        PointerPpe = PointerPpe.add(1);
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn MiMapPDEs(
    StartAddress: PVOID,
    EndAddress: PVOID
) {
    let mut TmplPte = ValidKernelPde;
    let mut PointerPde = MiAddressToPde(StartAddress);
    let EndPde = MiAddressToPde(EndAddress);

    while PointerPde <= EndPde {
        if !(*PointerPde).valid() {
            TmplPte.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(PointerPde, TmplPte);
            core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));
            RtlZeroMemory(MiPteToAddress(PointerPde), PAGE_SIZE);
        }
        PointerPde = PointerPde.add(1);
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn MiMapPTEs(
    StartAddress: PVOID,
    EndAddress: PVOID
) {
    let mut TmplPte = ValidKernelPte;
    let mut PointerPte = MiAddressToPte(StartAddress);
    let EndPte = MiAddressToPte(EndAddress);

    while PointerPte <= EndPte {
        if !(*PointerPte).valid() {
            TmplPte.set_page_frame_number(MxGetNextPage(1));
            MI_WRITE_VALID_PTE(PointerPte, TmplPte);
        }
        PointerPte = PointerPte.add(1);
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn MiEnsurePageTableHierarchy(
    StartAddress: PVOID,
    EndAddress: PVOID
) {
    MiMapPXEs(StartAddress, EndAddress);
    MiMapPPEs(StartAddress, EndAddress);
    MiMapPDEs(StartAddress, EndAddress);
}

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
unsafe fn MiInitializePageTable() {
    let PxePhysicalAddress: ULONG64;
    let mut TmplPte: MMPTE;
    let mut PointerPxe: PMMPTE;
    let mut PxePfn: PFN_NUMBER;

    PxePfn = (*(PXE_SELFMAP as PMMPTE)).page_frame_number();
    PxePhysicalAddress = (PxePfn as ULONG64) << PAGE_SHIFT;

    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[0] = PxePhysicalAddress;

    KeFlushCurrentTb();

    TmplPte = unsafe { core::mem::zeroed() };
    TmplPte.set_valid(true);
    TmplPte.Long |= PTE_NOT_LARGE_PAGE; // NotLargePage bit
    // Writable on ARM64 — set bit 51 (Writable in _HARDWARE_PTE)
    // But actually for the template PTE it's simpler to set the full mask:
    TmplPte.Long |= PTE_READWRITE;
    TmplPte.Long |= PTE_ACCESSED;
    // Same attributes as every other mapping of normal memory: a page zeroed
    // through a Non-shareable hyperspace alias and then used through its
    // Inner Shareable pool mapping is a mismatched-attribute alias.
    TmplPte.Long |= PTE_SH_INNER;
    HyperTemplatePte = TmplPte;

    PointerPxe = MiAddressToPxe(MiSystemViewStart);
    let EndPxe = MiAddressToPxe(MI_HIGHEST_SYSTEM_ADDRESS as PVOID);

    while PointerPxe <= EndPxe {
        if !(*PointerPxe).valid() {
            TmplPte.Long = ValidKernelPde.Long;
            TmplPte.set_page_frame_number(MxGetNextPage(1));
            *PointerPxe = TmplPte;

            core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));

            RtlZeroMemory(MiPteToAddress(PointerPxe), PAGE_SIZE);
        }
        PointerPxe = PointerPxe.add(1);
    }

    PxePfn = PFN_FROM_PXE(MiAddressToPxe(HYPER_SPACE as PVOID));
    (*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb.DirectoryTableBase[1] = (PxePfn as ULONG64) << PAGE_SHIFT;

    MiMapPPEs(MmPagedPoolStart, MmPagedPoolEnd);

    MiMapPPEs(HYPER_SPACE as PVOID, HYPER_SPACE_END as PVOID);

    MiMapPPEs(MmSessionBase, (MiSessionSpaceEnd as PCHAR).sub(1) as PVOID);

    MiMapPPEs(MiSystemViewStart, ((MiSystemViewStart as PCHAR).add(MmSystemViewSize)) as PVOID);

    MiEnsurePageTableHierarchy(MI_MAPPING_RANGE_START as PVOID, MI_MAPPING_RANGE_END as PVOID);

    MmFirstReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_START as PVOID);
    MmLastReservedMappingPte = MiAddressToPte(MI_MAPPING_RANGE_END as PVOID);
    (*MmFirstReservedMappingPte).set_page_frame_number(MI_HYPERSPACE_PTES as PFN_NUMBER);

    MiEnsurePageTableHierarchy(MI_DEBUG_MAPPING as PVOID, MI_DEBUG_MAPPING as PVOID);
    MmDebugPte = MiAddressToPte(MI_DEBUG_MAPPING as PVOID);

    MiEnsurePageTableHierarchy(MI_VAD_BITMAP as PVOID, (MI_WORKING_SET_LIST + PAGE_SIZE - 1) as PVOID);
    MiMapPTEs(MI_VAD_BITMAP as PVOID, (MI_WORKING_SET_LIST + PAGE_SIZE - 1) as PVOID);
}

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
pub unsafe fn uart_hex(label: &[u8], val: u64) {
    ARM64_UART_TRACE(label.as_ptr() as PCHAR);
    let digits = b"0123456789ABCDEF";
    let mut buf = [0u8; 21]; // "0x" + 16 hex digits + "\r\n\0"
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        buf[2 + i] = digits[((val >> (60 - i * 4)) & 0xF) as usize];
    }
    buf[18] = b'\r';
    buf[19] = b'\n';
    buf[20] = 0;
    ARM64_UART_TRACE(buf.as_ptr() as PCHAR);
}

unsafe fn MiBuildNonPagedPool() {
    ARM64_UART_TRACE(b"[MM] MiBuildNonPagedPool\r\n\0".as_ptr() as PCHAR);

    if (MmNumberOfPhysicalPages <= MI_MIN_PAGES_FOR_NONPAGED_POOL_TUNING) &&
        (MmSizeOfNonPagedPoolInBytes == 0) {
        MmSizeOfNonPagedPoolInBytes = 2 * 1024 * 1024;
    }

    if (MmSizeOfNonPagedPoolInBytes >> PAGE_SHIFT) >
        (MmNumberOfPhysicalPages * 7 / 8) {
        MmSizeOfNonPagedPoolInBytes = 0;
    }

    if MmSizeOfNonPagedPoolInBytes < MmMinimumNonPagedPoolSize {
        MmSizeOfNonPagedPoolInBytes = MmMinimumNonPagedPoolSize;
        uart_hex(b"  MinPoolSize=\0", MmMinimumNonPagedPoolSize as u64);
        uart_hex(b"  PhysPages=\0", MmNumberOfPhysicalPages as u64);
        uart_hex(b"  MinAddPerMb=\0", MmMinAdditionNonPagedPoolPerMb as u64);
        let addition = (MmNumberOfPhysicalPages - 1024) / 256 * MmMinAdditionNonPagedPoolPerMb as SIZE_T;
        uart_hex(b"  Addition=\0", addition as u64);
        MmSizeOfNonPagedPoolInBytes += addition;
        uart_hex(b"  PoolSizeAfterMin=\0", MmSizeOfNonPagedPoolInBytes as u64);
    }

    if MmSizeOfNonPagedPoolInBytes > MI_MAX_INIT_NONPAGED_POOL_SIZE {
        ARM64_UART_TRACE(b"  Capped at MI_MAX_INIT\r\n\0".as_ptr() as PCHAR);
        MmSizeOfNonPagedPoolInBytes = MI_MAX_INIT_NONPAGED_POOL_SIZE;
    }

    if MmMaximumNonPagedPoolPercent != 0 {
        // UNIMPLEMENTED
    }

    MmSizeOfNonPagedPoolInBytes &= !(PAGE_SIZE - 1);

    if MmMaximumNonPagedPoolInBytes == 0 {
        MmMaximumNonPagedPoolInBytes = MmDefaultMaximumNonPagedPool;
        MmMaximumNonPagedPoolInBytes += (MmNumberOfPhysicalPages - 1024) /
                                         256 * MmMaxAdditionNonPagedPoolPerMb as SIZE_T;
    }

    if MmMaximumNonPagedPoolInBytes > MI_MAX_NONPAGED_POOL_SIZE {
        MmMaximumNonPagedPoolInBytes = MI_MAX_NONPAGED_POOL_SIZE;
    }

    MmMaximumNonPagedPoolInPages = MmMaximumNonPagedPoolInBytes >> PAGE_SHIFT;

    MmNonPagedPoolStart = (MmPfnDatabase as PUCHAR).add(MxPfnAllocation * PAGE_SIZE) as PVOID;

    MmNonPagedPoolExpansionStart = (MmNonPagedPoolStart as PUCHAR).add(MmSizeOfNonPagedPoolInBytes) as PVOID;

    MmNonPagedPoolEnd = (MmNonPagedPoolStart as PUCHAR).add(MmMaximumNonPagedPoolInBytes) as PVOID;

    uart_hex(b"  PfnDb=\0", MmPfnDatabase as u64);
    uart_hex(b"  MxPfn=\0", MxPfnAllocation as u64);
    uart_hex(b"  PoolStart=\0", MmNonPagedPoolStart as u64);
    uart_hex(b"  PoolExpStart=\0", MmNonPagedPoolExpansionStart as u64);
    uart_hex(b"  PoolEnd=\0", MmNonPagedPoolEnd as u64);
    uart_hex(b"  PoolSize=\0", MmSizeOfNonPagedPoolInBytes as u64);
    uart_hex(b"  MaxPoolSize=\0", MmMaximumNonPagedPoolInBytes as u64);

    MiEnsurePageTableHierarchy(MmNonPagedPoolStart, MmNonPagedPoolEnd);

    ARM64_UART_TRACE(b"[MM] MiMapPTEs for pool\r\n\0".as_ptr() as PCHAR);
    MiMapPTEs(MmNonPagedPoolStart, (MmNonPagedPoolExpansionStart as PCHAR).sub(1) as PVOID);

    ARM64_UART_TRACE(b"[MM] MiInitializeNonPagedPool\r\n\0".as_ptr() as PCHAR);
    MiInitializeNonPagedPool();
    MiInitializeNonPagedPoolThresholds();
}

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
unsafe fn MiBuildSystemPteSpace() {
    let PointerPte: PMMPTE;
    let NonPagedSystemSize: SIZE_T;

    ARM64_UART_TRACE(b"[MM] MiBuildSystemPteSpace\r\n\0".as_ptr() as PCHAR);

    MmNumberOfSystemPtes = MI_NUMBER_SYSTEM_PTES as PFN_COUNT;
    NonPagedSystemSize = (MmNumberOfSystemPtes as SIZE_T + 1) * PAGE_SIZE;

    MiSystemPteSpaceStart = MmNonPagedSystemStart;
    MiSystemPteSpaceEnd = (MiSystemPteSpaceStart as PUCHAR).add(NonPagedSystemSize) as PVOID;

    MiEnsurePageTableHierarchy(MiSystemPteSpaceStart, MiSystemPteSpaceEnd);

    PointerPte = MiAddressToPte(MiSystemPteSpaceStart);
    MiInitializeSystemPtes(PointerPte, MmNumberOfSystemPtes, SystemPteSpace);

    MiFirstReservedZeroingPte = MiReserveSystemPtes(MI_ZERO_PTES + 1, SystemPteSpace);
    RtlZeroMemory(MiFirstReservedZeroingPte as PVOID, (MI_ZERO_PTES as SIZE_T + 1) * core::mem::size_of::<MMPTE>());

    (*MiFirstReservedZeroingPte).set_page_frame_number(MI_ZERO_PTES as PFN_NUMBER);

    ARM64_UART_TRACE(b"[MM] SysPTE space initialized\r\n\0".as_ptr() as PCHAR);
}

#[cfg(target_arch = "aarch64")]
unsafe fn MiSetupPfnForPageTable(
    PageFrameIndex: PFN_NUMBER,
    PointerPte: PMMPTE
) {
    let mut Pfn: PMMPFN;
    let PointerPde: PMMPDE;

    Pfn = MiGetPfnEntry(PageFrameIndex);

    if (PageFrameIndex <= MmHighestPhysicalPage) &&
        (MmIsAddressValid(Pfn as PVOID) != 0) {
        // u1.WsIndex = 0 — same offset as Flink
        (*Pfn).set_flink(0);
        (*Pfn).set_share_count((*Pfn).share_count() + 1);
        (*Pfn).set_pte_address(PointerPte);
        (*Pfn).set_original_pte(*PointerPte);
        (*Pfn).set_page_location(ActiveAndValid);
        (*Pfn).set_cache_attribute(MiNonCached as u16);
        (*Pfn).set_reference_count(1);
        (*Pfn).set_pte_frame(PFN_FROM_PTE(MiAddressToPte(PointerPte as PVOID)));
    }

    PointerPde = MiAddressToPde(MiPteToAddress(PointerPte));
    Pfn = MiGetPfnEntry(PFN_FROM_PTE(PointerPde));
    if (PFN_FROM_PTE(PointerPde) <= MmHighestPhysicalPage) &&
        (MmIsAddressValid(Pfn as PVOID) != 0) {
        (*Pfn).set_share_count((*Pfn).share_count() + 1);
    }
}

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
unsafe fn MiBuildPfnDatabaseFromPageTables() {
    let mut Address: PVOID = core::ptr::null_mut();
    let mut PageFrameIndex: PFN_NUMBER;
    let mut PointerPde: PMMPDE;
    let mut PointerPte: PMMPTE;
    let mut i: ULONG;
    let mut j: ULONG;
    let mut k: ULONG;
    let mut l: ULONG;
    let mut Pfn: PMMPFN;
    let mut PointerPpe: PMMPDE;
    let mut PointerPxe: PMMPDE;

    PageFrameIndex = PFN_FROM_PTE(MiAddressToPte(PXE_BASE as PVOID));
    Pfn = MiGetPfnEntry(PageFrameIndex);
    if MmIsAddressValid(Pfn as PVOID) != 0 {
        // u1.WsIndex = 0
        (*Pfn).set_flink(0);
        (*Pfn).set_share_count(1);
        (*Pfn).set_pte_address(core::ptr::null_mut());
        (*Pfn).set_reference_count(1);
        (*Pfn).set_page_location(ActiveAndValid);
        (*Pfn).set_cache_attribute(MiNonCached as u16);
        (*Pfn).set_pte_frame(0);
    }

    PointerPxe = MiAddressToPxe(Address) as PMMPDE;
    i = 0;
    while i < PXE_PER_PAGE as ULONG {
        if (*PointerPxe).valid() {
            PageFrameIndex = PFN_FROM_PXE(PointerPxe as PMMPTE);
            MiSetupPfnForPageTable(PageFrameIndex, PointerPxe as PMMPTE);

            Address = MiPxeToAddress(PointerPxe as PMMPTE);

            PointerPpe = MiAddressToPpe(Address) as PMMPDE;
            j = 0;
            while j < PPE_PER_PAGE as ULONG {
                if (*PointerPpe).valid() {
                    PageFrameIndex = PFN_FROM_PPE(PointerPpe as PMMPTE);
                    MiSetupPfnForPageTable(PageFrameIndex, PointerPpe as PMMPTE);

                    Address = MiPpeToAddress(PointerPpe as PMMPTE);

                    PointerPde = MiAddressToPde(Address);
                    k = 0;
                    while k < PDE_PER_PAGE as ULONG {
                        if (*PointerPde).valid() {
                            // PFN_FROM_PDE = PFN_FROM_PTE on ARM64
                            PageFrameIndex = PFN_FROM_PTE(PointerPde as PMMPTE);
                            MiSetupPfnForPageTable(PageFrameIndex, PointerPde as PMMPTE);

                            Address = MiPdeToAddress(PointerPde as PMMPTE);

                            PointerPte = MiAddressToPte(Address);
                            l = 0;
                            while l < PTE_PER_PAGE as ULONG {
                                if (*PointerPte).valid() {
                                    PageFrameIndex = PFN_FROM_PTE(PointerPte);
                                    if PageFrameIndex <= MmHighestPhysicalPage {
                                        Pfn = MiGetPfnEntry(PageFrameIndex);
                                        if (MmIsAddressValid(Pfn as PVOID) != 0) &&
                                            ((*Pfn).page_location() != ActiveAndValid) {
                                            (*Pfn).set_pte_frame(PFN_FROM_PTE(PointerPde as PMMPTE));
                                            (*Pfn).set_pte_address(PointerPte);
                                            (*Pfn).set_share_count((*Pfn).share_count() + 1);
                                            (*Pfn).set_reference_count(1);
                                            (*Pfn).set_page_location(ActiveAndValid);
                                            (*Pfn).set_cache_attribute(MiCached);
                                        }
                                    }
                                }
                                l += 1;
                                PointerPte = PointerPte.add(1);
                            }
                        }
                        k += 1;
                        PointerPde = PointerPde.add(1);
                    }
                }
                j += 1;
                PointerPpe = PointerPpe.add(1);
            }
        }
        i += 1;
        PointerPxe = PointerPxe.add(1);
    }
}

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
unsafe fn MiBuildPfnDatabase(
    LoaderBlock: PLOADER_PARAMETER_BLOCK
) {
    ARM64_UART_TRACE(b"[MM] MiBuildPfnDatabase\r\n\0".as_ptr() as PCHAR);

    MiEnsurePageTableHierarchy(MmPfnDatabase as PVOID,
                               ((MmPfnDatabase as PCHAR).add((MxPfnAllocation << PAGE_SHIFT) - 1)) as PVOID);

    MiMapPTEs(MmPfnDatabase as PVOID,
              ((MmPfnDatabase as PCHAR).add((MxPfnAllocation << PAGE_SHIFT) - 1)) as PVOID);

    RtlZeroMemory(MmPfnDatabase as PVOID, MxPfnAllocation << PAGE_SHIFT);

    MiInitializeColorTables();

    MiBuildPfnDatabaseFromPageTables();

    MiInitializePfnDatabase(LoaderBlock);
}

#[cfg(target_arch = "aarch64")]
const MI_PFN_DATABASE: ULONG_PTR = 0xFFFFFA0000000000;

#[cfg(target_arch = "aarch64")]
// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitMachineDependent(
    LoaderBlock: PLOADER_PARAMETER_BLOCK
) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    let mut Pfn: PMMPFN;
    let mut Flags: ULONG;

    ARM64_UART_TRACE(b"[MM] MiInitMachineDependent ARM64\r\n\0".as_ptr() as PCHAR);

    // Assert MxPfnAllocation != 0
    ntkrnl_sys::ASSERT!(MxPfnAllocation != 0);

    // Set some hardcoded addresses
    MmHyperSpaceEnd = HYPER_SPACE_END as PVOID;
    MmNonPagedSystemStart = MI_DEFAULT_SYSTEM_RANGE_START as PVOID;
    MmPfnDatabase = MI_PFN_DATABASE as PMMPFN;
    MmWorkingSetList = MI_WORKING_SET_LIST as PVOID;

    // Set the prototype PTE lookup field — already done in static init via PrototypePte

    MiInitializeSessionSpaceLayout();
    MiInitializePageTable();
    MiBuildNonPagedPool();
    MiBuildSystemPteSpace();
    MiBuildPfnDatabase(LoaderBlock);

    // Reset ref/share counts for MmInitializeProcessAddressSpace
    Pfn = MiGetPfnEntry(PFN_FROM_PTE(PXE_SELFMAP as PMMPTE));
    if MmIsAddressValid(Pfn as PVOID) != 0 {
        (*Pfn).set_share_count(0);
        (*Pfn).set_reference_count(0);
    }

    Pfn = MiGetPfnEntry(PFN_FROM_PTE(MiAddressToPde(HYPER_SPACE as PVOID)));
    if MmIsAddressValid(Pfn as PVOID) != 0 {
        (*Pfn).set_share_count(0);
        (*Pfn).set_reference_count(0);
    }

    Pfn = MiGetPfnEntry(PFN_FROM_PPE(MiAddressToPpe(HYPER_SPACE as PVOID)));
    if MmIsAddressValid(Pfn as PVOID) != 0 {
        (*Pfn).set_share_count(0);
        (*Pfn).set_reference_count(0);
    }

    Pfn = MiGetPfnEntry(PFN_FROM_PXE(MiAddressToPxe(HYPER_SPACE as PVOID)));
    if MmIsAddressValid(Pfn as PVOID) != 0 {
        (*Pfn).set_share_count(0);
        (*Pfn).set_reference_count(0);
    }

    Pfn = MiGetPfnEntry(PFN_FROM_PTE(MiAddressToPte(MmWorkingSetList)));
    if MmIsAddressValid(Pfn as PVOID) != 0 {
        (*Pfn).set_share_count(0);
        (*Pfn).set_reference_count(0);
    }

    // Initialize the nonpaged pool
    InitializePool(NonPagedPool, 0);

    // Initialize the bogus address space
    Flags = 0;
    let Status = MmInitializeProcessAddressSpace(
        PsGetCurrentProcess() as PEPROCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Flags as *mut ULONG,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Initialize the balancer
    MmInitializeBalancer(MmAvailablePages as ULONG, 0);

    ARM64_UART_TRACE(b"[MM] MiInitMachineDependent done\r\n\0".as_ptr() as PCHAR);

    // Sanity checks
    ntkrnl_sys::ASSERT!(!MmPfnDatabase.is_null());
    ntkrnl_sys::ASSERT!(!MmNonPagedSystemStart.is_null());
    ntkrnl_sys::ASSERT!(!MmNonPagedPoolStart.is_null());
    ntkrnl_sys::ASSERT!(MmSizeOfNonPagedPoolInBytes != 0);
    ntkrnl_sys::ASSERT!(MmMaximumNonPagedPoolInBytes != 0);
    ntkrnl_sys::ASSERT!(!MmNonPagedPoolExpansionStart.is_null());
    ntkrnl_sys::ASSERT!(!MmHyperSpaceEnd.is_null());
    ntkrnl_sys::ASSERT!(MmNumberOfSystemPtes != 0);

    STATUS_SUCCESS
}

#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmMaximumNonPagedPoolPercent: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSizeOfNonPagedPoolInBytes: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmMaximumNonPagedPoolInBytes: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmNonPagedSystemStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmNonPagedPoolStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmNonPagedPoolExpansionStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmPagedPoolEnd: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSessionSpaceEnd: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSessionImageEnd: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSessionImageStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSessionViewStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSessionPoolEnd: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSessionPoolStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSessionBase: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSessionSize: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSessionViewSize: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSessionPoolSize: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSessionImageSize: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiSystemViewStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemViewSize: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemPageDirectory: [PFN_NUMBER; PPE_PER_PAGE] = [0; PPE_PER_PAGE];
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemPagePtes: PMMPDE = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmNumberOfSystemPtes: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MxPfnAllocation: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MiPfnBitMap: RTL_BITMAP = unsafe { core::mem::zeroed() };
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmPhysicalMemoryBlock: PPHYSICAL_MEMORY_DESCRIPTOR = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MxFreeDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MxOldFreeDescriptor: MEMORY_ALLOCATION_DESCRIPTOR = unsafe { core::mem::zeroed() };
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmNumberOfPhysicalPages: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmHighestPhysicalPage: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmBootImageSize: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmUserProbeAddress: ULONG = 0;
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmHighestUserAddress: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemRangeStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemCacheStart: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemCacheEnd: PVOID = core::ptr::null_mut();
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmSystemCacheWs: MMSUPPORT = unsafe { core::mem::zeroed() };
#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
pub static mut MmHyperSpaceEnd: PVOID = core::ptr::null_mut();

#[cfg(not(target_arch = "aarch64"))]
// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitMachineDependent(LoaderBlock: PLOADER_PARAMETER_BLOCK) -> NTSTATUS {
    let LoaderBlock = opaque_ptr(LoaderBlock);
    STATUS_SUCCESS
}
