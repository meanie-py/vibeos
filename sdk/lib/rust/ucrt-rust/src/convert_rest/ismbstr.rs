//! `convert/ismbstr.cpp`: `_ismbstrail`/`_ismbstrail_l`, compiled only with
//! `_MBCS` defined in the vendored build (the cmake list sets
//! `COMPILE_DEFINITIONS _MBCS` on this one file) -- that macro gates nothing
//! in the function body itself (only an `#error` guard at the top), so the
//! port carries no conditional behaviour from it.

use super::common::{is_mbb_lead, EINVAL};
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

unsafe fn validate_return(ok: bool, err: c_int) -> Option<c_int> {
    if !ok {
        *_errno() = err;
        _invalid_parameter_noinfo();
        return Some(0);
    }
    None
}

/// `_ismbstrail_l`.
///
/// # Safety
/// `string` and `current` must, if non-null, point into the same
/// NUL-terminated byte buffer with `string <= current`.
#[no_mangle]
pub unsafe extern "C" fn _ismbstrail_l(
    string: *const u8,
    current: *const u8,
    locale: *mut CrtLocalePointers,
) -> c_int {
    if let Some(r) = validate_return(!string.is_null(), EINVAL) {
        return r;
    }
    if let Some(r) = validate_return(!current.is_null(), EINVAL) {
        return r;
    }

    let mut lu = LocaleUpdate::new(locale);
    if (*lu.mbcinfo()).ismbcodepage == 0 {
        return 0;
    }

    let mut p = string;
    while p <= current && *p != 0 {
        if is_mbb_lead(*p, lu.locale_t()) {
            p = p.add(1);
            if p == current {
                return -1;
            }
            if *p == 0 {
                return 0;
            }
        }
        p = p.add(1);
    }

    0
}

/// `_ismbstrail`.
///
/// # Safety
/// Same as `_ismbstrail_l` with a null locale.
#[no_mangle]
pub unsafe extern "C" fn _ismbstrail(string: *const u8, current: *const u8) -> c_int {
    _ismbstrail_l(string, current, core::ptr::null_mut())
}
