//! time/strdate.cpp: `_strdate_s`, `_wstrdate_s`, `_strdate`, `_wstrdate`.
//!
//! ## Details that are silent if wrong
//! * **The year is truncated to two digits by `% 100`** -- `_strdate`'s
//!   Y2K-unsafe "MM/DD/YY" format, unchanged from the original CRT.
//! * **`_RESET_STRING` runs (first byte only, release build) BEFORE the
//!   `size_in_chars >= 9` check**, so a too-small buffer still gets an
//!   empty string in `buffer[0]`, not untouched memory.
//! * **The buffer is written LOW TO HIGH in one pass with no bounds
//!   re-check per byte** -- exactly 9 characters (`MM/DD/YY\0`), assuming
//!   the `size_in_chars >= 9` validation already passed.

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
    fn slash() -> Self;
    fn nul() -> Self;
}
impl Chr for u8 {
    fn digit(v: i32) -> Self {
        b'0' + v as u8
    }
    fn slash() -> Self {
        b'/'
    }
    fn nul() -> Self {
        0
    }
}
impl Chr for u16 {
    fn digit(v: i32) -> Self {
        (b'0' + v as u8) as u16
    }
    fn slash() -> Self {
        b'/' as u16
    }
    fn nul() -> Self {
        0
    }
}

unsafe fn common_strdate_s<C: Chr>(buffer: *mut C, size_in_chars: usize) -> c_int {
    if !validate(!buffer.is_null() && size_in_chars > 0, EINVAL) {
        return EINVAL;
    }
    *buffer = C::nul();
    if !validate(size_in_chars >= 9, ERANGE) {
        return ERANGE;
    }

    let mut local_time: SystemTimeLocal = core::mem::zeroed();
    GetLocalTime(&mut local_time);

    let month = local_time.w_month as i32;
    let day = local_time.w_day as i32;
    let year = local_time.w_year as i32 % 100;

    *buffer.add(0) = C::digit(month / 10);
    *buffer.add(1) = C::digit(month % 10);
    *buffer.add(2) = C::slash();
    *buffer.add(3) = C::digit(day / 10);
    *buffer.add(4) = C::digit(day % 10);
    *buffer.add(5) = C::slash();
    *buffer.add(6) = C::digit(year / 10);
    *buffer.add(7) = C::digit(year % 10);
    *buffer.add(8) = C::nul();

    0
}

#[no_mangle]
pub unsafe extern "C" fn _strdate_s(buffer: *mut u8, size_in_chars: usize) -> c_int {
    common_strdate_s(buffer, size_in_chars)
}
#[no_mangle]
pub unsafe extern "C" fn _wstrdate_s(buffer: *mut u16, size_in_chars: usize) -> c_int {
    common_strdate_s(buffer, size_in_chars)
}

#[no_mangle]
pub unsafe extern "C" fn _strdate(buffer: *mut u8) -> *mut u8 {
    if common_strdate_s(buffer, 9) != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}
#[no_mangle]
pub unsafe extern "C" fn _wstrdate(buffer: *mut u16) -> *mut u16 {
    if common_strdate_s(buffer, 9) != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}
