//! `sdk/lib/ucrt/startup/{exit,onexit,abort,assert,thread}.cpp` -- process
//! termination (`exit` and its five siblings), the `_onexit` registries,
//! `abort`, `_assert`/`_wassert`, and `_beginthread(ex)`/`_endthread(ex)`.
//! One submodule per C file, matching the deletion granularity.
//!
//! Build facts every submodule relies on (all read off this tree's compile
//! command for these TUs, `build-<arch>/compile_commands.json`):
//! * `CRTDLL` is defined, so exit.cpp's statically-linked-CRT branches (the
//!   `_initterm(__xp_a..)`/`_initterm(__xt_a..)` calls and the debug
//!   `__scrt_uninitialize_crt`) are compiled OUT; ucrtbase runs its terminators
//!   from its own DLL_PROCESS_DETACH (`crate::internal::init`), not from here.
//! * `_DEBUG` is NOT defined (only `DBG=1`), so `__abort_behavior` starts at
//!   `_CALL_REPORTFAULT` and `abort` has no "abort() has been called" message.
//! * `_USE_DUMMY_PSEH=1`: every `__try`/`__except` in these files compiles to
//!   "run the body; the handler is dead code behind `_SEH2_Volatile0 != 0`".
//!   So exit.cpp's `atexit_exception_filter` -> `terminate()` and
//!   thread.cpp's `_seh_filter_exe` -> `_exit(code)` never run in the C
//!   either; the ports simply run the protected bodies.
//!
//! Details that are silent if wrong:
//! * `__crt_fast_encode_pointer`/`__crt_fast_decode_pointer` are both
//!   `ptr ^ __security_cookie` in this tree (see `crate::invalid_parameter`).
//!   The onexit tables and the thread-local exit callback are stored ENCODED;
//!   the "empty" value is the encoded null, i.e. the cookie itself.
//! * The exit lock is `__acrt_exit_lock` = lock id 2
//!   (`__acrt_select_exit_lock()` is a plain macro for it here,
//!   `ucrt_reactos/inc/corecrt_internal_state_isolation.h:91`).

use core::ffi::c_int;

pub mod abort;
pub mod assert;
pub mod exit;
pub mod onexit;
pub mod thread;

/// `L"..."`: an ASCII literal widened at compile time, NUL-terminated, as a
/// `&'static [u16; N]`.
macro_rules! w {
    ($s:literal) => {{
        const S: &str = $s;
        const N: usize = S.len() + 1;
        const W: [u16; N] = {
            let b = S.as_bytes();
            let mut w = [0u16; N];
            let mut i = 0;
            while i < b.len() {
                w[i] = b[i] as u16;
                i += 1;
            }
            w
        };
        &W
    }};
}
pub(crate) use w;

/// `__acrt_exit_lock` (`corecrt_internal.h`'s `__acrt_lock_id`: heap, debug,
/// exit -- index 2).
pub(crate) const ACRT_EXIT_LOCK: c_int = 2;

/// `SIGABRT` (`signal.h`: 22).
pub(crate) const CRT_SIGABRT: c_int = 22;

extern "C" {
    fn __acrt_lock(lock: c_int);
    fn __acrt_unlock(lock: c_int);
    /// crt-startup-rust's GS cookie: the key of every fast-encoded pointer.
    static mut __security_cookie: usize;
}

/// `__crt_fast_encode_pointer` (== `__crt_fast_decode_pointer`): XOR with the
/// GS cookie, as in `crate::invalid_parameter`.
#[inline]
pub(crate) unsafe fn fast_code(ptr: usize) -> usize {
    ptr ^ core::ptr::addr_of!(__security_cookie).read()
}

/// `__acrt_lock_and_call(__acrt_select_exit_lock(), f)`. The C's
/// `__try`/`__finally` unlock is plain sequencing under this build's dummy
/// PSEH.
#[inline]
pub(crate) unsafe fn with_exit_lock<R>(f: impl FnOnce() -> R) -> R {
    __acrt_lock(ACRT_EXIT_LOCK);
    let r = f();
    __acrt_unlock(ACRT_EXIT_LOCK);
    r
}
