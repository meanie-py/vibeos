//! `convert/wcrtomb.cpp`: `wcrtomb_s`, `wcrtomb`, `wcsrtombs`, `wcsrtombs_s`,
//! `wctob`, and `__crt_mbstring::__wcsrtombs_utf8`.

use super::c32rtomb::c32rtomb_utf8;
use super::common::{
    set_errno, validate_errcode, MbstateT, CP_UTF8, EILSEQ, EINVAL, INVALID,
    CRT_LC_CTYPE, MB_LEN_MAX,
};
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
    fn memcpy_s(dst: *mut core::ffi::c_void, dst_size: usize, src: *const core::ffi::c_void, count: usize) -> c_int;
}

const EOF_C: c_int = -1;
const WEOF: u16 = 0xFFFF;

/// `_wcrtomb_internal`.
unsafe fn wcrtomb_internal(
    return_value: *mut c_int,
    destination: *mut c_char,
    destination_count: usize,
    wchar: u16,
    state: *mut MbstateT,
    ptd: &mut PtdHost,
) -> c_int {
    let locale = ptd.get_locale();
    let locinfo = (*locale).locinfo;
    let public = (*locinfo).public_ref();

    if !state.is_null() {
        (*state).wchar = 0;
    }

    if (*public).locale_lc_codepage == CP_UTF8 {
        static mut LOCAL_STATE: MbstateT = MbstateT::zeroed();
        let st = if !state.is_null() { state } else { core::ptr::addr_of_mut!(LOCAL_STATE) };
        let result = c32rtomb_utf8(destination, wchar as u32, st, ptd);
        if !return_value.is_null() {
            *return_value = result as c_int;
        }
        return if result <= 4 {
            0
        } else if result == INVALID {
            EILSEQ // the only error this path ever records
        } else {
            0
        };
    }

    if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
        if wchar > 255 {
            if !return_value.is_null() {
                *return_value = -1;
            }
            return set_errno(ptd, EILSEQ);
        }
        *destination = wchar as u8 as c_char;
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
        destination,
        destination_count as c_int,
        core::ptr::null(),
        &mut default_used,
    );

    if size == 0 || default_used != 0 {
        if !return_value.is_null() {
            *return_value = -1;
        }
        return set_errno(ptd, EILSEQ);
    }

    if !return_value.is_null() {
        *return_value = size;
    }
    0
}

/// `wcrtomb_s_internal`.
unsafe fn wcrtomb_s_internal(
    return_value: *mut usize,
    destination: *mut c_char,
    destination_count: usize,
    wchar: u16,
    state: *mut MbstateT,
    ptd: &mut PtdHost,
) -> c_int {
    if let Some(r) = validate_errcode(
        ptd,
        (destination.is_null() && destination_count == 0) || !destination.is_null(),
        EINVAL,
    ) {
        return r;
    }

    let mut int_return_value: c_int = -1;
    let e;
    if destination.is_null() {
        let mut buf = [0 as c_char; MB_LEN_MAX];
        e = wcrtomb_internal(&mut int_return_value, buf.as_mut_ptr(), MB_LEN_MAX, wchar, state, ptd);
    } else {
        e = wcrtomb_internal(&mut int_return_value, destination, destination_count, wchar, state, ptd);
    }

    if !return_value.is_null() {
        *return_value = int_return_value as usize;
    }
    e
}

/// `wcrtomb_s`.
///
/// # Safety
/// `destination`, if non-null, must have room for `destination_count` bytes.
#[no_mangle]
pub unsafe extern "C" fn wcrtomb_s(
    return_value: *mut usize,
    destination: *mut c_char,
    destination_count: usize,
    wchar: u16,
    state: *mut MbstateT,
) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    wcrtomb_s_internal(return_value, destination, destination_count, wchar, state, &mut ptd)
}

/// `wcrtomb`.
///
/// # Safety
/// `destination`, if non-null, must have room for `MB_LEN_MAX` bytes.
#[no_mangle]
pub unsafe extern "C" fn wcrtomb(destination: *mut c_char, wchar: u16, state: *mut MbstateT) -> usize {
    let mut return_value: usize = usize::MAX;
    wcrtomb_s(
        &mut return_value,
        destination,
        if destination.is_null() { 0 } else { MB_LEN_MAX },
        wchar,
        state,
    );
    return_value
}

/// `_wcsrtombs_internal`.
unsafe fn wcsrtombs_internal(
    mut destination: *mut c_char,
    source: *mut *const u16,
    mut n: usize,
    state: *mut MbstateT,
    ptd: &mut PtdHost,
) -> usize {
    if let Some(r) = super::common::validate_return(ptd, !source.is_null(), EINVAL, usize::MAX) {
        return r;
    }

    let locale = ptd.get_locale();
    let public = (*(*locale).locinfo).public_ref();

    if (*public).locale_lc_codepage == CP_UTF8 {
        return wcsrtombs_utf8(destination, source, n, state, ptd);
    }

    let mut buf = [0 as c_char; MB_LEN_MAX];
    let mut i: c_int = 0;
    let mut nc: usize = 0;
    let mut wcs = *source;

    if destination.is_null() {
        loop {
            wcrtomb_internal(&mut i, buf.as_mut_ptr(), MB_LEN_MAX, *wcs, state, ptd);
            if i <= 0 {
                return usize::MAX;
            } else if buf[(i - 1) as usize] == 0 {
                return nc + (i - 1) as usize;
            }
            nc += i as usize;
            wcs = wcs.add(1);
        }
    }

    let mb_cur_max = (*public).locale_mb_cur_max as usize;
    while n > 0 {
        let t: *mut c_char = if n < mb_cur_max { buf.as_mut_ptr() } else { destination };

        i = 0;
        wcrtomb_internal(&mut i, t, MB_LEN_MAX, *wcs, state, ptd);
        if i <= 0 {
            return usize::MAX;
        }

        if destination == t {
            // Wrote in place; nothing to do.
        } else if n < i as usize {
            break; // Won't all fit.
        } else {
            memcpy_s(destination.cast(), n, buf.as_ptr().cast(), i as usize);
        }

        if *destination.add((i - 1) as usize) == 0 {
            *source = core::ptr::null();
            return nc + (i - 1) as usize;
        }

        nc += i as usize;
        wcs = wcs.add(1);
        destination = destination.add(i as usize);
        n -= i as usize;
    }

    *source = wcs;
    nc
}

/// `wcsrtombs`.
///
/// # Safety
/// `source` must point to a valid `const wchar_t*`; `destination`, if
/// non-null, must have room for `n` bytes.
#[no_mangle]
pub unsafe extern "C" fn wcsrtombs(destination: *mut c_char, source: *mut *const u16, n: usize, state: *mut MbstateT) -> usize {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    wcsrtombs_internal(destination, source, n, state, &mut ptd)
}

unsafe fn reset_string_c(buf: *mut c_char, count: usize) {
    // `_RESET_STRING`: `*s = 0` plus `_FILL_STRING`, which is a no-op in this release
    // build -- ONE element, not the whole count (the whole count is a memset; and
    // _gcvt passes (size_t)-1 >> 1 as its size).
    if count > 0 {
        *buf = 0;
    }
}

/// `wcsrtombs_s`.
///
/// # Safety
/// See `wcsrtombs`; `destination` must have room for `destination_count`
/// bytes when non-null.
#[no_mangle]
pub unsafe extern "C" fn wcsrtombs_s(
    return_value: *mut usize,
    destination: *mut c_char,
    destination_count: usize,
    source: *mut *const u16,
    n: usize,
    state: *mut MbstateT,
) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());

    if !return_value.is_null() {
        *return_value = usize::MAX;
    }

    if let Some(r) = validate_errcode(
        &mut ptd,
        (destination.is_null() && destination_count == 0) || (!destination.is_null() && destination_count > 0),
        EINVAL,
    ) {
        return r;
    }

    if !destination.is_null() {
        reset_string_c(destination, destination_count);
    }

    if let Some(r) = validate_errcode(&mut ptd, !source.is_null(), EINVAL) {
        return r;
    }

    let mut retsize = wcsrtombs_internal(
        destination,
        source,
        if n > destination_count { destination_count } else { n },
        state,
        &mut ptd,
    );

    if retsize == usize::MAX {
        if !destination.is_null() {
            reset_string_c(destination, destination_count);
        }
        return 0; // see mbsrtowcs_s's note on `value_or`
    }

    retsize += 1; // account for the null terminator

    if !destination.is_null() {
        // Return error if the string does not fit. Unlike `mbsrtowcs_s`'s
        // analogous check, this comparison direction makes the validate
        // fire EVERY time this branch is entered (`retsize > destination_count`
        // here always makes `retsize <= destination_count` false), so it is
        // a real bounds check, not a no-op -- and the unconditional
        // null-terminator write below is only reachable when it did NOT
        // fire, i.e. when the string DID fit.
        if retsize > destination_count {
            reset_string_c(destination, destination_count);
            if let Some(r) = validate_errcode(&mut ptd, retsize <= destination_count, super::common::ERANGE) {
                return r;
            }
        }
        *destination.add(retsize - 1) = 0;
    }

    if !return_value.is_null() {
        *return_value = retsize;
    }

    0
}

/// `wctob`.
///
/// # Safety
/// Always safe to call.
#[no_mangle]
pub unsafe extern "C" fn wctob(wchar: u16) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());

    if wchar == WEOF {
        return EOF_C;
    }

    let mut return_value: c_int = -1;
    let mut local_buffer = [0 as c_char; MB_LEN_MAX];
    let mut state = MbstateT::zeroed();
    let e = wcrtomb_internal(&mut return_value, local_buffer.as_mut_ptr(), MB_LEN_MAX, wchar, &mut state, &mut ptd);
    if e == 0 && return_value == 1 {
        local_buffer[0] as c_int
    } else {
        EOF_C
    }
}

/// `__crt_mbstring::__wcsrtombs_utf8`.
pub unsafe fn wcsrtombs_utf8(
    dst: *mut c_char,
    src: *mut *const u16,
    mut len: usize,
    ps: *mut MbstateT,
    ptd: &mut PtdHost,
) -> usize {
    use super::c16rtomb::c16rtomb_utf8;

    let mut current_src = *src;
    let mut buf = [0 as c_char; MB_LEN_MAX];

    if !dst.is_null() {
        let mut current_dest = dst;
        let mut start_of_code_point = current_src;
        loop {
            let temp: *mut c_char = if len < 4 { buf.as_mut_ptr() } else { current_dest };
            let retval = c16rtomb_utf8(temp, *current_src, ps, ptd);

            if retval == INVALID {
                *src = start_of_code_point;
                return retval;
            }

            if temp == current_dest {
                // Wrote in place.
            } else if len < retval {
                current_src = start_of_code_point;
                break;
            } else {
                core::ptr::copy_nonoverlapping(temp, current_dest, retval);
            }

            if retval > 0 && *current_dest.add(retval - 1) == 0 {
                current_src = core::ptr::null();
                current_dest = current_dest.add(retval - 1);
                break;
            }

            current_src = current_src.add(1);
            if retval > 0 {
                start_of_code_point = current_src;
            }

            len -= retval;
            current_dest = current_dest.add(retval);
        }
        *src = current_src;
        return current_dest.offset_from(dst) as usize;
    }

    let mut total_count = 0usize;
    loop {
        let retval = c16rtomb_utf8(buf.as_mut_ptr(), *current_src, ps, ptd);
        if retval == INVALID {
            return retval;
        } else if retval > 0 && buf[retval - 1] == 0 {
            total_count += retval - 1;
            break;
        }
        total_count += retval;
        current_src = current_src.add(1);
    }
    total_count
}
