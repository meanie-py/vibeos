//! `convert/cvt.cpp`: `__acrt_fp_format`, the `%a %e %f %g` formatters behind
//! printf and `_gcvt`.
//!
//! Every formatter works IN PLACE in the caller's result buffer: the digits
//! are rounded into it (leaving room for sign and decimal point), then sign,
//! point, padding zeros and exponent are shifted and poked around them. Buffer
//! sizes are the caller's contract, checked only where the C checks them;
//! `_CRT_UNBOUNDED_BUFFER_SIZE` (SIZE_MAX) passes through unreduced.

use super::cfout::fltout_bits;
use super::fptostr::__acrt_fp_strflt_to_string;
use super::{
    classify, errcheck, strcpy_s, strlen, validate, Components, Strflt, CVT_FE_DOWNWARD, CVT_FE_TONEAREST,
    CVT_FE_UPWARD, DBL_EXPONENT_BIAS, DBL_EXPONENT_MASK, DBL_MANTISSA_BITS, EINVAL, ENOMEM, ERANGE, FP_FINITE,
    PRECISION_STYLE_FIXED, PRECISION_STYLE_SCIENTIFIC, ROUNDING_LEGACY,
};
use crate::locale::Lconv;
use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

/// `_CRT_UNBOUNDED_BUFFER_SIZE`.
const UNBOUNDED_BUFFER_SIZE: usize = usize::MAX;

/// corecrt_stdio_config.h option bits `__acrt_fp_format` reads.
const PRINTF_LEGACY_MSVCRT_COMPATIBILITY: u64 = 1 << 3;
const PRINTF_LEGACY_THREE_DIGIT_EXPONENTS: u64 = 1 << 4;
const PRINTF_STANDARD_ROUNDING: u64 = 1 << 5;

/// `shift_bytes`: move the C string at `string` (and its NUL) `distance` bytes
/// to the right (left if negative). The buffer bounds are annotations only.
unsafe fn shift_bytes(string: *mut u8, distance: c_int) {
    if distance != 0 {
        core::ptr::copy(string, string.offset(distance as isize), strlen(string.cast()) + 1);
    }
}

/// `*ptd.get_locale()->locinfo->lconv->decimal_point`.
unsafe fn decimal_point(ptd: &mut PtdHost) -> u8 {
    let locale = ptd.get_locale();
    let lconv = (*(*locale).locinfo).lconv as *mut Lconv;
    *(*lconv).decimal_point as u8
}

// ── NaN and infinity ───────────────────────────────────────────────────────

/// `strings[][4]`: per class (infinity, quiet, signaling, indeterminate) the
/// long and short upper-case forms, then the long and short lower-case ones.
static NAN_INF_STRINGS: [[&[u8]; 4]; 4] = [
    [b"INF\0", b"INF\0", b"inf\0", b"inf\0"],
    [b"NAN\0", b"NAN\0", b"nan\0", b"nan\0"],
    [b"NAN(SNAN)\0", b"NAN\0", b"nan(snan)\0", b"nan\0"],
    [b"NAN(IND)\0", b"NAN\0", b"nan(ind)\0", b"nan\0"],
];

/// `fp_format_nan_or_infinity`.
unsafe fn fp_format_nan_or_infinity(
    classification: u32,
    is_negative: bool,
    mut result_buffer: *mut u8,
    mut result_buffer_count: usize,
    use_capitals: bool,
) -> c_int {
    // `_countof("INF") + is_negative`: room for "INF", its NUL, and the sign.
    if result_buffer_count < 4 + is_negative as usize {
        *result_buffer = 0;
        return ENOMEM;
    }

    if is_negative {
        *result_buffer = b'-';
        result_buffer = result_buffer.add(1);
        *result_buffer = 0;
        if result_buffer_count != UNBOUNDED_BUFFER_SIZE {
            result_buffer_count -= 1;
        }
    }

    let row = (classification - 1) as usize;
    let column = if use_capitals { 0 } else { 2 };

    let long_string_will_fit = result_buffer_count > strlen(NAN_INF_STRINGS[row][column].as_ptr().cast());
    let chosen = NAN_INF_STRINGS[row][column + !long_string_will_fit as usize];
    errcheck(strcpy_s(result_buffer, result_buffer_count, chosen.as_ptr()));
    0
}

// ── %e ─────────────────────────────────────────────────────────────────────

/// `fp_format_e_internal`: `[-]d.ddde(+/-)ddd` from digits already rounded into
/// the buffer (at offset sign + point for %e, at offset sign for %g, which is
/// why `g_fmt` shifts them right first).
#[allow(clippy::too_many_arguments)]
unsafe fn fp_format_e_internal(
    result_buffer: *mut u8,
    result_buffer_count: usize,
    precision: c_int,
    capitals: bool,
    min_exponent_digits: u32,
    pflt: *mut Strflt,
    g_fmt: bool,
    ptd: &mut PtdHost,
) -> c_int {
    // sign + first digit + point (3), the digits, e+ddd (5), a rounding spare (1).
    let minimum = 3i32.wrapping_add(if precision > 0 { precision } else { 0 }).wrapping_add(5 + 1) as isize as usize;
    if let Some(e) = validate(ptd, result_buffer_count > minimum, ERANGE) {
        return e;
    }

    let negative = (*pflt).sign == b'-' as c_int;

    if g_fmt {
        // Shift right one place for the decimal point, if there is one.
        let p = result_buffer.add(negative as usize);
        shift_bytes(p, (precision > 0) as c_int);
    }

    let mut p = result_buffer;

    if negative {
        *p = b'-';
        p = p.add(1);
    }

    // Move the first digit into the slot left for it, the point into its place.
    if precision > 0 {
        *p = *p.add(1);
        p = p.add(1);
        *p = decimal_point(ptd);
    }

    // The end of the digits: attach the exponent field.
    p = p.offset(precision as isize + if g_fmt { 0 } else { 1 });
    let remaining = if result_buffer_count == UNBOUNDED_BUFFER_SIZE {
        result_buffer_count
    } else {
        result_buffer_count.wrapping_sub(p.offset_from(result_buffer) as usize)
    };
    errcheck(strcpy_s(p, remaining, b"e+000\0".as_ptr()));
    let exponentpos = p.add(2);

    if capitals {
        *p = b'E';
    }

    p = p.add(1);

    // A zero mantissa is the value 0: exponent stays +000.
    if *(*pflt).mantissa != b'0' as c_char {
        let mut exp = (*pflt).decpt - 1;
        if exp < 0 {
            exp = -exp;
            *p = b'-';
        }

        p = p.add(1);

        if exp >= 100 {
            *p = (*p).wrapping_add((exp / 100) as u8);
            exp %= 100;
        }

        p = p.add(1);

        if exp >= 10 {
            *p = (*p).wrapping_add((exp / 10) as u8);
            exp %= 10;
        }

        p = p.add(1);
        *p = (*p).wrapping_add(exp as u8);
    }

    // Two-digit exponents drop a leading zero when there is one to drop.
    if min_exponent_digits == 2 && *exponentpos == b'0' {
        core::ptr::copy(exponentpos.add(1), exponentpos, 3);
    }

    0
}

/// `fp_format_e`.
#[allow(clippy::too_many_arguments)]
unsafe fn fp_format_e(
    argument: *const u64,
    result_buffer: *mut u8,
    result_buffer_count: usize,
    scratch_buffer: *mut u8,
    scratch_buffer_count: usize,
    precision: c_int,
    capitals: bool,
    min_exponent_digits: u32,
    rounding_mode: c_int,
    ptd: &mut PtdHost,
) -> c_int {
    // precision + 1 digits in all: the one left of the point is extra.
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };

    let trailing_digits = fltout_bits(
        *argument,
        precision.wrapping_add(1) as u32,
        PRECISION_STYLE_SCIENTIFIC,
        &mut strflt,
        scratch_buffer.cast(),
        scratch_buffer_count,
    );

    let negative = (strflt.sign == b'-' as c_int) as usize;
    let has_point = (precision > 0) as usize;
    let e = __acrt_fp_strflt_to_string(
        result_buffer.add(negative + has_point).cast(),
        if result_buffer_count == UNBOUNDED_BUFFER_SIZE {
            result_buffer_count
        } else {
            result_buffer_count.wrapping_sub(negative).wrapping_sub(has_point)
        },
        precision.wrapping_add(1),
        &mut strflt,
        trailing_digits,
        rounding_mode,
        ptd,
    );

    if e != 0 {
        *result_buffer = 0;
        return e;
    }

    fp_format_e_internal(result_buffer, result_buffer_count, precision, capitals, min_exponent_digits, &mut strflt, false, ptd)
}

// ── %a ─────────────────────────────────────────────────────────────────────

/// `fe_to_nearest`: round-to-nearest on the hex digit under `mask`, ties to
/// even on the digit above it (the implicit leading bit above the first).
fn fe_to_nearest(bits: u64, mask: u64, maskpos: i16) -> bool {
    let c = Components::of(bits);

    let mut digit = ((c.fraction & mask) >> maskpos) as u16;

    if digit > 8 {
        return true;
    }
    if digit < 8 {
        return false;
    }

    let round_bits_mask = (1u64 << maskpos) - 1;
    if c.fraction & round_bits_mask != 0 {
        return true;
    }

    // DBL_MANT_DIG - 5: the first hex digit, whose left neighbour is the
    // implicit bit (0 for a denormal).
    if maskpos != (DBL_MANTISSA_BITS - 5) as i16 {
        digit = (((c.fraction / 16) & mask) >> maskpos) as u16;
    } else {
        digit = if c.exponent == 0 { 0 } else { 1 };
    }

    digit % 2 == 1
}

/// `should_round_up` (the %a one). Upward/downward look at the next hex digit
/// only, not the bits below it -- Microsoft's, kept.
unsafe fn should_round_up_hex(bits: u64, mask: u64, maskpos: i16, rounding_mode: c_int) -> bool {
    let c = Components::of(bits);

    let digit = ((c.fraction & mask) >> maskpos) as u16;

    if rounding_mode == ROUNDING_LEGACY {
        return digit >= 8;
    }
    let round_mode = super::fegetround();

    if round_mode == CVT_FE_TONEAREST {
        return fe_to_nearest(bits, mask, maskpos);
    }
    if round_mode == CVT_FE_UPWARD {
        return digit != 0 && c.sign == 0;
    }
    if round_mode == CVT_FE_DOWNWARD {
        return digit != 0 && c.sign != 0;
    }
    false
}

/// `fp_format_a`: `[-]0xh.hhhhp(+/-)d`. The `0x` is printf's, not ours: this
/// writes from the leading digit on.
#[allow(clippy::too_many_arguments)]
unsafe fn fp_format_a(
    argument: *const u64,
    mut result_buffer: *mut u8,
    result_buffer_count: usize,
    scratch_buffer: *mut u8,
    scratch_buffer_count: usize,
    mut precision: c_int,
    capitals: bool,
    min_exponent_digits: u32,
    rounding_mode: c_int,
    ptd: &mut PtdHost,
) -> c_int {
    if precision < 0 {
        precision = 0;
    }

    *result_buffer = 0;

    // sign (1), "0xh." (4), the digits, "p+0000" (6) -- int arithmetic first.
    let minimum = (1i32 + 4).wrapping_add(precision).wrapping_add(6) as isize as usize;
    if let Some(e) = validate(ptd, result_buffer_count > minimum, ERANGE) {
        return e;
    }

    let bits = *argument;
    let c = Components::of(bits);

    // Special values (reached only in legacy-msvcrt mode): %e's spelling with
    // the 'e' turned into 'p' and the exponent cut to one digit.
    if c.exponent == DBL_EXPONENT_MASK {
        let e = fp_format_e(
            argument,
            result_buffer,
            result_buffer_count,
            scratch_buffer,
            scratch_buffer_count,
            precision,
            false,
            min_exponent_digits,
            rounding_mode,
            ptd,
        );

        if e != 0 {
            *result_buffer = 0;
            return e;
        }

        let mut p = strrchr(result_buffer, b'e' as c_int);
        if !p.is_null() {
            *p = if capitals { b'P' } else { b'p' };
            // Skip the sign and the first digit, terminate.
            p = p.add(3);
            *p = 0;
        }
        return e;
    }

    if c.sign != 0 {
        *result_buffer = b'-';
        result_buffer = result_buffer.add(1);
    }

    let hexadd: c_int = (if capitals { b'A' } else { b'a' }) as c_int - b'9' as c_int - 1;

    // Leading digit, and the bias to take off the exponent.
    let mut debias: u64 = DBL_EXPONENT_BIAS as u64;
    if c.exponent == 0 {
        *result_buffer = b'0';
        result_buffer = result_buffer.add(1);
        if c.fraction == 0 {
            debias = 0; // zero
        } else {
            debias -= 1; // denormal
        }
    } else {
        *result_buffer = b'1';
        result_buffer = result_buffer.add(1);
    }

    // The decimal point; a 0 in its slot marks "no point".
    let mut pos = result_buffer;
    result_buffer = result_buffer.add(1);
    if precision == 0 {
        *pos = 0;
    } else {
        *pos = decimal_point(ptd);
    }

    if c.fraction > 0 {
        // Four bits at a time, from the top of the fraction.
        let mut maskpos: i16 = (DBL_MANTISSA_BITS as i16 - 1) - 4;
        let mut mask: u64 = 0xf;
        mask <<= maskpos;

        while maskpos >= 0 && precision > 0 {
            let mut digit = ((c.fraction & mask) >> maskpos) as u16;
            digit += b'0' as u16;
            if digit > b'9' as u16 {
                digit = digit.wrapping_add(hexadd as u16);
            }
            *result_buffer = digit as u8;
            result_buffer = result_buffer.add(1);
            mask >>= 4;
            maskpos -= 4;
            precision -= 1;
        }

        // Digits left over: round.
        if maskpos >= 0 && should_round_up_hex(bits, mask, maskpos, rounding_mode) {
            let mut p = result_buffer.sub(1);
            // Carry through trailing f's.
            while *p == b'f' || *p == b'F' {
                *p = b'0';
                p = p.sub(1);
            }
            if p != pos {
                if *p == b'9' {
                    *p = (*p).wrapping_add((1 + hexadd) as u8);
                } else {
                    *p += 1;
                }
            } else {
                // Carried up to the point (0x0.fff -> 0x1.000): the leading
                // digit is 0 or 1, so no hexadd.
                p = p.sub(1);
                *p += 1;
            }
        }
    }

    // Pad with zeros.
    while precision > 0 {
        *result_buffer = b'0';
        result_buffer = result_buffer.add(1);
        precision -= 1;
    }

    // No point: back up over its slot.
    if *pos == 0 {
        result_buffer = pos;
    }

    // Exponent, minimum digits.
    *result_buffer = if capitals { b'P' } else { b'p' };
    result_buffer = result_buffer.add(1);
    let mut exponent: i64 = c.exponent.wrapping_sub(debias) as i64;
    if exponent >= 0 {
        *result_buffer = b'+';
        result_buffer = result_buffer.add(1);
    } else {
        *result_buffer = b'-';
        result_buffer = result_buffer.add(1);
        exponent = -exponent;
    }
    pos = result_buffer;
    *pos = b'0';

    if exponent >= 1000 {
        *result_buffer = b'0' + (exponent / 1000) as u8;
        result_buffer = result_buffer.add(1);
        exponent %= 1000;
    }
    if result_buffer != pos || exponent >= 100 {
        *result_buffer = b'0' + (exponent / 100) as u8;
        result_buffer = result_buffer.add(1);
        exponent %= 100;
    }
    if result_buffer != pos || exponent >= 10 {
        *result_buffer = b'0' + (exponent / 10) as u8;
        result_buffer = result_buffer.add(1);
        exponent %= 10;
    }

    *result_buffer = b'0' + exponent as u8;
    result_buffer = result_buffer.add(1);

    *result_buffer = 0;

    0
}

extern "C" {
    fn strrchr(s: *const u8, c: c_int) -> *mut u8;
}

// ── %f ─────────────────────────────────────────────────────────────────────

/// `fp_format_f_internal`: `[-]ddd.ddd` from digits rounded into the buffer at
/// offset sign.
unsafe fn fp_format_f_internal(
    buffer: *mut u8,
    _buffer_count: usize,
    precision: c_int,
    pflt: *mut Strflt,
    g_fmt: bool,
    ptd: &mut PtdHost,
) -> c_int {
    let g_magnitude = (*pflt).decpt - 1;
    let negative = (*pflt).sign == b'-' as c_int;

    // %g with exponent == precision: a placeholder '0' past the digits.
    if g_fmt && g_magnitude == precision {
        let p = buffer.offset(g_magnitude as isize + negative as isize);
        *p = b'0';
        *p.add(1) = 0;
    }

    let mut p = buffer;

    if negative {
        *p = b'-';
        p = p.add(1);
    }

    // Leading zero for a purely fractional value; otherwise step over the
    // integer digits.
    if (*pflt).decpt <= 0 {
        // %g of 0 would otherwise grow an extra trailing zero.
        let is_zero_pflt = (*pflt).decpt == 0 && *(*pflt).mantissa == b'0' as c_char;
        if !g_fmt || !is_zero_pflt {
            shift_bytes(p, 1);
        }
        *p = b'0';
        p = p.add(1);
    } else {
        p = p.offset((*pflt).decpt as isize);
    }

    if precision > 0 {
        shift_bytes(p, 1);
        *p = decimal_point(ptd);
        p = p.add(1);

        // Zeros between the point and the first significant digit.
        if (*pflt).decpt < 0 {
            let computed_precision =
                if g_fmt || -(*pflt).decpt < precision { -(*pflt).decpt } else { precision };

            shift_bytes(p, computed_precision);
            core::ptr::write_bytes(p, b'0', computed_precision as usize);
        }
    }

    0
}

/// `fp_format_f`.
#[allow(clippy::too_many_arguments)]
unsafe fn fp_format_f(
    argument: *const u64,
    result_buffer: *mut u8,
    result_buffer_count: usize,
    scratch_buffer: *mut u8,
    scratch_buffer_count: usize,
    precision: c_int,
    rounding_mode: c_int,
    ptd: &mut PtdHost,
) -> c_int {
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };
    let trailing_digits = fltout_bits(
        *argument,
        precision as u32,
        PRECISION_STYLE_FIXED,
        &mut strflt,
        scratch_buffer.cast(),
        scratch_buffer_count,
    );

    let negative = (strflt.sign == b'-' as c_int) as usize;
    let e = __acrt_fp_strflt_to_string(
        result_buffer.add(negative).cast(),
        if result_buffer_count == UNBOUNDED_BUFFER_SIZE {
            result_buffer_count
        } else {
            result_buffer_count.wrapping_sub(negative)
        },
        precision.wrapping_add(strflt.decpt),
        &mut strflt,
        trailing_digits,
        rounding_mode,
        ptd,
    );

    if e != 0 {
        *result_buffer = 0;
        return e;
    }

    fp_format_f_internal(result_buffer, result_buffer_count, precision, &mut strflt, false, ptd)
}

// ── %g ─────────────────────────────────────────────────────────────────────

/// `fp_format_g`: %e when the exponent is < -4 or >= precision, else %f.
/// Digits are generated for %f (always the larger count; generation cannot be
/// resumed), then the choice is made on the ROUNDED exponent.
#[allow(clippy::too_many_arguments)]
unsafe fn fp_format_g(
    argument: *const u64,
    result_buffer: *mut u8,
    result_buffer_count: usize,
    scratch_buffer: *mut u8,
    scratch_buffer_count: usize,
    precision: c_int,
    capitals: bool,
    min_exponent_digits: u32,
    rounding_mode: c_int,
    ptd: &mut PtdHost,
) -> c_int {
    let mut strflt = Strflt { sign: 0, decpt: 0, mantissa: core::ptr::null_mut() };

    let trailing_digits = fltout_bits(
        *argument,
        precision as u32,
        PRECISION_STYLE_FIXED,
        &mut strflt,
        scratch_buffer.cast(),
        scratch_buffer_count,
    );

    let minus_sign_length: usize = if strflt.sign == b'-' as c_int { 1 } else { 0 };

    let mut g_magnitude = strflt.decpt - 1;
    let mut p = result_buffer.add(minus_sign_length);

    let buffer_count_for_fptostr = if result_buffer_count == UNBOUNDED_BUFFER_SIZE {
        result_buffer_count
    } else {
        result_buffer_count.wrapping_sub(minus_sign_length)
    };

    let fptostr_result = __acrt_fp_strflt_to_string(
        p.cast(),
        buffer_count_for_fptostr,
        precision,
        &mut strflt,
        trailing_digits,
        rounding_mode,
        ptd,
    );
    if fptostr_result != 0 {
        *result_buffer = 0;
        return fptostr_result;
    }

    // Rounding carried into a new leading digit (9.99 -> 10.0).
    let g_round_expansion = g_magnitude < strflt.decpt - 1;

    g_magnitude = strflt.decpt - 1;

    if g_magnitude < -4 || g_magnitude >= precision {
        // %e: the expansion's extra digit is overwritten by "e+xxx".
        fp_format_e_internal(result_buffer, result_buffer_count, precision, capitals, min_exponent_digits, &mut strflt, true, ptd)
    } else {
        if g_round_expansion {
            // Drop the extra final digit.
            loop {
                let ch = *p;
                p = p.add(1);
                if ch == 0 {
                    break;
                }
            }
            *p.sub(2) = 0;
        }

        fp_format_f_internal(result_buffer, result_buffer_count, precision, &mut strflt, true, ptd)
    }
}

// ── dispatch ───────────────────────────────────────────────────────────────

/// `__acrt_fp_format` (corecrt_internal_fltintrn.h): format `*value` per
/// `format` (`a A e E f F g G`; anything else is %g) into `result_buffer`,
/// with `scratch_buffer` for the raw digits.
///
/// `value` is `double const*` in the C; `*const u64` here to match the two
/// existing declarations (`output.rs`, `convert_rest/gcvt.rs`) -- the bits are
/// the same.
///
/// # Safety
/// Buffers must hold their counts; `ptd` must be a live host.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn __acrt_fp_format(
    value: *const u64,
    result_buffer: *mut u8,
    result_buffer_count: usize,
    scratch_buffer: *mut u8,
    scratch_buffer_count: usize,
    format: c_int,
    precision: c_int,
    options: u64,
    mut rounding_mode: c_int,
    ptd: *mut PtdHost,
) -> c_int {
    let ptd = &mut *ptd;
    if let Some(e) = validate(ptd, !result_buffer.is_null(), EINVAL) {
        return e;
    }
    if let Some(e) = validate(ptd, result_buffer_count > 0, EINVAL) {
        return e;
    }
    if let Some(e) = validate(ptd, !scratch_buffer.is_null(), EINVAL) {
        return e;
    }
    if let Some(e) = validate(ptd, scratch_buffer_count > 0, EINVAL) {
        return e;
    }

    let use_capitals =
        format == b'A' as c_int || format == b'E' as c_int || format == b'F' as c_int || format == b'G' as c_int;

    // C99 spellings for INF/NAN, unless legacy msvcrt behaviour was asked for
    // (then __acrt_fltout's "1#INF" forms come through the formatters).
    if options & PRINTF_LEGACY_MSVCRT_COMPATIBILITY == 0 {
        let classification = classify(*value);
        if classification != FP_FINITE {
            return fp_format_nan_or_infinity(
                classification,
                Components::of(*value).sign == 1,
                result_buffer,
                result_buffer_count,
                use_capitals,
            );
        }
    }

    let min_exponent_digits: u32 = if options & PRINTF_LEGACY_THREE_DIGIT_EXPONENTS != 0 { 3 } else { 2 };
    if options & PRINTF_STANDARD_ROUNDING == 0 {
        rounding_mode = ROUNDING_LEGACY;
    }

    match format {
        f if f == b'a' as c_int || f == b'A' as c_int => fp_format_a(
            value,
            result_buffer,
            result_buffer_count,
            scratch_buffer,
            scratch_buffer_count,
            precision,
            use_capitals,
            min_exponent_digits,
            rounding_mode,
            ptd,
        ),
        f if f == b'e' as c_int || f == b'E' as c_int => fp_format_e(
            value,
            result_buffer,
            result_buffer_count,
            scratch_buffer,
            scratch_buffer_count,
            precision,
            use_capitals,
            min_exponent_digits,
            rounding_mode,
            ptd,
        ),
        f if f == b'f' as c_int || f == b'F' as c_int => fp_format_f(
            value,
            result_buffer,
            result_buffer_count,
            scratch_buffer,
            scratch_buffer_count,
            precision,
            rounding_mode,
            ptd,
        ),
        // 'g', 'G', and (the C's _ASSERTE aside) anything unsupported.
        _ => fp_format_g(
            value,
            result_buffer,
            result_buffer_count,
            scratch_buffer,
            scratch_buffer_count,
            precision,
            use_capitals,
            min_exponent_digits,
            rounding_mode,
            ptd,
        ),
    }
}
