//! Grades math.c — reference TU #10. 37 exports, and the file the "seal-calls is satisfied by
//! a PERMUTATION" lesson was learned on.
//!
//! WHAT THIS FILE CAN AND CANNOT GRADE. Almost every transcendental here is a THIN WRAPPER
//! whose body is `if (isnan(x)) return math_error(...); return <libm>(x);`. Both sides reach
//! the same host libm for the second line, so the arithmetic is trivially equal and grading it
//! proves nothing. What math.c actually owns is the FIRST line — the NaN test, the payload bit
//! it ORs in before reporting, which `_XXX` type code it reports, and whether errno is set.
//! That is exactly the surface a permutation damages, and exactly what these rows read.
//!
//! THREE INDEPENDENT VIEWS OF ONE CALL, because the first cannot see the class of bug this
//! file is known to carry:
//!
//!   1. the RETURN VALUE, compared BITWISE (never `==`: NaN != NaN, and -0.0 == 0.0, so a
//!      float comparison is blind to precisely the two results this file computes);
//!   2. ERRNO, read through the correct per-side accessor (see the trap below);
//!   3. the `struct _exception` the function REPORTS, captured by installing a recording
//!      handler through msvcrt's own `__setusermatherr` extension point.
//!
//! View 3 is the important one. `math_error(type, name, arg1, arg2, retval)` takes five
//! arguments of which three are `double`, so swapping two of them produces code that compiles,
//! links, returns the same value, and sets the same errno — `MSVCRT_exp` shipped
//! `math_error(0, …)` for `math_error(_DOMAIN, …)` once and nothing caught it. A handler that
//! records the whole struct sees the arguments themselves rather than their consequences.
//! Returning 0 from the handler leaves the `switch` and the errno store intact, so installing
//! it costs nothing: view 2 still works with view 3 armed.
//!
//! TWO ERRNO ACCESSORS, AND USING THE REFERENCE'S FOR BOTH IS A TRAP. `ref_errno_get` is
//! refimpl.c's, compiled under the refstub, so its `_errno` is `msvcrt_ref__errno` — the
//! REFERENCE's slot. The subject's is `subj_errno_get`. Reading the wrong one makes every row
//! report the subject as never setting errno, which looks exactly like the permutation bug
//! this file exists to find. It cost one false alarm before it was written down.
//!
//! THREAD DATA IS PER-SIDE, and that is what makes several of these rows possible.
//! `msvcrt_get_thread_data` is in refstub/msvcrt.h's anti-circularity namespace, so the
//! reference reaches refimpl.c's `msvcrt_ref_get_thread_data` and the subject reaches
//! subjectimpl.c's — two `thread_data_t`s. `_ecvt`/`_fcvt` cache their result buffer in one,
//! and `__fpecode` returns a pointer straight into it, so the two sides legitimately hand
//! back DIFFERENT addresses and the row asserts identity-per-side rather than equality.
//! String results are still copied out immediately after their own side's call: with separate
//! buffers that is no longer load-bearing, but it costs nothing and it is the habit that keeps
//! working if the storage is ever unified.
//!
//! MXCSR IS SHARED TOO, and worse, because it is hardware. On x86_64 `_setfp` is
//! `_setfp_sse`, so the whole `_control87`/`_controlfp`/`_statusfp`/`_clearfp` family
//! read-modify-writes ONE process register. Calling subject-then-reference would hand the
//! reference the state the subject just installed and grade a sequence, not a function. Each
//! side is given the same entry state by snapshot-and-restore.

#![allow(non_snake_case)]

extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_double, c_int, c_uint};

// ─────────────────────────────────────────────────────────────────────────────────────────
// The C surface
// ─────────────────────────────────────────────────────────────────────────────────────────

/// `struct _exception` — math.h. The struct `math_error` fills and hands to the user handler.
#[repr(C)]
#[derive(Clone, Copy)]
struct CException {
    r#type: c_int,
    name: *mut c_char,
    arg1: c_double,
    arg2: c_double,
    retval: c_double,
}

/// `struct _complex` — math.h, `_cabs`'s by-value argument.
#[repr(C)]
#[derive(Clone, Copy)]
struct CComplex {
    x: c_double,
    y: c_double,
}

type MatherrFunc = unsafe extern "C" fn(*mut CException) -> c_int;

#[rustfmt::skip]
extern "C" {
    // ── the matherr substrate ────────────────────────────────────────────────────────────
    fn __setusermatherr(func: MatherrFunc);
    fn ref___setusermatherr(func: MatherrFunc);
    fn _matherr(e: *mut CException) -> c_int;
    fn ref__matherr(e: *mut CException) -> c_int;

    // ── the matherr-bearing transcendentals ──────────────────────────────────────────────
    fn MSVCRT_atanf(x: f32) -> f32;
    fn ref_MSVCRT_atanf(x: f32) -> f32;
    fn MSVCRT_sqrtf(x: f32) -> f32;
    fn ref_MSVCRT_sqrtf(x: f32) -> f32;
    fn MSVCRT_tanhf(x: f32) -> f32;
    fn ref_MSVCRT_tanhf(x: f32) -> f32;
    fn MSVCRT_asin(x: f64) -> f64;
    fn ref_MSVCRT_asin(x: f64) -> f64;
    fn MSVCRT_atan(x: f64) -> f64;
    fn ref_MSVCRT_atan(x: f64) -> f64;
    fn MSVCRT_exp(x: f64) -> f64;
    fn ref_MSVCRT_exp(x: f64) -> f64;
    fn MSVCRT_sqrt(x: f64) -> f64;
    fn ref_MSVCRT_sqrt(x: f64) -> f64;
    fn MSVCRT_tanh(x: f64) -> f64;
    fn ref_MSVCRT_tanh(x: f64) -> f64;

    // ── classification and sign ──────────────────────────────────────────────────────────
    fn _fpclassf(num: f32) -> c_int;
    fn ref__fpclassf(num: f32) -> c_int;
    fn _chgsign(num: f64) -> f64;
    fn ref__chgsign(num: f64) -> f64;

    // ── integer / scaling ────────────────────────────────────────────────────────────────
    fn _abs64(n: i64) -> i64;
    fn ref__abs64(n: i64) -> i64;
    fn _scalb(num: f64, power: c_int) -> f64;
    fn ref__scalb(num: f64, power: c_int) -> f64;
    fn _scalbf(num: f32, power: c_int) -> f32;
    fn ref__scalbf(num: f32, power: c_int) -> f32;
    fn _cabs(num: CComplex) -> f64;
    fn ref__cabs(num: CComplex) -> f64;

    // ── the rotates. The export names are MSVCRT_-prefixed: msvcrt.spec maps `_rotl64` to
    //    `MSVCRT__rotl64`, and a row written against the spelled name does not fail, it
    //    fails to LINK.
    fn MSVCRT__rotl(num: c_uint, shift: c_int) -> c_uint;
    fn ref_MSVCRT__rotl(num: c_uint, shift: c_int) -> c_uint;
    fn MSVCRT__rotr(num: c_uint, shift: c_int) -> c_uint;
    fn ref_MSVCRT__rotr(num: c_uint, shift: c_int) -> c_uint;
    fn MSVCRT__lrotl(num: c_uint, shift: c_int) -> c_uint;
    fn ref_MSVCRT__lrotl(num: c_uint, shift: c_int) -> c_uint;
    fn MSVCRT__lrotr(num: c_uint, shift: c_int) -> c_uint;
    fn ref_MSVCRT__lrotr(num: c_uint, shift: c_int) -> c_uint;
    fn MSVCRT__rotl64(num: u64, shift: c_int) -> u64;
    fn ref_MSVCRT__rotl64(num: u64, shift: c_int) -> u64;
    fn MSVCRT__rotr64(num: u64, shift: c_int) -> u64;
    fn ref_MSVCRT__rotr64(num: u64, shift: c_int) -> u64;

    // ── the decimal conversions ──────────────────────────────────────────────────────────
    fn _ecvt(number: f64, ndigits: c_int, decpt: *mut c_int, sign: *mut c_int) -> *mut c_char;
    fn ref__ecvt(number: f64, ndigits: c_int, decpt: *mut c_int, sign: *mut c_int) -> *mut c_char;
    fn _fcvt(number: f64, ndigits: c_int, decpt: *mut c_int, sign: *mut c_int) -> *mut c_char;
    fn ref__fcvt(number: f64, ndigits: c_int, decpt: *mut c_int, sign: *mut c_int) -> *mut c_char;
    fn _gcvt(number: f64, ndigit: c_int, buff: *mut c_char) -> *mut c_char;
    fn ref__gcvt(number: f64, ndigit: c_int, buff: *mut c_char) -> *mut c_char;
    fn _ecvt_s(buf: *mut c_char, len: usize, number: f64, ndigits: c_int,
               decpt: *mut c_int, sign: *mut c_int) -> c_int;
    fn ref__ecvt_s(buf: *mut c_char, len: usize, number: f64, ndigits: c_int,
                   decpt: *mut c_int, sign: *mut c_int) -> c_int;
    fn _fcvt_s(buf: *mut c_char, size: usize, number: f64, ndigits: c_int,
               decpt: *mut c_int, sign: *mut c_int) -> c_int;
    fn ref__fcvt_s(buf: *mut c_char, size: usize, number: f64, ndigits: c_int,
                   decpt: *mut c_int, sign: *mut c_int) -> c_int;
    fn _gcvt_s(buff: *mut c_char, size: usize, number: f64, digits: c_int) -> c_int;
    fn ref__gcvt_s(buff: *mut c_char, size: usize, number: f64, digits: c_int) -> c_int;

    // ── the FPU control-word family ──────────────────────────────────────────────────────
    fn _control87(newval: c_uint, mask: c_uint) -> c_uint;
    fn ref__control87(newval: c_uint, mask: c_uint) -> c_uint;
    fn _controlfp(newval: c_uint, mask: c_uint) -> c_uint;
    fn ref__controlfp(newval: c_uint, mask: c_uint) -> c_uint;
    fn _set_controlfp(newval: c_uint, mask: c_uint);
    fn ref__set_controlfp(newval: c_uint, mask: c_uint);
    fn _controlfp_s(cur: *mut c_uint, newval: c_uint, mask: c_uint) -> c_int;
    fn ref__controlfp_s(cur: *mut c_uint, newval: c_uint, mask: c_uint) -> c_int;
    fn _statusfp() -> c_uint;
    fn ref__statusfp() -> c_uint;
    fn _clearfp() -> c_uint;
    fn ref__clearfp() -> c_uint;
    fn _fpreset();
    fn ref__fpreset();
    fn __fpecode() -> *mut c_int;
    fn ref___fpecode() -> *mut c_int;
    fn _set_SSE2_enable(flag: c_int) -> c_int;
    fn ref__set_SSE2_enable(flag: c_int) -> c_int;

    // ── harness accessors ────────────────────────────────────────────────────────────────
    // The harness's own bounded formatter (subjectimpl.c), used here ONLY in its
    // count-only form to predict how long a conversion would be before driving it.
    fn _snprintf(str_: *mut c_char, len: usize, format: *const c_char, ...) -> c_int;

    fn subj_errno_get() -> c_int;
    fn subj_errno_set(v: c_int);
    fn subj_invalid_pmt_count() -> c_int;
    fn subj_invalid_pmt_reset();
    fn ref_errno_get() -> c_int;
    fn ref_errno_set(v: c_int);
    fn ref_invalid_pmt_count() -> c_int;
    fn ref_invalid_pmt_reset();
}

// The `_XXX` math-error type codes and the errno values math_error stores for them, from
// sdk/include/crt/math.h and errno.h. Pinned as literals because a row that only compares the
// two sides cannot tell a shared misreading from a shared correct reading.
const T_DOMAIN: c_int = 1;
const E_DOM: c_int = 33;

// ─────────────────────────────────────────────────────────────────────────────────────────
// FP state discipline
// ─────────────────────────────────────────────────────────────────────────────────────────

#[allow(deprecated)]
fn mxcsr() -> u32 {
    unsafe { core::arch::x86_64::_mm_getcsr() }
}

#[allow(deprecated)]
fn set_mxcsr(v: u32) {
    unsafe { core::arch::x86_64::_mm_setcsr(v) }
}

/// Run `subject` then `reference` from the SAME MXCSR, and leave the register as it was found.
/// Returns both results plus the MXCSR each side left behind, so the register write is graded
/// as well as the return value — for `_control87` the register write IS the function.
fn paired_fp<T>(mut subject: impl FnMut() -> T, mut reference: impl FnMut() -> T) -> (T, u32, T, u32) {
    let entry = mxcsr();
    let s = subject();
    let s_csr = mxcsr();
    set_mxcsr(entry);
    let r = reference();
    let r_csr = mxcsr();
    set_mxcsr(entry);
    (s, s_csr, r, r_csr)
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// The recording matherr handlers
// ─────────────────────────────────────────────────────────────────────────────────────────

/// One `struct _exception` as reported, with the doubles kept as BITS. A permutation that
/// swaps `arg1` and `retval` on a NaN input produces two values that compare equal as floats
/// and differ as bit patterns.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Rec {
    ty: c_int,
    name: String,
    arg1: u64,
    arg2: u64,
    retval: u64,
}

mod rec {
    use super::Rec;
    pub static mut SUBJ: Vec<Rec> = Vec::new();
    pub static mut REF: Vec<Rec> = Vec::new();
    pub fn reset() {
        unsafe {
            (*core::ptr::addr_of_mut!(SUBJ)).clear();
            (*core::ptr::addr_of_mut!(REF)).clear();
        }
    }
    pub fn push(side: usize, r: Rec) {
        unsafe {
            let s = if side == 0 {
                core::ptr::addr_of_mut!(SUBJ)
            } else {
                core::ptr::addr_of_mut!(REF)
            };
            (*s).push(r);
        }
    }
    pub fn get(side: usize) -> Vec<Rec> {
        unsafe {
            let s = if side == 0 {
                core::ptr::addr_of!(SUBJ)
            } else {
                core::ptr::addr_of!(REF)
            };
            (*s).clone()
        }
    }
}

unsafe fn capture(side: usize, e: *mut CException) -> c_int {
    let e = &*e;
    let name = if e.name.is_null() {
        String::from("<null>")
    } else {
        let mut n = 0usize;
        while *e.name.add(n) != 0 {
            n += 1;
        }
        String::from_utf8_lossy(core::slice::from_raw_parts(e.name as *const u8, n)).into_owned()
    };
    rec::push(
        side,
        Rec { ty: e.r#type, name, arg1: e.arg1.to_bits(), arg2: e.arg2.to_bits(), retval: e.retval.to_bits() },
    );
    // 0, so math_error runs its `switch` and stores errno exactly as it would with no handler
    // installed. A non-zero return would short-circuit that and silently delete view 2.
    0
}

unsafe extern "C" fn matherr_subj(e: *mut CException) -> c_int {
    capture(0, e)
}
unsafe extern "C" fn matherr_ref(e: *mut CException) -> c_int {
    capture(1, e)
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// Inputs
// ─────────────────────────────────────────────────────────────────────────────────────────

/// Chosen for the wrapper, not the libm: every NaN encoding msvcrt distinguishes, both zeros,
/// both infinities, both subnormal extremes, and the ordinary values that must NOT take the
/// error path.
fn doubles() -> Vec<f64> {
    let mut v: Vec<f64> = [
        0x7ff8_0000_0000_0000u64, // +quiet NaN
        0xfff8_0000_0000_0000,    // -quiet NaN
        0x7ff0_0000_0000_0001,    // +signalling NaN, minimum payload
        0xfff0_0000_0000_0001,    // -signalling NaN
        0x7ff4_1234_5678_9abc,    // quiet NaN, non-trivial payload — the OR in tanh shows here
        0x7ff0_0000_0000_0000,    // +inf
        0xfff0_0000_0000_0000,    // -inf
        0x0000_0000_0000_0000,    // +0
        0x8000_0000_0000_0000,    // -0
        0x0000_0000_0000_0001,    // smallest subnormal
        0x800f_ffff_ffff_ffff,    // largest negative subnormal
        0x7fef_ffff_ffff_ffff,    // DBL_MAX
    ]
    .iter()
    .map(|b| f64::from_bits(*b))
    .collect();
    v.extend_from_slice(&[
        1.0, -1.0, 0.5, -0.5, 2.0, -2.0, 0.25, 1e-300, -1e-300, 1e300, -1e300,
        709.782_712_893_384, 710.0, -745.133_219_101_941_1, -746.0,
        core::f64::consts::PI, core::f64::consts::E, 1.0 - f64::EPSILON,
    ]);
    v
}

fn floats() -> Vec<f32> {
    let mut v: Vec<f32> = [
        0x7fc0_0000u32, // +quiet NaN
        0xffc0_0000,    // -quiet NaN
        0x7f80_0001,    // +signalling NaN
        0xff80_0001,    // -signalling NaN
        0x7fa1_2345,    // signalling NaN, payload — the 0x400000 OR in tanhf shows here
        0x7f80_0000,    // +inf
        0xff80_0000,    // -inf
        0x0000_0000,    // +0
        0x8000_0000,    // -0
        0x0000_0001,    // smallest subnormal
        0x807f_ffff,    // largest negative subnormal
        0x7f7f_ffff,    // FLT_MAX
    ]
    .iter()
    .map(|b| f32::from_bits(*b))
    .collect();
    v.extend_from_slice(&[
        1.0, -1.0, 0.5, -0.5, 2.0, -2.0, 88.0, 89.0, -87.0, -104.0,
        core::f32::consts::PI, core::f32::consts::E,
    ]);
    v
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// Row helpers
// ─────────────────────────────────────────────────────────────────────────────────────────

/// Everything one call is allowed to observe.
#[derive(PartialEq, Eq, Debug)]
struct Fx {
    bits: u64,
    errno: c_int,
    recs: Vec<Rec>,
}

const ERRNO_SEED: c_int = -12345;

/// Drive one double-taking wrapper on both sides and compare all three views.
fn row_d(what: &str, x: f64, subj: unsafe extern "C" fn(f64) -> f64, refr: unsafe extern "C" fn(f64) -> f64) {
    unsafe {
        rec::reset();
        subj_errno_set(ERRNO_SEED);
        let s = Fx { bits: subj(x).to_bits(), errno: subj_errno_get(), recs: rec::get(0) };
        ref_errno_set(ERRNO_SEED);
        let r = Fx { bits: refr(x).to_bits(), errno: ref_errno_get(), recs: rec::get(1) };
        assert_eq!(s, r, "{what}({x:?} = {:#018x})", x.to_bits());
    }
}

fn row_f(what: &str, x: f32, subj: unsafe extern "C" fn(f32) -> f32, refr: unsafe extern "C" fn(f32) -> f32) {
    unsafe {
        rec::reset();
        subj_errno_set(ERRNO_SEED);
        let s = Fx { bits: subj(x).to_bits() as u64, errno: subj_errno_get(), recs: rec::get(0) };
        ref_errno_set(ERRNO_SEED);
        let r = Fx { bits: refr(x).to_bits() as u64, errno: ref_errno_get(), recs: rec::get(1) };
        assert_eq!(s, r, "{what}({x:?} = {:#010x})", x.to_bits());
    }
}

/// Copy a NUL-terminated C string out RIGHT NOW. `_ecvt`/`_fcvt` return a pointer into the
/// shared `thread_data_t`, so the reference's call overwrites what the subject returned.
unsafe fn take(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let mut n = 0usize;
    while *p.add(n) != 0 {
        n += 1;
    }
    Some(String::from_utf8_lossy(core::slice::from_raw_parts(p as *const u8, n)).into_owned())
}

fn main() {
    eprintln!("\nmath.c differential\n");
    let mut rows = 0usize;

    // The handlers stay installed for the whole run: with them in place view 3 is live, and
    // because they return 0 views 1 and 2 are unaffected.
    unsafe {
        __setusermatherr(matherr_subj);
        ref___setusermatherr(matherr_ref);
    }

    // ── `_matherr` itself — the default handler, which must refuse every exception ────────
    unsafe {
        let mut e = CException {
            r#type: T_DOMAIN,
            name: b"probe\0".as_ptr() as *mut c_char,
            arg1: 1.0,
            arg2: 2.0,
            retval: 3.0,
        };
        assert_eq!(_matherr(&mut e), ref__matherr(&mut e), "_matherr disagreed");
        assert_eq!(_matherr(&mut e), 0, "_matherr must return 0 — a non-zero return would make \
                                          math_error skip its errno switch for every caller");
        rows += 2;
    }

    // ── the matherr-bearing transcendentals ──────────────────────────────────────────────
    for x in doubles() {
        row_d("asin", x, MSVCRT_asin, ref_MSVCRT_asin);
        row_d("atan", x, MSVCRT_atan, ref_MSVCRT_atan);
        row_d("exp", x, MSVCRT_exp, ref_MSVCRT_exp);
        row_d("tanh", x, MSVCRT_tanh, ref_MSVCRT_tanh);
        rows += 4;
    }
    for x in floats() {
        row_f("atanf", x, MSVCRT_atanf, ref_MSVCRT_atanf);
        row_f("tanhf", x, MSVCRT_tanhf, ref_MSVCRT_tanhf);
        rows += 2;
    }

    // ── sqrt / sqrtf, which need the ONE exemption in this file ──────────────────────────
    //
    // `sqrt_validate` reports a negative argument as `math_error(_DOMAIN, "sqrt", *x, 0,
    // ret_nan(TRUE))`, and `ret_nan(TRUE)` is `double x = 1.0; return (x - x) / (x - x);` —
    // a COMPILE-TIME 0.0/0.0, whose NaN sign C leaves unspecified. The two compilers disagree:
    //
    //     host gcc (the reference)          -> 0xfff8000000000000
    //     llvm-mingw clang, msvcrt's flags  -> 0x7ff8000000000000   <- what the DLL ships
    //     rustc/LLVM (the subject)          -> 0x7ff8000000000000
    //
    // Measured, not assumed: `x86_64-w64-mingw32-clang -O1 -march=athlon64 -mlong-double-64`
    // on that exact body emits `.quad 0x7ff8000000000000`. So the SUBJECT matches the shipping
    // binary and the REFERENCE is the outlier — this is the "ref_* bodies are host-compiler
    // rewrites, not the C as written" caveat arriving as a concrete row. Changing the Rust to
    // match the reference would be corrupting correct code to match a host artifact.
    //
    // The exemption is therefore narrow and one-directional: the reference is allowed to
    // differ in the NaN SIGN BIT ALONE, while the subject is pinned ABSOLUTELY to clang's
    // fold. That is stricter than comparing the two sides, not weaker — if the Rust ever
    // stops agreeing with the shipping compiler, this fails.
    const CLANG_FOLDED_NAN: u64 = 0x7ff8_0000_0000_0000;
    for x in doubles() {
        unsafe {
            rec::reset();
            subj_errno_set(ERRNO_SEED);
            let sb = MSVCRT_sqrt(x).to_bits();
            let (se, sr) = (subj_errno_get(), rec::get(0));
            ref_errno_set(ERRNO_SEED);
            let rb = ref_MSVCRT_sqrt(x).to_bits();
            let (re, rr) = (ref_errno_get(), rec::get(1));

            assert_eq!(se, re, "sqrt({x:?}): errno differs");
            assert_eq!(sr.len(), rr.len(), "sqrt({x:?}): different number of math_error reports");

            let folded = x.is_sign_negative() && !x.is_nan() && x != 0.0;
            if folded {
                assert_eq!(sb, CLANG_FOLDED_NAN,
                    "sqrt({x:?}) did not return the NaN llvm-mingw clang folds ret_nan(TRUE) to");
                assert_eq!(rb & !(1 << 63), sb & !(1 << 63),
                    "sqrt({x:?}): the two NaNs differ by more than the folded sign bit");
                // The reported struct must still match apart from that same one bit.
                assert_eq!(sr.len(), 1, "sqrt({x:?}) must report exactly one _DOMAIN");
                assert_eq!(sr[0].ty, T_DOMAIN, "sqrt({x:?}) reported the wrong error type");
                assert_eq!(sr[0].name, "sqrt");
                assert_eq!((sr[0].ty, &sr[0].name, sr[0].arg1, sr[0].arg2),
                           (rr[0].ty, &rr[0].name, rr[0].arg1, rr[0].arg2),
                           "sqrt({x:?}): the reported exception differs outside the NaN sign");
                assert_eq!(sr[0].retval & !(1 << 63), rr[0].retval & !(1 << 63),
                           "sqrt({x:?}): reported retval differs by more than the folded sign");
                assert_eq!(se, E_DOM, "sqrt of a negative must leave EDOM");
            } else {
                assert_eq!(sb, rb, "sqrt({x:?} = {:#018x}): result differs", x.to_bits());
                assert_eq!(sr, rr, "sqrt({x:?}): reported exception differs");
            }
            rows += 1;
        }
    }
    for x in floats() {
        // sqrtf's negative path is the same fold, one precision down.
        unsafe {
            rec::reset();
            subj_errno_set(ERRNO_SEED);
            let sb = MSVCRT_sqrtf(x).to_bits();
            let (se, sr) = (subj_errno_get(), rec::get(0));
            ref_errno_set(ERRNO_SEED);
            let rb = ref_MSVCRT_sqrtf(x).to_bits();
            let (re, rr) = (ref_errno_get(), rec::get(1));
            assert_eq!(se, re, "sqrtf({x:?}): errno differs");
            let folded = x.is_sign_negative() && !x.is_nan() && x != 0.0;
            if folded {
                assert_eq!(sb & !(1u32 << 31), rb & !(1u32 << 31),
                           "sqrtf({x:?}): differs by more than the folded sign bit");
                assert_eq!(sr.len(), rr.len(), "sqrtf({x:?}): report count differs");
                assert_eq!(sr[0].ty, T_DOMAIN);
                assert_eq!(sr[0].name, "sqrtf");
            } else {
                assert_eq!(sb, rb, "sqrtf({x:?} = {:#010x}): result differs", x.to_bits());
                assert_eq!(sr, rr, "sqrtf({x:?}): reported exception differs");
            }
            rows += 1;
        }
    }

    // ── classification, sign, magnitude ──────────────────────────────────────────────────
    unsafe {
        for x in floats() {
            assert_eq!(_fpclassf(x), ref__fpclassf(x),
                       "_fpclassf({x:?} = {:#010x})", x.to_bits());
            rows += 1;
        }
        for x in doubles() {
            assert_eq!(_chgsign(x).to_bits(), ref__chgsign(x).to_bits(),
                       "_chgsign({x:?} = {:#018x})", x.to_bits());
            rows += 1;
        }
        // i64::MIN is the row that matters: `n >= 0 ? n : -n` overflows there, and C's
        // wraparound and Rust's `-` are not the same operation in a debug build.
        for n in [0i64, 1, -1, i64::MAX, i64::MIN, i64::MIN + 1, 0x7fff_ffff, -0x8000_0000] {
            assert_eq!(_abs64(n), ref__abs64(n), "_abs64({n})");
            rows += 1;
        }
    }

    // ── _scalb / _scalbf / _cabs ─────────────────────────────────────────────────────────
    unsafe {
        for x in doubles() {
            for p in [0i32, 1, -1, 52, -52, 1023, -1023, 2000, -2000, i32::MAX, i32::MIN] {
                assert_eq!(_scalb(x, p).to_bits(), ref__scalb(x, p).to_bits(),
                           "_scalb({x:?}, {p})");
                rows += 1;
            }
        }
        for x in floats() {
            for p in [0i32, 1, -1, 24, -24, 127, -127, 400, -400] {
                assert_eq!(_scalbf(x, p).to_bits(), ref__scalbf(x, p).to_bits(),
                           "_scalbf({x:?}, {p})");
                rows += 1;
            }
        }
        for (x, y) in [(0.0f64, 0.0f64), (3.0, 4.0), (-3.0, 4.0), (3.0, -4.0),
                       (1e300, 1e300), (1e-300, 1e-300), (f64::INFINITY, 1.0),
                       (f64::NAN, 1.0), (1.0, f64::NAN)] {
            let c = CComplex { x, y };
            assert_eq!(_cabs(c).to_bits(), ref__cabs(c).to_bits(), "_cabs({x:?}, {y:?})");
            rows += 1;
        }
    }

    // ── the rotates ──────────────────────────────────────────────────────────────────────
    //
    // Shift 0 is the row worth writing them for. Every one of these bodies is
    // `(num << shift) | (num >> (WIDTH - shift))`, which at shift 0 shifts by the full width
    // — undefined in C, and a panic in a debug-built Rust. A port that reproduced the C
    // literally and a port that special-cased 0 both look right at every other shift.
    unsafe {
        let u32s: [c_uint; 8] = [0, 1, 0x8000_0000, 0xffff_ffff, 0x1234_5678, 0xdead_beef, 0x0000_00ff, 0xaaaa_5555];
        let u64s: [u64; 8] = [0, 1, 0x8000_0000_0000_0000, u64::MAX, 0x0123_4567_89ab_cdef,
                              0xdead_beef_cafe_babe, 0xff, 0xaaaa_aaaa_5555_5555];
        for &n in &u32s {
            for s in [-33i32, -32, -1, 0, 1, 7, 31, 32, 33, 63, 64, i32::MAX] {
                assert_eq!(MSVCRT__rotl(n, s), ref_MSVCRT__rotl(n, s), "_rotl({n:#x}, {s})");
                assert_eq!(MSVCRT__rotr(n, s), ref_MSVCRT__rotr(n, s), "_rotr({n:#x}, {s})");
                assert_eq!(MSVCRT__lrotl(n, s), ref_MSVCRT__lrotl(n, s), "_lrotl({n:#x}, {s})");
                assert_eq!(MSVCRT__lrotr(n, s), ref_MSVCRT__lrotr(n, s), "_lrotr({n:#x}, {s})");
                rows += 4;
            }
        }
        for &n in &u64s {
            for s in [-65i32, -64, -1, 0, 1, 7, 63, 64, 65, 127, i32::MAX] {
                assert_eq!(MSVCRT__rotl64(n, s), ref_MSVCRT__rotl64(n, s), "_rotl64({n:#x}, {s})");
                assert_eq!(MSVCRT__rotr64(n, s), ref_MSVCRT__rotr64(n, s), "_rotr64({n:#x}, {s})");
                rows += 2;
            }
        }
    }

    // ── _ecvt / _fcvt / _gcvt and the _s forms ───────────────────────────────────────────
    //
    // THESE TWO HAVE AN UNGRADEABLE DOMAIN, and it is the C's, not the port's. Both print
    // into a `char buf[80]` with `_snprintf`, which on overflow fills all 80 bytes and
    // writes NO terminator — and then both READ THE BUFFER BACK with an unbounded
    // `while (*ptr1 != '\0')`. Past the boundary they walk into stack garbage, so the two
    // sides read two different stacks and no comparison means anything. `_ecvt` goes
    // further: `len` is then -1 and `memmove(buf, buf + 1, len--)` is a `(size_t)-1` copy.
    //
    // `_ecvt` clamps `ndigits` to 72 to stay inside 80 ("8 : space for sign, dec point, e,
    // 4 for exponent and one for terminating '\0'", says the comment) — but the arithmetic
    // is short by one for a THREE-digit exponent with a sign, so `_ecvt(-1e300, 72)` is over
    // the line anyway. `_fcvt` does not clamp at all, so anything whose integer part is long
    // (1e300 is 301 digits) is over it at ndigits=0.
    //
    // So the rows ASK, rather than assume: the formatted length is measured with the same
    // formatter through a count-only call, and a case that would not fit is skipped and
    // COUNTED. A silent cap here would read as "the cvt family is graded" when the
    // interesting wide cases were dropped.
    let mut cvt_skipped = 0usize;
    unsafe {
        // `_ecvt` at math.c:1754 and 1762: ndigits clamped to 72, then `prec = ndigits`,
        // `if (prec < 1) prec = 2`, printed as `"%.*le"` with `prec - 1`.
        let ecvt_fits = |num: f64, nd: c_int| -> bool {
            let mut prec = nd.min(80 - 8);
            if prec < 1 {
                prec = 2;
            }
            _snprintf(core::ptr::null_mut(), 0, b"%.*le\0".as_ptr() as *const c_char,
                      prec - 1, num) < 80
        };
        // `_fcvt` at math.c:1851: `"%.*f"` with `ndigits < 0 ? 0 : ndigits`, unclamped.
        let fcvt_fits = |num: f64, nd: c_int| -> bool {
            _snprintf(core::ptr::null_mut(), 0, b"%.*f\0".as_ptr() as *const c_char,
                      if nd < 0 { 0 } else { nd }, num) < 80
        };
        // INFINITY and NAN are the second ungradeable domain, and a different mechanism to
        // the overflow above. `_ecvt` assumes the text it printed has the shape
        // `d.ddddde[+-]NN` — it writes `buf[prec] = '\0'` and then `sscanf(buf + prec + 1,
        // "%d")` to recover the exponent. `%.*le` gives it "inf" (three characters), so both
        // of those index PAST the string, into whatever the 80-byte `efcvt_buffer` last
        // held. That buffer lives in the SHARED thread data, so what they find is the other
        // side's leftovers — the two sides disagree for a reason that is about call order,
        // not about either implementation.
        let cvt_gradeable = |num: f64, nd: c_int| -> bool {
            num.is_finite() && ecvt_fits(num, nd) && fcvt_fits(num, nd)
        };

        let numbers = [0.0f64, -0.0, 1.0, -1.0, 0.5, 123.456, -123.456, 1e-5, 1e20, -1e20,
                       f64::INFINITY, f64::NEG_INFINITY, f64::NAN, 9.999_999_999_9,
                       1e300, 5e-324, 0.1, 2.5, -2.5];
        for &num in &numbers {
            for nd in [-1i32, 0, 1, 2, 5, 15, 17, 40, 72, 73, 80, 100] {
                // `_gcvt` is graded for EVERY input: it `sprintf`s straight into the
                // caller's buffer and never reads it back, so none of the domain below
                // applies to it.
                let mut sbuf = [0u8; 256];
                let mut rbuf = [0u8; 256];
                subj_errno_set(ERRNO_SEED);
                let sp = take(_gcvt(num, nd, sbuf.as_mut_ptr() as *mut c_char));
                let se = subj_errno_get();
                ref_errno_set(ERRNO_SEED);
                let rp = take(ref__gcvt(num, nd, rbuf.as_mut_ptr() as *mut c_char));
                assert_eq!((sp, se, sbuf), (rp, ref_errno_get(), rbuf),
                           "_gcvt({num:?}, {nd})");
                rows += 1;

                if !cvt_gradeable(num, nd) {
                    cvt_skipped += 1;
                    continue;
                }
                // `_ecvt` — capture immediately: the buffer lives in the SHARED thread data.
                let (mut sd, mut ss) = (0i32, 0i32);
                subj_errno_set(ERRNO_SEED);
                let sp = take(_ecvt(num, nd, &mut sd, &mut ss));
                let se = subj_errno_get();
                let (mut rd, mut rs) = (0i32, 0i32);
                ref_errno_set(ERRNO_SEED);
                let rp = take(ref__ecvt(num, nd, &mut rd, &mut rs));
                assert_eq!((sp, sd, ss, se), (rp, rd, rs, ref_errno_get()),
                           "_ecvt({num:?}, {nd})");

                let (mut sd, mut ss) = (0i32, 0i32);
                subj_errno_set(ERRNO_SEED);
                let sp = take(_fcvt(num, nd, &mut sd, &mut ss));
                let se = subj_errno_get();
                let (mut rd, mut rs) = (0i32, 0i32);
                ref_errno_set(ERRNO_SEED);
                let rp = take(ref__fcvt(num, nd, &mut rd, &mut rs));
                assert_eq!((sp, sd, ss, se), (rp, rd, rs, ref_errno_get()),
                           "_fcvt({num:?}, {nd})");
                rows += 2;
            }
        }

        // The `_s` forms, whose contract is the refusal: a bad size or a NULL buffer must
        // reach the invalid-parameter handler and return an errno CODE rather than set errno.
        for &num in &numbers {
            for &(len, nd) in &[(0usize, 5i32), (1, 5), (2, 5), (8, 5), (32, 5), (256, 5),
                                (256, 0), (256, -1), (256, 80), (256, 100)] {
                // `_ecvt_s`/`_fcvt_s` carry the same `char buf[80]` as their unsuffixed
                // twins, so the same domain is ungradeable for the same reason.
                //
                // AND A SECOND ONE, which is msvcrt's own and which this port reproduces
                // exactly: a destination SMALLER than the conversion HANGS. math.c:1961-1966
                //
                //     if ((ptr1 - buf) < stop) {
                //         if (size > 1) { *ptr2++ = *ptr1++; size--; }
                //     } else ptr1++;
                //
                // has no `else` on the inner `if`, so once `size` reaches 1 with digits still
                // to copy, neither pointer moves and the `while` spins forever. The same
                // shape appears again in the fraction loop below it. `_fcvt_s(1e20, 5)` into
                // an 8-byte buffer is enough to hit it, and the Rust hangs in exactly the
                // same loop — which is correct behaviour for a faithful port and useless as a
                // row. `size == 0` is kept, because that one takes the EINVAL refusal before
                // any of this and IS the `_s` contract worth grading.
                let printed = _snprintf(core::ptr::null_mut(), 0,
                                        b"%.*f\0".as_ptr() as *const c_char,
                                        if nd < 0 { 0 } else { nd }, num);
                if !cvt_gradeable(num, nd) || (len != 0 && (len as c_int) <= printed) {
                    cvt_skipped += 1;
                    continue;
                }
                let mut sbuf = [0x5au8; 256];
                let mut rbuf = [0x5au8; 256];
                let (mut sd, mut ss) = (0i32, 0i32);
                let (mut rd, mut rs) = (0i32, 0i32);

                subj_invalid_pmt_reset();
                subj_errno_set(ERRNO_SEED);
                let sr = _ecvt_s(sbuf.as_mut_ptr() as *mut c_char, len, num, nd, &mut sd, &mut ss);
                let s = (sr, subj_errno_get(), subj_invalid_pmt_count(), sd, ss, sbuf);
                ref_invalid_pmt_reset();
                ref_errno_set(ERRNO_SEED);
                let rr = ref__ecvt_s(rbuf.as_mut_ptr() as *mut c_char, len, num, nd, &mut rd, &mut rs);
                let r = (rr, ref_errno_get(), ref_invalid_pmt_count(), rd, rs, rbuf);
                assert_eq!(s, r, "_ecvt_s({num:?}, len={len}, nd={nd})");

                let mut sbuf = [0x5au8; 256];
                let mut rbuf = [0x5au8; 256];
                subj_invalid_pmt_reset();
                subj_errno_set(ERRNO_SEED);
                let sr = _fcvt_s(sbuf.as_mut_ptr() as *mut c_char, len, num, nd, &mut sd, &mut ss);
                let s = (sr, subj_errno_get(), subj_invalid_pmt_count(), sd, ss, sbuf);
                ref_invalid_pmt_reset();
                ref_errno_set(ERRNO_SEED);
                let rr = ref__fcvt_s(rbuf.as_mut_ptr() as *mut c_char, len, num, nd, &mut rd, &mut rs);
                let r = (rr, ref_errno_get(), ref_invalid_pmt_count(), rd, rs, rbuf);
                assert_eq!(s, r, "_fcvt_s({num:?}, len={len}, nd={nd})");

                let mut sbuf = [0x5au8; 256];
                let mut rbuf = [0x5au8; 256];
                subj_invalid_pmt_reset();
                subj_errno_set(ERRNO_SEED);
                let sr = _gcvt_s(sbuf.as_mut_ptr() as *mut c_char, len, num, nd);
                let s = (sr, subj_errno_get(), subj_invalid_pmt_count(), sbuf);
                ref_invalid_pmt_reset();
                ref_errno_set(ERRNO_SEED);
                let rr = ref__gcvt_s(rbuf.as_mut_ptr() as *mut c_char, len, num, nd);
                let r = (rr, ref_errno_get(), ref_invalid_pmt_count(), rbuf);
                assert_eq!(s, r, "_gcvt_s({num:?}, len={len}, nd={nd})");
                rows += 3;
            }
        }

        // NULL buffer / NULL out-parameters — the arms `_gcvt` handles itself with errno.
        subj_errno_set(0);
        let s = (take(_gcvt(1.0, 5, core::ptr::null_mut())), subj_errno_get());
        ref_errno_set(0);
        let r = (take(ref__gcvt(1.0, 5, core::ptr::null_mut())), ref_errno_get());
        assert_eq!(s, r, "_gcvt(NULL buffer)");
        let mut b = [0u8; 64];
        subj_errno_set(0);
        let s = (take(_gcvt(1.0, -1, b.as_mut_ptr() as *mut c_char)), subj_errno_get());
        ref_errno_set(0);
        let r = (take(ref__gcvt(1.0, -1, b.as_mut_ptr() as *mut c_char)), ref_errno_get());
        assert_eq!(s, r, "_gcvt(negative ndigit)");
        rows += 2;
    }

    // ── the FPU control-word family ──────────────────────────────────────────────────────
    //
    // Every row here snapshots and restores MXCSR so that both sides see the same entry
    // state, and grades the register the call LEAVES BEHIND as well as what it returns —
    // `_set_controlfp` returns nothing at all, so the register write is its whole contract.
    unsafe {
        // Read-only first: mask 0 must report without changing anything.
        let (s, scsr, r, rcsr) = paired_fp(|| _control87(0, 0), || ref__control87(0, 0));
        assert_eq!((s, scsr), (r, rcsr), "_control87(0, 0) — the pure read");
        rows += 1;

        let cases: [(c_uint, c_uint); 12] = [
            (0, 0),
            (msvcrt_sys::_EM_INVALID, msvcrt_sys::_MCW_EM),
            (msvcrt_sys::_EM_ZERODIVIDE, msvcrt_sys::_MCW_EM),
            (0, msvcrt_sys::_MCW_EM),
            (msvcrt_sys::_MCW_EM, msvcrt_sys::_MCW_EM),
            (msvcrt_sys::_RC_UP, msvcrt_sys::_MCW_RC),
            (msvcrt_sys::_RC_DOWN, msvcrt_sys::_MCW_RC),
            (msvcrt_sys::_RC_CHOP, msvcrt_sys::_MCW_RC),
            (msvcrt_sys::_RC_NEAR, msvcrt_sys::_MCW_RC),
            (msvcrt_sys::_DN_FLUSH, msvcrt_sys::_MCW_DN),
            (msvcrt_sys::_PC_24, msvcrt_sys::_MCW_PC),
            (0xffff_ffff, 0xffff_ffff),
        ];
        for (newval, mask) in cases {
            let (s, scsr, r, rcsr) = paired_fp(|| _control87(newval, mask),
                                               || ref__control87(newval, mask));
            assert_eq!((s, scsr), (r, rcsr), "_control87({newval:#x}, {mask:#x})");

            let (s, scsr, r, rcsr) = paired_fp(|| _controlfp(newval, mask),
                                               || ref__controlfp(newval, mask));
            assert_eq!((s, scsr), (r, rcsr), "_controlfp({newval:#x}, {mask:#x})");

            let (s, scsr, r, rcsr) = paired_fp(|| { _set_controlfp(newval, mask); },
                                               || { ref__set_controlfp(newval, mask); });
            assert_eq!((s, scsr), (r, rcsr), "_set_controlfp({newval:#x}, {mask:#x})");

            // `_controlfp_s` refuses a `newval & mask` outside the known flag set through the
            // invalid-parameter handler, and still fills `*cur`. Both halves are graded.
            // Written out rather than through `paired_fp` because four more things than the
            // return value have to be read between the two calls.
            let (mut scur, mut rcur) = (0u32, 0u32);
            subj_invalid_pmt_reset();
            subj_errno_set(ERRNO_SEED);
            let entry = mxcsr();
            let sret = _controlfp_s(&mut scur, newval, mask);
            let s = (sret, subj_errno_get(), subj_invalid_pmt_count(), scur, mxcsr());
            set_mxcsr(entry);
            ref_invalid_pmt_reset();
            ref_errno_set(ERRNO_SEED);
            let rret = ref__controlfp_s(&mut rcur, newval, mask);
            let r = (rret, ref_errno_get(), ref_invalid_pmt_count(), rcur, mxcsr());
            set_mxcsr(entry);
            assert_eq!(s, r, "_controlfp_s({newval:#x}, {mask:#x})");
            rows += 4;
        }

        // `_statusfp` reads the exception flags; `_clearfp` reads and clears them. Both need
        // real flags set to be worth anything, so each row raises one first — from a value
        // computation, not by poking MXCSR, so the flag arrives the way a caller's would.
        for (label, raise) in [("invalid", 0x0001u32), ("denormal", 0x0002), ("zerodivide", 0x0004),
                               ("overflow", 0x0008), ("underflow", 0x0010), ("inexact", 0x0020),
                               ("all", 0x003f), ("none", 0x0000)] {
            let entry = mxcsr();
            set_mxcsr((entry & !0x3f) | raise);
            let s = (_statusfp(), mxcsr());
            set_mxcsr((entry & !0x3f) | raise);
            let r = (ref__statusfp(), mxcsr());
            set_mxcsr(entry);
            assert_eq!(s, r, "_statusfp with {label} raised");

            set_mxcsr((entry & !0x3f) | raise);
            let s = (_clearfp(), mxcsr());
            set_mxcsr((entry & !0x3f) | raise);
            let r = (ref__clearfp(), mxcsr());
            set_mxcsr(entry);
            assert_eq!(s, r, "_clearfp with {label} raised");
            rows += 2;
        }

        // `_fpreset` returns nothing — the register it leaves IS the function.
        let entry = mxcsr();
        set_mxcsr(entry | 0x3f);
        _fpreset();
        let scsr = mxcsr();
        set_mxcsr(entry | 0x3f);
        ref__fpreset();
        let rcsr = mxcsr();
        set_mxcsr(entry);
        assert_eq!(scsr, rcsr, "_fpreset left a different MXCSR");
        rows += 1;

        // `__fpecode` is `&msvcrt_get_thread_data()->fpecode`, and the two sides have
        // different thread data, so what is graded is the three things that ARE the body:
        // the pointer is STABLE (it is a slot, not a scratch value), a write through it is
        // READABLE back through it, and the two sides' slots are DISTINCT — which is the
        // harness's own invariant. A shared slot would mean one side's fpecode could satisfy
        // the other's assertion, and every FP-exception row after this would be meaningless.
        assert_eq!(__fpecode(), __fpecode(), "the subject's __fpecode is not a stable slot");
        assert_eq!(ref___fpecode(), ref___fpecode(),
                   "the reference's __fpecode is not a stable slot");
        assert_ne!(__fpecode(), ref___fpecode(),
                   "both sides' __fpecode reached the SAME slot — the thread_data namespacing \
                    has broken, and every per-side reading in this binary is now suspect");
        *__fpecode() = 0x1234;
        *ref___fpecode() = 0x5678;
        assert_eq!((*__fpecode(), *ref___fpecode()), (0x1234, 0x5678),
                   "__fpecode does not report the slot it writes");
        *__fpecode() = 0;
        *ref___fpecode() = 0;
        rows += 5;

        // `_set_SSE2_enable` is `flag && sse2_supported`, and `sse2_supported` is a file
        // static only `msvcrt_init_math` writes — which never runs on the host. So both sides
        // hold 0 and the row grades the `&&`, not the detection.
        for f in [0i32, 1, -1, 0x7fff_ffff] {
            assert_eq!(_set_SSE2_enable(f), ref__set_SSE2_enable(f), "_set_SSE2_enable({f})");
            rows += 1;
        }
    }

    // ── the handler itself: __setusermatherr must be able to REPLACE and to CLEAR ────────
    unsafe {
        rec::reset();
        // A handler returning non-zero suppresses the errno store and lets the caller choose
        // the return value — the extension point's actual purpose, and untested by every row
        // above because those all return 0.
        __setusermatherr(matherr_override_subj);
        ref___setusermatherr(matherr_override_ref);
        subj_errno_set(ERRNO_SEED);
        let sb = MSVCRT_exp(f64::NAN).to_bits();
        let se = subj_errno_get();
        ref_errno_set(ERRNO_SEED);
        let rb = ref_MSVCRT_exp(f64::NAN).to_bits();
        assert_eq!((sb, se), (rb, ref_errno_get()),
                   "a non-zero matherr handler must override the return value and suppress errno");
        assert_eq!(se, ERRNO_SEED,
                   "errno was written despite the handler claiming the exception");
        assert_eq!(sb, OVERRIDE.to_bits(), "the handler's retval was not returned");
        rows += 3;
    }

    eprintln!("\nmath: {rows} rows ({cvt_skipped} cvt cases skipped, all in the two domains \
               the comments above the _ecvt/_fcvt blocks name: inputs that overflow the C's \
               own 80-byte scratch and then read back stack garbage, and `_s` destinations \
               smaller than the conversion, where msvcrt's own copy loop spins forever)\n");
}

/// The value the overriding handler substitutes — arbitrary, but not a value any real path
/// produces, so seeing it proves the override took effect.
const OVERRIDE: f64 = 1234.5;

unsafe extern "C" fn matherr_override_subj(e: *mut CException) -> c_int {
    (*e).retval = OVERRIDE;
    1
}
unsafe extern "C" fn matherr_override_ref(e: *mut CException) -> c_int {
    (*e).retval = OVERRIDE;
    1
}
