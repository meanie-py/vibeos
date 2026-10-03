//! The floating-point formatting engine, and the two stragglers that shared its
//! directory. Replaces, each file in full:
//!
//! | C file (`sdk/lib/ucrt/convert/`) | here           | defines                                   |
//! |----------------------------------|----------------|-------------------------------------------|
//! | `cfout.cpp`                      | `cfout.rs`     | `__acrt_fltout` (binary -> decimal digits) |
//! | `_fptostr.cpp`                   | `fptostr.rs`   | `__acrt_fp_strflt_to_string` (rounding)   |
//! | `cvt.cpp`                        | `format.rs`    | `__acrt_fp_format` (`%a %e %f %g`)         |
//! | `fp_flags.cpp`                   | `fp_flags.rs`  | `__fastflag`, `__setfflag`                |
//! | `wctype.cpp`                     | `wctype.rs`    | `wctype` (export)                         |
//!
//! The three engine entry points are internals: nothing exports them, and their
//! callers are Rust already -- the printf engine (`output.rs`, `__acrt_fp_format`)
//! and `_ecvt`/`_fcvt`/`_gcvt` (`convert_rest/`, `__acrt_fltout` +
//! `__acrt_fp_strflt_to_string`). They keep their C names and are
//! `#[no_mangle] extern "C"` so those callers' existing declarations bind here.
//!
//! The digit generator is Burger & Dybvig over the crate's `big_integer.rs`,
//! which is the same header (`corecrt_internal_big_integer.h`) cfout.cpp
//! includes; nothing about it is re-derived.
//!
//! ## Details that are silent if wrong
//!
//! * **`__acrt_fltout` generates precision + 1 digits and never rounds** -- the
//!   rounding is `__acrt_fp_strflt_to_string`'s, told by the `trailing` flag
//!   whether anything nonzero lay beyond the generated digits. Rounding a
//!   half-way `5` to even is only correct when NOTHING follows it, and the flag
//!   has to count nonzero digits that were generated but did not fit in the
//!   buffer (`unwritten_nonzero_digits_in_chunk`), not only the remainder.
//! * **Legacy rounding is half-up on the digit string**; standard rounding
//!   consults `fegetround()` every call. `%a` rounds on the hex digit: to-nearest
//!   ties to even on the NEXT HIGHER hex digit (or the implicit bit at the top),
//!   and upward/downward look only at the next digit, not the bits below it.
//! * **INF/NAN spellings**: `inf`, `nan`, `nan(snan)`, `nan(ind)`, upper-cased
//!   for `%A %E %F %G`, and the short `nan`/`NAN` when the long one does not fit.
//!   Only under `_CRT_INTERNAL_PRINTF_LEGACY_MSVCRT_COMPATIBILITY` do they reach
//!   the digit engine, which spells them `1#INF`, `1#QNAN`, `1#SNAN`, `1#IND`.
//!   "Indeterminate" is exactly sign set + quiet bit only (the default NaN x86
//!   produces); any payload makes it a plain quiet NaN.
//! * **Exponent digits**: two by default, three under
//!   `LEGACY_THREE_DIGIT_EXPONENTS` -- the C always writes three and then
//!   squeezes out a leading zero. `%a` writes the minimum (`p+0`).
//! * **Zero includes flushed denormals**: a denormal is formatted as zero when
//!   `_controlfp`'s `_DN_FLUSH` is set.
//! * **Digit generation runs with every FP exception masked** and restores both
//!   the control word and (when anything was unmasked) the full environment.
//! * **`__acrt_fltout` takes a `double`, not the C's `_CRT_DOUBLE` struct.** At
//!   the C ABI these differ on x64 only (the one-member struct goes in RCX, a
//!   double in XMM0). Every caller left is Rust and already declares `value:
//!   f64` (`convert_rest/fcvt.rs`, `gcvt.rs`) -- which, against the C, read
//!   garbage on amd64. Defining it the way they call it fixes that; no C caller
//!   remains once cvt.cpp is gone.
//! * **No double ever passes through the x87 stack on i386.** A no-SSE build
//!   moves an `f64` value with `fld`/`fstp`, and `fld` quiets a signaling NaN
//!   (legacy mode would print `1#QNAN` for `1#SNAN`). Everything here works on
//!   the bits; `__acrt_fltout` takes its i386 argument as a `u64` (the same
//!   stack slot). Its i386 CALLERS that pass an `f64` (fcvt.rs/gcvt.rs) still
//!   quiet it on their side. A deliberate divergence from ReactOS's OWN i386
//!   build of the C, whose cfout.cpp/cvt.cpp copy the double with
//!   `fldl`/`fstpl` and so print a legacy-mode sNaN as `1#QNAN`: the C's table
//!   says `1#SNAN`, its 64-bit builds print that, and so does an SSE2 i386
//!   ucrtbase. (Host check, C compiled -m32: every other case identical.)

// `transmute`, not `f64::from_bits`/`to_bits`: those are Rust-ABI functions
// passing a float, which the no-SSE i386 cell refuses (strtod.rs, same idiom).
#![allow(unnecessary_transmutes)]

mod cfout;
mod format;
mod fp_flags;
mod fptostr;
mod wctype;

pub use cfout::__acrt_fltout;
pub use format::__acrt_fp_format;
pub use fp_flags::{__fastflag, __setfflag};
pub use fptostr::__acrt_fp_strflt_to_string;
pub use wctype::wctype;

use crate::ptd_host::PtdHost;
use core::ffi::{c_char, c_int};

/// `_strflt` / `STRFLT` (corecrt_internal_fltintrn.h:162). `sign` holds the
/// CHARACTER `'-'` or `' '` (the header's "zero if positive" is wrong).
#[repr(C)]
pub struct Strflt {
    pub sign: c_int,
    pub decpt: c_int,
    pub mantissa: *mut c_char,
}

const EINVAL: c_int = 22;
const ERANGE: c_int = 34;
const ENOMEM: c_int = 12;

/// `__acrt_has_trailing_digits`.
const TRAILING: c_int = 0;
const NO_TRAILING: c_int = 1;

/// `__acrt_precision_style`.
const PRECISION_STYLE_FIXED: c_int = 0;
const PRECISION_STYLE_SCIENTIFIC: c_int = 1;

/// `__acrt_rounding_mode`.
const ROUNDING_LEGACY: c_int = 0;

/// fenv.h `FE_*` rounding modes (`_RC_*`).
const CVT_FE_TONEAREST: c_int = 0x0000;
const CVT_FE_UPWARD: c_int = 0x0200;
const CVT_FE_DOWNWARD: c_int = 0x0100;

/// `__acrt_fp_class`.
const FP_FINITE: u32 = 0;
const FP_INFINITY: u32 = 1;
const FP_QUIET_NAN: u32 = 2;
const FP_SIGNALING_NAN: u32 = 3;
const FP_INDETERMINATE: u32 = 4;

/// `__acrt_floating_type_traits<double>`.
const DBL_MANTISSA_BITS: u32 = 53;
const DBL_EXPONENT_BIAS: i32 = 1023;
const DBL_EXPONENT_MASK: u64 = 0x7ff;
const DBL_FRACTION_MASK: u64 = (1u64 << 52) - 1;
const DBL_SPECIAL_NAN_MANTISSA_MASK: u64 = 1u64 << 51;

extern "C" {
    fn strlen(s: *const core::ffi::c_char) -> usize;
    fn strcpy_s(dst: *mut u8, destsz: usize, src: *const u8) -> c_int;
    fn fegetround() -> c_int;
    fn _invoke_watson(e: *const u16, f: *const u16, file: *const u16, line: u32, r: usize) -> !;
}

/// The `_components` bit-fields of a double.
#[derive(Clone, Copy)]
struct Components {
    fraction: u64,
    exponent: u64,
    sign: u64,
}

impl Components {
    fn of(bits: u64) -> Self {
        Self { fraction: bits & DBL_FRACTION_MASK, exponent: (bits >> 52) & DBL_EXPONENT_MASK, sign: bits >> 63 }
    }
}

/// `__acrt_fp_classify`.
fn classify(bits: u64) -> u32 {
    let c = Components::of(bits);
    if c.exponent != DBL_EXPONENT_MASK {
        FP_FINITE
    } else if c.fraction == 0 {
        FP_INFINITY
    } else if c.sign == 1 && c.fraction == DBL_SPECIAL_NAN_MANTISSA_MASK {
        FP_INDETERMINATE
    } else if c.fraction & DBL_SPECIAL_NAN_MANTISSA_MASK != 0 {
        FP_QUIET_NAN
    } else {
        FP_SIGNALING_NAN
    }
}

/// `_ERRCHECK(e)`.
#[inline]
unsafe fn errcheck(e: c_int) {
    if e != 0 {
        _invoke_watson(core::ptr::null(), core::ptr::null(), core::ptr::null(), 0, 0);
    }
}

/// `_UCRT_VALIDATE_RETURN_ERRCODE(ptd, expr, err)`'s failure arm.
#[inline]
unsafe fn validate(ptd: &mut PtdHost, expr: bool, err: c_int) -> Option<c_int> {
    if expr {
        None
    } else {
        ptd.invalid_parameter(err);
        Some(err)
    }
}
