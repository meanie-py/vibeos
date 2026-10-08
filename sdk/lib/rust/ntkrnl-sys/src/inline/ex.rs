//! EX subsystem inline functions.
//!
//! Lookaside list operations, work item initialization, push locks,
//! fast references, rundown protection.

use crate::base::*;
use crate::ddk;

// ============================================================================
// Push lock constants (from ntoskrnl/include/internal/ex.h)
// ============================================================================

const EX_PUSH_LOCK_LOCK: usize = 0x1;
const EX_PUSH_LOCK_WAITING: usize = 0x2;
const EX_PUSH_LOCK_WAKING: usize = 0x4;
const EX_PUSH_LOCK_SHARE_INC: usize = 0x10;

// ============================================================================
// Lockdep taps (sync-instrumentation §4, feature = "lockdep").
//
// The pushlock API is the single all-paths acquire/release funnel (fast CAS
// here, slow park in Exf*), so tapping here gives lockdep a BALANCED per-thread
// held-set covering every pushlock — including the Mm WS lock (which is just a
// pushlock on Vm.WorkingSetMutex). The gmutex inlines in inline/ke.rs tap
// through the same pub(crate) helpers. The taps call ntoskrnl-rust's
// VibeosLockdep* by link name (this crate sits below it in the dep graph —
// same shape as VibeosFlightrecLog in inline/mm.rs). Zero cost when off.
//
// `mode`: 1 = shared, 2 = exclusive, 3 = guarded mutex.
// `site`: caller return address captured by an #[inline(never)] export shim
// (llvm.returnaddress) — 0 for direct-inline acquires.
// ============================================================================
#[cfg(feature = "lockdep")]
extern "C" {
    fn VibeosLockdepAcquire(thread: usize, lock: usize, mode: u32, site: usize);
    fn VibeosLockdepAcquireTry(thread: usize, lock: usize, mode: u32, site: usize);
    fn VibeosLockdepRelease(thread: usize, lock: usize);
}

/// Record a successful lock acquire: held-set push + Phase B order edges +
/// ABBA reverse-edge check.
#[cfg(feature = "lockdep")]
#[inline]
pub(crate) unsafe fn lockdep_acquire(lock: *mut core::ffi::c_void, mode: u32, site: usize) {
    VibeosLockdepAcquire(crate::mm::KeGetCurrentThread() as usize, lock as usize, mode, site);
}
#[cfg(not(feature = "lockdep"))]
#[inline(always)]
pub(crate) unsafe fn lockdep_acquire(_lock: *mut core::ffi::c_void, _mode: u32, _site: usize) {}

/// Try-acquire success: held-set only, NO order edges (a try never parks, so
/// a try-reversed order cannot deadlock — recording it would be an ABBA FP).
#[cfg(feature = "lockdep")]
#[inline]
pub(crate) unsafe fn lockdep_acquire_try(lock: *mut core::ffi::c_void, mode: u32, site: usize) {
    VibeosLockdepAcquireTry(crate::mm::KeGetCurrentThread() as usize, lock as usize, mode, site);
}
#[cfg(not(feature = "lockdep"))]
#[inline(always)]
pub(crate) unsafe fn lockdep_acquire_try(_lock: *mut core::ffi::c_void, _mode: u32, _site: usize) {
}

/// Drop a lock from the current thread's held-set on release.
#[cfg(feature = "lockdep")]
#[inline]
pub(crate) unsafe fn lockdep_release(lock: *mut core::ffi::c_void) {
    VibeosLockdepRelease(crate::mm::KeGetCurrentThread() as usize, lock as usize);
}
#[cfg(not(feature = "lockdep"))]
#[inline(always)]
pub(crate) unsafe fn lockdep_release(_lock: *mut core::ffi::c_void) {}

// ============================================================================
// Fast reference constants
// ============================================================================

// ReactOS uses 7 for ALL architectures (3 low bits), NOT the Windows NT
// value of 15 for 64-bit. The SD cache stores pointers at 8-byte-aligned
// offsets (SECURITY_DESCRIPTOR_HEADER.SecurityDescriptor is at offset 24),
// which requires MAX_FAST_REFS <= 7 to preserve pointer alignment.
pub const MAX_FAST_REFS: usize = 7;

// ============================================================================
// ExInitializeWorkItem (macro from wdm.h)
// ============================================================================

/// ExInitializeWorkItem — initialize a WORK_QUEUE_ITEM.
/// C: `Item->List.Flink = NULL; Item->WorkerRoutine = Routine; Item->Parameter = Context;`
#[inline]
pub unsafe fn ExInitializeWorkItem(
    item: *mut ddk::_WORK_QUEUE_ITEM,
    routine: ddk::PWORKER_THREAD_ROUTINE,
    context: PVOID,
) {
    (*item).List.Flink = core::ptr::null_mut();
    (*item).WorkerRoutine = routine;
    (*item).Parameter = context;
}

// ============================================================================
// ExAllocateFromNPagedLookasideList (FORCEINLINE from wdm.h)
// ============================================================================

/// ExAllocateFromNPagedLookasideList — allocate from a non-paged lookaside list.
#[inline]
pub unsafe fn ExAllocateFromNPagedLookasideList(
    lookaside: ddk::PNPAGED_LOOKASIDE_LIST,
) -> PVOID {
    let l = core::ptr::addr_of_mut!((*lookaside).L);
    (*l).TotalAllocates += 1;

    let list_head = core::ptr::addr_of_mut!((*l).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let entry = crate::interlocked::InterlockedPopEntrySList(list_head);

    if !entry.is_null() {
        entry as PVOID
    } else {
        (*l).__bindgen_anon_2.AllocateMisses += 1;
        match (*l).__bindgen_anon_4.Allocate {
            Some(f) => f((*l).Type, (*l).Size as ddk::SIZE_T, (*l).Tag),
            None => core::ptr::null_mut(),
        }
    }
}

// ============================================================================
// ExFreeToNPagedLookasideList (FORCEINLINE from wdm.h)
// ============================================================================

/// ExFreeToNPagedLookasideList — free to a non-paged lookaside list.
#[inline]
pub unsafe fn ExFreeToNPagedLookasideList(
    lookaside: ddk::PNPAGED_LOOKASIDE_LIST,
    entry: PVOID,
) {
    let l = core::ptr::addr_of_mut!((*lookaside).L);
    (*l).TotalFrees += 1;

    let list_head = core::ptr::addr_of_mut!((*l).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let depth = crate::interlocked::ExQueryDepthSList(list_head);

    if depth >= (*l).Depth {
        (*l).__bindgen_anon_3.FreeMisses += 1;
        if let Some(f) = (*l).__bindgen_anon_5.Free {
            f(entry);
        }
    } else {
        crate::interlocked::InterlockedPushEntrySList(
            list_head,
            entry as crate::base::PSLIST_ENTRY,
        );
    }
}

// ============================================================================
// ExAllocateFromPagedLookasideList (FORCEINLINE from wdm.h)
// ============================================================================

/// ExAllocateFromPagedLookasideList — allocate from a paged lookaside list.
#[inline]
pub unsafe fn ExAllocateFromPagedLookasideList(
    lookaside: ddk::PPAGED_LOOKASIDE_LIST,
) -> PVOID {
    let l = core::ptr::addr_of_mut!((*lookaside).L);
    (*l).TotalAllocates += 1;

    let list_head = core::ptr::addr_of_mut!((*l).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let entry = crate::interlocked::InterlockedPopEntrySList(list_head);

    if !entry.is_null() {
        entry as PVOID
    } else {
        (*l).__bindgen_anon_2.AllocateMisses += 1;
        match (*l).__bindgen_anon_4.Allocate {
            Some(f) => f((*l).Type, (*l).Size as ddk::SIZE_T, (*l).Tag),
            None => core::ptr::null_mut(),
        }
    }
}

// ============================================================================
// ExFreeToPagedLookasideList (FORCEINLINE from wdm.h)
// ============================================================================

/// ExFreeToPagedLookasideList — free to a paged lookaside list.
#[inline]
pub unsafe fn ExFreeToPagedLookasideList(
    lookaside: ddk::PPAGED_LOOKASIDE_LIST,
    entry: PVOID,
) {
    let l = core::ptr::addr_of_mut!((*lookaside).L);
    (*l).TotalFrees += 1;

    let list_head = core::ptr::addr_of_mut!((*l).__bindgen_anon_1.ListHead)
        as crate::base::PSLIST_HEADER;
    let depth = crate::interlocked::ExQueryDepthSList(list_head);

    if depth >= (*l).Depth {
        (*l).__bindgen_anon_3.FreeMisses += 1;
        if let Some(f) = (*l).__bindgen_anon_5.Free {
            f(entry);
        }
    } else {
        crate::interlocked::InterlockedPushEntrySList(
            list_head,
            entry as crate::base::PSLIST_ENTRY,
        );
    }
}

// ============================================================================
// ExInitializePushLock (macro → _ExInitializePushLock from ex.h)
// ============================================================================

/// ExInitializePushLock — set push lock value to zero.
/// C: `PushLock->Ptr = 0;`
#[inline]
pub unsafe fn ExInitializePushLock(push_lock: ddk::PEX_PUSH_LOCK) {
    core::ptr::write_volatile(push_lock as *mut usize, 0);
}

// ============================================================================
// ExAcquirePushLockExclusive (FORCEINLINE from ex.h)
// ============================================================================

/// ExAcquirePushLockExclusive — fast-path: set Lock bit, slow-path: ExfAcquirePushLockExclusive.
/// C: `if (InterlockedBitTestAndSet((PLONG)PushLock, EX_PUSH_LOCK_LOCK_V)) ExfAcquirePushLockExclusive(PushLock);`
#[inline]
pub unsafe fn ExAcquirePushLockExclusive(push_lock: ddk::PEX_PUSH_LOCK) {
    ExAcquirePushLockExclusiveSited(push_lock, 0)
}

/// Sited variant: the export shims pass their return address so lockdep held
/// entries / order edges name the true acquiring caller. Identical otherwise.
#[inline]
pub unsafe fn ExAcquirePushLockExclusiveSited(push_lock: ddk::PEX_PUSH_LOCK, site: usize) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    let old = atomic.fetch_or(EX_PUSH_LOCK_LOCK, Ordering::SeqCst);
    if old & EX_PUSH_LOCK_LOCK != 0 {
        ddk::ExfAcquirePushLockExclusive(push_lock);
    }
    // held now (fast or slow path) — record excl
    lockdep_acquire(push_lock as _, 2, site);
}

// ============================================================================
// ExAcquirePushLockShared (FORCEINLINE from ex.h)
// ============================================================================

/// ExAcquirePushLockShared — fast-path: CAS 0 → LOCK|SHARE_INC, slow-path: ExfAcquirePushLockShared.
#[inline]
pub unsafe fn ExAcquirePushLockShared(push_lock: ddk::PEX_PUSH_LOCK) {
    ExAcquirePushLockSharedSited(push_lock, 0)
}

/// Sited variant — see ExAcquirePushLockExclusiveSited.
#[inline]
pub unsafe fn ExAcquirePushLockSharedSited(push_lock: ddk::PEX_PUSH_LOCK, site: usize) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    let new_value = EX_PUSH_LOCK_LOCK | EX_PUSH_LOCK_SHARE_INC;
    if atomic
        .compare_exchange(0, new_value, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        ddk::ExfAcquirePushLockShared(push_lock);
    }
    // held now (fast or slow path) — record shared
    lockdep_acquire(push_lock as _, 1, site);
}

// ============================================================================
// ExReleasePushLock (FORCEINLINE from ex.h)
// ============================================================================

/// ExReleasePushLock — fast-path: CAS to decrement share or clear lock, slow-path: ExfReleasePushLock.
#[inline]
pub unsafe fn ExReleasePushLock(push_lock: ddk::PEX_PUSH_LOCK) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    lockdep_release(push_lock as _); // pop from held-set before we drop it
    let atomic = &*(push_lock as *const AtomicUsize);
    let old_value = atomic.load(Ordering::Relaxed);

    // Check if there are multiple shared owners
    let shared = old_value >> 4; // Share count from bits 4+
    let new_value = if shared > 1 {
        old_value - EX_PUSH_LOCK_SHARE_INC
    } else {
        0
    };

    if (old_value & EX_PUSH_LOCK_WAITING != 0)
        || atomic
            .compare_exchange(old_value, new_value, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
    {
        ddk::ExfReleasePushLock(push_lock);
    }
}

// ============================================================================
// ExReleasePushLockExclusive (FORCEINLINE from ex.h)
// ============================================================================

/// ExReleasePushLockExclusive — atomic subtract LOCK, wake if waiters.
/// C: `OldValue = InterlockedExchangeAddSizeT(PushLock, -LOCK); if (Waiting && !Waking) ExfTryToWakePushLock(PushLock);`
#[inline]
pub unsafe fn ExReleasePushLockExclusive(push_lock: ddk::PEX_PUSH_LOCK) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    lockdep_release(push_lock as _); // pop from held-set before we drop it
    let atomic = &*(push_lock as *const AtomicUsize);
    let old_value = atomic.fetch_sub(EX_PUSH_LOCK_LOCK, Ordering::SeqCst);

    if (old_value & EX_PUSH_LOCK_WAITING != 0)
        && (old_value & EX_PUSH_LOCK_WAKING == 0)
    {
        ddk::ExfTryToWakePushLock(push_lock);
    }
}

// ============================================================================
// ExConvertPushLockSharedToExclusive (FORCEINLINE from ex.h)
// ============================================================================

/// ExConvertPushLockSharedToExclusive — CAS LOCK|SHARE_INC → LOCK.
/// Returns TRUE if conversion succeeded (single shared → exclusive).
#[inline]
pub unsafe fn ExConvertPushLockSharedToExclusive(
    push_lock: ddk::PEX_PUSH_LOCK,
) -> BOOLEAN {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    let expected = EX_PUSH_LOCK_LOCK | EX_PUSH_LOCK_SHARE_INC;
    match atomic.compare_exchange(
        expected,
        EX_PUSH_LOCK_LOCK,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(_) => 1,
        Err(_) => 0,
    }
}

// ============================================================================
// EX Fast Reference functions (FORCEINLINE from ex.h)
// ============================================================================

/// ExGetObjectFastReference — return object pointer with low bits masked.
/// C: `return (PVOID)(FastRef.Value & ~MAX_FAST_REFS);`
#[inline]
pub unsafe fn ExGetObjectFastReference(value: usize) -> PVOID {
    (value & !MAX_FAST_REFS) as PVOID
}

/// ExGetCountFastReference — return reference count from low bits.
/// C: `return (ULONG)FastRef.RefCnt;`
#[inline]
pub unsafe fn ExGetCountFastReference(value: usize) -> ULONG {
    (value & MAX_FAST_REFS) as ULONG
}

/// ExInitializeFastReference — set fast ref to Object|MAX_FAST_REFS or clear.
#[inline]
pub unsafe fn ExInitializeFastReference(
    fast_ref: ddk::PEX_FAST_REF,
    object: PVOID,
) {
    if object.is_null() {
        core::ptr::write_volatile(fast_ref as *mut usize, 0);
    } else {
        core::ptr::write_volatile(
            fast_ref as *mut usize,
            (object as usize) | MAX_FAST_REFS,
        );
    }
}

/// ExAcquireFastReference — CAS loop: decrement RefCnt if > 0, return old value.
/// Returns old EX_FAST_REF value as usize.
#[inline]
pub unsafe fn ExAcquireFastReference(fast_ref: ddk::PEX_FAST_REF) -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(fast_ref as *const AtomicUsize);
    loop {
        let old_value = atomic.load(Ordering::Relaxed);
        if old_value & MAX_FAST_REFS != 0 {
            let new_value = old_value - 1;
            if atomic
                .compare_exchange(old_value, new_value, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return old_value;
            }
            continue;
        }
        return old_value;
    }
}

/// ExInsertFastReference — CAS loop: add MAX_FAST_REFS to count.
/// Only succeeds if current count is 0 and pointer matches.
#[inline]
pub unsafe fn ExInsertFastReference(
    fast_ref: ddk::PEX_FAST_REF,
    object: PVOID,
) -> BOOLEAN {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(fast_ref as *const AtomicUsize);
    let obj_usize = object as usize;
    loop {
        let old_value = atomic.load(Ordering::Relaxed);
        // Check: count + MAX_FAST_REFS > MAX_FAST_REFS (count is non-zero) OR pointer changed
        if ((old_value & MAX_FAST_REFS) + MAX_FAST_REFS > MAX_FAST_REFS)
            || ((old_value & !MAX_FAST_REFS) != obj_usize)
        {
            return 0; // FALSE
        }
        let new_value = old_value + MAX_FAST_REFS;
        if atomic
            .compare_exchange(old_value, new_value, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return 1; // TRUE
        }
    }
}

/// ExReleaseFastReference — CAS loop: increment RefCnt.
/// Fails if count is at MAX_FAST_REFS or pointer doesn't match.
#[inline]
pub unsafe fn ExReleaseFastReference(
    fast_ref: ddk::PEX_FAST_REF,
    object: PVOID,
) -> BOOLEAN {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(fast_ref as *const AtomicUsize);
    loop {
        let old_value = atomic.load(Ordering::Relaxed);
        // XOR trick: if pointer matches, result is just RefCnt (< MAX_FAST_REFS)
        if (old_value ^ (object as usize)) >= MAX_FAST_REFS {
            return 0; // FALSE — full or pointer mismatch
        }
        let new_value = old_value + 1;
        if atomic
            .compare_exchange(old_value, new_value, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return 1; // TRUE
        }
    }
}

/// ExSwapFastReference — atomically exchange fast ref with new Object|MAX_FAST_REFS.
/// Returns old EX_FAST_REF value as usize.
#[inline]
pub unsafe fn ExSwapFastReference(
    fast_ref: ddk::PEX_FAST_REF,
    object: PVOID,
) -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(fast_ref as *const AtomicUsize);
    let new_value = if object.is_null() {
        0
    } else {
        (object as usize) | MAX_FAST_REFS
    };
    atomic.swap(new_value, Ordering::SeqCst)
}

/// ExCompareSwapFastReference — CAS loop: swap fast ref if pointer matches OldObject.
/// Returns old EX_FAST_REF value as usize.
#[inline]
pub unsafe fn ExCompareSwapFastReference(
    fast_ref: ddk::PEX_FAST_REF,
    object: PVOID,
    old_object: PVOID,
) -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(fast_ref as *const AtomicUsize);
    loop {
        let old_value = atomic.load(Ordering::Relaxed);
        // Check pointer match: XOR of old_value and old_object should be <= MAX_FAST_REFS
        if (old_value ^ (old_object as usize)) > MAX_FAST_REFS {
            return old_value; // pointer mismatch
        }
        let new_value = if !object.is_null() {
            (object as usize) | MAX_FAST_REFS
        } else {
            0
        };
        match atomic.compare_exchange(
            old_value,
            new_value,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => return old_value,
            Err(_) => continue,
        }
    }
}

// ============================================================================
// EX Rundown Protection (macros from ex.h → Exf* functions)
// ============================================================================

/// ExAcquireRundownProtection — alias for ExfAcquireRundownProtection.
#[inline]
pub unsafe fn ExAcquireRundownProtection(run_ref: ddk::PEX_RUNDOWN_REF) -> BOOLEAN {
    ddk::ExfAcquireRundownProtection(run_ref)
}

/// ExReleaseRundownProtection — alias for ExfReleaseRundownProtection.
#[inline]
pub unsafe fn ExReleaseRundownProtection(run_ref: ddk::PEX_RUNDOWN_REF) {
    ddk::ExfReleaseRundownProtection(run_ref)
}

/// ExWaitForRundownProtectionRelease — alias for ExfWaitForRundownProtectionRelease.
#[inline]
pub unsafe fn ExWaitForRundownProtectionRelease(run_ref: ddk::PEX_RUNDOWN_REF) {
    ddk::ExfWaitForRundownProtectionRelease(run_ref)
}

/// ExRundownCompleted — alias for ExfRundownCompleted.
#[inline]
pub unsafe fn ExRundownCompleted(run_ref: ddk::PEX_RUNDOWN_REF) {
    ddk::ExfRundownCompleted(run_ref)
}

// ============================================================================
// Fast Mutex functions (FORCEINLINE from ntoskrnl/include/internal/ex.h)
// ============================================================================

// KiAcquireFastMutex is fastcall on i386, cdecl on others
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn KiAcquireFastMutex(FastMutex: ddk::PFAST_MUTEX);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn KiAcquireFastMutex(FastMutex: ddk::PFAST_MUTEX);
}

/// _ExAcquireFastMutex — raise IRQL to APC, acquire via InterlockedDecrement.
#[inline]
pub unsafe fn _ExAcquireFastMutex(fast_mutex: ddk::PFAST_MUTEX) {
    let mut old_irql: KIRQL = 0;
    super::tier1::KeRaiseIrqlInternal(crate::base::APC_LEVEL, &mut old_irql);

    let count = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!((*fast_mutex).Count) as *mut i32,
    );
    let old = count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
    if old != 1 {
        KiAcquireFastMutex(fast_mutex);
    }

    (*fast_mutex).Owner = crate::mm::KeGetCurrentThread() as ddk::PKTHREAD;
    (*fast_mutex).OldIrql = old_irql as ULONG;
}

/// _ExReleaseFastMutex — release via InterlockedIncrement, lower IRQL.
#[inline]
pub unsafe fn _ExReleaseFastMutex(fast_mutex: ddk::PFAST_MUTEX) {
    (*fast_mutex).Owner = core::ptr::null_mut::<ddk::_KTHREAD>();
    let old_irql = (*fast_mutex).OldIrql as KIRQL;

    let count = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!((*fast_mutex).Count) as *mut i32,
    );
    let old = count.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    if old < 0 {
        ddk::KeSetEventBoostPriority(
            core::ptr::addr_of_mut!((*fast_mutex).Event),
            core::ptr::null_mut(),
        );
    }

    super::tier1::KeLowerIrqlInternal(old_irql);
}

/// _ExAcquireFastMutexUnsafe — acquire without IRQL change (caller already at APC_LEVEL).
#[inline]
pub unsafe fn _ExAcquireFastMutexUnsafe(fast_mutex: ddk::PFAST_MUTEX) {
    let thread = crate::mm::KeGetCurrentThread() as ddk::PKTHREAD;

    let count = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!((*fast_mutex).Count) as *mut i32,
    );
    let old = count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst);
    if old != 1 {
        KiAcquireFastMutex(fast_mutex);
    }

    (*fast_mutex).Owner = thread;
}

/// _ExReleaseFastMutexUnsafe — release without IRQL change.
#[inline]
pub unsafe fn _ExReleaseFastMutexUnsafe(fast_mutex: ddk::PFAST_MUTEX) {
    (*fast_mutex).Owner = core::ptr::null_mut::<ddk::_KTHREAD>();

    let count = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!((*fast_mutex).Count) as *mut i32,
    );
    let old = count.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    if old < 0 {
        ddk::KeSetEventBoostPriority(
            core::ptr::addr_of_mut!((*fast_mutex).Event),
            core::ptr::null_mut(),
        );
    }
}

/// _ExEnterCriticalRegionAndAcquireFastMutexUnsafe — enter critical region + acquire.
#[inline]
pub unsafe fn _ExEnterCriticalRegionAndAcquireFastMutexUnsafe(fast_mutex: ddk::PFAST_MUTEX) {
    super::ke::KeEnterCriticalRegion();
    _ExAcquireFastMutexUnsafe(fast_mutex);
}

/// _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion — release + leave critical region.
#[inline]
pub unsafe fn _ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(fast_mutex: ddk::PFAST_MUTEX) {
    _ExReleaseFastMutexUnsafe(fast_mutex);
    super::ke::KeLeaveCriticalRegion();
}

/// _ExTryToAcquireFastMutex — try to acquire (CAS 1→0), return TRUE on success.
#[inline]
pub unsafe fn _ExTryToAcquireFastMutex(fast_mutex: ddk::PFAST_MUTEX) -> BOOLEAN {
    let mut old_irql: KIRQL = 0;
    super::tier1::KeRaiseIrqlInternal(crate::base::APC_LEVEL, &mut old_irql);

    let count = core::sync::atomic::AtomicI32::from_ptr(
        core::ptr::addr_of_mut!((*fast_mutex).Count) as *mut i32,
    );
    if count
        .compare_exchange(1, 0, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst)
        .is_ok()
    {
        (*fast_mutex).Owner = crate::mm::KeGetCurrentThread() as ddk::PKTHREAD;
        (*fast_mutex).OldIrql = old_irql as ULONG;
        1 // TRUE
    } else {
        super::tier1::KeLowerIrqlInternal(old_irql);
        core::hint::spin_loop();
        0 // FALSE
    }
}

// ============================================================================
// ExGetRunRefForGivenProcessor (FORCEINLINE from ex.h)
// ============================================================================

/// ExGetRunRefForGivenProcessor — compute per-processor rundown ref address.
#[inline]
pub unsafe fn ExGetRunRefForGivenProcessor(
    run_ref_cache_aware: ddk::PEX_RUNDOWN_REF_CACHE_AWARE,
    proc_number: ULONG,
) -> ddk::PEX_RUNDOWN_REF {
    let base = (*run_ref_cache_aware).RunRefs as usize;
    let size = (*run_ref_cache_aware).RunRefSize as usize;
    let number = (*run_ref_cache_aware).Number as usize;
    (base + size * ((proc_number as usize) % number)) as ddk::PEX_RUNDOWN_REF
}

// ============================================================================
// ExWaitOnPushLock (FORCEINLINE from ex.h)
// ============================================================================

/// ExWaitOnPushLock — wait for a push lock to become available.
/// If locked, acquire exclusively then release.
#[inline]
pub unsafe fn ExWaitOnPushLock(push_lock: ddk::PEX_PUSH_LOCK) {
    let locked = core::ptr::read_volatile(push_lock as *const usize) & EX_PUSH_LOCK_LOCK != 0;
    if locked {
        ddk::ExfAcquirePushLockExclusive(push_lock);
        ddk::ExfReleasePushLockExclusive(push_lock);
    }
}

// ============================================================================
// ExInitializeFastMutex (FORCEINLINE from exfuncs.h)
// ============================================================================

/// ExInitializeFastMutex — initialize a fast mutex.
#[inline]
pub unsafe fn ExInitializeFastMutex(fast_mutex: ddk::PFAST_MUTEX) {
    (*fast_mutex).Count = 1; // FM_LOCK_BIT
    (*fast_mutex).Owner = core::ptr::null_mut::<ddk::_KTHREAD>();
    (*fast_mutex).Contention = 0;
    ddk::KeInitializeEvent(
        core::ptr::addr_of_mut!((*fast_mutex).Event),
        ddk::_EVENT_TYPE_SynchronizationEvent,
        0, // FALSE
    );
}

// ============================================================================
// ExInitializeRundownProtection (_ExInitializeRundownProtection from ex.h)
// ============================================================================

/// ExInitializeRundownProtection — set RunRef->Count to zero.
#[inline]
pub unsafe fn ExInitializeRundownProtection(run_ref: ddk::PEX_RUNDOWN_REF) {
    (*run_ref).__bindgen_anon_1.Count = 0;
}

// ============================================================================
// ExpChangePushlock (macro from ex.h)
// ============================================================================

/// ExpChangePushlock — InterlockedCompareExchangePointer on a push lock.
/// Returns the old value (pointer-sized).
#[inline]
pub unsafe fn ExpChangePushlock(
    push_lock: ddk::PEX_PUSH_LOCK,
    new_val: usize,
    old_val: usize,
) -> usize {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(push_lock as *const AtomicUsize);
    match atomic.compare_exchange(old_val, new_val, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(prev) => prev,
        Err(prev) => prev,
    }
}

// ============================================================================
// ExDoCallBack (FORCEINLINE from ex.h)
// ============================================================================

/// ExDoCallBack — invoke a callback if registered.
///
/// References the callback block, calls the routine, then dereferences.
#[inline]
pub unsafe fn ExDoCallBack(
    callback: *mut ddk::EX_CALLBACK,
    context: PVOID,
    argument1: PVOID,
    argument2: PVOID,
) {
    let callback_block = ddk::ExReferenceCallBackBlock(callback);
    if !callback_block.is_null() {
        let function = ddk::ExGetCallBackBlockRoutine(callback_block);
        if let Some(func) = function {
            func(context, argument1, argument2);
        }
        ddk::ExDereferenceCallBackBlock(callback, callback_block);
    }
}

// ============================================================================
// ExReleasePushLockShared (FORCEINLINE from ex.h)
// ============================================================================

/// ExReleasePushLockShared — fast path release of shared push lock.
#[inline]
pub unsafe fn ExReleasePushLockShared(push_lock: ddk::PEX_PUSH_LOCK) {
    lockdep_release(push_lock as _); // pop from held-set before we drop it
    let old_val = (EX_PUSH_LOCK_LOCK | EX_PUSH_LOCK_SHARE_INC) as usize;
    let current = ExpChangePushlock(push_lock, 0, old_val);
    if current != old_val {
        ddk::ExfReleasePushLockShared(push_lock);
    }
}

// ============================================================================
// ExInterlockedFlushSList (exfuncs.h)
// ============================================================================

/// `ExInterlockedFlushSList(SListHead)` = `ExpInterlockedFlushSList(SListHead)` off x86. On
/// i386 it is a real (fastcall) export, which the bindgen glob already declares.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn ExInterlockedFlushSList(list_head: ddk::PSLIST_HEADER) -> ddk::PSLIST_ENTRY {
    ddk::ExpInterlockedFlushSList(list_head)
}

/// `InterlockedPushEntrySList(h, e)` as a DRIVER calls it. Off x86 wdm.h maps it to the export
/// `ExpInterlockedPushEntrySList`; on i386 it is itself a fastcall export. (`interlocked::`'s
/// version links `RtlInterlockedPushEntrySList`, ntoskrnl's internal implementation symbol --
/// right inside ntoskrnl, undefined in a .sys: atapi failed to link on it.)
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn InterlockedPushEntrySList(
    list_head: ddk::PSLIST_HEADER,
    list_entry: ddk::PSLIST_ENTRY,
) -> ddk::PSLIST_ENTRY {
    ddk::ExpInterlockedPushEntrySList(list_head, list_entry)
}
/// i386: the fastcall export (`@InterlockedPushEntrySList@8`).
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn InterlockedPushEntrySList(
    list_head: ddk::PSLIST_HEADER,
    list_entry: ddk::PSLIST_ENTRY,
) -> ddk::PSLIST_ENTRY {
    extern "fastcall" {
        #[link_name = "InterlockedPushEntrySList"]
        fn imp(list_head: ddk::PSLIST_HEADER, list_entry: ddk::PSLIST_ENTRY) -> ddk::PSLIST_ENTRY;
    }
    imp(list_head, list_entry)
}
