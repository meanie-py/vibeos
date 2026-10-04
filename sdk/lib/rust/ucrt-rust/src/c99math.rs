//! The C99 `<math.h>` functions ucrtbase exported as `-stub` until this module:
//! `acosh`, `asinh`, `atanh`, `cbrt`, `erf`/`erfc`, `expm1`, `log1p`, `lgamma`,
//! `tgamma`, `logb`/`ilogb`, `rint`/`nearbyint`/`lround`/`llround`/`llrint`,
//! `trunc`, `remainder`/`remquo`, `fmin`/`fmax`/`fdim`, `copysign`, `nan`, each
//! with its `f` form -- and `fma`, `log2`, `lrint`, which were worse than stubs in
//! dll/win32/ucrtbase/stubs.c (`lrint` was `__debugbreak()`) -- plus `_fpclassf`, `_dpcomp`/`_fdpcomp` and `_Cbuild`/`creal`.
//! A stub printed "calling stub" and returned whatever was in the return register,
//! so every UCRT program using one got a wrong number with no error.
//!
//! The ALGORITHMS are the `libm` crate (musl's libm in Rust, MIT), not ours. Wine's
//! own `dlls/msvcrt` takes these same functions from its copy of musl, so this is the
//! code Wine runs, minus a translation of it to debug.
//!
//! What is ours is the UCRT CONTRACT on top, which `libm` does not have:
//! * `errno`. The C99 functions set `EDOM`/`ERANGE` but, unlike the C89 set, never
//!   call `_matherr` -- Wine's `test_math_errors` table expects `-1` as the exception
//!   type for every one of them. The conditions are Wine's (its musl copy is patched
//!   at exactly these points), except where a native-verified test says otherwise
//!   (`remainder`, below) or C99 Annex F requires a range error Wine omits (`lgamma`
//!   and `tgamma` overflow); each divergence is named where it happens.
//! * Microsoft's constants: `FP_ILOGBNAN` is `0x7fffffff`, where musl's is `INT_MIN`.
//! * `fmin`/`fmax` order signed zeros (C99 F.9.9.2), which `libm`'s IEEE 754-2008
//!   `minNum` does not, and `fdim` returns a NaN operand rather than computing one.

use core::ffi::{c_char, c_int, c_long};

#[cfg(not(target_arch = "aarch64"))]
use crate::fenv::setfp;
#[cfg(target_arch = "x86")]
use crate::fenv::setfp_sse;

const EDOM: c_int = 33;
const ERANGE: c_int = 34;

extern "C" {
    fn _errno() -> *mut c_int;
    // cnt-rust's, the MSVC `_logb` contract (an infinity keeps its sign).
    fn _logb(x: f64) -> f64;
    // cnt-rust's, which reports overflow and underflow to `_matherr`.
    fn ldexp(x: f64, n: c_int) -> f64;
}

#[inline(always)]
fn set_errno(code: c_int) {
    unsafe { *_errno() = code };
}

// ── the inverse hyperbolics ──────────────────────────────────────────────────

/// `acosh`: `x < 1` (including `-inf`) is a domain error.
#[no_mangle]
#[inline(never)]
pub extern "C" fn acosh(x: f64) -> f64 {
    if x < 1.0 {
        set_errno(EDOM);
    }
    libm::acosh(x)
}

/// `acoshl`: `long double` is `double` on Windows. Exported under its own name
/// because the spec gives it no alias.
#[no_mangle]
#[inline(never)]
pub extern "C" fn acoshl(x: f64) -> f64 {
    acosh(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn acoshf(x: f32) -> f32 {
    if x < 1.0 {
        set_errno(EDOM);
    }
    libm::acoshf(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn asinh(x: f64) -> f64 {
    libm::asinh(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn asinhf(x: f32) -> f32 {
    libm::asinhf(x)
}

/// `atanh`: `|x| > 1` is a domain error, `|x| == 1` a pole (range) error.
#[no_mangle]
#[inline(never)]
pub extern "C" fn atanh(x: f64) -> f64 {
    let a = libm::fabs(x);
    if a > 1.0 {
        set_errno(EDOM);
    } else if a == 1.0 {
        set_errno(ERANGE);
    }
    libm::atanh(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn atanhl(x: f64) -> f64 {
    atanh(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn atanhf(x: f32) -> f32 {
    let a = libm::fabsf(x);
    if a > 1.0 {
        set_errno(EDOM);
    } else if a == 1.0 {
        set_errno(ERANGE);
    }
    libm::atanhf(x)
}

// ── roots, error function, exponentials, logarithms ──────────────────────────

#[no_mangle]
#[inline(never)]
pub extern "C" fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn cbrtf(x: f32) -> f32 {
    libm::cbrtf(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn erf(x: f64) -> f64 {
    libm::erf(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn erff(x: f32) -> f32 {
    libm::erff(x)
}

/// `erfc`: underflows to zero, with `ERANGE`, from `x >= 28` (Wine's musl patch
/// sits on exactly that branch). `+inf` returns 0 exactly and is not an error.
#[no_mangle]
#[inline(never)]
pub extern "C" fn erfc(x: f64) -> f64 {
    if x >= 28.0 && x.is_finite() {
        set_errno(ERANGE);
    }
    libm::erfc(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn erfcf(x: f32) -> f32 {
    if x >= 28.0 && x.is_finite() {
        set_errno(ERANGE);
    }
    libm::erfcf(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn expm1(x: f64) -> f64 {
    libm::expm1(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn expm1f(x: f32) -> f32 {
    libm::expm1f(x)
}

/// `log1p`: `x < -1` is a domain error, `x == -1` a pole (range) error.
#[no_mangle]
#[inline(never)]
pub extern "C" fn log1p(x: f64) -> f64 {
    if x == -1.0 {
        set_errno(ERANGE);
    } else if x < -1.0 {
        set_errno(EDOM);
    }
    libm::log1p(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn log1pf(x: f32) -> f32 {
    if x == -1.0 {
        set_errno(ERANGE);
    } else if x < -1.0 {
        set_errno(EDOM);
    }
    libm::log1pf(x)
}

/// `log2`: `±0` is a pole (`ERANGE`), a negative argument a domain error (Wine's musl
/// patch points). Was `log(x) * M_LOG2E` in stubs.c, which is not exact even at powers
/// of two.
#[no_mangle]
#[inline(never)]
pub extern "C" fn log2(x: f64) -> f64 {
    if x == 0.0 {
        set_errno(ERANGE);
    } else if x < 0.0 {
        set_errno(EDOM);
    }
    musl_math::log2(x).0
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn log2f(x: f32) -> f32 {
    if x == 0.0 {
        set_errno(ERANGE);
    } else if x < 0.0 {
        set_errno(EDOM);
    }
    musl_math::log2f(x).0
}

/// `fma`: one rounding (stubs.c computed `x * y + z`, two). An invalid operation -- a NaN
/// result from NaN-free operands, i.e. `inf * 0` or `inf - inf` -- is `EDOM`.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fma(x: f64, y: f64, z: f64) -> f64 {
    let r = libm::fma(x, y, z);
    if r.is_nan() && !x.is_nan() && !y.is_nan() && !z.is_nan() {
        set_errno(EDOM);
    }
    r
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn fmaf(x: f32, y: f32, z: f32) -> f32 {
    let r = libm::fmaf(x, y, z);
    if r.is_nan() && !x.is_nan() && !y.is_nan() && !z.is_nan() {
        set_errno(EDOM);
    }
    r
}

// ── gamma ────────────────────────────────────────────────────────────────────

/// `lgamma`: a non-positive integer is a pole, `ERANGE`. Wine flags only the
/// NEGATIVE integers; C99 F.10.5.3 makes `lgamma(±0)` a pole too, and an infinite
/// result from a finite argument is an overflow range error.
#[no_mangle]
#[inline(never)]
pub extern "C" fn lgamma(x: f64) -> f64 {
    let r = libm::lgamma(x);
    if x.is_finite() && (x <= 0.0 && x == libm::floor(x) || r.is_infinite()) {
        set_errno(ERANGE);
    }
    r
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn lgammaf(x: f32) -> f32 {
    let r = libm::lgammaf(x);
    if x.is_finite() && (x <= 0.0 && x == libm::floorf(x) || r.is_infinite()) {
        set_errno(ERANGE);
    }
    r
}

/// `tgamma`: `-inf` and the negative integers are domain errors, `±0` a pole.
/// Wine raises `ERANGE` for overflow only from `|x| >= 184`; C99 F.10.5.4 wants it
/// wherever a finite argument overflows (`x > 171.62...`), and the underflow
/// below -184 keeps Wine's `ERANGE`.
#[no_mangle]
#[inline(never)]
pub extern "C" fn tgamma(x: f64) -> f64 {
    let r = libm::tgamma(x);
    if x == f64::NEG_INFINITY || x < 0.0 && x == libm::floor(x) {
        set_errno(EDOM);
    } else if x == 0.0 || x.is_finite() && (r.is_infinite() || x <= -184.0) {
        set_errno(ERANGE);
    }
    r
}

/// `tgammaf` is Wine's `tgamma((double)x)`, so its errno is the double one, plus
/// the overflow the narrowing adds (`x > 35.04`), which the double path cannot see.
#[no_mangle]
#[inline(never)]
pub extern "C" fn tgammaf(x: f32) -> f32 {
    let d = tgamma(x as f64);
    let r = d as f32;
    if d.is_finite() && r.is_infinite() {
        set_errno(ERANGE);
    }
    r
}

// ── exponent extraction ──────────────────────────────────────────────────────

const FP_ILOGB0: c_int = -0x7fff_ffff - 1;
const FP_ILOGBNAN: c_int = 0x7fff_ffff;

/// `ilogb`: Microsoft's `FP_ILOGBNAN` is `INT_MAX`; musl's (and `libm`'s) is `INT_MIN`.
#[no_mangle]
#[inline(never)]
pub extern "C" fn ilogb(x: f64) -> c_int {
    if x.is_nan() {
        return FP_ILOGBNAN;
    }
    if x == 0.0 {
        return FP_ILOGB0;
    }
    libm::ilogb(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn ilogbf(x: f32) -> c_int {
    if x.is_nan() {
        return FP_ILOGBNAN;
    }
    if x == 0.0 {
        return FP_ILOGB0;
    }
    libm::ilogbf(x)
}

/// `logb`: the `_logb` value contract, with `logb(±0)` a pole error. As a C99
/// function it sets `errno` and does not reach `_matherr`.
#[no_mangle]
#[inline(never)]
pub extern "C" fn logb(x: f64) -> f64 {
    if x == 0.0 {
        // Here, not through `_logb`, which reports its pole to `_matherr`.
        set_errno(ERANGE);
        return f64::NEG_INFINITY;
    }
    unsafe { _logb(x) }
}

/// `logbf`: Wine's musl `logbf`, which is `ilogbf` as a float, `x*x` for a non-finite
/// argument, and `-inf` with `ERANGE` for zero.
#[no_mangle]
#[inline(never)]
pub extern "C" fn logbf(x: f32) -> f32 {
    if !x.is_finite() {
        return x * x;
    }
    if x == 0.0 {
        set_errno(ERANGE);
        return -1.0 / (x * x);
    }
    libm::ilogbf(x) as f32
}

// ── rounding to integers ─────────────────────────────────────────────────────

// `rint` and `nearbyint` round in the CURRENT mode, which is the one place this module
// cannot simply take `libm`: Rust has no rounding modes, so `libm` documents that it
// ignores the FP environment, and on aarch64 its `rint` is `frintn` -- always to nearest,
// no inexact. Measured in the guest: `rint(1.2)` under `fesetround(FE_UPWARD)` returned 1.
// So arm64 uses the instructions C means: `frintx` (current mode, raises inexact) for
// `rint`, `frinti` (current mode, no flags) for `nearbyint`. On x86 `libm`'s generic
// path is the `x + 2^52 - 2^52` idiom, which rounds in MXCSR's mode and raises inexact
// exactly as C requires, and the call is opaque to the optimizer (the argument comes
// from outside).

#[cfg(target_arch = "aarch64")]
fn rint_env(mut x: f64) -> f64 {
    // `frintx` reads FPCR and writes FPSR: not `pure`/`nomem`-safe to elide.
    unsafe { core::arch::asm!("frintx {x:d}, {x:d}", x = inout(vreg) x, options(nostack)) };
    x
}
#[cfg(target_arch = "aarch64")]
fn rintf_env(mut x: f32) -> f32 {
    unsafe { core::arch::asm!("frintx {x:s}, {x:s}", x = inout(vreg) x, options(nostack)) };
    x
}
#[cfg(not(target_arch = "aarch64"))]
fn rint_env(x: f64) -> f64 {
    libm::rint(x)
}
#[cfg(not(target_arch = "aarch64"))]
fn rintf_env(x: f32) -> f32 {
    libm::rintf(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn rint(x: f64) -> f64 {
    rint_env(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn rintf(x: f32) -> f32 {
    rintf_env(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn truncf(x: f32) -> f32 {
    libm::truncf(x)
}

#[cfg(not(target_arch = "aarch64"))]
const _EM_INEXACT: u32 = 0x1;
#[cfg(not(target_arch = "aarch64"))]
const _SW_INEXACT: u32 = 0x1;

#[cfg(not(target_arch = "aarch64"))]
type SetFp = unsafe fn(*mut u32, u32, *mut u32, u32);

#[cfg(not(target_arch = "aarch64"))]
/// Wine's `nearbyint`: `rint` with the inexact exception masked for the call and the
/// inexact status flag restored afterwards, so neither a trap nor a sticky flag
/// escapes -- for ONE unit, x87 or SSE, through its `_setfp`.
fn without_inexact_in<T>(setfp: SetFp, f: impl FnOnce() -> T) -> T {
    let (mut cw, mut sw) = (0u32, 0u32);
    unsafe { setfp(&mut cw, 0, &mut sw, 0) };
    let update_cw = cw & _EM_INEXACT == 0;
    let update_sw = sw & _SW_INEXACT == 0;
    if update_cw {
        cw |= _EM_INEXACT;
        unsafe { setfp(&mut cw, _EM_INEXACT, core::ptr::null_mut(), 0) };
    }
    let r = f();
    if update_cw || update_sw {
        sw = 0;
        cw &= !_EM_INEXACT;
        let cwp = if update_cw { &mut cw as *mut u32 } else { core::ptr::null_mut() };
        let swp = if update_sw { &mut sw as *mut u32 } else { core::ptr::null_mut() };
        unsafe { setfp(cwp, _EM_INEXACT, swp, _SW_INEXACT) };
    }
    r
}

#[cfg(not(target_arch = "aarch64"))]
/// On i386 Wine's `_setfp` is the x87 alone, because Wine's i386 libm runs on the
/// x87. Ours is built SSE2 (see dll/win32/ucrtbase/CMakeLists.txt), so `rint` raises
/// its inexact in MXCSR, and `_statusfp` reports both units: guard both.
fn without_inexact<T>(f: impl FnOnce() -> T) -> T {
    #[cfg(target_arch = "x86")]
    {
        without_inexact_in(setfp_sse, || without_inexact_in(setfp, f))
    }
    #[cfg(not(target_arch = "x86"))]
    {
        without_inexact_in(setfp, f)
    }
}

#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
#[inline(never)]
pub extern "C" fn nearbyint(x: f64) -> f64 {
    without_inexact(|| rint_env(x))
}

#[cfg(not(target_arch = "aarch64"))]
#[no_mangle]
#[inline(never)]
pub extern "C" fn nearbyintf(x: f32) -> f32 {
    without_inexact(|| rintf_env(x))
}

/// arm64 `nearbyint`: `frinti` is exactly "round in the current mode, raise nothing".
#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[inline(never)]
pub extern "C" fn nearbyint(mut x: f64) -> f64 {
    unsafe { core::arch::asm!("frinti {x:d}, {x:d}", x = inout(vreg) x, options(nostack)) };
    x
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
#[inline(never)]
pub extern "C" fn nearbyintf(mut x: f32) -> f32 {
    unsafe { core::arch::asm!("frinti {x:s}, {x:s}", x = inout(vreg) x, options(nostack)) };
    x
}

// Wine's check is `d != (double)(long)d`. The C cast is undefined out of range and
// x86 returns the "integer indefinite" pattern, which never equals `d`, so the check
// means "outside [-2^31, 2^31)". It cannot be written as a Rust `as` round trip:
// `as` SATURATES, and `(2^31 as f32) as i32 as f32` comes back as 2^31 -- equal, so
// `lroundf(2147483648.f)` would return INT_MAX with no error. Explicit bounds, then.
// `d` is already integral, so `<` on the upper bound is exact.
const TWO_31: f64 = 2_147_483_648.0;
const TWO_63: f64 = 9_223_372_036_854_775_808.0;
const TWO_64: f64 = 18_446_744_073_709_551_616.0;

fn in_long(d: f64) -> bool {
    d >= -TWO_31 && d < TWO_31
}

fn in_llong(d: f64) -> bool {
    d >= -TWO_63 && d < TWO_63
}

/// `lround`: `long` is 32-bit on Windows; an unrepresentable result is `EDOM`, 0.
#[no_mangle]
#[inline(never)]
pub extern "C" fn lround(x: f64) -> c_long {
    let d = libm::round(x);
    if !in_long(d) {
        set_errno(EDOM);
        return 0;
    }
    d as c_long
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn lroundf(x: f32) -> c_long {
    let d = libm::roundf(x) as f64;
    if !in_long(d) {
        set_errno(EDOM);
        return 0;
    }
    d as c_long
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn llround(x: f64) -> i64 {
    let d = libm::round(x);
    if !in_llong(d) {
        set_errno(EDOM);
        return 0;
    }
    d as i64
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn llroundf(x: f32) -> i64 {
    let d = libm::roundf(x) as f64;
    if !in_llong(d) {
        set_errno(EDOM);
        return 0;
    }
    d as i64
}

/// `llrint`: Wine accepts a negative result down to `INT64_MIN` and a positive one up
/// to `UINT64_MAX`, then converts it to `__int64`, which on x86 yields the indefinite
/// `INT64_MIN` for anything past `INT64_MAX`. Reproduced; arm64's conversion
/// saturates, and so does ours there.
fn llrint_range(d: f64) -> bool {
    d >= -TWO_63 && d < TWO_64
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn llrint(x: f64) -> i64 {
    let d = rint_env(x);
    if !llrint_range(d) {
        set_errno(EDOM);
        return 0;
    }
    to_i64_like_c(d)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn llrintf(x: f32) -> i64 {
    let d = rintf_env(x) as f64;
    if !llrint_range(d) {
        set_errno(EDOM);
        return 0;
    }
    to_i64_like_c(d)
}

/// `lrint`: Wine's range is `[LONG_MIN, ULONG_MAX]`, and the conversion to `long` of a
/// value past `LONG_MAX` is x86's indefinite `LONG_MIN`, as `llrint`'s is.
fn lrint_range(d: f64) -> bool {
    d >= -TWO_31 && d < 2.0 * TWO_31
}

fn to_long_like_c(d: f64) -> c_long {
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) && d >= TWO_31 {
        c_long::MIN
    } else {
        d as c_long
    }
}

/// `lrint`. Was `__debugbreak()` in dll/win32/ucrtbase/stubs.c.
#[no_mangle]
#[inline(never)]
pub extern "C" fn lrint(x: f64) -> c_long {
    let d = rint_env(x);
    if !lrint_range(d) {
        set_errno(EDOM);
        return 0;
    }
    to_long_like_c(d)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn lrintf(x: f32) -> c_long {
    let d = rintf_env(x) as f64;
    if !lrint_range(d) {
        set_errno(EDOM);
        return 0;
    }
    to_long_like_c(d)
}

fn to_i64_like_c(d: f64) -> i64 {
    if cfg!(any(target_arch = "x86", target_arch = "x86_64")) && d >= TWO_63 {
        i64::MIN
    } else {
        d as i64
    }
}

// ── remainders ───────────────────────────────────────────────────────────────

/// The domain test for `remainder`/`remquo`. Wine's musl says `y == 0 || isinf(x)`,
/// but Wine's own msvcr120 `test_remainder` -- run against native, 64-bit -- also
/// expects `EDOM` whenever EITHER operand is a NaN. Native wins.
fn rem_domain(x: f64, y: f64) -> bool {
    y == 0.0 || x.is_infinite() || x.is_nan() || y.is_nan()
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn remainder(x: f64, y: f64) -> f64 {
    if rem_domain(x, y) {
        set_errno(EDOM);
    }
    libm::remainder(x, y)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn remainderf(x: f32, y: f32) -> f32 {
    if rem_domain(x as f64, y as f64) {
        set_errno(EDOM);
    }
    libm::remainderf(x, y)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn remquo(x: f64, y: f64, quo: *mut c_int) -> f64 {
    if rem_domain(x, y) {
        set_errno(EDOM);
    }
    let (r, q) = libm::remquo(x, y);
    *quo = q;
    r
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn remquof(x: f32, y: f32, quo: *mut c_int) -> f32 {
    if rem_domain(x as f64, y as f64) {
        set_errno(EDOM);
    }
    let (r, q) = libm::remquof(x, y);
    *quo = q;
    r
}

// ── min, max, positive difference, sign, NaN ─────────────────────────────────

#[no_mangle]
#[inline(never)]
pub extern "C" fn fmin(x: f64, y: f64) -> f64 {
    if x.is_nan() {
        return y;
    }
    if y.is_nan() {
        return x;
    }
    if x.is_sign_negative() != y.is_sign_negative() {
        return if x.is_sign_negative() { x } else { y };
    }
    if x < y { x } else { y }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn fminf(x: f32, y: f32) -> f32 {
    if x.is_nan() {
        return y;
    }
    if y.is_nan() {
        return x;
    }
    if x.is_sign_negative() != y.is_sign_negative() {
        return if x.is_sign_negative() { x } else { y };
    }
    if x < y { x } else { y }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn fmax(x: f64, y: f64) -> f64 {
    if x.is_nan() {
        return y;
    }
    if y.is_nan() {
        return x;
    }
    if x.is_sign_negative() != y.is_sign_negative() {
        return if x.is_sign_negative() { y } else { x };
    }
    if x < y { y } else { x }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    if x.is_nan() {
        return y;
    }
    if y.is_nan() {
        return x;
    }
    if x.is_sign_negative() != y.is_sign_negative() {
        return if x.is_sign_negative() { y } else { x };
    }
    if x < y { y } else { x }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn fdim(x: f64, y: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    if y.is_nan() {
        return y;
    }
    if x > y { x - y } else { 0.0 }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn fdimf(x: f32, y: f32) -> f32 {
    if x.is_nan() {
        return x;
    }
    if y.is_nan() {
        return y;
    }
    if x > y { x - y } else { 0.0 }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn copysign(x: f64, y: f64) -> f64 {
    libm::copysign(x, y)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn copysignf(x: f32, y: f32) -> f32 {
    libm::copysignf(x, y)
}

/// `nan`: Wine ignores the tag string and returns the quiet NaN; so do we.
#[no_mangle]
#[inline(never)]
pub extern "C" fn nan(_tag: *const c_char) -> f64 {
    f64::from_bits(0x7ff8_0000_0000_0000)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn nanf(_tag: *const c_char) -> f32 {
    f32::from_bits(0x7fc0_0000)
}

// ── the MSVC extras that sat in the same stub list ───────────────────────────

const _FPCLASS_SNAN: c_int = 0x0001;
const _FPCLASS_QNAN: c_int = 0x0002;
const _FPCLASS_NINF: c_int = 0x0004;
const _FPCLASS_NN: c_int = 0x0008;
const _FPCLASS_ND: c_int = 0x0010;
const _FPCLASS_NZ: c_int = 0x0020;
const _FPCLASS_PZ: c_int = 0x0040;
const _FPCLASS_PD: c_int = 0x0080;
const _FPCLASS_PN: c_int = 0x0100;
const _FPCLASS_PINF: c_int = 0x0200;

/// `_fpclassf` (not on i386, where the spec has no such export).
#[no_mangle]
#[inline(never)]
pub extern "C" fn _fpclassf(num: f32) -> c_int {
    let i = num.to_bits();
    let s = i >> 31 != 0;
    match i >> 23 & 0xff {
        0 => match (i << 1 != 0, s) {
            (true, true) => _FPCLASS_ND,
            (true, false) => _FPCLASS_PD,
            (false, true) => _FPCLASS_NZ,
            (false, false) => _FPCLASS_PZ,
        },
        0xff if i << 9 != 0 => {
            if (i >> 22) & 1 != 0 { _FPCLASS_QNAN } else { _FPCLASS_SNAN }
        }
        0xff => {
            if s { _FPCLASS_NINF } else { _FPCLASS_PINF }
        }
        _ => {
            if s { _FPCLASS_NN } else { _FPCLASS_PN }
        }
    }
}

/// `_dpcomp`: 0 unordered, 1 less, 2 equal, 4 greater (Wine's msvcr120 tests).
#[no_mangle]
#[inline(never)]
pub extern "C" fn _dpcomp(x: f64, y: f64) -> c_int {
    if x.is_nan() || y.is_nan() {
        return 0;
    }
    if x == y {
        2
    } else if x < y {
        1
    } else {
        4
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _fdpcomp(x: f32, y: f32) -> c_int {
    _dpcomp(x as f64, y as f64)
}

/// `_Dcomplex`: `{ double _Val[2]; }`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Dcomplex {
    pub val: [f64; 2],
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _Cbuild(re: f64, im: f64) -> Dcomplex {
    Dcomplex { val: [re, im] }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn creal(z: Dcomplex) -> f64 {
    z.val[0]
}

// ── the C89 functions' error contract ───────────────────────────────────────

/// cnt-rust's math exports (`exp`, `log`, `pow`, ... -- its `crt-math-errors` feature)
/// report a domain/pole/range error here. Wine's `math_error`: the user's `_matherr` sees
/// it first and may replace the result; if it declines, `_DOMAIN` sets EDOM and
/// `_SING`/`_OVERFLOW` ERANGE (`_UNDERFLOW` sets nothing).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __cnt_math_error(
    type_: c_int,
    name: *const c_char,
    arg1: f64,
    arg2: f64,
    retval: f64,
) -> f64 {
    // A report msvcrt makes and msvcr140 dropped (atan/tanh of a NaN).
    if type_ & 0x100 != 0 {
        return retval;
    }
    let mut e = crate::ucrt_reactos::Exception { type_, name: name as *mut c_char, arg1, arg2, retval };
    if crate::ucrt_reactos::__acrt_invoke_user_matherr(&mut e) != 0 {
        return e.retval;
    }
    match type_ {
        1 => set_errno(EDOM),
        2 | 3 => set_errno(ERANGE),
        _ => {}
    }
    e.retval
}

/// `_scalb`: Wine's, which is `ldexp` -- so, unlike `scalbn`, it reports overflow and
/// underflow to `_matherr` (Wine's `test_math_errors` checks `_scalb(1, 1e9)`).
#[no_mangle]
#[inline(never)]
pub extern "C" fn _scalb(x: f64, n: c_long) -> f64 {
    unsafe { ldexp(x, n) }
}

/// `_chgsign` -- crtmath float/chgsign.c: flips the sign bit (NaNs included).
#[no_mangle]
pub extern "C" fn _chgsign(x: f64) -> f64 {
    f64::from_bits(x.to_bits() ^ (1u64 << 63))
}

/// `_scalbf`: `_scalb` in `float`.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub extern "C" fn _scalbf(x: f32, n: c_long) -> f32 {
    unsafe { ldexp(x as f64, n) as f32 }
}
