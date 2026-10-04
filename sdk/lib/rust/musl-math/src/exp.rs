//! musl src/math/exp.c and exp2.c (Copyright (c) 2018, Arm Limited. MIT).
//! The non-TOINT_INTRINSICS, EXP_USE_TOINT_NARROW = 0 arm, as musl builds it.

use crate::tables::*;
use crate::{asdouble, asuint64, barrier, ok, oflow, p2, uflow, MathErr};

const N: u64 = 1 << 7; // EXP_TABLE_BITS

#[inline(always)]
fn top12(x: f64) -> u32 {
    (asuint64(x) >> 52) as u32
}

/// `specialcase` for exp: scale*(1+tmp) without intermediate overflow/underflow.
/// Underflow is reported only when the result lands in the subnormal range, which is
/// where musl raises it explicitly (and Wine reports `_UNDERFLOW`); overflow when the
/// result is infinite.
fn specialcase(tmp: f64, mut sbits: u64, ki: u64, scale_down: u64, mul: f64) -> (f64, MathErr) {
    if ki & 0x8000_0000 == 0 {
        // k > 0: the exponent of scale might have overflowed.
        sbits = sbits.wrapping_sub(scale_down << 52);
        let scale = asdouble(sbits);
        let y = mul * (scale + scale * tmp);
        return (y, if y.is_infinite() { MathErr::Overflow } else { MathErr::None });
    }
    // k < 0: special care in the subnormal range.
    sbits = sbits.wrapping_add(1022u64 << 52);
    let scale = asdouble(sbits);
    let mut y = scale + scale * tmp;
    if y < 1.0 {
        // Round y to the right precision before scaling it into the subnormal range.
        let mut lo = scale - y + scale * tmp;
        let hi = 1.0 + y;
        lo = 1.0 - hi + y + lo;
        y = barrier(hi + lo) - 1.0;
        // Avoid -0.0 with downward rounding.
        if y == 0.0 {
            y = 0.0;
        }
        // The underflow exception, signalled explicitly.
        core::hint::black_box(barrier(p2(-1022)) * p2(-1022));
        return (p2(-1022) * y, MathErr::Underflow);
    }
    ok(p2(-1022) * y)
}

/// `exp`. `NaN` is `Domain` (Wine reports `exp(NaN)` to `_matherr`); the value is musl's.
pub fn exp(x: f64) -> (f64, MathErr) {
    if x.is_nan() {
        return (1.0 + x, MathErr::Domain);
    }
    let mut abstop = top12(x) & 0x7ff;
    if abstop.wrapping_sub(top12(p2(-54))) >= top12(512.0).wrapping_sub(top12(p2(-54))) {
        if abstop.wrapping_sub(top12(p2(-54))) >= 0x8000_0000 {
            // Tiny x (0 is common): avoid spurious underflow.
            return ok(1.0 + x);
        }
        if abstop >= top12(1024.0) {
            if asuint64(x) == asuint64(f64::NEG_INFINITY) {
                return ok(0.0);
            }
            if abstop >= top12(f64::INFINITY) {
                return ok(1.0 + x);
            }
            return if asuint64(x) >> 63 != 0 { uflow(0) } else { oflow(0) };
        }
        // Large x is special-cased below.
        abstop = 0;
    }
    // exp(x) = 2^(k/N) * exp(r), with exp(r) in [2^(-1/2N),2^(1/2N)].
    let z = EXP_INVLN2N * x;
    let mut kd = barrier(z + EXP_SHIFT);
    let ki = asuint64(kd);
    kd -= EXP_SHIFT;
    let r = x + kd * EXP_NEGLN2HIN + kd * EXP_NEGLN2LON;
    // 2^(k/N) ~= scale * (1 + tail).
    let idx = (2 * (ki % N)) as usize;
    let top = ki.wrapping_shl(52 - 7);
    let tail = asdouble(EXP_TAB[idx]);
    let sbits = EXP_TAB[idx + 1].wrapping_add(top);
    let r2 = r * r;
    let (c2, c3, c4, c5) = (EXP_POLY[0], EXP_POLY[1], EXP_POLY[2], EXP_POLY[3]);
    let tmp = tail + r + r2 * (c2 + r * c3) + r2 * r2 * (c4 + r * c5);
    if abstop == 0 {
        return specialcase(tmp, sbits, ki, 1009, p2(1009));
    }
    let scale = asdouble(sbits);
    ok(scale + scale * tmp)
}

/// `exp2`. Overflow/underflow are classified; Wine reports them through errno only, and
/// not at all below -2^31 -- that is the CRT layer's business, not this one's.
pub fn exp2(x: f64) -> (f64, MathErr) {
    let mut abstop = top12(x) & 0x7ff;
    if abstop.wrapping_sub(top12(p2(-54))) >= top12(512.0).wrapping_sub(top12(p2(-54))) {
        if abstop.wrapping_sub(top12(p2(-54))) >= 0x8000_0000 {
            return ok(1.0 + x);
        }
        if abstop >= top12(1024.0) {
            if asuint64(x) == asuint64(f64::NEG_INFINITY) {
                return ok(0.0);
            }
            if abstop >= top12(f64::INFINITY) {
                return ok(1.0 + x);
            }
            if asuint64(x) >> 63 == 0 {
                return oflow(0);
            } else if asuint64(x) >= asuint64(-1075.0) {
                return uflow(0);
            }
        }
        if 2 * asuint64(x) > 2 * asuint64(928.0) {
            abstop = 0;
        }
    }
    // exp2(x) = 2^(k/N) * 2^r, with 2^r in [2^(-1/2N),2^(1/2N)].
    let mut kd = barrier(x + EXP_EXP2_SHIFT);
    let ki = asuint64(kd);
    kd -= EXP_EXP2_SHIFT;
    let r = x - kd;
    let idx = (2 * (ki % N)) as usize;
    let top = ki.wrapping_shl(52 - 7);
    let tail = asdouble(EXP_TAB[idx]);
    let sbits = EXP_TAB[idx + 1].wrapping_add(top);
    let r2 = r * r;
    let c = &EXP_EXP2_POLY;
    let tmp = tail + r * c[0] + r2 * (c[1] + r * c[2]) + r2 * r2 * (c[3] + r * c[4]);
    if abstop == 0 {
        return specialcase(tmp, sbits, ki, 1, 2.0);
    }
    let scale = asdouble(sbits);
    ok(scale + scale * tmp)
}
