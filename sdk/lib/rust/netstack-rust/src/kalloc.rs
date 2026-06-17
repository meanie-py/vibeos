//! Global allocator over the kernel NonPaged pool.
//!
//! smoltcp (with the `alloc` feature) and the virtio queue bookkeeping need a
//! heap; in netstack.sys that heap is NonPaged pool — the stack is touched at
//! DISPATCH_LEVEL (RX DPC, poll worker), so paged pool is off the table.
//!
//! ExAllocatePoolWithTag only guarantees natural pool alignment (16 bytes on
//! 64-bit, 8 on i386). Layouts needing more get the classic over-allocate +
//! stash-the-real-pointer treatment; DMA buffers do NOT come from here (the
//! virtio Hal does its own contiguous, physically-addressable allocation).

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use core::mem::size_of;

use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};

/// Pool tag 'NetS' (little-endian "SteN" in pool dumps reads as NetS).
const POOL_TAG: u32 = u32::from_le_bytes(*b"NetS");

/// NonPagedPool = 0 (POOL_TYPE).
const NON_PAGED_POOL: i32 = 0;

/// Minimum alignment ExAllocatePool* guarantees on this arch.
#[cfg(target_pointer_width = "64")]
const POOL_ALIGN: usize = 16;
#[cfg(target_pointer_width = "32")]
const POOL_ALIGN: usize = 8;

pub struct KernelPoolAlloc;

unsafe impl GlobalAlloc for KernelPoolAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= POOL_ALIGN {
            return ExAllocatePoolWithTag(
                NON_PAGED_POOL as _, layout.size() as _, POOL_TAG,
            ) as *mut u8;
        }
        // Over-aligned: allocate size + align + one pointer of header space,
        // align up past the header, stash the raw pool pointer just below the
        // returned address so dealloc can recover it.
        let total = layout.size() + layout.align() + size_of::<usize>();
        let raw = ExAllocatePoolWithTag(NON_PAGED_POOL as _, total as _, POOL_TAG) as usize;
        if raw == 0 {
            return core::ptr::null_mut();
        }
        let aligned = (raw + size_of::<usize>() + layout.align() - 1) & !(layout.align() - 1);
        *((aligned - size_of::<usize>()) as *mut usize) = raw;
        aligned as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let raw = if layout.align() <= POOL_ALIGN {
            ptr as usize
        } else {
            *((ptr as usize - size_of::<usize>()) as *const usize)
        };
        ExFreePoolWithTag(raw as *mut c_void, POOL_TAG);
    }
}

#[global_allocator]
static KALLOC: KernelPoolAlloc = KernelPoolAlloc;
