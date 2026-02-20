//! RTL Interlocked Routines — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/interlck.c` (26 lines).
//! One function: `RtlInterlockedCompareExchange64`.
//! Atomic compare-and-exchange on 64-bit values.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

use core::sync::atomic::{AtomicI64, Ordering};

/// Atomic compare-exchange on a 64-bit value.
///
/// If `*destination == comparand`, sets `*destination = exchange`.
/// Returns the original value at `*destination` regardless.
///
/// # Safety
/// `destination` must point to a valid, 8-byte-aligned i64.
#[inline]
pub unsafe fn interlocked_compare_exchange_64(
    destination: *mut i64,
    exchange: i64,
    comparand: i64,
) -> i64 {
    let atomic = unsafe { &*(destination as *const AtomicI64) };
    match atomic.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) | Err(old) => old,
    }
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlInterlockedCompareExchange64(
        destination: *mut i64,
        exchange: i64,
        comparand: i64,
    ) -> i64 {
        unsafe { interlocked_compare_exchange_64(destination, exchange, comparand) }
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cas_success() {
        let mut val: i64 = 42;
        let old = unsafe { interlocked_compare_exchange_64(&mut val, 100, 42) };
        assert_eq!(old, 42);
        assert_eq!(val, 100);
    }

    #[test]
    fn test_cas_failure() {
        let mut val: i64 = 42;
        let old = unsafe { interlocked_compare_exchange_64(&mut val, 100, 99) };
        assert_eq!(old, 42);
        assert_eq!(val, 42); // unchanged
    }

    #[test]
    fn test_cas_zero() {
        let mut val: i64 = 0;
        let old = unsafe { interlocked_compare_exchange_64(&mut val, 1, 0) };
        assert_eq!(old, 0);
        assert_eq!(val, 1);
    }

    #[test]
    fn test_cas_negative() {
        let mut val: i64 = -1;
        let old = unsafe { interlocked_compare_exchange_64(&mut val, i64::MIN, -1) };
        assert_eq!(old, -1);
        assert_eq!(val, i64::MIN);
    }

    #[test]
    fn test_cas_large_values() {
        let mut val: i64 = i64::MAX;
        let old = unsafe { interlocked_compare_exchange_64(&mut val, i64::MIN, i64::MAX) };
        assert_eq!(old, i64::MAX);
        assert_eq!(val, i64::MIN);
    }
}
