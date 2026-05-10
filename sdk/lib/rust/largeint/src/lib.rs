//! RTL Large Integer Arithmetic — Rust implementation.
//!
//! Replaces `sdk/lib/rtl/largeint.c` (360 lines).
//! All functions operate on `i64` (= LARGE_INTEGER.QuadPart) or `u64`
//! (= ULARGE_INTEGER.QuadPart), matching the Windows ABI on all architectures.
//!
//! On i386 stdcall, i64 is passed on the stack and returned in EDX:EAX.
//! On amd64/arm64, i64 fits in a single register.

#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case)]

// ============================================================================
// Pure arithmetic functions (testable)
// ============================================================================

#[inline]
pub fn convert_long_to_large_integer(signed_integer: i32) -> i64 {
    signed_integer as i64
}

#[inline]
pub fn convert_ulong_to_large_integer(unsigned_integer: u32) -> i64 {
    unsigned_integer as u32 as i64
}

#[inline]
pub fn enlarged_integer_multiply(multiplicand: i32, multiplier: i32) -> i64 {
    multiplicand as i64 * multiplier as i64
}

#[inline]
pub fn enlarged_unsigned_divide(dividend: u64, divisor: u32, remainder: *mut u32) -> u32 {
    if !remainder.is_null() {
        unsafe { *remainder = (dividend % divisor as u64) as u32 };
    }
    (dividend / divisor as u64) as u32
}

#[inline]
pub fn enlarged_unsigned_multiply(multiplicand: u32, multiplier: u32) -> i64 {
    (multiplicand as u64 * multiplier as u64) as i64
}

#[inline]
pub fn extended_integer_multiply(multiplicand: i64, multiplier: i32) -> i64 {
    multiplicand.wrapping_mul(multiplier as i64)
}

#[inline]
pub fn extended_large_integer_divide(dividend: i64, divisor: u32, remainder: *mut u32) -> i64 {
    if !remainder.is_null() {
        unsafe { *remainder = (dividend as u64 % divisor as u64) as u32 };
    }
    // Signed division: the C code does dividend.QuadPart / Divisor
    // where QuadPart is LONGLONG and Divisor is ULONG.
    // In C, the ULONG gets promoted to LONGLONG for the division.
    dividend / divisor as i64
}

/// (Dividend * MagicDivisor) >> (64 + ShiftCount)
///
/// Allows replacing a division by a constant with a multiplication by the
/// inverse constant. MagicDivisor is treated as unsigned despite being i64.
pub fn extended_magic_divide(dividend: i64, magic_divisor: i64, shift_count: i8) -> i64 {
    let positive = dividend >= 0;
    let abs_dividend = if positive {
        dividend as u64
    } else {
        (-(dividend as i128)) as u64
    };

    let divisor = magic_divisor as u64;

    // 128-bit multiply decomposed into 32-bit parts
    let a_hi = abs_dividend >> 32;
    let a_lo = abs_dividend & 0xffffffff;
    let b_hi = divisor >> 32;
    let b_lo = divisor & 0xffffffff;

    let ah_bl = a_hi * b_lo;
    let al_bh = a_lo * b_hi;
    let al_bl = a_lo * b_lo;

    // Upper 64 bits of the 128-bit product, then shifted
    let result = ((a_hi * b_hi
        + (ah_bl >> 32)
        + (al_bh >> 32)
        + (((ah_bl & 0xffffffff) + (al_bh & 0xffffffff) + (al_bl >> 32)) >> 32))
        >> shift_count as u64) as i64;

    if positive { result } else { -result }
}

#[inline]
pub fn large_integer_add(addend1: i64, addend2: i64) -> i64 {
    addend1.wrapping_add(addend2)
}

/// Arithmetic right shift, matching the C implementation's behavior exactly.
///
/// For shifts >= 32, the C code has implementation-defined behavior for the
/// high-part shift. We replicate x86 semantics (shift count masked to 5 bits).
pub fn large_integer_arithmetic_shift(value: i64, shift_count: i8) -> i64 {
    let shift = (shift_count as i32 % 64) as u32;

    if shift < 32 {
        value >> shift
    } else {
        let high = (value >> 32) as i32;
        // HighPart = sign bit only (0 or 0x80000000)
        let sign_only = (high as u32 & 0x80000000) as i32;
        // LowPart = HighPart >> Shift (x86 masks shift to 5 bits)
        let low = (high as u32).wrapping_shr(shift) as u32;
        ((sign_only as i64) << 32) | (low as i64)
    }
}

#[inline]
pub fn large_integer_divide(dividend: i64, divisor: i64, remainder: *mut i64) -> i64 {
    if !remainder.is_null() {
        unsafe { *remainder = dividend % divisor };
    }
    dividend / divisor
}

#[inline]
pub fn large_integer_negate(subtrahend: i64) -> i64 {
    subtrahend.wrapping_neg()
}

#[inline]
pub fn large_integer_shift_left(value: i64, shift_count: i8) -> i64 {
    let shift = (shift_count as i32).rem_euclid(64) as u32;
    value.wrapping_shl(shift)
}

#[inline]
pub fn large_integer_shift_right(value: i64, shift_count: i8) -> i64 {
    let shift = (shift_count as i32).rem_euclid(64) as u32;
    // C code uses signed >> which is arithmetic right shift.
    value >> shift
}

#[inline]
pub fn large_integer_subtract(minuend: i64, subtrahend: i64) -> i64 {
    minuend.wrapping_sub(subtrahend)
}

// ============================================================================
// FFI entry points
// ============================================================================

#[cfg(not(test))]
mod ffi {
    use super::*;

    #[no_mangle]
    pub unsafe extern "system" fn RtlConvertLongToLargeInteger(signed_integer: i32) -> i64 {
        convert_long_to_large_integer(signed_integer)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlConvertUlongToLargeInteger(unsigned_integer: u32) -> i64 {
        convert_ulong_to_large_integer(unsigned_integer)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnlargedIntegerMultiply(
        multiplicand: i32,
        multiplier: i32,
    ) -> i64 {
        enlarged_integer_multiply(multiplicand, multiplier)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnlargedUnsignedDivide(
        dividend: u64,
        divisor: u32,
        remainder: *mut u32,
    ) -> u32 {
        enlarged_unsigned_divide(dividend, divisor, remainder)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlEnlargedUnsignedMultiply(
        multiplicand: u32,
        multiplier: u32,
    ) -> i64 {
        enlarged_unsigned_multiply(multiplicand, multiplier)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlExtendedIntegerMultiply(
        multiplicand: i64,
        multiplier: i32,
    ) -> i64 {
        extended_integer_multiply(multiplicand, multiplier)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlExtendedLargeIntegerDivide(
        dividend: i64,
        divisor: u32,
        remainder: *mut u32,
    ) -> i64 {
        extended_large_integer_divide(dividend, divisor, remainder)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlExtendedMagicDivide(
        dividend: i64,
        magic_divisor: i64,
        shift_count: i8,
    ) -> i64 {
        extended_magic_divide(dividend, magic_divisor, shift_count)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerAdd(
        addend1: i64,
        addend2: i64,
    ) -> i64 {
        large_integer_add(addend1, addend2)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerArithmeticShift(
        large_integer: i64,
        shift_count: i8,
    ) -> i64 {
        large_integer_arithmetic_shift(large_integer, shift_count)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerDivide(
        dividend: i64,
        divisor: i64,
        remainder: *mut i64,
    ) -> i64 {
        large_integer_divide(dividend, divisor, remainder)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerNegate(subtrahend: i64) -> i64 {
        large_integer_negate(subtrahend)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerShiftLeft(
        large_integer: i64,
        shift_count: i8,
    ) -> i64 {
        large_integer_shift_left(large_integer, shift_count)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerShiftRight(
        large_integer: i64,
        shift_count: i8,
    ) -> i64 {
        large_integer_shift_right(large_integer, shift_count)
    }

    #[no_mangle]
    pub unsafe extern "system" fn RtlLargeIntegerSubtract(
        minuend: i64,
        subtrahend: i64,
    ) -> i64 {
        large_integer_subtract(minuend, subtrahend)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_long() {
        assert_eq!(convert_long_to_large_integer(0), 0i64);
        assert_eq!(convert_long_to_large_integer(1), 1i64);
        assert_eq!(convert_long_to_large_integer(-1), -1i64);
        assert_eq!(convert_long_to_large_integer(i32::MAX), i32::MAX as i64);
        assert_eq!(convert_long_to_large_integer(i32::MIN), i32::MIN as i64);
    }

    #[test]
    fn test_convert_ulong() {
        assert_eq!(convert_ulong_to_large_integer(0), 0i64);
        assert_eq!(convert_ulong_to_large_integer(1), 1i64);
        assert_eq!(convert_ulong_to_large_integer(u32::MAX), u32::MAX as i64);
    }

    #[test]
    fn test_enlarged_multiply() {
        assert_eq!(enlarged_integer_multiply(100, 200), 20000i64);
        assert_eq!(enlarged_integer_multiply(-1, i32::MAX), -(i32::MAX as i64));
        assert_eq!(
            enlarged_integer_multiply(i32::MAX, i32::MAX),
            (i32::MAX as i64) * (i32::MAX as i64)
        );
    }

    #[test]
    fn test_enlarged_unsigned_divide() {
        let mut rem = 0u32;
        assert_eq!(enlarged_unsigned_divide(100, 7, &mut rem), 14);
        assert_eq!(rem, 2);

        // Null remainder pointer
        assert_eq!(enlarged_unsigned_divide(100, 10, core::ptr::null_mut()), 10);
    }

    #[test]
    fn test_enlarged_unsigned_multiply() {
        assert_eq!(enlarged_unsigned_multiply(0xFFFFFFFF, 2), 0x1FFFFFFFE_i64);
        assert_eq!(enlarged_unsigned_multiply(0, 12345), 0);
    }

    #[test]
    fn test_extended_integer_multiply() {
        assert_eq!(extended_integer_multiply(100, 200), 20000);
        assert_eq!(extended_integer_multiply(-1, 5), -5);
    }

    #[test]
    fn test_extended_large_integer_divide() {
        let mut rem = 0u32;
        assert_eq!(extended_large_integer_divide(100, 7, &mut rem), 14);
        assert_eq!(rem, 2);
    }

    #[test]
    fn test_magic_divide() {
        // Test: 1000000 / 10 using magic divisor
        // MagicDivisor = 2^(64+0) / 10 = 1844674407370955162
        let magic = 1844674407370955162_i64;
        let result = extended_magic_divide(1000000, magic, 0);
        assert_eq!(result, 100000);

        // Negative dividend
        let result = extended_magic_divide(-1000000, magic, 0);
        assert_eq!(result, -100000);
    }

    #[test]
    fn test_magic_divide_with_shift() {
        // Test: 1000000 / 100 using magic divisor with shift
        // MagicDivisor = 2^(64+3) / 100 = 1475739525896764130
        let magic = 1475739525896764130_i64;
        let result = extended_magic_divide(1000000, magic, 3);
        assert_eq!(result, 10000);
    }

    #[test]
    fn test_add() {
        assert_eq!(large_integer_add(1, 2), 3);
        assert_eq!(large_integer_add(-1, 1), 0);
        assert_eq!(large_integer_add(i64::MAX, 1), i64::MIN); // wrapping
    }

    #[test]
    fn test_arithmetic_shift_small() {
        assert_eq!(large_integer_arithmetic_shift(0x100, 4), 0x10);
        assert_eq!(large_integer_arithmetic_shift(-0x100, 4), -0x10);
        assert_eq!(large_integer_arithmetic_shift(1, 0), 1);
    }

    #[test]
    fn test_arithmetic_shift_large() {
        // Shift by exactly 32: LowPart = HighPart >> 0 = HighPart
        let value: i64 = 0x12345678_00000000;
        let result = large_integer_arithmetic_shift(value, 32);
        // HighPart = 0x12345678, sign bit = 0, so result HighPart = 0
        // LowPart = 0x12345678 >> 0 = 0x12345678 (wrapping_shr(32) masks to 0)
        assert_eq!(result & 0xFFFFFFFF, 0x12345678);
        assert_eq!(result >> 32, 0); // no sign bit

        // Shift by 33
        let result = large_integer_arithmetic_shift(value, 33);
        assert_eq!(result & 0xFFFFFFFF, 0x091A2B3C);

        // Negative value, shift >= 32
        let neg: i64 = -0x100_00000000; // HighPart = 0xFFFFFF00
        let result = large_integer_arithmetic_shift(neg, 32);
        let low = (result & 0xFFFFFFFF) as u32;
        let high = ((result >> 32) as u32) as i32;
        assert_eq!(low, 0xFFFFFF00_u32); // HighPart >> 0
        assert_eq!(high as u32, 0x80000000); // sign bit only
    }

    #[test]
    fn test_divide() {
        let mut rem = 0i64;
        assert_eq!(large_integer_divide(100, 7, &mut rem), 14);
        assert_eq!(rem, 2);

        assert_eq!(large_integer_divide(-100, 7, &mut rem), -14);
        assert_eq!(rem, -2);

        // Null remainder
        assert_eq!(large_integer_divide(100, 10, core::ptr::null_mut()), 10);
    }

    #[test]
    fn test_negate() {
        assert_eq!(large_integer_negate(1), -1);
        assert_eq!(large_integer_negate(-1), 1);
        assert_eq!(large_integer_negate(0), 0);
        assert_eq!(large_integer_negate(i64::MIN), i64::MIN); // wrapping
    }

    #[test]
    fn test_shift_left() {
        assert_eq!(large_integer_shift_left(1, 10), 1024);
        assert_eq!(large_integer_shift_left(1, 63), i64::MIN);
        assert_eq!(large_integer_shift_left(1, 0), 1);
    }

    #[test]
    fn test_shift_right() {
        assert_eq!(large_integer_shift_right(1024, 10), 1);
        assert_eq!(large_integer_shift_right(-1, 1), -1); // arithmetic shift
        assert_eq!(large_integer_shift_right(i64::MIN, 63), -1);
    }

    #[test]
    fn test_subtract() {
        assert_eq!(large_integer_subtract(5, 3), 2);
        assert_eq!(large_integer_subtract(0, 1), -1);
        assert_eq!(large_integer_subtract(i64::MIN, 1), i64::MAX); // wrapping
    }
}
