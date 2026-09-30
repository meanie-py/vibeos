//! time/localtime.cpp: `_localtime32_s`, `_localtime64_s`, `_localtime32`,
//! `_localtime64`.
//!
//! `_localtime32_s`/`_localtime64_s` are ALSO declared in `crate::ctime`'s
//! `extern` block (it calls through to them); this is that symbol's one
//! definition.
//!
//! ## Assumptions carried over from the C (violated => undefined behaviour,
//! not a Rust panic):
//! 1. `gmtime` must run before `_isindst` (satisfied here: `gmtime_s` is
//!    always called into `ptm` before `_isindst(ptm)`).
//! 2. `__time64_t` is non-negative seconds since the Unix epoch.
//! 3. The combined timezone + DST adjustment is under 3 days.
//!
//! ## Details that are silent if wrong
//! * **Two disjoint code paths chosen by a WIDE safety margin** (`*ptime >
//!   3*_DAY_SEC && *ptime < max_time_t - 3*_DAY_SEC`): the "safe middle"
//!   path adjusts the `time_t` itself and re-runs `gmtime_s`; the "near an
//!   edge" path adjusts the ALREADY-COMPUTED `tm` fields by hand (with
//!   explicit borrow/carry arithmetic for seconds -> minutes -> hours ->
//!   days) to avoid overflowing/underflowing the `time_t` representation.
//! * **In the "safe middle" path, DST is detected on the ALREADY
//!   timezone-adjusted `ptm`**, and if DST applies, `gmtime_s` is called a
//!   SECOND time with the DST-adjusted `ltime` -- `tm_isdst` is set only in
//!   this second call's aftermath, by hand (`gmtime_s` itself always zeroes
//!   `tm_isdst`).
//! * **In the "near an edge" path, borrowing from `tm_min`/`tm_hour` can
//!   underflow `tm_mday` below 1**, in which case the code assumes the
//!   result lies in December 1969 specifically (`tm_mday += 31; tm_mon =
//!   11; tm_year--;`) -- a hard-coded assumption justified by assumption
//!   #3 above, not a general "previous month" computation.
//! * **The C's `time_traits::localtime_s`/`::gmtime_s` dispatch is by
//!   INSTANTIATION** (`__crt_time_time_t_traits<TimeType>`), i.e. `_gmtime32_s`
//!   only ever recurses into itself, never crosses to the 64-bit sibling.

use core::ffi::c_int;

use super::Tm;
use super::gmtime::{_gmtime32_s, _gmtime64_s};
use super::tzset::_isindst;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn __tzset();
    fn _get_daylight(result: *mut c_int) -> c_int;
    fn _get_dstbias(result: *mut core::ffi::c_long) -> c_int;
    fn _get_timezone(result: *mut core::ffi::c_long) -> c_int;
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

pub(crate) trait LocalSrc: Copy + core::ops::Sub<Output = Self> + core::ops::Add<Output = Self> {
    const MAX_TIME_T: i64;
    fn to_i64(self) -> i64;
    fn from_i64(v: i64) -> Self;
    unsafe fn gmtime_s(ptm: *mut Tm, t: *const Self) -> c_int;
    unsafe fn localtime_s(ptm: *mut Tm, t: *const Self) -> c_int;
}
impl LocalSrc for i32 {
    const MAX_TIME_T: i64 = super::MAX_TIME_T_32;
    fn to_i64(self) -> i64 {
        self as i64
    }
    fn from_i64(v: i64) -> Self {
        v as i32
    }
    unsafe fn gmtime_s(ptm: *mut Tm, t: *const i32) -> c_int {
        _gmtime32_s(ptm, t)
    }
    unsafe fn localtime_s(ptm: *mut Tm, t: *const i32) -> c_int {
        _localtime32_s(ptm, t)
    }
}
impl LocalSrc for i64 {
    const MAX_TIME_T: i64 = super::MAX_TIME_T_64;
    fn to_i64(self) -> i64 {
        self
    }
    fn from_i64(v: i64) -> Self {
        v
    }
    unsafe fn gmtime_s(ptm: *mut Tm, t: *const i64) -> c_int {
        _gmtime64_s(ptm, t)
    }
    unsafe fn localtime_s(ptm: *mut Tm, t: *const i64) -> c_int {
        _localtime64_s(ptm, t)
    }
}

unsafe fn common_localtime_s<T: LocalSrc>(ptm: *mut Tm, ptime: *const T) -> c_int {
    if !validate(!ptm.is_null(), EINVAL) {
        return EINVAL;
    }
    core::ptr::write_bytes(ptm, 0xff, 1);

    if !validate(!ptime.is_null(), EINVAL) {
        return EINVAL;
    }
    let t0 = (*ptime).to_i64();

    if !validate_noexc(t0 >= 0, EINVAL) {
        return EINVAL;
    }
    if !validate_noexc(t0 <= T::MAX_TIME_T, EINVAL) {
        return EINVAL;
    }

    __tzset();

    let mut daylight: c_int = 0;
    let mut dstbias: core::ffi::c_long = 0;
    let mut timezone: core::ffi::c_long = 0;
    _get_daylight(&mut daylight);
    _get_dstbias(&mut dstbias);
    _get_timezone(&mut timezone);

    const DAY_SEC: i64 = super::DAY_SEC;

    if t0 > 3 * DAY_SEC && t0 < T::MAX_TIME_T - 3 * DAY_SEC {
        let mut ltime = T::from_i64(t0 - timezone as i64);

        let status0 = T::gmtime_s(ptm, &ltime);
        if status0 != 0 {
            return status0;
        }

        if daylight != 0 && _isindst(ptm) != 0 {
            ltime = T::from_i64(ltime.to_i64() - dstbias as i64);

            let status1 = T::gmtime_s(ptm, &ltime);
            if status1 != 0 {
                return status1;
            }
            (*ptm).tm_isdst = 1;
        }
    } else {
        let status0 = T::gmtime_s(ptm, ptime);
        if status0 != 0 {
            return status0;
        }

        let mut ltime: i64 = (*ptm).tm_sec as i64;

        if daylight != 0 && _isindst(ptm) != 0 {
            ltime -= timezone as i64 + dstbias as i64;
            (*ptm).tm_isdst = 1;
        } else {
            ltime -= timezone as i64;
        }

        (*ptm).tm_sec = (ltime % 60) as i32;
        if (*ptm).tm_sec < 0 {
            (*ptm).tm_sec += 60;
            ltime -= 60;
        }

        ltime = (*ptm).tm_min as i64 + ltime / 60;
        (*ptm).tm_min = (ltime % 60) as i32;
        if (*ptm).tm_min < 0 {
            (*ptm).tm_min += 60;
            ltime -= 60;
        }

        ltime = (*ptm).tm_hour as i64 + ltime / 60;
        (*ptm).tm_hour = (ltime % 24) as i32;
        if (*ptm).tm_hour < 0 {
            (*ptm).tm_hour += 24;
            ltime -= 24;
        }

        ltime /= 24;

        if ltime > 0 {
            (*ptm).tm_wday = ((*ptm).tm_wday + ltime as i32) % 7;
            (*ptm).tm_mday += ltime as i32;
            (*ptm).tm_yday += ltime as i32;
        } else if ltime < 0 {
            (*ptm).tm_wday = ((*ptm).tm_wday + 7 + ltime as i32) % 7;
            (*ptm).tm_mday += ltime as i32;
            if (*ptm).tm_mday <= 0 {
                (*ptm).tm_mday += 31;
                (*ptm).tm_yday = (*ptm).tm_yday + ltime as i32 + 365;
                (*ptm).tm_mon = 11;
                (*ptm).tm_year -= 1;
            } else {
                (*ptm).tm_yday += ltime as i32;
            }
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn _localtime32_s(ptm: *mut Tm, ptime: *const i32) -> c_int {
    common_localtime_s(ptm, ptime)
}

#[no_mangle]
pub unsafe extern "C" fn _localtime64_s(ptm: *mut Tm, ptime: *const i64) -> c_int {
    common_localtime_s(ptm, ptime)
}

unsafe fn common_localtime<T: LocalSrc>(ptime: *const T) -> *mut Tm {
    let ptm = super::gmtime::__getgmtimebuf();
    if ptm.is_null() {
        return core::ptr::null_mut();
    }
    if T::localtime_s(ptm, ptime) != 0 {
        return core::ptr::null_mut();
    }
    ptm
}

#[no_mangle]
pub unsafe extern "C" fn _localtime32(ptime: *const i32) -> *mut Tm {
    common_localtime(ptime)
}

#[no_mangle]
pub unsafe extern "C" fn _localtime64(ptime: *const i64) -> *mut Tm {
    common_localtime(ptime)
}
