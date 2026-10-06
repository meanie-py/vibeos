//! `virtio_drivers::Hal` over the kernel memory manager.
//!
//! The data path never hands virtio-drivers a caller's buffer: each adapter owns a
//! physically contiguous ARENA, the miniport copies through it, and every slice of an
//! arena resolves to `arena_phys + offset` with no translation. The only other memory
//! virtio-drivers shares is its own request header and status byte, which live on the
//! kernel stack; those are page-translated directly, and bounced through a small
//! contiguous buffer in the rare case one straddles a page (one physical address per
//! descriptor, so a straddling buffer would DMA half into the wrong page).
//!
//! x86 DMA is cache-coherent and QEMU has no IOMMU by default, so cached memory and
//! physical addresses are correct as they are.

use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicUsize, Ordering};

use virtio_drivers::{BufferDirection, Hal, PhysAddr};

use ntkrnl_sys::ddk::{
    MmAllocateContiguousMemorySpecifyCache, MmFreeContiguousMemory, MmGetPhysicalAddress,
    MmMapIoSpace, LARGE_INTEGER, PVOID, _MEMORY_CACHING_TYPE_MmCached as MM_CACHED,
    _MEMORY_CACHING_TYPE_MmNonCached as MM_NON_CACHED,
};
use ntkrnl_sys::guards::SpinLockGuard;
use ntkrnl_sys::ke::KSPIN_LOCK;

pub const PAGE_SIZE: usize = 4096;

#[inline]
fn phys(v: u64) -> LARGE_INTEGER {
    let mut p: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    p.QuadPart = v as i64;
    p
}

/// Physically contiguous, zeroed, below 4 GiB (addressable whatever the device
/// negotiated). Null on failure.
pub fn alloc_contiguous(bytes: usize) -> (*mut u8, u64) {
    let va = unsafe {
        MmAllocateContiguousMemorySpecifyCache(
            bytes as _,
            phys(0),
            phys(0xFFFF_FFFF),
            phys(0),
            MM_CACHED as _,
        )
    } as *mut u8;
    if va.is_null() {
        return (ptr::null_mut(), 0);
    }
    unsafe {
        ptr::write_bytes(va, 0, bytes);
        (va, MmGetPhysicalAddress(va as PVOID).QuadPart as u64)
    }
}

// ---- arenas: registered once per adapter in HwFindAdapter, then read-only ----------

const MAX_ARENAS: usize = 16;
static ARENA_COUNT: AtomicUsize = AtomicUsize::new(0);
static mut ARENAS: [(usize, u64, usize); MAX_ARENAS] = [(0, 0, 0); MAX_ARENAS];

/// Record a contiguous arena so `share` can resolve slices of it. False when full.
pub fn register_arena(va: *mut u8, pa: u64, len: usize) -> bool {
    let i = ARENA_COUNT.load(Ordering::Acquire);
    if i >= MAX_ARENAS {
        return false;
    }
    unsafe {
        (*(&raw mut ARENAS))[i] = (va as usize, pa, len);
    }
    ARENA_COUNT.store(i + 1, Ordering::Release);
    true
}

fn arena_phys(addr: usize, len: usize) -> Option<u64> {
    let n = ARENA_COUNT.load(Ordering::Acquire);
    let arenas = unsafe { &*(&raw const ARENAS) };
    arenas[..n]
        .iter()
        .find(|&&(va, _, alen)| addr >= va && addr + len <= va + alen)
        .map(|&(va, pa, _)| pa + (addr - va) as u64)
}

// ---- bounce for small page-straddling buffers ---------------------------------------
//
// Only virtio-drivers' own request header (16 bytes) and status byte ever reach this,
// and HwStartIo runs at DISPATCH_LEVEL or above, where contiguous memory can be neither
// allocated nor freed (MmFreeContiguousMemory requires < DISPATCH). So the slots are
// carved from ONE page allocated at HwFindAdapter time, at PASSIVE_LEVEL.

const SLOT: usize = 64;
const SLOTS: usize = PAGE_SIZE / SLOT;
static POOL_VA: AtomicUsize = AtomicUsize::new(0);
static mut POOL_PA: u64 = 0;
static mut POOL_OWNER: [usize; SLOTS] = [0; SLOTS];
/// `share`/`unshare` run inside HwStartIo; two adapters can be serviced on two CPUs.
static mut BOUNCE_LOCK: KSPIN_LOCK = 0;

/// Allocate the bounce page once (PASSIVE_LEVEL). False if it cannot be allocated.
pub fn init_bounce_pool() -> bool {
    if POOL_VA.load(Ordering::Acquire) != 0 {
        return true;
    }
    let (va, pa) = alloc_contiguous(PAGE_SIZE);
    if va.is_null() {
        return false;
    }
    unsafe {
        POOL_PA = pa;
    }
    POOL_VA.store(va as usize, Ordering::Release);
    true
}

#[inline]
fn crosses_page(addr: usize, len: usize) -> bool {
    (addr & (PAGE_SIZE - 1)) + len > PAGE_SIZE
}

pub struct VioHal;

unsafe impl Hal for VioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let (va, pa) = alloc_contiguous(pages * PAGE_SIZE);
        match NonNull::new(va) {
            Some(nn) => (pa, nn),
            None => (0, NonNull::dangling()),
        }
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, _pages: usize) -> i32 {
        MmFreeContiguousMemory(vaddr.as_ptr() as PVOID);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, size: usize) -> NonNull<u8> {
        let v = MmMapIoSpace(phys(paddr), size as _, MM_NON_CACHED as _);
        NonNull::new(v as *mut u8).expect("MmMapIoSpace failed for a virtio-blk BAR")
    }

    unsafe fn share(buffer: NonNull<[u8]>, direction: BufferDirection) -> PhysAddr {
        let addr = buffer.as_ptr() as *mut u8 as usize;
        let len = buffer.len();
        if let Some(pa) = arena_phys(addr, len) {
            return pa;
        }
        if len == 0 || !crosses_page(addr, len) {
            return MmGetPhysicalAddress(addr as PVOID).QuadPart as u64;
        }
        assert!(len <= SLOT, "vioblk: {len}-byte page-straddling buffer outside the arena");
        let pool = POOL_VA.load(Ordering::Acquire);
        assert!(pool != 0, "vioblk: bounce pool not initialised");
        let _lock = SpinLockGuard::acquire(&raw mut BOUNCE_LOCK);
        let owners = &mut *(&raw mut POOL_OWNER);
        let i = owners.iter().position(|&o| o == 0).expect("vioblk: bounce pool exhausted");
        owners[i] = addr;
        let va = (pool + i * SLOT) as *mut u8;
        if direction != BufferDirection::DeviceToDriver {
            ptr::copy_nonoverlapping(addr as *const u8, va, len);
        }
        POOL_PA + (i * SLOT) as u64
    }

    unsafe fn unshare(paddr: PhysAddr, buffer: NonNull<[u8]>, direction: BufferDirection) {
        let addr = buffer.as_ptr() as *mut u8 as usize;
        let len = buffer.len();
        let pool = POOL_VA.load(Ordering::Acquire);
        if pool == 0 || paddr < POOL_PA || paddr >= POOL_PA + PAGE_SIZE as u64 {
            return; // arena slice or direct page: nothing was copied
        }
        let i = ((paddr - POOL_PA) as usize) / SLOT;
        if direction != BufferDirection::DriverToDevice {
            ptr::copy_nonoverlapping((pool + i * SLOT) as *const u8, addr as *mut u8, len);
        }
        let _lock = SpinLockGuard::acquire(&raw mut BOUNCE_LOCK);
        (*(&raw mut POOL_OWNER))[i] = 0;
    }
}
