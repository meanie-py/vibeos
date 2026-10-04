//! musl src/math/{expf,exp2f,logf,log2f,powf}.c (Copyright (c) 2017-2018, Arm Limited.
//! MIT): single precision, evaluated in double and rounded once at the end.

use crate::tables::*;
use crate::{asdouble, asfloat, asuint, asuint64, barrierf, divzerof, invalidf, ok, oflowf, p2f, uflowf, MathErr};

const EXP2F_N: u64 = 1 << 5; // EXP2F_TABLE_BITS

#[inline(always)]
fn top12(x: f32) -> u32 {
    asuint(x) >> 20
}

/// 2^(k/N) * (C0*r^3 + C1*r^2 + C2*r + 1), shared by exp2f/expf/powf. `poly` is
/// `poly` for exp2f/powf and `poly_scaled` for expf.
#[inline(always)]
fn exp2f_core(r: f64, ki: u64, sign_bias: u64, c: &[f64; 3]) -> f32 {
    let mut t = EXP2F_TAB[(ki % EXP2F_N) as usize];
    t = t.wrapping_add(ki.wrapping_add(sign_bias).wrapping_shl(52 - 5));
    let s = asdouble(t);
    let z = c[0] * r + c[1];
    let r2 = r * r;
    let mut y = c[2] * r + 1.0;
    y = z * r2 + y;
    y *= s;
    y as f32
}

pub fn exp2f(x: f32) -> (f32, MathErr) {
    let xd = x as f64;
    let abstop = top12(x) & 0x7ff;
    if abstop >= top12(128.0) {
        if asuint(x) == asuint(f32::NEG_INFINITY) {
            return ok(0.0);
        }
        if abstop >= top12(f32::INFINITY) {
            return ok(x + x);
        }
        if x > 0.0 {
            return oflowf(0);
        }
        if x <= -150.0 {
            return uflowf(0);
        }
    }
    let mut kd = core::hint::black_box(xd + EXP2F_SHIFT_SCALED);
    let ki = asuint64(kd);
    kd -= EXP2F_SHIFT_SCALED; // k/N
    ok(exp2f_core(xd - kd, ki, 0, &EXP2F_POLY))
}

pub fn expf(x: f32) -> (f32, MathErr) {
    let xd = x as f64;
    let abstop = top12(x) & 0x7ff;
    if abstop >= top12(88.0) {
        if asuint(x) == asuint(f32::NEG_INFINITY) {
            return ok(0.0);
        }
        if abstop >= top12(f32::INFINITY) {
            return ok(x + x);
        }
        if x > f32::from_bits(0x42b1_7217) {
            // x > log(0x1p128) ~= 88.72
            return oflowf(0);
        }
        if x < f32::from_bits(0xc2cf_f1b4) {
            // x < log(0x1p-150) ~= -103.97
            return uflowf(0);
        }
    }
    let z = EXP2F_INVLN2_SCALED * xd;
    let mut kd = core::hint::black_box(z + EXP2F_SHIFT);
    let ki = asuint64(kd);
    kd -= EXP2F_SHIFT;
    ok(exp2f_core(z - kd, ki, 0, &EXP2F_POLY_SCALED))
}

const LOGF_OFF: u32 = 0x3f33_0000;

pub fn logf(x: f32) -> (f32, MathErr) {
    let mut ix = asuint(x);
    if ix == 0x3f80_0000 {
        return ok(0.0);
    }
    if ix.wrapping_sub(0x0080_0000) >= 0x7f80_0000 - 0x0080_0000 {
        if ix.wrapping_mul(2) == 0 {
            return divzerof(1);
        }
        if ix == 0x7f80_0000 {
            return ok(x);
        }
        if (ix & 0x8000_0000) != 0 || ix.wrapping_mul(2) >= 0xff00_0000 {
            return invalidf(x);
        }
        ix = asuint(x * p2f(23));
        ix = ix.wrapping_sub(23 << 23);
    }
    let tmp = ix.wrapping_sub(LOGF_OFF);
    let i = ((tmp >> (23 - 4)) % 16) as usize;
    let k = (tmp as i32) >> 23;
    let iz = ix.wrapping_sub(tmp & 0xff80_0000);
    let invc = LOGF_TAB[2 * i];
    let logc = LOGF_TAB[2 * i + 1];
    let z = asfloat(iz) as f64;
    let r = z * invc - 1.0;
    let y0 = logc + k as f64 * LOGF_LN2;
    let a = &LOGF_POLY;
    let r2 = r * r;
    let mut y = a[1] * r + a[2];
    y = a[0] * r2 + y;
    y = y * r2 + (y0 + r);
    ok(y as f32)
}

pub fn log2f(x: f32) -> (f32, MathErr) {
    let mut ix = asuint(x);
    if ix == 0x3f80_0000 {
        return ok(0.0);
    }
    if ix.wrapping_sub(0x0080_0000) >= 0x7f80_0000 - 0x0080_0000 {
        if ix.wrapping_mul(2) == 0 {
            return divzerof(1);
        }
        if ix == 0x7f80_0000 {
            return ok(x);
        }
        if (ix & 0x8000_0000) != 0 || ix.wrapping_mul(2) >= 0xff00_0000 {
            return invalidf(x);
        }
        ix = asuint(x * p2f(23));
        ix = ix.wrapping_sub(23 << 23);
    }
    let tmp = ix.wrapping_sub(LOGF_OFF);
    let i = ((tmp >> (23 - 4)) % 16) as usize;
    let top = tmp & 0xff80_0000;
    let iz = ix.wrapping_sub(top);
    let k = (tmp as i32) >> 23;
    let invc = LOG2F_TAB[2 * i];
    let logc = LOG2F_TAB[2 * i + 1];
    let z = asfloat(iz) as f64;
    let r = z * invc - 1.0;
    let y0 = logc + k as f64;
    let a = &LOG2F_POLY;
    let r2 = r * r;
    let mut y = a[1] * r + a[2];
    y = a[0] * r2 + y;
    let p = a[3] * r + y0;
    y = y * r2 + p;
    ok(y as f32)
}

/// log2(x) for powf; subnormal x is normalized so ix has a negative biased exponent.
#[inline(always)]
fn powf_log2_inline(ix: u32) -> f64 {
    let tmp = ix.wrapping_sub(LOGF_OFF);
    let i = ((tmp >> (23 - 4)) % 16) as usize;
    let top = tmp & 0xff80_0000;
    let iz = ix.wrapping_sub(top);
    let k = (top as i32) >> 23; // POWF_SCALE_BITS = 0
    let invc = POWF_LOG2_TAB[2 * i];
    let logc = POWF_LOG2_TAB[2 * i + 1];
    let z = asfloat(iz) as f64;
    let r = z * invc - 1.0;
    let y0 = logc + k as f64;
    let a = &POWF_LOG2_POLY;
    let r2 = r * r;
    let y = a[0] * r + a[1];
    let p = a[2] * r + a[3];
    let r4 = r2 * r2;
    let mut q = a[4] * r + y0;
    q = p * r2 + q;
    y * r4 + q
}

fn checkintf(iy: u32) -> i32 {
    let e = (iy >> 23 & 0xff) as i32;
    if e < 0x7f {
        return 0;
    }
    if e > 0x7f + 23 {
        return 2;
    }
    if iy & ((1u32 << (0x7f + 23 - e)) - 1) != 0 {
        return 0;
    }
    if iy & (1u32 << (0x7f + 23 - e)) != 0 {
        return 1;
    }
    2
}

fn zeroinfnanf(ix: u32) -> bool {
    ix.wrapping_mul(2).wrapping_sub(1) >= 2u32 * 0x7f80_0000 - 1
}

const POWF_SIGN_BIAS: u32 = 1 << (5 + 11);

pub fn powf(x: f32, y: f32) -> (f32, MathErr) {
    let mut sign_bias = 0u32;
    let mut ix = asuint(x);
    let iy = asuint(y);
    if ix.wrapping_sub(0x0080_0000) >= 0x7f80_0000 - 0x0080_0000 || zeroinfnanf(iy) {
        if zeroinfnanf(iy) {
            if iy.wrapping_mul(2) == 0 {
                return ok(1.0);
            }
            if ix == 0x3f80_0000 {
                return ok(1.0);
            }
            if ix.wrapping_mul(2) > 2u32 * 0x7f80_0000 || iy.wrapping_mul(2) > 2u32 * 0x7f80_0000 {
                return ok(x + y);
            }
            if ix.wrapping_mul(2) == 2 * 0x3f80_0000 {
                return ok(1.0);
            }
            if (ix.wrapping_mul(2) < 2 * 0x3f80_0000) == (iy & 0x8000_0000 == 0) {
                return ok(0.0);
            }
            return ok(y * y);
        }
        if zeroinfnanf(ix) {
            let mut x2 = x * x;
            if ix & 0x8000_0000 != 0 && checkintf(iy) == 1 {
                x2 = -x2;
            }
            if iy & 0x8000_0000 != 0 {
                let r = barrierf(1.0 / x2);
                return (r, if x2 == 0.0 { MathErr::Sing } else { MathErr::None });
            }
            return ok(x2);
        }
        // x and y are non-zero finite.
        if ix & 0x8000_0000 != 0 {
            let yint = checkintf(iy);
            if yint == 0 {
                return invalidf(x);
            }
            if yint == 1 {
                sign_bias = POWF_SIGN_BIAS;
            }
            ix &= 0x7fff_ffff;
        }
        if ix < 0x0080_0000 {
            ix = asuint(x * p2f(23));
            ix &= 0x7fff_ffff;
            ix = ix.wrapping_sub(23 << 23);
        }
    }
    let logx = powf_log2_inline(ix);
    let ylogx = y as f64 * logx; // cannot overflow, y is single precision
    if (asuint64(ylogx) >> 47 & 0xffff) >= asuint64(126.0) >> 47 {
        // |y*log(x)| >= 126.
        if ylogx > f64::from_bits(0x405f_ffff_ffd1_d571) {
            return oflowf(sign_bias);
        }
        if ylogx <= -150.0 {
            return uflowf(sign_bias);
        }
    }
    // exp2_inline(ylogx, sign_bias)
    let mut kd = core::hint::black_box(ylogx + EXP2F_SHIFT_SCALED);
    let ki = asuint64(kd);
    kd -= EXP2F_SHIFT_SCALED;
    ok(exp2f_core(ylogx - kd, ki, sign_bias as u64, &EXP2F_POLY))
}
