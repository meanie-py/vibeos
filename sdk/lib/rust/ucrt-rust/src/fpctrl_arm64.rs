//! The floating-point control/status family on arm64: `_statusfp`, `_clearfp`,
//! `_control87`, `_controlfp` (and through them `_set_controlfp`, the spec alias, and
//! `sdk/lib/crt`'s `_controlfp_s`, which calls `_controlfp`).
//!
//! i386 and amd64 take these from `sdk/lib/crt/float`'s x87/SSE bodies; that library has
//! no arm64 body, so until this module arm64 ucrtbase resolved all four through the msvcrt
//! import library and shipped a forward into msvcrt.dll. These are the same FPCR/FPSR
//! mappings msvcrt-crt-rust ports from Wine's `_setfp` (`msvcrt/math.rs`), so the
//! behaviour an arm64 caller sees is unchanged -- only the forward is gone.
//!
//! Details that are silent if wrong:
//! * FPCR's trap-ENABLE bits (8..12, 15) are the inverse of the `_EM_*` MASK bits: a set
//!   `_EM_INVALID` means bit 8 CLEAR.
//! * FPSR's cumulative flags sit at bits 0..4 and 7 (IDC), in the opposite order from
//!   `_SW_*` (bit 0 is IOC = `_SW_INVALID`, not `_SW_INEXACT`).
//! * `_controlfp` never touches `_EM_DENORMAL`; `_control87` does.

use core::arch::asm;

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
const _RC_DOWN: u32 = 0x100;
const _RC_UP: u32 = 0x200;
const _RC_CHOP: u32 = 0x300;

/// FPSR cumulative exception flags, `_SW_*` order: (FPSR bit, `_SW_` flag).
const FPSR_FLAGS: [(u64, u32); 6] = [
    (0x1, _SW_INVALID),
    (0x2, _SW_ZERODIVIDE),
    (0x4, _SW_OVERFLOW),
    (0x8, _SW_UNDERFLOW),
    (0x10, _SW_INEXACT),
    (0x80, _SW_DENORMAL),
];
/// FPCR trap-enable bits: (FPCR bit, the `_EM_` mask that is set when the bit is CLEAR).
const FPCR_TRAPS: [(u64, u32); 6] = [
    (0x100, _EM_INVALID),
    (0x200, _EM_ZERODIVIDE),
    (0x400, _EM_OVERFLOW),
    (0x800, _EM_UNDERFLOW),
    (0x1000, _EM_INEXACT),
    (0x8000, _EM_DENORMAL),
];
/// FPCR.RMode, bits 22-23.
const FPCR_RMODE: u64 = 0xc0_0000;

/// Wine's `_setfp` (aarch64): read the control word into `*cw` / the status word into
/// `*sw`, replacing the bits OUTSIDE each mask with the hardware's, and write back the bits
/// INSIDE it. A null pointer leaves that register alone.
unsafe fn setfp(cw: *mut u32, mut cw_mask: u32, sw: *mut u32, mut sw_mask: u32) {
    let (mut fpsr, mut fpcr): (u64, u64) = (0, 0);
    let (mut old_fpsr, mut old_fpcr): (u64, u64) = (0, 0);

    cw_mask &= _MCW_EM | _MCW_RC;
    sw_mask &= _MCW_EM;

    if !sw.is_null() {
        asm!("mrs {}, fpsr", out(reg) fpsr, options(nomem, nostack, preserves_flags));
        old_fpsr = fpsr;

        let mut flags = 0;
        for (bit, f) in FPSR_FLAGS {
            if fpsr & bit != 0 {
                flags |= f;
            }
        }
        *sw = (flags & !sw_mask) | (*sw & sw_mask);

        fpsr &= !0x9f;
        for (bit, f) in FPSR_FLAGS {
            if *sw & f != 0 {
                fpsr |= bit;
            }
        }
        *sw = flags;
    }

    if !cw.is_null() {
        asm!("mrs {}, fpcr", out(reg) fpcr, options(nomem, nostack, preserves_flags));
        old_fpcr = fpcr;

        let mut flags = 0;
        for (bit, f) in FPCR_TRAPS {
            if fpcr & bit == 0 {
                flags |= f;
            }
        }
        flags |= match fpcr & FPCR_RMODE {
            0x40_0000 => _RC_UP,
            0x80_0000 => _RC_DOWN,
            0xc0_0000 => _RC_CHOP,
            _ => 0,
        };
        *cw = (flags & !cw_mask) | (*cw & cw_mask);

        fpcr &= !(FPCR_RMODE | 0x9f00);
        for (bit, f) in FPCR_TRAPS {
            if *cw & f == 0 {
                fpcr |= bit;
            }
        }
        fpcr |= match *cw & _MCW_RC {
            _RC_CHOP => 0xc0_0000,
            _RC_UP => 0x40_0000,
            _RC_DOWN => 0x80_0000,
            _ => 0,
        };
    }

    // Before the flags are written, take the traps down if a flag that stays raised had
    // its trap enabled, so the FPSR write cannot fire one; the final FPCR write below
    // restores what was asked for. Wine writes this test as
    // `~(old_fpcr >> 8) & fpsr & 0x9f != fpsr & 0x9f`, where C's `!=` binds tighter than
    // `&` and folds the whole thing to bit 0 of `fpsr & (0x9f != fpsr)`;
    // msvcrt-crt-rust keeps that reading. This is the comparison the text means. The
    // difference is only ever this transient write.
    if old_fpcr != fpcr && (!(old_fpcr >> 8) & fpsr & 0x9f) != (fpsr & 0x9f) {
        let traps_down = fpcr & !0x9f00;
        asm!("msr fpcr, {}", in(reg) traps_down, options(nomem, nostack, preserves_flags));
    }
    if old_fpsr != fpsr {
        asm!("msr fpsr, {}", in(reg) fpsr, options(nomem, nostack, preserves_flags));
    }
    if old_fpcr != fpcr {
        asm!("msr fpcr, {}", in(reg) fpcr, options(nomem, nostack, preserves_flags));
    }
}

/// `_statusfp`: the accumulated exception flags, untouched.
#[no_mangle]
pub unsafe extern "C" fn _statusfp() -> u32 {
    let mut flags = 0;
    setfp(core::ptr::null_mut(), 0, &mut flags, 0);
    flags
}

/// `_clearfp`: the accumulated exception flags, then clears them.
#[no_mangle]
pub unsafe extern "C" fn _clearfp() -> u32 {
    let mut flags = 0;
    setfp(core::ptr::null_mut(), 0, &mut flags, _MCW_EM);
    flags
}

/// `_control87`: set the control-word bits in `mask` to `newval`'s, return the result.
#[no_mangle]
pub unsafe extern "C" fn _control87(newval: u32, mask: u32) -> u32 {
    let mut flags = newval;
    setfp(&mut flags, mask, core::ptr::null_mut(), 0);
    flags
}

/// `_controlfp`: `_control87` that never changes the denormal mask.
#[no_mangle]
pub unsafe extern "C" fn _controlfp(newval: u32, mask: u32) -> u32 {
    _control87(newval, mask & !_EM_DENORMAL)
}
