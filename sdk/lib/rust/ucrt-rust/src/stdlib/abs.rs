//! `abs.cpp` (`abs`, `_abs64`), `labs.cpp`, `llabs.cpp`, `imaxabs.cpp`.
//!
//! `-number` of the minimum value is signed overflow in C; the compiled
//! objects (neg/cmov, cneg, xor/sub) wrap it to itself, and so does
//! `wrapping_neg`. `long` is 32 bits on every Windows target.

use core::ffi::{c_int, c_long};

/// `int abs(int)`.
#[no_mangle]
pub extern "C" fn abs(number: c_int) -> c_int {
    if number >= 0 { number } else { number.wrapping_neg() }
}

/// `__int64 _abs64(__int64)`.
#[no_mangle]
pub extern "C" fn _abs64(number: i64) -> i64 {
    if number >= 0 { number } else { number.wrapping_neg() }
}

/// `long labs(long)`.
#[no_mangle]
pub extern "C" fn labs(number: c_long) -> c_long {
    if number >= 0 { number } else { number.wrapping_neg() }
}

/// `long long llabs(long long)`.
#[no_mangle]
pub extern "C" fn llabs(number: i64) -> i64 {
    if number >= 0 { number } else { number.wrapping_neg() }
}

/// `intmax_t imaxabs(intmax_t)` -- written `number < 0 ? -number : number`
/// in the C (Plauger's), same result.
#[no_mangle]
pub extern "C" fn imaxabs(number: i64) -> i64 {
    if number < 0 { number.wrapping_neg() } else { number }
}
