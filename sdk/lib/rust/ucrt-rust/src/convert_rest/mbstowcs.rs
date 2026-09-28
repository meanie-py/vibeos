//! `convert/mbstowcs.cpp`: `_mbstowcs_l`, `mbstowcs`, `_mbstowcs_s_l`,
//! `mbstowcs_s` (the deprecated non-restartable wrappers, sharing
//! `_mbstowcs_l_helper`).

use super::common::{
    is_leadbyte_fast, set_errno, validate_errcode, CP_UTF8, EILSEQ, EINVAL, ERANGE, CRT_LC_CTYPE,
    MB_ERR_INVALID_CHARS, MB_PRECOMPOSED, STRUNCATE, TRUNCATE,
};
use super::mbrtowc::mbsrtowcs_utf8;
use crate::locale::CrtLocalePointers;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

extern "C" {
    fn __acrt_MultiByteToWideChar(
        cp: core::ffi::c_uint,
        flags: core::ffi::c_uint,
        src: *const c_char,
        src_len: c_int,
        dst: *mut u16,
        dst_len: c_int,
    ) -> c_int;
    fn strlen(s: *const c_char) -> usize;
}
extern "system" {
    fn GetLastError() -> u32;
}

const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const INT_MAX: usize = c_int::MAX as usize;

/// `_mbstowcs_l_helper`.
unsafe fn mbstowcs_l_helper(pwcs: *mut u16, s: *const c_char, n: usize, ptd: &mut PtdHost) -> usize {
    if !pwcs.is_null() && n == 0 {
        return 0;
    }
    if !pwcs.is_null() && n > 0 {
        *pwcs = 0;
    }

    if let Some(r) = super::common::validate_return(ptd, !s.is_null(), EINVAL, usize::MAX) {
        return r;
    }

    let locale = ptd.get_locale();
    let locinfo = (*locale).locinfo;
    let public = (*locinfo).public_ref();

    if (*public).locale_lc_codepage == CP_UTF8 {
        let mut state = super::common::MbstateT::zeroed();
        let mut s_local = s;
        return mbsrtowcs_utf8(pwcs, &mut s_local, n, &mut state, ptd);
    }

    if !pwcs.is_null() {
        if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
            let mut count = 0usize;
            let mut p = pwcs;
            while count < n {
                *p = *s.add(count) as u8 as u16;
                if *s.add(count) == 0 {
                    return count;
                }
                count += 1;
                p = p.add(1);
            }
            return count;
        }

        // Assume the buffer is large enough.
        let count = __acrt_MultiByteToWideChar(
            (*public).locale_lc_codepage,
            MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
            s,
            -1,
            pwcs,
            n as c_int,
        );
        if count != 0 {
            return (count - 1) as usize; // don't count NUL
        }

        if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            set_errno(ptd, EILSEQ);
            *pwcs = 0;
            return usize::MAX;
        }

        // User-supplied buffer not large enough: figure out how many bytes
        // are in `n` characters of the string.
        let mut charcnt = n as isize;
        let mut p = s;
        while charcnt != 0 && *p != 0 {
            charcnt -= 1;
            if is_leadbyte_fast(*p as u8, locale) {
                if *p.add(1) == 0 {
                    set_errno(ptd, EILSEQ);
                    *pwcs = 0;
                    return usize::MAX;
                }
                p = p.add(1);
            }
            p = p.add(1);
        }
        let bytecnt = p.offset_from(s);

        let count = __acrt_MultiByteToWideChar(
            (*public).locale_lc_codepage,
            MB_PRECOMPOSED,
            s,
            bytecnt as c_int,
            pwcs,
            n as c_int,
        );
        if count == 0 {
            set_errno(ptd, EILSEQ);
            *pwcs = 0;
            return usize::MAX;
        }
        return count as usize; // no NUL in string
    }

    // pwcs == null: get size only, s must be NUL-terminated.
    if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
        return strlen(s);
    }
    let count = __acrt_MultiByteToWideChar(
        (*public).locale_lc_codepage,
        MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
        s,
        -1,
        core::ptr::null_mut(),
        0,
    );
    if count == 0 {
        set_errno(ptd, EILSEQ);
        usize::MAX
    } else {
        (count - 1) as usize
    }
}

/// `_mbstowcs_l`.
///
/// # Safety
/// `s` must be a valid NUL-terminated C string; `pwcs`, if non-null, must
/// have room for `n` wide characters.
#[no_mangle]
pub unsafe extern "C" fn _mbstowcs_l(pwcs: *mut u16, s: *const c_char, n: usize, plocinfo: *mut CrtLocalePointers) -> usize {
    let mut ptd = PtdHost::new(plocinfo);
    mbstowcs_l_helper(pwcs, s, n, &mut ptd)
}

/// `mbstowcs`.
///
/// # Safety
/// Same as `_mbstowcs_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn mbstowcs(pwcs: *mut u16, s: *const c_char, n: usize) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    mbstowcs_l_helper(pwcs, s, n, &mut ptd)
}

unsafe fn reset_string(buf: *mut u16, count: usize) {
    // `_RESET_STRING`: `*s = 0` plus `_FILL_STRING`, which is a no-op in this release
    // build -- ONE element, not the whole count (the whole count is a memset; and
    // _gcvt passes (size_t)-1 >> 1 as its size).
    if count > 0 {
        *buf = 0;
    }
}

/// `_mbstowcs_internal`.
unsafe fn mbstowcs_internal(
    p_converted_chars: *mut usize,
    pwcs: *mut u16,
    size_in_words: usize,
    s: *const c_char,
    n: usize,
    ptd: &mut PtdHost,
) -> c_int {
    let mut retvalue: c_int = 0;

    if let Some(r) = validate_errcode(ptd, (pwcs.is_null() && size_in_words == 0) || (!pwcs.is_null() && size_in_words > 0), EINVAL) {
        return r;
    }

    if !pwcs.is_null() {
        reset_string(pwcs, size_in_words);
    }
    if !p_converted_chars.is_null() {
        *p_converted_chars = 0;
    }

    let buffer_size = if n > size_in_words { size_in_words } else { n };
    if let Some(r) = validate_errcode(ptd, buffer_size <= INT_MAX, EINVAL) {
        return r;
    }

    let mut retsize = mbstowcs_l_helper(pwcs, s, buffer_size, ptd);

    if retsize == usize::MAX {
        if !pwcs.is_null() {
            reset_string(pwcs, size_in_words);
        }
        return 0; // see mbrtowc.rs's note on `value_or`
    }

    retsize += 1;

    if !pwcs.is_null() {
        if retsize > size_in_words {
            if n != TRUNCATE {
                reset_string(pwcs, size_in_words);
                if let Some(r) = validate_errcode(ptd, retsize <= size_in_words, ERANGE) {
                    return r;
                }
            }
            retsize = size_in_words;
            retvalue = STRUNCATE;
        }
        *pwcs.add(retsize - 1) = 0;
    }

    if !p_converted_chars.is_null() {
        *p_converted_chars = retsize;
    }

    retvalue
}

/// `_mbstowcs_s_l`.
///
/// # Safety
/// `pwcs` must have room for `size_in_words` wide characters when non-null;
/// `s` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn _mbstowcs_s_l(
    p_converted_chars: *mut usize,
    pwcs: *mut u16,
    size_in_words: usize,
    s: *const c_char,
    n: usize,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    let mut ptd = PtdHost::new(plocinfo);
    mbstowcs_internal(p_converted_chars, pwcs, size_in_words, s, n, &mut ptd)
}

/// `mbstowcs_s`.
///
/// # Safety
/// Same as `_mbstowcs_s_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn mbstowcs_s(
    p_converted_chars: *mut usize,
    pwcs: *mut u16,
    size_in_words: usize,
    s: *const c_char,
    n: usize,
) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    mbstowcs_internal(p_converted_chars, pwcs, size_in_words, s, n, &mut ptd)
}
