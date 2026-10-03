//! time/ftime.cpp: `_ftime32_s`, `_ftime32`, `_ftime64_s`, `_ftime64`.
//!
//! ## Details that are silent if wrong
//! * **The DST flag is CACHED, re-derived only once per elapsed minute**
//!   (`elapsed_minutes_cache`), not on every call -- ported as two
//!   process-wide `static mut`s, matching the C's own un-synchronized
//!   globals (no lock in the C either).
//! * **`GetTimeZoneInformation` failing (`0xFFFFFFFF`) sets the cache to
//!   `UNKNOWN_TIME` (-1)**, not "keep the previous value".
//! * **DST is considered active only when `tz_state ==
//!   TIME_ZONE_ID_DAYLIGHT` AND `DaylightDate.wMonth != 0` AND
//!   `DaylightBias != 0`** -- `TIME_ZONE_ID_UNKNOWN` and
//!   `TIME_ZONE_ID_STANDARD` both fall through to "assume Standard Time",
//!   same as a `DaylightDate.wMonth == 0` zone that happens to report the
//!   DAYLIGHT id.
//! * **`tp->time` truncates towards negative infinity via integer division**
//!   on `(scalar - _EPOCH_BIAS) / 10_000_000`, matching C++'s truncating `/`
//!   on a (post-1970, so non-negative) dividend.

use core::ffi::c_int;

const EINVAL: c_int = 22;

const DAYLIGHT_TIME: c_int = 1;
const STANDARD_TIME: c_int = 0;
const UNKNOWN_TIME: c_int = -1;

/// `_EPOCH_BIAS`, same value as `time_get.rs`.
const EPOCH_BIAS: i64 = 116_444_736_000_000_000;

const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn __tzset();
    fn _get_timezone(result: *mut core::ffi::c_long) -> c_int;
}

extern "system" {
    fn __acrt_GetSystemTimePreciseAsFileTime(ft: *mut u64);
    fn GetTimeZoneInformation(info: *mut TimeZoneInformation) -> u32;
}

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

/// `TIME_ZONE_INFORMATION` (`timezoneapi.h`): only the fields this file (and
/// `tzset.rs`) touch are named individually; `StandardName`/`DaylightName`
/// (32 wide chars each) are opaque byte arrays here, read only by `tzset.rs`
/// (which owns the shared, fully-laid-out copy) -- this file only needs
/// `DaylightDate.wMonth`/`DaylightBias`, both near the front, so the layout
/// is pinned generically via `super::tzset::TimeZoneInformation` instead of
/// being duplicated. See `tzset.rs` for the measured layout.
pub use super::tzset::TimeZoneInformation;

static mut ELAPSED_MINUTES_CACHE: i64 = 0;
static mut DSTFLAG_CACHE: c_int = UNKNOWN_TIME;

/// `__timeb32`/`__timeb64` (`sys/timeb.h`).
#[repr(C)]
pub struct Timeb32 {
    pub time: i32,
    pub millitm: u16,
    pub timezone: i16,
    pub dstflag: i16,
}
#[repr(C)]
pub struct Timeb64 {
    pub time: i64,
    pub millitm: u16,
    pub timezone: i16,
    pub dstflag: i16,
}

trait TimeBuf {
    unsafe fn set(&mut self, time: i64, millitm: u16, timezone: i16, dstflag: i16);
}
impl TimeBuf for Timeb32 {
    unsafe fn set(&mut self, time: i64, millitm: u16, timezone: i16, dstflag: i16) {
        self.time = time as i32;
        self.millitm = millitm;
        self.timezone = timezone;
        self.dstflag = dstflag;
    }
}
impl TimeBuf for Timeb64 {
    unsafe fn set(&mut self, time: i64, millitm: u16, timezone: i16, dstflag: i16) {
        self.time = time;
        self.millitm = millitm;
        self.timezone = timezone;
        self.dstflag = dstflag;
    }
}

unsafe fn common_ftime_s<T: TimeBuf>(tp: *mut T) -> c_int {
    if !validate(!tp.is_null(), EINVAL) {
        return EINVAL;
    }

    __tzset();

    let mut timezone: core::ffi::c_long = 0;
    _get_timezone(&mut timezone);

    let mut filetime: u64 = 0;
    __acrt_GetSystemTimePreciseAsFileTime(&mut filetime);
    let scalar = filetime as i64;

    let current_minutes_value = scalar / 600_000_000;
    if current_minutes_value != ELAPSED_MINUTES_CACHE {
        let mut tz_info: TimeZoneInformation = core::mem::zeroed();
        let tz_state = GetTimeZoneInformation(&mut tz_info);
        if tz_state == 0xFFFF_FFFF {
            DSTFLAG_CACHE = UNKNOWN_TIME;
        } else if tz_state == TIME_ZONE_ID_DAYLIGHT
            && tz_info.daylight_date.w_month != 0
            && tz_info.daylight_bias != 0
        {
            DSTFLAG_CACHE = DAYLIGHT_TIME;
        } else {
            DSTFLAG_CACHE = STANDARD_TIME;
        }
        ELAPSED_MINUTES_CACHE = current_minutes_value;
    }

    let millitm = ((scalar / 10_000) % 1000) as u16;
    let time = (scalar - EPOCH_BIAS) / 10_000_000;

    (*tp).set(time, millitm, (timezone / 60) as i16, DSTFLAG_CACHE as i16);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _ftime32_s(tp: *mut Timeb32) -> c_int {
    common_ftime_s(tp)
}
#[no_mangle]
pub unsafe extern "C" fn _ftime32(tp: *mut Timeb32) {
    _ftime32_s(tp);
}
#[no_mangle]
pub unsafe extern "C" fn _ftime64_s(tp: *mut Timeb64) -> c_int {
    common_ftime_s(tp)
}
#[no_mangle]
pub unsafe extern "C" fn _ftime64(tp: *mut Timeb64) {
    _ftime64_s(tp);
}
