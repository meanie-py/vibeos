//! startup/argv_data.cpp: the command-line/argument globals `__argc`, `__argv`,
//! `__wargv`, `_pgmptr`, `_wpgmptr`, `_acmdln`, `_wcmdln` and their accessors.
//!
//! `_CRT_DECLARE_GLOBAL_VARIABLES_DIRECTLY` is one of ucrt's own compile
//! definitions (`sdk/lib/ucrt/CMakeLists.txt`), so the public headers this
//! crate's C siblings see declare these as raw `extern` data (`stdlib.h`,
//! `corecrt_startup.h`), not the `#define __argc (*__p___argc())` macro form a
//! non-ucrt consumer gets. `startup/argv_parsing.cpp`, `startup/argv_winmain.cpp`
//! and `internal/initialization.cpp` (all still C) read and WRITE these seven
//! globals directly by exact name, so every one must keep it.
//!
//! Verified with `tools/compare_coff_data.py` against `argv_data.cpp.obj`:
//! matches on all three arches (see the batch report) once each C symbol's
//! true size is pinned (`__argc:4`, the rest `:4`/`:8` by pointer width) --
//! the tool's default "distance to next symbol" extent otherwise includes the
//! C++ compiler's alignment padding before the following pointer-typed field,
//! which is not part of the object itself.
//!
//! ## Details that are silent if wrong
//!
//! * **"Either the narrow or wide variables will be set, but not both"**, per
//!   the C's own comment, and any of the seven may be null during execution --
//!   nothing here assumes otherwise.
//! * **`_get_pgmptr`/`_get_wpgmptr` validate `_pgmptr`/`_wpgmptr` themselves,
//!   not just `result`** -- a program that hasn't set its own module name yet
//!   (or was started in a mode that never does) makes these fail with EINVAL
//!   even though `result` was perfectly valid.
//! * **`__acrt_initialize_command_line` unconditionally overwrites `_acmdln`/
//!   `_wcmdln`** with `GetCommandLineA`/`GetCommandLineW` every time it runs
//!   (once, at startup) -- it does not check whether they were already set.
//! * **`__acrt_uninitialize_command_line` does nothing** (`true`, ignoring
//!   `terminating`) -- the command-line pointers are never freed, because they
//!   point into memory Windows itself owns (`GetCommandLine*`'s return), not
//!   anything the CRT allocated.

#![allow(non_upper_case_globals)]

use core::ffi::c_int;

extern "system" {
    fn GetCommandLineA() -> *mut u8;
    fn GetCommandLineW() -> *mut u16;
}

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

/// `_VALIDATE_RETURN_ERRCODE(expr, err)`.
unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

// ── the data (exact C names -- see the module doc) ─────────────────────────────

/// The number of arguments in `__argv`/`__wargv`.
#[no_mangle]
pub static mut __argc: c_int = 0;
/// The arguments as narrow strings.
#[no_mangle]
pub static mut __argv: *mut *mut u8 = core::ptr::null_mut();
/// The arguments as wide strings.
#[no_mangle]
pub static mut __wargv: *mut *mut u16 = core::ptr::null_mut();
/// The name of the program as a narrow string.
#[no_mangle]
pub static mut _pgmptr: *mut u8 = core::ptr::null_mut();
/// The name of the program as a wide string.
#[no_mangle]
pub static mut _wpgmptr: *mut u16 = core::ptr::null_mut();
/// The raw command line as a narrow string.
#[no_mangle]
pub static mut _acmdln: *mut u8 = core::ptr::null_mut();
/// The raw command line as a wide string.
#[no_mangle]
pub static mut _wcmdln: *mut u16 = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn __p___argc() -> *mut c_int {
    core::ptr::addr_of_mut!(__argc)
}

#[no_mangle]
pub unsafe extern "C" fn __p___argv() -> *mut *mut *mut u8 {
    core::ptr::addr_of_mut!(__argv)
}

#[no_mangle]
pub unsafe extern "C" fn __p___wargv() -> *mut *mut *mut u16 {
    core::ptr::addr_of_mut!(__wargv)
}

#[no_mangle]
pub unsafe extern "C" fn __p__pgmptr() -> *mut *mut u8 {
    core::ptr::addr_of_mut!(_pgmptr)
}

#[no_mangle]
pub unsafe extern "C" fn __p__wpgmptr() -> *mut *mut u16 {
    core::ptr::addr_of_mut!(_wpgmptr)
}

#[no_mangle]
pub unsafe extern "C" fn __p__acmdln() -> *mut *mut u8 {
    core::ptr::addr_of_mut!(_acmdln)
}

#[no_mangle]
pub unsafe extern "C" fn __p__wcmdln() -> *mut *mut u16 {
    core::ptr::addr_of_mut!(_wcmdln)
}

#[no_mangle]
pub unsafe extern "C" fn _get_wpgmptr(result: *mut *mut u16) -> c_int {
    if !validate(!result.is_null(), EINVAL) {
        return EINVAL;
    }
    if !validate(!_wpgmptr.is_null(), EINVAL) {
        return EINVAL;
    }
    *result = _wpgmptr;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _get_pgmptr(result: *mut *mut u8) -> c_int {
    if !validate(!result.is_null(), EINVAL) {
        return EINVAL;
    }
    if !validate(!_pgmptr.is_null(), EINVAL) {
        return EINVAL;
    }
    *result = _pgmptr;
    0
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_command_line() -> bool {
    _acmdln = GetCommandLineA();
    _wcmdln = GetCommandLineW();
    true
}

#[no_mangle]
pub unsafe extern "C" fn __acrt_uninitialize_command_line(_terminating: bool) -> bool {
    true
}
