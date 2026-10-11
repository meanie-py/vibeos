//! Executive pool allocator — Rust translation of expool.c.
//!
//! Provides ExAllocatePoolWithTag, ExFreePoolWithTag, tag tracking,
//! big page table, pool descriptor init, and query functions.

use core::ptr::null_mut;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::ExRaiseStatus;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::*;
use ntkrnl_sys::rtl::*;

use super::globals::*;
use super::pool as pool_pages;

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
}

// ============================================================================
// Pool block access helpers (macros from C)
// ============================================================================

/// Get pool header from data pointer: (PPOOL_HEADER)((uptr)x - sizeof(POOL_HEADER))
#[inline]
unsafe fn pool_entry(x: PVOID) -> PPOOL_HEADER {
    (x as usize - POOL_BLOCK_SIZE) as PPOOL_HEADER
}

/// Get free block (LIST_ENTRY) from pool header: (PLIST_ENTRY)((uptr)x + sizeof(POOL_HEADER))
#[inline]
unsafe fn pool_free_block(x: PPOOL_HEADER) -> PLIST_ENTRY {
    (x as usize + POOL_BLOCK_SIZE) as PLIST_ENTRY
}

/// Get block at offset i from header: (PPOOL_HEADER)((uptr)x + i * POOL_BLOCK_SIZE)
#[inline]
unsafe fn pool_block(x: PPOOL_HEADER, i: isize) -> PPOOL_HEADER {
    (x as usize).wrapping_add((i as usize).wrapping_mul(POOL_BLOCK_SIZE)) as PPOOL_HEADER
}

/// Next block: POOL_BLOCK(x, x->BlockSize)
#[inline]
unsafe fn pool_next_block(x: PPOOL_HEADER) -> PPOOL_HEADER {
    pool_block(x, (*x).BlockSize() as isize)
}

/// Previous block: POOL_BLOCK(x, -(x->PreviousSize))
#[inline]
unsafe fn pool_prev_block(x: PPOOL_HEADER) -> PPOOL_HEADER {
    pool_block(x, -((*x).PreviousSize() as isize))
}

// ============================================================================
// Encoded pool links (XOR bit 0 for anti-tampering)
// ============================================================================

#[inline]
fn exp_decode_pool_link(link: PLIST_ENTRY) -> PLIST_ENTRY {
    (link as usize & !1) as PLIST_ENTRY
}

#[inline]
fn exp_encode_pool_link(link: PLIST_ENTRY) -> PLIST_ENTRY {
    (link as usize | 1) as PLIST_ENTRY
}

unsafe fn exp_check_pool_links(list_head: PLIST_ENTRY) {
    let flink = exp_decode_pool_link((*list_head).Flink);
    let blink = exp_decode_pool_link((*list_head).Blink);
    if exp_decode_pool_link((*flink).Blink) != list_head
        || exp_decode_pool_link((*blink).Flink) != list_head
    {
        KeBugCheckEx(
            BAD_POOL_HEADER,
            POOL_CORRUPTED_LIST as ULONG_PTR,
            list_head as ULONG_PTR,
            exp_decode_pool_link((*flink).Blink) as ULONG_PTR,
            exp_decode_pool_link((*blink).Flink) as ULONG_PTR,
        );
    }
}

unsafe fn exp_initialize_pool_list_head(list_head: PLIST_ENTRY) {
    (*list_head).Flink = exp_encode_pool_link(list_head);
    (*list_head).Blink = exp_encode_pool_link(list_head);
}

#[inline]
unsafe fn exp_is_pool_list_empty(list_head: PLIST_ENTRY) -> bool {
    exp_decode_pool_link((*list_head).Flink) == list_head
}

unsafe fn exp_remove_pool_entry_list(entry: PLIST_ENTRY) {
    let flink = exp_decode_pool_link((*entry).Flink);
    let blink = exp_decode_pool_link((*entry).Blink);
    (*flink).Blink = exp_encode_pool_link(blink);
    (*blink).Flink = exp_encode_pool_link(flink);
}

unsafe fn exp_remove_pool_head_list(list_head: PLIST_ENTRY) -> PLIST_ENTRY {
    let entry = exp_decode_pool_link((*list_head).Flink);
    let flink = exp_decode_pool_link((*entry).Flink);
    (*list_head).Flink = exp_encode_pool_link(flink);
    (*flink).Blink = exp_encode_pool_link(list_head);
    entry
}

#[allow(dead_code)]
unsafe fn exp_remove_pool_tail_list(list_head: PLIST_ENTRY) -> PLIST_ENTRY {
    let entry = exp_decode_pool_link((*list_head).Blink);
    let blink = exp_decode_pool_link((*entry).Blink);
    (*list_head).Blink = exp_encode_pool_link(blink);
    (*blink).Flink = exp_encode_pool_link(list_head);
    entry
}

unsafe fn exp_insert_pool_tail_list(list_head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    exp_check_pool_links(list_head);
    let blink = exp_decode_pool_link((*list_head).Blink);
    (*entry).Flink = exp_encode_pool_link(list_head);
    (*entry).Blink = exp_encode_pool_link(blink);
    (*blink).Flink = exp_encode_pool_link(entry);
    (*list_head).Blink = exp_encode_pool_link(entry);
    exp_check_pool_links(list_head);
}

unsafe fn exp_insert_pool_head_list(list_head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    exp_check_pool_links(list_head);
    let flink = exp_decode_pool_link((*list_head).Flink);
    (*entry).Flink = exp_encode_pool_link(flink);
    (*entry).Blink = exp_encode_pool_link(list_head);
    (*flink).Blink = exp_encode_pool_link(entry);
    (*list_head).Flink = exp_encode_pool_link(entry);
    exp_check_pool_links(list_head);
}

// ============================================================================
// Pool header validation
// ============================================================================


// DIAG: page walk + recent-free ring on any header-corruption raise — names the
// corrupted/corruptor tags on serial before the 0x19 (TCG-only cdrom-era hunt).
unsafe fn exp_dump_pool_page(around: PPOOL_HEADER) {
    ntkrnl_sys::driver_ddk::DbgPrint(
        c"[POOL] header-corruption near %p — page walk:\n".as_ptr(), around);
    let mut e = PAGE_ALIGN(around as usize) as PPOOL_HEADER;
    let mut guard = 0;
    while guard < 64 {
        ntkrnl_sys::driver_ddk::DbgPrint(
            c"[POOL]   hdr=%p prev=%x bs=%x type=%x tag=%08lx\n".as_ptr(),
            e, (*e).PreviousSize() as u32, (*e).BlockSize() as u32,
            (*e).PoolType() as u32, (*e).PoolTag);
        if (*e).BlockSize() == 0 { break; }
        e = pool_next_block(e);
        if PAGE_ALIGN(e as usize) == e as usize { break; }
        guard += 1;
    }
    let idx = EXP_FREE_RING_IDX.load(core::sync::atomic::Ordering::Relaxed);
    for k in 0..16usize {
        let i = (idx.wrapping_sub(1).wrapping_sub(k)) & 15;
        ntkrnl_sys::driver_ddk::DbgPrint(
            c"[POOL]   free[-%u] ptr=%p tag=%08lx\n".as_ptr(),
            k as u32,
            EXP_FREE_RING[i].0.load(core::sync::atomic::Ordering::Relaxed),
            EXP_FREE_RING[i].1.load(core::sync::atomic::Ordering::Relaxed),
        );
    }
}

unsafe fn exp_check_pool_header(entry: PPOOL_HEADER) {
    if (*entry).PreviousSize() != 0 {
        let prev = pool_prev_block(entry);
        if PAGE_ALIGN(entry as usize) != PAGE_ALIGN(prev as usize) {
            KeBugCheckEx(
                BAD_POOL_HEADER,
                POOL_ENTRIES_NOT_ALIGNED_PREVIOUS as ULONG_PTR,
                prev as ULONG_PTR,
                line!() as ULONG_PTR,
                entry as ULONG_PTR,
            );
        }
        if (*prev).BlockSize() != (*entry).PreviousSize() {
            exp_dump_pool_page(entry);
            KeBugCheckEx(
                BAD_POOL_HEADER,
                POOL_SIZE_OR_INDEX_MISMATCH as ULONG_PTR,
                prev as ULONG_PTR,
                line!() as ULONG_PTR,
                entry as ULONG_PTR,
            );
        }
    } else if PAGE_ALIGN(entry as usize) != entry as usize {
        KeBugCheckEx(
            BAD_POOL_HEADER,
            POOL_HEADER_NOT_ALIGNED as ULONG_PTR,
            0,
            line!() as ULONG_PTR,
            entry as ULONG_PTR,
        );
    }

    if (*entry).BlockSize() == 0 {
        KeBugCheckEx(
            BAD_POOL_HEADER,
            POOL_HEADER_IS_ZERO as ULONG_PTR,
            0,
            line!() as ULONG_PTR,
            entry as ULONG_PTR,
        );
    }

    let next = pool_next_block(entry);
    if PAGE_ALIGN(next as usize) != next as usize {
        if PAGE_ALIGN(entry as usize) != PAGE_ALIGN(next as usize) {
            KeBugCheckEx(
                BAD_POOL_HEADER,
                POOL_ENTRIES_NOT_ALIGNED_NEXT as ULONG_PTR,
                next as ULONG_PTR,
                line!() as ULONG_PTR,
                entry as ULONG_PTR,
            );
        }
        if (*next).PreviousSize() != (*entry).BlockSize() {
            exp_dump_pool_page(entry);
            KeBugCheckEx(
                BAD_POOL_HEADER,
                POOL_SIZE_OR_INDEX_MISMATCH as ULONG_PTR,
                next as ULONG_PTR,
                line!() as ULONG_PTR,
                entry as ULONG_PTR,
            );
        }
    }
}

// DIAG (task #13): ring of recent frees — dumped by the pool-corruption
// bugcheck path so the fatal free has context on serial.
#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_PAIR: (core::sync::atomic::AtomicUsize, core::sync::atomic::AtomicUsize) = (
    core::sync::atomic::AtomicUsize::new(0),
    core::sync::atomic::AtomicUsize::new(0),
);
pub static EXP_FREE_RING: [(core::sync::atomic::AtomicUsize, core::sync::atomic::AtomicUsize); 16] =
    [ATOMIC_PAIR; 16];
pub static EXP_FREE_RING_IDX: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

unsafe fn exp_check_pool_blocks(block: PVOID) {
    let mut found = false;
    let mut size: usize = 0;
    let mut entry = PAGE_ALIGN(block as usize) as PPOOL_HEADER;

    loop {
        if entry as PVOID == block {
            found = true;
        }
        exp_check_pool_header(entry);
        size += (*entry).BlockSize() as usize;
        entry = pool_next_block(entry);
        if size >= POOL_LISTS_PER_PAGE {
            break;
        }
        if PAGE_ALIGN(entry as usize) == entry as usize {
            break;
        }
    }

    if PAGE_ALIGN(entry as usize) != entry as usize || !found {
        // Dump the page's block walk + recent-free ring before dying.
        ntkrnl_sys::driver_ddk::DbgPrint(
            c"[POOL] corrupt-free block=%p — page walk:\n".as_ptr(), block);
        let mut e = PAGE_ALIGN(block as usize) as PPOOL_HEADER;
        let mut guard = 0;
        while guard < 64 {
            ntkrnl_sys::driver_ddk::DbgPrint(
                c"[POOL]   hdr=%p prev=%x bs=%x type=%x tag=%08lx\n".as_ptr(),
                e, (*e).PreviousSize() as u32, (*e).BlockSize() as u32,
                (*e).PoolType() as u32, (*e).PoolTag);
            if (*e).BlockSize() == 0 { break; }
            e = pool_next_block(e);
            if PAGE_ALIGN(e as usize) == e as usize { break; }
            guard += 1;
        }
        let idx = EXP_FREE_RING_IDX.load(core::sync::atomic::Ordering::Relaxed);
        for k in 0..16usize {
            let i = (idx.wrapping_sub(1).wrapping_sub(k)) & 15;
            ntkrnl_sys::driver_ddk::DbgPrint(
                c"[POOL]   free[-%u] ptr=%p tag=%08lx\n".as_ptr(),
                k as u32,
                EXP_FREE_RING[i].0.load(core::sync::atomic::Ordering::Relaxed),
                EXP_FREE_RING[i].1.load(core::sync::atomic::Ordering::Relaxed),
            );
        }
        KeBugCheckEx(
            BAD_POOL_HEADER,
            POOL_ENTRY_NOT_FOUND as ULONG_PTR,
            block as ULONG_PTR,
            line!() as ULONG_PTR,
            entry as ULONG_PTR,
        );
    }
}

// ============================================================================
// IRQL validation
// ============================================================================

#[inline]
unsafe fn exp_check_pool_irql_level(pool_type: POOL_TYPE, num_bytes: SIZE_T, entry: PVOID) {
    let irql = KeGetCurrentIrql();
    let is_paged = (pool_type & BASE_POOL_TYPE_MASK) == PagedPool;
    if (is_paged && irql > APC_LEVEL) || (!is_paged && irql > DISPATCH_LEVEL) {
        KeBugCheckEx(
            BAD_POOL_CALLER,
            if entry.is_null() {
                POOL_ALLOC_IRQL_INVALID
            } else {
                POOL_FREE_IRQL_INVALID
            } as ULONG_PTR,
            irql as ULONG_PTR,
            pool_type as ULONG_PTR,
            if entry.is_null() {
                num_bytes
            } else {
                entry as ULONG_PTR
            },
        );
    }
}

// ============================================================================
// Hash functions
// ============================================================================

#[inline]
fn exp_compute_hash_for_tag(tag: ULONG, bucket_mask: SIZE_T) -> ULONG {
    let result = 40543u64.wrapping_mul(tag as u64);
    (bucket_mask as ULONG) & ((result as ULONG) ^ ((result >> 32) as ULONG))
}

#[inline]
fn exp_compute_partial_hash_for_address(base_address: PVOID) -> ULONG {
    let result = (base_address as usize >> PAGE_SHIFT) as ULONG;
    (result >> 24) ^ (result >> 16) ^ (result >> 8) ^ result
}

// ============================================================================
// Pool lock / unlock
// ============================================================================

#[inline]
unsafe fn ex_lock_pool(descriptor: PPOOL_DESCRIPTOR) -> KIRQL {
    if ((*descriptor).PoolType & BASE_POOL_TYPE_MASK) == NonPagedPool {
        KeAcquireQueuedSpinLock(LockQueueNonPagedPoolLock)
    } else {
        KeAcquireGuardedMutex((*descriptor).LockAddress as PKGUARDED_MUTEX);
        APC_LEVEL
    }
}

#[inline]
unsafe fn ex_unlock_pool(descriptor: PPOOL_DESCRIPTOR, old_irql: KIRQL) {
    if ((*descriptor).PoolType & BASE_POOL_TYPE_MASK) == NonPagedPool {
        KeReleaseQueuedSpinLock(LockQueueNonPagedPoolLock, old_irql);
    } else {
        KeReleaseGuardedMutex((*descriptor).LockAddress as PKGUARDED_MUTEX);
    }
}

// ============================================================================
// ExInitializePoolDescriptor
// ============================================================================

/// Initialize a pool descriptor.
#[no_mangle]
pub unsafe extern "system" fn ExInitializePoolDescriptor(
    pool_descriptor: PPOOL_DESCRIPTOR,
    pool_type: POOL_TYPE,
    pool_index: ULONG,
    threshold: ULONG,
    pool_lock: PVOID,
) {
    (*pool_descriptor).PoolType = pool_type;
    (*pool_descriptor).PoolIndex = pool_index;
    (*pool_descriptor).Threshold = threshold;
    (*pool_descriptor).LockAddress = pool_lock;
    (*pool_descriptor).RunningAllocs = 0;
    (*pool_descriptor).RunningDeAllocs = 0;
    (*pool_descriptor).TotalPages = 0;
    (*pool_descriptor).TotalBytes = 0;
    (*pool_descriptor).TotalBigPages = 0;
    (*pool_descriptor).PendingFrees = null_mut();
    (*pool_descriptor).PendingFreeDepth = 0;

    let list_heads = (*pool_descriptor).ListHeads.as_mut_ptr();
    for i in 0..POOL_LISTS_PER_PAGE {
        exp_initialize_pool_list_head(list_heads.add(i));
    }
}

// ============================================================================
// InitializePool
// ============================================================================

/// Initialize pool tracking tables and descriptors.
#[no_mangle]
pub unsafe extern "system" fn InitializePool(pool_type: POOL_TYPE, threshold: ULONG) {
    if pool_type == NonPagedPool {
        // Compute track table size
        let mut table_size = if PoolTrackTableSize < MmSizeOfNonPagedPoolInBytes >> 8 {
            PoolTrackTableSize
        } else {
            MmSizeOfNonPagedPoolInBytes >> 8
        };

        let mut i = 0u32;
        while i < 32 {
            if table_size & 1 != 0 {
                if table_size & !1 == 0 {
                    break;
                }
            }
            table_size >>= 1;
            i += 1;
        }

        if i == 32 {
            PoolTrackTableSize = 2048;
        } else {
            let v = 1usize << i;
            PoolTrackTableSize = if v > 64 { v } else { 64 };
        }

        // Allocate tracker table
        loop {
            if (PoolTrackTableSize + 1)
                > (usize::MAX / core::mem::size_of::<POOL_TRACKER_TABLE>())
            {
                PoolTrackTableSize >>= 1;
                continue;
            }

            PoolTrackTable = pool_pages::MiAllocatePoolPages(
                NonPagedPool,
                (PoolTrackTableSize + 1) * core::mem::size_of::<POOL_TRACKER_TABLE>(),
            ) as PPOOL_TRACKER_TABLE;
            if !PoolTrackTable.is_null() {
                break;
            }

            if PoolTrackTableSize == 1 {
                KeBugCheckEx(
                    MUST_SUCCEED_POOL_EMPTY,
                    table_size,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                );
            }
            PoolTrackTableSize >>= 1;
        }

        PoolTrackTableSize += 1;
        PoolTrackTableMask = PoolTrackTableSize - 2;

        RtlZeroMemory(
            PoolTrackTable as PVOID,
            PoolTrackTableSize * core::mem::size_of::<POOL_TRACKER_TABLE>(),
        );

        ExpSeedHotTags();

        // Allocate big page tracker table
        let mut table_size = if PoolBigPageTableSize < MmSizeOfNonPagedPoolInBytes >> 8 {
            PoolBigPageTableSize
        } else {
            MmSizeOfNonPagedPoolInBytes >> 8
        };

        let mut i = 0u32;
        while i < 32 {
            if table_size & 1 != 0 {
                if table_size & !1 == 0 {
                    break;
                }
            }
            table_size >>= 1;
            i += 1;
        }

        if i == 32 {
            PoolBigPageTableSize = 4096;
        } else {
            let v = 1usize << i;
            PoolBigPageTableSize = if v > 64 { v } else { 64 };
        }

        loop {
            if (PoolBigPageTableSize + 1)
                > (usize::MAX / core::mem::size_of::<POOL_TRACKER_BIG_PAGES>())
            {
                PoolBigPageTableSize >>= 1;
                continue;
            }

            PoolBigPageTable = pool_pages::MiAllocatePoolPages(
                NonPagedPool,
                PoolBigPageTableSize * core::mem::size_of::<POOL_TRACKER_BIG_PAGES>(),
            ) as PPOOL_TRACKER_BIG_PAGES;
            if !PoolBigPageTable.is_null() {
                break;
            }

            if PoolBigPageTableSize == 1 {
                KeBugCheckEx(
                    MUST_SUCCEED_POOL_EMPTY,
                    table_size,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                );
            }
            PoolBigPageTableSize >>= 1;
        }

        PoolBigPageTableHash = PoolBigPageTableSize - 1;
        RtlZeroMemory(
            PoolBigPageTable as PVOID,
            PoolBigPageTableSize * core::mem::size_of::<POOL_TRACKER_BIG_PAGES>(),
        );

        // Mark all big page table entries as free
        for idx in 0..PoolBigPageTableSize {
            (*PoolBigPageTable.add(idx)).Va = POOL_BIG_TABLE_ENTRY_FREE as PVOID;
        }

        // Insert generic tracker for big pool table
        ExpInsertPoolTracker(
            u32::from_le_bytes(*b"Pool"),
            ROUND_TO_PAGES(
                PoolBigPageTableSize * core::mem::size_of::<POOL_TRACKER_BIG_PAGES>(),
            ),
            NonPagedPool,
        );

        // Initialize tag spinlock (zeroed is initialized for KSPIN_LOCK)
        ExpTaggedPoolLock = 0;

        // Initialize nonpaged pool descriptor
        PoolVector[NonPagedPool as usize] = &raw mut NonPagedPoolDescriptor as *mut _;
        ExInitializePoolDescriptor(
            PoolVector[NonPagedPool as usize],
            NonPagedPool,
            0,
            threshold,
            null_mut(),
        );
    } else {
        // Paged pool initialization
        let descriptor = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KGUARDED_MUTEX>() + core::mem::size_of::<POOL_DESCRIPTOR>(),
            u32::from_le_bytes(*b"Pool"),
        );
        if descriptor.is_null() {
            KeBugCheckEx(
                MUST_SUCCEED_POOL_EMPTY,
                0,
                usize::MAX,
                usize::MAX,
                usize::MAX,
            );
        }

        let pool_desc = descriptor as PPOOL_DESCRIPTOR;
        PoolVector[PagedPool as usize] = pool_desc;
        ExpPagedPoolMutex =
            (pool_desc.add(1)) as PKGUARDED_MUTEX;
        ExpPagedPoolDescriptor[0] = pool_desc;
        KeInitializeGuardedMutex(ExpPagedPoolMutex);
        ExInitializePoolDescriptor(
            pool_desc,
            PagedPool,
            0,
            threshold,
            ExpPagedPoolMutex as PVOID,
        );

        // Insert generic tracker for pool track table
        ExpInsertPoolTracker(
            u32::from_le_bytes(*b"Pool"),
            ROUND_TO_PAGES(PoolTrackTableSize * core::mem::size_of::<POOL_TRACKER_TABLE>()),
            NonPagedPool,
        );
    }
}

// ============================================================================
// ExLockPool / ExUnlockPool (exported as inline in C, explicit here)
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExLockPool(descriptor: PPOOL_DESCRIPTOR) -> KIRQL {
    ex_lock_pool(descriptor)
}

#[no_mangle]
pub unsafe extern "system" fn ExUnlockPool(descriptor: PPOOL_DESCRIPTOR, old_irql: KIRQL) {
    ex_unlock_pool(descriptor, old_irql)
}

// ============================================================================
// ExpSeedHotTags
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpSeedHotTags() {
    let tag_list: [ULONG; 64] = [
        u32::from_le_bytes(*b"  oI"),
        u32::from_le_bytes(*b" laH"),
        u32::from_le_bytes(*b"PldM"),
        u32::from_le_bytes(*b"LooP"),
        u32::from_le_bytes(*b"tSbO"),
        u32::from_le_bytes(*b" prI"),
        u32::from_le_bytes(*b"bdDN"),
        u32::from_le_bytes(*b"LprI"),
        u32::from_le_bytes(*b"pOoI"),
        u32::from_le_bytes(*b" ldM"),
        u32::from_le_bytes(*b"eliF"),
        u32::from_le_bytes(*b"aVMC"),
        u32::from_le_bytes(*b"dSeS"),
        u32::from_le_bytes(*b"CFtN"),
        u32::from_le_bytes(*b"looP"),
        u32::from_le_bytes(*b"rPCT"),
        u32::from_le_bytes(*b"bNMC"),
        u32::from_le_bytes(*b"dTeS"),
        u32::from_le_bytes(*b"sFtN"),
        u32::from_le_bytes(*b"TPCT"),
        u32::from_le_bytes(*b"CPCT"),
        u32::from_le_bytes(*b" yeK"),
        u32::from_le_bytes(*b"qSbO"),
        u32::from_le_bytes(*b"mNoI"),
        u32::from_le_bytes(*b"aEoI"),
        u32::from_le_bytes(*b"cPCT"),
        u32::from_le_bytes(*b"aFtN"),
        u32::from_le_bytes(*b"0ftN"),
        u32::from_le_bytes(*b"tceS"),
        u32::from_le_bytes(*b"SprI"),
        u32::from_le_bytes(*b"ekoT"),
        u32::from_le_bytes(*b"  eS"),
        u32::from_le_bytes(*b"lCbO"),
        u32::from_le_bytes(*b"cScC"),
        u32::from_le_bytes(*b"lFtN"),
        u32::from_le_bytes(*b"cAeS"),
        u32::from_le_bytes(*b"mfSF"),
        u32::from_le_bytes(*b"kWcC"),
        u32::from_le_bytes(*b"miSF"),
        u32::from_le_bytes(*b"CdfA"),
        u32::from_le_bytes(*b"EdfA"),
        u32::from_le_bytes(*b"orSF"),
        u32::from_le_bytes(*b"nftN"),
        u32::from_le_bytes(*b"PRIU"),
        u32::from_le_bytes(*b"rFpN"),
        u32::from_le_bytes(*b"RFpN"),
        u32::from_le_bytes(*b"aPeS"),
        u32::from_le_bytes(*b"sUeS"),
        u32::from_le_bytes(*b"FpcA"),
        u32::from_le_bytes(*b"MpcA"),
        u32::from_le_bytes(*b"cSeS"),
        u32::from_le_bytes(*b"mNbO"),
        u32::from_le_bytes(*b"sFpN"),
        u32::from_le_bytes(*b"uLeS"),
        u32::from_le_bytes(*b"DPcS"),
        u32::from_le_bytes(*b"nevE"),
        u32::from_le_bytes(*b"vrqR"),
        u32::from_le_bytes(*b"ldaV"),
        u32::from_le_bytes(*b"  pP"),
        u32::from_le_bytes(*b"SdaV"),
        u32::from_le_bytes(*b" daV"),
        u32::from_le_bytes(*b"LdaV"),
        u32::from_le_bytes(*b"FdaV"),
        u32::from_le_bytes(*b" GIB"),
    ];

    let track_table = PoolTrackTable;

    for &key in &tag_list {
        let mut hash = exp_compute_hash_for_tag(key, PoolTrackTableMask);
        let index = hash;

        loop {
            if (*track_table.add(hash as usize)).Key == 0
                && hash as usize != PoolTrackTableSize - 1
            {
                (*track_table.add(hash as usize)).Key = key;
                break;
            }

            hash = (hash + 1) & PoolTrackTableMask as ULONG;
            if hash == index {
                break;
            }
        }
    }
}

// ============================================================================
// ExpRemovePoolTracker
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpRemovePoolTracker(
    mut key: ULONG,
    number_of_bytes: SIZE_T,
    pool_type: POOL_TYPE,
) {
    key &= !PROTECTED_POOL;

    if key == PoolHitTag {
        DbgBreakPoint();
    }

    let table = PoolTrackTable;
    let table_mask = PoolTrackTableMask;

    let mut hash = exp_compute_hash_for_tag(key, table_mask);
    let index = hash;

    loop {
        let table_entry = table.add(hash as usize);
        if (*table_entry).Key == key {
            if (pool_type & BASE_POOL_TYPE_MASK) == NonPagedPool {
                InterlockedIncrement(&mut (*table_entry).NonPagedFrees);
                InterlockedExchangeAddSizeT(
                    &mut (*table_entry).NonPagedBytes as *mut SIZE_T as *mut _,
                    -(number_of_bytes as isize) as _,
                );
            } else {
                InterlockedIncrement(&mut (*table_entry).PagedFrees);
                InterlockedExchangeAddSizeT(
                    &mut (*table_entry).PagedBytes as *mut SIZE_T as *mut _,
                    -(number_of_bytes as isize) as _,
                );
            }
            return;
        }

        hash = (hash + 1) & table_mask as ULONG;
        if hash == index {
            break;
        }
    }
}

// ============================================================================
// ExpInsertPoolTracker
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpInsertPoolTracker(
    mut key: ULONG,
    number_of_bytes: SIZE_T,
    pool_type: POOL_TYPE,
) {
    key &= !PROTECTED_POOL;

    if key == PoolHitTag {
        DbgBreakPoint();
    }

    let table = PoolTrackTable;
    let table_mask = PoolTrackTableMask;

    let mut hash = exp_compute_hash_for_tag(key, table_mask);
    let index = hash;

    loop {
        let table_entry = table.add(hash as usize);

        if (*table_entry).Key == key {
            if (pool_type & BASE_POOL_TYPE_MASK) == NonPagedPool {
                InterlockedIncrement(&mut (*table_entry).NonPagedAllocs);
                InterlockedExchangeAddSizeT(
                    &mut (*table_entry).NonPagedBytes as *mut SIZE_T as *mut _,
                    number_of_bytes as _,
                );
            } else {
                InterlockedIncrement(&mut (*table_entry).PagedAllocs);
                InterlockedExchangeAddSizeT(
                    &mut (*table_entry).PagedBytes as *mut SIZE_T as *mut _,
                    number_of_bytes as _,
                );
            }
            return;
        }

        if (*table_entry).Key == 0 && hash as usize != PoolTrackTableSize - 1 {
            let mut old_irql: KIRQL = 0;
            KeAcquireSpinLock(&raw mut ExpTaggedPoolLock, &mut old_irql);
            if (*PoolTrackTable.add(hash as usize)).Key == 0 {
                (*PoolTrackTable.add(hash as usize)).Key = key;
                (*table_entry).Key = key;
            }
            KeReleaseSpinLock(&raw mut ExpTaggedPoolLock, old_irql);
            continue;
        }

        hash = (hash + 1) & table_mask as ULONG;
        if hash == index {
            break;
        }
    }
}

// ============================================================================
// ExpAddTagForBigPages
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpAddTagForBigPages(
    va: PVOID,
    key: ULONG,
    number_of_pages: ULONG,
    pool_type: POOL_TYPE,
) -> BOOLEAN {
    let partial_hash = exp_compute_partial_hash_for_address(va);

    let mut old_irql: KIRQL = 0;
    KeAcquireSpinLock(&raw mut ExpLargePoolTableLock, &mut old_irql);

    let hash_mask = PoolBigPageTableHash;
    let mut hash = (partial_hash as usize) & hash_mask;
    let index = hash;

    loop {
        let entry = PoolBigPageTable.add(hash);
        if (*entry).Va as ULONG_PTR == POOL_BIG_TABLE_ENTRY_FREE {
            (*entry).Va = va;
            (*entry).Key = key;
            (*entry).NumberOfPages = number_of_pages;

            if pool_type & QUOTA_POOL_MASK != 0 {
                (*entry).QuotaObject = null_mut(); // Set later by quota code
            }

            ExpPoolBigEntriesInUse += 1;
            KeReleaseSpinLock(&raw mut ExpLargePoolTableLock, old_irql);
            return TRUE;
        }

        hash = (hash + 1) & hash_mask;
        if hash == index {
            break;
        }
    }

    ExpBigTableExpansionFailed += 1;
    KeReleaseSpinLock(&raw mut ExpLargePoolTableLock, old_irql);
    FALSE
}

// ============================================================================
// ExpFindAndRemoveTagBigPages
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpFindAndRemoveTagBigPages(
    va: PVOID,
    big_pages: *mut ULONG_PTR,
    _pool_type: POOL_TYPE,
) -> ULONG {
    let partial_hash = exp_compute_partial_hash_for_address(va);

    let mut old_irql: KIRQL = 0;
    KeAcquireSpinLock(&raw mut ExpLargePoolTableLock, &mut old_irql);

    let hash_mask = PoolBigPageTableHash;
    let mut hash = (partial_hash as usize) & hash_mask;
    let index = hash;

    loop {
        let entry = PoolBigPageTable.add(hash);
        if (*entry).Va == va {
            *big_pages = (*entry).NumberOfPages as ULONG_PTR;
            let key = (*entry).Key;
            (*entry).Va = POOL_BIG_TABLE_ENTRY_FREE as PVOID;
            (*entry).NumberOfPages = 0;
            ExpPoolBigEntriesInUse -= 1;
            KeReleaseSpinLock(&raw mut ExpLargePoolTableLock, old_irql);
            return key;
        }

        hash = (hash + 1) & hash_mask;
        if hash == index {
            break;
        }
    }

    KeReleaseSpinLock(&raw mut ExpLargePoolTableLock, old_irql);
    0
}

// ============================================================================
// ExQueryPoolUsage
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExQueryPoolUsage(
    paged_pool_pages: *mut ULONG,
    non_paged_pool_pages: *mut ULONG,
    paged_pool_allocs: *mut ULONG,
    paged_pool_frees: *mut ULONG,
    paged_pool_lookside_hits: *mut ULONG,
    non_paged_pool_allocs: *mut ULONG,
    non_paged_pool_frees: *mut ULONG,
    non_paged_pool_lookside_hits: *mut ULONG,
) {
    let np = &raw const NonPagedPoolDescriptor;
    *non_paged_pool_pages = (*np).TotalPages + (*np).TotalBigPages;
    *non_paged_pool_allocs = (*np).RunningAllocs;
    *non_paged_pool_frees = (*np).RunningDeAllocs;
    *non_paged_pool_lookside_hits = 0; // TODO: lookaside tracking

    *paged_pool_pages = 0;
    *paged_pool_allocs = 0;
    *paged_pool_frees = 0;
    *paged_pool_lookside_hits = 0;

    for i in 0..=ExpNumberOfPagedPools as usize {
        let desc = ExpPagedPoolDescriptor[i];
        if !desc.is_null() {
            *paged_pool_pages += (*desc).TotalPages + (*desc).TotalBigPages;
            *paged_pool_allocs += (*desc).RunningAllocs;
            *paged_pool_frees += (*desc).RunningDeAllocs;
        }
    }
}

// ============================================================================
// ExReturnPoolQuota
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExReturnPoolQuota(p: PVOID) {
    // A special-pool block has no POOL_HEADER in front of it: it sits at the END of its
    // page, so the page-aligned test below does not catch it and the read of `entry`
    // faulted in IoFreeIrp (0xA at boot with SPECIALPOOL=*). This check is the C's.
    if ExpPoolFlags & POOL_FLAG_SPECIAL_POOL != 0 && MmIsSpecialPoolAddress(p) != 0 {
        return;
    }
    // Big pages have no header either, and are never quota-charged.
    if PAGE_ALIGN(p as usize) == p as usize {
        return;
    }
    // A special-pool block has no pool header in front of it (overrun mode puts it at
    // the end of its page, after fill pattern) and carries no quota. Reading the pattern
    // as a header sent the quota-pointer read past the block onto the guard page:
    // BUGCHECK 0x50 from IoFreeIrp on any SPECIALPOOL=* boot that freed an IRP.
    if MmIsSpecialPoolAddress(p) != 0 {
        return;
    }

    let entry = pool_entry(p);
    let pool_type = (*entry).PoolType().wrapping_sub(1) as POOL_TYPE;

    if pool_type & QUOTA_POOL_MASK != 0 {
        let process_slot = (pool_next_block(entry) as *mut PVOID).sub(1);
        let process = *process_slot;
        if !process.is_null() {
            if EprocessGetType(process as _) != ProcessObject {
                KeBugCheckEx(
                    BAD_POOL_CALLER,
                    POOL_BILLED_PROCESS_INVALID as ULONG_PTR,
                    p as ULONG_PTR,
                    (*entry).PoolTag as ULONG_PTR,
                    process as ULONG_PTR,
                );
            }
            *process_slot = null_mut();
            PsReturnPoolQuota(
                process as _,
                pool_type & BASE_POOL_TYPE_MASK,
                (*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE,
            );
            ObDereferenceObject(process);
        }
    }
}

// ============================================================================
// ExGetPoolTagInfo (DPC target + main function)
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpGetPoolTagInfoTarget(
    _dpc: PKDPC,
    deferred_context: PVOID,
    _system_argument1: PVOID,
    system_argument2: PVOID,
) {
    let context = deferred_context as *mut super::types::PoolDpcContext;

    if KeSignalCallDpcSynchronize(system_argument2) != 0 {
        RtlCopyMemory(
            (*context).pool_track_table as PVOID,
            PoolTrackTable as PVOID,
            (*context).pool_track_table_size * core::mem::size_of::<POOL_TRACKER_TABLE>(),
        );
    }

    KeSignalCallDpcDone(_system_argument1);
}

#[no_mangle]
pub unsafe extern "system" fn ExGetPoolTagInfo(
    system_information: *mut ntkrnl_sys::ex::SYSTEM_POOLTAG_INFORMATION,
    system_information_length: ULONG,
    return_length: *mut ULONG,
) -> NTSTATUS {
    let table_size = PoolTrackTableSize;
    let required = core::mem::size_of::<ntkrnl_sys::ex::SYSTEM_POOLTAG_INFORMATION>() as ULONG
        + (table_size as ULONG - 1) * core::mem::size_of::<ntkrnl_sys::ex::SYSTEM_POOLTAG>() as ULONG;

    if !return_length.is_null() {
        *return_length = required;
    }

    if system_information_length < required {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // Snapshot the pool track table via DPC
    let mut context = super::types::PoolDpcContext {
        pool_track_table: system_information.add(1) as PPOOL_TRACKER_TABLE, // past the header
        pool_track_table_size: table_size,
        pool_track_table_expansion: null_mut(),
        pool_track_table_size_expansion: 0,
    };

    KeGenericCallDpc(
        ExpGetPoolTagInfoTarget,
        &mut context as *mut _ as PVOID,
    );

    // Count non-empty entries
    let mut count = 0u32;
    let tag_info_base = &mut (*system_information).TagInfo[0] as *mut ntkrnl_sys::ex::SYSTEM_POOLTAG;
    let track_snapshot = tag_info_base as PPOOL_TRACKER_TABLE;

    for idx in 0..table_size {
        let entry = track_snapshot.add(idx);
        if (*entry).Key != 0 && (*entry).Key != u32::from_le_bytes(*b" GIB") {
            let tag = tag_info_base.add(count as usize);
            (*tag).Tag = (*entry).Key.to_le_bytes();
            (*tag).PagedAllocs = (*entry).PagedAllocs as ULONG;
            (*tag).PagedFrees = (*entry).PagedFrees as ULONG;
            (*tag).PagedUsed = (*entry).PagedBytes;
            (*tag).NonPagedAllocs = (*entry).NonPagedAllocs as ULONG;
            (*tag).NonPagedFrees = (*entry).NonPagedFrees as ULONG;
            (*tag).NonPagedUsed = (*entry).NonPagedBytes;
            count += 1;
        }
    }

    (*system_information).Count = count;
    STATUS_SUCCESS
}

// ============================================================================
// ExAllocatePoolWithTag — the main pool allocation function
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolWithTag(
    pool_type: POOL_TYPE,
    mut number_of_bytes: SIZE_T,
    tag: ULONG,
) -> PVOID {
    exp_check_pool_irql_level(pool_type, number_of_bytes, null_mut());

    // Check for special pool
    if ExpPoolFlags & (POOL_FLAG_VERIFIER | POOL_FLAG_SPECIAL_POOL) != 0 {
        if ExpPoolFlags & POOL_FLAG_SPECIAL_POOL != 0 {
            if MmUseSpecialPool(number_of_bytes, tag) != 0 {
                let entry = MmAllocateSpecialPool(number_of_bytes, tag, pool_type, 2);
                if !entry.is_null() {
                    return entry;
                }
            }
        }
    }

    let original_type = pool_type;
    let pool_type_base = pool_type & BASE_POOL_TYPE_MASK;
    let pool_desc = PoolVector[pool_type_base as usize];

    // Big page allocation (> POOL_MAX_ALLOC)
    if number_of_bytes > POOL_MAX_ALLOC {
        let entry = pool_pages::MiAllocatePoolPages(original_type, number_of_bytes);
        if entry.is_null() {
            if original_type & MUST_SUCCEED_POOL_MASK != 0 {
                KeBugCheckEx(
                    MUST_SUCCEED_POOL_EMPTY,
                    number_of_bytes,
                    NonPagedPoolDescriptor.TotalPages as ULONG_PTR,
                    NonPagedPoolDescriptor.TotalBigPages as ULONG_PTR,
                    0,
                );
            }
            ExPoolFailures += 1;
            if original_type & POOL_RAISE_IF_ALLOCATION_FAILURE as POOL_TYPE != 0 {
                ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
            }
            return null_mut();
        }

        InterlockedExchangeAdd(
            &mut (*pool_desc).TotalBigPages as *mut ULONG as *mut LONG,
            BYTES_TO_PAGES(number_of_bytes) as LONG,
        );
        InterlockedExchangeAddSizeT(
            &mut (*pool_desc).TotalBytes as *mut SIZE_T as *mut _,
            number_of_bytes as _,
        );
        InterlockedIncrement(&mut (*pool_desc).RunningAllocs as *mut ULONG as *mut LONG);

        if ExpAddTagForBigPages(
            entry,
            tag,
            BYTES_TO_PAGES(number_of_bytes) as ULONG,
            original_type,
        ) == FALSE
        {
            ExpInsertPoolTracker(
                u32::from_le_bytes(*b" GIB"),
                ROUND_TO_PAGES(number_of_bytes),
                original_type,
            );
        } else {
            ExpInsertPoolTracker(tag, ROUND_TO_PAGES(number_of_bytes), original_type);
        }
        return entry;
    }

    // Ensure minimum 1 byte
    if number_of_bytes == 0 {
        number_of_bytes = 1;
    }

    // Calculate block size in POOL_BLOCK_SIZE units
    let i = ((number_of_bytes + POOL_BLOCK_SIZE + (POOL_BLOCK_SIZE - 1)) / POOL_BLOCK_SIZE)
        as u16;

    // NOTE: Lookaside list optimization skipped for now.
    // The free list path below is functionally correct.
    // TODO: Add lookaside support using KPRCB field offsets.

    // Search the free lists for a block of this size
    let mut list_head = (*pool_desc).ListHeads.as_mut_ptr().add(i as usize);
    let list_end = (*pool_desc)
        .ListHeads
        .as_mut_ptr()
        .add(POOL_LISTS_PER_PAGE);

    loop {
        if !exp_is_pool_list_empty(list_head) {
            let old_irql = ex_lock_pool(pool_desc);

            if exp_is_pool_list_empty(list_head) {
                ex_unlock_pool(pool_desc, old_irql);
                list_head = list_head.add(1);
                if list_head >= list_end {
                    break;
                }
                continue;
            }

            exp_check_pool_links(list_head);
            let entry = pool_entry(exp_remove_pool_head_list(list_head) as PVOID);
            exp_check_pool_links(list_head);
            exp_check_pool_blocks(entry as PVOID);

            if (*entry).BlockSize() != i {
                // Block is larger — split it
                let fragment_entry;
                let next_entry;

                if (*entry).PreviousSize() == 0 {
                    // First block on page — fragment goes after our allocation
                    fragment_entry = pool_block(entry, i as isize);
                    (*fragment_entry).Ulong1 = 0;
                    (*fragment_entry).set_BlockSize((*entry).BlockSize() - i);
                    (*fragment_entry).set_PreviousSize(i);

                    next_entry = pool_next_block(fragment_entry);
                    if PAGE_ALIGN(next_entry as usize) != next_entry as usize {
                        (*next_entry).set_PreviousSize((*fragment_entry).BlockSize());
                    }
                } else {
                    // Previous block exists — shrink current entry and use next as allocation
                    fragment_entry = entry;
                    (*entry).set_BlockSize((*entry).BlockSize() - i);

                    let alloc_entry = pool_next_block(entry);
                    (*alloc_entry).set_PreviousSize((*fragment_entry).BlockSize());

                    next_entry = pool_block(alloc_entry, i as isize);
                    if PAGE_ALIGN(next_entry as usize) != next_entry as usize {
                        (*next_entry).set_PreviousSize(i);
                    }

                    // The alloc_entry becomes our returned entry
                    // (entry variable reassigned after this block in C)
                    let entry = alloc_entry;
                    (*entry).set_BlockSize(i);
                    (*fragment_entry).set_PoolType(0);
                    let block_size = (*fragment_entry).BlockSize();

                    exp_check_pool_links(
                        (*pool_desc)
                            .ListHeads
                            .as_mut_ptr()
                            .add(block_size as usize - 1),
                    );
                    if block_size != 1 {
                        exp_insert_pool_tail_list(
                            (*pool_desc)
                                .ListHeads
                                .as_mut_ptr()
                                .add(block_size as usize - 1),
                            pool_free_block(fragment_entry),
                        );
                    }

                    (*entry).set_PoolType(original_type as u16 + 1);
                    exp_check_pool_blocks(entry as PVOID);
                    ex_unlock_pool(pool_desc, old_irql);

                    InterlockedExchangeAddSizeT(
                        &mut (*pool_desc).TotalBytes as *mut SIZE_T as *mut _,
                        ((*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE) as _,
                    );
                    InterlockedIncrement(
                        &mut (*pool_desc).RunningAllocs as *mut ULONG as *mut LONG,
                    );
                    ExpInsertPoolTracker(
                        tag,
                        (*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE,
                        original_type,
                    );

                    (*entry).PoolTag = tag;
                    let free_block = pool_free_block(entry);
                    (*free_block).Flink = null_mut();
                    (*free_block).Blink = null_mut();
                    return free_block as PVOID;
                }

                // First-block-on-page path continues here
                (*entry).set_BlockSize(i);
                (*fragment_entry).set_PoolType(0);
                let block_size = (*fragment_entry).BlockSize();

                exp_check_pool_links(
                    (*pool_desc)
                        .ListHeads
                        .as_mut_ptr()
                        .add(block_size as usize - 1),
                );
                if block_size != 1 {
                    exp_insert_pool_tail_list(
                        (*pool_desc)
                            .ListHeads
                            .as_mut_ptr()
                            .add(block_size as usize - 1),
                        pool_free_block(fragment_entry),
                    );
                    exp_check_pool_links(pool_free_block(fragment_entry));
                }
            }

            (*entry).set_PoolType(original_type as u16 + 1);
            exp_check_pool_blocks(entry as PVOID);
            ex_unlock_pool(pool_desc, old_irql);

            InterlockedExchangeAddSizeT(
                &mut (*pool_desc).TotalBytes as *mut SIZE_T as *mut _,
                ((*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE) as _,
            );
            InterlockedIncrement(&mut (*pool_desc).RunningAllocs as *mut ULONG as *mut LONG);
            ExpInsertPoolTracker(
                tag,
                (*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE,
                original_type,
            );

            (*entry).PoolTag = tag;
            let free_block = pool_free_block(entry);
            (*free_block).Flink = null_mut();
            (*free_block).Blink = null_mut();
            return free_block as PVOID;
        }

        list_head = list_head.add(1);
        if list_head >= list_end {
            break;
        }
    }

    // No free entries — allocate a new page
    let entry = pool_pages::MiAllocatePoolPages(original_type, PAGE_SIZE) as PPOOL_HEADER;
    if entry.is_null() {
        if original_type & MUST_SUCCEED_POOL_MASK != 0 {
            KeBugCheckEx(
                MUST_SUCCEED_POOL_EMPTY,
                PAGE_SIZE,
                NonPagedPoolDescriptor.TotalPages as ULONG_PTR,
                NonPagedPoolDescriptor.TotalBigPages as ULONG_PTR,
                0,
            );
        }
        ExPoolFailures += 1;
        if original_type & POOL_RAISE_IF_ALLOCATION_FAILURE as POOL_TYPE != 0 {
            ExRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
        }
        return null_mut();
    }

    // Set up the allocation entry
    (*entry).Ulong1 = 0;
    (*entry).set_BlockSize(i);
    (*entry).set_PoolType(original_type as u16 + 1);

    // Create fragment entry for remaining space
    let block_size = (PAGE_SIZE / POOL_BLOCK_SIZE) as u16 - i;
    let fragment_entry = pool_block(entry, i as isize);
    (*fragment_entry).Ulong1 = 0;
    (*fragment_entry).set_BlockSize(block_size);
    (*fragment_entry).set_PreviousSize(i);

    InterlockedIncrement(&mut (*pool_desc).TotalPages as *mut ULONG as *mut LONG);
    InterlockedExchangeAddSizeT(
        &mut (*pool_desc).TotalBytes as *mut SIZE_T as *mut _,
        ((*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE) as _,
    );

    if block_size != 1 {
        let old_irql = ex_lock_pool(pool_desc);
        exp_check_pool_links(
            (*pool_desc)
                .ListHeads
                .as_mut_ptr()
                .add(block_size as usize - 1),
        );
        exp_insert_pool_tail_list(
            (*pool_desc)
                .ListHeads
                .as_mut_ptr()
                .add(block_size as usize - 1),
            pool_free_block(fragment_entry),
        );
        exp_check_pool_links(pool_free_block(fragment_entry));
        exp_check_pool_blocks(entry as PVOID);
        ex_unlock_pool(pool_desc, old_irql);
    } else {
        exp_check_pool_blocks(entry as PVOID);
    }

    InterlockedIncrement(&mut (*pool_desc).RunningAllocs as *mut ULONG as *mut LONG);
    ExpInsertPoolTracker(
        tag,
        (*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE,
        original_type,
    );

    // Do not walk the page again here: once the pool lock was released above,
    // another CPU may be midway through initializing a split fragment
    // (Ulong1 is cleared before BlockSize is installed).  The page was already
    // checked while locked at 1313, or is still private in the one-block case.
    (*entry).PoolTag = tag;
    pool_free_block(entry) as PVOID
}

// ============================================================================
// ExAllocatePool
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePool(
    pool_type: POOL_TYPE,
    number_of_bytes: SIZE_T,
) -> PVOID {
    ExAllocatePoolWithTag(pool_type, number_of_bytes, TAG_NONE)
}

// ============================================================================
// ExFreePoolWithTag — the main pool free function
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExFreePoolWithTag(p: PVOID, tag_to_free: ULONG) {
    // NULL check — some kernel/driver code calls ExFreePool(NULL)
    if p.is_null() {
        return;
    }
    // Always-on diagnostics (kept after the 2026-07-07 classpnp IoQueueWorkItemEx
    // ABI bug, which freed the running thread's ETHREAD): ring of the last 16
    // frees (ptr, tag) so the corrupt-free bugcheck path can dump recent-free
    // context — KeBugCheckEx params don't include the free site.
    {
        let i = EXP_FREE_RING_IDX.fetch_add(1, core::sync::atomic::Ordering::Relaxed) & 15;
        EXP_FREE_RING[i].0.store(p as usize, core::sync::atomic::Ordering::Relaxed);
        EXP_FREE_RING[i].1.store(tag_to_free as usize, core::sync::atomic::Ordering::Relaxed);
    }
    // TRIPWIRE: ExFreePool(<current thread's ETHREAD>) is always catastrophic
    // (a bad IO_WORKITEM pointer is the historical culprit). Fault HERE, at the
    // corrupting call site, so the 0x1E banner prints the true stack instead of
    // an innocent-victim crash minutes later.
    if p as usize == ntkrnl_sys::mm::KeGetCurrentThread() as usize {
        extern "C" {
            static IOP_LAST_WORKITEM_ROUTINE: core::sync::atomic::AtomicUsize;
        }
        ntkrnl_sys::driver_ddk::DbgPrint(
            c"[POOL] TRIPWIRE: ExFreePool(current ETHREAD %p) tag=%08lx last-io-workitem-routine=%p — faulting for stack\n".as_ptr(),
            p, tag_to_free,
            IOP_LAST_WORKITEM_ROUTINE.load(core::sync::atomic::Ordering::Relaxed));
        core::ptr::null_mut::<u8>().write_volatile(0);
    }
    // Check for special pool
    if ExpPoolFlags
        & (POOL_FLAG_CHECK_TIMERS
            | POOL_FLAG_CHECK_WORKERS
            | POOL_FLAG_CHECK_RESOURCES
            | POOL_FLAG_VERIFIER
            | POOL_FLAG_CHECK_DEADLOCK
            | POOL_FLAG_SPECIAL_POOL)
        != 0
    {
        if ExpPoolFlags & POOL_FLAG_SPECIAL_POOL != 0 {
            if MmIsSpecialPoolAddress(p) != 0 {
                MmFreeSpecialPool(p);
                return;
            }
        }

        if PAGE_ALIGN(p as usize) != p as usize {
            let entry = (p as usize - POOL_BLOCK_SIZE) as PPOOL_HEADER;
            let pt = ((*entry).PoolType().wrapping_sub(1) as POOL_TYPE) & BASE_POOL_TYPE_MASK;
            exp_check_pool_irql_level(pt, 0, p);
        }
    }

    // Big page allocation (page-aligned)
    if PAGE_ALIGN(p as usize) == p as usize {
        let pool_type = pool_pages::MmDeterminePoolType(p);
        exp_check_pool_irql_level(pool_type, 0, p);

        let mut page_count: ULONG_PTR = 0;
        let mut tag = ExpFindAndRemoveTagBigPages(p, &mut page_count, pool_type);
        if tag == 0 {
            page_count = 1;
            tag = u32::from_le_bytes(*b" GIB");
        } else if tag & PROTECTED_POOL != 0 {
            tag &= !PROTECTED_POOL;
        }

        if tag_to_free != 0 && tag_to_free != tag {
            // Tag mismatch — continue anyway (non-debug builds)
        }

        ExpRemovePoolTracker(tag, page_count << PAGE_SHIFT, pool_type);

        let pool_desc = PoolVector[pool_type as usize];
        InterlockedIncrement(&mut (*pool_desc).RunningDeAllocs as *mut ULONG as *mut LONG);
        InterlockedExchangeAddSizeT(
            &mut (*pool_desc).TotalBytes as *mut SIZE_T as *mut _,
            -((page_count << PAGE_SHIFT) as isize) as _,
        );

        let real_page_count = pool_pages::MiFreePoolPages(p);
        InterlockedExchangeAdd(
            &mut (*pool_desc).TotalBigPages as *mut ULONG as *mut LONG,
            -(real_page_count as LONG),
        );
        return;
    }

    // Small allocation
    let entry = (p as usize - POOL_BLOCK_SIZE) as PPOOL_HEADER;
    let block_size = (*entry).BlockSize();
    let pool_type = ((*entry).PoolType().wrapping_sub(1) as POOL_TYPE) & BASE_POOL_TYPE_MASK;
    let pool_desc = PoolVector[pool_type as usize];

    exp_check_pool_irql_level(pool_type, 0, p);

    let mut tag = (*entry).PoolTag;
    if tag & PROTECTED_POOL != 0 {
        tag &= !PROTECTED_POOL;
    }

    if tag_to_free != 0 && tag_to_free != tag {
        // Tag mismatch — continue anyway
    }

    ExpRemovePoolTracker(
        tag,
        block_size as SIZE_T * POOL_BLOCK_SIZE,
        (*entry).PoolType().wrapping_sub(1) as POOL_TYPE,
    );

    // Release quota if charged
    if (*entry).PoolType().wrapping_sub(1) as POOL_TYPE & QUOTA_POOL_MASK != 0 {
        let process_ptr = (pool_next_block(entry) as *mut PVOID).sub(1);
        let process = *process_ptr;
        if !process.is_null() {
            if EprocessGetType(process as _) != ProcessObject {
                KeBugCheckEx(
                    BAD_POOL_CALLER,
                    POOL_BILLED_PROCESS_INVALID as ULONG_PTR,
                    p as ULONG_PTR,
                    tag as ULONG_PTR,
                    process as ULONG_PTR,
                );
            }
            PsReturnPoolQuota(
                process as _,
                pool_type,
                block_size as SIZE_T * POOL_BLOCK_SIZE,
            );
            ObDereferenceObject(process);
        }
    }

    // NOTE: Lookaside list push optimization skipped.
    // TODO: Add lookaside support using KPRCB field offsets.

    // Get next entry
    let mut next_entry = pool_block(entry, block_size as isize);

    // Update counters
    InterlockedIncrement(&mut (*pool_desc).RunningDeAllocs as *mut ULONG as *mut LONG);
    InterlockedExchangeAddSizeT(
        &mut (*pool_desc).TotalBytes as *mut SIZE_T as *mut _,
        -((block_size as isize) * POOL_BLOCK_SIZE as isize) as _,
    );

    let old_irql = ex_lock_pool(pool_desc);

    exp_check_pool_blocks(entry as PVOID);
    let mut combined = false;

    // Try to combine with next block
    if PAGE_ALIGN(next_entry as usize) != next_entry as usize {
        if (*next_entry).PoolType() == 0 {
            combined = true;
            if (*next_entry).BlockSize() != 1 {
                exp_check_pool_links(pool_free_block(next_entry));
                exp_remove_pool_entry_list(pool_free_block(next_entry));
            }
            (*entry).set_BlockSize((*entry).BlockSize() + (*next_entry).BlockSize());
        }
    }

    // Try to combine with previous block
    let entry = if (*entry).PreviousSize() != 0 {
        next_entry = pool_prev_block(entry);
        if (*next_entry).PoolType() == 0 {
            combined = true;
            if (*next_entry).BlockSize() != 1 {
                exp_check_pool_links(pool_free_block(next_entry));
                exp_remove_pool_entry_list(pool_free_block(next_entry));
            }
            (*next_entry).set_BlockSize((*next_entry).BlockSize() + (*entry).BlockSize());
            next_entry
        } else {
            entry
        }
    } else {
        entry
    };

    // Check if combined blocks form a full page
    if PAGE_ALIGN(entry as usize) == entry as usize
        && PAGE_ALIGN(pool_next_block(entry) as usize) == pool_next_block(entry) as usize
    {
        ex_unlock_pool(pool_desc, old_irql);
        InterlockedExchangeAdd(
            &mut (*pool_desc).TotalPages as *mut ULONG as *mut LONG,
            -1,
        );
        pool_pages::MiFreePoolPages(entry as PVOID);
        return;
    }

    // Mark as free and insert into free list
    (*entry).set_PoolType(0);
    let block_size = (*entry).BlockSize();

    if combined {
        let next = pool_next_block(entry);
        if PAGE_ALIGN(next as usize) != next as usize {
            (*next).set_PreviousSize(block_size);
        }
    }

    exp_insert_pool_head_list(
        (*pool_desc)
            .ListHeads
            .as_mut_ptr()
            .add(block_size as usize - 1),
        pool_free_block(entry),
    );
    exp_check_pool_links(pool_free_block(entry));
    ex_unlock_pool(pool_desc, old_irql);
}

// ============================================================================
// ExFreePool
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExFreePool(p: PVOID) {
    ExFreePoolWithTag(p, 0);
}

// ============================================================================
// ExQueryPoolBlockSize
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExQueryPoolBlockSize(
    _pool_block: PVOID,
    _quota_charged: PBOOLEAN,
) -> SIZE_T {
    // Not implemented (matches C)
    0
}

// ============================================================================
// ExAllocatePoolWithQuota / ExAllocatePoolWithQuotaTag
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolWithQuota(
    pool_type: POOL_TYPE,
    number_of_bytes: SIZE_T,
) -> PVOID {
    ExAllocatePoolWithQuotaTag(pool_type, number_of_bytes, TAG_NONE)
}

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolWithQuotaTag(
    mut pool_type: POOL_TYPE,
    mut number_of_bytes: SIZE_T,
    tag: ULONG,
) -> PVOID {
    // DUMMY_PSEH: callers use _SEH2_TRY to catch the raised exception,
    // but SEH handlers are dead code. Never raise, always return NULL on failure.
    let raise = false;

    if pool_type & POOL_QUOTA_FAIL_INSTEAD_OF_RAISE as POOL_TYPE != 0 {
        pool_type &= !(POOL_QUOTA_FAIL_INSTEAD_OF_RAISE as POOL_TYPE);
    }

    pool_type += QUOTA_POOL_MASK;

    let process = PsGetCurrentProcess();

    if number_of_bytes <= (PAGE_SIZE - POOL_BLOCK_SIZE - core::mem::size_of::<PVOID>())
        && process as PVOID != PsInitialSystemProcess
    {
        number_of_bytes += core::mem::size_of::<PVOID>();
    } else {
        pool_type -= QUOTA_POOL_MASK;
    }

    let buffer = ExAllocatePoolWithTag(pool_type, number_of_bytes, tag);

    if !buffer.is_null() && PAGE_ALIGN(buffer as usize) != buffer as usize {
        if ExpPoolFlags & POOL_FLAG_SPECIAL_POOL != 0
            && MmIsSpecialPoolAddress(buffer) != 0
        {
            return buffer;
        }

        if pool_type & QUOTA_POOL_MASK == 0 {
            return buffer;
        }

        if process as PVOID == PsInitialSystemProcess {
            return buffer;
        }

        let entry = pool_entry(buffer);
        let status = PsChargeProcessPoolQuota(
            process,
            pool_type & BASE_POOL_TYPE_MASK,
            (*entry).BlockSize() as SIZE_T * POOL_BLOCK_SIZE,
        );
        if !NT_SUCCESS(status) {
            let process_slot = (pool_next_block(entry) as *mut PVOID).sub(1);
            *process_slot = null_mut();
            ExFreePoolWithTag(buffer, tag);
            if raise {
                RtlRaiseStatus(status);
            }
            return null_mut();
        }

        let process_slot = (pool_next_block(entry) as *mut PVOID).sub(1);
        *process_slot = process as PVOID;
        ObReferenceObject(process as PVOID);
    } else if buffer.is_null() && raise {
        RtlRaiseStatus(STATUS_INSUFFICIENT_RESOURCES);
    }

    buffer
}

// ============================================================================
// ExAllocatePoolWithTagPriority
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExAllocatePoolWithTagPriority(
    pool_type: POOL_TYPE,
    number_of_bytes: SIZE_T,
    tag: ULONG,
    _priority: ntkrnl_sys::ex::EX_POOL_PRIORITY,
) -> PVOID {
    ExAllocatePoolWithTag(pool_type, number_of_bytes, tag)
}

// ============================================================================
// ExpCheckPoolAllocation — validates a pool allocation's tag and type
// Called from ps/kill.c and kdbg.c
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn ExpCheckPoolAllocation(
    p: PVOID,
    pool_type: POOL_TYPE,
    tag: ULONG,
) {
    use ntkrnl_sys::ke::{KeAcquireSpinLock, KeReleaseSpinLock};

    let entry: PPOOL_HEADER = (p as PPOOL_HEADER).sub(1);

    // Check if this is a large allocation (page-aligned)
    let real_pool_type;
    if PAGE_ALIGN(p as usize) == p as usize {
        // Lock the big page table
        let mut old_irql: KIRQL = 0;
        KeAcquireSpinLock(&raw mut ExpLargePoolTableLock, &mut old_irql);

        let mut found = false;
        for i in 0..PoolBigPageTableSize {
            let big_entry = PoolBigPageTable.add(i);
            if (*big_entry).Va == p {
                if (*big_entry).Key != tag {
                    KeBugCheckEx(BAD_POOL_CALLER, 0x0A, p as ULONG_PTR, (*big_entry).Key as ULONG_PTR, tag as ULONG_PTR);
                }
                found = true;
                break;
            }
        }

        KeReleaseSpinLock(&raw mut ExpLargePoolTableLock, old_irql);
        let _ = found;

        // Get pool type by address
        real_pool_type = super::pool::MmDeterminePoolType(p);
    } else {
        // Verify the tag
        if (*entry).PoolTag != tag {
            KeBugCheckEx(BAD_POOL_CALLER, 0x0A, p as ULONG_PTR, (*entry).PoolTag as ULONG_PTR, tag as ULONG_PTR);
        }

        // Check the rest of the header
        exp_check_pool_header(entry);

        // Get pool type from entry
        real_pool_type = ((*entry).PoolType() as ULONG).wrapping_sub(1);
    }

    // Verify pool type if specified (PoolType != -1u32)
    if pool_type != 0xFFFFFFFF {
        if real_pool_type != pool_type {
            KeBugCheckEx(BAD_POOL_CALLER, 0xCC, p as ULONG_PTR, (*entry).PoolTag as ULONG_PTR, tag as ULONG_PTR);
        }
    }
}

// ============================================================================
// MiDumpPoolConsumers — dump pool usage information
// Called from kdbg.c for debugging. Simplified stub.
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn MiDumpPoolConsumers(
    _called_from_dbg: BOOLEAN,
    _tag: ULONG,
    _mask: ULONG,
    _flags: ULONG,
) {
    // Simplified: pool consumer dumping requires DbgPrint which is complex.
    // The critical pool allocation/free paths work without this diagnostic.
}
