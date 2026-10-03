//! time/ctime.cpp: `_ctime32`, `_ctime32_s`, `_ctime64`, `_ctime64_s`,
//! `_wctime32`, `_wctime32_s`, `_wctime64`, `_wctime64_s`.
//!
//! All eight are one C++ template (`common_ctime_s<TimeType, Character>` /
//! `common_ctime<TimeType, Character>`) instantiated four ways; ported here as
//! two generic functions over the `TimeValue` (`__time32_t`/`__time64_t`) and
//! `Chr` (`char`/`wchar_t`) axes. Neither `localtime_s` (`_localtime32_s` /
//! `_localtime64_s`) nor the `asctime`/`_wasctime` family is ported -- both stay
//! C, called through by exact name, and `asctime`/`_wasctime` are where the
//! per-thread `asctime_buffer`/`wasctime_buffer` in `__acrt_ptd` (ptd.rs)
//! actually lives; nothing here touches the ptd directly.
//!
//! ## Details that are silent if wrong
//!
//! * **`_RESET_STRING` is release-build no-op except for the leading NUL.** Its
//!   `_FILL_STRING` half is `#ifdef _DEBUG` only, so in a release UCRT the "reset"
//!   is just `buffer[0] = 0` before the length check -- a caller who passes
//!   `size_in_chars < 26` still gets an empty string in `buffer[0]`, not an
//!   unrifled one, as long as `buffer` itself was non-null. Modelled with a
//!   single `write_bytes(buffer, 0, 1)`, which zeroes exactly one `Chr` unit
//!   (1 byte narrow, 2 wide) as the C's `*(_String) = 0` does for its element
//!   type.
//! * **Validation order matters and mixes handler-calling and silent checks**:
//!   `buffer!=null && size>0` and `size>=26` and `time_t_value!=null` all invoke
//!   the invalid-parameter handler (`_VALIDATE_RETURN_ERRCODE`); the final
//!   `*time_t_value >= 0` does NOT (`_VALIDATE_RETURN_ERRCODE_NOEXC`) -- so a
//!   negative time_t is the one case that sets `errno=EINVAL` and returns
//!   without ever invoking the handler. The buffer is reset (step 2) BEFORE the
//!   `size_in_chars >= 26` check (step 3), so a too-small buffer still comes
//!   back empty rather than untouched.
//! * **The pointer-returning forms skip the buffer/size checks entirely**
//!   (there is no buffer) and validate only `time_t_value != nullptr` (handler)
//!   then `*time_t_value >= 0` (no handler, same as above).
//! * **A `localtime_s` failure propagates its own status code untranslated**;
//!   `common_ctime` (pointer form) instead collapses any failure to `nullptr`.
//! * **Negative `time_t` is invalid here even though `_localtime32_s`/
//!   `_localtime64_s` might otherwise handle it** -- the check happens in
//!   `ctime.cpp` before `localtime_s` is ever called.

use core::ffi::c_int;
use core::ptr::null_mut;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();

    fn _localtime32_s(tm: *mut Tm, time: *const i32) -> c_int;
    fn _localtime64_s(tm: *mut Tm, time: *const i64) -> c_int;

    fn asctime_s(buffer: *mut u8, size_in_chars: usize, tm: *const Tm) -> c_int;
    fn _wasctime_s(buffer: *mut u16, size_in_chars: usize, tm: *const Tm) -> c_int;
    fn asctime(tm: *const Tm) -> *mut u8;
    fn _wasctime(tm: *const Tm) -> *mut u16;
}

/// `struct tm` (`corecrt_wtime.h`): nine plain `int`s, no padding.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Tm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
}

/// `_VALIDATE_RETURN_ERRCODE(expr, err)`: on failure, errno=err, invoke the
/// (release, no-info) handler, and report failure.
unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

/// `_VALIDATE_RETURN_ERRCODE_NOEXC` / `_VALIDATE_RETURN_NOEXC`: errno only, the
/// handler is never invoked.
unsafe fn validate_noexc(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
    }
    ok
}

/// `__crt_time_time_t_traits<TimeType>::localtime_s`: `_localtime32_s` for
/// `__time32_t` (`i32`), `_localtime64_s` for `__time64_t` (`i64`).
trait TimeValue: Copy + PartialOrd + Default {
    unsafe fn localtime_s(tm: *mut Tm, time: *const Self) -> c_int;
}

impl TimeValue for i32 {
    unsafe fn localtime_s(tm: *mut Tm, time: *const i32) -> c_int {
        _localtime32_s(tm, time)
    }
}

impl TimeValue for i64 {
    unsafe fn localtime_s(tm: *mut Tm, time: *const i64) -> c_int {
        _localtime64_s(tm, time)
    }
}

/// `__crt_char_traits<Character>::tasctime`/`tasctime_s`: `asctime`/`asctime_s`
/// for `char`, `_wasctime`/`_wasctime_s` for `wchar_t`.
trait Chr: Copy {
    unsafe fn tasctime_s(buffer: *mut Self, size_in_chars: usize, tm: *const Tm) -> c_int;
    unsafe fn tasctime(tm: *const Tm) -> *mut Self;
}

impl Chr for u8 {
    unsafe fn tasctime_s(buffer: *mut u8, size_in_chars: usize, tm: *const Tm) -> c_int {
        asctime_s(buffer, size_in_chars, tm)
    }
    unsafe fn tasctime(tm: *const Tm) -> *mut u8 {
        asctime(tm)
    }
}

impl Chr for u16 {
    unsafe fn tasctime_s(buffer: *mut u16, size_in_chars: usize, tm: *const Tm) -> c_int {
        _wasctime_s(buffer, size_in_chars, tm)
    }
    unsafe fn tasctime(tm: *const Tm) -> *mut u16 {
        _wasctime(tm)
    }
}

/// `common_ctime_s<TimeType, Character>`.
///
/// # Safety
/// `buffer`, if non-null, must be writable for `size_in_chars` elements of `C`;
/// `time_t_value`, if non-null, must point to a valid `T`.
unsafe fn common_ctime_s<T: TimeValue, C: Chr>(buffer: *mut C, size_in_chars: usize, time_t_value: *const T) -> c_int {
    if !validate(!buffer.is_null() && size_in_chars > 0, EINVAL) {
        return EINVAL;
    }

    // `_RESET_STRING`, release build: `*(_String) = 0` only.
    core::ptr::write_bytes(buffer, 0, 1);

    if !validate(size_in_chars >= 26, EINVAL) {
        return EINVAL;
    }
    if !validate(!time_t_value.is_null(), EINVAL) {
        return EINVAL;
    }
    if !validate_noexc(*time_t_value >= T::default(), EINVAL) {
        return EINVAL;
    }

    let mut tm_value = Tm::default();
    let status = T::localtime_s(&mut tm_value, time_t_value);
    if status != 0 {
        return status;
    }

    C::tasctime_s(buffer, size_in_chars, &tm_value)
}

/// `common_ctime<TimeType, Character>`.
///
/// # Safety
/// `time_t_value`, if non-null, must point to a valid `T`.
unsafe fn common_ctime<T: TimeValue, C: Chr>(time_t_value: *const T) -> *mut C {
    if !validate(!time_t_value.is_null(), EINVAL) {
        return null_mut();
    }
    if !validate_noexc(*time_t_value >= T::default(), EINVAL) {
        return null_mut();
    }

    let mut tm_value = Tm::default();
    let status = T::localtime_s(&mut tm_value, time_t_value);
    if status != 0 {
        return null_mut();
    }

    C::tasctime(&tm_value)
}

#[no_mangle]
pub unsafe extern "C" fn _ctime32_s(buffer: *mut u8, size_in_chars: usize, time_t_value: *const i32) -> c_int {
    common_ctime_s::<i32, u8>(buffer, size_in_chars, time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _wctime32_s(buffer: *mut u16, size_in_chars: usize, time_t_value: *const i32) -> c_int {
    common_ctime_s::<i32, u16>(buffer, size_in_chars, time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _ctime64_s(buffer: *mut u8, size_in_chars: usize, time_t_value: *const i64) -> c_int {
    common_ctime_s::<i64, u8>(buffer, size_in_chars, time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _wctime64_s(buffer: *mut u16, size_in_chars: usize, time_t_value: *const i64) -> c_int {
    common_ctime_s::<i64, u16>(buffer, size_in_chars, time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _ctime32(time_t_value: *const i32) -> *mut u8 {
    common_ctime::<i32, u8>(time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _wctime32(time_t_value: *const i32) -> *mut u16 {
    common_ctime::<i32, u16>(time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _ctime64(time_t_value: *const i64) -> *mut u8 {
    common_ctime::<i64, u8>(time_t_value)
}

#[no_mangle]
pub unsafe extern "C" fn _wctime64(time_t_value: *const i64) -> *mut u16 {
    common_ctime::<i64, u16>(time_t_value)
}
