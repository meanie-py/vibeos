//! musl src/math/pow.c (Copyright (c) 2018, Arm Limited. MIT), non-FMA arm.
//! Worst-case error 0.52 ULP; the result is the nearest double far more often than the
//! FreeBSD e_pow.c the `libm` crate carries.

use crate::tables::*;
use crate::{asdouble, asuint64, barrier, invalid, ok, oflow, p2, uflow, MathErr};

const POW_N: u64 = 1 << 7; // POW_LOG_TABLE_BITS
const OFF: u64 = 0x3fe6_9555_0000_0000;
const EXP_N: u64 = 1 << 7; // EXP_TABLE_BITS
const SIGN_BIAS: u32 = 0x800 << 7;

#[inline(always)]
fn top12(x: f64) -> u32 {
    (asuint64(x) >> 52) as u32
}

/// `log(x)` as `hi + tail` with ~15 extra bits. `ix` is x normalized in the subnormal
/// range using the sign bit for the exponent.
#[inline(always)]
fn log_inline(ix: u64) -> (f64, f64) {
    let tmp = ix.wrapping_sub(OFF);
    let i = ((tmp >> (52 - 7)) % POW_N) as usize;
    let k = (tmp as i64) >> 52;
    let iz = ix.wrapping_sub(tmp & (0xfffu64 << 52));
    let z = asdouble(iz);
    let kd = k as f64;
    // tab[i] = { invc, pad, logc, logctail }
    let invc = POW_LOG_TAB[4 * i];
    let logc = POW_LOG_TAB[4 * i + 2];
    let logctail = POW_LOG_TAB[4 * i + 3];
    // Split z such that rhi, rlo and rhi*rhi are exact and |rlo| <= |r|.
    let zhi = asdouble(iz.wrapping_add(1u64 << 31) & (u64::MAX << 32));
    let zlo = z - zhi;
    let rhi = zhi * invc - 1.0;
    let rlo = zlo * invc;
    let r = rhi + rlo;
    // k*Ln2 + log(c) + r.
    let t1 = kd * POW_LOG_LN2HI + logc;
    let t2 = t1 + r;
    let lo1 = kd * POW_LOG_LN2LO + logctail;
    let lo2 = t1 - t2 + r;
    let a = &POW_LOG_POLY;
    let ar = a[0] * r; // A[0] = -0.5
    let ar2 = r * ar;
    let ar3 = r * ar2;
    let arhi = a[0] * rhi;
    let arhi2 = rhi * arhi;
    let hi = t2 + arhi2;
    let lo3 = rlo * (ar + arhi);
    let lo4 = t2 - hi + arhi2;
    // p = log1p(r) - r - A[0]*r*r.
    let p = ar3 * (a[1] + r * a[2] + ar2 * (a[3] + r * a[4] + ar2 * (a[5] + r * a[6])));
    let lo = lo1 + lo2 + lo3 + lo4 + p;
    let y = hi + lo;
    (y, hi - y + lo)
}

fn specialcase(tmp: f64, mut sbits: u64, ki: u64) -> (f64, MathErr) {
    if ki & 0x8000_0000 == 0 {
        // k > 0: the exponent of scale might have overflowed by <= 460.
        sbits = sbits.wrapping_sub(1009u64 << 52);
        let scale = asdouble(sbits);
        let y = p2(1009) * (scale + scale * tmp);
        return (y, if y.is_infinite() { MathErr::Overflow } else { MathErr::None });
    }
    // k < 0: special care in the subnormal range. sbits is the SIGNED scale.
    sbits = sbits.wrapping_add(1022u64 << 52);
    let scale = asdouble(sbits);
    let mut y = scale + scale * tmp;
    if y.abs() < 1.0 {
        let one = if y < 0.0 { -1.0 } else { 1.0 };
        let mut lo = scale - y + scale * tmp;
        let hi = one + y;
        lo = one - hi + y + lo;
        y = barrier(hi + lo) - one;
        // Fix the sign of 0.
        if y == 0.0 {
            y = asdouble(sbits & 0x8000_0000_0000_0000);
        }
        core::hint::black_box(barrier(p2(-1022)) * p2(-1022));
        return (p2(-1022) * y, MathErr::Underflow);
    }
    ok(p2(-1022) * y)
}

/// sign * exp(x + xtail), |xtail| < 2^-8/N and |xtail| <= |x|; `sign_bias` is SIGN_BIAS
/// for a negative result.
#[inline(always)]
fn exp_inline(x: f64, xtail: f64, sign_bias: u32) -> (f64, MathErr) {
    let mut abstop = top12(x) & 0x7ff;
    if abstop.wrapping_sub(top12(p2(-54))) >= top12(512.0).wrapping_sub(top12(p2(-54))) {
        if abstop.wrapping_sub(top12(p2(-54))) >= 0x8000_0000 {
            let one = 1.0 + x;
            return ok(if sign_bias != 0 { -one } else { one });
        }
        if abstop >= top12(1024.0) {
            // inf and nan are already handled.
            return if asuint64(x) >> 63 != 0 { uflow(sign_bias) } else { oflow(sign_bias) };
        }
        abstop = 0;
    }
    let z = EXP_INVLN2N * x;
    let mut kd = barrier(z + EXP_SHIFT);
    let ki = asuint64(kd);
    kd -= EXP_SHIFT;
    let mut r = x + kd * EXP_NEGLN2HIN + kd * EXP_NEGLN2LON;
    r += xtail;
    let idx = (2 * (ki % EXP_N)) as usize;
    let top = ki.wrapping_add(sign_bias as u64).wrapping_shl(52 - 7);
    let tail = asdouble(EXP_TAB[idx]);
    let sbits = EXP_TAB[idx + 1].wrapping_add(top);
    let r2 = r * r;
    let (c2, c3, c4, c5) = (EXP_POLY[0], EXP_POLY[1], EXP_POLY[2], EXP_POLY[3]);
    let tmp = tail + r + r2 * (c2 + r * c3) + r2 * r2 * (c4 + r * c5);
    if abstop == 0 {
        return specialcase(tmp, sbits, ki);
    }
    let scale = asdouble(sbits);
    ok(scale + scale * tmp)
}

/// 0 if not an integer, 1 if odd, 2 if even. `iy` is a non-zero finite value's bits.
fn checkint(iy: u64) -> i32 {
    let e = (iy >> 52 & 0x7ff) as i32;
    if e < 0x3ff {
        return 0;
    }
    if e > 0x3ff + 52 {
        return 2;
    }
    if iy & ((1u64 << (0x3ff + 52 - e)) - 1) != 0 {
        return 0;
    }
    if iy & (1u64 << (0x3ff + 52 - e)) != 0 {
        return 1;
    }
    2
}

/// 0, infinity or NaN.
fn zeroinfnan(i: u64) -> bool {
    i.wrapping_mul(2).wrapping_sub(1) >= asuint64(f64::INFINITY).wrapping_mul(2) - 1
}

/// `pow`. `pow(±0, y<0)` is `Sing`; a negative finite x to a non-integer power is
/// `Domain`; results too large or small for a double are `Overflow`/`Underflow`.
pub fn pow(x: f64, y: f64) -> (f64, MathErr) {
    let mut sign_bias = 0u32;
    let mut ix = asuint64(x);
    let iy = asuint64(y);
    let mut topx = top12(x);
    let topy = top12(y);
    if topx.wrapping_sub(0x001) >= 0x7ff - 0x001 || (topy & 0x7ff).wrapping_sub(0x3be) >= 0x43e - 0x3be {
        // Special cases: (x < 0x1p-126 or inf or nan) or (|y| < 0x1p-65 or |y| >= 0x1p63 or nan).
        if zeroinfnan(iy) {
            if iy.wrapping_mul(2) == 0 {
                return ok(1.0);
            }
            if ix == asuint64(1.0) {
                return ok(1.0);
            }
            if ix.wrapping_mul(2) > asuint64(f64::INFINITY).wrapping_mul(2)
                || iy.wrapping_mul(2) > asuint64(f64::INFINITY).wrapping_mul(2)
            {
                return ok(x + y);
            }
            if ix.wrapping_mul(2) == asuint64(1.0).wrapping_mul(2) {
                return ok(1.0);
            }
            if (ix.wrapping_mul(2) < asuint64(1.0).wrapping_mul(2)) == (iy >> 63 == 0) {
                return ok(0.0); // |x|<1 && y==inf or |x|>1 && y==-inf
            }
            return ok(y * y);
        }
        if zeroinfnan(ix) {
            let mut x2 = x * x;
            if ix >> 63 != 0 && checkint(iy) == 1 {
                x2 = -x2;
            }
            if iy >> 63 != 0 {
                // 1/x2: a pole when x is ±0, plain 0 when x is ±inf.
                let r = barrier(1.0 / x2);
                return (r, if x2 == 0.0 { MathErr::Sing } else { MathErr::None });
            }
            return ok(x2);
        }
        // Here x and y are non-zero finite.
        if ix >> 63 != 0 {
            // Finite x < 0.
            let yint = checkint(iy);
            if yint == 0 {
                return invalid(x);
            }
            if yint == 1 {
                sign_bias = SIGN_BIAS;
            }
            ix &= 0x7fff_ffff_ffff_ffff;
            topx &= 0x7ff;
        }
        if (topy & 0x7ff).wrapping_sub(0x3be) >= 0x43e - 0x3be {
            // sign_bias == 0 here because y is not odd.
            if ix == asuint64(1.0) {
                return ok(1.0);
            }
            if (topy & 0x7ff) < 0x3be {
                // |y| < 2^-65, x^y ~= 1 + y*log(x).
                return ok(if ix > asuint64(1.0) { 1.0 + y } else { 1.0 - y });
            }
            return if (ix > asuint64(1.0)) == (topy < 0x800) { oflow(0) } else { uflow(0) };
        }
        if topx == 0 {
            // Normalize subnormal x so the exponent becomes negative.
            ix = asuint64(x * p2(52));
            ix &= 0x7fff_ffff_ffff_ffff;
            ix = ix.wrapping_sub(52u64 << 52);
        }
    }
    let (hi, lo) = log_inline(ix);
    let yhi = asdouble(iy & (u64::MAX << 27));
    let ylo = y - yhi;
    let lhi = asdouble(asuint64(hi) & (u64::MAX << 27));
    let llo = hi - lhi + lo;
    let ehi = yhi * lhi;
    let elo = ylo * lhi + y * llo; // |elo| < |ehi| * 2^-25
    exp_inline(ehi, elo, sign_bias)
}
