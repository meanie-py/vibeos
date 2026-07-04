use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::inline::ke as ke_inline;

// GuardedMutex functions are FASTCALL on i386, extern "C" on other architectures.
// These are the exported wrappers (ntoskrnl.spec) that call the inline implementations.
// The C originals in gmutex.c do: #undef KeInitializeGuardedMutex, then call
// the _KeInitializeGuardedMutex FORCEINLINE from ke_x.h.

/// lockdep §4: these #[no_mangle] wrappers are the ddk-route funnel for gmutex
/// acquires — capture the return address here so lockdep names the true
/// caller (see shim_exports.rs pushlock shims for the same pattern).
#[cfg(feature = "lockdep")]
macro_rules! lockdep_site {
    () => {
        crate::ke::lockdep::return_address0()
    };
}
#[cfg(not(feature = "lockdep"))]
macro_rules! lockdep_site {
    () => {
        0usize
    };
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeInitializeGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeInitializeGuardedMutex(GuardedMutex as _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeInitializeGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeInitializeGuardedMutex(GuardedMutex as _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeAcquireGuardedMutexSited(GuardedMutex as _, lockdep_site!());
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeAcquireGuardedMutexSited(GuardedMutex as _, lockdep_site!());
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeReleaseGuardedMutex(GuardedMutex as _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeReleaseGuardedMutex(GuardedMutex as _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeAcquireGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeAcquireGuardedMutexUnsafeSited(GuardedMutex as _, lockdep_site!());
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeAcquireGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeAcquireGuardedMutexUnsafeSited(GuardedMutex as _, lockdep_site!());
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeReleaseGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeReleaseGuardedMutexUnsafe(GuardedMutex as _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeReleaseGuardedMutexUnsafe(GuardedMutex: PKGUARDED_MUTEX) {
    ke_inline::_KeReleaseGuardedMutexUnsafe(GuardedMutex as _);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn KeTryToAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) -> BOOLEAN {
    ke_inline::_KeTryToAcquireGuardedMutexSited(GuardedMutex as _, lockdep_site!())
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn KeTryToAcquireGuardedMutex(GuardedMutex: PKGUARDED_MUTEX) -> BOOLEAN {
    ke_inline::_KeTryToAcquireGuardedMutexSited(GuardedMutex as _, lockdep_site!())
}

#[no_mangle]
pub unsafe extern "system" fn _KeEnterGuardedRegion() {
    ntkrnl_sys::ddk::KeEnterGuardedRegion();
}

#[no_mangle]
pub unsafe extern "system" fn _KeLeaveGuardedRegion() {
    ntkrnl_sys::ddk::KeLeaveGuardedRegion();
}
