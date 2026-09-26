//! Rust impls of `ke_x.h` FORCEINLINEs whose C trampolines live in
//! `ke_rust_shims.c`. Batches landed so far:
//!   - In-stack-queued spinlocks (8): APC lock family ×5, device queue ×2,
//!     `KiReleaseProcessLockFromSynchLevel` ×1.
//!   - Dispatcher locks (6): `KiAcquireDispatcherLock` +
//!     `KiReleaseDispatcherLock`, `KiAcquire/ReleaseDispatcherLockAtSynchLevel/
//!     FromSynchLevel`, `KiAcquire/ReleaseDispatcherObject`. Bodies match the
//!     `#ifndef CONFIG_SMP` paths in `ke_x.h` (kernel runs UP — `CONFIG_SMP`
//!     is undefined). UP semantics: dispatcher object lock is a no-op (single
//!     CPU acquires it implicitly), dispatcher lock raises to SYNCH_LEVEL via
//!     `KfRaiseIrql` and exits via `KiExitDispatcher`.
//!   - Thread / PRCB locks (5): `KiAcquire/ReleaseThreadLock`,
//!     `KiTryThreadLock`, `KiAcquire/ReleasePrcbLock`. UP-only: per-thread
//!     and per-PRCB locks are no-ops (single CPU implies the caller already
//!     owns them); `KiTryThreadLock` returns FALSE.
//!   - Scheduler helpers (5): `KiInsertDeferredReadyList`,
//!     `KiRescheduleThread`, `KiSetThreadSwapBusy`, `KiCheckDeferredReadyList`,
//!     `KiRequestApcInterrupt`. Four are UP no-ops; only
//!     `KiInsertDeferredReadyList` does real work — sets the thread's State
//!     to DeferredReady, DeferredProcessor to 0 (boot CPU), and calls
//!     `KiDeferredReadyThread`.
//!   - Timer-lock management (3): `KiAcquireTimerLock`, `KiReleaseTimerLock`,
//!     `KiComputeTimerTableIndex`. UP: lock pair is no-op (acquire returns
//!     NULL). `KiComputeTimerTableIndex` is the
//!     `(DueTime / KeMaximumIncrement) & (TIMER_TABLE_SIZE - 1)` bit math;
//!     `KeMaximumIncrement` is a `pub static ULONG` and `TIMER_TABLE_SIZE`
//!     is a compile-time constant 512.
//!   - Spinlock primitives (2): `KxAcquire/ReleaseSpinLock`. With our
//!     `USE_DUMMY_PSEH` + UP kernel combo, `spinlock.h`'s FORCEINLINE bodies
//!     collapse to just `KeMemoryBarrierWithoutFence()` — a compiler
//!     reordering barrier with no machine-code emission. The C trampoline
//!     at `ke_rust_shims.c` already chose to skip the `#if DBG` owner
//!     tracking (inconsistent Rust/C paths produced false SPIN_LOCK_NOT_OWNED
//!     bugchecks on i386); the Rust impl matches by emitting just a
//!     `compiler_fence(SeqCst)`.
//!   - NMI list lock (2): `KiAcquire/ReleaseNmiListLock`. Wraps
//!     `KeAcquireSpinLock` / `KeReleaseSpinLock` over the global
//!     `KiNmiCallbackListLock` (defined in `ntoskrnl-rust::ke::bug`). The
//!     spinlock primitive is arch-conditional: i386 uses HAL `KfAcquire/
//!     KfReleaseSpinLock` (fastcall), amd64/arm64 use `KeAcquireSpinLock-
//!     RaiseToDpc` + `KeReleaseSpinLock` (cdecl).
//!   - Stack/alert checks (2): `KiCheckThreadStackSwap`, `KiCheckAlertability`.
//!     Read-only KTHREAD field walks: stack-swap eligibility (priority +
//!     EnableStackSwap + WaitMode) and alertability dispatch (Alerted[mode],
//!     UserApcPending, ApcState.ApcListHead[UserMode]).
//!   - Ready-queue select / new-priority (2): `KiSelectReadyThread`,
//!     `KiComputeNewPriority`. The first scans `Prcb->ReadySummary` for the
//!     highest priority bit at-or-above the requested floor, pulls the head
//!     thread off `DispatcherReadyListHead[Priority]`, and clears the
//!     ReadySummary bit if the bucket is now empty. The second is the dynamic-
//!     priority decay math (Priority -= PriorityDecrement + Adjustment, clamped
//!     to BasePriority).
//!   - Timer helpers (5): `KiRecalculateDueTime`, `KxInsertTimer`,
//!     `KiComputeDueTime`, `KxRemoveTreeTimer`, `KxSetTimerForThreadWait`.
//!     Drive the timer-table wheel rooted at `KiTimerTableListHead[Hand]`.
//!     Bitfield writes on `DISPATCHER_HEADER` (Absolute, Inserted, Hand) go
//!     through bindgen's deeply-nested setter accessors. `KeQueryInterruptTime`
//!     and `KeQuerySystemTime` are read directly from `SharedUserData` on
//!     amd64/arm64 (where they're macros, no exported symbol); i386 has a
//!     real `KeQuerySystemTime` export which we call via stdcall.
//!   - Guarded mutex (6): `_KeInitializeGuardedMutex`,
//!     `_KeAcquire/ReleaseGuardedMutexUnsafe`, `_KeAcquire/ReleaseGuardedMutex`,
//!     `_KeTryToAcquireGuardedMutex`. Lock-bit fast path via `AtomicI32`
//!     fetch_and / fetch_add / compare_exchange; contested path delegates to
//!     `KiAcquireGuardedMutex` and `KeSignalGateBoostPriority` exports.
//!     `KeEnterGuardedRegionThread` macro is inlined as a `SpecialApcDisable--`
//!     write; the matching Leave macro increments and (on transition to 0)
//!     calls `KiCheckForKernelApcDelivery` if a kernel APC is queued.
//!   - Unwait + queue-ready helpers (3): `KxUnwaitThread`,
//!     `KxUnwaitThreadForEvent`, `KxQueueReadyThread`. The first two walk
//!     `Object->WaitListHead`, pull each KWAIT_BLOCK and wake its thread via
//!     `KiUnwaitThread` (with the existing C trampoline's defensive null-thread
//!     guard preserved). `KxUnwaitThreadForEvent` early-outs on the first
//!     `WaitAny` block (un-signaling the event). `KxQueueReadyThread` runs the
//!     UP fast-path: sets State=Ready, stamps `KeTickCount.LowPart` into
//!     WaitTime, head/tail-inserts on `DispatcherReadyListHead[Priority]`,
//!     OR-sets the ReadySummary bit.
//!
//! Architecturally this crate exists in **isolation** from `ntoskrnl-rust`
//! to dodge the LLVM captures(none) cross-CGU contagion that killed the
//! first Phase 1 pilot — see `docs/engineering/shim-elimination.md` §
//! "Phase 1 — PILOT 2 LANDED" for the validated pattern. cm-inlines is the
//! sister crate (KCB push-locks, same shape).
//!
//! C side keeps the `Ki*` symbols alive in `ke_rust_shims.c`; their bodies
//! collapse to one delegating call into the `_rs_*` impls below. Existing
//! Rust callers (in `ntoskrnl-rust`) keep using the unchanged
//! `extern "C" { fn KiAcquireApcLockRaiseToSynch(...) }` decls — no source
//! change in `ntoskrnl-rust` for this round.

#![no_std]

use ntkrnl_sys::base::{
    InsertHeadList, InsertTailList, RemoveEntryList, BOOLEAN, KIRQL, LONG, LONG_PTR, NTSTATUS,
    UCHAR, ULONG, ULONG_PTR,
};
use ntkrnl_sys::ddk::{
    KPROCESSOR_MODE, PKDEVICE_QUEUE, PKEVENT, PKGATE, PKGUARDED_MUTEX, PKLOCK_QUEUE_HANDLE,
    PKPRCB, PKSPIN_LOCK_QUEUE, PKTHREAD, PKTIMER, PKWAIT_BLOCK, _DISPATCHER_HEADER, _KPCR,
    _KSYSTEM_TIME, _KTHREAD, _KTIMER_TABLE_ENTRY, _KUSER_SHARED_DATA, _KWAIT_BLOCK, _LARGE_INTEGER,
    _LIST_ENTRY,
};
use ntkrnl_sys::CONTAINING_RECORD;

/// `_KTHREAD_STATE_DeferredReady` — same value (= 7) on all 3 archs per bindgen
/// (`bindgen_{i386,amd64,arm64}.rs`). Replicated as a u8 so we don't pull in
/// the `_KTHREAD_STATE` enum just for this constant.
const KTHREAD_STATE_DEFERRED_READY: u8 = 7;

/// `TIMER_TABLE_SIZE` from `sdk/include/xdk/ketypes.h` — fixed 512.
const TIMER_TABLE_SIZE: u32 = 512;

/// `LOW_REALTIME_PRIORITY` — base priority of the realtime range. From
/// `ntkrnl_sys::constants` (= 16). Replicated to keep ke-inlines isolated.
const LOW_REALTIME_PRIORITY: i8 = 16;

/// NTSTATUS values used by `KiCheckAlertability`.
const STATUS_ALERTED: NTSTATUS = 0x00000101u32 as NTSTATUS;
const STATUS_USER_APC: NTSTATUS = 0x000000C0u32 as NTSTATUS;
const STATUS_WAIT_0: NTSTATUS = 0;

/// `STATUS_KERNEL_APC` from `ntstatus.h` — used by Kx{Unwait}* to wake threads
/// that should re-enter the wait after a kernel APC is delivered.
const STATUS_KERNEL_APC: LONG_PTR = 0x00000100;

/// `WaitAny` enum value (`_WAIT_TYPE_WaitAny`). Same on all 3 arches.
const WAIT_ANY: u8 = 1;

/// `KTHREAD_STATE_Ready` (`_KTHREAD_STATE_Ready` = 1).
const KTHREAD_STATE_READY: u8 = 1;

/// Guarded mutex bits from `ketypes.h`: `GM_LOCK_BIT` (the "lock available"
/// bit) at position 0; `GM_LOCK_WAITER_WOKEN` at bit 1; and the per-waiter
/// increment in the upper bits.
const GM_LOCK_BIT: LONG = 0x1;
const GM_LOCK_BIT_V: u32 = 0; // bit position
const GM_LOCK_WAITER_WOKEN: LONG = 0x2;
const GM_LOCK_WAITER_INC: LONG = 0x4;

extern "C" {
    /// `KeMaximumIncrement` — clock tick increment (100ns units), set at boot
    /// from the HAL. Used as the divisor in the timer table index computation.
    static KeMaximumIncrement: ULONG;

    /// `KiNmiCallbackListLock` — global spinlock guarding the NMI callback
    /// list. Defined in `ntoskrnl-rust::ke::bug` as a `pub static mut
    /// KSPIN_LOCK = 0` (== `ULONG_PTR`).
    static mut KiNmiCallbackListLock: ULONG_PTR;

    /// `KeTickCount` — system tick count as `KSYSTEM_TIME`. Defined in
    /// `ntoskrnl-rust::ke::clock`. We only read the LowPart in `KxQueueReadyThread`.
    static KeTickCount: _KSYSTEM_TIME;

    /// `KiTimerTableListHead` — global array of `KTIMER_TABLE_ENTRY`. The
    /// `K7` timer-helpers (KxInsertTimer, KxRemoveTreeTimer, KiRemoveEntryTimer)
    /// look up entries indexed by Hand to find/empty a timer bucket.
    static mut KiTimerTableListHead: [_KTIMER_TABLE_ENTRY; 512];

    /// `SharedUserData` — kernel pointer to the read-only KUSER_SHARED_DATA
    /// page. Used in lieu of `KeQueryInterruptTime` / `KeQuerySystemTime`
    /// macros (the former is a real export on i386 but a macro on amd64/arm64).
    static SharedUserData: *mut _KUSER_SHARED_DATA;
}

// Spinlock primitives backing KiAcquire/ReleaseNmiListLock. amd64/arm64 use
// the cdecl `KeAcquireSpinLockRaiseToDpc` + `KeReleaseSpinLock` exports;
// i386 uses HAL's fastcall `KfAcquireSpinLock` / `KfReleaseSpinLock`.
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn KeAcquireSpinLockRaiseToDpc(spin_lock: *mut ULONG_PTR) -> KIRQL;
    fn KeReleaseSpinLock(spin_lock: *mut ULONG_PTR, new_irql: KIRQL);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KfAcquireSpinLock(spin_lock: *mut ULONG_PTR) -> KIRQL;
    fn KfReleaseSpinLock(spin_lock: *mut ULONG_PTR, new_irql: KIRQL);
}

/// SYNCH_LEVEL (matches `ntkrnl_sys::inline::ke::SYNCH_LEVEL`; replicated
/// locally to avoid pulling the whole `inline::ke` module into this crate).
/// UP kernel: DISPATCH_LEVEL = 2. SMP kernel: IPI_LEVEL - 2 on all three arches
/// (ndk asm.h/ketypes.h — arm64's ketypes.h:27-31 carries the same CONFIG_SMP pair).
/// arm64 was 2 here after the 2026-07-25 fix moved the other three copies to 12
/// (smp-bringup-quest.md); unused under smp today, which is how it survived.
#[cfg(not(feature = "smp"))]
const SYNCH_LEVEL: KIRQL = 2;
#[cfg(all(feature = "smp", target_arch = "x86"))]
const SYNCH_LEVEL: KIRQL = 0x1D - 2; // 27
#[cfg(all(feature = "smp", target_arch = "x86_64"))]
const SYNCH_LEVEL: KIRQL = 14 - 2; // 12
#[cfg(all(feature = "smp", target_arch = "aarch64"))]
const SYNCH_LEVEL: KIRQL = 14 - 2; // 12

// ============================================================================
// SMP (feature = "smp") support: constants + externs for the ke_x.h
// CONFIG_SMP lock bodies (ke_x.h lines 305-603). Values from ndk/xdk headers.
// ============================================================================

#[cfg(feature = "smp")]
const KOBJECT_LOCK_BIT: LONG = 0x80; // ndk/ketypes.h
#[cfg(feature = "smp")]
const LOCK_QUEUE_TIMER_LOCK_SHIFT: ULONG = 4; // xdk/ketypes.h
#[cfg(feature = "smp")]
const LOCK_QUEUE_TIMER_TABLE_LOCKS: ULONG = 1 << (8 - LOCK_QUEUE_TIMER_LOCK_SHIFT);
#[cfg(feature = "smp")]
const LOCK_QUEUE_DISPATCHER_LOCK: usize = 0; // LockQueueDispatcherLock
#[cfg(feature = "smp")]
const LOCK_QUEUE_TIMER_TABLE_LOCK: usize = 17; // LockQueueTimerTableLock
#[cfg(feature = "smp")]
const IPI_APC: ULONG = 1; // ndk/<arch>/ketypes.h
#[cfg(feature = "smp")]
const IPI_DPC: ULONG = 2;
#[cfg(feature = "smp")]
const APC_LEVEL: KIRQL = 1;

#[cfg(all(feature = "smp", target_arch = "x86"))]
extern "fastcall" {
    fn KeAcquireQueuedSpinLockRaiseToSynch(
        lock_number: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
    ) -> KIRQL;
    fn KeAcquireQueuedSpinLockAtDpcLevel(lock_queue: PKSPIN_LOCK_QUEUE);
    fn KeReleaseQueuedSpinLockFromDpcLevel(lock_queue: PKSPIN_LOCK_QUEUE);
    fn KiIpiSend(target_set: ULONG_PTR, ipi_request: ULONG);
    fn KiProcessDeferredReadyList(prcb: PKPRCB);
    fn HalRequestSoftwareInterrupt(irql: KIRQL);
}
#[cfg(all(feature = "smp", not(target_arch = "x86")))]
extern "C" {
    fn KeAcquireQueuedSpinLockRaiseToSynch(
        lock_number: ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
    ) -> KIRQL;
    fn KeAcquireQueuedSpinLockAtDpcLevel(lock_queue: PKSPIN_LOCK_QUEUE);
    fn KeReleaseQueuedSpinLockFromDpcLevel(lock_queue: PKSPIN_LOCK_QUEUE);
    fn KiIpiSend(target_set: ULONG_PTR, ipi_request: ULONG);
    fn KiProcessDeferredReadyList(prcb: PKPRCB);
    fn HalRequestSoftwareInterrupt(irql: KIRQL);
}

/// SMP simple-spinlock acquire on a lock word, using 32-bit interlocked ops on
/// the low DWORD exactly like the C (`InterlockedExchange((PLONG)&Lock, 1)`).
#[cfg(feature = "smp")]
#[inline]
unsafe fn smp_simple_lock_acquire(lock: *mut i32) {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(lock as *const AtomicI32);
    loop {
        if atomic.swap(1, Ordering::SeqCst) == 0 {
            break;
        }
        while atomic.load(Ordering::Relaxed) != 0 {
            core::hint::spin_loop();
        }
    }
}

/// SMP simple-spinlock release (`InterlockedAnd((PLONG)&Lock, 0)`).
#[cfg(feature = "smp")]
#[inline]
unsafe fn smp_simple_lock_release(lock: *mut i32) {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(lock as *const AtomicI32)).swap(0, Ordering::SeqCst);
}

// ============================================================================
// Underlying primitives — extern decls (i386 fastcall, others cdecl). These
// resolve at link time to the shim_exports.rs Rust impls (i386) or the HAL/
// kernel exports (amd64/arm64). Same shape as ntkrnl_sys::inline::ke.
// ============================================================================

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
        spin_lock: *mut ULONG_PTR,
        lock_handle: PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLockAtDpcLevel(
        spin_lock: *mut ULONG_PTR,
        lock_handle: PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLock(
        spin_lock: *mut ULONG_PTR,
        lock_handle: PKLOCK_QUEUE_HANDLE,
    );
    fn KeReleaseInStackQueuedSpinLock(lock_handle: PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLockFromDpcLevel(lock_handle: PKLOCK_QUEUE_HANDLE);
    fn KfRaiseIrql(new_irql: KIRQL) -> KIRQL;
    fn KiExitDispatcher(old_irql: KIRQL);
    fn KiDeferredReadyThread(thread: PKTHREAD);
    fn KiUnwaitThread(thread: PKTHREAD, wait_status: LONG_PTR, increment: LONG);
    fn KeInitializeGate(gate: PKGATE);
    fn KiAcquireGuardedMutex(gm: PKGUARDED_MUTEX);
    fn KeSignalGateBoostPriority(gate: PKGATE);
    fn KiInsertTimerTable(timer: PKTIMER, hand: ULONG) -> BOOLEAN;
    fn KiCompleteTimer(timer: PKTIMER, lock_queue: PKSPIN_LOCK_QUEUE);
}

// `KiCheckForKernelApcDelivery` is stdcall on i386 (no args), cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "stdcall" {
    fn KiCheckForKernelApcDelivery();
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn KiCheckForKernelApcDelivery();
}

#[cfg(not(target_arch = "x86"))]
extern "fastcall" {
    fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
        spin_lock: *mut ULONG_PTR,
        lock_handle: PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLockAtDpcLevel(
        spin_lock: *mut ULONG_PTR,
        lock_handle: PKLOCK_QUEUE_HANDLE,
    );
    fn KeAcquireInStackQueuedSpinLock(
        spin_lock: *mut ULONG_PTR,
        lock_handle: PKLOCK_QUEUE_HANDLE,
    );
    fn KeReleaseInStackQueuedSpinLock(lock_handle: PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLockFromDpcLevel(lock_handle: PKLOCK_QUEUE_HANDLE);
    fn KfRaiseIrql(new_irql: KIRQL) -> KIRQL;
    fn KiExitDispatcher(old_irql: KIRQL);
    fn KiDeferredReadyThread(thread: PKTHREAD);
    fn KiUnwaitThread(thread: PKTHREAD, wait_status: LONG_PTR, increment: LONG);
    fn KeInitializeGate(gate: PKGATE);
    fn KiAcquireGuardedMutex(gm: PKGUARDED_MUTEX);
    fn KeSignalGateBoostPriority(gate: PKGATE);
    fn KiInsertTimerTable(timer: PKTIMER, hand: ULONG) -> BOOLEAN;
    fn KiCompleteTimer(timer: PKTIMER, lock_queue: PKSPIN_LOCK_QUEUE);
}

// On i386 `KeQuerySystemTime` is a real export (stdcall, no return value).
// On amd64/arm64 it's a macro that loops over `SharedUserData->SystemTime` —
// inlined locally below.
#[cfg(target_arch = "x86")]
extern "stdcall" {
    fn KeQuerySystemTime(current_time: *mut _LARGE_INTEGER);
}

// ============================================================================
// KeGetPcr / KeGetCurrentPrcb — replicated locally to dodge the cross-crate
// `#[inline]` over `core::arch::asm!` non-inline at opt-level=1
// (see feedback_cross_crate_inline_asm.md).
// ============================================================================

#[inline(always)]
unsafe fn ke_get_pcr() -> *mut _KPCR {
    let result: *mut _KPCR;
    #[cfg(target_arch = "x86_64")]
    {
        core::arch::asm!(
            "mov {}, gs:[0x18]",
            out(reg) result,
            options(nostack, nomem, preserves_flags),
        );
    }
    #[cfg(target_arch = "x86")]
    {
        let raw: u32;
        core::arch::asm!(
            "mov {:e}, dword ptr fs:[0x1C]",
            out(reg) raw,
            options(nostack, nomem, preserves_flags),
        );
        result = raw as *mut _KPCR;
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!(
            "mrs {}, tpidr_el1",
            out(reg) result,
            options(nostack, nomem, preserves_flags),
        );
    }
    result
}

#[inline(always)]
unsafe fn ke_get_current_prcb() -> PKPRCB {
    #[cfg(target_arch = "x86")]
    {
        (*ke_get_pcr()).Prcb
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*ke_get_pcr()).__bindgen_anon_1.__bindgen_anon_1.CurrentPrcb
    }
}

#[inline(always)]
unsafe fn ke_get_current_thread() -> PKTHREAD {
    (*ke_get_current_prcb()).CurrentThread as PKTHREAD
}

/// Pointer to KTHREAD.SpecialApcDisable (i16). Path differs per arch:
/// - i386: `__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable`
/// - amd64/arm64: `__bindgen_anon_5.__bindgen_anon_8.__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable`
#[inline(always)]
unsafe fn kt_special_apc_disable_ptr(t: PKTHREAD) -> *mut i16 {
    #[cfg(target_arch = "x86")]
    {
        core::ptr::addr_of_mut!((*t).__bindgen_anon_4.__bindgen_anon_1.SpecialApcDisable)
    }
    #[cfg(not(target_arch = "x86"))]
    {
        core::ptr::addr_of_mut!(
            (*t).__bindgen_anon_5
                .__bindgen_anon_8
                .__bindgen_anon_1
                .__bindgen_anon_1
                .SpecialApcDisable
        )
    }
}

#[inline(always)]
unsafe fn dpc_thread_active(prcb: PKPRCB) -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        (*prcb).__bindgen_anon_2.__bindgen_anon_2.DpcThreadActive() != 0
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        (*prcb).DpcThreadActive != 0
    }
}

// ============================================================================
// APC lock family
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireApcLockRaiseToSynch(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    KeAcquireInStackQueuedSpinLockRaiseToSynch(
        core::ptr::addr_of_mut!((*thread).ApcQueueLock) as *mut ULONG_PTR,
        handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireApcLockAtSynchLevel(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    KeAcquireInStackQueuedSpinLockAtDpcLevel(
        core::ptr::addr_of_mut!((*thread).ApcQueueLock) as *mut ULONG_PTR,
        handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireApcLockRaiseToDpc(
    thread: PKTHREAD,
    handle: PKLOCK_QUEUE_HANDLE,
) {
    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*thread).ApcQueueLock) as *mut ULONG_PTR,
        handle,
    );
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseApcLock(handle: PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLock(handle);
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseApcLockFromSynchLevel(handle: PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLockFromDpcLevel(handle);
}

// ============================================================================
// Process lock — only the "FromSynchLevel" half lives in ke_rust_shims.c
// (KiAcquireProcessLockRaiseToSynch + KiReleaseProcessLock are defined in
// mm_rust_shims.c because they're PCH-covered and can't be _saved_-renamed
// from ke_rust_shims.c — see the comment at mm_rust_shims.c:840).
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseProcessLockFromSynchLevel(handle: PKLOCK_QUEUE_HANDLE) {
    KeReleaseInStackQueuedSpinLockFromDpcLevel(handle);
}

// ============================================================================
// Device queue lock — branches on KPRCB.DpcThreadActive
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDeviceQueueLock(
    device_queue: PKDEVICE_QUEUE,
    device_lock: PKLOCK_QUEUE_HANDLE,
) {
    if dpc_thread_active(ke_get_current_prcb()) {
        KeAcquireInStackQueuedSpinLock(
            core::ptr::addr_of_mut!((*device_queue).Lock) as *mut ULONG_PTR,
            device_lock,
        );
    } else {
        KeAcquireInStackQueuedSpinLockAtDpcLevel(
            core::ptr::addr_of_mut!((*device_queue).Lock) as *mut ULONG_PTR,
            device_lock,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDeviceQueueLock(device_lock: PKLOCK_QUEUE_HANDLE) {
    if dpc_thread_active(ke_get_current_prcb()) {
        KeReleaseInStackQueuedSpinLock(device_lock);
    } else {
        KeReleaseInStackQueuedSpinLockFromDpcLevel(device_lock);
    }
}

// ============================================================================
// Dispatcher locks (ke_x.h, #ifndef CONFIG_SMP — the kernel runs UP)
//
// On UP the dispatcher object lock is meaningless (single CPU implies the
// caller already holds it), the dispatcher lock proper just raises to
// SYNCH_LEVEL via KfRaiseIrql, the release pair runs KiExitDispatcher (which
// drains the deferred-ready list and lowers IRQL).
// ============================================================================

/// Per-object dispatcher lock acquire — UP no-op. SMP: spin on
/// `Object->Lock & KOBJECT_LOCK_BIT`, then CAS-set the bit (ke_x.h:309).
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherObject(_object: *mut _DISPATCHER_HEADER) {}

#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherObject(object: *mut _DISPATCHER_HEADER) {
    use core::sync::atomic::{AtomicI32, Ordering};
    let lock = &*(core::ptr::addr_of_mut!((*object).__bindgen_anon_1.Lock) as *const AtomicI32);
    loop {
        let old = loop {
            let v = lock.load(Ordering::Relaxed);
            if v & KOBJECT_LOCK_BIT == 0 {
                break v;
            }
            core::hint::spin_loop();
        };
        if lock
            .compare_exchange(old, old | KOBJECT_LOCK_BIT, Ordering::SeqCst, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
}

/// Per-object dispatcher lock release — UP no-op. SMP:
/// `InterlockedAnd(&Object->Lock, ~KOBJECT_LOCK_BIT)` (ke_x.h:338).
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherObject(_object: *mut _DISPATCHER_HEADER) {}

#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherObject(object: *mut _DISPATCHER_HEADER) {
    use core::sync::atomic::{AtomicI32, Ordering};
    (&*(core::ptr::addr_of_mut!((*object).__bindgen_anon_1.Lock) as *const AtomicI32))
        .fetch_and(!KOBJECT_LOCK_BIT, Ordering::SeqCst);
}

/// Acquire dispatcher lock — UP: raise to SYNCH_LEVEL; returns previous IRQL.
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherLock() -> KIRQL {
    KfRaiseIrql(SYNCH_LEVEL)
}

/// Acquire dispatcher lock — SMP: raise to synch + take the queued
/// dispatcher lock (ke_x.h:349).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherLock() -> KIRQL {
    KeAcquireQueuedSpinLockRaiseToSynch(
        LOCK_QUEUE_DISPATCHER_LOCK as ntkrnl_sys::ddk::KSPIN_LOCK_QUEUE_NUMBER,
    )
}

/// Release dispatcher lock — UP: exit through the dispatcher (which lowers
/// IRQL to OldIrql after draining deferred-ready / pending APCs).
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherLock(old_irql: KIRQL) {
    KiExitDispatcher(old_irql)
}

/// Release dispatcher lock — SMP: drop the queued lock first, then exit the
/// dispatcher (ke_x.h:357).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherLock(old_irql: KIRQL) {
    KeReleaseQueuedSpinLockFromDpcLevel(core::ptr::addr_of_mut!(
        (*ke_get_current_prcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK]
    ));
    KiExitDispatcher(old_irql)
}

/// Acquire dispatcher lock when already at SYNCH_LEVEL — UP no-op.
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherLockAtSynchLevel() {}

/// SMP: `KeAcquireQueuedSpinLockAtDpcLevel(LockQueueDispatcherLock)` (ke_x.h:369).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherLockAtSynchLevel() {
    KeAcquireQueuedSpinLockAtDpcLevel(core::ptr::addr_of_mut!(
        (*ke_get_current_prcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK]
    ));
}

/// Release dispatcher lock from SYNCH_LEVEL without lowering IRQL — UP no-op.
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherLockFromSynchLevel() {}

/// SMP: `KeReleaseQueuedSpinLockFromDpcLevel` (ke_x.h:379).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherLockFromSynchLevel() {
    KeReleaseQueuedSpinLockFromDpcLevel(core::ptr::addr_of_mut!(
        (*ke_get_current_prcb()).LockQueue[LOCK_QUEUE_DISPATCHER_LOCK]
    ));
}

// ============================================================================
// Thread / PRCB locks (ke_x.h, #ifndef CONFIG_SMP)
//
// All UP no-ops — the kernel only runs on one CPU, so per-thread and
// per-PRCB locks are implicit. KiTryThreadLock returns FALSE on UP
// (matches the C body: `UNREFERENCED_PARAMETER(Thread); return FALSE;`).
// ============================================================================

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireThreadLock(_thread: PKTHREAD) {}

/// SMP: spin on `Thread->ThreadLock` (ke_x.h:488).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireThreadLock(thread: PKTHREAD) {
    smp_simple_lock_acquire(core::ptr::addr_of_mut!((*thread).ThreadLock) as *mut i32);
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseThreadLock(_thread: PKTHREAD) {}

/// SMP: release `Thread->ThreadLock` (ke_x.h:517).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseThreadLock(thread: PKTHREAD) {
    smp_simple_lock_release(core::ptr::addr_of_mut!((*thread).ThreadLock) as *mut i32);
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiTryThreadLock(_thread: PKTHREAD) -> BOOLEAN {
    0
}

/// SMP (faithful to ke_x.h:528): FALSE if the lock is free, else exchange-1
/// and report whether it was still held.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiTryThreadLock(thread: PKTHREAD) -> BOOLEAN {
    use core::sync::atomic::{AtomicI32, Ordering};
    let lock = core::ptr::addr_of_mut!((*thread).ThreadLock) as *mut i32;
    if core::ptr::read_volatile(lock) == 0 {
        return 0;
    }
    ((&*(lock as *const AtomicI32)).swap(1, Ordering::SeqCst) == 1) as BOOLEAN
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquirePrcbLock(_prcb: PKPRCB) {}

/// SMP: spin on `Prcb->PrcbLock` (ke_x.h:440).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquirePrcbLock(prcb: PKPRCB) {
    smp_simple_lock_acquire(core::ptr::addr_of_mut!((*prcb).PrcbLock) as *mut i32);
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleasePrcbLock(_prcb: PKPRCB) {}

/// SMP: release `Prcb->PrcbLock` (ke_x.h:469).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleasePrcbLock(prcb: PKPRCB) {
    smp_simple_lock_release(core::ptr::addr_of_mut!((*prcb).PrcbLock) as *mut i32);
}

// ============================================================================
// Scheduler helpers (ke_x.h, #ifndef CONFIG_SMP)
//
// All UP no-ops except KiInsertDeferredReadyList, which sets the thread's
// scheduling state and hands off to KiDeferredReadyThread. The KTHREAD
// field nesting differs per arch — bindgen unions for State (i386 direct,
// amd64/arm64 nested two anon unions deep) and DeferredProcessor (all
// arches nested under __bindgen_anon_1.__bindgen_anon_1). Replicated
// locally to avoid pulling ntkrnl_sys::inline::ke's private accessors.
// ============================================================================

#[inline(always)]
unsafe fn kt_set_state(thread: PKTHREAD, val: u8) {
    #[cfg(target_arch = "x86")]
    {
        (*thread).State = val;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*thread).__bindgen_anon_5.__bindgen_anon_6.State = val;
    }
}

#[inline(always)]
unsafe fn kt_set_deferred_processor(thread: PKTHREAD, val: u8) {
    (*thread).__bindgen_anon_1.__bindgen_anon_1.DeferredProcessor = val;
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiInsertDeferredReadyList(thread: PKTHREAD) {
    kt_set_state(thread, KTHREAD_STATE_DEFERRED_READY);
    kt_set_deferred_processor(thread, 0);
    KiDeferredReadyThread(thread);
}

/// SMP: push onto this CPU's deferred-ready list (ke_x.h:391) —
/// `PushEntryList(&Prcb->DeferredReadyListHead, &Thread->SwapListEntry)`.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiInsertDeferredReadyList(thread: PKTHREAD) {
    let prcb = ke_get_current_prcb();
    kt_set_state(thread, KTHREAD_STATE_DEFERRED_READY);
    kt_set_deferred_processor(thread, (*prcb).Number as u8);
    // SwapListEntry shares the __bindgen_anon_3 union with WaitListEntry.
    (*thread).__bindgen_anon_3.SwapListEntry.Next = (*prcb).DeferredReadyListHead.Next;
    (*prcb).DeferredReadyListHead.Next =
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.SwapListEntry);
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiRescheduleThread(_new_thread: BOOLEAN, _cpu: ULONG) {}

/// SMP: IPI the target CPU if a new thread needs to run there (ke_x.h:405).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiRescheduleThread(new_thread: BOOLEAN, cpu: ULONG) {
    if new_thread != 0 && (*ke_get_current_prcb()).Number as ULONG != cpu {
        KiIpiSend(1 << cpu, IPI_DPC);
    }
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiSetThreadSwapBusy(_thread: PKTHREAD) {}

/// SMP: mark the thread as being swapped (ke_x.h:422).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiSetThreadSwapBusy(thread: PKTHREAD) {
    (*thread).SwapBusy = 1;
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiCheckDeferredReadyList(_prcb: PKPRCB) {}

/// SMP: drain the PRCB deferred-ready list if non-empty (ke_x.h:545).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiCheckDeferredReadyList(prcb: PKPRCB) {
    if !(*prcb).DeferredReadyListHead.Next.is_null() {
        KiProcessDeferredReadyList(prcb);
    }
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiRequestApcInterrupt(_need_apc: BOOLEAN, _processor: UCHAR) {}

/// SMP: IPI the target CPU for APC delivery, or soft-int self (ke_x.h:553).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiRequestApcInterrupt(need_apc: BOOLEAN, processor: UCHAR) {
    if need_apc != 0 {
        if (*ke_get_current_prcb()).Number as UCHAR != processor {
            KiIpiSend(1 << processor, IPI_APC);
        } else {
            HalRequestSoftwareInterrupt(APC_LEVEL);
        }
    }
}

// ============================================================================
// Timer-lock management (ke_x.h, #ifndef CONFIG_SMP)
// ============================================================================

/// `KiAcquireTimerLock` — UP returns NULL (no real spinlock to take).
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireTimerLock(_hand: ULONG) -> PKSPIN_LOCK_QUEUE {
    core::ptr::null_mut()
}

/// `KiAcquireTimerLock` — SMP: acquire the hashed timer-table queued lock
/// (ke_x.h:575).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireTimerLock(hand: ULONG) -> PKSPIN_LOCK_QUEUE {
    let lock_index =
        ((hand >> LOCK_QUEUE_TIMER_LOCK_SHIFT) & (LOCK_QUEUE_TIMER_TABLE_LOCKS - 1)) as usize;
    let lock_queue = core::ptr::addr_of_mut!(
        (*ke_get_current_prcb()).LockQueue[LOCK_QUEUE_TIMER_TABLE_LOCK + lock_index]
    );
    KeAcquireQueuedSpinLockAtDpcLevel(lock_queue);
    lock_queue
}

/// `KiReleaseTimerLock` — UP no-op.
#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseTimerLock(_lock_queue: PKSPIN_LOCK_QUEUE) {}

/// `KiReleaseTimerLock` — SMP: release the timer-table queued lock (ke_x.h:595).
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseTimerLock(lock_queue: PKSPIN_LOCK_QUEUE) {
    KeReleaseQueuedSpinLockFromDpcLevel(lock_queue);
}

/// `KiComputeTimerTableIndex` — `(DueTime / KeMaximumIncrement) & 0x1FF`.
/// Reading `KeMaximumIncrement` via `read_volatile` matches the C semantics
/// (the global is set once at boot but the compiler doesn't know that).
#[no_mangle]
pub unsafe extern "C" fn _rs_KiComputeTimerTableIndex(due_time: u64) -> ULONG {
    let max_inc = core::ptr::read_volatile(core::ptr::addr_of!(KeMaximumIncrement)) as u64;
    ((due_time / max_inc) as ULONG) & (TIMER_TABLE_SIZE - 1)
}

// ============================================================================
// Spinlock primitives (spinlock.h FORCEINLINEs — KxAcquire/ReleaseSpinLock)
//
// The full `spinlock.h` body has three optional sections:
//   1. DBG-only `KeBugCheckEx` if the same thread already owns the lock
//   2. CONFIG_SMP-only spin-loop on InterlockedBitTestAndSet + YieldProcessor
//   3. Always: `KeMemoryBarrierWithoutFence()` — compiler reordering barrier
//
// We don't define CONFIG_SMP, and our DBG owner-tracking was disabled by
// the existing C trampoline (kept inconsistent Rust/C tracking paths from
// false-bugchecking on i386 — see the comment in `ke_rust_shims.c`). So
// the Rust impl is just the compiler fence. `compiler_fence(SeqCst)`
// emits no machine code; it only constrains the compiler's reordering.
// ============================================================================

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KxAcquireSpinLock(_spin_lock: *mut ULONG_PTR) {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

/// SMP (spinlock.h CONFIG_SMP): try-acquire via InterlockedBitTestAndSet(bit
/// 0); on contention spin-read until free, then retry.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KxAcquireSpinLock(spin_lock: *mut ULONG_PTR) {
    use core::sync::atomic::{AtomicI32, Ordering};
    let low_dword = &*(spin_lock as *const AtomicI32);
    while low_dword.fetch_or(1, Ordering::SeqCst) & 1 != 0 {
        while core::ptr::read_volatile(spin_lock) & 1 != 0 {
            core::hint::spin_loop();
        }
    }
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[cfg(not(feature = "smp"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KxReleaseSpinLock(_spin_lock: *mut ULONG_PTR) {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

/// SMP (spinlock.h CONFIG_SMP): `InterlockedAnd(64)(SpinLock, 0)` —
/// full-barrier atomic clear of the whole lock word.
#[cfg(feature = "smp")]
#[no_mangle]
pub unsafe extern "C" fn _rs_KxReleaseSpinLock(spin_lock: *mut ULONG_PTR) {
    core::sync::atomic::AtomicUsize::from_ptr(spin_lock as *mut usize)
        .swap(0, core::sync::atomic::Ordering::SeqCst);
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

// ============================================================================
// NMI list lock (ke_x.h — wraps KeAcquire/ReleaseSpinLock over the global
// KiNmiCallbackListLock; arch-conditional dispatch matches `KeAcquireSpinLock`
// macro: i386 → HAL fastcall, amd64/arm64 → cdecl Ke* primitives).
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireNmiListLock(old_irql: *mut KIRQL) {
    let lock = core::ptr::addr_of_mut!(KiNmiCallbackListLock);
    #[cfg(target_arch = "x86")]
    {
        *old_irql = KfAcquireSpinLock(lock);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        *old_irql = KeAcquireSpinLockRaiseToDpc(lock);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseNmiListLock(old_irql: KIRQL) {
    let lock = core::ptr::addr_of_mut!(KiNmiCallbackListLock);
    #[cfg(target_arch = "x86")]
    {
        KfReleaseSpinLock(lock, old_irql);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        KeReleaseSpinLock(lock, old_irql);
    }
}

// ============================================================================
// Stack-swap eligibility + alertability checks (ke_x.h)
//
// `KPROCESSOR_MODE` is a CCHAR enum: KernelMode = 0, UserMode = 1. The
// `Alerted` field is `[BOOLEAN; 2]` indexed by mode. ApcState lives inside
// `_KTHREAD__bindgen_ty_1` union (anonymous), accessed through
// `__bindgen_anon_1.ApcState` on all 3 arches.
// ============================================================================

/// `IsListEmpty` — head->Flink == head. Local replica of the macro.
#[inline(always)]
unsafe fn is_list_empty(head: *const _LIST_ENTRY) -> bool {
    (*head).Flink as *const _LIST_ENTRY == head
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiCheckThreadStackSwap(
    thread: PKTHREAD,
    wait_mode: KPROCESSOR_MODE,
) -> BOOLEAN {
    if wait_mode != 0 /* KernelMode */
        && (*thread).EnableStackSwap != 0
        && (*thread).Priority >= (LOW_REALTIME_PRIORITY + 9)
    {
        1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiCheckAlertability(
    thread: PKTHREAD,
    alertable: BOOLEAN,
    wait_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    if alertable != 0 {
        if (*thread).Alerted[wait_mode as usize] != 0 {
            (*thread).Alerted[wait_mode as usize] = 0;
            return STATUS_ALERTED;
        }
        if wait_mode != 0 /* KernelMode */ {
            // ApcListHead is `[LIST_ENTRY; 2]`; index 1 == UserMode.
            let user_apc_list = core::ptr::addr_of!(
                (*thread).__bindgen_anon_1.ApcState.ApcListHead[1]
            );
            if !is_list_empty(user_apc_list) {
                (*thread).__bindgen_anon_1.ApcState.UserApcPending = 1;
                return STATUS_USER_APC;
            }
        }
        if (*thread).Alerted[0] != 0 {
            (*thread).Alerted[0] = 0;
            return STATUS_ALERTED;
        }
    } else if wait_mode != 0 /* KernelMode */
        && (*thread).__bindgen_anon_1.ApcState.UserApcPending != 0
    {
        return STATUS_USER_APC;
    }
    STATUS_WAIT_0
}

// ============================================================================
// Ready-queue select / new-priority (ke_x.h)
//
// `KiSelectReadyThread` finds the highest-priority ready thread for a CPU at
// or above a floor. Bit math mirrors the BitScanReverse in the C body:
// `31 - leading_zeros(PrioritySet)` gives the position of the highest set
// bit (BitScanReverse's output). The KTHREAD pointer is recovered from
// the LIST_ENTRY via CONTAINING_RECORD on `__bindgen_anon_3` — the union
// containing WaitListEntry is at the same offset on all 3 arches.
//
// `KiComputeNewPriority` is the dynamic-priority decay path from the C body:
// Priority -= PriorityDecrement + Adjustment, then clamped up to BasePriority.
// Realtime threads (Priority >= LOW_REALTIME_PRIORITY = 16) are not decayed.
// Wrapping arithmetic mirrors the C (priorities are bounded 0..16 here, so
// signed overflow shouldn't occur in practice — but `wrapping_*` keeps the
// translation literal regardless).
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _rs_KiSelectReadyThread(priority: LONG, prcb: PKPRCB) -> PKTHREAD {
    let priority_set: ULONG = (*prcb).ReadySummary >> priority;
    if priority_set == 0 {
        return core::ptr::null_mut();
    }
    let high_priority: u32 = 31 - priority_set.leading_zeros() + priority as u32;
    let list_head =
        core::ptr::addr_of_mut!((*prcb).DispatcherReadyListHead[high_priority as usize]);
    let list_entry = (*list_head).Flink;
    let thread = CONTAINING_RECORD!(list_entry, _KTHREAD, __bindgen_anon_3);
    let wait_list_entry =
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.WaitListEntry);
    if RemoveEntryList(wait_list_entry) {
        (*prcb).ReadySummary ^= 1u32 << high_priority;
    }
    thread
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiComputeNewPriority(
    thread: PKTHREAD,
    adjustment: i8,
) -> i8 {
    let mut priority: i8 = (*thread).Priority;
    if priority < LOW_REALTIME_PRIORITY {
        priority = priority
            .wrapping_sub((*thread).PriorityDecrement.wrapping_add(adjustment));
        if priority < (*thread).BasePriority {
            priority = (*thread).BasePriority;
        }
        (*thread).PriorityDecrement = 0;
    }
    priority
}

// ============================================================================
// Unwait + queue-ready helpers (ke_x.h)
//
// `KxUnwaitThread` walks the wait list rooted at `Object->WaitListHead`,
// pulling each KWAIT_BLOCK and waking its thread via `KiUnwaitThread`. The
// list-empty exit condition matches the C `do { ... } while (Flink != head)`
// loop — `KiUnwaitThread` removes the wait blocks as it goes.
//
// `KxUnwaitThreadForEvent` is the early-out variant for KEVENT: the first
// `WaitAny` block consumes the signal (un-signals the event) and breaks;
// remaining blocks are woken with STATUS_KERNEL_APC.
//
// `KxQueueReadyThread` is the UP-only path of the C body (`#ifdef CONFIG_SMP`
// affinity check folded away — UP always takes the THIS-CPU branch). Sets
// State=Ready, captures Preempted, clears Preempted, stamps WaitTime from
// `KeTickCount.LowPart`, head/tail-inserts on the priority bucket, OR-sets
// the corresponding ReadySummary bit, and releases the (UP no-op) PRCB lock.
// ============================================================================

#[inline(always)]
unsafe fn kt_set_wait_time(thread: PKTHREAD, val: ULONG) {
    #[cfg(target_arch = "x86")]
    {
        (*thread).WaitTime = val;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*thread).__bindgen_anon_5.__bindgen_anon_7.WaitTime = val;
    }
}

// The C trampolines for `KxUnwaitThread`/`KxUnwaitThreadForEvent` carry an
// added defensive null-check on `WaitBlock->Thread` (not present in the
// original ke_x.h FORCEINLINE) — a corrupt WaitBlock with a NULL thread is
// removed from the list and skipped instead of dereferenced. Preserve the
// guard in the Rust translation so this round is a strict no-op behaviorally.
#[no_mangle]
pub unsafe extern "C" fn _rs_KxUnwaitThread(
    object: *mut _DISPATCHER_HEADER,
    increment: LONG,
) {
    let wait_list = core::ptr::addr_of_mut!((*object).WaitListHead);
    let mut wait_entry = (*wait_list).Flink;
    loop {
        let wait_block: PKWAIT_BLOCK =
            CONTAINING_RECORD!(wait_entry, _KWAIT_BLOCK, WaitListEntry);
        let wait_thread = (*wait_block).Thread;
        if wait_thread.is_null() {
            RemoveEntryList(wait_entry);
            wait_entry = (*wait_list).Flink;
            if wait_entry as *const _LIST_ENTRY == wait_list as *const _LIST_ENTRY {
                break;
            }
            continue;
        }
        let wait_key: LONG_PTR = if (*wait_block).WaitType == WAIT_ANY {
            (*wait_block).WaitKey as LONG_PTR
        } else {
            STATUS_KERNEL_APC
        };
        KiUnwaitThread(wait_thread, wait_key, increment);
        wait_entry = (*wait_list).Flink;
        if wait_entry as *const _LIST_ENTRY == wait_list as *const _LIST_ENTRY {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KxUnwaitThreadForEvent(event: PKEVENT, increment: LONG) {
    let header = core::ptr::addr_of_mut!((*event).Header);
    let wait_list = core::ptr::addr_of_mut!((*header).WaitListHead);
    let mut wait_entry = (*wait_list).Flink;
    loop {
        let wait_block: PKWAIT_BLOCK =
            CONTAINING_RECORD!(wait_entry, _KWAIT_BLOCK, WaitListEntry);
        let wait_thread = (*wait_block).Thread;
        if wait_thread.is_null() {
            RemoveEntryList(wait_entry);
            wait_entry = (*wait_list).Flink;
            if wait_entry as *const _LIST_ENTRY == wait_list as *const _LIST_ENTRY {
                break;
            }
            continue;
        }
        if (*wait_block).WaitType == WAIT_ANY {
            // Un-signal the event and unwait this one thread, then bail.
            (*header).SignalState = 0;
            KiUnwaitThread(
                wait_thread,
                (*wait_block).WaitKey as LONG_PTR,
                increment,
            );
            break;
        }
        KiUnwaitThread(wait_thread, STATUS_KERNEL_APC, increment);
        wait_entry = (*wait_list).Flink;
        if wait_entry as *const _LIST_ENTRY == wait_list as *const _LIST_ENTRY {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KxQueueReadyThread(thread: PKTHREAD, prcb: PKPRCB) {
    // UP: the SMP affinity gate `Thread->Affinity & Prcb->SetMember` is
    // unconditionally true (`#else if (TRUE)` in ke_x.h) — single CPU.
    // SMP: a thread not affine to this CPU is deferred instead (ke_x.h:1387).
    #[cfg(feature = "smp")]
    {
        if (*thread).Affinity & (*prcb).SetMember == 0 {
            kt_set_state(thread, KTHREAD_STATE_DEFERRED_READY);
            kt_set_deferred_processor(thread, (*prcb).Number as u8);
            _rs_KiReleasePrcbLock(prcb);
            KiDeferredReadyThread(thread);
            return;
        }
    }
    kt_set_state(thread, KTHREAD_STATE_READY);
    let priority = (*thread).Priority;
    let preempted = (*thread).Preempted != 0;
    (*thread).Preempted = 0;
    kt_set_wait_time(
        thread,
        core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.LowPart)),
    );
    let list_head =
        core::ptr::addr_of_mut!((*prcb).DispatcherReadyListHead[priority as usize]);
    let wait_list_entry =
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.WaitListEntry);
    if preempted {
        InsertHeadList(list_head, wait_list_entry);
    } else {
        InsertTailList(list_head, wait_list_entry);
    }
    (*prcb).ReadySummary |= 1u32 << (priority as u32);
    // KiReleasePrcbLock — UP no-op; SMP: really drop the PRCB lock.
    #[cfg(feature = "smp")]
    _rs_KiReleasePrcbLock(prcb);
}

// ============================================================================
// Guarded mutex (ke_x.h)
//
// The Count field is a packed encoding: bit 0 = "lock available" (`GM_LOCK_BIT`),
// bit 1 = "wakeup pending" (`GM_LOCK_WAITER_WOKEN`), bits ≥ 2 = waiter count
// (each waiter contributes `GM_LOCK_WAITER_INC` = 4). The fast path on acquire
// is `InterlockedBitTestAndReset(Count, 0)` — succeed iff bit 0 was set; on
// failure, fall through to `KiAcquireGuardedMutex` which contests via
// the embedded KGATE. Release: add GM_LOCK_BIT atomically; if there were
// waiters (OldValue != 0) and no wake is in flight (`!(OldValue & WAITER_WOKEN)`),
// CAS the count to mark wake pending and pop one waiter, then signal the gate.
//
// `KeEnterGuardedRegionThread`/`Leave...` are macros that decrement/increment
// `Thread->SpecialApcDisable`. Inlined locally — see helpers below.
//
// `KiCheckForKernelApcDelivery` calling convention is stdcall on i386 (no
// args), cdecl elsewhere; both are declared in the per-arch extern blocks
// above.
// ============================================================================

#[inline(always)]
unsafe fn count_atomic(gm: PKGUARDED_MUTEX) -> &'static core::sync::atomic::AtomicI32 {
    core::sync::atomic::AtomicI32::from_ptr(core::ptr::addr_of_mut!((*gm).Count))
}

/// `KeEnterGuardedRegionThread` macro: disable special APCs.
#[inline(always)]
unsafe fn enter_guarded_region_thread(thread: PKTHREAD) {
    let p = kt_special_apc_disable_ptr(thread);
    *p = (*p).wrapping_sub(1);
}

/// `KeLeaveGuardedRegionThread` macro: re-enable special APCs; if the count
/// reached 0 and a kernel-mode APC is queued, drain it.
#[inline(always)]
unsafe fn leave_guarded_region_thread(thread: PKTHREAD) {
    let p = kt_special_apc_disable_ptr(thread);
    *p = (*p).wrapping_add(1);
    if *p == 0 {
        // ApcListHead[KernelMode=0] under ApcState (anon union @ index 0).
        let kernel_apc_head =
            core::ptr::addr_of!((*thread).__bindgen_anon_1.ApcState.ApcListHead[0]);
        if !is_list_empty(kernel_apc_head) {
            KiCheckForKernelApcDelivery();
        }
    }
}

#[inline(always)]
unsafe fn gm_special_apc_disable_ptr(gm: PKGUARDED_MUTEX) -> *mut i16 {
    core::ptr::addr_of_mut!((*gm).__bindgen_anon_1.__bindgen_anon_1.SpecialApcDisable)
}

#[no_mangle]
pub unsafe extern "C" fn _rs__KeInitializeGuardedMutex(gm: PKGUARDED_MUTEX) {
    (*gm).Count = GM_LOCK_BIT;
    (*gm).Owner = core::ptr::null_mut();
    (*gm).Contention = 0;
    KeInitializeGate(core::ptr::addr_of_mut!((*gm).Gate));
}

#[no_mangle]
pub unsafe extern "C" fn _rs__KeAcquireGuardedMutexUnsafe(gm: PKGUARDED_MUTEX) {
    let thread = ke_get_current_thread();
    // InterlockedBitTestAndReset(Count, GM_LOCK_BIT_V): atomically clear bit 0,
    // return TRUE iff old bit was set.
    let mask = !(1i32 << GM_LOCK_BIT_V);
    let old = count_atomic(gm).fetch_and(mask, core::sync::atomic::Ordering::SeqCst);
    if old & (1 << GM_LOCK_BIT_V) == 0 {
        // Lock was held — contested path
        KiAcquireGuardedMutex(gm);
    }
    (*gm).Owner = thread;
}

#[no_mangle]
pub unsafe extern "C" fn _rs__KeReleaseGuardedMutexUnsafe(gm: PKGUARDED_MUTEX) {
    (*gm).Owner = core::ptr::null_mut();
    let old = count_atomic(gm)
        .fetch_add(GM_LOCK_BIT, core::sync::atomic::Ordering::SeqCst);
    if old != 0 && (old & GM_LOCK_WAITER_WOKEN) == 0 {
        // Reflect the just-applied add into our local copy and try to CAS
        // it to "wake pending, one fewer waiter".
        let old_after_add = old + GM_LOCK_BIT;
        let new_value = old_after_add + GM_LOCK_WAITER_WOKEN - GM_LOCK_WAITER_INC;
        if count_atomic(gm)
            .compare_exchange(
                old_after_add,
                new_value,
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
        {
            KeSignalGateBoostPriority(core::ptr::addr_of_mut!((*gm).Gate));
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs__KeAcquireGuardedMutex(gm: PKGUARDED_MUTEX) {
    let thread = ke_get_current_thread();
    enter_guarded_region_thread(thread);
    let mask = !(1i32 << GM_LOCK_BIT_V);
    let old = count_atomic(gm).fetch_and(mask, core::sync::atomic::Ordering::SeqCst);
    if old & (1 << GM_LOCK_BIT_V) == 0 {
        KiAcquireGuardedMutex(gm);
    }
    (*gm).Owner = thread;
    *gm_special_apc_disable_ptr(gm) = *kt_special_apc_disable_ptr(thread);
}

#[no_mangle]
pub unsafe extern "C" fn _rs__KeReleaseGuardedMutex(gm: PKGUARDED_MUTEX) {
    let thread = ke_get_current_thread();
    (*gm).Owner = core::ptr::null_mut();
    let old = count_atomic(gm)
        .fetch_add(GM_LOCK_BIT, core::sync::atomic::Ordering::SeqCst);
    if old != 0 && (old & GM_LOCK_WAITER_WOKEN) == 0 {
        let old_after_add = old + GM_LOCK_BIT;
        let new_value = old_after_add + GM_LOCK_WAITER_WOKEN - GM_LOCK_WAITER_INC;
        if count_atomic(gm)
            .compare_exchange(
                old_after_add,
                new_value,
                core::sync::atomic::Ordering::SeqCst,
                core::sync::atomic::Ordering::SeqCst,
            )
            .is_ok()
        {
            KeSignalGateBoostPriority(core::ptr::addr_of_mut!((*gm).Gate));
        }
    }
    leave_guarded_region_thread(thread);
}

#[no_mangle]
pub unsafe extern "C" fn _rs__KeTryToAcquireGuardedMutex(gm: PKGUARDED_MUTEX) -> BOOLEAN {
    let thread = ke_get_current_thread();
    enter_guarded_region_thread(thread);
    let mask = !(1i32 << GM_LOCK_BIT_V);
    let old = count_atomic(gm).fetch_and(mask, core::sync::atomic::Ordering::SeqCst);
    if old & (1 << GM_LOCK_BIT_V) == 0 {
        // Couldn't grab the bit — undo the guarded region.
        leave_guarded_region_thread(thread);
        core::hint::spin_loop();
        return 0;
    }
    (*gm).Owner = thread;
    *gm_special_apc_disable_ptr(gm) = *kt_special_apc_disable_ptr(thread);
    1
}

// ============================================================================
// Timer helpers (ke_x.h)
//
// `KiRecalculateDueTime` is the relative-due-time path of any wait: returns
// the original DueTime unchanged if the caller passed an absolute one (high
// bit clear → it's already a system-time stamp), or recomputes a new
// relative due-time from KeQueryInterruptTime() otherwise.
//
// `KiComputeDueTime` is the harder cousin used by KeSetTimerEx. It
// converts an absolute due-time to a relative one by subtracting
// SystemTime, marks the timer Absolute, and computes the timer-table Hand.
// If the absolute deadline is already in the past it short-circuits with
// SignalState=1.
//
// `KxInsertTimer` and `KxRemoveTreeTimer` walk `KiTimerTableListHead[Hand]`,
// the chained-list timer wheel, and call out to `KiInsertTimerTable` /
// `KiCompleteTimer` (real C exports) for the actual table mutation.
//
// `KxSetTimerForThreadWait` is a sibling of `KiComputeDueTime` used for
// thread-wait timers — same absolute/relative branch, but doesn't mark
// SignalState (the wait machinery handles that).
//
// `dh_set_*` helpers below mirror `inline::ke::dh_set_*` — the
// DISPATCHER_HEADER union nesting is identical on all 3 arches, so no cfg
// is needed for these accessors.
// ============================================================================

#[inline(always)]
unsafe fn dh_set_absolute(header: *mut _DISPATCHER_HEADER, val: u8) {
    (*header)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .__bindgen_anon_1
        .__bindgen_anon_1
        .__bindgen_anon_1
        .set_Absolute(val);
}

#[inline(always)]
unsafe fn dh_set_inserted(header: *mut _DISPATCHER_HEADER, val: u8) {
    (*header)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .__bindgen_anon_3
        .Inserted = val;
}

#[inline(always)]
unsafe fn dh_set_hand(header: *mut _DISPATCHER_HEADER, val: u8) {
    (*header).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Hand = val;
}

#[inline(always)]
unsafe fn dh_get_hand(header: *const _DISPATCHER_HEADER) -> u8 {
    (*header).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Hand
}

/// Read the high1/low/high2 system-time triple from `SharedUserData` and
/// reconstruct the 64-bit value. Loop until high1 == high2 to defeat torn
/// reads. Used as `KeQuerySystemTime` on amd64/arm64 (it's a macro on those
/// arches; i386 has a real export).
#[cfg(not(target_arch = "x86"))]
#[inline(always)]
unsafe fn ke_query_system_time_inline(out: *mut _LARGE_INTEGER) {
    loop {
        let high1 = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.High1Time));
        let low = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.LowPart));
        let high2 = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.High2Time));
        if high1 == high2 {
            (*out).QuadPart = ((high1 as i64) << 32) | (low as u32 as i64);
            break;
        }
    }
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn ke_query_system_time_inline(out: *mut _LARGE_INTEGER) {
    KeQuerySystemTime(out);
}

/// `KeQueryInterruptTime` is a macro on amd64/arm64 (read InterruptTime from
/// SharedUserData). On i386 it's a real export; either way, reading the
/// SharedUserData InterruptTime fields directly works on all 3 arches.
#[inline(always)]
unsafe fn ke_query_interrupt_time_inline() -> u64 {
    loop {
        let high1 = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).InterruptTime.High1Time));
        let low = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).InterruptTime.LowPart));
        let high2 = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).InterruptTime.High2Time));
        if high1 == high2 {
            return ((high1 as u64) << 32) | (low as u32 as u64);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiRecalculateDueTime(
    original_due_time: *mut _LARGE_INTEGER,
    due_time: *mut _LARGE_INTEGER,
    new_due_time: *mut _LARGE_INTEGER,
) -> *mut _LARGE_INTEGER {
    if (*original_due_time).QuadPart >= 0 {
        return original_due_time;
    }
    (*new_due_time).QuadPart = ke_query_interrupt_time_inline() as i64;
    (*new_due_time).QuadPart -= (*due_time).QuadPart;
    new_due_time
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KxInsertTimer(timer: PKTIMER, hand: ULONG) {
    let header = core::ptr::addr_of_mut!((*timer).Header);
    // ASSERT(KeGetCurrentIrql() >= SYNCH_LEVEL) — skip in retail
    let _ = header; // keep the addr_of_mut! to mirror inline::ke shape
    // Acquire timer lock + release dispatcher lock (UP: both no-ops, but the
    // K3/K6 trampolines are wired up to point at our `_rs_*` impls).
    let lock_queue = _rs_KiAcquireTimerLock(hand);
    _rs_KiReleaseDispatcherLockFromSynchLevel();
    if KiInsertTimerTable(timer, hand) != 0 {
        KiCompleteTimer(timer, lock_queue);
    } else {
        _rs_KiReleaseTimerLock(lock_queue);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiComputeDueTime(
    timer: PKTIMER,
    due_time: _LARGE_INTEGER,
    hand: *mut ULONG,
) -> BOOLEAN {
    let header = core::ptr::addr_of_mut!((*timer).Header);
    dh_set_absolute(header, 0);

    let mut effective = due_time;
    if due_time.QuadPart >= 0 {
        // Absolute time — convert to relative.
        let mut system_time: _LARGE_INTEGER = core::mem::zeroed();
        ke_query_system_time_inline(core::ptr::addr_of_mut!(system_time));
        let diff = system_time.QuadPart - due_time.QuadPart;
        dh_set_absolute(header, 1);
        if diff >= 0 {
            // Already expired.
            (*header).SignalState = 1;
            dh_set_hand(header, 0);
            (*timer).DueTime.QuadPart = 0;
            *hand = 0;
            return 0;
        }
        effective.QuadPart = diff;
    }

    let interrupt_time = ke_query_interrupt_time_inline() as i64;
    (*timer).DueTime.QuadPart = (interrupt_time - effective.QuadPart) as u64;
    *hand = ((((*timer).DueTime.QuadPart) / read_max_increment()) as ULONG)
        & (TIMER_TABLE_SIZE - 1);
    dh_set_hand(header, *hand as u8);
    dh_set_inserted(header, 1);
    1
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KxRemoveTreeTimer(timer: PKTIMER) {
    let header = core::ptr::addr_of_mut!((*timer).Header);
    let hand: ULONG = dh_get_hand(header) as ULONG & (TIMER_TABLE_SIZE - 1);
    let lock_queue = _rs_KiAcquireTimerLock(hand);
    dh_set_inserted(header, 0);
    if RemoveEntryList(core::ptr::addr_of_mut!((*timer).TimerListEntry)) {
        let timer_entry =
            core::ptr::addr_of_mut!(KiTimerTableListHead[hand as usize]);
        let entry_head = core::ptr::addr_of_mut!((*timer_entry).Entry);
        if is_list_empty(entry_head) {
            (*timer_entry).Time.u.HighPart = 0xFFFFFFFFu32;
        }
    }
    _rs_KiReleaseTimerLock(lock_queue);
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KxSetTimerForThreadWait(
    timer: PKTIMER,
    interval: _LARGE_INTEGER,
    hand: *mut ULONG,
) {
    let header = core::ptr::addr_of_mut!((*timer).Header);
    dh_set_absolute(header, 0);

    let mut effective = interval;
    if interval.QuadPart >= 0 {
        let mut system_time: _LARGE_INTEGER = core::mem::zeroed();
        ke_query_system_time_inline(core::ptr::addr_of_mut!(system_time));
        let diff = system_time.QuadPart - interval.QuadPart;
        dh_set_absolute(header, 1);
        if diff >= 0 {
            (*timer).DueTime.QuadPart = 0;
            *hand = 0;
            dh_set_hand(header, 0);
            return;
        }
        effective.QuadPart = diff;
    }

    let interrupt_time = ke_query_interrupt_time_inline() as i64;
    let due = (interrupt_time - effective.QuadPart) as u64;
    (*timer).DueTime.QuadPart = due;
    *hand = ((due / read_max_increment()) as ULONG) & (TIMER_TABLE_SIZE - 1);
    dh_set_hand(header, *hand as u8);
}

#[inline(always)]
unsafe fn read_max_increment() -> u64 {
    core::ptr::read_volatile(core::ptr::addr_of!(KeMaximumIncrement)) as u64
}
