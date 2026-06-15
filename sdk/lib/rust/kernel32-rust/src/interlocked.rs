//! interlocked.rs — the Interlocked* intrinsics kernel32 must provide as real
//! symbols.
//!
//! On i386 the kernel32.spec lists InterlockedCompareExchange / Decrement /
//! Exchange / ExchangeAdd / Increment with `-arch=i386` and **no forward target**
//! — so kernel32.dll itself owns them (MSVC got the bodies from `<intrin.h>`
//! intrinsics; the Rust translation declares them `extern "system"`, so they need
//! real `_Name@N` stdcall definitions here).  amd64/arm64 inline these from
//! intrinsics and the spec does not export them, so the 32-bit-value family is
//! gated to x86.
//!
//! `InterlockedExchangePointer` is **not** a spec export on any arch (it is a
//! FORCEINLINE wrapper in winnt.h); except.rs declares it `extern "system"`, so we
//! provide it on every arch — it can't clash with a forwarder export.
//!
//! Semantics (MSVC):
//!   - CompareExchange / Exchange / ExchangeAdd / ExchangePointer → return the OLD value
//!   - Increment / Decrement                                      → return the NEW value

use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, AtomicPtr, Ordering};

// ---- 32-bit-value family: i386-only (spec -arch=i386, no forward target) ----

/// `InterlockedCompareExchange(Destination, Exchange, Comperand)` → OLD value.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn InterlockedCompareExchange(
    destination: *mut i32,
    exchange: i32,
    comperand: i32,
) -> i32 {
    let a = &*(destination as *const AtomicI32);
    match a.compare_exchange(comperand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) | Err(old) => old,
    }
}

/// `InterlockedDecrement(Addend)` → NEW value.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn InterlockedDecrement(addend: *mut i32) -> i32 {
    let a = &*(addend as *const AtomicI32);
    a.fetch_sub(1, Ordering::SeqCst).wrapping_sub(1)
}

/// `InterlockedIncrement(Addend)` → NEW value.
/// Provided on all arches: i386 spec-export has no forward target, and
/// amd64/arm64 callers (npipe.rs) reference it as an undecorated symbol that the
/// inlined-on-those-arches C never emits. Not a spec export on amd64/arm64, so
/// this cannot clash with a forwarder.
#[no_mangle]
pub unsafe extern "system" fn InterlockedIncrement(addend: *mut i32) -> i32 {
    let a = &*(addend as *const AtomicI32);
    a.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

/// `InterlockedExchange(Target, Value)` → OLD value.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn InterlockedExchange(target: *mut i32, value: i32) -> i32 {
    let a = &*(target as *const AtomicI32);
    a.swap(value, Ordering::SeqCst)
}

/// `InterlockedExchangeAdd(Addend, Value)` → OLD value.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn InterlockedExchangeAdd(addend: *mut i32, value: i32) -> i32 {
    let a = &*(addend as *const AtomicI32);
    a.fetch_add(value, Ordering::SeqCst)
}

// ---- pointer-sized exchange: all arches (FORCEINLINE wrapper, not exported) ----

/// `InterlockedExchangePointer(Target, Value)` → OLD value.
#[no_mangle]
pub unsafe extern "system" fn InterlockedExchangePointer(
    target: *mut *mut c_void,
    value: *mut c_void,
) -> *mut c_void {
    let a = &*(target as *const AtomicPtr<c_void>);
    a.swap(value, Ordering::SeqCst)
}
