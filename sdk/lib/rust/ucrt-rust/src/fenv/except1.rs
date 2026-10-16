// ucrtbase `_except1` (MSVCR120+/UCRT, cdecl). Child module of `fenv` (src/fenv/except1.rs),
// so it reaches fenv's private `fenv_encode` / `FenvT` / `setfp` exactly as the C reaches
// math.c's statics. A PORT of Wine's dlls/msvcrt/math.c `_except1` (`_MSVCR_VER >= 120`).
//
// The compiler's libm-call wrappers call this when an FP operation raised `fpe` (a
// 0x1/0x2/0x4/0x8/0x10 overflow/underflow/zerodiv/invalid/inexact bit, +0x10 = also inexact).
// If the matching exception is MASKED in the caller's control word `cw` the flag is raised
// silently and the (possibly saturated) result returned; otherwise a structured FP exception
// is raised. Either way the FP environment is then reloaded from `cw`.
//
// `cw` is the x87 control-word layout on i386 (EM bits 0-5, RC 10-11, PC 8-9, IC bit 12) but
// the MXCSR layout on every 64-bit target (mask bits 7-12, RC 13-14); the `_WIN64` arm
// converts MXCSR -> x87 so the rest is shared. That is `target_pointer_width = "64"`, which
// is amd64 AND arm64, as the C's `#ifdef _WIN64`.

use super::{fegetenv, fenv_encode, fesetenv, setfp, FenvT, FE_ALL_EXCEPT};
use core::ffi::{c_int, c_void};

#[path = "except1_pure.rs"]
mod pure;

use pure::*;

extern "system" {
    fn RaiseException(code: u32, flags: u32, nargs: u32, args: *const usize);
}

/// `feraiseexcept` (math.c): OR the encoded flags into the environment and reinstall it.
/// Kept private and local: nothing else in the crate raises flags yet.
unsafe fn feraiseexcept(flags: u32) -> c_int {
    let mut env = FenvT { fe_ctl: 0, fe_stat: 0 };
    fegetenv(&mut env);
    let flags = flags & FE_ALL_EXCEPT;
    env.fe_stat |= fenv_encode(flags, flags);
    fesetenv(&env)
}

/// `_except1`. Unused `arg` / `unk` are unused in Wine too (its TODO: the meaning of `unk` and
/// of the ignored `op` / `cw` bits is unknown).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _except1(
    fpe: u32,
    op: i32,
    _arg: f64,
    mut res: f64,
    cw: u32,
    _unk: *mut c_void,
) -> f64 {
    let cw = normalize_cw(cw);
    // `WORD operation = op << 5;` -- the exception's single argument points at it.
    let operation: u16 = (op as u32).wrapping_shl(5) as u16;
    let exception_arg = &operation as *const u16 as usize;

    let d = decide(fpe, cw);
    match d.res_kind {
        1 => res = if res.is_sign_negative() { f64::NEG_INFINITY } else { f64::INFINITY },
        2 => res = if res.is_sign_negative() { -0.0 } else { 0.0 },
        _ => {}
    }

    feraiseexcept(d.raise);
    if d.exception != 0 {
        RaiseException(d.exception, 0, 1, &exception_arg);
    }

    let mut fpword = fpword_from_cw(cw);
    setfp(&mut fpword, MCW_EM | MCW_RC | MCW_PC | MCW_IC, core::ptr::null_mut(), 0);

    res
}
