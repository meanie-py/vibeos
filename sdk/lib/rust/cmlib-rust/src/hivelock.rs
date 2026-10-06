//! Per-hive lock over the cell map, the free lists and the dirty vector.
//!
//! Exceeds the C. ReactOS writers hold the registry lock SHARED plus their own
//! KCB lock, so two threads setting values under different keys of one hive run
//! `HvAllocateCell` concurrently: both walk and edit the free lists, and both may
//! `HvpAddBin`, which builds a new `BlockList`, copies the old one, frees it and
//! publishes its own. The loser's bins vanish from the map (a `BinAddress` of
//! NULL), and every reader still indexing the freed list reads garbage. On one
//! CPU the window was never hit; with `-smp 2` on arm64 it bugchecked 1 boot in 5
//! (0x1E in CmpCreateKeyControlBlock through a garbage CM_KEY_NODE, while the
//! other CPU spun in HvFreeCell on a NULL bin).
//!
//! Windows keeps the map stable with a two-level directory and serializes cell
//! allocation under `CMHIVE.HiveLock`; ReactOS declares that field and never
//! takes it. This takes it: EXCLUSIVE around every operation that edits the map,
//! free lists or dirty vector, and SHARED for just the instant a reader indexes
//! `BlockList` (the cell memory it gets back never moves; only the map does).
//! `HiveLockOwner` makes it recursive for the owning thread, so the hv* helpers
//! may call each other and `HvGetCell` freely while holding it.
//!
//! Flat hives (the loader's, CmGetSystemControlValues' early SYSTEM view) are
//! read-only and never index a `BlockList`, and their HHIVE need not sit in a
//! CMHIVE, so they are never locked. The loader build has no threads at all.

use ntkrnl_sys::ddk::HHIVE;

pub(crate) struct HiveLockGuard {
    #[cfg(not(feature = "bldr"))]
    lock: *mut ntkrnl_sys::ddk::EX_PUSH_LOCK,
    #[cfg(not(feature = "bldr"))]
    owner: *mut ntkrnl_sys::ddk::PKTHREAD,
    #[cfg(not(feature = "bldr"))]
    mode: u8, // 0 = not taken (flat or recursive), 1 = shared, 2 = exclusive
}

#[cfg(not(feature = "bldr"))]
unsafe fn acquire(hive: *mut HHIVE, exclusive: bool) -> HiveLockGuard {
    use ntkrnl_sys::ddk::{CMHIVE, PKTHREAD};
    let none = HiveLockGuard {
        lock: core::ptr::null_mut(),
        owner: core::ptr::null_mut(),
        mode: 0,
    };
    if hive.is_null() || (*hive).Flat != 0 {
        return none;
    }
    // HHIVE is the first member of CMHIVE, and every non-flat kernel hive is
    // created by CmpInitializeHive, which initializes HiveLock before HvInitialize.
    let cm = hive as *mut CMHIVE;
    let me = ntkrnl_sys::mm::KeGetCurrentThread() as PKTHREAD;
    let owner = core::ptr::addr_of_mut!((*cm).HiveLockOwner);
    if core::ptr::read_volatile(owner) == me {
        return none;
    }
    let lock = core::ptr::addr_of_mut!((*cm).HiveLock);
    ntkrnl_sys::ddk::KeEnterCriticalRegion();
    if exclusive {
        ntkrnl_sys::ddk::ExAcquirePushLockExclusive(lock);
        core::ptr::write_volatile(owner, me);
    } else {
        ntkrnl_sys::ddk::ExAcquirePushLockShared(lock);
    }
    HiveLockGuard { lock, owner, mode: if exclusive { 2 } else { 1 } }
}

/// Hold for any edit of the cell map, the free lists or the dirty vector.
#[inline]
pub(crate) unsafe fn HvpLockHiveExclusive(hive: *mut HHIVE) -> HiveLockGuard {
    #[cfg(not(feature = "bldr"))]
    {
        acquire(hive, true)
    }
    #[cfg(feature = "bldr")]
    {
        let _ = hive;
        HiveLockGuard {}
    }
}

/// Hold only while indexing `BlockList` / reading `DirtyVector`.
#[inline]
pub(crate) unsafe fn HvpLockHiveShared(hive: *mut HHIVE) -> HiveLockGuard {
    #[cfg(not(feature = "bldr"))]
    {
        acquire(hive, false)
    }
    #[cfg(feature = "bldr")]
    {
        let _ = hive;
        HiveLockGuard {}
    }
}

impl Drop for HiveLockGuard {
    fn drop(&mut self) {
        #[cfg(not(feature = "bldr"))]
        unsafe {
            match self.mode {
                2 => {
                    core::ptr::write_volatile(self.owner, core::ptr::null_mut());
                    ntkrnl_sys::ddk::ExReleasePushLockExclusive(self.lock);
                    ntkrnl_sys::ddk::KeLeaveCriticalRegion();
                }
                1 => {
                    ntkrnl_sys::ddk::ExReleasePushLockShared(self.lock);
                    ntkrnl_sys::ddk::KeLeaveCriticalRegion();
                }
                _ => {}
            }
        }
    }
}
