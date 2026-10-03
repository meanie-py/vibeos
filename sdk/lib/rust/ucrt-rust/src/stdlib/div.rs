//! `div.cpp`, `ldiv.cpp`, `lldiv.cpp`, `imaxdiv.cpp`.
//!
//! The quotient comes from the SAME instruction (or, for 64-bit on i386, the
//! same `__divdi3` call) Microsoft's compiled C uses, so a zero divisor or
//! `MIN / -1` behaves identically: #DE on x86 (STATUS_INTEGER_DIVIDE_BY_ZERO /
//! STATUS_INTEGER_OVERFLOW), `sdiv`'s 0 / MIN on arm64. Rust's `/` would
//! panic on both instead. The remainder is then `n - q*d`, exactly what the
//! C objects compute (`msub` on arm64, the second half of `idiv` on x86 --
//! identical wherever the division did not trap).

use core::ffi::{c_int, c_long};

/// `div_t`.
#[repr(C)]
pub struct DivT {
    pub quot: c_int,
    pub rem: c_int,
}

/// `ldiv_t` (`long` is 32 bits on Windows).
#[repr(C)]
pub struct LDivT {
    pub quot: c_long,
    pub rem: c_long,
}

/// `lldiv_t`.
#[repr(C)]
pub struct LLDivT {
    pub quot: i64,
    pub rem: i64,
}

/// `imaxdiv_t` (`intmax_t` is `long long`).
#[repr(C)]
pub struct ImaxDivT {
    pub quot: i64,
    pub rem: i64,
}

/// 32-bit signed quotient via the hardware divide.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn quot32(n: i32, d: i32) -> i32 {
    let q: i32;
    core::arch::asm!(
        "cdq",
        "idiv {d:e}",
        d = in(reg) d,
        inout("eax") n => q,
        out("edx") _,
        options(nomem, nostack),
    );
    q
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn quot32(n: i32, d: i32) -> i32 {
    let q: i32;
    core::arch::asm!(
        "sdiv {q:w}, {n:w}, {d:w}",
        q = lateout(reg) q,
        n = in(reg) n,
        d = in(reg) d,
        options(pure, nomem, nostack),
    );
    q
}

/// 64-bit signed quotient.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn quot64(n: i64, d: i64) -> i64 {
    let q: i64;
    core::arch::asm!(
        "cqo",
        "idiv {d}",
        d = in(reg) d,
        inout("rax") n => q,
        out("rdx") _,
        options(nomem, nostack),
    );
    q
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn quot64(n: i64, d: i64) -> i64 {
    let q: i64;
    core::arch::asm!(
        "sdiv {q}, {n}, {d}",
        q = lateout(reg) q,
        n = in(reg) n,
        d = in(reg) d,
        options(pure, nomem, nostack),
    );
    q
}

// i386 has no 64-bit divide: Microsoft's object calls compiler-rt's
// `__divdi3`, and so do we (same symbol, same link).
#[cfg(target_arch = "x86")]
extern "C" {
    fn __divdi3(a: i64, b: i64) -> i64;
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn quot64(n: i64, d: i64) -> i64 {
    __divdi3(n, d)
}

/// `div_t div(int, int)`.
#[no_mangle]
pub unsafe extern "C" fn div(numerator: c_int, denominator: c_int) -> DivT {
    let quot = quot32(numerator, denominator);
    DivT { quot, rem: numerator.wrapping_sub(quot.wrapping_mul(denominator)) }
}

/// `ldiv_t ldiv(long, long)`.
#[no_mangle]
pub unsafe extern "C" fn ldiv(numerator: c_long, denominator: c_long) -> LDivT {
    let quot = quot32(numerator, denominator);
    LDivT { quot, rem: numerator.wrapping_sub(quot.wrapping_mul(denominator)) }
}

/// `lldiv_t lldiv(long long, long long)`.
#[no_mangle]
pub unsafe extern "C" fn lldiv(numerator: i64, denominator: i64) -> LLDivT {
    let quot = quot64(numerator, denominator);
    LLDivT { quot, rem: numerator.wrapping_sub(quot.wrapping_mul(denominator)) }
}

/// `imaxdiv_t imaxdiv(intmax_t, intmax_t)` (Plauger's). The C spells the
/// remainder `numerator - denominator * quot` and then applies a "fix
/// incorrect truncation" step guarded by `static bool const fix_required =
/// (-1 / 2) < 0` -- false under C++11 truncating division, so the step is
/// dead and the result is exactly `lldiv`'s.
#[no_mangle]
pub unsafe extern "C" fn imaxdiv(numerator: i64, denominator: i64) -> ImaxDivT {
    let quot = quot64(numerator, denominator);
    ImaxDivT { quot, rem: numerator.wrapping_sub(denominator.wrapping_mul(quot)) }
}
