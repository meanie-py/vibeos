//! `alloc` over the process heap. HeapAlloc already aligns to MEMORY_ALLOCATION_ALIGNMENT
//! (8 on i386, 16 on 64-bit); a stricter alignment over-allocates and keeps the heap's own
//! pointer just below the aligned one.

use crate::sys::{GetProcessHeap, HeapAlloc, HeapFree};
use core::alloc::{GlobalAlloc, Layout};

pub struct ProcessHeap;

const HEAP_ALIGN: usize = if cfg!(target_pointer_width = "64") { 16 } else { 8 };

unsafe impl GlobalAlloc for ProcessHeap {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if l.align() <= HEAP_ALIGN {
            return HeapAlloc(GetProcessHeap(), 0, l.size().max(1)) as *mut u8;
        }
        let raw = HeapAlloc(GetProcessHeap(), 0, l.size() + l.align()) as *mut u8;
        if raw.is_null() {
            return raw;
        }
        let aligned = ((raw as usize + l.align()) & !(l.align() - 1)) as *mut u8;
        *(aligned as *mut *mut u8).sub(1) = raw;
        aligned
    }

    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        let raw = if l.align() <= HEAP_ALIGN { p } else { *(p as *mut *mut u8).sub(1) };
        HeapFree(GetProcessHeap(), 0, raw as *mut core::ffi::c_void);
    }
}
