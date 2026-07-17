//! Global state for the pool allocator.
//!
//! These correspond to the globals declared in expool.c and pool.c.
//! All are `#[no_mangle]` so C code can reference them.

use core::ptr::null_mut;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;

// ============================================================================
// expool.c globals
// ============================================================================

#[no_mangle]
pub static mut ExpNumberOfPagedPools: ULONG = 0;

#[no_mangle]
pub static mut NonPagedPoolDescriptor: POOL_DESCRIPTOR = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ExpPagedPoolDescriptor: [PPOOL_DESCRIPTOR; 17] = [null_mut(); 17];

#[no_mangle]
pub static mut PoolVector: [PPOOL_DESCRIPTOR; 2] = [null_mut(); 2];

#[no_mangle]
pub static mut ExpPagedPoolMutex: PKGUARDED_MUTEX = null_mut();

#[no_mangle]
pub static mut PoolTrackTableSize: SIZE_T = 0;

#[no_mangle]
pub static mut PoolTrackTableMask: SIZE_T = 0;

#[no_mangle]
pub static mut PoolBigPageTableSize: SIZE_T = 0;

#[no_mangle]
pub static mut PoolBigPageTableHash: SIZE_T = 0;

#[no_mangle]
pub static mut ExpBigTableExpansionFailed: ULONG = 0;

#[no_mangle]
pub static mut PoolTrackTable: PPOOL_TRACKER_TABLE = null_mut();

#[no_mangle]
pub static mut PoolBigPageTable: PPOOL_TRACKER_BIG_PAGES = null_mut();

#[no_mangle]
pub static mut ExpTaggedPoolLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut PoolHitTag: ULONG = 0;

#[no_mangle]
pub static mut ExStopBadTags: BOOLEAN = 0;

#[no_mangle]
pub static mut ExpLargePoolTableLock: KSPIN_LOCK = 0;

#[no_mangle]
pub static mut ExpPoolBigEntriesInUse: ULONG = 0;

#[no_mangle]
pub static mut ExpPoolFlags: ULONG = 0;

#[no_mangle]
pub static mut ExPoolFailures: ULONG = 0;

// ============================================================================
// pool.c globals
// ============================================================================

#[no_mangle]
pub static mut MmNonPagedPoolFreeListHead: [LIST_ENTRY; MI_MAX_FREE_PAGE_LISTS] =
    [LIST_ENTRY::new(); MI_MAX_FREE_PAGE_LISTS];

#[no_mangle]
pub static mut MmNumberOfFreeNonPagedPool: PFN_COUNT = 0;

#[no_mangle]
pub static mut MiExpansionPoolPagesInitialCharge: PFN_COUNT = 0;

#[no_mangle]
pub static mut MmNonPagedPoolEnd0: PVOID = null_mut();

#[no_mangle]
pub static mut MiStartOfInitialPoolFrame: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MiEndOfInitialPoolFrame: PFN_NUMBER = 0;

#[no_mangle]
pub static mut MmPagedPoolMutex: KGUARDED_MUTEX = KGUARDED_MUTEX::zeroed();

#[no_mangle]
pub static mut MmPagedPoolInfo: MM_PAGED_POOL_INFO = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut MmAllocatedNonPagedPool: SIZE_T = 0;

#[no_mangle]
pub static mut MmTotalNonPagedPoolQuota: SIZE_T = 0;

#[no_mangle]
pub static mut MmTotalPagedPoolQuota: SIZE_T = 0;

#[no_mangle]
pub static mut MmSpecialPoolTag: ULONG = 0;

#[no_mangle]
pub static mut MmConsumedPoolPercentage: ULONG = 0;

#[no_mangle]
pub static mut MmProtectFreedNonPagedPool: BOOLEAN = 0;

#[no_mangle]
pub static mut MiNonPagedPoolSListHead: SLIST_HEADER = SLIST_HEADER::zeroed();

#[no_mangle]
pub static mut MiNonPagedPoolSListMaximum: ULONG = 4;

#[no_mangle]
pub static mut MiPagedPoolSListHead: SLIST_HEADER = SLIST_HEADER::zeroed();

#[no_mangle]
pub static mut MiPagedPoolSListMaximum: ULONG = 8;

// Registry-controlled values owned by the pool crate. Concatenated into
// CmControlVector by ntoskrnl-rust::config::cmdata. See
// docs/engineering/cmdata-chopping-plan.md.
#[no_mangle]
pub static POOL_CONTROL_VECTOR: ntkrnl_sys::SubVector<1> =
    ntkrnl_sys::SubVector({ [
        ntkrnl_sys::cv!("Session Manager\\Memory Management", "PoolUsageMaximum",
            core::ptr::addr_of_mut!(MmConsumedPoolPercentage)),
    ] });
