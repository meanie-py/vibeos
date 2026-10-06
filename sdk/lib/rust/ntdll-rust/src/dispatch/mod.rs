//! Rust translation of `dll/ntdll/dispatch/dispatch.c` — User-Mode NT
//! Dispatchers.
//!
//! ARM64-ONLY. On i386/amd64 the entire dispatcher set lives in the
//! per-arch `dispatch/<arch>/dispatch.S` (this `.c` is not compiled there).
//! On arm64 the `.S` provides `LdrInitializeThunk` / `KiUserApcDispatcher` /
//! `KiUserExceptionDispatcher` (the last is `#ifndef _M_ARM64` in the C), so
//! `dispatch.c` contributes only the remaining two — translated here:
//!   * `KiRaiseUserExceptionDispatcher`
//!   * `KiUserCallbackDispatcher` (the win32k user-mode callback entry)
//!
//! Hence this module is `#[cfg(target_arch = "aarch64")]`-gated in lib.rs:
//! defining these `#[no_mangle]` symbols on x86 would collide with the `.S`.
//!
//! Both are `NTAPI` (`extern "system"`); on arm64 that is the standard AAPCS
//! with no name decoration, matching the symbols the `.S`/kernel expect.

use core::ffi::c_void;
use core::mem::zeroed;
use core::ptr::{addr_of_mut, null_mut};
use ntdll_sys::nt::*;

/// `USER_CALL` (dispatch.c): a kernel-callback-table entry —
/// `NTSTATUS (NTAPI *)(PVOID Argument, ULONG ArgumentLength)`.
type USER_CALL = unsafe extern "system" fn(Argument: *mut c_void, ArgumentLength: ULONG) -> NTSTATUS;
const USER32_CALLBACK_COUNT: ULONG = 20;

/// `KiRaiseUserExceptionDispatcher` — re-raises the pending exception code the
/// kernel stashed in `TEB->ExceptionCode` as a user-mode exception.
#[no_mangle]
pub unsafe extern "system" fn KiRaiseUserExceptionDispatcher() {
    // C leaves ExceptionAddress / ExceptionInformation uninitialized; zero-init
    // here is behaviorally equivalent and avoids reading uninit stack.
    let mut ExceptionRecord: EXCEPTION_RECORD = zeroed();

    ExceptionRecord.ExceptionCode = (*NtCurrentTeb()).ExceptionCode as DWORD;
    ExceptionRecord.ExceptionFlags = 0;
    ExceptionRecord.ExceptionRecord = null_mut();
    ExceptionRecord.NumberParameters = 0;

    // Raise the exception.
    RtlRaiseException(addr_of_mut!(ExceptionRecord));
}

/// Set by wowx64.dll in an x86-64 guest process (and nowhere else): every
/// win32k callback is then the GUEST's, because PEB->KernelCallbackTable is
/// the guest user32's table of x86-64 code, which must not be called natively.
/// The hook runs it in the guest and returns its status. Exported as data
/// (ntdll.spec) so wowx64.dll can fill it without a private interface.
#[no_mangle]
pub static mut WowX64UserCallbackHook: Option<
    unsafe extern "system" fn(Index: ULONG, Argument: *mut c_void, ArgumentLength: ULONG) -> NTSTATUS,
> = None;

/// `KiUserCallbackDispatcher` — the kernel transfers here (via
/// `KeUserModeCallback`) to invoke a win32k user-mode callback by table index,
/// then returns the result to the kernel via `ZwCallbackReturn` (which does not
/// return). Drives every win32k → user callout (window procs, hooks, …).
#[no_mangle]
pub unsafe extern "system" fn KiUserCallbackDispatcher(
    Index: ULONG,
    Argument: *mut c_void,
    ArgumentLength: ULONG,
) {
    if Index >= USER32_CALLBACK_COUNT {
        ZwCallbackReturn(null_mut(), 0, STATUS_INVALID_PARAMETER);
    }

    if let Some(Hook) = *core::ptr::addr_of!(WowX64UserCallbackHook) {
        ZwCallbackReturn(null_mut(), 0, Hook(Index, Argument, ArgumentLength));
    }

    // PEB->KernelCallbackTable is an array of USER_CALL function pointers.
    let peb = (*NtCurrentTeb()).ProcessEnvironmentBlock as *const PEB;
    let KernelCallbackTable = (*peb).KernelCallbackTable as *const USER_CALL;

    // Return to the kernel with the result of the callback function.
    let callback = *KernelCallbackTable.add(Index as usize);
    let result = callback(Argument, ArgumentLength);
    ZwCallbackReturn(null_mut(), 0, result);
}
