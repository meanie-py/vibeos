//! The floating-point control/status family on amd64: `_statusfp`, `_clearfp`, `_control87`,
//! `_controlfp` (and through them `_set_controlfp` and `_controlfp_s`), over MXCSR. A port of
//! sdk/lib/crt/float/amd64/{_statusfp,_clearfp,_control87,_controlfp,machfpcw}.c, which
//! ucrtbase linked as C until this module (i386 takes the x87+SSE pair from `fenv`, arm64
//! from `fpctrl_arm64`).
//!
//! Details that are silent if wrong:
//! * MXCSR holds the exception FLAGS (bits 0-5) beside the control fields; `_control87`
//!   keeps them, so even a read (`_control87(0, 0)`) does not clear pending exceptions.
//! * `_DN_*` is a two-bit field mapped onto two independent MXCSR bits (DAZ, FZ).
//! * `_control87` honours only `_MCW_DN | _MCW_EM | _MCW_RC` of the mask; `_controlfp`
//!   additionally never touches `_EM_DENORMAL`.

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

const _MCW_DN: u32 = 0x300_0000;
const _DN_SAVE: u32 = 0x0;
const _DN_FLUSH: u32 = 0x100_0000;
const _DN_FLUSH_OPERANDS_SAVE_RESULTS: u32 = 0x200_0000;
const _DN_SAVE_OPERANDS_FLUSH_RESULTS: u32 = 0x300_0000;

// xmmintrin.h's MXCSR fields.
const MM_EXCEPT_MASK: u32 = 0x3f;
const MM_EXCEPT_INVALID: u32 = 0x1;
const MM_EXCEPT_DENORM: u32 = 0x2;
const MM_EXCEPT_DIV_ZERO: u32 = 0x4;
const MM_EXCEPT_OVERFLOW: u32 = 0x8;
const MM_EXCEPT_UNDERFLOW: u32 = 0x10;
const MM_EXCEPT_INEXACT: u32 = 0x20;
const MM_DENORMALS_ARE_ZERO: u32 = 0x40;
const MM_MASK_INVALID: u32 = 0x80;
const MM_MASK_DENORM: u32 = 0x100;
const MM_MASK_DIV_ZERO: u32 = 0x200;
const MM_MASK_OVERFLOW: u32 = 0x400;
const MM_MASK_UNDERFLOW: u32 = 0x800;
const MM_MASK_INEXACT: u32 = 0x1000;
const MM_ROUND_MASK: u32 = 0x6000;
const MM_ROUND_DOWN: u32 = 0x2000;
const MM_ROUND_UP: u32 = 0x4000;
const MM_ROUND_TOWARD_ZERO: u32 = 0x6000;
const MM_FLUSH_ZERO_MASK: u32 = 0x8000;

/// (MXCSR mask bit, `_EM_` flag): a set mask bit is a masked (disabled) exception, as is a
/// set `_EM_` bit, so the mapping is direct.
const MASKS: [(u32, u32); 6] = [
    (MM_MASK_INVALID, _EM_INVALID),
    (MM_MASK_DENORM, _EM_DENORMAL),
    (MM_MASK_DIV_ZERO, _EM_ZERODIVIDE),
    (MM_MASK_OVERFLOW, _EM_OVERFLOW),
    (MM_MASK_UNDERFLOW, _EM_UNDERFLOW),
    (MM_MASK_INEXACT, _EM_INEXACT),
];
/// (MXCSR rounding field, `_RC_` value); nearest is 0 on both sides.
const ROUNDING: [(u32, u32); 3] = [
    (MM_ROUND_DOWN, _RC_DOWN),
    (MM_ROUND_UP, _RC_UP),
    (MM_ROUND_TOWARD_ZERO, _RC_CHOP),
];

/// machfpcw.c `_fpcw_native_to_abstract`.
fn native_to_abstract(native: u32) -> u32 {
    let mut abstract_ = 0;
    for (n, a) in MASKS {
        if native & n != 0 {
            abstract_ |= a;
        }
    }
    for (n, a) in ROUNDING {
        if native & MM_ROUND_MASK == n {
            abstract_ |= a;
        }
    }
    abstract_ |= match (native & MM_DENORMALS_ARE_ZERO != 0, native & MM_FLUSH_ZERO_MASK != 0) {
        (true, true) => _DN_FLUSH,
        (true, false) => _DN_FLUSH_OPERANDS_SAVE_RESULTS,
        (false, true) => _DN_SAVE_OPERANDS_FLUSH_RESULTS,
        (false, false) => _DN_SAVE,
    };
    abstract_
}

/// machfpcw.c `_fpcw_abstract_to_native`.
fn abstract_to_native(abstract_: u32) -> u32 {
    let mut native = 0;
    for (n, a) in MASKS {
        if abstract_ & a != 0 {
            native |= n;
        }
    }
    for (n, a) in ROUNDING {
        if abstract_ & _MCW_RC == a {
            native |= n;
        }
    }
    native |= match abstract_ & _MCW_DN {
        _DN_FLUSH => MM_DENORMALS_ARE_ZERO | MM_FLUSH_ZERO_MASK,
        _DN_FLUSH_OPERANDS_SAVE_RESULTS => MM_DENORMALS_ARE_ZERO,
        _DN_SAVE_OPERANDS_FLUSH_RESULTS => MM_FLUSH_ZERO_MASK,
        _ => 0,
    };
    native
}

fn getcsr() -> u32 {
    let mut v = 0u32;
    unsafe { asm!("stmxcsr [{}]", in(reg) &mut v, options(nostack, preserves_flags)) };
    v
}
fn setcsr(v: u32) {
    unsafe { asm!("ldmxcsr [{}]", in(reg) &v, options(nostack, preserves_flags)) };
}

/// `_statusfp`: MXCSR's exception flags as `_SW_*`.
#[no_mangle]
pub extern "C" fn _statusfp() -> u32 {
    let mxcsr = getcsr();
    let mut status = 0;
    for (m, s) in [
        (MM_EXCEPT_INVALID, _SW_INVALID),
        (MM_EXCEPT_DENORM, _SW_DENORMAL),
        (MM_EXCEPT_DIV_ZERO, _SW_ZERODIVIDE),
        (MM_EXCEPT_OVERFLOW, _SW_OVERFLOW),
        (MM_EXCEPT_UNDERFLOW, _SW_UNDERFLOW),
        (MM_EXCEPT_INEXACT, _SW_INEXACT),
    ] {
        if mxcsr & m != 0 {
            status |= s;
        }
    }
    status
}

/// `_clearfp`: the status before, then MXCSR's exception flags cleared.
#[no_mangle]
pub extern "C" fn _clearfp() -> u32 {
    let retval = _statusfp();
    setcsr(getcsr() & !MM_EXCEPT_MASK);
    retval
}

/// `_control87`.
#[no_mangle]
pub extern "C" fn _control87(newval: u32, mask: u32) -> u32 {
    let mask = mask & (_MCW_DN | _MCW_EM | _MCW_RC);
    let native = getcsr();
    let updated = (native_to_abstract(native) & !mask) | (newval & mask);
    setcsr(abstract_to_native(updated) | (native & MM_EXCEPT_MASK));
    updated
}

/// `_controlfp`: `_control87` that never touches `_EM_DENORMAL`.
#[no_mangle]
pub extern "C" fn _controlfp(newval: u32, mask: u32) -> u32 {
    _control87(newval, mask & !_EM_DENORMAL)
}
