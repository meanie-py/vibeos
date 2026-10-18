//! PS subsystem inline functions (from ps_x.h).

use crate::base::*;
use crate::ddk;

// ============================================================================
// PspSetCrossThreadFlag / PspSetProcessFlag / PspClearProcessFlag
// ============================================================================

/// PspSetCrossThreadFlag — InterlockedOr on Thread->CrossThreadFlags.
#[inline]
pub unsafe fn PspSetCrossThreadFlag(thread: ddk::PETHREAD, flag: ULONG) {
    let flags_ptr = core::ptr::addr_of_mut!((*thread).__bindgen_anon_7.CrossThreadFlags) as *mut i32;
    let atomic = core::sync::atomic::AtomicI32::from_ptr(flags_ptr);
    atomic.fetch_or(flag as i32, core::sync::atomic::Ordering::SeqCst);
}

/// PspSetProcessFlag — InterlockedOr on Process->Flags.
#[inline]
pub unsafe fn PspSetProcessFlag(process: ddk::PEPROCESS, flag: ULONG) {
    let flags_ptr = core::ptr::addr_of_mut!((*process).__bindgen_anon_2.Flags) as *mut i32;
    let atomic = core::sync::atomic::AtomicI32::from_ptr(flags_ptr);
    atomic.fetch_or(flag as i32, core::sync::atomic::Ordering::SeqCst);
}

/// PspClearProcessFlag — InterlockedAnd on Process->Flags with ~Flag.
#[inline]
pub unsafe fn PspClearProcessFlag(process: ddk::PEPROCESS, flag: ULONG) {
    let flags_ptr = core::ptr::addr_of_mut!((*process).__bindgen_anon_2.Flags) as *mut i32;
    let atomic = core::sync::atomic::AtomicI32::from_ptr(flags_ptr);
    atomic.fetch_and(!(flag as i32), core::sync::atomic::Ordering::SeqCst);
}

// ============================================================================
// PspPrioritySeparationFromMask / PspQuantumTypeFromMask / PspQuantumLengthFromMask
// ============================================================================

/// PspPrioritySeparationFromMask — extract bits 0-1.
#[inline]
pub unsafe fn PspPrioritySeparationFromMask(mask: ULONG) -> ULONG {
    mask & 3
}

/// PspQuantumTypeFromMask — extract bits 2-3 (value & 12).
#[inline]
pub unsafe fn PspQuantumTypeFromMask(mask: ULONG) -> ULONG {
    mask & 12
}

/// PspQuantumLengthFromMask — extract bits 4-5 (value & 48).
#[inline]
pub unsafe fn PspQuantumLengthFromMask(mask: ULONG) -> ULONG {
    mask & 48
}

// ============================================================================
// PspRunCreate*NotifyRoutines / PspRunLegoRoutine
// ============================================================================

// 64 slots as Windows has, not ps.h's 8; every reader of these arrays is Rust,
// and both copies agree.
const PSP_MAX_CREATE_THREAD_NOTIFY: usize = 64; // nolint: shadowed-c-define
const PSP_MAX_CREATE_PROCESS_NOTIFY: usize = 64; // nolint: shadowed-c-define
const PSP_MAX_LOAD_IMAGE_NOTIFY: usize = 64; // nolint: shadowed-c-define

extern "C" {
    static mut PspThreadNotifyRoutineCount: ULONG;
    static mut PspProcessNotifyRoutineCount: ULONG;
    static mut PspThreadNotifyRoutine: [ddk::EX_CALLBACK; PSP_MAX_CREATE_THREAD_NOTIFY];
    static mut PspProcessNotifyRoutine: [ddk::EX_CALLBACK; PSP_MAX_CREATE_PROCESS_NOTIFY];
    static mut PspLoadImageNotifyRoutine: [ddk::EX_CALLBACK; PSP_MAX_LOAD_IMAGE_NOTIFY];
    static mut PspLegoNotifyRoutine: Option<unsafe extern "C" fn(ddk::PKTHREAD)>;
}

/// PspRunCreateThreadNotifyRoutines — invoke registered thread creation callbacks.
#[inline]
pub unsafe fn PspRunCreateThreadNotifyRoutines(
    current_thread: ddk::PETHREAD,
    create: BOOLEAN,
) {
    if PspThreadNotifyRoutineCount != 0 {
        for i in 0..PSP_MAX_CREATE_THREAD_NOTIFY {
            super::ex::ExDoCallBack(
                &mut PspThreadNotifyRoutine[i],
                (*current_thread).Cid.UniqueProcess as PVOID,
                (*current_thread).Cid.UniqueThread as PVOID,
                create as ULONG_PTR as PVOID,
            );
        }
    }
}

/// PspRunCreateProcessNotifyRoutines — invoke registered process creation callbacks.
#[inline]
pub unsafe fn PspRunCreateProcessNotifyRoutines(
    current_process: ddk::PEPROCESS,
    create: BOOLEAN,
) {
    if PspProcessNotifyRoutineCount != 0 {
        for i in 0..PSP_MAX_CREATE_PROCESS_NOTIFY {
            super::ex::ExDoCallBack(
                &mut PspProcessNotifyRoutine[i],
                (*current_process).InheritedFromUniqueProcessId as PVOID,
                (*current_process).UniqueProcessId as PVOID,
                create as ULONG_PTR as PVOID,
            );
        }
    }
}

/// PspRunLoadImageNotifyRoutines — invoke registered image load callbacks.
#[inline]
pub unsafe fn PspRunLoadImageNotifyRoutines(
    full_image_name: ddk::PUNICODE_STRING,
    process_id: ddk::HANDLE,
    image_info: *mut ddk::_IMAGE_INFO,
) {
    for i in 0..PSP_MAX_LOAD_IMAGE_NOTIFY {
        super::ex::ExDoCallBack(
            &mut PspLoadImageNotifyRoutine[i],
            full_image_name as PVOID,
            process_id as PVOID,
            image_info as PVOID,
        );
    }
}

/// PspRunLegoRoutine — call the LEGO routine if set.
#[inline]
pub unsafe fn PspRunLegoRoutine(thread: ddk::PKTHREAD) {
    if let Some(routine) = PspLegoNotifyRoutine {
        routine(thread);
    }
}

// ============================================================================
// PspLock/UnlockProcessSecurity, PspLock/UnlockThreadSecurity
// ============================================================================

/// PspLockProcessSecurityShared — enter critical region + acquire shared push lock.
#[inline]
pub unsafe fn PspLockProcessSecurityShared(process: ddk::PEPROCESS) {
    super::ke::KeEnterCriticalRegion();
    super::ex::ExAcquirePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));
}

/// PspUnlockProcessSecurityShared — release shared push lock + leave critical region.
#[inline]
pub unsafe fn PspUnlockProcessSecurityShared(process: ddk::PEPROCESS) {
    super::ex::ExReleasePushLockShared(core::ptr::addr_of_mut!((*process).ProcessLock));
    super::ke::KeLeaveCriticalRegion();
}

/// PspLockProcessSecurityExclusive — enter critical region + acquire exclusive push lock.
#[inline]
pub unsafe fn PspLockProcessSecurityExclusive(process: ddk::PEPROCESS) {
    super::ke::KeEnterCriticalRegion();
    super::ex::ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*process).ProcessLock));
}

/// PspUnlockProcessSecurityExclusive — release exclusive push lock + leave critical region.
#[inline]
pub unsafe fn PspUnlockProcessSecurityExclusive(process: ddk::PEPROCESS) {
    super::ex::ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*process).ProcessLock));
    super::ke::KeLeaveCriticalRegion();
}

/// PspLockThreadSecurityShared — enter critical region + acquire shared push lock.
#[inline]
pub unsafe fn PspLockThreadSecurityShared(thread: ddk::PETHREAD) {
    super::ke::KeEnterCriticalRegion();
    super::ex::ExAcquirePushLockShared(core::ptr::addr_of_mut!((*thread).ThreadLock));
}

/// PspUnlockThreadSecurityShared — release shared push lock + leave critical region.
#[inline]
pub unsafe fn PspUnlockThreadSecurityShared(thread: ddk::PETHREAD) {
    super::ex::ExReleasePushLockShared(core::ptr::addr_of_mut!((*thread).ThreadLock));
    super::ke::KeLeaveCriticalRegion();
}

/// PspLockThreadSecurityExclusive — enter critical region + acquire exclusive push lock.
#[inline]
pub unsafe fn PspLockThreadSecurityExclusive(thread: ddk::PETHREAD) {
    super::ke::KeEnterCriticalRegion();
    super::ex::ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*thread).ThreadLock));
}

/// PspUnlockThreadSecurityExclusive — release exclusive push lock + leave critical region.
#[inline]
pub unsafe fn PspUnlockThreadSecurityExclusive(thread: ddk::PETHREAD) {
    super::ex::ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*thread).ThreadLock));
    super::ke::KeLeaveCriticalRegion();
}
