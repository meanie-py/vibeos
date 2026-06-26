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

// ============================================================================
// Model-based randomized tests — each operation cross-checked against an
// independent native oracle (i128 / u64 / u128). Complements the hand-written
// `tests` above with broad random coverage of the edge cases the RTL semantics
// hinge on: wrapping arithmetic, signed/unsigned remainder asymmetry, shift
// count masking, and the 128-bit magic-divide multiply (verified against u128).
// Deterministic SplitMix64 PRNG — no external dependency.
// ============================================================================
#[cfg(test)]
mod model_tests {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }
        fn i64(&mut self) -> i64 {
            self.next() as i64
        }
        fn i32(&mut self) -> i32 {
            self.next() as i32
        }
        fn u32(&mut self) -> u32 {
            self.next() as u32
        }
    }

    const ITERS: usize = 20_000;

    #[test]
    fn convert_widening() {
        let mut r = Rng(1);
        for _ in 0..ITERS {
            let s = r.i32();
            assert_eq!(convert_long_to_large_integer(s), s as i64);
            let u = r.u32();
            assert_eq!(convert_ulong_to_large_integer(u), u as i64);
            assert!(convert_ulong_to_large_integer(u) >= 0, "ulong zero-extends");
        }
    }

    #[test]
    fn multiplies_vs_widening() {
        let mut r = Rng(2);
        for _ in 0..ITERS {
            let (a, b) = (r.i32(), r.i32());
            assert_eq!(enlarged_integer_multiply(a, b), a as i64 * b as i64);
            let (ua, ub) = (r.u32(), r.u32());
            assert_eq!(enlarged_unsigned_multiply(ua, ub) as u64, ua as u64 * ub as u64);
            let (big, m) = (r.i64(), r.i32());
            assert_eq!(extended_integer_multiply(big, m), (big as i128 * m as i128) as i64);
        }
    }

    #[test]
    fn unsigned_divide_reconstructs() {
        let mut r = Rng(3);
        for _ in 0..ITERS {
            let divisor = r.u32() | 1; // non-zero
            let q_target = r.u32();
            let rem_in = r.u32() % divisor;
            let dividend = q_target as u64 * divisor as u64 + rem_in as u64;
            let mut rem: u32 = 0;
            assert_eq!(enlarged_unsigned_divide(dividend, divisor, &mut rem), q_target);
            assert_eq!(rem, rem_in);
            // null remainder pointer path still returns the quotient
            assert_eq!(
                enlarged_unsigned_divide(dividend, divisor, core::ptr::null_mut()),
                q_target
            );
        }
    }

    #[test]
    fn extended_divide_signed_quotient_unsigned_rem() {
        let mut r = Rng(4);
        for _ in 0..ITERS {
            let dividend = r.i64();
            let divisor = r.u32() | 1;
            let mut rem: u32 = 0;
            let q = extended_large_integer_divide(dividend, divisor, &mut rem);
            assert_eq!(q, dividend / divisor as i64); // signed quotient
            assert_eq!(rem, (dividend as u64 % divisor as u64) as u32); // unsigned rem
        }
    }

    #[test]
    fn add_sub_negate_wrapping() {
        let mut r = Rng(5);
        for _ in 0..ITERS {
            let (a, b) = (r.i64(), r.i64());
            assert_eq!(large_integer_add(a, b), (a as i128 + b as i128) as i64);
            assert_eq!(large_integer_subtract(a, b), (a as i128 - b as i128) as i64);
            assert_eq!(large_integer_negate(a), (-(a as i128)) as i64);
        }
    }

    #[test]
    fn signed_divide_reconstructs() {
        let mut r = Rng(6);
        for _ in 0..ITERS {
            let dividend = r.i64();
            let mut divisor = r.i64();
            if divisor == 0 {
                divisor = 1;
            }
            if dividend == i64::MIN && divisor == -1 {
                continue; // the one overflowing signed division
            }
            let mut rem: i64 = 0;
            let q = large_integer_divide(dividend, divisor, &mut rem);
            assert_eq!(q, dividend / divisor);
            assert_eq!(rem, dividend % divisor);
            assert_eq!(q.wrapping_mul(divisor).wrapping_add(rem), dividend);
        }
    }

    #[test]
    fn shifts_vs_independent_oracle() {
        let mut r = Rng(7);
        for _ in 0..ITERS {
            let v = r.i64();
            let sc = r.i32() as i8; // full i8 range, incl negative (rem_euclid masking)
            let s = (sc as i32).rem_euclid(64) as u32;
            assert_eq!(large_integer_shift_left(v, sc), (v as u64).wrapping_shl(s) as i64);
            assert_eq!(large_integer_shift_right(v, sc), (v as i128 >> s) as i64);
        }
    }

    #[test]
    fn arithmetic_shift_matches_right_below_32() {
        let mut r = Rng(8);
        for _ in 0..ITERS {
            let v = r.i64();
            let sc = (r.u32() % 32) as i8; // 0..=31 → plain arithmetic right shift
            assert_eq!(large_integer_arithmetic_shift(v, sc), v >> sc as u32);
        }
    }

    #[test]
    fn arithmetic_shift_high_mode() {
        // For 32..=63 the C code emulates the i386 two-register SAR: the high
        // dword collapses to its sign bit and the low dword is the old high
        // dword shifted (x86 masks the count to 5 bits). Lock that behavior in.
        let mut r = Rng(9);
        for _ in 0..ITERS {
            let v = r.i64();
            let sc = (32 + r.u32() % 32) as i8; // 32..=63
            let high = (v >> 32) as i32;
            let sign_only = (high as u32 & 0x8000_0000) as i32;
            let low = (high as u32).wrapping_shr(sc as u32);
            let expected = ((sign_only as i64) << 32) | (low as i64);
            assert_eq!(large_integer_arithmetic_shift(v, sc), expected);
        }
    }

    #[test]
    fn magic_divide_vs_u128() {
        // The hand-rolled 32-bit-decomposed 128-bit multiply must equal the high
        // 64 bits of a real u128 product, then shifted, then re-signed.
        let mut r = Rng(10);
        for _ in 0..ITERS {
            let dividend = r.i64();
            let magic = r.i64();
            let shift = (r.u32() % 64) as i8; // 0..=63
            let positive = dividend >= 0;
            let abs = dividend.unsigned_abs();
            let hi = ((abs as u128 * (magic as u64) as u128) >> 64) as u64;
            let res = (hi >> shift) as i64;
            if !positive && res == i64::MIN {
                continue; // impl negates `result`; i64::MIN can't be negated
            }
            let expected = if positive { res } else { -res };
            assert_eq!(extended_magic_divide(dividend, magic, shift), expected);
        }
    }
}

// ============================================================================
// Kani exhaustive proofs (run with `cargo kani -p largeint`; inert otherwise).
// Same class as the model tests above but proved over the entire input domain.
// ============================================================================
#[cfg(kani)]
mod proofs {
    use super::*;

    #[kani::proof]
    fn enlarged_multiplies_are_widening() {
        let (a, b): (i32, i32) = (kani::any(), kani::any());
        assert_eq!(enlarged_integer_multiply(a, b), a as i64 * b as i64);
        let (ua, ub): (u32, u32) = (kani::any(), kani::any());
        assert_eq!(enlarged_unsigned_multiply(ua, ub) as u64, ua as u64 * ub as u64);
    }

    #[kani::proof]
    fn add_sub_negate_wrap() {
        let (a, b): (i64, i64) = (kani::any(), kani::any());
        assert_eq!(large_integer_add(a, b), (a as i128 + b as i128) as i64);
        assert_eq!(large_integer_subtract(a, b), (a as i128 - b as i128) as i64);
        assert_eq!(large_integer_negate(a), (-(a as i128)) as i64);
        assert_eq!(large_integer_negate(large_integer_negate(a)), a); // involution
    }

    #[kani::proof]
    fn shift_left_masks_count() {
        let v: i64 = kani::any();
        let sc: i8 = kani::any();
        let s = (sc as i32).rem_euclid(64) as u32;
        assert_eq!(large_integer_shift_left(v, sc), (v as u64).wrapping_shl(s) as i64);
    }

    #[kani::proof]
    fn unsigned_divide_reconstructs() {
        let dividend: u64 = kani::any();
        let divisor: u32 = kani::any();
        kani::assume(divisor != 0);
        kani::assume(dividend / divisor as u64 <= u32::MAX as u64); // quotient fits u32
        let mut rem: u32 = 0;
        let q = enlarged_unsigned_divide(dividend, divisor, &mut rem);
        assert_eq!(q as u64 * divisor as u64 + rem as u64, dividend);
        assert!(rem < divisor);
    }

    #[kani::proof]
    fn magic_divide_matches_u128_positive() {
        let dividend: i64 = kani::any();
        kani::assume(dividend >= 0);
        let magic: i64 = kani::any();
        let shift: i8 = kani::any();
        kani::assume(shift >= 0 && shift < 64);
        let hi = ((dividend as u128 * (magic as u64) as u128) >> 64) as u64;
        let expected = (hi >> shift) as i64;
        assert_eq!(extended_magic_divide(dividend, magic, shift), expected);
    }
}
