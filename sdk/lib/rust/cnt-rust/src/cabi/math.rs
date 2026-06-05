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
// ceil/floor/ceilf/floorf are defined below (sub-wave 4); round/roundf call
// those module-local impls directly.

/// `double round(double)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn round(arg: f64) -> f64 {
    if arg < 0.0 {
        ceil(arg - 0.5)
    } else {
        floor(arg + 0.5)
    }
}

/// `float roundf(float)`.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn roundf(arg: f32) -> f32 {
    if arg < 0.0 {
        ceilf(arg - 0.5)
    } else {
        floorf(arg + 0.5)
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

// ===========================================================================
// Wave MATH-TIERC sub-wave 3 — tan (f64)
//
// `tan(x) = sin(x)/cos(x)` — the portable `arm64/tan.c` form, applied
// uniformly to BOTH amd64 and arm64 per the standing directive ("give every
// arch the same thing — we don't care about hand-optimized"). This replaces
// amd64's hand-optimized AMD-libm `libm_sse2/tan.c` with the same simple
// quotient arm64 already used. Bit-exact: our Rust `sin`/`cos` are proven
// equal to the C `sin`/`cos` (sw1), so this quotient equals the C
// `arm64/tan.c` quotient bit-for-bit (incl. the cos(x)→0 ±inf limit, where
// both divide identical operands). amd64/arm64 only (the `float` feature);
// i386 keeps its x87 `tan_asm.s`.
// ===========================================================================

/// `double tan(double)` — sin/cos quotient (the portable arm64 form).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn tan(x: f64) -> f64 {
    let s = sin(x);
    let c = cos(x);
    s / c
}

// ===========================================================================
// Wave MATH-TIERC sub-wave 4 — ceil / floor / ceilf / floorf
//
// Exact IEEE directed rounding (toward +inf / -inf), pure bit-manipulation
// (musl-style) — no FPU rounding-mode dependence, no SSE4.1 `roundsd`. One
// uniform Rust impl for BOTH amd64 (was AMD-libm `libm_sse2/{ceil,floor}*.c`)
// and arm64 (was `arm64/{ceil,floor}.c` + `math/{ceilf,floorf}.c`), per the
// "give every arch the same thing" directive; i386 keeps its x87
// `{ceil,floor}_asm.s`/`.S`. These are EXACT operations (the result is the
// unique nearer integer in the rounding direction), so the impl is bit-exact
// against any correct reference regardless of rounding mode. The variable
// shift count `e` is in [0, 51]/[0, 22] here, and the carry `wrapping_add`
// mirrors C's wraparound (also keeps debug overflow-checks happy).
// ===========================================================================

/// `double floor(double)` — round toward -inf.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    let mut i = x.to_bits();
    let e = ((i >> 52) & 0x7ff) as i32 - 0x3ff;
    if e >= 52 {
        return x; // already integral, or inf/NaN
    }
    if e < 0 {
        // |x| < 1
        if i << 1 == 0 {
            return x; // ±0
        }
        return if i >> 63 != 0 { -1.0 } else { 0.0 };
    }
    let m = 0x000f_ffff_ffff_ffff_u64 >> e; // fraction-bit mask
    if i & m == 0 {
        return x; // no fraction → integral
    }
    if i >> 63 != 0 {
        i = i.wrapping_add(m); // negative: round toward -inf (increase magnitude)
    }
    i &= !m;
    f64::from_bits(i)
}

/// `double ceil(double)` — round toward +inf.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn ceil(x: f64) -> f64 {
    let mut i = x.to_bits();
    let e = ((i >> 52) & 0x7ff) as i32 - 0x3ff;
    if e >= 52 {
        return x;
    }
    if e < 0 {
        if i << 1 == 0 {
            return x; // ±0
        }
        return if i >> 63 != 0 { -0.0 } else { 1.0 };
    }
    let m = 0x000f_ffff_ffff_ffff_u64 >> e;
    if i & m == 0 {
        return x;
    }
    if i >> 63 == 0 {
        i = i.wrapping_add(m); // positive: round toward +inf
    }
    i &= !m;
    f64::from_bits(i)
}

/// `float floorf(float)` — round toward -inf.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn floorf(x: f32) -> f32 {
    let mut i = x.to_bits();
    let e = ((i >> 23) & 0xff) as i32 - 0x7f;
    if e >= 23 {
        return x;
    }
    if e < 0 {
        if i << 1 == 0 {
            return x;
        }
        return if i >> 31 != 0 { -1.0 } else { 0.0 };
    }
    let m = 0x007f_ffff_u32 >> e;
    if i & m == 0 {
        return x;
    }
    if i >> 31 != 0 {
        i = i.wrapping_add(m);
    }
    i &= !m;
    f32::from_bits(i)
}

/// `float ceilf(float)` — round toward +inf.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn ceilf(x: f32) -> f32 {
    let mut i = x.to_bits();
    let e = ((i >> 23) & 0xff) as i32 - 0x7f;
    if e >= 23 {
        return x;
    }
    if e < 0 {
        if i << 1 == 0 {
            return x;
        }
        return if i >> 31 != 0 { -0.0 } else { 1.0 };
    }
    let m = 0x007f_ffff_u32 >> e;
    if i & m == 0 {
        return x;
    }
    if i >> 31 == 0 {
        i = i.wrapping_add(m);
    }
    i &= !m;
    f32::from_bits(i)
}

// ===========================================================================
// Wave MATH-TIERC sub-wave 5 — ldexp / _logb / _logbf
//
// Faithful ports, amd64/arm64 only (the `float` feature; i386 keeps its C).
// ldexp(x,n) = x*2^n — the portable arm64/ldexp.c (identical stepping to
// scalbn), applied uniformly (replaces amd64's AMD-libm errno-setting ldexp)
// per the "give every arch the same thing" directive. _logb/_logbf extract
// the unbiased binary exponent as a float (MSVC underscore names — both
// arches' C export `_logb`/`_logbf`). All return f64/f32 by value, so i386
// (no-SSE, hard ABI error on float return) keeps the C.
// ===========================================================================

/// `double ldexp(double, int)` — x * 2^n (portable arm64 stepping).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn ldexp(x: f64, mut n: i32) -> f64 {
    if x == 0.0 || n == 0 {
        return x;
    }
    let mut xx = x;
    if n > 1023 {
        xx *= f64::from_bits(0x7FE0_0000_0000_0000); // 0x1p1023
        n -= 1023;
        if n > 1023 {
            xx *= f64::from_bits(0x7FE0_0000_0000_0000);
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        xx *= f64::from_bits(0x0010_0000_0000_0000); // 0x1p-1022
        n += 1022;
        if n < -1022 {
            xx *= f64::from_bits(0x0010_0000_0000_0000);
            n += 1022;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    let u = ((0x3FF + n) as u64) << 52;
    xx * f64::from_bits(u)
}

/// `double _logb(double)` — unbiased binary exponent (MSVC name).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn _logb(x: f64) -> f64 {
    let mut u = x.to_bits() & 0x7FFF_FFFF_FFFF_FFFF; // clear sign
    if u == 0 {
        return f64::NEG_INFINITY; // _logb(0) = -inf
    }
    let mut exp = (u >> 52) as i32;
    if exp == 0x7FF {
        return x * x; // +inf for inf, NaN for NaN
    }
    if exp == 0 {
        // subnormal: normalize by 2^64, re-read exponent
        u = (f64::from_bits(u) * f64::from_bits(0x43F0_0000_0000_0000)).to_bits(); // 0x1p64
        exp = (u >> 52) as i32 - 64;
    }
    (exp - 0x3FF) as f64
}

/// `float _logbf(float)` — unbiased binary exponent (MSVC name).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn _logbf(x: f32) -> f32 {
    let mut u = x.to_bits() & 0x7FFF_FFFF;
    if u == 0 {
        return f32::NEG_INFINITY;
    }
    let mut exp = (u >> 23) as i32;
    if exp == 0xFF {
        return x * x;
    }
    if exp == 0 {
        u = (f32::from_bits(u) * f32::from_bits(0x4C00_0000)).to_bits(); // 0x1p25f
        exp = (u >> 23) as i32 - 25;
    }
    (exp - 0x7F) as f32
}

// ── Wave MATH-TIERC (sub-wave 6): fabs / fabsf ───────────────────────────
// Absolute value via the sign bit. Bit-exact replacements for amd64's
// asm (amd64/fabs.S, amd64/fabsf.S) AND arm64's C (math/fabs.c, fabsf.c) —
// both arches agree, so one Rust impl each. NaN handling differs between
// the two functions and is preserved exactly:
//   fabs : a NaN input is returned UNCHANGED (sign bit NOT cleared) — both
//          the amd64 `cmova rax, rcx` and the arm64 fabs.c `return x` keep
//          the original bits. (The C's i386-only error-bit branch never
//          fires here; i386 keeps the C, this Rust is amd64/arm64-only.)
//   fabsf: a NaN input is returned with the quiet bit 0x00400000 OR'd in
//          and its sign preserved — both amd64 fabsf.S `bts ecx,22` and
//          arm64 fabsf.c unconditionally set it.

/// `double fabs(double)` — magnitude; NaN returned unchanged.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn fabs(x: f64) -> f64 {
    let bits = x.to_bits();
    let abs = bits & 0x7FFF_FFFF_FFFF_FFFF;
    if abs > 0x7FF0_0000_0000_0000 {
        // NaN: return the original bits unchanged.
        return x;
    }
    f64::from_bits(abs)
}

/// `float fabsf(float)` — magnitude; NaN returned with quiet bit set.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn fabsf(x: f32) -> f32 {
    let bits = x.to_bits();
    let abs = bits & 0x7FFF_FFFF;
    if abs > 0x7F80_0000 {
        // NaN: original | quiet bit (sign preserved).
        return f32::from_bits(bits | 0x0040_0000);
    }
    f32::from_bits(abs)
}

// ── Wave MATH-TIERC (sub-wave 7): modf ───────────────────────────────────
// Split x into integer (*iptr) and fraction (return) parts, each with the
// sign of x. amd64-only in libcntpr (arm64/i386 lack the f64 modf there);
// faithful port of math/libm_sse2/modf.c (AMD libm). The whole thing is
// exact bit-manipulation — no FP rounding — so the translation is bit-exact
// by construction. The *iptr write uses write_volatile (the captures(none)
// guard for #[no_mangle] output-pointer params).

/// `double modf(double x, double *iptr)` — integer/fraction split.
#[cfg(feature = "float")]
#[no_mangle]
pub unsafe extern "C" fn modf(x: f64, iptr: *mut f64) -> f64 {
    let ux = x.to_bits();
    let ax = ux & 0x7FFF_FFFF_FFFF_FFFF; // !SIGNBIT_DP64
    if ax >= 0x4340_0000_0000_0000 {
        // abs(x) is NaN, infinity, or >= 2^53
        if ax > 0x7FF0_0000_0000_0000 {
            // x is NaN
            core::ptr::write_volatile(iptr, x);
            return x + x; // raise invalid for a signalling NaN
        }
        // x is infinity or large: integer part is x, fraction is ±0
        core::ptr::write_volatile(iptr, x);
        return f64::from_bits(ux & 0x8000_0000_0000_0000); // SIGNBIT_DP64
    } else if ax < 0x3FF0_0000_0000_0000 {
        // abs(x) < 1.0: integer part is ±0, fraction is x
        core::ptr::write_volatile(iptr, f64::from_bits(ux & 0x8000_0000_0000_0000));
        return x;
    }
    // 1.0 <= abs(x) < 2^53: mask off the fractional mantissa bits
    let xexp = (((ux & 0x7FF0_0000_0000_0000) >> 52) as i64) - 1023;
    let mask: u64 = (1u64 << (52 - xexp)) - 1;
    let int_part = f64::from_bits(ux & !mask);
    core::ptr::write_volatile(iptr, int_part);
    x - int_part
}

// ── Wave MATH-TIERC (sub-wave 7): atan / atan2 ───────────────────────────
// Faithful ports of the portable fdlibm math/arm64/{atan,atan2}.c — range
// reduction + an 11-term odd polynomial. Used uniformly on amd64+arm64 (the
// "give every arch the same thing" directive), replacing amd64's AMD-libm
// versions; i386 keeps its x87 atan_asm.s. Both call the Rust `fabs` above;
// atan2 calls `atan`. Bit-exact vs the genuine portable C.

#[cfg(feature = "float")]
const ATANHI: [f64; 4] = [
    4.63647609000806093515e-01, // atan(0.5)hi
    7.85398163397448278999e-01, // atan(1.0)hi
    9.82793723247329054082e-01, // atan(1.5)hi
    1.57079632679489655800e+00, // atan(inf)hi
];
#[cfg(feature = "float")]
const ATANLO: [f64; 4] = [
    2.26987774529616870924e-17, // atan(0.5)lo
    3.06161699786838301793e-17, // atan(1.0)lo
    1.39033110312309984516e-17, // atan(1.5)lo
    6.12323399573676603587e-17, // atan(inf)lo
];
#[cfg(feature = "float")]
const ATAN_AT: [f64; 11] = [
    3.33333333333329318027e-01,
    -1.99999999998764832476e-01,
    1.42857142725034663711e-01,
    -1.11111104054623557880e-01,
    9.09088713343650656196e-02,
    -7.69187620504482999495e-02,
    6.66107313738753120669e-02,
    -5.83357013379057348645e-02,
    4.97687799461593236017e-02,
    -3.65315727442169155270e-02,
    1.62858201153657823623e-02,
];

/// `double atan(double)` — arctangent (portable fdlibm).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn atan(mut x: f64) -> f64 {
    let hx = (x.to_bits() >> 32) as u32 as i32;
    let ix = hx & 0x7FFF_FFFF;
    let id: i32;

    if ix >= 0x4410_0000 {
        // |x| >= 2^66
        let lx = (x.to_bits() & 0xFFFF_FFFF) as u32;
        if ix > 0x7FF0_0000 || (ix == 0x7FF0_0000 && lx != 0) {
            return x; // NaN
        }
        return if hx > 0 {
            ATANHI[3] + ATANLO[3]
        } else {
            -ATANHI[3] - ATANLO[3]
        };
    }

    if ix < 0x3FDC_0000 {
        // |x| < 0.4375
        if ix < 0x3E20_0000 {
            // |x| < 2^-29
            return x;
        }
        id = -1;
    } else {
        x = fabs(x);
        if ix < 0x3FF3_0000 {
            // |x| < 1.1875
            if ix < 0x3FE6_0000 {
                id = 0;
                x = (2.0 * x - 1.0) / (2.0 + x);
            } else {
                id = 1;
                x = (x - 1.0) / (x + 1.0);
            }
        } else if ix < 0x4003_8000 {
            // |x| < 2.4375
            id = 2;
            x = (x - 1.5) / (1.0 + 1.5 * x);
        } else {
            // 2.4375 <= |x| < 2^66
            id = 3;
            x = -1.0 / x;
        }
    }

    let z = x * x;
    let w = z * z;
    let s1 = z
        * (ATAN_AT[0]
            + w * (ATAN_AT[2] + w * (ATAN_AT[4] + w * (ATAN_AT[6] + w * (ATAN_AT[8] + w * ATAN_AT[10])))));
    let s2 = w
        * (ATAN_AT[1] + w * (ATAN_AT[3] + w * (ATAN_AT[5] + w * (ATAN_AT[7] + w * ATAN_AT[9]))));

    if id < 0 {
        return x - x * (s1 + s2);
    }
    let z = ATANHI[id as usize] - ((x * (s1 + s2) - ATANLO[id as usize]) - x);
    if hx < 0 {
        -z
    } else {
        z
    }
}

#[cfg(feature = "float")]
const ATAN2_PI: f64 = 3.14159265358979311600e+00;
#[cfg(feature = "float")]
const ATAN2_PI_LO: f64 = 1.22464679914735320717e-16;
#[cfg(feature = "float")]
const ATAN2_PI_2: f64 = 1.57079632679489655800e+00;
#[cfg(feature = "float")]
const ATAN2_PI_4: f64 = 7.85398163397448278999e-01;

/// `double atan2(double y, double x)` — arctangent of y/x (portable fdlibm).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn atan2(y: f64, x: f64) -> f64 {
    let xb = x.to_bits();
    let yb = y.to_bits();
    let hx = (xb >> 32) as u32 as i32;
    let lx = (xb & 0xFFFF_FFFF) as u32;
    let hy = (yb >> 32) as u32 as i32;
    let ly = (yb & 0xFFFF_FFFF) as u32;

    let ix = hx & 0x7FFF_FFFF;
    let iy = hy & 0x7FFF_FFFF;

    // x or y is NaN. (lx | -lx) >> 31 == (lx != 0), kept arithmetic-faithful.
    let lx_nz = (lx | (0u32.wrapping_sub(lx))) >> 31;
    let ly_nz = (ly | (0u32.wrapping_sub(ly))) >> 31;
    if ((ix as u32 | lx_nz) > 0x7FF0_0000) || ((iy as u32 | ly_nz) > 0x7FF0_0000) {
        return x + y;
    }

    // x == 1.0
    if hx == 0x3FF0_0000 && lx == 0 {
        return atan(y);
    }

    // m = 2 * sign(x) + sign(y)
    let m = ((hy >> 31) & 1) | ((hx >> 31) & 2);

    // y == 0
    if (iy as u32 | ly) == 0 {
        return match m {
            0 | 1 => y,           // atan(±0, +anything) = ±0
            2 => ATAN2_PI,        // atan(+0, -anything) = pi
            _ => -ATAN2_PI,       // atan(-0, -anything) = -pi
        };
    }

    // x == 0
    if (ix as u32 | lx) == 0 {
        return if hy < 0 { -ATAN2_PI_2 } else { ATAN2_PI_2 };
    }

    // x == inf
    if ix == 0x7FF0_0000 {
        if iy == 0x7FF0_0000 {
            return match m {
                0 => ATAN2_PI_4,
                1 => -ATAN2_PI_4,
                2 => 3.0 * ATAN2_PI_4,
                _ => -3.0 * ATAN2_PI_4,
            };
        }
        return match m {
            0 => 0.0,
            1 => -0.0,
            2 => ATAN2_PI,
            _ => -ATAN2_PI,
        };
    }

    // y == inf
    if iy == 0x7FF0_0000 {
        return if hy < 0 { -ATAN2_PI_2 } else { ATAN2_PI_2 };
    }

    // compute y/x
    let k = (iy - ix) >> 20;
    let z = if k > 60 {
        // |y/x| > 2^60
        ATAN2_PI_2 + 0.5 * ATAN2_PI_LO
    } else if hx < 0 && k < -60 {
        // 0 > |y|/x > -2^-60
        0.0
    } else {
        atan(fabs(y / x))
    };

    match m {
        0 => z,                            // atan(+, +)
        1 => -z,                           // atan(-, +)
        2 => ATAN2_PI - (z - ATAN2_PI_LO), // atan(+, -)
        _ => (z - ATAN2_PI_LO) - ATAN2_PI, // atan(-, -)
    }
}
