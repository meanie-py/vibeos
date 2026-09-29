//! `convert/_fptostr.cpp`: `__acrt_fp_strflt_to_string` -- copies a STRFLT's
//! digits into a buffer, rounds at the requested digit, and moves the decimal
//! point when the rounding carries out of the top digit. The STRFLT's mantissa
//! is left untouched (its callers rely on that); only `decpt` may change.

use super::{
    strlen, validate, Strflt, CVT_FE_DOWNWARD, CVT_FE_TONEAREST, CVT_FE_UPWARD, EINVAL, ERANGE,
    ROUNDING_LEGACY, TRAILING,
};
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

/// `check_trailing`: "no trailing" from the generator only says nothing lay
/// beyond the generated digits; rounding earlier than their end must also see
/// that the generated digits after the rounding position are all zero.
unsafe fn check_trailing(mut mantissa_it: *const c_char, trailing_digits: c_int) -> bool {
    if trailing_digits == TRAILING {
        return true;
    }
    while *mantissa_it == b'0' as c_char {
        mantissa_it = mantissa_it.add(1);
    }
    *mantissa_it != 0
}

/// `should_round_up`. `mantissa_it` points at the first digit NOT copied.
unsafe fn should_round_up(
    mantissa_base: *const c_char,
    mantissa_it: *const c_char,
    sign: c_int,
    trailing_digits: c_int,
    rounding_mode: c_int,
) -> bool {
    if rounding_mode == ROUNDING_LEGACY {
        return *mantissa_it >= b'5' as c_char;
    }

    let round_mode = super::fegetround();

    if round_mode == CVT_FE_TONEAREST {
        if *mantissa_it > b'5' as c_char {
            return true;
        }
        if *mantissa_it < b'5' as c_char {
            return false;
        }
        // .5000000001 -- something nonzero follows the 5: round up.
        if check_trailing(mantissa_it.add(1), trailing_digits) {
            return true;
        }
        // Exactly half: ties to even. No preceding digit counts as zero.
        if mantissa_it == mantissa_base {
            return false;
        }
        return *mantissa_it.sub(1) % 2 != 0;
    }

    if round_mode == CVT_FE_UPWARD {
        return check_trailing(mantissa_it, trailing_digits) && sign != b'-' as c_int;
    }

    if round_mode == CVT_FE_DOWNWARD {
        return check_trailing(mantissa_it, trailing_digits) && sign == b'-' as c_int;
    }

    false
}

/// `__acrt_fp_strflt_to_string` (corecrt_internal_fltintrn.h). `buffer` gets
/// `digits` digits (zero-padded past the mantissa's end), rounded; a carry out
/// of the top digit increments `pflt->decpt` instead of growing the string.
///
/// # Safety
/// `buffer` must hold `buffer_count` bytes; `pflt`, if non-null, must point at a
/// STRFLT whose mantissa is NUL-terminated; `ptd` must be a live host.
#[no_mangle]
pub unsafe extern "C" fn __acrt_fp_strflt_to_string(
    buffer: *mut c_char,
    buffer_count: usize,
    mut digits: c_int,
    pflt: *mut Strflt,
    trailing_digits: c_int,
    rounding_mode: c_int,
    ptd: *mut PtdHost,
) -> c_int {
    let ptd = &mut *ptd;
    if let Some(e) = validate(ptd, !buffer.is_null(), EINVAL) {
        return e;
    }
    if let Some(e) = validate(ptd, buffer_count > 0, EINVAL) {
        return e;
    }
    *buffer = 0;

    // `static_cast<size_t>((digits > 0 ? digits : 0) + 1)`: int arithmetic first.
    let minimum = (if digits > 0 { digits } else { 0 }).wrapping_add(1) as isize as usize;
    if let Some(e) = validate(ptd, buffer_count > minimum, ERANGE) {
        return e;
    }
    if let Some(e) = validate(ptd, !pflt.is_null(), EINVAL) {
        return e;
    }

    let mut buffer_it = buffer;
    let mantissa_base: *const c_char = (*pflt).mantissa;
    let mut mantissa_it: *const c_char = (*pflt).mantissa;

    // The first position is the overflow digit for a carry (9.99 -> 10.00):
    // '0', not NUL.
    *buffer_it = b'0' as c_char;
    buffer_it = buffer_it.add(1);

    while digits > 0 {
        if *mantissa_it != 0 {
            *buffer_it = *mantissa_it;
            mantissa_it = mantissa_it.add(1);
        } else {
            *buffer_it = b'0' as c_char;
        }
        buffer_it = buffer_it.add(1);
        digits -= 1;
    }
    *buffer_it = 0;

    // digits < 0: the rounding digit is not part of the output at all.
    if digits >= 0 && should_round_up(mantissa_base, mantissa_it, (*pflt).sign, trailing_digits, rounding_mode) {
        buffer_it = buffer_it.sub(1);
        while *buffer_it == b'9' as c_char {
            *buffer_it = b'0' as c_char;
            buffer_it = buffer_it.sub(1);
        }
        *buffer_it += 1;
    }

    if *buffer == b'1' as c_char {
        // The carry reached the overflow digit.
        (*pflt).decpt += 1;
    } else {
        // Drop the unused overflow digit.
        let tail = buffer.add(1);
        core::ptr::copy(tail, buffer, strlen(tail) + 1);
    }

    0
}
