//! convert/strtod.cpp and the engine under it (`corecrt_internal_strtox.h`'s
//! floating-point half): strtod, strtof, strtold, their wide and `_l` forms.
//!
//! A port of Microsoft's parser and its correctly-rounded conversion. The parser
//! turns the subject string into up to 768 mantissa digits and an exponent; the
//! converter then builds the result with big-integer arithmetic (big_integer.rs)
//! -- the integer part exactly, the fractional part as an exact N/M division --
//! and rounds once, in the current rounding mode. msvcrt's `fpnum.rs` is Wine's
//! parser and `_MSVCR_VER=0`'s contract (no hex floats, no "infinity"/"nan"
//! words, no `strtof`), so it is not this.
//!
//! Everything is built from bits: no float arithmetic anywhere, so i386's no-SSE
//! discipline holds. The only float values are the returns, which the `extern "C"`
//! ABI hands back in x87 `ST(0)` there, exactly as the C's do. They are made with
//! `transmute`, never `f64::from_bits`: that is a Rust-ABI function returning a
//! float, which the no-SSE i386 cell refuses to compile.
//!
//! `long double` is double on every target here (`-mlong-double-64` on x86; it
//! already is on arm64), so strtold is strtod, as in the C.

use core::ffi::c_int;

use crate::big_integer::{
    add, bsr, bsr64, divide, multiply_by_power_of_ten, multiply_small, shift_left, BigInteger,
};
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use crate::strtox::Char;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

// fenv.h / float.h: _RC_NEAR, _RC_DOWN, _RC_UP, _RC_CHOP.
const FE_TONEAREST: c_int = 0x000;
const FE_DOWNWARD: c_int = 0x100;
const FE_UPWARD: c_int = 0x200;
const FE_TOWARDZERO: c_int = 0x300;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    /// The current rounding mode. The C calls this on every inexact conversion,
    /// so the Rust does too: today it is ucrtbase's generated stub (returns 0,
    /// round-to-nearest, and says so on the debug port), and when fenv is
    /// ported this picks it up with no change here.
    fn fegetround() -> c_int;
}

// ── the parse result ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParseResult {
    DecimalDigits,
    HexadecimalDigits,
    Zero,
    Infinity,
    Qnan,
    Snan,
    Indeterminate,
    NoDigits,
    Underflow,
    Overflow,
}

/// `SLD_STATUS`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Ok,
    NoDigits,
    Underflow,
    Overflow,
}

const MAXIMUM_TEMPORARY_DECIMAL_EXPONENT: i32 = 5200;
const MINIMUM_TEMPORARY_DECIMAL_EXPONENT: i32 = -5200;

/// `floating_point_string`: `[sign] 0.mantissa * B^exponent`, the mantissa as
/// big-endian digit values (base 10, or 16 for a hex float with a base-2
/// exponent).
pub(crate) struct FloatingPointString {
    exponent: i32,
    mantissa_count: u32,
    mantissa: [u8; 768],
    is_negative: bool,
}

// ── the result type ───────────────────────────────────────────────────────────

/// `__acrt_floating_type_traits`, for the two types the precise algorithm
/// serves. `floating_point_value` in the C.
pub(crate) struct Traits {
    mantissa_bits: i32,
    maximum_binary_exponent: i32,
    minimum_binary_exponent: i32,
    exponent_bias: i32,
    exponent_mask: u64,
    normal_mantissa_mask: u64,
    denormal_mantissa_mask: u64,
    special_nan_mantissa_mask: u64,
    total_bits: u32,
}

pub(crate) const DOUBLE: Traits = Traits {
    mantissa_bits: 53,
    maximum_binary_exponent: 1023,
    minimum_binary_exponent: -1022,
    exponent_bias: 1023,
    exponent_mask: 0x7FF,
    normal_mantissa_mask: (1 << 53) - 1,
    denormal_mantissa_mask: (1 << 52) - 1,
    special_nan_mantissa_mask: 1 << 51,
    total_bits: 64,
};

pub(crate) const FLOAT: Traits = Traits {
    mantissa_bits: 24,
    maximum_binary_exponent: 127,
    minimum_binary_exponent: -126,
    exponent_bias: 127,
    exponent_mask: 0xFF,
    normal_mantissa_mask: (1 << 24) - 1,
    denormal_mantissa_mask: (1 << 23) - 1,
    special_nan_mantissa_mask: 1 << 22,
    total_bits: 32,
};

impl Traits {
    /// Writing the components struct: the bitfields truncate, as they do here.
    fn assemble(&self, is_negative: bool, exponent_field: u64, mantissa: u64) -> u64 {
        ((is_negative as u64) << (self.total_bits - 1))
            | ((exponent_field & self.exponent_mask) << (self.mantissa_bits - 1))
            | (mantissa & self.denormal_mantissa_mask)
    }
    fn zero(&self, is_negative: bool) -> u64 {
        self.assemble(is_negative, 0, 0)
    }
    fn infinity(&self, is_negative: bool) -> u64 {
        self.assemble(is_negative, self.exponent_mask, 0)
    }
    fn qnan(&self, is_negative: bool) -> u64 {
        self.assemble(is_negative, self.exponent_mask, self.denormal_mantissa_mask)
    }
    fn snan(&self, is_negative: bool) -> u64 {
        self.assemble(is_negative, self.exponent_mask, 1)
    }
    /// The indeterminate is "negative".
    fn ind(&self) -> u64 {
        self.assemble(true, self.exponent_mask, self.special_nan_mantissa_mask)
    }
}

// ── rounding and assembly ─────────────────────────────────────────────────────

fn should_round_up(is_negative: bool, lsb_bit: bool, round_bit: bool, has_tail_bits: bool) -> bool {
    if !round_bit && !has_tail_bits {
        return false;
    }
    match unsafe { fegetround() } {
        FE_TONEAREST => round_bit && (has_tail_bits || lsb_bit),
        FE_UPWARD => !is_negative,
        FE_DOWNWARD => is_negative,
        FE_TOWARDZERO => false,
        _ => false,
    }
}

fn right_shift_with_rounding(is_negative: bool, value: u64, shift: u32, has_zero_tail: bool) -> u64 {
    if shift >= 64 {
        return 0;
    }
    let extra_bits_mask = (1u64 << (shift - 1)) - 1;
    let round_bit_mask = 1u64 << (shift - 1);
    let lsb_bit_mask = 1u64 << shift;
    let lsb_bit = value & lsb_bit_mask != 0;
    let round_bit = value & round_bit_mask != 0;
    let tail_bits = !has_zero_tail || value & extra_bits_mask != 0;
    (value >> shift) + should_round_up(is_negative, lsb_bit, round_bit, tail_bits) as u64
}

/// `assemble_floating_point_value`: `[sign] mantissa * 2^exponent` (mantissa as
/// an integer whose top bit is at `exponent`), rounded into the type.
fn assemble_value(
    t: &Traits,
    initial_mantissa: u64,
    initial_exponent: i32,
    is_negative: bool,
    has_zero_tail: bool,
    out: &mut u64,
) -> Status {
    let initial_mantissa_bits = bsr64(initial_mantissa);
    let normal_mantissa_shift = t.mantissa_bits - initial_mantissa_bits as i32;
    let normal_exponent = initial_exponent - normal_mantissa_shift;

    let mut mantissa = initial_mantissa;
    let mut exponent = normal_exponent;

    if normal_exponent > t.maximum_binary_exponent {
        *out = t.infinity(is_negative);
        return Status::Overflow;
    } else if normal_exponent < t.minimum_binary_exponent {
        let denormal_mantissa_shift = normal_mantissa_shift + normal_exponent + t.exponent_bias - 1;
        exponent = -t.exponent_bias;
        if denormal_mantissa_shift < 0 {
            mantissa = right_shift_with_rounding(is_negative, mantissa, (-denormal_mantissa_shift) as u32, has_zero_tail);
            if mantissa == 0 {
                *out = t.zero(is_negative);
                return Status::Underflow;
            }
            // Rounding a denormal up can carry it into a normal.
            if mantissa > t.denormal_mantissa_mask {
                exponent = initial_exponent - (denormal_mantissa_shift + 1) - normal_mantissa_shift;
            }
        } else {
            mantissa = mantissa.wrapping_shl(denormal_mantissa_shift as u32);
        }
    } else if normal_mantissa_shift < 0 {
        mantissa = right_shift_with_rounding(is_negative, mantissa, (-normal_mantissa_shift) as u32, has_zero_tail);
        if mantissa > t.normal_mantissa_mask {
            mantissa >>= 1;
            exponent += 1;
            if exponent > t.maximum_binary_exponent {
                *out = t.infinity(is_negative);
                return Status::Overflow;
            }
        }
    } else if normal_mantissa_shift > 0 {
        mantissa = mantissa.wrapping_shl(normal_mantissa_shift as u32);
    }

    mantissa &= t.denormal_mantissa_mask;
    *out = t.assemble(is_negative, (exponent + t.exponent_bias) as u32 as u64, mantissa);
    Status::Ok
}

/// `assemble_floating_point_value_from_big_integer`: the integer fast track.
fn assemble_from_big_integer(
    t: &Traits,
    integer_value: &BigInteger,
    integer_bits_of_precision: u32,
    is_negative: bool,
    has_nonzero_fractional_part: bool,
    out: &mut u64,
) -> Status {
    let base_exponent = t.mantissa_bits - 1;

    if integer_bits_of_precision <= 64 {
        let low = if integer_value.used > 0 { integer_value.data[0] } else { 0 };
        let high = if integer_value.used > 1 { integer_value.data[1] } else { 0 };
        let mantissa = low as u64 + ((high as u64) << 32);
        return assemble_value(t, mantissa, base_exponent, is_negative, !has_nonzero_fractional_part, out);
    }

    let top_element_bits = integer_bits_of_precision % 32;
    let top_element_index = integer_bits_of_precision / 32;
    let middle_element_index = top_element_index - 1;
    let bottom_element_index = top_element_index - 2;

    if top_element_bits == 0 {
        let exponent = base_exponent + bottom_element_index as i32 * 32;
        let mantissa = integer_value.data[bottom_element_index as usize] as u64
            + ((integer_value.data[middle_element_index as usize] as u64) << 32);
        let mut has_zero_tail = !has_nonzero_fractional_part;
        for i in 0..bottom_element_index {
            has_zero_tail &= integer_value.data[i as usize] == 0;
        }
        return assemble_value(t, mantissa, exponent, is_negative, has_zero_tail, out);
    }

    let top_element_mask = (1u32 << top_element_bits) - 1;
    let top_element_shift = 64 - top_element_bits;
    let middle_element_shift = top_element_shift - 32;
    let bottom_element_bits = 32 - top_element_bits;
    let bottom_element_mask = !top_element_mask;
    let bottom_element_shift = 32 - bottom_element_bits;

    let exponent = base_exponent + bottom_element_index as i32 * 32 + top_element_bits as i32;
    let mantissa = (((integer_value.data[top_element_index as usize] & top_element_mask) as u64) << top_element_shift)
        + ((integer_value.data[middle_element_index as usize] as u64) << middle_element_shift)
        + (((integer_value.data[bottom_element_index as usize] & bottom_element_mask) as u64) >> bottom_element_shift);

    let mut has_zero_tail = !has_nonzero_fractional_part
        && integer_value.data[bottom_element_index as usize] & top_element_mask == 0;
    for i in 0..bottom_element_index {
        has_zero_tail &= integer_value.data[i as usize] == 0;
    }
    assemble_value(t, mantissa, exponent, is_negative, has_zero_tail, out)
}

/// `accumulate_decimal_digits_into_big_integer`: nine digits at a time.
fn accumulate_decimal_digits(digits: &[u8], result: &mut BigInteger) {
    let mut accumulator = 0u32;
    let mut accumulator_count = 0u32;
    for &d in digits {
        if accumulator_count == 9 {
            multiply_small(result, 1_000_000_000);
            add(result, accumulator);
            accumulator = 0;
            accumulator_count = 0;
        }
        accumulator = accumulator * 10 + d as u32;
        accumulator_count += 1;
    }
    if accumulator_count != 0 {
        multiply_by_power_of_ten(result, accumulator_count);
        add(result, accumulator);
    }
}

/// `convert_decimal_string_to_floating_type_common`.
fn convert_decimal(data: &FloatingPointString, t: &Traits, out: &mut u64) -> Status {
    let required_bits_of_precision = (t.mantissa_bits + 1) as u32;

    let positive_exponent = data.exponent.max(0) as u32;
    let integer_digits_present = positive_exponent.min(data.mantissa_count);
    let integer_digits_missing = positive_exponent - integer_digits_present;
    let mantissa = &data.mantissa[..data.mantissa_count as usize];
    let (integer_digits, fractional_digits) = mantissa.split_at(integer_digits_present as usize);
    let fractional_digits_present = fractional_digits.len() as u32;

    let mut integer_value = BigInteger::zero();
    accumulate_decimal_digits(integer_digits, &mut integer_value);

    if integer_digits_missing > 0 && !multiply_by_power_of_ten(&mut integer_value, integer_digits_missing) {
        *out = t.infinity(data.is_negative);
        return Status::Overflow;
    }

    let integer_bits_of_precision = bsr(&integer_value);
    if integer_bits_of_precision >= required_bits_of_precision || fractional_digits_present == 0 {
        return assemble_from_big_integer(
            t,
            &integer_value,
            integer_bits_of_precision,
            data.is_negative,
            fractional_digits_present != 0,
            out,
        );
    }

    let mut fractional_numerator = BigInteger::zero();
    accumulate_decimal_digits(fractional_digits, &mut fractional_numerator);

    let fractional_denominator_exponent = if data.exponent < 0 {
        fractional_digits_present + data.exponent.unsigned_abs()
    } else {
        fractional_digits_present
    };

    let mut fractional_denominator = BigInteger::from_u64(1);
    if !multiply_by_power_of_ten(&mut fractional_denominator, fractional_denominator_exponent) {
        *out = t.zero(data.is_negative);
        return Status::Underflow;
    }

    let fractional_numerator_bits = bsr(&fractional_numerator);
    let fractional_denominator_bits = bsr(&fractional_denominator);
    let fractional_shift = fractional_denominator_bits.saturating_sub(fractional_numerator_bits);
    if fractional_shift > 0 {
        shift_left(&mut fractional_numerator, fractional_shift);
    }

    let required_fractional_bits_of_precision = required_bits_of_precision - integer_bits_of_precision;
    let mut remaining_bits_of_precision_required = required_fractional_bits_of_precision;
    if integer_bits_of_precision > 0 {
        if fractional_shift > remaining_bits_of_precision_required {
            return assemble_from_big_integer(
                t,
                &integer_value,
                integer_bits_of_precision,
                data.is_negative,
                fractional_digits_present != 0,
                out,
            );
        }
        remaining_bits_of_precision_required -= fractional_shift;
    }

    let fractional_exponent = if fractional_numerator.less_than(&fractional_denominator) {
        fractional_shift + 1
    } else {
        fractional_shift
    };

    shift_left(&mut fractional_numerator, remaining_bits_of_precision_required);
    let mut fractional_mantissa = divide(&mut fractional_numerator, &fractional_denominator);
    let mut has_zero_tail = fractional_numerator.used == 0;

    let fractional_mantissa_bits = bsr64(fractional_mantissa);
    if fractional_mantissa_bits > required_fractional_bits_of_precision {
        let shift = fractional_mantissa_bits - required_fractional_bits_of_precision;
        has_zero_tail = has_zero_tail && fractional_mantissa & ((1u64 << shift) - 1) == 0;
        fractional_mantissa >>= shift;
    }

    let low = if integer_value.used > 0 { integer_value.data[0] } else { 0 };
    let high = if integer_value.used > 1 { integer_value.data[1] } else { 0 };
    let integer_mantissa = low as u64 + ((high as u64) << 32);
    let complete_mantissa = integer_mantissa
        .wrapping_shl(required_fractional_bits_of_precision)
        .wrapping_add(fractional_mantissa);

    let final_exponent = if integer_bits_of_precision > 0 {
        integer_bits_of_precision as i32 - 2
    } else {
        -(fractional_exponent as i32) - 1
    };
    assemble_value(t, complete_mantissa, final_exponent, data.is_negative, has_zero_tail, out)
}

/// `convert_hexadecimal_string_to_floating_type_common`.
fn convert_hexadecimal(data: &FloatingPointString, t: &Traits, out: &mut u64) -> Status {
    let mut mantissa = 0u64;
    let mut exponent = data.exponent + t.mantissa_bits - 1;
    let digits = &data.mantissa[..data.mantissa_count as usize];
    let mut i = 0;
    while i != digits.len() && mantissa <= t.normal_mantissa_mask {
        mantissa = mantissa * 16 + digits[i] as u64;
        exponent -= 4;
        i += 1;
    }
    let mut has_zero_tail = true;
    while i != digits.len() && has_zero_tail {
        has_zero_tail = digits[i] == 0;
        i += 1;
    }
    assemble_value(t, mantissa, exponent, data.is_negative, has_zero_tail, out)
}

// ── the parser ────────────────────────────────────────────────────────────────

/// `c_string_character_source`: a cursor whose final position is the end pointer.
struct Source<C> {
    p: *const C,
}

impl<C: Char> Source<C> {
    unsafe fn get(&mut self) -> C {
        let c = *self.p;
        self.p = self.p.add(1);
        c
    }
    /// `unget`. Its `_VALIDATE_RETURN_VOID(c == '\0' || *_p == c)` always holds:
    /// every caller ungets the character it just read.
    unsafe fn unget(&mut self) {
        self.p = self.p.sub(1);
    }
}

fn is(c: u32, ch: u8) -> bool {
    c == ch as u32
}

/// `parse_next_characters_from_source`: match `word` case-insensitively
/// (upper- and lowercase ASCII only), advancing `c` past each match.
unsafe fn match_word<C: Char>(word: &[u8], c: &mut C, source: &mut Source<C>) -> bool {
    for &w in word {
        let code = c.code();
        if code != w.to_ascii_uppercase() as u32 && code != w.to_ascii_lowercase() as u32 {
            return false;
        }
        *c = source.get();
    }
    true
}

/// `is_digit_or_nondigit`.
fn is_digit_or_nondigit(c: u32) -> bool {
    matches!(c, 0x30..=0x39 | 0x61..=0x7A | 0x41..=0x5A | 0x5F)
}

/// The `restore_state` lambda: unget `c`, forget it, rewind to `state`. A
/// C-string source can always restore, so it always answers true.
unsafe fn restore<C: Char>(c: &mut C, zero: C, source: &mut Source<C>, state: *const C) -> bool {
    source.unget();
    *c = zero;
    source.p = state;
    true
}

unsafe fn parse_possible_infinity<C: Char>(
    c: &mut C,
    zero: C,
    source: &mut Source<C>,
    mut stored_state: *const C,
) -> ParseResult {
    if !match_word(b"inf", c, source) {
        restore(c, zero, source, stored_state);
        return ParseResult::NoDigits;
    }
    source.unget();
    stored_state = source.p;
    *c = source.get();
    if !match_word(b"inity", c, source) {
        return if restore(c, zero, source, stored_state) { ParseResult::Infinity } else { ParseResult::NoDigits };
    }
    source.unget();
    ParseResult::Infinity
}

unsafe fn parse_possible_nan<C: Char>(
    c: &mut C,
    zero: C,
    source: &mut Source<C>,
    mut stored_state: *const C,
) -> ParseResult {
    if !match_word(b"nan", c, source) {
        restore(c, zero, source, stored_state);
        return ParseResult::NoDigits;
    }
    source.unget();
    stored_state = source.p;
    *c = source.get();
    if !is(c.code(), b'(') {
        return if restore(c, zero, source, stored_state) { ParseResult::Qnan } else { ParseResult::NoDigits };
    }
    *c = source.get();
    if match_word(b"snan)", c, source) {
        source.unget();
        return ParseResult::Snan;
    }
    if match_word(b"ind)", c, source) {
        source.unget();
        return ParseResult::Indeterminate;
    }
    while !is(c.code(), b')') && c.code() != 0 {
        if !is_digit_or_nondigit(c.code()) {
            return if restore(c, zero, source, stored_state) { ParseResult::Qnan } else { ParseResult::NoDigits };
        }
        *c = source.get();
    }
    if !is(c.code(), b')') {
        return if restore(c, zero, source, stored_state) { ParseResult::Qnan } else { ParseResult::NoDigits };
    }
    ParseResult::Qnan
}

/// The locale's decimal point, compared the way the C compares it: its `char`
/// against the source's character type, so a wide source sees a byte above
/// 0x7F sign-extended and never matches it.
unsafe fn radix_point_matches<C: Char>(c: C, radix: u8) -> bool {
    if core::mem::size_of::<C>() == 1 {
        c.code() == radix as u32
    } else {
        c.code() as i32 == radix as i8 as i32
    }
}

/// `parse_floating_point_from_source`.
unsafe fn parse_from_source<C: Char>(
    locale: *mut CrtLocalePointers,
    source: &mut Source<C>,
    fp: &mut FloatingPointString,
    zero: C,
) -> ParseResult {
    let mut stored_state = source.p;
    let mut c = source.get();

    while c.is_space(locale) {
        c = source.get();
    }

    fp.is_negative = is(c.code(), b'-');
    if is(c.code(), b'-') || is(c.code(), b'+') {
        c = source.get();
    }

    if is(c.code(), b'I') || is(c.code(), b'i') {
        return parse_possible_infinity(&mut c, zero, source, stored_state);
    }
    if is(c.code(), b'N') || is(c.code(), b'n') {
        return parse_possible_nan(&mut c, zero, source, stored_state);
    }

    let mut is_hexadecimal = false;
    if is(c.code(), b'0') {
        let next_stored_state = source.p;
        let next_c = source.get();
        if is(next_c.code(), b'x') || is(next_c.code(), b'X') {
            is_hexadecimal = true;
            c = source.get();
            // "0x" with no digits after it is the subject "0" and a final "x...".
            stored_state = next_stored_state;
        } else {
            source.unget();
        }
    }

    let mantissa_last = fp.mantissa.len();
    let mut mantissa_it = 0usize;
    let mut exponent_adjustment: i32 = 0;
    let mut found_digits = false;
    let max_digit_value = if is_hexadecimal { 0xF } else { 9 };

    while is(c.code(), b'0') {
        found_digits = true;
        c = source.get();
    }

    loop {
        let digit_value = c.digit();
        if digit_value > max_digit_value {
            break;
        }
        found_digits = true;
        if mantissa_it != mantissa_last {
            fp.mantissa[mantissa_it] = digit_value as u8;
            mantissa_it += 1;
        }
        exponent_adjustment += 1;
        c = source.get();
    }

    let radix_point = **((*(*locale).locinfo).lconv as *const *const u8);
    if radix_point_matches(c, radix_point) {
        c = source.get();
        if mantissa_it == 0 {
            while is(c.code(), b'0') {
                found_digits = true;
                exponent_adjustment -= 1;
                c = source.get();
            }
        }
        loop {
            let digit_value = c.digit();
            if digit_value > max_digit_value {
                break;
            }
            found_digits = true;
            if mantissa_it != mantissa_last {
                fp.mantissa[mantissa_it] = digit_value as u8;
                mantissa_it += 1;
            }
            c = source.get();
        }
    }

    if !found_digits {
        if !restore(&mut c, zero, source, stored_state) {
            return ParseResult::NoDigits;
        }
        return if is_hexadecimal { ParseResult::Zero } else { ParseResult::NoDigits };
    }

    source.unget();
    stored_state = source.p;
    c = source.get();

    let has_exponent = match c.code() {
        0x65 | 0x45 => !is_hexadecimal, // e E
        0x70 | 0x50 => is_hexadecimal,  // p P
        _ => false,
    };

    let mut exponent: i32 = 0;
    if has_exponent {
        c = source.get();
        let exponent_is_negative = is(c.code(), b'-');
        if is(c.code(), b'+') || is(c.code(), b'-') {
            c = source.get();
        }
        let mut has_exponent_digits = false;
        while is(c.code(), b'0') {
            has_exponent_digits = true;
            c = source.get();
        }
        loop {
            let digit_value = c.digit();
            if digit_value >= 10 {
                break;
            }
            has_exponent_digits = true;
            exponent = exponent * 10 + digit_value as i32;
            if exponent > MAXIMUM_TEMPORARY_DECIMAL_EXPONENT {
                exponent = MAXIMUM_TEMPORARY_DECIMAL_EXPONENT + 1;
                break;
            }
            c = source.get();
        }
        // Skip the rest of an over-long exponent so the end pointer is right.
        while c.digit() < 10 {
            c = source.get();
        }
        if exponent_is_negative {
            exponent = -exponent;
        }
        if !has_exponent_digits {
            if restore(&mut c, zero, source, stored_state) {
                c = source.get();
            } else {
                return ParseResult::NoDigits;
            }
        }
    }

    source.unget();
    let _ = c;

    while mantissa_it != 0 && fp.mantissa[mantissa_it - 1] == 0 {
        mantissa_it -= 1;
    }
    if mantissa_it == 0 {
        return ParseResult::Zero;
    }
    if exponent > MAXIMUM_TEMPORARY_DECIMAL_EXPONENT {
        return ParseResult::Overflow;
    }
    if exponent < MINIMUM_TEMPORARY_DECIMAL_EXPONENT {
        return ParseResult::Underflow;
    }
    let exponent_adjustment_multiplier = if is_hexadecimal { 4 } else { 1 };
    exponent += exponent_adjustment * exponent_adjustment_multiplier;
    if exponent > MAXIMUM_TEMPORARY_DECIMAL_EXPONENT {
        return ParseResult::Overflow;
    }
    if exponent < MINIMUM_TEMPORARY_DECIMAL_EXPONENT {
        return ParseResult::Underflow;
    }
    fp.exponent = exponent;
    fp.mantissa_count = mantissa_it as u32;
    if is_hexadecimal {
        ParseResult::HexadecimalDigits
    } else {
        ParseResult::DecimalDigits
    }
}

/// `parse_floating_point_write_result`.
fn write_result(parse_result: ParseResult, fp: &FloatingPointString, t: &Traits, out: &mut u64) -> Status {
    match parse_result {
        ParseResult::DecimalDigits => convert_decimal(fp, t, out),
        ParseResult::HexadecimalDigits => convert_hexadecimal(fp, t, out),
        ParseResult::Zero => {
            *out = t.zero(fp.is_negative);
            Status::Ok
        }
        ParseResult::Infinity => {
            *out = t.infinity(fp.is_negative);
            Status::Ok
        }
        ParseResult::Qnan => {
            *out = t.qnan(fp.is_negative);
            Status::Ok
        }
        ParseResult::Snan => {
            *out = t.snan(fp.is_negative);
            Status::Ok
        }
        ParseResult::Indeterminate => {
            *out = t.ind();
            Status::Ok
        }
        ParseResult::NoDigits => {
            *out = t.zero(false);
            Status::NoDigits
        }
        ParseResult::Underflow => {
            *out = t.zero(fp.is_negative);
            Status::Underflow
        }
        ParseResult::Overflow => {
            *out = t.infinity(fp.is_negative);
            Status::Overflow
        }
    }
}

/// `parse_floating_point` over a C string. The source's end pointer is written
/// on construction (the start) and again from its final position.
pub(crate) unsafe fn parse_floating_point<C: Char>(
    locale: *mut CrtLocalePointers,
    string: *const C,
    end: *mut *const C,
    zero: C,
    t: &Traits,
    out: &mut u64,
) -> Status {
    let mut source = Source { p: string };
    // `fp_string` is deliberately uninitialised in the C: only the fields a
    // successful parse writes are read.
    let mut fp = FloatingPointString { exponent: 0, mantissa_count: 0, mantissa: [0; 768], is_negative: false };
    let parse_result = parse_from_source(locale, &mut source, &mut fp, zero);
    if !end.is_null() {
        *end = source.p;
    }
    write_result(parse_result, &fp, t, out)
}

/// `common_strtod_l`.
unsafe fn common_strtod_l<C: Char>(
    string: *const C,
    end_ptr: *mut *const C,
    locale: *mut CrtLocalePointers,
    zero: C,
    t: &Traits,
) -> u64 {
    if !end_ptr.is_null() {
        *end_ptr = string;
    }
    if string.is_null() {
        // _VALIDATE_RETURN: errno, then the handler.
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    }
    let mut locale_update = LocaleUpdate::new(locale);
    let mut result = 0u64;
    let status = parse_floating_point(locale_update.locale_t(), string, end_ptr, zero, t, &mut result);
    if status == Status::Overflow || status == Status::Underflow {
        *_errno() = ERANGE;
    }
    result
}

macro_rules! strtod {
    ($name:ident, $ch:ty, f64) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(string: *const $ch, end_ptr: *mut *mut $ch) -> f64 {
            core::mem::transmute::<u64, f64>(common_strtod_l::<$ch>(string, end_ptr as *mut *const $ch, core::ptr::null_mut(), 0, &DOUBLE))
        }
    };
    ($name:ident, $ch:ty, f32) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(string: *const $ch, end_ptr: *mut *mut $ch) -> f32 {
            core::mem::transmute::<u32, f32>(common_strtod_l::<$ch>(string, end_ptr as *mut *const $ch, core::ptr::null_mut(), 0, &FLOAT) as u32)
        }
    };
    ($name:ident, $ch:ty, f64, l) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(string: *const $ch, end_ptr: *mut *mut $ch, locale: *mut CrtLocalePointers) -> f64 {
            core::mem::transmute::<u64, f64>(common_strtod_l::<$ch>(string, end_ptr as *mut *const $ch, locale, 0, &DOUBLE))
        }
    };
    ($name:ident, $ch:ty, f32, l) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(string: *const $ch, end_ptr: *mut *mut $ch, locale: *mut CrtLocalePointers) -> f32 {
            core::mem::transmute::<u32, f32>(common_strtod_l::<$ch>(string, end_ptr as *mut *const $ch, locale, 0, &FLOAT) as u32)
        }
    };
}

strtod!(strtof, u8, f32);
strtod!(_strtof_l, u8, f32, l);
strtod!(strtod, u8, f64);
strtod!(_strtod_l, u8, f64, l);
strtod!(strtold, u8, f64);
strtod!(_strtold_l, u8, f64, l);
strtod!(wcstof, u16, f32);
strtod!(_wcstof_l, u16, f32, l);
strtod!(wcstod, u16, f64);
strtod!(_wcstod_l, u16, f64, l);
strtod!(wcstold, u16, f64);
strtod!(_wcstold_l, u16, f64, l);
