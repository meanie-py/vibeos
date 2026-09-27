//! time/mktime.cpp: `_mktime32`, `_mktime64`, `_mkgmtime32`, `_mkgmtime64`.
//!
//! ## Details that are silent if wrong
//! * **`tm_mon` is normalized into `0..=11` BEFORE anything else**, folding
//!   the excess/deficit into `tm_year` (`tmptm1 += tb->tm_mon / 12`, then
//!   `tb->tm_mon %= 12`, adjusting for a negative remainder) -- so passing
//!   `tm_mon = 13` for "February of next year" works, and the C++ `%`
//!   (truncating, so `-1 % 12 == -1`) drives the negative-remainder fixup;
//!   Rust's `%` on signed integers has the same truncating behaviour, so no
//!   extra adjustment is needed to match it.
//! * **Every intermediate widening (elapsed days -> hours -> minutes ->
//!   seconds) is overflow-CHECKED** (`ChkAdd`/`ChkMul`, transcribed as
//!   `checked_add`/`checked_mul` here) and fails EINVAL on overflow, in
//!   `TimeType`'s own width -- the C macros test the sign-pattern of a
//!   wrapped result rather than trapping, which is exactly what
//!   `wrapping_add`/`wrapping_mul` + a post-hoc sign check would reproduce,
//!   but Rust's `checked_*` gives the identical answer (`None` iff the
//!   C macro would have flagged overflow) more directly for two's-complement
//!   wraparound.
//! * **`tm_year` is checked against `_BASE_YEAR - 1 ..= max_year + 1`
//!   (note the +/-1 slack) both BEFORE and AFTER the `tm_mon` folding** --
//!   two separate checks, not one.
//! * **DST for `mktime` (not `mkgmtime`) uses the ANSI rule**: if the
//!   caller's `tb->tm_isdst > 0`, DST is forced on; if `< 0`
//!   ("unspecified"), the FIRST (non-DST) `localtime_s` call's own computed
//!   `tm_isdst` decides whether to redo the conversion with the bias added;
//!   if `== 0`, DST is never applied regardless of what `localtime_s` would
//!   have computed.
//! * **`_mkgmtime` (the `use_local_time = false` path) never looks at
//!   `tb->tm_isdst` at all** -- straight `gmtime_s`, no DST branch.
//! * **On ANY failure the ORIGINAL `*tb` is left unmodified** (the C writes
//!   `*tb = tbtemp` only at the very end, on the success path) --
//!   `errno = EINVAL` is set on every failure path here, even the ones the
//!   C reaches via `_VALIDATE_RETURN_NOEXC` (which normally would not also
//!   invoke `_VALIDATE_RETURN`'s handler) BEFORE any tm mutation, so no
//!   handler-invocation asymmetry is introduced besides what the C already
//!   has (`_VALIDATE_RETURN` at entry does call the handler; the `ChkAdd`/
//!   `ChkMul` guards, in the C's own `_VALIDATE_RETURN_NOEXC`, do not).

use core::ffi::c_int;

use super::Tm;
use super::localtime::LocalSrc;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn __tzset();
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
trait MkSrc: LocalSrc {
    const MAX_YEAR: i32;
    fn checked_add(self, other: Self) -> Option<Self>;
    fn checked_mul(self, other: i64) -> Option<Self>;
}
impl MkSrc for i32 {
    const MAX_YEAR: i32 = super::MAX_YEAR_32;
    fn checked_add(self, other: i32) -> Option<i32> {
        self.checked_add(other)
    }
    fn checked_mul(self, other: i64) -> Option<i32> {
        self.checked_mul(other as i32)
    }
}
impl MkSrc for i64 {
    const MAX_YEAR: i32 = super::MAX_YEAR_64;
    fn checked_add(self, other: i64) -> Option<i64> {
        self.checked_add(other)
    }
    fn checked_mul(self, other: i64) -> Option<i64> {
        self.checked_mul(other)
    }
}

unsafe fn common_mktime<T: MkSrc>(tb: *mut Tm, use_local_time: bool) -> T {
    let invalid = T::from_i64(-1);

    if !validate(!tb.is_null(), EINVAL) {
        return invalid;
    }

    let mut tmptm1: i32 = (*tb).tm_year;
    if tmptm1 < super::BASE_YEAR - 1 || tmptm1 > T::MAX_YEAR + 1 {
        *_errno() = EINVAL;
        return invalid;
    }

    if (*tb).tm_mon < 0 || (*tb).tm_mon > 11 {
        tmptm1 += (*tb).tm_mon / 12;
        (*tb).tm_mon %= 12;
        if (*tb).tm_mon < 0 {
            (*tb).tm_mon += 12;
            tmptm1 -= 1;
        }

        if tmptm1 < super::BASE_YEAR - 1 || tmptm1 > T::MAX_YEAR + 1 {
            *_errno() = EINVAL;
            return invalid;
        }
    }

    // HERE: tmptm1 = elapsed years.

    let mut tmptm2: i64 = super::days::_days[(*tb).tm_mon as usize] as i64;
    if super::is_leap_year(tmptm1) && (*tb).tm_mon > 1 {
        tmptm2 += 1;
    }

    let mut tmptm3: i64 = (tmptm1 - super::BASE_YEAR) as i64 * 365 + super::elapsed_leap_years(tmptm1) as i64;
    tmptm3 += tmptm2;

    // Elapsed days to current date -- overflow now possible, in T's width.
    let days_t = T::from_i64(tmptm3);
    let mday_t = T::from_i64((*tb).tm_mday as i64);
    let elapsed_days = match days_t.checked_add(mday_t) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };

    let hours = match elapsed_days.checked_mul(24) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };
    let hours = match hours.checked_add(T::from_i64((*tb).tm_hour as i64)) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };

    let minutes = match hours.checked_mul(60) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };
    let minutes = match minutes.checked_add(T::from_i64((*tb).tm_min as i64)) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };

    let seconds = match minutes.checked_mul(60) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };
    let mut elapsed_seconds = match seconds.checked_add(T::from_i64((*tb).tm_sec as i64)) {
        Some(v) => v,
        None => {
            *_errno() = EINVAL;
            return invalid;
        }
    };

    // HERE: elapsed_seconds holds the (as-yet-unadjusted) result.

    let mut tbtemp = Tm::default();

    if use_local_time {
        __tzset();

        let mut dstbias: core::ffi::c_long = 0;
        let mut timezone: core::ffi::c_long = 0;
        _get_dstbias(&mut dstbias);
        _get_timezone(&mut timezone);

        elapsed_seconds = match elapsed_seconds.checked_add(T::from_i64(timezone as i64)) {
            Some(v) => v,
            None => {
                *_errno() = EINVAL;
                return invalid;
            }
        };

        if T::localtime_s(&mut tbtemp, &elapsed_seconds) != 0 {
            *_errno() = EINVAL;
            return invalid;
        }

        if (*tb).tm_isdst > 0 || ((*tb).tm_isdst < 0 && tbtemp.tm_isdst > 0) {
            elapsed_seconds = match elapsed_seconds.checked_add(T::from_i64(dstbias as i64)) {
                Some(v) => v,
                None => {
                    *_errno() = EINVAL;
                    return invalid;
                }
            };
            if T::localtime_s(&mut tbtemp, &elapsed_seconds) != 0 {
                *_errno() = EINVAL;
                return invalid;
            }
        }
    } else if T::gmtime_s(&mut tbtemp, &elapsed_seconds) != 0 {
        *_errno() = EINVAL;
        return invalid;
    }

    *tb = tbtemp;
    elapsed_seconds
}

#[no_mangle]
pub unsafe extern "C" fn _mktime32(tb: *mut Tm) -> i32 {
    common_mktime::<i32>(tb, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mktime64(tb: *mut Tm) -> i64 {
    common_mktime::<i64>(tb, true)
}

#[no_mangle]
pub unsafe extern "C" fn _mkgmtime32(tb: *mut Tm) -> i32 {
    common_mktime::<i32>(tb, false)
}

#[no_mangle]
pub unsafe extern "C" fn _mkgmtime64(tb: *mut Tm) -> i64 {
    common_mktime::<i64>(tb, false)
}
