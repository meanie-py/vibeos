//! Native startup state + lock (startup/natstart.c) and the interlocked
//! helpers the entry/dll init use to serialize first-thread init.

use crate::winapi::*;
use core::ffi::c_void;
use core::sync::atomic::{AtomicPtr, Ordering};

// __enative_startup_state (internal.h): 0 uninitialized, 1 initializing,
// 2 initialized.
pub const NATIVE_UNINITIALIZED: u32 = 0;
pub const NATIVE_INITIALIZING: u32 = 1;
pub const NATIVE_INITIALIZED: u32 = 2;

#[no_mangle]
pub static mut __native_dllmain_reason: u32 = u32::MAX;
#[no_mangle]
pub static mut __native_vcclrit_reason: u32 = u32::MAX;
#[no_mangle]
pub static mut __native_startup_state: u32 = NATIVE_UNINITIALIZED;
#[no_mangle]
pub static mut __native_startup_lock: *mut c_void = core::ptr::null_mut();

#[inline]
fn lock_atomic() -> &'static AtomicPtr<c_void> {
    // __native_startup_lock is pointer-sized; reinterpret its storage as
    // an atomic for the InterlockedCompareExchangePointer/Exchange ops.
    unsafe { &*(core::ptr::addr_of!(__native_startup_lock) as *const AtomicPtr<c_void>) }
}

/// InterlockedCompareExchangePointer(&__native_startup_lock, exch, cmp)
/// → previous value.
#[inline]
pub fn lock_cmpxchg(exch: PVOID, cmp: PVOID) -> PVOID {
    match lock_atomic().compare_exchange(cmp, exch, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(prev) => prev, // == cmp
        Err(prev) => prev,
    }
}

/// InterlockedExchangePointer(&__native_startup_lock, exch).
#[inline]
pub fn lock_xchg(exch: PVOID) -> PVOID {
    lock_atomic().swap(exch, Ordering::SeqCst)
}
