//! `convert/gcvt.cpp`: `_gcvt_s`, `_gcvt`.

use crate::locale::Lconv;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;
/// `_CVTBUFSIZE`.
const CVTBUFSIZE: usize = 349;
/// `_CRT_INTERNAL_PRINTF_LEGACY_THREE_DIGIT_EXPONENTS` (corecrt_stdio_config.h).
const LEGACY_THREE_DIGIT_EXPONENTS: u64 = 1 << 4;
/// `__acrt_rounding_mode::legacy`.
const ROUNDING_LEGACY: c_int = 0;
/// `__acrt_precision_style::fixed`.
const PRECISION_STYLE_FIXED: c_int = 0;

#[repr(C)]
#[allow(dead_code)]
struct Strflt {
    sign: c_int,
    decpt: c_int,
    mantissa: *mut c_char,
}

extern "C" {
    fn __acrt_fltout(
        value: f64,
        precision: c_int,
        precision_style: c_int,
        result: *mut Strflt,
        buffer: *mut c_char,
        buffer_count: usize,
    ) -> c_int;
    /// Matches `output.rs`'s declaration exactly -- `*const u64`, not
    /// `*const f64`: the C's `double const*` is passed through as a raw bit
    /// pointer at this ABI boundary, and the two pointee types would
    /// otherwise clash under `#![deny(clashing_extern_declarations)]`.
    fn __acrt_fp_format(
        value: *const u64,
        result_buffer: *mut u8,
        result_buffer_count: usize,
        scratch_buffer: *mut u8,
        scratch_buffer_count: usize,
        format: c_int,
        precision: c_int,
        options: u64,
        rounding_mode: c_int,
        ptd: *mut PtdHost,
    ) -> c_int;
}

/// `_gcvt_s_internal`.
unsafe fn gcvt_s_internal(buffer: *mut c_char, buffer_count: usize, value: f64, precision: c_int, ptd: &mut PtdHost) -> c_int {
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
    if let Some(r) = super::common::validate_errcode(ptd, (precision as usize) < buffer_count, ERANGE) {
        return r;
    }

    let lconv = (*(*ptd.get_locale()).locinfo).lconv as *mut Lconv;
    let decimal_point = *(*lconv).decimal_point;

    // We only call `__acrt_fltout` to parse the exponent (`strflt.decpt`),
    // so pass a buffer just large enough for inf/nan/ind to avoid failure.
    const RESTRICTED_COUNT: usize = 7; // "1#SNAN" + NUL
    let mut result_string = [0 as c_char; RESTRICTED_COUNT];
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };
    __acrt_fltout(value, precision, PRECISION_STYLE_FIXED, &mut strflt, result_string.as_mut_ptr(), RESTRICTED_COUNT);

    let magnitude = strflt.decpt - 1;

    let mut scratch_buffer = [0u8; CVTBUFSIZE + 1];
    let e = if magnitude < -1 || magnitude > precision - 1 {
        // Ew.d where d = precision.
        __acrt_fp_format(
            (&value as *const f64).cast(),
            buffer.cast(),
            buffer_count,
            scratch_buffer.as_mut_ptr(),
            scratch_buffer.len(),
            b'e' as c_int,
            precision - 1,
            LEGACY_THREE_DIGIT_EXPONENTS,
            ROUNDING_LEGACY,
            ptd,
        )
    } else {
        // Fw.d where d = precision - decpt.
        __acrt_fp_format(
            (&value as *const f64).cast(),
            buffer.cast(),
            buffer_count,
            scratch_buffer.as_mut_ptr(),
            scratch_buffer.len(),
            b'f' as c_int,
            precision - strflt.decpt,
            LEGACY_THREE_DIGIT_EXPONENTS,
            ROUNDING_LEGACY,
            ptd,
        )
    };
    if e != 0 {
        return super::common::set_errno(ptd, e);
    }

    // Remove the trailing zeroes before the exponent.
    let mut p = buffer;
    while *p != 0 && *p != decimal_point {
        p = p.add(1);
    }
    if *p == 0 {
        return 0;
    }
    p = p.add(1);
    while *p != 0 && *p != b'e' as c_char {
        p = p.add(1);
    }
    let mut stop = p;
    p = p.sub(1);
    while *p == b'0' as c_char {
        p = p.sub(1);
    }
    loop {
        p = p.add(1);
        *p = *stop;
        stop = stop.add(1);
        if *p == 0 {
            break;
        }
    }

    0
}

/// `_gcvt_s`.
///
/// # Safety
/// `buffer` must have room for `buffer_count` bytes.
#[no_mangle]
pub unsafe extern "C" fn _gcvt_s(buffer: *mut c_char, buffer_count: usize, value: f64, precision: c_int) -> c_int {
    let mut ptd = PtdHost::new(core::ptr::null_mut());
    gcvt_s_internal(buffer, buffer_count, value, precision, &mut ptd)
}

/// `_gcvt`.
///
/// # Safety
/// `buffer` must have room for `_CRT_UNBOUNDED_BUFFER_SIZE` bytes -- the same
/// unchecked contract the C has (it calls `_gcvt_s` with `(size_t)-1`).
#[no_mangle]
pub unsafe extern "C" fn _gcvt(value: f64, precision: c_int, buffer: *mut c_char) -> *mut c_char {
    /// `_CRT_UNBOUNDED_BUFFER_SIZE` (corecrt_internal.h): `(size_t)-1 >> 1`.
    const CRT_UNBOUNDED_BUFFER_SIZE: usize = usize::MAX >> 1;
    let e = _gcvt_s(buffer, CRT_UNBOUNDED_BUFFER_SIZE, value, precision);
    if e != 0 {
        core::ptr::null_mut()
    } else {
        buffer
    }
}
