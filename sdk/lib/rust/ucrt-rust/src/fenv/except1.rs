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

// PURE-BEGIN (no dependencies beyond core; the host test extracts this region verbatim)
#[allow(dead_code)]
mod pure {
    pub const FE_INVALID: u32 = 0x10;
    pub const FE_DIVBYZERO: u32 = 0x08;
    pub const FE_OVERFLOW: u32 = 0x04;
    pub const FE_UNDERFLOW: u32 = 0x02;
    pub const FE_INEXACT: u32 = 0x01;

    pub const EXCEPTION_FLT_DIVIDE_BY_ZERO: u32 = 0xC000_008E;
    pub const EXCEPTION_FLT_INEXACT_RESULT: u32 = 0xC000_008F;
    pub const EXCEPTION_FLT_INVALID_OPERATION: u32 = 0xC000_0090;
    pub const EXCEPTION_FLT_OVERFLOW: u32 = 0xC000_0091;
    pub const EXCEPTION_FLT_UNDERFLOW: u32 = 0xC000_0093;

    pub const EM_INVALID: u32 = 0x10;
    pub const EM_DENORMAL: u32 = 0x8_0000;
    pub const EM_ZERODIVIDE: u32 = 0x08;
    pub const EM_OVERFLOW: u32 = 0x04;
    pub const EM_UNDERFLOW: u32 = 0x02;
    pub const EM_INEXACT: u32 = 0x01;
    pub const MCW_EM: u32 = 0x8_001f;
    pub const MCW_RC: u32 = 0x300;
    pub const RC_DOWN: u32 = 0x100;
    pub const RC_UP: u32 = 0x200;
    pub const MCW_PC: u32 = 0x3_0000;
    pub const PC_24: u32 = 0x2_0000;
    pub const PC_53: u32 = 0x1_0000;
    pub const PC_64: u32 = 0x0;
    pub const MCW_IC: u32 = 0x4_0000;
    pub const IC_AFFINE: u32 = 0x4_0000;

    /// What `_except1` decides before touching any hardware state.
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub struct Decision {
        /// `FE_*` flags to raise silently (0 when an exception is raised instead).
        pub raise: u32,
        /// SEH code to raise, or 0.
        pub exception: u32,
        /// How the result is replaced: 0 = unchanged, 1 = +-infinity, 2 = +-zero (sign of res).
        pub res_kind: u8,
    }

    /// `cw` as an x87 control word: identity on i386; on 64-bit targets the C's
    /// `cw = ((cw >> 7) & 0x3f) | ((cw >> 3) & 0xc00)` (MXCSR mask bits 7-12 -> 0-5,
    /// MXCSR RC bits 13-14 -> 10-11). PC/IC (bits 8-9, 12) are not produced by it.
    pub const fn normalize_cw(cw: u32) -> u32 {
        if cfg!(target_pointer_width = "64") {
            ((cw >> 7) & 0x3f) | ((cw >> 3) & 0xc00)
        } else {
            cw
        }
    }

    /// The `if (fpe & ...)` ladder. `cw` is already x87-layout. Only the FIRST matching
    /// class is considered (overflow shadows underflow, ...), as the C's else-if chain.
    pub const fn decide(fpe: u32, cw: u32) -> Decision {
        let mut d = Decision { raise: 0, exception: 0, res_kind: 0 };
        if fpe & 0x1 != 0 {
            // overflow: masked -> raise, and saturate to +-inf
            if (fpe == 0x1 && cw & 0x8 != 0) || (fpe == 0x11 && cw & 0x28 != 0) {
                d.raise |= FE_OVERFLOW;
                if fpe & 0x10 != 0 {
                    d.raise |= FE_INEXACT;
                }
                d.res_kind = 1;
            } else {
                d.exception = EXCEPTION_FLT_OVERFLOW;
            }
        } else if fpe & 0x2 != 0 {
            if (fpe == 0x2 && cw & 0x10 != 0) || (fpe == 0x12 && cw & 0x30 != 0) {
                d.raise |= FE_UNDERFLOW;
                if fpe & 0x10 != 0 {
                    d.raise |= FE_INEXACT;
                }
                d.res_kind = 2;
            } else {
                d.exception = EXCEPTION_FLT_UNDERFLOW;
            }
        } else if fpe & 0x4 != 0 {
            if (fpe == 0x4 && cw & 0x4 != 0) || (fpe == 0x14 && cw & 0x24 != 0) {
                d.raise |= FE_DIVBYZERO;
                if fpe & 0x10 != 0 {
                    d.raise |= FE_INEXACT;
                }
            } else {
                d.exception = EXCEPTION_FLT_DIVIDE_BY_ZERO;
            }
        } else if fpe & 0x8 != 0 {
            if fpe == 0x8 && cw & 0x1 != 0 {
                d.raise |= FE_INVALID;
            } else {
                d.exception = EXCEPTION_FLT_INVALID_OPERATION;
            }
        } else if fpe & 0x10 != 0 {
            if fpe == 0x10 && cw & 0x20 != 0 {
                d.raise |= FE_INEXACT;
            } else {
                d.exception = EXCEPTION_FLT_INEXACT_RESULT;
            }
        }
        // `if (exception) raise = 0;`
        if d.exception != 0 {
            d.raise = 0;
        }
        d
    }

    /// The CRT `_EM_*|_RC_*|_PC_*|_IC_*` word `_setfp` is loaded with, from an x87-layout `cw`.
    pub const fn fpword_from_cw(cw: u32) -> u32 {
        let mut w = 0;
        if cw & 0x1 != 0 {
            w |= EM_INVALID;
        }
        if cw & 0x2 != 0 {
            w |= EM_DENORMAL;
        }
        if cw & 0x4 != 0 {
            w |= EM_ZERODIVIDE;
        }
        if cw & 0x8 != 0 {
            w |= EM_OVERFLOW;
        }
        if cw & 0x10 != 0 {
            w |= EM_UNDERFLOW;
        }
        if cw & 0x20 != 0 {
            w |= EM_INEXACT;
        }
        match cw & 0xc00 {
            0xc00 => w |= RC_UP | RC_DOWN,
            0x800 => w |= RC_UP,
            0x400 => w |= RC_DOWN,
            _ => {}
        }
        match cw & 0x300 {
            0x0 => w |= PC_24,
            0x200 => w |= PC_53,
            0x300 => w |= PC_64,
            _ => {}
        }
        if cw & 0x1000 != 0 {
            w |= IC_AFFINE;
        }
        w
    }
}
// PURE-END

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
