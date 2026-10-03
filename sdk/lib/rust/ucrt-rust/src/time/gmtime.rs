//! time/gmtime.cpp: `_gmtime32_s`, `_gmtime64_s`, `_gmtime32`, `_gmtime64`,
//! `__getgmtimebuf`.
//!
//! ## Details that are silent if wrong
//! * **The tm struct is memset to `0xff` BEFORE validating `timp`** -- a
//!   caller with `ptm != null` but `timp == null` still gets an
//!   all-`0xff`-byte (NOT zeroed, NOT untouched) `tm` back.
//! * **The `caltim <= max_time_t` upper-bound check does NOT apply the
//!   `_MAX_LOCAL_TIME` slack for `_gmtime32_s`** (any positive `__time32_t`
//!   is `<= LONG_MAX`, the comment notes, so this is a no-op there), but
//!   DOES for `_gmtime64_s` (`max_time_t + _MAX_LOCAL_TIME`) -- both are
//!   coded as the SAME expression here, since the constant folding is what
//!   makes the 32-bit case a no-op, not a special case in the source.
//! * **`compute_year`'s two overloads use genuinely different arithmetic**:
//!   the 32-bit one special-cases a 4-year interval directly (with an
//!   explicit leap-year flag threaded through by hand); the 64-bit one
//!   divides by a plain 365-day year and then CORRECTS for elapsed leap
//!   years, with its own underflow-adjustment branch. Not a shared
//!   generic body in the C, and not unified here either.
//! * **`tm_wday` is computed from the ORIGINAL `*timp`, not the working
//!   `caltim`** (which has already been reduced to seconds-within-year by
//!   the time `tm_wday` is computed).
//! * **The month/day loop scans `mdays` (leap or non-leap `_days`/`_lpdays`)
//!   for the first index whose table value is `>= tm_yday`, using a
//!   strict-less-than continuation condition**, then backs up one --
//!   ported as the identical linear scan, not replaced with a binary
//!   search or arithmetic shortcut.
//! * **`__getgmtimebuf` allocates lazily and is NEVER freed by this file**
//!   (freed only at thread teardown, elsewhere, not yet ported).

use core::ffi::c_int;

use super::Tm;
use crate::ptd::AcrtPtd;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
    fn malloc(size: usize) -> *mut core::ffi::c_void;
}

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}
unsafe fn validate_noexc(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
    }
    ok
}

/// `compute_year<__time32_t>`.
fn compute_year32(caltim: &mut i32, is_leap_year: &mut bool) -> i32 {
    const FOUR_YEAR_SEC: i32 = super::FOUR_YEAR_SEC as i32;
    const YEAR_SEC: i32 = super::YEAR_SEC as i32;
    const DAY_SEC: i32 = super::DAY_SEC as i32;

    let mut tmptim = *caltim / FOUR_YEAR_SEC;
    *caltim -= tmptim * FOUR_YEAR_SEC;

    tmptim = tmptim * 4 + 70;

    if *caltim >= YEAR_SEC {
        tmptim += 1;
        *caltim -= YEAR_SEC;

        if *caltim >= YEAR_SEC {
            tmptim += 1;
            *caltim -= YEAR_SEC;

            if *caltim >= YEAR_SEC + DAY_SEC {
                tmptim += 1;
                *caltim -= YEAR_SEC + DAY_SEC;
            } else {
                *is_leap_year = true;
            }
        }
    }

    tmptim
}

/// `compute_year<__time64_t>`.
fn compute_year64(caltim: &mut i64, is_leap_year: &mut bool) -> i32 {
    let mut tmptim = (*caltim / super::YEAR_SEC) as i32 + 70;
    *caltim -= (tmptim - 70) as i64 * super::YEAR_SEC;

    *caltim -= super::elapsed_leap_years(tmptim) as i64 * super::DAY_SEC;

    if *caltim < 0 {
        *caltim += super::YEAR_SEC;
        tmptim -= 1;
        if super::is_leap_year(tmptim) {
            *caltim += super::DAY_SEC;
            *is_leap_year = true;
        }
    } else if super::is_leap_year(tmptim) {
        *is_leap_year = true;
    }

    tmptim
}

trait GmtimeSrc: Copy {
    fn to_i64(self) -> i64;
    fn compute_year(caltim_i64: i64, is_leap_year: &mut bool) -> (i32, i64);
}
impl GmtimeSrc for i32 {
    fn to_i64(self) -> i64 {
        self as i64
    }
    fn compute_year(caltim_i64: i64, is_leap_year: &mut bool) -> (i32, i64) {
        let mut c32 = caltim_i64 as i32;
        let y = compute_year32(&mut c32, is_leap_year);
        (y, c32 as i64)
    }
}
impl GmtimeSrc for i64 {
    fn to_i64(self) -> i64 {
        self
    }
    fn compute_year(caltim_i64: i64, is_leap_year: &mut bool) -> (i32, i64) {
        let mut c64 = caltim_i64;
        let y = compute_year64(&mut c64, is_leap_year);
        (y, c64)
    }
}

/// `common_gmtime_s<TimeType>`. Note the traits used for the bounds check
/// are ALWAYS the `__time64_t` traits, per the C
/// (`typedef __crt_time_time_t_traits<__time64_t> time_traits;`).
unsafe fn common_gmtime_s<T: GmtimeSrc>(ptm: *mut Tm, timp: *const T) -> c_int {
    if !validate(!ptm.is_null(), EINVAL) {
        return EINVAL;
    }
    core::ptr::write_bytes(ptm, 0xff, 1);

    if !validate(!timp.is_null(), EINVAL) {
        return EINVAL;
    }
    let orig = *timp;
    let caltim0 = orig.to_i64();

    if !validate_noexc(caltim0 >= super::MIN_LOCAL_TIME, EINVAL) {
        return EINVAL;
    }
    if !validate_noexc(caltim0 <= super::MAX_TIME_T_64 + super::MAX_LOCAL_TIME, EINVAL) {
        return EINVAL;
    }

    let mut is_leap_year = false;
    let (year, mut caltim) = T::compute_year(caltim0, &mut is_leap_year);
    (*ptm).tm_year = year;

    let tm_yday = (caltim / super::DAY_SEC) as i32;
    (*ptm).tm_yday = tm_yday;
    caltim -= tm_yday as i64 * super::DAY_SEC;

    let mdays: &[i32; 13] = if is_leap_year { &super::days::_lpdays } else { &super::days::_days };

    let mut tmptim = 1usize;
    while mdays[tmptim] < tm_yday {
        tmptim += 1;
    }
    tmptim -= 1;
    (*ptm).tm_mon = tmptim as i32;
    (*ptm).tm_mday = tm_yday - mdays[tmptim];

    (*ptm).tm_wday = ((caltim0 / super::DAY_SEC) as i32 + super::BASE_DOW) % 7;

    (*ptm).tm_hour = (caltim / 3600) as i32;
    caltim -= (*ptm).tm_hour as i64 * 3600;

    (*ptm).tm_min = (caltim / 60) as i32;
    (*ptm).tm_sec = (caltim - (*ptm).tm_min as i64 * 60) as i32;

    (*ptm).tm_isdst = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime32_s(result: *mut Tm, time_value: *const i32) -> c_int {
    common_gmtime_s(result, time_value)
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime64_s(result: *mut Tm, time_value: *const i64) -> c_int {
    common_gmtime_s(result, time_value)
}

#[no_mangle]
pub unsafe extern "C" fn __getgmtimebuf() -> *mut Tm {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        *_errno() = 12; // ENOMEM
        return core::ptr::null_mut();
    }

    if !(*ptd).gmtime_buffer.is_null() {
        return (*ptd).gmtime_buffer as *mut Tm;
    }

    let buf = malloc(core::mem::size_of::<Tm>()) as *mut Tm;
    if buf.is_null() {
        *_errno() = 12; // ENOMEM
        return core::ptr::null_mut();
    }
    (*ptd).gmtime_buffer = buf as *mut core::ffi::c_void;
    buf
}

unsafe fn common_gmtime<T: GmtimeSrc>(time_value: *const T) -> *mut Tm {
    let ptm = __getgmtimebuf();
    if ptm.is_null() {
        return core::ptr::null_mut();
    }
    if common_gmtime_s(ptm, time_value) != 0 {
        return core::ptr::null_mut();
    }
    ptm
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime32(time_value: *const i32) -> *mut Tm {
    common_gmtime(time_value)
}

#[no_mangle]
pub unsafe extern "C" fn _gmtime64(time_value: *const i64) -> *mut Tm {
    common_gmtime(time_value)
}
