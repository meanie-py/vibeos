//! `MEMCENSUS=1` (tripwire): one line splitting physical memory by what holds it, printed as
//! each process's address space is torn down. It exists to find where free memory goes when a
//! long sweep (appstorm64) loses it app after app while the commit charge stays flat: commit
//! accounting covers private memory only, so a drift it cannot see is in one of the other
//! columns here.
//!
//! Every PFN is classified once, without the PFN lock, so a column is a snapshot that can be a
//! few pages off; the trend across processes is what it is for. Columns, in pages:
//!   zero free standby modified   the page lists
//!   private   a user address's page (process private memory)
//!   pagetbl   page-table pages (the PTE self-map)
//!   proto     pages behind a prototype PTE (ARM3 pagefile-backed sections)
//!   ros       pages the ReactOS Mm allocated (image and data-file sections, the cache); the
//!             MC_USER / MC_SYSTEM consumer counts are its own split
//!   npool ppool   nonpaged and paged pool
//!   sys       any other active system page (stacks, drivers, system PTE mappings)
//!   other     transition, bad and unclassified pages

use core::sync::atomic::{AtomicBool, Ordering};

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{_EPROCESS, PEPROCESS};
use ntkrnl_sys::mm::*;

/// Armed by `MEMCENSUS=1` (tripwire.rs).
pub static MEMCENSUS_ARMED: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
struct Census {
    zero: usize,
    free: usize,
    standby: usize,
    modified: usize,
    private: usize,
    pagetbl: usize,
    proto: usize,
    ros: usize,
    npool: usize,
    ppool: usize,
    sys: usize,
    other: usize,
}

#[inline]
fn within(va: usize, start: PVOID, end: PVOID) -> bool {
    va >= start as usize && va <= end as usize
}

unsafe fn take() -> Census {
    let mut c = Census::default();
    let highest_user = ntkrnl_sys::globals::MmHighestUserAddress as usize;
    for pfn in 0..=MmHighestPhysicalPage {
        let entry = MiGetPfnEntry(pfn);
        if entry.is_null() {
            continue;
        }
        let e = &*entry;
        match e.page_location() {
            ZeroedPageList => c.zero += 1,
            FreePageList => c.free += 1,
            StandbyPageList => c.standby += 1,
            ModifiedPageList | ModifiedNoWritePageList => c.modified += 1,
            ActiveAndValid => {
                if super::helpers::MI_IS_ROS_PFN(entry) {
                    c.ros += 1;
                } else if e.prototype_pte() {
                    c.proto += 1;
                } else {
                    let va = MiPteToAddress(e.pte_address()) as usize;
                    if va >= PTE_BASE && va <= PTE_TOP {
                        c.pagetbl += 1;
                    } else if va <= highest_user {
                        c.private += 1;
                    } else if within(va, MmNonPagedPoolStart, MmNonPagedPoolEnd) {
                        c.npool += 1;
                    } else if within(va, MmPagedPoolStart, MmPagedPoolEnd) {
                        c.ppool += 1;
                    } else {
                        c.sys += 1;
                    }
                }
            }
            _ => c.other += 1,
        }
    }
    c
}

/// Called at the end of `MmCleanProcessAddressSpace`, once `Process`'s VADs are gone.
pub unsafe fn MiMemoryCensusAtExit(Process: PEPROCESS) {
    if !MEMCENSUS_ARMED.load(Ordering::Relaxed) {
        return;
    }
    let c = take();
    let p = Process as *mut _EPROCESS;
    let consumers = &*core::ptr::addr_of!(crate::mm::balance::MiMemoryConsumers);
    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[MEMCENSUS] exit %.15s pid=%Iu avail=%Iu commit=%Iu | zero=%Iu free=%Iu standby=%Iu modified=%Iu private=%Iu pagetbl=%Iu proto=%Iu ros=%Iu (user=%lu system=%lu) npool=%Iu ppool=%Iu sys=%Iu other=%Iu\n\0"
            .as_ptr() as _,
        (*p).ImageFileName.as_ptr(),
        (*p).UniqueProcessId as usize,
        MmAvailablePages as usize,
        core::ptr::read_volatile(core::ptr::addr_of!(crate::mm::freelist::MmTotalCommittedPages)) as usize,
        c.zero,
        c.free,
        c.standby,
        c.modified,
        c.private,
        c.pagetbl,
        c.proto,
        c.ros,
        consumers[MC_USER as usize].PagesUsed,
        consumers[MC_SYSTEM as usize].PagesUsed,
        c.npool,
        c.ppool,
        c.sys,
        c.other,
    );
}
