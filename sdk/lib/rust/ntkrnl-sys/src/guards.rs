//! RAII guard types for kernel state management.
//!
//! These guards encode kernel invariants (IRQL, spinlocks) as visible values
//! that LLVM's optimizer must preserve, preventing miscompilation.
//!
//! # Architecture
//! - Guards use `Drop` to restore state — LLVM cannot eliminate a used value's destructor
//! - Internal primitives use asm barriers to prevent optimization of hardware state transitions
//! - C-facing exports (KeRaiseIrql, KeAcquireSpinLock) remain unchanged for driver compatibility

use crate::base::*;
use crate::ke::{KSPIN_LOCK, PKSPIN_LOCK};

// ── Sacred IRQL Primitives ──────────────────────────────────────────
// These use asm barriers to prevent LLVM from eliminating IRQL transitions.
// On amd64, IRQL is in CR8 (direct asm access). On ARM64/i386, it goes
// through HAL functions with asm clobbers around the call.

/// Raise IRQL — returns old IRQL. Asm-fenced to survive LLVM optimization.
#[inline(always)]
pub unsafe fn irql_raise(new_irql: KIRQL) -> KIRQL {
    #[cfg(target_arch = "x86_64")]
    {
        let old: u64;
        core::arch::asm!(
            "mov {old}, cr8",
            "mov cr8, {new}",
            old = out(reg) old,
            new = in(reg) new_irql as u64,
        );
        old as KIRQL
    }
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
        core::arch::asm!("", options(nostack));
        let old = KfRaiseIrql(new_irql);
        core::arch::asm!("", options(nostack));
        old
    }
    #[cfg(target_arch = "x86")]
    {
        extern "fastcall" { fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL; }
        core::arch::asm!("", options(nostack));
        let old = KfRaiseIrql(new_irql);
        core::arch::asm!("", options(nostack));
        old
    }
}

/// Lower IRQL. Asm-fenced.
#[inline(always)]
pub unsafe fn irql_lower(new_irql: KIRQL) {
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!("mov cr8, {}", in(reg) new_irql as u64);
    }
    #[cfg(target_arch = "aarch64")]
    {
        extern "C" { fn KfLowerIrql(NewIrql: KIRQL); }
        core::arch::asm!("", options(nostack));
        KfLowerIrql(new_irql);
        core::arch::asm!("", options(nostack));
    }
    #[cfg(target_arch = "x86")]
    {
        extern "fastcall" { fn KfLowerIrql(NewIrql: KIRQL); }
        core::arch::asm!("", options(nostack));
        KfLowerIrql(new_irql);
        core::arch::asm!("", options(nostack));
    }
}

/// Read current IRQL. Asm-fenced.
#[inline(always)]
pub unsafe fn irql_current() -> KIRQL {
    #[cfg(target_arch = "x86_64")]
    {
        let val: u64;
        core::arch::asm!("mov {}, cr8", out(reg) val, options(nomem, nostack));
        val as KIRQL
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        extern "system" { fn KeGetCurrentIrql() -> KIRQL; }
        core::arch::asm!("", options(nostack));
        let r = KeGetCurrentIrql();
        core::arch::asm!("", options(nostack));
        r
    }
}


// ── IrqlGuard ───────────────────────────────────────────────────────

/// RAII guard that restores IRQL when dropped.
///
/// ```rust,ignore
/// let _irql = unsafe { IrqlGuard::raise(DISPATCH_LEVEL) };
/// // ... code at DISPATCH_LEVEL ...
/// // IRQL automatically restored when _irql goes out of scope
/// ```
pub struct IrqlGuard {
    previous: KIRQL,
}

impl IrqlGuard {
    /// Raise IRQL and return a guard that restores it on drop.
    #[inline]
    pub unsafe fn raise(new_irql: KIRQL) -> Self {
        let previous = irql_raise(new_irql);
        Self { previous }
    }

    /// Raise to DISPATCH_LEVEL.
    #[inline]
    pub unsafe fn raise_to_dpc() -> Self {
        Self::raise(DISPATCH_LEVEL)
    }

    /// Raise to APC_LEVEL.
    #[inline]
    pub unsafe fn raise_to_apc() -> Self {
        Self::raise(APC_LEVEL)
    }

    /// Get the saved (previous) IRQL.
    #[inline]
    pub fn previous(&self) -> KIRQL {
        self.previous
    }
}

impl Drop for IrqlGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { irql_lower(self.previous) }
    }
}


// ── SpinLockGuard ───────────────────────────────────────────────────

/// RAII guard for kernel spinlocks. Raises IRQL to DISPATCH on acquire,
/// restores on drop.
///
/// ```rust,ignore
/// let _lock = unsafe { SpinLockGuard::acquire(&raw mut MyLock) };
/// // ... critical section at DISPATCH_LEVEL ...
/// // lock released + IRQL restored when _lock goes out of scope
/// ```
pub struct SpinLockGuard {
    lock: PKSPIN_LOCK,
    previous_irql: KIRQL,
}

impl SpinLockGuard {
    /// Acquire a spinlock using the kernel's real KeAcquireSpinLock implementation.
    /// This ensures compatibility with queued spinlocks and other kernel-internal
    /// locking mechanisms that may use the same KSPIN_LOCK.
    #[inline]
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self {
        let mut previous_irql: KIRQL = 0;
        crate::ke::KeAcquireSpinLock(lock, &mut previous_irql);
        Self { lock, previous_irql }
    }

    /// Get the IRQL that was active before lock acquisition.
    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }
}

impl Drop for SpinLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            crate::ke::KeReleaseSpinLock(self.lock, self.previous_irql);
        }
    }
}

// ── SpinLockAtDpcGuard ──────────────────────────────────────────────

/// RAII guard for kernel spinlocks when ALREADY at DISPATCH_LEVEL or higher.
/// Does not change IRQL.
pub struct SpinLockAtDpcGuard {
    lock: PKSPIN_LOCK,
}

impl SpinLockAtDpcGuard {
    /// Acquire a spinlock at DPC level.
    #[inline]
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self {
        crate::ke::KeAcquireSpinLockAtDpcLevel(lock);
        Self { lock }
    }
}

impl Drop for SpinLockAtDpcGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            crate::ke::KeReleaseSpinLockFromDpcLevel(self.lock);
        }
    }
}


// ── DispatcherLockGuard ─────────────────────────────────────────────

/// RAII guard for the kernel dispatcher lock (KiAcquireDispatcherLock/KiReleaseDispatcherLock).
/// Raises to SYNCH_LEVEL on acquire, lowers on drop.
pub struct DispatcherLockGuard {
    previous_irql: KIRQL,
}

impl DispatcherLockGuard {
    #[inline]
    pub unsafe fn acquire() -> Self {
        let previous_irql = crate::inline::ke::KiAcquireDispatcherLock();
        Self { previous_irql }
    }

    /// Wrap a raw IRQL value (from a prior `KiAcquireDispatcherLock` call) into
    /// a guard. The caller must ensure the dispatcher lock is currently held and
    /// `previous_irql` is the value returned by the acquire call.
    #[inline]
    pub unsafe fn from_raw(previous_irql: KIRQL) -> Self {
        Self { previous_irql }
    }

    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }
}

impl Drop for DispatcherLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::inline::ke::KiReleaseDispatcherLock(self.previous_irql) }
    }
}


// ── QueuedSpinLockGuard ─────────────────────────────────────────────

/// RAII guard for queued spinlocks (KeAcquireQueuedSpinLock/KeReleaseQueuedSpinLock).
pub struct QueuedSpinLockGuard {
    number: u32,
    previous_irql: KIRQL,
}

impl QueuedSpinLockGuard {
    #[inline]
    pub unsafe fn acquire(number: u32) -> Self {
        let previous_irql = crate::ke::KeAcquireQueuedSpinLock(number);
        Self { number, previous_irql }
    }

    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }

    /// Wrap an already-acquired queued spinlock into a guard.
    #[inline]
    pub unsafe fn from_raw(number: u32, previous_irql: KIRQL) -> Self {
        Self { number, previous_irql }
    }

    /// Consume the guard without releasing the lock. Returns the saved IRQL.
    #[inline]
    pub fn into_raw(self) -> KIRQL {
        let irql = self.previous_irql;
        core::mem::forget(self);
        irql
    }
}

impl Drop for QueuedSpinLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::ke::KeReleaseQueuedSpinLock(self.number, self.previous_irql) }
    }
}


// ── CancelSpinLockGuard ─────────────────────────────────────────────

/// RAII guard for the IO cancel spinlock (IoAcquireCancelSpinLock/IoReleaseCancelSpinLock).
pub struct CancelSpinLockGuard {
    previous_irql: KIRQL,
}

impl CancelSpinLockGuard {
    #[inline]
    pub unsafe fn acquire() -> Self {
        let mut previous_irql: KIRQL = 0;
        crate::ddk::IoAcquireCancelSpinLock(&mut previous_irql);
        Self { previous_irql }
    }

    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }
}

impl Drop for CancelSpinLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::ddk::IoReleaseCancelSpinLock(self.previous_irql) }
    }
}


// ── VpbSpinLockGuard ────────────────────────────────────────────────

/// RAII guard for the VPB spinlock (IoAcquireVpbSpinLock/IoReleaseVpbSpinLock).
pub struct VpbSpinLockGuard {
    previous_irql: KIRQL,
}

impl VpbSpinLockGuard {
    #[inline]
    pub unsafe fn acquire() -> Self {
        let mut previous_irql: KIRQL = 0;
        crate::ddk::IoAcquireVpbSpinLock(&mut previous_irql);
        Self { previous_irql }
    }

    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }
}

impl Drop for VpbSpinLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::ddk::IoReleaseVpbSpinLock(self.previous_irql) }
    }
}


// ── ExpansionLockGuard ──────────────────────────────────────────────

/// RAII guard for the MM expansion lock (MiAcquireExpansionLock/MiReleaseExpansionLock).
pub struct ExpansionLockGuard {
    previous_irql: KIRQL,
}

impl ExpansionLockGuard {
    #[inline]
    pub unsafe fn acquire() -> Self {
        let previous_irql = crate::inline::mm::MiAcquireExpansionLock();
        Self { previous_irql }
    }

    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }
}

impl Drop for ExpansionLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::inline::mm::MiReleaseExpansionLock(self.previous_irql) }
    }
}

