//! Hand-written cnt-rust math shims — Wave MATH-CLASS.
//!
//! The IEEE-754 classification / sign predicates and the integer
//! `abs`/`labs`. Every one is *pure bit manipulation* (reinterpret the
//! float as its integer image, mask, compare) or pure integer — there
//! is **no floating-point arithmetic**, so they compile and run on the
//! i386 no-SSE kernel build with the crate's `float` feature OFF (the
//! `f64`/`f32` arguments arrive on the stack and are read as their bit
//! image; `to_bits()` is a bitcast, never an FP op). Uniform Rust on
//! all three arches, replacing the C in
//! `math/{abs,labs,_isnan,_isnanf,_finite,_finitef,_dclass,_fdclass,
//! _dsign,_fdsign,_dtest,_fdtest}.c` (REMOVE_ITEM'd from libcntpr only;
//! msvcrt keeps the C). Differential-pinned in cnt-difftest.
//!
//! Not a gen_cnt family (float signatures + bespoke bodies); this
//! module is stitched into `cabi` like `streamout.rs`.

// ReactOS <math.h> classification codes (sdk/include/crt/math.h):
//   _NANCODE 2, _INFCODE 1, _FINITE -1, _DENORM -2, FP_ZERO 0.
// (These differ from the host/glibc FP_* values — hardcode the genuine
// kernel-header values so _dclass/_fdclass match the C bit-for-bit.)
const FP_NAN: i16 = 2;
const FP_INFINITE: i16 = 1;
const FP_NORMAL: i16 = -1;
const FP_SUBNORMAL: i16 = -2;
const FP_ZERO: i16 = 0;

// ---- integer abs (no FP at all) ----------------------------------------

/// `int abs(int)` — `wrapping_neg` matches C's INT_MIN wrap (the C
/// `j<0 ? -j : j` is UB at INT_MIN but in practice returns INT_MIN).
#[no_mangle]
pub extern "C" fn abs(j: i32) -> i32 {
    if j < 0 {
        j.wrapping_neg()
    } else {
        j
    }
}

/// `long labs(long)` — Windows `long` is 32-bit on all three targets.
#[no_mangle]
pub extern "C" fn labs(j: i32) -> i32 {
    if j < 0 {
        j.wrapping_neg()
    } else {
        j
    }
}

// ---- IEEE-754 predicates (bitcast + mask, no FP arithmetic) ------------

/// `_isnan(double)` — `(bits & ~sign) > +inf-bits`.
#[no_mangle]
pub extern "C" fn _isnan(x: f64) -> i32 {
    ((x.to_bits() & 0x7FFF_FFFF_FFFF_FFFF) > 0x7FF0_0000_0000_0000) as i32
}

/// `_isnanf(float)`.
#[no_mangle]
pub extern "C" fn _isnanf(x: f32) -> i32 {
    ((x.to_bits() & 0x7FFF_FFFF) > 0x7F80_0000) as i32
}

/// `_finite(double)` — exponent field not all-ones.
#[no_mangle]
pub extern "C" fn _finite(x: f64) -> i32 {
    ((x.to_bits() & 0x7FF0_0000_0000_0000) != 0x7FF0_0000_0000_0000) as i32
}

/// `_finitef(float)`.
#[no_mangle]
pub extern "C" fn _finitef(x: f32) -> i32 {
    ((x.to_bits() & 0x7F80_0000) != 0x7F80_0000) as i32
}

/// `_dsign(double)` — "what Windows returns": `(bits >> 48) & 0x8000`.
#[no_mangle]
pub extern "C" fn _dsign(x: f64) -> i32 {
    ((x.to_bits() >> 48) & 0x8000) as i32
}

/// `_fdsign(float)` — `(bits >> 16) & 0x8000`.
#[no_mangle]
pub extern "C" fn _fdsign(x: f32) -> i32 {
    ((x.to_bits() >> 16) & 0x8000) as i32
}

// ---- IEEE-754 classification ------------------------------------------
// Private inline cores operate on the bit image so `_dtest`/`_fdtest`
// never have to pass an `f64`/`f32` by value (which on i386-no-SSE could
// route a double through x87) — they read the bits and classify directly.

#[inline]
fn dclass_bits(u: u64) -> i16 {
    let e = u & 0x7FF0_0000_0000_0000;
    let m = u & 0x000F_FFFF_FFFF_FFFF;
    if e == 0x7FF0_0000_0000_0000 {
        if m != 0 {
            FP_NAN
        } else {
            FP_INFINITE
        }
    } else if e == 0 {
        if m != 0 {
            FP_SUBNORMAL
        } else {
            FP_ZERO
        }
    } else {
        FP_NORMAL
    }
}

#[inline]
fn fdclass_bits(u: u32) -> i16 {
    let e = u & 0x7F80_0000;
    let m = u & 0x007F_FFFF;
    if e == 0x7F80_0000 {
        if m != 0 {
            FP_NAN
        } else {
            FP_INFINITE
        }
    } else if e == 0 {
        if m != 0 {
            FP_SUBNORMAL
        } else {
            FP_ZERO
        }
    } else {
        FP_NORMAL
    }
}

/// `short _dclass(double)`.
#[no_mangle]
pub extern "C" fn _dclass(x: f64) -> i16 {
    dclass_bits(x.to_bits())
}

/// `short _fdclass(float)`.
#[no_mangle]
pub extern "C" fn _fdclass(x: f32) -> i16 {
    fdclass_bits(x.to_bits())
}

/// `short _dtest(double*)` — classify `*px`. Reads the bit image
/// directly (no `f64` pass-by-value → no x87 on i386).
#[no_mangle]
pub unsafe extern "C" fn _dtest(px: *const f64) -> i16 {
    dclass_bits((px as *const u64).read_unaligned())
}

/// `short _fdtest(float*)`.
#[no_mangle]
pub unsafe extern "C" fn _fdtest(px: *const f32) -> i16 {
    fdclass_bits((px as *const u32).read_unaligned())
}

// ======================================================================
// Wave MATH-TIERB — float-returning bit-ops + integer `div`.
//
// Unlike Tier A (int-returning predicates), these RETURN floats by value.
// On i386 the crate is built no-SSE, where rustc 1.92 makes an `f32`/`f64`
// *return* a hard ABI error (float *args* are only a warning — hence Tier
// A's predicates compiled). So every float-returning shim here is gated
// behind the `float` feature, which is ON for amd64/arm64 and OFF for the
// i386 kernel link — i386 keeps the C (exactly the printf/format_float
// split). `div` has no float in its signature, so it is universal (all 3).
// `nexttowardf` is additionally i386-incompatible by ABI: its second arg
// is an 80-bit x87 `long double` Rust cannot name (on amd64/arm64
// `long double == double`).
// ======================================================================

// ---- _chgsignf — pure sign-bit flip (no FP arithmetic) ----------------

/// `float _chgsignf(float)` — C does `(float)_chgsign((double)x)`, i.e.
/// flip the sign. Sign flip commutes with the float↔double round-trip
/// exactly, so it is just `bits ^ sign`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn _chgsignf(x: f32) -> f32 {
    f32::from_bits(x.to_bits() ^ 0x8000_0000)
}

// ---- scalbn / scalbnf — musl (FP multiply by power-of-two) ------------
// Hex-float constants from the musl source, written via `from_bits` so
// they are bit-exact (Rust has no hex-float literals):
//   0x1p1023  = 0x7FE0_0000_0000_0000   0x1p-1022*0x1p53 = 2^-969 = 0x0360_…
//   0x1p127f  = 0x7F00_0000             0x1p-126f*0x1p24f = 2^-102 = 0x0C80_0000

/// `double scalbn(double, int)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn scalbn(x: f64, mut n: i32) -> f64 {
    let mut y = x;
    if n > 1023 {
        y *= f64::from_bits(0x7FE0_0000_0000_0000);
        n -= 1023;
        if n > 1023 {
            y *= f64::from_bits(0x7FE0_0000_0000_0000);
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        y *= f64::from_bits(0x0360_0000_0000_0000);
        n += 1022 - 53;
        if n < -1022 {
            y *= f64::from_bits(0x0360_0000_0000_0000);
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    let u = ((0x3ff + n) as u64) << 52;
    y * f64::from_bits(u)
}

/// `float scalbnf(float, int)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn scalbnf(x: f32, mut n: i32) -> f32 {
    let mut y = x;
    if n > 127 {
        y *= f32::from_bits(0x7F00_0000);
        n -= 127;
        if n > 127 {
            y *= f32::from_bits(0x7F00_0000);
            n -= 127;
            if n > 127 {
                n = 127;
            }
        }
    } else if n < -126 {
        y *= f32::from_bits(0x0C80_0000);
        n += 126 - 24;
        if n < -126 {
            y *= f32::from_bits(0x0C80_0000);
            n += 126 - 24;
            if n < -126 {
                n = -126;
            }
        }
    }
    let u = ((0x7f + n) as u32) << 23;
    y * f32::from_bits(u)
}

// ---- round / roundf — ReactOS portable (ceil/floor + FP add) ----------

#[cfg(feature = "float")]
extern "C" {
    fn ceil(x: f64) -> f64;
    fn floor(x: f64) -> f64;
    fn ceilf(x: f32) -> f32;
    fn floorf(x: f32) -> f32;
}

/// `double round(double)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn round(arg: f64) -> f64 {
    unsafe {
        if arg < 0.0 {
            ceil(arg - 0.5)
        } else {
            floor(arg + 0.5)
        }
    }
}

/// `float roundf(float)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn roundf(arg: f32) -> f32 {
    unsafe {
        if arg < 0.0 {
            ceilf(arg - 0.5)
        } else {
            floorf(arg + 0.5)
        }
    }
}

// ---- nextafter / nextafterf — musl (bit walk; FP only on NaN path) ----
// The musl `FORCE_EVAL(x+x)` / `FORCE_EVAL(x*x + …)` calls exist solely
// to raise fenv overflow/underflow flags; the kernel has no fenv
// consumer and they do not affect the return value, so they are omitted.

/// `double nextafter(double, double)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn nextafter(x: f64, y: f64) -> f64 {
    let mut uxi = x.to_bits();
    let uyi = y.to_bits();
    if _isnan(x) != 0 || _isnan(y) != 0 {
        return x + y;
    }
    if uxi == uyi {
        return y;
    }
    let ax = uxi & 0x7FFF_FFFF_FFFF_FFFF;
    let ay = uyi & 0x7FFF_FFFF_FFFF_FFFF;
    if ax == 0 {
        if ay == 0 {
            return y;
        }
        uxi = (uyi & (1u64 << 63)) | 1;
    } else if ax > ay || ((uxi ^ uyi) & (1u64 << 63)) != 0 {
        uxi -= 1;
    } else {
        uxi += 1;
    }
    f64::from_bits(uxi)
}

/// `float nextafterf(float, float)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn nextafterf(x: f32, y: f32) -> f32 {
    let mut uxi = x.to_bits();
    let uyi = y.to_bits();
    if _isnanf(x) != 0 || _isnanf(y) != 0 {
        return x + y;
    }
    if uxi == uyi {
        return y;
    }
    let ax = uxi & 0x7FFF_FFFF;
    let ay = uyi & 0x7FFF_FFFF;
    if ax == 0 {
        if ay == 0 {
            return y;
        }
        uxi = (uyi & 0x8000_0000) | 1;
    } else if ax > ay || ((uxi ^ uyi) & 0x8000_0000) != 0 {
        uxi -= 1;
    } else {
        uxi += 1;
    }
    f32::from_bits(uxi)
}

// ---- nexttowardf — musl; amd64/arm64 only (i386 long double is 80-bit) -
// On Windows amd64/arm64 `long double == double`, so the second argument
// is a plain `f64`. On i386 it is an 80-bit x87 type Rust cannot name,
// so i386 keeps the C implementation (not REMOVE_ITEM'd there).

/// `float nexttowardf(float, long double)`.
#[cfg(all(feature = "float", not(target_arch = "x86")))]
#[no_mangle]
pub extern "C" fn nexttowardf(x: f32, y: f64) -> f32 {
    let mut uxi = x.to_bits();
    if _isnanf(x) != 0 || _isnan(y) != 0 {
        return x + y as f32;
    }
    if x as f64 == y {
        return y as f32;
    }
    if x == 0.0 {
        uxi = 1;
        if (y.to_bits() >> 63) != 0 {
            uxi |= 0x8000_0000;
        }
    } else if (x as f64) < y {
        if (uxi >> 31) != 0 {
            uxi -= 1;
        } else {
            uxi += 1;
        }
    } else if (uxi >> 31) != 0 {
        uxi += 1;
    } else {
        uxi -= 1;
    }
    f32::from_bits(uxi)
}

// ---- div — pure integer, struct-return ABI ----------------------------

/// `div_t { int quot; int rem; }`.
#[repr(C)]
pub struct div_t {
    pub quot: i32,
    pub rem: i32,
}

/// `div_t div(int num, int denom)` — DJ Delorie's C-truncation-toward-zero
/// implementation (matches MSVCRT `div`).
#[no_mangle]
pub extern "C" fn div(mut num: i32, mut denom: i32) -> div_t {
    if num > 0 && denom < 0 {
        num = -num;
        denom = -denom;
    }
    let mut r = div_t {
        quot: num / denom,
        rem: num % denom,
    };
    if num < 0 && denom > 0 && r.rem > 0 {
        r.quot += 1;
        r.rem -= denom;
    }
    r
}

// ======================================================================
// Wave MATH-TIERC — transcendentals (amd64/arm64 only; i386 keeps its
// x87 asm — same float-return ABI constraint as Tier B).
//
// Sub-wave 1: sin / cos / sqrt / sqrtf.
//   sin/cos: faithful port of ReactOS `math/{sin,cos}.c` (Timo Kreuzer's
//     quadrant reduction + unrolled Taylor series, PRECISION=9). Both
//     amd64 (llvm-mingw) and arm64 used that same portable C, so one Rust
//     impl serves both; bit-exact vs the C on the finite domain (the
//     `(int)` quadrant cast is meaningful only for moderate |x|).
//   sqrt/sqrtf: the hardware instruction (sqrtsd/sqrtss on amd64, fsqrt on
//     arm64) — IEEE correctly-rounded, so it matches host libm bit-for-bit
//     and is >= the precision of the arm64 portable `math/sqrt.c` (a
//     fast-inverse-sqrt approximation) it replaces.
// ======================================================================

#[cfg(feature = "float")]
const M_PI: f64 = 3.141592653589793238462643;

/// Shared quadrant-reduced Taylor core for `sin`/`cos` (PRECISION=9).
/// `q` is the already-normalized quadrant (sin: `(quad-1)&3`, cos:
/// `quad&3`). Mirrors the `sin_off_tbl`/`sin_sign_tbl` of `math/sin.c`.
#[cfg(feature = "float")]
#[inline]
fn sincos_taylor(mut x: f64, q: usize) -> f64 {
    const OFF: [f64; 4] = [0.0, -M_PI / 2.0, 0.0, -M_PI / 2.0];
    const SIGN: [f64; 4] = [1.0, -1.0, -1.0, 1.0];
    x += OFF[q];
    let x2 = -(x * x);
    let mut result = 0.0f64;
    // PRECISION >= 9 .. >= 5 (1/18! down to 1/10!)
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0 * 7.0 * 8.0 * 9.0 * 10.0 * 11.0 * 12.0 * 13.0 * 14.0 * 15.0 * 16.0 * 17.0 * 18.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0 * 7.0 * 8.0 * 9.0 * 10.0 * 11.0 * 12.0 * 13.0 * 14.0 * 15.0 * 16.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0 * 7.0 * 8.0 * 9.0 * 10.0 * 11.0 * 12.0 * 13.0 * 14.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0 * 7.0 * 8.0 * 9.0 * 10.0 * 11.0 * 12.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0 * 7.0 * 8.0 * 9.0 * 10.0);
    result *= x2;
    // unconditional terms 1/8! .. 1/2!
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0 * 7.0 * 8.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0 * 5.0 * 6.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0 * 3.0 * 4.0);
    result *= x2;
    result += 1.0 / (1.0 * 2.0);
    result *= x2;
    result += 1.0;
    result * SIGN[q]
}

/// `double sin(double)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn sin(x: f64) -> f64 {
    let quadrant = (x * (2.0 / M_PI)) as i32;
    let xr = x - (quadrant as f64) * (M_PI / 2.0);
    sincos_taylor(xr, ((quadrant - 1) & 0x3) as usize)
}

/// `double cos(double)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn cos(x: f64) -> f64 {
    let quadrant = (x * (2.0 / M_PI)) as i32;
    let xr = x - (quadrant as f64) * (M_PI / 2.0);
    sincos_taylor(xr, (quadrant & 0x3) as usize)
}

/// `double sqrt(double)` — hardware `sqrtsd` (correctly rounded; handles
/// ±0/±inf/NaN/negative per IEEE, so no special-case prologue needed).
#[cfg(all(feature = "float", target_arch = "x86_64"))]
#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    let r: f64;
    unsafe {
        core::arch::asm!("sqrtsd {0}, {1}", out(xmm_reg) r, in(xmm_reg) x, options(pure, nomem, nostack));
    }
    r
}

/// `double sqrt(double)` — hardware `fsqrt`.
#[cfg(all(feature = "float", target_arch = "aarch64"))]
#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    let r: f64;
    unsafe {
        core::arch::asm!("fsqrt {0:d}, {1:d}", out(vreg) r, in(vreg) x, options(pure, nomem, nostack));
    }
    r
}

/// `float sqrtf(float)` — hardware `sqrtss`.
#[cfg(all(feature = "float", target_arch = "x86_64"))]
#[no_mangle]
pub extern "C" fn sqrtf(x: f32) -> f32 {
    let r: f32;
    unsafe {
        core::arch::asm!("sqrtss {0}, {1}", out(xmm_reg) r, in(xmm_reg) x, options(pure, nomem, nostack));
    }
    r
}

/// `float sqrtf(float)` — hardware `fsqrt` (single).
#[cfg(all(feature = "float", target_arch = "aarch64"))]
#[no_mangle]
pub extern "C" fn sqrtf(x: f32) -> f32 {
    let r: f32;
    unsafe {
        core::arch::asm!("fsqrt {0:s}, {1:s}", out(vreg) r, in(vreg) x, options(pure, nomem, nostack));
    }
    r
}

// ===========================================================================
// Wave MATH-TIERC sub-wave 2 — exp / log / log10 / pow / fmod (f64)
//
// Faithful ports of the portable kernel-CRT C used on BOTH amd64-llvm-mingw
// and arm64 (`math/arm64/{exp,log,log10,pow,fmod}.c`). `exp`/`log`/`fmod` are
// real fdlibm-style implementations; `log10`/`pow` are the upstream's
// deliberate rough stubs (their `FIXME: general case`), translated verbatim
// so the difftest stays bit-exact against the C reference. amd64/arm64 only
// (the `float` feature) — i386 keeps its x87 `*_asm.s` (no-SSE: an `f64`
// return is a hard ABI error). All bitwise; the variable-count shifts use
// `wrapping_sh*` to mirror x86's count-masking (the C is technically UB on a
// >=64 count but the host reference compiles to masked `shr`/`shl`).
// ===========================================================================

/// `double exp(double)` — range reduction + degree-5 rational approximation.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn exp(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058500170e-10;
    const INV_LN2: f64 = 1.44269504088896338700e+00;
    const P1: f64 = 1.66666666666666019037e-01;
    const P2: f64 = -2.77777777770155933842e-03;
    const P3: f64 = 6.61375632143793436117e-05;
    const P4: f64 = -1.65339022054652515390e-06;
    const P5: f64 = 4.13813679705723846039e-08;

    let mut bits = x.to_bits();
    let hx = ((bits >> 32) as u32) & 0x7FFF_FFFF;

    // Filter special cases: |x| >= 709.78...
    if hx >= 0x4086_2E42 {
        if hx >= 0x7FF0_0000 {
            // NaN or Inf
            let lx = (bits & 0xFFFF_FFFF) as u32;
            if ((hx & 0xF_FFFF) | lx) != 0 {
                return x; // NaN
            }
            return if (bits >> 63) != 0 { 0.0 } else { x }; // exp(-inf)=0, exp(inf)=inf
        }
        if x > 709.782712893383973096 {
            return f64::from_bits(0x7FF0_0000_0000_0000); // overflow → +inf
        }
        if x < -745.13321910194110842 {
            return 0.0; // underflow
        }
    }

    // Argument reduction: r = x - k*ln2, |r| <= ln2/2
    let r: f64;
    let k: i32;
    if hx > 0x3FD6_2E42 {
        // |x| > 0.5*ln2
        if hx < 0x3FF0_A2B2 {
            // |x| < 1.5*ln2
            if (bits >> 63) == 0 {
                let hi = x - LN2_HI;
                let lo = LN2_LO;
                k = 1;
                r = hi - lo;
            } else {
                let hi = x + LN2_HI;
                let lo = -LN2_LO;
                k = -1;
                r = hi - lo;
            }
        } else {
            k = (INV_LN2 * x + (if (bits >> 63) != 0 { -0.5 } else { 0.5 })) as i32;
            let hi = x - (k as f64) * LN2_HI;
            let lo = (k as f64) * LN2_LO;
            r = hi - lo;
        }
    } else if hx < 0x3E30_0000 {
        // |x| < 2^-28
        return 1.0 + x;
    } else {
        k = 0;
        r = x;
    }

    // exp(r) via rational approximation
    let t = r * r;
    let c = r - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    let y = 1.0 - ((r * c) / (c - 2.0) - r);

    // Scale by 2^k
    if k == 0 {
        return y;
    }
    if k > 0 && k < 1024 {
        bits = ((0x3FF + k) as u64) << 52;
        y * f64::from_bits(bits)
    } else if k < 0 && k > -1023 {
        bits = ((0x3FF + k) as u64) << 52;
        y * f64::from_bits(bits)
    } else {
        // Extreme exponents
        bits = y.to_bits().wrapping_add((k as i64 as u64) << 52);
        f64::from_bits(bits)
    }
}

/// `double log(double)` — natural logarithm, fdlibm-style.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn log(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058500170e-10;
    const LG1: f64 = 6.666666666666735130e-01;
    const LG2: f64 = 3.999999999940941908e-01;
    const LG3: f64 = 2.857142874366239149e-01;
    const LG4: f64 = 2.222219843214978396e-01;
    const LG5: f64 = 1.818357216161805012e-01;
    const LG6: f64 = 1.531383769920937332e-01;
    const LG7: f64 = 1.479819860511658591e-01;
    const TWO54: f64 = f64::from_bits(0x4350_0000_0000_0000); // 0x1p54

    let mut xv = x;
    let mut bits = xv.to_bits();
    let mut hx = (bits >> 32) as i32;
    let lx = (bits & 0xFFFF_FFFF) as u32;

    let mut k: i32 = 0;

    // x < 2^-1022 (subnormal or negative)
    if hx < 0x0010_0000 {
        if (hx & 0x7FFF_FFFF) == 0 && lx == 0 {
            return -1.0 / 0.0; // log(0) = -inf
        }
        if hx < 0 {
            return (xv - xv) / 0.0; // log(negative) = NaN
        }
        // Subnormal: normalize
        k -= 54;
        xv *= TWO54;
        bits = xv.to_bits();
        hx = (bits >> 32) as i32;
    }

    // Inf or NaN
    if hx >= 0x7FF0_0000 {
        return xv + xv;
    }

    // log(1) = 0 exactly
    if hx == 0x3FF0_0000 && lx == 0 {
        return 0.0;
    }

    // Reduce x to [sqrt(2)/2, sqrt(2)]
    k += (hx >> 20) - 1023;
    hx &= 0x000F_FFFF;
    if hx < 0x6A09E {
        // < sqrt(2)/2 in high word
        bits = ((hx as u32 as u64) << 32) | (lx as u64);
        bits |= 0x3FF0_0000_0000_0000;
    } else {
        k += 1;
        bits = ((hx as u32 as u64) << 32) | (lx as u64);
        bits |= 0x3FE0_0000_0000_0000;
    }
    xv = f64::from_bits(bits);

    let f = xv - 1.0;

    // log(1+f) = f - f^2/2 + ... for small f (|f| < 2^-20)
    if (0x000F_FFFF & (2 + hx)) < 3 {
        if f == 0.0 {
            let dk = k as f64;
            return dk * LN2_HI + dk * LN2_LO;
        }
        let rr = f * f * (0.5 - 0.33333333333333333 * f);
        let dk = k as f64;
        return dk * LN2_HI - ((rr - dk * LN2_LO) - f);
    }

    let s = f / (2.0 + f);
    let dk = k as f64;
    let z = s * s;
    let w = z * z;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    let rr = t2 + t1;
    let hfsq = 0.5 * f * f;
    dk * LN2_HI - ((hfsq - (s * (hfsq + rr) + dk * LN2_LO)) - f)
}

/// `double log10(double)` — upstream's rough stub (FIXME), verbatim.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn log10(x: f64) -> f64 {
    if x <= 0.0 {
        return -1.0e308; // -infinity approximation
    }
    let mut result = 0.0f64;
    let mut temp = x;
    if temp >= 1.0 {
        while temp >= 10.0 {
            temp /= 10.0;
            result += 1.0;
        }
        result += (temp - 1.0) / 9.0;
    } else {
        while temp < 1.0 {
            temp *= 10.0;
            result -= 1.0;
        }
        result += (temp - 1.0) / 9.0;
    }
    result
}

/// `double pow(double, double)` — upstream's integer-exponent stub (FIXME),
/// verbatim. Non-integer / general case returns 0.0 exactly as the C does.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn pow(base: f64, exponent: f64) -> f64 {
    let mut result = 1.0f64;

    if exponent == 0.0 {
        return 1.0;
    }
    if base == 0.0 {
        return 0.0;
    }
    if base == 1.0 {
        return 1.0;
    }

    let iexp = exponent as i32; // truncate toward zero
    if (iexp as f64) == exponent && iexp >= 0 {
        for _ in 0..iexp {
            result *= base;
        }
        return result;
    }
    if (iexp as f64) == exponent && iexp < 0 {
        let iexp = -iexp;
        for _ in 0..iexp {
            result *= base;
        }
        return 1.0 / result;
    }

    0.0 // FIXME: general case not implemented (matches C)
}

/// `double fmod(double, double)` — fdlibm bit-twiddling remainder.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn fmod(x: f64, y: f64) -> f64 {
    let mut xi = x.to_bits();
    let mut yi = y.to_bits();

    // Sign of x
    let sx = xi & 0x8000_0000_0000_0000;

    // Clear signs
    xi &= 0x7FFF_FFFF_FFFF_FFFF;
    yi &= 0x7FFF_FFFF_FFFF_FFFF;

    // Special cases
    if yi == 0 || xi >= 0x7FF0_0000_0000_0000 || yi > 0x7FF0_0000_0000_0000 {
        return (x * y) / (x * y); // NaN
    }
    if xi < yi {
        return x;
    }

    // Extract exponents
    let mut ex = (xi >> 52) as i32;
    let mut ey = (yi >> 52) as i32;

    // Normalize subnormal x
    if ex == 0 {
        let mut i = xi << 12;
        ex -= 11;
        while (i >> 63) == 0 {
            i <<= 1;
            ex -= 1;
        }
        xi = xi.wrapping_shl((1 - ex) as u32);
        ex = 0;
    } else {
        xi &= 0x000F_FFFF_FFFF_FFFF;
        xi |= 0x0010_0000_0000_0000;
    }

    // Normalize subnormal y
    if ey == 0 {
        let mut i = yi << 12;
        ey -= 11;
        while (i >> 63) == 0 {
            i <<= 1;
            ey -= 1;
        }
        yi = yi.wrapping_shl((1 - ey) as u32);
        ey = 0;
    } else {
        yi &= 0x000F_FFFF_FFFF_FFFF;
        yi |= 0x0010_0000_0000_0000;
    }

    // Modular reduction
    while ex > ey {
        if xi >= yi {
            xi -= yi;
            if xi == 0 {
                return f64::from_bits(sx);
            }
        }
        xi <<= 1;
        ex -= 1;
    }

    if xi >= yi {
        xi -= yi;
        if xi == 0 {
            return f64::from_bits(sx);
        }
    }

    // Normalize result
    while (xi >> 52) == 0 {
        xi <<= 1;
        ex -= 1;
    }

    // Construct result
    if ex > 0 {
        xi -= 0x0010_0000_0000_0000;
        xi |= (ex as u64) << 52;
    } else {
        xi = xi.wrapping_shr((1 - ex) as u32);
    }

    xi |= sx;
    f64::from_bits(xi)
}
