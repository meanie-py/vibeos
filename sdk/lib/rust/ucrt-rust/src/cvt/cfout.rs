//! `convert/cfout.cpp`: `__acrt_fltout`, binary -> decimal digit generation.
//!
//! Burger & Dybvig ("Printing floating point numbers quickly and accurately",
//! PLDI 1996), except that it does NOT stop at the shortest round-tripping
//! string: it keeps generating until the buffer is full, the requested %f/%e
//! precision is covered, or the remainder is zero -- so exactly representable
//! values (large powers of two) print exactly.

use super::{
    classify, errcheck, strcpy_s, Components, Strflt, DBL_EXPONENT_BIAS, DBL_MANTISSA_BITS, FP_FINITE,
    FP_INDETERMINATE, FP_INFINITY, FP_QUIET_NAN, FP_SIGNALING_NAN, NO_TRAILING, PRECISION_STYLE_FIXED, TRAILING,
};
use crate::big_integer::{divide, multiply_by_power_of_ten, multiply_small, shift_left, BigInteger, ELEMENT_BITS};
use core::ffi::{c_char, c_int, c_uint};

/// float.h `_MCW_EM`, `_DN_FLUSH`.
const CVT_MCW_EM: c_uint = 0x0008_001f;
const CVT_DN_FLUSH: c_uint = 0x0100_0000;
/// fenv.h `FE_ALL_EXCEPT`.
const CVT_FE_ALL_EXCEPT: u32 = 0x1f;
/// `__acrt_floating_type_traits<double>::minimum_binary_exponent` (DBL_MIN_EXP - 1).
const DBL_MINIMUM_BINARY_EXPONENT: i32 = -1022;
const INT_MAX: u32 = 0x7fff_ffff;

/// fenv.h `fenv_t`.
#[repr(C)]
struct FenvT {
    fe_ctl: u32,
    fe_stat: u32,
}

extern "C" {
    fn _controlfp_s(current: *mut c_uint, new_value: c_uint, mask: c_uint) -> c_int;
    fn fegetenv(env: *mut FenvT) -> c_int;
    fn feholdexcept(env: *mut FenvT) -> c_int;
    fn fesetenv(env: *const FenvT) -> c_int;
    fn log10(x: f64) -> f64;
    fn ceil(x: f64) -> f64;
}

/// `fp_control_word_guard(new_control, mask)`: sets the masked control bits,
/// restores the original ones (under the same mask) on drop.
struct FpControlWordGuard {
    original_control_word: c_uint,
    mask: c_uint,
}

impl FpControlWordGuard {
    unsafe fn new(new_control: c_uint, mask: c_uint) -> Self {
        let mut guard = Self { original_control_word: 0, mask };
        let mut float_control: c_uint = 0;
        _controlfp_s(&mut guard.original_control_word, 0, 0);
        _controlfp_s(&mut float_control, new_control, mask);
        guard
    }
}

impl Drop for FpControlWordGuard {
    fn drop(&mut self) {
        let mut reset_cw: c_uint = 0;
        unsafe { _controlfp_s(&mut reset_cw, self.original_control_word, self.mask) };
    }
}

/// `scoped_fp_state_reset`: only when some exception is UNMASKED does it pay for
/// `feholdexcept` now and `fesetenv` on drop.
///
/// The C's `fenv_t` is uninitialised before `fegetenv`. ucrtbase's
/// `fegetenv`/`feholdexcept`/`fesetenv` are still `-stub` exports (a DbgPrint
/// and `return 0`, filling nothing), so the C decides from stack garbage. Here
/// the record starts as the default, all-masked environment, so a stub
/// `fegetenv` leaves "nothing unmasked" and no save/restore is attempted; a
/// real one overwrites it and the logic is the C's.
struct ScopedFpStateReset {
    environment: FenvT,
    requires_reset: bool,
}

impl ScopedFpStateReset {
    unsafe fn new() -> Self {
        let mut s = Self { environment: FenvT { fe_ctl: CVT_FE_ALL_EXCEPT, fe_stat: 0 }, requires_reset: false };
        fegetenv(&mut s.environment);
        if s.environment.fe_ctl & CVT_FE_ALL_EXCEPT == CVT_FE_ALL_EXCEPT {
            s.requires_reset = false;
        } else {
            feholdexcept(&mut s.environment);
            s.requires_reset = true;
        }
        s
    }
}

impl Drop for ScopedFpStateReset {
    fn drop(&mut self) {
        if self.requires_reset {
            unsafe { fesetenv(&self.environment) };
        }
    }
}

/// `make_big_integer_power_of_two`.
fn power_of_two(power: u32) -> BigInteger {
    let mut x = BigInteger::zero();
    let element_index = power / ELEMENT_BITS;
    let bit_index = power % ELEMENT_BITS;
    x.data[element_index as usize] = 1u32 << bit_index;
    x.used = element_index + 1;
    x
}

/// `operator!=(big_integer, big_integer)`.
fn not_equal(lhs: &BigInteger, rhs: &BigInteger) -> bool {
    if lhs.used != rhs.used {
        return true;
    }
    for i in 0..lhs.used as usize {
        if lhs.data[i] != rhs.data[i] {
            return true;
        }
    }
    false
}

/// `convert_to_fos_high_precision<double>`. `bits` is a finite, positive,
/// nonzero double. Returns `__acrt_has_trailing_digits`.
unsafe fn convert_to_fos_high_precision(
    bits: u64,
    precision: u32,
    precision_style: c_int,
    exponent: *mut c_int,
    mantissa_buffer: *mut c_char,
    mantissa_buffer_count: usize,
) -> c_int {
    let components = Components::of(bits);

    // Denormals: no implicit bit, and one more step of exponent.
    let is_denormal = components.exponent == 0;
    let mantissa_adjustment: u64 = if is_denormal { 0 } else { 1u64 << (DBL_MANTISSA_BITS - 1) };
    let exponent_adjustment: i32 = if is_denormal { 2 } else { 1 };

    let f: u64 = components.fraction + mantissa_adjustment;
    let e: i32 = components.exponent as i32 - DBL_EXPONENT_BIAS - DBL_MANTISSA_BITS as i32 + exponent_adjustment;

    // First guess at the decimal exponent; may be one off, corrected below.
    // log10/ceil are the CRT's own (as the C calls them), taken through the C
    // ABI so i386's no-SSE Rust never returns a float by its own ABI.
    let value = core::mem::transmute::<u64, f64>(bits);
    let mut k: i32 = ceil(log10(value)) as i32;
    if k == i32::MAX || k == i32::MIN {
        k = 0;
    }

    let mut r = BigInteger::from_u64(f);
    let mut s;

    let hidden_bit = power_of_two(DBL_MANTISSA_BITS - 1);
    if e >= 0 {
        if not_equal(&r, &hidden_bit) {
            let _ = shift_left(&mut r, (e + 1) as u32); // f * b^e * 2
            s = BigInteger::from_u64(2); // 2
        } else {
            let _ = shift_left(&mut r, (e + 2) as u32); // f * b^(e+1) * 2
            s = BigInteger::from_u64(4); // b * 2
        }
    } else {
        // Microsoft compares the SCALED e against the unscaled minimum exponent
        // here; kept as written (it only picks which exact fraction is formed).
        if e == DBL_MINIMUM_BINARY_EXPONENT || not_equal(&r, &hidden_bit) {
            let _ = shift_left(&mut r, 1); // f * 2
            s = power_of_two((-e + 1) as u32); // b^-e * 2
        } else {
            let _ = shift_left(&mut r, 2); // f * b * 2
            s = power_of_two((-e + 2) as u32); // b^(-e+1) * 2
        }
    }

    if k >= 0 {
        let _ = multiply_by_power_of_ten(&mut s, k as u32);
    } else {
        let _ = multiply_by_power_of_ten(&mut r, k.wrapping_neg() as u32);
    }

    let mut mantissa_it = mantissa_buffer;

    // Trial digit: 10 means k was one too small, 0 one too large.
    let _ = multiply_small(&mut r, 10);
    let initial_digit = divide(&mut r, &s) as u32;

    if initial_digit == 10 {
        k += 1;
        *mantissa_it = b'1' as c_char;
        mantissa_it = mantissa_it.add(1);
        let _ = multiply_small(&mut s, 10);
    } else if initial_digit == 0 {
        k -= 1;
    } else {
        *mantissa_it = (b'0' as u32 + initial_digit) as u8 as c_char;
        mantissa_it = mantissa_it.add(1);
    }

    *exponent = k;

    // %f needs k more digits than %e for the same precision.
    let required_digits: u32 = if k >= 0 && precision <= INT_MAX && precision_style == PRECISION_STYLE_FIXED {
        (k as u32).wrapping_add(precision)
    } else {
        precision
    };

    let mantissa_last = mantissa_buffer.add(core::cmp::min(mantissa_buffer_count - 1, required_digits as usize));

    // Nonzero digits generated but not written count as trailing, exactly as a
    // nonzero remainder does.
    let mut unwritten_nonzero_digits_in_chunk = false;
    loop {
        if mantissa_it == mantissa_last {
            break;
        }
        if r.used == 0 {
            break;
        }

        // Nine digits per (expensive) division: 10^9 is the largest power of
        // ten a uint32_t quotient holds.
        const DIGITS_PER_ITERATION: u32 = 9;
        const DIGITS_PER_ITERATION_MULTIPLIER: u32 = 1000 * 1000 * 1000;

        let _ = multiply_small(&mut r, DIGITS_PER_ITERATION_MULTIPLIER);
        let mut quotient = divide(&mut r, &s) as u32;

        // Decompose into nine digits, lowest first. `room` is NEGATIVE when a
        // one-byte buffer let the initial digit step past `mantissa_last`.
        let room: isize = mantissa_last.offset_from(mantissa_it);
        let mut i = DIGITS_PER_ITERATION - 1;
        loop {
            let d = (b'0' as u32 + quotient % 10) as u8 as c_char;
            quotient /= 10;

            // `static_cast<uint32_t>(mantissa_last - mantissa_it) <= i`
            if room as u32 <= i {
                if d != b'0' as c_char {
                    unwritten_nonzero_digits_in_chunk = true;
                }
            } else {
                *mantissa_it.add(i as usize) = d;
            }

            if i == 0 {
                break;
            }
            i -= 1;
        }

        // `__min(uint32_t 9, ptrdiff_t room)`: the comparison happens in the
        // wider type. On 64-bit that is a signed ptrdiff_t (a negative room
        // steps BACK to `mantissa_last` and the loop ends); on i386 ptrdiff_t is
        // a 32-bit int converted to unsigned, so a negative room is huge and the
        // pointer moves on by 9. Only a one-byte buffer reaches this.
        #[cfg(target_pointer_width = "64")]
        let advance: isize = core::cmp::min(DIGITS_PER_ITERATION as isize, room);
        #[cfg(not(target_pointer_width = "64"))]
        let advance: isize = core::cmp::min(DIGITS_PER_ITERATION, room as u32) as isize;
        mantissa_it = mantissa_it.offset(advance);
    }

    *mantissa_it = 0;

    if r.used == 0 && !unwritten_nonzero_digits_in_chunk {
        return NO_TRAILING;
    }
    TRAILING
}

/// `__acrt_fltout` (corecrt_internal_fltintrn.h). Fills `flt` (sign character,
/// decimal exponent, `mantissa = result`) and writes precision + 1 digits of a
/// finite value into `result`, or the legacy `1#INF`-style spelling of a
/// non-finite one. Returns `__acrt_has_trailing_digits`.
///
/// `value` is a plain double, not the C's `_CRT_DOUBLE` struct: see the module
/// doc. On i386 it is taken as its 8 BITS (the same stack slot a cdecl double
/// occupies): a no-SSE build moves an `f64` through the x87 stack, whose `fld`
/// turns a signaling NaN quiet, and "1#SNAN" would read "1#QNAN".
/// `precision` is `unsigned` in the C; the callers declare `int`, same bits.
///
/// # Safety
/// `flt` must be writable; `result` must hold `result_count` (>= 1) bytes.
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_fltout(
    value: u64,
    precision: c_int,
    precision_style: c_int,
    flt: *mut Strflt,
    result: *mut c_char,
    result_count: usize,
) -> c_int {
    fltout_bits(value, precision as u32, precision_style, flt, result, result_count)
}

/// `__acrt_fltout` -- see the i386 twin above.
///
/// # Safety
/// `flt` must be writable; `result` must hold `result_count` (>= 1) bytes.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_fltout(
    value: f64,
    precision: c_int,
    precision_style: c_int,
    flt: *mut Strflt,
    result: *mut c_char,
    result_count: usize,
) -> c_int {
    fltout_bits(core::mem::transmute::<f64, u64>(value), precision as u32, precision_style, flt, result, result_count)
}

/// The body of `__acrt_fltout`, on the double's bits (the in-crate callers,
/// `format.rs`, hold bits and never materialise an `f64`).
pub(super) unsafe fn fltout_bits(
    mut bits: u64,
    precision: u32,
    precision_style: c_int,
    flt: *mut Strflt,
    result: *mut c_char,
    result_count: usize,
) -> c_int {
    let _reset_fp_state = ScopedFpStateReset::new();

    let components = Components::of(bits);

    (*flt).sign = if components.sign == 1 { b'-' as c_int } else { b' ' as c_int };
    (*flt).mantissa = result;

    let mut float_control: c_uint = 0;
    _controlfp_s(&mut float_control, 0, 0);
    let value_is_zero =
        components.exponent == 0 && (components.fraction == 0 || float_control & CVT_DN_FLUSH != 0);
    if value_is_zero {
        (*flt).decpt = 0;
        errcheck(strcpy_s(result.cast(), result_count, b"0\0".as_ptr()));
        return NO_TRAILING;
    }

    let classification = classify(bits);
    if classification != FP_FINITE {
        (*flt).decpt = 1;
    }

    match classification {
        FP_INFINITY => {
            errcheck(strcpy_s(result.cast(), result_count, b"1#INF\0".as_ptr()));
            return TRAILING;
        }
        FP_QUIET_NAN => {
            errcheck(strcpy_s(result.cast(), result_count, b"1#QNAN\0".as_ptr()));
            return NO_TRAILING;
        }
        FP_SIGNALING_NAN => {
            errcheck(strcpy_s(result.cast(), result_count, b"1#SNAN\0".as_ptr()));
            return NO_TRAILING;
        }
        FP_INDETERMINATE => {
            errcheck(strcpy_s(result.cast(), result_count, b"1#IND\0".as_ptr()));
            return NO_TRAILING;
        }
        _ => {}
    }

    // Make the value positive for the digit generator.
    bits &= !(1u64 << 63);

    // One extra digit, so the caller can round.
    let _fpc = FpControlWordGuard::new(CVT_MCW_EM, CVT_MCW_EM);
    convert_to_fos_high_precision(
        bits,
        precision.wrapping_add(1),
        precision_style,
        &mut (*flt).decpt,
        result,
        result_count,
    )
}
