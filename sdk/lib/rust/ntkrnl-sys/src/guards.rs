//! RAII guard types for kernel state management.
//!
//! These guards encode kernel invariants (IRQL, spinlocks) as visible values
//! that LLVM's optimizer must preserve, preventing miscompilation.
//!
//! # Architecture
//! - Guards use `Drop` to restore state — LLVM cannot eliminate a used value's destructor
//! - Internal primitives use asm barriers to prevent optimization of hardware state transitions
//! - C-facing exports (KeRaiseIrql, KeAcquireSpinLock) remain unchanged for driver compatibility
//!
//! # Execution-context ownership (Layer 1, phase 1)
//! Guards are `!Send + !Sync` (`_ctx: PhantomData<*const ()>`): a raised-IRQL
//! token or a held spinlock certifies "*this* CPU, *this* thread, *now*" —
//! moving one to another thread (or holding it across a context switch on a
//! future work-stealing abstraction) is meaningless and unsound. This is the
//! first step of the RunToken model; the brand lifetime and the CPS wait API
//! arrive with `RunToken` in phase 2. See
//! `docs/engineering/execution-context-ownership-model.md`.

use core::marker::PhantomData;

use crate::base::*;
use crate::ke::PKSPIN_LOCK;

/// `!Send + !Sync` marker: execution-context capabilities must not leave the
/// context that minted them (raw-pointer PhantomData opts out of both).
type NotSend = PhantomData<*const ()>;

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
    _ctx: NotSend,
}

impl IrqlGuard {
    /// Raise IRQL and return a guard that restores it on drop.
    #[inline]
    pub unsafe fn raise(new_irql: KIRQL) -> Self {
        let previous = irql_raise(new_irql);
        Self { previous, _ctx: PhantomData }
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
    _ctx: NotSend,
}

impl SpinLockGuard {
    /// Acquire a spinlock using the kernel's real KeAcquireSpinLock implementation.
    /// This ensures compatibility with queued spinlocks and other kernel-internal
    /// locking mechanisms that may use the same KSPIN_LOCK.
    #[inline]
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self {
        let mut previous_irql: KIRQL = 0;
        crate::ke::KeAcquireSpinLock(lock, &mut previous_irql);
        Self { lock, previous_irql, _ctx: PhantomData }
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
    _ctx: NotSend,
}

impl SpinLockAtDpcGuard {
    /// Acquire a spinlock at DPC level.
    #[inline]
    pub unsafe fn acquire(lock: PKSPIN_LOCK) -> Self {
        crate::ke::KeAcquireSpinLockAtDpcLevel(lock);
        Self { lock, _ctx: PhantomData }
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


