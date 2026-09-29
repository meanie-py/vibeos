//! `startup/abort.cpp` -- `abort`, `_set_abort_behavior`, and the
//! `__abort_behavior` flags word.
//!
//! Details that are silent if wrong:
//! * `__abort_behavior` starts at `_CALL_REPORTFAULT` (2) -- the release
//!   value; `_DEBUG` is not defined for this build, so `_WRITE_ABORT_MSG` is
//!   neither set initially nor acted on (the "abort() has been called" report
//!   is compiled out; the flag is still stored and returned).
//! * `abort` raises SIGABRT only when a handler is INSTALLED
//!   (`__acrt_get_sigabrt_handler() != SIG_DFL`); with the default action it
//!   goes straight on. If the handler returns, it continues too.
//! * With `_CALL_REPORTFAULT`: arm64 fast-fails unconditionally
//!   (`FAST_FAIL_FATAL_APP_EXIT` = 7, `brk #0xF003`); i386/amd64 fast-fail
//!   (`int 0x29`) only if `PF_FASTFAIL_AVAILABLE`, otherwise
//!   `__acrt_call_reportfault(_CRT_DEBUGGER_ABORT = 3, STATUS_FATAL_APP_EXIT,
//!   EXCEPTION_NONCONTINUABLE)` and then falls through to `_exit(3)`.
//! * Without it: `_exit(3)` -- no atexit, no flush.
//! * `_set_abort_behavior(flags, mask)` = `old & ~mask | flags & mask`,
//!   returning `old`; no validation, not locked.

use core::ffi::{c_int, c_uint};

use super::exit::_exit;
use super::CRT_SIGABRT;

/// `_WRITE_ABORT_MSG` (`stdlib.h`).
#[allow(dead_code)]
const WRITE_ABORT_MSG: c_uint = 0x1;
/// `_CALL_REPORTFAULT`.
const CALL_REPORTFAULT: c_uint = 0x2;

/// `SIG_DFL`.
const SIG_DFL: usize = 0;

/// `FAST_FAIL_FATAL_APP_EXIT` (`winnt.h`).
const FAST_FAIL_FATAL_APP_EXIT: u32 = 7;

/// `PF_FASTFAIL_AVAILABLE` (`sdk/include/xdk/ketypes.h`).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const PF_FASTFAIL_AVAILABLE: u32 = 23;
/// `_CRT_DEBUGGER_ABORT` (`corecrt_internal.h`).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const CRT_DEBUGGER_ABORT: c_int = 3;
/// `STATUS_FATAL_APP_EXIT`.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const STATUS_FATAL_APP_EXIT: u32 = 0x4000_0015;
/// `EXCEPTION_NONCONTINUABLE`.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const EXCEPTION_NONCONTINUABLE: u32 = 0x1;

/// `unsigned int __abort_behavior = _INIT_ABORT_BEHAVIOR;` -- a C-named
/// global (`extern "C"` in abort.cpp).
#[no_mangle]
pub static mut __abort_behavior: c_uint = CALL_REPORTFAULT;

extern "C" {
    fn __acrt_get_sigabrt_handler() -> usize;
    fn raise(signum: c_int) -> c_int;
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn __acrt_call_reportfault(debugger_hook_code: c_int, exception_code: u32, exception_flags: u32);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "system" {
    fn IsProcessorFeaturePresent(feature: u32) -> i32;
}

/// `__fastfail(code)`: `int 0x29` with the code in ecx. Not declared
/// `noreturn`, as in `crate::invalid_parameter` (a noreturn asm block gets a
/// trailing `ud2` that `scan_ud2` refuses); the instruction ends the process.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn fastfail(code: u32) {
    core::arch::asm!("int 0x29", in("ecx") code, options(nomem, nostack));
}
/// `__fastfail(code)` on arm64: `brk #0xF003` with the code in x0 -- what the
/// compiled C emits (`mov w0, #7; brk #0xf003`).
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn fastfail(code: u32) -> ! {
    core::arch::asm!("brk #0xF003", in("x0") code as u64, options(noreturn));
}

/// `abort`.
#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    // (_DEBUG only: _WRITE_ABORT_MSG -> __acrt_report_runtime_error.)

    // Raise SIGABRT only if the user installed a handler for it.
    let sigabrt_action = __acrt_get_sigabrt_handler();
    if sigabrt_action != SIG_DFL {
        raise(CRT_SIGABRT);
    }

    // No handler, or it returned: exit anyway.
    if core::ptr::addr_of!(__abort_behavior).read() & CALL_REPORTFAULT != 0 {
        #[cfg(target_arch = "aarch64")]
        fastfail(FAST_FAIL_FATAL_APP_EXIT);

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if IsProcessorFeaturePresent(PF_FASTFAIL_AVAILABLE) != 0 {
                fastfail(FAST_FAIL_FATAL_APP_EXIT);
            }

            __acrt_call_reportfault(CRT_DEBUGGER_ABORT, STATUS_FATAL_APP_EXIT, EXCEPTION_NONCONTINUABLE);
        }
    }

    // The same as the default SIGABRT action.
    _exit(3)
}

/// `_set_abort_behavior`: returns the old flags.
#[no_mangle]
pub unsafe extern "C" fn _set_abort_behavior(flags: c_uint, mask: c_uint) -> c_uint {
    let oldflags = core::ptr::addr_of!(__abort_behavior).read();
    core::ptr::addr_of_mut!(__abort_behavior).write(oldflags & !mask | flags & mask);
    oldflags
}
