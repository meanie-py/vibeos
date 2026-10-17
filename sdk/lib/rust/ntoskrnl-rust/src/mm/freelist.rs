use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _MDL,
    PMDL, PPFN_NUMBER, PMM_RMAP_ENTRY,
    PHYSICAL_ADDRESS,
    MmCreateMdl,
    ExFreePoolWithTag as ddk_ExFreePoolWithTag,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub static mut MmPfnDatabase: PMMPFN = core::ptr::null_mut();

#[no_mangle]
pub static mut MmAvailablePages: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MmResidentAvailablePages: PFN_NUMBER = 0;
#[no_mangle]
pub static mut MmResidentAvailableAtInit: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmTotalCommittedPages: usize = 0;
#[no_mangle]
pub static mut MmSharedCommit: usize = 0;
#[no_mangle]
pub static mut MmDriverCommit: usize = 0;
#[no_mangle]
pub static mut MmProcessCommit: usize = 0;
#[no_mangle]
pub static mut MmPagedPoolCommit: usize = 0;
#[no_mangle]
pub static mut MmPeakCommitment: usize = 0;
#[no_mangle]
pub static mut MmtotalCommitLimitMaximum: usize = 0;

static mut FirstUserLRUPfn: PMMPFN = core::ptr::null_mut();
static mut LastUserLRUPfn: PMMPFN = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn MmGetLRUFirstUserPage() -> PFN_NUMBER {
    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

    if FirstUserLRUPfn.is_null() {
        // [guard-drop] MiReleasePfnLock(old_irql);
        return 0;
    }

    let page = MiGetPfnEntryIndex(FirstUserLRUPfn);
    MmReferencePage(page);

    // [guard-drop] MiReleasePfnLock(old_irql);

    page
}

pub(crate) unsafe fn MmInsertLRULastUserPage(page: PFN_NUMBER) {
    let pfn = MiGetPfnEntry(page);

    if FirstUserLRUPfn.is_null() {
        FirstUserLRUPfn = pfn;
    }

    (*pfn).set_previous_lru(LastUserLRUPfn);

    if !LastUserLRUPfn.is_null() {
        (*LastUserLRUPfn).set_next_lru(pfn);
    }
    LastUserLRUPfn = pfn;
}

pub(crate) unsafe fn MmRemoveLRUUserPage(page: PFN_NUMBER) {
    let pfn = MiGetPfnEntry(page);

    let prev = (*pfn).previous_lru();
    let next = (*pfn).next_lru();

    if !prev.is_null() {
        (*prev).set_next_lru(next);
    } else {
        FirstUserLRUPfn = next;
    }

    if !next.is_null() {
        (*next).set_previous_lru(prev);
    } else {
        LastUserLRUPfn = prev;
    }

    (*pfn).set_previous_lru(core::ptr::null_mut());
    (*pfn).set_next_lru(core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn MmGetLRUNextUserPage(
    previous_page: PFN_NUMBER,
    move_to_last: BOOLEAN,
) -> PFN_NUMBER {
    let mut page: PFN_NUMBER = 0;

    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

    let previous_pfn = MiGetPfnEntry(previous_page);
    let next_pfn = (*previous_pfn).next_lru();

    if (move_to_last != 0) && (MmGetReferenceCountPage(previous_page) > 1) {
        MmRemoveLRUUserPage(previous_page);
        MmInsertLRULastUserPage(previous_page);
    }

    if !next_pfn.is_null() {
        page = MiGetPfnEntryIndex(next_pfn);
        MmReferencePage(page);
    }

    MmDereferencePage(previous_page);

    drop(pfn_guard);

    page
}

#[no_mangle]
pub unsafe extern "system" fn MiIsPfnFree(pfn1: PMMPFN) -> BOOLEAN {
    let pfn1 = opaque_ptr(pfn1);
    if (*pfn1).page_location() <= StandbyPageList as u16
        && (*pfn1).flink() != 0
        && (*pfn1).blink() != 0
        && (*pfn1).reference_count() == 0
    {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiIsPfnInUse(pfn1: PMMPFN) -> BOOLEAN {
    let pfn1 = opaque_ptr(pfn1);
    if MiIsPfnFree(pfn1) != 0 {
        0u8
    } else {
        1u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiAllocatePagesForMdl(
    low_address: PHYSICAL_ADDRESS,
    high_address: PHYSICAL_ADDRESS,
    skip_bytes: PHYSICAL_ADDRESS,
    total_bytes: usize,
    _cache_attribute: MI_PFN_CACHE_ATTRIBUTE,
    _mdl_flags: ULONG,
) -> PMDL {
    let mut mdl: PMDL;
    let mut page_count: PFN_NUMBER;
    let low_page: PFN_NUMBER;
    let mut high_page: PFN_NUMBER;
    let _skip_pages: PFN_NUMBER;
    let mut pages_found: PFN_NUMBER = 0;
    let mut page: PFN_NUMBER;
    let mut mdl_page: PPFN_NUMBER;
    let old_irql: KIRQL;
    let mut look_for_zeroed_pages: i32;

    low_page = (low_address.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;

    high_page = (high_address.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;
    if high_page > MmHighestPhysicalPage {
        high_page = MmHighestPhysicalPage;
    }

    if (skip_bytes.__bindgen_anon_1.LowPart & (PAGE_SIZE as u32 - 1)) != 0 {
        return core::ptr::null_mut();
    }
    _skip_pages = (skip_bytes.QuadPart >> PAGE_SHIFT) as PFN_NUMBER;

    page_count = ((total_bytes + PAGE_SIZE - 1) >> PAGE_SHIFT) as PFN_NUMBER;
    loop {
        mdl = MmCreateMdl(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (page_count << PAGE_SHIFT) as _,
        );
        if !mdl.is_null() {
            break;
        }

        page_count -= page_count >> 4;
        if page_count == 0 {
            break;
        }
    }

    if mdl.is_null() {
        return core::ptr::null_mut();
    }

    mdl_page = (mdl as *mut u8).add(core::mem::size_of::<_MDL>()) as PPFN_NUMBER;

    old_irql = MiAcquirePfnLock();

    if (low_page == 0) && (high_page == MmHighestPhysicalPage) {
        while pages_found < page_count {
            ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_MDL);
            ntkrnl_sys::MI_SET_PROCESS2!("Kernel");

            page = 0;
            if MmAvailablePages != 0 {
                page = MiRemoveAnyPage(0);
            }

            if page == 0 {
                break;
            }

            let pfn1 = MiGetPfnEntry(page);

            (*pfn1).set_reference_count(1);
            (*pfn1).set_share_count(1);
            MI_SET_PFN_DELETED(pfn1);
            (*pfn1).set_pte_frame(0x1FFEDCB);
            (*pfn1).set_start_of_allocation(true);
            (*pfn1).set_end_of_allocation(true);
            (*pfn1).set_verifier_allocation(false);

            *mdl_page = MiGetPfnEntryIndex(pfn1) as _;
            mdl_page = mdl_page.add(1);
            pages_found += 1;
        }
    } else {
        look_for_zeroed_pages = 1;
        while look_for_zeroed_pages >= 0 {
            page = low_page;
            while page < high_page {
                let pfn1 = MiGetPfnEntry(page);
                if pfn1.is_null() {
                    page += 1;
                    continue;
                }

                if MiIsPfnInUse(pfn1) != 0 {
                    page += 1;
                    continue;
                }
                if ((*pfn1).page_location() == ZeroedPageList as u16) as i32 != look_for_zeroed_pages {
                    page += 1;
                    continue;
                }

                ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_MDL);
                ntkrnl_sys::MI_SET_PROCESS2!("Kernel");
                MiUnlinkFreeOrZeroedPage(pfn1);

                (*pfn1).set_reference_count(1);
                (*pfn1).set_share_count(1);
                MI_SET_PFN_DELETED(pfn1);
                (*pfn1).set_pte_frame(0x1FFEDCB);
                (*pfn1).set_start_of_allocation(true);
                (*pfn1).set_end_of_allocation(true);
                (*pfn1).set_verifier_allocation(false);

                *mdl_page = page as _;
                mdl_page = mdl_page.add(1);
                pages_found += 1;
                if pages_found == page_count {
                    break;
                }

                page += 1;
            }

            if pages_found == page_count {
                break;
            }

            look_for_zeroed_pages -= 1;
        }
    }

    MiReleasePfnLock(old_irql);

    if pages_found == 0 {
        ddk_ExFreePoolWithTag(mdl as PVOID, TAG_MDL);
        return core::ptr::null_mut();
    }

    (*mdl).ByteCount = (pages_found << PAGE_SHIFT) as ULONG;

    if pages_found != page_count {
        *mdl_page = LIST_HEAD as _;
    }

    mdl_page = (mdl as *mut u8).add(core::mem::size_of::<_MDL>()) as PPFN_NUMBER;
    let last_mdl_page = mdl_page.add(pages_found);
    while mdl_page < last_mdl_page {
        page = *mdl_page as PFN_NUMBER;
        mdl_page = mdl_page.add(1);
        if page == LIST_HEAD {
            break;
        }

        let pfn1 = MiGetPfnEntry(page);
        if (*pfn1).page_location() != ZeroedPageList as u16 {
            MiZeroPhysicalPage(page);
        }
        (*pfn1).set_page_location(ActiveAndValid as u16);
    }

    (*mdl).Process = core::ptr::null_mut();
    (*mdl).MdlFlags |= MDL_PAGES_LOCKED as i16;
    mdl
}

#[no_mangle]
pub unsafe extern "system" fn MmSetRmapListHeadPage(pfn: PFN_NUMBER, list_head: PMM_RMAP_ENTRY) {
    let pfn1 = MiGetPfnEntry(pfn);

    if !list_head.is_null() {
        (*pfn1).set_rmap_list_head(list_head as PVOID);
    } else {
        (*pfn1).set_rmap_list_head(core::ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmGetRmapListHeadPage(pfn: PFN_NUMBER) -> PMM_RMAP_ENTRY {
    let pfn1 = MiGetPfnEntry(pfn);

    if !(*pfn1).awe_allocation() {
        return core::ptr::null_mut();
    }

    // NOTE: an arm64-only "reject non-page-aligned heads" hack used to live here. It was
    // papering over the RMAP_LIST_HEAD accessor reading the wrong MMPFN slot (PreviousLRU —
    // an unaligned PFN-database pointer), which made every real head look like garbage and
    // silently disabled rmaps on arm64. The offset is fixed (mm.rs pfn_offsets_extra), so
    // the head is trustworthy again on all arches.
    (*pfn1).rmap_list_head() as PMM_RMAP_ENTRY
}

#[no_mangle]
pub unsafe extern "system" fn MmSetSavedSwapEntryPage(pfn: PFN_NUMBER, swap_entry: ULONG_PTR) {
    let pfn1 = MiGetPfnEntry(pfn);

    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
    (*pfn1).set_swap_entry(swap_entry);
    drop(pfn_guard);
}

#[no_mangle]
pub unsafe extern "system" fn MmGetSavedSwapEntryPage(pfn: PFN_NUMBER) -> ULONG_PTR {
    let pfn1 = MiGetPfnEntry(pfn);

    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
    let swap_entry = (*pfn1).swap_entry();
    drop(pfn_guard);

    swap_entry
}

#[no_mangle]
pub unsafe extern "system" fn MmReferencePage(pfn: PFN_NUMBER) {
    let pfn1 = MiGetPfnEntry(pfn);

    let rc = (*pfn1).reference_count();
    (*pfn1).set_reference_count(rc + 1);
}

#[no_mangle]
pub unsafe extern "system" fn MmGetReferenceCountPage(pfn: PFN_NUMBER) -> ULONG {
    let pfn1 = MiGetPfnEntry(pfn);

    (*pfn1).reference_count() as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn MmIsPageInUse(pfn: PFN_NUMBER) -> BOOLEAN {
    MiIsPfnInUse(MiGetPfnEntry(pfn))
}

#[no_mangle]
pub unsafe extern "system" fn MmDereferencePage(pfn: PFN_NUMBER) {
    let pfn1 = MiGetPfnEntry(pfn);

    let rc = (*pfn1).reference_count();
    (*pfn1).set_reference_count(rc - 1);
    if rc - 1 == 0 {
        // [FREEMAPPED] free-while-mapped tripwire (§9.97 physical-page-reuse).
        //
        // A frame whose rmap list is still populated is, by definition, still mapped
        // into somebody's address space — handing it back to the free list is the
        // corruption we have been chasing from its victims (a user image whose .text
        // reads back as garbage, ntdll heap init faulting on a page-aligned address).
        // This fires AT the free, naming the culprit PFN instead of the victim.
        //
        // Placement is load-bearing: it MUST precede `set_awe_allocation(false)` two
        // lines down, because MmGetRmapListHeadPage returns NULL unless awe_allocation
        // is set (freelist.rs:333). A tripwire anywhere below reads NULL on every page
        // and silently proves nothing.
        //
        // Loud, deliberately NOT fatal — same call as the [PFNMISMATCH] precedent.
        // MmAllocPage defensively clears rmap_list_head on ALLOCATION, which is itself
        // evidence that stale non-NULL heads survive frees today, so an assert here
        // would fire on benign leftovers as well as on a real free-while-mapped.
        // Bounded, because an unbounded print at page-free rate would bury the very
        // serial log it exists to inform; the total still tells us the true frequency.
        #[cfg(target_arch = "aarch64")]
        {
            let head = MmGetRmapListHeadPage(pfn);
            if !head.is_null() {
                use core::sync::atomic::{AtomicU32, Ordering};
                static HITS: AtomicU32 = AtomicU32::new(0);
                const LOUD: u32 = 32;
                extern "C" {
                    fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
                }
                let n = HITS.fetch_add(1, Ordering::Relaxed);
                if n < LOUD {
                    ARM64_UART_TRACE_HEX(
                        b"[FREEMAPPED] MmDereferencePage pfn=\0".as_ptr(),
                        pfn as ULONG_PTR,
                    );
                    ARM64_UART_TRACE_HEX(
                        b"[FREEMAPPED]   rmap_list_head=\0".as_ptr(),
                        head as ULONG_PTR,
                    );
                    ARM64_UART_TRACE_HEX(b"[FREEMAPPED]   hit#=\0".as_ptr(), n as ULONG_PTR + 1);
                } else if n == LOUD {
                    ARM64_UART_TRACE_HEX(
                        b"[FREEMAPPED] further hits suppressed after \0".as_ptr(),
                        LOUD as ULONG_PTR,
                    );
                }
            }
        }

        if !(*pfn1).pte_address().is_null() && (*pfn1).awe_allocation() {
            // Freeing a page that is still on the ROS standby list would leave it linked
            // there and handed out twice. Every path to a standby page claims it first.
            mm_guard_trip!(
                "freelist.MmDereferencePage page-on-standby", 4,
                "pfn" => pfn,
                "slot" => (*pfn1).pte_address() as ULONG_PTR,
            );
            ntkrnl_sys::ddk::KeBugCheck(ntkrnl_sys::bugcodes::MEMORY_MANAGEMENT);
        }
        if (*pfn1).must_be_cached() {
            MmRemoveLRUUserPage(pfn);
            (*pfn1).set_must_be_cached(false);
        }

        (*pfn1).set_page_location(ActiveAndValid as u16);

        (*pfn1).set_awe_allocation(false);

        MiInsertPageInFreeList(pfn);
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn ARM64_UART_TRACE_HEX_QUIET(prefix: *const u8, val: ULONG_PTR) {
    extern "C" {
        fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
    }
    ARM64_UART_TRACE_HEX(prefix, val);
}

#[no_mangle]
pub unsafe extern "system" fn MmAllocPage(typ: ULONG) -> PFN_NUMBER {
    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

    let pfn_offset = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
    if pfn_offset == 0 {
        // [guard-drop] MiReleasePfnLock(old_irql);
        return 0;
    }

    let pfn1 = MiGetPfnEntry(pfn_offset);
    (*pfn1).set_reference_count(1);
    (*pfn1).set_page_location(ActiveAndValid as u16);

    (*pfn1).set_awe_allocation(true);

    (*pfn1).set_swap_entry(0);
    (*pfn1).set_rmap_list_head(core::ptr::null_mut());

    (*pfn1).set_next_lru(core::ptr::null_mut());
    (*pfn1).set_previous_lru(core::ptr::null_mut());
    // The ROS Mm has no use for PteAddress, and a page off the free list still carries
    // its last ARM3 owner's. Clear it: on a ROS page, non-null means "on the standby
    // list, and this is its segment slot" (mm/standby.rs).
    (*pfn1).set_pte_address(core::ptr::null_mut());

    if typ == MC_USER {
        (*pfn1).set_must_be_cached(true);
        MmInsertLRULastUserPage(pfn_offset);
    }

    // [guard-drop] MiReleasePfnLock(old_irql);

    pfn_offset
}
