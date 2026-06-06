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

// ═════════════════════════════════════════════════════════════════════════
// AMD-libm shared foundation (faithful ports of math/libm_sse2/*) — used by
// the amd64-only transcendentals exp2/sinh/cosh/tanh (splitexp + scaleDouble
// + the two_to_jby32 tables). All amd64/arm64 (float feature); i386 keeps C.
// `_handle_error(...,value,...)` reduces to `from_bits(value)` (its only
// value-effect; errno/FP-flags are kernel no-ops), and `val_with_flags(v,_)`
// to `v` — so those are inlined at the call sites, not modeled here.
// ═════════════════════════════════════════════════════════════════════════

// pow(2, j/32) for j=0..31, split into leading (first 25 bits) and trailing
// (further 53 bits) parts. Encoded by their exact stored bit patterns (the
// `.h` comments) so no decimal-rounding drift can creep in; the difftest vs
// the genuine decimal-literal C is the cross-check.
#[cfg(feature = "float")]
const TWO_TO_JBY32_LEAD: [f64; 32] = [
    f64::from_bits(0x3ff0000000000000), f64::from_bits(0x3ff059b0d0000000),
    f64::from_bits(0x3ff0b55860000000), f64::from_bits(0x3ff11301d0000000),
    f64::from_bits(0x3ff172b830000000), f64::from_bits(0x3ff1d48730000000),
    f64::from_bits(0x3ff2387a60000000), f64::from_bits(0x3ff29e9df0000000),
    f64::from_bits(0x3ff306fe00000000), f64::from_bits(0x3ff371a730000000),
    f64::from_bits(0x3ff3dea640000000), f64::from_bits(0x3ff44e0860000000),
    f64::from_bits(0x3ff4bfdad0000000), f64::from_bits(0x3ff5342b50000000),
    f64::from_bits(0x3ff5ab07d0000000), f64::from_bits(0x3ff6247eb0000000),
    f64::from_bits(0x3ff6a09e60000000), f64::from_bits(0x3ff71f75e0000000),
    f64::from_bits(0x3ff7a11470000000), f64::from_bits(0x3ff8258990000000),
    f64::from_bits(0x3ff8ace540000000), f64::from_bits(0x3ff93737b0000000),
    f64::from_bits(0x3ff9c49180000000), f64::from_bits(0x3ffa5503b0000000),
    f64::from_bits(0x3ffae89f90000000), f64::from_bits(0x3ffb7f76f0000000),
    f64::from_bits(0x3ffc199bd0000000), f64::from_bits(0x3ffcb720d0000000),
    f64::from_bits(0x3ffd5818d0000000), f64::from_bits(0x3ffdfc9730000000),
    f64::from_bits(0x3ffea4afa0000000), f64::from_bits(0x3fff507650000000),
];
#[cfg(feature = "float")]
const TWO_TO_JBY32_TRAIL: [f64; 32] = [
    f64::from_bits(0x0000000000000000), f64::from_bits(0x3e48ac2ba1d73e2a),
    f64::from_bits(0x3e69f3121ec53172), f64::from_bits(0x3df25b50a4ebbf1b),
    f64::from_bits(0x3e68faa2f5b9bef9), f64::from_bits(0x3e368b9aa7805b80),
    f64::from_bits(0x3e6ceac470cd83f6), f64::from_bits(0x3e547f7b84b09745),
    f64::from_bits(0x3e64636e2a5bd1ab), f64::from_bits(0x3e5ceaa72a9c5154),
    f64::from_bits(0x3e682468446b6824), f64::from_bits(0x3e18624b40c4dbd0),
    f64::from_bits(0x3e54d8a89c750e5e), f64::from_bits(0x3e5a753e077c2a0f),
    f64::from_bits(0x3e6a90a852b19260), f64::from_bits(0x3e0d2ac258f87d03),
    f64::from_bits(0x3e59fcef32422cbf), f64::from_bits(0x3e61d8bee7ba46e2),
    f64::from_bits(0x3e4f580c36bea881), f64::from_bits(0x3e62999c25159f11),
    f64::from_bits(0x3e415506dadd3e2a), f64::from_bits(0x3e29b8bc9e8a0388),
    f64::from_bits(0x3e451f8480e3e236), f64::from_bits(0x3e41f12ae45a1224),
    f64::from_bits(0x3e62b5a75abd0e6a), f64::from_bits(0x3e47daf237553d84),
    f64::from_bits(0x3e6b0aa538444196), f64::from_bits(0x3e69df20d22a0798),
    f64::from_bits(0x3e69f7490e4bb40b), f64::from_bits(0x3e4bdcdaf5cb4656),
    f64::from_bits(0x3e452486cc2c7b9d), f64::from_bits(0x3e66dc8a80ce9f09),
];

/// Faithful port of libm_inlines.h `splitexp`: find m, z1, z2 such that
/// base**x = 2**m * (z1 + z2). Returns (m, z1, z2).
#[cfg(feature = "float")]
fn splitexp(
    x: f64,
    logbase: f64,
    thirtytwo_by_logbaseof2: f64,
    logbaseof2_by_32_lead: f64,
    logbaseof2_by_32_trail: f64,
) -> (i32, f64, f64) {
    let r = x * thirtytwo_by_logbaseof2;
    let n = if r > 0.0 { (r + 0.5) as i32 } else { (r - 0.5) as i32 };
    let mut r1 = x - (n as f64) * logbaseof2_by_32_lead;
    let mut r2 = -(n as f64) * logbaseof2_by_32_trail;
    let j = (n & 0x1f) as usize;
    let f1 = TWO_TO_JBY32_LEAD[j];
    let f2 = TWO_TO_JBY32_TRAIL[j];
    let m = (n - j as i32) / 32;
    r1 *= logbase;
    r2 *= logbase;
    let r = r1 + r2;
    let q = r1
        + (r2
            + r * r
                * (5.00000000000000008883e-01
                    + r * (1.66666666665260878863e-01
                        + r * (4.16666666662260795726e-02
                            + r * (8.33336798434219616221e-03
                                + r * (1.38889490863777199667e-03))))));
    let z1 = f1;
    let z2 = f2 + ((f1 + f2) * q);
    (m, z1, z2)
}

/// libm_inlines.h `scaleDouble_1`: x * 2**n (EMIN < n < EMAX).
#[cfg(feature = "float")]
fn scale_double_1(x: f64, n: i32) -> f64 {
    let t = f64::from_bits(((n as i64 + 1023) << 52) as u64);
    x * t
}

/// libm_inlines.h `scaleDouble_2`: x * 2**n (2*EMIN <= n <= 2*EMAX).
#[cfg(feature = "float")]
fn scale_double_2(x: f64, n: i32) -> f64 {
    let n1 = n / 2;
    let n2 = n - n1;
    let t1 = f64::from_bits(((n1 as i64 + 1023) << 52) as u64);
    let t2 = f64::from_bits(((n2 as i64 + 1023) << 52) as u64);
    (x * t1) * t2
}

// ── Wave MATH-TIERC (sub-wave 8): exp2 ───────────────────────────────────
// Faithful port of math/libm_sse2/exp2.c (AMD libm). amd64-only in libcntpr;
// flipped on amd64+arm64 behind the float feature, i386 keeps C.

/// `double exp2(double)` — 2**x.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn exp2(x: f64) -> f64 {
    const MAX_EXP2_ARG: f64 = 1024.0;
    const MIN_EXP2_ARG: f64 = -1074.0;
    const LOG2: f64 = 6.931471805599453094178e-01;
    const LOG2_LEAD: f64 = 6.93147167563438415527e-01;
    const LOG2_TAIL: f64 = 1.29965068938898869640e-08;
    const ONE_BY_32_LEAD: f64 = 0.03125;

    let ux = x.to_bits();
    let ax = ux & 0x7FFF_FFFF_FFFF_FFFF;

    if ax >= 0x4090_0000_0000_0000 {
        // abs(x) >= 1024.0
        if ax >= 0x7ff0_0000_0000_0000 {
            // NaN or infinity
            if ux & 0x000f_ffff_ffff_ffff != 0 {
                return f64::from_bits(ux | 0x0008_0000_0000_0000); // NaN
            } else if ux & 0x8000_0000_0000_0000 != 0 {
                return 0.0; // -inf → +0
            }
            return x; // +inf
        }
        if x > MAX_EXP2_ARG {
            return f64::from_bits(0x7ff0_0000_0000_0000); // overflow → +inf
        } else if x < MIN_EXP2_ARG {
            return 0.0; // underflow → +0
        }
    }

    // Small arguments: abs(x) < 1/(16*log2)
    if ax < 0x3fb7154764ee6c2f {
        if ax < 0x3c00_0000_0000_0000 {
            // abs(x) < 2^-63
            return 1.0 + x;
        }
        // Split x into hx (head) and tx (tail).
        let hx = f64::from_bits(ux & 0xffff_ffff_f800_0000);
        let tx = x - hx;
        let y1 = x * LOG2_LEAD;
        let y2 = (((hx * LOG2_LEAD - y1) + hx * LOG2_TAIL) + tx * LOG2_LEAD) + tx * LOG2_TAIL;
        let y = y1 + y2;
        let z = (9.99564649780173690e-1
            + (1.61251249355268050e-5
                + (2.37986978239838493e-2 + 2.68724774856111190e-7 * y) * y)
                * y)
            / (9.99564649780173692e-1
                + (-4.99766199765151309e-1
                    + (1.070876894098586184e-1
                        + (-1.189773642681502232e-2 + 5.9480622371960190616e-4 * y) * y)
                        * y)
                    * y);
        return ((z * y1) + (z * y2)) + 1.0;
    }

    // Find m, z1, z2 such that exp2(x) = 2**m * (z1 + z2)
    let (m, z1, z2) = splitexp(x, LOG2, 32.0, ONE_BY_32_LEAD, 0.0);
    if m > -1022 && m < 1023 {
        scale_double_1(z1 + z2, m)
    } else {
        scale_double_2(z1 + z2, m)
    }
}

// ── Wave MATH-TIERC (sub-wave 8): tanh ───────────────────────────────────
// Faithful port of math/libm_sse2/tanh.c (AMD libm). amd64-only in libcntpr.
// Small |x| → x; |x| in (0,1] → [3,3] Remez approximations (two sub-ranges);
// larger → 1 - 2/(exp(2y)+1) via splitexp; |x| > 20 → ±1; NaN preserved.
// val_with_flags(x,_) reduces to x; _handle_error(...,bits,...) to
// from_bits(bits).

/// `double tanh(double)` — hyperbolic tangent.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn tanh(x: f64) -> f64 {
    const THIRTYTWO_BY_LOG2: f64 = 4.61662413084468283841e+01;
    const LOG2_BY_32_LEAD: f64 = 2.16608493356034159660e-02;
    const LOG2_BY_32_TAIL: f64 = 5.68948749532545630390e-11;
    const LARGE_THRESHOLD: f64 = 20.0;

    let ux = x.to_bits();
    let aux = ux & 0x7FFF_FFFF_FFFF_FFFF;
    if aux < 0x3e30_0000_0000_0000 {
        // |x| small enough that tanh(x) = x (with inexact for x != 0)
        return x;
    } else if aux > 0x7ff0_0000_0000_0000 {
        // |x| is NaN
        return f64::from_bits(ux | 0x0008_0000_0000_0000);
    }

    let xneg = aux != ux;
    let y = if xneg { -x } else { x };

    let mut z;
    if y > LARGE_THRESHOLD {
        // exp(-x) negligible → ±1
        z = 1.0;
    } else if y <= 1.0 {
        let y2 = y * y;
        if y < 0.9 {
            // [3,3] Remez approximation on [0, 0.9].
            z = y + y * y2
                * (-0.274030424656179760118928e0
                    + (-0.176016349003044679402273e-1
                        + (-0.200047621071909498730453e-3
                            - 0.142077926378834722618091e-7 * y2)
                            * y2)
                        * y2)
                / (0.822091273968539282568011e0
                    + (0.381641414288328849317962e0
                        + (0.201562166026937652780575e-1
                            + 0.2091140262529164482568557e-3 * y2)
                            * y2)
                        * y2);
        } else {
            // [3,3] Remez approximation on [0.9, 1].
            z = y + y * y2
                * (-0.227793870659088295252442e0
                    + (-0.146173047288731678404066e-1
                        + (-0.165597043903549960486816e-3
                            - 0.115475878996143396378318e-7 * y2)
                            * y2)
                        * y2)
                / (0.683381611977295894959554e0
                    + (0.317204558977294374244770e0
                        + (0.167358775461896562588695e-1
                            + 0.173076050126225961768710e-3 * y2)
                            * y2)
                        * y2);
        }
    } else {
        // p = exp(2*y) + 1; tanh = 1 - 2/p
        let (m, z1, z2) = splitexp(2.0 * y, 1.0, THIRTYTWO_BY_LOG2, LOG2_BY_32_LEAD, LOG2_BY_32_TAIL);
        let p = scale_double_2(z1 + z2, m) + 1.0;
        z = 1.0 - 2.0 / p;
    }

    if xneg {
        z = -z;
    }
    z
}

// ── Wave MATH-TIERC (sub-wave 9): sinh / cosh ────────────────────────────
// Faithful ports of math/libm_sse2/{sinh,cosh}.c (AMD libm). amd64-only in
// libcntpr. Both share four tabulated lead/tail tables of sinh(i)/cosh(i)
// for i=0..36 (encoded via from_bits of the exact stored bits — generated
// from the .c decimal literals, cross-checked by the difftest) and an
// identical degree-13 odd/even pair (sdy ≈ sinh(dy)-dy, cdy ≈ cosh(dy)-1).
// Large |x| → splitexp-based exp(|x|)/2 (sign applied for sinh; cosh even);
// small |x| → table reconstruction. NaN/Inf/overflow preserved exactly.

#[cfg(feature = "float")]
const SINH_LEAD: [f64; 37] = [
    f64::from_bits(0x0000000000000000), f64::from_bits(0x3ff2cd9fc0000000), f64::from_bits(0x400d03cf60000000),
    f64::from_bits(0x40240926e0000000), f64::from_bits(0x403b4a3800000000), f64::from_bits(0x40528d0160000000),
    f64::from_bits(0x406936d228000000), f64::from_bits(0x4081228768000000), f64::from_bits(0x409749ea50000000),
    f64::from_bits(0x40afa71570000000), f64::from_bits(0x40c5829dc8000000), f64::from_bits(0x40dd3c4488000000),
    f64::from_bits(0x40f3de1650000000), f64::from_bits(0x410b00b590000000), f64::from_bits(0x412259ac48000000),
    f64::from_bits(0x4138f0cca8000000), f64::from_bits(0x4150f2ebd0000000), f64::from_bits(0x4167093488000000),
    f64::from_bits(0x417f4f2208000000), f64::from_bits(0x419546d8f8000000), f64::from_bits(0x41aceb0888000000),
    f64::from_bits(0x41c3a6e1f8000000), f64::from_bits(0x41dab5adb8000000), f64::from_bits(0x41f226af30000000),
    f64::from_bits(0x4208ab7fb0000000), f64::from_bits(0x4220c3d390000000), f64::from_bits(0x4236c93268000000),
    f64::from_bits(0x424ef822f0000000), f64::from_bits(0x42650bba30000000), f64::from_bits(0x427c9aae40000000),
    f64::from_bits(0x4293704708000000), f64::from_bits(0x42aa6b7658000000), f64::from_bits(0x42c1f43fc8000000),
    f64::from_bits(0x42d866f348000000), f64::from_bits(0x42f0953e28000000), f64::from_bits(0x430689e220000000),
    f64::from_bits(0x431ea215a0000000),
];
#[cfg(feature = "float")]
const SINH_TAIL: [f64; 37] = [
    f64::from_bits(0x0000000000000000), f64::from_bits(0x3e513ae6096a0092), f64::from_bits(0x3e5db70cfb79a640),
    f64::from_bits(0x3e8c2526b66dc067), f64::from_bits(0x3e8b81b18647f380), f64::from_bits(0x3ebbc1cdd1e1eb08),
    f64::from_bits(0x3ecd9f201534fb09), f64::from_bits(0x3edd1c064a4e9954), f64::from_bits(0x3ed4eca65d06ea74),
    f64::from_bits(0x3f00c259bcc0ecc5), f64::from_bits(0x3f2b5a6647cf9016), f64::from_bits(0x3f09691adefb0870),
    f64::from_bits(0x3f53410fc29cde38), f64::from_bits(0x3f46a31a50b6fb3c), f64::from_bits(0x3f57defc71805c40),
    f64::from_bits(0x3f9eb49fd80e0bab), f64::from_bits(0x3f84fffc7bcd5920), f64::from_bits(0x3fc03a93b6c63435),
    f64::from_bits(0x3fb1940bb255fd1c), f64::from_bits(0x3fded26e14260b50), f64::from_bits(0x3ffb47401fc9f2a2),
    f64::from_bits(0x40267bb3f55634f1), f64::from_bits(0x401c435ff8194ddc), f64::from_bits(0x404d8fee052ba63a),
    f64::from_bits(0x40651d7edccde3f6), f64::from_bits(0x40704b1644557d1a), f64::from_bits(0x4076a6b5ca0a9dc4),
    f64::from_bits(0x40afd9cc72249aba), f64::from_bits(0x40ce58de693edab5), f64::from_bits(0x40d8c70158ac6363),
    f64::from_bits(0x40e7614764f43e20), f64::from_bits(0x4106337db36fc718), f64::from_bits(0x41212d98b1f611e2),
    f64::from_bits(0x412392bc108b37cc), f64::from_bits(0x415ce87bdc3473dc), f64::from_bits(0x414bc8d5ae99ad14),
    f64::from_bits(0x415d20d76744835c),
];
#[cfg(feature = "float")]
const COSH_LEAD: [f64; 37] = [
    f64::from_bits(0x3ff0000000000000), f64::from_bits(0x3ff8b07550000000), f64::from_bits(0x400e18fa08000000),
    f64::from_bits(0x402422a490000000), f64::from_bits(0x403b4ee858000000), f64::from_bits(0x40528d6fc8000000),
    f64::from_bits(0x406936e678000000), f64::from_bits(0x4081228948000000), f64::from_bits(0x409749eaa8000000),
    f64::from_bits(0x40afa71580000000), f64::from_bits(0x40c5829dd0000000), f64::from_bits(0x40dd3c4488000000),
    f64::from_bits(0x40f3de1650000000), f64::from_bits(0x410b00b590000000), f64::from_bits(0x412259ac48000000),
    f64::from_bits(0x4138f0cca8000000), f64::from_bits(0x4150f2ebd0000000), f64::from_bits(0x4167093488000000),
    f64::from_bits(0x417f4f2208000000), f64::from_bits(0x419546d8f8000000), f64::from_bits(0x41aceb0888000000),
    f64::from_bits(0x41c3a6e1f8000000), f64::from_bits(0x41dab5adb8000000), f64::from_bits(0x41f226af30000000),
    f64::from_bits(0x4208ab7fb0000000), f64::from_bits(0x4220c3d390000000), f64::from_bits(0x4236c93268000000),
    f64::from_bits(0x424ef822f0000000), f64::from_bits(0x42650bba30000000), f64::from_bits(0x427c9aae40000000),
    f64::from_bits(0x4293704708000000), f64::from_bits(0x42aa6b7658000000), f64::from_bits(0x42c1f43fc8000000),
    f64::from_bits(0x42d866f348000000), f64::from_bits(0x42f0953e28000000), f64::from_bits(0x430689e220000000),
    f64::from_bits(0x431ea215a0000000),
];
#[cfg(feature = "float")]
const COSH_TAIL: [f64; 37] = [
    f64::from_bits(0x0000000000000000), f64::from_bits(0x3e3d9f5504c2bd28), f64::from_bits(0x3e67cb66f0a4c9fd),
    f64::from_bits(0x3e8f58617928e588), f64::from_bits(0x3e6bc7d000c38d48), f64::from_bits(0x3eaf7f9d4e329998),
    f64::from_bits(0x3ec6e6e464885269), f64::from_bits(0x3ecba3a8b946c154), f64::from_bits(0x3ed3f4e76110d5a4),
    f64::from_bits(0x3f017622515a3e2b), f64::from_bits(0x3ee4dc4b528af3d0), f64::from_bits(0x3f11156278615e10),
    f64::from_bits(0x3f535ad50ed821f5), f64::from_bits(0x3f46b61055f2935c), f64::from_bits(0x3f57e2794a601240),
    f64::from_bits(0x3f9eb4b45f6aadd3), f64::from_bits(0x3f85000b967b3698), f64::from_bits(0x3fc03a940fadc092),
    f64::from_bits(0x3fb1940bf3bf874c), f64::from_bits(0x3fded26e1a2a2110), f64::from_bits(0x3ffb4740205796d6),
    f64::from_bits(0x40267bb3f55cb85d), f64::from_bits(0x401c435ff81e18ac), f64::from_bits(0x404d8fee052bdea4),
    f64::from_bits(0x40651d7edccde926), f64::from_bits(0x40704b1644557e0e), f64::from_bits(0x4076a6b5ca0a9e1c),
    f64::from_bits(0x40afd9cc72249abe), f64::from_bits(0x40ce58de693edab5), f64::from_bits(0x40d8c70158ac6364),
    f64::from_bits(0x40e7614764f43e20), f64::from_bits(0x4106337db36fc718), f64::from_bits(0x41212d98b1f611e2),
    f64::from_bits(0x412392bc108b37cc), f64::from_bits(0x415ce87bdc3473dc), f64::from_bits(0x414bc8d5ae99ad14),
    f64::from_bits(0x415d20d76744835c),
];

/// Shared degree-13 series used by sinh/cosh in the small-|x| table branch:
/// returns (sdy ≈ sinh(dy) - dy, cdy ≈ cosh(dy) - 1), identical in both .c.
#[cfg(feature = "float")]
fn sinh_cosh_dy_poly(dy: f64) -> (f64, f64) {
    let dy2 = dy * dy;
    let sdy = dy
        * dy2
        * (0.166666666666666667013899e0
            + (0.833333333333329931873097e-2
                + (0.198412698413242405162014e-3
                    + (0.275573191913636406057211e-5
                        + (0.250521176994133472333666e-7
                            + (0.160576793121939886190847e-9
                                + 0.7746188980094184251527126e-12 * dy2)
                                * dy2)
                            * dy2)
                        * dy2)
                    * dy2)
                * dy2);
    let cdy = dy2
        * (0.500000000000000005911074e0
            + (0.416666666666660876512776e-1
                + (0.138888888889814854814536e-2
                    + (0.248015872460622433115785e-4
                        + (0.275573350756016588011357e-6
                            + (0.208744349831471353536305e-8
                                + 0.1163921388172173692062032e-10 * dy2)
                                * dy2)
                            * dy2)
                        * dy2)
                    * dy2)
                * dy2);
    (sdy, cdy)
}

/// `double sinh(double)` — hyperbolic sine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn sinh(x: f64) -> f64 {
    const MAX_SINH_ARG: f64 = 7.10475860073943977113e+02;
    const THIRTYTWO_BY_LOG2: f64 = 4.61662413084468283841e+01;
    const LOG2_BY_32_LEAD: f64 = 2.16608493356034159660e-02;
    const LOG2_BY_32_TAIL: f64 = 5.68948749532545630390e-11;
    // 8 * BASEDIGITS_DP64(15) * log10(2)
    let small_threshold = 120.0 * 0.30102999566398119521373889;

    let ux = x.to_bits();
    let aux = ux & 0x7FFF_FFFF_FFFF_FFFF;
    if aux < 0x3e30_0000_0000_0000 {
        // |x| small enough that sinh(x) = x
        return x;
    } else if aux >= 0x7ff0_0000_0000_0000 {
        if aux > 0x7ff0_0000_0000_0000 {
            return f64::from_bits(ux | 0x0008_0000_0000_0000); // NaN
        }
        return x + x; // ±Inf
    }

    let xneg = aux != ux;
    let y = if xneg { -x } else { x };

    let z;
    if y >= MAX_SINH_ARG {
        // overflow → ±Inf
        return if xneg {
            f64::from_bits(0xfff0_0000_0000_0000)
        } else {
            f64::from_bits(0x7ff0_0000_0000_0000)
        };
    } else if y >= small_threshold {
        // sinh(y) ≈ exp(y)/2 via splitexp (÷2 by reducing m by 1)
        let (mut m, z1, z2) = splitexp(y, 1.0, THIRTYTWO_BY_LOG2, LOG2_BY_32_LEAD, LOG2_BY_32_TAIL);
        m -= 1;
        z = if m >= -1022 && m <= 1023 {
            scale_double_1(z1 + z2, m)
        } else {
            scale_double_2(z1 + z2, m)
        };
    } else {
        // z = sinh(y0)cosh(dy) + cosh(y0)sinh(dy) from the tables
        let ind = (y as i32) as usize;
        let dy = y - ind as f64;
        let (sdy, cdy) = sinh_cosh_dy_poly(dy);
        // Shift some significant bits from dy to sdy.
        let sdy1 = f64::from_bits(dy.to_bits() & 0xffff_ffff_f800_0000);
        let sdy2 = sdy + (dy - sdy1);
        z = ((((((COSH_TAIL[ind] * sdy2 + SINH_TAIL[ind] * cdy) + COSH_TAIL[ind] * sdy1)
            + SINH_TAIL[ind])
            + COSH_LEAD[ind] * sdy2)
            + SINH_LEAD[ind] * cdy)
            + COSH_LEAD[ind] * sdy1)
            + SINH_LEAD[ind];
    }

    if xneg {
        -z
    } else {
        z
    }
}

/// `double cosh(double)` — hyperbolic cosine (even).
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn cosh(x: f64) -> f64 {
    const MAX_COSH_ARG: f64 = 7.10475860073943977113e+02;
    const THIRTYTWO_BY_LOG2: f64 = 4.61662413084468283841e+01;
    const LOG2_BY_32_LEAD: f64 = 2.16608493356034159660e-02;
    const LOG2_BY_32_TAIL: f64 = 5.68948749532545630390e-11;
    const SMALL_THRESHOLD: f64 = 20.0;

    let ux = x.to_bits();
    let aux = ux & 0x7FFF_FFFF_FFFF_FFFF;
    if aux < 0x3e30_0000_0000_0000 {
        // |x| small enough that cosh(x) = 1
        return 1.0;
    } else if aux >= 0x7ff0_0000_0000_0000 {
        if aux > 0x7ff0_0000_0000_0000 {
            return f64::from_bits(ux | 0x0008_0000_0000_0000); // NaN
        }
        return f64::from_bits(0x7ff0_0000_0000_0000); // Inf → +Inf
    }

    let xneg = aux != ux;
    let y = if xneg { -x } else { x };

    if y >= MAX_COSH_ARG {
        return f64::from_bits(0x7ff0_0000_0000_0000); // overflow → +Inf
    } else if y >= SMALL_THRESHOLD {
        // cosh(y) ≈ exp(y)/2 via splitexp (÷2 by reducing m by 1)
        let (mut m, z1, z2) = splitexp(y, 1.0, THIRTYTWO_BY_LOG2, LOG2_BY_32_LEAD, LOG2_BY_32_TAIL);
        m -= 1;
        return if m >= -1022 && m <= 1023 {
            scale_double_1(z1 + z2, m)
        } else {
            scale_double_2(z1 + z2, m)
        };
    }
    // z = cosh(y0)cosh(dy) + sinh(y0)sinh(dy) from the tables (#else branch:
    // dy used directly, no sdy1/sdy2 split).
    let ind = (y as i32) as usize;
    let dy = y - ind as f64;
    let (sdy, cdy) = sinh_cosh_dy_poly(dy);
    ((((((COSH_TAIL[ind] * cdy + SINH_TAIL[ind] * sdy) + SINH_TAIL[ind] * dy) + COSH_TAIL[ind])
        + COSH_LEAD[ind] * cdy)
        + SINH_LEAD[ind] * sdy)
        + SINH_LEAD[ind] * dy)
        + COSH_LEAD[ind]
}

// ── Wave MATH-TIERC (sub-wave 10): acos / asin ───────────────────────────
// Faithful ports of math/libm_sse2/{acos,asin}.c (AMD libm). amd64-only in
// libcntpr. Both reduce to the same rational minimax R(r) ≈ (asin(x)-x)/x^3
// on [0,0.5] (factored below) and, for |x|>0.5, the sqrt-based half-angle
// identity. They use hardware sqrt (the genuine .c call `_mm_sqrt_sd`) — the
// in-module cnt-rust `sqrt` (sqrtsd/fsqrt) is correctly rounded, so bit-exact.

/// The shared [0,0.5] rational R(r) of acos/asin (`u` in the C sources).
#[cfg(feature = "float")]
fn asin_rational(r: f64) -> f64 {
    r * (0.227485835556935010735943483075
        + (-0.445017216867635649900123110649
            + (0.275558175256937652532686256258
                + (-0.0549989809235685841612020091328
                    + (0.00109242697235074662306043804220
                        + 0.0000482901920344786991880522822991 * r)
                        * r)
                    * r)
                * r)
            * r)
        / (1.36491501334161032038194214209
            + (-3.28431505720958658909889444194
                + (2.76568859157270989520376345954
                    + (-0.943639137032492685763471240072 + 0.105869422087204370341222318533 * r)
                        * r)
                    * r)
                * r)
}

/// `double asin(double)` — arcsine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn asin(x: f64) -> f64 {
    const PIBY2_TAIL: f64 = 6.1232339957367660e-17;
    const HPIBY2_HEAD: f64 = 7.8539816339744831e-01;
    const PIBY2: f64 = 1.5707963267948965e+00;

    let ux = x.to_bits();
    let aux = ux & 0x7FFF_FFFF_FFFF_FFFF;
    let xneg = ux & 0x8000_0000_0000_0000 != 0;
    let xnan = aux > 0x7ff0_0000_0000_0000;
    let xexp = ((ux & 0x7ff0_0000_0000_0000) >> 52) as i32 - 1023;

    if xnan {
        return f64::from_bits(ux | 0x0008_0000_0000_0000);
    } else if xexp < -28 {
        // arcsin(x) = x
        return x;
    } else if xexp >= 0 {
        // abs(x) >= 1.0
        if x == 1.0 {
            return PIBY2;
        } else if x == -1.0 {
            return -PIBY2;
        }
        return f64::from_bits(0xfff8_0000_0000_0000); // INDEFBITPATT_DP64
    }

    let mut y = if xneg { -x } else { x };
    let transform = xexp >= -1; // abs(x) >= 0.5
    let r;
    let mut s = 0.0;
    if transform {
        r = 0.5 * (1.0 - y);
        s = sqrt(r);
        y = s;
    } else {
        r = y * y;
    }

    let u = asin_rational(r);

    let v;
    if transform {
        let s1 = f64::from_bits(0xffff_ffff_0000_0000 & s.to_bits());
        let c = (r - s1 * s1) / (s + s1);
        let p = 2.0 * s * u - (PIBY2_TAIL - 2.0 * c);
        let q = HPIBY2_HEAD - 2.0 * s1;
        v = HPIBY2_HEAD - (p - q);
    } else {
        // temporary prevents reassociating y + y*u into y*(1+u)
        let tmp = y * u;
        v = y + tmp;
    }

    if xneg {
        -v
    } else {
        v
    }
}

/// `double acos(double)` — arccosine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn acos(x: f64) -> f64 {
    const PI: f64 = 3.1415926535897933e+00;
    const PIBY2: f64 = 1.5707963267948965580e+00;
    const PIBY2_HEAD: f64 = 1.5707963267948965580e+00;
    const PIBY2_TAIL: f64 = 6.12323399573676603587e-17;

    let ux = x.to_bits();
    let aux = ux & 0x7FFF_FFFF_FFFF_FFFF;
    let xneg = ux & 0x8000_0000_0000_0000 != 0;
    let xnan = aux > 0x7ff0_0000_0000_0000;
    let xexp = ((ux & 0x7ff0_0000_0000_0000) >> 52) as i32 - 1023;

    if xnan {
        return f64::from_bits(ux | 0x0008_0000_0000_0000);
    } else if xexp < -56 {
        // arccos(x) = pi/2
        return PIBY2;
    } else if xexp >= 0 {
        // abs(x) >= 1.0
        if x == 1.0 {
            return 0.0;
        } else if x == -1.0 {
            return PI;
        }
        return f64::from_bits(0xfff8_0000_0000_0000); // INDEFBITPATT_DP64
    }

    let mut y = if xneg { -x } else { x };
    let transform = xexp >= -1; // abs(x) >= 0.5
    let r;
    let mut s = 0.0;
    if transform {
        r = 0.5 * (1.0 - y);
        s = sqrt(r);
        y = s;
    } else {
        r = y * y;
    }

    let u = asin_rational(r);

    if transform {
        if xneg {
            return PI - 2.0 * (s + (y * u - PIBY2_TAIL));
        }
        let s1 = f64::from_bits(0xffff_ffff_0000_0000 & s.to_bits());
        let c = (r - s1 * s1) / (s + s1);
        return 2.0 * s1 + (2.0 * c + 2.0 * y * u);
    }
    PIBY2_HEAD - (x - (PIBY2_TAIL - x * u))
}

// ── Wave MATH-TIERC (sub-wave 11): _hypot ────────────────────────────────
// Faithful port of math/libm_sse2/hypot.c (AMD libm, the accurate non-FAST
// path). amd64-only in libcntpr; symbol is `_hypot` (the MS CRT name).
// sqrt(x*x + y*y) with exponent scaling to dodge spurious over/underflow.
// hypot.c defines FAST_BUT_GREATER_THAN_ONE_ULP, so r = x*x + y*y directly
// (the accurate head/tail split is the dead #else branch — not replicated).
// Uses scaleDouble_1 + the in-module hardware sqrt. NaN/Inf per C99 (Inf wins
// over NaN); overflow→+Inf.

/// `double _hypot(double x, double y)` — Euclidean distance, no spurious
/// overflow/underflow.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn _hypot(x: f64, y: f64) -> f64 {
    const LARGE: f64 = 1.79769313486231570815e+308;

    let mut ux = x.to_bits() & 0x7FFF_FFFF_FFFF_FFFF;
    let mut uy = y.to_bits() & 0x7FFF_FFFF_FFFF_FFFF;
    let xexp = ux >> 52;
    let yexp = uy >> 52;

    if xexp == 2047 || yexp == 2047 {
        // BIASEDEMAX_DP64 + 1: one/both args are NaN or Inf. Uses the
        // original (signed) x, y — abs is applied only below.
        let retval = x * x + y * y;
        if ((xexp == 2047) && (ux & 0x000f_ffff_ffff_ffff) == 0)
            || ((yexp == 2047) && (uy & 0x000f_ffff_ffff_ffff) == 0)
        {
            // x or y is Inf → +Inf even if the other is NaN (C99).
            return f64::from_bits(0x7ff0_0000_0000_0000);
        }
        // NaN (neither Inf).
        return retval;
    }

    // x = abs(x), y = abs(y)
    let mut x = f64::from_bits(ux);
    let mut y = f64::from_bits(uy);

    let dexp = (xexp as i64 - yexp as i64) as i32;
    let mut expadjust = 0i32;

    if ux == 0 {
        return y;
    } else if uy == 0 {
        return x;
    } else if dexp > 53 + 1 || dexp < -53 - 1 {
        // one insignificant c.f. the other
        return x + y;
    } else if xexp > 1023 + 500 || yexp > 1023 + 500 {
        // danger of overflow; scale down by 2**600
        expadjust = 600;
        ux -= 0x2580000000000000;
        x = f64::from_bits(ux);
        uy -= 0x2580000000000000;
        y = f64::from_bits(uy);
    } else if xexp < 1023 - 500 || yexp < 1023 - 500 {
        // danger of underflow; scale up by 2**600
        expadjust = -600;
        if xexp == 0 {
            x = f64::from_bits(ux + 0x2590000000000000);
            x -= 9.23297861778573578076e-128;
            ux = x.to_bits();
        } else {
            ux += 0x2580000000000000;
            x = f64::from_bits(ux);
        }
        if yexp == 0 {
            y = f64::from_bits(uy + 0x2590000000000000);
            y -= 9.23297861778573578076e-128;
            uy = y.to_bits();
        } else {
            uy += 0x2580000000000000;
            y = f64::from_bits(uy);
        }
    }

    // FAST_BUT_GREATER_THAN_ONE_ULP is defined in hypot.c, so r = x*x + y*y
    // directly (trading <1 ulp accuracy for speed; the accurate head/tail
    // path is the dead #else). ux/uy are unused past here in this path.
    let _ = (ux, uy);
    let r = x * x + y * y;

    let mut retval = sqrt(r);
    retval = scale_double_1(retval, expadjust);

    if retval > LARGE {
        // overflow → +Inf
        return f64::from_bits(0x7ff0_0000_0000_0000);
    }
    retval
}

// ── Wave MATH-TIERC (sub-wave 12): sincos + modff + hypotf ───────────────
// sincos is the combined sin/cos (void return → no i386 ABI issue, but it
// reuses the Rust sin/cos which only exist on amd64/arm64, so it stays
// float-feature-gated like them). modff/hypotf are the f32 twins of modf/
// hypot — amd64-only in libcntpr. modff is an exact bit-split; hypotf does
// the whole computation in f64 (float² always fits, so no scaling needed).

/// `void sincos(double x, double *s, double *c)` — sin and cos together.
/// Bit-exact: it *is* the Rust sin/cos, which are difftest-pinned to the C.
#[cfg(feature = "float")]
#[no_mangle]
pub unsafe extern "C" fn sincos(x: f64, s: *mut f64, c: *mut f64) {
    core::ptr::write_volatile(s, sin(x));
    core::ptr::write_volatile(c, cos(x));
}

/// `float modff(float x, float *iptr)` — f32 integer/fraction split (exact).
/// Faithful port of math/libm_sse2/modff.c.
#[cfg(feature = "float")]
#[no_mangle]
pub unsafe extern "C" fn modff(x: f32, iptr: *mut f32) -> f32 {
    let ux = x.to_bits();
    let xexp = ((ux & 0x7fff_ffff) >> 23) as i32 - 127;
    if xexp < 0 {
        // abs(x) < 1.0: integer part is ±0, fraction is x
        core::ptr::write_volatile(iptr, f32::from_bits(ux & 0x8000_0000));
        x
    } else if xexp < 23 {
        // 1.0 <= abs(x) < 2**24: mask off the fractional mantissa bits
        let mask = (1u32 << (23 - xexp)) - 1;
        let int_part = f32::from_bits(ux & !mask);
        core::ptr::write_volatile(iptr, int_part);
        x - int_part
    } else if (ux & 0x7fff_ffff) > 0x7f80_0000 {
        // x is NaN
        core::ptr::write_volatile(iptr, x);
        x + x
    } else {
        // x is infinity or large: integer part is x, fraction is ±0
        core::ptr::write_volatile(iptr, x);
        f32::from_bits(ux & 0x8000_0000)
    }
}

/// `float _hypotf(float x, float y)` — f32 Euclidean distance. Faithful port
/// of math/libm_sse2/hypotf.c: all math in f64 (float² fits, no scaling),
/// then round to f32. NaN/Inf per C99 (Inf wins); overflow → +Inf.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn _hypotf(x: f32, y: f32) -> f32 {
    // The largest finite float, as a double.
    const LARGE: f64 = 3.40282346638528859812e+38;

    let dx = x as f64;
    let dy = y as f64;
    let avx = dx.to_bits() & 0x7FFF_FFFF_FFFF_FFFF;
    let avy = dy.to_bits() & 0x7FFF_FFFF_FFFF_FFFF;
    let ux = avx >> 52;
    let uy = avy >> 52;

    if ux == 2047 || uy == 2047 {
        // BIASEDEMAX_DP64 + 1: one/both args are NaN or Inf.
        let retval = x * x + y * y; // f32 arithmetic
        if ((ux == 2047) && (avx & 0x000f_ffff_ffff_ffff) == 0)
            || ((uy == 2047) && (avy & 0x000f_ffff_ffff_ffff) == 0)
        {
            // x or y is Inf → +Inf even if the other is NaN (C99).
            return f32::from_bits(0x7f80_0000);
        }
        return retval; // NaN
    }

    let dr = dx * dx + dy * dy;
    let retval = sqrt(dr); // f64 sqrtsd

    if retval > LARGE {
        // overflow → +Inf
        return f32::from_bits(0x7f80_0000);
    }
    retval as f32
}

#[cfg(feature = "float")]
const TWO_TO_JBY32_LEAD_F32: [f32; 32] = [
    f32::from_bits(0x3F800000), f32::from_bits(0x3F82C000), f32::from_bits(0x3F858000), f32::from_bits(0x3F888000),
    f32::from_bits(0x3F8B8000), f32::from_bits(0x3F8E8000), f32::from_bits(0x3F91C000), f32::from_bits(0x3F94C000),
    f32::from_bits(0x3F980000), f32::from_bits(0x3F9B8000), f32::from_bits(0x3F9EC000), f32::from_bits(0x3FA24000),
    f32::from_bits(0x3FA5C000), f32::from_bits(0x3FA98000), f32::from_bits(0x3FAD4000), f32::from_bits(0x3FB10000),
    f32::from_bits(0x3FB50000), f32::from_bits(0x3FB8C000), f32::from_bits(0x3FBD0000), f32::from_bits(0x3FC10000),
    f32::from_bits(0x3FC54000), f32::from_bits(0x3FC98000), f32::from_bits(0x3FCE0000), f32::from_bits(0x3FD28000),
    f32::from_bits(0x3FD74000), f32::from_bits(0x3FDBC000), f32::from_bits(0x3FE0C000), f32::from_bits(0x3FE58000),
    f32::from_bits(0x3FEAC000), f32::from_bits(0x3FEFC000), f32::from_bits(0x3FF50000), f32::from_bits(0x3FFA8000),
];

#[cfg(feature = "float")]
const TWO_TO_JBY32_TRAIL_F32: [f32; 32] = [
    f32::from_bits(0x00000000), f32::from_bits(0x39D86988), f32::from_bits(0x3AAB0D9F), f32::from_bits(0x3A407404),
    f32::from_bits(0x3A2E0F1E), f32::from_bits(0x3A90E62D), f32::from_bits(0x38F4DCE0), f32::from_bits(0x3AD3BEA3),
    f32::from_bits(0x3ADFC146), f32::from_bits(0x39D39B9C), f32::from_bits(0x3AD4C982), f32::from_bits(0x3AC10C0C),
    f32::from_bits(0x3AFB5AA6), f32::from_bits(0x3A856AD3), f32::from_bits(0x3A41F752), f32::from_bits(0x3A8FD607),
    f32::from_bits(0x391E6678), f32::from_bits(0x3AEEBD1D), f32::from_bits(0x398A39F4), f32::from_bits(0x3AB13329),
    f32::from_bits(0x3A9CA845), f32::from_bits(0x3AE6F619), f32::from_bits(0x3A923054), f32::from_bits(0x3AA07647),
    f32::from_bits(0x391F9958), f32::from_bits(0x3AEEDE5F), f32::from_bits(0x39CDEEC0), f32::from_bits(0x3AE41B9D),
    f32::from_bits(0x37C6E7C0), f32::from_bits(0x3A92E66F), f32::from_bits(0x3A95F454), f32::from_bits(0x38ECB6D0),
];

#[cfg(feature = "float")]
const ATAN_JBY256: [f64; 241] = [
    f64::from_bits(0x3faff55bb72cfde9), f64::from_bits(0x3fb0f99ea71d52a6), f64::from_bits(0x3fb1f86dbf082d58),
    f64::from_bits(0x3fb2f719318a4a9a), f64::from_bits(0x3fb3f59f0e7c559d), f64::from_bits(0x3fb4f3fd677292fb),
    f64::from_bits(0x3fb5f2324fd2d7b2), f64::from_bits(0x3fb6f03bdcea4b0c), f64::from_bits(0x3fb7ee182602f10e),
    f64::from_bits(0x3fb8ebc54478fb28), f64::from_bits(0x3fb9e94153cfdcf1), f64::from_bits(0x3fbae68a71c722b8),
    f64::from_bits(0x3fbbe39ebe6f07c3), f64::from_bits(0x3fbce07c5c3cca32), f64::from_bits(0x3fbddd21701eba6e),
    f64::from_bits(0x3fbed98c2190043a), f64::from_bits(0x3fbfd5ba9aac2f6d), f64::from_bits(0x3fc068d584212b3d),
    f64::from_bits(0x3fc0e6adccf40881), f64::from_bits(0x3fc1646541060850), f64::from_bits(0x3fc1e1fafb043726),
    f64::from_bits(0x3fc25f6e171a535c), f64::from_bits(0x3fc2dcbdb2fba1ff), f64::from_bits(0x3fc359e8edeb99a3),
    f64::from_bits(0x3fc3d6eee8c6626c), f64::from_bits(0x3fc453cec6092a9e), f64::from_bits(0x3fc4d087a9da4f17),
    f64::from_bits(0x3fc54d18ba11570a), f64::from_bits(0x3fc5c9811e3ec269), f64::from_bits(0x3fc645bfffb3aa73),
    f64::from_bits(0x3fc6c1d4898933d8), f64::from_bits(0x3fc73dbde8a7d201), f64::from_bits(0x3fc7b97b4bce5b02),
    f64::from_bits(0x3fc8350be398ebc7), f64::from_bits(0x3fc8b06ee2879c28), f64::from_bits(0x3fc92ba37d050271),
    f64::from_bits(0x3fc9a6a8e96c8626), f64::from_bits(0x3fca217e601081a5), f64::from_bits(0x3fca9c231b403279),
    f64::from_bits(0x3fcb1696574d780b), f64::from_bits(0x3fcb90d7529260a2), f64::from_bits(0x3fcc0ae54d768466),
    f64::from_bits(0x3fcc84bf8a742e6d), f64::from_bits(0x3fccfe654e1d5395), f64::from_bits(0x3fcd77d5df205736),
    f64::from_bits(0x3fcdf110864c9d9d), f64::from_bits(0x3fce6a148e96ec4d), f64::from_bits(0x3fcee2e1451d980c),
    f64::from_bits(0x3fcf5b75f92c80dd), f64::from_bits(0x3fcfd3d1fc40dbe4), f64::from_bits(0x3fd025fa510665b5),
    f64::from_bits(0x3fd061eea03d6290), f64::from_bits(0x3fd09dc597d86362), f64::from_bits(0x3fd0d97ee509acb3),
    f64::from_bits(0x3fd1151a362431c9), f64::from_bits(0x3fd150973a9ce546), f64::from_bits(0x3fd18bf5a30bf178),
    f64::from_bits(0x3fd1c735212dd883), f64::from_bits(0x3fd2025567e47c95), f64::from_bits(0x3fd23d562b381041),
    f64::from_bits(0x3fd278372057ef45), f64::from_bits(0x3fd2b2f7fd9b5fe2), f64::from_bits(0x3fd2ed987a823cfe),
    f64::from_bits(0x3fd328184fb58951), f64::from_bits(0x3fd362773707ebcb), f64::from_bits(0x3fd39cb4eb76157b),
    f64::from_bits(0x3fd3d6d129271134), f64::from_bits(0x3fd410cbad6c7d32), f64::from_bits(0x3fd44aa436c2af09),
    f64::from_bits(0x3fd4845a84d0c21b), f64::from_bits(0x3fd4bdee586890e6), f64::from_bits(0x3fd4f75f73869978),
    f64::from_bits(0x3fd530ad9951cd49), f64::from_bits(0x3fd569d88e1b4cd7), f64::from_bits(0x3fd5a2e0175e0f4e),
    f64::from_bits(0x3fd5dbc3fbbe768d), f64::from_bits(0x3fd614840309cfe1), f64::from_bits(0x3fd64d1ff635c1c5),
    f64::from_bits(0x3fd685979f5fa6fd), f64::from_bits(0x3fd6bdeac9cbd76c), f64::from_bits(0x3fd6f61941e4def0),
    f64::from_bits(0x3fd72e22d53aa2a9), f64::from_bits(0x3fd7660752817501), f64::from_bits(0x3fd79dc6899118d1),
    f64::from_bits(0x3fd7d5604b63b3f7), f64::from_bits(0x3fd80cd46a14b1d0), f64::from_bits(0x3fd84422b8df95d7),
    f64::from_bits(0x3fd87b4b0c1ebedb), f64::from_bits(0x3fd8b24d394a1b25), f64::from_bits(0x3fd8e92916f5cde8),
    f64::from_bits(0x3fd91fde7cd0c662), f64::from_bits(0x3fd9566d43a34907), f64::from_bits(0x3fd98cd5454d6b18),
    f64::from_bits(0x3fd9c3165cc58107), f64::from_bits(0x3fd9f93066168001), f64::from_bits(0x3fda2f233e5e530b),
    f64::from_bits(0x3fda64eec3cc23fc), f64::from_bits(0x3fda9a92d59e98cf), f64::from_bits(0x3fdad00f5422058b),
    f64::from_bits(0x3fdb056420ae9343), f64::from_bits(0x3fdb3a911da65c6c), f64::from_bits(0x3fdb6f962e737efb),
    f64::from_bits(0x3fdba473378624a5), f64::from_bits(0x3fdbd9281e528191), f64::from_bits(0x3fdc0db4c94ec9ef),
    f64::from_bits(0x3fdc42191ff11eb6), f64::from_bits(0x3fdc76550aad71f8), f64::from_bits(0x3fdcaa6872f3631b),
    f64::from_bits(0x3fdcde53432c1350), f64::from_bits(0x3fdd121566b7f2ad), f64::from_bits(0x3fdd45aec9ec862b),
    f64::from_bits(0x3fdd791f5a1226f4), f64::from_bits(0x3fddac670561bb4f), f64::from_bits(0x3fdddf85bb026974),
    f64::from_bits(0x3fde127b6b0744af), f64::from_bits(0x3fde4548066cf51a), f64::from_bits(0x3fde77eb7f175a34),
    f64::from_bits(0x3fdeaa65c7cf28c4), f64::from_bits(0x3fdedcb6d43f8434), f64::from_bits(0x3fdf0ede98f393cf),
    f64::from_bits(0x3fdf40dd0b541417), f64::from_bits(0x3fdf72b221a4e495), f64::from_bits(0x3fdfa45dd3029258),
    f64::from_bits(0x3fdfd5e0175fdf83), f64::from_bits(0x3fe0039c73c1a40b), f64::from_bits(0x3fe01c341e82422d),
    f64::from_bits(0x3fe034b709250488), f64::from_bits(0x3fe04d25314342e5), f64::from_bits(0x3fe0657e94db30cf),
    f64::from_bits(0x3fe07dc3324e9b38), f64::from_bits(0x3fe095f30861a58f), f64::from_bits(0x3fe0ae0e1639866c),
    f64::from_bits(0x3fe0c6145b5b43da), f64::from_bits(0x3fe0de05d7aa6f7c), f64::from_bits(0x3fe0f5e28b67e295),
    f64::from_bits(0x3fe10daa77307a0d), f64::from_bits(0x3fe1255d9bfbd2a8), f64::from_bits(0x3fe13cfbfb1b056e),
    f64::from_bits(0x3fe1548596376469), f64::from_bits(0x3fe16bfa6f5137e1), f64::from_bits(0x3fe1835a88be7c13),
    f64::from_bits(0x3fe19aa5e5299f99), f64::from_bits(0x3fe1b1dc87904284), f64::from_bits(0x3fe1c8fe7341f64f),
    f64::from_bits(0x3fe1e00babdefeb3), f64::from_bits(0x3fe1f7043557138a), f64::from_bits(0x3fe20de813e823b1),
    f64::from_bits(0x3fe224b74c1d192a), f64::from_bits(0x3fe23b71e2cc9e6a), f64::from_bits(0x3fe25217dd17e501),
    f64::from_bits(0x3fe268a940696da6), f64::from_bits(0x3fe27f261273d1b3), f64::from_bits(0x3fe2958e59308e30),
    f64::from_bits(0x3fe2abe21aded073), f64::from_bits(0x3fe2c2215e024465), f64::from_bits(0x3fe2d84c2961e48b),
    f64::from_bits(0x3fe2ee628406cbca), f64::from_bits(0x3fe30464753b090a), f64::from_bits(0x3fe31a52048874be),
    f64::from_bits(0x3fe3302b39b78856), f64::from_bits(0x3fe345f01cce37bb), f64::from_bits(0x3fe35ba0b60eccce),
    f64::from_bits(0x3fe3713d0df6c503), f64::from_bits(0x3fe386c52d3db11e), f64::from_bits(0x3fe39c391cd41719),
    f64::from_bits(0x3fe3b198e5e2564a), f64::from_bits(0x3fe3c6e491c78dc4), f64::from_bits(0x3fe3dc1c2a188504),
    f64::from_bits(0x3fe3f13fb89e96f4), f64::from_bits(0x3fe4064f47569f48), f64::from_bits(0x3fe41b4ae06fea41),
    f64::from_bits(0x3fe430328e4b26d5), f64::from_bits(0x3fe445065b795b55), f64::from_bits(0x3fe459c652badc7f),
    f64::from_bits(0x3fe46e727efe4715), f64::from_bits(0x3fe4830aeb5f7bfd), f64::from_bits(0x3fe4978fa3269ee1),
    f64::from_bits(0x3fe4ac00b1c71762), f64::from_bits(0x3fe4c05e22de94e4), f64::from_bits(0x3fe4d4a8023414e8),
    f64::from_bits(0x3fe4e8de5bb6ec04), f64::from_bits(0x3fe4fd013b7dd17e), f64::from_bits(0x3fe51110adc5ed81),
    f64::from_bits(0x3fe5250cbef1e9fa), f64::from_bits(0x3fe538f57b89061e), f64::from_bits(0x3fe54ccaf0362c8f),
    f64::from_bits(0x3fe5608d29c70c34), f64::from_bits(0x3fe5743c352b33b9), f64::from_bits(0x3fe587d81f732fba),
    f64::from_bits(0x3fe59b60f5cfab9d), f64::from_bits(0x3fe5aed6c5909517), f64::from_bits(0x3fe5c2399c244260),
    f64::from_bits(0x3fe5d58987169b18), f64::from_bits(0x3fe5e8c6941043cf), f64::from_bits(0x3fe5fbf0d0d5cc49),
    f64::from_bits(0x3fe60f084b46e05e), f64::from_bits(0x3fe6220d115d7b8d), f64::from_bits(0x3fe634ff312d1f3b),
    f64::from_bits(0x3fe647deb8e20b8f), f64::from_bits(0x3fe65aabb6c07b02), f64::from_bits(0x3fe66d663923e086),
    f64::from_bits(0x3fe6800e4e7e2857), f64::from_bits(0x3fe692a40556fb6a), f64::from_bits(0x3fe6a5276c4b0575),
    f64::from_bits(0x3fe6b798920b3d98), f64::from_bits(0x3fe6c9f7855c3198), f64::from_bits(0x3fe6dc44551553ae),
    f64::from_bits(0x3fe6ee7f10204aef), f64::from_bits(0x3fe700a7c5784633), f64::from_bits(0x3fe712be84295198),
    f64::from_bits(0x3fe724c35b4fae7b), f64::from_bits(0x3fe736b65a172dff), f64::from_bits(0x3fe748978fba8e0f),
    f64::from_bits(0x3fe75a670b82d8d8), f64::from_bits(0x3fe76c24dcc6c6c0), f64::from_bits(0x3fe77dd112ea22c7),
    f64::from_bits(0x3fe78f6bbd5d315e), f64::from_bits(0x3fe7a0f4eb9c19a2), f64::from_bits(0x3fe7b26cad2e50fd),
    f64::from_bits(0x3fe7c3d311a6092b), f64::from_bits(0x3fe7d528289fa093), f64::from_bits(0x3fe7e66c01c114fd),
    f64::from_bits(0x3fe7f79eacb97898), f64::from_bits(0x3fe808c03940694a), f64::from_bits(0x3fe819d0b7158a4c),
    f64::from_bits(0x3fe82ad036000005), f64::from_bits(0x3fe83bbec5cdee22), f64::from_bits(0x3fe84c9c7653f7ea),
    f64::from_bits(0x3fe85d69576cc2c5), f64::from_bits(0x3fe86e2578f87ae5), f64::from_bits(0x3fe87ed0eadc5a2a),
    f64::from_bits(0x3fe88f6bbd023118), f64::from_bits(0x3fe89ff5ff57f1f7), f64::from_bits(0x3fe8b06fc1cf3dfe),
    f64::from_bits(0x3fe8c0d9145cf49d), f64::from_bits(0x3fe8d13206f8c4ca), f64::from_bits(0x3fe8e17aa99cc05d),
    f64::from_bits(0x3fe8f1b30c44f167), f64::from_bits(0x3fe901db3eeef187), f64::from_bits(0x3fe911f35199833b),
    f64::from_bits(0x3fe921fb54442d18),
];

// ════════════════════════════════════════════════════════════════════════
// Wave MATH-TIERC (sub-wave 13): the f32 transcendental twins
//   acosf asinf atanf sinhf coshf tanhf tanf atan2f
// Faithful ports of the AMD-libm math/libm_sse2/*f.c. amd64-only in libcntpr
// (the libm_sse2 set); flipped behind the `float` feature (amd64+arm64),
// i386 keeps C. The f64 foundation (splitexp, scale_double_1/2, SINH_LEAD/
// COSH_LEAD, sinh_cosh_dy_poly) is reused; the new f32 foundation is
// splitexpf + scale_float_2 + the two_to_jby32 f32 tables, plus the atan2f
// look-up table, the tanf Payne-Hanek reducer, and scale_down_double.
// _handle_errorf reduces to from_bits(value); valf_with_flags(v,_) → v.
// ════════════════════════════════════════════════════════════════════════

// Full-precision cosh(i)/sinh(i) tables (the f32 .c `cosh_lead`/`sinh_lead`,
// FULL values — distinct from sw9's lead/tail-split SINH_LEAD/COSH_LEAD).
#[cfg(feature = "float")]
const SINH_LEAD_F32: [f64; 37] = [
    f64::from_bits(0x0000000000000000), f64::from_bits(0x3ff2cd9fc44eb982), f64::from_bits(0x400d03cf63b6e19f),
    f64::from_bits(0x40240926e70949ad), f64::from_bits(0x403b4a3803703630), f64::from_bits(0x40528d0166f07374),
    f64::from_bits(0x406936d22f67c805), f64::from_bits(0x408122876ba380c9), f64::from_bits(0x409749ea514eca65),
    f64::from_bits(0x40afa7157430966f), f64::from_bits(0x40c5829dced69991), f64::from_bits(0x40dd3c4488cb48d6),
    f64::from_bits(0x40f3de1654d043f0), f64::from_bits(0x410b00b5916a31a5), f64::from_bits(0x412259ac48bef7e3),
    f64::from_bits(0x4138f0ccafad27f6), f64::from_bits(0x4150f2ebd0a7ffe3), f64::from_bits(0x416709348c0ea4ed),
    f64::from_bits(0x417f4f22091940bb), f64::from_bits(0x419546d8f9ed26e1), f64::from_bits(0x41aceb088b68e803),
    f64::from_bits(0x41c3a6e1fd9eecfd), f64::from_bits(0x41dab5adb9c435ff), f64::from_bits(0x41f226af33b1fdc0),
    f64::from_bits(0x4208ab7fb5475fb7), f64::from_bits(0x4220c3d3920962c8), f64::from_bits(0x4236c932696a6b5c),
    f64::from_bits(0x424ef822f7f6731c), f64::from_bits(0x42650bba3796379a), f64::from_bits(0x427c9aae4631c056),
    f64::from_bits(0x429370470aec28ec), f64::from_bits(0x42aa6b765d8cdf6c), f64::from_bits(0x42c1f43fcc4b662c),
    f64::from_bits(0x42d866f34a725782), f64::from_bits(0x42f0953e2f3a1ef7), f64::from_bits(0x430689e221bc8d5a),
    f64::from_bits(0x431ea215a1d20d76),
];

#[cfg(feature = "float")]
const COSH_LEAD_F32: [f64; 37] = [
    f64::from_bits(0x3ff0000000000000), f64::from_bits(0x3ff8b07551d9f550), f64::from_bits(0x400e18fa0df2d9bc),
    f64::from_bits(0x402422a497d6185e), f64::from_bits(0x403b4ee858de3e80), f64::from_bits(0x40528d6fcbeff3a9),
    f64::from_bits(0x406936e67db9b919), f64::from_bits(0x4081228949ba3a8b), f64::from_bits(0x409749eaa93f4e76),
    f64::from_bits(0x40afa715845d8894), f64::from_bits(0x40c5829dd053712d), f64::from_bits(0x40dd3c4489115627),
    f64::from_bits(0x40f3de1654d6b543), f64::from_bits(0x410b00b5916b6105), f64::from_bits(0x412259ac48bf13ca),
    f64::from_bits(0x4138f0ccafad2d17), f64::from_bits(0x4150f2ebd0a8005c), f64::from_bits(0x416709348c0ea503),
    f64::from_bits(0x417f4f22091940bf), f64::from_bits(0x419546d8f9ed26e1), f64::from_bits(0x41aceb088b68e804),
    f64::from_bits(0x41c3a6e1fd9eecfd), f64::from_bits(0x41dab5adb9c435ff), f64::from_bits(0x41f226af33b1fdc0),
    f64::from_bits(0x4208ab7fb5475fb7), f64::from_bits(0x4220c3d3920962c8), f64::from_bits(0x4236c932696a6b5c),
    f64::from_bits(0x424ef822f7f6731c), f64::from_bits(0x42650bba3796379a), f64::from_bits(0x427c9aae4631c056),
    f64::from_bits(0x429370470aec28ec), f64::from_bits(0x42aa6b765d8cdf6c), f64::from_bits(0x42c1f43fcc4b662c),
    f64::from_bits(0x42d866f34a725782), f64::from_bits(0x42f0953e2f3a1ef7), f64::from_bits(0x430689e221bc8d5a),
    f64::from_bits(0x431ea215a1d20d76),
];

/// libm_inlines.h `scaleFloat_2`: x * 2**n (2*EMIN <= n <= 2*EMAX).
#[cfg(feature = "float")]
fn scale_float_2(x: f32, n: i32) -> f32 {
    let n1 = n / 2;
    let n2 = n - n1;
    let t1 = f32::from_bits(((n1 + 127) << 23) as u32);
    let t2 = f32::from_bits(((n2 + 127) << 23) as u32);
    (x * t1) * t2
}

/// libm_inlines.h `splitexpf`: base**x = 2**m * (z1 + z2), single precision.
#[cfg(feature = "float")]
fn splitexpf(
    x: f32,
    logbase: f32,
    thirtytwo_by_logbaseof2: f32,
    logbaseof2_by_32_lead: f32,
    logbaseof2_by_32_trail: f32,
) -> (i32, f32, f32) {
    let r = x * thirtytwo_by_logbaseof2;
    let n = if r > 0.0 { (r + 0.5) as i32 } else { (r - 0.5) as i32 };
    let mut r1 = x - (n as f32) * logbaseof2_by_32_lead;
    let mut r2 = -(n as f32) * logbaseof2_by_32_trail;
    let j = (n & 0x1f) as usize;
    let f1 = TWO_TO_JBY32_LEAD_F32[j];
    let f2 = TWO_TO_JBY32_TRAIL_F32[j];
    let m = (n - j as i32) / 32;
    r1 *= logbase;
    r2 *= logbase;
    let r = r1 + r2;
    let q = r1 + (r2 + r * r * (5.00000000000000008883e-01_f32 + r * (1.66666666665260878863e-01_f32)));
    let z1 = f1;
    let z2 = f2 + ((f1 + f2) * q);
    (m, z1, z2)
}

/// libm_inlines.h `scaleDownDouble`: scale ux (bit pattern) down by 2**k,
/// returning the result bit pattern (handles denormal underflow + rounding).
#[cfg(feature = "float")]
fn scale_down_double(ux: u64, k: i32) -> u64 {
    let xsign = ux & 0x8000_0000_0000_0000;
    let ax = ux & 0x7fff_ffff_ffff_ffff;
    let n = ((ax & 0x7ff0_0000_0000_0000) >> 52) as i32 - k;
    let uy;
    if n > 0 {
        let uk = (n as u64) << 52;
        uy = (ax & !0x7ff0_0000_0000_0000) | uk;
    } else {
        let mut t = (ax & !0x7ff0_0000_0000_0000) | 0x0010_0000_0000_0000;
        let shift = 1 - n;
        if shift > 52 + 1 {
            t = 0;
        } else {
            t >>= shift - 1;
            t = (t >> 1) + (t & 1);
        }
        uy = t;
    }
    uy | xsign
}

/// The shared [0,0.5] float rational R(r) of acosf/asinf (`u` in the C).
#[cfg(feature = "float")]
fn asinf_rational(r: f32) -> f32 {
    r * (0.184161606965100694821398249421_f32
        + (-0.0565298683201845211985026327361_f32
            + (-0.0133819288943925804214011424456_f32
                - 0.00396137437848476485201154797087_f32 * r)
                * r)
            * r)
        / (1.10496961524520294485512696706_f32 - 0.836411276854206731913362287293_f32 * r)
}

/// `float asinf(float)` — arcsine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn asinf(x: f32) -> f32 {
    const PIBY2_TAIL: f32 = 7.5497894159e-08;
    const HPIBY2_HEAD: f32 = 7.8539812565e-01;
    const PIBY2: f32 = 1.5707963705e+00;

    let ux = x.to_bits();
    let aux = ux & !0x8000_0000;
    let xneg = ux & 0x8000_0000;
    let xnan = aux > 0x7f80_0000;
    let xexp = ((ux & 0x7f80_0000) >> 23) as i32 - 127;

    if xnan {
        return f32::from_bits(ux | 0x0040_0000);
    } else if xexp < -14 {
        return x;
    } else if xexp >= 0 {
        if x == 1.0 {
            return PIBY2;
        } else if x == -1.0 {
            return -PIBY2;
        }
        return f32::from_bits(0xffc0_0000); // INDEFBITPATT_SP32
    }

    let mut y = if xneg != 0 { -x } else { x };
    let transform = xexp >= -1;
    let r;
    let mut s = 0.0f32;
    if transform {
        r = 0.5 * (1.0 - y);
        s = sqrtf(r);
        y = s;
    } else {
        r = y * y;
    }

    let u = asinf_rational(r);

    let v;
    if transform {
        let s1 = f32::from_bits(0xffff_0000 & s.to_bits());
        let c = (r - s1 * s1) / (s + s1);
        let p = 2.0 * s * u - (PIBY2_TAIL - 2.0 * c);
        let q = HPIBY2_HEAD - 2.0 * s1;
        v = HPIBY2_HEAD - (p - q);
    } else {
        let tmp = y * u;
        v = y + tmp;
    }

    if xneg != 0 {
        -v
    } else {
        v
    }
}

/// `float acosf(float)` — arccosine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn acosf(x: f32) -> f32 {
    const PIBY2_F: f32 = 1.5707963705e+00;
    const PI: f64 = 3.1415926535897933e+00;
    const PIBY2_HEAD: f64 = 1.5707963267948965580e+00;
    const PIBY2_TAIL: f64 = 6.12323399573676603587e-17;

    let ux = x.to_bits();
    let aux = ux & !0x8000_0000;
    let xneg = ux & 0x8000_0000;
    let xnan = aux > 0x7f80_0000;
    let xexp = ((ux & 0x7f80_0000) >> 23) as i32 - 127;

    if xnan {
        return f32::from_bits(ux | 0x0040_0000);
    } else if xexp < -26 {
        return PIBY2_F;
    } else if xexp >= 0 {
        if x == 1.0 {
            return 0.0;
        } else if x == -1.0 {
            return PI as f32;
        }
        return f32::from_bits(0xffc0_0000);
    }

    let mut y = if xneg != 0 { -x } else { x };
    let transform = xexp >= -1;
    let r;
    let mut s = 0.0f32;
    if transform {
        r = 0.5 * (1.0 - y);
        s = sqrtf(r);
        y = s;
    } else {
        r = y * y;
    }

    let u = asinf_rational(r);

    if transform {
        if xneg != 0 {
            (PI - 2.0 * (s as f64 + ((y * u) as f64 - PIBY2_TAIL))) as f32
        } else {
            let s1 = f32::from_bits(0xffff_0000 & s.to_bits());
            let c = (r - s1 * s1) / (s + s1);
            2.0 * s1 + (2.0 * c + 2.0 * y * u)
        }
    } else {
        (PIBY2_HEAD - (x as f64 - (PIBY2_TAIL - (x * u) as f64))) as f32
    }
}

/// `float atanf(float)` — arctangent. Computed in f64, returned as f32.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn atanf(fx: f32) -> f32 {
    const PIBY2: f64 = 1.5707963267948966e+00;

    let x0 = fx as f64;
    let ux = x0.to_bits();
    let aux = ux & 0x7fff_ffff_ffff_ffff;
    let xneg = ux & 0x8000_0000_0000_0000;

    let v = if xneg != 0 { -x0 } else { x0 };

    let x;
    let c;
    if aux < 0x3fdc_0000_0000_0000 {
        x = v;
        c = 0.0;
    } else if aux < 0x3fe6_0000_0000_0000 {
        x = (2.0 * v - 1.0) / (2.0 + v);
        c = 4.63647609000806093515e-01;
    } else if aux < 0x3ff3_0000_0000_0000 {
        x = (v - 1.0) / (1.0 + v);
        c = 7.85398163397448278999e-01;
    } else if aux < 0x4003_8000_0000_0000 {
        x = (v - 1.5) / (1.0 + 1.5 * v);
        c = 9.82793723247329054082e-01;
    } else {
        let xnan = aux > 0x7ff0_0000_0000_0000;
        if xnan {
            let uhx = fx.to_bits();
            return f32::from_bits(uhx | 0x0040_0000);
        } else if v > 0x4c80_0000_0000_0000_u64 as f64 {
            if xneg != 0 {
                return (-PIBY2) as f32;
            } else {
                return PIBY2 as f32;
            }
        }
        x = -1.0 / v;
        c = 1.57079632679489655800e+00;
    }

    let s = x * x;
    let q = x * s
        * (0.296528598819239217902158651186e0
            + (0.192324546402108583211697690500e0 + 0.470677934286149214138357545549e-2 * s) * s)
        / (0.889585796862432286486651434570e0
            + (0.111072499995399550138837673349e1 + 0.299309699959659728404442796915e0 * s) * s);

    let mut z = c - (q - x);
    if xneg != 0 {
        z = -z;
    }
    z as f32
}

/// `float sinhf(float)` — hyperbolic sine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn sinhf(fx: f32) -> f32 {
    const MAX_SINH_ARG: f64 = 8.94159862922329438106e+01;
    const T32: f64 = 4.61662413084468283841e+01;
    const L2_LEAD: f64 = 2.16608493356034159660e-02;
    const L2_TAIL: f64 = 5.68948749532545630390e-11;
    let small_threshold = 8.0 * 15.0 * 0.30102999566398119521373889;

    let x = fx as f64;
    let ux = x.to_bits();
    let aux = ux & 0x7fff_ffff_ffff_ffff;
    if aux < 0x3f10_0000_0000_0000 {
        return fx; // |x| tiny: sinh(x) = x (with/without inexact)
    } else if aux >= 0x7ff0_0000_0000_0000 {
        if aux > 0x7ff0_0000_0000_0000 {
            let uhx = fx.to_bits();
            return f32::from_bits(uhx | 0x0040_0000);
        }
        return fx + fx; // ±Inf
    }

    let xneg = aux != ux;
    let y = if xneg { -x } else { x };

    let mut z;
    if y >= MAX_SINH_ARG {
        return if xneg {
            f32::from_bits(0xff80_0000)
        } else {
            f32::from_bits(0x7f80_0000)
        };
    } else if y >= small_threshold {
        let (mut m, z1, z2) = splitexp(y, 1.0, T32, L2_LEAD, L2_TAIL);
        m -= 1;
        z = scale_double_1(z1 + z2, m);
    } else {
        let ind = (y as i32) as usize;
        let dy = y - ind as f64;
        let (sdy_p, cdy_p) = sinh_cosh_dy_poly(dy);
        let sdy = dy + sdy_p;
        let cdy = 1.0 + cdy_p;
        z = SINH_LEAD_F32[ind] * cdy + COSH_LEAD_F32[ind] * sdy;
    }

    if xneg {
        z = -z;
    }
    z as f32
}

/// `float coshf(float)` — hyperbolic cosine.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn coshf(fx: f32) -> f32 {
    const MAX_COSH_ARG: f64 = 8.94159862922329438106e+01;
    const T32: f64 = 4.61662413084468283841e+01;
    const L2_LEAD: f64 = 2.16608493356034159660e-02;
    const L2_TAIL: f64 = 5.68948749532545630390e-11;
    const LAMBDA: f64 = 1.0e300; // LAMBDA_DP64 (inexact-raising sentinel)
    let small_threshold = 8.0 * 15.0 * 0.30102999566398119521373889;

    let x = fx as f64;
    let ux = x.to_bits();
    let aux = ux & 0x7fff_ffff_ffff_ffff;
    if aux < 0x3f10_0000_0000_0000 {
        if aux == 0 {
            return 1.0;
        }
        if LAMBDA + x > 1.0 {
            return 1.0;
        }
        // (unreachable for finite x; falls through faithfully otherwise)
    } else if aux >= 0x7ff0_0000_0000_0000 {
        if aux > 0x7ff0_0000_0000_0000 {
            let uhx = fx.to_bits();
            return f32::from_bits(uhx | 0x0040_0000);
        }
        return f32::from_bits(0x7f80_0000); // +Inf
    }

    let xneg = aux != ux;
    let y = if xneg { -x } else { x };

    let z;
    if y >= MAX_COSH_ARG {
        return f32::from_bits(0x7f80_0000); // overflow → +Inf
    } else if y >= small_threshold {
        let (mut m, z1, z2) = splitexp(y, 1.0, T32, L2_LEAD, L2_TAIL);
        m -= 1;
        z = scale_double_1(z1 + z2, m);
    } else {
        let ind = (y as i32) as usize;
        let dy = y - ind as f64;
        let (sdy_p, cdy_p) = sinh_cosh_dy_poly(dy);
        let sdy = dy + sdy_p;
        let cdy = 1.0 + cdy_p;
        z = COSH_LEAD_F32[ind] * cdy + SINH_LEAD_F32[ind] * sdy;
    }

    z as f32 // cosh is even (the C `if(xneg) z=-z` is commented out)
}

/// `float tanhf(float)` — hyperbolic tangent. All single precision.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn tanhf(x: f32) -> f32 {
    const T32: f32 = 4.6166240692e+01;
    const L2_LEAD: f32 = 2.1659851074e-02;
    const L2_TAIL: f32 = 9.9831822808e-07;
    const LARGE_THRESHOLD: f32 = 10.0;

    let ux = x.to_bits();
    let aux = ux & 0x7fff_ffff;
    if aux < 0x3900_0000 {
        return x; // |x| tiny: tanh(x) = x
    } else if aux > 0x7f80_0000 {
        return f32::from_bits(ux | 0x0040_0000); // NaN
    }

    let xneg = 1.0f32 - 2.0f32 * ((aux != ux) as i32 as f32);
    let y = xneg * x;

    let z;
    if y > LARGE_THRESHOLD {
        z = 1.0;
    } else if y <= 1.0 {
        let y2 = y * y;
        if y < 0.9 {
            z = y + y * y2
                * (-0.28192806108402678e0_f32
                    + (-0.14628356048797849e-2_f32 + 0.4891631088530669873e-4_f32 * y2) * y2)
                / (0.845784192581041099e0_f32 + 0.3427017942262751343e0_f32 * y2);
        } else {
            z = y + y * y2
                * (-0.24069858695196524e0_f32
                    + (-0.12325644183611929e-2_f32 + 0.3827534993599483396e-4_f32 * y2) * y2)
                / (0.72209738473684982e0_f32 + 0.292529068698052819e0_f32 * y2);
        }
    } else {
        let (m, z1, z2) = splitexpf(2.0 * y, 1.0, T32, L2_LEAD, L2_TAIL);
        let p = scale_float_2(z1 + z2, m) + 1.0;
        z = 1.0 - 2.0 / p;
    }

    xneg * z
}

/// tan(x) on [-pi/4,pi/4]; if `recip` return -1/tan(x). C `tanf_piby4`.
#[cfg(feature = "float")]
fn tanf_piby4(x: f64, recip: i32) -> f64 {
    let r = x * x;
    let t = x + x * r * (0.385296071263995406715129e0 - 0.172032480471481694693109e-1 * r)
        / (0.115588821434688393452299e+1
            + (-0.51396505478854532132342e0 + 0.1844239256901656082986661e-1 * r) * r);
    if recip != 0 {
        -1.0 / t
    } else {
        t
    }
}

/// libm_inlines.h `__remainder_piby2f_inline`: reduce |x| (bits in `ux`) to
/// [-pi/4,pi/4]; returns (r, region) where region = #pi/2 subtracted mod 4.
#[cfg(feature = "float")]
fn remainder_piby2f_inline(ux_in: u64) -> (f64, i32) {
    const BITSPER: i32 = 36;
    const PIBY2: f64 = 1.57079632679489655800e+00;
    const PIBITS: [u64; 13] = [
        0,
        5215,
        13000023176,
        11362338026,
        67174558139,
        34819822259,
        10612056195,
        67816420731,
        57840157550,
        19558516809,
        50025467026,
        25186875954,
        18152700886,
    ];

    let xexp = (((ux_in & 0x7ff0_0000_0000_0000) >> 52) as i64 - 1023) as i32;
    let ux = ((ux_in & 0x000f_ffff_ffff_ffff) | 0x0010_0000_0000_0000) >> 29;
    let mask: u64 = (1u64 << BITSPER) - 1;
    let first = xexp / BITSPER;
    let resexp = xexp - first * BITSPER;
    let last = first + 120 / BITSPER;

    let mut res = [0u64; 10];
    res[4] = 0;
    let mut u = PIBITS[last as usize] * ux;
    res[3] = u & mask;
    let mut carry = u >> BITSPER;
    u = PIBITS[(last - 1) as usize] * ux + carry;
    res[2] = u & mask;
    carry = u >> BITSPER;
    u = PIBITS[(last - 2) as usize] * ux + carry;
    res[1] = u & mask;
    carry = u >> BITSPER;
    u = PIBITS[first as usize] * ux + carry;
    res[0] = u & mask;

    let ltb = (((res[0] << BITSPER) | res[1]) >> (BITSPER - 1 - resexp)) & 7;
    let determ = ltb & 1;

    let mut i = 1usize;
    let region;
    let mut mant: u64;
    let nextbits: u64;
    if determ != 0 {
        region = (((ltb >> 1) + 1) & 3) as i32;
        mant = !res[1] & ((1u64 << (BITSPER - resexp)) - 1);
        while mant < 0x0000_0000_0001_0000 {
            i += 1;
            mant = (mant << BITSPER) | (!res[i] & mask);
        }
        nextbits = !res[i + 1] & mask;
    } else {
        region = (ltb >> 1) as i32;
        mant = res[1] & ((1u64 << (BITSPER - resexp)) - 1);
        while mant < 0x0000_0000_0001_0000 {
            i += 1;
            mant = (mant << BITSPER) | res[i];
        }
        nextbits = res[i + 1];
    }

    let mut bc = 0i32;
    while mant < 0x0000_4000_0000_0000 {
        bc += 6;
        mant <<= 6;
    }
    while mant < 0x0010_0000_0000_0000 {
        bc += 1;
        mant <<= 1;
    }
    mant |= nextbits >> (BITSPER - bc);

    let rexp = 52 + resexp - bc - (i as i32) * BITSPER;
    let mut uxr = ((rexp as i64 + 1023) as u64) << 52;
    uxr = (mant & 0x000f_ffff_ffff_ffff) | uxr;
    if determ != 0 {
        uxr |= 0x8000_0000_0000_0000;
    }
    let dx = f64::from_bits(uxr);
    (dx * PIBY2, region)
}

/// `float tanf(float)` — tangent. Reduction in f64, core via tanf_piby4.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn tanf(x: f32) -> f32 {
    let dx = x as f64;
    let ux = dx.to_bits();
    let ax = ux & 0x7fff_ffff_ffff_ffff;

    if ax <= 0x3fe9_21fb_5444_2d18 {
        if ax < 0x3f80_0000_0000_0000 {
            if ax < 0x3f20_0000_0000_0000 {
                return x;
            }
            return (dx + dx * dx * dx * 0.333333333333333333) as f32;
        }
        return tanf_piby4(dx, 0) as f32;
    } else if (ux & 0x7ff0_0000_0000_0000) == 0x7ff0_0000_0000_0000 {
        if ux & 0x000f_ffff_ffff_ffff != 0 {
            let ufx = x.to_bits();
            return f32::from_bits(ufx | 0x0040_0000); // NaN
        }
        return f32::from_bits(0xffc0_0000); // Inf → NaN (INDEFBITPATT_SP32)
    }

    let xneg = (ux >> 63) as i32;
    let dx = if xneg != 0 { -dx } else { dx };

    let r;
    let region;
    if dx < 5.0e5 {
        const TWOBYPI: f64 = 6.36619772367581382433e-01;
        const PIBY2_1: f64 = 1.57079632673412561417e+00;
        const PIBY2_1TAIL: f64 = 6.07710050650619224932e-11;
        const PIBY2_2: f64 = 6.07710050630396597660e-11;
        const PIBY2_2TAIL: f64 = 2.02226624879595063154e-21;
        const PIBY2_3: f64 = 2.02226624871116645580e-21;
        const PIBY2_3TAIL: f64 = 8.47842766036889956997e-32;
        let xexp = ax >> 52;
        let npi2: i32;
        if ax <= 0x400f_6a7a_2955_385e {
            if ax <= 0x4002_d97c_7f33_21d2 {
                npi2 = 1;
            } else {
                npi2 = 2;
            }
        } else if ax <= 0x401c_463a_becc_b2bb {
            if ax <= 0x4015_fdbb_e9bb_a775 {
                npi2 = 3;
            } else {
                npi2 = 4;
            }
        } else {
            npi2 = (dx * TWOBYPI + 0.5) as i32;
        }
        let mut rhead = dx - npi2 as f64 * PIBY2_1;
        let mut rtail = npi2 as f64 * PIBY2_1TAIL;
        let uy = rhead.to_bits();
        let expdiff = xexp.wrapping_sub((uy & 0x7ff0_0000_0000_0000) >> 52);
        if expdiff > 15 {
            let t = rhead;
            rtail = npi2 as f64 * PIBY2_2;
            rhead = t - rtail;
            rtail = npi2 as f64 * PIBY2_2TAIL - ((t - rhead) - rtail);
            if expdiff > 48 {
                let t = rhead;
                rtail = npi2 as f64 * PIBY2_3;
                rhead = t - rtail;
                rtail = npi2 as f64 * PIBY2_3TAIL - ((t - rhead) - rtail);
            }
        }
        r = rhead - rtail;
        region = npi2 & 3;
    } else {
        let (rr, reg) = remainder_piby2f_inline(ax);
        r = rr;
        region = reg;
    }

    if xneg != 0 {
        (-tanf_piby4(r, region & 1)) as f32
    } else {
        tanf_piby4(r, region & 1) as f32
    }
}

/// `float atan2f(float y, float x)` — full-quadrant arctangent. f64 core +
/// the atan(j/256) look-up table; returned as f32.
#[cfg(feature = "float")]
#[no_mangle]
pub extern "C" fn atan2f(fy: f32, fx: f32) -> f32 {
    const PI: f64 = 3.1415926535897932e+00;
    const PIBY2: f64 = 1.5707963267948966e+00;
    const PIBY4: f64 = 7.8539816339744831e-01;
    const THREE_PIBY4: f64 = 2.3561944901923449e+00;

    let x = fx as f64;
    let y = fy as f64;

    let ux = x.to_bits();
    let uy = y.to_bits();
    let aux = ux & 0x7fff_ffff_ffff_ffff;
    let auy = uy & 0x7fff_ffff_ffff_ffff;
    let xexp = ((ux & 0x7ff0_0000_0000_0000) >> 52) as i32;
    let yexp = ((uy & 0x7ff0_0000_0000_0000) >> 52) as i32;
    let xneg = ux & 0x8000_0000_0000_0000;
    let yneg = uy & 0x8000_0000_0000_0000;
    let xzero = aux == 0;
    let yzero = auy == 0;
    let xnan = aux > 0x7ff0_0000_0000_0000;
    let ynan = auy > 0x7ff0_0000_0000_0000;
    let xinf = aux == 0x7ff0_0000_0000_0000;
    let yinf = auy == 0x7ff0_0000_0000_0000;

    let diffexp = yexp - xexp;

    if xnan {
        let ufx = fx.to_bits();
        return f32::from_bits(ufx | 0x0040_0000);
    } else if ynan {
        let ufy = fy.to_bits();
        return f32::from_bits(ufy | 0x0040_0000);
    } else if yzero {
        if xneg != 0 {
            if yneg != 0 {
                return (-PI) as f32;
            } else {
                return PI as f32;
            }
        } else {
            return y as f32;
        }
    } else if xzero {
        if yneg != 0 {
            return (-PIBY2) as f32;
        }
        // BUG-COMPATIBLE: the C has no `return` for the y>0 case here; it
        // falls through to the diffexp/general path (which yields +pi/2).
    }

    if diffexp > 26 {
        if yneg != 0 {
            return (-PIBY2) as f32;
        } else {
            return PIBY2 as f32;
        }
    } else if diffexp < -13 && xneg == 0 {
        if diffexp < -150 {
            if yneg != 0 {
                return -0.0f32;
            } else {
                return 0.0f32;
            }
        } else if diffexp < -126 {
            let ys = scale_double_1(y, 100);
            let yq = ys / x;
            let uyq = scale_down_double(yq.to_bits(), 100);
            let yr = f64::from_bits(uyq);
            if (uyq & 0x7ff0_0000_0000_0000) == 0 {
                return yr as f32;
            } else {
                return yr as f32;
            }
        } else {
            return (y / x) as f32;
        }
    } else if diffexp < -26 && xneg != 0 {
        if yneg != 0 {
            return (-PI) as f32;
        } else {
            return PI as f32;
        }
    } else if yinf && xinf {
        if xneg != 0 {
            if yneg != 0 {
                return (-THREE_PIBY4) as f32;
            } else {
                return THREE_PIBY4 as f32;
            }
        } else if yneg != 0 {
            return (-PIBY4) as f32;
        } else {
            return PIBY4 as f32;
        }
    }

    let mut u = x;
    let mut v = y;
    if xneg != 0 {
        u = -x;
    }
    if yneg != 0 {
        v = -y;
    }

    let swap_vu = u < v;
    if swap_vu {
        let uu = u;
        u = v;
        v = uu;
    }
    let vbyu = v / u;

    let mut q;
    if vbyu > 0.0625 {
        let index = (256.0 * vbyu + 0.5) as i32;
        let r = (256.0 * v - index as f64 * u) / (256.0 * u + index as f64 * v);
        let s = r * r;
        q = ATAN_JBY256[(index - 16) as usize] + r - r * s * 0.33333333333224095522;
    } else if vbyu < 1.0e-4 {
        q = vbyu;
    } else {
        let s = vbyu * vbyu;
        q = vbyu
            - vbyu * s
                * (0.33333333333333170500
                    - s * (0.19999999999393223405 - s * 0.14285713561807169030));
    }

    if swap_vu {
        q = PIBY2 - q;
    }
    if xneg != 0 {
        q = PI - q;
    }
    if yneg != 0 {
        q = -q;
    }
    q as f32
}
