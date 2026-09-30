//! `convert/wctomb.cpp`: `_wctomb_internal`, `_wctomb_s_l`, `wctomb_s`,
//! `_wctomb_l`, `wctomb`.
//!
//! `_wctomb_internal`'s signature is pinned to `output.rs`'s declaration --
//! `fn(size_converted: *mut c_int, mbch: *mut u8, size_in_bytes: usize, wch:
//! u16, ptd: *mut PtdHost) -> c_int` -- for the same
//! `clashing_extern_declarations` reason as `mbtowc.rs`.

use super::c32rtomb::c32rtomb_utf8;
use super::common::{
    __acrt_WideCharToMultiByte, set_errno, MbstateT, CP_UTF8, EILSEQ, EINVAL, ERANGE, CRT_LC_CTYPE,
};
use crate::locale::CrtLocalePointers;
use crate::ptd_host::PtdHost;
use core::ffi::c_int;

extern "system" {
    fn GetLastError() -> u32;
}

const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const INT_MAX: usize = c_int::MAX as usize;

/// `_wctomb_internal`.
///
/// # Safety
/// `destination`, if non-null, must have room for `destination_count` bytes;
/// `return_value`, if non-null, must be writable; `ptd` a valid `*mut PtdHost`.
#[no_mangle]
pub unsafe extern "C" fn _wctomb_internal(
    return_value: *mut c_int,
    destination: *mut u8,
    destination_count: usize,
    wchar: u16,
    ptd: *mut PtdHost,
) -> c_int {
    let ptd = &mut *ptd;

    if destination.is_null() && destination_count > 0 {
        if !return_value.is_null() {
            *return_value = 0;
        }
        return 0;
    }

    if !return_value.is_null() {
        *return_value = -1;
    }

    if let Some(r) = super::common::validate_errcode(ptd, destination_count <= INT_MAX, EINVAL) {
        return r;
    }

    let locale = ptd.get_locale();
    let locinfo = (*locale).locinfo;
    let public = (*locinfo).public_ref();

    if (*public).locale_lc_codepage == CP_UTF8 {
        let mut state = MbstateT::zeroed();
        let result = c32rtomb_utf8(destination.cast(), wchar as u32, &mut state, ptd);
        if !return_value.is_null() {
            *return_value = result as c_int;
        }
        return if result <= 4 { 0 } else { set_errno(ptd, EILSEQ) };
    }

    if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
        if wchar > 255 {
            if !destination.is_null() && destination_count > 0 {
                core::ptr::write_bytes(destination, 0, destination_count);
            }
            return set_errno(ptd, EILSEQ);
        }

        if !destination.is_null() {
            if let Some(r) = super::common::validate_errcode(ptd, destination_count > 0, ERANGE) {
                return r;
            }
            *destination = wchar as u8;
        }

        if !return_value.is_null() {
            *return_value = 1;
        }
        return 0;
    }

    let mut default_used: c_int = 0;
    let size = __acrt_WideCharToMultiByte(
        (*public).locale_lc_codepage,
        0,
        &wchar,
        1,
        destination.cast(),
        destination_count as c_int,
        core::ptr::null(),
        &mut default_used,
    );

    if size == 0 || default_used != 0 {
        if size == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            if !destination.is_null() && destination_count > 0 {
                core::ptr::write_bytes(destination, 0, destination_count);
            }
            if let Some(r) = super::common::validate_errcode(ptd, false, ERANGE) {
                return r;
            }
        }
        return set_errno(ptd, EILSEQ);
    }

    if !return_value.is_null() {
        *return_value = size;
    }
    0
}

/// `_wctomb_s_l`.
///
/// # Safety
/// Same as `_wctomb_internal`; `locale` must be null or a valid `_locale_t`.
#[no_mangle]
pub unsafe extern "C" fn _wctomb_s_l(
    return_value: *mut c_int,
    destination: *mut u8,
    destination_count: usize,
    wchar: u16,
    locale: *mut CrtLocalePointers,
) -> c_int {
    let mut ptd = PtdHost::new(locale);
    _wctomb_internal(return_value, destination, destination_count, wchar, &mut ptd)
}

/// `wctomb_s`.
///
/// # Safety
/// Same as `_wctomb_internal`.
#[no_mangle]
pub unsafe extern "C" fn wctomb_s(return_value: *mut c_int, destination: *mut u8, destination_count: usize, wchar: u16) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    _wctomb_internal(return_value, destination, destination_count, wchar, &mut ptd)
}

/// `_wctomb_l`.
///
/// # Safety
/// `destination`, if non-null, must have room for the locale's `MB_CUR_MAX`
/// bytes (this function does not itself bound-check beyond that -- neither
/// does the C).
#[no_mangle]
pub unsafe extern "C" fn _wctomb_l(destination: *mut u8, wchar: u16, locale: *mut CrtLocalePointers) -> c_int {
    let mut ptd = PtdHost::new(locale);
    let mb_cur_max = (*(*(*ptd.get_locale()).locinfo).public_ref()).locale_mb_cur_max;
    let mut return_value: c_int = 0;
    let e = _wctomb_internal(&mut return_value, destination, mb_cur_max as usize, wchar, &mut ptd);
    if e != 0 {
        -1
    } else {
        return_value
    }
}

extern "C" {
    /// `MB_CUR_MAX` (a macro over `___mb_cur_max_func()` in the real header);
    /// declared here as the function it expands to since this module has no
    /// other reason to pull in the whole locale surface.
    fn ___mb_cur_max_func() -> c_int;
}

/// `wctomb`.
///
/// # Safety
/// Same caveat as `_wctomb_l` (`wctomb`'s C contract is inherently
/// unbounded: it takes no buffer size).
#[no_mangle]
pub unsafe extern "C" fn wctomb(destination: *mut u8, wchar: u16) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    let mut return_value: c_int = 0;
    let e = _wctomb_internal(&mut return_value, destination, ___mb_cur_max_func() as usize, wchar, &mut ptd);
    if e != 0 {
        -1
    } else {
        return_value
    }
}
