//! Rust impls for the expansion + PFN locks (Phase 1 batch M7).
//!
//! Four wrappers:
//!
//! * `MiAcquireExpansionLock` — `KeAcquireSpinLock(&MmExpansionLock)` (debug
//!   `MiExpansionLockOwner` tracking stays in the C trampoline)
//! * `MiReleaseExpansionLock` — `KeReleaseSpinLock(&MmExpansionLock)`
//! * `MiAcquirePfnLock` — `KeAcquireQueuedSpinLock(LockQueuePfnLock)`
//! * `MiReleasePfnLock` — `KeReleaseQueuedSpinLock(LockQueuePfnLock, _)`
//!
//! Note that PFN lock uses the *queued* spinlock variant (per-CPU queue
//! pointed at by `Prcb->LockQueue[LockQueuePfnLock]`) — different from
//! `inline::mm::MiAcquirePfnLock` which uses `KeAcquireSpinLock` against
//! `MmPfnLock` directly. We replicate the queued-spinlock semantics here
//! to preserve C trampoline behavior. The Expansion lock uses the regular
//! spinlock and just delegates to `inline::mm`.
//!
//! Same augment+isolated-rlib pattern as M1-M6.

#![no_std]

use ntkrnl_sys::ddk::*;

// LockQueuePfnLock = 2 — stable Windows kernel constant. Defined as a
// real enum on i386/arm64 (`_KSPIN_LOCK_QUEUE_NUMBER::LockQueuePfnLock`)
// but on amd64 the type is just `ULONG64`. We pass it through
// `KSPIN_LOCK_QUEUE_NUMBER as _`, letting the bindgen typedef resolve to
// the correct width per arch.
const LOCK_QUEUE_PFN_LOCK_VALUE: u32 = 2;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiAcquireExpansionLock() -> KIRQL {
    ntkrnl_sys::inline::mm::MiAcquireExpansionLock()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiReleaseExpansionLock(old_irql: KIRQL) {
    ntkrnl_sys::inline::mm::MiReleaseExpansionLock(old_irql)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiAcquirePfnLock() -> KIRQL {
    let lock: KSPIN_LOCK_QUEUE_NUMBER = LOCK_QUEUE_PFN_LOCK_VALUE as _;
    KeAcquireQueuedSpinLock(lock)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiReleasePfnLock(old_irql: KIRQL) {
    let lock: KSPIN_LOCK_QUEUE_NUMBER = LOCK_QUEUE_PFN_LOCK_VALUE as _;
    KeReleaseQueuedSpinLock(lock, old_irql)
}
