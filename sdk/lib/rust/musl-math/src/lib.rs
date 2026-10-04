//! musl's current `exp`/`exp2`/`log`/`log2`/`pow` and their `f32` forms (ARM
//! optimized-routines, MIT; see `ref/COPYRIGHT`). Each returns its value AND why it
//! failed, if it did ([`MathErr`]): musl reports overflow/underflow/divide-by-zero/invalid
//! through `__math_oflow`/`__math_uflow`/`__math_divzero`/`__math_invalid`, and those are
//! exactly the points where Wine's copy calls the CRT's `math_error` -- so the CRT layer
//! (cnt-rust) can raise `errno`/`_matherr` without re-deriving the conditions.
#![no_std]

mod exp;
mod f32;
mod log;
mod log2;
mod pow;
mod tables;

pub use exp::{exp, exp2};
pub use f32::{exp2f, expf, log2f, logf, powf};
pub use log::log;
pub use log2::log2;
pub use pow::pow;

/// Why a result is special, in the CRT's `_exception.type` vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathErr {
    None,
    /// `__math_invalid`: `_DOMAIN`.
    Domain,
    /// `__math_divzero`: `_SING`.
    Sing,
    /// `__math_oflow`: `_OVERFLOW`.
    Overflow,
    /// `__math_uflow`: `_UNDERFLOW`.
    Underflow,
}

pub(crate) use MathErr::{Domain, Overflow, Sing, Underflow};

#[inline(always)]
pub(crate) fn asuint64(x: f64) -> u64 {
    x.to_bits()
}
#[inline(always)]
pub(crate) fn asdouble(u: u64) -> f64 {
    f64::from_bits(u)
}
#[inline(always)]
pub(crate) fn asuint(x: f32) -> u32 {
    x.to_bits()
}
#[inline(always)]
pub(crate) fn asfloat(u: u32) -> f32 {
    f32::from_bits(u)
}

/// `0x1pE` for a normal exponent.
pub(crate) const fn p2(e: i32) -> f64 {
    f64::from_bits(((1023 + e) as u64) << 52)
}
pub(crate) const fn p2f(e: i32) -> f32 {
    f32::from_bits(((127 + e) as u32) << 23)
}

/// `fp_barrier`: keep the compiler from folding an expression whose only purpose is its
/// rounding or its exception.
#[inline(always)]
pub(crate) fn barrier(x: f64) -> f64 {
    core::hint::black_box(x)
}
#[inline(always)]
pub(crate) fn barrierf(x: f32) -> f32 {
    core::hint::black_box(x)
}

// musl src/math/__math_{xflow,oflow,uflow,invalid,divzero}.c
pub(crate) fn xflow(sign: u32, y: f64) -> f64 {
    barrier(if sign != 0 { -y } else { y }) * y
}
pub(crate) fn oflow(sign: u32) -> (f64, MathErr) {
    (xflow(sign, p2(769)), Overflow)
}
pub(crate) fn uflow(sign: u32) -> (f64, MathErr) {
    (xflow(sign, p2(-767)), Underflow)
}
pub(crate) fn invalid(x: f64) -> (f64, MathErr) {
    ((x - x) / (x - x), Domain)
}
pub(crate) fn divzero(sign: u32) -> (f64, MathErr) {
    (barrier(if sign != 0 { -1.0 } else { 1.0 }) / 0.0, Sing)
}
pub(crate) fn xflowf(sign: u32, y: f32) -> f32 {
    barrierf(if sign != 0 { -y } else { y }) * y
}
pub(crate) fn oflowf(sign: u32) -> (f32, MathErr) {
    (xflowf(sign, p2f(97)), Overflow)
}
pub(crate) fn uflowf(sign: u32) -> (f32, MathErr) {
    (xflowf(sign, p2f(-95)), Underflow)
}
pub(crate) fn invalidf(x: f32) -> (f32, MathErr) {
    ((x - x) / (x - x), Domain)
}
pub(crate) fn divzerof(sign: u32) -> (f32, MathErr) {
    (barrierf(if sign != 0 { -1.0 } else { 1.0 }) / 0.0, Sing)
}

#[inline(always)]
pub(crate) fn ok<T>(v: T) -> (T, MathErr) {
    (v, MathErr::None)
}
