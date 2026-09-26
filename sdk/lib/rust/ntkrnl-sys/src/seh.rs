//! Real `__try/__except` and `__try/__finally` for Rust kernel code.
//!
//! The kernel is built with `USE_DUMMY_PSEH`, so every translated `_SEH2_EXCEPT` arm
//! is dead and a raise-by-design (`CdRaiseStatus`, `STATUS_CANT_WAIT`, a Cc in-page
//! error) bugchecks 0x1E instead of reaching the handler the C wrote for it. These
//! wrappers call the body through the C trampolines in `ntoskrnl/sehtry.c`, which
//! own a real handler: native SEH on amd64/arm64, a hand-built `fs:[0]` frame on
//! i386. An exception raised anywhere below the body unwinds back here.
//!
//! Two rules the closures must keep:
//! - **Nothing captured by value may need `Drop`.** A caught exception abandons the
//!   body's frames without running Rust destructors. Capture raw pointers and
//!   references; the translated FS code already does.
//! - **Cleanup that must run on the exception path goes in [`try_finally`]'s `fin`,**
//!   never after the body. A translated `__finally` written inline after the try
//!   block is skipped when the body raises, and the lock it releases stays held.

use core::ffi::c_void;

use crate::base::NTSTATUS;

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
