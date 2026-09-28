//! `convert/_mbslen.cpp`: `_mbstrlen`/`_mbstrlen_l`/`_mbstrnlen`/`_mbstrnlen_l`.

use super::common::is_leadbyte_fast;
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use core::ffi::{c_char, c_int};

const EINVAL: c_int = 22;
/// `_CRT_UNBOUNDED_BUFFER_SIZE` (corecrt_internal.h): `(size_t)-1 >> 1` --
/// half of SIZE_MAX, used as an effectively-unbounded max for `_mbstrlen`.
const CRT_UNBOUNDED_BUFFER_SIZE: usize = usize::MAX >> 1;

extern "C" {
    fn strnlen(s: *const c_char, max: usize) -> usize;
    fn strlen(s: *const c_char) -> usize;
    fn __acrt_MultiByteToWideChar(
        cp: core::ffi::c_uint,
        flags: core::ffi::c_uint,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

/// `__acrt_locale_changed()` (corecrt_internal.h:744, `__inline`, so no
/// external symbol to link against): `__acrt_locale_changed_data != FALSE`.
/// Reuses `locale_update.rs`'s `pub(crate)` extern for the same global so two
/// modules don't independently redeclare it.
#[inline]
unsafe fn acrt_locale_changed() -> bool {
    crate::locale_update::__acrt_locale_changed_data != 0
}

const MB_PRECOMPOSED: core::ffi::c_uint = 0x1;
const MB_ERR_INVALID_CHARS: core::ffi::c_uint = 0x8;

unsafe fn common_mbstrlen_l(string: *const c_char, max_size: usize, locale: *mut CrtLocalePointers) -> usize {
    let mut lu = LocaleUpdate::new(locale);
    let locinfo = lu.locinfo();
    let public = (*locinfo).public_ref();

    if (*public).locale_mb_cur_max == 1 {
        return strnlen(string, max_size);
    }

    if __acrt_MultiByteToWideChar(
        (*public).locale_lc_codepage,
        MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
        string,
        max_size as c_int,
        core::ptr::null_mut(),
        0,
    ) == 0
    {
        *_errno() = 42; // EILSEQ
        return usize::MAX;
    }

    let mut n = 0usize; // multibyte characters read
    let mut size = 0usize; // bytes read
    let mut it = string;
    while size < max_size && *it != 0 {
        if is_leadbyte_fast(*it as u8, lu.locale_t()) {
            size += 1;
            if size >= max_size {
                break;
            }
            it = it.add(1);
            if *it == 0 {
                break;
            }
        }
        n += 1;
        it = it.add(1);
        size += 1;
    }

    if size >= max_size {
        max_size
    } else {
        n
    }
}

/// `_mbstrlen_l`.
///
/// # Safety
/// `string` must be a valid NUL-terminated (or at least readable) C string;
/// `locale` must be null or a valid `_locale_t`.
#[no_mangle]
pub unsafe extern "C" fn _mbstrlen_l(string: *const c_char, locale: *mut CrtLocalePointers) -> usize {
    common_mbstrlen_l(string, CRT_UNBOUNDED_BUFFER_SIZE, locale)
}

/// `_mbstrlen`.
///
/// # Safety
/// `string` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn _mbstrlen(string: *const c_char) -> usize {
    if !acrt_locale_changed() {
        strlen(string)
    } else {
        _mbstrlen_l(string, core::ptr::null_mut())
    }
}

/// `_mbstrnlen_l`.
///
/// # Safety
/// `string` must be readable for `max_size` bytes or up to the first NUL.
#[no_mangle]
pub unsafe extern "C" fn _mbstrnlen_l(
    string: *const c_char,
    max_size: usize,
    locale: *mut CrtLocalePointers,
) -> usize {
    if string.is_null() {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return usize::MAX;
    }
    if max_size > c_int::MAX as usize {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return usize::MAX;
    }

    common_mbstrlen_l(string, max_size, locale)
}

/// `_mbstrnlen`.
///
/// # Safety
/// Same as `_mbstrnlen_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn _mbstrnlen(string: *const c_char, max_size: usize) -> usize {
    _mbstrnlen_l(string, max_size, core::ptr::null_mut())
}
