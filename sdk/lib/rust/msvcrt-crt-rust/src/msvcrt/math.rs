
//! `dll/win32/msvcrt/math.c`, translated whole. **WIRED** — `mod.rs:197` says `pub mod math;`,
//! so this file compiles and ships.
//!
//! The `ORPHAN-BY-DESIGN` banner that used to sit here was removed on 2026-08-05 because it
//! had become false, and a false banner is not merely untidy: `scan_extern_arity` and
//! `scan_cc_mismatch` both SKIP a file the moment they see it, on the premise that nothing
//! compiles it. Unmasking this file was measured against both scanners first — zero new
//! findings — so nothing is being papered over by the removal.
//!
//! WHAT WAS OWED WHEN IT WAS BANKED, and is worth re-reading before touching the asm or the
//! flagged bodies:
//!
//! 1. **The 64 inline-asm sites are wrong and must be rewritten by hand.** math.c reaches
//!    the x87 control word and MXCSR through GCC memory operands —
//!    `__asm__ __volatile__("stmxcsr %0" : "=m"(ret))` — and the translation rendered them
//!    as register operands in Intel syntax (`asm!("stmxcsr {}", out(reg) ret)`). `stmxcsr`,
//!    `fstsw`, `fstcw`, `fnstenv` and `fldcw` all REQUIRE a memory operand, so most of these
//!    will not assemble; the danger is the handful that would. This is the FP control state
//!    for the whole OS — getting a mask bit into the wrong field changes the result of every
//!    float operation, silently — so it wants a careful pass with a smoke behind it, not a
//!    hurried one. Correct shape: `asm!("stmxcsr [{}]", in(reg) core::ptr::addr_of_mut!(ret),
//!    options(nostack))`.
//! 2. **Fifteen bodies are flagged UNVERIFIED CONSTANTS** (see the wave's AUDIT.md): the
//!    model ignored the fact envelope for `_fpclass`, `_fpclassf`, `fenv_encode`, `_ecvt`,
//!    `_ecvt_s`, `_fcvt`, `_fcvt_s`, `sqrt_validate`, `sqrtf_validate`, `_setfp`, `_setfp_sse`
//!    and friends, so their constants are unverified. Spot-checks so far (`MSVCRT_asin`,
//!    `sqrt_validate`, `_setfp`'s head) came back faithful — see the field manual on why
//!    recall usually produces the right body — but "usually" is not a check. Diff each
//!    against the C before the flip.
//! 3. **Nine symbols are already owned elsewhere** and are `#[cfg(any())]` below with the
//!    reason at each site: cnt-rust's difftest-pinned `cabi/math.rs` exports them into the
//!    same archive, and two strong definitions in one archive resolve by member order.
//! 4. Then `pub mod math;`, `cargo check` on all three arches, difftest, gate.
//!
//! Everything else about the file is the ordinary whole-file translation contract:
//! identifiers exactly as the C spells them, and the original C appended as a comment block.

use crate::prelude::*;
use msvcrt_sys::*;
// Both globs above export these, so every bare use is ambiguous. Take the
// bindgen island's copies: they are BOOL (an i32), derived from windef.h.
use msvcrt_sys::{FALSE, TRUE};
use core::ffi::c_void;

// The libm routines math.c calls out to. This list is the object file's own UNDEFINED
// symbol table (`llvm-nm --undefined-only math.c.obj`), not a guess: getting it from the
// binary is what caught that `asm_sqrt`/`asm_sqrtf`/`x87_asin` are never called at all here.
extern "C" {
    pub fn acos(x: f64) -> f64;
    pub fn asin(x: f64) -> f64;
    pub fn atan(x: f64) -> f64;
    pub fn atan2(y: f64, x: f64) -> f64;
    pub fn atanf(x: f32) -> f32;
    pub fn cos(x: f64) -> f64;
    pub fn cosh(x: f64) -> f64;
    pub fn exp(x: f64) -> f64;
    pub fn fmod(x: f64, y: f64) -> f64;
    pub fn log(x: f64) -> f64;
    pub fn log10(x: f64) -> f64;
    pub fn pow(x: f64, y: f64) -> f64;
    pub fn sin(x: f64) -> f64;
    pub fn sinh(x: f64) -> f64;
    pub fn sqrt(x: f64) -> f64;
    pub fn sqrtf(x: f32) -> f32;
    pub fn tan(x: f64) -> f64;
    pub fn tanh(x: f64) -> f64;
    pub fn tanhf(x: f32) -> f32;
}

#[cfg(target_arch = "arm64ec")]
extern "system" {
    static __os_arm64x_get_x64_information: Option<unsafe extern "system" fn(ULONG, *mut core::ffi::c_void, *mut core::ffi::c_void) -> NTSTATUS>;
    static __os_arm64x_set_x64_information: Option<unsafe extern "system" fn(ULONG, ULONG_PTR, *mut core::ffi::c_void) -> NTSTATUS>;
}



// `dll/win32/msvcrt/include/math.h:310-312` defines these as MACROS over the CRT's own
// classification entry points, so they expand at each use site and never appear as calls in
// the object. Reproduced as inline fns rather than macros because they are expressions with
// one argument and no token-pasting — the §9.65 rule that forbids an out-of-line no-op `fn`
// for a DEBUG macro does not apply: these have real bodies, so there is no ABI-mismatch
// barrier to uncloak.
//
// `signbit` is the one place this diverges in FORM: the header routes it through `_dsign`,
// which appears in no arch's undefined-symbol list, so the C is already reaching an inline.
// A sign-bit test is exact for every input including -0.0 and NaN, which is what `_dsign`
// documents.
#[inline]
unsafe fn isnan(x: f64) -> i32 {
    (_dclass(x) as i32 == FP_NAN as i32) as i32
}
#[inline]
unsafe fn isnanf(x: f32) -> i32 {
    (_fdclass(x) as i32 == FP_NAN as i32) as i32
}
#[inline]
fn signbit(x: f64) -> i32 {
    (x.to_bits() >> 63) as i32
}
#[inline]
fn signbitf(x: f32) -> i32 {
    (x.to_bits() >> 31) as i32
}

/// `NAN` from math.h. `core::f64::NAN` is the same quiet NaN the C constant expands to.
const NAN: f64 = f64::NAN;

// Two more CRT siblings math.c calls that have no Rust definition yet, plus the
// classification entry points the macros above are built on. `sscanf` and `_scprintf` are
// C-variadic: DECLARING one is stable Rust, only DEFINING one needs nightly.
extern "C" {
    pub fn _dclass(x: f64) -> i16;
    pub fn _fdclass(x: f32) -> i16;
    pub fn ldexp(num: f64, exp: i32) -> f64;
    pub fn sscanf(src: *const i8, fmt: *const i8, ...) -> i32;
    pub fn _scprintf(fmt: *const i8, ...) -> i32;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
}

// ---- file-scope preamble (translated) ----
pub const _DOMAIN: i32 = 1;
pub const _SING: i32 = 2;
pub const _OVERFLOW: i32 = 3;
pub const _UNDERFLOW: i32 = 4;

pub type MSVCRT_matherr_func = Option<extern "C" fn(*mut _exception) -> i32>;

pub static mut MSVCRT_default_matherr_func: MSVCRT_matherr_func = None;
pub static mut sse2_supported: BOOL = 0;
pub static mut sse2_enabled: BOOL = 0;

pub const FENV_X_INVALID: u32 = 0x00100010;
pub const FENV_X_DENORMAL: u32 = 0x00200020;
pub const FENV_X_ZERODIVIDE: u32 = 0x00080008;
pub const FENV_X_OVERFLOW: u32 = 0x00040004;
pub const FENV_X_UNDERFLOW: u32 = 0x00020002;
pub const FENV_X_INEXACT: u32 = 0x00010001;
pub const FENV_X_AFFINE: u32 = 0x00004000;
pub const FENV_X_UP: u32 = 0x00800200;
pub const FENV_X_DOWN: u32 = 0x00400100;
pub const FENV_X_24: u32 = 0x00002000;
pub const FENV_X_53: u32 = 0x00001000;
pub const FENV_Y_INVALID: u32 = 0x10000010;
pub const FENV_Y_DENORMAL: u32 = 0x20000020;
pub const FENV_Y_ZERODIVIDE: u32 = 0x08000008;
pub const FENV_Y_OVERFLOW: u32 = 0x04000004;
pub const FENV_Y_UNDERFLOW: u32 = 0x02000002;
pub const FENV_Y_INEXACT: u32 = 0x01000001;
pub const FENV_Y_UP: u32 = 0x80000200;
pub const FENV_Y_DOWN: u32 = 0x40000100;
pub const FENV_Y_FLUSH: u32 = 0x00000400;
pub const FENV_Y_FLUSH_SAVE: u32 = 0x00000800;

// msvcrt.spec:320 exports this as `_adjust_fdiv`, so the DLL export table names the
// symbol and a Rust-mangled static does not satisfy it. `-arch=i386` there, and
// `#ifdef __i386__` in the C, so it exists on exactly one arch.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub static mut MSVCRT__adjust_fdiv: i32 = 0;
#[cfg(not(target_arch = "x86"))]
pub static mut MSVCRT__adjust_fdiv: i32 = 0;

/// `KUSER_SHARED_DATA.ProcessorFeatures`, at the fixed user-mode address the C spells
/// inline: `((volatile const BYTE *)(ULONG_PTR)0x7FFE0274)[...]`. Typed as the ARRAY it
/// is rather than as `*const u8` plus an offset — a hand-computed byte offset is the one
/// shape `scan_magic_offsets` exists to forbid, and it is no less wrong for having been
/// copied faithfully from a C cast.
const KSHARED_PROCESSOR_FEATURES: *const [u8; 64] = 0x7FFE0274 as *const [u8; 64];
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_math(module: *mut core::ffi::c_void) {
    sse2_supported =
        ((*KSHARED_PROCESSOR_FEATURES)[PF_XMMI64_INSTRUCTIONS_AVAILABLE as usize] != 0) as BOOL;
    sse2_enabled = FALSE as BOOL;
}
unsafe extern "C" fn ret_nan(update_sw: u8) -> f64 {
    let x = 1.0;
    if update_sw == 0 {
        return -NAN;
    }
    (x - x) / (x - x)
}

#[no_mangle]
pub unsafe extern "C" fn _matherr(e: *mut _exception) -> i32 {
    0
}
// CMakeLists.txt does `-Dmath_error=_invoke_matherr`, so the symbol this file must
// define is `_invoke_matherr` — `math_error` appears in no arch's math.c.obj. Exporting
// the spelled name instead would leave the C copy of `_invoke_matherr` unsatisfied, which
// drags math.c.obj back into the link and duplicates every other symbol here.
#[export_name = "_invoke_matherr"]
pub unsafe extern "C" fn math_error(
    r#type: i32,
    name: *const i8,
    arg1: f64,
    arg2: f64,
    retval: f64,
) -> f64 {
    let mut exception = _exception {
        type_: r#type,
        name: name as *mut i8,
        arg1,
        arg2,
        retval,
    };

    if MSVCRT_default_matherr_func
        .is_some_and(|f| f(core::ptr::addr_of_mut!(exception)) != 0)
    {
        return exception.retval;
    }

    match r#type {
        0 => {
        }
        _DOMAIN => {
            *_errno() = EDOM as i32;
        }
        _SING | _OVERFLOW => {
            *_errno() = ERANGE as i32;
        }
        _UNDERFLOW => {
        }
        _ => {
            ERR!("Unhandled math error!\n");
        }
    }

    exception.retval
}

/// cnt-rust's math exports (its `crt-math-errors` feature) report a domain/pole/range
/// error here: msvcrt's `math_error`, so errno and the user's `_matherr` behave as Wine's.
#[no_mangle]
pub unsafe extern "C" fn __cnt_math_error(
    r#type: i32,
    name: *const i8,
    arg1: f64,
    arg2: f64,
    retval: f64,
) -> f64 {
    // 0x100 marks a report only msvcrt (before msvcr140) makes; this is that msvcrt.
    math_error(r#type & 0xff, name, arg1, arg2, retval)
}

#[no_mangle]
pub unsafe extern "C" fn __setusermatherr(func: MSVCRT_matherr_func) {
    MSVCRT_default_matherr_func = func;
}

#[no_mangle]
pub unsafe extern "C" fn _set_SSE2_enable(flag: i32) -> i32 {
    sse2_enabled = ((flag != 0) && sse2_supported != 0) as BOOL;
    sse2_enabled as i32
}
// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn _chgsignf(num: f32) -> f32 {
    let mut u = core::mem::transmute::<f32, u32>(num);
    u ^= 0x80000000;
    core::mem::transmute::<u32, f32>(u)
}

#[cfg(not(target_arch = "x86"))] // `#ifndef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _fpclassf(num: f32) -> i32 {
    let u = core::mem::transmute::<f32, u32>(num);
    let e = (u >> 23) & 0xff;
    let s = u >> 31;

    match e {
        0 => {
            if u << 1 != 0 {
                if s != 0 { _FPCLASS_ND as i32 } else { _FPCLASS_PD as i32 }
            } else {
                if s != 0 { _FPCLASS_NZ as i32 } else { _FPCLASS_PZ as i32 }
            }
        }
        0xff => {
            if u << 9 != 0 {
                if ((u >> 22) & 1) != 0 {
                    _FPCLASS_QNAN as i32
                } else {
                    _FPCLASS_SNAN as i32
                }
            } else {
                if s != 0 { _FPCLASS_NINF as i32 } else { _FPCLASS_PINF as i32 }
            }
        }
        _ => {
            if s != 0 { _FPCLASS_NN as i32 } else { _FPCLASS_PN as i32 }
        }
    }
}
// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn _finitef(num: f32) -> i32 {
    let u: u32 = core::mem::transmute(num);
    ((u & 0x7fffffff) < 0x7f800000) as i32
}

// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn _isnanf(num: f32) -> i32 {
    let u = core::mem::transmute::<f32, u32>(num);
    if (u & 0x7fffffff) > 0x7f800000 {
        1
    } else {
        0
    }
}

#[cfg(not(target_arch = "x86"))] // `#ifndef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn MSVCRT_atanf(x: f32) -> f32 {
    if isnan(x as f64) != 0 {
        return math_error(_DOMAIN, b"atanf\0".as_ptr() as *const i8, x as f64, 0.0, x as f64) as f32;
    }
    return atanf(x);
}
#[cfg(not(target_arch = "x86"))] // `#ifndef __i386__` in math.c
unsafe extern "C" fn sqrtf_validate(x: *mut f32) -> u8 {
    let c: i16 = _fdclass(*x);

    if c == FP_ZERO as i16 {
        return FALSE as u8;
    }
    if c == FP_NAN as i16 {
        return FALSE as u8;
    }
    if signbitf(*x) != 0 {
        *x = math_error(
            _DOMAIN,
            b"sqrtf\0".as_ptr() as *const i8,
            *x as f64,
            0.0,
            ret_nan(TRUE as u8),
        ) as f32;
        return FALSE as u8;
    }
    if c == FP_INFINITE as i16 {
        return FALSE as u8;
    }
    TRUE as u8
}
#[cfg(any())] // dll/win32/msvcrt/CMakeLists.txt does `-Dasm_sqrtf=sqrtf`, which rewrites every
              // CALL SITE to the libm routine and leaves this hand-written stub defined
              // but unreachable. Nothing references it, so nothing pulls math.c.obj for
              // it either — see the extern block at the top of this file.
unsafe extern "C" fn asm_sqrtf(x: f32) -> f32 {
    let result: f32;
    core::arch::asm!(
        "fsqrt s0, s0",
        inout("s0") x => result,
        options(nostack, preserves_flags)
    );
    result
}

#[cfg(not(target_arch = "x86"))] // `#ifndef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn MSVCRT_sqrtf(mut x: f32) -> f32 {
    #[cfg(not(target_arch = "x86"))]
    {
        if sqrtf_validate(core::ptr::addr_of_mut!(x)) == 0 {
            return x;
        }
        return sqrtf(x); // C: `asm_sqrtf(x)`; CMake -Dasm_sqrtf=sqrtf
    }
    #[cfg(target_arch = "x86")]
    {
        return sqrtf(x);
    }
}

#[cfg(not(target_arch = "x86"))] // `#ifndef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn MSVCRT_tanhf(mut x: f32) -> f32 {
    if isnan(x as f64) != 0 {
        *(core::ptr::addr_of_mut!(x) as *mut u32) |= 0x400000;
        return math_error(_DOMAIN, b"tanhf\0".as_ptr() as PCHAR, x as f64, 0.0, x as f64) as f32;
    }
    return tanhf(x);
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT_asin(x: f64) -> f64 {
    #[cfg(target_arch = "x86")]
    {
        let mut x87_cw: u32 = 0;
        let mut sse2_cw: u32 = 0;
        let hx: u32 = (*(core::ptr::addr_of!(x) as *const u64) >> 32) as u32;
        let ix: u32 = hx & 0x7fffffff;

        if isnan(x) != 0 {
            return math_error(_DOMAIN, b"asin\0".as_ptr() as *const i8, x, 0.0, x);
        }

        if ix < 0x3ff00000 {
            __control87_2(0, 0, core::ptr::addr_of_mut!(x87_cw), core::ptr::addr_of_mut!(sse2_cw));
            if sse2_enabled == 0 || (x87_cw & _MCW_EM) != _MCW_EM
                || (sse2_cw & (_MCW_EM | _MCW_RC)) != _MCW_EM
            {
                return asin(x); // C: `x87_asin(x)`; CMake -Dx87_asin=asin
            }
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        if isnan(x) != 0 {
            return x;
        }
    }

    asin(x)
}
#[no_mangle]
pub unsafe extern "C" fn MSVCRT_atan(x: f64) -> f64 {
    if isnan(x) != 0 {
        return math_error(_DOMAIN, b"atan\0".as_ptr() as PCHAR, x, 0.0, x);
    }
    return atan(x);
}
#[no_mangle]
pub unsafe extern "C" fn MSVCRT_exp(x: f64) -> f64 {
    if isnan(x) != 0 {
        return math_error(
            _DOMAIN,
            b"exp\0".as_ptr() as *const i8,
            x,
            0.0,
            1.0 + x,
        );
    }
    return exp(x);
}
unsafe extern "C" fn sqrt_validate(x: *mut f64, update_sw: u8) -> u8 {
    let c: i16 = _dclass(*x);

    if c == FP_ZERO as i16 {
        return FALSE as u8;
    }
    if c == FP_NAN as i16 {
        #[cfg(target_arch = "x86")]
        {
            if update_sw != 0 {
                *x = math_error(_DOMAIN, b"sqrt\0".as_ptr() as *const i8, *x, 0.0, *x);
            }
        }
        #[cfg(not(target_arch = "x86"))]
        {
            *(x as *mut u64) |= 0x8000000000000u64;
        }
        return FALSE as u8;
    }
    if signbit(*x) != 0 {
        *x = math_error(_DOMAIN, b"sqrt\0".as_ptr() as *const i8, *x, 0.0, ret_nan(update_sw));
        return FALSE as u8;
    }
    if c == FP_INFINITE as i16 {
        return FALSE as u8;
    }
    return TRUE as u8;
}
#[cfg(any())] // dll/win32/msvcrt/CMakeLists.txt does `-Dasm_sqrt=sqrt`, which rewrites every
              // CALL SITE to the libm routine and leaves this hand-written stub defined
              // but unreachable. Nothing references it, so nothing pulls math.c.obj for
              // it either — see the extern block at the top of this file.
unsafe extern "C" fn asm_sqrt(x: f64) -> f64 {
    let result: f64;
    core::arch::asm!(
        "fsqrt d0, d0",
        inlateout("d0") x => result,
        options(nostack, nomem)
    );
    result
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT_sqrt(mut x: f64) -> f64 {
    if sqrt_validate(core::ptr::addr_of_mut!(x), TRUE as u8) == 0 {
        return x;
    }

    return sqrt(x); // C: `asm_sqrt(x)`; CMake -Dasm_sqrt=sqrt
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT_tanh(x: f64) -> f64 {
    let mut x = x;
    if isnan(x) != 0 {
        *(core::ptr::addr_of_mut!(x) as *mut u64) |= 0x0008000000000000u64;
        return math_error(_DOMAIN, b"tanh\0".as_ptr() as *const i8, x, 0.0, x);
    }
    return tanh(x);
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn _fpclass(num: f64) -> i32 {
    let u = core::mem::transmute::<f64, u64>(num);
    let e = ((u >> 52) & 0x7ff) as i32;
    let s = (u >> 63) as i32;

    match e {
        0 => {
            if (u << 1) != 0 {
                if s != 0 { _FPCLASS_ND as i32 } else { _FPCLASS_PD as i32 }
            } else {
                if s != 0 { _FPCLASS_NZ as i32 } else { _FPCLASS_PZ as i32 }
            }
        }
        0x7ff => {
            if (u << 12) != 0 {
                if ((u >> 51) & 1) != 0 {
                    _FPCLASS_QNAN as i32
                } else {
                    _FPCLASS_SNAN as i32
                }
            } else {
                if s != 0 { _FPCLASS_NINF as i32 } else { _FPCLASS_PINF as i32 }
            }
        }
        _ => {
            if s != 0 { _FPCLASS_NN as i32 } else { _FPCLASS_PN as i32 }
        }
    }
}

// The six rotates, and the `& W` on the COUNTERPART shift is not a tidy-up — it is what
// makes `shift == 0` work at all.
//
// The C is `shift &= W-1; return (num << shift) | (num >> (W-shift));`, so at shift == 0
// the second operand is a shift by the full width. C calls that undefined; x86's `shr` and
// arm64's `lsr` both MASK the count to log2(W) bits, so the hardware performs `>> 0` and
// the expression returns `num` — which is the right answer and what msvcrt.dll does.
//
// Rust does not have that undefined behaviour to exploit: `>>` is a CHECKED shift that
// panics in a debug profile and is defined to mask in release, so the literal translation
// is a live abort inside msvcrt.dll for one input per function. Writing the hardware's
// mask out makes the answer identical on every profile and every arch we build, and puts
// the assumption where a reader can see it. (Not `rotate_left`/`rotate_right`, which is the
// same instruction but hides which of these two facts is being relied on.)
//
// Found by the first `_rotl(x, 0)` row, in a plain `cargo test` — `vibeos-cli difftest`
// builds --release, where the check is compiled out and both sides agree. See _abs64 below
// for the negation half of the same class.
#[no_mangle]
pub unsafe extern "C" fn MSVCRT__rotl(num: u32, shift: i32) -> u32 {
    let shift = shift & 31;
    (num << shift) | (num >> ((32 - shift) & 31))
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT__lrotl(num: __msvcrt_ulong, shift: i32) -> __msvcrt_ulong {
    let shift = shift & 0x1f;
    (num << shift) | (num >> ((32 - shift) & 0x1f))
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT__lrotr(num: __msvcrt_ulong, shift: i32) -> __msvcrt_ulong {
    let shift = shift & 0x1f;
    (num >> shift) | (num << ((32 - shift) & 0x1f))
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT__rotr(num: u32, shift: i32) -> u32 {
    let shift = shift & 0x1f;
    (num >> shift) | (num << ((32 - shift) & 0x1f))
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT__rotl64(num: u64, shift: i32) -> u64 {
    let shift = shift & 63;
    (num << shift) | (num >> ((64 - shift) & 63))
}

#[no_mangle]
pub unsafe extern "C" fn MSVCRT__rotr64(num: u64, shift: i32) -> u64 {
    let shift = shift & 63;
    (num >> shift) | (num << ((64 - shift) & 63))
}

// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn abs(n: i32) -> i32 {
    if n >= 0 {
        n
    } else {
        // `wrapping_neg` for the reason `_abs64` below spells out: at the type's minimum
        // the C's `-n` wraps and Rust's `-` panics in a debug profile.
        n.wrapping_neg()
    }
}

// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn labs(n: __msvcrt_long) -> __msvcrt_long {
    if n >= 0 {
        n
    } else {
        // `wrapping_neg` for the reason `_abs64` below spells out: at the type's minimum
        // the C's `-n` wraps and Rust's `-` panics in a debug profile.
        n.wrapping_neg()
    }
}

// `wrapping_neg`, not `-`, and the three of these are one class. The C is
// `n >= 0 ? n : -n`, and at `n == INT64_MIN` that negation has no representable answer:
// C calls it undefined, every compiler emits a `neg` instruction, and the value that
// comes back is INT64_MIN itself. Rust's `-` is not UB — it is a CHECKED negation that
// PANICS in a debug profile — so the faithful translation is the wrapping one, and the
// literal one is a live abort inside msvcrt.dll for a single input.
//
// It got as far as a graded row precisely because `vibeos-cli difftest` builds
// `--release`, where the overflow check is compiled out and the two sides agree. A plain
// `cargo test` in the crate is the debug profile, and that is where it fired. Worth
// remembering when a difftest is green: release-only coverage cannot see this class at
// all, which is the same reason mbcs.rs's `_mbsdec` underflow had to be fixed rather than
// tolerated.
#[no_mangle]
pub unsafe extern "C" fn _abs64(n: i64) -> i64 {
    if n >= 0 {
        n
    } else {
        n.wrapping_neg()
    }
}
// MXCSR only exists on x86 — the C wraps both of these in
// `#if defined(__i386__) || defined(__x86_64__)`. The translation left them ungated and
// spelled the arm64ec arm as `#[cfg(__arm64ec__)]`, a bare cfg FLAG that is never set, so
// the selection happened to work for the wrong reason. `arm64ec` is a real `target_arch`;
// use it, and keep the arm so the file still mirrors the C.
//
// `stmxcsr`/`ldmxcsr` take a MEMORY operand — there is no register form. The C says so
// with GCC's `"=m"`/`"m"` constraints; the translation emitted `out(reg)`/`in(reg)`, which
// either fails to assemble or, worse, assembles against the wrong operand. Rust's `asm!`
// has no `m` constraint, so the shape is a register holding the ADDRESS, dereferenced in
// the template. AT&T syntax here so the instruction text is the C's, character for
// character.
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm64ec"))]
unsafe extern "C" fn get_mxcsr() -> u32 {
    // The C leaves `ret` uninitialized; `stmxcsr` writes all four bytes, so seeding it is
    // only for rustc's definite-initialization analysis, which cannot see through asm.
    let mut ret: u32 = 0;
    #[cfg(target_arch = "arm64ec")]
    {
        __os_arm64x_get_x64_information(0, core::ptr::addr_of_mut!(ret) as *mut core::ffi::c_void, core::ptr::null_mut());
    }
    #[cfg(not(target_arch = "arm64ec"))]
    {
        core::arch::asm!(
            "stmxcsr ({})",
            in(reg) core::ptr::addr_of_mut!(ret),
            options(att_syntax, nostack)
        );
    }
    ret
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm64ec"))]
unsafe extern "C" fn set_mxcsr(val: u32) {
    #[cfg(target_arch = "arm64ec")]
    {
        __os_arm64x_set_x64_information(0, val as ULONG_PTR, core::ptr::null_mut());
    }
    #[cfg(not(target_arch = "arm64ec"))]
    {
        core::arch::asm!(
            "ldmxcsr ({})",
            in(reg) core::ptr::addr_of!(val),
            options(att_syntax, nostack)
        );
    }
}
unsafe extern "C" fn _setfp_sse(
    cw: *mut u32,
    mut cw_mask: u32,
    sw: *mut u32,
    mut sw_mask: u32,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let mut fpword: u32 = get_mxcsr();
        let old_fpword: u32 = fpword;
        let mut flags: u32;

        cw_mask &= _MCW_EM | _MCW_RC | _MCW_DN;
        sw_mask &= _MCW_EM;

        if !sw.is_null() {
            flags = 0;
            if fpword & 0x1 != 0 {
                flags |= _SW_INVALID;
            }
            if fpword & 0x2 != 0 {
                flags |= _SW_DENORMAL;
            }
            if fpword & 0x4 != 0 {
                flags |= _SW_ZERODIVIDE;
            }
            if fpword & 0x8 != 0 {
                flags |= _SW_OVERFLOW;
            }
            if fpword & 0x10 != 0 {
                flags |= _SW_UNDERFLOW;
            }
            if fpword & 0x20 != 0 {
                flags |= _SW_INEXACT;
            }

            *sw = (flags & !sw_mask) | (*sw & sw_mask);
            TRACE!(b"sse2 update sw %08x to %08x\n\0".as_ptr() as *const i8, flags, *sw);

            fpword &= !0x3f;
            if *sw & _SW_INVALID != 0 {
                fpword |= 0x1;
            }
            if *sw & _SW_DENORMAL != 0 {
                fpword |= 0x2;
            }
            if *sw & _SW_ZERODIVIDE != 0 {
                fpword |= 0x4;
            }
            if *sw & _SW_OVERFLOW != 0 {
                fpword |= 0x8;
            }
            if *sw & _SW_UNDERFLOW != 0 {
                fpword |= 0x10;
            }
            if *sw & _SW_INEXACT != 0 {
                fpword |= 0x20;
            }
            *sw = flags;
        }

        if !cw.is_null() {
            flags = 0;
            if fpword & 0x80 != 0 {
                flags |= _EM_INVALID;
            }
            if fpword & 0x100 != 0 {
                flags |= _EM_DENORMAL;
            }
            if fpword & 0x200 != 0 {
                flags |= _EM_ZERODIVIDE;
            }
            if fpword & 0x400 != 0 {
                flags |= _EM_OVERFLOW;
            }
            if fpword & 0x800 != 0 {
                flags |= _EM_UNDERFLOW;
            }
            if fpword & 0x1000 != 0 {
                flags |= _EM_INEXACT;
            }
            match fpword & 0x6000 {
                0x6000 => {
                    flags |= _RC_UP | _RC_DOWN;
                }
                0x4000 => {
                    flags |= _RC_UP;
                }
                0x2000 => {
                    flags |= _RC_DOWN;
                }
                _ => {}
            }
            match fpword & 0x8040 {
                0x0040 => {
                    flags |= _DN_FLUSH_OPERANDS_SAVE_RESULTS;
                }
                0x8000 => {
                    flags |= _DN_SAVE_OPERANDS_FLUSH_RESULTS;
                }
                0x8040 => {
                    flags |= _DN_FLUSH;
                }
                _ => {}
            }

            *cw = (flags & !cw_mask) | (*cw & cw_mask);
            TRACE!(b"sse2 update cw %08x to %08x\n\0".as_ptr() as *const i8, flags, *cw);

            fpword &= !0xffc0;
            if *cw & _EM_INVALID != 0 {
                fpword |= 0x80;
            }
            if *cw & _EM_DENORMAL != 0 {
                fpword |= 0x100;
            }
            if *cw & _EM_ZERODIVIDE != 0 {
                fpword |= 0x200;
            }
            if *cw & _EM_OVERFLOW != 0 {
                fpword |= 0x400;
            }
            if *cw & _EM_UNDERFLOW != 0 {
                fpword |= 0x800;
            }
            if *cw & _EM_INEXACT != 0 {
                fpword |= 0x1000;
            }
            match *cw & _MCW_RC {
                val if val == _RC_UP | _RC_DOWN => {
                    fpword |= 0x6000;
                }
                _RC_UP => {
                    fpword |= 0x4000;
                }
                _RC_DOWN => {
                    fpword |= 0x2000;
                }
                _ => {}
            }
            match *cw & _MCW_DN {
                _DN_FLUSH_OPERANDS_SAVE_RESULTS => {
                    fpword |= 0x0040;
                }
                _DN_SAVE_OPERANDS_FLUSH_RESULTS => {
                    fpword |= 0x8000;
                }
                _DN_FLUSH => {
                    fpword |= 0x8040;
                }
                _ => {}
            }

            if fpword != old_fpword && sw.is_null() {
                fpword &= !0x3f;
            }
        }

        if fpword != old_fpword {
            set_mxcsr(fpword);
        }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        if !cw.is_null() {
            *cw = 0;
        }
        if !sw.is_null() {
            *sw = 0;
        }
    }
}
unsafe extern "C" fn _setfp(
    cw: *mut u32,
    mut cw_mask: u32,
    sw: *mut u32,
    mut sw_mask: u32,
) {
    #[cfg(target_arch = "x86")]
    {
        let mut oldcw: u32 = 0;
        let mut newcw: u32 = 0;
        let mut oldsw: u32 = 0;
        let mut newsw: u32 = 0;
        let mut flags: u32;

        cw_mask &= _MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC;
        sw_mask &= _MCW_EM;

        if !sw.is_null() {
            // `"fstsw %0" : "=m"(newsw)`. Memory form only, and it writes TWO bytes
            // into a four-byte slot — `newsw` is pre-zeroed above exactly as the C
            // declares it `= 0`, so the high half stays clear.
            core::arch::asm!(
                "fstsw ({})",
                in(reg) core::ptr::addr_of_mut!(newsw),
                options(att_syntax, nostack)
            );
            oldsw = newsw;

            flags = 0;
            if newsw & 0x1 != 0 { flags |= _SW_INVALID; }
            if newsw & 0x2 != 0 { flags |= _SW_DENORMAL; }
            if newsw & 0x4 != 0 { flags |= _SW_ZERODIVIDE; }
            if newsw & 0x8 != 0 { flags |= _SW_OVERFLOW; }
            if newsw & 0x10 != 0 { flags |= _SW_UNDERFLOW; }
            if newsw & 0x20 != 0 { flags |= _SW_INEXACT; }

            *sw = (flags & !sw_mask) | (*sw & sw_mask);
            TRACE!(b"x86 update sw %08x to %08x\n\0".as_ptr() as *const i8, flags, *sw);

            newsw &= !0x3f;
            if *sw & _SW_INVALID != 0 { newsw |= 0x1; }
            if *sw & _SW_DENORMAL != 0 { newsw |= 0x2; }
            if *sw & _SW_ZERODIVIDE != 0 { newsw |= 0x4; }
            if *sw & _SW_OVERFLOW != 0 { newsw |= 0x8; }
            if *sw & _SW_UNDERFLOW != 0 { newsw |= 0x10; }
            if *sw & _SW_INEXACT != 0 { newsw |= 0x20; }
            *sw = flags;
        }

        if !cw.is_null() {
            // `"fstcw %0" : "=m"(newcw)` — memory form only, two bytes written.
            core::arch::asm!(
                "fstcw ({})",
                in(reg) core::ptr::addr_of_mut!(newcw),
                options(att_syntax, nostack)
            );
            oldcw = newcw;

            flags = 0;
            if newcw & 0x1 != 0 { flags |= _EM_INVALID; }
            if newcw & 0x2 != 0 { flags |= _EM_DENORMAL; }
            if newcw & 0x4 != 0 { flags |= _EM_ZERODIVIDE; }
            if newcw & 0x8 != 0 { flags |= _EM_OVERFLOW; }
            if newcw & 0x10 != 0 { flags |= _EM_UNDERFLOW; }
            if newcw & 0x20 != 0 { flags |= _EM_INEXACT; }
            match newcw & 0xc00 {
                0xc00 => flags |= _RC_UP | _RC_DOWN,
                0x800 => flags |= _RC_UP,
                0x400 => flags |= _RC_DOWN,
                _ => {}
            }
            match newcw & 0x300 {
                0x0 => flags |= _PC_24,
                0x200 => flags |= _PC_53,
                0x300 => flags |= _PC_64,
                _ => {}
            }
            if newcw & 0x1000 != 0 { flags |= _IC_AFFINE; }

            *cw = (flags & !cw_mask) | (*cw & cw_mask);
            TRACE!(b"x86 update cw %08x to %08x\n\0".as_ptr() as *const i8, flags, *cw);

            newcw &= !0x1f3f;
            if *cw & _EM_INVALID != 0 { newcw |= 0x1; }
            if *cw & _EM_DENORMAL != 0 { newcw |= 0x2; }
            if *cw & _EM_ZERODIVIDE != 0 { newcw |= 0x4; }
            if *cw & _EM_OVERFLOW != 0 { newcw |= 0x8; }
            if *cw & _EM_UNDERFLOW != 0 { newcw |= 0x10; }
            if *cw & _EM_INEXACT != 0 { newcw |= 0x20; }
            match *cw & _MCW_RC {
                x if x == _RC_UP | _RC_DOWN => newcw |= 0xc00,
                _RC_UP => newcw |= 0x800,
                _RC_DOWN => newcw |= 0x400,
                _ => {}
            }
            match *cw & _MCW_PC {
                _PC_64 => newcw |= 0x300,
                _PC_53 => newcw |= 0x200,
                _PC_24 => newcw |= 0x0,
                _ => {}
            }
            if *cw & _IC_AFFINE != 0 { newcw |= 0x1000; }
        }

        if oldsw != newsw && (newsw & 0x3f) != 0 {
            #[repr(C)]
            struct Fenv {
                control_word: u16,
                unused1: u16,
                status_word: u16,
                unused2: u16,
                tag_word: u16,
                unused3: u16,
                instruction_pointer: u32,
                code_segment: u16,
                unused4: u16,
                operand_addr: u32,
                data_segment: u16,
                unused5: u16,
            }
            let mut fenv: Fenv = core::mem::zeroed();

            assert!(!cw.is_null());

            // `"fnstenv %0" : "=m"(fenv)` — a 28-byte STORE. The translation had it as
            // an `in(reg)` with a register operand: wrong direction and wrong form.
            core::arch::asm!(
                "fnstenv ({})",
                in(reg) core::ptr::addr_of_mut!(fenv),
                options(att_syntax, nostack)
            );
            fenv.control_word = newcw as u16;
            fenv.status_word = newsw as u16;
            // `"fldenv %0" : : "m"(fenv) : "st", "st(1)" ... "st(7)"` — a 28-byte LOAD that
            // reloads the whole x87 environment, so the C names every stack register as a
            // clobber. Rust's x86 `asm!` has no `st` register class and rejects those
            // operands outright, so the clobber list cannot be transcribed. It is not
            // needed here for the same reason it is harmless in the C: this function is
            // integer-only, holds no float in a local, and the enclosing `_setfp` returns
            // immediately after — there is no live x87 value for the reload to destroy.
            // Do NOT copy this pattern into a function that computes with floats.
            core::arch::asm!(
                "fldenv ({})",
                in(reg) core::ptr::addr_of!(fenv),
                options(att_syntax, nostack)
            );
            return;
        }

        if oldsw != newsw {
            core::arch::asm!("fnclex", options(att_syntax, nostack));
        }
        if oldcw != newcw {
            // `"fldcw %0" : : "m"(newcw)` — memory form only.
            core::arch::asm!(
                "fldcw ({})",
                in(reg) core::ptr::addr_of!(newcw),
                options(att_syntax, nostack)
            );
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        _setfp_sse(cw, cw_mask, sw, sw_mask);
    }

    #[cfg(target_arch = "aarch64")]
    {
        let mut old_fpsr: u64 = 0;
        let mut fpsr: u64 = 0;
        let mut old_fpcr: u64 = 0;
        let mut fpcr: u64 = 0;
        let mut flags: u32;

        cw_mask &= _MCW_EM | _MCW_RC;
        sw_mask &= _MCW_EM;

        if !sw.is_null() {
            core::arch::asm!("mrs {}, fpsr", out(reg) fpsr);
            old_fpsr = fpsr;

            flags = 0;
            if fpsr & 0x1 != 0 { flags |= _SW_INVALID; }
            if fpsr & 0x2 != 0 { flags |= _SW_ZERODIVIDE; }
            if fpsr & 0x4 != 0 { flags |= _SW_OVERFLOW; }
            if fpsr & 0x8 != 0 { flags |= _SW_UNDERFLOW; }
            if fpsr & 0x10 != 0 { flags |= _SW_INEXACT; }
            if fpsr & 0x80 != 0 { flags |= _SW_DENORMAL; }

            *sw = (flags & !sw_mask) | (*sw & sw_mask);
            TRACE!(b"aarch64 update sw %08x to %08x\n\0".as_ptr() as *const i8, flags, *sw);

            fpsr &= !0x9f;
            if *sw & _SW_INVALID != 0 { fpsr |= 0x1; }
            if *sw & _SW_ZERODIVIDE != 0 { fpsr |= 0x2; }
            if *sw & _SW_OVERFLOW != 0 { fpsr |= 0x4; }
            if *sw & _SW_UNDERFLOW != 0 { fpsr |= 0x8; }
            if *sw & _SW_INEXACT != 0 { fpsr |= 0x10; }
            if *sw & _SW_DENORMAL != 0 { fpsr |= 0x80; }
            *sw = flags;
        }

        if !cw.is_null() {
            core::arch::asm!("mrs {}, fpcr", out(reg) fpcr);
            old_fpcr = fpcr;

            flags = 0;
            if fpcr & 0x100 == 0 { flags |= _EM_INVALID; }
            if fpcr & 0x200 == 0 { flags |= _EM_ZERODIVIDE; }
            if fpcr & 0x400 == 0 { flags |= _EM_OVERFLOW; }
            if fpcr & 0x800 == 0 { flags |= _EM_UNDERFLOW; }
            if fpcr & 0x1000 == 0 { flags |= _EM_INEXACT; }
            if fpcr & 0x8000 == 0 { flags |= _EM_DENORMAL; }
            match fpcr & 0xc00000 {
                0x400000 => flags |= _RC_UP,
                0x800000 => flags |= _RC_DOWN,
                0xc00000 => flags |= _RC_CHOP,
                _ => {}
            }

            *cw = (flags & !cw_mask) | (*cw & cw_mask);
            TRACE!(b"aarch64 update cw %08x to %08x\n\0".as_ptr() as *const i8, flags, *cw);

            fpcr &= !0xc09f00;
            if *cw & _EM_INVALID == 0 { fpcr |= 0x100; }
            if *cw & _EM_ZERODIVIDE == 0 { fpcr |= 0x200; }
            if *cw & _EM_OVERFLOW == 0 { fpcr |= 0x400; }
            if *cw & _EM_UNDERFLOW == 0 { fpcr |= 0x800; }
            if *cw & _EM_INEXACT == 0 { fpcr |= 0x1000; }
            if *cw & _EM_DENORMAL == 0 { fpcr |= 0x8000; }
            match *cw & _MCW_RC {
                _RC_CHOP => fpcr |= 0xc00000,
                _RC_UP => fpcr |= 0x400000,
                _RC_DOWN => fpcr |= 0x800000,
                _ => {}
            }
        }

        // The C is `if (old_fpcr != fpcr && ~(old_fpcr >> 8) & fpsr & 0x9f != fpsr & 0x9f)`
        // and it does NOT mean what it reads like. In C, `!=` binds TIGHTER than `&`, so the
        // right operand parses as
        //     ~(old_fpcr >> 8) & fpsr & (0x9f != fpsr) & 0x9f
        // — a chain whose third term is the 0-or-1 result of a comparison, which masks the
        // whole expression down to bit 0. Rust's `&` binds tighter than `!=`, so copying the
        // text across gives the "obviously intended" masked comparison instead, and that
        // silently changes which FP exceptions get masked on arm64. Parenthesised here to
        // reproduce C's grammar exactly. (It looks like an upstream Wine bug; translating it
        // faithfully is still the job — fixing it belongs in a patch that says so.)
        if old_fpcr != fpcr
            && (!(old_fpcr >> 8) & fpsr & ((0x9f != fpsr) as ULONG_PTR) & 0x9f) != 0
        {
            let mask = fpcr & !0x9f00;
            core::arch::asm!("msr fpcr, {}", in(reg) mask);
        }

        if old_fpsr != fpsr {
            core::arch::asm!("msr fpsr, {}", in(reg) fpsr);
        }
        if old_fpcr != fpcr {
            core::arch::asm!("msr fpcr, {}", in(reg) fpcr);
        }
    }

    #[cfg(target_arch = "arm")]
    {
        let mut old_fpscr: u32;
        let mut fpscr: u32;
        let mut flags: u32;

        core::arch::asm!("vmrs {}, fpscr", out(reg) fpscr);
        old_fpscr = fpscr;

        cw_mask &= _MCW_EM | _MCW_RC;
        sw_mask &= _MCW_EM;

        if !sw.is_null() {
            flags = 0;
            if fpscr & 0x1 != 0 { flags |= _SW_INVALID; }
            if fpscr & 0x2 != 0 { flags |= _SW_ZERODIVIDE; }
            if fpscr & 0x4 != 0 { flags |= _SW_OVERFLOW; }
            if fpscr & 0x8 != 0 { flags |= _SW_UNDERFLOW; }
            if fpscr & 0x10 != 0 { flags |= _SW_INEXACT; }
            if fpscr & 0x80 != 0 { flags |= _SW_DENORMAL; }

            *sw = (flags & !sw_mask) | (*sw & sw_mask);
            TRACE!(b"arm update sw %08x to %08x\n\0".as_ptr() as *const i8, flags, *sw);

            fpscr &= !0x9f;
            if *sw & _SW_INVALID != 0 { fpscr |= 0x1; }
            if *sw & _SW_ZERODIVIDE != 0 { fpscr |= 0x2; }
            if *sw & _SW_OVERFLOW != 0 { fpscr |= 0x4; }
            if *sw & _SW_UNDERFLOW != 0 { fpscr |= 0x8; }
            if *sw & _SW_INEXACT != 0 { fpscr |= 0x10; }
            if *sw & _SW_DENORMAL != 0 { fpscr |= 0x80; }
            *sw = flags;
        }

        if !cw.is_null() {
            flags = 0;
            if fpscr & 0x100 == 0 { flags |= _EM_INVALID; }
            if fpscr & 0x200 == 0 { flags |= _EM_ZERODIVIDE; }
            if fpscr & 0x400 == 0 { flags |= _EM_OVERFLOW; }
            if fpscr & 0x800 == 0 { flags |= _EM_UNDERFLOW; }
            if fpscr & 0x1000 == 0 { flags |= _EM_INEXACT; }
            if fpscr & 0x8000 == 0 { flags |= _EM_DENORMAL; }
            match fpscr & 0xc00000 {
                0x400000 => flags |= _RC_UP,
                0x800000 => flags |= _RC_DOWN,
                0xc00000 => flags |= _RC_CHOP,
                _ => {}
            }

            *cw = (flags & !cw_mask) | (*cw & cw_mask);
            TRACE!(b"arm update cw %08x to %08x\n\0".as_ptr() as *const i8, flags, *cw);

            fpscr &= !0xc09f00;
            if *cw & _EM_INVALID == 0 { fpscr |= 0x100; }
            if *cw & _EM_ZERODIVIDE == 0 { fpscr |= 0x200; }
            if *cw & _EM_OVERFLOW == 0 { fpscr |= 0x400; }
            if *cw & _EM_UNDERFLOW == 0 { fpscr |= 0x800; }
            if *cw & _EM_INEXACT == 0 { fpscr |= 0x1000; }
            if *cw & _EM_DENORMAL == 0 { fpscr |= 0x8000; }
            match *cw & _MCW_RC {
                _RC_CHOP => fpscr |= 0xc00000,
                _RC_UP => fpscr |= 0x400000,
                _RC_DOWN => fpscr |= 0x800000,
                _ => {}
            }
        }

        if old_fpscr != fpscr {
            core::arch::asm!("vmsr fpscr, {}", in(reg) fpscr);
        }
    }

    #[cfg(not(any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "arm"
    )))]
    {
        if !cw.is_null() { *cw = 0; }
        if !sw.is_null() { *sw = 0; }
    }
}
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _statusfp2(x86_sw: *mut u32, sse2_sw: *mut u32) {
    if !x86_sw.is_null() {
        _setfp(core::ptr::null_mut(), 0, x86_sw, 0);
    }
    if sse2_sw.is_null() {
        return;
    }
    if sse2_supported != 0 {
        _setfp_sse(core::ptr::null_mut(), 0, sse2_sw, 0);
    } else {
        *sse2_sw = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn _statusfp() -> u32 {
    // The C's `flags = 0` is overwritten on x86 but handed to `_setfp` as in/out on the
    // other arches, so each arm owns its own initializer.
    let flags: u32;
    #[cfg(target_arch = "x86")]
    {
        let mut x86_sw: u32 = 0;
        let mut sse2_sw: u32 = 0;

        _statusfp2(&mut x86_sw, &mut sse2_sw);
        flags = x86_sw | sse2_sw;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let mut sw: u32 = 0;
        _setfp(
            core::ptr::null_mut(),
            0,
            &mut sw,
            0,
        );
        flags = sw;
    }
    flags
}

#[no_mangle]
pub unsafe extern "C" fn _clearfp() -> u32 {
    let mut flags = 0;
#[cfg(target_arch = "x86")]
    {
        _setfp(core::ptr::null_mut(), 0, &mut flags, _MCW_EM);
        if sse2_supported != 0 {
            let mut sse_sw = 0;

            _setfp_sse(core::ptr::null_mut(), 0, &mut sse_sw, _MCW_EM);
            flags |= sse_sw;
        }
    }
#[cfg(not(target_arch = "x86"))]
    {
        _setfp(core::ptr::null_mut(), 0, &mut flags, _MCW_EM);
    }
    flags
}

#[no_mangle]
pub unsafe extern "C" fn __fpecode() -> *mut i32 {
    core::ptr::addr_of_mut!((*msvcrt_get_thread_data()).fpecode)
}

// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn ldexp(num: f64, exp: i32) -> f64 {
    let z: f64 = scalbn(num, exp);

    if isfinite(num) != 0 && isfinite(z) == 0 {
        return math_error(_OVERFLOW, b"ldexp\0".as_ptr() as *const i8, num, exp as f64, z);
    }
    if num != 0.0 && isfinite(num) != 0 && z == 0.0 {
        return math_error(_UNDERFLOW, b"ldexp\0".as_ptr() as *const i8, num, exp as f64, z);
    }
    z
}

#[no_mangle]
pub unsafe extern "C" fn _cabs(num: _complex) -> f64 {
    sqrt(num.x * num.x + num.y * num.y)
}

#[no_mangle]
pub unsafe extern "C" fn _chgsign(num: f64) -> f64 {
    let mut u: UINT64 = core::mem::transmute(num);
    u ^= 1u64 << 63;
    core::mem::transmute(u)
}
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn __control87_2(
    newval: u32,
    mask: u32,
    x86_cw: *mut u32,
    sse2_cw: *mut u32,
) -> i32 {
    if !x86_cw.is_null() {
        *x86_cw = newval;
        _setfp(x86_cw, mask, core::ptr::null_mut(), 0);
    }

    if sse2_cw.is_null() {
        return 1;
    }

    if sse2_supported != 0 {
        *sse2_cw = newval;
        _setfp_sse(sse2_cw, mask, core::ptr::null_mut(), 0);
    } else {
        *sse2_cw = 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn _control87(newval: u32, mask: u32) -> u32 {
    // The C's `flags = 0` is overwritten by `flags = newval` off x86 but handed to
    // `__control87_2` on x86, so each arm owns its own initializer.
    let flags: u32;
    // C: `#if defined(__i386__) && (_MSVCR_VER == 0 || _MSVCR_VER >= 80)` — TRUE at 0.
    #[cfg(target_arch = "x86")]
    {
        let mut x86_cw: u32 = 0;
        let mut sse2_cw: u32 = 0;

        __control87_2(newval, mask, &mut x86_cw, &mut sse2_cw);

        if sse2_supported != 0 {
            if ((x86_cw ^ sse2_cw) & (_MCW_EM | _MCW_RC)) != 0 {
                x86_cw |= _EM_AMBIGUOUS;
            }
            x86_cw |= sse2_cw;
        }
        flags = x86_cw;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let mut cw: u32 = newval;
        _setfp(&mut cw, mask, core::ptr::null_mut(), 0);
        flags = cw;
    }
    flags
}

#[no_mangle]
pub unsafe extern "C" fn _controlfp(newval: u32, mask: u32) -> u32 {
    _control87(newval, mask & !_EM_DENORMAL)
}

#[no_mangle]
pub unsafe extern "C" fn _set_controlfp(newval: u32, mask: u32) {
    _controlfp(newval, mask);
}

#[no_mangle]
pub unsafe extern "C" fn _controlfp_s(cur: *mut u32, newval: u32, mask: u32) -> i32 {
    const all_flags: u32 = (_MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC | _MCW_DN);
    let val: u32;

    if MSVCRT_CHECK_PMT!((newval & mask & !all_flags) == 0) == 0 {
        if !cur.is_null() {
            *cur = _controlfp(0, 0);
        }
        return EINVAL as i32;
    }
    val = _controlfp(newval, mask);
    if !cur.is_null() {
        *cur = val;
    }
    return 0;
}
#[cfg(any())] // `#if _MSVCR_VER>=120/140` in math.c; msvcrt builds at _MSVCR_VER == 0,
              // so neither variant is compiled. Kept for the ucrtbase share of this file.
unsafe extern "C" fn fenv_encode(mut x: u32, mut y: u32) -> __msvcrt_ulong {
    let mut ret: __msvcrt_ulong = 0;

    #[cfg(target_arch = "x86")]
    {
        if x & _EM_INVALID != 0 { ret |= FENV_X_INVALID; }
        if x & _EM_DENORMAL != 0 { ret |= FENV_X_DENORMAL; }
        if x & _EM_ZERODIVIDE != 0 { ret |= FENV_X_ZERODIVIDE; }
        if x & _EM_OVERFLOW != 0 { ret |= FENV_X_OVERFLOW; }
        if x & _EM_UNDERFLOW != 0 { ret |= FENV_X_UNDERFLOW; }
        if x & _EM_INEXACT != 0 { ret |= FENV_X_INEXACT; }
        if x & _IC_AFFINE != 0 { ret |= FENV_X_AFFINE; }
        if x & _RC_UP != 0 { ret |= FENV_X_UP; }
        if x & _RC_DOWN != 0 { ret |= FENV_X_DOWN; }
        if x & _PC_24 != 0 { ret |= FENV_X_24; }
        if x & _PC_53 != 0 { ret |= FENV_X_53; }
    }
    x &= !(_MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC);

    if y & _EM_INVALID != 0 { ret |= FENV_Y_INVALID; }
    if y & _EM_DENORMAL != 0 { ret |= FENV_Y_DENORMAL; }
    if y & _EM_ZERODIVIDE != 0 { ret |= FENV_Y_ZERODIVIDE; }
    if y & _EM_OVERFLOW != 0 { ret |= FENV_Y_OVERFLOW; }
    if y & _EM_UNDERFLOW != 0 { ret |= FENV_Y_UNDERFLOW; }
    if y & _EM_INEXACT != 0 { ret |= FENV_Y_INEXACT; }
    if y & _RC_UP != 0 { ret |= FENV_Y_UP; }
    if y & _RC_DOWN != 0 { ret |= FENV_Y_DOWN; }
    if y & _DN_FLUSH != 0 { ret |= FENV_Y_FLUSH; }
    if y & _DN_FLUSH_OPERANDS_SAVE_RESULTS != 0 { ret |= FENV_Y_FLUSH_SAVE; }
    y &= !(_MCW_EM | _MCW_IC | _MCW_RC | _MCW_DN);

    if x != 0 || y != 0 {
        FIXME!("unsupported flags: %x, %x\n", x, y);
    }
    ret
}
#[cfg(any())] // `#if _MSVCR_VER>=120/140` in math.c; msvcrt builds at _MSVCR_VER == 0,
              // so neither variant is compiled. Kept for the ucrtbase share of this file.
unsafe extern "C" fn fenv_decode(enc: __msvcrt_ulong, x: *mut u32, y: *mut u32) -> u8 {
    *x = 0;
    *y = 0;
    if (enc & FENV_X_INVALID) == FENV_X_INVALID { *x |= _EM_INVALID; }
    if (enc & FENV_X_DENORMAL) == FENV_X_DENORMAL { *x |= _EM_DENORMAL; }
    if (enc & FENV_X_ZERODIVIDE) == FENV_X_ZERODIVIDE { *x |= _EM_ZERODIVIDE; }
    if (enc & FENV_X_OVERFLOW) == FENV_X_OVERFLOW { *x |= _EM_OVERFLOW; }
    if (enc & FENV_X_UNDERFLOW) == FENV_X_UNDERFLOW { *x |= _EM_UNDERFLOW; }
    if (enc & FENV_X_INEXACT) == FENV_X_INEXACT { *x |= _EM_INEXACT; }
    if (enc & FENV_X_AFFINE) == FENV_X_AFFINE { *x |= _IC_AFFINE; }
    if (enc & FENV_X_UP) == FENV_X_UP { *x |= _RC_UP; }
    if (enc & FENV_X_DOWN) == FENV_X_DOWN { *x |= _RC_DOWN; }
    if (enc & FENV_X_24) == FENV_X_24 { *x |= _PC_24; }
    if (enc & FENV_X_53) == FENV_X_53 { *x |= _PC_53; }

    if (enc & FENV_Y_INVALID) == FENV_Y_INVALID { *y |= _EM_INVALID; }
    if (enc & FENV_Y_DENORMAL) == FENV_Y_DENORMAL { *y |= _EM_DENORMAL; }
    if (enc & FENV_Y_ZERODIVIDE) == FENV_Y_ZERODIVIDE { *y |= _EM_ZERODIVIDE; }
    if (enc & FENV_Y_OVERFLOW) == FENV_Y_OVERFLOW { *y |= _EM_OVERFLOW; }
    if (enc & FENV_Y_UNDERFLOW) == FENV_Y_UNDERFLOW { *y |= _EM_UNDERFLOW; }
    if (enc & FENV_Y_INEXACT) == FENV_Y_INEXACT { *y |= _EM_INEXACT; }
    if (enc & FENV_Y_UP) == FENV_Y_UP { *y |= _RC_UP; }
    if (enc & FENV_Y_DOWN) == FENV_Y_DOWN { *y |= _RC_DOWN; }
    if (enc & FENV_Y_FLUSH) == FENV_Y_FLUSH { *y |= _DN_FLUSH; }
    if (enc & FENV_Y_FLUSH_SAVE) == FENV_Y_FLUSH_SAVE { *y |= _DN_FLUSH_OPERANDS_SAVE_RESULTS; }

    if fenv_encode(*x, *y) != enc {
        return FALSE;
    }
    return TRUE;
}

// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn _finite(num: f64) -> i32 {
    let u: u64 = core::mem::transmute(num);
    ((u & !0u64 >> 1) < (0x7ffu64 << 52)) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _fpreset() {
#[cfg(target_arch = "x86")]
{
    let x86_cw: u32 = 0x27f;
    // `"fninit; fldcw %0" : : "m"(x86_cw)` — the `({0})` dereference is AT&T syntax, so
    // it needs `options(att_syntax)`; without it the default Intel parser reads the
    // parentheses as something else entirely.
    core::arch::asm!(
        "fninit; fldcw ({0})",
        in(reg) core::ptr::addr_of!(x86_cw),
        options(att_syntax, nostack)
    );
    if sse2_supported != 0 {
        let mut cw: u32 = _MCW_EM;
        let mut sw: u32 = 0;
        _setfp_sse(&mut cw, !0, &mut sw, !0);
    }
}
#[cfg(not(target_arch = "x86"))]
{
    let mut cw: u32 = _MCW_EM;
    let mut sw: u32 = 0;
    _setfp(&mut cw, !0, &mut sw, !0);
}
}

// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn _isnan(num: f64) -> i32 {
    let u = core::mem::transmute::<f64, u64>(num);
    ((u & !0u64 >> 1) > 0x7ffu64 << 52) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _ecvt(
    number: f64,
    ndigits: i32,
    decpt: *mut i32,
    sign: *mut i32,
) -> *mut i8 {
    let mut prec: i32;
    let mut len: i32;
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    let ndigits = min!(ndigits, 80 - 8);

    if (*data).efcvt_buffer.is_null() {
        (*data).efcvt_buffer = malloc(80) as *mut i8;
    }

    prec = ndigits;
    if prec < 1 {
        prec = 2;
    }
    len = _snprintf(
        (*data).efcvt_buffer,
        80,
        b"%.*le\0".as_ptr() as *const i8,
        prec - 1,
        number,
    );

    if *(*data).efcvt_buffer.offset(0 as isize) == b'-' as i8 {
        memmove(
            (*data).efcvt_buffer as *mut c_void,
            (*data).efcvt_buffer.offset(1 as isize) as *const c_void,
            len as usize,
        );
        len -= 1;
        *sign = 1;
    } else {
        *sign = 0;
    }

    if prec != 1 {
        memmove(
            (*data).efcvt_buffer.offset(1 as isize) as *mut c_void,
            (*data).efcvt_buffer.offset(2 as isize) as *const c_void,
            (len - 1) as usize,
        );
    }

    *(*data).efcvt_buffer.offset(prec as isize) = b'\0' as i8;

    sscanf(
        (*data).efcvt_buffer.offset((prec + 1) as isize),
        b"%d\0".as_ptr() as *const i8,
        decpt,
    );
    *decpt += 1;

    if *(*data).efcvt_buffer.offset(0 as isize) == b'0' as i8 {
        *decpt = 0;
    }

    if ndigits < 1 {
        if *(*data).efcvt_buffer.offset(0 as isize) >= b'5' as i8 {
            *decpt += 1;
        }
        *(*data).efcvt_buffer.offset(0 as isize) = b'\0' as i8;
    }

    (*data).efcvt_buffer
}

#[no_mangle]
pub unsafe extern "C" fn _ecvt_s(
    buffer: *mut i8,
    length: usize,
    number: f64,
    ndigits: i32,
    decpt: *mut i32,
    sign: *mut i32,
) -> i32 {
    let mut prec: i32;
    let mut len: i32;
    let result: *mut i8;

    if MSVCRT_CHECK_PMT!(!buffer.is_null()) == 0 {
        return EINVAL as i32;
    }
    if MSVCRT_CHECK_PMT!(!decpt.is_null()) == 0 {
        return EINVAL as i32;
    }
    if MSVCRT_CHECK_PMT!(!sign.is_null()) == 0 {
        return EINVAL as i32;
    }
    if MSVCRT_CHECK_PMT_ERR!(length > 2, ERANGE as i32) == 0 {
        return ERANGE as i32;
    }
    if MSVCRT_CHECK_PMT_ERR!(
        ndigits < (length as i32) - 1,
        ERANGE as i32,
    ) == 0
    {
        return ERANGE as i32;
    }

    prec = ndigits;
    if prec < 1 {
        prec = 2;
    }
    result = malloc((prec + 8) as usize) as *mut i8;

    len = _snprintf(
        result,
        (prec + 8) as usize,
        b"%.*le\0".as_ptr() as *const i8,
        prec - 1,
        number,
    );
    if *result.offset(0 as isize) == b'-' as i8 {
        memmove(
            result as *mut c_void,
            result.offset(1 as isize) as *const c_void,
            len as usize,
        );
        len -= 1;
        *sign = 1;
    } else {
        *sign = 0;
    }

    if prec != 1 {
        memmove(
            result.offset(1 as isize) as *mut c_void,
            result.offset(2 as isize) as *const c_void,
            (len - 1) as usize,
        );
    }
    *result.offset(prec as isize) = b'\0' as i8;
    sscanf(
        result.offset((prec + 1) as isize),
        b"%d\0".as_ptr() as *const i8,
        decpt,
    );
    *decpt += 1;
    if *result.offset(0 as isize) == b'0' as i8 {
        *decpt = 0;
    }
    if ndigits < 1 {
        if *result.offset(0 as isize) >= b'5' as i8 {
            *decpt += 1;
        }
        *result.offset(0 as isize) = b'\0' as i8;
    }
    memcpy(
        buffer as *mut c_void,
        result as *const c_void,
        core::cmp::max(ndigits + 1, 1) as usize,
    );
    free(result as *mut c_void);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn _fcvt(
    number: f64,
    ndigits: i32,
    decpt: *mut i32,
    sign: *mut i32,
) -> *mut i8 {
    let data = msvcrt_get_thread_data();
    let mut stop: i32;
    let mut dec1: i32;
    let mut dec2: i32;
    let mut ptr1: *mut i8;
    let mut ptr2: *mut i8;
    let mut first: *mut i8;
    let mut buf: [i8; 80] = [0; 80];
    let decimal_separator: i8 = (*(*get_locinfo()).lconv).decimal_point.offset(0 as isize).read();

    if (*data).efcvt_buffer.is_null() {
        (*data).efcvt_buffer = malloc(80) as *mut i8;
    }

    stop = _snprintf(
        buf.as_mut_ptr(),
        80,
        b"%.*f\0".as_ptr() as *const i8,
        if ndigits < 0 { 0 } else { ndigits },
        number,
    );
    ptr1 = buf.as_mut_ptr();
    ptr2 = (*data).efcvt_buffer;
    first = core::ptr::null_mut();
    dec1 = 0;
    dec2 = 0;

    if *ptr1 == b'-' as i8 {
        *sign = 1;
        ptr1 = ptr1.offset(1);
    } else {
        *sign = 0;
    }

    if number < 1.0 && number > 0.0 {
        dec2 = log10(number + 1e-10) as i32;
        if -dec2 <= ndigits {
            dec2 = 0;
        }
    }

    if ndigits < 1 {
        stop += ndigits;
    }

    while *ptr1 == b'0' as i8 {
        ptr1 = ptr1.offset(1);
    }
    while *ptr1 != b'\0' as i8 && *ptr1 != decimal_separator {
        if first.is_null() {
            first = ptr2;
        }
        if (ptr1.offset_from(buf.as_ptr()) as i32) < stop {
            *ptr2 = *ptr1;
            ptr2 = ptr2.offset(1);
            ptr1 = ptr1.offset(1);
        } else {
            ptr1 = ptr1.offset(1);
        }
        dec1 += 1;
    }

    if ndigits > 0 {
        ptr1 = ptr1.offset(1);
        if first.is_null() {
            while *ptr1 == b'0' as i8 {
                *ptr2 = *ptr1;
                ptr2 = ptr2.offset(1);
                ptr1 = ptr1.offset(1);
                dec1 -= 1;
            }
        }
        while *ptr1 != b'\0' as i8 {
            if first.is_null() {
                first = ptr2;
            }
            *ptr2 = *ptr1;
            ptr2 = ptr2.offset(1);
            ptr1 = ptr1.offset(1);
        }
    }

    *ptr2 = b'\0' as i8;

    if first.is_null() {
        if number > 0.0 {
            first = ptr2;
        } else {
            first = (*data).efcvt_buffer;
            dec1 = 0;
        }
    }

    *decpt = if dec2 != 0 { dec2 } else { dec1 };
    first
}

#[no_mangle]
pub unsafe extern "C" fn _fcvt_s(
    outbuffer: *mut i8,
    size: usize,
    number: f64,
    ndigits: i32,
    decpt: *mut i32,
    sign: *mut i32,
) -> i32 {
    let mut stop: i32;
    let mut dec1: i32;
    let mut dec2: i32;
    let mut ptr1: *mut i8;
    let mut ptr2: *mut i8;
    let mut first: *mut i8;
    let mut buf: [i8; 80] = [0; 80];
    let decimal_separator: i8 = (*(*get_locinfo()).lconv).decimal_point.offset(0).read();

    if outbuffer.is_null() || decpt.is_null() || sign.is_null() || size == 0 {
        *_errno() = EINVAL as i32;
        return EINVAL as i32;
    }
    // ONE mutable copy of the by-value parameter. The translation re-shadowed `size` inside
    // each `if size > 1` block, so every `size--` mutated a copy that died at the closing
    // brace and the bound never moved.
    let mut size = size;

    stop = _snprintf(
        buf.as_mut_ptr(),
        80,
        b"%.*f\0".as_ptr() as *const i8,
        if ndigits < 0 {
            0
        } else {
            ndigits
        },
        number,
    );
    ptr1 = buf.as_mut_ptr();
    ptr2 = outbuffer;
    first = core::ptr::null_mut();
    dec1 = 0;
    dec2 = 0;

    if *ptr1 == b'-' as i8 {
        *sign = 1;
        ptr1 = ptr1.offset(1);
    } else {
        *sign = 0;
    }

    if number < 1.0 && number > 0.0 {
        dec2 = log10(number + 1e-10) as i32;
        if -dec2 <= ndigits {
            dec2 = 0;
        }
    }

    if ndigits < 1 {
        stop += ndigits;
    }

    while *ptr1 == b'0' as i8 {
        ptr1 = ptr1.offset(1);
    }
    while *ptr1 != b'\0' as i8 && *ptr1 != decimal_separator {
        if first.is_null() {
            first = ptr2;
        }
        if (ptr1 as isize - buf.as_mut_ptr() as isize) < stop as isize {
            if size > 1 {
                *ptr2 = *ptr1;
                ptr2 = ptr2.offset(1);
                ptr1 = ptr1.offset(1);
                size -= 1;
            }
        } else {
            ptr1 = ptr1.offset(1);
        }
        dec1 += 1;
    }

    if ndigits > 0 {
        ptr1 = ptr1.offset(1);
        if first.is_null() {
            while *ptr1 == b'0' as i8 {
                if number == 0.0 && size > 1 {
                    *ptr2 = b'0' as i8;
                    ptr2 = ptr2.offset(1);
                    size -= 1;
                }
                ptr1 = ptr1.offset(1);
                dec1 -= 1;
            }
        }
        while *ptr1 != b'\0' as i8 {
            if first.is_null() {
                first = ptr2;
            }
            if size > 1 {
                *ptr2 = *ptr1;
                ptr2 = ptr2.offset(1);
                ptr1 = ptr1.offset(1);
                size -= 1;
            }
        }
    }

    *ptr2 = b'\0' as i8;

    if first.is_null() && number <= 0.0 {
        dec1 = 0;
    }

    *decpt = if dec2 != 0 { dec2 } else { dec1 };
    0
}

#[no_mangle]
pub unsafe extern "C" fn _gcvt(number: f64, ndigit: i32, buff: PCHAR) -> PCHAR {
    if buff.is_null() {
        *_errno() = EINVAL as i32;
        return core::ptr::null_mut();
    }

    if ndigit < 0 {
        *_errno() = ERANGE as i32;
        return core::ptr::null_mut();
    }

    sprintf(buff, b"%.*g\0".as_ptr() as PCHAR, ndigit, number);
    buff
}

#[no_mangle]
pub unsafe extern "C" fn _gcvt_s(
    buff: *mut u8,
    size: usize,
    number: f64,
    digits: i32,
) -> i32 {
    let len: i32;

    if buff.is_null() {
        *_errno() = EINVAL as i32;
        return EINVAL as i32;
    }

    if digits < 0 || digits >= size as i32 {
        if size != 0 {
            *buff.add(0) = b'\0';
        }

        *_errno() = ERANGE as i32;
        return ERANGE as i32;
    }

    len = _scprintf(
        b"%.*g\0".as_ptr() as *const i8,
        digits,
        number,
    );
    if len > size as i32 {
        *buff.add(0) = b'\0';
        *_errno() = ERANGE as i32;
        return ERANGE as i32;
    }

    sprintf(
        buff as *mut i8,
        b"%.*g\0".as_ptr() as *const i8,
        digits,
        number,
    );
    0
}
// NOT FLIPPED, and not because it is broken: `cnt-rust`'s `cabi/math.rs` already
// exports this into the SAME archive, and it is pinned by cnt-difftest. Two strong
// definitions of one symbol inside one archive resolve by member order, which is the
// non-determinism the cohab rule exists to avoid. The body stays here, compiled and
// readable, so a later wave can move ownership deliberately rather than rediscover it.
#[cfg(any())]
#[no_mangle]
pub unsafe extern "C" fn div(num: i32, denom: i32) -> u64 {
    #[repr(C)]
    union RetUnion {
        div: div_t,
        uint64: u64,
    }

    let mut ret = RetUnion { uint64: 0 };
    ret.div.quot = num / denom;
    ret.div.rem = num % denom;
    ret.uint64
}
#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn ldiv(num: __msvcrt_long, denom: __msvcrt_long) -> u64 {
    union Ret {
        ldiv: ldiv_t,
        uint64: u64,
    }
    let mut ret = Ret { uint64: 0 };
    ret.ldiv.quot = num / denom;
    ret.ldiv.rem = num % denom;
    ret.uint64
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdiv_m16i"]
pub unsafe extern "stdcall" fn _adj_fdiv_m16i(arg: i16) {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdiv_m32"]
pub unsafe extern "stdcall" fn _adj_fdiv_m32(arg: u32) {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdiv_m32i"]
pub unsafe extern "stdcall" fn _adj_fdiv_m32i(arg: i32) {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdiv_m64"]
pub unsafe extern "stdcall" fn _adj_fdiv_m64(arg: u64) {
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _adj_fdiv_r() {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdivr_m16i"]
pub unsafe extern "stdcall" fn _adj_fdivr_m16i(arg: i16) {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdivr_m32"]
pub unsafe extern "stdcall" fn _adj_fdivr_m32(arg: u32) {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdivr_m32i"]
pub unsafe extern "stdcall" fn _adj_fdivr_m32i(arg: i32) {
    // TRACE("(): stub\n");
}

// A `__stdcall` export, so on i386 PE the symbol is `_name@<arg bytes>`. `#[no_mangle]`
// emits the name verbatim and drops BOTH halves of that decoration, which lands on a
// symbol nothing imports and leaves the C copy live — the flip would be a no-op.
#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[export_name = "_adj_fdivr_m64"]
pub unsafe extern "stdcall" fn _adj_fdivr_m64(arg: u64) {
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _adj_fpatan() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _adj_fprem() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _adj_fprem1() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _adj_fptan() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _safe_fdiv() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _safe_fdivr() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _safe_fprem() {
    // TRACE("(): stub\n");
}

#[cfg(target_arch = "x86")] // `#ifdef __i386__` in math.c
#[no_mangle]
pub unsafe extern "C" fn _safe_fprem1() {
    // TRACE("(): stub\n");
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_acos() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = acos(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_acosf() {
    let mut f: f32;
    core::arch::asm!("movd %xmm0, {}", out(reg) f, options(att_syntax));
    f = acosf(f);
    core::arch::asm!("movd {}, %xmm0", in(reg) f, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_asin() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = asin(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_asinf() {
    let mut f: f32;
    core::arch::asm!("movd %xmm0, {}", out(reg) f, options(att_syntax));
    f = asinf(f);
    core::arch::asm!("movd {}, %xmm0", in(reg) f, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_atan() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = atan(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_atan2() {
    let mut d1: f64;
    let mut d2: f64;
    core::arch::asm!(
        "movq %xmm0, {0}; movq %xmm1, {1}",
        out(reg) d1,
        out(reg) d2,
        options(att_syntax)
    );
    d1 = atan2(d1, d2);
    core::arch::asm!(
        "movq {0}, %xmm0",
        in(reg) d1,
        options(att_syntax)
    );
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_atanf() {
    let mut f: f32;
    core::arch::asm!("movd %xmm0, {}", out(reg) f, options(att_syntax));
    f = atanf(f);
    core::arch::asm!("movd {}, %xmm0", in(reg) f, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_cos() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = cos(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_cosf() {
    let mut f: f32;
    core::arch::asm!(
        "movd xmm0, {0}",
        out(reg) f,
        options(att_syntax)
    );
    f = cosf(f);
    core::arch::asm!(
        "movd {0}, xmm0",
        in(reg) f,
        options(att_syntax)
    );
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_exp() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = exp(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_expf() {
    let mut f: f32;
    core::arch::asm!("movd %xmm0, {0}", out(reg) f, options(att_syntax));
    f = expf(f);
    core::arch::asm!("movd {0}, %xmm0", in(reg) f, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_log() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = log(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_log10() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {0}", out(reg) d, options(att_syntax));
    d = log10(d);
    core::arch::asm!("movq {0}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_log10f() {
    let mut f: f32;
    core::arch::asm!(
        "movd %xmm0, {0}",
        out(reg) f,
        options(att_syntax)
    );
    f = log10f(f);
    core::arch::asm!(
        "movd {0}, %xmm0",
        in(reg) f,
        options(att_syntax)
    );
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_logf() {
    let mut f: f32;
    core::arch::asm!("movd %xmm0, {}", out(reg) f, options(att_syntax));
    f = logf(f);
    core::arch::asm!("movd {}, %xmm0", in(reg) f, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_pow() {
    let mut d1: f64;
    let mut d2: f64;
    core::arch::asm!(
        "movq %xmm0, {0}; movq %xmm1, {1}",
        out(reg) d1,
        out(reg) d2,
        options(att_syntax)
    );
    d1 = pow(d1, d2);
    core::arch::asm!(
        "movq {0}, %xmm0",
        in(reg) d1,
        options(att_syntax)
    );
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_powf() {
    let mut f1: f32;
    let mut f2: f32;
    core::arch::asm!(
        "movd %xmm0, {0}; movd %xmm1, {1}",
        out(reg) f1,
        out(reg) f2,
    );
    f1 = powf(f1, f2);
    core::arch::asm!(
        "movd {0}, %xmm0",
        in(reg) f1,
    );
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_sin() {
    let mut d: f64;
    core::arch::asm!("movq %xmm0, {}", out(reg) d, options(att_syntax));
    d = sin(d);
    core::arch::asm!("movq {}, %xmm0", in(reg) d, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_sinf() {
    let mut f: f32;
    core::arch::asm!("movd xmm0, {}", out(reg) f, options(nostack));
    f = sinf(f);
    core::arch::asm!("movd {}, xmm0", in(reg) f, options(nostack));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_tan() {
    let mut d: f64;
    core::arch::asm!(
        "movq %xmm0, {0}",
        out(reg) d,
        options(att_syntax)
    );
    d = tan(d);
    core::arch::asm!(
        "movq {0}, %xmm0",
        in(reg) d,
        options(att_syntax)
    );
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_tanf() {
    let mut f: f32;
    core::arch::asm!("movd %xmm0, {}", out(reg) f, options(att_syntax));
    f = tanf(f);
    core::arch::asm!("movd {}, %xmm0", in(reg) f, options(att_syntax));
}

#[cfg(any())] // `#ifndef __REACTOS__` in math.c — never compiled here, and the object
              // symbol table proves it: no arch's math.c.obj defines this.
#[no_mangle]
pub unsafe extern "C" fn __libm_sse2_sqrt_precise() {
    let mut cw: u32;
    let mut d: f64;

    core::arch::asm!("movq %xmm0, {}", out(reg) d);
    __control87_2(0, 0, core::ptr::null_mut(), core::ptr::addr_of_mut!(cw));
    if (cw & _MCW_RC) != 0 {
        d = sqrt(d);
        core::arch::asm!("movq {}, %xmm0", in(reg) d);
        return;
    }

    if sqrt_validate(core::ptr::addr_of_mut!(d), FALSE) == 0 {
        core::arch::asm!("movq {}, %xmm0", in(reg) d);
        return;
    }
    core::arch::asm!("sqrtsd %xmm0, %xmm0");
}

#[no_mangle]
pub unsafe extern "C" fn _scalb(num: f64, power: __msvcrt_long) -> f64 {
    ldexp(num, power)
}

#[no_mangle]
pub unsafe extern "C" fn _scalbf(num: f32, power: __msvcrt_long) -> f32 {
    // C's `return ldexp(num, power);` from a `float` function: default argument promotion
    // widens `num` to double and the return narrows back. Neither is implicit in Rust.
    ldexp(num as f64, power) as f32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//


// ============================================================================
// i386 whole-function assembly stubs, transcribed from math.c's `__ASM_GLOBAL_FUNC`
// ============================================================================
//
// These are msvcrt EXPORTS (dll/win32/msvcrt/msvcrt.spec), so something references them and
// the linker WILL go looking. If this file left them undefined, lld would pull math.c.obj in
// to satisfy them — and that object also defines every other symbol here, so the whole cohab
// flip would collapse into duplicate definitions. Defining them is what keeps the C object
// out of the link entirely.
//
// `global_asm!` and not a naked `fn`: naked functions are unstable and this crate is on
// pinned stable. The instruction text is copied verbatim from the C, in AT&T syntax so the
// two can be diffed character for character; `__ASM_GLOBAL_FUNC` was expanded by hand
// (`.text` / `.align 4` / `.globl` / `.def` / label), and its `.cfi_*` directives were
// dropped — they are DWARF unwind metadata the C emits only when the toolchain defines
// `__GCC_HAVE_DWARF2_CFI_ASM`, and carry no semantics.
//
// The doubled leading underscore in `__CIacos` / `__ftol` is not a typo: the C name already
// starts with `_`, and `__ASM_NAME` prepends the i386 PE underscore on top of it.

// ============================================================================
// 4. The sixteen _CI* functions — math.c:476-566, i386 only
//    (#if (defined(__GNUC__) || defined(__clang__)) && defined(__i386__)).
//    Two macros, CREATE_FPU_FUNC1(name, call) and CREATE_FPU_FUNC2(name, call),
//    mirrored here as macro_rules! create_fpu_func1!/create_fpu_func2!, each
//    invoked exactly as math.c:551-566 invokes the C macros.
//
//    ASMNAME = __ASM_NAME(#name): name already carries a leading "_" in every
//    call site (e.g. _CIacos), so ASMNAME gets a SECOND leading "_" prepended
//    -> "__CIacos" etc. The `call` operand is bare (e.g. acos) and also goes
//    through __ASM_NAME -> "_acos" in the `call` instruction.
// ============================================================================

// The bodies (two macros and their sixteen invocations, math.c:477-566) live in
// crt-shared-rust's `crtmath_i386` module, verbatim: ucrtbase needs the same sixteen
// exports and does not link this crate, and msvcrt reaches crt-shared-rust through its
// dependency on it, so one copy serves every CRT DLL.

// extern "C" declarations for the sixteen symbols (i386 "C" linkage adds the
// matching leading underscore automatically, so `_CIacos` here resolves to
// the `__CIacos` symbol emitted above — same trick the task's own x87_asin
// example relies on). math.c itself declares none of these (they're pure
// linker-visible ABI helpers, not called from C source in this file); the
// no-arg cdecl signature is confirmed by dll/win32/msvcrt/msvcrt.spec:129-149
// (`@ cdecl -arch=i386 _CIacos()` etc — args/return travel via the x87 stack,
// not the normal ABI, hence no Rust-visible parameters).
#[cfg(target_arch = "x86")]
extern "C" {
    pub fn _CIacos() -> f64;
    pub fn _CIasin() -> f64;
    pub fn _CIatan() -> f64;
    pub fn _CIatan2() -> f64;
    pub fn _CIcos() -> f64;
    pub fn _CIcosh() -> f64;
    pub fn _CIexp() -> f64;
    pub fn _CIfmod() -> f64;
    pub fn _CIlog() -> f64;
    pub fn _CIlog10() -> f64;
    pub fn _CIpow() -> f64;
    pub fn _CIsin() -> f64;
    pub fn _CIsinh() -> f64;
    pub fn _CIsqrt() -> f64;
    pub fn _CItan() -> f64;
    pub fn _CItanh() -> f64;
}

// ============================================================================
// 5. _ftol — math.c:568-587, i386 only.
// ============================================================================

// C: __ASM_GLOBAL_FUNC(_ftol,
//        "pushl   %ebp\n\t"
//        "movl    %esp, %ebp\n\t"
//        "subl    $12, %esp\n\t"     /* sizeof(LONGLONG) + 2*sizeof(WORD) */
//        "fnstcw  (%esp)\n\t"
//        "mov     (%esp), %ax\n\t"
//        "or      $0xc00, %ax\n\t"
//        "mov     %ax, 2(%esp)\n\t"
//        "fldcw   2(%esp)\n\t"
//        "fistpq  4(%esp)\n\t"
//        "fldcw   (%esp)\n\t"
//        "movl    4(%esp), %eax\n\t"
//        "movl    8(%esp), %edx\n\t"
//        "leave\n\t"
//        "ret")
// ASMNAME = __ASM_NAME("_ftol") = "__ftol" (second leading underscore).
// The body (math.c:568-587) lives in crt-shared-rust's `crtmath_i386` module, verbatim,
// beside `_ftol2`/`_ftol2_sse`, for the same reason as the _CI* functions above.
// msvcrt.spec:489 `@ cdecl -arch=i386 -ret64 _ftol()` confirms the no-arg,
// 64-bit-integer-return (edx:eax) signature used here.
#[cfg(target_arch = "x86")]
extern "C" {
    pub fn _ftol() -> i64;
}

// /*
//  * msvcrt.dll math functions
//  *
//  * Copyright 2000 Jon Griffiths
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  *
//  * For functions copied from musl libc (http://musl.libc.org/):
//  * ====================================================
//  * Copyright 2005-2020 Rich Felker, et al.
//  *
//  * Permission is hereby granted, free of charge, to any person obtaining
//  * a copy of this software and associated documentation files (the
//  * "Software"), to deal in the Software without restriction, including
//  * without limitation the rights to use, copy, modify, merge, publish,
//  * distribute, sublicense, and/or sell copies of the Software, and to
//  * permit persons to whom the Software is furnished to do so, subject to
//  * the following conditions:
//  *
//  * The above copyright notice and this permission notice shall be
//  * included in all copies or substantial portions of the Software.
//  * ====================================================
//  */
//
// #include <assert.h>
// #include <complex.h>
// #include <stdio.h>
// #include <fenv.h>
// #include <fpieee.h>
// #include <limits.h>
// #include <locale.h>
// #include <math.h>
//
// #include "msvcrt.h"
// #include "winternl.h"
//
// #include "wine/asm.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// #ifdef _MSC_VER
// #pragma function(abs,labs,div,ldiv)
// #endif
//
// #undef div
// #undef ldiv
//
// #define _DOMAIN         1       /* domain error in argument */
// #define _SING           2       /* singularity */
// #define _OVERFLOW       3       /* range overflow */
// #define _UNDERFLOW      4       /* range underflow */
//
// typedef int (CDECL *MSVCRT_matherr_func)(struct _exception *);
//
// static MSVCRT_matherr_func MSVCRT_default_matherr_func = NULL;
//
// BOOL sse2_supported;
// static BOOL sse2_enabled;
//
// void msvcrt_init_math( void *module )
// {
//     sse2_supported = ((volatile const BYTE *)(ULONG_PTR)0x7FFE0274)[PF_XMMI64_INSTRUCTIONS_AVAILABLE];
//     sse2_enabled = FALSE as BOOL;
// }
//
// static inline double ret_nan( BOOL update_sw )
// {
//     double x = 1.0;
//     if (!update_sw) return -NAN;
//     return (x - x) / (x - x);
// }
//
// #define SET_X87_CW(MASK) \
//     "subl $4, %esp\n\t" \
//     __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t") \
//     "fnstcw (%esp)\n\t" \
//     "movw (%esp), %ax\n\t" \
//     "movw %ax, 2(%esp)\n\t" \
//     "testw $" #MASK ", %ax\n\t" \
//     "jz 1f\n\t" \
//     "andw $~" #MASK ", %ax\n\t" \
//     "movw %ax, 2(%esp)\n\t" \
//     "fldcw 2(%esp)\n\t" \
//     "1:\n\t"
//
// #define RESET_X87_CW \
//     "movw (%esp), %ax\n\t" \
//     "cmpw %ax, 2(%esp)\n\t" \
//     "je 1f\n\t" \
//     "fstpl 8(%esp)\n\t" \
//     "fldcw (%esp)\n\t" \
//     "fldl 8(%esp)\n\t" \
//     "fwait\n\t" \
//     "1:\n\t" \
//     "addl $4, %esp\n\t" \
//     __ASM_CFI(".cfi_adjust_cfa_offset -4\n\t")
//
// /*********************************************************************
//  *      _matherr (CRTDLL.@)
//  */
// int CDECL _matherr(struct _exception *e)
// {
//     return 0;
// }
//
//
// double math_error(int type, const char *name, double arg1, double arg2, double retval)
// {
//     struct _exception exception = {type, (char *)name, arg1, arg2, retval};
//
//     TRACE("(%d, %s, %g, %g, %g)\n", type, debugstr_a(name), arg1, arg2, retval);
//
//     if (MSVCRT_default_matherr_func && MSVCRT_default_matherr_func(&exception))
//         return exception.retval;
//
//     switch (type)
//     {
//     case 0:
//         /* don't set errno */
//         break;
//     case _DOMAIN:
//         *_errno() = EDOM as i32;
//         break;
//     case _SING:
//     case _OVERFLOW:
//         *_errno() = ERANGE as i32;
//         break;
//     case _UNDERFLOW:
//         /* don't set errno */
//         break;
//     default:
//         ERR("Unhandled math error!\n");
//     }
//
//     return exception.retval;
// }
//
// /*********************************************************************
//  *      __setusermatherr (MSVCRT.@)
//  */
// void CDECL __setusermatherr(MSVCRT_matherr_func func)
// {
//     MSVCRT_default_matherr_func = func;
//     TRACE("new matherr handler %p\n", func);
// }
//
// /*********************************************************************
//  *      _set_SSE2_enable (MSVCRT.@)
//  */
// int CDECL _set_SSE2_enable(int flag)
// {
//     sse2_enabled = flag && sse2_supported;
//     return sse2_enabled;
// }
//
// #if defined(_WIN64)
//
// #endif
//
// #if !defined(__i386__) || _MSVCR_VER>=120
//
// /*********************************************************************
//  *      _chgsignf (MSVCRT.@)
//  */
// float CDECL _chgsignf( float num )
// {
//     union { float f; UINT32 i; } u = { num };
//     u.i ^= 0x80000000;
//     return u.f;
// }
//
// #endif
//
// #ifndef __i386__
//
// /*********************************************************************
//  *      _fpclassf (MSVCRT.@)
//  */
// int CDECL _fpclassf( float num )
// {
//     union { float f; UINT32 i; } u = { num };
//     int e = u.i >> 23 & 0xff;
//     int s = u.i >> 31;
//
//     switch (e)
//     {
//     case 0:
//         if (u.i << 1) return s ? _FPCLASS_ND : _FPCLASS_PD;
//         return s ? _FPCLASS_NZ : _FPCLASS_PZ;
//     case 0xff:
//         if (u.i << 9) return ((u.i >> 22) & 1) ? _FPCLASS_QNAN : _FPCLASS_SNAN;
//         return s ? _FPCLASS_NINF : _FPCLASS_PINF;
//     default:
//         return s ? _FPCLASS_NN : _FPCLASS_PN;
//     }
// }
//
// /*********************************************************************
//  *      _finitef (MSVCRT.@)
//  */
// int CDECL _finitef( float num )
// {
//     union { float f; UINT32 i; } u = { num };
//     return (u.i & 0x7fffffff) < 0x7f800000;
// }
//
// /*********************************************************************
//  *      _isnanf (MSVCRT.@)
//  */
// int CDECL _isnanf( float num )
// {
//     union { float f; UINT32 i; } u = { num };
//     return (u.i & 0x7fffffff) > 0x7f800000;
// }
//
// /*********************************************************************
//  *      atanf (MSVCRT.@)
//  */
// float CDECL MSVCRT_atanf( float x )
// {
//     if (isnan(x)) return math_error(_DOMAIN, "atanf", x, 0.0, x);
//     return atanf( x );
// }
//
// #ifndef __i386__
// extern short CDECL _fdclass(float x);
//
// static BOOL sqrtf_validate( float *x )
// {
//     short c = _fdclass(*x);
//
//     if (c == FP_ZERO as i16) return FALSE;
//     if (c == FP_NAN as i16) return FALSE;
//     if (signbit(*x))
//     {
//         *x = math_error(_DOMAIN, "sqrtf", *x, 0.0, ret_nan(TRUE as u8));
//         return FALSE;
//     }
//     if (c == FP_INFINITE as i16) return FALSE;
//     return TRUE;
// }
//
// #ifdef __arm64ec__
// static float __attribute__((naked)) CDECL asm_sqrtf(float x)
// {
//     asm( "fsqrt s0,s0; ret" );
// }
// #elif defined __aarch64__
// float CDECL asm_sqrtf(float);
// __ASM_GLOBAL_FUNC( asm_sqrtf, "fsqrt s0,s0; ret" )
// #elif defined __arm__
// float CDECL asm_sqrtf(float);
// __ASM_GLOBAL_FUNC( asm_sqrtf, "vsqrt s0,s0; bx lr" )
// #elif defined __x86_64__
// float CDECL asm_sqrtf(float);
// __ASM_GLOBAL_FUNC( asm_sqrtf, "sqrtss %xmm0, %xmm0; ret" )
// #endif
// #endif
//
// /*********************************************************************
//  *      sqrtf (MSVCRT.@)
//  */
// float CDECL MSVCRT_sqrtf( float x )
// {
// #ifndef __i386__
//     if (!sqrtf_validate(&x))
//         return x;
//
//     return asm_sqrtf(x);
// #else
//     return sqrtf( x );
// #endif
// }
//
// /*********************************************************************
//  *      tanhf (MSVCRT.@)
//  */
// float CDECL MSVCRT_tanhf( float x )
// {
//     if (isnan( x ))
//     {
//         *(UINT32*)&x |= 0x400000;
//         return math_error(_DOMAIN, "tanhf", x, 0.0, x);
//     }
//     return tanhf( x );
// }
//
// #endif
//
// /*********************************************************************
//  *		asin (MSVCRT.@)
//  */
// #ifdef __i386__
// double CDECL x87_asin(double);
// __ASM_GLOBAL_FUNC( x87_asin,
//         "fldl 4(%esp)\n\t"
//         SET_X87_CW(~0x37f)
//         "fld %st\n\t"
//         "fld1\n\t"
//         "fsubp\n\t"
//         "fld1\n\t"
//         "fadd %st(2)\n\t"
//         "fmulp\n\t"
//         "fsqrt\n\t"
//         "fpatan\n\t"
//         RESET_X87_CW
//         "ret" )
// #endif
//
// double CDECL MSVCRT_asin( double x )
// {
// #ifdef __i386__
//     unsigned int x87_cw, sse2_cw;
//     unsigned int hx = *(ULONGLONG*)&x >> 32;
//     unsigned int ix = hx & 0x7fffffff;
//
//     if (isnan(x)) return math_error(_DOMAIN, "asin", x, 0.0, x);
//
//     /* |x| < 1 */
//     if (ix < 0x3ff00000)
//     {
//         __control87_2(0, 0, &x87_cw, &sse2_cw);
//         if (!sse2_enabled || (x87_cw & _MCW_EM) != _MCW_EM
//             || (sse2_cw & (_MCW_EM | _MCW_RC)) != _MCW_EM)
//             return x87_asin(x);
//     }
// #else
//     if (isnan(x)) return x;
// #endif
//
//     return asin( x );
// }
//
// /*********************************************************************
//  *		atan (MSVCRT.@)
//  */
// double CDECL MSVCRT_atan( double x )
// {
//     if (isnan(x)) return math_error(_DOMAIN, "atan", x, 0.0, x);
//     return atan( x );
// }
//
// /*********************************************************************
//  *		exp (MSVCRT.@)
//  */
// double CDECL MSVCRT_exp( double x )
// {
//     if (isnan( x )) return math_error(_DOMAIN, "exp", x, 0.0, 1.0 + x);
//     return exp( x );
// }
//
// extern short CDECL _dclass(double x);
//
// static BOOL sqrt_validate( double *x, BOOL update_sw )
// {
//     short c = _dclass(*x);
//
//     if (c == FP_ZERO as i16) return FALSE;
//     if (c == FP_NAN as i16)
//     {
// #ifdef __i386__
//         if (update_sw)
//             *x = math_error(_DOMAIN, "sqrt", *x, 0.0, *x);
// #else
//         /* set signaling bit */
//         *(ULONGLONG*)x |= 0x8000000000000u64;
// #endif
//         return FALSE;
//     }
//     if (signbit(*x))
//     {
//         *x = math_error(_DOMAIN, "sqrt", *x, 0.0, ret_nan(update_sw));
//         return FALSE;
//     }
//     if (c == FP_INFINITE as i16) return FALSE;
//     return TRUE;
// }
//
// #ifdef __arm64ec__
// static double __attribute__((naked)) CDECL asm_sqrt(double x)
// {
//     asm( "fsqrt d0,d0; ret" );
// }
// #elif defined __aarch64__
// double CDECL asm_sqrt(double);
// __ASM_GLOBAL_FUNC( asm_sqrt, "fsqrt d0,d0; ret" )
// #elif defined __arm__
// double CDECL asm_sqrt(double);
// __ASM_GLOBAL_FUNC( asm_sqrt, "vsqrt d0,d0; bx lr" )
// #elif defined __x86_64__
// double CDECL asm_sqrt(double);
// __ASM_GLOBAL_FUNC( asm_sqrt, "sqrtsd %xmm0, %xmm0; ret" )
// #elif defined __i386__
// double CDECL asm_sqrt(double);
// __ASM_GLOBAL_FUNC( asm_sqrt,
//         "fldl 4(%esp)\n\t"
//         SET_X87_CW(0xc00)
//         "fsqrt\n\t"
//         RESET_X87_CW
//         "ret" )
// #endif
//
// /*********************************************************************
//  *		sqrt (MSVCRT.@)
//  */
// double CDECL MSVCRT_sqrt( double x )
// {
//     if (!sqrt_validate(&x, TRUE))
//         return x;
//
//     return asm_sqrt(x);
// }
//
// /*********************************************************************
//  *		tanh (MSVCRT.@)
//  */
// double CDECL MSVCRT_tanh( double x )
// {
//     if (isnan( x ))
//     {
//         *(UINT64*)&x |= 0x0008000000000000u64;
//         return math_error(_DOMAIN, "tanh", x, 0.0, x);
//     }
//     return tanh( x );
// }
//
// #if (defined(__GNUC__) || defined(__clang__)) && defined(__i386__)
//
// #define CREATE_FPU_FUNC1(name, call) \
//     __ASM_GLOBAL_FUNC(name, \
//             "pushl   %ebp\n\t" \
//             __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t") \
//             __ASM_CFI(".cfi_rel_offset %ebp,0\n\t") \
//             "movl    %esp, %ebp\n\t" \
//             __ASM_CFI(".cfi_def_cfa_register %ebp\n\t") \
//             "subl    $68, %esp\n\t" /* sizeof(double)*8 + sizeof(int) */ \
//             "fstpl   (%esp)\n\t"    /* store function argument */ \
//             "fwait\n\t" \
//             "movl    $1, %ecx\n\t"  /* empty FPU stack */ \
//             "1:\n\t" \
//             "fxam\n\t" \
//             "fstsw   %ax\n\t" \
//             "and     $0x4500, %ax\n\t" \
//             "cmp     $0x4100, %ax\n\t" \
//             "je      2f\n\t" \
//             "fstpl    (%esp,%ecx,8)\n\t" \
//             "fwait\n\t" \
//             "incl    %ecx\n\t" \
//             "jmp     1b\n\t" \
//             "2:\n\t" \
//             "movl    %ecx, -4(%ebp)\n\t" \
//             "call    " __ASM_NAME( #call ) "\n\t" \
//             "movl    -4(%ebp), %ecx\n\t" \
//             "fstpl   (%esp)\n\t"    /* save result */ \
//             "3:\n\t"                /* restore FPU stack */ \
//             "decl    %ecx\n\t" \
//             "fldl    (%esp,%ecx,8)\n\t" \
//             "cmpl    $0, %ecx\n\t" \
//             "jne     3b\n\t" \
//             "leave\n\t" \
//             __ASM_CFI(".cfi_def_cfa %esp,4\n\t") \
//             __ASM_CFI(".cfi_same_value %ebp\n\t") \
//             "ret")
//
// #define CREATE_FPU_FUNC2(name, call) \
//     __ASM_GLOBAL_FUNC(name, \
//             "pushl   %ebp\n\t" \
//             __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t") \
//             __ASM_CFI(".cfi_rel_offset %ebp,0\n\t") \
//             "movl    %esp, %ebp\n\t" \
//             __ASM_CFI(".cfi_def_cfa_register %ebp\n\t") \
//             "subl    $68, %esp\n\t" /* sizeof(double)*8 + sizeof(int) */ \
//             "fstpl   8(%esp)\n\t"   /* store function argument */ \
//             "fwait\n\t" \
//             "fstpl   (%esp)\n\t" \
//             "fwait\n\t" \
//             "movl    $2, %ecx\n\t"  /* empty FPU stack */ \
//             "1:\n\t" \
//             "fxam\n\t" \
//             "fstsw   %ax\n\t" \
//             "and     $0x4500, %ax\n\t" \
//             "cmp     $0x4100, %ax\n\t" \
//             "je      2f\n\t" \
//             "fstpl    (%esp,%ecx,8)\n\t" \
//             "fwait\n\t" \
//             "incl    %ecx\n\t" \
//             "jmp     1b\n\t" \
//             "2:\n\t" \
//             "movl    %ecx, -4(%ebp)\n\t" \
//             "call    " __ASM_NAME( #call ) "\n\t" \
//             "movl    -4(%ebp), %ecx\n\t" \
//             "fstpl   8(%esp)\n\t"   /* save result */ \
//             "3:\n\t"                /* restore FPU stack */ \
//             "decl    %ecx\n\t" \
//             "fldl    (%esp,%ecx,8)\n\t" \
//             "cmpl    $1, %ecx\n\t" \
//             "jne     3b\n\t" \
//             "leave\n\t" \
//             __ASM_CFI(".cfi_def_cfa %esp,4\n\t") \
//             __ASM_CFI(".cfi_same_value %ebp\n\t") \
//             "ret")
//
// CREATE_FPU_FUNC1(_CIacos, acos)
// CREATE_FPU_FUNC1(_CIasin, asin)
// CREATE_FPU_FUNC1(_CIatan, atan)
// CREATE_FPU_FUNC2(_CIatan2, atan2)
// CREATE_FPU_FUNC1(_CIcos, cos)
// CREATE_FPU_FUNC1(_CIcosh, cosh)
// CREATE_FPU_FUNC1(_CIexp, exp)
// CREATE_FPU_FUNC2(_CIfmod, fmod)
// CREATE_FPU_FUNC1(_CIlog, log)
// CREATE_FPU_FUNC1(_CIlog10, log10)
// CREATE_FPU_FUNC2(_CIpow, pow)
// CREATE_FPU_FUNC1(_CIsin, sin)
// CREATE_FPU_FUNC1(_CIsinh, sinh)
// CREATE_FPU_FUNC1(_CIsqrt, sqrt)
// CREATE_FPU_FUNC1(_CItan, tan)
// CREATE_FPU_FUNC1(_CItanh, tanh)
//
// __ASM_GLOBAL_FUNC(_ftol,
//         "pushl   %ebp\n\t"
//         __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//         __ASM_CFI(".cfi_rel_offset %ebp,0\n\t")
//         "movl    %esp, %ebp\n\t"
//         __ASM_CFI(".cfi_def_cfa_register %ebp\n\t")
//         "subl    $12, %esp\n\t"     /* sizeof(LONGLONG) + 2*sizeof(WORD) */
//         "fnstcw  (%esp)\n\t"
//         "mov     (%esp), %ax\n\t"
//         "or      $0xc00, %ax\n\t"
//         "mov     %ax, 2(%esp)\n\t"
//         "fldcw   2(%esp)\n\t"
//         "fistpq  4(%esp)\n\t"
//         "fldcw   (%esp)\n\t"
//         "movl    4(%esp), %eax\n\t"
//         "movl    8(%esp), %edx\n\t"
//         "leave\n\t"
//         __ASM_CFI(".cfi_def_cfa %esp,4\n\t")
//         __ASM_CFI(".cfi_same_value %ebp\n\t")
//         "ret")
//
// #endif /* (defined(__GNUC__) || defined(__clang__)) && defined(__i386__) */
//
// #ifndef __REACTOS__
// /*********************************************************************
//  *		_fpclass (MSVCRT.@)
//  */
// int CDECL _fpclass(double num)
// {
//     union { double f; UINT64 i; } u = { num };
//     int e = u.i >> 52 & 0x7ff;
//     int s = u.i >> 63;
//
//     switch (e)
//     {
//     case 0:
//         if (u.i << 1) return s ? _FPCLASS_ND : _FPCLASS_PD;
//         return s ? _FPCLASS_NZ : _FPCLASS_PZ;
//     case 0x7ff:
//         if (u.i << 12) return ((u.i >> 51) & 1) ? _FPCLASS_QNAN : _FPCLASS_SNAN;
//         return s ? _FPCLASS_NINF : _FPCLASS_PINF;
//     default:
//         return s ? _FPCLASS_NN : _FPCLASS_PN;
//     }
// }
// #endif // __REACTOS__
//
// /*********************************************************************
//  *		_rotl (MSVCRT.@)
//  */
// unsigned int CDECL MSVCRT__rotl(unsigned int num, int shift)
// {
//   shift &= 31;
//   return (num << shift) | (num >> (32-shift));
// }
//
// /*********************************************************************
//  *		_lrotl (MSVCRT.@)
//  */
// __msvcrt_ulong CDECL MSVCRT__lrotl(__msvcrt_ulong num, int shift)
// {
//   shift &= 0x1f;
//   return (num << shift) | (num >> (32-shift));
// }
//
// /*********************************************************************
//  *		_lrotr (MSVCRT.@)
//  */
// __msvcrt_ulong CDECL MSVCRT__lrotr(__msvcrt_ulong num, int shift)
// {
//   shift &= 0x1f;
//   return (num >> shift) | (num << (32-shift));
// }
//
// /*********************************************************************
//  *		_rotr (MSVCRT.@)
//  */
// unsigned int CDECL MSVCRT__rotr(unsigned int num, int shift)
// {
//     shift &= 0x1f;
//     return (num >> shift) | (num << (32-shift));
// }
//
// /*********************************************************************
//  *		_rotl64 (MSVCRT.@)
//  */
// unsigned __int64 CDECL MSVCRT__rotl64(unsigned __int64 num, int shift)
// {
//   shift &= 63;
//   return (num << shift) | (num >> (64-shift));
// }
//
// /*********************************************************************
//  *		_rotr64 (MSVCRT.@)
//  */
// unsigned __int64 CDECL MSVCRT__rotr64(unsigned __int64 num, int shift)
// {
//     shift &= 63;
//     return (num >> shift) | (num << (64-shift));
// }
//
// /*********************************************************************
//  *		abs (MSVCRT.@)
//  */
// int CDECL abs( int n )
// {
//     return n >= 0 ? n : -n;
// }
//
// /*********************************************************************
//  *		labs (MSVCRT.@)
//  */
// __msvcrt_long CDECL labs( __msvcrt_long n )
// {
//     return n >= 0 ? n : -n;
// }
//
//
//
// /*********************************************************************
//  *		_abs64 (MSVCRT.@)
//  */
// #ifdef _MSC_VER
// #pragma function(_abs64)
// #endif
// __int64 CDECL _abs64( __int64 n )
// {
//     return n >= 0 ? n : -n;
// }
//
// #if defined(__i386__) || defined(__x86_64__)
// #ifdef _MSC_VER
// #define get_mxcsr() _mm_getcsr()
// #define set_mxcsr(val) _mm_setcsr(val)
// #else
// static unsigned int get_mxcsr(void)
// {
//     unsigned int ret;
// #ifdef __arm64ec__
//     extern NTSTATUS (*__os_arm64x_get_x64_information)(ULONG,void*,void*);
//     __os_arm64x_get_x64_information( 0, &ret, NULL );
// #else
//     __asm__ __volatile__( "stmxcsr %0" : "=m" (ret) );
// #endif
//     return ret;
// }
//
// static void set_mxcsr( unsigned int val )
// {
// #ifdef __arm64ec__
//     extern NTSTATUS (*__os_arm64x_set_x64_information)(ULONG,ULONG_PTR,void*);
//     __os_arm64x_set_x64_information( 0, val, NULL );
// #else
//     __asm__ __volatile__( "ldmxcsr %0" : : "m" (val) );
// #endif
// }
// #endif
//
// #ifdef __REACTOS__
// __ATTRIBUTE_SSE__
// #endif
// static void _setfp_sse( unsigned int *cw, unsigned int cw_mask,
//         unsigned int *sw, unsigned int sw_mask )
// {
// #if defined(__GNUC__) || defined(__clang__) || defined(_MSC_VER)
//     unsigned int old_fpword, fpword = get_mxcsr();
//     unsigned int flags;
//
//     old_fpword = fpword;
//
//     cw_mask &= _MCW_EM | _MCW_RC | _MCW_DN;
//     sw_mask &= _MCW_EM;
//
//     if (sw)
//     {
//         flags = 0;
//         if (fpword & 0x1) flags |= _SW_INVALID;
//         if (fpword & 0x2) flags |= _SW_DENORMAL;
//         if (fpword & 0x4) flags |= _SW_ZERODIVIDE;
//         if (fpword & 0x8) flags |= _SW_OVERFLOW;
//         if (fpword & 0x10) flags |= _SW_UNDERFLOW;
//         if (fpword & 0x20) flags |= _SW_INEXACT;
//
//         *sw = (flags & ~sw_mask) | (*sw & sw_mask);
//         TRACE("sse2 update sw %08x to %08x\n", flags, *sw);
//         fpword &= ~0x3f;
//         if (*sw & _SW_INVALID) fpword |= 0x1;
//         if (*sw & _SW_DENORMAL) fpword |= 0x2;
//         if (*sw & _SW_ZERODIVIDE) fpword |= 0x4;
//         if (*sw & _SW_OVERFLOW) fpword |= 0x8;
//         if (*sw & _SW_UNDERFLOW) fpword |= 0x10;
//         if (*sw & _SW_INEXACT) fpword |= 0x20;
//         *sw = flags;
//     }
//
//     if (cw)
//     {
//         flags = 0;
//         if (fpword & 0x80) flags |= _EM_INVALID;
//         if (fpword & 0x100) flags |= _EM_DENORMAL;
//         if (fpword & 0x200) flags |= _EM_ZERODIVIDE;
//         if (fpword & 0x400) flags |= _EM_OVERFLOW;
//         if (fpword & 0x800) flags |= _EM_UNDERFLOW;
//         if (fpword & 0x1000) flags |= _EM_INEXACT;
//         switch (fpword & 0x6000)
//         {
//         case 0x6000: flags |= _RC_UP|_RC_DOWN; break;
//         case 0x4000: flags |= _RC_UP; break;
//         case 0x2000: flags |= _RC_DOWN; break;
//         }
//         switch (fpword & 0x8040)
//         {
//         case 0x0040: flags |= _DN_FLUSH_OPERANDS_SAVE_RESULTS; break;
//         case 0x8000: flags |= _DN_SAVE_OPERANDS_FLUSH_RESULTS; break;
//         case 0x8040: flags |= _DN_FLUSH; break;
//         }
//
//         *cw = (flags & ~cw_mask) | (*cw & cw_mask);
//         TRACE("sse2 update cw %08x to %08x\n", flags, *cw);
//         fpword &= ~0xffc0;
//         if (*cw & _EM_INVALID) fpword |= 0x80;
//         if (*cw & _EM_DENORMAL) fpword |= 0x100;
//         if (*cw & _EM_ZERODIVIDE) fpword |= 0x200;
//         if (*cw & _EM_OVERFLOW) fpword |= 0x400;
//         if (*cw & _EM_UNDERFLOW) fpword |= 0x800;
//         if (*cw & _EM_INEXACT) fpword |= 0x1000;
//         switch (*cw & _MCW_RC)
//         {
//         case _RC_UP|_RC_DOWN: fpword |= 0x6000; break;
//         case _RC_UP: fpword |= 0x4000; break;
//         case _RC_DOWN: fpword |= 0x2000; break;
//         }
//         switch (*cw & _MCW_DN)
//         {
//         case _DN_FLUSH_OPERANDS_SAVE_RESULTS: fpword |= 0x0040; break;
//         case _DN_SAVE_OPERANDS_FLUSH_RESULTS: fpword |= 0x8000; break;
//         case _DN_FLUSH: fpword |= 0x8040; break;
//         }
//
//         /* clear status word if anything changes */
//         if (fpword != old_fpword && !sw)
//         {
//             TRACE("sse2 clear status word\n");
//             fpword &= ~0x3f;
//         }
//     }
//
//     if (fpword != old_fpword) set_mxcsr( fpword );
// #else
//     FIXME("not implemented\n");
//     if (cw) *cw = 0;
//     if (sw) *sw = 0;
// #endif
// }
// #endif
//
// static void _setfp( unsigned int *cw, unsigned int cw_mask,
//         unsigned int *sw, unsigned int sw_mask )
// {
// #if (defined(__GNUC__) || defined(__clang__) || defined(_MSC_VER)) && defined(__i386__)
//     unsigned long oldcw = 0, newcw = 0;
//     unsigned long oldsw = 0, newsw = 0;
//     unsigned int flags;
//
//     cw_mask &= _MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC;
//     sw_mask &= _MCW_EM;
//
//     if (sw)
//     {
// #ifdef _MSC_VER
//         __asm { fstsw newsw }
// #else
//         __asm__ __volatile__( "fstsw %0" : "=m" (newsw) );
// #endif
//         oldsw = newsw;
//
//         flags = 0;
//         if (newsw & 0x1) flags |= _SW_INVALID;
//         if (newsw & 0x2) flags |= _SW_DENORMAL;
//         if (newsw & 0x4) flags |= _SW_ZERODIVIDE;
//         if (newsw & 0x8) flags |= _SW_OVERFLOW;
//         if (newsw & 0x10) flags |= _SW_UNDERFLOW;
//         if (newsw & 0x20) flags |= _SW_INEXACT;
//
//         *sw = (flags & ~sw_mask) | (*sw & sw_mask);
//         TRACE("x86 update sw %08x to %08x\n", flags, *sw);
//         newsw &= ~0x3f;
//         if (*sw & _SW_INVALID) newsw |= 0x1;
//         if (*sw & _SW_DENORMAL) newsw |= 0x2;
//         if (*sw & _SW_ZERODIVIDE) newsw |= 0x4;
//         if (*sw & _SW_OVERFLOW) newsw |= 0x8;
//         if (*sw & _SW_UNDERFLOW) newsw |= 0x10;
//         if (*sw & _SW_INEXACT) newsw |= 0x20;
//         *sw = flags;
//     }
//
//     if (cw)
//     {
// #ifdef _MSC_VER
//         __asm { fstcw newcw }
// #else
//        __asm__ __volatile__( "fstcw %0" : "=m" (newcw) );
// #endif
//         oldcw = newcw;
//
//         flags = 0;
//         if (newcw & 0x1) flags |= _EM_INVALID;
//         if (newcw & 0x2) flags |= _EM_DENORMAL;
//         if (newcw & 0x4) flags |= _EM_ZERODIVIDE;
//         if (newcw & 0x8) flags |= _EM_OVERFLOW;
//         if (newcw & 0x10) flags |= _EM_UNDERFLOW;
//         if (newcw & 0x20) flags |= _EM_INEXACT;
//         switch (newcw & 0xc00)
//         {
//         case 0xc00: flags |= _RC_UP|_RC_DOWN; break;
//         case 0x800: flags |= _RC_UP; break;
//         case 0x400: flags |= _RC_DOWN; break;
//         }
//         switch (newcw & 0x300)
//         {
//         case 0x0: flags |= _PC_24; break;
//         case 0x200: flags |= _PC_53; break;
//         case 0x300: flags |= _PC_64; break;
//         }
//         if (newcw & 0x1000) flags |= _IC_AFFINE;
//
//         *cw = (flags & ~cw_mask) | (*cw & cw_mask);
//         TRACE("x86 update cw %08x to %08x\n", flags, *cw);
//         newcw &= ~0x1f3f;
//         if (*cw & _EM_INVALID) newcw |= 0x1;
//         if (*cw & _EM_DENORMAL) newcw |= 0x2;
//         if (*cw & _EM_ZERODIVIDE) newcw |= 0x4;
//         if (*cw & _EM_OVERFLOW) newcw |= 0x8;
//         if (*cw & _EM_UNDERFLOW) newcw |= 0x10;
//         if (*cw & _EM_INEXACT) newcw |= 0x20;
//         switch (*cw & _MCW_RC)
//         {
//         case _RC_UP|_RC_DOWN: newcw |= 0xc00; break;
//         case _RC_UP: newcw |= 0x800; break;
//         case _RC_DOWN: newcw |= 0x400; break;
//         }
//         switch (*cw & _MCW_PC)
//         {
//         case _PC_64: newcw |= 0x300; break;
//         case _PC_53: newcw |= 0x200; break;
//         case _PC_24: newcw |= 0x0; break;
//         }
//         if (*cw & _IC_AFFINE) newcw |= 0x1000;
//     }
//
//     if (oldsw != newsw && (newsw & 0x3f))
//     {
//         struct {
//             WORD control_word;
//             WORD unused1;
//             WORD status_word;
//             WORD unused2;
//             WORD tag_word;
//             WORD unused3;
//             DWORD instruction_pointer;
//             WORD code_segment;
//             WORD unused4;
//             DWORD operand_addr;
//             WORD data_segment;
//             WORD unused5;
//         } fenv;
//
//         assert(cw);
//
// #ifdef _MSC_VER
//         __asm { fnstenv fenv }
// #else
//         __asm__ __volatile__( "fnstenv %0" : "=m" (fenv) );
// #endif
//         fenv.control_word = newcw;
//         fenv.status_word = newsw;
// #ifdef _MSC_VER
//         __asm { fldenv fenv }
// #else
//         __asm__ __volatile__( "fldenv %0" : : "m" (fenv) : "st", "st(1)",
//                 "st(2)", "st(3)", "st(4)", "st(5)", "st(6)", "st(7)" );
// #endif
//         return;
//     }
//
//     if (oldsw != newsw)
// #ifdef _MSC_VER
//         __asm { fnclex }
// #else
//         __asm__ __volatile__( "fnclex" );
// #endif
//     if (oldcw != newcw)
// #ifdef _MSC_VER
//         __asm { fldcw newcw }
// #else
//         __asm__ __volatile__( "fldcw %0" : : "m" (newcw) );
// #endif
// #elif defined(__x86_64__)
//     _setfp_sse(cw, cw_mask, sw, sw_mask);
// #elif defined(__aarch64__)
//     ULONG_PTR old_fpsr = 0, fpsr = 0, old_fpcr = 0, fpcr = 0;
//     unsigned int flags;
//
//     cw_mask &= _MCW_EM | _MCW_RC;
//     sw_mask &= _MCW_EM;
//
//     if (sw)
//     {
//         __asm__ __volatile__( "mrs %0, fpsr" : "=r" (fpsr) );
//         old_fpsr = fpsr;
//
//         flags = 0;
//         if (fpsr & 0x1) flags |= _SW_INVALID;
//         if (fpsr & 0x2) flags |= _SW_ZERODIVIDE;
//         if (fpsr & 0x4) flags |= _SW_OVERFLOW;
//         if (fpsr & 0x8) flags |= _SW_UNDERFLOW;
//         if (fpsr & 0x10) flags |= _SW_INEXACT;
//         if (fpsr & 0x80) flags |= _SW_DENORMAL;
//
//         *sw = (flags & ~sw_mask) | (*sw & sw_mask);
//         TRACE("aarch64 update sw %08x to %08x\n", flags, *sw);
//         fpsr &= ~0x9f;
//         if (*sw & _SW_INVALID) fpsr |= 0x1;
//         if (*sw & _SW_ZERODIVIDE) fpsr |= 0x2;
//         if (*sw & _SW_OVERFLOW) fpsr |= 0x4;
//         if (*sw & _SW_UNDERFLOW) fpsr |= 0x8;
//         if (*sw & _SW_INEXACT) fpsr |= 0x10;
//         if (*sw & _SW_DENORMAL) fpsr |= 0x80;
//         *sw = flags;
//     }
//
//     if (cw)
//     {
//         __asm__ __volatile__( "mrs %0, fpcr" : "=r" (fpcr) );
//         old_fpcr = fpcr;
//
//         flags = 0;
//         if (!(fpcr & 0x100)) flags |= _EM_INVALID;
//         if (!(fpcr & 0x200)) flags |= _EM_ZERODIVIDE;
//         if (!(fpcr & 0x400)) flags |= _EM_OVERFLOW;
//         if (!(fpcr & 0x800)) flags |= _EM_UNDERFLOW;
//         if (!(fpcr & 0x1000)) flags |= _EM_INEXACT;
//         if (!(fpcr & 0x8000)) flags |= _EM_DENORMAL;
//         switch (fpcr & 0xc00000)
//         {
//         case 0x400000: flags |= _RC_UP; break;
//         case 0x800000: flags |= _RC_DOWN; break;
//         case 0xc00000: flags |= _RC_CHOP; break;
//         }
//
//         *cw = (flags & ~cw_mask) | (*cw & cw_mask);
//         TRACE("aarch64 update cw %08x to %08x\n", flags, *cw);
//         fpcr &= ~0xc09f00ul;
//         if (!(*cw & _EM_INVALID)) fpcr |= 0x100;
//         if (!(*cw & _EM_ZERODIVIDE)) fpcr |= 0x200;
//         if (!(*cw & _EM_OVERFLOW)) fpcr |= 0x400;
//         if (!(*cw & _EM_UNDERFLOW)) fpcr |= 0x800;
//         if (!(*cw & _EM_INEXACT)) fpcr |= 0x1000;
//         if (!(*cw & _EM_DENORMAL)) fpcr |= 0x8000;
//         switch (*cw & _MCW_RC)
//         {
//         case _RC_CHOP: fpcr |= 0xc00000; break;
//         case _RC_UP: fpcr |= 0x400000; break;
//         case _RC_DOWN: fpcr |= 0x800000; break;
//         }
//     }
//
//     /* mask exceptions if needed */
//     if (old_fpcr != fpcr && ~(old_fpcr >> 8) & fpsr & 0x9f != fpsr & 0x9f)
//     {
//         ULONG_PTR mask = fpcr & ~0x9f00;
//         __asm__ __volatile__( "msr fpcr, %0" :: "r" (mask) );
//     }
//
//     if (old_fpsr != fpsr)
//         __asm__ __volatile__( "msr fpsr, %0" :: "r" (fpsr) );
//     if (old_fpcr != fpcr)
//         __asm__ __volatile__( "msr fpcr, %0" :: "r" (fpcr) );
// #elif defined(__arm__)
//     DWORD old_fpscr, fpscr;
//     unsigned int flags;
//
//     __asm__ __volatile__( "vmrs %0, fpscr" : "=r" (fpscr) );
//     old_fpscr = fpscr;
//
//     cw_mask &= _MCW_EM | _MCW_RC;
//     sw_mask &= _MCW_EM;
//
//     if (sw)
//     {
//         flags = 0;
//         if (fpscr & 0x1) flags |= _SW_INVALID;
//         if (fpscr & 0x2) flags |= _SW_ZERODIVIDE;
//         if (fpscr & 0x4) flags |= _SW_OVERFLOW;
//         if (fpscr & 0x8) flags |= _SW_UNDERFLOW;
//         if (fpscr & 0x10) flags |= _SW_INEXACT;
//         if (fpscr & 0x80) flags |= _SW_DENORMAL;
//
//         *sw = (flags & ~sw_mask) | (*sw & sw_mask);
//         TRACE("arm update sw %08x to %08x\n", flags, *sw);
//         fpscr &= ~0x9f;
//         if (*sw & _SW_INVALID) fpscr |= 0x1;
//         if (*sw & _SW_ZERODIVIDE) fpscr |= 0x2;
//         if (*sw & _SW_OVERFLOW) fpscr |= 0x4;
//         if (*sw & _SW_UNDERFLOW) fpscr |= 0x8;
//         if (*sw & _SW_INEXACT) fpscr |= 0x10;
//         if (*sw & _SW_DENORMAL) fpscr |= 0x80;
//         *sw = flags;
//     }
//
//     if (cw)
//     {
//         flags = 0;
//         if (!(fpscr & 0x100)) flags |= _EM_INVALID;
//         if (!(fpscr & 0x200)) flags |= _EM_ZERODIVIDE;
//         if (!(fpscr & 0x400)) flags |= _EM_OVERFLOW;
//         if (!(fpscr & 0x800)) flags |= _EM_UNDERFLOW;
//         if (!(fpscr & 0x1000)) flags |= _EM_INEXACT;
//         if (!(fpscr & 0x8000)) flags |= _EM_DENORMAL;
//         switch (fpscr & 0xc00000)
//         {
//         case 0x400000: flags |= _RC_UP; break;
//         case 0x800000: flags |= _RC_DOWN; break;
//         case 0xc00000: flags |= _RC_CHOP; break;
//         }
//
//         *cw = (flags & ~cw_mask) | (*cw & cw_mask);
//         TRACE("arm update cw %08x to %08x\n", flags, *cw);
//         fpscr &= ~0xc09f00ul;
//         if (!(*cw & _EM_INVALID)) fpscr |= 0x100;
//         if (!(*cw & _EM_ZERODIVIDE)) fpscr |= 0x200;
//         if (!(*cw & _EM_OVERFLOW)) fpscr |= 0x400;
//         if (!(*cw & _EM_UNDERFLOW)) fpscr |= 0x800;
//         if (!(*cw & _EM_INEXACT)) fpscr |= 0x1000;
//         if (!(*cw & _EM_DENORMAL)) fpscr |= 0x8000;
//         switch (*cw & _MCW_RC)
//         {
//         case _RC_CHOP: fpscr |= 0xc00000; break;
//         case _RC_UP: fpscr |= 0x400000; break;
//         case _RC_DOWN: fpscr |= 0x800000; break;
//         }
//     }
//
//     if (old_fpscr != fpscr)
//         __asm__ __volatile__( "vmsr fpscr, %0" :: "r" (fpscr) );
// #else
//     FIXME("not implemented\n");
//     if (cw) *cw = 0;
//     if (sw) *sw = 0;
// #endif
// }
//
// /**********************************************************************
//  *		_statusfp2 (MSVCR80.@)
//  */
// #if defined(__i386__)
// void CDECL _statusfp2( unsigned int *x86_sw, unsigned int *sse2_sw )
// {
//     if (x86_sw)
//         _setfp(NULL, 0, x86_sw, 0);
//     if (!sse2_sw) return;
//     if (sse2_supported)
//         _setfp_sse(NULL, 0, sse2_sw, 0);
//     else *sse2_sw = 0;
// }
// #endif
//
// /**********************************************************************
//  *		_statusfp (MSVCRT.@)
//  */
// unsigned int CDECL _statusfp(void)
// {
//     unsigned int flags = 0;
// #if defined(__i386__)
//     unsigned int x86_sw, sse2_sw;
//
//     _statusfp2( &x86_sw, &sse2_sw );
//     /* FIXME: there's no definition for ambiguous status, just return all status bits for now */
//     flags = x86_sw | sse2_sw;
// #else
//     _setfp(NULL, 0, &flags, 0);
// #endif
//     return flags;
// }
//
// /*********************************************************************
//  *		_clearfp (MSVCRT.@)
//  */
// unsigned int CDECL _clearfp(void)
// {
//     unsigned int flags = 0;
// #ifdef __i386__
//     _setfp(NULL, 0, &flags, _MCW_EM);
//     if (sse2_supported)
//     {
//         unsigned int sse_sw = 0;
//
//         _setfp_sse(NULL, 0, &sse_sw, _MCW_EM);
//         flags |= sse_sw;
//     }
// #else
//     _setfp(NULL, 0, &flags, _MCW_EM);
// #endif
//     return flags;
// }
//
// /*********************************************************************
//  *		__fpecode (MSVCRT.@)
//  */
// int *  __fpecode(void)
// {
//     return &msvcrt_get_thread_data()->fpecode;
// }
//
// #ifndef __REACTOS__
// /*********************************************************************
//  *		ldexp (MSVCRT.@)
//  */
// double CDECL ldexp(double num, int exp)
// {
//   double z = scalbn(num, exp);
//
//   if (isfinite(num) && !isfinite(z))
//     return math_error(_OVERFLOW, "ldexp", num, exp, z);
//   if (num && isfinite(num) && !z)
//     return math_error(_UNDERFLOW, "ldexp", num, exp, z);
//   return z;
// }
// #endif // __REACTOS__
//
// /*********************************************************************
//  *		_cabs (MSVCRT.@)
//  */
// double CDECL _cabs(struct _complex num)
// {
//   return sqrt(num.x * num.x + num.y * num.y);
// }
//
// /*********************************************************************
//  *		_chgsign (MSVCRT.@)
//  */
// double CDECL _chgsign(double num)
// {
//     union { double f; UINT64 i; } u = { num };
//     u.i ^= 1u64 << 63;
//     return u.f;
// }
//
// /*********************************************************************
//  *		__control87_2 (MSVCR80.@)
//  *
//  * Not exported by native msvcrt, added in msvcr80.
//  */
// #ifdef __i386__
// int CDECL __control87_2( unsigned int newval, unsigned int mask,
//                          unsigned int *x86_cw, unsigned int *sse2_cw )
// {
//     if (x86_cw)
//     {
//         *x86_cw = newval;
//         _setfp(x86_cw, mask, NULL, 0);
//     }
//
//     if (!sse2_cw) return 1;
//
//     if (sse2_supported)
//     {
//         *sse2_cw = newval;
//         _setfp_sse(sse2_cw, mask, NULL, 0);
//     }
//     else *sse2_cw = 0;
//
//     return 1;
// }
// #endif
//
// /*********************************************************************
//  *		_control87 (MSVCRT.@)
//  */
// unsigned int CDECL _control87(unsigned int newval, unsigned int mask)
// {
//     unsigned int flags = 0;
// #if defined(__i386__) && (_MSVCR_VER == 0 || _MSVCR_VER >= 80)
//     unsigned int sse2_cw;
//
//     __control87_2( newval, mask, &flags, &sse2_cw );
//
//     if (sse2_supported)
//     {
//         if ((flags ^ sse2_cw) & (_MCW_EM | _MCW_RC)) flags |= _EM_AMBIGUOUS;
//         flags |= sse2_cw;
//     }
// #else
//     flags = newval;
//     _setfp(&flags, mask, NULL, 0);
// #endif
//     return flags;
// }
//
// /*********************************************************************
//  *		_controlfp (MSVCRT.@)
//  */
// unsigned int CDECL _controlfp(unsigned int newval, unsigned int mask)
// {
//   return _control87( newval, mask & ~_EM_DENORMAL );
// }
//
// /*********************************************************************
//  *		_set_controlfp (MSVCRT.@)
//  */
// void CDECL _set_controlfp( unsigned int newval, unsigned int mask )
// {
//     _controlfp( newval, mask );
// }
//
// /*********************************************************************
//  *              _controlfp_s (MSVCRT.@)
//  */
// int CDECL _controlfp_s(unsigned int *cur, unsigned int newval, unsigned int mask)
// {
//     static const unsigned int all_flags = (_MCW_EM | _MCW_IC | _MCW_RC |
//                                            _MCW_PC | _MCW_DN);
//     unsigned int val;
//
//     if (!MSVCRT_CHECK_PMT( !(newval & mask & ~all_flags) ))
//     {
//         if (cur) *cur = _controlfp( 0, 0 );  /* retrieve it anyway */
//         return EINVAL as i32;
//     }
//     val = _controlfp( newval, mask );
//     if (cur) *cur = val;
//     return 0;
// }
//
// #if _MSVCR_VER >= 140 && (defined(__i386__) || defined(__x86_64__))
// enum fenv_masks
// {
//     FENV_X_INVALID = 0x00100010,
//     FENV_X_DENORMAL = 0x00200020,
//     FENV_X_ZERODIVIDE = 0x00080008,
//     FENV_X_OVERFLOW = 0x00040004,
//     FENV_X_UNDERFLOW = 0x00020002,
//     FENV_X_INEXACT = 0x00010001,
//     FENV_X_AFFINE = 0x00004000,
//     FENV_X_UP = 0x00800200,
//     FENV_X_DOWN = 0x00400100,
//     FENV_X_24 = 0x00002000,
//     FENV_X_53 = 0x00001000,
//     FENV_Y_INVALID = 0x10000010,
//     FENV_Y_DENORMAL = 0x20000020,
//     FENV_Y_ZERODIVIDE = 0x08000008,
//     FENV_Y_OVERFLOW = 0x04000004,
//     FENV_Y_UNDERFLOW = 0x02000002,
//     FENV_Y_INEXACT = 0x01000001,
//     FENV_Y_UP = 0x80000200,
//     FENV_Y_DOWN = 0x40000100,
//     FENV_Y_FLUSH = 0x00000400,
//     FENV_Y_FLUSH_SAVE = 0x00000800
// };
//
// /* encodes x87/sse control/status word in ulong */
// static __msvcrt_ulong fenv_encode(unsigned int x, unsigned int y)
// {
//     __msvcrt_ulong ret = 0;
//
// #ifdef __i386__
//     if (x & _EM_INVALID) ret |= FENV_X_INVALID;
//     if (x & _EM_DENORMAL) ret |= FENV_X_DENORMAL;
//     if (x & _EM_ZERODIVIDE) ret |= FENV_X_ZERODIVIDE;
//     if (x & _EM_OVERFLOW) ret |= FENV_X_OVERFLOW;
//     if (x & _EM_UNDERFLOW) ret |= FENV_X_UNDERFLOW;
//     if (x & _EM_INEXACT) ret |= FENV_X_INEXACT;
//     if (x & _IC_AFFINE) ret |= FENV_X_AFFINE;
//     if (x & _RC_UP) ret |= FENV_X_UP;
//     if (x & _RC_DOWN) ret |= FENV_X_DOWN;
//     if (x & _PC_24) ret |= FENV_X_24;
//     if (x & _PC_53) ret |= FENV_X_53;
// #endif
//     x &= ~(_MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC);
//
//     if (y & _EM_INVALID) ret |= FENV_Y_INVALID;
//     if (y & _EM_DENORMAL) ret |= FENV_Y_DENORMAL;
//     if (y & _EM_ZERODIVIDE) ret |= FENV_Y_ZERODIVIDE;
//     if (y & _EM_OVERFLOW) ret |= FENV_Y_OVERFLOW;
//     if (y & _EM_UNDERFLOW) ret |= FENV_Y_UNDERFLOW;
//     if (y & _EM_INEXACT) ret |= FENV_Y_INEXACT;
//     if (y & _RC_UP) ret |= FENV_Y_UP;
//     if (y & _RC_DOWN) ret |= FENV_Y_DOWN;
//     if (y & _DN_FLUSH) ret |= FENV_Y_FLUSH;
//     if (y & _DN_FLUSH_OPERANDS_SAVE_RESULTS) ret |= FENV_Y_FLUSH_SAVE;
//     y &= ~(_MCW_EM | _MCW_IC | _MCW_RC | _MCW_DN);
//
//     if(x || y) FIXME("unsupported flags: %x, %x\n", x, y);
//     return ret;
// }
//
// /* decodes x87/sse control/status word, returns FALSE on error */
// static BOOL fenv_decode(__msvcrt_ulong enc, unsigned int *x, unsigned int *y)
// {
//     *x = *y = 0;
//     if ((enc & FENV_X_INVALID) == FENV_X_INVALID) *x |= _EM_INVALID;
//     if ((enc & FENV_X_DENORMAL) == FENV_X_DENORMAL) *x |= _EM_DENORMAL;
//     if ((enc & FENV_X_ZERODIVIDE) == FENV_X_ZERODIVIDE) *x |= _EM_ZERODIVIDE;
//     if ((enc & FENV_X_OVERFLOW) == FENV_X_OVERFLOW) *x |= _EM_OVERFLOW;
//     if ((enc & FENV_X_UNDERFLOW) == FENV_X_UNDERFLOW) *x |= _EM_UNDERFLOW;
//     if ((enc & FENV_X_INEXACT) == FENV_X_INEXACT) *x |= _EM_INEXACT;
//     if ((enc & FENV_X_AFFINE) == FENV_X_AFFINE) *x |= _IC_AFFINE;
//     if ((enc & FENV_X_UP) == FENV_X_UP) *x |= _RC_UP;
//     if ((enc & FENV_X_DOWN) == FENV_X_DOWN) *x |= _RC_DOWN;
//     if ((enc & FENV_X_24) == FENV_X_24) *x |= _PC_24;
//     if ((enc & FENV_X_53) == FENV_X_53) *x |= _PC_53;
//
//     if ((enc & FENV_Y_INVALID) == FENV_Y_INVALID) *y |= _EM_INVALID;
//     if ((enc & FENV_Y_DENORMAL) == FENV_Y_DENORMAL) *y |= _EM_DENORMAL;
//     if ((enc & FENV_Y_ZERODIVIDE) == FENV_Y_ZERODIVIDE) *y |= _EM_ZERODIVIDE;
//     if ((enc & FENV_Y_OVERFLOW) == FENV_Y_OVERFLOW) *y |= _EM_OVERFLOW;
//     if ((enc & FENV_Y_UNDERFLOW) == FENV_Y_UNDERFLOW) *y |= _EM_UNDERFLOW;
//     if ((enc & FENV_Y_INEXACT) == FENV_Y_INEXACT) *y |= _EM_INEXACT;
//     if ((enc & FENV_Y_UP) == FENV_Y_UP) *y |= _RC_UP;
//     if ((enc & FENV_Y_DOWN) == FENV_Y_DOWN) *y |= _RC_DOWN;
//     if ((enc & FENV_Y_FLUSH) == FENV_Y_FLUSH) *y |= _DN_FLUSH;
//     if ((enc & FENV_Y_FLUSH_SAVE) == FENV_Y_FLUSH_SAVE) *y |= _DN_FLUSH_OPERANDS_SAVE_RESULTS;
//
//     if (fenv_encode(*x, *y) != enc)
//     {
//         WARN("can't decode: %lx\n", enc);
//         return FALSE;
//     }
//     return TRUE;
// }
// #elif _MSVCR_VER >= 120
// static __msvcrt_ulong fenv_encode(unsigned int x, unsigned int y)
// {
//     if (y & _EM_DENORMAL)
//         y = (y & ~_EM_DENORMAL) | 0x20;
//
//     return x | y;
// }
//
// static BOOL fenv_decode(__msvcrt_ulong enc, unsigned int *x, unsigned int *y)
// {
//     if (enc & 0x20)
//         enc = (enc & ~0x20) | _EM_DENORMAL;
//
//     *x = *y = enc;
//     return TRUE;
// }
// #endif
//
//
//
//
// /*********************************************************************
//  *		_finite (MSVCRT.@)
//  */
// int CDECL _finite(double num)
// {
//     union { double f; UINT64 i; } u = { num };
//     return (u.i & ~0ull >> 1) < 0x7ffu64 << 52;
// }
//
// /*********************************************************************
//  *		_fpreset (MSVCRT.@)
//  */
// void CDECL _fpreset(void)
// {
// #if (defined(__GNUC__) || defined(__clang__)) && defined(__i386__)
//     const unsigned int x86_cw = 0x27f;
//     __asm__ __volatile__( "fninit; fldcw %0" : : "m" (x86_cw) );
//     if (sse2_supported)
//     {
//         unsigned int cw = _MCW_EM, sw = 0;
//         _setfp_sse(&cw, ~0, &sw, ~0);
//     }
// #else
//     unsigned int cw = _MCW_EM, sw = 0;
//     _setfp(&cw, ~0, &sw, ~0);
// #endif
// }
//
//
// /*********************************************************************
//  *		_isnan (MSVCRT.@)
//  */
// int CDECL _isnan(double num)
// {
//     union { double f; UINT64 i; } u = { num };
//     return (u.i & ~0ull >> 1) > 0x7ffu64 << 52;
// }
//
//
// /*********************************************************************
//  *		_ecvt (MSVCRT.@)
//  */
// char *  _ecvt( double number, int ndigits, int *decpt, int *sign )
// {
//     int prec, len;
//     thread_data_t *data = msvcrt_get_thread_data();
//     /* FIXME: check better for overflow (native supports over 300 chars) */
//     ndigits = min( ndigits, 80 - 8); /* 8 : space for sign, dec point, "e",
//                                       * 4 for exponent and one for
//                                       * terminating '\0' */
//     if (!data->efcvt_buffer)
//         data->efcvt_buffer = malloc( 80 ); /* ought to be enough */
//
//     /* handle cases with zero ndigits or less */
//     prec = ndigits;
//     if( prec < 1) prec = 2;
//     len = _snprintf(data->efcvt_buffer, 80, "%.*le", prec - 1, number);
//
//     if (data->efcvt_buffer[0] == '-') {
//         memmove( data->efcvt_buffer, data->efcvt_buffer + 1, len-- );
//         *sign = 1;
//     } else *sign = 0;
//
//     /* take the decimal "point away */
//     if( prec != 1)
//         memmove( data->efcvt_buffer + 1, data->efcvt_buffer + 2, len - 1 );
//     /* take the exponential "e" out */
//     data->efcvt_buffer[ prec] = '\0';
//     /* read the exponent */
//     sscanf( data->efcvt_buffer + prec + 1, "%d", decpt);
//     (*decpt)++;
//     /* adjust for some border cases */
//     if( data->efcvt_buffer[0] == '0')/* value is zero */
//         *decpt = 0;
//     /* handle cases with zero ndigits or less */
//     if( ndigits < 1){
//         if( data->efcvt_buffer[ 0] >= '5')
//             (*decpt)++;
//         data->efcvt_buffer[ 0] = '\0';
//     }
//     TRACE("out=\"%s\"\n",data->efcvt_buffer);
//     return data->efcvt_buffer;
// }
//
// /*********************************************************************
//  *		_ecvt_s (MSVCRT.@)
//  */
// int CDECL _ecvt_s( char *buffer, size_t length, double number, int ndigits, int *decpt, int *sign )
// {
//     int prec, len;
//     char *result;
//
//     if (!MSVCRT_CHECK_PMT(buffer != NULL)) return EINVAL as i32;
//     if (!MSVCRT_CHECK_PMT(decpt != NULL)) return EINVAL as i32;
//     if (!MSVCRT_CHECK_PMT(sign != NULL)) return EINVAL as i32;
//     if (!MSVCRT_CHECK_PMT_ERR( length > 2, ERANGE )) return ERANGE as i32;
//     if (!MSVCRT_CHECK_PMT_ERR(ndigits < (int)length - 1, ERANGE )) return ERANGE as i32;
//
//     /* handle cases with zero ndigits or less */
//     prec = ndigits;
//     if( prec < 1) prec = 2;
//     result = malloc(prec + 8);
//
//     len = _snprintf(result, prec + 8, "%.*le", prec - 1, number);
//     if (result[0] == '-') {
//         memmove( result, result + 1, len-- );
//         *sign = 1;
//     } else *sign = 0;
//
//     /* take the decimal "point away */
//     if( prec != 1)
//         memmove( result + 1, result + 2, len - 1 );
//     /* take the exponential "e" out */
//     result[ prec] = '\0';
//     /* read the exponent */
//     sscanf( result + prec + 1, "%d", decpt);
//     (*decpt)++;
//     /* adjust for some border cases */
//     if( result[0] == '0')/* value is zero */
//         *decpt = 0;
//     /* handle cases with zero ndigits or less */
//     if( ndigits < 1){
//         if( result[ 0] >= '5')
//             (*decpt)++;
//         result[ 0] = '\0';
//     }
//     memcpy( buffer, result, max(ndigits + 1, 1) );
//     free( result );
//     return 0;
// }
//
// /***********************************************************************
//  *		_fcvt  (MSVCRT.@)
//  */
// char *  _fcvt( double number, int ndigits, int *decpt, int *sign )
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     int stop, dec1, dec2;
//     char *ptr1, *ptr2, *first;
//     char buf[80]; /* ought to be enough */
//     char decimal_separator = get_locinfo()->lconv->decimal_point[0];
//
//     if (!data->efcvt_buffer)
//         data->efcvt_buffer = malloc( 80 ); /* ought to be enough */
//
//     stop = _snprintf(buf, 80, "%.*f", ndigits < 0 ? 0 : ndigits, number);
//     ptr1 = buf;
//     ptr2 = data->efcvt_buffer;
//     first = NULL;
//     dec1 = 0;
//     dec2 = 0;
//
//     if (*ptr1 == '-') {
//         *sign = 1;
//         ptr1++;
//     } else *sign = 0;
//
//     /* For numbers below the requested resolution, work out where
//        the decimal point will be rather than finding it in the string */
//     if (number < 1.0 && number > 0.0) {
// 	dec2 = log10(number + 1e-10);
// 	if (-dec2 <= ndigits) dec2 = 0;
//     }
//
//     /* If requested digits is zero or less, we will need to truncate
//      * the returned string */
//     if (ndigits < 1) {
// 	stop += ndigits;
//     }
//
//     while (*ptr1 == '0') ptr1++; /* Skip leading zeroes */
//     while (*ptr1 != '\0' && *ptr1 != decimal_separator) {
// 	if (!first) first = ptr2;
// 	if ((ptr1 - buf) < stop) {
// 	    *ptr2++ = *ptr1++;
// 	} else {
// 	    ptr1++;
// 	}
// 	dec1++;
//     }
//
//     if (ndigits > 0) {
// 	ptr1++;
// 	if (!first) {
// 	    while (*ptr1 == '0') { /* Process leading zeroes */
// 		*ptr2++ = *ptr1++;
// 		dec1--;
// 	    }
// 	}
// 	while (*ptr1 != '\0') {
// 	    if (!first) first = ptr2;
// 	    *ptr2++ = *ptr1++;
// 	}
//     }
//
//     *ptr2 = '\0';
//
//     /* We never found a non-zero digit, then our number is either
//      * smaller than the requested precision, or 0.0 */
//     if (!first) {
// 	if (number > 0.0) {
// 	    first = ptr2;
// 	} else {
// 	    first = data->efcvt_buffer;
// 	    dec1 = 0;
// 	}
//     }
//
//     *decpt = dec2 ? dec2 : dec1;
//     return first;
// }
//
// /***********************************************************************
//  *		_fcvt_s  (MSVCRT.@)
//  */
// int CDECL _fcvt_s(char* outbuffer, size_t size, double number, int ndigits, int *decpt, int *sign)
// {
//     int stop, dec1, dec2;
//     char *ptr1, *ptr2, *first;
//     char buf[80]; /* ought to be enough */
//     char decimal_separator = get_locinfo()->lconv->decimal_point[0];
//
//     if (!outbuffer || !decpt || !sign || size == 0)
//     {
//         *_errno() = EINVAL as i32;
//         return EINVAL as i32;
//     }
//
//     stop = _snprintf(buf, 80, "%.*f", ndigits < 0 ? 0 : ndigits, number);
//     ptr1 = buf;
//     ptr2 = outbuffer;
//     first = NULL;
//     dec1 = 0;
//     dec2 = 0;
//
//     if (*ptr1 == '-') {
//         *sign = 1;
//         ptr1++;
//     } else *sign = 0;
//
//     /* For numbers below the requested resolution, work out where
//        the decimal point will be rather than finding it in the string */
//     if (number < 1.0 && number > 0.0) {
// 	dec2 = log10(number + 1e-10);
// 	if (-dec2 <= ndigits) dec2 = 0;
//     }
//
//     /* If requested digits is zero or less, we will need to truncate
//      * the returned string */
//     if (ndigits < 1) {
// 	stop += ndigits;
//     }
//
//     while (*ptr1 == '0') ptr1++; /* Skip leading zeroes */
//     while (*ptr1 != '\0' && *ptr1 != decimal_separator) {
// 	if (!first) first = ptr2;
// 	if ((ptr1 - buf) < stop) {
// 	    if (size > 1) {
//                 *ptr2++ = *ptr1++;
//                 size--;
//             }
// 	} else {
// 	    ptr1++;
// 	}
// 	dec1++;
//     }
//
//     if (ndigits > 0) {
// 	ptr1++;
// 	if (!first) {
// 	    while (*ptr1 == '0') { /* Process leading zeroes */
//                 if (number == 0.0 && size > 1) {
//                     *ptr2++ = '0';
//                     size--;
//                 }
//                 ptr1++;
// 		dec1--;
// 	    }
// 	}
// 	while (*ptr1 != '\0') {
// 	    if (!first) first = ptr2;
// 	    if (size > 1) {
//                 *ptr2++ = *ptr1++;
//                 size--;
//             }
// 	}
//     }
//
//     *ptr2 = '\0';
//
//     /* We never found a non-zero digit, then our number is either
//      * smaller than the requested precision, or 0.0 */
//     if (!first && (number <= 0.0))
//         dec1 = 0;
//
//     *decpt = dec2 ? dec2 : dec1;
//     return 0;
// }
//
// /***********************************************************************
//  *		_gcvt  (MSVCRT.@)
//  */
// char *  _gcvt( double number, int ndigit, char *buff )
// {
//     if(!buff) {
//         *_errno() = EINVAL as i32;
//         return NULL;
//     }
//
//     if(ndigit < 0) {
//         *_errno() = ERANGE as i32;
//         return NULL;
//     }
//
//     sprintf(buff, "%.*g", ndigit, number);
//     return buff;
// }
//
// /***********************************************************************
//  *              _gcvt_s  (MSVCRT.@)
//  */
// int CDECL _gcvt_s(char *buff, size_t size, double number, int digits)
// {
//     int len;
//
//     if(!buff) {
//         *_errno() = EINVAL as i32;
//         return EINVAL as i32;
//     }
//
//     if( digits<0 || digits>=size) {
//         if(size)
//             buff[0] = '\0';
//
//         *_errno() = ERANGE as i32;
//         return ERANGE as i32;
//     }
//
//     len = _scprintf("%.*g", digits, number);
//     if(len > size) {
//         buff[0] = '\0';
//         *_errno() = ERANGE as i32;
//         return ERANGE as i32;
//     }
//
//     sprintf(buff, "%.*g", digits, number);
//     return 0;
// }
//
// #include <stdlib.h> /* div_t, ldiv_t */
//
// #ifndef __REACTOS__
// /*********************************************************************
//  *		div (MSVCRT.@)
//  * VERSION
//  *	[i386] Windows binary compatible - returns the struct in eax/edx.
//  */
// #ifdef __i386__
// unsigned __int64 CDECL div(int num, int denom)
// {
//     union {
//         div_t div;
//         unsigned __int64 uint64;
//     } ret;
//
//     ret.div.quot = num / denom;
//     ret.div.rem = num % denom;
//     return ret.uint64;
// }
// #else
// /*********************************************************************
//  *		div (MSVCRT.@)
//  * VERSION
//  *	[!i386] Non-x86 can't run win32 apps so we don't need binary compatibility
//  */
// div_t CDECL div(int num, int denom)
// {
//     div_t ret;
//
//     ret.quot = num / denom;
//     ret.rem = num % denom;
//     return ret;
// }
// #endif /* ifdef __i386__ */
//
//
// /*********************************************************************
//  *		ldiv (MSVCRT.@)
//  * VERSION
//  * 	[i386] Windows binary compatible - returns the struct in eax/edx.
//  */
// #ifdef __i386__
// unsigned __int64 CDECL ldiv(__msvcrt_long num, __msvcrt_long denom)
// {
//     union {
//         ldiv_t ldiv;
//         unsigned __int64 uint64;
//     } ret;
//
//     ret.ldiv.quot = num / denom;
//     ret.ldiv.rem = num % denom;
//     return ret.uint64;
// }
// #else
// /*********************************************************************
//  *		ldiv (MSVCRT.@)
//  * VERSION
//  *	[!i386] Non-x86 can't run win32 apps so we don't need binary compatibility
//  */
// ldiv_t CDECL ldiv(__msvcrt_long num, __msvcrt_long denom)
// {
//     ldiv_t ret;
//
//     ret.quot = num / denom;
//     ret.rem = num % denom;
//     return ret;
// }
// #endif /* ifdef __i386__ */
// #endif /* !__REACTOS__ */
//
//
// #ifdef __i386__
//
// /*********************************************************************
//  *		_adjust_fdiv (MSVCRT.@)
//  * Used by the MSVC compiler to work around the Pentium FDIV bug.
//  */
// int MSVCRT__adjust_fdiv = 0;
//
// /***********************************************************************
//  *		_adj_fdiv_m16i (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdiv_m16i( short arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdiv_m32 (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdiv_m32( unsigned int arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdiv_m32i (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdiv_m32i( int arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdiv_m64 (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdiv_m64( unsigned __int64 arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdiv_r (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _adj_fdiv_r(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdivr_m16i (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdivr_m16i( short arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdivr_m32 (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdivr_m32( unsigned int arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdivr_m32i (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdivr_m32i( int arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fdivr_m64 (MSVCRT.@)
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void __stdcall _adj_fdivr_m64( unsigned __int64 arg )
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fpatan (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _adj_fpatan(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fprem (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _adj_fprem(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fprem1 (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _adj_fprem1(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_adj_fptan (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _adj_fptan(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_safe_fdiv (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _safe_fdiv(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_safe_fdivr (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _safe_fdivr(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_safe_fprem (MSVCRT.@)
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _safe_fprem(void)
// {
//   TRACE("(): stub\n");
// }
//
// /***********************************************************************
//  *		_safe_fprem1 (MSVCRT.@)
//  *
//  * FIXME
//  *    This function is likely to have the wrong number of arguments.
//  *
//  * NOTE
//  *    I _think_ this function is intended to work around the Pentium
//  *    fdiv bug.
//  */
// void _safe_fprem1(void)
// {
//   TRACE("(): stub\n");
// }
//
// #ifndef __REACTOS__
// /***********************************************************************
//  *		__libm_sse2_acos   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_acos(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = acos( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_acosf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_acosf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = acosf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_asin   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_asin(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = asin( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_asinf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_asinf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = asinf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_atan   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_atan(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = atan( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_atan2   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_atan2(void)
// {
//     double d1, d2;
//     __asm__ __volatile__( "movq %%xmm0,%0; movq %%xmm1,%1 " : "=m" (d1), "=m" (d2) );
//     d1 = atan2( d1, d2 );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d1) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_atanf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_atanf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = atanf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_cos   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_cos(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = cos( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_cosf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_cosf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = cosf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_exp   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_exp(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = exp( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_expf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_expf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = expf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_log   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_log(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = log( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_log10   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_log10(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = log10( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_log10f   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_log10f(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = log10f( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_logf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_logf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = logf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_pow   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_pow(void)
// {
//     double d1, d2;
//     __asm__ __volatile__( "movq %%xmm0,%0; movq %%xmm1,%1 " : "=m" (d1), "=m" (d2) );
//     d1 = pow( d1, d2 );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d1) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_powf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_powf(void)
// {
//     float f1, f2;
//     __asm__ __volatile__( "movd %%xmm0,%0; movd %%xmm1,%1" : "=g" (f1), "=g" (f2) );
//     f1 = powf( f1, f2 );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f1) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_sin   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_sin(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = sin( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_sinf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_sinf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = sinf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_tan   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_tan(void)
// {
//     double d;
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     d = tan( d );
//     __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_tanf   (MSVCRT.@)
//  */
// void __cdecl __libm_sse2_tanf(void)
// {
//     float f;
//     __asm__ __volatile__( "movd %%xmm0,%0" : "=g" (f) );
//     f = tanf( f );
//     __asm__ __volatile__( "movd %0,%%xmm0" : : "g" (f) );
// }
//
// /***********************************************************************
//  *		__libm_sse2_sqrt_precise   (MSVCR110.@)
//  */
// void __cdecl __libm_sse2_sqrt_precise(void)
// {
//     unsigned int cw;
//     double d;
//
//     __asm__ __volatile__( "movq %%xmm0,%0" : "=m" (d) );
//     __control87_2(0, 0, NULL, &cw);
//     if (cw & _MCW_RC)
//     {
//         d = sqrt(d);
//         __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
//         return;
//     }
//
//     if (!sqrt_validate(&d, FALSE))
//     {
//         __asm__ __volatile__( "movq %0,%%xmm0" : : "m" (d) );
//         return;
//     }
//     __asm__ __volatile__( "sqrtsd %xmm0, %xmm0" );
// }
// #endif /* !__REACTOS__ */
// #endif  /* __i386__ */
//
//
// /*********************************************************************
//  *      _scalb  (MSVCRT.@)
//  *      scalbn  (MSVCR120.@)
//  *      scalbln (MSVCR120.@)
//  */
// double CDECL _scalb(double num, __msvcrt_long power)
// {
//   return ldexp(num, power);
// }
//
// /*********************************************************************
//  *      _scalbf  (MSVCRT.@)
//  *      scalbnf  (MSVCR120.@)
//  *      scalblnf (MSVCR120.@)
//  */
// float CDECL _scalbf(float num, __msvcrt_long power)
// {
//   return ldexp(num, power);
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
