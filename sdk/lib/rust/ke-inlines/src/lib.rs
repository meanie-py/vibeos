//! Rust impls of the in-stack-queued-spinlock FORCEINLINEs from `ke_x.h`:
//! the APC lock family (5), the device queue lock pair (2), and
//! `KiReleaseProcessLockFromSynchLevel` (the only ProcessLock fn whose C
//! trampoline lives in `ke_rust_shims.c` rather than `mm_rust_shims.c`).
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

use ntkrnl_sys::base::ULONG_PTR;
use ntkrnl_sys::ddk::{PKDEVICE_QUEUE, PKLOCK_QUEUE_HANDLE, PKPRCB, PKTHREAD, _KPCR};

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
