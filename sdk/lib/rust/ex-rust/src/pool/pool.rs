//! Page-level pool allocator — Rust translation of pool.c.
//!
//! Provides MiAllocatePoolPages, MiFreePoolPages, pool initialization,
//! pool protection, quota management, and mapping address functions.

use core::ptr::null_mut;
use ntkrnl_sys::base::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::CONTAINING_RECORD;

use super::globals::*;
// Re-import to disambiguate from ntkrnl_sys::mm::MmPagedPoolInfo (extern declaration)
// — this crate owns the actual definition in globals.rs.
use super::globals::MmPagedPoolInfo;

// ============================================================================
// Pool protection helpers (MiProtect/MiUnProtect)
// ============================================================================

unsafe fn mi_protect_free_non_paged_pool(virtual_address: PVOID, page_count: ULONG) {
    if MI_IS_PHYSICAL_ADDRESS(virtual_address) != 0 {
        return;
    }

    let mut pointer_pte = MiAddressToPte(virtual_address);
    let last_pte = pointer_pte.add(page_count as usize);
    while pointer_pte < last_pte {
        let mut temp_pte = *pointer_pte;
        temp_pte.set_valid(false);
        temp_pte.set_prototype(true);
        MI_WRITE_INVALID_PTE(pointer_pte, temp_pte);
        pointer_pte = pointer_pte.add(1);
    }

    // i386: inline TLB flush
    #[cfg(target_arch = "x86")]
    {
        let cr3: u32;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags));
    }
    #[cfg(not(target_arch = "x86"))]
    KeFlushEntireTb(TRUE, TRUE);
}

unsafe fn mi_unprotect_free_non_paged_pool(
    virtual_address: PVOID,
    page_count: ULONG,
) -> BOOLEAN {
    if MI_IS_PHYSICAL_ADDRESS(virtual_address) != 0 {
        return FALSE;
    }

    let mut pointer_pte = MiAddressToPte(virtual_address);
    let mut temp_pte = *pointer_pte;
    let mut unprotected_pages: PFN_COUNT = 0;

    while !temp_pte.valid() && temp_pte.prototype() {
        temp_pte.set_valid(true);
        temp_pte.set_prototype(false);
        MI_WRITE_VALID_PTE(pointer_pte, temp_pte);

        unprotected_pages += 1;
        if page_count != 0 && unprotected_pages == page_count as PFN_COUNT {
            break;
        }

        pointer_pte = pointer_pte.add(1);
        temp_pte = *pointer_pte;
    }

    if unprotected_pages != 0 { TRUE } else { FALSE }
}

unsafe fn mi_protected_pool_unprotect_links(
    links: PLIST_ENTRY,
    pool_flink: *mut PVOID,
    pool_blink: *mut PVOID,
) {
    *pool_flink = null_mut();
    *pool_blink = null_mut();

    if !IsListEmpty(links) {
        let pool_va = (*links).Flink as PVOID;
        let safe = mi_unprotect_free_non_paged_pool(pool_va, 1);
        if safe != 0 {
            *pool_flink = pool_va;
        }
    }

    if links as PVOID != (*links).Blink as PVOID {
        let pool_va = (*links).Blink as PVOID;
        let safe = mi_unprotect_free_non_paged_pool(pool_va, 1);
        if safe != 0 {
            *pool_blink = pool_va;
        }
    }
}

unsafe fn mi_protected_pool_protect_links(pool_flink: PVOID, pool_blink: PVOID) {
    if !pool_flink.is_null() {
        mi_protect_free_non_paged_pool(pool_flink, 1);
    }
    if !pool_blink.is_null() {
        mi_protect_free_non_paged_pool(pool_blink, 1);
    }
}

unsafe fn mi_protected_pool_insert_list(
    list_head: PLIST_ENTRY,
    entry: PLIST_ENTRY,
    critical: BOOLEAN,
) {
    let mut pool_flink: PVOID = null_mut();
    let mut pool_blink: PVOID = null_mut();

    mi_protected_pool_unprotect_links(list_head, &mut pool_flink, &mut pool_blink);

    if critical != 0 {
        InsertHeadList(list_head, entry);
    } else {
        InsertTailList(list_head, entry);
    }

    mi_protected_pool_protect_links(pool_flink, pool_blink);
}

unsafe fn mi_protected_pool_remove_entry_list(entry: PLIST_ENTRY) {
    let mut pool_flink: PVOID = null_mut();
    let mut pool_blink: PVOID = null_mut();

    mi_protected_pool_unprotect_links(entry, &mut pool_flink, &mut pool_blink);

    RemoveEntryList(entry);

    if !pool_flink.is_null() {
        mi_protect_free_non_paged_pool(pool_flink, 1);
    }
    if !pool_blink.is_null() {
        mi_protect_free_non_paged_pool(pool_blink, 1);
    }
}

// ============================================================================
// MiProtectFreeNonPagedPool / MiUnProtectFreeNonPagedPool (exported)
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiProtectFreeNonPagedPool(
    VirtualAddress: PVOID,
    PageCount: ULONG,
) {
    mi_protect_free_non_paged_pool(VirtualAddress, PageCount);
}

#[no_mangle]
pub unsafe extern "system" fn MiUnProtectFreeNonPagedPool(
    VirtualAddress: PVOID,
    PageCount: ULONG,
) -> BOOLEAN {
    mi_unprotect_free_non_paged_pool(VirtualAddress, PageCount)
}

// ============================================================================
// MiInitializeNonPagedPoolThresholds
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiInitializeNonPagedPoolThresholds() {
    let size = MmMaximumNonPagedPoolInPages;

    // Default low threshold of 8MB or one third of nonpaged pool
    MiLowNonPagedPoolThreshold = core::cmp::min(
        (8 * _1MB) >> PAGE_SHIFT,
        size / 3,
    );

    // Default high threshold of 20MB or 50%
    MiHighNonPagedPoolThreshold = core::cmp::min(
        (20 * _1MB) >> PAGE_SHIFT,
        size / 2,
    );
}

// ============================================================================
// MiInitializePoolEvents
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiInitializePoolEvents() {
    // Lock paged pool
    KeAcquireGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);

    let free_pool_in_pages =
        MmSizeOfPagedPoolInPages.wrapping_sub(MmPagedPoolInfo.AllocatedPagedPool as PFN_NUMBER);

    if free_pool_in_pages >= MiHighPagedPoolThreshold {
        KeSetEvent(MiHighPagedPoolEvent, 0, FALSE);
    } else {
        KeClearEvent(MiHighPagedPoolEvent);
    }

    if free_pool_in_pages <= MiLowPagedPoolThreshold {
        KeSetEvent(MiLowPagedPoolEvent, 0, FALSE);
    } else {
        KeClearEvent(MiLowPagedPoolEvent);
    }

    KeReleaseGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);

    // Nonpaged pool
    let old_irql = KeAcquireQueuedSpinLock(LockQueueMmNonPagedPoolLock);

    let free_pool_in_pages =
        MmMaximumNonPagedPoolInPages.wrapping_sub(MmAllocatedNonPagedPool);

    if free_pool_in_pages >= MiHighNonPagedPoolThreshold {
        KeSetEvent(MiHighNonPagedPoolEvent, 0, FALSE);
    } else {
        KeClearEvent(MiHighNonPagedPoolEvent);
    }

    if free_pool_in_pages <= MiLowNonPagedPoolThreshold {
        KeSetEvent(MiLowNonPagedPoolEvent, 0, FALSE);
    } else {
        KeClearEvent(MiLowNonPagedPoolEvent);
    }

    KeReleaseQueuedSpinLock(LockQueueMmNonPagedPoolLock, old_irql);
}

// ============================================================================
// MiInitializeNonPagedPool
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiInitializeNonPagedPool() {
    // Initialize S-LISTs
    InitializeSListHead(&raw mut MiPagedPoolSListHead);
    InitializeSListHead(&raw mut MiNonPagedPoolSListHead);

    if MmNumberOfPhysicalPages >= ((2 * _1GB) / PAGE_SIZE) {
        MiNonPagedPoolSListMaximum *= 8;
        MiPagedPoolSListMaximum *= 8;
    } else if MmNumberOfPhysicalPages >= (_1GB / PAGE_SIZE) {
        MiNonPagedPoolSListMaximum *= 2;
        MiPagedPoolSListMaximum *= 2;
    }

    if MmProtectFreedNonPagedPool != 0 {
        MiNonPagedPoolSListMaximum = 0;
        MiPagedPoolSListMaximum = 0;
    }

    // Initialize 4 free page lists
    for i in 0..MI_MAX_FREE_PAGE_LISTS {
        InitializeListHead(&mut MmNonPagedPoolFreeListHead[i]);
    }

    // Calculate initial pool pages
    let pool_pages = BYTES_TO_PAGES(MmSizeOfNonPagedPoolInBytes) as PFN_COUNT;
    MmNumberOfFreeNonPagedPool = pool_pages;

    // Initialize first free entry
    let free_entry = MmNonPagedPoolStart as PMMFREE_POOL_ENTRY;
    let first_entry = free_entry;
    (*free_entry).Size = pool_pages;
    (*free_entry).Signature = MM_FREE_POOL_SIGNATURE;
    (*free_entry).Owner = first_entry;

    InsertHeadList(
        &mut MmNonPagedPoolFreeListHead[MI_MAX_FREE_PAGE_LISTS - 1],
        &mut (*free_entry).List,
    );

    // Link remaining pages to the first entry
    let mut remaining = pool_pages;
    let mut fe = free_entry;
    while remaining > 1 {
        remaining -= 1;
        fe = (fe as usize + PAGE_SIZE) as PMMFREE_POOL_ENTRY;
        (*fe).Owner = first_entry;
        (*fe).Signature = MM_FREE_POOL_SIGNATURE;
    }

    // Record first pool PFN
    let pointer_pte = MiAddressToPte(MmNonPagedPoolStart);
    MiStartOfInitialPoolFrame = PFN_FROM_PTE(pointer_pte);

    // Record end of initial pool
    MmNonPagedPoolEnd0 =
        (MmNonPagedPoolStart as usize + MmSizeOfNonPagedPoolInBytes) as PVOID;

    // Record last pool PFN
    let pointer_pte = MiAddressToPte((MmNonPagedPoolEnd0 as usize - 1) as PVOID);
    MiEndOfInitialPoolFrame = PFN_FROM_PTE(pointer_pte);

    // Calculate expansion pool charge
    MiExpansionPoolPagesInitialCharge =
        BYTES_TO_PAGES(MmMaximumNonPagedPoolInBytes - MmSizeOfNonPagedPoolInBytes) as PFN_COUNT;
    MiExpansionPoolPagesInitialCharge -= 2; // guard pages top & bottom

    // Initialize nonpaged pool expansion PTE space
    let pointer_pte = MiAddressToPte(MmNonPagedPoolExpansionStart);
    MiInitializeSystemPtes(
        pointer_pte.add(1),
        MiExpansionPoolPagesInitialCharge,
        NonPagedPoolExpansion,
    );
}

// ============================================================================
// MmDeterminePoolType
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MmDeterminePoolType(pool_address: PVOID) -> POOL_TYPE {
    if pool_address >= MmPagedPoolStart && pool_address <= MmPagedPoolEnd {
        return PagedPool;
    }
    if pool_address >= MmNonPagedPoolStart && pool_address <= MmNonPagedPoolEnd {
        return NonPagedPool;
    }
    KeBugCheckEx(BAD_POOL_CALLER, 0x42, pool_address as ULONG_PTR, 0, 0);
}

// ============================================================================
// MiAllocatePoolPages — the page-level allocation function
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiAllocatePoolPages(
    pool_type: POOL_TYPE,
    size_in_bytes: SIZE_T,
) -> PVOID {
    let size_in_pages = BYTES_TO_PAGES(size_in_bytes) as PFN_COUNT;

    if size_in_pages == 0 {
        return null_mut();
    }

    // ===== Paged pool path =====
    if (pool_type & BASE_POOL_TYPE_MASK) == PagedPool {
        // DBG: MiAllocatePoolPages PagedPool entry

        // Try S-LIST for single-page allocations
        if size_in_pages == 1 && ExQueryDepthSList(&raw mut MiPagedPoolSListHead) != 0 {
            let base_va = InterlockedPopEntrySList(&raw mut MiPagedPoolSListHead) as PVOID;
            if !base_va.is_null() {
                return base_va;
            }
        }

        // DBG: before KeAcquireGuardedMutex

        // Lock paged pool
        KeAcquireGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);

        // DBG: after KeAcquireGuardedMutex

        // Find clear bits in the allocation bitmap
        let mut i = RtlFindClearBitsAndSet(
            MmPagedPoolInfo.PagedPoolAllocationMap,
            size_in_pages,
            MmPagedPoolInfo.PagedPoolHint,
        );

        // DBG: after RtlFindClearBitsAndSet

        if i == 0xFFFFFFFF {
            // Need to expand paged pool
            let pde_count = ((size_in_pages as ULONG - 1) / PTE_PER_PAGE as ULONG) + 1;
            let page_table_count;

            // Check expansion space
            if MmPagedPoolInfo.NextPdeForPagedPoolExpansion
                > MiAddressToPte(MmPagedPoolInfo.LastPteForPagedPool) as PVOID
            {
                KeReleaseGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);
                return null_mut();
            }

            // Check how many PDEs we can expand
            let last_expansion_pde =
                MiPteToPde(MmPagedPoolInfo.LastPteForPagedPool as PMMPTE);
            let next_pde = MmPagedPoolInfo.NextPdeForPagedPoolExpansion as PMMPDE;
            let available = (last_expansion_pde as usize + core::mem::size_of::<MMPTE>()
                - next_pde as usize)
                / core::mem::size_of::<MMPTE>();

            if available < pde_count as usize {
                page_table_count = available as ULONG;
            } else {
                page_table_count = pde_count;
            }

            // Get the template PDE
            let temp_pde_template = ValidKernelPde;
            let mut pointer_pde = next_pde;
            let base_va_start = MiPdeToPte(pointer_pde) as PVOID;

            // Lock PFN database
            let old_irql = MiAcquirePfnLock();
            let mut remaining = page_table_count;
            while remaining > 0 {
                MI_SET_USAGE(MI_USAGE_PAGED_POOL);
                MI_SET_PROCESS2("Kernel");
                let page_frame_number = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
                let mut temp_pde = temp_pde_template;
                temp_pde.set_page_frame_number(page_frame_number);

                #[cfg(not(target_arch = "x86"))]
                {
                    // 3+ level paging: no double-buffering
                    MiInitializePfnForOtherProcess(
                        page_frame_number,
                        pointer_pde as PMMPTE,
                        PFN_FROM_PTE(MiAddressToPte(pointer_pde as PVOID)),
                    );
                    MI_WRITE_VALID_PDE(pointer_pde, temp_pde);
                }

                #[cfg(target_arch = "x86")]
                {
                    // i386: double-buffered system page directory
                    let idx = ((pointer_pde as usize) & (SYSTEM_PD_SIZE - 1))
                        / core::mem::size_of::<MMPTE>();
                    *MmSystemPagePtes.add(idx) = temp_pde;
                    let pde_index = pointer_pde.offset_from(MiAddressToPde(null_mut())) as usize;
                    let spd_index = pde_index / PDE_PER_PAGE;
                    MiInitializePfnForOtherProcess(
                        page_frame_number,
                        pointer_pde as PMMPTE,
                        *(*(&raw const MmSystemPageDirectory)).as_ptr().add(spd_index),
                    );
                }

                pointer_pde = pointer_pde.add(1);
                remaining -= 1;
            }
            MiReleasePfnLock(old_irql);

            // Clear availability bits
            let end_alloc = ((next_pde as usize
                - MiAddressToPte(MmPagedPoolInfo.FirstPteForPagedPool as PVOID) as usize)
                / core::mem::size_of::<MMPTE>())
                * PTE_PER_PAGE;
            RtlClearBits(
                MmPagedPoolInfo.PagedPoolAllocationMap,
                end_alloc as ULONG,
                page_table_count * PTE_PER_PAGE as ULONG,
            );

            // Update expansion pointer
            MmPagedPoolInfo.NextPdeForPagedPoolExpansion =
                (next_pde as usize + page_table_count as usize * core::mem::size_of::<MMPTE>())
                    as PVOID;

            // Zero the new pages
            RtlZeroMemory(base_va_start, page_table_count as usize * PAGE_SIZE);

            // Try again
            i = RtlFindClearBitsAndSet(
                MmPagedPoolInfo.PagedPoolAllocationMap,
                size_in_pages,
                0,
            );
            if i == 0xFFFFFFFF {
                KeReleaseGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);
                return null_mut();
            }
        }

        // Update hint for single-page allocations
        if size_in_pages == 1 {
            MmPagedPoolInfo.PagedPoolHint = i + 1;
        }

        // Mark end bit
        let end_alloc = i + size_in_pages - 1;
        RtlSetBit(MmPagedPoolInfo.EndOfPagedPoolBitmap, end_alloc);

        // DBG: before KeReleaseGuardedMutex

        KeReleaseGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);

        // DBG: after KeReleaseGuardedMutex

        // Calculate VA
        let base_va = (MmPagedPoolStart as usize + ((i as usize) << PAGE_SHIFT)) as PVOID;

        // i386: inline TLB flush (KeFlushEntireTb has a separate crash bug — investigate later)
        #[cfg(target_arch = "x86")]
        {
            let cr3: u32;
            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags));
            core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags));
        }
        #[cfg(not(target_arch = "x86"))]
        KeFlushEntireTb(TRUE, TRUE);

        // DBG: after TLB flush

        {
            // Demand-zero writable PTEs
            let mut temp_pte = MMPTE { Long: 0 };
            MI_MAKE_SOFTWARE_PTE(&mut temp_pte, MM_READWRITE);

            let mut pointer_pte = MiAddressToPte(base_va);
            let end_pte = pointer_pte.add(size_in_pages as usize);
            while pointer_pte < end_pte {
                MI_WRITE_INVALID_PTE(pointer_pte, temp_pte);
                pointer_pte = pointer_pte.add(1);
            }

            // DBG: PagedPool MiAllocatePoolPages returning

            return base_va;
        }
    }

    // ===== Nonpaged pool path =====

    // Try S-LIST for single-page allocations
    if size_in_pages == 1 && ExQueryDepthSList(&raw mut MiNonPagedPoolSListHead) != 0 {
        let base_va = InterlockedPopEntrySList(&raw mut MiNonPagedPoolSListHead) as PVOID;
        if !base_va.is_null() {
            return base_va;
        }
    }

    // Search free page lists
    let bucket = core::cmp::min(size_in_pages as usize, MI_MAX_FREE_PAGE_LISTS) - 1;
    let mut next_head = (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(bucket) as PLIST_ENTRY;
    let last_head =
        (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(MI_MAX_FREE_PAGE_LISTS) as PLIST_ENTRY;

    let old_irql = KeAcquireQueuedSpinLock(LockQueueMmNonPagedPoolLock);

    while next_head < last_head {
        let mut next_entry = (*next_head).Flink;
        while next_entry != next_head {
            // Unprotect if needed
            if MmProtectFreedNonPagedPool != 0 {
                mi_unprotect_free_non_paged_pool(next_entry as PVOID, 0);
            }

            // Get free entry from LIST_ENTRY
            let free_entry = CONTAINING_RECORD!(next_entry, MMFREE_POOL_ENTRY, List)
                as PMMFREE_POOL_ENTRY;

            if (*free_entry).Size >= size_in_pages {
                // Consume pages from this entry
                (*free_entry).Size -= size_in_pages;

                let base_va = (free_entry as usize
                    + (((*free_entry).Size as usize) << PAGE_SHIFT))
                    as PVOID;

                // Remove from list
                if MmProtectFreedNonPagedPool != 0 {
                    mi_protected_pool_remove_entry_list(&mut (*free_entry).List);
                } else {
                    RemoveEntryList(&mut (*free_entry).List);
                }

                // Re-insert if still has free space
                if (*free_entry).Size != 0 {
                    let idx =
                        core::cmp::min((*free_entry).Size as usize, MI_MAX_FREE_PAGE_LISTS) - 1;

                    if MmProtectFreedNonPagedPool != 0 {
                        mi_protected_pool_insert_list(
                            (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(idx) as PLIST_ENTRY,
                            &mut (*free_entry).List,
                            TRUE,
                        );
                        mi_protect_free_non_paged_pool(
                            free_entry as PVOID,
                            (*free_entry).Size,
                        );
                    } else {
                        InsertTailList(
                            (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(idx) as PLIST_ENTRY,
                            &mut (*free_entry).List,
                        );
                    }
                }

                // Mark start PFN
                let pointer_pte = MiAddressToPte(base_va);
                let pfn1 = MiGetPfnEntry(PFN_FROM_PTE(pointer_pte));


                (*pfn1).set_start_of_allocation(true);

                if pool_type & VERIFIER_POOL_MASK != 0 {
                    (*pfn1).set_verifier_allocation(true);
                }

                // Mark end PFN
                if size_in_pages != 1 {
                    let end_pte = pointer_pte.add(size_in_pages as usize - 1);
                    let pfn_end = MiGetPfnEntry((*end_pte).page_frame_number());

                    (*pfn_end).set_end_of_allocation(true);
                } else {
                    (*pfn1).set_end_of_allocation(true);
                }

                KeReleaseQueuedSpinLock(LockQueueMmNonPagedPoolLock, old_irql);
                return base_va;
            }

            // Try next entry
            next_entry = (*free_entry).List.Flink;

            if MmProtectFreedNonPagedPool != 0 {
                mi_protect_free_non_paged_pool(free_entry as PVOID, (*free_entry).Size);
            }
        }

        next_head = (next_head as usize + core::mem::size_of::<LIST_ENTRY>()) as PLIST_ENTRY;
    }

    // No free entries found — allocate expansion PTEs
    KeReleaseQueuedSpinLock(LockQueueMmNonPagedPoolLock, old_irql);

    let start_pte = MiReserveSystemPtes(size_in_pages, NonPagedPoolExpansion);
    if start_pte.is_null() {
        return null_mut();
    }
    let mut pointer_pte = start_pte;

    // Re-acquire locks
    let old_irql = KeAcquireQueuedSpinLock(LockQueueMmNonPagedPoolLock);
    MiAcquirePfnLockAtDpcLevel();

    // Check available pages
    if MmAvailablePages < size_in_pages as PFN_NUMBER {
        MiReleasePfnLockFromDpcLevel();
        KeReleaseQueuedSpinLock(LockQueueMmNonPagedPoolLock, old_irql);
        MiReleaseSystemPtes(start_pte, size_in_pages, NonPagedPoolExpansion);
        return null_mut();
    }

    // Allocate and map pages
    let mut temp_pte = ValidKernelPte;
    let mut remaining = size_in_pages;
    let mut pfn1: PMMPFN = null_mut();
    while remaining > 0 {
        MI_SET_USAGE(MI_USAGE_PAGED_POOL);
        MI_SET_PROCESS2("Kernel");
        let page_frame_number = MiRemoveAnyPage(MI_GET_NEXT_COLOR());

        pfn1 = MiGetPfnEntry(page_frame_number);
        (*pfn1).set_reference_count(1);
        (*pfn1).set_share_count(1);
        (*pfn1).set_pte_address(pointer_pte);
        (*pfn1).set_page_location(ActiveAndValid);
        (*pfn1).set_verifier_allocation(false);

        temp_pte.set_page_frame_number(page_frame_number);
        MI_WRITE_VALID_PTE(pointer_pte, temp_pte);

        pointer_pte = pointer_pte.add(1);
        remaining -= 1;
    }

    // Mark last page as end of allocation
    (*pfn1).set_end_of_allocation(true);

    // Mark first page as start of allocation
    let start_pfn = MiGetPfnEntry((*start_pte).page_frame_number());
    (*start_pfn).set_start_of_allocation(true);

    if pool_type & VERIFIER_POOL_MASK != 0 {
        (*start_pfn).set_verifier_allocation(true);
    }

    MiReleasePfnLockFromDpcLevel();
    KeReleaseQueuedSpinLock(LockQueueMmNonPagedPoolLock, old_irql);

    MiPteToAddress(start_pte)
}

// ============================================================================
// MiFreePoolPages — the page-level free function
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MiFreePoolPages(starting_va: PVOID) -> ULONG {
    // ===== Paged pool path =====
    if starting_va >= MmPagedPoolStart && starting_va <= MmPagedPoolEnd {
        let offset = starting_va as usize - MmPagedPoolStart as usize;
        let i = (offset >> PAGE_SHIFT) as ULONG;
        let mut end = i;

        // Scan end bitmap to find allocation end
        while RtlTestBit(MmPagedPoolInfo.EndOfPagedPoolBitmap, end) == 0 {
            end += 1;
        }

        let number_of_pages = end - i + 1;

        // Try S-LIST for single-page frees
        if number_of_pages == 1
            && ExQueryDepthSList(&raw mut MiPagedPoolSListHead)
                < MiPagedPoolSListMaximum as USHORT
        {
            InterlockedPushEntrySList(&raw mut MiPagedPoolSListHead, starting_va as PSLIST_ENTRY);
            return 1;
        }

        // Delete the actual pages
        let pointer_pte = (MmPagedPoolInfo.FirstPteForPagedPool as PMMPTE).add(i as usize);
        MiDeleteSystemPageableVm(pointer_pte, number_of_pages as PFN_NUMBER, 0, null_mut());

        // Clear bitmaps
        KeAcquireGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);

        RtlClearBit(MmPagedPoolInfo.EndOfPagedPoolBitmap, end);
        RtlClearBits(
            MmPagedPoolInfo.PagedPoolAllocationMap,
            i,
            number_of_pages,
        );

        if i < MmPagedPoolInfo.PagedPoolHint {
            MmPagedPoolInfo.PagedPoolHint = i;
        }

        KeReleaseGuardedMutex(&raw mut MmPagedPoolMutex as PKGUARDED_MUTEX);

        return number_of_pages;
    }

    // ===== Nonpaged pool path =====
    let start_pte = MiAddressToPte(starting_va);
    let mut pointer_pte = start_pte;
    let start_pfn = MiGetPfnEntry((*pointer_pte).page_frame_number());
    let mut pfn1 = start_pfn;

    // Try S-LIST for single-page frees
    if (*pfn1).end_of_allocation()
        && ExQueryDepthSList(&raw mut MiNonPagedPoolSListHead)
            < MiNonPagedPoolSListMaximum as USHORT
    {
        InterlockedPushEntrySList(&raw mut MiNonPagedPoolSListHead, starting_va as PSLIST_ENTRY);
        return 1;
    }

    // Find end of allocation
    while !(*pfn1).end_of_allocation() {
        pointer_pte = pointer_pte.add(1);
        pfn1 = MiGetPfnEntry((*pointer_pte).page_frame_number());
    }

    let number_of_pages =
        (pointer_pte.offset_from(start_pte) as ULONG) + 1;

    // Acquire lock
    let old_irql = KeAcquireQueuedSpinLock(LockQueueMmNonPagedPoolLock);

    // Clear allocation markers
    (*start_pfn).set_start_of_allocation(false);
    (*pfn1).set_end_of_allocation(false);

    let mut free_pages = number_of_pages;

    // Check page after allocation for coalescing
    pointer_pte = pointer_pte.add(1);

    let pfn1_check;
    if MiGetPfnEntryIndex(pfn1) == MiEndOfInitialPoolFrame {
        pfn1_check = null_mut();
    } else {
        if MmProtectFreedNonPagedPool != 0 {
            mi_unprotect_free_non_paged_pool(MiPteToAddress(pointer_pte), 0);
        }

        if (*pointer_pte).valid() {
            pfn1_check = MiGetPfnEntry((*pointer_pte).page_frame_number());
        } else {
            pfn1_check = null_mut();
        }
    }

    // Coalesce with next free region
    if !pfn1_check.is_null() && !(*pfn1_check).start_of_allocation() {
        let next_free = (starting_va as usize + ((number_of_pages as usize) << PAGE_SHIFT))
            as PMMFREE_POOL_ENTRY;

        free_pages += (*next_free).Size;

        if MmProtectFreedNonPagedPool != 0 {
            mi_protected_pool_remove_entry_list(&mut (*next_free).List);
        } else {
            RemoveEntryList(&mut (*next_free).List);
        }
    }

    // Create our free entry
    let mut free_entry = starting_va as PMMFREE_POOL_ENTRY;

    // Check page before allocation for coalescing
    let pfn1_check;
    if MiGetPfnEntryIndex(start_pfn) == MiStartOfInitialPoolFrame {
        pfn1_check = null_mut();
    } else {
        let before_pte = start_pte.sub(1);

        if MmProtectFreedNonPagedPool != 0 {
            mi_unprotect_free_non_paged_pool(MiPteToAddress(before_pte), 0);
        }

        if (*before_pte).valid() {
            pfn1_check = MiGetPfnEntry((*before_pte).page_frame_number());
        } else {
            pfn1_check = null_mut();
        }
    }

    // Coalesce with previous free region
    if !pfn1_check.is_null() && !(*pfn1_check).end_of_allocation() {
        let prev_page = (starting_va as usize - PAGE_SIZE) as PMMFREE_POOL_ENTRY;
        let prev_free = (*prev_page).Owner;

        if MmProtectFreedNonPagedPool != 0 {
            mi_unprotect_free_non_paged_pool(prev_free as PVOID, 0);
        }

        if (*prev_free).Size < MI_MAX_FREE_PAGE_LISTS as PFN_COUNT {
            // Small enough to be on a specific free list — remove and re-insert
            if MmProtectFreedNonPagedPool != 0 {
                mi_protected_pool_remove_entry_list(&mut (*prev_free).List);
            } else {
                RemoveEntryList(&mut (*prev_free).List);
            }

            (*prev_free).Size += free_pages;

            let idx =
                core::cmp::min((*prev_free).Size as usize, MI_MAX_FREE_PAGE_LISTS) - 1;

            if MmProtectFreedNonPagedPool != 0 {
                mi_protected_pool_insert_list(
                    (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(idx) as PLIST_ENTRY,
                    &mut (*prev_free).List,
                    TRUE,
                );
            } else {
                InsertTailList(
                    (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(idx) as PLIST_ENTRY,
                    &mut (*prev_free).List,
                );
            }
        } else {
            // Large entry — just add our pages
            (*prev_free).Size += free_pages;
        }

        free_entry = prev_free;
    }

    // If we couldn't coalesce with previous, create new free entry
    if free_entry == starting_va as PMMFREE_POOL_ENTRY {
        (*free_entry).Size = free_pages;

        let idx = core::cmp::min(free_pages as usize, MI_MAX_FREE_PAGE_LISTS) - 1;

        if MmProtectFreedNonPagedPool != 0 {
            mi_protected_pool_insert_list(
                (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(idx) as PLIST_ENTRY,
                &mut (*free_entry).List,
                TRUE,
            );
        } else {
            InsertTailList(
                (&raw mut MmNonPagedPoolFreeListHead as *mut LIST_ENTRY).add(idx) as PLIST_ENTRY,
                &mut (*free_entry).List,
            );
        }
    }

    // Mark all pages in the merged free range (including coalesced next-range pages)
    // with Owner and Signature. Must use free_pages (total merged size), not just
    // number_of_pages, so that coalesced next-range pages get their Owner updated.
    let mut next_entry = starting_va as PMMFREE_POOL_ENTRY;
    let last_entry =
        (next_entry as usize + ((free_pages as usize) << PAGE_SHIFT)) as PMMFREE_POOL_ENTRY;
    while next_entry < last_entry {
        (*next_entry).Owner = free_entry;
        (*next_entry).Signature = MM_FREE_POOL_SIGNATURE;
        next_entry = (next_entry as usize + PAGE_SIZE) as PMMFREE_POOL_ENTRY;
    }

    // Protect the freed pool if enabled
    if MmProtectFreedNonPagedPool != 0 {
        mi_protect_free_non_paged_pool(free_entry as PVOID, (*free_entry).Size);
    }

    KeReleaseQueuedSpinLock(LockQueueMmNonPagedPoolLock, old_irql);
    number_of_pages
}

// ============================================================================
// MiInitializeSessionPool — session paged pool initialization
// ============================================================================

// MiInitializeSessionPool is heavily dependent on MM_SESSION_SPACE internal
// layout. We provide a stub that returns STATUS_SUCCESS.
// Session pool initialization is handled by the C code in session.c for now.
#[no_mangle]
pub unsafe extern "system" fn MiInitializeSessionPool() -> NTSTATUS {
    // TODO: Full implementation requires MM_SESSION_SPACE struct layout.
    // This function is called during session creation (MiSessionInitializeStub).
    // On ARM64 single-session boots, session pool is initialized from C code.
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// MmRaisePoolQuota
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MmRaisePoolQuota(
    pool_type: POOL_TYPE,
    current_max_quota: SIZE_T,
    new_max_quota: *mut SIZE_T,
) -> BOOLEAN {
    match pool_type {
        NonPagedPool => {
            if MmAvailablePages < MI_QUOTA_NON_PAGED_NEEDED_PAGES {
                return FALSE;
            }
            if MmMaximumNonPagedPoolInPages < (MmAllocatedNonPagedPool >> PAGE_SHIFT) {
                return FALSE;
            }
            if MmResidentAvailablePages < MI_NON_PAGED_QUOTA_MIN_RESIDENT_PAGES {
                return FALSE;
            }
            MmTotalNonPagedPoolQuota += MI_CHARGE_NON_PAGED_POOL_QUOTA;
            core::ptr::write_volatile(
                new_max_quota,
                current_max_quota + MI_CHARGE_NON_PAGED_POOL_QUOTA,
            );
            TRUE
        }
        PagedPool => {
            if MmSizeOfPagedPoolInPages
                < (MmPagedPoolInfo.AllocatedPagedPool as PFN_NUMBER >> PAGE_SHIFT)
            {
                return FALSE;
            }
            MmTotalPagedPoolQuota += MI_CHARGE_PAGED_POOL_QUOTA;
            core::ptr::write_volatile(
                new_max_quota,
                current_max_quota + MI_CHARGE_PAGED_POOL_QUOTA,
            );
            TRUE
        }
        _ => {
            KeBugCheckEx(0, 0, 0, 0, 0); // nolint: bugcheck-code (C is DEFAULT_UNREACHABLE — deliberate fail-loud on an impossible POOL_TYPE)
        }
    }
}

// ============================================================================
// MmReturnPoolQuota
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MmReturnPoolQuota(
    pool_type: POOL_TYPE,
    quota_to_return: SIZE_T,
) {
    match pool_type {
        NonPagedPool => {
            MmTotalNonPagedPoolQuota -= quota_to_return;
        }
        PagedPool => {
            MmTotalPagedPoolQuota -= quota_to_return;
        }
        _ => {
            KeBugCheckEx(0, 0, 0, 0, 0); // nolint: bugcheck-code (C is DEFAULT_UNREACHABLE — deliberate fail-loud on an impossible POOL_TYPE)
        }
    }
}

// ============================================================================
// MmAllocateMappingAddress
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MmAllocateMappingAddress(
    number_of_bytes: SIZE_T,
    pool_tag: ULONG,
) -> PVOID {
    if pool_tag == 0 {
        return null_mut();
    }

    let mut size_in_pages = BYTES_TO_PAGES(number_of_bytes) as PFN_NUMBER;
    if size_in_pages == 0 {
        KeBugCheckEx(
            SYSTEM_PTE_MISUSE,
            PTE_MAPPING_NONE,
            size_in_pages,
            pool_tag as ULONG_PTR,
            0, // _ReturnAddress() not available in Rust
        );
    }

    // Two extra PTEs for size and pool tag
    size_in_pages += 2;

    let pointer_pte = MiReserveSystemPtes(size_in_pages as ULONG, SystemPteSpace);
    if pointer_pte.is_null() {
        return null_mut();
    }

    // Store size in PTE[0] via the List.NextEntry field
    let mut temp_pte = MMPTE { Long: 0 };
    // The C code stores size_in_pages in u.List.NextEntry which is the full Long
    // minus the valid bit. We just store size_in_pages directly (valid=0).
    temp_pte.Long = size_in_pages;
    MI_WRITE_INVALID_PTE(pointer_pte, temp_pte);

    // Store pool tag in PTE[1]
    let mut temp_pte = MMPTE { Long: pool_tag as ULONG_PTR };
    temp_pte.set_valid(false);
    MI_WRITE_INVALID_PTE(pointer_pte.add(1), temp_pte);

    MiPteToAddress(pointer_pte.add(2))
}

// ============================================================================
// MmFreeMappingAddress
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn MmFreeMappingAddress(
    base_address: PVOID,
    pool_tag: ULONG,
) {
    let pointer_pte = MiAddressToPte(base_address).sub(2);

    // Verify pool tag
    let mut expected_pte = MMPTE { Long: pool_tag as ULONG_PTR };
    expected_pte.set_valid(false);
    if (*pointer_pte.add(1)).Long != expected_pte.Long {
        KeBugCheckEx(
            SYSTEM_PTE_MISUSE,
            PTE_MAPPING_NOT_OWNED,
            base_address as ULONG_PTR,
            pool_tag as ULONG_PTR,
            (*pointer_pte.add(1)).Long,
        );
    }

    // Get size
    let size_in_pages = (*pointer_pte).Long as PFN_NUMBER;
    if size_in_pages < 3 {
        KeBugCheckEx(
            SYSTEM_PTE_MISUSE,
            PTE_MAPPING_EMPTY,
            base_address as ULONG_PTR,
            pool_tag as ULONG_PTR,
            0,
        );
    }

    // Verify all PTEs are empty
    for idx in 2..size_in_pages {
        if (*pointer_pte.add(idx)).Long != 0 {
            KeBugCheckEx(
                SYSTEM_PTE_MISUSE,
                PTE_MAPPING_RESERVED,
                pointer_pte as ULONG_PTR,
                pool_tag as ULONG_PTR,
                size_in_pages - 2,
            );
        }
    }

    MiReleaseSystemPtes(pointer_pte, size_in_pages as ULONG, SystemPteSpace);
}
