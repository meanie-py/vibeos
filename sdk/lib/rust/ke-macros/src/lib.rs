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

    let alertable: BOOLEAN = if wait_mode == USER_MODE { 1 } else { 0 };
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

// `_KMUTANT` and `_KWAIT_BLOCK` are referenced via the C trampolines only as
// `*mut` opaque handles — silence the unused-import lint when the type
// alias didn't end up referenced because of arch-cfg paths.
#[allow(dead_code)]
const _ASSERT_KMUTANT: usize = core::mem::size_of::<_KMUTANT>();
#[allow(dead_code)]
const _ASSERT_KWAIT_BLOCK: usize = core::mem::size_of::<_KWAIT_BLOCK>();
