//! The Vista threadpool WORK-object subset: CreateThreadpoolWork, SubmitThreadpoolWork,
//! WaitForThreadpoolWorkCallbacks, CloseThreadpoolWork and TrySubmitThreadpoolCallback.
//!
//! Upstream's kernel32_vista/threadpool.c is a thin layer over ntdll's NT threadpool (Tp*),
//! which upstream implemented after our fork and our Rust ntdll does not have; porting that
//! is owed (upstream-sync-merge-decisions.md). This is NOT that port. It is the work-object
//! subset, built on QueueUserWorkItem, because Wine-10 quartz's filter graph needs exactly
//! these calls and nothing else in the tree does yet.
//!
//! Deliberate limits, all LOUD rather than silent:
//! - A callback environment (custom pool, cleanup group, priority) is refused with
//!   ERROR_NOT_SUPPORTED and a NULL return. Ignoring it would run the callback somewhere
//!   the caller did not ask for.
//! - The callback instance is NULL. The CallbackMayRunLong / *WhenCallbackReturns family
//!   has no implementation here to hand one to.
//! - WaitForThreadpoolWorkCallbacks(cancel = TRUE) WAITS for queued callbacks instead of
//!   cancelling them: QueueUserWorkItem cannot dequeue. That is the conservative direction
//!   (the callbacks run, as they would have without the cancel), and quartz's callbacks
//!   re-check their state under the graph lock.

use crate::common::{RtlGetProcessHeap, SetLastError};
use ntdll_sys::nt::{BOOL, DWORD, NTSTATUS, PRTL_CONDITION_VARIABLE, PRTL_SRWLOCK, PVOID, ULONG};

const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
const ERROR_NOT_SUPPORTED: DWORD = 50;
const ERROR_INVALID_PARAMETER: DWORD = 87;
const HEAP_ZERO_MEMORY: ULONG = 0x8;
const WT_EXECUTEDEFAULT: ULONG = 0;

/// PTP_WORK_CALLBACK: (Instance, Context, Work).
type WorkCallback = Option<unsafe extern "system" fn(PVOID, PVOID, PVOID)>;
/// PTP_SIMPLE_CALLBACK: (Instance, Context).
type SimpleCallback = Option<unsafe extern "system" fn(PVOID, PVOID)>;

extern "system" {
    fn QueueUserWorkItem(
        Function: Option<unsafe extern "system" fn(PVOID) -> DWORD>,
        Context: PVOID,
        Flags: ULONG,
    ) -> BOOL;
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: usize) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> u8;
    fn RtlAcquireSRWLockExclusive(SRWLock: PRTL_SRWLOCK);
    fn RtlReleaseSRWLockExclusive(SRWLock: PRTL_SRWLOCK);
    fn RtlWakeAllConditionVariable(CondVar: PRTL_CONDITION_VARIABLE);
    fn RtlSleepConditionVariableSRW(
        CondVar: PRTL_CONDITION_VARIABLE,
        SRWLock: PRTL_SRWLOCK,
        Timeout: *mut ntdll_sys::nt::LARGE_INTEGER,
        Flags: ULONG,
    ) -> NTSTATUS;
}

/// The TP_WORK behind a PTP_WORK. `Pending` (callbacks submitted but not finished) and
/// `Closed` are only touched under `Lock`; `Idle` is signalled when Pending reaches 0.
/// Zeroed memory is a valid initial RTL_SRWLOCK and RTL_CONDITION_VARIABLE.
#[repr(C)]
struct Work {
    Lock: ntdll_sys::nt::RTL_SRWLOCK,
    Idle: ntdll_sys::nt::RTL_CONDITION_VARIABLE,
    Callback: WorkCallback,
    Context: PVOID,
    Pending: u32,
    Closed: bool,
}

#[repr(C)]
struct Simple {
    Callback: SimpleCallback,
    Context: PVOID,
}

unsafe fn alloc<T>() -> *mut T {
    RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<T>()) as *mut T
}

unsafe fn free<T>(p: *mut T) {
    RtlFreeHeap(RtlGetProcessHeap(), 0, p as PVOID);
}

unsafe extern "system" fn work_trampoline(Param: PVOID) -> DWORD {
    let w = Param as *mut Work;
    if let Some(cb) = (*w).Callback {
        cb(core::ptr::null_mut(), (*w).Context, w as PVOID);
    }
    RtlAcquireSRWLockExclusive(&mut (*w).Lock);
    (*w).Pending -= 1;
    let last = (*w).Pending == 0;
    if last {
        RtlWakeAllConditionVariable(&mut (*w).Idle);
    }
    let free_now = last && (*w).Closed;
    RtlReleaseSRWLockExclusive(&mut (*w).Lock);
    if free_now {
        free(w);
    }
    0
}

unsafe extern "system" fn simple_trampoline(Param: PVOID) -> DWORD {
    let s = Param as *mut Simple;
    let (cb, ctx) = ((*s).Callback, (*s).Context);
    free(s);
    if let Some(cb) = cb {
        cb(core::ptr::null_mut(), ctx);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CreateThreadpoolWork(pfnwk: WorkCallback, pv: PVOID, pcbe: PVOID) -> PVOID {
    if pfnwk.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }
    if !pcbe.is_null() {
        SetLastError(ERROR_NOT_SUPPORTED);
        return core::ptr::null_mut();
    }
    let w = alloc::<Work>();
    if w.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }
    (*w).Callback = pfnwk;
    (*w).Context = pv;
    w as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn SubmitThreadpoolWork(pwk: PVOID) {
    let w = pwk as *mut Work;
    RtlAcquireSRWLockExclusive(&mut (*w).Lock);
    (*w).Pending += 1;
    RtlReleaseSRWLockExclusive(&mut (*w).Lock);
    if QueueUserWorkItem(Some(work_trampoline), w as PVOID, WT_EXECUTEDEFAULT) == 0 {
        // SubmitThreadpoolWork has no failure return. Undo the count so a waiter is not
        // left blocked on a callback that will never run.
        RtlAcquireSRWLockExclusive(&mut (*w).Lock);
        (*w).Pending -= 1;
        if (*w).Pending == 0 {
            RtlWakeAllConditionVariable(&mut (*w).Idle);
        }
        RtlReleaseSRWLockExclusive(&mut (*w).Lock);
    }
}

#[no_mangle]
pub unsafe extern "system" fn WaitForThreadpoolWorkCallbacks(pwk: PVOID, _fCancelPendingCallbacks: BOOL) {
    let w = pwk as *mut Work;
    RtlAcquireSRWLockExclusive(&mut (*w).Lock);
    while (*w).Pending != 0 {
        RtlSleepConditionVariableSRW(&mut (*w).Idle, &mut (*w).Lock, core::ptr::null_mut(), 0);
    }
    RtlReleaseSRWLockExclusive(&mut (*w).Lock);
}

#[no_mangle]
pub unsafe extern "system" fn CloseThreadpoolWork(pwk: PVOID) {
    let w = pwk as *mut Work;
    RtlAcquireSRWLockExclusive(&mut (*w).Lock);
    // Callbacks still outstanding: the last one to finish frees it.
    let free_now = (*w).Pending == 0;
    (*w).Closed = true;
    RtlReleaseSRWLockExclusive(&mut (*w).Lock);
    if free_now {
        free(w);
    }
}

#[no_mangle]
pub unsafe extern "system" fn TrySubmitThreadpoolCallback(pfns: SimpleCallback, pv: PVOID, pcbe: PVOID) -> BOOL {
    if pfns.is_none() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if !pcbe.is_null() {
        SetLastError(ERROR_NOT_SUPPORTED);
        return 0;
    }
    let s = alloc::<Simple>();
    if s.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }
    (*s).Callback = pfns;
    (*s).Context = pv;
    if QueueUserWorkItem(Some(simple_trampoline), s as PVOID, WT_EXECUTEDEFAULT) == 0 {
        free(s);
        return 0;
    }
    1
}
