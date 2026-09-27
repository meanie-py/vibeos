//! time/strftime.cpp: `_Getdays_l`/`_Getdays`, `_Getmonths_l`/`_Getmonths`,
//! `_Gettnames`, and the narrow `_Strftime_l`/`_Strftime`/`_strftime_l`/
//! `strftime` family -- all of which convert through the wide engine in
//! `wcsftime.rs` (time/wcsftime.cpp).
//!
//! ## Details that are silent if wrong
//! * **`_Strftime_l` converts the FORMAT string mbs->wcs, calls
//!   `_Wcsftime_l`, then converts the RESULT wcs->mbs** -- three
//!   conversions/calls, each of which can fail independently, and each
//!   failure returns `0` with no further diagnostic distinguishing which
//!   step failed (matching the C, which discards `cvt1`/`cvt2`'s specific
//!   codes too, beyond "nonzero").
//! * **The intermediate wide buffer is sized `maxsize` wide characters**,
//!   the SAME numeric size as the narrow output buffer request, even though
//!   a wide encoding of the same text is never longer in element COUNT than
//!   its narrow multibyte form for any codepage this build supports -- not
//!   a tight bound in bytes, but never too small in elements for this
//!   reason.
//! * **The final narrow conversion writes directly into the CALLER's
//!   buffer with a FIXED size (`maxsize`), never growing** -- a converted
//!   result that does not fit is a hard failure (`cvt2 != 0`), not a
//!   partial write.
//! * **`_Getdays_l`/`_Getmonths_l` join each name with a LEADING `:` per
//!   entry, abbreviated then full, no trailing separator before the final
//!   NUL** -- `:Sun:Sunday:Mon:Monday:...:Sat:Saturday\0`, the format
//!   several legacy callers of `_Getdays`/`_Getmonths` still parse.

use core::ffi::{c_char, c_int, c_void};

use super::Tm;
use super::wcsftime::_Wcsftime_l;
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;

const EINVAL: c_int = 22;

type LocaleT = *mut CrtLocalePointers;

const MB_PRECOMPOSED: u32 = 1;
const MB_ERR_INVALID_CHARS: u32 = 8;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn malloc(size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn __acrt_MultiByteToWideChar(cp: u32, flags: u32, src: *const c_char, src_len: c_int, dst: *mut u16, dst_len: c_int) -> c_int;
    fn __acrt_WideCharToMultiByte(
        cp: u32,
        flags: u32,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
}

unsafe fn validate(ok: bool, err: c_int) -> bool {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
    }
    ok
}

unsafe fn strlen_narrow(s: *const c_char) -> usize {
    let mut n = 0usize;
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
        n += 1;
    }
    n
}

/// `strcpy_s`-equivalent, narrow: copies `src`'s NUL-terminated bytes
/// (including the NUL) into `dst`, which the caller has already sized. No
/// separate bounds check here (mirroring `_ERRCHECK(strcpy_s(...))`'s own
/// "the caller already computed a large-enough size" invariant) -- the C's
/// `strcpy_s` argument IS the remaining space, sized identically by both
/// ports from the same running length total.
unsafe fn strcpy_narrow(dst: *mut c_char, src: *const c_char) {
    let len = strlen_narrow(src) + 1;
    core::ptr::copy_nonoverlapping(src, dst, len);
}

// ── `_Getdays_l` / `_Getdays` ────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _Getdays_l(locale: LocaleT) -> *mut c_char {
    let lu = LocaleUpdate::new(locale);
    let time_data = (*lu.locinfo()).lc_time_curr as *const crate::locale::LcTimeData;

    let mut length: usize = 0;
    for n in 0..7 {
        length += strlen_narrow((*time_data).wday_abbr[n]) + strlen_narrow((*time_data).wday[n]) + 2;
    }

    let buffer = malloc(length + 1) as *mut c_char;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    let mut it = buffer;
    for n in 0..7 {
        *it = b':' as c_char;
        it = it.add(1);
        strcpy_narrow(it, (*time_data).wday_abbr[n]);
        it = it.add(strlen_narrow(it));
        *it = b':' as c_char;
        it = it.add(1);
        strcpy_narrow(it, (*time_data).wday[n]);
        it = it.add(strlen_narrow(it));
    }
    *it = 0;

    buffer
}

#[no_mangle]
pub unsafe extern "C" fn _Getdays() -> *mut c_char {
    _Getdays_l(core::ptr::null_mut())
}

// ── `_Getmonths_l` / `_Getmonths` ────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _Getmonths_l(locale: LocaleT) -> *mut c_char {
    let lu = LocaleUpdate::new(locale);
    let time_data = (*lu.locinfo()).lc_time_curr as *const crate::locale::LcTimeData;

    let mut length: usize = 0;
    for n in 0..12 {
        length += strlen_narrow((*time_data).month_abbr[n]) + strlen_narrow((*time_data).month[n]) + 2;
    }

    let buffer = malloc(length + 1) as *mut c_char;
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    let mut it = buffer;
    for n in 0..12 {
        *it = b':' as c_char;
        it = it.add(1);
        strcpy_narrow(it, (*time_data).month_abbr[n]);
        it = it.add(strlen_narrow(it));
        *it = b':' as c_char;
        it = it.add(1);
        strcpy_narrow(it, (*time_data).month[n]);
        it = it.add(strlen_narrow(it));
    }
    *it = 0;

    buffer
}

#[no_mangle]
pub unsafe extern "C" fn _Getmonths() -> *mut c_char {
    _Getmonths_l(core::ptr::null_mut())
}

// ── `_Gettnames` ─────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _Gettnames() -> *mut c_void {
    super::wcsftime::_W_Gettnames()
}

// ── `_Strftime_l` and friends ────────────────────────────────────────────

/// A tiny heap buffer, freed on drop -- the `__crt_internal_win32_buffer`/
/// `__crt_no_alloc_win32_buffer` role, reduced to what this file needs (a
/// single fixed-size allocation, no growth).
struct WBuf(*mut u16);
impl Drop for WBuf {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { free(self.0 as *mut c_void) };
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _Strftime_l(string: *mut c_char, maxsize: usize, format: *const c_char, timeptr: *const Tm, lc_time_arg: *mut c_void, locale: LocaleT) -> usize {
    let lu = LocaleUpdate::new(locale);
    let lc_time_cp = (*lu.locinfo()).lc_time_cp;

    if !validate(!string.is_null(), EINVAL) {
        return 0;
    }
    if !validate(maxsize != 0, EINVAL) {
        return 0;
    }
    *string = 0;

    if !validate(!format.is_null(), EINVAL) {
        return 0;
    }
    if !validate(!timeptr.is_null(), EINVAL) {
        return 0;
    }

    // `__acrt_mbs_to_wcs_cp`: size query, then fill (both flagged
    // MB_PRECOMPOSED | MB_ERR_INVALID_CHARS, matching the C's inline body
    // exactly -- unlike `_Wcsftime_l`'s FINAL narrow conversion below, this
    // one is free to grow, since it is sizing a local scratch buffer, not
    // writing into the caller's fixed one).
    let need = __acrt_MultiByteToWideChar(lc_time_cp, MB_PRECOMPOSED | MB_ERR_INVALID_CHARS, format, -1, core::ptr::null_mut(), 0);
    if need == 0 {
        return 0;
    }
    let wformat = WBuf(malloc(need as usize * 2) as *mut u16);
    if wformat.0.is_null() {
        return 0;
    }
    if __acrt_MultiByteToWideChar(lc_time_cp, MB_PRECOMPOSED | MB_ERR_INVALID_CHARS, format, -1, wformat.0, need) == 0 {
        return 0;
    }

    // A fresh wide OUTPUT buffer, `maxsize` wide characters (see module doc
    // for why that element count is never too small).
    let wstring = WBuf(malloc(maxsize * 2) as *mut u16);
    if wstring.0.is_null() {
        return 0;
    }

    let wcsftime_result = _Wcsftime_l(wstring.0, maxsize, wformat.0, timeptr, lc_time_arg, locale);
    if wcsftime_result == 0 {
        return 0;
    }

    // Final conversion writes DIRECTLY into the caller's fixed-size
    // `string` buffer -- no growth, matching `__crt_no_alloc_win32_buffer`.
    let cvt2 = __acrt_WideCharToMultiByte(lc_time_cp, 0, wstring.0, -1, string, maxsize as c_int, core::ptr::null(), core::ptr::null_mut());
    if cvt2 == 0 {
        return 0;
    }

    strlen_narrow(string)
}

#[no_mangle]
pub unsafe extern "C" fn _Strftime(string: *mut c_char, max_size: usize, format: *const c_char, timeptr: *const Tm, lc_time_arg: *mut c_void) -> usize {
    _Strftime_l(string, max_size, format, timeptr, lc_time_arg, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _strftime_l(string: *mut c_char, max_size: usize, format: *const c_char, timeptr: *const Tm, locale: LocaleT) -> usize {
    _Strftime_l(string, max_size, format, timeptr, core::ptr::null_mut(), locale)
}

#[no_mangle]
pub unsafe extern "C" fn strftime(string: *mut c_char, max_size: usize, format: *const c_char, timeptr: *const Tm) -> usize {
    _Strftime_l(string, max_size, format, timeptr, core::ptr::null_mut(), core::ptr::null_mut())
}
