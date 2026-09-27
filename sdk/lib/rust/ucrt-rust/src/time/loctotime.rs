//! time/loctotime.cpp: `__loctotime32_t`, `__loctotime64_t`. Internal-only
//! (no export), but depended on by EXACT NAME from `crate::findfile`'s
//! `CrtTime` impls and (via `_days`-adjacent logic) `crate::stat` -- both
//! outside this batch, both already calling through an `extern "C"` block
//! declaring these two prototypes; this file's signatures must match those
//! exactly (checked: `findfile.rs`'s `fn __loctotime32_t(yr: c_int, mo:
//! c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dst: c_int) -> i32`).
//!
//! ## Details that are silent if wrong
//! * **`yr` is adjusted by `-= 1900` FIRST**, then validated against
//!   `_BASE_YEAR..=max_year` -- so the function's own `yr` parameter is the
//!   ABSOLUTE year (e.g. 2024), unlike `tm_year` elsewhere in this module,
//!   which is already years-since-1900.
//! * **`dstflag == 1` returns unconditionally DST-adjusted**, without ever
//!   calling `_isindst`; `dstflag == -1` calls `_isindst` only if `daylight
//!   != 0`; any other `dstflag` value (including `0`) returns the
//!   NON-DST-adjusted time -- there is no explicit `dstflag == 0` branch,
//!   it falls out of the same `if` that handles "unspecified but not DST"
//!   for `-1`.
//! * **Leap-day handling**: `elapsed_days_this_year` gets `+1` when the year
//!   is a leap year AND `mo > 2` -- reusing `_days` (not `_lpdays`) as the
//!   base table and patching the leap day in by hand, rather than switching
//!   tables as `gmtime.rs` does.

use core::ffi::c_int;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn _get_daylight(result: *mut c_int) -> c_int;
    fn _get_dstbias(result: *mut core::ffi::c_long) -> c_int;
    fn _get_timezone(result: *mut core::ffi::c_long) -> c_int;
}

// `__tzset`/`_isindst` are this same batch's own (`tzset.rs`) -- called
// directly rather than re-declared `extern`.
use super::tzset::{__tzset, _isindst};

unsafe fn validate_noexc(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
    }
    ok
}

trait LocSrc: Copy {
    const MAX_YEAR: i32;
    fn from_i64(v: i64) -> Self;
    fn invalid() -> Self;
}
impl LocSrc for i32 {
    const MAX_YEAR: i32 = super::MAX_YEAR_32;
    fn from_i64(v: i64) -> Self {
        v as i32
    }
    fn invalid() -> Self {
        -1
    }
}
impl LocSrc for i64 {
    const MAX_YEAR: i32 = super::MAX_YEAR_64;
    fn from_i64(v: i64) -> Self {
        v
    }
    fn invalid() -> Self {
        -1
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn common_loctotime_t<T: LocSrc>(mut yr: i32, mo: i32, dy: i32, hr: i32, mn: i32, sc: i32, dstflag: i32) -> T {
    yr -= 1900;

    if !validate_noexc(yr >= super::BASE_YEAR && yr <= T::MAX_YEAR, EINVAL) {
        return T::invalid();
    }
    if !validate_noexc(mo >= 1 && mo <= 12, EINVAL) {
        return T::invalid();
    }
    if !validate_noexc(super::is_day_valid(yr, mo - 1, dy), EINVAL) {
        return T::invalid();
    }
    if !validate_noexc(hr >= 0 && hr <= 23, EINVAL) {
        return T::invalid();
    }
    if !validate_noexc(mn >= 0 && mn <= 59, EINVAL) {
        return T::invalid();
    }
    if !validate_noexc(sc >= 0 && sc <= 59, EINVAL) {
        return T::invalid();
    }

    let mut elapsed_days_this_year = dy + super::days::_days[(mo - 1) as usize];
    if super::is_leap_year(yr) && mo > 2 {
        elapsed_days_this_year += 1;
    }

    let elapsed_years = (yr - super::BASE_YEAR) as i64;
    let elapsed_leap_years = super::elapsed_leap_years(yr) as i64;

    let elapsed_days = elapsed_years * 365 + elapsed_leap_years + elapsed_days_this_year as i64;
    let elapsed_hours = elapsed_days * 24 + hr as i64;
    let elapsed_minutes = elapsed_hours * 60 + mn as i64;
    let elapsed_seconds = elapsed_minutes * 60 + sc as i64;

    __tzset();

    let mut daylight: c_int = 0;
    let mut dstbias: core::ffi::c_long = 0;
    let mut timezone: core::ffi::c_long = 0;
    _get_daylight(&mut daylight);
    _get_dstbias(&mut dstbias);
    _get_timezone(&mut timezone);

    let timezone_adjusted_seconds = elapsed_seconds + timezone as i64;
    let dst_adjusted_seconds = timezone_adjusted_seconds + dstbias as i64;

    if dstflag == 1 {
        return T::from_i64(dst_adjusted_seconds);
    }

    let mut tm_value = super::Tm::default();
    tm_value.tm_yday = elapsed_days_this_year;
    tm_value.tm_year = yr;
    tm_value.tm_mon = mo - 1;
    tm_value.tm_hour = hr;
    tm_value.tm_min = mn;
    tm_value.tm_sec = sc;

    if dstflag == -1 && daylight != 0 && _isindst(&mut tm_value) != 0 {
        return T::from_i64(dst_adjusted_seconds);
    }

    T::from_i64(timezone_adjusted_seconds)
}

#[no_mangle]
pub unsafe extern "C" fn __loctotime32_t(yr: c_int, mo: c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dstflag: c_int) -> i32 {
    common_loctotime_t::<i32>(yr, mo, dy, hr, mn, sc, dstflag)
}

#[no_mangle]
pub unsafe extern "C" fn __loctotime64_t(yr: c_int, mo: c_int, dy: c_int, hr: c_int, mn: c_int, sc: c_int, dstflag: c_int) -> i64 {
    common_loctotime_t::<i64>(yr, mo, dy, hr, mn, sc, dstflag)
}
