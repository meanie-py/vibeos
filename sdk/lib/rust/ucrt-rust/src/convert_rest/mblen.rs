//! `convert/mblen.cpp`: `mblen`/`_mblen_l`.

use super::common::{
    is_leadbyte_fast, __acrt_MultiByteToWideChar, CP_UTF8, MB_ERR_INVALID_CHARS, MB_PRECOMPOSED,
    MbstateT,
};
use super::mbrtoc32::mbrtoc32_utf8;
use crate::locale::CrtLocalePointers;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

unsafe fn mblen_internal(string: *const c_char, max_count: usize, ptd: &mut PtdHost) -> c_int {
    let mut internal_state = MbstateT::zeroed();
    if string.is_null() || *string == 0 || max_count == 0 {
        return 0;
    }

    let locale = ptd.get_locale();
    let locinfo = (*locale).locinfo;
    let public = (*locinfo).public_ref();

    if (*public).locale_lc_codepage == CP_UTF8 {
        let result = mbrtoc32_utf8(core::ptr::null_mut(), string, max_count, &mut internal_state, ptd);
        return if (result as isize) < 0 { -1 } else { result as c_int };
    }

    if is_leadbyte_fast(*string as u8, locale) {
        let mb_cur_max = (*public).locale_mb_cur_max;
        if mb_cur_max <= 1 {
            return -1;
        }

        if max_count > c_int::MAX as usize || (max_count as c_int) < mb_cur_max {
            return -1;
        }

        let status = __acrt_MultiByteToWideChar(
            (*public).locale_lc_codepage,
            MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
            string,
            mb_cur_max,
            core::ptr::null_mut(),
            0,
        );

        if status == 0 {
            return -1;
        }

        return mb_cur_max;
    }

    // Single byte character; verify that it is valid. CP_ACP is known valid
    // for all values.
    if (*public).locale_lc_codepage != super::common::CP_ACP {
        let status = __acrt_MultiByteToWideChar(
            (*public).locale_lc_codepage,
            MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
            string,
            1,
            core::ptr::null_mut(),
            0,
        );
        if status == 0 {
            return -1;
        }
    }

    1
}

/// `_mblen_l`.
///
/// # Safety
/// `string`, if non-null, must be readable up to `max_count` bytes or the
/// first NUL; `locale` must be null or a valid `_locale_t`.
#[no_mangle]
pub unsafe extern "C" fn _mblen_l(
    string: *const c_char,
    max_count: usize,
    locale: *mut CrtLocalePointers,
) -> c_int {
    let mut ptd = PtdHost::new(locale);
    mblen_internal(string, max_count, &mut ptd)
}

/// `mblen`.
///
/// # Safety
/// Same as `_mblen_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn mblen(string: *const c_char, max_count: usize) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    mblen_internal(string, max_count, &mut ptd)
}
