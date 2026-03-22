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
    /// Acquire a spinlock, raising IRQL to DISPATCH_LEVEL.
    #[inline]
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self {
        // Raise IRQL first, then spin on the lock
        let previous_irql = irql_raise(DISPATCH_LEVEL);

        // Atomic acquire — spin until we get it
        let lock_val = lock as *mut core::sync::atomic::AtomicUsize;
        while (*lock_val).compare_exchange_weak(
            0, 1,
            core::sync::atomic::Ordering::Acquire,
            core::sync::atomic::Ordering::Relaxed,
        ).is_err() {
            while (*lock_val).load(core::sync::atomic::Ordering::Relaxed) != 0 {
                core::hint::spin_loop();
            }
        }

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
            // Release the spinlock
            let lock_val = self.lock as *mut core::sync::atomic::AtomicUsize;
            (*lock_val).store(0, core::sync::atomic::Ordering::Release);

            // Lower IRQL back
            irql_lower(self.previous_irql);
        }
    }
}


// ── PfnLockGuard ────────────────────────────────────────────────────

/// Specialized guard for the PFN database lock (MmPfnLock).
/// This is the most frequently used lock in MM code.
///
/// ```rust,ignore
/// let _pfn = unsafe { PfnLockGuard::acquire() };
/// // ... PFN database operations at DISPATCH_LEVEL ...
/// ```
pub struct PfnLockGuard {
    previous_irql: KIRQL,
}

impl PfnLockGuard {
    /// Acquire the PFN lock.
    #[inline]
    pub unsafe fn acquire() -> Self {
        // Use the existing MiAcquirePfnLock which handles the
        // queued spinlock implementation
        extern "system" {
            fn MiAcquirePfnLock() -> KIRQL;
        }
        core::arch::asm!("", options(nostack));
        let previous_irql = MiAcquirePfnLock();
        core::arch::asm!("", options(nostack));
        Self { previous_irql }
    }

    /// Get the IRQL before lock acquisition.
    #[inline]
    pub fn previous_irql(&self) -> KIRQL {
        self.previous_irql
    }
}

impl Drop for PfnLockGuard {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            extern "system" {
                fn MiReleasePfnLock(OldIrql: KIRQL);
            }
            core::arch::asm!("", options(nostack));
            MiReleasePfnLock(self.previous_irql);
            core::arch::asm!("", options(nostack));
        }
    }
}


// ── Constants ───────────────────────────────────────────────────────

pub const PASSIVE_LEVEL: KIRQL = 0;
pub const APC_LEVEL: KIRQL = 1;
pub const DISPATCH_LEVEL: KIRQL = 2;
