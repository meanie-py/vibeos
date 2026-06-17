//! `virtio_drivers::Hal` implementation over the Windows kernel memory manager.
//!
//! - **DMA rings** (`dma_alloc`): physically-contiguous, sub-4 GiB, zeroed pool
//!   via `MmAllocateContiguousMemorySpecifyCache`. x86 DMA is cache-coherent,
//!   so cached memory is correct (and faster) — no uncached/write-combine need.
//! - **BAR mapping** (`mmio_phys_to_virt`): `MmMapIoSpace`, non-cached.
//! - **share/unshare**: with no IOMMU (QEMU default) and coherent x86 DMA, the
//!   device uses the buffer's physical address directly; unshare is a no-op.
//!   Net tx/rx buffers are sub-page pool allocations, hence contiguous.

use core::ptr::NonNull;

use virtio_drivers::{BufferDirection, Hal, PhysAddr};

use ntkrnl_sys::ddk::{
    LARGE_INTEGER, MmAllocateContiguousMemorySpecifyCache, MmFreeContiguousMemory,
    MmGetPhysicalAddress, MmMapIoSpace, PVOID, _MEMORY_CACHING_TYPE_MmCached as MM_CACHED,
    _MEMORY_CACHING_TYPE_MmNonCached as MM_NON_CACHED,
};

const PAGE_SIZE: usize = 4096;

/// Build a PHYSICAL_ADDRESS (LARGE_INTEGER) from a 64-bit value.
#[inline]
fn phys(v: u64) -> LARGE_INTEGER {
    let mut p: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    p.QuadPart = v as i64;
    p
}

pub struct NetHal;

unsafe impl Hal for NetHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let bytes = pages * PAGE_SIZE;
        // Restrict to below 4 GiB: always DMA-addressable for virtio regardless
        // of whether the device negotiated 64-bit addressing.
        let vaddr = unsafe {
            MmAllocateContiguousMemorySpecifyCache(
                bytes as _,
                phys(0),
                phys(0xFFFF_FFFF),
                phys(0),
                MM_CACHED as _,
            )
        };
        if vaddr.is_null() {
            return (0, NonNull::dangling());
        }
        unsafe {
            core::ptr::write_bytes(vaddr as *mut u8, 0, bytes);
        }
        let paddr = unsafe { MmGetPhysicalAddress(vaddr).QuadPart } as u64;
        (paddr, NonNull::new(vaddr as *mut u8).unwrap())
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, _pages: usize) -> i32 {
        MmFreeContiguousMemory(vaddr.as_ptr() as PVOID);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        let v = MmMapIoSpace(phys(paddr), size as _, MM_NON_CACHED as _);
        NonNull::new(v as *mut u8).expect("MmMapIoSpace failed for virtio BAR")
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let ptr = buffer.as_ptr() as *mut u8 as PVOID;
        MmGetPhysicalAddress(ptr).QuadPart as u64
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Cache-coherent, no bounce buffer — nothing to reconcile.
    }
}
