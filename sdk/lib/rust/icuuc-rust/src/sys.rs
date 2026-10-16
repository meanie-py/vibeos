//! Allocation seam: process heap on Windows, std allocator under host tests.
#[cfg(not(test))]
mod imp {
    use core::ffi::c_void;
    extern "system" {
        fn GetProcessHeap() -> *mut c_void;
        fn HeapAlloc(h: *mut c_void, flags: u32, size: usize) -> *mut c_void;
        fn HeapFree(h: *mut c_void, flags: u32, p: *mut c_void) -> i32;
    }
    pub unsafe fn alloc<T>() -> *mut T {
        HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<T>()) as *mut T
    }
    pub unsafe fn free<T>(p: *mut T) {
        HeapFree(GetProcessHeap(), 0, p as *mut c_void);
    }
}
#[cfg(test)]
mod imp {
    pub unsafe fn alloc<T>() -> *mut T {
        std::alloc::alloc(std::alloc::Layout::new::<T>()) as *mut T
    }
    pub unsafe fn free<T>(p: *mut T) {
        std::alloc::dealloc(p as *mut u8, std::alloc::Layout::new::<T>())
    }
}
pub use imp::*;
