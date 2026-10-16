//! `_except1` (ucrt-rust src/fenv/except1.rs): its decision ladder graded on the host.
//!
//! There is no Microsoft UCRT body to compare against -- the export was a `-stub` -- so
//! the oracle is Wine's msvcrt math.c `_except1`, read case by case: which FE_* flags are
//! raised silently, which SEH code is raised instead, how the result saturates, and the
//! MXCSR -> x87 control-word remap the 64-bit targets apply first. The hardware half
//! (`feraiseexcept`, `RaiseException`, `_setfp`) is the fenv family's, graded by
//! tools/payloads/fenv.

#[path = "../../ucrt-rust/src/fenv/except1_pure.rs"]
mod pure;
use pure::*;

#[test]
fn except1_ladder_matches_wine() {
    let mut n = 0;
    macro_rules! eq { ($a:expr,$b:expr) => { assert_eq!($a,$b); n+=1; } }
    // overflow masked (cw bit 3), res -> inf, only OVERFLOW
    eq!(decide(0x1, 0x8), Decision{raise:FE_OVERFLOW, exception:0, res_kind:1});
    // overflow unmasked -> exception, raise cleared
    eq!(decide(0x1, 0x0), Decision{raise:0, exception:EXCEPTION_FLT_OVERFLOW, res_kind:0});
    // 0x11: masked if overflow(0x8) or inexact(0x20) bit set ("cw&0x28"); raises both
    eq!(decide(0x11, 0x20), Decision{raise:FE_OVERFLOW|FE_INEXACT, exception:0, res_kind:1});
    eq!(decide(0x11, 0x10), Decision{raise:0, exception:EXCEPTION_FLT_OVERFLOW, res_kind:0});
    // underflow
    eq!(decide(0x2, 0x10), Decision{raise:FE_UNDERFLOW, exception:0, res_kind:2});
    eq!(decide(0x12, 0x20), Decision{raise:FE_UNDERFLOW|FE_INEXACT, exception:0, res_kind:2});
    eq!(decide(0x2, 0x8).exception, EXCEPTION_FLT_UNDERFLOW);
    // zerodivide: result unchanged
    eq!(decide(0x4, 0x4), Decision{raise:FE_DIVBYZERO, exception:0, res_kind:0});
    eq!(decide(0x14, 0x20), Decision{raise:FE_DIVBYZERO|FE_INEXACT, exception:0, res_kind:0});
    eq!(decide(0x4, 0x0).exception, EXCEPTION_FLT_DIVIDE_BY_ZERO);
    // invalid
    eq!(decide(0x8, 0x1), Decision{raise:FE_INVALID, exception:0, res_kind:0});
    eq!(decide(0x8, 0x0).exception, EXCEPTION_FLT_INVALID_OPERATION);
    // inexact
    eq!(decide(0x10, 0x20), Decision{raise:FE_INEXACT, exception:0, res_kind:0});
    eq!(decide(0x10, 0x0).exception, EXCEPTION_FLT_INEXACT_RESULT);
    // fpe==0 -> nothing
    eq!(decide(0, 0xffff), Decision{raise:0, exception:0, res_kind:0});
    // 0x9 hits overflow branch first (else-if chain), invalid ignored
    eq!(decide(0x9, 0x1).exception, EXCEPTION_FLT_OVERFLOW);
    // normalize (64-bit host): MXCSR mask bits 7..12 -> 0..5, RC 13-14 -> 10-11
    eq!(normalize_cw(0x1f80), 0x3f);
    eq!(normalize_cw(0x1f80 | 0x6000), 0x3f | 0xc00);
    eq!(normalize_cw(0x1f80 | 0x2000), 0x3f | 0x400);
    eq!(normalize_cw(0x1f80 | 0x4000), 0x3f | 0x800);
    eq!(normalize_cw(0x0000), 0);
    // overflow masked via MXCSR: bit 10 (0x400) -> 0x8
    eq!(decide(0x1, normalize_cw(0x400)), Decision{raise:FE_OVERFLOW, exception:0, res_kind:1});
    // fpword
    eq!(fpword_from_cw(0x3f), EM_INVALID|EM_DENORMAL|EM_ZERODIVIDE|EM_OVERFLOW|EM_UNDERFLOW|EM_INEXACT|PC_24);
    eq!(fpword_from_cw(0x27f), 0x8_001f | PC_53);
    eq!(fpword_from_cw(0x37f), 0x8_001f | PC_64);
    eq!(fpword_from_cw(0xc00|0x1000), PC_24|RC_UP|RC_DOWN|IC_AFFINE);
    eq!(fpword_from_cw(0x800) & MCW_RC, RC_UP);
    eq!(fpword_from_cw(0x400) & MCW_RC, RC_DOWN);
    // exhaustive invariant: exception != 0 => raise == 0; and exactly one outcome per nonzero fpe
    for fpe in 0..0x20u32 { for cw in 0..0x40u32 {
        let d = decide(fpe, cw);
        assert!(!(d.exception != 0 && d.raise != 0));
        if fpe != 0 { assert!((d.exception != 0) ^ (d.raise != 0)); } n+=1;
    }}
    println!("OK {} checks", n);
}
