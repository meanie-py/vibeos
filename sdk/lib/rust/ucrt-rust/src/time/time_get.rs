//! time/time.cpp: `_timespec32_get`, `_timespec64_get`, `_time32`, `_time64`.
//! (File renamed `time_get.rs` to avoid colliding with the `time` module
//! itself; the exported symbol names are unaffected.)
//!
//! ## Details that are silent if wrong
//! * **`base != TIME_UTC` returns `0` without touching `*ts`** -- not an
//!   error, just "unsupported base", and the caller's buffer is left alone.
//! * **The FILETIME-to-`time_t` conversion truncates toward negative
//!   infinity in units, not zero**: `epoch_time / filetime_scale` and `%` are
//!   plain C++ integer div/mod on a (usually positive, post-1601) value, so
//!   no special-casing is needed to match it with Rust's `/`/`%` (also
//!   truncating), but this stops being true if `epoch_time` is ever negative
//!   (pre-1601 FILETIME) -- not reachable from `GetSystemTimeAsFileTime`.
//! * **`_time32`/`_time64` fall back to `(TimeType)(-1)` when the underlying
//!   `timespec_get` call did not return `TIME_UTC`**, then still write that
//!   `-1` through `result` if non-null and return it -- never propagates an
//!   error code, only the sentinel value.
//! * **`_time32`/`_time64` intentionally use the LESS precise
//!   `GetSystemTimeAsFileTime`** (not …`Precise`…), for speed; `timespec_get`
//!   itself always uses the precise API. Kept as two distinct Win32 imports.

use core::ffi::c_int;

/// `_EPOCH_BIAS` (`corecrt_internal_time.h`): 100ns units between the
/// FILETIME epoch (1601-01-01) and the Unix epoch (1970-01-01).
const EPOCH_BIAS: i64 = 116_444_736_000_000_000;
/// `TIME_UTC` (`time.h`).
const TIME_UTC: c_int = 1;

const EINVAL: c_int = 22;

extern "system" {
    fn GetSystemTimeAsFileTime(ft: *mut u64);
    fn __acrt_GetSystemTimePreciseAsFileTime(ft: *mut u64);
}

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

/// `_timespec32`/`_timespec64` (`corecrt_internal_time.h`): `time_t tv_sec;
/// long tv_nsec;`.
#[repr(C)]
pub struct Timespec32 {
    pub tv_sec: i32,
    pub tv_nsec: i32,
}
#[repr(C)]
pub struct Timespec64 {
    pub tv_sec: i64,
    pub tv_nsec: i32,
}

trait TimeType: Copy + PartialOrd {
    const MAX_TIME_T_TS: i64;
    fn from_i64(v: i64) -> Self;
    fn neg_one() -> Self;
}
impl TimeType for i32 {
    const MAX_TIME_T_TS: i64 = i32::MAX as i64;
    fn from_i64(v: i64) -> Self {
        v as i32
    }
    fn neg_one() -> Self {
        -1
    }
}
impl TimeType for i64 {
    // `time_traits::max_time_t` for `__time64_t`: `_MAX__TIME64_T`
    // (3000-12-31 23:59:59 UTC), not `i64::MAX` -- but that bound belongs to
    // `localtime`/`gmtime`, not to this file's own conversion, which never
    // range-checks `seconds` against anything narrower than the destination
    // type's own width (the C's `if (seconds > time_traits::max_time_t)`
    // check IS present for both instantiations, so it is modelled here too).
    const MAX_TIME_T_TS: i64 = 32_535_215_999; // 3000-12-31T23:59:59Z - epoch, seconds
    fn from_i64(v: i64) -> Self {
        v
    }
    fn neg_one() -> Self {
        -1
    }
}

/// `common_timespec_get<GetSystemTimeFunction, TimeSpecType>`.
unsafe fn common_timespec_get_sec_nsec(get: unsafe extern "system" fn(*mut u64), base: c_int) -> Option<(i64, i32)> {
    if base != TIME_UTC {
        return None;
    }
    let mut filetime: u64 = 0;
    get(&mut filetime);

    let epoch_time = filetime as i64 - EPOCH_BIAS;
    let filetime_scale: i64 = 10 * 1000 * 1000;
    let seconds = epoch_time / filetime_scale;
    let nanoseconds = (epoch_time % filetime_scale) * 100;
    Some((seconds, nanoseconds as i32))
}

#[no_mangle]
pub unsafe extern "C" fn _timespec32_get(ts: *mut Timespec32, base: c_int) -> c_int {
    if !validate(!ts.is_null(), EINVAL) {
        return 0;
    }
    match common_timespec_get_sec_nsec(__acrt_GetSystemTimePreciseAsFileTime, base) {
        None => 0,
        Some((seconds, _)) if seconds > i32::MAX as i64 => 0,
        Some((seconds, nsec)) => {
            (*ts).tv_sec = seconds as i32;
            (*ts).tv_nsec = nsec;
            base
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _timespec64_get(ts: *mut Timespec64, base: c_int) -> c_int {
    if !validate(!ts.is_null(), EINVAL) {
        return 0;
    }
    match common_timespec_get_sec_nsec(__acrt_GetSystemTimePreciseAsFileTime, base) {
        None => 0,
        Some((seconds, _)) if seconds > <i64 as TimeType>::MAX_TIME_T_TS => 0,
        Some((seconds, nsec)) => {
            (*ts).tv_sec = seconds;
            (*ts).tv_nsec = nsec;
            base
        }
    }
}

/// `common_time<TimeType>`, using the LESS-precise `GetSystemTimeAsFileTime`
/// (see module doc).
unsafe fn common_time<T: TimeType>(result: *mut T, max: i64) -> T {
    let sec = match common_timespec_get_sec_nsec(GetSystemTimeAsFileTime, TIME_UTC) {
        Some((seconds, _)) if seconds <= max => T::from_i64(seconds),
        _ => T::neg_one(),
    };
    if !result.is_null() {
        *result = sec;
    }
    sec
}

#[no_mangle]
pub unsafe extern "C" fn _time32(result: *mut i32) -> i32 {
    common_time::<i32>(result, i32::MAX as i64)
}

#[no_mangle]
pub unsafe extern "C" fn _time64(result: *mut i64) -> i64 {
    common_time::<i64>(result, <i64 as TimeType>::MAX_TIME_T_TS)
}
