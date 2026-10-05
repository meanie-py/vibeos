//! The C99 floating-point environment -- `fegetenv`, `fesetenv`, `feholdexcept`,
//! `fegetround`, `fesetround`, `fetestexcept`, `feclearexcept`, `fegetexceptflag`,
//! `fesetexceptflag` -- with `__fpe_flt_rounds` and `_fpreset`, and on i386 the
//! x87+SSE pair `__control87_2` / `_statusfp2` they are built on. Every one was a
//! `-stub` export of ucrtbase until this module: a DbgPrint and `return 0`, so
//! `fegetround()` always said "to nearest" and `fesetround` silently did nothing,
//! and ucrt-rust's own float formatting and parsing called the stubs on every
//! conversion.
//!
//! A PORT of Wine's `dlls/msvcrt/math.c` (`dll/win32/msvcrt/math.c` in this tree)
//! at `_MSVCR_VER >= 140`, the ucrtbase configuration. The register-level pieces
//! (`_setfp` for x87, `_setfp_sse` for MXCSR) are taken from msvcrt-crt-rust's port
//! of the same file, which ucrtbase may not link; arm64 uses `fpctrl_arm64`'s.
//!
//! Details that are silent if wrong:
//! * `fenv_t` is two `unsigned long`s in an ENCODED form, not raw control words:
//!   on i386 and amd64 each `_EM_*`/`_RC_*`/... bit of the x87 word and of the SSE
//!   word maps to its own `FENV_X_*` / `FENV_Y_*` pattern (Wine matched native's),
//!   so an x87-only and an SSE-only setting are distinguishable; arm64 stores the
//!   word itself with `_EM_DENORMAL` folded to 0x20. `cvt/cfout.rs` reads
//!   `_Fe_ctl & 0x1f`, which both encodings keep meaning "these are masked".
//! * amd64 reads through `_control87`/`_statusfp` (sdk/lib/crt's), i386 through
//!   `__control87_2`/`_statusfp2` below, and both write through `_setfp`, as the C.
//! * `fesetenv` of an all-zero `fenv_t` is `_fpreset`, not "unmask everything".

// Each arch uses its own subset of the float.h constants below.
#![allow(dead_code)]

use core::ffi::c_int;

const _SW_INEXACT: u32 = 0x1;
const _SW_UNDERFLOW: u32 = 0x2;
const _SW_OVERFLOW: u32 = 0x4;
const _SW_ZERODIVIDE: u32 = 0x8;
const _SW_INVALID: u32 = 0x10;
const _SW_DENORMAL: u32 = 0x8_0000;

const _EM_INEXACT: u32 = 0x1;
const _EM_UNDERFLOW: u32 = 0x2;
const _EM_OVERFLOW: u32 = 0x4;
const _EM_ZERODIVIDE: u32 = 0x8;
const _EM_INVALID: u32 = 0x10;
const _EM_DENORMAL: u32 = 0x8_0000;
const _MCW_EM: u32 = 0x8_001f;

const _MCW_RC: u32 = 0x300;
const _RC_NEAR: u32 = 0x0;
const _RC_DOWN: u32 = 0x100;
const _RC_UP: u32 = 0x200;
const _RC_CHOP: u32 = 0x300;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _MCW_PC: u32 = 0x3_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _PC_24: u32 = 0x2_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _PC_53: u32 = 0x1_0000;
#[cfg(target_arch = "x86")]
const _PC_64: u32 = 0x0;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _MCW_IC: u32 = 0x4_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _IC_AFFINE: u32 = 0x4_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _MCW_DN: u32 = 0x300_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _DN_FLUSH: u32 = 0x100_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _DN_FLUSH_OPERANDS_SAVE_RESULTS: u32 = 0x200_0000;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const _DN_SAVE_OPERANDS_FLUSH_RESULTS: u32 = 0x300_0000;

/// `FE_ALL_EXCEPT` (Wine's fenv.h: the five `_SW_*` exception bits).
const FE_ALL_EXCEPT: u32 = 0x1f;

/// `fenv_t`: `{ unsigned long _Fe_ctl, _Fe_stat; }`, both in the encoded form.
#[repr(C)]
pub struct FenvT {
    fe_ctl: u32,
    fe_stat: u32,
}
/// `fexcept_t`: `unsigned long`.
pub type FexceptT = u32;

// sdk/lib/crt's on amd64, fpctrl_arm64's on arm64; i386 defines its own below.
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn _control87(newval: u32, mask: u32) -> u32;
    fn _controlfp(newval: u32, mask: u32) -> u32;
    fn _statusfp() -> u32;
}

// ── encoding ─────────────────────────────────────────────────────────────────

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod enc {
    pub const FENV_X_INVALID: u32 = 0x0010_0010;
    pub const FENV_X_DENORMAL: u32 = 0x0020_0020;
    pub const FENV_X_ZERODIVIDE: u32 = 0x0008_0008;
    pub const FENV_X_OVERFLOW: u32 = 0x0004_0004;
    pub const FENV_X_UNDERFLOW: u32 = 0x0002_0002;
    pub const FENV_X_INEXACT: u32 = 0x0001_0001;
    pub const FENV_X_AFFINE: u32 = 0x0000_4000;
    pub const FENV_X_UP: u32 = 0x0080_0200;
    pub const FENV_X_DOWN: u32 = 0x0040_0100;
    pub const FENV_X_24: u32 = 0x0000_2000;
    pub const FENV_X_53: u32 = 0x0000_1000;
    pub const FENV_Y_INVALID: u32 = 0x1000_0010;
    pub const FENV_Y_DENORMAL: u32 = 0x2000_0020;
    pub const FENV_Y_ZERODIVIDE: u32 = 0x0800_0008;
    pub const FENV_Y_OVERFLOW: u32 = 0x0400_0004;
    pub const FENV_Y_UNDERFLOW: u32 = 0x0200_0002;
    pub const FENV_Y_INEXACT: u32 = 0x0100_0001;
    pub const FENV_Y_UP: u32 = 0x8000_0200;
    pub const FENV_Y_DOWN: u32 = 0x4000_0100;
    pub const FENV_Y_FLUSH: u32 = 0x0000_0400;
    pub const FENV_Y_FLUSH_SAVE: u32 = 0x0000_0800;
}

/// `fenv_encode`: the x87 word `x` (i386 only; ignored elsewhere, as the C's
/// `#ifdef __i386__` does) and the SSE word `y`, as one encoded `unsigned long`.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn fenv_encode(x: u32, y: u32) -> u32 {
    use enc::*;
    let mut ret = 0;
    #[cfg(target_arch = "x86")]
    {
        for (bit, pat) in [
            (_EM_INVALID, FENV_X_INVALID),
            (_EM_DENORMAL, FENV_X_DENORMAL),
            (_EM_ZERODIVIDE, FENV_X_ZERODIVIDE),
            (_EM_OVERFLOW, FENV_X_OVERFLOW),
            (_EM_UNDERFLOW, FENV_X_UNDERFLOW),
            (_EM_INEXACT, FENV_X_INEXACT),
            (_IC_AFFINE, FENV_X_AFFINE),
            (_RC_UP, FENV_X_UP),
            (_RC_DOWN, FENV_X_DOWN),
            (_PC_24, FENV_X_24),
            (_PC_53, FENV_X_53),
        ] {
            if x & bit != 0 {
                ret |= pat;
            }
        }
    }
    #[cfg(not(target_arch = "x86"))]
    let _ = x;
    for (bit, pat) in [
        (_EM_INVALID, FENV_Y_INVALID),
        (_EM_DENORMAL, FENV_Y_DENORMAL),
        (_EM_ZERODIVIDE, FENV_Y_ZERODIVIDE),
        (_EM_OVERFLOW, FENV_Y_OVERFLOW),
        (_EM_UNDERFLOW, FENV_Y_UNDERFLOW),
        (_EM_INEXACT, FENV_Y_INEXACT),
        (_RC_UP, FENV_Y_UP),
        (_RC_DOWN, FENV_Y_DOWN),
        (_DN_FLUSH, FENV_Y_FLUSH),
        (_DN_FLUSH_OPERANDS_SAVE_RESULTS, FENV_Y_FLUSH_SAVE),
    ] {
        // `_DN_FLUSH` (0x1000000) is a bit of the two-bit `_MCW_DN` field, and
        // `_DN_SAVE_OPERANDS_FLUSH_RESULTS` (0x3000000) sets both patterns: the C
        // tests each bit on its own, and so does this.
        if y & bit != 0 {
            ret |= pat;
        }
    }
    ret
}

/// `fenv_decode`: `None` when `enc` is not something `fenv_encode` produces (the
/// C's `return FALSE` after re-encoding and comparing).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn fenv_decode(enc: u32) -> Option<(u32, u32)> {
    use enc::*;
    let (mut x, mut y) = (0, 0);
    let has = |pat: u32| enc & pat == pat;
    for (pat, bit) in [
        (FENV_X_INVALID, _EM_INVALID),
        (FENV_X_DENORMAL, _EM_DENORMAL),
        (FENV_X_ZERODIVIDE, _EM_ZERODIVIDE),
        (FENV_X_OVERFLOW, _EM_OVERFLOW),
        (FENV_X_UNDERFLOW, _EM_UNDERFLOW),
        (FENV_X_INEXACT, _EM_INEXACT),
        (FENV_X_AFFINE, _IC_AFFINE),
        (FENV_X_UP, _RC_UP),
        (FENV_X_DOWN, _RC_DOWN),
        (FENV_X_24, _PC_24),
        (FENV_X_53, _PC_53),
    ] {
        if has(pat) {
            x |= bit;
        }
    }
    for (pat, bit) in [
        (FENV_Y_INVALID, _EM_INVALID),
        (FENV_Y_DENORMAL, _EM_DENORMAL),
        (FENV_Y_ZERODIVIDE, _EM_ZERODIVIDE),
        (FENV_Y_OVERFLOW, _EM_OVERFLOW),
        (FENV_Y_UNDERFLOW, _EM_UNDERFLOW),
        (FENV_Y_INEXACT, _EM_INEXACT),
        (FENV_Y_UP, _RC_UP),
        (FENV_Y_DOWN, _RC_DOWN),
        (FENV_Y_FLUSH, _DN_FLUSH),
        (FENV_Y_FLUSH_SAVE, _DN_FLUSH_OPERANDS_SAVE_RESULTS),
    ] {
        if has(pat) {
            y |= bit;
        }
    }
    (fenv_encode(x, y) == enc).then_some((x, y))
}

/// arm64: the word itself, `_EM_DENORMAL` folded to 0x20 (the C's
/// `#elif _MSVCR_VER >= 120` pair).
#[cfg(target_arch = "aarch64")]
fn fenv_encode(_x: u32, mut y: u32) -> u32 {
    if y & _EM_DENORMAL != 0 {
        y = (y & !_EM_DENORMAL) | 0x20;
    }
    y
}
#[cfg(target_arch = "aarch64")]
fn fenv_decode(mut enc: u32) -> Option<(u32, u32)> {
    if enc & 0x20 != 0 {
        enc = (enc & !0x20) | _EM_DENORMAL;
    }
    Some((enc, enc))
}

// ── the hardware: _setfp / _setfp_sse ────────────────────────────────────────

/// `sse2_supported`: `KUSER_SHARED_DATA.ProcessorFeatures[PF_XMMI64_INSTRUCTIONS_AVAILABLE]`,
/// read where msvcrt reads it once at init.
#[cfg(target_arch = "x86")]
fn sse2_supported() -> bool {
    const KSHARED_PROCESSOR_FEATURES: *const [u8; 64] = 0x7FFE_0274 as *const [u8; 64];
    const PF_XMMI64_INSTRUCTIONS_AVAILABLE: usize = 10;
    unsafe { (*KSHARED_PROCESSOR_FEATURES)[PF_XMMI64_INSTRUCTIONS_AVAILABLE] != 0 }
}

/// `stmxcsr`/`ldmxcsr` take a memory operand only, so the asm holds its address.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn get_mxcsr() -> u32 {
    let mut ret: u32 = 0;
    core::arch::asm!("stmxcsr ({})", in(reg) core::ptr::addr_of_mut!(ret), options(att_syntax, nostack));
    ret
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn set_mxcsr(val: u32) {
    core::arch::asm!("ldmxcsr ({})", in(reg) core::ptr::addr_of!(val), options(att_syntax, nostack));
}

/// `_setfp_sse`: MXCSR. Read into `*cw`/`*sw` (bits outside the mask come from
/// the hardware), write back what is inside it; a null pointer leaves that half.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(crate) unsafe fn setfp_sse(cw: *mut u32, mut cw_mask: u32, sw: *mut u32, mut sw_mask: u32) {
    let mut fpword = get_mxcsr();
    let old_fpword = fpword;

    cw_mask &= _MCW_EM | _MCW_RC | _MCW_DN;
    sw_mask &= _MCW_EM;

    const SW_BITS: [(u32, u32); 6] = [
        (0x1, _SW_INVALID),
        (0x2, _SW_DENORMAL),
        (0x4, _SW_ZERODIVIDE),
        (0x8, _SW_OVERFLOW),
        (0x10, _SW_UNDERFLOW),
        (0x20, _SW_INEXACT),
    ];
    const EM_BITS: [(u32, u32); 6] = [
        (0x80, _EM_INVALID),
        (0x100, _EM_DENORMAL),
        (0x200, _EM_ZERODIVIDE),
        (0x400, _EM_OVERFLOW),
        (0x800, _EM_UNDERFLOW),
        (0x1000, _EM_INEXACT),
    ];

    if !sw.is_null() {
        let mut flags = 0;
        for (hw, f) in SW_BITS {
            if fpword & hw != 0 {
                flags |= f;
            }
        }
        *sw = (flags & !sw_mask) | (*sw & sw_mask);
        fpword &= !0x3f;
        for (hw, f) in SW_BITS {
            if *sw & f != 0 {
                fpword |= hw;
            }
        }
        *sw = flags;
    }

    if !cw.is_null() {
        let mut flags = 0;
        for (hw, f) in EM_BITS {
            if fpword & hw != 0 {
                flags |= f;
            }
        }
        flags |= match fpword & 0x6000 {
            0x6000 => _RC_UP | _RC_DOWN,
            0x4000 => _RC_UP,
            0x2000 => _RC_DOWN,
            _ => 0,
        };
        flags |= match fpword & 0x8040 {
            0x0040 => _DN_FLUSH_OPERANDS_SAVE_RESULTS,
            0x8000 => _DN_SAVE_OPERANDS_FLUSH_RESULTS,
            0x8040 => _DN_FLUSH,
            _ => 0,
        };
        *cw = (flags & !cw_mask) | (*cw & cw_mask);

        fpword &= !0xffc0;
        for (hw, f) in EM_BITS {
            if *cw & f != 0 {
                fpword |= hw;
            }
        }
        fpword |= match *cw & _MCW_RC {
            _RC_CHOP => 0x6000,
            _RC_UP => 0x4000,
            _RC_DOWN => 0x2000,
            _ => 0,
        };
        fpword |= match *cw & _MCW_DN {
            _DN_FLUSH_OPERANDS_SAVE_RESULTS => 0x0040,
            _DN_SAVE_OPERANDS_FLUSH_RESULTS => 0x8000,
            _DN_FLUSH => 0x8040,
            _ => 0,
        };

        // A control-only update must not also raise stale flags.
        if fpword != old_fpword && sw.is_null() {
            fpword &= !0x3f;
        }
    }

    if fpword != old_fpword {
        set_mxcsr(fpword);
    }
}

/// `_setfp` on i386: the x87 control and status words.
#[cfg(target_arch = "x86")]
pub(crate) unsafe fn setfp(cw: *mut u32, mut cw_mask: u32, sw: *mut u32, mut sw_mask: u32) {
    let (mut oldcw, mut newcw, mut oldsw, mut newsw): (u32, u32, u32, u32) = (0, 0, 0, 0);

    cw_mask &= _MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC;
    sw_mask &= _MCW_EM;

    const SW_BITS: [(u32, u32); 6] = [
        (0x1, _SW_INVALID),
        (0x2, _SW_DENORMAL),
        (0x4, _SW_ZERODIVIDE),
        (0x8, _SW_OVERFLOW),
        (0x10, _SW_UNDERFLOW),
        (0x20, _SW_INEXACT),
    ];
    const EM_BITS: [(u32, u32); 6] = [
        (0x1, _EM_INVALID),
        (0x2, _EM_DENORMAL),
        (0x4, _EM_ZERODIVIDE),
        (0x8, _EM_OVERFLOW),
        (0x10, _EM_UNDERFLOW),
        (0x20, _EM_INEXACT),
    ];

    if !sw.is_null() {
        // `fstsw` stores two bytes into the zeroed four-byte slot.
        core::arch::asm!("fstsw ({})", in(reg) core::ptr::addr_of_mut!(newsw), options(att_syntax, nostack));
        oldsw = newsw;
        let mut flags = 0;
        for (hw, f) in SW_BITS {
            if newsw & hw != 0 {
                flags |= f;
            }
        }
        *sw = (flags & !sw_mask) | (*sw & sw_mask);
        newsw &= !0x3f;
        for (hw, f) in SW_BITS {
            if *sw & f != 0 {
                newsw |= hw;
            }
        }
        *sw = flags;
    }

    if !cw.is_null() {
        core::arch::asm!("fstcw ({})", in(reg) core::ptr::addr_of_mut!(newcw), options(att_syntax, nostack));
        oldcw = newcw;
        let mut flags = 0;
        for (hw, f) in EM_BITS {
            if newcw & hw != 0 {
                flags |= f;
            }
        }
        flags |= match newcw & 0xc00 {
            0xc00 => _RC_UP | _RC_DOWN,
            0x800 => _RC_UP,
            0x400 => _RC_DOWN,
            _ => 0,
        };
        flags |= match newcw & 0x300 {
            0x0 => _PC_24,
            0x200 => _PC_53,
            0x300 => _PC_64,
            _ => 0,
        };
        if newcw & 0x1000 != 0 {
            flags |= _IC_AFFINE;
        }
        *cw = (flags & !cw_mask) | (*cw & cw_mask);

        newcw &= !0x1f3f;
        for (hw, f) in EM_BITS {
            if *cw & f != 0 {
                newcw |= hw;
            }
        }
        newcw |= match *cw & _MCW_RC {
            _RC_CHOP => 0xc00,
            _RC_UP => 0x800,
            _RC_DOWN => 0x400,
            _ => 0,
        };
        newcw |= match *cw & _MCW_PC {
            _PC_64 => 0x300,
            _PC_53 => 0x200,
            _ => 0,
        };
        if *cw & _IC_AFFINE != 0 {
            newcw |= 0x1000;
        }
    }

    if oldsw != newsw && newsw & 0x3f != 0 {
        // Raising flags: rewrite the whole x87 environment (fnstenv/fldenv), as the
        // C does -- fldcw/fnclex cannot SET a status bit. Integer-only function, no
        // live x87 value for the reload to clobber.
        #[repr(C)]
        struct X87Env {
            control_word: u16,
            unused1: u16,
            status_word: u16,
            rest: [u16; 11],
        }
        let mut env: X87Env = core::mem::zeroed();
        core::arch::asm!("fnstenv ({})", in(reg) core::ptr::addr_of_mut!(env), options(att_syntax, nostack));
        // The C asserts cw here: every caller that raises passes both halves.
        if !cw.is_null() {
            env.control_word = newcw as u16;
        }
        env.status_word = newsw as u16;
        core::arch::asm!("fldenv ({})", in(reg) core::ptr::addr_of!(env), options(att_syntax, nostack));
        return;
    }

    if oldsw != newsw {
        core::arch::asm!("fnclex", options(att_syntax, nostack));
    }
    if oldcw != newcw {
        core::arch::asm!("fldcw ({})", in(reg) core::ptr::addr_of!(newcw), options(att_syntax, nostack));
    }
}

/// `_setfp` on amd64 is MXCSR alone.
#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn setfp(cw: *mut u32, cw_mask: u32, sw: *mut u32, sw_mask: u32) {
    setfp_sse(cw, cw_mask, sw, sw_mask);
}

#[cfg(target_arch = "aarch64")]
pub(crate) use crate::fpctrl_arm64::setfp;

// ── i386: the x87 + SSE pair ─────────────────────────────────────────────────

/// `_statusfp2`: the x87 and the SSE status words, separately.
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _statusfp2(x86_sw: *mut u32, sse2_sw: *mut u32) {
    if !x86_sw.is_null() {
        setfp(core::ptr::null_mut(), 0, x86_sw, 0);
    }
    if sse2_sw.is_null() {
        return;
    }
    if sse2_supported() {
        setfp_sse(core::ptr::null_mut(), 0, sse2_sw, 0);
    } else {
        *sse2_sw = 0;
    }
}

/// `__control87_2`: set and read the x87 and SSE control words separately; a null
/// pointer skips that unit. Returns 1 (the C's TRUE).
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __control87_2(newval: u32, mask: u32, x86_cw: *mut u32, sse2_cw: *mut u32) -> c_int {
    if !x86_cw.is_null() {
        *x86_cw = newval;
        setfp(x86_cw, mask, core::ptr::null_mut(), 0);
    }
    if sse2_cw.is_null() {
        return 1;
    }
    if sse2_supported() {
        *sse2_cw = newval;
        setfp_sse(sse2_cw, mask, core::ptr::null_mut(), 0);
    } else {
        *sse2_cw = 0;
    }
    1
}

const _EM_AMBIGUOUS: u32 = 0x8000_0000;

// The four public control/status entry points, i386. sdk/lib/crt's i386 bodies (DJ
// Delorie's, float/i386/{cntrlfp,statfp,clearfp}.c) drive the x87 ALONE, so on an SSE2
// machine `fesetround`, `_controlfp(_RC_UP, _MCW_RC)` and friends left MXCSR -- where
// every SSE2-compiled caller does its arithmetic, ucrtbase's own Rust included -- at
// round-to-nearest, and `_statusfp` never saw an SSE exception. Windows drives both
// units; so does Wine, whose versions these are, and `_control87` reports a disagreement
// between them as `_EM_AMBIGUOUS` (Wine's misc.c checks exactly that).

/// `_control87`: both units through `__control87_2`; the x87 word, OR'd with the SSE
/// word, plus `_EM_AMBIGUOUS` when their masks or rounding modes differ.
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _control87(newval: u32, mask: u32) -> u32 {
    let (mut flags, mut sse2_cw) = (0u32, 0u32);
    __control87_2(newval, mask, &mut flags, &mut sse2_cw);
    if sse2_supported() {
        if (flags ^ sse2_cw) & (_MCW_EM | _MCW_RC) != 0 {
            flags |= _EM_AMBIGUOUS;
        }
        flags |= sse2_cw;
    }
    flags
}

/// `_controlfp`: `_control87` that never touches `_EM_DENORMAL`.
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _controlfp(newval: u32, mask: u32) -> u32 {
    _control87(newval, mask & !_EM_DENORMAL)
}

/// `_controlfp_s` -- crtmath float/_controlfp_s.c: a mask/value pair touching bits
/// outside the documented control fields is an invalid parameter (errno EINVAL, the
/// handler, and the current word still reported); otherwise `_controlfp`.
#[no_mangle]
pub unsafe extern "C" fn _controlfp_s(cur: *mut u32, newval: u32, mask: u32) -> c_int {
    // !(_MCW_EM | _MCW_IC | _MCW_RC | _MCW_PC | _MCW_DN), spelled out because this file
    // defines _MCW_IC/_MCW_PC/_MCW_DN for x86 only. The C's `#ifdef _M_ARM` (32-bit ARM)
    // narrowing applies to no target here, so arm64 uses the full mask too, as it did.
    const INVALID_MASK: u32 = !(0x8_001f | 0x4_0000 | 0x300 | 0x3_0000 | 0x300_0000);
    extern "C" {
        fn _errno() -> *mut c_int;
        fn _invalid_parameter_noinfo();
    }
    if newval & mask & INVALID_MASK != 0 {
        *_errno() = 22; // EINVAL
        _invalid_parameter_noinfo();
        if !cur.is_null() {
            *cur = _controlfp(0, 0);
        }
        return 22;
    }
    let val = _controlfp(newval, mask);
    if !cur.is_null() {
        *cur = val;
    }
    0
}

/// `_statusfp`: the x87 and SSE status words OR'd (Windows defines no "ambiguous"
/// status bit, so Wine returns both).
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _statusfp() -> u32 {
    let (mut x86_sw, mut sse2_sw) = (0u32, 0u32);
    _statusfp2(&mut x86_sw, &mut sse2_sw);
    x86_sw | sse2_sw
}

/// `_clearfp`: both units' exception flags, then clears them.
#[cfg(target_arch = "x86")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _clearfp() -> u32 {
    let mut flags = 0u32;
    setfp(core::ptr::null_mut(), 0, &mut flags, _MCW_EM);
    if sse2_supported() {
        let mut sse_sw = 0u32;
        setfp_sse(core::ptr::null_mut(), 0, &mut sse_sw, _MCW_EM);
        flags |= sse_sw;
    }
    flags
}

// ── the exports ──────────────────────────────────────────────────────────────

/// `fegetenv`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fegetenv(env: *mut FenvT) -> c_int {
    #[cfg(target_arch = "x86")]
    {
        let (mut x87, mut sse) = (0, 0);
        __control87_2(0, 0, &mut x87, &mut sse);
        (*env).fe_ctl = fenv_encode(x87, sse);
        _statusfp2(&mut x87, &mut sse);
        (*env).fe_stat = fenv_encode(x87, sse);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        (*env).fe_ctl = fenv_encode(0, _control87(0, 0));
        (*env).fe_stat = fenv_encode(0, _statusfp());
    }
    0
}

/// `fesetenv`. Nonzero when `env` holds bits no `fegetenv` could have produced.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fesetenv(env: *const FenvT) -> c_int {
    if (*env).fe_ctl == 0 && (*env).fe_stat == 0 {
        _fpreset();
        return 0;
    }
    let Some((mut x87_cw, mut cw)) = fenv_decode((*env).fe_ctl) else { return 1 };
    let Some((mut x87_stat, mut stat)) = fenv_decode((*env).fe_stat) else { return 1 };
    // `_MSVCR_VER >= 140`: every control bit is written.
    let mask = !0;
    #[cfg(target_arch = "x86")]
    {
        setfp(&mut x87_cw, mask, &mut x87_stat, !0);
        if sse2_supported() {
            setfp_sse(&mut cw, mask, &mut stat, !0);
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let _ = (&mut x87_cw, &mut x87_stat);
        setfp(&mut cw, mask, &mut stat, !0);
    }
    0
}

/// `feholdexcept`: save the environment, then clear every flag.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn feholdexcept(env: *mut FenvT) -> c_int {
    fegetenv(env);
    feclearexcept(FE_ALL_EXCEPT as c_int)
}

/// `fetestexcept`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fetestexcept(flags: c_int) -> c_int {
    (_statusfp() & flags as u32) as c_int
}

/// `feclearexcept`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn feclearexcept(flags: c_int) -> c_int {
    let mut env = FenvT { fe_ctl: 0, fe_stat: 0 };
    fegetenv(&mut env);
    let flags = flags as u32 & FE_ALL_EXCEPT;
    env.fe_stat &= !fenv_encode(flags, flags);
    fesetenv(&env)
}

/// `fegetexceptflag`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fegetexceptflag(status: *mut FexceptT, excepts: c_int) -> c_int {
    let excepts = excepts as u32;
    #[cfg(target_arch = "x86")]
    {
        let (mut x87, mut sse) = (0, 0);
        _statusfp2(&mut x87, &mut sse);
        *status = fenv_encode(x87 & excepts, sse & excepts);
    }
    #[cfg(not(target_arch = "x86"))]
    {
        *status = fenv_encode(0, _statusfp() & excepts);
    }
    0
}

/// `fesetexceptflag`: set the flags in `excepts` to their state in `*status`.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fesetexceptflag(status: *const FexceptT, excepts: c_int) -> c_int {
    let excepts = excepts as u32 & FE_ALL_EXCEPT;
    if excepts == 0 {
        return 0;
    }
    let mut env = FenvT { fe_ctl: 0, fe_stat: 0 };
    fegetenv(&mut env);
    env.fe_stat &= !fenv_encode(excepts, excepts);
    env.fe_stat |= *status & fenv_encode(excepts, excepts);
    fesetenv(&env)
}

/// `fegetround`: `FE_TONEAREST`/`FE_DOWNWARD`/`FE_UPWARD`/`FE_TOWARDZERO`, which
/// are the `_RC_*` values.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fegetround() -> c_int {
    (_controlfp(0, 0) & _MCW_RC) as c_int
}

/// `fesetround`: nonzero for anything but the four modes.
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn fesetround(round_mode: c_int) -> c_int {
    if round_mode as u32 & !_MCW_RC != 0 {
        return 1;
    }
    _controlfp(round_mode as u32, _MCW_RC);
    0
}

/// `__fpe_flt_rounds`: C's `FLT_ROUNDS` (0 toward zero, 1 nearest, 2 up, 3 down).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __fpe_flt_rounds() -> c_int {
    match _controlfp(0, 0) & _RC_CHOP {
        _RC_CHOP => 0,
        _RC_NEAR => 1,
        _RC_UP => 2,
        _ => 3,
    }
}

/// `_fpreset`: the default floating-point state -- every exception masked, no
/// flags, round to nearest (and on i386 the x87 reinitialised at 64-bit precision).
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn _fpreset() {
    #[cfg(target_arch = "x86")]
    {
        let x86_cw: u32 = 0x27f;
        core::arch::asm!("fninit", "fldcw ({})", in(reg) core::ptr::addr_of!(x86_cw), options(att_syntax, nostack));
        if sse2_supported() {
            let (mut cw, mut sw) = (_MCW_EM, 0);
            setfp_sse(&mut cw, !0, &mut sw, !0);
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let (mut cw, mut sw) = (_MCW_EM, 0);
        setfp(&mut cw, !0, &mut sw, !0);
    }
}
