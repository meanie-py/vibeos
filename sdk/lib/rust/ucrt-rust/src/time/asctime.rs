//! time/asctime.cpp: `asctime_s`, `_wasctime_s`, `asctime`, `_wasctime`.
//!
//! `asctime_s`/`_wasctime_s` are ALSO declared, with the exact same
//! prototypes, in `crate::ctime`'s `extern` block (it calls through to them);
//! this is that symbol's one definition.
//!
//! ## Details that are silent if wrong
//! * **The buffer is ALWAYS exactly 26 characters**, regardless of
//!   `size_in_chars` (which is only a minimum, never used to pick a shorter
//!   form) -- `Tue May  1 02:34:55 1984\n\0`.
//! * **Day-of-month is space-padded, everything else is zero-padded**:
//!   `common_asctime_s_write_value`'s `zero_fill` is `false` only for
//!   `tm_mday`.
//! * **`tm_sec` may be 60** (leap second) and is still accepted.
//! * **Validation order**: buffer/size (handler) -> reset buffer's first
//!   byte -> size>=26 (handler) -> `tm_value != null` (handler) -> tm_year>=0
//!   (handler) -> tm_mon/hour/min/sec/wday ranges (handler, one call each) ->
//!   `__crt_time_is_day_valid` (handler). ALL of these go through the
//!   handler-calling `_VALIDATE_RETURN_ERRCODE`, unlike `gmtime`/`ctime`'s mix
//!   of exc/noexc checks.
//! * **`__crt_time_is_day_valid` is checked LAST**, after the individual
//!   field range checks, so e.g. `tm_mday = 31` in April fails here, not
//!   earlier.
//! * **The per-thread/static buffer choice for the pointer-returning forms**:
//!   a lazily-allocated per-thread buffer if a `ptd` is available (persists
//!   across calls on that thread, freed only at thread teardown -- not ported
//!   here, since teardown lives elsewhere), else a function-local `static`
//!   buffer shared (and racily overwritten) by every thread that has no ptd.

use core::ffi::c_int;
use core::ptr::null_mut;

use super::Tm;

const EINVAL: c_int = 22;
const ASCBUFSIZE: usize = 26;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

use crate::ptd::AcrtPtd;
extern "C" {
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
    fn calloc(count: usize, size: usize) -> *mut core::ffi::c_void;
}

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

/// `__dnames`/`__mnames` (`timeset.rs`): three-character day/month name
/// abbreviations, packed with no separators.
fn dnames() -> &'static [u8; 22] {
    &crate::timeset::__dnames
}
fn mnames() -> &'static [u8; 37] {
    &crate::timeset::__mnames
}

trait Chr: Copy + From<u8> {
    fn digit(v: i32) -> Self;
    fn space() -> Self;
    fn colon() -> Self;
    fn newline() -> Self;
    fn nul() -> Self;
}
impl Chr for u8 {
    fn digit(v: i32) -> Self {
        b'0' + v as u8
    }
    fn space() -> Self {
        b' '
    }
    fn colon() -> Self {
        b':'
    }
    fn newline() -> Self {
        b'\n'
    }
    fn nul() -> Self {
        0
    }
}
impl Chr for u16 {
    fn digit(v: i32) -> Self {
        (b'0' + v as u8) as u16
    }
    fn space() -> Self {
        b' ' as u16
    }
    fn colon() -> Self {
        b':' as u16
    }
    fn newline() -> Self {
        b'\n' as u16
    }
    fn nul() -> Self {
        0
    }
}

/// `common_asctime_s_write_value`.
unsafe fn write_value<C: Chr>(mut p: *mut C, value: i32, zero_fill: bool) -> *mut C {
    if value >= 10 || zero_fill {
        *p = C::digit(value / 10);
    } else {
        *p = C::space();
    }
    p = p.add(1);
    *p = C::digit(value % 10);
    p.add(1)
}

/// `common_asctime_s`.
unsafe fn common_asctime_s<C: Chr>(buffer: *mut C, size_in_chars: usize, tm_value: *const Tm) -> c_int {
    if !validate(!buffer.is_null() && size_in_chars > 0, EINVAL) {
        return EINVAL;
    }

    // `_RESET_STRING`, release build.
    *buffer = C::nul();

    if !validate(size_in_chars >= ASCBUFSIZE, EINVAL) {
        return EINVAL;
    }
    if !validate(!tm_value.is_null(), EINVAL) {
        return EINVAL;
    }
    let tm = &*tm_value;
    if !validate(tm.tm_year >= 0, EINVAL) {
        return EINVAL;
    }
    if !validate(tm.tm_mon >= 0 && tm.tm_mon <= 11, EINVAL) {
        return EINVAL;
    }
    if !validate(tm.tm_hour >= 0 && tm.tm_hour <= 23, EINVAL) {
        return EINVAL;
    }
    if !validate(tm.tm_min >= 0 && tm.tm_min <= 59, EINVAL) {
        return EINVAL;
    }
    if !validate(tm.tm_sec >= 0 && tm.tm_sec <= 60, EINVAL) {
        return EINVAL;
    }
    if !validate(tm.tm_wday >= 0 && tm.tm_wday <= 6, EINVAL) {
        return EINVAL;
    }
    if !validate(super::is_day_valid(tm.tm_year, tm.tm_mon, tm.tm_mday), EINVAL) {
        return EINVAL;
    }

    let mut it = buffer;

    let dn = dnames();
    let day_first = (tm.tm_wday * 3) as usize;
    for &b in &dn[day_first..day_first + 3] {
        *it = C::from(b);
        it = it.add(1);
    }
    *it = C::space();
    it = it.add(1);

    let mn = mnames();
    let mon_first = (tm.tm_mon * 3) as usize;
    for &b in &mn[mon_first..mon_first + 3] {
        *it = C::from(b);
        it = it.add(1);
    }
    *it = C::space();
    it = it.add(1);

    it = write_value(it, tm.tm_mday, false);
    *it = C::space();
    it = it.add(1);

    it = write_value(it, tm.tm_hour, true);
    *it = C::colon();
    it = it.add(1);
    it = write_value(it, tm.tm_min, true);
    *it = C::colon();
    it = it.add(1);
    it = write_value(it, tm.tm_sec, true);
    *it = C::space();
    it = it.add(1);

    it = write_value(it, super::get_century(tm.tm_year), true);
    it = write_value(it, super::get_2digit_year(tm.tm_year), true);

    *it = C::newline();
    it = it.add(1);
    *it = C::nul();

    0
}

#[no_mangle]
pub unsafe extern "C" fn asctime_s(buffer: *mut u8, size_in_chars: usize, tm_value: *const Tm) -> c_int {
    common_asctime_s(buffer, size_in_chars, tm_value)
}

#[no_mangle]
pub unsafe extern "C" fn _wasctime_s(buffer: *mut u16, size_in_chars: usize, tm_value: *const Tm) -> c_int {
    common_asctime_s(buffer, size_in_chars, tm_value)
}

trait PtdBuffer: Sized + Copy {
    unsafe fn ptd_slot(ptd: *mut AcrtPtd) -> *mut *mut Self;
}
impl PtdBuffer for u8 {
    unsafe fn ptd_slot(ptd: *mut AcrtPtd) -> *mut *mut u8 {
        core::ptr::addr_of_mut!((*ptd).asctime_buffer)
    }
}
impl PtdBuffer for u16 {
    unsafe fn ptd_slot(ptd: *mut AcrtPtd) -> *mut *mut u16 {
        core::ptr::addr_of_mut!((*ptd).wasctime_buffer)
    }
}

/// `common_asctime`. The C's function-local `static Character static_buffer[26]`
/// fallback (used when no ptd is available) is one shared buffer PER
/// INSTANTIATION (one for `char`, one for `wchar_t`), matching the C's own
/// template instantiation having one such static each.
unsafe fn common_asctime<C: Chr + PtdBuffer>(tm_value: *const Tm, static_buffer: *mut C) -> *mut C {
    let ptd = __acrt_getptd_noexit();
    let buffer = if !ptd.is_null() {
        let slot = C::ptd_slot(ptd);
        if (*slot).is_null() {
            *slot = calloc(ASCBUFSIZE, core::mem::size_of::<C>()) as *mut C;
        }
        if !(*slot).is_null() {
            *slot
        } else {
            static_buffer
        }
    } else {
        static_buffer
    };

    if common_asctime_s(buffer, ASCBUFSIZE, tm_value) != 0 {
        return null_mut();
    }
    buffer
}

#[no_mangle]
pub unsafe extern "C" fn asctime(tm_value: *const Tm) -> *mut u8 {
    static mut STATIC_BUFFER: [u8; ASCBUFSIZE] = [0; ASCBUFSIZE];
    common_asctime(tm_value, (&raw mut STATIC_BUFFER) as *mut _)
}

#[no_mangle]
pub unsafe extern "C" fn _wasctime(tm_value: *const Tm) -> *mut u16 {
    static mut STATIC_BUFFER: [u16; ASCBUFSIZE] = [0; ASCBUFSIZE];
    common_asctime(tm_value, (&raw mut STATIC_BUFFER) as *mut _)
}
