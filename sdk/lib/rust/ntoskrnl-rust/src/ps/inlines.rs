//! Rust wrappers for `ps_x.h` FORCEINLINE functions whose bodies live in
//! `ntkrnl_sys::inline::ps`.
//!
//! Each wrapper is a `#[no_mangle] #[inline(never)]` extern-C entry that
//! launders pointers through `opaque_ptr` and delegates to
//! `ntkrnl_sys::inline::ps::*`.
//!
//! 18 functions total. Kept out (still in `shim_exports`):
//!   - `PspClearCrossThreadFlag` — body in `inline::common` (cross-subsystem
//!     macro), not `inline::ps`.
//!
//! Pilot — folded back from the standalone `ps-inlines` rlib 2026-05-09.
//! The captures(none) firewall today is workspace-wide `lto = false` +
//! `codegen-units = 256`, so the rlib boundary that originally protected
//! these wrappers is no longer load-bearing.

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

// --- Macro trampolines (NTAPI in original C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspSetCrossThreadFlag(thread: PETHREAD, flag: ULONG) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ps::PspSetCrossThreadFlag(thread, flag)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspSetProcessFlag(process: PEPROCESS, flag: ULONG) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::ps::PspSetProcessFlag(process, flag)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspClearProcessFlag(process: PEPROCESS, flag: ULONG) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::ps::PspClearProcessFlag(process, flag)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspPrioritySeparationFromMask(mask: ULONG) -> ULONG {
    ntkrnl_sys::inline::ps::PspPrioritySeparationFromMask(mask)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspQuantumTypeFromMask(mask: ULONG) -> ULONG {
    ntkrnl_sys::inline::ps::PspQuantumTypeFromMask(mask)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspQuantumLengthFromMask(mask: ULONG) -> ULONG {
    ntkrnl_sys::inline::ps::PspQuantumLengthFromMask(mask)
}

// --- FORCEINLINE function wrappers (cdecl in original C shim) ---

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunCreateThreadNotifyRoutines(
    current_thread: PETHREAD,
    create: BOOLEAN,
) {
    let current_thread = opaque_ptr(current_thread);
    ntkrnl_sys::inline::ps::PspRunCreateThreadNotifyRoutines(current_thread, create)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunCreateProcessNotifyRoutines(
    current_process: PEPROCESS,
    create: BOOLEAN,
) {
    let current_process = opaque_ptr(current_process);
    ntkrnl_sys::inline::ps::PspRunCreateProcessNotifyRoutines(current_process, create)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunLoadImageNotifyRoutines(
    full_image_name: PUNICODE_STRING,
    process_id: HANDLE,
    image_info: *mut _IMAGE_INFO,
) {
    let full_image_name = opaque_ptr(full_image_name);
    let process_id = opaque_ptr(process_id);
    let image_info = opaque_ptr(image_info);
    ntkrnl_sys::inline::ps::PspRunLoadImageNotifyRoutines(
        full_image_name,
        process_id,
        image_info,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspRunLegoRoutine(thread: PKTHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ps::PspRunLegoRoutine(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockProcessSecurityShared(process: PEPROCESS) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::ps::PspLockProcessSecurityShared(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockProcessSecurityShared(process: PEPROCESS) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::ps::PspUnlockProcessSecurityShared(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockProcessSecurityExclusive(process: PEPROCESS) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::ps::PspLockProcessSecurityExclusive(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockProcessSecurityExclusive(process: PEPROCESS) {
    let process = opaque_ptr(process);
    ntkrnl_sys::inline::ps::PspUnlockProcessSecurityExclusive(process)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockThreadSecurityShared(thread: PETHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ps::PspLockThreadSecurityShared(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockThreadSecurityShared(thread: PETHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ps::PspUnlockThreadSecurityShared(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspLockThreadSecurityExclusive(thread: PETHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ps::PspLockThreadSecurityExclusive(thread)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspUnlockThreadSecurityExclusive(thread: PETHREAD) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::ps::PspUnlockThreadSecurityExclusive(thread)
}
