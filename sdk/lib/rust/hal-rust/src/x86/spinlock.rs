//! Spinlock support — exports the HAL flavor of the kernel spinlock /
//! queued-spinlock / CMOS-hardware-lock primitives.
//!
//! Translated from `hal/halx86/generic/spinlock.c`, whose first comment is
//! "This file is compiled twice. Once for UP and once for MP" — and since the
//! halmp.dll twin landed, so is this crate: `hal-cargo` builds hal.dll and
//! `halmp-cargo` builds halmp.dll from these same sources, one `smp` feature
//! apart.  So the `KxAcquireSpinLock` / `KxReleaseSpinLock` bodies below must
//! track BOTH axes of `ntoskrnl/include/internal/spinlock.h`:
//!
//!   * `CONFIG_SMP` — the interlocked acquire loop, on for halmp only;
//!   * `KI_TRACK_SPINLOCK_OWNER` (plain `#if DBG`) — the owner stamp, which
//!     the C half writes on EVERY checked build, UP included, and which the
//!     release then clears (`CONFIG_SMP || KI_TRACK_SPINLOCK_OWNER`).
//!
//! Hence: no private copy.  Both funnel straight into
//! `ntkrnl_sys::inline::ke`, the same implementation ntoskrnl-rust uses, so
//! the kernel and HAL halves cannot drift.  They already had: this file used
//! to hard-code a bare compiler fence for both, under a docstring asserting
//! "we're a UP-only build".  The kernel stamped locks that the HAL never
//! cleared, and on halmp a real interlocked `KeTryToAcquireSpinLockAtDpcLevel`
//! got paired with a release that did nothing — so the i386 PFN queued lock
//! could be try-acquired exactly once per boot and failed forever after
//! (kmtest KeSpinLock runs 22-26).  Worse than the test row: on i386 the whole
//! `Ke*SpinLock*` surface is a HAL export, so halmp offered no mutual
//! exclusion at all beyond the IRQL raise.
//!
//! The queued-spinlock variants still degenerate to plain IRQL raise/lower
//! with the lock-walk skipped — that HACK is upstream's, and unchanged here.
//!
//! The two `Halp*CmosSpinLock` exports are compiled for both i386 and
//! amd64 — they're the CMOS spinlock that `cmos.rs`, `spinlock.c`,
//! `timer.c`, `rtctimer.c` use.  The full `Ke*SpinLock*` /
//! `Ke*QueuedSpinLock*` FASTCALL surface is i386-only (amd64 has a
//! parallel set in `apic/apic.c` / `apic/halinit_xpc.c`).

use ntkrnl_sys::hal_ddk::{KIRQL, KSPIN_LOCK, PKSPIN_LOCK, ULONG_PTR};

// ---------------------------------------------------------------------------
// Globals — externally referenced by the kernel and the i386 spinlock fns
// below.  Symbol layout matches the C definitions in spinlock.c.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpSystemHardwareFlags: ULONG_PTR = 0;

#[unsafe(no_mangle)]
pub static mut HalpSystemHardwareLock: KSPIN_LOCK = 0;

// ---------------------------------------------------------------------------
// __readeflags / __writeeflags / _disable — local replicas of the MSVC
// intrinsics.  In our UP + DUMMY_PSEH config they only matter for the CMOS
// fns, but writing them once keeps spinlock.rs self-contained.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: u32;
    unsafe { core::arch::asm!("pushfd", "pop {:e}", out(reg) f, options(nomem, preserves_flags)); }
    f as ULONG_PTR
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: u64;
    unsafe { core::arch::asm!("pushfq", "pop {}", out(reg) f, options(nomem, preserves_flags)); }
    f as ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    unsafe { core::arch::asm!("push {:e}", "popfd", in(reg) f as u32, options(nomem)); }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    unsafe { core::arch::asm!("push {}", "popfq", in(reg) f as u64, options(nomem)); }
}

#[inline(always)]
unsafe fn disable_interrupts() {
    unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
}

// ---------------------------------------------------------------------------
// KxAcquireSpinLock / KxReleaseSpinLock — delegate to the kernel's copy so
// there is exactly one implementation of the funnel (see the module docs).
// hal-rust's `smp` feature forwards to `ntkrnl-sys/smp`; `dbg` is on by
// default there, mirroring the C tree's global -DDBG=1.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "minihal"))]
#[inline(always)]
pub(crate) unsafe fn kx_acquire_spinlock(lock: PKSPIN_LOCK) {
    unsafe { ntkrnl_sys::inline::ke::KxAcquireSpinLock(lock as *mut _) }
}

#[cfg(not(feature = "minihal"))]
#[inline(always)]
pub(crate) unsafe fn kx_release_spinlock(lock: PKSPIN_LOCK) {
    unsafe { ntkrnl_sys::inline::ke::KxReleaseSpinLock(lock as *mut _) }
}

// FreeLDR's loader-time mini-HAL keeps the bare fence. The owner stamp reads
// KeGetCurrentThread(), and pre-kernel there is no KPCR to read it out of;
// the C mini_hal's own KefAcquireSpinLockAtDpcLevel stub is the only place
// upstream writes the stamp there, and this port already declined to mirror
// it (see that fn below). Single-threaded by construction, so the acquire has
// nothing to exclude either way.
#[cfg(feature = "minihal")]
#[inline(always)]
pub(crate) unsafe fn kx_acquire_spinlock(_lock: PKSPIN_LOCK) {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[cfg(feature = "minihal")]
#[inline(always)]
pub(crate) unsafe fn kx_release_spinlock(_lock: PKSPIN_LOCK) {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// i386-only FASTCALL spinlock surface (the C file gates this block on
// _M_IX86; amd64 spinlock impls live in apic/apic.c).  All return the
// previous IRQL so the caller can pair with a KeLowerIrql later.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
mod i386 {
    use super::*;
    use ntkrnl_sys::ddk::KeTryToAcquireSpinLockAtDpcLevel;
    use ntkrnl_sys::hal_ddk::{
        BOOLEAN, LOGICAL, PKLOCK_QUEUE_HANDLE,
        _KSPIN_LOCK_QUEUE_NUMBER as KSPIN_LOCK_QUEUE_NUMBER,
    };
    use crate::x86::pcr::KeGetCurrentPrcb;

    // The in-stack queued lock is ntkrnl-sys's MCS lock, the same one ntoskrnl's exports use:
    // a driver may acquire through hal.dll and release through ntoskrnl. The mini-HAL keeps
    // its bare fence, single-threaded and before there is a KPCR (see kx_acquire_spinlock).
    #[cfg(not(feature = "minihal"))]
    #[inline(always)]
    unsafe fn hal_inst_acquire(h: PKLOCK_QUEUE_HANDLE) {
        unsafe {
            ntkrnl_sys::inline::ke::KxAcquireInStackQueuedSpinLock(
                core::ptr::addr_of_mut!((*h).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE,
            )
        }
    }
    #[cfg(not(feature = "minihal"))]
    #[inline(always)]
    unsafe fn hal_inst_release(h: PKLOCK_QUEUE_HANDLE) {
        unsafe {
            ntkrnl_sys::inline::ke::KxReleaseInStackQueuedSpinLock(
                core::ptr::addr_of_mut!((*h).LockQueue) as *mut ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE,
            )
        }
    }
    #[cfg(feature = "minihal")]
    #[inline(always)]
    unsafe fn hal_inst_acquire(h: PKLOCK_QUEUE_HANDLE) {
        unsafe { kx_acquire_spinlock((*h).LockQueue.Lock) }
    }
    #[cfg(feature = "minihal")]
    #[inline(always)]
    unsafe fn hal_inst_release(h: PKLOCK_QUEUE_HANDLE) {
        unsafe { kx_release_spinlock((*h).LockQueue.Lock) }
    }

    // KfRaiseIrql / KfLowerIrql are NTHALAPI — declared in xdk/x86/ke.h, defined
    // in hal/halx86/pic/pic.c, exported from hal.dll (see hal.spec).  The
    // hal_bindgen output drops them (no extern fn produced) so declare locally.
    // Rust 2021 edition uses `extern { ... }` (without leading `unsafe` keyword
    // on the block — that's a 2024 thing); same shape as ntkrnl-sys/guards.rs.
    extern "fastcall" {
        fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
        fn KfLowerIrql(NewIrql: KIRQL);
    }

    use crate::x86::buildtype::SYNCH_LEVEL; // 2 on the UP HAL, 27 on the SMP HAL (i386)
    const DISPATCH_LEVEL: KIRQL = 2;

    /// `KeAcquireSpinLockRaiseToSynch` — raise to SYNCH, acquire, return old IRQL.
    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeAcquireSpinLockRaiseToSynch(SpinLock: PKSPIN_LOCK) -> KIRQL {
        unsafe {
            let old = KfRaiseIrql(SYNCH_LEVEL);
            kx_acquire_spinlock(SpinLock);
            old
        }
    }

    /// `KfAcquireSpinLock` — raise to DISPATCH, acquire, return old IRQL.
    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KfAcquireSpinLock(SpinLock: PKSPIN_LOCK) -> KIRQL {
        unsafe {
            let old = KfRaiseIrql(DISPATCH_LEVEL);
            kx_acquire_spinlock(SpinLock);
            old
        }
    }

    /// `KfReleaseSpinLock` — release lock, lower IRQL.
    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KfReleaseSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL) {
        unsafe {
            kx_release_spinlock(SpinLock);
            KfLowerIrql(OldIrql);
        }
    }

    // Pull KPRCB.LockQueue[N].Lock for the queued-spinlock fns.  HACK per the
    // C source — proper queued spinlocks would walk a per-CPU queue.
    #[inline(always)]
    unsafe fn queue_lock(number: KSPIN_LOCK_QUEUE_NUMBER) -> PKSPIN_LOCK {
        unsafe {
            let prcb = KeGetCurrentPrcb();
            (*prcb).LockQueue[number as usize].Lock
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeAcquireQueuedSpinLock(
        LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
    ) -> KIRQL {
        unsafe {
            let old = KfRaiseIrql(DISPATCH_LEVEL);
            kx_acquire_spinlock(queue_lock(LockNumber));
            old
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeAcquireQueuedSpinLockRaiseToSynch(
        LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
    ) -> KIRQL {
        unsafe {
            let old = KfRaiseIrql(SYNCH_LEVEL);
            kx_acquire_spinlock(queue_lock(LockNumber));
            old
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeAcquireInStackQueuedSpinLock(
        SpinLock: PKSPIN_LOCK,
        LockHandle: PKLOCK_QUEUE_HANDLE,
    ) {
        unsafe {
            (*LockHandle).LockQueue.Next = core::ptr::null_mut();
            (*LockHandle).LockQueue.Lock = SpinLock;
            (*LockHandle).OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
            hal_inst_acquire(LockHandle);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
        SpinLock: PKSPIN_LOCK,
        LockHandle: PKLOCK_QUEUE_HANDLE,
    ) {
        unsafe {
            (*LockHandle).LockQueue.Next = core::ptr::null_mut();
            (*LockHandle).LockQueue.Lock = SpinLock;
            (*LockHandle).OldIrql = KfRaiseIrql(SYNCH_LEVEL);
            hal_inst_acquire(LockHandle);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeReleaseQueuedSpinLock(
        LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
        OldIrql: KIRQL,
    ) {
        unsafe {
            kx_release_spinlock(queue_lock(LockNumber));
            KfLowerIrql(OldIrql);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeReleaseInStackQueuedSpinLock(LockHandle: PKLOCK_QUEUE_HANDLE) {
        unsafe {
            hal_inst_release(LockHandle);
            KfLowerIrql((*LockHandle).OldIrql);
        }
    }

    /// `KeTryToAcquireQueuedSpinLockRaiseToSynch` — KM tests show this raises
    /// IRQL even on failure; mirror that.
    #[cfg(not(feature = "minihal"))]
    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeTryToAcquireQueuedSpinLockRaiseToSynch(
        LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
        OldIrql: *mut KIRQL,
    ) -> BOOLEAN {
        unsafe {
            let lock = queue_lock(LockNumber);
            *OldIrql = KfRaiseIrql(SYNCH_LEVEL);
            KeTryToAcquireSpinLockAtDpcLevel(lock)
        }
    }

    /// `KeTryToAcquireQueuedSpinLock` — same shape as above, DISPATCH IRQL.
    #[cfg(not(feature = "minihal"))]
    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KeTryToAcquireQueuedSpinLock(
        LockNumber: KSPIN_LOCK_QUEUE_NUMBER,
        OldIrql: *mut KIRQL,
    ) -> LOGICAL {
        unsafe {
            let lock = queue_lock(LockNumber);
            *OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
            KeTryToAcquireSpinLockAtDpcLevel(lock) as LOGICAL
        }
    }

    /// `KefAcquireSpinLockAtDpcLevel` — under MINIHAL only.  In runtime
    /// builds this symbol is provided by ntoskrnl; under FreeLDR's
    /// mini_hal there's no full kernel, so the HAL stubs it.  The C body
    /// in hal/halx86/generic/spinlock.c#ifdef _MINIHAL_ writes a debug
    /// sentinel into the lock value under DBG; we mirror that.
    #[cfg(feature = "minihal")]
    #[unsafe(no_mangle)]
    pub unsafe extern "fastcall" fn KefAcquireSpinLockAtDpcLevel(_SpinLock: PKSPIN_LOCK) {
        // No-op in release.  DBG instrumentation in the C original is not
        // load-bearing for boot.
    }
}

// ---------------------------------------------------------------------------
// CMOS-hardware-lock — NTAPI, compiled for both i386 and amd64.  Saves
// eflags into HalpSystemHardwareFlags AFTER taking the lock so that
// interrupts stay off across the protected region.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpAcquireCmosSpinLock() {
    unsafe {
        let flags = read_eflags();
        disable_interrupts();
        kx_acquire_spinlock(&raw mut HalpSystemHardwareLock);
        core::ptr::write_volatile(&raw mut HalpSystemHardwareFlags, flags);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReleaseCmosSpinLock() {
    unsafe {
        let flags = core::ptr::read_volatile(&raw const HalpSystemHardwareFlags);
        kx_release_spinlock(&raw mut HalpSystemHardwareLock);
        write_eflags(flags);
    }
}
