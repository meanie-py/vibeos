use super::ke_defs::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;

// GuardedMutex functions are FASTCALL on i386, extern "C" on other architectures.

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeInitializeGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ke::KeInitializeGuardedMutex(GuardedMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeInitializeGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ke::KeInitializeGuardedMutex(GuardedMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ke::KeAcquireGuardedMutex(GuardedMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ke::KeAcquireGuardedMutex(GuardedMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ke::KeReleaseGuardedMutex(GuardedMutex);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ke::KeReleaseGuardedMutex(GuardedMutex);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ddk::KeAcquireGuardedMutexUnsafe(GuardedMutex as _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ddk::KeAcquireGuardedMutexUnsafe(GuardedMutex as _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ddk::KeReleaseGuardedMutexUnsafe(GuardedMutex as _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ntkrnl_sys::ddk::KeReleaseGuardedMutexUnsafe(GuardedMutex as _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeTryToAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) -> BOOLEAN {
    ntkrnl_sys::ddk::KeTryToAcquireGuardedMutex(GuardedMutex as _)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeTryToAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) -> BOOLEAN {
    ntkrnl_sys::ddk::KeTryToAcquireGuardedMutex(GuardedMutex as _)
}

#[no_mangle]
pub unsafe extern "system" fn _KeEnterGuardedRegion() {
    ntkrnl_sys::ddk::KeEnterGuardedRegion();
}

#[no_mangle]
pub unsafe extern "system" fn _KeLeaveGuardedRegion() {
    ntkrnl_sys::ddk::KeLeaveGuardedRegion();
}
