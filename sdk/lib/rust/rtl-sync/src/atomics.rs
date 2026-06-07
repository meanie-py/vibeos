//! Atomic-pointer intrinsics for the SRW-lock / condition-variable translations.
//!
//! ntkrnl-sys surfaces the base seq-cst CAS/Exchange primitives; the
//! pointer-width OR/AND/bit-test helpers and the acquire/release CAS variants
//! that srw.c / condvar.c spell out as `Interlocked*Pointer` macros live here.
//!
//! On x86/x64/arm64 every `core::sync::atomic` RMW we use is sequentially
//! consistent, so the `*Acquire` / `*Release` ordered variants simply alias the
//! base seq-cst operation (a stronger ordering is always a legal substitute).
use core::ffi::c_long;
use core::sync::atomic::{AtomicI32, AtomicIsize, Ordering};
use ntkrnl_sys::base::*;

// YieldProcessor — `pause` (x86) / `yield` (arm64); reuse ntkrnl-sys' inline.
pub use ntkrnl_sys::inline::ke::YieldProcessor;

/// `InterlockedOr` (32-bit) — atomic OR, returns the OLD value.
#[inline]
pub unsafe fn InterlockedOr(Target: *mut LONG, Value: LONG) -> LONG {
    (*(Target as *const AtomicI32)).fetch_or(Value, Ordering::SeqCst)
}

/// `InterlockedOrPointer` — pointer-width atomic OR, returns the OLD value.
#[inline]
pub unsafe fn InterlockedOrPointer(Target: *mut PVOID, Value: LONG_PTR) -> LONG_PTR {
    (*(Target as *const AtomicIsize)).fetch_or(Value as isize, Ordering::SeqCst) as LONG_PTR
}

/// `InterlockedAndPointer` — pointer-width atomic AND, returns the OLD value.
#[inline]
pub unsafe fn InterlockedAndPointer(Target: *mut PVOID, Value: LONG_PTR) -> LONG_PTR {
    (*(Target as *const AtomicIsize)).fetch_and(Value as isize, Ordering::SeqCst) as LONG_PTR
}

/// `InterlockedBitTestAndSetPointer` — set bit `Bit` in the lock word, return
/// the previous value of that bit (0/1).
///
/// The SRW code only ever touches `RTL_SRWLOCK_OWNED_BIT` (== 0), so a 32-bit
/// bit-test-and-set on the low dword is equivalent (little-endian) to the
/// `InterlockedBitTestAndSet64` the C `#ifdef _WIN64` path expands to.
#[inline]
pub unsafe fn InterlockedBitTestAndSetPointer(Base: *mut c_long, Bit: c_long) -> c_long {
    let mask = 1i32 << Bit;
    let old = (*(Base as *const AtomicI32)).fetch_or(mask, Ordering::SeqCst);
    if old & mask != 0 { 1 } else { 0 }
}

/// `InterlockedCompareExchangePointerAcquire` — seq-cst aliases acquire on x86/arm64.
#[inline]
pub unsafe fn InterlockedCompareExchangePointerAcquire(
    Destination: *mut PVOID,
    Exchange: PVOID,
    Comparand: PVOID,
) -> PVOID {
    ntkrnl_sys::interlocked::InterlockedCompareExchangePointer(Destination, Exchange, Comparand)
}

/// `InterlockedCompareExchangePointerRelease` — seq-cst aliases release on x86/arm64.
#[inline]
pub unsafe fn InterlockedCompareExchangePointerRelease(
    Destination: *mut PVOID,
    Exchange: PVOID,
    Comparand: PVOID,
) -> PVOID {
    ntkrnl_sys::interlocked::InterlockedCompareExchangePointer(Destination, Exchange, Comparand)
}
