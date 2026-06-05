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
