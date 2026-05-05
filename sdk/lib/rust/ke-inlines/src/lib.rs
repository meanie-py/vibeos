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

use ntkrnl_sys::base::{BOOLEAN, KIRQL, NTSTATUS, UCHAR, ULONG, ULONG_PTR};
use ntkrnl_sys::ddk::{
    KPROCESSOR_MODE, PKDEVICE_QUEUE, PKLOCK_QUEUE_HANDLE, PKPRCB, PKSPIN_LOCK_QUEUE, PKTHREAD,
    _DISPATCHER_HEADER, _KPCR, _LIST_ENTRY,
};

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

extern "C" {
    /// `KeMaximumIncrement` — clock tick increment (100ns units), set at boot
    /// from the HAL. Used as the divisor in the timer table index computation.
    static KeMaximumIncrement: ULONG;

    /// `KiNmiCallbackListLock` — global spinlock guarding the NMI callback
    /// list. Defined in `ntoskrnl-rust::ke::bug` as a `pub static mut
    /// KSPIN_LOCK = 0` (== `ULONG_PTR`).
    static mut KiNmiCallbackListLock: ULONG_PTR;
}

// Spinlock primitives backing KiAcquire/ReleaseNmiListLock. amd64/arm64 use
// the cdecl `KeAcquireSpinLockRaiseToDpc` + `KeReleaseSpinLock` exports;
// i386 uses HAL's fastcall `KfAcquireSpinLock` / `KfReleaseSpinLock`.
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KeAcquireSpinLockRaiseToDpc(spin_lock: *mut ULONG_PTR) -> KIRQL;
    fn KeReleaseSpinLock(spin_lock: *mut ULONG_PTR, new_irql: KIRQL);
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KfAcquireSpinLock(spin_lock: *mut ULONG_PTR) -> KIRQL;
    fn KfReleaseSpinLock(spin_lock: *mut ULONG_PTR, new_irql: KIRQL);
}

/// SYNCH_LEVEL on UP kernel = DISPATCH_LEVEL = 2 (matches
/// `ntkrnl_sys::inline::ke::SYNCH_LEVEL`; replicated locally to avoid pulling
/// the whole `inline::ke` module into this crate).
const SYNCH_LEVEL: KIRQL = 2;

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
}

#[cfg(not(target_arch = "x86"))]
extern "C" {
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

/// Per-object dispatcher lock acquire — UP no-op. SMP version would spin on
/// `Object->Lock & KOBJECT_LOCK_BIT` and CAS-set the bit; not present here.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherObject(_object: *mut _DISPATCHER_HEADER) {}

/// Per-object dispatcher lock release — UP no-op. SMP version would
/// `InterlockedAnd(&Object->Lock, ~KOBJECT_LOCK_BIT)`.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherObject(_object: *mut _DISPATCHER_HEADER) {}

/// Acquire dispatcher lock by raising to SYNCH_LEVEL; returns previous IRQL.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherLock() -> KIRQL {
    KfRaiseIrql(SYNCH_LEVEL)
}

/// Release dispatcher lock by exiting through the dispatcher (which lowers
/// IRQL to OldIrql after draining deferred-ready / pending APCs).
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherLock(old_irql: KIRQL) {
    KiExitDispatcher(old_irql)
}

/// Acquire dispatcher lock when already at SYNCH_LEVEL — UP no-op (SMP would
/// `KeAcquireQueuedSpinLockAtDpcLevel(LockQueueDispatcherLock)`).
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireDispatcherLockAtSynchLevel() {}

/// Release dispatcher lock from SYNCH_LEVEL without lowering IRQL — UP no-op
/// (SMP would `KeReleaseQueuedSpinLockFromDpcLevel`).
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseDispatcherLockFromSynchLevel() {}

// ============================================================================
// Thread / PRCB locks (ke_x.h, #ifndef CONFIG_SMP)
//
// All UP no-ops — the kernel only runs on one CPU, so per-thread and
// per-PRCB locks are implicit. KiTryThreadLock returns FALSE on UP
// (matches the C body: `UNREFERENCED_PARAMETER(Thread); return FALSE;`).
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireThreadLock(_thread: PKTHREAD) {}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseThreadLock(_thread: PKTHREAD) {}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiTryThreadLock(_thread: PKTHREAD) -> BOOLEAN {
    0
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquirePrcbLock(_prcb: PKPRCB) {}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleasePrcbLock(_prcb: PKPRCB) {}

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

#[no_mangle]
pub unsafe extern "C" fn _rs_KiInsertDeferredReadyList(thread: PKTHREAD) {
    kt_set_state(thread, KTHREAD_STATE_DEFERRED_READY);
    kt_set_deferred_processor(thread, 0);
    KiDeferredReadyThread(thread);
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiRescheduleThread(_new_thread: BOOLEAN, _cpu: ULONG) {}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiSetThreadSwapBusy(_thread: PKTHREAD) {}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiCheckDeferredReadyList(_prcb: PKPRCB) {}

#[no_mangle]
pub unsafe extern "C" fn _rs_KiRequestApcInterrupt(_need_apc: BOOLEAN, _processor: UCHAR) {}

// ============================================================================
// Timer-lock management (ke_x.h, #ifndef CONFIG_SMP)
// ============================================================================

/// `KiAcquireTimerLock` — UP returns NULL (no real spinlock to take). SMP
/// would `KeAcquireQueuedSpinLockAtDpcLevel(&KiTimerTableLock[Hand])`.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAcquireTimerLock(_hand: ULONG) -> PKSPIN_LOCK_QUEUE {
    core::ptr::null_mut()
}

/// `KiReleaseTimerLock` — UP no-op.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiReleaseTimerLock(_lock_queue: PKSPIN_LOCK_QUEUE) {}

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

#[no_mangle]
pub unsafe extern "C" fn _rs_KxAcquireSpinLock(_spin_lock: *mut ULONG_PTR) {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[no_mangle]
pub unsafe extern "C" fn _rs_KxReleaseSpinLock(_spin_lock: *mut ULONG_PTR) {
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
