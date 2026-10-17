//! Cached section pages on a standby list: step 2 of
//! `docs/engineering/mm-section-cache-standby-design.md`.
//!
//! A page of an image or data-file section stays owned by its segment after the last view
//! of it goes. Until now it then sat on the user LRU, counted as in use, and came back only
//! when the balancer paged it out. Windows instead moves such a page to standby: it still
//! names its section, it counts as available, and the allocator takes it synchronously
//! when the free and zeroed lists run dry. This module is that, for ROS pages.
//!
//! The three moves, and the lock each happens under:
//!
//! - **Insert** (`MiRosStandbyInsert`, segment lock + PFN lock): the last unshare of a
//!   clean page whose only rmap is its segment association. The page leaves the user LRU
//!   for this list (the same `NextLRU`/`PreviousLRU` links), `PteAddress` records its
//!   segment slot, and it leaves `MC_USER`'s count and joins `MmAvailablePages`.
//! - **Claim** (`MiRosStandbyClaim`, segment lock, takes the PFN lock): every read and
//!   write of a slot in `section_table.rs` goes through here first. A page still on the
//!   list comes back to the LRU exactly as it left; nothing outside this file ever sees a
//!   standby page as anything but an ordinary cached one.
//! - **Repurpose** (`MiRosStandbyTakeForRepurpose`, PFN lock only, any IRQL up to
//!   DISPATCH): the allocator found nothing free. It cannot take a segment's fast mutex
//!   and must not call into pool under the PFN lock, so it does neither. It writes
//!   `MM_REPURPOSED_SSE` into the slot (read back as "not resident"), detaches the page's
//!   one rmap entry onto a deferred list, and takes a segment reference for that entry.
//!   The balancer thread (`MiRosStandbyDrainRepurposed`, passive) then clears the slot
//!   under the segment lock if it still holds the marker, drops the reference, and frees
//!   the rmap entry. The entry already names its table and index, so the deferred list
//!   needs no storage of its own.
//!
//! Only clean pages come here. Dirt reaches the SSE before the last unshare: a user
//! view's PTE dirty bit is passed to it, and Cc marks the segment (`MmMakeSegmentDirty`)
//! while its VACB still maps the page. A dirty page stays on the LRU for the balancer.

use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{PMM_RMAP_ENTRY, PMM_SECTION_SEGMENT, LARGE_INTEGER, MmDereferenceSegment};
use core::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

extern "system" {
    fn MmLockSectionSegment(Segment: PVOID);
    fn MmUnlockSectionSegment(Segment: PVOID);
    fn MmRebalanceMemoryConsumers();
}

/// What a repurposed page's slot holds until the balancer clears it: swap-format (bit 0)
/// so the table code treats it as "nothing resident here", and distinct from both a real
/// swap entry and `MM_WAIT_ENTRY`. `_MmGetPageEntrySectionSegment` reports it as 0.
#[cfg(target_pointer_width = "64")]
pub const MM_REPURPOSED_SSE: ULONG_PTR = (0x7FFF_FFFF_FFFF_FD00usize << 1) | 1;
#[cfg(target_pointer_width = "32")]
pub const MM_REPURPOSED_SSE: ULONG_PTR = (0x7fff_fd00usize << 1) | 1;

// The SSE layout (mm/section.rs): bit 0 swap, bit 1 dirty, bits 3..11 share count.
#[inline]
fn sse_is_unshared_clean_page(e: ULONG_PTR) -> bool {
    e != 0 && (e & 0x1) == 0 && (e & 0x2) == 0 && ((e & 0xFFC) >> 3) == 0
}
#[inline]
fn sse_pfn(e: ULONG_PTR) -> PFN_NUMBER {
    (e >> PAGE_SHIFT) as PFN_NUMBER
}

const RMAP_SEGMENT_MASK: ULONG_PTR = !0xFFusize;
#[inline]
fn rmap_is_segment(address: PVOID) -> bool {
    (address as ULONG_PTR) & RMAP_SEGMENT_MASK == RMAP_SEGMENT_MASK
}

// The list, oldest first. PFN lock.
static mut RosStandbyFirst: PMMPFN = core::ptr::null_mut();
static mut RosStandbyLast: PMMPFN = core::ptr::null_mut();
// Detached segment rmap entries of repurposed pages, chained through `Next`. PFN lock.
static mut RosRepurposed: PMM_RMAP_ENTRY = core::ptr::null_mut();

/// Pages on the list now (the census prints it).
pub static MiRosStandbyPages: AtomicUsize = AtomicUsize::new(0);
static ROS_REPURPOSED_TOTAL: AtomicU32 = AtomicU32::new(0);

#[inline]
unsafe fn on_standby(pfn: PMMPFN, slot: *mut ULONG_PTR) -> bool {
    (*pfn).awe_allocation() && !(*pfn).must_be_cached() && (*pfn).pte_address() as *mut ULONG_PTR == slot
}

unsafe fn list_append(pfn: PMMPFN) {
    (*pfn).set_next_lru(core::ptr::null_mut());
    (*pfn).set_previous_lru(RosStandbyLast);
    if RosStandbyLast.is_null() {
        RosStandbyFirst = pfn;
    } else {
        (*RosStandbyLast).set_next_lru(pfn);
    }
    RosStandbyLast = pfn;
}

unsafe fn list_unlink(pfn: PMMPFN) {
    let prev = (*pfn).previous_lru();
    let next = (*pfn).next_lru();
    if prev.is_null() {
        RosStandbyFirst = next;
    } else {
        (*prev).set_next_lru(next);
    }
    if next.is_null() {
        RosStandbyLast = prev;
    } else {
        (*next).set_previous_lru(prev);
    }
    (*pfn).set_next_lru(core::ptr::null_mut());
    (*pfn).set_previous_lru(core::ptr::null_mut());
}

#[inline]
unsafe fn user_pages_used() -> &'static AtomicU32 {
    &*(core::ptr::addr_of_mut!(crate::mm::balance::MiMemoryConsumers[MC_USER as usize].PagesUsed)
        as *const AtomicU32)
}

/// Back from standby to the LRU, as the page left it. PFN lock held.
unsafe fn restore_to_lru(pfn: PMMPFN) {
    list_unlink(pfn);
    (*pfn).set_pte_address(core::ptr::null_mut());
    (*pfn).set_must_be_cached(true);
    crate::mm::freelist::MmInsertLRULastUserPage(MiGetPfnEntryIndex(pfn));
    MiRosStandbyPages.fetch_sub(1, Ordering::Relaxed);
    user_pages_used().fetch_add(1, Ordering::Relaxed);
    crate::ARM3::pfnlist::MiDecrementAvailablePages();
}

/// The last unshare left `Entry` in `Offset`'s slot: put the page on standby if nothing
/// but the segment still names it. Segment lock held. Anything unexpected leaves the page
/// where it is, on the LRU, which is how every cached page lived before this list.
pub unsafe fn MiRosStandbyInsert(Segment: PMM_SECTION_SEGMENT, Offset: *mut LARGE_INTEGER, Entry: ULONG_PTR) {
    if !sse_is_unshared_clean_page(Entry) {
        return;
    }
    let slot = crate::mm::section_table::MiSectionPageEntrySlot(Segment, Offset);
    if slot.is_null() {
        return;
    }
    let page = sse_pfn(Entry);
    let _pfn_guard = crate::ARM3::helpers::PfnLockGuard::acquire();
    let pfn = MiGetPfnEntry(page);
    if pfn.is_null() || core::ptr::read_volatile(slot) != Entry {
        return;
    }
    // A ROS user page on the LRU, held by nothing but its consumer reference (an MDL or
    // the balancer's walk would hold another), with no swap copy...
    if !(*pfn).awe_allocation()
        || !(*pfn).must_be_cached()
        || (*pfn).reference_count() != 1
        || (*pfn).swap_entry() != 0
        || !(*pfn).pte_address().is_null()
    {
        return;
    }
    // ...and no mapping: its rmap list is exactly the segment association.
    let head = (*pfn).rmap_list_head() as PMM_RMAP_ENTRY;
    if head.is_null() || !(*head).Next.is_null() || !rmap_is_segment((*head).Address) {
        return;
    }
    crate::mm::freelist::MmRemoveLRUUserPage(page);
    (*pfn).set_must_be_cached(false);
    (*pfn).set_pte_address(slot as PMMPTE);
    list_append(pfn);
    MiRosStandbyPages.fetch_add(1, Ordering::Relaxed);
    user_pages_used().fetch_sub(1, Ordering::Relaxed);
    crate::ARM3::pfnlist::MiIncrementAvailablePages();
}

/// Read `Slot` for the segment code, taking its page back off the standby list first.
/// Segment lock held. Returns the entry as it now stands (possibly `MM_REPURPOSED_SSE`).
pub unsafe fn MiRosStandbyClaim(Slot: *mut ULONG_PTR) -> ULONG_PTR {
    let entry = core::ptr::read_volatile(Slot);
    if !sse_is_unshared_clean_page(entry) {
        return entry;
    }
    let _pfn_guard = crate::ARM3::helpers::PfnLockGuard::acquire();
    // Re-read under the lock: a repurpose may have replaced it since.
    let entry = core::ptr::read_volatile(Slot);
    if sse_is_unshared_clean_page(entry) {
        let pfn = MiGetPfnEntry(sse_pfn(entry));
        if !pfn.is_null() && on_standby(pfn, Slot) {
            restore_to_lru(pfn);
        }
    }
    entry
}

/// The allocator found the free and zeroed lists empty: hand it the oldest standby page.
/// PFN lock held. The caller resets the PFN the way a free-list removal leaves it.
pub unsafe fn MiRosStandbyTakeForRepurpose() -> PFN_NUMBER {
    loop {
        let pfn = RosStandbyFirst;
        if pfn.is_null() {
            return 0;
        }
        let page = MiGetPfnEntryIndex(pfn);
        let slot = (*pfn).pte_address() as *mut ULONG_PTR;
        let entry = core::ptr::read_volatile(slot);
        let head = (*pfn).rmap_list_head() as PMM_RMAP_ENTRY;
        if !sse_is_unshared_clean_page(entry)
            || sse_pfn(entry) != page
            || head.is_null()
            || !(*head).Next.is_null()
            || !rmap_is_segment((*head).Address)
        {
            // Something touched a standby page without claiming it. Say so, and give it
            // back to the LRU rather than repurpose a page somebody may still name.
            mm_guard_trip!(
                "standby.MiRosStandbyTakeForRepurpose page-changed-on-standby", 4,
                "pfn" => page,
                "slot" => slot as ULONG_PTR,
                "entry" => entry,
                "rmap" => head as ULONG_PTR,
            );
            restore_to_lru(pfn);
            continue;
        }

        core::ptr::write_volatile(slot, MM_REPURPOSED_SSE);
        // The deferred entry keeps the segment, and so its table, alive until the
        // balancer has cleared the marker; the slot's own reference goes with the marker.
        let mut offset: i64 = 0;
        let (segment, _) = crate::mm::section_table::MiSectionPageTableResolve(
            (*head).Process as PVOID,
            ((*head).Address as ULONG_PTR & 0xFF) as usize,
            &mut offset,
        );
        let refcount = &*((*segment).ReferenceCount as *const core::sync::atomic::AtomicI64);
        refcount.fetch_add(1, Ordering::SeqCst);
        (*pfn).set_rmap_list_head(core::ptr::null_mut());
        (*head).Next = RosRepurposed;
        RosRepurposed = head;

        list_unlink(pfn);
        (*pfn).set_pte_address(core::ptr::null_mut());
        (*pfn).set_awe_allocation(false);
        (*pfn).set_must_be_cached(false);
        MiRosStandbyPages.fetch_sub(1, Ordering::Relaxed);

        let n = ROS_REPURPOSED_TOTAL.fetch_add(1, Ordering::Relaxed) + 1;
        if n & 0x3FF == 1 {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[MM] repurposed standby section page #%lu\n\0".as_ptr() as _,
                n,
            );
        }
        // Wake the balancer to clear the slot.
        MmRebalanceMemoryConsumers();
        return page;
    }
}

/// Clear the slots of repurposed pages and drop what held their segments. Passive level
/// (the balancer thread), no locks held.
pub unsafe fn MiRosStandbyDrainRepurposed() {
    let mut entry = {
        let _pfn_guard = crate::ARM3::helpers::PfnLockGuard::acquire();
        let list = RosRepurposed;
        RosRepurposed = core::ptr::null_mut();
        list
    };
    while !entry.is_null() {
        let next = (*entry).Next;
        let mut offset = LARGE_INTEGER { QuadPart: 0 };
        let (segment, slot) = crate::mm::section_table::MiSectionPageTableResolve(
            (*entry).Process as PVOID,
            ((*entry).Address as ULONG_PTR & 0xFF) as usize,
            core::ptr::addr_of_mut!(offset.QuadPart),
        );
        MmLockSectionSegment(segment as PVOID);
        // Anything else there means the segment already reused the slot, taking over the
        // marker's reference when it did.
        if core::ptr::read_volatile(slot) == MM_REPURPOSED_SSE {
            crate::mm::section_table::_MmSetPageEntrySectionSegment(
                segment,
                core::ptr::addr_of_mut!(offset),
                0,
                core::ptr::null(),
                0,
            );
        }
        MmUnlockSectionSegment(segment as PVOID);
        MmDereferenceSegment(segment);
        crate::mm::rmap::MiFreeRmapEntry(entry);
        entry = next;
    }
}
