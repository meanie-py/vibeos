//! Rust impls of `ke_x.h` `#define` MACROS whose C trampolines live in
//! `ntoskrnl/ke/ke_rust_shims.c`. Phase 2 sibling rlib (separate from
//! `ke-inlines`, which holds the FORCEINLINE bodies — Phase 1).
//!
//! Pilot batch (4 fns): KiSatisfyMutantWait, KiSatisfyNonMutantWait,
//! KiSatisfyObjectWait, KiAddThreadToWaitList. The C trampoline previously
//! materialised each macro inside a `static __inline _tramp_*` to capture the
//! expansion, then the wrapper called the inline; bodies now collapse to one
//! `_rs_*` delegation in the same shape as the Phase 1 augment pattern.
//!
//! Wait-setup batch (4 fns): KxSingleThreadWait, KxDelayThreadWait,
//! KxMultiThreadWait, KxQueueThreadWait. These are the per-thread wait-block
//! initialisers that `KeWaitForSingleObject` / `KeDelayExecutionThread` /
//! `KeWaitForMultipleObjects` / `KeWaitForGate` (queues) call before
//! suspending. The C macros in `ke_x.h` reference caller-locals (Thread,
//! WaitBlock[], Timer, TimerBlock); the C trampoline parameterised over
//! Object + Timeout + WaitMode + … and derived the locals from
//! `KeGetCurrentThread()`. Same shape preserved here. The 2026-05-05
//! amd64 taskbar wait-block-corruption fix (use `InitializeListHead` for
//! KxMultiThreadWait, defensive `Flink/Blink → &TimerBlock.WaitListEntry +
//! TimerBlock.WaitListEntry → &Timer.WaitListHead` pre-set for the other
//! three) is preserved exactly.
//!
//! Architecturally identical to `ke-inlines` (sibling rlib, isolated rustc
//! invocation cuts the cross-CGU captures(none) attribute deduction wire).
//! Replicates the same `KeGetCurrentPrcb` inline asm locally to dodge the
//! cross-crate `#[inline(always)]`-over-`asm!` non-inline gotcha at
//! `opt-level = 1` (see `feedback_cross_crate_inline_asm.md`).

#![no_std]

use ntkrnl_sys::base::{InitializeListHead, InsertHeadList, InsertTailList, BOOLEAN};
use ntkrnl_sys::ddk::{
    PKMUTANT, PKPRCB, PKTHREAD, PKTIMER, PKWAIT_BLOCK, PLARGE_INTEGER, _DISPATCHER_HEADER,
    _KMUTANT, _KPCR, _KSYSTEM_TIME, _KTHREAD, _KWAIT_BLOCK, _LARGE_INTEGER, _LIST_ENTRY,
};

// `_KOBJECTS` enum values from bindgen (same on all 3 arches; replicated to
// avoid pulling in the full `_KOBJECTS` type alias).
const MUTANT_OBJECT: u8 = 2;
const EVENT_SYNCHRONIZATION_OBJECT: u8 = 1;
const SEMAPHORE_OBJECT: u8 = 5;

/// `TIMER_OR_EVENT_TYPE` from `ntoskrnl/include/internal/ke.h`.
const TIMER_OR_EVENT_TYPE: u8 = 0x7;

/// `STATUS_ABANDONED` (`ntstatus.h`) — set on `WaitStatus` when a mutant is
/// claimed after its previous owner abandoned it. KTHREAD::WaitStatus is
/// `LONG_PTR` (i32 on i386, i64 on amd64/arm64); use `as _` at the
/// assignment site so the literal coerces to the right width.
const STATUS_ABANDONED: i32 = 0x00000080;

// ============================================================================
// KeGetCurrentPrcb / KeGetCurrentThread — replicated locally to dodge the
// cross-crate `#[inline]` over `core::arch::asm!` non-inline at opt-level=1.
// Same pattern as `ke-inlines/src/lib.rs::ke_get_pcr` etc.
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

// ============================================================================
// Field accessors — DISPATCHER_HEADER.Type lives inside a deeply-nested
// bindgen union (`__bindgen_anon_1.__bindgen_anon_1.Type`); KTHREAD's
// KernelApcDisable lives inside the per-arch `__bindgen_anon_{4,5}` ApcDisable
// union. Hide both behind small inline accessors so the call sites read like
// the C macro source.
// ============================================================================

#[inline(always)]
unsafe fn header_type(h: *const _DISPATCHER_HEADER) -> u8 {
    (*h).__bindgen_anon_1.__bindgen_anon_1.Type
}

#[inline(always)]
unsafe fn kt_kernel_apc_disable_ptr(t: PKTHREAD) -> *mut i16 {
    #[cfg(target_arch = "x86")]
    {
        core::ptr::addr_of_mut!((*t).__bindgen_anon_4.__bindgen_anon_1.KernelApcDisable)
    }
    #[cfg(not(target_arch = "x86"))]
    {
        core::ptr::addr_of_mut!(
            (*t).__bindgen_anon_5
                .__bindgen_anon_8
                .__bindgen_anon_1
                .__bindgen_anon_1
                .KernelApcDisable
        )
    }
}

// ============================================================================
// `KiSatisfyMutantWait(Object, Thread)` — `ke_x.h:715`
// ============================================================================

/// Satisfies a wait on a `KMUTANT` dispatcher object. Decrements signal state;
/// if it hit zero, takes ownership for `Thread`, applies the APC-disable
/// adjustment, clears any abandoned flag (and propagates `STATUS_ABANDONED`
/// to the thread's wait status), and chains the mutant onto the thread's
/// owned-mutant list.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiSatisfyMutantWait(object: PKMUTANT, thread: PKTHREAD) {
    // (Object)->Header.SignalState--
    let header = core::ptr::addr_of_mut!((*object).Header);
    let new_state = (*header).SignalState - 1;
    (*header).SignalState = new_state;

    // if (!(Object)->Header.SignalState) { ... }
    if new_state == 0 {
        // (Object)->OwnerThread = Thread
        (*object).OwnerThread = thread as *mut _KTHREAD;

        // Thread->KernelApcDisable -= (Object)->ApcDisable
        let kapc = kt_kernel_apc_disable_ptr(thread);
        let apc_disable = (*object).ApcDisable as i16;
        *kapc = (*kapc).wrapping_sub(apc_disable);

        // if ((Object)->Abandoned) { Abandoned = FALSE; WaitStatus = STATUS_ABANDONED; }
        if (*object).Abandoned != 0 {
            (*object).Abandoned = 0;
            (*thread).WaitStatus = STATUS_ABANDONED as _;
        }

        // InsertHeadList(Thread->MutantListHead.Blink, &(Object)->MutantListEntry)
        // — the C macro uses InsertHeadList against the *Blink* of the head,
        // which is equivalent to inserting at the tail. Match the C verbatim.
        let mutant_list_head = core::ptr::addr_of_mut!((*thread).MutantListHead);
        let insert_after = (*mutant_list_head).Blink;
        let entry = core::ptr::addr_of_mut!((*object).MutantListEntry);
        InsertHeadList(insert_after, entry);
    }
}

// ============================================================================
// `KiSatisfyNonMutantWait(Object)` — `ke_x.h:749`
// ============================================================================

/// Satisfies a wait on any non-mutant dispatcher object. Caller passes a
/// `*mut _KMUTANT` because the macro is parameter-polymorphic in C — this
/// only ever touches `Header`, which is the first field of every dispatcher
/// object, so the cast is safe.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiSatisfyNonMutantWait(object: PKMUTANT) {
    let header = core::ptr::addr_of_mut!((*object).Header);
    let typ = header_type(header);

    if (typ & TIMER_OR_EVENT_TYPE) == EVENT_SYNCHRONIZATION_OBJECT {
        // Synchronization Timers and Events just get un-signaled
        (*header).SignalState = 0;
    } else if typ == SEMAPHORE_OBJECT {
        // Semaphores can have multiple states, so we only decrease it
        (*header).SignalState -= 1;
    }
}

// ============================================================================
// `KiSatisfyObjectWait(Object, Thread)` — `ke_x.h:767`
// ============================================================================

/// Dispatcher entry-point: routes mutants through `KiSatisfyMutantWait`, all
/// other dispatcher objects through `KiSatisfyNonMutantWait`. Thread parameter
/// is only consumed by the mutant path.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiSatisfyObjectWait(object: PKMUTANT, thread: PKTHREAD) {
    let header = core::ptr::addr_of_mut!((*object).Header);
    if header_type(header) == MUTANT_OBJECT {
        _rs_KiSatisfyMutantWait(object, thread);
    } else {
        _rs_KiSatisfyNonMutantWait(object);
    }
}

// ============================================================================
// `KiAddThreadToWaitList(Thread, Swappable)` — `ke_x.h:824`
// ============================================================================

/// Conditionally inserts the thread into the current PRCB's wait list (only
/// when `Swappable` is non-zero — i.e. `KiCheckThreadStackSwap` previously
/// returned TRUE).
#[no_mangle]
pub unsafe extern "C" fn _rs_KiAddThreadToWaitList(thread: PKTHREAD, swappable: BOOLEAN) {
    if swappable != 0 {
        let prcb = ke_get_current_prcb();
        let head = core::ptr::addr_of_mut!((*prcb).WaitListHead);
        // KTHREAD::WaitListEntry lives in the `__bindgen_anon_3`
        // union (overlaps with SwapListEntry) on all 3 archs.
        let entry = core::ptr::addr_of_mut!((*thread).__bindgen_anon_3.WaitListEntry);
        InsertTailList(head as *mut _LIST_ENTRY, entry as *mut _LIST_ENTRY);
    }
}

// ============================================================================
// Wait-setup batch — `Kx{Single,Delay,Multi,Queue}ThreadWait` (`ke_x.h:1076`+)
// ============================================================================

/// `TIMER_WAIT_BLOCK` — index into `Thread->WaitBlock[]` for the timeout
/// wait block. Defined in `ntoskrnl/include/internal/ke.h:169`.
const TIMER_WAIT_BLOCK: usize = 3;

/// `WaitAny` — `_WAIT_TYPE` enum value (1). Same on all 3 archs per bindgen.
const WAIT_ANY_TYPE: u8 = 1;

/// `_KWAIT_REASON_DelayExecution` (= 4) and `_KWAIT_REASON_WrQueue` (= 15).
const DELAY_EXECUTION: u8 = 4;
const WR_QUEUE: u8 = 15;

/// `KPROCESSOR_MODE` value `UserMode` (= 1). Used by `KxDelayThreadWait` to
/// derive the `Alertable` flag.
const USER_MODE: i8 = 1;

extern "C" {
    /// `KeTickCount` — system tick count as `KSYSTEM_TIME`. Defined in
    /// `ntoskrnl-rust::ke::clock`. We only read the LowPart for `WaitTime`.
    static KeTickCount: _KSYSTEM_TIME;

    /// Phase 1 K7 timer-helpers exported by `ke-inlines`. Setup the per-
    /// thread KTIMER for the wait timeout; on return the caller reads
    /// `Timer->DueTime.QuadPart` into the local `DueTime` parameter.
    /// `interval` MUST be `_LARGE_INTEGER` (an 8-byte aggregate union) — NOT
    /// `i64`. On i686-pc-windows-gnu cdecl the two are stack-equivalent at
    /// the byte level, but rustc's ABI lowering can treat aggregate-union
    /// args differently from scalar i64 args under captures(none) inference;
    /// a stale i64 declaration here was the i386-regression root cause for
    /// Phase 2 batch 2. Match `ke-inlines::_rs_KxSetTimerForThreadWait` exactly.
    fn _rs_KxSetTimerForThreadWait(timer: PKTIMER, interval: _LARGE_INTEGER, hand: *mut u32);
    /// Phase 1 K11 stack-swap eligibility check exported by `ke-inlines`.
    fn _rs_KiCheckThreadStackSwap(thread: PKTHREAD, wait_mode: i8) -> BOOLEAN;
}

// ----------------------------------------------------------------------------
// Per-arch KTHREAD field accessors — match the `ke-inlines` / `inline::ke`
// path conventions verbatim. Replicated locally to dodge the cross-crate
// `#[inline]` non-inline gotcha.
// ----------------------------------------------------------------------------

/// `Thread->WaitBlock[index]` — `__bindgen_anon_5.WaitBlock[index]` on
/// amd64/arm64, `__bindgen_anon_7.WaitBlock[index]` on i386. Mirrors
/// `inline::ke::kt_wait_block`.
#[inline(always)]
unsafe fn kt_wait_block(thread: PKTHREAD, index: usize) -> PKWAIT_BLOCK {
    #[cfg(target_arch = "x86")]
    {
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_7.WaitBlock[index])
    }
    #[cfg(not(target_arch = "x86"))]
    {
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_5.WaitBlock[index])
    }
}

/// `Thread->Timer` — `__bindgen_anon_5.Timer` on i386, `__bindgen_anon_4.Timer`
/// on amd64/arm64.
#[inline(always)]
unsafe fn kt_timer(thread: PKTHREAD) -> PKTIMER {
    #[cfg(target_arch = "x86")]
    {
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_5.Timer)
    }
    #[cfg(not(target_arch = "x86"))]
    {
        core::ptr::addr_of_mut!((*thread).__bindgen_anon_4.Timer)
    }
}

/// `Thread->WaitBlockList = …`. Lives in `__bindgen_anon_2.WaitBlockList`
/// on all 3 arches.
#[inline(always)]
unsafe fn kt_set_wait_block_list(thread: PKTHREAD, val: PKWAIT_BLOCK) {
    (*thread).__bindgen_anon_2.WaitBlockList = val;
}

/// `Thread->WaitMode = …`. Direct field on i386 (typed as
/// `KPROCESSOR_MODE = CCHAR = c_char` — signed on i686-pc-windows-gnu),
/// nested in the WaitBlock-overlay union on amd64/arm64
/// (`__bindgen_anon_5.__bindgen_anon_6.WaitMode`, also `CHAR = i8`).
#[inline(always)]
unsafe fn kt_set_wait_mode(thread: PKTHREAD, val: i8) {
    #[cfg(target_arch = "x86")]
    {
        (*thread).WaitMode = val;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*thread).__bindgen_anon_5.__bindgen_anon_6.WaitMode = val;
    }
}

/// `Thread->WaitTime = …`. Direct field on i386, nested in
/// `__bindgen_anon_5.__bindgen_anon_7.WaitTime` on amd64/arm64.
#[inline(always)]
unsafe fn kt_set_wait_time(thread: PKTHREAD, val: u32) {
    #[cfg(target_arch = "x86")]
    {
        (*thread).WaitTime = val;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*thread).__bindgen_anon_5.__bindgen_anon_7.WaitTime = val;
    }
}

/// `Thread->WaitListEntry.Flink = NULL`. Cleared by every Kx*ThreadWait —
/// the thread is not yet on a PRCB wait list (KiAddThreadToWaitList may
/// later insert it if Swappable=TRUE).
#[inline(always)]
unsafe fn kt_clear_wait_list_entry_flink(thread: PKTHREAD) {
    (*thread).__bindgen_anon_3.WaitListEntry.Flink = core::ptr::null_mut();
}

// ----------------------------------------------------------------------------
// Wait-block / timer-block linkage helpers
// ----------------------------------------------------------------------------

/// "Defensive full-init" of the timer-block ↔ timer-wait-list linkage used by
/// KxSingle/Delay/Queue paths. Wrong shape for KxMulti — see comment in
/// `_rs_KxMultiThreadWait`.
#[inline(always)]
unsafe fn link_timer_block_full(timer: PKTIMER, timer_block: PKWAIT_BLOCK) {
    let head = core::ptr::addr_of_mut!((*timer).Header.WaitListHead);
    let entry = core::ptr::addr_of_mut!((*timer_block).WaitListEntry);
    (*entry).Flink = head;
    (*entry).Blink = head;
    (*head).Flink = entry;
    (*head).Blink = entry;
}

// ----------------------------------------------------------------------------
// `KxSingleThreadWait` — `ke_x.h:1173`
// ----------------------------------------------------------------------------

/// Per-thread wait-block setup for a single-object wait (no-op timer chain
/// when `timeout == NULL`). Caller passes `Object` (the dispatcher object
/// being waited on), `Timeout` (LARGE_INTEGER pointer or NULL), `wait_mode`,
/// `wait_reason`, `alertable`, plus `swappable`/`due_time`/`hand` out-pointers.
#[no_mangle]
pub unsafe extern "C" fn _rs_KxSingleThreadWait(
    object: *mut core::ffi::c_void,
    timeout: PLARGE_INTEGER,
    wait_mode: i8,
    wait_reason: u8,
    alertable: BOOLEAN,
    swappable: *mut BOOLEAN,
    due_time: PLARGE_INTEGER,
    hand: *mut u32,
) {
    let thread = ke_get_current_thread();
    let wait_block = kt_wait_block(thread, 0);
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, wait_block);
    (*wait_block).WaitKey = 0; // STATUS_SUCCESS
    (*wait_block).Object = object;
    (*wait_block).WaitType = WAIT_ANY_TYPE;
    (*thread).WaitStatus = 0;

    if !timeout.is_null() {
        _rs_KxSetTimerForThreadWait(timer, *timeout, hand);
        (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
        (*wait_block).NextWaitBlock = timer_block;
        (*timer_block).NextWaitBlock = wait_block;
        // Defensive full-init — see KxMultiThreadWait comment for why this
        // shape is correct here (single-object path doesn't double-insert
        // through KeWaitForSingleObject's loop; the linkage is final).
        link_timer_block_full(timer, timer_block);
    } else {
        (*wait_block).NextWaitBlock = wait_block;
    }

    (*thread).Alertable = alertable;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = wait_reason;
    kt_clear_wait_list_entry_flink(thread);
    *swappable = _rs_KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.LowPart)));
}

// ----------------------------------------------------------------------------
// `KxDelayThreadWait` — `ke_x.h:1076`
// ----------------------------------------------------------------------------

/// Setup for a pure timeout wait (`KeDelayExecutionThread`). No object
/// wait-block — only the TimerBlock chains to itself, so the wait list is
/// `[TimerBlock]`.
#[no_mangle]
pub unsafe extern "C" fn _rs_KxDelayThreadWait(
    interval: PLARGE_INTEGER,
    wait_mode: i8,
    alertable: BOOLEAN,
    swappable: *mut BOOLEAN,
    due_time: PLARGE_INTEGER,
    hand: *mut u32,
) {
    let thread = ke_get_current_thread();
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, timer_block);
    (*timer_block).NextWaitBlock = timer_block;

    _rs_KxSetTimerForThreadWait(timer, *interval, hand);
    (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
    // Defensive full-init — KxDelay doesn't go through KeWaitForMultipleObjects's
    // InsertTailList loop, so pre-setting Timer.WaitListHead is correct.
    link_timer_block_full(timer, timer_block);

    // ke_x.h's macro takes Alertable from the caller — deriving it from
    // wait_mode (the old mistranslation) breaks non-alertable user Sleeps
    // and alertable kernel delays. See inline/ke.rs KxDelayThreadWait.
    (*thread).Alertable = alertable;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = DELAY_EXECUTION;
    kt_clear_wait_list_entry_flink(thread);
    *swappable = _rs_KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.LowPart)));
}

// ----------------------------------------------------------------------------
// `KxMultiThreadWait` — `ke_x.h:1117` (with the 2026-05-05 wait-block fix)
// ----------------------------------------------------------------------------

/// Multi-object wait setup. Walks the `Objects[]` array and chains each
/// caller-provided wait block into a circular list, then optionally appends
/// the timer block. CRITICAL: when `timeout != NULL` we must call
/// `InitializeListHead(&Timer.WaitListHead)` so the subsequent
/// `KeWaitForMultipleObjects` `InsertTailList(Head, TimerBlock)` cleanly
/// inserts. Pre-setting the linkage manually here corrupts the wait block
/// into a self-reference and produces the `KiTimerExpiration → KxUnwaitThread
/// → KiUnwaitThread` storm that gated the amd64 taskbar pre-2026-05-05.
#[no_mangle]
pub unsafe extern "C" fn _rs_KxMultiThreadWait(
    wait_block_array: PKWAIT_BLOCK,
    objects: *mut *mut core::ffi::c_void,
    count: u32,
    timeout: PLARGE_INTEGER,
    wait_type: u8,
    wait_mode: i8,
    wait_reason: u8,
    alertable: BOOLEAN,
    swappable: *mut BOOLEAN,
    due_time: PLARGE_INTEGER,
    hand: *mut u32,
) {
    let thread = ke_get_current_thread();
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, wait_block_array);

    let count_us = count as usize;
    for i in 0..count_us {
        let wb = wait_block_array.add(i);
        (*wb).Object = *objects.add(i);
        (*wb).WaitKey = i as u16;
        (*wb).WaitType = wait_type;
        (*wb).Thread = thread;
        (*wb).NextWaitBlock = wait_block_array.add(i + 1);
    }
    (*thread).WaitStatus = 0;

    if !timeout.is_null() {
        _rs_KxSetTimerForThreadWait(timer, *timeout, hand);
        (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
        let last = wait_block_array.add(count_us - 1);
        (*last).NextWaitBlock = timer_block;
        (*timer_block).NextWaitBlock = wait_block_array;
        // CRITICAL: KeWaitForMultipleObjects's loop will InsertTailList
        // every wait block in the chain (now including TimerBlock) into
        // its object's WaitListHead — see ntoskrnl/ke/wait.c. For TimerBlock
        // that means inserting into Timer.WaitListHead. We must hand it an
        // EMPTY list head; pre-setting Flink/Blink to &TimerBlock corrupts
        // the wait block into a self-referencing entry. See translation
        // audit headline (2026-05-05) for the full storm-trace.
        InitializeListHead(core::ptr::addr_of_mut!((*timer).Header.WaitListHead));
    } else {
        let last = wait_block_array.add(count_us - 1);
        (*last).NextWaitBlock = wait_block_array;
    }

    (*thread).Alertable = alertable;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = wait_reason;
    kt_clear_wait_list_entry_flink(thread);
    *swappable = _rs_KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.LowPart)));
}

// ----------------------------------------------------------------------------
// `KxQueueThreadWait` — `ke_x.h:1221`
// ----------------------------------------------------------------------------

/// Queue-object wait (`KeRemoveQueue`). Same shape as KxSingle but with
/// `WaitBlock[0].Thread = Thread` set explicitly (queues use this back-
/// reference for entry routing) and `Alertable=FALSE`/`WaitReason=WrQueue`
/// hardcoded.
#[no_mangle]
pub unsafe extern "C" fn _rs_KxQueueThreadWait(
    queue: *mut core::ffi::c_void,
    timeout: PLARGE_INTEGER,
    wait_mode: i8,
    swappable: *mut BOOLEAN,
    due_time: PLARGE_INTEGER,
    hand: *mut u32,
) {
    let thread = ke_get_current_thread();
    let wait_block = kt_wait_block(thread, 0);
    let timer = kt_timer(thread);
    let timer_block = kt_wait_block(thread, TIMER_WAIT_BLOCK);

    kt_set_wait_block_list(thread, wait_block);
    (*wait_block).WaitKey = 0;
    (*wait_block).Object = queue;
    (*wait_block).WaitType = WAIT_ANY_TYPE;
    (*wait_block).Thread = thread;
    (*thread).WaitStatus = 0;

    if !timeout.is_null() {
        _rs_KxSetTimerForThreadWait(timer, *timeout, hand);
        (*due_time).QuadPart = (*timer).DueTime.QuadPart as i64;
        (*wait_block).NextWaitBlock = timer_block;
        (*timer_block).NextWaitBlock = wait_block;
        link_timer_block_full(timer, timer_block);
    } else {
        (*wait_block).NextWaitBlock = wait_block;
    }

    (*thread).Alertable = 0;
    kt_set_wait_mode(thread, wait_mode);
    (*thread).WaitReason = WR_QUEUE;
    kt_clear_wait_list_entry_flink(thread);
    *swappable = _rs_KiCheckThreadStackSwap(thread, wait_mode);
    kt_set_wait_time(thread, core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.LowPart)));
}

// ============================================================================
// Trap-frame & arch-intrinsic batch — `YieldProcessor`, `KeGetTrapFramePc`,
// `KiRundownThread`, `KiGetLinkedTrapFrame`, `KiGetUserModeStackAddress`.
// All five are per-arch `cfg` bodies (originally `#define` / FORCEINLINE in
// arch-specific headers, materialised in `ke_rust_shims.c`). Closes out the
// macro/intrinsic surface of `ke_rust_shims.c`.
// ============================================================================

use ntkrnl_sys::ddk::{PKTRAP_FRAME, ULONG_PTR, _KTRAP_FRAME};

/// `YieldProcessor()` — compiler intrinsic. `pause` on x86, `yield` on ARM64.
/// On i386 + amd64 we want `pause` directly; on ARM64 use the WFE-derived
/// `yield`. No SSE/SSE2 dependency despite `_mm_pause` historically being
/// `SSE2`-named — the encoding (rep nop / `f3 90`) decodes as a regular nop
/// on pre-SSE2 chips.
#[no_mangle]
pub unsafe extern "C" fn _rs_YieldProcessor() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!("pause", options(nostack, nomem, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!("yield", options(nostack, nomem, preserves_flags));
    }
}

/// `KeGetTrapFramePc(TrapFrame)` — returns the instruction pointer captured
/// in the trap frame. Direct field on all three arches (`Eip`/`Rip`/`Pc`).
/// NTAPI on i386 (stdcall — `_KeGetTrapFramePc@4`); cdecl elsewhere. The
/// rustc 1.92 stdcall ABI bug only affects call-sites declaring extern
/// stdcall, not function definitions, so promoting from `_rs_X` cdecl-via-
/// trampoline to the bare stdcall export is safe (re-probed 2026-05-08).
#[no_mangle]
pub unsafe extern "system" fn KeGetTrapFramePc(trap_frame: PKTRAP_FRAME) -> ULONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        (*trap_frame).Eip as ULONG_PTR
    }
    #[cfg(target_arch = "x86_64")]
    {
        (*trap_frame).Rip as ULONG_PTR
    }
    #[cfg(target_arch = "aarch64")]
    {
        (*trap_frame).Pc as ULONG_PTR
    }
}

/// `KiRundownThread(Thread)` — clears any per-PRCB state pinned to the
/// terminating thread. On i386: clears `Prcb->NpxThread` if it points at us
/// (otherwise the FPU context restore would race against the freed KTHREAD).
/// On amd64/arm64: no-op (FPU context lives elsewhere).
#[no_mangle]
pub unsafe extern "C" fn _rs_KiRundownThread(thread: PKTHREAD) {
    #[cfg(target_arch = "x86")]
    {
        let prcb = ke_get_current_prcb();
        if (*prcb).NpxThread == thread as *mut _KTHREAD {
            (*prcb).NpxThread = core::ptr::null_mut();
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = thread;
    }
}

/// `KiGetLinkedTrapFrame(TrapFrame)` — returns the previous (caller's) trap
/// frame. On amd64: direct `TrapFrame: UINT64` field cast to PKTRAP_FRAME.
/// On arm64: `__bindgen_anon_2.__bindgen_anon_1.TrapFrame`. On i386: there
/// is no real "linked trap frame" pointer — the macro reuses the saved Edx
/// register slot, where the prior dispatcher stashed the previous frame.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiGetLinkedTrapFrame(trap_frame: PKTRAP_FRAME) -> PKTRAP_FRAME {
    #[cfg(target_arch = "x86_64")]
    {
        (*trap_frame).TrapFrame as PKTRAP_FRAME
    }
    #[cfg(target_arch = "aarch64")]
    {
        (*trap_frame).__bindgen_anon_2.__bindgen_anon_1.TrapFrame as PKTRAP_FRAME
    }
    #[cfg(target_arch = "x86")]
    {
        (*trap_frame).Edx as PKTRAP_FRAME
    }
}

/// `KiGetUserModeStackAddress()` — returns a pointer (mutable u32/u64) to the
/// user-mode stack pointer field inside the current thread's trap frame.
/// `Rsp`/`Sp`/`HardwareEsp` per arch. Used by `KiUserModeCallout` and by
/// some PSEH unwind paths to patch the user-mode RSP before delivering a
/// software exception.
#[no_mangle]
pub unsafe extern "C" fn _rs_KiGetUserModeStackAddress() -> *mut ULONG_PTR {
    let thread = ke_get_current_thread();
    let trap_frame = (*thread).TrapFrame;
    #[cfg(target_arch = "x86")]
    {
        core::ptr::addr_of_mut!((*trap_frame).HardwareEsp) as *mut ULONG_PTR
    }
    #[cfg(target_arch = "x86_64")]
    {
        core::ptr::addr_of_mut!((*trap_frame).Rsp) as *mut ULONG_PTR
    }
    #[cfg(target_arch = "aarch64")]
    {
        core::ptr::addr_of_mut!((*trap_frame).Sp) as *mut ULONG_PTR
    }
}

// ============================================================================
// HAL dispatch batch — 3 macros that expand to `HALDISPATCH->Foo(...)` /
// `HALPRIVATEDISPATCH->Foo(...)`. The C trampolines wrap NTAPI (stdcall on
// i386) functions that profobj.rs / I/O paths call via `extern "system"`
// decls. Bodies fetch the Option<fn> slot from the global dispatch table
// and call it; if NULL (HAL not yet initialised), return STATUS_NOT_IMPLEMENTED
// — same semantics as the underlying macro on a freshly-booted system.
// ============================================================================

use ntkrnl_sys::base::{NTSTATUS, PVOID, ULONG};
use ntkrnl_sys::ddk::{
    HAL_DISPATCH, HAL_QUERY_INFORMATION_CLASS, HAL_SET_INFORMATION_CLASS,
    PULONG, UCHAR, _HAL_PRIVATE_DISPATCH,
};

/// `STATUS_NOT_IMPLEMENTED` — dispatch slot was NULL (HAL hasn't published
/// the table entry yet). Same fallback the C body would have hit by
/// dereferencing a null function pointer (with a guaranteed crash).
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as NTSTATUS;

extern "C" {
    static HalDispatchTable: HAL_DISPATCH;
    static HalPrivateDispatchTable: _HAL_PRIVATE_DISPATCH;
}

// HAL dispatch bridges — promoted from `_rs_*` cdecl-via-C-trampoline to
// bare-name `extern "system"` exports (2026-05-08). NTAPI = stdcall on
// i386, cdecl on amd64/arm64. The function-pointer dispatch into
// HalDispatchTable / HalPrivateDispatchTable inherits its ABI from the
// pointer's type, so the rustc 1.92 stdcall ABI bug doesn't apply here.

/// `HalQuerySystemInformation(InformationClass, BufferSize, Buffer, ReturnedLength)`
/// — `HALDISPATCH->HalQuerySystemInformation(...)`.
#[no_mangle]
pub unsafe extern "system" fn HalQuerySystemInformation(
    information_class: HAL_QUERY_INFORMATION_CLASS,
    buffer_size: ULONG,
    buffer: PVOID,
    returned_length: PULONG,
) -> NTSTATUS {
    if let Some(f) = HalDispatchTable.HalQuerySystemInformation {
        f(information_class, buffer_size, buffer, returned_length)
    } else {
        STATUS_NOT_IMPLEMENTED
    }
}

/// `HalSetSystemInformation(InformationClass, BufferSize, Buffer)`
/// — `HALDISPATCH->HalSetSystemInformation(...)`.
#[no_mangle]
pub unsafe extern "system" fn HalSetSystemInformation(
    information_class: HAL_SET_INFORMATION_CLASS,
    buffer_size: ULONG,
    buffer: PVOID,
) -> NTSTATUS {
    if let Some(f) = HalDispatchTable.HalSetSystemInformation {
        f(information_class, buffer_size, buffer)
    } else {
        STATUS_NOT_IMPLEMENTED
    }
}

/// `HalVectorToIDTEntry(Vector)` — `HALPRIVATEDISPATCH->HalVectorToIDTEntry(...)`.
/// Returns the IDT slot index for an interrupt vector; falls back to the
/// vector itself (lower 8 bits) which is the historical i386 mapping the
/// HAL would have applied if the dispatch table were missing.
#[no_mangle]
pub unsafe extern "system" fn HalVectorToIDTEntry(vector: ULONG) -> UCHAR {
    if let Some(f) = HalPrivateDispatchTable.HalVectorToIDTEntry {
        f(vector)
    } else {
        (vector & 0xff) as UCHAR
    }
}

// `_KMUTANT` and `_KWAIT_BLOCK` are referenced via the C trampolines only as
// `*mut` opaque handles — silence the unused-import lint when the type
// alias didn't end up referenced because of arch-cfg paths.
#[allow(dead_code)]
const _ASSERT_KMUTANT: usize = core::mem::size_of::<_KMUTANT>();
#[allow(dead_code)]
const _ASSERT_KWAIT_BLOCK: usize = core::mem::size_of::<_KWAIT_BLOCK>();
#[allow(dead_code)]
const _ASSERT_KTRAP_FRAME: usize = core::mem::size_of::<_KTRAP_FRAME>();

// ============================================================================
// 2026-05-07: more ke_rust_shims.c translations (i386 + amd64).
// Covers Ke386Get{Global,Local}DescriptorTable, KiEndInterrupt,
// KeGdiFlushUserBatch_call, ARM64 KdbpPrint stub.
// ============================================================================

use ntkrnl_sys::base::KIRQL;

/// `Ke386GetGlobalDescriptorTable(Limit)` — macro from intrin_i.h that
/// expands to `__sgdt(Limit)`. The `_rs_*` prefix is preserved because
/// of a dual-decoration constraint on i386: this fn has both
/// `extern "C"` cdecl callers (kdbg/kdb_cli.rs — uses the bare-name
/// `extern "C"` impl below at ~line 863) AND `extern "system"` stdcall
/// callers (ke/i386/cpu.rs — uses the `_X@4` symbol that the C
/// trampoline in `ke_rust_shims.c` supplies by delegating to this body).
/// Rust forbids two source-level definitions of the same name, so the
/// dual-decoration is split across crates.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn _rs_Ke386GetGlobalDescriptorTable(limit: PVOID) {
    core::arch::asm!("sgdt [{}]", in(reg) limit, options(nostack));
}

/// `Ke386GetLocalDescriptorTable(Selector)` — macro expanding to
/// `__sldt(Selector)`. Same dual-decoration story as
/// `_rs_Ke386GetGlobalDescriptorTable` above.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn _rs_Ke386GetLocalDescriptorTable(selector: *mut u16) {
    core::arch::asm!("sldt [{}]", in(reg) selector, options(nostack));
}

/// `KeGdiFlushUserBatch_call()` — dispatches through the global function-pointer
/// `KeGdiFlushUserBatch` (PGDI_BATCHFLUSH_ROUTINE). On a freshly-booted system
/// the slot is NULL; only after win32k loads does it get populated. ARM64
/// doesn't have win32k GDI batching.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn _rs_KeGdiFlushUserBatch_call() {
    use ntkrnl_sys::ddk::PGDI_BATCHFLUSH_ROUTINE;
    extern "C" {
        static KeGdiFlushUserBatch: PGDI_BATCHFLUSH_ROUTINE;
    }
    if let Some(f) = KeGdiFlushUserBatch {
        f();
    }
}

// ARM64 KdbpPrint: stays in C — Rust can't define C-variadic exports on
// stable, even when the body is empty.

// ============================================================================
// ARM64 boot-stack trampoline + interrupt enable/disable stubs.
// Translated from ntoskrnl/ke/arm64/{boot,stubs_asm}.S to Rust global_asm!
// 2026-05-07. Same instructions, hosted in this crate.
// ============================================================================

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(r#"
    .text

    // VOID NTAPI KiSwitchToBootStack(ULONG_PTR InitialStack)
    // Switches to the kernel boot stack and tail-calls KiSystemStartupBootStack.
    // x0 = new stack pointer (top of stack, 16-byte aligned).
    .globl KiSwitchToBootStack
    KiSwitchToBootStack:
        mov sp, x0
        b KiSystemStartupBootStack

    // void _enable(void) — unmask DAIF (interrupt enable on ARM64).
    .globl _enable
    _enable:
        msr daifclr, #0xF
        ret

    // void _disable(void) — mask DAIF (interrupt disable on ARM64).
    .globl _disable
    _disable:
        msr daifset, #0xF
        ret
"#);

// ============================================================================
// kdbg cdecl macro/inline wrappers (was kdbg_rust_shims.c batch)
//
// `Ke386Get{Global,Local}DescriptorTable` cdecl variant — distinct
// from the stdcall `_X@4` variant in `ke_rust_shims.c`. `Ke386GetTr`
// (i386 `str` asm). `KeGet/SetContextFrameRegister` and the trap-frame
// register helpers — plain CONTEXT/KTRAP_FRAME field accessors that
// kdb_cli.rs declares as `extern "C"`.
// ============================================================================

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn Ke386GetGlobalDescriptorTable(limit: *mut u16) {
    core::arch::asm!("sgdt [{}]", in(reg) limit, options(nostack));
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn Ke386GetLocalDescriptorTable(selector: *mut u16) {
    core::arch::asm!("sldt [{}]", in(reg) selector, options(nostack));
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[no_mangle]
pub unsafe extern "C" fn Ke386GetTr() -> u16 {
    let tr: u16;
    core::arch::asm!("str {0:x}", out(reg) tr, options(nostack, nomem, preserves_flags));
    tr
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KeGetContextFrameRegister(
    context: *mut ntkrnl_sys::ddk::_CONTEXT,
) -> ntkrnl_sys::base::ULONG_PTR {
    (*context).Ebp as ntkrnl_sys::base::ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KeSetContextFrameRegister(
    context: *mut ntkrnl_sys::ddk::_CONTEXT,
    frame: ntkrnl_sys::base::ULONG_PTR,
) {
    (*context).Ebp = frame as u32;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KeGetTrapFrameFrameRegister(
    trap_frame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
) -> ntkrnl_sys::base::ULONG_PTR {
    (*trap_frame).Ebp as ntkrnl_sys::base::ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KeGetTrapFrameStackRegister(
    trap_frame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
) -> ntkrnl_sys::base::ULONG_PTR {
    // KernelMode = 0; PreviousPreviousMode is byte-sized in some headers but
    // bindgen renders the i386 KTRAP_FRAME field as `ULONG`.
    if (*trap_frame).PreviousPreviousMode == 0 {
        (*trap_frame).TempEsp as ntkrnl_sys::base::ULONG_PTR
    } else {
        (*trap_frame).HardwareEsp as ntkrnl_sys::base::ULONG_PTR
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetContextFrameRegister(
    context: *mut ntkrnl_sys::ddk::_CONTEXT,
) -> ntkrnl_sys::base::ULONG_PTR {
    (*context).Rbp as ntkrnl_sys::base::ULONG_PTR
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeSetContextFrameRegister(
    context: *mut ntkrnl_sys::ddk::_CONTEXT,
    frame: ntkrnl_sys::base::ULONG_PTR,
) {
    (*context).Rbp = frame as u64;
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetTrapFrameFrameRegister(
    trap_frame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
) -> ntkrnl_sys::base::ULONG_PTR {
    (*trap_frame).Rbp as ntkrnl_sys::base::ULONG_PTR
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetTrapFrameStackRegister(
    trap_frame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
) -> ntkrnl_sys::base::ULONG_PTR {
    (*trap_frame).Rsp as ntkrnl_sys::base::ULONG_PTR
}

// ============================================================================
// i386 trap-exit FORCEINLINE outlines (`trap_x.h`). Pilot of the trap-exit
// chain port; was previously captured in `ke_rust_shims.c` via the
// `_saved_*` rename trick. Direct Rust impls here resolve the bare
// stdcall-decorated symbol (`_KiExitSystemCallDebugChecks@8`), so the
// Rust caller in `ke/i386/traphdlr.rs` and (defunct) C `traphdlr.c` link
// against this body directly. amd64 and arm64 don't have these — i386-only.
// ============================================================================

/// Trap-exit consistency check (DBG builds only — the C macro is
/// `#define KiExitTrapDebugChecks(x,y)` in retail). Validates: interrupts
/// disabled, trap-frame magic, FS = PCR, exception lists non-NULL,
/// SkipPreviousMode contract. Recursion-guarded via the existing global
/// `StopChecking` (defined in `shim_exports.rs`). The KDBG-gated DR-reg
/// validation block in the original FORCEINLINE is omitted here because
/// the active i386 build defines KDBG, so the captured `_saved_*` body
/// never included it either.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KiExitTrapDebugChecks(
    TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
    SkipPreviousMode: ntkrnl_sys::base::BOOLEAN,
) {
    extern "C" {
        static mut StopChecking: ntkrnl_sys::base::BOOLEAN;
        // DbgPrint is C-variadic; declare per-shape aliases through the same
        // symbol so each call site can pass the right number of args under
        // i386 cdecl (caller pushes right-to-left + cleans, so fixed-arg
        // ABI matches variadic ABI).
        #[link_name = "DbgPrint"]
        fn DbgPrint_0(fmt: *const u8) -> ntkrnl_sys::base::ULONG;
        #[link_name = "DbgPrint"]
        fn DbgPrint_u(fmt: *const u8, a: ntkrnl_sys::base::ULONG) -> ntkrnl_sys::base::ULONG;
        #[link_name = "DbgPrint"]
        fn DbgPrint_p(
            fmt: *const u8,
            a: *const core::ffi::c_void,
        ) -> ntkrnl_sys::base::ULONG;
    }
    extern "C" {
        fn KiDumpTrapFrame(TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME);
    }

    const EFLAGS_INTERRUPT_MASK: ntkrnl_sys::base::ULONG = 0x200;
    const KGDT_R0_PCR: u16 = 0x30;
    const TRAP_FRAME_MAGIC: ntkrnl_sys::base::ULONG = 0xBADB0D00;

    #[inline(always)]
    unsafe fn read_eflags() -> ntkrnl_sys::base::ULONG {
        let v: ntkrnl_sys::base::ULONG;
        core::arch::asm!("pushfd; pop {0:e}", out(reg) v, options(nomem, preserves_flags));
        v
    }
    #[inline(always)]
    unsafe fn read_fs() -> u16 {
        let v: u16;
        core::arch::asm!("mov {0:x}, fs", out(reg) v, options(nomem, nostack, preserves_flags));
        v
    }
    #[inline(always)]
    unsafe fn debugbreak() {
        core::arch::asm!("int 3", options(nomem, nostack));
    }

    /* Don't check recursively */
    if StopChecking != 0 {
        return;
    }
    StopChecking = 1;

    /* Make sure interrupts are disabled */
    let eflags = read_eflags();
    if (eflags & EFLAGS_INTERRUPT_MASK) != 0 {
        DbgPrint_u(b"Exiting with interrupts enabled: %lx\n\0".as_ptr(), eflags);
        debugbreak();
    }

    /* Make sure this is a real trap frame */
    if (*TrapFrame).DbgArgMark != TRAP_FRAME_MAGIC {
        DbgPrint_0(
            b"Exiting with an invalid trap frame? (No MAGIC in trap frame)\n\0".as_ptr(),
        );
        KiDumpTrapFrame(TrapFrame);
        debugbreak();
    }

    /* Make sure we're not in user-mode or something */
    let fs = read_fs();
    if fs != KGDT_R0_PCR {
        DbgPrint_u(
            b"Exiting with an invalid FS: %lx\n\0".as_ptr(),
            fs as ntkrnl_sys::base::ULONG,
        );
        debugbreak();
    }

    /* Make sure we have a valid SEH chain */
    let pcr_excl = (*ke_get_pcr()).__bindgen_anon_1.NtTib.ExceptionList;
    if pcr_excl.is_null() {
        DbgPrint_p(
            b"Exiting with NULL exception chain: %p\n\0".as_ptr(),
            pcr_excl as *const core::ffi::c_void,
        );
        debugbreak();
    }

    /* Make sure we're restoring a valid SEH chain */
    let tf_excl = (*TrapFrame).ExceptionList;
    if tf_excl.is_null() {
        DbgPrint_p(
            b"Entered a trap with a NULL exception chain: %p\n\0".as_ptr(),
            tf_excl as *const core::ffi::c_void,
        );
        debugbreak();
    }

    /* If we're ignoring previous mode, make sure caller doesn't actually want it */
    if SkipPreviousMode != 0 && (*TrapFrame).PreviousPreviousMode != !0u32 {
        DbgPrint_u(
            b"Exiting a trap witout restoring previous mode, yet previous mode seems valid: %lx\n\0".as_ptr(),
            (*TrapFrame).PreviousPreviousMode,
        );
        debugbreak();
    }

    StopChecking = 0;
}

/// V86-mode trap exit (NORETURN). Drains pending user APCs in a loop,
/// optionally restores debug registers, then tail-calls
/// `KiTrapReturnNoSegments` (asm in `i386/trap.s`). Tail callees:
/// `KiEoiHelper` (Rust, fastcall, NORETURN — taken when EFLAGS.V86 is
/// already cleared) and `KiTrapReturnNoSegments` (asm fastcall NORETURN).
/// The C trampoline declared this `FASTCALL` so the linked symbol is
/// `@KiExitV86Trap@4` — match that here.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KiExitV86Trap(
    TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
) -> ! {
    extern "fastcall" {
        fn KiEoiHelper(TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME) -> !;
        fn KfRaiseIrql(NewIrql: ntkrnl_sys::base::KIRQL) -> ntkrnl_sys::base::KIRQL;
        fn KfLowerIrql(NewIrql: ntkrnl_sys::base::KIRQL);
        fn KiTrapReturnNoSegments(TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME) -> !;
    }
    extern "system" {
        fn KiDeliverApc(
            DeliveryMode: i8, // KPROCESSOR_MODE = CCHAR
            ExceptionFrame: *mut core::ffi::c_void,
            TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
        );
        fn KiHandleDebugRegistersOnTrapExit(
            TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
        );
    }

    const EFLAGS_V86_MASK: ntkrnl_sys::base::ULONG = 0x00020000;
    const APC_LEVEL: ntkrnl_sys::base::KIRQL = 1;
    const KERNEL_MODE: usize = 0;
    const USER_MODE: i8 = 1;
    // DR7: bits we care about for "active debugging". Matches
    // `~DR7_RESERVED_MASK` in trap_x.h — DR7_RESERVED_MASK = 0xDC00 from
    // sdk/include/ndk/i386/asm.h.
    const DR7_RESERVED_MASK: ntkrnl_sys::base::ULONG = 0xDC00;

    let thread = ke_get_current_thread();
    loop {
        // Return if this isn't V86 mode anymore (NORETURN tail).
        if ((*TrapFrame).EFlags & EFLAGS_V86_MASK) == 0 {
            KiEoiHelper(TrapFrame);
        }

        // Turn off the alerted state for kernel mode.
        (*thread).Alerted[KERNEL_MODE] = 0;

        // Are there pending user APCs? (the `__builtin_expect(..., 1)` in
        // C marks the no-pending path as the hot one — Rust has no stable
        // equivalent, so we just rely on the natural branch layout.)
        if (*thread).__bindgen_anon_1.ApcState.UserApcPending == 0 {
            break;
        }

        // Raise to APC level and enable interrupts.
        let old_irql = KfRaiseIrql(APC_LEVEL);
        core::arch::asm!("sti", options(nomem, nostack));

        // Deliver APCs.
        KiDeliverApc(USER_MODE, core::ptr::null_mut(), TrapFrame);

        // Restore IRQL and disable interrupts again.
        KfLowerIrql(old_irql);
        core::arch::asm!("cli", options(nomem, nostack));
    }

    // If we got here, we're still in a valid V8086 context, so quit it.
    if ((*TrapFrame).Dr7 & !DR7_RESERVED_MASK) != 0 {
        KiHandleDebugRegistersOnTrapExit(TrapFrame);
    }

    // Return from interrupt (NORETURN asm tail).
    KiTrapReturnNoSegments(TrapFrame)
}

/// `KiEndInterrupt(Irql, TrapFrame)` — FORCEINLINE in `ke.h`. Disables
/// interrupts, calls HAL EOI, then tail-jumps into `KiEoiHelper`
/// (NORETURN). The audit's "rustc 1.92 stdcall/fastcall ABI bug"
/// re-probe came back negative on rustc 1.92.0 (ded5c06cf, 2025-12-08):
/// disasm of `KiExitV86Trap` shows correct fastcall arg-passing for
/// `@KfRaiseIrql@4` / `@KiEoiHelper@4` calls, so this no longer needs
/// a C trampoline. i386 cdecl signature matches the original
/// `void __cdecl KiEndInterrupt(...)`.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KiEndInterrupt(
    irql: ntkrnl_sys::base::KIRQL,
    trap_frame: PKTRAP_FRAME,
) -> ! {
    extern "system" {
        fn HalEndSystemInterrupt(Irql: ntkrnl_sys::base::KIRQL, TrapFrame: PKTRAP_FRAME);
    }
    extern "fastcall" {
        fn KiEoiHelper(TrapFrame: PKTRAP_FRAME) -> !;
    }

    core::arch::asm!("cli", options(nomem, nostack));
    HalEndSystemInterrupt(irql, trap_frame);
    KiEoiHelper(trap_frame)
}

/// Pure debug helper: dump every KTRAP_FRAME field via DbgPrint. The C
/// FORCEINLINE has no NTAPI annotation so the calling convention is cdecl
/// — match that here so call-sites can declare it `extern "C"`. Called only
/// from `KiExitTrapDebugChecks` on the trap-frame-magic-failure path; not
/// hot.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn KiDumpTrapFrame(TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME) {
    extern "C" {
        #[link_name = "DbgPrint"]
        fn DbgPrint_u(fmt: *const u8, a: ntkrnl_sys::base::ULONG) -> ntkrnl_sys::base::ULONG;
        #[link_name = "DbgPrint"]
        fn DbgPrint_p(
            fmt: *const u8,
            a: *const core::ffi::c_void,
        ) -> ntkrnl_sys::base::ULONG;
    }
    let tf = &*TrapFrame;
    DbgPrint_u(b"DbgEbp: %x\n\0".as_ptr(), tf.DbgEbp);
    DbgPrint_u(b"DbgEip: %x\n\0".as_ptr(), tf.DbgEip);
    DbgPrint_u(b"DbgArgMark: %x\n\0".as_ptr(), tf.DbgArgMark);
    DbgPrint_u(b"DbgArgPointer: %x\n\0".as_ptr(), tf.DbgArgPointer);
    DbgPrint_u(b"TempSegCs: %x\n\0".as_ptr(), tf.TempSegCs);
    DbgPrint_u(b"TempEsp: %x\n\0".as_ptr(), tf.TempEsp);
    DbgPrint_u(b"Dr0: %x\n\0".as_ptr(), tf.Dr0);
    DbgPrint_u(b"Dr1: %x\n\0".as_ptr(), tf.Dr1);
    DbgPrint_u(b"Dr2: %x\n\0".as_ptr(), tf.Dr2);
    DbgPrint_u(b"Dr3: %x\n\0".as_ptr(), tf.Dr3);
    DbgPrint_u(b"Dr6: %x\n\0".as_ptr(), tf.Dr6);
    DbgPrint_u(b"Dr7: %x\n\0".as_ptr(), tf.Dr7);
    DbgPrint_u(b"SegGs: %x\n\0".as_ptr(), tf.SegGs);
    DbgPrint_u(b"SegEs: %x\n\0".as_ptr(), tf.SegEs);
    DbgPrint_u(b"SegDs: %x\n\0".as_ptr(), tf.SegDs);
    DbgPrint_u(b"Edx: %x\n\0".as_ptr(), tf.Edx);
    DbgPrint_u(b"Ecx: %x\n\0".as_ptr(), tf.Ecx);
    DbgPrint_u(b"Eax: %x\n\0".as_ptr(), tf.Eax);
    DbgPrint_u(b"PreviousPreviousMode: %x\n\0".as_ptr(), tf.PreviousPreviousMode);
    DbgPrint_p(
        b"ExceptionList: %p\n\0".as_ptr(),
        tf.ExceptionList as *const core::ffi::c_void,
    );
    DbgPrint_u(b"SegFs: %x\n\0".as_ptr(), tf.SegFs);
    DbgPrint_u(b"Edi: %x\n\0".as_ptr(), tf.Edi);
    DbgPrint_u(b"Esi: %x\n\0".as_ptr(), tf.Esi);
    DbgPrint_u(b"Ebx: %x\n\0".as_ptr(), tf.Ebx);
    DbgPrint_u(b"Ebp: %x\n\0".as_ptr(), tf.Ebp);
    DbgPrint_u(b"ErrCode: %x\n\0".as_ptr(), tf.ErrCode);
    DbgPrint_u(b"Eip: %x\n\0".as_ptr(), tf.Eip);
    DbgPrint_u(b"SegCs: %x\n\0".as_ptr(), tf.SegCs);
    DbgPrint_u(b"EFlags: %x\n\0".as_ptr(), tf.EFlags);
    DbgPrint_u(b"HardwareEsp: %x\n\0".as_ptr(), tf.HardwareEsp);
    DbgPrint_u(b"HardwareSegSs: %x\n\0".as_ptr(), tf.HardwareSegSs);
    DbgPrint_u(b"V86Es: %x\n\0".as_ptr(), tf.V86Es);
    DbgPrint_u(b"V86Ds: %x\n\0".as_ptr(), tf.V86Ds);
    DbgPrint_u(b"V86Fs: %x\n\0".as_ptr(), tf.V86Fs);
    DbgPrint_u(b"V86Gs: %x\n\0".as_ptr(), tf.V86Gs);
}

/// Syscall-exit consistency check. On user-mode return: bugcheck if IRQL is
/// non-PASSIVE; if APCs are disabled or the thread is attached, log + reset
/// CombinedApcDisable instead of bugchecking (PSEH_CLANG longjmp can leak
/// critical regions when exceptions skip KeLeaveCriticalRegion).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn KiExitSystemCallDebugChecks(
    SystemCall: ntkrnl_sys::base::ULONG,
    TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME,
) {
    extern "system" {
        fn KiUserTrap(TrapFrame: *mut ntkrnl_sys::ddk::_KTRAP_FRAME) -> ntkrnl_sys::base::BOOLEAN;
        fn KeBugCheckEx(
            BugCheckCode: ntkrnl_sys::base::ULONG,
            BugCheckParameter1: ntkrnl_sys::base::ULONG_PTR,
            BugCheckParameter2: ntkrnl_sys::base::ULONG_PTR,
            BugCheckParameter3: ntkrnl_sys::base::ULONG_PTR,
            BugCheckParameter4: ntkrnl_sys::base::ULONG_PTR,
        ) -> !;
    }
    // DbgPrint is C-variadic; on i386 cdecl, calling with a fixed arg list
    // matches the variadic ABI (caller pushes right-to-left, caller cleans up).
    extern "C" {
        fn DbgPrint(
            fmt: *const u8,
            syscall: ntkrnl_sys::base::ULONG,
            apc_state_index: u8,
            combined_apc_disable: ntkrnl_sys::base::ULONG,
        ) -> ntkrnl_sys::base::ULONG;
    }

    const PASSIVE_LEVEL: u8 = 0;
    const IRQL_GT_ZERO_AT_SYSTEM_SERVICE: ntkrnl_sys::base::ULONG = 0x4A;
    const ORIGINAL_APC_ENVIRONMENT: u8 = 0;

    if KiUserTrap(TrapFrame) == 0 {
        return;
    }

    let pcr = ke_get_pcr();
    let old_irql = (*pcr).Irql;
    if old_irql != PASSIVE_LEVEL {
        // Forcibly put us in a sane state.
        (*pcr).Irql = PASSIVE_LEVEL;
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
        KeBugCheckEx(
            IRQL_GT_ZERO_AT_SYSTEM_SERVICE,
            SystemCall as ntkrnl_sys::base::ULONG_PTR,
            old_irql as ntkrnl_sys::base::ULONG_PTR,
            0,
            0,
        );
    }

    let thread = ke_get_current_thread();
    let apc_state_index = (*thread).ApcStateIndex;
    let combined = (*thread).__bindgen_anon_4.CombinedApcDisable;
    if apc_state_index != ORIGINAL_APC_ENVIRONMENT || combined != 0 {
        const FMT: &[u8] =
            b"APC_INDEX_MISMATCH: syscall=%lx ApcStateIndex=%d CombinedApcDisable=%lx\n\0";
        DbgPrint(FMT.as_ptr(), SystemCall, apc_state_index, combined);
        (*thread).__bindgen_anon_4.CombinedApcDisable = 0;
    }
}
