//! time/strtime.cpp: `_strtime_s`, `_wstrtime_s`, `_strtime`, `_wstrtime`.
//!
//! Structurally identical to `strdate.rs`, just `HH:MM:SS` from
//! `GetLocalTime`'s hour/minute/second instead of `MM/DD/YY` from its
//! month/day/year -- see that file's module doc for the shared details
//! (reset-before-range-check, exactly 9 bytes written, etc).

use core::ffi::c_int;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

use super::{GetLocalTime, SystemTimeLocal};

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

trait Chr: Copy {
    fn digit(v: i32) -> Self;
    fn colon() -> Self;
    fn nul() -> Self;
}
impl Chr for u8 {
    fn digit(v: i32) -> Self {
        b'0' + v as u8
    }
    fn colon() -> Self {
        b':'
    }
    fn nul() -> Self {
        0
    }
}
impl Chr for u16 {
    fn digit(v: i32) -> Self {
        (b'0' + v as u8) as u16
    }
    fn colon() -> Self {
        b':' as u16
    }
    fn nul() -> Self {
        0
    }
}

unsafe fn common_strtime_s<C: Chr>(buffer: *mut C, size_in_chars: usize) -> c_int {
    if !validate(!buffer.is_null() && size_in_chars > 0, EINVAL) {
        return EINVAL;
    }
    *buffer = C::nul();
    if !validate(size_in_chars >= 9, ERANGE) {
        return ERANGE;
    }

    let mut local_time: SystemTimeLocal = core::mem::zeroed();
    GetLocalTime(&mut local_time);

    let hours = local_time.w_hour as i32;
    let minutes = local_time.w_minute as i32;
    let seconds = local_time.w_second as i32;

    *buffer.add(0) = C::digit(hours / 10);
    *buffer.add(1) = C::digit(hours % 10);
    *buffer.add(2) = C::colon();
    *buffer.add(3) = C::digit(minutes / 10);
    *buffer.add(4) = C::digit(minutes % 10);
    *buffer.add(5) = C::colon();
    *buffer.add(6) = C::digit(seconds / 10);
    *buffer.add(7) = C::digit(seconds % 10);
    *buffer.add(8) = C::nul();

    0
}

#[no_mangle]
pub unsafe extern "C" fn _strtime_s(buffer: *mut u8, size_in_chars: usize) -> c_int {
    common_strtime_s(buffer, size_in_chars)
}
#[no_mangle]
pub unsafe extern "C" fn _wstrtime_s(buffer: *mut u16, size_in_chars: usize) -> c_int {
    common_strtime_s(buffer, size_in_chars)
}

#[no_mangle]
pub unsafe extern "C" fn _strtime(buffer: *mut u8) -> *mut u8 {
    if common_strtime_s(buffer, 9) != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}
#[no_mangle]
pub unsafe extern "C" fn _wstrtime(buffer: *mut u16) -> *mut u16 {
    if common_strtime_s(buffer, 9) != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}
