//! `sdk/lib/ucrt/time/*.cpp` -- tzset, localtime/gmtime/mktime,
//! asctime, utime/ftime/strdate/strtime/time/clock/difftime, and the two
//! internal-only files `days.cpp`/`loctotime.cpp`.
//!
//! `strftime.cpp`/`wcsftime.cpp` are NOT YET PORTED (see the batch report);
//! their C stays in the build.
//!
//! Shared across this directory: [`Tm`] (the C `struct tm`, `corecrt_wtime.h`
//! -- nine plain `int`s, no padding, identical to the private `Tm` that
//! `crate::ctime` declares its own copy of purely for its own `extern`
//! prototypes: two nominally-distinct Rust types over one C layout is not a
//! `clashing_extern_declarations` hazard because neither module's `extern`
//! block names the other's type).

use core::ffi::c_int;

/// `struct tm` (`corecrt_wtime.h`).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
}

// ── shared constants and math (`corecrt_internal_time.h`, header-inline in
//    the C -- not exported symbols, so ported once here rather than per file) ──

/// `_DAY_SEC`.
pub(crate) const DAY_SEC: i64 = 24 * 60 * 60;
/// `_YEAR_SEC`.
pub(crate) const YEAR_SEC: i64 = 365 * DAY_SEC;
/// `_FOUR_YEAR_SEC`.
pub(crate) const FOUR_YEAR_SEC: i64 = 1461 * DAY_SEC;
/// `_BASE_YEAR`: the epoch year (1970), as years-since-1900.
pub(crate) const BASE_YEAR: i32 = 70;
/// `_BASE_DOW`: day of week of 1970-01-01 (Thursday).
pub(crate) const BASE_DOW: i32 = 4;
/// `_MAX_LOCAL_TIME` / `_MIN_LOCAL_TIME`.
pub(crate) const MAX_LOCAL_TIME: i64 = 14 * 60 * 60;
pub(crate) const MIN_LOCAL_TIME: i64 = -12 * 60 * 60;

/// `__crt_time_time_t_traits<__time32_t>::max_time_t` /
/// `::max_year`, `<__time64_t>::max_time_t` / `::max_year`.
pub(crate) const MAX_TIME_T_32: i64 = 0x7fff_d27f;
pub(crate) const MAX_YEAR_32: i32 = 138;
pub(crate) const MAX_TIME_T_64: i64 = 0x793_582a_ff;
pub(crate) const MAX_YEAR_64: i32 = 1101;

/// `__crt_time_is_leap_year<TimeType>`, generic over `i32`/`i64` via a small
/// widen-to-`i64`-and-back trait (the C template is instantiated at both
/// widths across `tzset.cpp`/`gmtime.cpp`/`mktime.cpp`).
pub(crate) trait TimeMathInt: Copy {
    fn to_i64(self) -> i64;
    fn from_i64(v: i64) -> Self;
}
impl TimeMathInt for i32 {
    fn to_i64(self) -> i64 {
        self as i64
    }
    fn from_i64(v: i64) -> Self {
        v as i32
    }
}
impl TimeMathInt for i64 {
    fn to_i64(self) -> i64 {
        self
    }
    fn from_i64(v: i64) -> Self {
        v
    }
}

/// `__crt_time_is_leap_year`. `yr` is years-since-1900, not the absolute year.
pub(crate) fn is_leap_year<T: TimeMathInt>(yr: T) -> bool {
    let yr = yr.to_i64();
    if yr % 4 == 0 && yr % 100 != 0 {
        return true;
    }
    (yr + 1900) % 400 == 0
}

/// `__crt_time_elapsed_leap_years`: leap years elapsed from 1970 up to, but
/// not including, `yr` (years-since-1900).
pub(crate) fn elapsed_leap_years<T: TimeMathInt>(yr: T) -> T {
    let yr = yr.to_i64();
    const LEAP_YEARS_BETWEEN_1900_AND_1970: i64 = 17;
    let elapsed = (yr - 1) / 4 - (yr - 1) / 100 + (yr + 299) / 400;
    T::from_i64(elapsed - LEAP_YEARS_BETWEEN_1900_AND_1970)
}

/// `__crt_time_is_day_valid`. `mo` is ZERO-based here (as the C's own
/// callers pass it: `mo - 1` from a one-based month, or `tb->tm_mon`
/// directly).
pub(crate) fn is_day_valid(yr: i32, mo: i32, dy: i32) -> bool {
    if dy <= 0 {
        return false;
    }
    let days_in_month = days::_days[(mo + 1) as usize] - days::_days[mo as usize];
    if dy <= days_in_month {
        return true;
    }
    is_leap_year(yr) && mo == 1 && dy <= 29
}

/// `__crt_get_2digit_year`.
pub(crate) fn get_2digit_year(year: i32) -> i32 {
    (1900 + year) % 100
}

/// `__crt_get_century`.
pub(crate) fn get_century(year: i32) -> i32 {
    (1900 + year) / 100
}

/// `SYSTEMTIME`, the "local" flavour returned by `GetLocalTime` -- shared by
/// `strdate.rs`/`strtime.rs` (both call `GetLocalTime` for one piece of it
/// each) so the extern declaration exists exactly once (two per-module
/// copies of an identical-layout-but-nominally-distinct struct is exactly
/// the ambiguity `#![deny(clashing_extern_declarations)]` exists to catch).
#[repr(C)]
pub(crate) struct SystemTimeLocal {
    pub w_year: u16,
    pub w_month: u16,
    pub w_day_of_week: u16,
    pub w_day: u16,
    pub w_hour: u16,
    pub w_minute: u16,
    pub w_second: u16,
    pub w_milliseconds: u16,
}

extern "system" {
    pub(crate) fn GetLocalTime(st: *mut SystemTimeLocal);
}

pub mod asctime;
pub mod clock;
pub mod days;
pub mod difftime;
pub mod ftime;
pub mod gmtime;
pub mod loctotime;
pub mod localtime;
pub mod mktime;
pub mod strdate;
pub mod strftime;
pub mod strtime;
pub mod time_get;
pub mod tzset;
pub mod utime;
pub mod wcsftime;
