//! `convert/fcvt.cpp`: `_fcvt_s`, `_fcvt`, `_ecvt_s`, `_ecvt`.
//!
//! The float-formatting engine itself (`__acrt_fltout`, `cfout.cpp`;
//! `__acrt_fp_strflt_to_string`, `_fptostr.cpp`) stays vendored C -- it is
//! also `output.rs`'s printf engine's dependency (`__acrt_fp_format`,
//! `cvt.cpp`), so taking it over is that lane's call, not this batch's. This
//! file only ports the orchestration around it: the per-thread `_cvtbuf`
//! buffer and the two families' argument shaping.

use super::common::__acrt_getptd_noexit;
use crate::ptd::AcrtPtd;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

/// `_CVTBUFSIZE` (corecrt_internal_fltintrn.h): 309 + 40.
const CVTBUFSIZE: usize = 349;
const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

/// `_strflt` / `STRFLT` (corecrt_internal_fltintrn.h:162). `mantissa` is
/// layout-required (the C internals index into it) but never read directly
/// here -- `__acrt_fp_strflt_to_string` writes the formatted digits straight
/// into our output `buffer` instead.
#[repr(C)]
#[allow(dead_code)]
struct Strflt {
    sign: c_int,
    decpt: c_int,
    mantissa: *mut c_char,
}

extern "C" {
    /// `__acrt_fltout` (cfout.cpp) -- `extern "C"` in the header despite its
    /// C++ reference/enum-class parameters, since a C++ reference is
    /// ABI-identical to a pointer and an `enum class` with no explicit
    /// underlying type is `int`-sized.
    fn __acrt_fltout(
        value: f64,
        precision: c_int,
        precision_style: c_int,
        result: *mut Strflt,
        buffer: *mut c_char,
        buffer_count: usize,
    ) -> c_int;
    /// `__acrt_fp_strflt_to_string` (_fptostr.cpp).
    fn __acrt_fp_strflt_to_string(
        buffer: *mut c_char,
        buffer_count: usize,
        digits: c_int,
        value: *mut Strflt,
        trailing_digits: c_int,
        rounding_mode: c_int,
        ptd: *mut PtdHost,
    ) -> c_int;
}

/// `__acrt_precision_style::fixed` (corecrt_internal_fltintrn.h).
const PRECISION_STYLE_FIXED: c_int = 0;
/// `__acrt_has_trailing_digits::trailing`.
const TRAILING: c_int = 0;
/// `__acrt_rounding_mode::legacy`.
const ROUNDING_LEGACY: c_int = 0;

/// `try_get_ptd_buffer`: the per-thread `_cvtbuf`, lazily allocated. Returns
/// null on failure (either no ptd, or the allocation failed), exactly as the
/// C does -- callers turn that into a null return, not an error code.
///
/// # Safety
/// Must be called with the CRT initialised.
unsafe fn try_get_ptd_buffer(_ptd: &mut PtdHost) -> *mut c_char {
    let raw_ptd: *mut AcrtPtd = __acrt_getptd_noexit();
    if raw_ptd.is_null() {
        return core::ptr::null_mut();
    }
    if !(*raw_ptd).cvtbuf.is_null() {
        return (*raw_ptd).cvtbuf.cast();
    }
    extern "C" {
        fn malloc(size: usize) -> *mut core::ffi::c_void;
    }
    let buf = malloc(CVTBUFSIZE) as *mut c_char;
    (*raw_ptd).cvtbuf = buf.cast();
    buf
}

/// `internal_to_string`.
unsafe fn internal_to_string(
    buffer: *mut c_char,
    buffer_count: usize,
    strflt: *mut Strflt,
    requested_digits: c_int,
    decimal_point: *mut c_int,
    sign: *mut c_int,
    ptd: &mut PtdHost,
) -> c_int {
    let minimum_buffer_count = (if requested_digits > 0 { requested_digits } else { 0 }) as usize + 2;
    if let Some(r) = super::common::validate_errcode(ptd, buffer_count >= minimum_buffer_count, ERANGE) {
        return r;
    }

    let capped_digits = core::cmp::min(requested_digits, (buffer_count - 2) as c_int);

    let e = __acrt_fp_strflt_to_string(buffer, buffer_count, capped_digits, strflt, TRAILING, ROUNDING_LEGACY, ptd);
    if e != 0 {
        return super::common::set_errno(ptd, e);
    }

    // `strflt->sign == '-'`: __acrt_fltout stores the CHARACTER '-' or ' ' (the header's
    // "zero if positive" is wrong), so a non-zero test called every positive value negative.
    *sign = if (*strflt).sign == b'-' as c_int { 1 } else { 0 };
    *decimal_point = (*strflt).decpt;
    0
}

/// `_fcvt_s_internal`.
unsafe fn fcvt_s_internal(
    buffer: *mut c_char,
    buffer_count: usize,
    value: f64,
    requested_digits: c_int,
    decimal_point: *mut c_int,
    sign: *mut c_int,
    ptd: &mut PtdHost,
) -> c_int {
    if let Some(r) = super::common::validate_errcode(ptd, !buffer.is_null(), EINVAL) {
        return r;
    }
    if let Some(r) = super::common::validate_errcode(ptd, buffer_count > 0, EINVAL) {
        return r;
    }
    // `_RESET_STRING`: `*s = 0` plus `_FILL_STRING`, which is a no-op in this release
    // build -- ONE element, not the whole count (the whole count is a memset; and
    // _gcvt passes (size_t)-1 >> 1 as its size).
    *buffer = 0;
    if let Some(r) = super::common::validate_errcode(ptd, !decimal_point.is_null(), EINVAL) {
        return r;
    }
    if let Some(r) = super::common::validate_errcode(ptd, !sign.is_null(), EINVAL) {
        return r;
    }

    let mut result_string = [0 as c_char; CVTBUFSIZE + 1];
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };
    __acrt_fltout(value, result_string.len() as c_int, PRECISION_STYLE_FIXED, &mut strflt, result_string.as_mut_ptr(), result_string.len());

    let actual_digits = strflt.decpt + requested_digits;
    let buffer_insufficiently_large = requested_digits > 0 && strflt.decpt > 0 && actual_digits < requested_digits;
    let capped_digits = if buffer_insufficiently_large { c_int::MAX } else { actual_digits };

    internal_to_string(buffer, buffer_count, &mut strflt, capped_digits, decimal_point, sign, ptd)
}

/// `_fcvt_s`.
///
/// # Safety
/// `buffer` must have room for `buffer_count` bytes; `decimal_point`/`sign`
/// must be writable.
#[no_mangle]
pub unsafe extern "C" fn _fcvt_s(
    buffer: *mut c_char,
    buffer_count: usize,
    value: f64,
    requested_digits: c_int,
    decimal_point: *mut c_int,
    sign: *mut c_int,
) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    fcvt_s_internal(buffer, buffer_count, value, requested_digits, decimal_point, sign, &mut ptd)
}

/// `_fcvt`.
///
/// # Safety
/// `decimal_point`/`sign` must be writable. Returns null (rather than the
/// per-thread buffer) on allocation failure, faithfully.
#[no_mangle]
pub unsafe extern "C" fn _fcvt(value: f64, requested_digits: c_int, decimal_point: *mut c_int, sign: *mut c_int) -> *mut c_char {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    let buffer = try_get_ptd_buffer(&mut ptd);
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    let mut result_string = [0 as c_char; CVTBUFSIZE + 1];
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };
    __acrt_fltout(value, result_string.len() as c_int, PRECISION_STYLE_FIXED, &mut strflt, result_string.as_mut_ptr(), result_string.len());

    let capped_digits = core::cmp::min(requested_digits, CVTBUFSIZE as c_int - 2 - strflt.decpt);

    let status = fcvt_s_internal(buffer, CVTBUFSIZE, value, capped_digits, decimal_point, sign, &mut ptd);
    if status != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}

/// `_ecvt_s_internal`.
unsafe fn ecvt_s_internal(
    buffer: *mut c_char,
    buffer_count: usize,
    value: f64,
    requested_digits: c_int,
    decimal_point: *mut c_int,
    sign: *mut c_int,
    ptd: &mut PtdHost,
) -> c_int {
    if let Some(r) = super::common::validate_errcode(ptd, !buffer.is_null(), EINVAL) {
        return r;
    }
    if let Some(r) = super::common::validate_errcode(ptd, buffer_count > 0, EINVAL) {
        return r;
    }
    // `_RESET_STRING`: `*s = 0` plus `_FILL_STRING`, which is a no-op in this release
    // build -- ONE element, not the whole count (the whole count is a memset; and
    // _gcvt passes (size_t)-1 >> 1 as its size).
    *buffer = 0;
    if let Some(r) = super::common::validate_errcode(ptd, !decimal_point.is_null(), EINVAL) {
        return r;
    }
    if let Some(r) = super::common::validate_errcode(ptd, !sign.is_null(), EINVAL) {
        return r;
    }

    let mut result_string = [0 as c_char; CVTBUFSIZE + 1];
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };
    __acrt_fltout(value, result_string.len() as c_int, PRECISION_STYLE_FIXED, &mut strflt, result_string.as_mut_ptr(), result_string.len());

    let e = internal_to_string(buffer, buffer_count, &mut strflt, requested_digits, decimal_point, sign, ptd);

    let capped_digits = core::cmp::min(requested_digits, (buffer_count - 2) as c_int);
    if capped_digits >= 0 && *buffer.add(capped_digits as usize) != 0 {
        *buffer.add(capped_digits as usize) = 0;
    }

    e
}

/// `_ecvt_s`.
///
/// # Safety
/// Same as `_fcvt_s`.
#[no_mangle]
pub unsafe extern "C" fn _ecvt_s(
    buffer: *mut c_char,
    buffer_count: usize,
    value: f64,
    requested_digits: c_int,
    decimal_point: *mut c_int,
    sign: *mut c_int,
) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    ecvt_s_internal(buffer, buffer_count, value, requested_digits, decimal_point, sign, &mut ptd)
}

/// `_ecvt`.
///
/// # Safety
/// Same as `_fcvt`.
#[no_mangle]
pub unsafe extern "C" fn _ecvt(value: f64, requested_digits: c_int, decimal_point: *mut c_int, sign: *mut c_int) -> *mut c_char {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    let buffer = try_get_ptd_buffer(&mut ptd);
    if buffer.is_null() {
        return core::ptr::null_mut();
    }

    let capped_digits = core::cmp::min(requested_digits, CVTBUFSIZE as c_int - 2);

    let e = ecvt_s_internal(buffer, CVTBUFSIZE, value, capped_digits, decimal_point, sign, &mut ptd);
    if e != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}
