//! `convert/mbtowc.cpp`: `_mbtowc_internal`, `_mbtowc_l`, `mbtowc`.
//!
//! `_mbtowc_internal` is declared in `corecrt_internal_mbstring.h` and called
//! by `output.rs`'s printf processor (still C++ elsewhere in spirit, but here
//! it's another Rust module in THIS crate) with the exact signature
//! `fn _mbtowc_internal(dst: *mut u16, src: *const u8, src_size: usize, ptd:
//! *mut PtdHost) -> c_int` -- matched byte-for-byte (`*const u8`, not `*const
//! c_char`, per that file's own declaration) so `#![deny(clashing_extern_declarations)]`
//! has nothing to say when both modules are enabled together.

use super::common::{
    __acrt_MultiByteToWideChar, is_leadbyte_fast, set_errno, CP_UTF8, EILSEQ, CRT_LC_CTYPE,
    MB_ERR_INVALID_CHARS, MB_PRECOMPOSED,
};
use super::mbrtowc::mbrtowc_utf8;
use crate::locale::CrtLocalePointers;
use crate::ptd_host::PtdHost;
use core::ffi::c_int;

/// `_mbtowc_internal` (corecrt_internal_mbstring.h). Signature pinned to
/// `output.rs`'s declaration -- see module docs.
///
/// # Safety
/// `s`, if non-null, must be readable for `n` bytes or up to the first NUL;
/// `pwc`, if non-null, must be writable; `ptd` must be a valid `*mut PtdHost`.
#[no_mangle]
pub unsafe extern "C" fn _mbtowc_internal(pwc: *mut u16, s: *const u8, n: usize, ptd: *mut PtdHost) -> c_int {
    static mut INTERNAL_STATE: super::common::MbstateT = super::common::MbstateT::zeroed();
    let ptd = &mut *ptd;

    if s.is_null() || n == 0 {
        INTERNAL_STATE = super::common::MbstateT::zeroed();
        return 0;
    }

    if *s == 0 {
        if !pwc.is_null() {
            *pwc = 0;
        }
        return 0;
    }

    let locale = ptd.get_locale();
    let locinfo = (*locale).locinfo;
    let public = (*locinfo).public_ref();

    if (*public).locale_lc_codepage == CP_UTF8 {
        let mut result = mbrtowc_utf8(pwc, s.cast(), n, core::ptr::addr_of_mut!(INTERNAL_STATE), ptd) as isize;
        if result < 0 {
            result = -1;
        }
        return result as c_int;
    }

    let mb_cur_max = (*public).locale_mb_cur_max;

    if (*locinfo).locale_name[CRT_LC_CTYPE].is_null() {
        if !pwc.is_null() {
            *pwc = *s as u16;
        }
        return 1;
    }

    if is_leadbyte_fast(*s, locale) {
        if mb_cur_max <= 1
            || (n as c_int) < mb_cur_max
            || __acrt_MultiByteToWideChar(
                (*public).locale_lc_codepage,
                MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
                s.cast(),
                mb_cur_max,
                pwc,
                if !pwc.is_null() { 1 } else { 0 },
            ) == 0
        {
            if n < mb_cur_max as usize || *s.add(1) == 0 {
                set_errno(ptd, EILSEQ);
                return -1;
            }
        }
        return mb_cur_max;
    }

    if __acrt_MultiByteToWideChar(
        (*public).locale_lc_codepage,
        MB_PRECOMPOSED | MB_ERR_INVALID_CHARS,
        s.cast(),
        1,
        pwc,
        if !pwc.is_null() { 1 } else { 0 },
    ) == 0
    {
        set_errno(ptd, EILSEQ);
        return -1;
    }
    1
}

/// `_mbtowc_l`.
///
/// # Safety
/// Same as `_mbtowc_internal`; `plocinfo` must be null or a valid `_locale_t`.
#[no_mangle]
pub unsafe extern "C" fn _mbtowc_l(pwc: *mut u16, s: *const u8, n: usize, plocinfo: *mut CrtLocalePointers) -> c_int {
    let mut ptd = PtdHost::new(plocinfo);
    _mbtowc_internal(pwc, s, n, &mut ptd)
}

/// `mbtowc`.
///
/// # Safety
/// Same as `_mbtowc_internal` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn mbtowc(pwc: *mut u16, s: *const u8, n: usize) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    _mbtowc_internal(pwc, s, n, &mut ptd)
}
