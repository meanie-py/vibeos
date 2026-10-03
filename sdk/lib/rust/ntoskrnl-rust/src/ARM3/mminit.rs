use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{ACCESS_ALLOWED_ACE, ACL, ExInitializePushLock, KEVENT, KeFlushCurrentTb, KeInitializeEvent, KeInitializeMutant, KeReadStateEvent, LARGE_INTEGER, MEMORY_ALLOCATION_DESCRIPTOR, MMSUPPORT, MiDecrementShareCount, MiInitMachineDependent, MiInitializeDriverLargePageList, MiInitializeLargePageSupport, MiInitializeLoadedModuleList, MiInitializePoolEvents, MiInitializeSessionSpaceLayout, MiInitializeSpecialPool, MiInitializeSystemSpaceMap, MiIsMemoryTypeFree, MiIsMemoryTypeInvisible, MiReloadBootLoadedDrivers, MiSyncCachedRanges, MmIsAddressValid, ObReferenceObjectByHandle, PACL, PKEVENT, PLOADER_PARAMETER_BLOCK, PMEMORY_ALLOCATION_DESCRIPTOR, PMMWSL, PPHYSICAL_MEMORY_RUN, PUNICODE_STRING, RtlAddAccessAllowedAce, RtlCreateAcl, RtlCreateSecurityDescriptor, RtlLengthSid, ZwClose, ZwCreateEvent, _EVENT_TYPE_NotificationEvent, _KEVENT, _KMUTANT, _KPCR, _LOADER_PARAMETER_BLOCK, _MEMORY_ALLOCATION_DESCRIPTOR, _MMPFN, _MMPTE, _OBJECT_ATTRIBUTES, _OBJECT_TYPE, _PHYSICAL_MEMORY_DESCRIPTOR, _PHYSICAL_MEMORY_RUN, _RTL_BITMAP, _SECURITY_DESCRIPTOR};
#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::ddk::MiMapPfnDatabase;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::helpers::*;

extern "C" {
    fn InitializeObjectAttributes( p: *mut _OBJECT_ATTRIBUTES, n: PUNICODE_STRING, a: ULONG, r: HANDLE, s: PVOID, );
}

/// CONTAINING_RECORD macro: given a pointer to a field within a struct, return
/// a pointer to the containing struct. For MEMORY_ALLOCATION_DESCRIPTOR where
/// ListEntry is always at offset 0, this is just a cast.
macro_rules! CONTAINING_RECORD {
    ($address:expr, $type:ty, $field:ident) => {
        ($address as *const u8).sub(core::mem::offset_of!($type, $field)) as *mut $type
    };
}

#[cfg(target_arch = "aarch64")]
extern "C" {
    static mut Arm64UartKernelBase: PVOID;
}

#[cfg(target_arch = "aarch64")]
unsafe fn _arm3_trace(s: &[u8]) {
    let uart_base = Arm64UartKernelBase;
    if uart_base.is_null() {
        return;
    }
    let u = uart_base as *mut ULONG;
    for &byte in s {
        crate::arm64_dbg::uart_putc(u as *mut u32, byte);
    }
}

#[cfg(target_arch = "aarch64")]
macro_rules! _A3_TRACE {
    ($s:expr) => {
        _arm3_trace($s)
    };
}

#[cfg(not(target_arch = "aarch64"))]
macro_rules! _A3_TRACE {
    ($s:expr) => {{}};
}

#[no_mangle]
pub static mut MmMaximumNonPagedPoolPercent: ULONG = 0;
#[no_mangle]
pub static mut MmSizeOfNonPagedPoolInBytes: SIZE_T = 0;
#[no_mangle]
pub static mut MmMaximumNonPagedPoolInBytes: SIZE_T = 0;
#[no_mangle]
pub static mut MmMaximumNonPagedPoolInPages: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmMinimumNonPagedPoolSize: SIZE_T = 256 * 1024;
#[no_mangle]
pub static mut MmMinAdditionNonPagedPoolPerMb: ULONG = 32 * 1024;
#[no_mangle]
pub static mut MmDefaultMaximumNonPagedPool: SIZE_T = 1024 * 1024;
#[no_mangle]
pub static mut MmMaxAdditionNonPagedPoolPerMb: ULONG = 400 * 1024;

#[no_mangle]
pub static mut MmNonPagedSystemStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmNonPagedPoolStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmNonPagedPoolExpansionStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmNonPagedPoolEnd: PVOID = MI_NONPAGED_POOL_END as PVOID;

#[no_mangle]
pub static mut MmPagedPoolStart: PVOID = MI_PAGED_POOL_START as PVOID;
#[no_mangle]
pub static mut MmPagedPoolEnd: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MmSizeOfPagedPoolInBytes: SIZE_T = MI_MIN_INIT_PAGED_POOLSIZE;
#[no_mangle]
pub static mut MmSizeOfPagedPoolInPages: PFN_NUMBER = MI_MIN_INIT_PAGED_POOLSIZE / PAGE_SIZE;

#[no_mangle]
pub static mut MiSessionSpaceEnd: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionImageEnd: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionImageStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionSpaceWs: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionViewStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionPoolEnd: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionPoolStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmSessionBase: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmSessionSize: SIZE_T = 0;
#[no_mangle]
pub static mut MmSessionViewSize: SIZE_T = 0;
#[no_mangle]
pub static mut MmSessionPoolSize: SIZE_T = 0;
#[no_mangle]
pub static mut MmSessionImageSize: SIZE_T = 0;

#[no_mangle]
pub static mut MiSessionImagePteStart: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionImagePteEnd: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionBasePte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MiSessionLastPte: PMMPTE = core::ptr::null_mut();

#[no_mangle]
pub static mut MiSystemViewStart: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmSystemViewSize: SIZE_T = 0;

#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub static mut MmSystemPageDirectory: [PFN_NUMBER; PPE_PER_PAGE] = [0; PPE_PER_PAGE];
#[cfg(not(target_pointer_width = "64"))]
#[no_mangle]
pub static mut MmSystemPagePtes: PMMPTE = core::ptr::null_mut();

#[no_mangle]
pub static mut MmSystemCacheWorkingSetList: PMMWSL = MI_SYSTEM_CACHE_WS_START as PMMWSL;

#[no_mangle]
pub static mut MmNumberOfSystemPtes: PFN_COUNT = 0;
#[no_mangle]
pub static mut MxPfnAllocation: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MiPfnBitMap: _RTL_BITMAP = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmPhysicalMemoryBlock: PPHYSICAL_MEMORY_DESCRIPTOR = core::ptr::null_mut();

#[no_mangle]
pub static mut MmHighestPhysicalPage: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MmLowestPhysicalPage: PFN_NUMBER = PFN_NUMBER::MAX;
#[no_mangle]
pub static mut MmNumberOfPhysicalPages: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmBootImageSize: SIZE_T = 0;

#[no_mangle]
pub static mut MmUserProbeAddress: ULONG_PTR = 0;
#[no_mangle]
pub static mut MmHighestUserAddress: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmSystemRangeStart: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MiHighestUserPte: PMMPTE = core::ptr::null_mut();
#[no_mangle]
pub static mut MiHighestUserPde: PMMPTE = core::ptr::null_mut();
#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub static mut MiHighestUserPpe: PMMPTE = core::ptr::null_mut();
#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub static mut MiHighestUserPxe: PMMPTE = core::ptr::null_mut();

#[no_mangle]
pub static mut MmSystemCacheStart: PVOID = MI_SYSTEM_CACHE_START as PVOID;
#[no_mangle]
pub static mut MmSystemCacheEnd: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut MmSizeOfSystemCacheInPages: ULONG_PTR = 0;
#[no_mangle]
pub static mut MmSystemCacheWs: MMSUPPORT = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmHyperSpaceEnd: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut MmSecondaryColors: ULONG = 0;
#[no_mangle]
pub static mut MmSecondaryColorMask: ULONG = 0;

#[no_mangle]
pub static mut MmLargeStackSize: ULONG = (KERNEL_LARGE_STACK_SIZE) as u32;

#[no_mangle]
pub static mut MxFreeDescriptor: PMEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();
#[no_mangle]
pub static mut MxOldFreeDescriptor: _MEMORY_ALLOCATION_DESCRIPTOR = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmFreePagesByColor: [PMMCOLOR_TABLES; (FreePageList + 1) as usize] = 
    [core::ptr::null_mut(); (FreePageList + 1) as usize];

static mut MiTempEvent: KEVENT = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MiLowMemoryEvent: PKEVENT = core::ptr::null_mut();
#[no_mangle]
pub static mut MiHighMemoryEvent: PKEVENT = core::ptr::null_mut();
#[no_mangle]
pub static mut MiLowPagedPoolEvent: PKEVENT = core::ptr::null_mut();
#[no_mangle]
pub static mut MiHighPagedPoolEvent: PKEVENT = core::ptr::null_mut();
#[no_mangle]
pub static mut MiLowNonPagedPoolEvent: PKEVENT = core::ptr::null_mut();
#[no_mangle]
pub static mut MiHighNonPagedPoolEvent: PKEVENT = core::ptr::null_mut();

#[no_mangle]
pub static mut MmLowMemoryThreshold: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MmHighMemoryThreshold: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MiLowPagedPoolThreshold: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MiHighPagedPoolThreshold: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MiLowNonPagedPoolThreshold: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MiHighNonPagedPoolThreshold: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmMinimumFreePages: PFN_NUMBER = 26;
#[no_mangle]
pub static mut MmPlentyFreePages: PFN_NUMBER = 400;

#[no_mangle]
pub static mut MmProductType: ULONG = 0;
#[no_mangle]
pub static mut MmSystemSize: MM_SYSTEMSIZE = 0;

#[no_mangle]
pub static mut MmSystemCacheWsMinimum: PFN_NUMBER = 288;
#[no_mangle]
pub static mut MmSystemCacheWsMaximum: PFN_NUMBER = 350;

#[no_mangle]
pub static mut MmLargeSystemCache: BOOLEAN = 0;

#[no_mangle]
pub static mut MmAllocationFragment: SIZE_T = 0;

#[no_mangle]
pub static mut MmTotalCommitLimit: SIZE_T = 0;
static mut MmTotalCommitLimitMaximum: SIZE_T = 0;

#[no_mangle]
pub static mut MmHeapSegmentReserve: SIZE_T = 1 * _1MB;
#[no_mangle]
pub static mut MmHeapSegmentCommit: SIZE_T = 2 * PAGE_SIZE;
#[no_mangle]
pub static mut MmHeapDeCommitTotalFreeThreshold: SIZE_T = 64 * _1KB;
#[no_mangle]
pub static mut MmHeapDeCommitFreeBlockThreshold: SIZE_T = PAGE_SIZE;
#[no_mangle]
pub static mut MmMinimumStackCommitInBytes: SIZE_T = 0;

#[cfg(target_arch = "arm")]
static mut MiDbgEnableMdDump: BOOLEAN = 1;
#[cfg(not(target_arch = "arm"))]
static mut MiDbgEnableMdDump: BOOLEAN = 0;

static mut MiNumberDescriptors: ULONG = 0;
#[no_mangle]
pub static mut MiNumberOfFreePages: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmCritsectTimeoutSeconds: ULONG = 150;
#[no_mangle]
pub static mut MmCriticalSectionTimeout: LARGE_INTEGER = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmThrottleTop: ULONG = 0;
#[no_mangle]
pub static mut MmThrottleBottom: ULONG = 0;

// Registry-controlled values owned by ARM3/mminit. Concatenated into
// CmControlVector by config::cmdata. See docs/engineering/cmdata-chopping-plan.md.
// Renamed from MMINIT_CONTROL_VECTOR (collided with mm::mminit's) so it can be a
// unique #[no_mangle] link symbol for config-rust's runtime CmControlVector assembler.
#[no_mangle]
pub static ARM3_MMINIT_CONTROL_VECTOR: ntkrnl_sys::SubVector<12> =
    ntkrnl_sys::SubVector(unsafe { [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "MapAllocationFragment",
            core::ptr::addr_of_mut!(MmAllocationFragment)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "PagedPoolSize",
            core::ptr::addr_of_mut!(MmSizeOfPagedPoolInBytes)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "NonPagedPoolSize",
            core::ptr::addr_of_mut!(MmSizeOfNonPagedPoolInBytes)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "LargeSystemCache",
            core::ptr::addr_of_mut!(MmLargeSystemCache)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "SystemPages",
            core::ptr::addr_of_mut!(MmNumberOfSystemPtes)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "LowMemoryThreshold",
            core::ptr::addr_of_mut!(MmLowMemoryThreshold)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "SecondLevelDataCache",
            core::ptr::addr_of_mut!(MmSecondaryColors)),
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "MinimumStackCommitInBytes",
            core::ptr::addr_of_mut!(MmMinimumStackCommitInBytes)),
        ntkrnl_sys::cv!("Session Manager", "CriticalSectionTimeout",
            core::ptr::addr_of_mut!(MmCritsectTimeoutSeconds)),
        ntkrnl_sys::cv!("Session Manager", "HeapDeCommitTotalFreeThreshold",
            core::ptr::addr_of_mut!(MmHeapDeCommitTotalFreeThreshold)),
        ntkrnl_sys::cv!("Session Manager", "HeapDeCommitFreeBlockThreshold",
            core::ptr::addr_of_mut!(MmHeapDeCommitFreeBlockThreshold)),
        ntkrnl_sys::cv!("ProductOptions", "ProductType",
            core::ptr::addr_of_mut!(MmProductType)),
    ] });

unsafe fn MiScanMemoryDescriptors(loader_block: PLOADER_PARAMETER_BLOCK) {
    let mut list_entry: PLIST_ENTRY;
    let mut descriptor: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut page_frame_index: PFN_NUMBER;
    let mut free_pages: PFN_NUMBER = 0;

    list_entry = (*loader_block).MemoryDescriptorListHead.Flink;
    while list_entry != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        descriptor = CONTAINING_RECORD!(list_entry, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;

        MiNumberDescriptors += 1;

        if (MiIsMemoryTypeInvisible((*descriptor).MemoryType)) != 0 {
            list_entry = (*list_entry).Flink;
            continue;
        }

        if (*descriptor).MemoryType != LoaderBad {
            MmNumberOfPhysicalPages += (*descriptor).PageCount as PFN_NUMBER;
        }

        if (*descriptor).BasePage < (MmLowestPhysicalPage) as _ {
            MmLowestPhysicalPage = ((*descriptor).BasePage) as usize;
        }

        page_frame_index = ((*descriptor).BasePage + (*descriptor).PageCount) as usize;
        if page_frame_index > MmHighestPhysicalPage {
            MmHighestPhysicalPage = page_frame_index - 1;
        }

        if (MiIsMemoryTypeFree((*descriptor).MemoryType)) != 0 {
            MiNumberOfFreePages += ((*descriptor).PageCount) as usize;

            if (*descriptor).PageCount > (free_pages) as _ {
                MxFreeDescriptor = descriptor;
                free_pages = ((*descriptor).PageCount) as usize;
            }
        }

        list_entry = (*list_entry).Flink;
    }

    MxOldFreeDescriptor = *MxFreeDescriptor;
}

#[no_mangle]
pub unsafe extern "system" fn MxGetNextPage(page_count: PFN_NUMBER) -> PFN_NUMBER {
    if page_count > ((*MxFreeDescriptor).PageCount) as usize {
        KeBugCheckEx(
            INSTALL_MORE_MEMORY,
            (MmNumberOfPhysicalPages) as usize,
            (*MxFreeDescriptor).PageCount as usize,
            MxOldFreeDescriptor.PageCount as usize,
            page_count,
        );
    }

    let pfn = (*MxFreeDescriptor).BasePage;
    (*MxFreeDescriptor).BasePage = ((*MxFreeDescriptor).BasePage as usize + page_count) as _;
    (*MxFreeDescriptor).PageCount = ((*MxFreeDescriptor).PageCount as usize - page_count) as _;
    (pfn) as usize
}

#[no_mangle]
pub unsafe extern "system" fn MiComputeColorInformation() {
    let l2_associativity: ULONG;

    if MmSecondaryColors == 0 {
        l2_associativity = (*(KeGetPcr() as *mut _KPCR)).SecondLevelCacheAssociativity as u32;

        MmSecondaryColors = (*(KeGetPcr() as *mut _KPCR)).SecondLevelCacheSize;
        if l2_associativity != 0 {
            MmSecondaryColors /= l2_associativity;
        }
    }

    MmSecondaryColors >>= PAGE_SHIFT;
    if MmSecondaryColors == 0 {
        MmSecondaryColors = (MI_SECONDARY_COLORS) as u32;
    } else {
        if MmSecondaryColors > (MI_MAX_SECONDARY_COLORS) as u32 {
            MmSecondaryColors = (MI_MAX_SECONDARY_COLORS) as u32;
        }

        if MmSecondaryColors < (MI_MIN_SECONDARY_COLORS) as u32 {
            MmSecondaryColors = (MI_SECONDARY_COLORS) as u32;
        }

        if (MmSecondaryColors & (MmSecondaryColors - 1)) != 0 {
            MmSecondaryColors = (MI_SECONDARY_COLORS) as u32;
        }
    }

    MmSecondaryColorMask = MmSecondaryColors - 1;
    // KPRCB.SecondaryColorMask not in bindgen, elided
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeColorTables() {
    let mut i: ULONG;
    let mut pointer_pte: PMMPTE;
    let mut last_pte: PMMPTE;
    let mut temp_pte = ValidKernelPte;

    MmFreePagesByColor[0] = MmPfnDatabase
        .add(MmHighestPhysicalPage as usize + 1) as PMMCOLOR_TABLES;

    pointer_pte = MiAddressToPte(MmFreePagesByColor[0] as PVOID);
    last_pte = MiAddressToPte(
        ((MmFreePagesByColor[0] as ULONG_PTR)
            + (2 * MmSecondaryColors as usize * core::mem::size_of::<MMCOLOR_TABLES>())
            - 1) as PVOID,
    );

    while pointer_pte <= last_pte {
        if ((*pointer_pte).valid() as usize) == 0 {
            temp_pte.set_page_frame_number((MxGetNextPage(1)) as PFN_NUMBER);
            MI_WRITE_VALID_PTE(pointer_pte, temp_pte);

            RtlZeroMemory(MiPteToAddress(pointer_pte), PAGE_SIZE);
        }

        pointer_pte = pointer_pte.add(1);
    }

    MmFreePagesByColor[1] = MmFreePagesByColor[0].add(MmSecondaryColors as usize);

    for i in 0..MmSecondaryColors {
        (*MmFreePagesByColor[ZeroedPageList as usize].add(i as usize)).Flink = LIST_HEAD;
        (*MmFreePagesByColor[ZeroedPageList as usize].add(i as usize)).Blink = LIST_HEAD as PVOID;
        (*MmFreePagesByColor[ZeroedPageList as usize].add(i as usize)).Count = 0;
        (*MmFreePagesByColor[FreePageList as usize].add(i as usize)).Flink = LIST_HEAD;
        (*MmFreePagesByColor[FreePageList as usize].add(i as usize)).Blink = LIST_HEAD as PVOID;
        (*MmFreePagesByColor[FreePageList as usize].add(i as usize)).Count = 0;
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn MiIsRegularMemory(
    loader_block: PLOADER_PARAMETER_BLOCK,
    pfn: PFN_NUMBER,
) -> BOOLEAN {
    let mut next_entry: PLIST_ENTRY;
    let mut md_block: PMEMORY_ALLOCATION_DESCRIPTOR;

    next_entry = (*loader_block).MemoryDescriptorListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        md_block = CONTAINING_RECORD!(next_entry, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;

        if pfn >= (*md_block).BasePage as usize {
            if pfn < ((*md_block).BasePage as usize + (*md_block).PageCount as usize) {
                if (*md_block).MemoryType == LoaderFirmwarePermanent
                    || (*md_block).MemoryType == LoaderBBTMemory
                    || (*md_block).MemoryType == LoaderSpecialMemory
                {
                    break;
                }

                return 1;
            }
        } else {
            break;
        }

        next_entry = (*md_block).ListEntry.Flink;
    }

    if pfn >= MxOldFreeDescriptor.BasePage as usize
        && pfn < MxOldFreeDescriptor.BasePage as usize + MxOldFreeDescriptor.PageCount as usize
    {
        return 1;
    }

    0
}

#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn MiMapPfnDatabase(loader_block: PLOADER_PARAMETER_BLOCK) {
    let loader_block = opaque_ptr(loader_block);
    let mut free_page: PFN_NUMBER;
    let mut free_page_count: PFN_NUMBER;
    let mut pages_left: PFN_NUMBER;
    let mut base_page: PFN_NUMBER;
    let mut page_count: PFN_NUMBER;
    let mut next_entry: PLIST_ENTRY;
    let mut md_block: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut pointer_pte: PMMPTE;
    let mut last_pte: PMMPTE;
    let mut temp_pte = ValidKernelPte;

    free_page = (*MxFreeDescriptor).BasePage as usize;
    free_page_count = (*MxFreeDescriptor).PageCount as usize;
    pages_left = 0;

    next_entry = (*loader_block).MemoryDescriptorListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        md_block = CONTAINING_RECORD!(next_entry, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;

        if (*md_block).MemoryType == LoaderFirmwarePermanent
            || (*md_block).MemoryType == LoaderBBTMemory
            || (*md_block).MemoryType == LoaderSpecialMemory
        {
            next_entry = (*md_block).ListEntry.Flink;
            continue;
        }

        if md_block == MxFreeDescriptor {
            base_page = MxOldFreeDescriptor.BasePage as usize;
            page_count = MxOldFreeDescriptor.PageCount as usize;
        } else {
            base_page = (*md_block).BasePage as usize;
            page_count = (*md_block).PageCount as usize;
        }

        pointer_pte = MiAddressToPte(MmPfnDatabase.add(base_page as usize) as PVOID);
        last_pte = MiAddressToPte(
            ((MmPfnDatabase.add(base_page as usize + page_count as usize) as ULONG_PTR) - 1) as PVOID,
        );

        while pointer_pte <= last_pte {
            if ((*pointer_pte).valid() as usize) == 0 {
                temp_pte.set_page_frame_number((free_page) as PFN_NUMBER);

                free_page += 1;
                free_page_count -= 1;
                if free_page_count == 0 {
                    KeBugCheckEx(
                        INSTALL_MORE_MEMORY,
                        MmNumberOfPhysicalPages as _,
                        free_page_count as usize,
                        MxOldFreeDescriptor.PageCount as usize,
                        1,
                    );
                }

                pages_left += 1;
                MI_WRITE_VALID_PTE(pointer_pte, temp_pte);

                RtlZeroMemory(MiPteToAddress(pointer_pte), PAGE_SIZE);
            }

            pointer_pte = pointer_pte.add(1);
        }

        next_entry = (*md_block).ListEntry.Flink;
    }

    (*MxFreeDescriptor).BasePage = free_page as _;
    (*MxFreeDescriptor).PageCount = free_page_count as _;
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn MiBuildPfnDatabaseFromPages(loader_block: PLOADER_PARAMETER_BLOCK) {
    let mut pointer_pde: PMMPTE;
    let mut pointer_pte: PMMPTE;
    let mut i: ULONG;
    let count: ULONG;
    let mut j: ULONG;
    let mut page_frame_index: PFN_NUMBER;
    let startup_pd_index: PFN_NUMBER;
    let mut pte_page_index: PFN_NUMBER;
    let mut pfn1: PMMPFN;
    let mut pfn2: PMMPFN;
    let mut base_address: ULONG_PTR = 0;

    startup_pd_index = PFN_FROM_PTE(MiAddressToPde(PDE_BASE as PVOID));

    pointer_pde = MiAddressToPde(core::ptr::null_mut());
    count = (PPE_PER_PAGE * PDE_PER_PAGE) as ULONG;
    for i in 0..count {
        if ((*pointer_pde).valid() as usize) == 1 {
            page_frame_index = PFN_FROM_PTE(pointer_pde as PMMPTE);

            if MiIsRegularMemory(loader_block, page_frame_index) != 0 {
                pfn1 = MiGetPfnEntry(page_frame_index);
                (*pfn1).set_pte_frame((startup_pd_index) as PFN_NUMBER);
                (*pfn1).set_pte_address((pointer_pde as PMMPTE) as PMMPTE);
                (*pfn1).set_share_count((*pfn1).share_count().wrapping_add((1) as ULONG_PTR));
                (*pfn1).set_reference_count((1) as USHORT);
                (*pfn1).set_page_location((ActiveAndValid) as u16);
                (*pfn1).set_cache_attribute((MiNonCached) as u16);
            } else {
                pfn1 = core::ptr::null_mut();
            }

            pointer_pte = MiAddressToPte(base_address as PVOID);
            for j in 0..PTE_PER_PAGE {
                if ((*pointer_pte).valid() as usize) == 1 {
                    (*pfn1).set_share_count((*pfn1).share_count().wrapping_add((1) as ULONG_PTR));

                    pte_page_index = PFN_FROM_PTE(pointer_pte);
                    if MiIsRegularMemory(loader_block, pte_page_index) != 0 {
                        if base_address >= 0xA0000000
                            || (base_address >= MmNonPagedPoolStart as ULONG_PTR
                                && base_address
                                    < (MmNonPagedPoolStart as ULONG_PTR
                                        + MmSizeOfNonPagedPoolInBytes))
                        {
                            pfn2 = MiGetPfnEntry(pte_page_index);
                            if MmIsAddressValid(pfn2 as PVOID) != 0
                                && MmIsAddressValid(pfn2.add(1) as PVOID) != 0
                            {
                                (*pfn2).set_pte_frame((page_frame_index) as PFN_NUMBER);
                                (*pfn2).set_pte_address((pointer_pte) as PMMPTE);
                                (*pfn2).set_share_count((*pfn2).share_count().wrapping_add((1) as ULONG_PTR));
                                (*pfn2).set_reference_count((1) as USHORT);
                                (*pfn2).set_page_location((ActiveAndValid) as u16);
                                (*pfn2).set_cache_attribute((MiNonCached) as u16);
                            }
                        }
                    }
                }

                pointer_pte = pointer_pte.add(1);
                base_address += PAGE_SIZE;
            }
        } else {
            base_address += PDE_MAPPED_VA;
        }

        pointer_pde = pointer_pde.add(1);
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn MiBuildPfnDatabaseZeroPage() {
    let pfn1: PMMPFN;
    let pointer_pde: PMMPTE;

    pfn1 = MiGetPfnEntry(MmLowestPhysicalPage);
    if MmLowestPhysicalPage == 0 && (*pfn1).reference_count() == 0 {
        pointer_pde = MiAddressToPde(0xFFFFFFFF as PVOID);
        (*pfn1).set_pte_frame(PFN_FROM_PTE(pointer_pde as PMMPTE));
        (*pfn1).set_pte_address(pointer_pde as PMMPTE);
        (*pfn1).set_share_count((*pfn1).share_count().wrapping_add((1) as ULONG_PTR));
        (*pfn1).set_reference_count((0xFFF0) as USHORT);
        (*pfn1).set_page_location((ActiveAndValid) as u16);
        (*pfn1).set_cache_attribute((MiNonCached) as u16);
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn MiBuildPfnDatabaseFromLoaderBlock(loader_block: PLOADER_PARAMETER_BLOCK) {
    let mut next_entry: PLIST_ENTRY;
    let mut page_count: PFN_NUMBER = 0;
    let mut md_block: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut page_frame_index: PFN_NUMBER;
    let mut pfn1: PMMPFN;
    let mut pointer_pte: PMMPTE;
    let mut pointer_pde: PMMPTE;
    let mut old_irql: KIRQL;

    next_entry = (*loader_block).MemoryDescriptorListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        md_block = CONTAINING_RECORD!(next_entry, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;

        page_count = (*md_block).PageCount as _;
        page_frame_index = (*md_block).BasePage as _;

        if page_frame_index > MmHighestPhysicalPage {
            break;
        }

        if (page_frame_index + page_count) > (MmHighestPhysicalPage + 1) {
            page_count = MmHighestPhysicalPage + 1 - page_frame_index;
            (*md_block).PageCount = page_count as _;

            if page_count == 0 {
                break;
            }
        }

        match (*md_block).MemoryType {
            LoaderBad => {}
            LoaderFree | LoaderLoadedProgram | LoaderFirmwareTemporary | LoaderOsloaderStack => {
                page_frame_index += page_count - 1;
                pfn1 = MiGetPfnEntry(page_frame_index);

                old_irql = MiAcquirePfnLock();
                while page_count > 0 {
                    page_count -= 1;
                    if (*pfn1).reference_count() == 0 {
                        (*pfn1).set_cache_attribute(MiNonCached as u16);
                        MiInsertPageInFreeList(page_frame_index);
                    }

                    pfn1 = pfn1.sub(1);
                    page_frame_index -= 1;
                }

                MiReleasePfnLock(old_irql);
            }
            LoaderFirmwarePermanent | LoaderSpecialMemory | LoaderBBTMemory => {}
            _ => {
                pointer_pte = MiAddressToPte((KSEG0_BASE + (page_frame_index << PAGE_SHIFT)) as PVOID);
                pfn1 = MiGetPfnEntry(page_frame_index);
                while page_count > 0 {
                    page_count -= 1;
                    pointer_pde = MiAddressToPde((KSEG0_BASE + (page_frame_index << PAGE_SHIFT)) as PVOID);
                    if (*pfn1).reference_count() == 0 {
                        (*pfn1).set_pte_frame(PFN_FROM_PTE(pointer_pde as PMMPTE));
                        (*pfn1).set_pte_address(pointer_pte as PMMPTE);
                        (*pfn1).set_share_count((*pfn1).share_count().wrapping_add((1) as ULONG_PTR));
                        (*pfn1).set_reference_count((1) as USHORT);
                        (*pfn1).set_page_location((ActiveAndValid) as u16);
                        (*pfn1).set_cache_attribute((MiNonCached) as u16);

                        if (*md_block).MemoryType == LoaderXIPRom {
                            (*pfn1).set_flink(0);
                            (*pfn1).set_share_count((0) as ULONG_PTR);
                            (*pfn1).set_reference_count((0) as USHORT);
                            (*pfn1).set_page_location((0) as u16);
                            mmpfn_set_rom_pte(pfn1, true);
                            mmpfn_set_in_page_error_u4(pfn1, (0) != 0);
                            (*pfn1).set_prototype_pte((1) != 0);
                        }
                    }

                    pfn1 = pfn1.add(1);
                    page_frame_index += 1;
                    pointer_pte = pointer_pte.add(1);
                }
            }
        }

        next_entry = (*md_block).ListEntry.Flink;
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn MiBuildPfnDatabaseSelf() {
    let mut pointer_pte: PMMPTE;
    let last_pte: PMMPTE;
    let mut pfn1: PMMPFN;

    pointer_pte = MiAddressToPte(MiGetPfnEntry(MmLowestPhysicalPage) as PVOID);
    last_pte = MiAddressToPte(MiGetPfnEntry(MmHighestPhysicalPage) as PVOID);
    while pointer_pte <= last_pte {
        if ((*pointer_pte).valid() as usize) == 1 {
            pfn1 = MiGetPfnEntry((*pointer_pte).page_frame_number());
            (*pfn1).set_share_count((1) as ULONG_PTR);
            (*pfn1).set_reference_count((1) as USHORT);
        }

        pointer_pte = pointer_pte.add(1);
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "system" fn MiInitializePfnDatabase(loader_block: PLOADER_PARAMETER_BLOCK) {
    let loader_block = opaque_ptr(loader_block);
    #[cfg(not(target_arch = "aarch64"))]
    MiBuildPfnDatabaseFromPages(loader_block);

    MiBuildPfnDatabaseZeroPage();

    MiBuildPfnDatabaseFromLoaderBlock(loader_block);

    MiBuildPfnDatabaseSelf();
}

#[no_mangle]
pub unsafe extern "system" fn MmFreeLoaderBlock(loader_block: PLOADER_PARAMETER_BLOCK) {
    let loader_block = opaque_ptr(loader_block);
    let mut next_md: PLIST_ENTRY;
    let mut md_block: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut i: ULONG_PTR;
    let mut base_page: PFN_NUMBER;
    let mut loader_pages: PFN_NUMBER;
    let mut pfn1: PMMPFN;
    let mut old_irql: KIRQL;
    let mut buffer: PPHYSICAL_MEMORY_RUN;
    let mut entry: PPHYSICAL_MEMORY_RUN;

    i = 0;
    next_md = (*loader_block).MemoryDescriptorListHead.Flink;
    while next_md != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        md_block = CONTAINING_RECORD!(next_md, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;
        i += 1;
        next_md = (*md_block).ListEntry.Flink;
    }

    buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        i * core::mem::size_of::<_PHYSICAL_MEMORY_RUN>(),
        TAG_MM,
    ) as PPHYSICAL_MEMORY_RUN;
    entry = buffer;

    next_md = (*loader_block).MemoryDescriptorListHead.Flink;
    while next_md != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        md_block = CONTAINING_RECORD!(next_md, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;

        match (*md_block).MemoryType {
            LoaderRegistryData | LoaderOsloaderHeap | LoaderNlsData => {
                (*entry).BasePage = (*md_block).BasePage;
                (*entry).PageCount = (*md_block).PageCount;
                entry = entry.add(1);
            }
            _ => {}
        }

        next_md = (*md_block).ListEntry.Flink;
    }

    old_irql = MiAcquirePfnLock();

    loader_pages = 0;
    while entry > buffer {
        entry = entry.sub(1);

        i = (*entry).PageCount as ULONG_PTR;
        base_page = ((*entry).BasePage) as usize;

        pfn1 = MiGetPfnEntry(base_page);
        while i > 0 {
            i -= 1;
            if (*pfn1).reference_count() == 0 && (*pfn1).flink() == 0 {
                (*pfn1).set_pte_address((base_page << PAGE_SHIFT) as PMMPTE);
                MiInsertPageInFreeList(base_page);
                loader_pages += 1;
            } else if base_page != 0 {
                (*(*pfn1).pte_address()).Long = 0;
                MI_SET_PFN_DELETED(pfn1);
                MiDecrementShareCount(pfn1 as *mut _MMPFN, (base_page) as _);
                loader_pages += 1;
            }

            pfn1 = pfn1.add(1);
            base_page += 1;
        }
    }

    MiReleasePfnLock(old_irql);
    KeFlushCurrentTb();

    ExFreePoolWithTag(buffer as PVOID, TAG_MM);

    #[cfg(target_arch = "aarch64")]
    MiUnmapLowGuardArm64();
}

/// Unmap VA 0-64K, which FreeLDR identity-mapped (VA == PA) and nothing ever
/// tore down, so a NULL dereference read physical page 0 instead of faulting.
/// Runs once the loader's memory is released: the kernel no longer needs the
/// loader's identity mappings then. Only the guard region goes; higher identity
/// entries (the PL011 at 0x09000000) still have users, and the lazy cleanup in
/// NtAllocateVirtualMemory handles user allocations that land on the rest.
/// AP bring-up does not touch PA 0 either: it starts cores with PSCI CPU_ON on
/// a private identity map (hal mpboot.rs), not through the Pi 4's spin table.
#[cfg(target_arch = "aarch64")]
unsafe fn MiUnmapLowGuardArm64() {
    const GUARD_END: ULONG_PTR = 0x10000;
    // Descriptor bits[1:0]: 0b11 = table/page, 0b01 = 2 MB block (see virtual.rs).
    const NOT_LARGE_PAGE: ULONG_PTR = 1 << 1;
    let mut cleared: u32 = 0;
    let mut va: ULONG_PTR = 0;
    while va < GUARD_END {
        let pxe = MiAddressToPxe(va as PVOID);
        let ppe = MiAddressToPpe(va as PVOID);
        let pde = MiAddressToPde(va as PVOID);
        if !((*pxe).valid() && (*ppe).valid() && (*pde).valid()) {
            break; // nothing mapped below this PDE
        }
        if ((*pde).Long & NOT_LARGE_PAGE) == 0 {
            // A 2 MB block also covers live mappings above the guard; leave it
            // and say so rather than drop them.
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[MM] arm64: VA 0 is inside a 2MB block (PDE %p); low guard left mapped\n\0".as_ptr() as *const i8,
                (*pde).Long as PVOID,
            );
            return;
        }
        let pte = MiAddressToPte(va as PVOID);
        if (*pte).valid() {
            (*pte).Long = 0;
            cleared += 1;
        }
        va += PAGE_SIZE as ULONG_PTR;
    }
    if cleared != 0 {
        KeFlushEntireTb(1, 1);
    }
    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[MM] arm64: low guard VA 0-64K unmapped (%u identity pages cleared); MmIsAddressValid(0x10)=%u\n\0".as_ptr() as *const i8,
        cleared,
        MmIsAddressValid(0x10 as PVOID) as u32,
    );
}

unsafe fn MiAdjustWorkingSetManagerParameters(client: BOOLEAN) {
    if MmNumberOfPhysicalPages >= ((63 * _1MB) / PAGE_SIZE) as PFN_NUMBER {
        MmPlentyFreePages *= 2;
    }
}

unsafe fn MiNotifyMemoryEvents() {
    if MmAvailablePages < MmLowMemoryThreshold {
        if KeReadStateEvent(MiHighMemoryEvent) != 0 {
            KeClearEvent(MiHighMemoryEvent as *mut c_void);
        }
        if KeReadStateEvent(MiLowMemoryEvent) == 0 {
            KeSetEvent(MiLowMemoryEvent as *mut c_void, 0, 0);
        }
    } else if MmAvailablePages < MmHighMemoryThreshold {
        if KeReadStateEvent(MiHighMemoryEvent) != 0 {
            KeClearEvent(MiHighMemoryEvent as *mut c_void);
        }
        if KeReadStateEvent(MiLowMemoryEvent) != 0 {
            KeClearEvent(MiLowMemoryEvent as *mut c_void);
        }
    } else {
        if KeReadStateEvent(MiLowMemoryEvent) != 0 {
            KeClearEvent(MiLowMemoryEvent as *mut c_void);
        }
        if KeReadStateEvent(MiHighMemoryEvent) == 0 {
            KeSetEvent(MiHighMemoryEvent as *mut c_void, 0, 0);
        }
    }
}

unsafe fn MiCreateMemoryEvent(name: PUNICODE_STRING, event: *mut PKEVENT) -> NTSTATUS {
    let mut dacl: PACL;
    let mut event_handle: HANDLE = core::ptr::null_mut();
    let dacl_length: ULONG;
    let mut status: NTSTATUS;
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut security_descriptor: _SECURITY_DESCRIPTOR = core::mem::zeroed();

    status = RtlCreateSecurityDescriptor(
        core::ptr::addr_of_mut!(security_descriptor) as *mut c_void,
        SECURITY_DESCRIPTOR_REVISION,
    );
    if status < 0 {
        return status;
    }

    dacl_length = core::mem::size_of::<ACL>() as ULONG
        + 3 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
        + RtlLengthSid(SeLocalSystemSid)
        + RtlLengthSid(SeAliasAdminsSid)
        + RtlLengthSid(SeWorldSid);

    dacl = ExAllocatePoolWithTag(PagedPool, dacl_length as SIZE_T, TAG_DACL) as PACL;
    if dacl.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    status = RtlCreateAcl(dacl, dacl_length, ACL_REVISION);
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_DACL);
        return status;
    }

    status = RtlAddAccessAllowedAce(
        dacl,
        ACL_REVISION,
        SYNCHRONIZE | EVENT_QUERY_STATE | READ_CONTROL,
        SeWorldSid,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_DACL);
        return status;
    }

    status = RtlAddAccessAllowedAce(dacl, ACL_REVISION, EVENT_ALL_ACCESS, SeAliasAdminsSid);
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_DACL);
        return status;
    }

    status = RtlAddAccessAllowedAce(dacl, ACL_REVISION, EVENT_ALL_ACCESS, SeLocalSystemSid);
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_DACL);
        return status;
    }

    status = RtlSetDaclSecurityDescriptor(
        core::ptr::addr_of_mut!(security_descriptor) as PVOID,
        1u8,
        dacl as PVOID,
        0u8,
    );
    if status < 0 {
        ExFreePoolWithTag(dacl as PVOID, TAG_DACL);
        return status;
    }

    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(object_attributes),
        name as PUNICODE_STRING,
        OBJ_KERNEL_HANDLE | OBJ_PERMANENT,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(security_descriptor) as PVOID,
    );

    status = ZwCreateEvent(
        core::ptr::addr_of_mut!(event_handle),
        EVENT_ALL_ACCESS,
        core::ptr::addr_of_mut!(object_attributes),
        _EVENT_TYPE_NotificationEvent,
        0,
    );

    ExFreePoolWithTag(dacl as PVOID, TAG_DACL);

    if status >= 0 {
        status = ObReferenceObjectByHandle(
            event_handle,
            EVENT_MODIFY_STATE,
            ExEventObjectType as *mut _OBJECT_TYPE,
            (0i32) as i8,
            event as *mut PVOID,
            core::ptr::null_mut(),
        );
        ZwClose(event_handle);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn MiInitializeMemoryEvents() -> BOOLEAN {
    // RTL_CONSTANT_STRING(L"\\KernelObjects\\LowMemoryCondition")
    static LOW_STR: [u16; 34] = {
        let mut buf = [0u16; 34];
        let s = b"\\KernelObjects\\LowMemoryCondition";
        let mut i = 0;
        while i < 33 { buf[i] = s[i] as u16; i += 1; }
        buf
    };
    let low_string = _UNICODE_STRING {
        Length: 66,   // 33 chars * 2
        MaximumLength: 68,
        Buffer: LOW_STR.as_ptr() as *mut _,
    };
    // RTL_CONSTANT_STRING(L"\\KernelObjects\\HighMemoryCondition")
    static HIGH_STR: [u16; 35] = {
        let mut buf = [0u16; 35];
        let s = b"\\KernelObjects\\HighMemoryCondition";
        let mut i = 0;
        while i < 34 { buf[i] = s[i] as u16; i += 1; }
        buf
    };
    let high_string = _UNICODE_STRING {
        Length: 68,   // 34 chars * 2
        MaximumLength: 70,
        Buffer: HIGH_STR.as_ptr() as *mut _,
    };
    // RTL_CONSTANT_STRING(L"\\KernelObjects\\LowPagedPoolCondition")
    static LOW_PAGED_STR: [u16; 37] = {
        let mut buf = [0u16; 37];
        let s = b"\\KernelObjects\\LowPagedPoolCondition";
        let mut i = 0;
        while i < 36 { buf[i] = s[i] as u16; i += 1; }
        buf
    };
    let low_paged_pool_string = _UNICODE_STRING {
        Length: 72,   // 36 chars * 2
        MaximumLength: 74,
        Buffer: LOW_PAGED_STR.as_ptr() as *mut _,
    };
    // RTL_CONSTANT_STRING(L"\\KernelObjects\\HighPagedPoolCondition")
    static HIGH_PAGED_STR: [u16; 38] = {
        let mut buf = [0u16; 38];
        let s = b"\\KernelObjects\\HighPagedPoolCondition";
        let mut i = 0;
        while i < 37 { buf[i] = s[i] as u16; i += 1; }
        buf
    };
    let high_paged_pool_string = _UNICODE_STRING {
        Length: 74,   // 37 chars * 2
        MaximumLength: 76,
        Buffer: HIGH_PAGED_STR.as_ptr() as *mut _,
    };
    // RTL_CONSTANT_STRING(L"\\KernelObjects\\LowNonPagedPoolCondition")
    static LOW_NONPAGED_STR: [u16; 40] = {
        let mut buf = [0u16; 40];
        let s = b"\\KernelObjects\\LowNonPagedPoolCondition";
        let mut i = 0;
        while i < 39 { buf[i] = s[i] as u16; i += 1; }
        buf
    };
    let low_non_paged_pool_string = _UNICODE_STRING {
        Length: 78,   // 39 chars * 2
        MaximumLength: 80,
        Buffer: LOW_NONPAGED_STR.as_ptr() as *mut _,
    };
    // RTL_CONSTANT_STRING(L"\\KernelObjects\\HighNonPagedPoolCondition")
    static HIGH_NONPAGED_STR: [u16; 41] = {
        let mut buf = [0u16; 41];
        let s = b"\\KernelObjects\\HighNonPagedPoolCondition";
        let mut i = 0;
        while i < 40 { buf[i] = s[i] as u16; i += 1; }
        buf
    };
    let high_non_paged_pool_string = _UNICODE_STRING {
        Length: 80,   // 40 chars * 2
        MaximumLength: 82,
        Buffer: HIGH_NONPAGED_STR.as_ptr() as *mut _,
    };
    let mut status: NTSTATUS;

    if MmLowMemoryThreshold != 0 {
        MmLowMemoryThreshold *= _1MB / PAGE_SIZE;
    } else {
        MmLowMemoryThreshold = MmPlentyFreePages;

        if MmNumberOfPhysicalPages > 0x40000 {
            MmLowMemoryThreshold = (32 * _1MB) / PAGE_SIZE;
            MmLowMemoryThreshold += (((MmNumberOfPhysicalPages - 0x40000) >> 7)) as usize;
        } else if MmNumberOfPhysicalPages > 0x8000 {
            MmLowMemoryThreshold += (((MmNumberOfPhysicalPages - 0x8000) >> 5)) as usize;
        }

        if MmLowMemoryThreshold > (64 * _1MB) / PAGE_SIZE {
            MmLowMemoryThreshold = (64 * _1MB) / PAGE_SIZE;
        }
    }

    if MmHighMemoryThreshold != 0 {
        MmHighMemoryThreshold *= _1MB / PAGE_SIZE;
    } else {
        MmHighMemoryThreshold = 3 * MmLowMemoryThreshold;
    }

    if MmHighMemoryThreshold < MmLowMemoryThreshold {
        MmHighMemoryThreshold = MmLowMemoryThreshold;
    }

    _A3_TRACE!(b"[MM1] MiCreateMemoryEvent LowMem\r\n");
    status = MiCreateMemoryEvent(
        core::ptr::addr_of!(low_string) as PUNICODE_STRING,
        core::ptr::addr_of_mut!(MiLowMemoryEvent),
    );
    if status < 0 {
        return 0;
    }
    _A3_TRACE!(b"[MM1] MiCreateMemoryEvent HighMem\r\n");
    status = MiCreateMemoryEvent(
        core::ptr::addr_of!(high_string) as PUNICODE_STRING,
        core::ptr::addr_of_mut!(MiHighMemoryEvent),
    );
    if status < 0 {
        return 0;
    }
    _A3_TRACE!(b"[MM1] MiCreateMemoryEvent LowPagedPool\r\n");
    status = MiCreateMemoryEvent(
        core::ptr::addr_of!(low_paged_pool_string) as PUNICODE_STRING,
        core::ptr::addr_of_mut!(MiLowPagedPoolEvent),
    );
    if status < 0 {
        return 0;
    }
    _A3_TRACE!(b"[MM1] MiCreateMemoryEvent HighPagedPool\r\n");
    status = MiCreateMemoryEvent(
        core::ptr::addr_of!(high_paged_pool_string) as PUNICODE_STRING,
        core::ptr::addr_of_mut!(MiHighPagedPoolEvent),
    );
    if status < 0 {
        return 0;
    }
    _A3_TRACE!(b"[MM1] MiCreateMemoryEvent LowNonPagedPool\r\n");
    status = MiCreateMemoryEvent(
        core::ptr::addr_of!(low_non_paged_pool_string) as PUNICODE_STRING,
        core::ptr::addr_of_mut!(MiLowNonPagedPoolEvent),
    );
    if status < 0 {
        return 0;
    }
    _A3_TRACE!(b"[MM1] MiCreateMemoryEvent HighNonPagedPool\r\n");
    status = MiCreateMemoryEvent(
        core::ptr::addr_of!(high_non_paged_pool_string) as PUNICODE_STRING,
        core::ptr::addr_of_mut!(MiHighNonPagedPoolEvent),
    );
    if status < 0 {
        return 0;
    }

    _A3_TRACE!(b"[MM1] MiInitializePoolEvents\r\n");
    MiInitializePoolEvents();

    _A3_TRACE!(b"[MM1] MiNotifyMemoryEvents\r\n");
    MiNotifyMemoryEvents();
    _A3_TRACE!(b"[MM1] MiInitializeMemoryEvents done\r\n");
    1
}

unsafe fn MiAddHalIoMappings() {
    let mut base_address: PVOID;
    let mut pointer_pde: PMMPTE;
    let last_pde: PMMPTE;
    let mut pointer_pte: PMMPTE;
    let mut j: ULONG;
    let mut page_frame_index: PFN_NUMBER;

    base_address = MM_HAL_VA_START as PVOID;

    pointer_pde = MiAddressToPde(base_address);
    last_pde = MiAddressToPde(MM_HAL_VA_END as PVOID);

    // DBG: MiAddHalIoMappings entry

    while pointer_pde <= last_pde {
        if ((*pointer_pde).valid() as usize) == 1 && MI_IS_PAGE_LARGE(pointer_pde) == 0 {
            // DBG: valid PDE, scanning PTEs
            pointer_pte = MiAddressToPte(base_address);
            for j in 0..PTE_PER_PAGE {
                if ((*pointer_pte).valid() as usize) == 1 {
                    page_frame_index = PFN_FROM_PTE(pointer_pte);
                    if MiGetPfnEntry(page_frame_index).is_null() {
                    }
                }

                base_address = (base_address as ULONG_PTR + PAGE_SIZE) as PVOID;
                pointer_pte = pointer_pte.add(1);
            }
            // DBG: done scanning PTEs
        } else {
            base_address = (base_address as ULONG_PTR + PDE_MAPPED_VA) as PVOID;
        }

        pointer_pde = pointer_pde.add(1);
    }
    // DBG: MiAddHalIoMappings done
}

#[no_mangle]
pub unsafe extern "system" fn MmDumpArmPfnDatabase(status_only: BOOLEAN) {
    let mut i: ULONG;
    let mut pfn1: PMMPFN;
    let mut old_irql: KIRQL = 0;
    let mut active_pages: ULONG = 0;
    let mut free_pages: ULONG = 0;
    let mut other_pages: ULONG = 0;

    // [guard] KeRaiseIrql(HIGH_LEVEL as u8, core::ptr::addr_of_mut!(old_irql));
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL as u8) };
    for i in 0..=MmHighestPhysicalPage as ULONG {
        pfn1 = MiGetPfnEntry(i as PFN_NUMBER);
        if pfn1.is_null() {
            continue;
        }

        match (*pfn1).page_location() {
            ActiveAndValid => {
                active_pages += 1;
            }
            ZeroedPageList => {
                free_pages += 1;
            }
            FreePageList => {
                free_pages += 1;
            }
            _ => {
                other_pages += 1;
            }
        }
    }

    drop(irql_guard);
}

#[no_mangle]
pub unsafe extern "system" fn MmInitializeMemoryLimits(
    loader_block: PLOADER_PARAMETER_BLOCK,
    include_type: PBOOLEAN,
) -> PPHYSICAL_MEMORY_DESCRIPTOR {
    let loader_block = opaque_ptr(loader_block);
    let include_type = opaque_ptr(include_type);
    let mut next_entry: PLIST_ENTRY;
    let mut run: ULONG = 0;
    let initial_runs: ULONG;
    let mut next_page: PFN_NUMBER = PFN_NUMBER::MAX;
    let mut page_count: PFN_NUMBER = 0;
    let mut buffer: PPHYSICAL_MEMORY_DESCRIPTOR;
    let mut new_buffer: PPHYSICAL_MEMORY_DESCRIPTOR;
    let mut md_block: PMEMORY_ALLOCATION_DESCRIPTOR;

    #[cfg(target_arch = "aarch64")]
    _A3_TRACE!(b"[MM] MmInitializeMemoryLimits enter\r\n");

    initial_runs = MiNumberDescriptors;

    #[cfg(target_arch = "aarch64")]
    _A3_TRACE!(b"[MM] MmInitializeMemoryLimits alloc\r\n");

    buffer = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_PHYSICAL_MEMORY_DESCRIPTOR>()
            + core::mem::size_of::<_PHYSICAL_MEMORY_RUN>() * (initial_runs as usize - 1),
        TAG_MM,
    ) as PPHYSICAL_MEMORY_DESCRIPTOR;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    (*buffer).NumberOfRuns = initial_runs;

    #[cfg(target_arch = "aarch64")]
    _A3_TRACE!(b"[MM] MmInitializeMemoryLimits loop\r\n");

    next_entry = (*loader_block).MemoryDescriptorListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*loader_block).MemoryDescriptorListHead) as PLIST_ENTRY {
        md_block = CONTAINING_RECORD!(next_entry, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;

        if (*md_block).MemoryType < LoaderMaximum
            && *include_type.add((*md_block).MemoryType as usize) != 0
        {
            page_count += ((*md_block).PageCount) as usize;

            if (*md_block).BasePage == (next_page) as _ {
                (*(*buffer).Run.as_mut_ptr().add(run as usize - 1)).PageCount += ((*md_block).PageCount) as usize;
                next_page += ((*md_block).PageCount) as usize;
            } else {
                (*(*buffer).Run.as_mut_ptr().add(run as usize)).BasePage = ((*md_block).BasePage) as usize;
                (*(*buffer).Run.as_mut_ptr().add(run as usize)).PageCount = ((*md_block).PageCount) as usize;
                next_page = (*(*buffer).Run.as_mut_ptr().add(run as usize)).BasePage + (*(*buffer).Run.as_mut_ptr().add(run as usize)).PageCount;

                run += 1;
            }
        }

        next_entry = (*md_block).ListEntry.Flink;
    }

    if initial_runs > run {
        new_buffer = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_PHYSICAL_MEMORY_DESCRIPTOR>()
                + core::mem::size_of::<_PHYSICAL_MEMORY_RUN>() * (run as usize - 1),
            TAG_MM,
        ) as PPHYSICAL_MEMORY_DESCRIPTOR;
        if !new_buffer.is_null() {
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*new_buffer).Run) as PVOID,
                core::ptr::addr_of_mut!((*buffer).Run) as PVOID,
                core::mem::size_of::<_PHYSICAL_MEMORY_RUN>() * run as usize,
            );
            ExFreePoolWithTag(buffer as PVOID, TAG_MM);

            buffer = new_buffer;
        }
    }

    (*buffer).NumberOfRuns = run;
    (*buffer).NumberOfPages = page_count;
    buffer
}

unsafe fn MiBuildPagedPool() {
    // DBG: MiBuildPagedPool entry

    let mut pointer_pte: PMMPTE;
    let mut pointer_pde: PMMPTE;
    let mut temp_pde = ValidKernelPde;
    let mut page_frame_index: PFN_NUMBER;
    let mut old_irql: KIRQL;
    let mut size: SIZE_T;
    let mut number_of_pages: SIZE_T;
    let mut number_of_pdes: SIZE_T;
    let mut bit_map_size: ULONG;

    #[cfg(not(target_pointer_width = "64"))]
    {
        let mut temp_pte = ValidKernelPte;

        pointer_pte = MiAddressToPte(PDE_BASE as PVOID);
        MmSystemPageDirectory[0] = PFN_FROM_PTE(pointer_pte);

        pointer_pte = MiReserveSystemPtes(1, SystemPteSpace);
        MmSystemPagePtes = MiPteToAddress(pointer_pte) as PMMPTE;

        temp_pte.set_page_frame_number((MmSystemPageDirectory[0]) as PFN_NUMBER);
        MI_WRITE_VALID_PTE(pointer_pte, temp_pte);
    }

    // DBG: after 32-bit page directory setup

    #[cfg(target_arch = "x86")]
    {
        MmSizeOfPagedPoolInBytes = 2 * MmMaximumNonPagedPoolInBytes;
        if MmSizeOfPagedPoolInBytes
            > ((MmNonPagedSystemStart as ULONG_PTR) - (MmPagedPoolStart as ULONG_PTR))
        {
            MmSizeOfPagedPoolInBytes =
                (MmNonPagedSystemStart as ULONG_PTR) - (MmPagedPoolStart as ULONG_PTR);
        }
    }

    size = MmSizeOfPagedPoolInBytes;
    if size < MI_MIN_INIT_PAGED_POOLSIZE {
        size = MI_MIN_INIT_PAGED_POOLSIZE;
    }
    number_of_pages = BYTES_TO_PAGES(size);

    number_of_pdes = (number_of_pages + (PTE_PER_PAGE - 1)) / PTE_PER_PAGE;

    MmSizeOfPagedPoolInBytes = number_of_pdes * PTE_PER_PAGE * PAGE_SIZE;
    MmSizeOfPagedPoolInPages = MmSizeOfPagedPoolInBytes >> PAGE_SHIFT;

    #[cfg(target_arch = "x86")]
    {}

    MmPagedPoolEnd =
        ((MmPagedPoolStart as ULONG_PTR + MmSizeOfPagedPoolInBytes) - 1) as PVOID;

    // DBG: before MiAcquirePfnLock in MiBuildPagedPool

    old_irql = MiAcquirePfnLock();

    #[cfg(target_pointer_width = "64")]
    {
        let mut temp_ppe = ValidKernelPpe;
        let mut pointer_ppe: PMMPPE;

        pointer_ppe = MiAddressToPpe(MmPagedPoolStart);
        while pointer_ppe <= MiAddressToPpe(MmPagedPoolEnd) {
            if ((*pointer_ppe).valid() as usize) == 0 {
                temp_ppe.set_page_frame_number((MiRemoveZeroPage(0)) as PFN_NUMBER);
                MI_WRITE_VALID_PPE(pointer_ppe, temp_ppe);
                MiInitializePfnForOtherProcess(
                    temp_ppe.page_frame_number(),
                    pointer_ppe as PMMPTE,
                    PFN_FROM_PTE(MiAddressToPte(pointer_ppe as PVOID)),
                );
            }
            pointer_ppe = pointer_ppe.add(1);
        }
    }

    pointer_pde = MiAddressToPde(MmPagedPoolStart);

    // DBG: before RtlZeroMemory in MiBuildPagedPool

    RtlZeroMemory(
        pointer_pde as PVOID,
        MiAddressToPde(MmPagedPoolEnd) as usize - pointer_pde as usize
            + core::mem::size_of::<_MMPTE>(),
    );

    // DBG: after RtlZeroMemory, before PFN setup

    pointer_pte = MiAddressToPte(MmPagedPoolStart);
    MmPagedPoolInfo.FirstPteForPagedPool = pointer_pte as *mut c_void;
    MmPagedPoolInfo.LastPteForPagedPool = MiAddressToPte(MmPagedPoolEnd) as *mut c_void;

    // DBG: before MiRemoveZeroPage

    MI_SET_USAGE(MI_USAGE_PAGED_POOL);
    super::helpers::MI_SET_PROCESS2(core::ptr::null_mut());
    page_frame_index = MiRemoveZeroPage(0);

    // DBG: after MiRemoveZeroPage

    temp_pde.set_page_frame_number((page_frame_index) as PFN_NUMBER);
    MI_WRITE_VALID_PDE(pointer_pde, temp_pde);

    // DBG: after MI_WRITE_VALID_PDE

    #[cfg(target_pointer_width = "64")]
    {
        MiInitializePfnForOtherProcess(
            page_frame_index,
            pointer_pde as PMMPTE,
            PFN_FROM_PTE(MiAddressToPpe(MmPagedPoolStart) as PMMPTE),
        );
    }
    #[cfg(not(target_pointer_width = "64"))]
    {
        // C code: MmSystemPageDirectory[(PointerPde - (PMMPDE)PDE_BASE) / PDE_PER_PAGE]
        // C pointer subtraction gives element count, not byte count.
        // In Rust we must divide byte offset by sizeof(MMPTE) first.
        let pde_index = (pointer_pde as usize - PDE_BASE)
            / core::mem::size_of::<_MMPTE>()
            / PDE_PER_PAGE;
        MiInitializePfnForOtherProcess(
            page_frame_index,
            pointer_pde as PMMPTE,
            MmSystemPageDirectory[pde_index],
        );
    }

    // DBG: after MiInitializePfnForOtherProcess

    MiReleasePfnLock(old_irql);

    // DBG: after PfnLock release, before bitmap alloc

    MmPagedPoolInfo.NextPdeForPagedPoolExpansion = pointer_pde.add(1) as *mut c_void;

    number_of_pages = number_of_pdes * PTE_PER_PAGE;
    bit_map_size = number_of_pages as ULONG;
    size = core::mem::size_of::<_RTL_BITMAP>() + (((bit_map_size + 31) / 32) * 4) as usize;

    MmPagedPoolInfo.PagedPoolAllocationMap =
        ExAllocatePoolWithTag(NonPagedPool, size, TAG_MM) as *mut _RTL_BITMAP as *mut RTL_BITMAP;

    RtlInitializeBitMap(
        MmPagedPoolInfo.PagedPoolAllocationMap,
        (MmPagedPoolInfo.PagedPoolAllocationMap as *mut _RTL_BITMAP).add(1) as *mut ULONG,
        bit_map_size,
    );
    RtlSetAllBits(MmPagedPoolInfo.PagedPoolAllocationMap);
    RtlClearBits(MmPagedPoolInfo.PagedPoolAllocationMap, 0, (PTE_PER_PAGE) as u32);

    MmPagedPoolInfo.EndOfPagedPoolBitmap =
        ExAllocatePoolWithTag(NonPagedPool, size, TAG_MM) as *mut _RTL_BITMAP as *mut RTL_BITMAP;
    RtlInitializeBitMap(
        MmPagedPoolInfo.EndOfPagedPoolBitmap,
        (MmPagedPoolInfo.EndOfPagedPoolBitmap as *mut _RTL_BITMAP).add(1) as *mut ULONG,
        bit_map_size,
    );

    RtlClearAllBits(MmPagedPoolInfo.EndOfPagedPoolBitmap);

    // DBG: before InitializePool(PagedPool)

    InitializePool(PagedPool, 0);

    // DBG: after InitializePool, before MiInitializeSpecialPool

    MiInitializeSpecialPool();

    // Tripwire: report arming outcome loudly + optional destructive selftest.
    crate::tripwire::vibe_tripwire_post_pool_init();

    // DBG: after MiInitializeSpecialPool

    MiLowPagedPoolThreshold = (30 * _1MB) >> PAGE_SHIFT;
    if MiLowPagedPoolThreshold > size / 5 {
        MiLowPagedPoolThreshold = size / 5;
    }

    MiHighPagedPoolThreshold = (60 * _1MB) >> PAGE_SHIFT;
    if MiHighPagedPoolThreshold > (size * 2) / 5 {
        MiHighPagedPoolThreshold = (size * 2) / 5;
    }

    // DBG: before MiInitializeSystemSpaceMap

    MiInitializeSystemSpaceMap(core::ptr::null_mut());

    // DBG: MiBuildPagedPool exit
}

unsafe fn MiDbgDumpMemoryDescriptors() {
    let mut next_entry: PLIST_ENTRY;
    let mut md: PMEMORY_ALLOCATION_DESCRIPTOR;
    let mut total_pages: PFN_NUMBER = 0;

    next_entry = (*KeLoaderBlock).MemoryDescriptorListHead.Flink;
    while next_entry != core::ptr::addr_of_mut!((*KeLoaderBlock).MemoryDescriptorListHead) as PLIST_ENTRY {
        md = CONTAINING_RECORD!(next_entry, _MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
            as PMEMORY_ALLOCATION_DESCRIPTOR;
        total_pages += ((*md).PageCount) as usize;
        next_entry = (*md).ListEntry.Flink;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmArmInitSystem(
    phase: ULONG,
    loader_block: PLOADER_PARAMETER_BLOCK,
) -> BOOLEAN {
    let loader_block = opaque_ptr(loader_block);
    let mut i: ULONG;
    let mut include_type: [BOOLEAN; LoaderMaximum as usize] = [0; LoaderMaximum as usize];
    let mut bitmap: PVOID;
    let mut run: PPHYSICAL_MEMORY_RUN;
    let mut page_count: PFN_NUMBER;

    #[cfg(target_arch = "aarch64")]
    _A3_TRACE!(b"[MM] MmArmInitSystem entered\r\n");

    if MiDbgEnableMdDump != 0 {
        MiDbgDumpMemoryDescriptors();
    }

    // vibeos tripwire channel: read per-boot debug knobs (special pool arming)
    // from QEMU fw_cfg BEFORE system-PTE sizing and MiInitializeSpecialPool so
    // an armed MmSpecialPoolTag influences both. No-op without -fw_cfg.
    if phase == 0 {
        crate::tripwire::vibe_tripwire_init();
    }

    for i in 0..LoaderMaximum {
        include_type[i as usize] = 1;
    }
    include_type[LoaderBad as usize] = 0;
    include_type[LoaderFirmwarePermanent as usize] = 0;
    include_type[LoaderSpecialMemory as usize] = 0;
    include_type[LoaderBBTMemory as usize] = 0;

    if phase == 0 {
        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] MiScanMemoryDescriptors\r\n");
        MiScanMemoryDescriptors(loader_block);
        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] MiScanMemoryDescriptors done\r\n");

        KeInitializeEvent(
            core::ptr::addr_of_mut!(MiTempEvent),
            _EVENT_TYPE_NotificationEvent,
            0,
        );

        MiLowMemoryEvent = core::ptr::addr_of_mut!(MiTempEvent);
        MiHighMemoryEvent = core::ptr::addr_of_mut!(MiTempEvent);
        MiLowPagedPoolEvent = core::ptr::addr_of_mut!(MiTempEvent);
        MiHighPagedPoolEvent = core::ptr::addr_of_mut!(MiTempEvent);
        MiLowNonPagedPoolEvent = core::ptr::addr_of_mut!(MiTempEvent);
        MiHighNonPagedPoolEvent = core::ptr::addr_of_mut!(MiTempEvent);

        MmThrottleTop = 450;
        MmThrottleBottom = 127;

        MmSystemRangeStart = MI_DEFAULT_SYSTEM_RANGE_START as PVOID;
        MmUserProbeAddress = MI_USER_PROBE_ADDRESS as ULONG_PTR;
        MmHighestUserAddress = MI_HIGHEST_USER_ADDRESS as PVOID;

        MiHighestUserPte = MiAddressToPte(MmHighestUserAddress);
        MiHighestUserPde = MiAddressToPde(MmHighestUserAddress);
        #[cfg(target_pointer_width = "64")]
        {
            MiHighestUserPpe = MiAddressToPpe(MmHighestUserAddress);
            MiHighestUserPxe = MiAddressToPxe(MmHighestUserAddress);
        }

        MmBootImageSize = ((*(*loader_block).Extension).LoaderPagesSpanned) as usize;
        MmBootImageSize *= PAGE_SIZE;
        MmBootImageSize = (MmBootImageSize + PDE_MAPPED_VA - 1) & !(PDE_MAPPED_VA - 1);

        MiInitializeSessionSpaceLayout();

        /* SEC_BASED means EVERY mapping of the section lands on one system-wide
         * address, and MiFindEmptyAddressRangeDownBasedTree picks it by
         * searching downward from here. On a 64-bit kernel the user ceiling is
         * ~8 TB, so a based section lands far above 4 GB -- and csrss's shared
         * section is SEC_BASED. A WoW64 guest is then handed a base it cannot
         * hold: CSR_API_CONNECTINFO's SharedSectionBase narrows to (ULONG) in
         * wow64, 0x7fffebf0000 truncates to 0xfebf0000, and kernel32
         * dereferences that and takes 0xc0000005. (wow64 has been printing
         * "shared section ABOVE 4GB -- guest-invisible!" on every guest connect.)
         *
         * A based address must therefore be nameable by every client the section
         * can reach, 32-bit ones included, so cap the ceiling at the 32-bit user
         * ceiling. On i386 that is what this already computed, so the cap is a
         * no-op there and only bites on 64-bit. csrss is the only SEC_BASED
         * creator in the tree, so the address space this gives up is unused. */
        const MI_BASED_CEILING_WOW64: ULONG_PTR = 0x7F7E_FFFF;
        {
            let ceiling = (MmHighestUserAddress as ULONG_PTR) - 0x800000;
            MmHighSectionBase = (if ceiling > MI_BASED_CEILING_WOW64 {
                MI_BASED_CEILING_WOW64
            } else {
                ceiling
            }) as PVOID;
        }

        for i in 0..8 {
            MmStandbyPageListByPriority[i as usize].Total = 0;
            MmStandbyPageListByPriority[i as usize].ListName = (StandbyPageList) as u32;
            MmStandbyPageListByPriority[i as usize].Flink = MM_EMPTY_LIST;
            MmStandbyPageListByPriority[i as usize].Blink = MM_EMPTY_LIST;
        }

        InitializeListHead(core::ptr::addr_of_mut!(MmLoadedUserImageList) as PLIST_ENTRY);

        InitializeListHead(core::ptr::addr_of_mut!(MmWorkingSetExpansionHead) as PLIST_ENTRY);

        MmCriticalSectionTimeout.QuadPart = (MmCritsectTimeoutSeconds as i64) * (-10000000i64);

        KeInitializeGuardedMutex(core::ptr::addr_of_mut!(MmPagedPoolMutex) as *mut KGUARDED_MUTEX);
        KeInitializeGuardedMutex(core::ptr::addr_of_mut!(MmSectionCommitMutex) as *mut KGUARDED_MUTEX);
        KeInitializeGuardedMutex(core::ptr::addr_of_mut!(MmSectionBasedMutex) as *mut KGUARDED_MUTEX);

        KeInitializeMutant(
            core::ptr::addr_of_mut!(MmSystemLoadLock) as *mut _KMUTANT,
            0,
        );

        KeInitializeEvent(
            core::ptr::addr_of_mut!(MmZeroingPageEvent) as *mut _KEVENT,
            _EVENT_TYPE_NotificationEvent,
            0,
        );

        InitializeSListHead(core::ptr::addr_of_mut!(MmDeadStackSListHead) as *mut SLIST_HEADER);

        page_count = (MmNumberOfPhysicalPages) as usize;
        if page_count < MI_MIN_PAGES_FOR_SYSPTE_TUNING {
            MmNumberOfSystemPtes = 7000;
        } else {
            MmNumberOfSystemPtes = 11000;
            if page_count > MI_MIN_PAGES_FOR_SYSPTE_BOOST {
                MmNumberOfSystemPtes <<= 1;
            }
            if page_count > MI_MIN_PAGES_FOR_SYSPTE_BOOST_BOOST {
                MmNumberOfSystemPtes <<= 1;
            }
            if MmSpecialPoolTag != 0 && MmSpecialPoolTag != u32::MAX {
                MmNumberOfSystemPtes += 0x6000;
            }
        }

        if MmHeapSegmentReserve == 0 {
            MmHeapSegmentReserve = 2 * _1MB;
        }

        if MmHeapSegmentCommit == 0 {
            MmHeapSegmentCommit = 2 * PAGE_SIZE;
        }

        if MmHeapDeCommitTotalFreeThreshold == 0 {
            MmHeapDeCommitTotalFreeThreshold = 64 * _1KB;
        }

        if MmHeapDeCommitFreeBlockThreshold == 0 {
            MmHeapDeCommitFreeBlockThreshold = PAGE_SIZE;
        }

        ExInitializePushLock(core::ptr::addr_of_mut!(MmSystemCacheWs.WorkingSetMutex) as *mut _);

        MmTotalCommitLimit = (2 * _1GB) >> PAGE_SHIFT;
        MmTotalCommitLimitMaximum = MmTotalCommitLimit;

        if MmAllocationFragment == 0 {
            MmAllocationFragment = MI_ALLOCATION_FRAGMENT;
            if page_count < ((256 * _1MB) / PAGE_SIZE) {
                MmAllocationFragment = MI_ALLOCATION_FRAGMENT / 4;
            } else if page_count < (_1GB / PAGE_SIZE) {
                MmAllocationFragment = MI_ALLOCATION_FRAGMENT / 2;
            }
        } else {
            MmAllocationFragment *= _1KB;
            MmAllocationFragment = ROUND_TO_PAGES(MmAllocationFragment);

            if MmAllocationFragment > MI_MAX_ALLOCATION_FRAGMENT {
                MmAllocationFragment = MI_MAX_ALLOCATION_FRAGMENT;
            }

            if MmAllocationFragment < MI_MIN_ALLOCATION_FRAGMENT {
                MmAllocationFragment = MI_MIN_ALLOCATION_FRAGMENT;
            }
        }

        if MmLargeStackSize > ((KERNEL_LARGE_STACK_SIZE / _1KB)) as u32 {
            MmLargeStackSize = (KERNEL_LARGE_STACK_SIZE) as u32;
        } else {
            MmLargeStackSize *= (_1KB) as u32;

            MmLargeStackSize = (((MmLargeStackSize as usize) + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)) as u32;

            if MmLargeStackSize < (KERNEL_STACK_SIZE) as u32 {
                MmLargeStackSize = (KERNEL_STACK_SIZE) as u32;
            }
        }

        MiComputeColorInformation();

        MxPfnAllocation = (MmHighestPhysicalPage + 1) * core::mem::size_of::<_MMPFN>();
        MxPfnAllocation += MmSecondaryColors as usize * core::mem::size_of::<MMCOLOR_TABLES>() * 2;
        MxPfnAllocation >>= PAGE_SHIFT;

        MxPfnAllocation += 1;

        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] MiInitMachineDependent\r\n");
        MiInitMachineDependent(loader_block);

        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] post-MiInitMachineDependent\r\n");

        // DBG: before MmInitializeMemoryLimits
        MmPhysicalMemoryBlock = MmInitializeMemoryLimits(
            loader_block,
            include_type.as_mut_ptr(),
        );

        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] post-MmInitializeMemoryLimits\r\n");

        // DBG: before ExAllocatePoolWithTag (bitmap)
        bitmap = ExAllocatePoolWithTag(
            NonPagedPool,
            (((MmHighestPhysicalPage + 1) + 31) / 32) * 4,
            TAG_MM,
        );
        if bitmap.is_null() {
            KeBugCheckEx(
                INSTALL_MORE_MEMORY,
                (MmNumberOfPhysicalPages) as usize,
                MmLowestPhysicalPage,
                MmHighestPhysicalPage,
                0x101,
            );
        }

        // DBG: before RtlInitializeBitMap
        RtlInitializeBitMap(
            core::ptr::addr_of_mut!(MiPfnBitMap) as *mut RTL_BITMAP,
            bitmap as *mut ULONG,
            (MmHighestPhysicalPage + 1) as ULONG,
        );
        RtlClearAllBits(core::ptr::addr_of_mut!(MiPfnBitMap) as *mut RTL_BITMAP);

        // DBG: before memory runs loop
        for i in 0..(*MmPhysicalMemoryBlock).NumberOfRuns {
            run = (*MmPhysicalMemoryBlock).Run.as_mut_ptr().add(i as usize) as *mut _PHYSICAL_MEMORY_RUN;

            if (*run).PageCount != 0 {
                RtlSetBits(
                    core::ptr::addr_of_mut!(MiPfnBitMap) as *mut c_void,
                    (*run).BasePage as ULONG,
                    (*run).PageCount as ULONG,
                );
            }
        }

        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] post-bitmap\r\n");

        // DBG: before MiSyncCachedRanges
        MiSyncCachedRanges();

        // DBG: before MiAddHalIoMappings
        MiAddHalIoMappings();

        // DBG: after MiAddHalIoMappings

        MmResidentAvailablePages = (MmAvailablePages - 32) as isize;

        // DBG: after MmResidentAvailablePages assignment

        MiInitializeLargePageSupport();

        // DBG: after MiInitializeLargePageSupport

        MiInitializeDriverLargePageList();

        // DBG: after MiInitializeDriverLargePageList

        MiReloadBootLoadedDrivers(loader_block);

        // DBG: after MiReloadBootLoadedDrivers

        if MmNumberOfPhysicalPages <= (((13 * _1MB) / PAGE_SIZE)) as PFN_NUMBER {
            MmSystemSize = MmSmallSystem;
            MmMaximumDeadKernelStacks = 0;
        } else if MmNumberOfPhysicalPages <= (((19 * _1MB) / PAGE_SIZE)) as PFN_NUMBER {
            MmSystemSize = MmSmallSystem;
            MmSystemCacheWsMinimum += 100;
            MmMaximumDeadKernelStacks = 2;
        } else {
            MmSystemSize = MmMediumSystem;
            MmSystemCacheWsMinimum += 400;
            MmMaximumDeadKernelStacks = 5;
        }

        if MmNumberOfPhysicalPages < (((24 * _1MB) / PAGE_SIZE)) as PFN_NUMBER {
            MmSystemCacheWsMinimum = 32;
        }

        if MmNumberOfPhysicalPages >= (((32 * _1MB) / PAGE_SIZE)) as PFN_NUMBER {
            if MmProductType == 0x00690057 {
                MmSystemSize = MmLargeSystem;
            } else {
                if MmNumberOfPhysicalPages >= (((64 * _1MB) / PAGE_SIZE)) as PFN_NUMBER {
                    MmSystemSize = MmLargeSystem;
                }
            }
        }

        if MmNumberOfPhysicalPages > (((33 * _1MB) / PAGE_SIZE)) as PFN_NUMBER {
            MmSystemCacheWsMinimum += 500;
        }

        // DBG: before SharedUserData writes

        (*SharedUserData).NumberOfPhysicalPages = MmNumberOfPhysicalPages as ULONG;
        (*SharedUserData).LargePageMinimum = 0;

        if MmProductType == 0x00690057 {
            (*SharedUserData).NtProductType = NtProductWinNt;
            MmProductType = 0;

            MmThrottleTop = 250;
            MmThrottleBottom = 30;
        } else {
            if MmProductType == 0x00610020 {
                (*SharedUserData).NtProductType = NtProductLanManNt;
            } else {
                (*SharedUserData).NtProductType = NtProductServer;
            }

            MmProductType = 1;
            MmMinimumFreePages = 81;

            MmThrottleTop = 450;
            MmThrottleBottom = 80;
        }

        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] post-product\r\n");

        // DBG: before MiAdjustWorkingSetManagerParameters

        MiAdjustWorkingSetManagerParameters(if MmProductType != 0 { 0 } else { 1 });

        // DBG: after MiAdjustWorkingSetManagerParameters

        // Dump values before adjustment
        MmResidentAvailablePages -= (MiExpansionPoolPagesInitialCharge) as isize;
        MmResidentAvailablePages -= (MmSystemCacheWsMinimum) as isize;
        MmResidentAvailableAtInit = MmResidentAvailablePages;
        if MmResidentAvailablePages <= 0 {
            return 0;
        }

        #[cfg(target_arch = "x86_64")]
        {
            MmSizeOfSystemCacheInPages =
                ((MI_SYSTEM_CACHE_END + 1) - MI_SYSTEM_CACHE_START) / PAGE_SIZE;
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            MmSizeOfSystemCacheInPages =
                ((MI_PAGED_POOL_START as ULONG_PTR) - (MI_SYSTEM_CACHE_START as ULONG_PTR))
                    / PAGE_SIZE;
        }
        MmSystemCacheEnd = ((MmSystemCacheStart as ULONG_PTR)
            + (MmSizeOfSystemCacheInPages * PAGE_SIZE)
            - 1) as PVOID;

        MmTotalCommitLimit = MmAvailablePages;
        if MmTotalCommitLimit > 1024 {
            MmTotalCommitLimit -= 1024;
        }
        MmTotalCommitLimitMaximum = MmTotalCommitLimit;


        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] MiBuildPagedPool\r\n");
        MiBuildPagedPool();


        MmDebugPte = MiAddressToPte(MiDebugMapping as PVOID);


        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] MiInitializeLoadedModuleList\r\n");
        MiInitializeLoadedModuleList(loader_block);


        #[cfg(target_arch = "aarch64")]
        _A3_TRACE!(b"[MM] MiInitializeLoadedModuleList returned\r\n");
    }


    #[cfg(target_arch = "aarch64")]
    _A3_TRACE!(b"[MM] MmArmInitSystem about to return\r\n");
    #[cfg(target_arch = "aarch64")]
    _A3_TRACE!(b"[MM] MmArmInitSystem returning TRUE\r\n");
    1
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/mminit.c
//  * PURPOSE:         ARM Memory Manager Initialization
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* ARM64 UART diagnostic helper */
// #ifdef _M_ARM64
// static void _arm3_trace(const char *s) {
//     volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
//     while (*s) { *u = (ULONG)*s++; }
// }
// #define _A3_TRACE!(s) _arm3_trace(s)
// #else
// #define _A3_TRACE!(s) ((void)0)
// #endif
//
// #define MODULE_INVOLVED_IN_ARM3
// #include "miarm.h"
// #undef MmSystemRangeStart
//
// /* GLOBALS ********************************************************************/
//
// //
// // These are all registry-configurable, but by default, the memory manager will
// // figure out the most appropriate values.
// //
// ULONG MmMaximumNonPagedPoolPercent;
// SIZE_T MmSizeOfNonPagedPoolInBytes;
// SIZE_T MmMaximumNonPagedPoolInBytes;
//
// /* Some of the same values, in pages */
// PFN_NUMBER MmMaximumNonPagedPoolInPages;
//
// //
// // These numbers describe the discrete equation components of the nonpaged
// // pool sizing algorithm.
// //
// // They are described on http://support.microsoft.com/default.aspx/kb/126402/ja (DEAD_LINK)
// // along with the algorithm that uses them, which is implemented later below.
// //
// SIZE_T MmMinimumNonPagedPoolSize = 256 * 1024;
// ULONG MmMinAdditionNonPagedPoolPerMb = 32 * 1024;
// SIZE_T MmDefaultMaximumNonPagedPool = 1024 * 1024;
// ULONG MmMaxAdditionNonPagedPoolPerMb = 400 * 1024;
//
// //
// // The memory layout (and especially variable names) of the NT kernel mode
// // components can be a bit hard to twig, especially when it comes to the non
// // paged area.
// //
// // There are really two components to the non-paged pool:
// //
// // - The initial nonpaged pool, sized dynamically up to a maximum.
// // - The expansion nonpaged pool, sized dynamically up to a maximum.
// //
// // The initial nonpaged pool is physically continuous for performance, and
// // immediately follows the PFN database, typically sharing the same PDE. It is
// // a very small resource (32MB on a 1GB system), and capped at 128MB.
// //
// // Right now we call this the "ARM³ Nonpaged Pool" and it begins somewhere after
// // the PFN database (which starts at 0xB0000000).
// //
// // The expansion nonpaged pool, on the other hand, can grow much bigger (400MB
// // for a 1GB system). On ARM³ however, it is currently capped at 128MB.
// //
// // The address where the initial nonpaged pool starts is aptly named
// // MmNonPagedPoolStart, and it describes a range of MmSizeOfNonPagedPoolInBytes
// // bytes.
// //
// // Expansion nonpaged pool starts at an address described by the variable called
// // MmNonPagedPoolExpansionStart, and it goes on for MmMaximumNonPagedPoolInBytes
// // minus MmSizeOfNonPagedPoolInBytes bytes, always reaching MmNonPagedPoolEnd
// // (because of the way it's calculated) at 0xFFBE0000.
// //
// // Initial nonpaged pool is allocated and mapped early-on during boot, but what
// // about the expansion nonpaged pool? It is instead composed of special pages
// // which belong to what are called System PTEs. These PTEs are the matter of a
// // later discussion, but they are also considered part of the "nonpaged" OS, due
// // to the fact that they are never paged out -- once an address is described by
// // a System PTE, it is always valid, until the System PTE is torn down.
// //
// // System PTEs are actually composed of two "spaces", the system space proper,
// // and the nonpaged pool expansion space. The latter, as we've already seen,
// // begins at MmNonPagedPoolExpansionStart. Based on the number of System PTEs
// // that the system will support, the remaining address space below this address
// // is used to hold the system space PTEs. This address, in turn, is held in the
// // variable named MmNonPagedSystemStart, which itself is never allowed to go
// // below 0xEB000000 (thus creating an upper bound on the number of System PTEs).
// //
// // This means that 330MB are reserved for total nonpaged system VA, on top of
// // whatever the initial nonpaged pool allocation is.
// //
// // The following URLs, valid as of April 23rd, 2008, support this evidence:
// //
// // http://www.cs.miami.edu/~burt/journal/NT/memory.html
// // https://web.archive.org/web/20130412053421/http://www.ditii.com/2007/09/28/windows-memory-management-x86-virtual-address-space/
// //
// PVOID MmNonPagedSystemStart;
// PVOID MmNonPagedPoolStart;
// PVOID MmNonPagedPoolExpansionStart;
// PVOID MmNonPagedPoolEnd = MI_NONPAGED_POOL_END;
//
// //
// // This is where paged pool starts by default
// //
// PVOID MmPagedPoolStart = MI_PAGED_POOL_START;
// PVOID MmPagedPoolEnd;
//
// //
// // And this is its default size
// //
// SIZE_T MmSizeOfPagedPoolInBytes = MI_MIN_INIT_PAGED_POOLSIZE;
// PFN_NUMBER MmSizeOfPagedPoolInPages = MI_MIN_INIT_PAGED_POOLSIZE / PAGE_SIZE;
//
// //
// // Session space starts at 0xBFFFFFFF and grows downwards
// // By default, it includes an 8MB image area where we map win32k and video card
// // drivers, followed by a 4MB area containing the session's working set. This is
// // then followed by a 20MB mapped view area and finally by the session's paged
// // pool, by default 16MB.
// //
// // On a normal system, this results in session space occupying the region from
// // 0xBD000000 to 0xC0000000
// //
// // See miarm.h for the defines that determine the sizing of this region. On an
// // NT system, some of these can be configured through the registry, but we don't
// // support that yet.
// //
// PVOID MiSessionSpaceEnd;    // 0xC0000000
// PVOID MiSessionImageEnd;    // 0xC0000000
// PVOID MiSessionImageStart;  // 0xBF800000
// PVOID MiSessionSpaceWs;
// PVOID MiSessionViewStart;   // 0xBE000000
// PVOID MiSessionPoolEnd;     // 0xBE000000
// PVOID MiSessionPoolStart;   // 0xBD000000
// PVOID MmSessionBase;        // 0xBD000000
// SIZE_T MmSessionSize;
// SIZE_T MmSessionViewSize;
// SIZE_T MmSessionPoolSize;
// SIZE_T MmSessionImageSize;
//
// /*
//  * These are the PTE addresses of the boundaries carved out above
//  */
// PMMPTE MiSessionImagePteStart;
// PMMPTE MiSessionImagePteEnd;
// PMMPTE MiSessionBasePte;
// PMMPTE MiSessionLastPte;
//
// //
// // The system view space, on the other hand, is where sections that are memory
// // mapped into "system space" end up.
// //
// // By default, it is a 16MB region, but we hack it to be 32MB for ReactOS
// //
// PVOID MiSystemViewStart;
// SIZE_T MmSystemViewSize;
//
// #if (_MI_PAGING_LEVELS == 2)
// //
// // A copy of the system page directory (the page directory associated with the
// // System process) is kept (double-mapped) by the manager in order to lazily
// // map paged pool PDEs into external processes when they fault on a paged pool
// // address.
// //
// PFN_NUMBER MmSystemPageDirectory[PPE_PER_PAGE];
// PMMPDE MmSystemPagePtes;
// #endif
//
// //
// // The system cache starts right after hyperspace. The first few pages are for
// // keeping track of the system working set list.
// //
// // This should be 0xC0C00000 -- the cache itself starts at 0xC1000000
// //
// PMMWSL MmSystemCacheWorkingSetList = (PVOID)MI_SYSTEM_CACHE_WS_START;
//
// //
// // Windows NT seems to choose between 7000, 11000 and 50000
// // On systems with more than 32MB, this number is then doubled, and further
// // aligned up to a PDE boundary (4MB).
// //
// PFN_COUNT MmNumberOfSystemPtes;
//
// //
// // This is how many pages the PFN database will take up
// // In Windows, this includes the Quark Color Table, but not in ARM³
// //
// PFN_NUMBER MxPfnAllocation;
//
// //
// // Unlike the old ReactOS Memory Manager, ARM³ (and Windows) does not keep track
// // of pages that are not actually valid physical memory, such as ACPI reserved
// // regions, BIOS address ranges, or holes in physical memory address space which
// // could indicate device-mapped I/O memory.
// //
// // In fact, the lack of a PFN entry for a page usually indicates that this is
// // I/O space instead.
// //
// // A bitmap, called the PFN bitmap, keeps track of all page frames by assigning
// // a bit to each. If the bit is set, then the page is valid physical RAM.
// //
// RTL_BITMAP MiPfnBitMap;
//
// //
// // This structure describes the different pieces of RAM-backed address space
// //
// PPHYSICAL_MEMORY_DESCRIPTOR MmPhysicalMemoryBlock;
//
// //
// // This is where we keep track of the most basic physical layout markers
// //
// PFN_NUMBER MmHighestPhysicalPage, MmLowestPhysicalPage = -1;
// PFN_COUNT MmNumberOfPhysicalPages;
//
// //
// // The total number of pages mapped by the boot loader, which include the kernel
// // HAL, boot drivers, registry, NLS files and other loader data structures is
// // kept track of here. This depends on "LoaderPagesSpanned" being correct when
// // coming from the loader.
// //
// // This number is later aligned up to a PDE boundary.
// //
// SIZE_T MmBootImageSize;
//
// //
// // These three variables keep track of the core separation of address space that
// // exists between kernel mode and user mode.
// //
// ULONG_PTR MmUserProbeAddress;
// PVOID MmHighestUserAddress;
// PVOID MmSystemRangeStart;
//
// /* And these store the respective highest PTE/PDE address */
// PMMPTE MiHighestUserPte;
// PMMPDE MiHighestUserPde;
// #if (_MI_PAGING_LEVELS >= 3)
// PMMPTE MiHighestUserPpe;
// #if (_MI_PAGING_LEVELS >= 4)
// PMMPTE MiHighestUserPxe;
// #endif
// #endif
//
// /* These variables define the system cache address space */
// PVOID MmSystemCacheStart = (PVOID)MI_SYSTEM_CACHE_START;
// PVOID MmSystemCacheEnd;
// ULONG_PTR MmSizeOfSystemCacheInPages;
// MMSUPPORT MmSystemCacheWs;
//
// //
// // This is where hyperspace ends (followed by the system cache working set)
// //
// PVOID MmHyperSpaceEnd;
//
// //
// // Page coloring algorithm data
// //
// ULONG MmSecondaryColors;
// ULONG MmSecondaryColorMask;
//
// //
// // Actual (registry-configurable) size of a GUI thread's stack
// //
// ULONG MmLargeStackSize = KERNEL_LARGE_STACK_SIZE;
//
// //
// // Before we have a PFN database, memory comes straight from our physical memory
// // blocks, which is nice because it's guaranteed contiguous and also because once
// // we take a page from here, the system doesn't see it anymore.
// // However, once the fun is over, those pages must be re-integrated back into
// // PFN society life, and that requires us keeping a copy of the original layout
// // so that we can parse it later.
// //
// PMEMORY_ALLOCATION_DESCRIPTOR MxFreeDescriptor;
// MEMORY_ALLOCATION_DESCRIPTOR MxOldFreeDescriptor;
//
// /*
//  * For each page's worth bytes of L2 cache in a given set/way line, the zero and
//  * free lists are organized in what is called a "color".
//  *
//  * This array points to the two lists, so it can be thought of as a multi-dimensional
//  * array of MmFreePagesByColor[2][MmSecondaryColors]. Since the number is dynamic,
//  * we describe the array in pointer form instead.
//  *
//  * On a final note, the color tables themselves are right after the PFN database.
//  */
// C_ASSERT(FreePageList == 1);
// PMMCOLOR_TABLES MmFreePagesByColor[FreePageList + 1];
//
// /* An event used in Phase 0 before the rest of the system is ready to go */
// KEVENT MiTempEvent;
//
// /* All the events used for memory threshold notifications */
// PKEVENT MiLowMemoryEvent;
// PKEVENT MiHighMemoryEvent;
// PKEVENT MiLowPagedPoolEvent;
// PKEVENT MiHighPagedPoolEvent;
// PKEVENT MiLowNonPagedPoolEvent;
// PKEVENT MiHighNonPagedPoolEvent;
//
// /* The actual thresholds themselves, in page numbers */
// PFN_NUMBER MmLowMemoryThreshold;
// PFN_NUMBER MmHighMemoryThreshold;
// PFN_NUMBER MiLowPagedPoolThreshold;
// PFN_NUMBER MiHighPagedPoolThreshold;
// PFN_NUMBER MiLowNonPagedPoolThreshold;
// PFN_NUMBER MiHighNonPagedPoolThreshold;
//
// /*
//  * This number determines how many free pages must exist, at minimum, until we
//  * start trimming working sets and flushing modified pages to obtain more free
//  * pages.
//  *
//  * This number changes if the system detects that this is a server product
//  */
// PFN_NUMBER MmMinimumFreePages = 26;
//
// /*
//  * This number indicates how many pages we consider to be a low limit of having
//  * "plenty" of free memory.
//  *
//  * It is doubled on systems that have more than 63MB of memory
//  */
// PFN_NUMBER MmPlentyFreePages = 400;
//
// /* These values store the type of system this is (small, med, large) and if server */
// ULONG MmProductType;
// MM_SYSTEMSIZE MmSystemSize;
//
// /*
//  * These values store the cache working set minimums and maximums, in pages
//  *
//  * The minimum value is boosted on systems with more than 24MB of RAM, and cut
//  * down to only 32 pages on embedded (<24MB RAM) systems.
//  *
//  * An extra boost of 2MB is given on systems with more than 33MB of RAM.
//  */
// PFN_NUMBER MmSystemCacheWsMinimum = 288;
// PFN_NUMBER MmSystemCacheWsMaximum = 350;
//
// /* FIXME: Move to cache/working set code later */
// BOOLEAN MmLargeSystemCache;
//
// /*
//  * This value determines in how many fragments/chunks the subsection prototype
//  * PTEs should be allocated when mapping a section object. It is configurable in
//  * the registry through the MapAllocationFragment parameter.
//  *
//  * The default is 64KB on systems with more than 1GB of RAM, 32KB on systems with
//  * more than 256MB of RAM, and 16KB on systems with less than 256MB of RAM.
//  *
//  * The maximum it can be set to is 2MB, and the minimum is 4KB.
//  */
// SIZE_T MmAllocationFragment;
//
// /*
//  * These two values track how much virtual memory can be committed, and when
//  * expansion should happen.
//  */
//  // FIXME: They should be moved elsewhere since it's not an "init" setting?
// SIZE_T MmTotalCommitLimit;
// SIZE_T MmTotalCommitLimitMaximum;
//
// /*
//  * These values tune certain user parameters. They have default values set here,
//  * as well as in the code, and can be overwritten by registry settings.
//  */
// SIZE_T MmHeapSegmentReserve = 1 * _1MB;
// SIZE_T MmHeapSegmentCommit = 2 * PAGE_SIZE;
// SIZE_T MmHeapDeCommitTotalFreeThreshold = 64 * _1KB;
// SIZE_T MmHeapDeCommitFreeBlockThreshold = PAGE_SIZE;
// SIZE_T MmMinimumStackCommitInBytes = 0;
//
// /* Internal setting used for debugging memory descriptors */
// BOOLEAN MiDbgEnableMdDump =
// #ifdef _ARM_
// TRUE;
// #else
// FALSE;
// #endif
//
// /* Number of memory descriptors in the loader block */
// ULONG MiNumberDescriptors = 0;
//
// /* Number of free pages in the loader block */
// PFN_NUMBER MiNumberOfFreePages = 0;
//
// /* Timeout value for critical sections (2.5 minutes) */
// ULONG MmCritsectTimeoutSeconds = 150; // NT value: 720 * 60 * 60; (30 days)
// LARGE_INTEGER MmCriticalSectionTimeout;
//
// //
// // Throttling limits for Cc (in pages)
// // Above top, we don't throttle
// // Above bottom, we throttle depending on the amount of modified pages
// // Otherwise, we throttle!
// //
// ULONG MmThrottleTop;
// ULONG MmThrottleBottom;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// VOID
// NTAPI
// MiScanMemoryDescriptors(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY ListEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR Descriptor;
//     PFN_NUMBER PageFrameIndex, FreePages = 0;
//
//     /* Loop the memory descriptors */
//     for (ListEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//          ListEntry != &LoaderBlock->MemoryDescriptorListHead;
//          ListEntry = ListEntry->Flink)
//     {
//         /* Get the descriptor */
//         Descriptor = CONTAINING_RECORD(ListEntry,
//                                        MEMORY_ALLOCATION_DESCRIPTOR,
//                                        ListEntry);
//         DPRINT("MD Type: %lx Base: %lx Count: %lx\n",
//             Descriptor->MemoryType, Descriptor->BasePage, Descriptor->PageCount);
//
//         /* Count this descriptor */
//         MiNumberDescriptors++;
//
//         /* If this is invisible memory, skip this descriptor */
//         if (MiIsMemoryTypeInvisible(Descriptor->MemoryType))
//             continue;
//
//         /* Check if this isn't bad memory */
//         if (Descriptor->MemoryType != LoaderBad)
//         {
//             /* Count it in the physical pages */
//             MmNumberOfPhysicalPages += (PFN_COUNT)Descriptor->PageCount;
//         }
//
//         /* Check if this is the new lowest page */
//         if (Descriptor->BasePage < MmLowestPhysicalPage)
//         {
//             /* Update the lowest page */
//             MmLowestPhysicalPage = Descriptor->BasePage;
//         }
//
//         /* Check if this is the new highest page */
//         PageFrameIndex = Descriptor->BasePage + Descriptor->PageCount;
//         if (PageFrameIndex > MmHighestPhysicalPage)
//         {
//             /* Update the highest page */
//             MmHighestPhysicalPage = PageFrameIndex - 1;
//         }
//
//         /* Check if this is free memory */
//         if (MiIsMemoryTypeFree(Descriptor->MemoryType))
//         {
//             /* Count it in the free pages */
//             MiNumberOfFreePages += Descriptor->PageCount;
//
//             /* Check if this is the largest memory descriptor */
//             if (Descriptor->PageCount > FreePages)
//             {
//                 /* Remember it */
//                 MxFreeDescriptor = Descriptor;
//                 FreePages = Descriptor->PageCount;
//             }
//         }
//     }
//
//     /* Save original values of the free descriptor, since it'll be
//      * altered by early allocations */
//     MxOldFreeDescriptor = *MxFreeDescriptor;
// }
//
// CODE_SEG("INIT")
// PFN_NUMBER
// NTAPI
// MxGetNextPage(IN PFN_NUMBER PageCount)
// {
//     PFN_NUMBER Pfn;
//
//     /* Make sure we have enough pages */
//     if (PageCount > MxFreeDescriptor->PageCount)
//     {
//         /* Crash the system */
//         KeBugCheckEx(INSTALL_MORE_MEMORY,
//                      MmNumberOfPhysicalPages,
//                      MxFreeDescriptor->PageCount,
//                      MxOldFreeDescriptor.PageCount,
//                      PageCount);
//     }
//
//     /* Use our lowest usable free pages */
//     Pfn = MxFreeDescriptor->BasePage;
//     MxFreeDescriptor->BasePage += PageCount;
//     MxFreeDescriptor->PageCount -= PageCount;
//     return Pfn;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiComputeColorInformation(VOID)
// {
//     ULONG L2Associativity;
//
//     /* Check if no setting was provided already */
//     if (!MmSecondaryColors)
//     {
//         /* Get L2 cache information */
//         L2Associativity = KeGetPcr()->SecondLevelCacheAssociativity;
//
//         /* The number of colors is the number of cache bytes by set/way */
//         MmSecondaryColors = KeGetPcr()->SecondLevelCacheSize;
//         if (L2Associativity) MmSecondaryColors /= L2Associativity;
//     }
//
//     /* Now convert cache bytes into pages */
//     MmSecondaryColors >>= PAGE_SHIFT;
//     if (!MmSecondaryColors)
//     {
//         /* If there was no cache data from the KPCR, use the default colors */
//         MmSecondaryColors = MI_SECONDARY_COLORS;
//     }
//     else
//     {
//         /* Otherwise, make sure there aren't too many colors */
//         if (MmSecondaryColors > MI_MAX_SECONDARY_COLORS)
//         {
//             /* Set the maximum */
//             MmSecondaryColors = MI_MAX_SECONDARY_COLORS;
//         }
//
//         /* Make sure there aren't too little colors */
//         if (MmSecondaryColors < MI_MIN_SECONDARY_COLORS)
//         {
//             /* Set the default */
//             MmSecondaryColors = MI_SECONDARY_COLORS;
//         }
//
//         /* Finally make sure the colors are a power of two */
//         if (MmSecondaryColors & (MmSecondaryColors - 1))
//         {
//             /* Set the default */
//             MmSecondaryColors = MI_SECONDARY_COLORS;
//         }
//     }
//
//     /* Compute the mask and store it */
//     MmSecondaryColorMask = MmSecondaryColors - 1;
//     KeGetCurrentPrcb()->SecondaryColorMask = MmSecondaryColorMask;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiInitializeColorTables(VOID)
// {
//     ULONG i;
//     PMMPTE PointerPte, LastPte;
//     MMPTE TempPte = ValidKernelPte;
//
//     /* The color table starts after the ARM3 PFN database */
//     MmFreePagesByColor[0] = (PMMCOLOR_TABLES)&MmPfnDatabase[MmHighestPhysicalPage + 1];
//
//     /* Loop the PTEs. We have two color tables for each secondary color */
//     PointerPte = MiAddressToPte(&MmFreePagesByColor[0][0]);
//     LastPte = MiAddressToPte((ULONG_PTR)MmFreePagesByColor[0] +
//                              (2 * MmSecondaryColors * sizeof(MMCOLOR_TABLES))
//                              - 1);
//     while (PointerPte <= LastPte)
//     {
//         /* Check for valid PTE */
//         if (PointerPte->u.Hard.Valid == 0)
//         {
//             /* Get a page and map it */
//             TempPte.set_page_frame_number((MxGetNextPage(1)) as PFN_NUMBER);
//             MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//             /* Zero out the page */
//             RtlZeroMemory(MiPteToAddress(PointerPte), PAGE_SIZE);
//         }
//
//         /* Next */
//         PointerPte++;
//     }
//
//     /* Now set the address of the next list, right after this one */
//     MmFreePagesByColor[1] = &MmFreePagesByColor[0][MmSecondaryColors];
//
//     /* Now loop the lists to set them up */
//     for (i = 0; i < MmSecondaryColors; i++)
//     {
//         /* Set both free and zero lists for each color */
//         MmFreePagesByColor[ZeroedPageList][i].Flink = LIST_HEAD;
//         MmFreePagesByColor[ZeroedPageList][i].Blink = (PVOID)LIST_HEAD;
//         MmFreePagesByColor[ZeroedPageList][i].Count = 0;
//         MmFreePagesByColor[FreePageList][i].Flink = LIST_HEAD;
//         MmFreePagesByColor[FreePageList][i].Blink = (PVOID)LIST_HEAD;
//         MmFreePagesByColor[FreePageList][i].Count = 0;
//     }
// }
//
// #ifndef _M_AMD64
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// MiIsRegularMemory(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                   IN PFN_NUMBER Pfn)
// {
//     PLIST_ENTRY NextEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR MdBlock;
//
//     /* Loop the memory descriptors */
//     NextEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//     while (NextEntry != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         /* Get the memory descriptor */
//         MdBlock = CONTAINING_RECORD(NextEntry,
//                                     MEMORY_ALLOCATION_DESCRIPTOR,
//                                     ListEntry);
//
//         /* Check if this PFN could be part of the block */
//         if (Pfn >= (MdBlock->BasePage))
//         {
//             /* Check if it really is part of the block */
//             if (Pfn < (MdBlock->BasePage + MdBlock->PageCount))
//             {
//                 /* Check if the block is actually memory we don't map */
//                 if ((MdBlock->MemoryType == LoaderFirmwarePermanent) ||
//                     (MdBlock->MemoryType == LoaderBBTMemory) ||
//                     (MdBlock->MemoryType == LoaderSpecialMemory))
//                 {
//                     /* We don't need PFN database entries for this memory */
//                     break;
//                 }
//
//                 /* This is memory we want to map */
//                 return TRUE;
//             }
//         }
//         else
//         {
//             /* Blocks are ordered, so if it's not here, it doesn't exist */
//             break;
//         }
//
//         /* Get to the next descriptor */
//         NextEntry = MdBlock->ListEntry.Flink;
//     }
//
//     /* Check if this PFN is actually from our free memory descriptor */
//     if ((Pfn >= MxOldFreeDescriptor.BasePage) &&
//         (Pfn < MxOldFreeDescriptor.BasePage + MxOldFreeDescriptor.PageCount))
//     {
//         /* We use these pages for initial mappings, so we do want to count them */
//         return TRUE;
//     }
//
//     /* Otherwise this isn't memory that we describe or care about */
//     return FALSE;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiMapPfnDatabase(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PFN_NUMBER FreePage, FreePageCount, PagesLeft, BasePage, PageCount;
//     PLIST_ENTRY NextEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR MdBlock;
//     PMMPTE PointerPte, LastPte;
//     MMPTE TempPte = ValidKernelPte;
//
//     /* Get current page data, since we won't be using MxGetNextPage as it would corrupt our state */
//     FreePage = MxFreeDescriptor->BasePage;
//     FreePageCount = MxFreeDescriptor->PageCount;
//     PagesLeft = 0;
//
//     /* Loop the memory descriptors */
//     NextEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//     while (NextEntry != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         /* Get the descriptor */
//         MdBlock = CONTAINING_RECORD(NextEntry,
//                                     MEMORY_ALLOCATION_DESCRIPTOR,
//                                     ListEntry);
//         if ((MdBlock->MemoryType == LoaderFirmwarePermanent) ||
//             (MdBlock->MemoryType == LoaderBBTMemory) ||
//             (MdBlock->MemoryType == LoaderSpecialMemory))
//         {
//             /* These pages are not part of the PFN database */
//             NextEntry = MdBlock->ListEntry.Flink;
//             continue;
//         }
//
//         /* Next, check if this is our special free descriptor we've found */
//         if (MdBlock == MxFreeDescriptor)
//         {
//             /* Use the real numbers instead */
//             BasePage = MxOldFreeDescriptor.BasePage;
//             PageCount = MxOldFreeDescriptor.PageCount;
//         }
//         else
//         {
//             /* Use the descriptor's numbers */
//             BasePage = MdBlock->BasePage;
//             PageCount = MdBlock->PageCount;
//         }
//
//         /* Get the PTEs for this range */
//         PointerPte = MiAddressToPte(&MmPfnDatabase[BasePage]);
//         LastPte = MiAddressToPte(((ULONG_PTR)&MmPfnDatabase[BasePage + PageCount]) - 1);
//         DPRINT("MD Type: %lx Base: %lx Count: %lx\n", MdBlock->MemoryType, BasePage, PageCount);
//
//         /* Loop them */
//         while (PointerPte <= LastPte)
//         {
//             /* We'll only touch PTEs that aren't already valid */
//             if (PointerPte->u.Hard.Valid == 0)
//             {
//                 /* Use the next free page */
//                 TempPte.set_page_frame_number((FreePage) as PFN_NUMBER);
//                 ASSERT(FreePageCount != 0);
//
//                 /* Consume free pages */
//                 FreePage++;
//                 FreePageCount--;
//                 if (!FreePageCount)
//                 {
//                     /* Out of memory */
//                     KeBugCheckEx(INSTALL_MORE_MEMORY,
//                                  MmNumberOfPhysicalPages,
//                                  FreePageCount,
//                                  MxOldFreeDescriptor.PageCount,
//                                  1);
//                 }
//
//                 /* Write out this PTE */
//                 PagesLeft++;
//                 MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//                 /* Zero this page */
//                 RtlZeroMemory(MiPteToAddress(PointerPte), PAGE_SIZE);
//             }
//
//             /* Next! */
//             PointerPte++;
//         }
//
//         /* Do the next address range */
//         NextEntry = MdBlock->ListEntry.Flink;
//     }
//
//     /* Now update the free descriptors to consume the pages we used up during the PFN allocation loop */
//     MxFreeDescriptor->BasePage = FreePage;
//     MxFreeDescriptor->PageCount = FreePageCount;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiBuildPfnDatabaseFromPages(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PMMPDE PointerPde;
//     PMMPTE PointerPte;
//     ULONG i, Count, j;
//     PFN_NUMBER PageFrameIndex, StartupPdIndex, PtePageIndex;
//     PMMPFN Pfn1, Pfn2;
//     ULONG_PTR BaseAddress = 0;
//
//     /* PFN of the startup page directory */
//     StartupPdIndex = PFN_FROM_PTE(MiAddressToPde(PDE_BASE));
//
//     /* Start with the first PDE and scan them all */
//     PointerPde = MiAddressToPde(NULL);
//     Count = PPE_PER_PAGE * PDE_PER_PAGE;
//     for (i = 0; i < Count; i++)
//     {
//         /* Check for valid PDE */
//         if (PointerPde->u.Hard.Valid == 1)
//         {
//             /* Get the PFN from it */
//             PageFrameIndex = PFN_FROM_PTE(PointerPde);
//
//             /* Do we want a PFN entry for this page? */
//             if (MiIsRegularMemory(LoaderBlock, PageFrameIndex))
//             {
//                 /* Yes we do, set it up */
//                 Pfn1 = MiGetPfnEntry(PageFrameIndex);
//                 Pfn1->u4.PteFrame = StartupPdIndex;
//                 Pfn1->PteAddress = (PMMPTE)PointerPde;
//                 Pfn1->u2.ShareCount++;
//                 Pfn1->u3.e2.ReferenceCount = 1;
//                 Pfn1->u3.e1.PageLocation = ActiveAndValid;
//                 Pfn1->u3.e1.CacheAttribute = MiNonCached;
// #if MI_TRACE_PFNS
//                 Pfn1->PfnUsage = MI_USAGE_INIT_MEMORY;
//                 MI_SET_PFN_PROCESS_NAME(Pfn1, "Initial PDE");
// #endif
//             }
//             else
//             {
//                 /* No PFN entry */
//                 Pfn1 = NULL;
//             }
//
//             /* Now get the PTE and scan the pages */
//             PointerPte = MiAddressToPte(BaseAddress);
//             for (j = 0; j < PTE_PER_PAGE; j++)
//             {
//                 /* Check for a valid PTE */
//                 if (PointerPte->u.Hard.Valid == 1)
//                 {
//                     /* Increase the shared count of the PFN entry for the PDE */
//                     ASSERT(Pfn1 != NULL);
//                     Pfn1->u2.ShareCount++;
//
//                     /* Now check if the PTE is valid memory too */
//                     PtePageIndex = PFN_FROM_PTE(PointerPte);
//                     if (MiIsRegularMemory(LoaderBlock, PtePageIndex))
//                     {
//                         /*
//                          * Only add pages above the end of system code or pages
//                          * that are part of nonpaged pool
//                          */
//                         if ((BaseAddress >= 0xA0000000) ||
//                             ((BaseAddress >= (ULONG_PTR)MmNonPagedPoolStart) &&
//                              (BaseAddress < (ULONG_PTR)MmNonPagedPoolStart +
//                                             MmSizeOfNonPagedPoolInBytes)))
//                         {
//                             /* Get the PFN entry and make sure it too is valid */
//                             Pfn2 = MiGetPfnEntry(PtePageIndex);
//                             if ((MmIsAddressValid(Pfn2)) &&
//                                 (MmIsAddressValid(Pfn2 + 1)))
//                             {
//                                 /* Setup the PFN entry */
//                                 Pfn2->u4.PteFrame = PageFrameIndex;
//                                 Pfn2->PteAddress = PointerPte;
//                                 Pfn2->u2.ShareCount++;
//                                 Pfn2->u3.e2.ReferenceCount = 1;
//                                 Pfn2->u3.e1.PageLocation = ActiveAndValid;
//                                 Pfn2->u3.e1.CacheAttribute = MiNonCached;
// #if MI_TRACE_PFNS
//                                 Pfn2->PfnUsage = MI_USAGE_INIT_MEMORY;
//                                 MI_SET_PFN_PROCESS_NAME(Pfn2, "Initial PTE");
// #endif
//                             }
//                         }
//                     }
//                 }
//
//                 /* Next PTE */
//                 PointerPte++;
//                 BaseAddress += PAGE_SIZE;
//             }
//         }
//         else
//         {
//             /* Next PDE mapped address */
//             BaseAddress += PDE_MAPPED_VA;
//         }
//
//         /* Next PTE */
//         PointerPde++;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiBuildPfnDatabaseZeroPage(VOID)
// {
//     PMMPFN Pfn1;
//     PMMPDE PointerPde;
//
//     /* Grab the lowest page and check if it has no real references */
//     Pfn1 = MiGetPfnEntry(MmLowestPhysicalPage);
//     if (!(MmLowestPhysicalPage) && !(Pfn1->u3.e2.ReferenceCount))
//     {
//         /* Make it a bogus page to catch errors */
//         PointerPde = MiAddressToPde(0xFFFFFFFF);
//         Pfn1->u4.PteFrame = PFN_FROM_PTE(PointerPde);
//         Pfn1->PteAddress = (PMMPTE)PointerPde;
//         Pfn1->u2.ShareCount++;
//         Pfn1->u3.e2.ReferenceCount = 0xFFF0;
//         Pfn1->u3.e1.PageLocation = ActiveAndValid;
//         Pfn1->u3.e1.CacheAttribute = MiNonCached;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiBuildPfnDatabaseFromLoaderBlock(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextEntry;
//     PFN_NUMBER PageCount = 0;
//     PMEMORY_ALLOCATION_DESCRIPTOR MdBlock;
//     PFN_NUMBER PageFrameIndex;
//     PMMPFN Pfn1;
//     PMMPTE PointerPte;
//     PMMPDE PointerPde;
//     KIRQL OldIrql;
//
//     /* Now loop through the descriptors */
//     NextEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//     while (NextEntry != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         /* Get the current descriptor */
//         MdBlock = CONTAINING_RECORD(NextEntry,
//                                     MEMORY_ALLOCATION_DESCRIPTOR,
//                                     ListEntry);
//
//         /* Read its data */
//         PageCount = MdBlock->PageCount;
//         PageFrameIndex = MdBlock->BasePage;
//
//         /* Don't allow memory above what the PFN database is mapping */
//         if (PageFrameIndex > MmHighestPhysicalPage)
//         {
//             /* Since they are ordered, everything past here will be larger */
//             break;
//         }
//
//         /* On the other hand, the end page might be higher up... */
//         if ((PageFrameIndex + PageCount) > (MmHighestPhysicalPage + 1))
//         {
//             /* In which case we'll trim the descriptor to go as high as we can */
//             PageCount = MmHighestPhysicalPage + 1 - PageFrameIndex;
//             MdBlock->PageCount = PageCount;
//
//             /* But if there's nothing left to trim, we got too high, so quit */
//             if (!PageCount) break;
//         }
//
//         /* Now check the descriptor type */
//         switch (MdBlock->MemoryType)
//         {
//             /* Check for bad RAM */
//             case LoaderBad:
//
//                 DPRINT1("You either have specified /BURNMEMORY or damaged RAM modules.\n");
//                 break;
//
//             /* Check for free RAM */
//             case LoaderFree:
//             case LoaderLoadedProgram:
//             case LoaderFirmwareTemporary:
//             case LoaderOsloaderStack:
//
//                 /* Get the last page of this descriptor. Note we loop backwards */
//                 PageFrameIndex += PageCount - 1;
//                 Pfn1 = MiGetPfnEntry(PageFrameIndex);
//
//                 /* Lock the PFN Database */
//                 OldIrql = MiAcquirePfnLock();
//                 while (PageCount--)
//                 {
//                     /* If the page really has no references, mark it as free */
//                     if (!Pfn1->u3.e2.ReferenceCount)
//                     {
//                         /* Add it to the free list */
//                         Pfn1->u3.e1.CacheAttribute = MiNonCached;
//                         MiInsertPageInFreeList(PageFrameIndex);
//                     }
//
//                     /* Go to the next page */
//                     Pfn1--;
//                     PageFrameIndex--;
//                 }
//
//                 /* Release PFN database */
//                 MiReleasePfnLock(OldIrql);
//
//                 /* Done with this block */
//                 break;
//
//             /* Check for pages that are invisible to us */
//             case LoaderFirmwarePermanent:
//             case LoaderSpecialMemory:
//             case LoaderBBTMemory:
//
//                 /* And skip them */
//                 break;
//
//             default:
//
//                 /* Map these pages with the KSEG0 mapping that adds 0x80000000 */
//                 PointerPte = MiAddressToPte(KSEG0_BASE + (PageFrameIndex << PAGE_SHIFT));
//                 Pfn1 = MiGetPfnEntry(PageFrameIndex);
//                 while (PageCount--)
//                 {
//                     /* Check if the page is really unused */
//                     PointerPde = MiAddressToPde(KSEG0_BASE + (PageFrameIndex << PAGE_SHIFT));
//                     if (!Pfn1->u3.e2.ReferenceCount)
//                     {
//                         /* Mark it as being in-use */
//                         Pfn1->u4.PteFrame = PFN_FROM_PTE(PointerPde);
//                         Pfn1->PteAddress = PointerPte;
//                         Pfn1->u2.ShareCount++;
//                         Pfn1->u3.e2.ReferenceCount = 1;
//                         Pfn1->u3.e1.PageLocation = ActiveAndValid;
//                         Pfn1->u3.e1.CacheAttribute = MiNonCached;
// #if MI_TRACE_PFNS
//                         Pfn1->PfnUsage = MI_USAGE_BOOT_DRIVER;
// #endif
//
//                         /* Check for RAM disk page */
//                         if (MdBlock->MemoryType == LoaderXIPRom)
//                         {
//                             /* Make it a pseudo-I/O ROM mapping */
//                             Pfn1->u1.Flink = 0;
//                             Pfn1->u2.ShareCount = 0;
//                             Pfn1->u3.e2.ReferenceCount = 0;
//                             Pfn1->u3.e1.PageLocation = 0;
//                             Pfn1->u3.e1.Rom = 1;
//                             Pfn1->u4.InPageError = 0;
//                             Pfn1->u3.e1.PrototypePte = 1;
//                         }
//                     }
//
//                     /* Advance page structures */
//                     Pfn1++;
//                     PageFrameIndex++;
//                     PointerPte++;
//                 }
//                 break;
//         }
//
//         /* Next descriptor entry */
//         NextEntry = MdBlock->ListEntry.Flink;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiBuildPfnDatabaseSelf(VOID)
// {
//     PMMPTE PointerPte, LastPte;
//     PMMPFN Pfn1;
//     /* Loop the PFN database page */
//     PointerPte = MiAddressToPte(MiGetPfnEntry(MmLowestPhysicalPage));
//     LastPte = MiAddressToPte(MiGetPfnEntry(MmHighestPhysicalPage));
//     while (PointerPte <= LastPte)
//     {
//         /* Make sure the page is valid */
//         if (PointerPte->u.Hard.Valid == 1)
//         {
//             /* Get the PFN entry and just mark it referenced */
//             Pfn1 = MiGetPfnEntry(PointerPte->u.Hard.PageFrameNumber);
//             Pfn1->u2.ShareCount = 1;
//             Pfn1->u3.e2.ReferenceCount = 1;
// #if MI_TRACE_PFNS
//             Pfn1->PfnUsage = MI_USAGE_PFN_DATABASE;
// #endif
//         }
//
//         /* Next */
//         PointerPte++;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiInitializePfnDatabase(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     /* Scan memory and start setting up PFN entries.
//      * MiBuildPfnDatabaseFromPages does a linear PDE scan that only works on
//      * i386 (single-level PDE). ARM64 uses MiBuildPfnDatabaseFromPageTables
//      * (hierarchical PXE→PPE→PDE→PTE walk) instead, called separately. */
// #ifndef _M_ARM64
//     MiBuildPfnDatabaseFromPages(LoaderBlock);
// #endif
//
//     /* Add the zero page */
//     MiBuildPfnDatabaseZeroPage();
//
//     /* Scan the loader block and build the rest of the PFN database */
//     MiBuildPfnDatabaseFromLoaderBlock(LoaderBlock);
//
//     /* Finally add the pages for the PFN database itself */
//     MiBuildPfnDatabaseSelf();
// }
// #endif /* !_M_AMD64 */
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MmFreeLoaderBlock(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PLIST_ENTRY NextMd;
//     PMEMORY_ALLOCATION_DESCRIPTOR MdBlock;
//     ULONG_PTR i;
//     PFN_NUMBER BasePage, LoaderPages;
//     PMMPFN Pfn1;
//     KIRQL OldIrql;
//     PPHYSICAL_MEMORY_RUN Buffer, Entry;
//
//     /* Loop the descriptors in order to count them */
//     i = 0;
//     NextMd = LoaderBlock->MemoryDescriptorListHead.Flink;
//     while (NextMd != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         MdBlock = CONTAINING_RECORD(NextMd,
//                                     MEMORY_ALLOCATION_DESCRIPTOR,
//                                     ListEntry);
//         i++;
//         NextMd = MdBlock->ListEntry.Flink;
//     }
//
//     /* Allocate a structure to hold the physical runs */
//     Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                    i * sizeof(PHYSICAL_MEMORY_RUN),
//                                    'lMmM');
//     ASSERT(Buffer != NULL);
//     Entry = Buffer;
//
//     /* Loop the descriptors again */
//     NextMd = LoaderBlock->MemoryDescriptorListHead.Flink;
//     while (NextMd != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         /* Check what kind this was */
//         MdBlock = CONTAINING_RECORD(NextMd,
//                                     MEMORY_ALLOCATION_DESCRIPTOR,
//                                     ListEntry);
//         switch (MdBlock->MemoryType)
//         {
//             /* Registry, NLS, and heap data */
//             case LoaderRegistryData:
//             case LoaderOsloaderHeap:
//             case LoaderNlsData:
//                 /* Are all a candidate for deletion */
//                 Entry->BasePage = MdBlock->BasePage;
//                 Entry->PageCount = MdBlock->PageCount;
//                 Entry++;
//
//             /* We keep the rest */
//             default:
//                 break;
//         }
//
//         /* Move to the next descriptor */
//         NextMd = MdBlock->ListEntry.Flink;
//     }
//
//     /* Acquire the PFN lock */
//     OldIrql = MiAcquirePfnLock();
//
//     /* Loop the runs */
//     LoaderPages = 0;
//     while (--Entry >= Buffer)
//     {
//         /* See how many pages are in this run */
//         i = Entry->PageCount;
//         BasePage = Entry->BasePage;
//
//         /* Loop each page */
//         Pfn1 = MiGetPfnEntry(BasePage);
//         while (i--)
//         {
//             /* Check if it has references or is in any kind of list */
//             if (!(Pfn1->u3.e2.ReferenceCount) && (!Pfn1->u1.Flink))
//             {
//                 /* Set the new PTE address and put this page into the free list */
//                 Pfn1->PteAddress = (PMMPTE)(BasePage << PAGE_SHIFT);
//                 MiInsertPageInFreeList(BasePage);
//                 LoaderPages++;
//             }
//             else if (BasePage)
//             {
//                 /* It has a reference, so simply drop it */
//                 ASSERT(MI_IS_PHYSICAL_ADDRESS(MiPteToAddress(Pfn1->PteAddress)) == FALSE);
//
//                 /* Drop a dereference on this page, which should delete it */
//                 Pfn1->PteAddress->u.Long = 0;
//                 MI_SET_PFN_DELETED(Pfn1);
//                 MiDecrementShareCount(Pfn1 as *mut _MMPFN, BasePage);
//                 LoaderPages++;
//             }
//
//             /* Move to the next page */
//             Pfn1++;
//             BasePage++;
//         }
//     }
//
//     /* Release the PFN lock and flush the TLB */
//     DPRINT("Loader pages freed: %lx\n", LoaderPages);
//     MiReleasePfnLock(OldIrql);
//     KeFlushCurrentTb();
//
//     /* Free our run structure */
//     ExFreePoolWithTag(Buffer, 'lMmM');
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiAdjustWorkingSetManagerParameters(IN BOOLEAN Client)
// {
//     /* This function needs to do more work, for now, we tune page minimums */
//
//     /* Check for a system with around 64MB RAM or more */
//     if (MmNumberOfPhysicalPages >= (63 * _1MB) / PAGE_SIZE)
//     {
//         /* Double the minimum amount of pages we consider for a "plenty free" scenario */
//         MmPlentyFreePages *= 2;
//     }
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiNotifyMemoryEvents(VOID)
// {
//     /* Are we in a low-memory situation? */
//     if (MmAvailablePages < MmLowMemoryThreshold)
//     {
//         /* Clear high, set low  */
//         if (KeReadStateEvent(MiHighMemoryEvent)) KeClearEvent(MiHighMemoryEvent);
//         if (!KeReadStateEvent(MiLowMemoryEvent)) KeSetEvent(MiLowMemoryEvent, 0, FALSE);
//     }
//     else if (MmAvailablePages < MmHighMemoryThreshold)
//     {
//         /* We are in between, clear both */
//         if (KeReadStateEvent(MiHighMemoryEvent)) KeClearEvent(MiHighMemoryEvent);
//         if (KeReadStateEvent(MiLowMemoryEvent)) KeClearEvent(MiLowMemoryEvent);
//     }
//     else
//     {
//         /* Clear low, set high  */
//         if (KeReadStateEvent(MiLowMemoryEvent)) KeClearEvent(MiLowMemoryEvent);
//         if (!KeReadStateEvent(MiHighMemoryEvent)) KeSetEvent(MiHighMemoryEvent, 0, FALSE);
//     }
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// MiCreateMemoryEvent(IN PUNICODE_STRING Name,
//                     OUT PKEVENT *Event)
// {
//     PACL Dacl;
//     HANDLE EventHandle;
//     ULONG DaclLength;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     SECURITY_DESCRIPTOR SecurityDescriptor;
//
//     /* Create the SD */
//     Status = RtlCreateSecurityDescriptor(&SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* One ACL with 3 ACEs, containing each one SID */
//     DaclLength = sizeof(ACL) +
//                  3 * sizeof(ACCESS_ALLOWED_ACE) +
//                  RtlLengthSid(SeLocalSystemSid) +
//                  RtlLengthSid(SeAliasAdminsSid) +
//                  RtlLengthSid(SeWorldSid);
//
//     /* Allocate space for the DACL */
//     Dacl = ExAllocatePoolWithTag(PagedPool, DaclLength, TAG_DACL);
//     if (!Dacl) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Setup the ACL inside it */
//     Status = RtlCreateAcl(Dacl, DaclLength, ACL_REVISION);
//     if (!NT_SUCCESS(Status)) goto CleanUp;
//
//     /* Add query rights for everyone */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     SYNCHRONIZE | EVENT_QUERY_STATE | READ_CONTROL,
//                                     SeWorldSid);
//     if (!NT_SUCCESS(Status)) goto CleanUp;
//
//     /* Full rights for the admin */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     EVENT_ALL_ACCESS,
//                                     SeAliasAdminsSid);
//     if (!NT_SUCCESS(Status)) goto CleanUp;
//
//     /* As well as full rights for the system */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     EVENT_ALL_ACCESS,
//                                     SeLocalSystemSid);
//     if (!NT_SUCCESS(Status)) goto CleanUp;
//
//     /* Set this DACL inside the SD */
//     Status = RtlSetDaclSecurityDescriptor(&SecurityDescriptor,
//                                           TRUE,
//                                           Dacl,
//                                           FALSE);
//     if (!NT_SUCCESS(Status)) goto CleanUp;
//
//     /* Setup the event attributes, making sure it's a permanent one */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                Name,
//                                OBJ_KERNEL_HANDLE | OBJ_PERMANENT,
//                                NULL,
//                                &SecurityDescriptor);
//
//     /* Create the event */
//     Status = ZwCreateEvent(&EventHandle,
//                            EVENT_ALL_ACCESS,
//                            &ObjectAttributes,
//                            NotificationEvent,
//                            FALSE);
// CleanUp:
//     /* Free the DACL */
//     ExFreePoolWithTag(Dacl, TAG_DACL);
//
//     /* Check if this is the success path */
//     if (NT_SUCCESS(Status))
//     {
//         /* Add a reference to the object, then close the handle we had */
//         Status = ObReferenceObjectByHandle(EventHandle,
//                                            EVENT_MODIFY_STATE,
//                                            ExEventObjectType,
//                                            KernelMode,
//                                            (PVOID*)Event,
//                                            NULL);
//         ZwClose (EventHandle);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// MiInitializeMemoryEvents(VOID)
// {
//     UNICODE_STRING LowString = RTL_CONSTANT_STRING(L"\\KernelObjects\\LowMemoryCondition");
//     UNICODE_STRING HighString = RTL_CONSTANT_STRING(L"\\KernelObjects\\HighMemoryCondition");
//     UNICODE_STRING LowPagedPoolString = RTL_CONSTANT_STRING(L"\\KernelObjects\\LowPagedPoolCondition");
//     UNICODE_STRING HighPagedPoolString = RTL_CONSTANT_STRING(L"\\KernelObjects\\HighPagedPoolCondition");
//     UNICODE_STRING LowNonPagedPoolString = RTL_CONSTANT_STRING(L"\\KernelObjects\\LowNonPagedPoolCondition");
//     UNICODE_STRING HighNonPagedPoolString = RTL_CONSTANT_STRING(L"\\KernelObjects\\HighNonPagedPoolCondition");
//     NTSTATUS Status;
//
//     /* Check if we have a registry setting */
//     if (MmLowMemoryThreshold)
//     {
//         /* Convert it to pages */
//         MmLowMemoryThreshold *= (_1MB / PAGE_SIZE);
//     }
//     else
//     {
//         /* The low memory threshold is hit when we don't consider that we have "plenty" of free pages anymore */
//         MmLowMemoryThreshold = MmPlentyFreePages;
//
//         /* More than one GB of memory? */
//         if (MmNumberOfPhysicalPages > 0x40000)
//         {
//             /* Start at 32MB, and add another 16MB for each GB */
//             MmLowMemoryThreshold = (32 * _1MB) / PAGE_SIZE;
//             MmLowMemoryThreshold += ((MmNumberOfPhysicalPages - 0x40000) >> 7);
//         }
//         else if (MmNumberOfPhysicalPages > 0x8000)
//         {
//             /* For systems with > 128MB RAM, add another 4MB for each 128MB */
//             MmLowMemoryThreshold += ((MmNumberOfPhysicalPages - 0x8000) >> 5);
//         }
//
//         /* Don't let the minimum threshold go past 64MB */
//         MmLowMemoryThreshold = min(MmLowMemoryThreshold, (64 * _1MB) / PAGE_SIZE);
//     }
//
//     /* Check if we have a registry setting */
//     if (MmHighMemoryThreshold)
//     {
//         /* Convert it into pages */
//         MmHighMemoryThreshold *= (_1MB / PAGE_SIZE);
//     }
//     else
//     {
//         /* Otherwise, the default is three times the low memory threshold */
//         MmHighMemoryThreshold = 3 * MmLowMemoryThreshold;
//         ASSERT(MmHighMemoryThreshold > MmLowMemoryThreshold);
//     }
//
//     /* Make sure high threshold is actually higher than the low */
//     MmHighMemoryThreshold = max(MmHighMemoryThreshold, MmLowMemoryThreshold);
//
//     /* Create the memory events for all the thresholds */
//     _A3_TRACE!("[MM1] MiCreateMemoryEvent LowMem\r\n");
//     Status = MiCreateMemoryEvent(&LowString, &MiLowMemoryEvent);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     _A3_TRACE!("[MM1] MiCreateMemoryEvent HighMem\r\n");
//     Status = MiCreateMemoryEvent(&HighString, &MiHighMemoryEvent);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     _A3_TRACE!("[MM1] MiCreateMemoryEvent LowPagedPool\r\n");
//     Status = MiCreateMemoryEvent(&LowPagedPoolString, &MiLowPagedPoolEvent);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     _A3_TRACE!("[MM1] MiCreateMemoryEvent HighPagedPool\r\n");
//     Status = MiCreateMemoryEvent(&HighPagedPoolString, &MiHighPagedPoolEvent);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     _A3_TRACE!("[MM1] MiCreateMemoryEvent LowNonPagedPool\r\n");
//     Status = MiCreateMemoryEvent(&LowNonPagedPoolString, &MiLowNonPagedPoolEvent);
//     if (!NT_SUCCESS(Status)) return FALSE;
//     _A3_TRACE!("[MM1] MiCreateMemoryEvent HighNonPagedPool\r\n");
//     Status = MiCreateMemoryEvent(&HighNonPagedPoolString, &MiHighNonPagedPoolEvent);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Now setup the pool events */
//     _A3_TRACE!("[MM1] MiInitializePoolEvents\r\n");
//     MiInitializePoolEvents();
//
//     /* Set the initial event state */
//     _A3_TRACE!("[MM1] MiNotifyMemoryEvents\r\n");
//     MiNotifyMemoryEvents();
//     _A3_TRACE!("[MM1] MiInitializeMemoryEvents done\r\n");
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiAddHalIoMappings(VOID)
// {
//     PVOID BaseAddress;
//     PMMPDE PointerPde, LastPde;
//     PMMPTE PointerPte;
//     ULONG j;
//     PFN_NUMBER PageFrameIndex;
//
//     /* HAL Heap address -- should be on a PDE boundary */
//     BaseAddress = (PVOID)MM_HAL_VA_START;
//     ASSERT(MiAddressToPteOffset(BaseAddress) == 0);
//
//     /* Check how many PDEs the heap has */
//     PointerPde = MiAddressToPde(BaseAddress);
//     LastPde = MiAddressToPde((PVOID)MM_HAL_VA_END);
//
//     while (PointerPde <= LastPde)
//     {
//         /* Does the HAL own this mapping? */
//         if ((PointerPde->u.Hard.Valid == 1) &&
//             (MI_IS_PAGE_LARGE(PointerPde) == FALSE))
//         {
//             /* Get the PTE for it and scan each page */
//             PointerPte = MiAddressToPte(BaseAddress);
//             for (j = 0; j < PTE_PER_PAGE; j++)
//             {
//                 /* Does the HAL own this page? */
//                 if (PointerPte->u.Hard.Valid == 1)
//                 {
//                     /* Is the HAL using it for device or I/O mapped memory? */
//                     PageFrameIndex = PFN_FROM_PTE(PointerPte);
//                     if (!MiGetPfnEntry(PageFrameIndex))
//                     {
//                         /* FIXME: For PAT, we need to track I/O cache attributes for coherency */
//                         DPRINT1("HAL I/O Mapping at %p is unsafe\n", BaseAddress);
//                     }
//                 }
//
//                 /* Move to the next page */
//                 BaseAddress = (PVOID)((ULONG_PTR)BaseAddress + PAGE_SIZE);
//                 PointerPte++;
//             }
//         }
//         else
//         {
//             /* Move to the next address */
//             BaseAddress = (PVOID)((ULONG_PTR)BaseAddress + PDE_MAPPED_VA);
//         }
//
//         /* Move to the next PDE */
//         PointerPde++;
//     }
// }
//
// VOID
// NTAPI
// MmDumpArmPfnDatabase(IN BOOLEAN StatusOnly)
// {
//     ULONG i;
//     PMMPFN Pfn1;
//     PCHAR Consumer = "Unknown";
//     KIRQL OldIrql;
//     ULONG ActivePages = 0, FreePages = 0, OtherPages = 0;
// #if MI_TRACE_PFNS
//     ULONG UsageBucket[MI_USAGE_FREE_PAGE + 1] = {0};
//     PCHAR MI_USAGE_TEXT[MI_USAGE_FREE_PAGE + 1] =
//     {
//         "Not set",
//         "Paged Pool",
//         "Nonpaged Pool",
//         "Nonpaged Pool Ex",
//         "Kernel Stack",
//         "Kernel Stack Ex",
//         "System PTE",
//         "VAD",
//         "PEB/TEB",
//         "Section",
//         "Page Table",
//         "Page Directory",
//         "Old Page Table",
//         "Driver Page",
//         "Contiguous Alloc",
//         "MDL",
//         "Demand Zero",
//         "Zero Loop",
//         "Cache",
//         "PFN Database",
//         "Boot Driver",
//         "Initial Memory",
//         "Free Page"
//     };
// #endif
//     //
//     // Loop the PFN database
//     //
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql);
//     for (i = 0; i <= MmHighestPhysicalPage; i++)
//     {
//         Pfn1 = MiGetPfnEntry(i);
//         if (!Pfn1) continue;
// #if MI_TRACE_PFNS
//         ASSERT(Pfn1->PfnUsage <= MI_USAGE_FREE_PAGE);
// #endif
//         //
//         // Get the page location
//         //
//         switch (Pfn1->u3.e1.PageLocation)
//         {
//             case ActiveAndValid:
//
//                 Consumer = "Active and Valid";
//                 ActivePages++;
//                 break;
//
//             case ZeroedPageList:
//
//                 Consumer = "Zero Page List";
//                 FreePages++;
//                 break;//continue;
//
//             case FreePageList:
//
//                 Consumer = "Free Page List";
//                 FreePages++;
//                 break;//continue;
//
//             default:
//
//                 Consumer = "Other (ASSERT!)";
//                 OtherPages++;
//                 break;
//         }
//
// #if MI_TRACE_PFNS
//         /* Add into bucket */
//         UsageBucket[Pfn1->PfnUsage]++;
// #endif
//
//         //
//         // Pretty-print the page
//         //
//         if (!StatusOnly)
//         DbgPrint("0x%08p:\t%20s\t(%04d.%04d)\t[%16s - %16s]\n",
//                  i << PAGE_SHIFT,
//                  Consumer,
//                  Pfn1->u3.e2.ReferenceCount,
//                  Pfn1->u2.ShareCount == LIST_HEAD ? 0xFFFF : Pfn1->u2.ShareCount,
// #if MI_TRACE_PFNS
//                  MI_USAGE_TEXT[Pfn1->PfnUsage],
//                  Pfn1->ProcessName);
// #else
//                  "Page tracking",
//                  "is disabled");
// #endif
//     }
//
//     DbgPrint("Active:               %5d pages\t[%6d KB]\n", ActivePages,  (ActivePages    << PAGE_SHIFT) / 1024);
//     DbgPrint("Free:                 %5d pages\t[%6d KB]\n", FreePages,    (FreePages      << PAGE_SHIFT) / 1024);
//     DbgPrint("Other:                %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     DbgPrint("-----------------------------------------\n");
// #if MI_TRACE_PFNS
//     OtherPages = UsageBucket[MI_USAGE_BOOT_DRIVER];
//     DbgPrint("Boot Images:          %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_DRIVER_PAGE];
//     DbgPrint("System Drivers:       %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_PFN_DATABASE];
//     DbgPrint("PFN Database:         %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_PAGE_TABLE] + UsageBucket[MI_USAGE_PAGE_DIRECTORY] + UsageBucket[MI_USAGE_LEGACY_PAGE_DIRECTORY];
//     DbgPrint("Page Tables:          %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_SYSTEM_PTE];
//     DbgPrint("System PTEs:          %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_VAD];
//     DbgPrint("VADs:                 %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_CONTINOUS_ALLOCATION];
//     DbgPrint("Continuous Allocs:    %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_MDL];
//     DbgPrint("MDLs:                 %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_NONPAGED_POOL] + UsageBucket[MI_USAGE_NONPAGED_POOL_EXPANSION];
//     DbgPrint("NonPaged Pool:        %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_PAGED_POOL];
//     DbgPrint("Paged Pool:           %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_DEMAND_ZERO];
//     DbgPrint("Demand Zero:          %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_ZERO_LOOP];
//     DbgPrint("Zero Loop:            %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_PEB_TEB];
//     DbgPrint("PEB/TEB:              %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_KERNEL_STACK] + UsageBucket[MI_USAGE_KERNEL_STACK_EXPANSION];
//     DbgPrint("Kernel Stack:         %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_INIT_MEMORY];
//     DbgPrint("Init Memory:          %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_SECTION];
//     DbgPrint("Sections:             %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_CACHE];
//     DbgPrint("Cache:                %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
//     OtherPages = UsageBucket[MI_USAGE_FREE_PAGE];
//     DbgPrint("Free:                 %5d pages\t[%6d KB]\n", OtherPages,   (OtherPages     << PAGE_SHIFT) / 1024);
// #endif
//     KeLowerIrql(OldIrql);
// }
//
// CODE_SEG("INIT")
// PPHYSICAL_MEMORY_DESCRIPTOR
// NTAPI
// MmInitializeMemoryLimits(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                          IN PBOOLEAN IncludeType)
// {
//     PLIST_ENTRY NextEntry;
//     ULONG Run = 0, InitialRuns;
//     PFN_NUMBER NextPage = -1, PageCount = 0;
//     PPHYSICAL_MEMORY_DESCRIPTOR Buffer, NewBuffer;
//     PMEMORY_ALLOCATION_DESCRIPTOR MdBlock;
//
//     //
//     // Start with the maximum we might need
//     //
//     InitialRuns = MiNumberDescriptors;
//
//     //
//     // Allocate the maximum we'll ever need
//     //
//     Buffer = ExAllocatePoolWithTag(NonPagedPool,
//                                    sizeof(PHYSICAL_MEMORY_DESCRIPTOR) +
//                                    sizeof(PHYSICAL_MEMORY_RUN) *
//                                    (InitialRuns - 1),
//                                    'lMmM');
//     if (!Buffer) return NULL;
//
//     //
//     // For now that's how many runs we have
//     //
//     Buffer->NumberOfRuns = InitialRuns;
//
//     //
//     // Now loop through the descriptors again
//     //
//     NextEntry = LoaderBlock->MemoryDescriptorListHead.Flink;
//     while (NextEntry != &LoaderBlock->MemoryDescriptorListHead)
//     {
//         //
//         // Grab each one, and check if it's one we should include
//         //
//         MdBlock = CONTAINING_RECORD(NextEntry,
//                                     MEMORY_ALLOCATION_DESCRIPTOR,
//                                     ListEntry);
//         if ((MdBlock->MemoryType < LoaderMaximum) &&
//             (IncludeType[MdBlock->MemoryType]))
//         {
//             //
//             // Add this to our running total
//             //
//             PageCount += MdBlock->PageCount;
//
//             //
//             // Check if the next page is described by the next descriptor
//             //
//             if (MdBlock->BasePage == NextPage)
//             {
//                 //
//                 // Combine it into the same physical run
//                 //
//                 ASSERT(MdBlock->PageCount != 0);
//                 Buffer->Run[Run - 1].PageCount += MdBlock->PageCount;
//                 NextPage += MdBlock->PageCount;
//             }
//             else
//             {
//                 //
//                 // Otherwise just duplicate the descriptor's contents
//                 //
//                 Buffer->Run[Run].BasePage = MdBlock->BasePage;
//                 Buffer->Run[Run].PageCount = MdBlock->PageCount;
//                 NextPage = Buffer->Run[Run].BasePage + Buffer->Run[Run].PageCount;
//
//                 //
//                 // And in this case, increase the number of runs
//                 //
//                 Run++;
//             }
//         }
//
//         //
//         // Try the next descriptor
//         //
//         NextEntry = MdBlock->ListEntry.Flink;
//     }
//
//     //
//     // We should not have been able to go past our initial estimate
//     //
//     ASSERT(Run <= Buffer->NumberOfRuns);
//
//     //
//     // Our guess was probably exaggerated...
//     //
//     if (InitialRuns > Run)
//     {
//         //
//         // Allocate a more accurately sized buffer
//         //
//         NewBuffer = ExAllocatePoolWithTag(NonPagedPool,
//                                           sizeof(PHYSICAL_MEMORY_DESCRIPTOR) +
//                                           sizeof(PHYSICAL_MEMORY_RUN) *
//                                           (Run - 1),
//                                           'lMmM');
//         if (NewBuffer)
//         {
//             //
//             // Copy the old buffer into the new, then free it
//             //
//             RtlCopyMemory(NewBuffer->Run,
//                           Buffer->Run,
//                           sizeof(PHYSICAL_MEMORY_RUN) * Run);
//             ExFreePoolWithTag(Buffer, 'lMmM');
//
//             //
//             // Now use the new buffer
//             //
//             Buffer = NewBuffer;
//         }
//     }
//
//     //
//     // Write the final numbers, and return it
//     //
//     Buffer->NumberOfRuns = Run;
//     Buffer->NumberOfPages = PageCount;
//     return Buffer;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiBuildPagedPool(VOID)
// {
//     PMMPTE PointerPte;
//     PMMPDE PointerPde;
//     MMPDE TempPde = ValidKernelPde;
//     PFN_NUMBER PageFrameIndex;
//     KIRQL OldIrql;
//     SIZE_T Size, NumberOfPages, NumberOfPdes;
//     ULONG BitMapSize;
// #if (_MI_PAGING_LEVELS >= 3)
//     MMPPE TempPpe = ValidKernelPpe;
//     PMMPPE PointerPpe;
// #elif (_MI_PAGING_LEVELS == 2)
//     MMPTE TempPte = ValidKernelPte;
//
//     //
//     // Get the page frame number for the system page directory
//     //
//     PointerPte = MiAddressToPte(PDE_BASE);
//     ASSERT(PPE_PER_PAGE == 1);
//     MmSystemPageDirectory[0] = PFN_FROM_PTE(PointerPte);
//
//     //
//     // Allocate a system PTE which will hold a copy of the page directory
//     //
//     PointerPte = MiReserveSystemPtes(1, SystemPteSpace);
//     ASSERT(PointerPte);
//     MmSystemPagePtes = MiPteToAddress(PointerPte);
//
//     //
//     // Make this system PTE point to the system page directory.
//     // It is now essentially double-mapped. This will be used later for lazy
//     // evaluation of PDEs accross process switches, similarly to how the Global
//     // page directory array in the old ReactOS Mm is used (but in a less hacky
//     // way).
//     //
//     TempPte = ValidKernelPte;
//     ASSERT(PPE_PER_PAGE == 1);
//     TempPte.set_page_frame_number((MmSystemPageDirectory[0]) as PFN_NUMBER);
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
// #endif
//
// #ifdef _M_IX86
//     //
//     // Let's get back to paged pool work: size it up.
//     // By default, it should be twice as big as nonpaged pool.
//     //
//     MmSizeOfPagedPoolInBytes = 2 * MmMaximumNonPagedPoolInBytes;
//     if (MmSizeOfPagedPoolInBytes > ((ULONG_PTR)MmNonPagedSystemStart -
//                                     (ULONG_PTR)MmPagedPoolStart))
//     {
//         //
//         // On the other hand, we have limited VA space, so make sure that the VA
//         // for paged pool doesn't overflow into nonpaged pool VA. Otherwise, set
//         // whatever maximum is possible.
//         //
//         MmSizeOfPagedPoolInBytes = (ULONG_PTR)MmNonPagedSystemStart -
//                                    (ULONG_PTR)MmPagedPoolStart;
//     }
// #endif // _M_IX86
//
//     //
//     // Get the size in pages and make sure paged pool is at least 32MB.
//     //
//     Size = MmSizeOfPagedPoolInBytes;
//     if (Size < MI_MIN_INIT_PAGED_POOLSIZE) Size = MI_MIN_INIT_PAGED_POOLSIZE;
//     NumberOfPages = BYTES_TO_PAGES(Size);
//
//     //
//     // Now check how many PDEs will be required for these many pages.
//     //
//     NumberOfPdes = (NumberOfPages + (PTE_PER_PAGE - 1)) / PTE_PER_PAGE;
//
//     //
//     // Recompute the PDE-aligned size of the paged pool, in bytes and pages.
//     //
//     MmSizeOfPagedPoolInBytes = NumberOfPdes * PTE_PER_PAGE * PAGE_SIZE;
//     MmSizeOfPagedPoolInPages = MmSizeOfPagedPoolInBytes >> PAGE_SHIFT;
//
// #ifdef _M_IX86
//     //
//     // Let's be really sure this doesn't overflow into nonpaged system VA
//     //
//     ASSERT((MmSizeOfPagedPoolInBytes + (ULONG_PTR)MmPagedPoolStart) <=
//            (ULONG_PTR)MmNonPagedSystemStart);
// #endif // _M_IX86
//
//     //
//     // This is where paged pool ends
//     //
//     MmPagedPoolEnd = (PVOID)(((ULONG_PTR)MmPagedPoolStart +
//                               MmSizeOfPagedPoolInBytes) - 1);
//
//     //
//     // Lock the PFN database
//     //
//     OldIrql = MiAcquirePfnLock();
//
// #if (_MI_PAGING_LEVELS >= 3)
//     /* On these systems, there's no double-mapping, so instead, the PPEs
//      * are setup to span the entire paged pool area, so there's no need for the
//      * system PD */
//     for (PointerPpe = MiAddressToPpe(MmPagedPoolStart);
//          PointerPpe <= MiAddressToPpe(MmPagedPoolEnd);
//          PointerPpe++)
//     {
//         /* Check if the PPE is already valid */
//         if (!PointerPpe->u.Hard.Valid)
//         {
//             /* It is not, so map a fresh zeroed page */
//             TempPpe.set_page_frame_number((MiRemoveZeroPage(0)) as PFN_NUMBER);
//             MI_WRITE_VALID_PPE(PointerPpe, TempPpe);
//             MiInitializePfnForOtherProcess(TempPpe.page_frame_number(),
//                                            (PMMPTE)PointerPpe,
//                                            PFN_FROM_PTE(MiAddressToPte(PointerPpe)));
//         }
//     }
// #endif
//
//     //
//     // So now get the PDE for paged pool and zero it out
//     //
//     PointerPde = MiAddressToPde(MmPagedPoolStart);
//     RtlZeroMemory(PointerPde,
//                   (1 + MiAddressToPde(MmPagedPoolEnd) - PointerPde) * sizeof(MMPDE));
//
//     //
//     // Next, get the first and last PTE
//     //
//     PointerPte = MiAddressToPte(MmPagedPoolStart);
//     MmPagedPoolInfo.FirstPteForPagedPool = PointerPte;
//     MmPagedPoolInfo.LastPteForPagedPool = MiAddressToPte(MmPagedPoolEnd);
//
//     /* Allocate a page and map the first paged pool PDE */
//     MI_SET_USAGE(MI_USAGE_PAGED_POOL);
//     super::helpers::MI_SET_PROCESS2("Kernel");
//     PageFrameIndex = MiRemoveZeroPage(0);
//     TempPde.set_page_frame_number((PageFrameIndex) as PFN_NUMBER);
//     MI_WRITE_VALID_PDE(PointerPde, TempPde);
// #if (_MI_PAGING_LEVELS >= 3)
//     /* Use the PPE of MmPagedPoolStart that was setup above */
// //    Bla = PFN_FROM_PTE(PpeAddress(MmPagedPool...));
//
//     /* Initialize the PFN entry for it */
//     MiInitializePfnForOtherProcess(PageFrameIndex,
//                                    (PMMPTE)PointerPde,
//                                    PFN_FROM_PTE(MiAddressToPpe(MmPagedPoolStart)));
// #else
//     /* Do it this way */
// //    Bla = MmSystemPageDirectory[(PointerPde - (PMMPTE)PDE_BASE) / PDE_PER_PAGE]
//
//     /* Initialize the PFN entry for it */
//     MiInitializePfnForOtherProcess(PageFrameIndex,
//                                    (PMMPTE)PointerPde,
//                                    MmSystemPageDirectory[(PointerPde - (PMMPDE)PDE_BASE) / PDE_PER_PAGE]);
// #endif
//
//     //
//     // Release the PFN database lock
//     //
//     MiReleasePfnLock(OldIrql);
//
//     //
//     // We only have one PDE mapped for now... at fault time, additional PDEs
//     // will be allocated to handle paged pool growth. This is where they'll have
//     // to start.
//     //
//     MmPagedPoolInfo.NextPdeForPagedPoolExpansion = PointerPde + 1;
//
//     //
//     // We keep track of each page via a bit, so check how big the bitmap will
//     // have to be (make sure to align our page count such that it fits nicely
//     // into a 4-byte aligned bitmap.
//     //
//     // We'll also allocate the bitmap header itself part of the same buffer.
//     //
//     NumberOfPages = NumberOfPdes * PTE_PER_PAGE;
//     ASSERT(NumberOfPages == MmSizeOfPagedPoolInPages);
//     BitMapSize = (ULONG)NumberOfPages;
//     Size = sizeof(RTL_BITMAP) + (((BitMapSize + 31) / 32) * sizeof(ULONG));
//
//     //
//     // Allocate the allocation bitmap, which tells us which regions have not yet
//     // been mapped into memory
//     //
//     MmPagedPoolInfo.PagedPoolAllocationMap = ExAllocatePoolWithTag(NonPagedPool,
//                                                                    Size,
//                                                                    TAG_MM);
//     ASSERT(MmPagedPoolInfo.PagedPoolAllocationMap);
//
//     //
//     // Initialize it such that at first, only the first page's worth of PTEs is
//     // marked as allocated (incidentially, the first PDE we allocated earlier).
//     //
//     RtlInitializeBitMap(MmPagedPoolInfo.PagedPoolAllocationMap,
//                         (PULONG)(MmPagedPoolInfo.PagedPoolAllocationMap + 1),
//                         BitMapSize);
//     RtlSetAllBits(MmPagedPoolInfo.PagedPoolAllocationMap);
//     RtlClearBits(MmPagedPoolInfo.PagedPoolAllocationMap, 0, PTE_PER_PAGE);
//
//     //
//     // We have a second bitmap, which keeps track of where allocations end.
//     // Given the allocation bitmap and a base address, we can therefore figure
//     // out which page is the last page of that allocation, and thus how big the
//     // entire allocation is.
//     //
//     MmPagedPoolInfo.EndOfPagedPoolBitmap = ExAllocatePoolWithTag(NonPagedPool,
//                                                                  Size,
//                                                                  TAG_MM);
//     ASSERT(MmPagedPoolInfo.EndOfPagedPoolBitmap);
//     RtlInitializeBitMap(MmPagedPoolInfo.EndOfPagedPoolBitmap,
//                         (PULONG)(MmPagedPoolInfo.EndOfPagedPoolBitmap + 1),
//                         BitMapSize);
//
//     //
//     // Since no allocations have been made yet, there are no bits set as the end
//     //
//     RtlClearAllBits(MmPagedPoolInfo.EndOfPagedPoolBitmap);
//
//     //
//     // Initialize paged pool.
//     //
//     InitializePool(PagedPool, 0);
//
//     /* Initialize special pool */
//     MiInitializeSpecialPool();
//
//     /* Default low threshold of 30MB or one fifth of paged pool */
//     MiLowPagedPoolThreshold = (30 * _1MB) >> PAGE_SHIFT;
//     MiLowPagedPoolThreshold = min(MiLowPagedPoolThreshold, Size / 5);
//
//     /* Default high threshold of 60MB or 25% */
//     MiHighPagedPoolThreshold = (60 * _1MB) >> PAGE_SHIFT;
//     MiHighPagedPoolThreshold = min(MiHighPagedPoolThreshold, (Size * 2) / 5);
//     ASSERT(MiLowPagedPoolThreshold < MiHighPagedPoolThreshold);
//
//     /* Setup the global session space */
//     MiInitializeSystemSpaceMap(NULL);
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// MiDbgDumpMemoryDescriptors(VOID)
// {
//     PLIST_ENTRY NextEntry;
//     PMEMORY_ALLOCATION_DESCRIPTOR Md;
//     PFN_NUMBER TotalPages = 0;
//     PCHAR
//     MemType[] =
//     {
//         "ExceptionBlock    ",
//         "SystemBlock       ",
//         "Free              ",
//         "Bad               ",
//         "LoadedProgram     ",
//         "FirmwareTemporary ",
//         "FirmwarePermanent ",
//         "OsloaderHeap      ",
//         "OsloaderStack     ",
//         "SystemCode        ",
//         "HalCode           ",
//         "BootDriver        ",
//         "ConsoleInDriver   ",
//         "ConsoleOutDriver  ",
//         "StartupDpcStack   ",
//         "StartupKernelStack",
//         "StartupPanicStack ",
//         "StartupPcrPage    ",
//         "StartupPdrPage    ",
//         "RegistryData      ",
//         "MemoryData        ",
//         "NlsData           ",
//         "SpecialMemory     ",
//         "BBTMemory         ",
//         "LoaderReserve     ",
//         "LoaderXIPRom      "
//     };
//
//     DPRINT1("Base\t\tLength\t\tType\n");
//     for (NextEntry = KeLoaderBlock->MemoryDescriptorListHead.Flink;
//          NextEntry != &KeLoaderBlock->MemoryDescriptorListHead;
//          NextEntry = NextEntry->Flink)
//     {
//         Md = CONTAINING_RECORD(NextEntry, MEMORY_ALLOCATION_DESCRIPTOR, ListEntry);
//         DPRINT1("%08lX\t%08lX\t%s\n", Md->BasePage, Md->PageCount, MemType[Md->MemoryType]);
//         TotalPages += Md->PageCount;
//     }
//
//     DPRINT1("Total: %08lX (%lu MB)\n", (ULONG)TotalPages, (ULONG)(TotalPages * PAGE_SIZE) / 1024 / 1024);
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// MmArmInitSystem(IN ULONG Phase,
//                 IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     ULONG i;
//     BOOLEAN IncludeType[LoaderMaximum];
//     PVOID Bitmap;
//     PPHYSICAL_MEMORY_RUN Run;
//     PFN_NUMBER PageCount;
// #if DBG
//     ULONG j;
//     PMMPTE PointerPte, TestPte;
//     MMPTE TempPte;
// #endif
//
//
//     ARM64_UART_TRACE("[MM] MmArmInitSystem entered\r\n");
//
//     /* Dump memory descriptors */
//     if (MiDbgEnableMdDump) MiDbgDumpMemoryDescriptors();
//
//     //
//     // Instantiate memory that we don't consider RAM/usable
//     // We use the same exclusions that Windows does, in order to try to be
//     // compatible with WinLDR-style booting
//     //
//     for (i = 0; i < LoaderMaximum; i++) IncludeType[i] = TRUE;
//     IncludeType[LoaderBad] = FALSE;
//     IncludeType[LoaderFirmwarePermanent] = FALSE;
//     IncludeType[LoaderSpecialMemory] = FALSE;
//     IncludeType[LoaderBBTMemory] = FALSE;
//     if (Phase == 0)
//     {
//         /* Count physical pages on the system */
//         MiScanMemoryDescriptors(LoaderBlock);
//
//         /* Initialize the phase 0 temporary event */
//         KeInitializeEvent(&MiTempEvent, NotificationEvent, FALSE);
//
//         /* Set all the events to use the temporary event for now */
//         MiLowMemoryEvent = &MiTempEvent;
//         MiHighMemoryEvent = &MiTempEvent;
//         MiLowPagedPoolEvent = &MiTempEvent;
//         MiHighPagedPoolEvent = &MiTempEvent;
//         MiLowNonPagedPoolEvent = &MiTempEvent;
//         MiHighNonPagedPoolEvent = &MiTempEvent;
//
//         //
//         // Default throttling limits for Cc
//         // May be ajusted later on depending on system type
//         //
//         MmThrottleTop = 450;
//         MmThrottleBottom = 127;
//
//         //
//         // Define the basic user vs. kernel address space separation
//         //
//         MmSystemRangeStart = (PVOID)MI_DEFAULT_SYSTEM_RANGE_START;
//         MmUserProbeAddress = (ULONG_PTR)MI_USER_PROBE_ADDRESS;
//         MmHighestUserAddress = (PVOID)MI_HIGHEST_USER_ADDRESS;
//
//         /* Highest PTE and PDE based on the addresses above */
//         MiHighestUserPte = MiAddressToPte(MmHighestUserAddress);
//         MiHighestUserPde = MiAddressToPde(MmHighestUserAddress);
// #if (_MI_PAGING_LEVELS >= 3)
//         MiHighestUserPpe = MiAddressToPpe(MmHighestUserAddress);
// #if (_MI_PAGING_LEVELS >= 4)
//         MiHighestUserPxe = MiAddressToPxe(MmHighestUserAddress);
// #endif
// #endif
//         //
//         // Get the size of the boot loader's image allocations and then round
//         // that region up to a PDE size, so that any PDEs we might create for
//         // whatever follows are separate from the PDEs that boot loader might've
//         // already created (and later, we can blow all that away if we want to).
//         //
//         MmBootImageSize = KeLoaderBlock->Extension->LoaderPagesSpanned;
//         MmBootImageSize *= PAGE_SIZE;
//         MmBootImageSize = (MmBootImageSize + PDE_MAPPED_VA - 1) & ~(PDE_MAPPED_VA - 1);
//         ASSERT((MmBootImageSize % PDE_MAPPED_VA) == 0);
//
//         /* Initialize session space address layout */
//         MiInitializeSessionSpaceLayout();
//
//         /* Set the based section highest address */
//         MmHighSectionBase = (PVOID)((ULONG_PTR)MmHighestUserAddress - 0x800000);
//
//         /* Loop all 8 standby lists */
//         for (i = 0; i < 8; i++)
//         {
//             /* Initialize them */
//             MmStandbyPageListByPriority[i].Total = 0;
//             MmStandbyPageListByPriority[i].ListName = StandbyPageList;
//             MmStandbyPageListByPriority[i].Flink = MM_EMPTY_LIST;
//             MmStandbyPageListByPriority[i].Blink = MM_EMPTY_LIST;
//         }
//
//         /* Initialize the user mode image list */
//         InitializeListHead(&MmLoadedUserImageList);
//
//         /* Initalize the Working set list */
//         InitializeListHead(&MmWorkingSetExpansionHead);
//
//         /* Initialize critical section timeout value (relative time is negative) */
//         MmCriticalSectionTimeout.QuadPart = MmCritsectTimeoutSeconds * (-10000000LL);
//
//         /* Initialize the paged pool mutex and the section commit mutex */
//         KeInitializeGuardedMutex(&MmPagedPoolMutex);
//         KeInitializeGuardedMutex(&MmSectionCommitMutex);
//         KeInitializeGuardedMutex(&MmSectionBasedMutex);
//
//         /* Initialize the Loader Lock */
//         KeInitializeMutant(&MmSystemLoadLock, FALSE);
//
//         /* Set up the zero page event */
//         KeInitializeEvent(&MmZeroingPageEvent, NotificationEvent, FALSE);
//
//         /* Initialize the dead stack S-LIST */
//         InitializeSListHead(&MmDeadStackSListHead);
//
//         //
//         // Check if this is a machine with less than 19MB of RAM
//         //
//         PageCount = MmNumberOfPhysicalPages;
//         if (PageCount < MI_MIN_PAGES_FOR_SYSPTE_TUNING)
//         {
//             //
//             // Use the very minimum of system PTEs
//             //
//             MmNumberOfSystemPtes = 7000;
//         }
//         else
//         {
//             //
//             // Use the default
//             //
//             MmNumberOfSystemPtes = 11000;
//             if (PageCount > MI_MIN_PAGES_FOR_SYSPTE_BOOST)
//             {
//                 //
//                 // Double the amount of system PTEs
//                 //
//                 MmNumberOfSystemPtes <<= 1;
//             }
//             if (PageCount > MI_MIN_PAGES_FOR_SYSPTE_BOOST_BOOST)
//             {
//                 //
//                 // Double the amount of system PTEs
//                 //
//                 MmNumberOfSystemPtes <<= 1;
//             }
//             if (MmSpecialPoolTag != 0 && MmSpecialPoolTag != -1)
//             {
//                 //
//                 // Add some extra PTEs for special pool
//                 //
//                 MmNumberOfSystemPtes += 0x6000;
//             }
//         }
//
//         DPRINT("System PTE count has been tuned to %lu (%lu bytes)\n",
//                MmNumberOfSystemPtes, MmNumberOfSystemPtes * PAGE_SIZE);
//
//         /* Check if no values are set for the heap limits */
//         if (MmHeapSegmentReserve == 0)
//         {
//             MmHeapSegmentReserve = 2 * _1MB;
//         }
//
//         if (MmHeapSegmentCommit == 0)
//         {
//             MmHeapSegmentCommit = 2 * PAGE_SIZE;
//         }
//
//         if (MmHeapDeCommitTotalFreeThreshold == 0)
//         {
//             MmHeapDeCommitTotalFreeThreshold = 64 * _1KB;
//         }
//
//         if (MmHeapDeCommitFreeBlockThreshold == 0)
//         {
//             MmHeapDeCommitFreeBlockThreshold = PAGE_SIZE;
//         }
//
//         /* Initialize the working set lock */
//         ExInitializePushLock(&MmSystemCacheWs.WorkingSetMutex);
//
//         /* Set commit limit */
//         MmTotalCommitLimit = (2 * _1GB) >> PAGE_SHIFT;
//         MmTotalCommitLimitMaximum = MmTotalCommitLimit;
//
//         /* Has the allocation fragment been setup? */
//         if (!MmAllocationFragment)
//         {
//             /* Use the default value */
//             MmAllocationFragment = MI_ALLOCATION_FRAGMENT;
//             if (PageCount < ((256 * _1MB) / PAGE_SIZE))
//             {
//                 /* On memory systems with less than 256MB, divide by 4 */
//                 MmAllocationFragment = MI_ALLOCATION_FRAGMENT / 4;
//             }
//             else if (PageCount < (_1GB / PAGE_SIZE))
//             {
//                 /* On systems with less than 1GB, divide by 2 */
//                 MmAllocationFragment = MI_ALLOCATION_FRAGMENT / 2;
//             }
//         }
//         else
//         {
//             /* Convert from 1KB fragments to pages */
//             MmAllocationFragment *= _1KB;
//             MmAllocationFragment = ROUND_TO_PAGES(MmAllocationFragment);
//
//             /* Don't let it past the maximum */
//             MmAllocationFragment = min(MmAllocationFragment,
//                                        MI_MAX_ALLOCATION_FRAGMENT);
//
//             /* Don't let it too small either */
//             MmAllocationFragment = max(MmAllocationFragment,
//                                        MI_MIN_ALLOCATION_FRAGMENT);
//         }
//
//         /* Check for kernel stack size that's too big */
//         if (MmLargeStackSize > (KERNEL_LARGE_STACK_SIZE / _1KB))
//         {
//             /* Sanitize to default value */
//             MmLargeStackSize = KERNEL_LARGE_STACK_SIZE;
//         }
//         else
//         {
//             /* Take the registry setting, and convert it into bytes */
//             MmLargeStackSize *= _1KB;
//
//             /* Now align it to a page boundary */
//             MmLargeStackSize = PAGE_ROUND_UP(MmLargeStackSize);
//
//             /* Sanity checks */
//             ASSERT(MmLargeStackSize <= KERNEL_LARGE_STACK_SIZE);
//             ASSERT((MmLargeStackSize & (PAGE_SIZE - 1)) == 0);
//
//             /* Make sure it's not too low */
//             if (MmLargeStackSize < KERNEL_STACK_SIZE) MmLargeStackSize = KERNEL_STACK_SIZE;
//         }
//
//         /* Compute color information (L2 cache-separated paging lists) */
//         MiComputeColorInformation();
//
//         // Calculate the number of bytes for the PFN database
//         // then add the color tables and convert to pages
//         MxPfnAllocation = (MmHighestPhysicalPage + 1) * sizeof(MMPFN);
//         MxPfnAllocation += (MmSecondaryColors * sizeof(MMCOLOR_TABLES) * 2);
//         MxPfnAllocation >>= PAGE_SHIFT;
//
//         // We have to add one to the count here, because in the process of
//         // shifting down to the page size, we actually ended up getting the
//         // lower aligned size (so say, 0x5FFFF bytes is now 0x5F pages).
//         // Later on, we'll shift this number back into bytes, which would cause
//         // us to end up with only 0x5F000 bytes -- when we actually want to have
//         // 0x60000 bytes.
//         MxPfnAllocation++;
//
//         /* Initialize the platform-specific parts */
//         ARM64_UART_TRACE("[MM] MiInitMachineDependent\r\n");
//         MiInitMachineDependent(LoaderBlock);
// #if DBG && !defined(_M_ARM64)
//         /* Prototype PTEs are assumed to be in paged pool, so check if the math works */
//         PointerPte = (PMMPTE)MmPagedPoolStart;
//         MI_MAKE_PROTOTYPE_PTE(&TempPte, PointerPte);
//         TestPte = MiProtoPteToPte(&TempPte);
//         ASSERT(PointerPte == TestPte);
//
//         /* Try the last nonpaged pool address */
//         PointerPte = (PMMPTE)MI_NONPAGED_POOL_END;
//         MI_MAKE_PROTOTYPE_PTE(&TempPte, PointerPte);
//         TestPte = MiProtoPteToPte(&TempPte);
//         ASSERT(PointerPte == TestPte);
//
//         /* Try a bunch of random addresses near the end of the address space */
//         PointerPte = (PMMPTE)((ULONG_PTR)MI_HIGHEST_SYSTEM_ADDRESS - 0x37FFF);
//         for (j = 0; j < 20; j += 1)
//         {
//             MI_MAKE_PROTOTYPE_PTE(&TempPte, PointerPte);
//             TestPte = MiProtoPteToPte(&TempPte);
//             ASSERT(PointerPte == TestPte);
//             PointerPte++;
//         }
//
//         /* Subsection PTEs are always in nonpaged pool, pick a random address to try */
//         PointerPte = (PMMPTE)((ULONG_PTR)MmNonPagedPoolStart + (MmSizeOfNonPagedPoolInBytes / 2));
//         MI_MAKE_SUBSECTION_PTE(&TempPte, PointerPte);
//         TestPte = MiSubsectionPteToSubsection(&TempPte);
//         ASSERT(PointerPte == TestPte);
// #endif
//
//         //
//         // Build the physical memory block
//         //
//         MmPhysicalMemoryBlock = MmInitializeMemoryLimits(LoaderBlock,
//                                                          IncludeType);
//
//         //
//         // Allocate enough buffer for the PFN bitmap
//         // Align it up to a 32-bit boundary
//         //
//         Bitmap = ExAllocatePoolWithTag(NonPagedPool,
//                                        (((MmHighestPhysicalPage + 1) + 31) / 32) * 4,
//                                        TAG_MM);
//         if (!Bitmap)
//         {
//             //
//             // This is critical
//             //
//             KeBugCheckEx(INSTALL_MORE_MEMORY,
//                          MmNumberOfPhysicalPages,
//                          MmLowestPhysicalPage,
//                          MmHighestPhysicalPage,
//                          0x101);
//         }
//
//         //
//         // Initialize it and clear all the bits to begin with
//         //
//         RtlInitializeBitMap(&MiPfnBitMap,
//                             Bitmap,
//                             (ULONG)MmHighestPhysicalPage + 1);
//         RtlClearAllBits(&MiPfnBitMap);
//
//         //
//         // Loop physical memory runs
//         //
//         for (i = 0; i < MmPhysicalMemoryBlock->NumberOfRuns; i++)
//         {
//             //
//             // Get the run
//             //
//             Run = &MmPhysicalMemoryBlock->Run[i];
//             DPRINT("PHYSICAL RAM [0x%08p to 0x%08p]\n",
//                    Run->BasePage << PAGE_SHIFT,
//                    (Run->BasePage + Run->PageCount) << PAGE_SHIFT);
//
//             //
//             // Make sure it has pages inside it
//             //
//             if (Run->PageCount)
//             {
//                 //
//                 // Set the bits in the PFN bitmap
//                 //
//                 RtlSetBits(&MiPfnBitMap, (ULONG)Run->BasePage, (ULONG)Run->PageCount);
//             }
//         }
//
//         /* Look for large page cache entries that need caching */
//         MiSyncCachedRanges();
//
//         /* Loop for HAL Heap I/O device mappings that need coherency tracking */
//         MiAddHalIoMappings();
//
//         /* Set the initial resident page count */
//         MmResidentAvailablePages = MmAvailablePages - 32;
//
//         /* Initialize large page structures on PAE/x64, and MmProcessList on x86 */
//         MiInitializeLargePageSupport();
//
//         /* Check if the registry says any drivers should be loaded with large pages */
//         MiInitializeDriverLargePageList();
//
//         /* Relocate the boot drivers into system PTE space and fixup their PFNs */
//         MiReloadBootLoadedDrivers(LoaderBlock);
//
//         /* FIXME: Call out into Driver Verifier for initialization  */
//         /* Check how many pages the system has */
//         if (MmNumberOfPhysicalPages <= ((13 * _1MB) / PAGE_SIZE))
//         {
//             /* Set small system */
//             MmSystemSize = MmSmallSystem;
//             MmMaximumDeadKernelStacks = 0;
//         }
//         else if (MmNumberOfPhysicalPages <= ((19 * _1MB) / PAGE_SIZE))
//         {
//             /* Set small system and add 100 pages for the cache */
//             MmSystemSize = MmSmallSystem;
//             MmSystemCacheWsMinimum += 100;
//             MmMaximumDeadKernelStacks = 2;
//         }
//         else
//         {
//             /* Set medium system and add 400 pages for the cache */
//             MmSystemSize = MmMediumSystem;
//             MmSystemCacheWsMinimum += 400;
//             MmMaximumDeadKernelStacks = 5;
//         }
//
//         /* Check for less than 24MB */
//         if (MmNumberOfPhysicalPages < ((24 * _1MB) / PAGE_SIZE))
//         {
//             /* No more than 32 pages */
//             MmSystemCacheWsMinimum = 32;
//         }
//
//         /* Check for more than 32MB */
//         if (MmNumberOfPhysicalPages >= ((32 * _1MB) / PAGE_SIZE))
//         {
//             /* Check for product type being "Wi" for WinNT */
//             if (MmProductType == '\0i\0W')
//             {
//                 /* Then this is a large system */
//                 MmSystemSize = MmLargeSystem;
//             }
//             else
//             {
//                 /* For servers, we need 64MB to consider this as being large */
//                 if (MmNumberOfPhysicalPages >= ((64 * _1MB) / PAGE_SIZE))
//                 {
//                     /* Set it as large */
//                     MmSystemSize = MmLargeSystem;
//                 }
//             }
//         }
//
//         /* Check for more than 33 MB */
//         if (MmNumberOfPhysicalPages > ((33 * _1MB) / PAGE_SIZE))
//         {
//             /* Add another 500 pages to the cache */
//             MmSystemCacheWsMinimum += 500;
//         }
//
//         /* Now setup the shared user data fields */
//         ASSERT(SharedUserData->NumberOfPhysicalPages == 0);
//         SharedUserData->NumberOfPhysicalPages = MmNumberOfPhysicalPages;
//         SharedUserData->LargePageMinimum = 0;
//
//         /* Check for workstation (Wi for WinNT) */
//         if (MmProductType == '\0i\0W')
//         {
//             /* Set Windows NT Workstation product type */
//             SharedUserData->NtProductType = NtProductWinNt;
//             MmProductType = 0;
//
//             /* For this product, we wait till the last moment to throttle */
//             MmThrottleTop = 250;
//             MmThrottleBottom = 30;
//         }
//         else
//         {
//             /* Check for LanMan server (La for LanmanNT) */
//             if (MmProductType == '\0a\0L')
//             {
//                 /* This is a domain controller */
//                 SharedUserData->NtProductType = NtProductLanManNt;
//             }
//             else
//             {
//                 /* Otherwise it must be a normal server (Se for ServerNT) */
//                 SharedUserData->NtProductType = NtProductServer;
//             }
//
//             /* Set the product type, and make the system more aggressive with low memory */
//             MmProductType = 1;
//             MmMinimumFreePages = 81;
//
//             /* We will throttle earlier to preserve memory */
//             MmThrottleTop = 450;
//             MmThrottleBottom = 80;
//         }
//
//         /* Update working set tuning parameters */
//         MiAdjustWorkingSetManagerParameters(!MmProductType);
//
//         /* Finetune the page count by removing working set and NP expansion */
//         MmResidentAvailablePages -= MiExpansionPoolPagesInitialCharge;
//         MmResidentAvailablePages -= MmSystemCacheWsMinimum;
//         MmResidentAvailableAtInit = MmResidentAvailablePages;
//         if (MmResidentAvailablePages <= 0)
//         {
//             /* This should not happen */
//             DPRINT1("System cache working set too big\n");
//             return FALSE;
//         }
//
//         /* Define limits for system cache */
// #ifdef _M_AMD64
//         MmSizeOfSystemCacheInPages = ((MI_SYSTEM_CACHE_END + 1) - MI_SYSTEM_CACHE_START) / PAGE_SIZE;
// #else
//         MmSizeOfSystemCacheInPages = ((ULONG_PTR)MI_PAGED_POOL_START - (ULONG_PTR)MI_SYSTEM_CACHE_START) / PAGE_SIZE;
// #endif
//         MmSystemCacheEnd = (PVOID)((ULONG_PTR)MmSystemCacheStart + (MmSizeOfSystemCacheInPages * PAGE_SIZE) - 1);
// #ifdef _M_AMD64
//         ASSERT(MmSystemCacheEnd == (PVOID)MI_SYSTEM_CACHE_END);
// #else
//         ASSERT(MmSystemCacheEnd == (PVOID)((ULONG_PTR)MI_PAGED_POOL_START - 1));
// #endif
//
//         /* Initialize the system cache */
//         //MiInitializeSystemCache(MmSystemCacheWsMinimum, MmAvailablePages);
//
//         /* Update the commit limit */
//         MmTotalCommitLimit = MmAvailablePages;
//         if (MmTotalCommitLimit > 1024) MmTotalCommitLimit -= 1024;
//         MmTotalCommitLimitMaximum = MmTotalCommitLimit;
//
//         /* Size up paged pool and build the shadow system page directory */
//         ARM64_UART_TRACE("[MM] MiBuildPagedPool\r\n");
//         MiBuildPagedPool();
//
//         /* Debugger physical memory support is now ready to be used */
//         MmDebugPte = MiAddressToPte(MiDebugMapping);
//
//         /* Initialize the loaded module list */
//         ARM64_UART_TRACE("[MM] MiInitializeLoadedModuleList\r\n");
//         MiInitializeLoadedModuleList(LoaderBlock);
// #ifdef _M_ARM64
//         {
//             extern volatile PVOID Arm64UartKernelBase;
//             volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//             const char *_s = "[MM] MiInitializeLoadedModuleList returned\r\n";
//             if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//         }
// #endif
//     }
//
//     //
//     // Always return success for now
//     //
// #ifdef _M_ARM64
//         {
//             extern volatile PVOID Arm64UartKernelBase;
//             volatile ULONG *_u = (volatile ULONG *)Arm64UartKernelBase;
//             const char *_s = "[MM] MmArmInitSystem about to return\r\n";
//             if (_u) { while (*_s) *_u = (ULONG)*_s++; }
//         }
// #endif
//     ARM64_UART_TRACE("[MM] MmArmInitSystem returning TRUE\r\n");
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
