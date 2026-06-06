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

// Bit→float reinterpret routed through the audited floatbits shims so
// this module compiles on the no-SSE i386 kernel (f64::from_bits has a
// Rust-ABI f64 return that needs SSE; transmute does not). See
// floatbits.rs. fabs64 = no-SSE `f64::abs` (sign-bit clear).
use super::floatbits::{fabs64, fb64};

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
            let ip = fb64(((i0 & 0x8000_0000) as u64) << 32);
            return (x - ip, ip);
        }
        let i = (0x000f_ffff_u32) >> j0;
        if ((i0 & i) | i1) == 0 {
            let ip = x;
            let ret = fb64(((i0 & 0x8000_0000) as u64) << 32);
            return (ret, ip);
        }
        let ip = fb64(((i0 & !i) as u64) << 32);
        (x - ip, ip)
    } else if j0 > 51 {
        let ip = x;
        let ret = fb64(((i0 & 0x8000_0000) as u64) << 32);
        (ret, ip)
    } else {
        let i = (0xffff_ffff_u32) >> (j0 - 20);
        if (i1 & i) == 0 {
            let ip = x;
            let ret = fb64(((i0 & 0x8000_0000) as u64) << 32);
            return (ret, ip);
        }
        let ip = fb64(((i0 as u64) << 32) | ((i1 & !i) as u64));
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

// ---------------------------------------------------------------------
// _ecvt — `sprintf("%-+.*E", ndigits, value)` then text surgery
// (ecvt.c). The sprintf float path is the genuine ReactOS
// `printf/streamout.c::format_float`:
//
//   #define get_exp(f) (int)floor(f==0?0:(f>=0?log10(f):log10(-f)))
//   #define round(x)   floor((x)+0.5)
//   fpval2 = round(sign*fpval*pow(10., precision - exponent));
//   if (fpval2 >= (u64)pow(10., precision+1)) { exponent++;
//       fpval2 = round(sign*fpval*pow(10., precision-exponent)); }
//   ...peel `precision` frac digits then the integer digit(s) by
//   `digit = (u64)fpval2 % 10; fpval2 /= 10.;` (f64 divide chain).
//
// `format_float` casts to `long double`; on the Windows target (MSVC
// ABI) that is 64-bit == f64 (the diff ref forces `-mlong-double-64`
// so ref == target == this f64 core — the scanf-P2 precedent). The
// host libm `floor`/`log10`/`pow` are NOT bit-portable to a no_std
// reimplementation, so the diff models the SAME three primitives on
// BOTH sides (the Wave E2 CP1252 discipline): the harness `#define`s
// the genuine `floor`/`log10`/`pow` to C functions bit-identical to
// `fl` below, so the diff pins the genuine ecvt/format_float
// *surgery* deterministically with libm-exactness factored out.
// ---------------------------------------------------------------------

/// `ecvtbuf[DBL_MAX_10_EXP + 10]` (DBL_MAX_10_EXP == 308).
pub const ECVT_BUF: usize = 318;
/// `NUMBER_EFMT` (ecvt's scratch padding).
pub const NUMBER_EFMT: usize = 18;

/// libm primitives the genuine `get_exp`/`round`/`pow` use. The diff
/// harness provides C functions *bit-identical to these* to the
/// reference build (Wave E2 shared-model discipline), so the residual
/// libm-exactness question is modeled the same on both sides instead
/// of being a flaky cross-impl divergence.
pub mod fl {
    /// `floor` for finite f64 (genuine host `floor`; the harness uses
    /// the identical `(double)(long long)x` form). Inputs reaching
    /// here are bounded (|.|<10^18 < 2^63) so the i64 cast is exact.
    // `extern "C"` (x87 ST(0) return, not xmm) so the f64 return compiles
    // on the no-SSE i386 kernel; calling a Rust-defined `extern "C"` fn is
    // safe. (A plain `-> f64` Rust-ABI return hard-errors there.)
    #[inline]
    pub extern "C" fn floor(x: f64) -> f64 {
        // `super::is_finite` is the bit-based local (the std `is_finite`
        // pulls `f64::abs`, whose Rust-ABI f64 return needs SSE).
        if !super::is_finite(x) {
            return x;
        }
        let i = x as i64 as f64;
        if i > x {
            i - 1.0
        } else {
            i
        }
    }

    /// `round(x) = floor(x + 0.5)` — the genuine macro verbatim.
    #[inline]
    pub extern "C" fn round(x: f64) -> f64 {
        floor(x + 0.5)
    }

    /// `pow(10., n)` for integer n (the only use). Binary-exponent
    /// square-and-multiply; the harness reference uses the identical
    /// scheme so `pow` is modeled the same on both sides.
    #[inline]
    pub extern "C" fn pow10(n: i32) -> f64 {
        let mut r = 1.0_f64;
        let mut k = n.unsigned_abs();
        let mut b = 10.0_f64;
        while k > 0 {
            if k & 1 == 1 {
                r *= b;
            }
            b *= b;
            k >>= 1;
        }
        if n < 0 {
            1.0 / r
        } else {
            r
        }
    }

    /// `(int)floor(log10(f))` — `get_exp`. Returns the integer
    /// directly (the harness `#define`s `log10(f)` to a C function
    /// returning exactly this as a double, so `(int)floor(...)` of it
    /// is identity → ref == this by construction).
    #[inline]
    pub fn exp10_floor(mut f: f64) -> i32 {
        if f == 0.0 {
            return 0;
        }
        if f < 0.0 {
            f = -f;
        }
        let bits = f.to_bits();
        let biased = ((bits >> 52) & 0x7ff) as i32;
        let mut e = (((biased - 1023) as f64) * 0.301_029_995_663_981_2) as i32;
        while pow10(e) > f {
            e -= 1;
        }
        while pow10(e + 1) <= f {
            e += 1;
        }
        e
    }
}

/// The genuine `%-+.*E` mantissa as ecvt re-parses it: the decimal
/// digits of `(u64)fpval2` MSD-first, the post-compensation
/// `exponent`, the sign, and a special tag.
pub struct FloatText {
    pub negative: bool, // genuine: fpval < 0 (NOT signbit; -0.0 ⇒ false)
    pub special: u8,    // 0 finite, 1 NaN, 2 Inf
    pub digits: [u8; ECVT_BUF + NUMBER_EFMT + 8],
    pub ndigits: usize,
    pub exp: i32,
}

/// Reproduce `format_float(chr='E', precision, value)`'s digit run +
/// exponent, bit-faithfully (incl. the `(u64)fpval2 % 10; fpval2/=10.`
/// f64-divide peel — NOT a floor-based peel; the chained rounding is
/// part of the genuine output).
pub fn format_e(value: f64, mut precision: i32) -> FloatText {
    let mut out = FloatText {
        negative: value < 0.0,
        special: 0,
        digits: [0; ECVT_BUF + NUMBER_EFMT + 8],
        ndigits: 0,
        exp: 0,
    };
    if is_nan(value) {
        out.special = 1;
        return out;
    }
    if !is_finite(value) {
        out.special = 2;
        return out;
    }
    // format_float precision normalize (ecvt passes >=0; the >17
    // padding is emitted as text by format_float and re-padded by
    // ecvt's own zero loop — same result).
    if precision < 0 {
        precision = 6;
    } else if precision > 17 {
        precision = 17;
    }
    let mut exponent = if value == 0.0 {
        0
    } else {
        fl::floor(fl::exp10_floor(fabs64(value)) as f64) as i32
    };
    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    let mut fpval2 = fl::round(sign * value * fl::pow10(precision - exponent));
    if fpval2 >= fl::pow10(precision + 1) {
        exponent += 1;
        fpval2 = fl::round(sign * value * fl::pow10(precision - exponent));
    }
    out.exp = exponent;
    // Peel exactly like format_float: precision frac digits, then the
    // integer digit(s) (do-while), each step `(u64)x % 10; x /= 10.`.
    let mut lsb = [0u8; ECVT_BUF + NUMBER_EFMT + 8];
    let mut n = 0usize;
    let mut x = fpval2;
    let mut k = precision;
    while k > 0 {
        lsb[n] = b'0' + ((x as u64) % 10) as u8;
        n += 1;
        x /= 10.0;
        k -= 1;
    }
    loop {
        lsb[n] = b'0' + ((x as u64) % 10) as u8;
        n += 1;
        x /= 10.0;
        if (x as u64) == 0 {
            break;
        }
    }
    for i in 0..n {
        out.digits[i] = lsb[n - 1 - i]; // LSB-first → MSD-first
    }
    out.ndigits = n;
    out
}

/// `_ecvt(value, ndigits, *decpt, *sign)` content (ecvt.c lines
/// 15–118). Returns the bytes to copy into the shim's non-reentrant
/// static `ecvtbuf` plus decpt/sign. `special` ⇒ the NaN/Inf
/// `strncmp`/memcpy path — a documented UNFUZZED edge (the genuine
/// `%-+.*E` of NaN under format_float is "+1.#QNAN…", so ecvt's
/// `strncmp(s,"NaN",3)` is dead and the value flows through the
/// finite path on the special glyphs; pinning that exact garbage is
/// owed, not modeled here — scanf-extreme-exponent precedent).
pub struct EcvtResult {
    pub buf: [u8; ECVT_BUF],
    pub buf_len: usize,
    pub decpt: i32,
    pub sign: i32,
}

pub fn ecvt(value: f64, ndigits: i32) -> Option<EcvtResult> {
    let ndigits = if ndigits < 0 { 0 } else { ndigits };
    let ft = format_e(value, ndigits);
    if ft.special != 0 {
        return None; // unfuzzed special path (see doc)
    }
    let mut r = EcvtResult {
        buf: [0; ECVT_BUF],
        buf_len: 0,
        decpt: 0,
        sign: 0,
    };
    r.sign = ft.negative as i32;
    let nd = ndigits as usize;
    let mut d = 0usize;
    // first (integer) digit
    if ft.ndigits > 0 && d < nd {
        r.buf[d] = ft.digits[0];
        d += 1;
    }
    // fractional digits
    let mut si = 1usize;
    while si < ft.ndigits {
        if d < nd {
            r.buf[d] = ft.digits[si];
            d += 1;
        }
        si += 1;
    }
    // `*decpt = atoi(exponent-text) + 1` == exponent + 1
    r.decpt = ft.exp + 1;
    // `if (cvtbuf[1] == '0') *decpt = 0;` — cvtbuf[1] is the first
    // digit char (cvtbuf[0] is the '+'/'-' prefix) == ft.digits[0].
    if ft.ndigits > 0 && ft.digits[0] == b'0' {
        r.decpt = 0;
    }
    // ndigits<1 enhanced-precision nudge: reprint at ndigits+2; if
    // tbuf[1] (first digit) >= '5' → (*decpt)++.
    if ndigits < 1 {
        let t = format_e(value, ndigits + 2);
        if t.special == 0 && t.ndigits > 0 && t.digits[0] >= b'5' {
            r.decpt += 1;
        }
    }
    // pad zeroes: `while (d - ecvtbuf < ndigits) *d++ = '0';`
    while d < nd {
        r.buf[d] = b'0';
        d += 1;
    }
    r.buf[d] = 0; // `*d = '\0';`
    r.buf_len = d + 1;
    Some(r)
}

// ---------------------------------------------------------------------
// _gcvt / _gcvt_s — pure `sprintf("%.*g", ndigit, number)` (gcvt.c).
// The genuine `%.*g` is `format_float(chr='g', flags=0,
// precision=ndigit)` + streamout's generic int/frac emission. For
// `"%.*g"` streamout has no flags and fieldwidth 0, so the output is
// just `prefix` ("-" iff number<0; %g has NO '+') followed by the
// format_float buffer — reproduced verbatim here (no surgery, unlike
// ecvt). Same `fl` shared-model + bounded domain as Phase 2.
//
//   case 'g'/'G': if(precision>0)precision--;
//     if(exponent<-4 || exponent>=precision) goto case_e;   // %e form
//     else { fpval2 = round(sign*fpval*pow(10,precision));
//            while(precision && (u64)fpval2%10==0){precision--;
//                                                  fpval2/=10;} }  // %f
//   ...then padding + `precision` frac digits + optional '.' +
//   integer do-while (digits_l for 'g').
// ---------------------------------------------------------------------

/// The `%.*g` body (no sign) as the genuine sprintf renders it.
pub struct GcvtText {
    pub negative: bool, // number < 0 (prefix "-"); %g has no '+'
    pub special: u8,    // 0 finite, 1 NaN, 2 Inf (unfuzzed)
    pub body: [u8; ECVT_BUF + NUMBER_EFMT + 8],
    pub len: usize,
}

pub fn format_g(value: f64, precision: i32) -> GcvtText {
    let mut out = GcvtText {
        negative: value < 0.0,
        special: 0,
        body: [0; ECVT_BUF + NUMBER_EFMT + 8],
        len: 0,
    };
    if is_nan(value) {
        out.special = 1;
        return out;
    }
    if !is_finite(value) {
        out.special = 2;
        return out;
    }
    // precision normalize (format_float lines 99-105)
    let (mut precision, mut padding) = if precision < 0 {
        (6, 0)
    } else if precision > 17 {
        (17, precision - 17)
    } else {
        (precision, 0)
    };
    let exponent0 = if value == 0.0 {
        0
    } else {
        fl::floor(fl::exp10_floor(fabs64(value)) as f64) as i32
    };
    let sign = if value < 0.0 { -1.0 } else { 1.0 };
    let mut exponent = exponent0;

    // backward scratch (mirrors streamout's `buffer`); we fill from the
    // end and reverse at the close.
    let mut sc = [0u8; ECVT_BUF + NUMBER_EFMT + 8];
    let mut hi = sc.len(); // write position decrements (`*--string`)
    let push = |sc: &mut [u8], hi: &mut usize, c: u8| {
        *hi -= 1;
        sc[*hi] = c;
    };
    let digit = b"0123456789abcdef";

    // case 'g': precision--; decide e-form vs f-form.
    if precision > 0 {
        precision -= 1;
    }
    let mut fpval2;
    let use_e = exponent < -4 || exponent >= precision;
    if use_e {
        // case_e
        fpval2 = fl::round(sign * value * fl::pow10(precision - exponent));
        if fpval2 >= fl::pow10(precision + 1) {
            exponent += 1;
            fpval2 = fl::round(sign * value * fl::pow10(precision - exponent));
        }
        let mut val32 = if exponent >= 0 { exponent } else { -exponent };
        let mut k = 3;
        while k > 0 {
            push(&mut sc, &mut hi, digit[(val32 % 10) as usize]);
            val32 /= 10;
            k -= 1;
        }
        push(&mut sc, &mut hi, if exponent >= 0 { b'+' } else { b'-' });
        push(&mut sc, &mut hi, digit[0xe]); // 'e' (digits_l)
    } else {
        fpval2 = fl::round(sign * value * fl::pow10(precision));
        // strip trailing zeros: `while(precision && (u64)fpval2%10==0)`
        while precision != 0 && (fpval2 as u64) % 10 == 0 {
            precision -= 1;
            fpval2 /= 10.0;
        }
    }

    // common emission (fpval<0 / FORCE_SIGN handled by caller via
    // `negative`; %g sets no '+'). Not nan/inf (special handled above).
    while padding > 0 {
        push(&mut sc, &mut hi, b'0');
        padding -= 1;
    }
    let mut num_digits = precision;
    while num_digits > 0 {
        push(&mut sc, &mut hi, digit[((fpval2 as u64) % 10) as usize]);
        fpval2 /= 10.0; // base == 10
        num_digits -= 1;
    }
    if precision > 0 {
        push(&mut sc, &mut hi, b'.');
    }
    // integer do-while
    loop {
        push(&mut sc, &mut hi, digit[((fpval2 as u64) % 10) as usize]);
        fpval2 /= 10.0;
        if (fpval2 as u64) == 0 {
            break;
        }
    }
    let n = sc.len() - hi;
    out.body[..n].copy_from_slice(&sc[hi..]);
    out.len = n;
    out
}
