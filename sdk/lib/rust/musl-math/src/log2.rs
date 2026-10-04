//! musl src/math/log2.c (Copyright (c) 2018, Arm Limited. MIT), non-FMA arm.

use crate::tables::*;
use crate::{asdouble, asuint64, divzero, invalid, ok, p2, MathErr};

const N: u64 = 1 << 6; // LOG2_TABLE_BITS
const OFF: u64 = 0x3fe6_0000_0000_0000;

/// `log2`: `±0` is `Sing`, a negative argument or NaN `Domain`.
pub fn log2(x: f64) -> (f64, MathErr) {
    let mut ix = asuint64(x);
    let top = (ix >> 48) as u32;
    let lo_b = asuint64(1.0 - f64::from_bits(0x3fa5_b510_0000_0000)); // 1 - 0x1.5b51p-5
    let hi_b = asuint64(1.0 + f64::from_bits(0x3fa6_ab20_0000_0000)); // 1 + 0x1.6ab2p-5
    if ix.wrapping_sub(lo_b) < hi_b.wrapping_sub(lo_b) {
        if ix == asuint64(1.0) {
            return ok(0.0);
        }
        let b = &LOG2_POLY1;
        let r = x - 1.0;
        let rhi = asdouble(asuint64(r) & (u64::MAX << 32));
        let rlo = r - rhi;
        let hi = rhi * LOG2_INVLN2HI;
        let mut lo = rlo * LOG2_INVLN2HI + r * LOG2_INVLN2LO;
        let r2 = r * r;
        let r4 = r2 * r2;
        let p = r2 * (b[0] + r * b[1]);
        let mut y = hi + p;
        lo += hi - y + p;
        lo += r4
            * (b[2] + r * b[3] + r2 * (b[4] + r * b[5]) + r4 * (b[6] + r * b[7] + r2 * (b[8] + r * b[9])));
        y += lo;
        return ok(y);
    }
    if top.wrapping_sub(0x0010) >= 0x7ff0 - 0x0010 {
        if ix.wrapping_mul(2) == 0 {
            return divzero(1);
        }
        if ix == asuint64(f64::INFINITY) {
            return ok(x);
        }
        if (top & 0x8000) != 0 || (top & 0x7ff0) == 0x7ff0 {
            return invalid(x);
        }
        ix = asuint64(x * p2(52));
        ix = ix.wrapping_sub(52u64 << 52);
    }
    let tmp = ix.wrapping_sub(OFF);
    let i = ((tmp >> (52 - 6)) % N) as usize;
    let k = (tmp as i64) >> 52;
    let iz = ix.wrapping_sub(tmp & (0xfffu64 << 52));
    let invc = LOG2_TAB[2 * i];
    let logc = LOG2_TAB[2 * i + 1];
    let z = asdouble(iz);
    let kd = k as f64;
    let r = (z - LOG2_TAB2[2 * i] - LOG2_TAB2[2 * i + 1]) * invc;
    let rhi = asdouble(asuint64(r) & (u64::MAX << 32));
    let rlo = r - rhi;
    let t1 = rhi * LOG2_INVLN2HI;
    let t2 = rlo * LOG2_INVLN2HI + r * LOG2_INVLN2LO;
    // hi + lo = r/ln2 + log2(c) + k.
    let t3 = kd + logc;
    let hi = t3 + t1;
    let lo = t3 - hi + t1 + t2;
    let a = &LOG2_POLY;
    let r2 = r * r;
    let r4 = r2 * r2;
    let p = a[0] + r * a[1] + r2 * (a[2] + r * a[3]) + r4 * (a[4] + r * a[5]);
    ok(lo + r2 * p + hi)
}
