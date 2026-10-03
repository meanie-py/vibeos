//! `convert/wcstombs.cpp`: `_wcstombs_l`, `wcstombs`, `_wcstombs_s_l`,
//! `wcstombs_s` (the deprecated non-restartable wrappers).

use super::common::{
    set_errno, validate_errcode, CP_UTF8, EILSEQ, EINVAL, ERANGE, CRT_LC_CTYPE, MB_LEN_MAX,
    STRUNCATE, TRUNCATE,
};
use super::wcrtomb::wcsrtombs_utf8;
use crate::locale::CrtLocalePointers;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

extern "C" {
    fn __acrt_WideCharToMultiByte(
        cp: core::ffi::c_uint,
        flags: core::ffi::c_uint,
        src: *const u16,
        src_len: c_int,
        dst: *mut c_char,
        dst_len: c_int,
        default_char: *const c_char,
        used_default: *mut c_int,
    ) -> c_int;
}
extern "system" {
    fn GetLastError() -> u32;
}

const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const INT_MAX: usize = c_int::MAX as usize;

/// `wcsncnt`: count wide characters (incl. NUL) up to `cnt`, without
/// overrunning `cnt` even when no NUL is found in range.
unsafe fn wcsncnt(string: *const u16, cnt: usize) -> usize {
    let mut n = cnt + 1;
    let mut cp = string;
    while n != 0 {
        n -= 1;
        if n == 0 || *cp == 0 {
            break;
        }
        cp = cp.add(1);
    }
    if n != 0 && *cp == 0 {
        (cp.offset_from(string) as usize) + 1
    } else {
        cnt
    }
}

/// `_wcstombs_l_helper`.
unsafe fn wcstombs_l_helper(s: *mut c_char, pwcs: *const u16, n: usize, ptd: &mut PtdHost) -> usize {
    let mut count: usize;

    if !s.is_null() && n == 0 {
        return 0;
    }

    if let Some(r) = super::common::validate_return(ptd, !pwcs.is_null(), EINVAL, usize::MAX) {
        return r;
    }

    let locale = ptd.get_locale();
    let locinfo = (*locale).locinfo;
    let public = (*locinfo).public_ref();

    if (*public).locale_lc_codepage == CP_UTF8 {
        let mut state = super::common::MbstateT::zeroed();
        let mut pwcs_local = pwcs;
        return wcsrtombs_utf8(s, &mut pwcs_local, n, &mut state, ptd);
    }

    if !s.is_null() {
        if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
            count = 0;
            let mut p = pwcs;
            while count < n {
                if *p > 0xff {
                    set_errno(ptd, EILSEQ);
                    return usize::MAX;
                }
                *s.add(count) = *p as u8 as c_char;
                let was_null = *p == 0;
                p = p.add(1);
                if was_null {
                    return count;
                }
                count += 1;
            }
            return count;
        }

        let mut defused: c_int = 0;
        if (*public).locale_mb_cur_max == 1 {
            let n_clamped = if n > 0 { wcsncnt(pwcs, n) } else { n };
            count = __acrt_WideCharToMultiByte(
                (*public).locale_lc_codepage,
                0,
                pwcs,
                n_clamped as c_int,
                s,
                n_clamped as c_int,
                core::ptr::null(),
                &mut defused,
            ) as usize;
            if count != 0 && defused == 0 {
                if *s.add(count - 1) == 0 {
                    count -= 1;
                }
                return count;
            }
            set_errno(ptd, EILSEQ);
            return usize::MAX;
        }

        // MBCS: wchar_t to char mapping unknown. Assume the buffer is
        // usually large enough.
        count = __acrt_WideCharToMultiByte(
            (*public).locale_lc_codepage,
            0,
            pwcs,
            -1,
            s,
            n as c_int,
            core::ptr::null(),
            &mut defused,
        ) as usize;
        if count != 0 && defused == 0 {
            return count - 1; // don't count NUL
        }

        if defused != 0 || GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            set_errno(ptd, EILSEQ);
            return usize::MAX;
        }

        // Buffer not large enough, must do char by char.
        count = 0;
        let mb_cur_max = (*public).locale_mb_cur_max as usize;
        let mut pwcs = pwcs;
        let mut buffer = [0 as c_char; MB_LEN_MAX];
        while count < n {
            let retval = __acrt_WideCharToMultiByte(
                (*public).locale_lc_codepage,
                0,
                pwcs,
                1,
                buffer.as_mut_ptr(),
                core::cmp::min(MB_LEN_MAX, mb_cur_max) as c_int,
                core::ptr::null(),
                &mut defused,
            );
            if retval == 0 || defused != 0 {
                set_errno(ptd, EILSEQ);
                return usize::MAX;
            }
            if retval < 0 || retval as usize > buffer.len() {
                set_errno(ptd, EILSEQ);
                return usize::MAX;
            }

            if count + retval as usize > n {
                return count;
            }

            for i in 0..retval as usize {
                *s.add(count) = buffer[i];
                count += 1;
                if buffer[i] == 0 {
                    return count - 1;
                }
            }

            pwcs = pwcs.add(1);
        }

        return count;
    }

    // s == null: get size only, pwcs must be NUL-terminated.
    if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
        let mut len = 0usize;
        let mut pw = pwcs;
        while *pw != 0 {
            if *pw > 255 {
                set_errno(ptd, EILSEQ);
                return usize::MAX;
            }
            len += 1;
            pw = pw.add(1);
        }
        return len;
    }

    let mut defused: c_int = 0;
    let count = __acrt_WideCharToMultiByte(
        (*public).locale_lc_codepage,
        0,
        pwcs,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        &mut defused,
    );
    if count == 0 || defused != 0 {
        set_errno(ptd, EILSEQ);
        usize::MAX
    } else {
        (count - 1) as usize
    }
}

/// `_wcstombs_l`.
///
/// # Safety
/// `pwcs` must be a valid NUL-terminated wide string; `s`, if non-null, must
/// have room for `n` bytes.
#[no_mangle]
pub unsafe extern "C" fn _wcstombs_l(s: *mut c_char, pwcs: *const u16, n: usize, plocinfo: *mut CrtLocalePointers) -> usize {
    let mut ptd = PtdHost::new(plocinfo);
    wcstombs_l_helper(s, pwcs, n, &mut ptd)
}

/// `wcstombs`.
///
/// # Safety
/// Same as `_wcstombs_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn wcstombs(s: *mut c_char, pwcs: *const u16, n: usize) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    wcstombs_l_helper(s, pwcs, n, &mut ptd)
}

unsafe fn reset_string(buf: *mut c_char, count: usize) {
    // `_RESET_STRING`: `*s = 0` plus `_FILL_STRING`, which is a no-op in this release
    // build -- ONE element, not the whole count (the whole count is a memset; and
    // _gcvt passes (size_t)-1 >> 1 as its size).
    if count > 0 {
        *buf = 0;
    }
}

/// `_wcstombs_internal`.
unsafe fn wcstombs_internal(
    p_converted_chars: *mut usize,
    dst: *mut c_char,
    size_in_bytes: usize,
    src: *const u16,
    n: usize,
    ptd: &mut PtdHost,
) -> c_int {
    let mut retvalue: c_int = 0;

    if let Some(r) = validate_errcode(ptd, (!dst.is_null() && size_in_bytes > 0) || (dst.is_null() && size_in_bytes == 0), EINVAL) {
        return r;
    }
    if !dst.is_null() {
        reset_string(dst, size_in_bytes);
    }
    if !p_converted_chars.is_null() {
        *p_converted_chars = 0;
    }

    let buffer_size = if n > size_in_bytes { size_in_bytes } else { n };
    if let Some(r) = validate_errcode(ptd, buffer_size <= INT_MAX, EINVAL) {
        return r;
    }

    let mut retsize = wcstombs_l_helper(dst, src, buffer_size, ptd);

    if retsize == usize::MAX {
        if !dst.is_null() {
            reset_string(dst, size_in_bytes);
        }
        return 0; // see mbrtowc.rs's note on `value_or`
    }

    retsize += 1;

    if !dst.is_null() {
        if retsize > size_in_bytes {
            if n != TRUNCATE {
                reset_string(dst, size_in_bytes);
                if let Some(r) = validate_errcode(ptd, size_in_bytes > retsize, ERANGE) {
                    return r;
                }
            }
            retsize = size_in_bytes;
            retvalue = STRUNCATE;
        }
        *dst.add(retsize - 1) = 0;
    }

    if !p_converted_chars.is_null() {
        *p_converted_chars = retsize;
    }

    retvalue
}

/// `_wcstombs_s_l`.
///
/// # Safety
/// `dst`, if non-null, must have room for `size_in_bytes` bytes; `src` must
/// be a valid NUL-terminated wide string.
#[no_mangle]
pub unsafe extern "C" fn _wcstombs_s_l(
    p_converted_chars: *mut usize,
    dst: *mut c_char,
    size_in_bytes: usize,
    src: *const u16,
    n: usize,
    plocinfo: *mut CrtLocalePointers,
) -> c_int {
    let mut ptd = PtdHost::new(plocinfo);
    wcstombs_internal(p_converted_chars, dst, size_in_bytes, src, n, &mut ptd)
}

/// `wcstombs_s`.
///
/// # Safety
/// Same as `_wcstombs_s_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn wcstombs_s(
    p_converted_chars: *mut usize,
    dst: *mut c_char,
    size_in_bytes: usize,
    src: *const u16,
    n: usize,
) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    wcstombs_internal(p_converted_chars, dst, size_in_bytes, src, n, &mut ptd)
}
