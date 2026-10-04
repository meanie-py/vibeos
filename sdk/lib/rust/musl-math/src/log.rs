//! musl src/math/log.c (Copyright (c) 2018, Arm Limited. MIT), non-FMA arm.

use crate::tables::*;
use crate::{asdouble, asuint64, divzero, invalid, ok, p2, MathErr};

const N: u64 = 1 << 7; // LOG_TABLE_BITS
const OFF: u64 = 0x3fe6_0000_0000_0000;

/// `log`. `log(±0)` is `Sing`, a negative argument or NaN `Domain` -- musl's
/// `__math_divzero`/`__math_invalid`, Wine's `_SING`/`_DOMAIN`.
pub fn log(x: f64) -> (f64, MathErr) {
    let mut ix = asuint64(x);
    let top = (ix >> 48) as u32;
    let lo_b = asuint64(1.0 - p2(-4));
    let hi_b = asuint64(1.0 + f64::from_bits(0x3fb0_9000_0000_0000)); // 1 + 0x1.09p-4
    if ix.wrapping_sub(lo_b) < hi_b.wrapping_sub(lo_b) {
        // Close to 1.0.
        if ix == asuint64(1.0) {
            return ok(0.0);
        }
        let b = &LOG_POLY1;
        let r = x - 1.0;
        let r2 = r * r;
        let r3 = r * r2;
        let mut y = r3
            * (b[1] + r * b[2] + r2 * b[3]
                + r3 * (b[4] + r * b[5] + r2 * b[6] + r3 * (b[7] + r * b[8] + r2 * b[9] + r3 * b[10])));
        // Worst-case error is around 0.507 ULP.
        let mut w = r * p2(27);
        let rhi = r + w - w;
        let rlo = r - rhi;
        w = rhi * rhi * b[0]; // B[0] == -0.5
        let hi = r + w;
        let mut lo = r - hi + w;
        lo += b[0] * rlo * (rhi + r);
        y += lo;
        y += hi;
        return ok(y);
    }
    if top.wrapping_sub(0x0010) >= 0x7ff0 - 0x0010 {
        // x < 0x1p-1022 or inf or nan.
        if ix.wrapping_mul(2) == 0 {
            return divzero(1);
        }
        if ix == asuint64(f64::INFINITY) {
            return ok(x);
        }
        if (top & 0x8000) != 0 || (top & 0x7ff0) == 0x7ff0 {
            return invalid(x);
        }
        // Subnormal: normalize.
        ix = asuint64(x * p2(52));
        ix = ix.wrapping_sub(52u64 << 52);
    }
    let tmp = ix.wrapping_sub(OFF);
    let i = ((tmp >> (52 - 7)) % N) as usize;
    let k = (tmp as i64) >> 52;
    let iz = ix.wrapping_sub(tmp & (0xfffu64 << 52));
    let invc = LOG_TAB[2 * i];
    let logc = LOG_TAB[2 * i + 1];
    let z = asdouble(iz);
    // r ~= z/c - 1, |r| < 1/(2*N).
    let r = (z - LOG_TAB2[2 * i] - LOG_TAB2[2 * i + 1]) * invc;
    let kd = k as f64;
    // hi + lo = r + log(c) + k*Ln2.
    let w = kd * LOG_LN2HI + logc;
    let hi = w + r;
    let lo = w - hi + r + kd * LOG_LN2LO;
    let a = &LOG_POLY;
    let r2 = r * r;
    let y = lo + r2 * a[0] + r * r2 * (a[1] + r * a[2] + r2 * (a[3] + r * a[4])) + hi;
    ok(y)
}
