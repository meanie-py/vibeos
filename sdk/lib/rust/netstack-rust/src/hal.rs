//! `virtio_drivers::Hal` implementation over the Windows kernel memory manager.
//!
//! - **DMA rings** (`dma_alloc`): physically-contiguous, sub-4 GiB, zeroed pool
//!   via `MmAllocateContiguousMemorySpecifyCache`. x86 DMA is cache-coherent,
//!   so cached memory is correct (and faster) — no uncached/write-combine need.
//! - **BAR mapping** (`mmio_phys_to_virt`): `MmMapIoSpace`, non-cached.
//! - **The PCI NIC** (`dma.rs`): once bring-up has its DMA adapter, the rings are
//!   common buffers and every frame is bounced through the adapter's pool, so
//!   behind an IOMMU the device reaches only what its domain holds. The rest of
//!   this file serves the platform NICs (GEM, GENET), which no IOMMU translates.
//! - **share/unshare**: with no IOMMU (QEMU default) and coherent x86 DMA,
//!   single-page buffers use their physical address directly. Buffers that
//!   cross a page boundary are bounced through contiguous DMA memory because
//!   `virtio-drivers` passes only one physical address per descriptor.

use core::ptr::{self, NonNull};

use virtio_drivers::{BufferDirection, Hal, PhysAddr};

use ntkrnl_sys::ddk::{
    LARGE_INTEGER, MmAllocateContiguousMemorySpecifyCache, MmFreeContiguousMemory,
    MmGetPhysicalAddress, MmMapIoSpace, PVOID, _MEMORY_CACHING_TYPE_MmCached as MM_CACHED,
    _MEMORY_CACHING_TYPE_MmNonCached as MM_NON_CACHED,
};
use ntkrnl_sys::guards::SpinLockGuard;
use ntkrnl_sys::ke::KSPIN_LOCK;

const PAGE_SIZE: usize = 4096;
const MAX_BOUNCES: usize = 128;

/// Build a PHYSICAL_ADDRESS (LARGE_INTEGER) from a 64-bit value.
#[inline]
fn phys(v: u64) -> LARGE_INTEGER {
    let mut p: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    p.QuadPart = v as i64;
    p
}

#[derive(Copy, Clone)]
struct BounceEntry {
    original: usize,
    paddr: PhysAddr,
    vaddr: *mut u8,
    len: usize,
}

const EMPTY_BOUNCE: BounceEntry = BounceEntry {
    original: 0,
    paddr: 0,
    vaddr: ptr::null_mut(),
    len: 0,
};

static mut BOUNCES: [BounceEntry; MAX_BOUNCES] = [EMPTY_BOUNCE; MAX_BOUNCES];

/// Guards the `BOUNCES` table scan/mutate. `share`/`unshare` run on the poll
/// worker (PASSIVE→DISPATCH via the virtio queue ops), never the ISR, so a
/// DISPATCH-level KSPIN_LOCK is the right primitive. Was unlocked because "on
/// QEMU's single vCPU map/unmap never overlap" — a UP assumption: under -smp 2
/// two concurrent `share`s could claim the same empty slot (one bounce leaked +
/// a descriptor pointed at the wrong physaddr → DMA to stale memory), or a
/// `share`/`unshare` pair could tear an entry. Real MP spin under feature=smp;
/// no-op fence on UP (§9.55). `0` = the KeInitializeSpinLock initial state.
static mut BOUNCE_LOCK: KSPIN_LOCK = 0;

#[inline]
fn crosses_page(ptr: usize, len: usize) -> bool {
    (ptr & (PAGE_SIZE - 1)) + len > PAGE_SIZE
}

unsafe fn remember_bounce(original: usize, paddr: PhysAddr, vaddr: *mut u8, len: usize) -> bool {
    let _lock = SpinLockGuard::acquire(&raw mut BOUNCE_LOCK);
    let table = (&raw mut BOUNCES).cast::<BounceEntry>();
    for i in 0..MAX_BOUNCES {
        let entry = &mut *table.add(i);
        if entry.original == 0 {
            *entry = BounceEntry {
                original,
                paddr,
                vaddr,
                len,
            };
            return true;
        }
    }
    false
}

unsafe fn take_bounce(original: usize, paddr: PhysAddr) -> Option<BounceEntry> {
    let _lock = SpinLockGuard::acquire(&raw mut BOUNCE_LOCK);
    let table = (&raw mut BOUNCES).cast::<BounceEntry>();
    for i in 0..MAX_BOUNCES {
        let entry = &mut *table.add(i);
        if entry.original == original && entry.paddr == paddr {
            let found = *entry;
            *entry = EMPTY_BOUNCE;
            return Some(found);
        }
    }
    None
}

pub struct NetHal;

unsafe impl Hal for NetHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let bytes = pages * PAGE_SIZE;
        if crate::dma::attached() {
            return match unsafe { crate::dma::alloc(bytes) } {
                Some((va, device)) => (device, NonNull::new(va).unwrap_or(NonNull::dangling())),
                None => (0, NonNull::dangling()),
            };
        }
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

    unsafe fn dma_dealloc(paddr: PhysAddr, vaddr: NonNull<u8>, _pages: usize) -> i32 {
        if crate::dma::free(vaddr.as_ptr(), paddr) {
            return 0;
        }
        MmFreeContiguousMemory(vaddr.as_ptr() as PVOID);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        let v = MmMapIoSpace(phys(paddr), size as _, MM_NON_CACHED as _);
        NonNull::new(v as *mut u8).expect("MmMapIoSpace failed for virtio BAR")
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        if crate::dma::attached() {
            // Every frame through the adapter's pool. 0 (a full pool) the queue op
            // turns into a descriptor nothing can reach; the device then faults on
            // it rather than reach memory its domain never held.
            return crate::dma::share(buffer, direction != BufferDirection::DeviceToDriver).unwrap_or_else(|| {
                crate::serial::emit(format_args!("[NETSTACK] the NIC's bounce pool is full: a frame gets no DMA address\n"));
                0
            });
        }
        let ptr = buffer.as_ptr() as *mut u8 as PVOID;
        let len = buffer.len();
        if len == 0 || !crosses_page(ptr as usize, len) {
            return MmGetPhysicalAddress(ptr).QuadPart as u64;
        }

        let pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
        let bounce = MmAllocateContiguousMemorySpecifyCache(
            (pages * PAGE_SIZE) as _,
            phys(0),
            phys(0xFFFF_FFFF),
            phys(0),
            MM_CACHED as _,
        );
        if bounce.is_null() {
            return 0;
        }

        ptr::write_bytes(bounce as *mut u8, 0, pages * PAGE_SIZE);
        if direction != BufferDirection::DeviceToDriver {
            ptr::copy_nonoverlapping(ptr as *const u8, bounce as *mut u8, len);
        }

        let paddr = MmGetPhysicalAddress(bounce).QuadPart as u64;
        if !remember_bounce(ptr as usize, paddr, bounce as *mut u8, len) {
            MmFreeContiguousMemory(bounce);
            return 0;
        }
        paddr
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, direction: BufferDirection) {
        if crate::dma::unshare(paddr, buffer, direction != BufferDirection::DriverToDevice) {
            return;
        }
        let ptr = buffer.as_ptr() as *mut u8 as PVOID;
        if let Some(entry) = take_bounce(ptr as usize, paddr) {
            if direction != BufferDirection::DriverToDevice {
                ptr::copy_nonoverlapping(entry.vaddr as *const u8, ptr as *mut u8, entry.len);
            }
            MmFreeContiguousMemory(entry.vaddr as PVOID);
        }
    }
}
