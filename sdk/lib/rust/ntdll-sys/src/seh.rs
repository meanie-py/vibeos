//! Real `__try/__except` and `__try/__finally` for user-mode Rust.
//!
//! The user-mode translations inherited DUMMY_PSEH's dead `_SEH2_EXCEPT` arms, and the
//! C's `RpcTryExcept` with them: an RPC stub raises `RPC_S_SERVER_UNAVAILABLE` when its
//! server is not up (dnsapi before Dnscache starts, advapi32 before services.exe answers),
//! and with no handler the caller's process dies instead of getting an error code.
//! ntdll exports the kernel's trampolines (`ntoskrnl/sehtry.c`, built with
//! `-DSEHTRY_USER_MODE`), so this is the kernel's `ntkrnl_sys::seh` over the same ABI,
//! with the same two rules for the closures:
//! - **Nothing captured by value may need `Drop`.** A caught exception abandons the
//!   body's frames without running Rust destructors.
//! - **Cleanup that must run on the exception path goes in [`try_finally`]'s `fin`.**

use core::ffi::c_void;

use crate::nt::NTSTATUS;

/// Filter verdicts, as `excpt.h`.
pub const EXCEPTION_EXECUTE_HANDLER: i32 = 1;
pub const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
pub const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;

type Body = unsafe extern "C" fn(*mut c_void);
type Filter = unsafe extern "C" fn(*mut c_void, *mut c_void) -> i32;
type Finally = unsafe extern "C" fn(*mut c_void, u8);

extern "C" {
    fn RtlSehTryExcept(
        body: Body,
        body_ctx: *mut c_void,
        filter: Option<Filter>,
        filter_ctx: *mut c_void,
        code: *mut i32,
    ) -> u8;
    fn RtlSehTryFinally(body: Body, body_ctx: *mut c_void, fin: Finally, fin_ctx: *mut c_void);
}

struct Pair<S, F> {
    state: *mut S,
    f: Option<F>,
}

unsafe extern "C" fn call_body<S, F: FnOnce(&mut S)>(ctx: *mut c_void) {
    let p = &mut *(ctx as *mut Pair<S, F>);
    if let Some(f) = p.f.take() {
        f(&mut *p.state);
    }
}

unsafe extern "C" fn call_filter<S, G: FnMut(&mut S, *mut c_void) -> i32>(
    ctx: *mut c_void,
    pointers: *mut c_void,
) -> i32 {
    let p = &mut *(ctx as *mut Pair<S, G>);
    match p.f.as_mut() {
        Some(g) => g(&mut *p.state, pointers),
        None => EXCEPTION_CONTINUE_SEARCH,
    }
}

unsafe extern "C" fn call_finally<S, H: FnOnce(&mut S, bool)>(ctx: *mut c_void, abnormal: u8) {
    let p = &mut *(ctx as *mut Pair<S, H>);
    if let Some(h) = p.f.take() {
        h(&mut *p.state, abnormal != 0);
    }
}

/// `__try { body(state) } __except (filter(state, ExceptionPointers)) { .. }`.
///
/// `Ok(())` if the body completed; `Err(code)` if an exception was handled here.
/// `filter` receives the `EXCEPTION_POINTERS` as a raw pointer and returns one of
/// the `EXCEPTION_*` verdicts. `state` is how the body hands results out, and how
/// the filter sees what the body did (`IrpContext` in the FS drivers).
pub unsafe fn try_except<S, F, G>(state: &mut S, body: F, filter: G) -> Result<(), NTSTATUS>
where
    F: FnOnce(&mut S),
    G: FnMut(&mut S, *mut c_void) -> i32,
{
    let state: *mut S = state;
    let mut b = Pair { state, f: Some(body) };
    let mut g = Pair { state, f: Some(filter) };
    let mut code: i32 = 0;
    let caught = RtlSehTryExcept(
        call_body::<S, F>,
        &mut b as *mut Pair<S, F> as *mut c_void,
        Some(call_filter::<S, G>),
        &mut g as *mut Pair<S, G> as *mut c_void,
        &mut code,
    );
    if caught != 0 {
        Err(code)
    } else {
        Ok(())
    }
}

/// `__try { body(state) } __finally { fin(state, AbnormalTermination()) }`.
///
/// `fin` runs exactly once: after a normal return with `false`, or with `true`
/// while an exception unwinds through this frame toward a handler further out.
pub unsafe fn try_finally<S, F, H>(state: &mut S, body: F, fin: H)
where
    F: FnOnce(&mut S),
    H: FnOnce(&mut S, bool),
{
    let state: *mut S = state;
    let mut b = Pair { state, f: Some(body) };
    let mut h = Pair { state, f: Some(fin) };
    RtlSehTryFinally(
        call_body::<S, F>,
        &mut b as *mut Pair<S, F> as *mut c_void,
        call_finally::<S, H>,
        &mut h as *mut Pair<S, H> as *mut c_void,
    );
}

/// The exception code in an `EXCEPTION_POINTERS` handed to a filter.
pub unsafe fn exception_code(pointers: *mut c_void) -> NTSTATUS {
    // EXCEPTION_POINTERS { PEXCEPTION_RECORD, PCONTEXT }; ExceptionCode is the
    // record's first field on every arch.
    let record = *(pointers as *const *const i32);
    *record
}

/// `RpcTryExcept { body } RpcExcept(EXCEPTION_EXECUTE_HANDLER) { RpcExceptionCode() }`:
/// the body's result, or the RPC exception code (an `RPC_S_*`/`ERROR_*` value) it raised.
/// Callers map the code exactly as their C did (`ScmRpcStatusToWinError`, `I_RpcMapWin32Status`, ...).
pub unsafe fn rpc_try<T, F: FnOnce() -> T>(body: F) -> Result<T, u32> {
    let mut out = core::mem::MaybeUninit::<T>::uninit();
    match try_except(&mut out, |o| { o.write(body()); }, |_, _| EXCEPTION_EXECUTE_HANDLER) {
        // Ok means the body returned, and its last act was the write.
        Ok(()) => Ok(out.assume_init()),
        Err(code) => Err(code as u32),
    }
}
