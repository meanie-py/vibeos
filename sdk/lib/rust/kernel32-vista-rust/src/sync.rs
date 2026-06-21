//! SRW locks / condition variables / InitializeCriticalSectionEx — sync.c.
//! Thin WINAPI wrappers over the ntdll Rtl* primitives (0x600+ exports,
//! resolved via ntdll_vista in kernel32_vista.dll; dormant in kernel32.dll).

use crate::common::{NT_SUCCESS, SetLastError};
use ntdll_sys::nt::{
    BOOL, BOOLEAN, DWORD, INFINITE, LARGE_INTEGER, NTSTATUS, PRTL_CONDITION_VARIABLE,
    PRTL_CRITICAL_SECTION, PRTL_SRWLOCK, STATUS_TIMEOUT, TRUE, ULONG,
};

// SRWLOCK/CONDITION_VARIABLE/CRITICAL_SECTION are layout-identical to their RTL_*
// counterparts (the C casts (PRTL_*)Lock); use the RTL pointer types directly.
extern "system" {
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
    fn RtlAcquireSRWLockExclusive(SRWLock: PRTL_SRWLOCK);
    fn RtlAcquireSRWLockShared(SRWLock: PRTL_SRWLOCK);
    fn RtlReleaseSRWLockExclusive(SRWLock: PRTL_SRWLOCK);
    fn RtlReleaseSRWLockShared(SRWLock: PRTL_SRWLOCK);
    fn RtlInitializeSRWLock(SRWLock: PRTL_SRWLOCK);
    fn RtlTryAcquireSRWLockExclusive(SRWLock: PRTL_SRWLOCK) -> BOOLEAN;
    fn RtlTryAcquireSRWLockShared(SRWLock: PRTL_SRWLOCK) -> BOOLEAN;
    fn RtlInitializeConditionVariable(CondVar: PRTL_CONDITION_VARIABLE);
    fn RtlWakeConditionVariable(CondVar: PRTL_CONDITION_VARIABLE);
    fn RtlWakeAllConditionVariable(CondVar: PRTL_CONDITION_VARIABLE);
    fn RtlSleepConditionVariableCS(
        CondVar: PRTL_CONDITION_VARIABLE,
        CritSect: PRTL_CRITICAL_SECTION,
        Timeout: *mut LARGE_INTEGER,
    ) -> NTSTATUS;
    fn RtlSleepConditionVariableSRW(
        CondVar: PRTL_CONDITION_VARIABLE,
        SRWLock: PRTL_SRWLOCK,
        Timeout: *mut LARGE_INTEGER,
        Flags: ULONG,
    ) -> NTSTATUS;
    fn RtlInitializeCriticalSectionAndSpinCount(
        CritSect: PRTL_CRITICAL_SECTION,
        SpinCount: ULONG,
    ) -> NTSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn AcquireSRWLockExclusive(Lock: PRTL_SRWLOCK) {
    RtlAcquireSRWLockExclusive(Lock);
}
#[no_mangle]
pub unsafe extern "system" fn AcquireSRWLockShared(Lock: PRTL_SRWLOCK) {
    RtlAcquireSRWLockShared(Lock);
}
#[no_mangle]
pub unsafe extern "system" fn InitializeConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE) {
    RtlInitializeConditionVariable(ConditionVariable);
}
#[no_mangle]
pub unsafe extern "system" fn InitializeSRWLock(Lock: PRTL_SRWLOCK) {
    RtlInitializeSRWLock(Lock);
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseSRWLockExclusive(Lock: PRTL_SRWLOCK) {
    RtlReleaseSRWLockExclusive(Lock);
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseSRWLockShared(Lock: PRTL_SRWLOCK) {
    RtlReleaseSRWLockShared(Lock);
}
#[no_mangle]
pub unsafe extern "system" fn TryAcquireSRWLockExclusive(Lock: PRTL_SRWLOCK) -> BOOLEAN {
    RtlTryAcquireSRWLockExclusive(Lock)
}
#[no_mangle]
pub unsafe extern "system" fn TryAcquireSRWLockShared(Lock: PRTL_SRWLOCK) -> BOOLEAN {
    RtlTryAcquireSRWLockShared(Lock)
}

/// FORCEINLINE GetNtTimeout (sync.c): ms → relative NT 100ns, or NULL for INFINITE.
#[inline]
unsafe fn GetNtTimeout(Time: *mut LARGE_INTEGER, Timeout: DWORD) -> *mut LARGE_INTEGER {
    if Timeout == INFINITE {
        return core::ptr::null_mut();
    }
    // C: Time->QuadPart = (ULONGLONG)Timeout * -10000;
    (*Time).QuadPart = (Timeout as u64).wrapping_mul((-10000i64) as u64) as i64;
    Time
}

#[no_mangle]
pub unsafe extern "system" fn SleepConditionVariableCS(
    ConditionVariable: PRTL_CONDITION_VARIABLE,
    CriticalSection: PRTL_CRITICAL_SECTION,
    Timeout: DWORD,
) -> BOOL {
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let Status =
        RtlSleepConditionVariableCS(ConditionVariable, CriticalSection, GetNtTimeout(&mut Time, Timeout));
    if !NT_SUCCESS(Status) || Status == STATUS_TIMEOUT {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SleepConditionVariableSRW(
    ConditionVariable: PRTL_CONDITION_VARIABLE,
    Lock: PRTL_SRWLOCK,
    Timeout: DWORD,
    Flags: ULONG,
) -> BOOL {
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let Status =
        RtlSleepConditionVariableSRW(ConditionVariable, Lock, GetNtTimeout(&mut Time, Timeout), Flags);
    if !NT_SUCCESS(Status) || Status == STATUS_TIMEOUT {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }
    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn WakeAllConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE) {
    RtlWakeAllConditionVariable(ConditionVariable);
}
#[no_mangle]
pub unsafe extern "system" fn WakeConditionVariable(ConditionVariable: PRTL_CONDITION_VARIABLE) {
    RtlWakeConditionVariable(ConditionVariable);
}

#[no_mangle]
pub unsafe extern "system" fn InitializeCriticalSectionEx(
    lpCriticalSection: PRTL_CRITICAL_SECTION,
    dwSpinCount: DWORD,
    _flags: DWORD,
) -> BOOL {
    /* FIXME: Flags ignored (as in C) */
    let Status = RtlInitializeCriticalSectionAndSpinCount(lpCriticalSection, dwSpinCount);
    if !NT_SUCCESS(Status) {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }
    TRUE as BOOL
}
