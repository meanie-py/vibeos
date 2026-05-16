//! Float→decimal-string cores — faithful to ReactOS libcntpr
//! `stdlib/fcvtbuf.c` (the sanos `cvt()`), `stdlib/fcvt.c`. (`_ecvt`/
//! `_gcvt` route through the genuine `streamout.c` sprintf and land in
//! later phases.)
//!
//! `_fcvt`→`fcvtbuf`→`cvt(.., eflag=0)` is a `modf`-based digit-by-
//! digit loop with the `(fj + .03) * 10` integer-part fudge and a
//! carry-propagating `*p1 += 5` half-up round. Bit-faithful port: the
//! genuine uses host `modf`, which is exact for binary64; the
//! `modf` below is the fdlibm `s_modf` bit-form (also exact) so cnt
//! and the genuine reference agree bit-for-bit on every finite double.
//! `double` is IEEE754 binary64 on host and the Windows target alike
//! (no x87) — no long-double caveat here.
//!
//! NaN/Inf go through `snprintf(buf, ndigits, "1.#QNAN"/"1.#INF")`
//! where `ndigits` is the snprintf SIZE (a genuine truncating quirk);
//! that special path is a documented unfuzzed domain limit (the
//! scanf-extreme-exponent / strpbrk≤2 precedent) — `is_nan`/`is_finite`
//! are kept faithful for when a later phase pins it.

#![allow(clippy::needless_range_loop)]

/// `fcvt_buf[2 * DBL_MAX_10_EXP + 10]` and `CVTBUFSIZE`
/// (DBL_MAX_10_EXP == 308).
pub const FCVT_BUF: usize = 2 * 308 + 10; // 626

/// `_isnan` (bit form, genuine `math/_isnan.c`).
#[inline]
pub fn is_nan(x: f64) -> bool {
    (x.to_bits() & !0x8000_0000_0000_0000) > 0x7FF0_0000_0000_0000
}
/// `_finite` (bit form, genuine `math/_finite.c`).
#[inline]
pub fn is_finite(x: f64) -> bool {
    (x.to_bits() & 0x7FF0_0000_0000_0000) != 0x7FF0_0000_0000_0000
}

/// Genuine `math/s_modf.c::modf` → `(frac, ipart)`. Bit-faithful
/// (EXTRACT_WORDS / INSERT_WORDS via to_bits/from_bits); exact for all
/// finite f64, matching host glibc `modf` used by the reference.
fn modf(x: f64) -> (f64, f64) {
    let bits = x.to_bits();
    let i0 = (bits >> 32) as u32;
    let i1 = bits as u32;
    let j0 = (((i0 >> 20) & 0x7ff) as i32) - 0x3ff;
    if j0 < 20 {
        if j0 < 0 {
            // |x| < 1 → ipart = ±0
            let ip = f64::from_bits(((i0 & 0x8000_0000) as u64) << 32);
            return (x - ip, ip);
        }
        let i = (0x000f_ffff_u32) >> j0;
        if ((i0 & i) | i1) == 0 {
            let ip = x;
            let ret = f64::from_bits(((i0 & 0x8000_0000) as u64) << 32);
            return (ret, ip);
        }
        let ip = f64::from_bits(((i0 & !i) as u64) << 32);
        (x - ip, ip)
    } else if j0 > 51 {
        let ip = x;
        let ret = f64::from_bits(((i0 & 0x8000_0000) as u64) << 32);
        (ret, ip)
    } else {
        let i = (0xffff_ffff_u32) >> (j0 - 20);
        if (i1 & i) == 0 {
            let ip = x;
            let ret = f64::from_bits(((i0 & 0x8000_0000) as u64) << 32);
            return (ret, ip);
        }
        let ip = f64::from_bits(((i0 as u64) << 32) | ((i1 & !i) as u64));
        (x - ip, ip)
    }
}

/// One `_fcvt`/`fcvtbuf` result. `buf[..buf_len]` is the genuine
/// `cvt()` output INCLUDING the terminating NUL (so the shim copies it
/// verbatim into the static `fcvt_buf`). `decpt`/`sign` are the
/// out-params.
pub struct FcvtResult {
    pub buf: [u8; FCVT_BUF],
    pub buf_len: usize,
    pub decpt: i32,
    pub sign: i32,
}

/// `cvt(arg, ndigits, *decpt, *sign, buf, eflag)` — `fcvtbuf` always
/// passes `eflag == 0`. Bit-faithful to fcvtbuf.c lines 45–136.
/// `special` ⇒ caller (the shim) emits the genuine
/// `snprintf(buf, ndigits, "1.#QNAN"/"1.#INF")`; this is the
/// documented unfuzzed edge.
pub fn fcvtbuf(mut arg: f64, mut ndigits: i32) -> Result<FcvtResult, (bool, i32, i32)> {
    // Err((is_inf, decpt=0, sign=0)) signals the special path.
    if is_nan(arg) {
        return Err((false, 0, 0));
    }
    if !is_finite(arg) {
        return Err((true, 0, 0));
    }
    let mut r = FcvtResult {
        buf: [0; FCVT_BUF],
        buf_len: 0,
        decpt: 0,
        sign: 0,
    };
    let cvtbufsize = FCVT_BUF as i32; // 626
    if ndigits >= cvtbufsize - 1 {
        ndigits = cvtbufsize - 2;
    }
    let mut r2: i32 = 0;
    r.sign = 0;
    let mut p: usize = 0;
    if arg < 0.0 {
        r.sign = 1;
        arg = -arg;
    }
    let (mut argf, mut fi) = modf(arg);
    if fi != 0.0 {
        // `p1 = &buf[CVTBUFSIZE];` then digits written backward.
        let mut p1: usize = cvtbufsize as usize;
        while fi != 0.0 {
            let (fj, nfi) = modf(fi / 10.0);
            fi = nfi;
            p1 -= 1;
            // `*--p1 = (int)((fj + .03) * 10) + '0';`
            let dig = ((fj + 0.03) * 10.0) as i32;
            r.buf[p1] = (dig as u8).wrapping_add(b'0');
            r2 += 1;
        }
        // `while (p1 < &buf[CVTBUFSIZE]) *p++ = *p1++;`
        while p1 < cvtbufsize as usize {
            r.buf[p] = r.buf[p1];
            p += 1;
            p1 += 1;
        }
    } else if argf > 0.0 {
        // `while ((fj = arg * 10) < 1) { arg = fj; r2--; }`
        loop {
            let fj = argf * 10.0;
            if fj >= 1.0 {
                break;
            }
            argf = fj;
            r2 -= 1;
        }
    }
    // `p1 = &buf[ndigits]; if (eflag==0) p1 += r2; *decpt = r2;`
    let p1i: i32 = ndigits + r2; // eflag == 0
    r.decpt = r2;
    if p1i < 0 {
        // `if (p1 < &buf[0]) { buf[0]='\0'; return buf; }`
        r.buf[0] = 0;
        r.buf_len = 1;
        return Ok(r);
    }
    let p1u = p1i as usize;
    // `while (p <= p1 && p < &buf[CVTBUFSIZE]) { arg*=10;
    //   arg = modf(arg,&fj); *p++ = (int)fj + '0'; }`
    while p <= p1u && p < cvtbufsize as usize {
        argf *= 10.0;
        let (nf, fj) = modf(argf);
        argf = nf;
        r.buf[p] = (fj as i32 as u8).wrapping_add(b'0');
        p += 1;
    }
    if p1u >= cvtbufsize as usize {
        // `buf[CVTBUFSIZE-1]='\0'; return buf;`
        r.buf[cvtbufsize as usize - 1] = 0;
        r.buf_len = cvtbufsize as usize;
        return Ok(r);
    }
    // round: `p = p1; *p1 += 5; while (*p1 > '9') { ... }`
    let mut p1r = p1u;
    p = p1u;
    r.buf[p1r] = r.buf[p1r].wrapping_add(5);
    while r.buf[p1r] > b'9' {
        r.buf[p1r] = b'0';
        if p1r > 0 {
            // `if (p1 > buf) ++*--p1;`
            p1r -= 1;
            r.buf[p1r] = r.buf[p1r].wrapping_add(1);
        } else {
            // `else { *p1='1'; (*decpt)++; if(eflag==0){
            //   if(p>buf)*p='0'; p++; } }`
            r.buf[p1r] = b'1';
            r.decpt += 1;
            if p > 0 {
                r.buf[p] = b'0';
            }
            p += 1;
        }
    }
    r.buf[p] = 0; // `*p = '\0';`
    r.buf_len = p + 1;
    Ok(r)
}
