//! Rust impls for the working-set lock FORCEINLINEs from `miarm.h`
//! (Phase 1 batch M4).
//!
//! 13 wrappers covering the Mi*WorkingSet* family + `MM_ANY_WS_LOCK_HELD`:
//!
//! * `MiLockWorkingSet`, `MiUnlockWorkingSet`
//! * `MiLockWorkingSetShared`, `MiUnlockWorkingSetShared`
//! * `MiLockProcessWorkingSet`, `MiUnlockProcessWorkingSet`
//! * `MiLockProcessWorkingSetUnsafe`, `MiUnlockProcessWorkingSetUnsafe`
//! * `MiLockProcessWorkingSetShared`, `MiUnlockProcessWorkingSetShared`
//! * `MiLockProcessWorkingSetForFault`, `MiUnlockProcessWorkingSetForFault`
//! * `MM_ANY_WS_LOCK_HELD`
//!
//! Same augment+isolated-rlib pattern as M1/M2/M3. The C trampolines in
//! `mm_rust_shims.c` retain the ARM64 IRQL UART tracing and the
//! `MmDbgLast{Safe,Excl}{Locker,Unlocker}` debug instrumentation
//! (`__builtin_return_address(0)` is meaningful only in the C frame).
//! The trampoline body collapses to one `_rs_*` call; this rlib provides
//! the actual lock-acquisition impl by delegating to
//! `ntkrnl_sys::inline::mm`.

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiLockWorkingSet(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiLockWorkingSet(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiUnlockWorkingSet(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiUnlockWorkingSet(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiLockWorkingSetShared(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiLockWorkingSetShared(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiUnlockWorkingSetShared(thread: PETHREAD, ws: PMMSUPPORT) {
    let thread = opaque_ptr(thread);
    let ws = opaque_ptr(ws);
    ntkrnl_sys::inline::mm::MiUnlockWorkingSetShared(thread, ws)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiLockProcessWorkingSet(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSet(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiUnlockProcessWorkingSet(process: PEPROCESS, thread: PETHREAD) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSet(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiLockProcessWorkingSetUnsafe(
    process: PEPROCESS,
    thread: PETHREAD,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetUnsafe(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiUnlockProcessWorkingSetUnsafe(
    process: PEPROCESS,
    thread: PETHREAD,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetUnsafe(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiLockProcessWorkingSetShared(
    process: PEPROCESS,
    thread: PETHREAD,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetShared(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiUnlockProcessWorkingSetShared(
    process: PEPROCESS,
    thread: PETHREAD,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetShared(process, thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiLockProcessWorkingSetForFault(
    process: PEPROCESS,
    thread: PETHREAD,
    safe: BOOLEAN,
    shared: BOOLEAN,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MiLockProcessWorkingSetForFault(process, thread, safe, shared)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MiUnlockProcessWorkingSetForFault(
    process: PEPROCESS,
    thread: PETHREAD,
    safe: *mut BOOLEAN,
    shared: *mut BOOLEAN,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let safe = opaque_ptr(safe);
    let shared = opaque_ptr(shared);
    ntkrnl_sys::inline::mm::MiUnlockProcessWorkingSetForFault(process, thread, safe, shared)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _rs_MM_ANY_WS_LOCK_HELD(thread: PETHREAD) -> BOOLEAN {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::mm::MM_ANY_WS_LOCK_HELD(thread)
}
