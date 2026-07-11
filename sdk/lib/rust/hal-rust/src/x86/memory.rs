//! HAL physical/virtual memory helpers — early-boot physical-memory
//! carving from the loader-block descriptor list, plus the runtime
//! HAL-heap PTE manipulator that maps device-physical pages into the
//! 4 MB HAL VA window at `MM_HAL_VA_START`.
//!
//! Translated from `hal/halx86/generic/memory.c`.  Three exported
//! globals stay defined here:
//!
//!   HalpUsedAllocDescriptors            — ULONG, # entries used in the array
//!   HalpAllocationDescriptorArray[64]   — MEMORY_ALLOCATION_DESCRIPTOR
//!   HalpHeapStart                       — PVOID, bumped by every successful map
//!
//! The arch split: `HalAddressToPte(va)` is a HAL-private inline that
//! resolves to the kernel's `MiAddressToPte`.  i386 has a closed-form
//! formula at `PTE_BASE = 0xC0000000`; amd64 has a 4-level walk macro at
//! `PTE_BASE = 0xFFFFF68000000000`.  Both inline directly here — no
//! cross-DLL call overhead, and the kernel macros are header-only so
//! there's nothing to export anyway.

use ntkrnl_sys::hal_ddk::{
    HalpFlushTLB, BOOLEAN, LIST_ENTRY, MEMORY_ALLOCATION_DESCRIPTOR, PFN_COUNT, PFN_NUMBER,
    PHYSICAL_ADDRESS, PLIST_ENTRY, PLOADER_PARAMETER_BLOCK, PMEMORY_ALLOCATION_DESCRIPTOR, PVOID,
    ULONG, ULONG_PTR, ULONG64,
    _TYPE_OF_MEMORY_LoaderFree as LOADER_FREE,
    _TYPE_OF_MEMORY_LoaderFirmwareTemporary as LOADER_FIRMWARE_TEMPORARY,
    _TYPE_OF_MEMORY_LoaderHALCachedMemory as LOADER_HAL_CACHED_MEMORY,
};

const PAGE_SHIFT: u32 = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const BYTE_OFFSET_MASK: ULONG_PTR = (PAGE_SIZE as ULONG_PTR) - 1;

// MM_HAL_VA_START — start of the 4 MB HAL VA window.  HAL_HEAP_START is
// bumped 1 MB into that window (the first 1 MB is reserved for things like
// the BIOS data area mapping).  Per ndk/{i386,amd64}/ketypes.h.
#[cfg(target_arch = "x86")]
const MM_HAL_VA_START: ULONG_PTR = 0xFFC00000;
#[cfg(target_arch = "x86_64")]
const MM_HAL_VA_START: ULONG_PTR = 0xFFFFFFFFFFC00000;
const MM_HAL_HEAP_OFFSET: ULONG_PTR = 1024 * 1024; // 1 MB

// PTE_BASE per arch.  i386: 4-byte PTEs at 0xC0000000.  amd64: 8-byte PTEs
// reachable via the self-map at 0xFFFFF68000000000.
#[cfg(target_arch = "x86")]
const PTE_BASE: ULONG_PTR = 0xC0000000;
#[cfg(target_arch = "x86_64")]
const PTE_BASE: ULONG_PTR = 0xFFFFF68000000000;

// Native PTE word width and the type alias bindgen uses (bitfield-typed
// "_HARDWARE_PTE_X86" on i386, "_HARDWARE_PTE" on amd64 — different names
// but identical accessor surface for what we need).
#[cfg(target_arch = "x86")]
use ntkrnl_sys::hal_ddk::_HARDWARE_PTE_X86 as HardwarePte;
#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::hal_ddk::_HARDWARE_PTE as HardwarePte;

#[cfg(target_arch = "x86")]
type PteWord = u32;
#[cfg(target_arch = "x86_64")]
type PteWord = u64;

/// MiAddressToPte — i386 closed form.
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn address_to_pte(va: ULONG_PTR) -> *mut HardwarePte {
    (PTE_BASE + (va >> PAGE_SHIFT) * 4) as *mut HardwarePte
}

/// MiAddressToPte — amd64.  Mirrors `_MiAddressToPte` in
/// ntoskrnl/include/internal/amd64/mm.h: `Offset = (VA >> 9) & (0xFFFFFFFFF << 3)`.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn address_to_pte(va: ULONG_PTR) -> *mut HardwarePte {
    let offset = (va >> 9) & (0xFFFFFFFFF << 3);
    (PTE_BASE + offset) as *mut HardwarePte
}

// ---------------------------------------------------------------------------
// Globals — symbol layout matches the C definitions in memory.c.
// HalpAllocationDescriptorArray is one zeroed slot * 64; layout has only
// raw pointers + ULONG_PTRs + a u32 enum, so all-zero is a valid initial
// state (LoaderExceptionBlock = 0 is a "free / unused" sentinel and we
// never inspect uninit slots — only the ones we've populated).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpUsedAllocDescriptors: ULONG = 0;

#[unsafe(no_mangle)]
pub static mut HalpAllocationDescriptorArray: [MEMORY_ALLOCATION_DESCRIPTOR; 64] = [
    MEMORY_ALLOCATION_DESCRIPTOR {
        ListEntry: LIST_ENTRY {
            Flink: core::ptr::null_mut(),
            Blink: core::ptr::null_mut(),
        },
        MemoryType: 0,
        BasePage: 0,
        PageCount: 0,
    };
    64
];

#[unsafe(no_mangle)]
pub static mut HalpHeapStart: PVOID = (MM_HAL_VA_START + MM_HAL_HEAP_OFFSET) as PVOID;

// ---------------------------------------------------------------------------
// LIST_ENTRY ops — RtlInsertHeadList / InsertTailList / RemoveEntryList are
// macros in xdk/rtltypes.h, so bindgen drops them.  Inlined here.
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn insert_head_list(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    unsafe {
        let next = (*head).Flink;
        (*entry).Flink = next;
        (*entry).Blink = head;
        (*next).Blink = entry;
        (*head).Flink = entry;
    }
}

#[inline(always)]
unsafe fn insert_tail_list(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    unsafe {
        let prev = (*head).Blink;
        (*entry).Flink = head;
        (*entry).Blink = prev;
        (*prev).Flink = entry;
        (*head).Blink = entry;
    }
}

#[inline(always)]
unsafe fn remove_entry_list(entry: PLIST_ENTRY) {
    unsafe {
        let prev = (*entry).Blink;
        let next = (*entry).Flink;
        (*prev).Flink = next;
        (*next).Blink = prev;
    }
}

// ---------------------------------------------------------------------------
// HalpAllocPhysicalMemory — walk loader memory descriptors looking for a
// run of `PageCount` free pages below `MaxAddress`, optionally on a 64KB
// alignment boundary.  Carves the run into a new descriptor slot in
// HalpAllocationDescriptorArray and patches the loader list to reflect
// the consumption.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpAllocPhysicalMemory(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    MaxAddress: ULONG64,
    PageCount: PFN_NUMBER,
    Aligned: BOOLEAN,
) -> ULONG64 {
    unsafe {
        let max_page: PFN_NUMBER = (MaxAddress >> PAGE_SHIFT) as PFN_NUMBER;

        // Need room for the new alloc descriptor + a possible leftover.
        let used = core::ptr::read_volatile(&raw const HalpUsedAllocDescriptors);
        if used + 2 > 64 {
            return 0;
        }
        let mut used_descriptors = used;

        let head: PLIST_ENTRY = &raw mut (*LoaderBlock).MemoryDescriptorListHead;
        let mut next_entry: PLIST_ENTRY = (*head).Flink;
        let mut md_block: PMEMORY_ALLOCATION_DESCRIPTOR = core::ptr::null_mut();
        let mut alignment: PFN_NUMBER = 0;
        let mut physical_address: ULONG64 = 0;

        while next_entry != head {
            md_block = list_entry_to_descriptor(next_entry);
            alignment = if Aligned != 0 {
                (((*md_block).BasePage + 0x0F) & !0x0F) - (*md_block).BasePage
            } else {
                0
            };

            let mt = (*md_block).MemoryType;
            if mt == LOADER_FREE || mt == LOADER_FIRMWARE_TEMPORARY {
                let base_page = (*md_block).BasePage;
                if base_page != 0
                    && (*md_block).PageCount >= PageCount + alignment
                    && base_page + PageCount + alignment < max_page
                {
                    physical_address = ((base_page as ULONG64) + (alignment as ULONG64))
                        << PAGE_SHIFT;
                    break;
                }
            }
            next_entry = (*next_entry).Flink;
        }

        if next_entry == head {
            return 0;
        }

        // Carve a NewBlock descriptor for the allocation.
        let new_block: PMEMORY_ALLOCATION_DESCRIPTOR =
            &raw mut HalpAllocationDescriptorArray[used_descriptors as usize];
        (*new_block).PageCount = PageCount as ULONG_PTR;
        (*new_block).BasePage = (*md_block).BasePage + alignment;
        (*new_block).MemoryType = LOADER_HAL_CACHED_MEMORY;
        used_descriptors += 1;
        core::ptr::write_volatile(&raw mut HalpUsedAllocDescriptors, used_descriptors);

        if alignment != 0 {
            // Stash any leftover pages past the allocation in a FreeBlock.
            if (*md_block).PageCount > PageCount + alignment {
                let free_block: PMEMORY_ALLOCATION_DESCRIPTOR =
                    &raw mut HalpAllocationDescriptorArray[used_descriptors as usize];
                (*free_block).PageCount =
                    (*md_block).PageCount - alignment - PageCount;
                (*free_block).BasePage = (*md_block).BasePage + alignment + PageCount;
                core::ptr::write_volatile(
                    &raw mut HalpUsedAllocDescriptors,
                    used_descriptors + 1,
                );
                insert_head_list(
                    &raw mut (*md_block).ListEntry,
                    &raw mut (*free_block).ListEntry,
                );
            }
            // Trim the original block to the alignment-only piece.
            (*md_block).PageCount = alignment;
            insert_head_list(
                &raw mut (*md_block).ListEntry,
                &raw mut (*new_block).ListEntry,
            );
        } else {
            // Consume from the front of the original block.
            (*md_block).BasePage += PageCount;
            (*md_block).PageCount -= PageCount;
            insert_tail_list(
                &raw mut (*md_block).ListEntry,
                &raw mut (*new_block).ListEntry,
            );
            if (*md_block).PageCount == 0 {
                remove_entry_list(&raw mut (*md_block).ListEntry);
            }
        }

        physical_address
    }
}

/// CONTAINING_RECORD(entry, MEMORY_ALLOCATION_DESCRIPTOR, ListEntry)
#[inline(always)]
pub(crate) unsafe fn list_entry_to_descriptor(
    entry: PLIST_ENTRY,
) -> PMEMORY_ALLOCATION_DESCRIPTOR {
    // ListEntry is the first field of MEMORY_ALLOCATION_DESCRIPTOR — offset 0.
    entry as PMEMORY_ALLOCATION_DESCRIPTOR
}

// ---------------------------------------------------------------------------
// Map a run of physical pages into a contiguous virtual range starting at
// HalpHeapStart.  Walks PTEs looking for PageCount consecutive invalid
// entries; on finding a run, fills them with present-write PTEs pointing
// at the supplied physical pages and bumps HalpHeapStart past the
// allocation.  Caller-controlled TLB flush via FlushCurrentTLB.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpMapPhysicalMemory64Vista(
    PhysicalAddress: PHYSICAL_ADDRESS,
    PageCount: PFN_COUNT,
    FlushCurrentTLB: BOOLEAN,
) -> PVOID {
    unsafe {
        let mut used_pages: PFN_COUNT = 0;
        let mut base_address = core::ptr::read_volatile(&raw const HalpHeapStart);
        let mut virtual_address = base_address;

        // Find PageCount consecutive invalid PTEs; on a hit, base/used
        // describe the run we'll map.
        while used_pages < PageCount {
            if virtual_address.is_null() {
                return core::ptr::null_mut();
            }
            let pte = address_to_pte(virtual_address as ULONG_PTR);
            virtual_address = (virtual_address as ULONG_PTR + PAGE_SIZE as ULONG_PTR) as PVOID;
            if (*pte).Valid() != 0 {
                base_address = virtual_address;
                used_pages = 0;
                continue;
            }
            used_pages += 1;
        }

        // Compute the in-page byte offset of the requested physical address.
        let phys_qpart = PhysicalAddress.QuadPart;
        let page_byte_offset = (phys_qpart as ULONG_PTR) & BYTE_OFFSET_MASK;
        let result_va = (base_address as ULONG_PTR + page_byte_offset) as PVOID;

        // Bump the HAL heap if we mapped at the current boundary.
        let heap_now = core::ptr::read_volatile(&raw const HalpHeapStart);
        if base_address == heap_now {
            let new_heap =
                (base_address as ULONG_PTR + (PageCount as ULONG_PTR) * (PAGE_SIZE as ULONG_PTR))
                    as PVOID;
            core::ptr::write_volatile(&raw mut HalpHeapStart, new_heap);
        }

        // Fill the PTEs page-by-page.  The C uses bitfield writes on the
        // HARDWARE_PTE struct — same shape via the bindgen accessors.
        let mut phys = phys_qpart;
        let mut va_iter = base_address;
        let mut pages = used_pages;
        while pages > 0 {
            let pte = address_to_pte(va_iter as ULONG_PTR);
            (*pte).set_PageFrameNumber((phys >> PAGE_SHIFT) as _);
            (*pte).set_Valid(1);
            (*pte).set_Write(1);
            phys += PAGE_SIZE as i64;
            va_iter = (va_iter as ULONG_PTR + PAGE_SIZE as ULONG_PTR) as PVOID;
            pages -= 1;
        }

        if FlushCurrentTLB != 0 {
            HalpFlushTLB();
        }
        result_va
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpMapPhysicalMemory64(
    PhysicalAddress: PHYSICAL_ADDRESS,
    PageCount: PFN_COUNT,
) -> PVOID {
    unsafe { HalpMapPhysicalMemory64Vista(PhysicalAddress, PageCount, 1 /* TRUE */) }
}

// ---------------------------------------------------------------------------
// Unmap — zero out PTEs for the supplied range, optionally flush TLB,
// retract HalpHeapStart if we're unmapping at or below it.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpUnmapVirtualAddressVista(
    VirtualAddress: PVOID,
    PageCount: PFN_COUNT,
    FlushCurrentTLB: BOOLEAN,
) {
    unsafe {
        if (VirtualAddress as ULONG_PTR) < MM_HAL_VA_START {
            return;
        }

        // Page-align down.
        let va = (VirtualAddress as ULONG_PTR) & !BYTE_OFFSET_MASK;

        // Zero PTEs in-place.  C code writes the raw u32/u64 directly; do
        // the same — the bitfield accessors would generate ~3× more code.
        let mut pte_word = address_to_pte(va) as *mut PteWord;
        for _ in 0..PageCount {
            *pte_word = 0;
            pte_word = pte_word.add(1);
        }

        if FlushCurrentTLB != 0 {
            HalpFlushTLB();
        }

        // Retract heap if this unmap freed up pages below the bump pointer.
        let heap_now = core::ptr::read_volatile(&raw const HalpHeapStart);
        let aligned_va = va as PVOID;
        if heap_now > aligned_va {
            core::ptr::write_volatile(&raw mut HalpHeapStart, aligned_va);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpUnmapVirtualAddress(
    VirtualAddress: PVOID,
    PageCount: PFN_COUNT,
) {
    unsafe { HalpUnmapVirtualAddressVista(VirtualAddress, PageCount, 1 /* TRUE */) }
}
