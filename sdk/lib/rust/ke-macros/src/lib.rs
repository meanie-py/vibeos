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
//! Architecturally identical to `ke-inlines` (sibling rlib, isolated rustc
//! invocation cuts the cross-CGU captures(none) attribute deduction wire).
//! Replicates the same `KeGetCurrentPrcb` inline asm locally to dodge the
//! cross-crate `#[inline(always)]`-over-`asm!` non-inline gotcha at
//! `opt-level = 1` (see `feedback_cross_crate_inline_asm.md`).

#![no_std]

use ntkrnl_sys::base::{InsertHeadList, InsertTailList, BOOLEAN};
use ntkrnl_sys::ddk::{
    PKMUTANT, PKPRCB, PKTHREAD, _DISPATCHER_HEADER, _KMUTANT, _KPCR, _KTHREAD, _LIST_ENTRY,
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

// `_KMUTANT` is referenced via the C trampoline only as a `*mut` opaque
// handle — silence the unused-import lint when the type alias didn't end up
// referenced because of arch-cfg paths.
#[allow(dead_code)]
const _ASSERT_KMUTANT: usize = core::mem::size_of::<_KMUTANT>();
