//! `heap_handle.cpp`: the CRT heap handle and its startup/shutdown hooks.

use super::{acrt_heap, GetProcessHeap, Handle};
use core::ptr::null_mut;

/// `extern "C" HANDLE __acrt_heap` -- written only by CRT startup/shutdown.
/// Named and typed exactly as the C: `internal/initialization.cpp` reaches the
/// two functions below by name, `corecrt_internal.h` declares this variable
/// `extern`, and every allocator reads it.
#[no_mangle]
pub static mut __acrt_heap: Handle = null_mut();

/// `bool __acrt_initialize_heap()` -- `internal/initialization.cpp`'s table.
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_heap() -> bool {
    let heap = GetProcessHeap();
    core::ptr::addr_of_mut!(__acrt_heap).write(heap);
    !heap.is_null()
}

/// `bool __acrt_uninitialize_heap(bool terminating)`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_heap(_terminating: bool) -> bool {
    core::ptr::addr_of_mut!(__acrt_heap).write(null_mut());
    true
}

/// `intptr_t _get_heap_handle()`. (The C's `_ASSERTE` is debug-only.)
#[no_mangle]
pub unsafe extern "C" fn _get_heap_handle() -> isize {
    acrt_heap() as isize
}

/// `HANDLE __acrt_getheap()` -- the internal twin returning a HANDLE. No
/// caller remains in this tree; defined because heap_handle.cpp defines it.
#[no_mangle]
pub unsafe extern "C" fn __acrt_getheap() -> Handle {
    acrt_heap()
}
