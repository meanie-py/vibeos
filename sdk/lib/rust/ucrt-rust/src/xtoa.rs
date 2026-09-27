//! `_itoa` and its 19 siblings — integer to string, narrow and wide, plain and
//! `_s`. Replaces `sdk/lib/ucrt/convert/xtoa.cpp` in full.
//!
//! Two templates: `common_xtox` (the digit loop) and `common_xtox_s` (the
//! checked front end), instantiated for two widths and two character types.
//!
//! ## Details that are silent if wrong
//!
//! * **Only radix 10 is ever signed.** `_itoa(-1, buf, 16)` is `"ffffffff"`, the
//!   two's-complement bit pattern, not `"-1"`. `_ltoa` is the same: `long` is 32
//!   bits.
//! * **Digits above 9 are LOWERCASE.**
//! * **`_s` clears `buffer[0]` BEFORE the size and radix checks**, so a call that
//!   fails with ERANGE or EINVAL still leaves an empty string -- but a NULL buffer
//!   or a zero count fails first, with nothing written.
//! * **A too-small `_s` buffer is detected inside the digit loop**: the C writes
//!   digits until it runs out of room, THEN clears `buffer[0]` and fails ERANGE.
//!   The bytes after `buffer[0]` keep whatever digits were written.
//! * **The plain forms do not check the radix at all.** Radix 0 divides by zero
//!   and radix 1 never terminates, in the C and here alike; the only contract is
//!   2..=36, which is what `_s` enforces.

use core::ffi::c_int;

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

trait Char: Copy {
    fn from_ascii(b: u8) -> Self;
}
impl Char for u8 {
    fn from_ascii(b: u8) -> Self {
        b
    }
}
impl Char for u16 {
    fn from_ascii(b: u8) -> Self {
        b as u16
    }
}

#[cold]
unsafe fn fail(code: c_int) -> c_int {
    *_errno() = code;
    _invalid_parameter_noinfo();
    code
}

/// `common_xtox`. `value` holds the accumulator's bit pattern; `bits` (32 or 64)
/// is its width, which only matters for the negation.
unsafe fn xtox<C: Char>(
    value: u64,
    buffer: *mut C,
    buffer_count: usize,
    radix: u32,
    is_negative: bool,
    bits: u32,
) -> c_int {
    let mask: u64 = if bits == 64 { u64::MAX } else { (1u64 << bits) - 1 };
    let mut p = buffer;
    let mut length: usize = 0;
    let mut remaining = value;
    if is_negative {
        *p = C::from_ascii(b'-');
        p = p.add(1);
        length += 1;
        remaining = remaining.wrapping_neg() & mask;
    }
    let mut first_digit = p;
    let radix = radix as u64;
    loop {
        let digit = (remaining % radix) as u8;
        remaining /= radix;
        *p = C::from_ascii(if digit > 9 { digit - 10 + b'a' } else { digit + b'0' });
        p = p.add(1);
        length += 1;
        if !(remaining > 0 && length < buffer_count) {
            break;
        }
    }
    if length >= buffer_count {
        *buffer = C::from_ascii(0);
        return fail(ERANGE);
    }
    // `*p-- = '\0'`, then reverse the digits in place.
    *p = C::from_ascii(0);
    p = p.sub(1);
    loop {
        let t = *p;
        *p = *first_digit;
        *first_digit = t;
        p = p.wrapping_sub(1);
        first_digit = first_digit.add(1);
        if first_digit >= p {
            break;
        }
    }
    0
}

/// `common_xtox_s`.
unsafe fn xtox_s<C: Char>(
    value: u64,
    buffer: *mut C,
    buffer_count: usize,
    radix: c_int,
    is_negative: bool,
    bits: u32,
) -> c_int {
    if buffer.is_null() {
        return fail(EINVAL);
    }
    if buffer_count == 0 {
        return fail(EINVAL);
    }
    *buffer = C::from_ascii(0); // `_RESET_STRING`: the fill is off in release builds
    if buffer_count <= if is_negative { 2 } else { 1 } {
        return fail(ERANGE);
    }
    if !(2..=36).contains(&radix) {
        return fail(EINVAL);
    }
    xtox(value, buffer, buffer_count, radix as u32, is_negative, bits)
}

/// `_CRT_UNBOUNDED_BUFFER_SIZE`
const UNBOUNDED: usize = usize::MAX;

macro_rules! xtoa_signed {
    ($ch:ty, $int:ty, $uint:ty, $bits:expr, $plain:ident, $s:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $s(value: $int, buffer: *mut $ch, count: usize, radix: c_int) -> c_int {
            let neg = radix == 10 && value < 0;
            xtox_s(value as $uint as u64, buffer, count, radix, neg, $bits)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(value: $int, buffer: *mut $ch, radix: c_int) -> *mut $ch {
            let neg = radix == 10 && value < 0;
            xtox(value as $uint as u64, buffer, UNBOUNDED, radix as u32, neg, $bits);
            buffer
        }
    };
}

macro_rules! xtoa_unsigned {
    ($ch:ty, $uint:ty, $bits:expr, $plain:ident, $s:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $s(value: $uint, buffer: *mut $ch, count: usize, radix: c_int) -> c_int {
            xtox_s(value as u64, buffer, count, radix, false, $bits)
        }
        #[no_mangle]
        pub unsafe extern "C" fn $plain(value: $uint, buffer: *mut $ch, radix: c_int) -> *mut $ch {
            xtox(value as u64, buffer, UNBOUNDED, radix as u32, false, $bits);
            buffer
        }
    };
}

// `int` and `long` both go through `unsigned long`: 32 bits on Windows.
xtoa_signed!(u8, c_int, u32, 32, _itoa, _itoa_s);
xtoa_signed!(u8, i32, u32, 32, _ltoa, _ltoa_s);
xtoa_unsigned!(u8, u32, 32, _ultoa, _ultoa_s);
xtoa_signed!(u8, i64, u64, 64, _i64toa, _i64toa_s);
xtoa_unsigned!(u8, u64, 64, _ui64toa, _ui64toa_s);
xtoa_signed!(u16, c_int, u32, 32, _itow, _itow_s);
xtoa_signed!(u16, i32, u32, 32, _ltow, _ltow_s);
xtoa_unsigned!(u16, u32, 32, _ultow, _ultow_s);
xtoa_signed!(u16, i64, u64, 64, _i64tow, _i64tow_s);
xtoa_unsigned!(u16, u64, 64, _ui64tow, _ui64tow_s);
