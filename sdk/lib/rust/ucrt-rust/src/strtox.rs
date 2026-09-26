//! `strtol` and its 31 siblings — string to integer, narrow and wide.
//! Replaces `sdk/lib/ucrt/convert/strtox.cpp` in full (32 exports).
//!
//! Every export there is a one-line wrapper over one template,
//! `__crt_strtox::parse_integer` (`corecrt_internal_strtox.h:222`), instantiated
//! for two accumulator widths and two character types. This is that template,
//! with the width a runtime parameter: `long` is 32 bits on every Windows target,
//! `long long`/`__int64`/`intmax_t` are 64.
//!
//! A port, not msvcrt's bodies: the Tier 2 table graded msvcrt's `strtoul` and
//! friends DIFF against UCRT on 14 of 598 cases, beginning with
//! `strtoul("-2147483648", 16)` — Wine's `_MSVCR_VER=0` overflow contract, not
//! UCRT's.
//!
//! ## Details that are silent if wrong
//!
//! * **The end pointer is written FIRST, then again on the way out.** The C's
//!   character source stores `string` into `*end` in its constructor and the
//!   final position in its destructor. So a NULL `string` leaves `*end = NULL`,
//!   a bad base leaves it at `string`, and a string with no digits restores it
//!   to `string` — whitespace and sign NOT consumed.
//! * **`"0x"` with no hex digit after it is "no digits"**: the `0` was consumed
//!   as a prefix, not as a digit, so the result is 0 with `*end = string`, where
//!   C99 would stop after the `0`. UCRT behaviour, transcribed.
//! * **Base detection only looks for a leading `'0'`** (`parse_digit(c) != 0`),
//!   and base 16 accepts an optional `0x` the same way.
//! * **Overflow for an UNSIGNED result ignores the sign**: `strtoul("-1")` is
//!   `ULONG_MAX` with no ERANGE — the negation wraps — and only a magnitude
//!   beyond the type sets ERANGE and returns `ULONG_MAX`. A SIGNED result
//!   overflows at the signed limits and returns them.
//! * **Letters are digits in any case, wide included**: a wide `c` is first
//!   tried as one of 17 Unicode decimal-digit blocks (so U+0663 is 3), then as
//!   ASCII A-Z/a-z. Narrow whitespace is `_isspace_l` in the resolved locale;
//!   wide whitespace is `iswspace`, locale-independent.

use crate::ctype::{_isspace_l, iswspace};
use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use core::ffi::c_int;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

const FL_SIGNED: u32 = 0x01;
const FL_NEGATIVE: u32 = 0x02;
const FL_OVERFLOW: u32 = 0x04;
const FL_READ_DIGIT: u32 = 0x08;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

/// The two character types the template is instantiated for.
pub(crate) trait Char: Copy + Eq {
    fn code(self) -> u32;
    /// `parse_digit`: the digit value, or `u32::MAX` (the C's `(unsigned)-1`).
    fn digit(self) -> u32;
    unsafe fn is_space(self, locale: *mut CrtLocalePointers) -> bool;
}

impl Char for u8 {
    fn code(self) -> u32 {
        self as u32
    }
    fn digit(self) -> u32 {
        match self {
            b'0'..=b'9' => (self - b'0') as u32,
            b'a'..=b'z' => (self - b'a') as u32 + 10,
            b'A'..=b'Z' => (self - b'A') as u32 + 10,
            _ => u32::MAX,
        }
    }
    unsafe fn is_space(self, locale: *mut CrtLocalePointers) -> bool {
        _isspace_l(self as c_int, locale) != 0
    }
}

/// `wide_character_to_digit` (corecrt_internal_strtox.h:87): the zero of each
/// Unicode decimal-digit block below FF10, tested in ascending order exactly as
/// the C's DIGIT_RANGE_TEST chain does -- a `c` below the next zero ends the scan.
const WIDE_DIGIT_ZEROS: [u16; 16] = [
    0x0660, 0x06F0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0C66, 0x0CE6, 0x0D66, 0x0E50,
    0x0ED0, 0x0F20, 0x1040, 0x17E0, 0x1810,
];

fn wide_character_to_digit(c: u16) -> Option<u32> {
    if c < 0x0030 {
        return None;
    }
    if c < 0x0030 + 10 {
        return Some((c - 0x0030) as u32);
    }
    if c < 0xFF10 {
        for zero in WIDE_DIGIT_ZEROS {
            if c < zero {
                return None;
            }
            if c < zero + 10 {
                return Some((c - zero) as u32);
            }
        }
        return None;
    }
    if c < 0xFF10 + 10 {
        return Some((c - 0xFF10) as u32);
    }
    None
}

impl Char for u16 {
    fn code(self) -> u32 {
        self as u32
    }
    fn digit(self) -> u32 {
        if let Some(d) = wide_character_to_digit(self) {
            return d;
        }
        // `__ascii_iswalpha` then `__ascii_towupper(c) - 'A' + 10`
        match self {
            0x41..=0x5A => (self - 0x41) as u32 + 10,
            0x61..=0x7A => (self - 0x61) as u32 + 10,
            _ => u32::MAX,
        }
    }
    unsafe fn is_space(self, _locale: *mut CrtLocalePointers) -> bool {
        iswspace(self) != 0
    }
}

#[cold]
unsafe fn invalid_parameter() {
    *_errno() = EINVAL;
    _invalid_parameter_noinfo();
}

/// `parse_integer_from_string` + `parse_integer`, for an accumulator of `bits`
/// (32 or 64) bits. The result is the accumulator's bit pattern, which each
/// export reinterprets as its own type, as the C's `static_cast<Integer>` does.
pub(crate) unsafe fn parse<C: Char>(
    string: *const C,
    end: *mut *const C,
    mut base: c_int,
    locale: *mut CrtLocalePointers,
    bits: u32,
    is_result_signed: bool,
) -> u64 {
    // The character source's constructor.
    if !end.is_null() {
        *end = string;
    }
    // `source.validate()`
    if string.is_null() {
        invalid_parameter();
        return 0;
    }
    if !(base == 0 || (2..=36).contains(&base)) {
        invalid_parameter();
        return 0; // *end is still `string`
    }

    let mask: u64 = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
    let sign_bit: u64 = 1u64 << (bits - 1); // -minimum_signed_value, as unsigned
    let signed_max: u64 = sign_bit - 1;

    let mut lu = LocaleUpdate::new(locale);
    let loc = lu.locale_t();

    // `source.get()`: read and advance.
    let mut p = string;
    macro_rules! get {
        () => {{
            let c = *p;
            p = p.add(1);
            c
        }};
    }

    let mut c = get!();
    while c.is_space(loc) {
        c = get!();
    }

    let mut flags = if is_result_signed { FL_SIGNED } else { 0 };
    if c.code() == b'-' as u32 {
        flags |= FL_NEGATIVE;
    }
    if c.code() == b'-' as u32 || c.code() == b'+' as u32 {
        c = get!();
    }

    if base == 0 || base == 16 {
        if c.digit() != 0 {
            if base == 0 {
                base = 10;
            }
        } else {
            let next = get!();
            if next.code() == b'x' as u32 || next.code() == b'X' as u32 {
                if base == 0 {
                    base = 16;
                }
                c = get!();
            } else {
                if base == 0 {
                    base = 8;
                }
                // `source.unget(next_c)`
                p = p.sub(1);
            }
        }
    }

    let base = base as u64;
    let max_pre_multiply_value = mask / base;
    let mut number: u64 = 0;
    loop {
        let digit = c.digit();
        if digit as u64 >= base {
            break;
        }
        flags |= FL_READ_DIGIT;
        let after_multiply = number.wrapping_mul(base) & mask;
        let after_add = after_multiply.wrapping_add(digit as u64) & mask;
        if number > max_pre_multiply_value || after_add < after_multiply {
            flags |= FL_OVERFLOW;
        }
        number = after_add;
        c = get!();
    }
    // `source.unget(c)`: the character that ended the scan is not consumed.
    p = p.sub(1);

    if flags & FL_READ_DIGIT == 0 {
        // `source.restore_state(initial_state)`
        if !end.is_null() {
            *end = string;
        }
        return 0;
    }
    if !end.is_null() {
        *end = p;
    }

    // `is_overflow_condition`
    let overflow = flags & FL_OVERFLOW != 0
        || (flags & FL_SIGNED != 0
            && if flags & FL_NEGATIVE != 0 { number > sign_bit } else { number > signed_max });
    if overflow {
        *_errno() = ERANGE;
        if flags & FL_SIGNED == 0 {
            return mask;
        }
        return if flags & FL_NEGATIVE != 0 { sign_bit } else { signed_max };
    }
    if flags & FL_NEGATIVE != 0 {
        number = number.wrapping_neg() & mask;
    }
    number
}

macro_rules! strtox {
    ($ch:ty, $ret:ty, $bits:expr, $signed:expr, $plain:ident, $l:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $l(
            string: *const $ch,
            end_ptr: *mut *mut $ch,
            base: c_int,
            locale: *mut CrtLocalePointers,
        ) -> $ret {
            parse(string, end_ptr as *mut *const $ch, base, locale, $bits, $signed) as $ret
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(string: *const $ch, end_ptr: *mut *mut $ch, base: c_int) -> $ret {
            parse(string, end_ptr as *mut *const $ch, base, core::ptr::null_mut(), $bits, $signed)
                as $ret
        }
    };
}

// `long` / `unsigned long` are 32 bits on every Windows target.
strtox!(u8, i32, 32, true, strtol, _strtol_l);
strtox!(u8, u32, 32, false, strtoul, _strtoul_l);
strtox!(u8, i64, 64, true, _strtoi64, _strtoi64_l);
strtox!(u8, i64, 64, true, strtoll, _strtoll_l);
strtox!(u8, i64, 64, true, strtoimax, _strtoimax_l);
strtox!(u8, u64, 64, false, _strtoui64, _strtoui64_l);
strtox!(u8, u64, 64, false, strtoull, _strtoull_l);
strtox!(u8, u64, 64, false, strtoumax, _strtoumax_l);

strtox!(u16, i32, 32, true, wcstol, _wcstol_l);
strtox!(u16, u32, 32, false, wcstoul, _wcstoul_l);
strtox!(u16, i64, 64, true, _wcstoi64, _wcstoi64_l);
strtox!(u16, i64, 64, true, wcstoll, _wcstoll_l);
strtox!(u16, i64, 64, true, wcstoimax, _wcstoimax_l);
strtox!(u16, u64, 64, false, _wcstoui64, _wcstoui64_l);
strtox!(u16, u64, 64, false, wcstoull, _wcstoull_l);
strtox!(u16, u64, 64, false, wcstoumax, _wcstoumax_l);
