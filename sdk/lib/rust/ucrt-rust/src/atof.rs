//! convert/atof.cpp: `atof`, `_wtof`, `_atoflt`, `_atodbl` and their `_l` forms.
//!
//! Legacy front ends kept "for legacy support" (the C's own comment) over the
//! same engine `strtod.rs` already ports: `crate::strtod::parse_floating_point`,
//! called here exactly as `common_atodbl_l`/`common_atof_l` call
//! `__crt_strtox::parse_floating_point` -- with a null end pointer, since none
//! of these functions report one back.
//!
//! `atof`/`_wtof` discard the parser's status entirely (no error return, no
//! errno on overflow/underflow -- the C's `common_atof_l` throws the `SLD_STATUS`
//! away too). Only `_atoflt`/`_atodbl` translate it, into `_OVERFLOW`(3) /
//! `_UNDERFLOW`(4) (`corecrt_math.h`) -- note these are NOT errno values; they
//! are the function's own return code, and neither case touches errno.
//!
//! ## Details that are silent if wrong
//!
//! * **`_atodbl`/`_atodbl_l` take `char*` (non-const)**, unlike every sibling
//!   here which takes `char const*` -- an inconsistency in the Microsoft header
//!   (`stdlib.h`), kept faithfully since it is part of the exported ABI.
//! * **A null `result` in `_atoflt`/`_atodbl`/`_l` fails with `_DOMAIN`(1)**, not
//!   `_OVERFLOW`/`_UNDERFLOW` and not the `EINVAL`-keyed codes `atof`/`_wtof`
//!   use for a null `string` -- two different validations, two different
//!   returned codes, both errno=EINVAL.
//! * **A null `string` in every one of these calls the invalid-parameter
//!   handler** (`_VALIDATE_RETURN`, not the `_NOEXC` form) and sets
//!   `errno=EINVAL`; `atof`/`_wtof` then return `0.0`, `_atoflt`/`_atodbl`
//!   return `_DOMAIN`.
//! * **`_CRT_FLOAT`/`_CRT_DOUBLE` are single-field structs** (`{ f: f32 }` /
//!   `{ x: f64 }`) that exist only to stop the compiler generating float
//!   instructions when copying/pushing the value -- ordinary `#[repr(C)]` matches
//!   them exactly, no packing.

use core::ffi::c_int;

use crate::locale::CrtLocalePointers;
use crate::locale_update::LocaleUpdate;
use crate::strtod::{parse_floating_point, Status, DOUBLE, FLOAT};

const EINVAL: c_int = 22;
/// `corecrt_math.h`: `_DOMAIN` (1), `_OVERFLOW` (3), `_UNDERFLOW` (4).
const DOMAIN: c_int = 1;
const OVERFLOW_CODE: c_int = 3;
const UNDERFLOW_CODE: c_int = 4;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
}

#[repr(C)]
pub struct CrtFloat {
    pub f: f32,
}

#[repr(C)]
pub struct CrtDouble {
    pub x: f64,
}

/// `_VALIDATE_RETURN(expr, errorcode, retexpr)`: on failure, set errno, invoke
/// the (release-build, no-info) handler, and hand back `ret`.
unsafe fn validate<T>(ok: bool, errorcode: c_int, ret: T) -> Result<(), T> {
    if ok {
        Ok(())
    } else {
        *_errno() = errorcode;
        _invalid_parameter_noinfo();
        Err(ret)
    }
}

/// `common_atodbl_l<FloatingType, Character>`. Returns the parsed value's raw
/// bits (low 32 for `FLOAT`, all 64 for `DOUBLE` -- `parse_floating_point`
/// already writes only as many bits as `t` describes) and the C's `int`
/// return code, instead of writing through a `result` pointer sized for the
/// wrong output width: `_atoflt`'s `_CRT_FLOAT` is 4 bytes and `_atodbl`'s
/// `_CRT_DOUBLE` is 8, so one write-through-pointer helper cannot serve both
/// without either truncating or over-writing.
///
/// # Safety
/// `string`, if non-null, must be a NUL-terminated `C`-string; `locale`, if
/// non-null, must be a valid `_locale_t`.
unsafe fn common_atodbl_l<C: crate::strtox::Char>(
    have_result: bool,
    zero: C,
    t: &crate::strtod::Traits,
    string: *const C,
    locale: *mut CrtLocalePointers,
) -> (u64, c_int) {
    if let Err(ret) = validate(have_result, EINVAL, DOMAIN) {
        return (0, ret);
    }

    let mut locale_update = LocaleUpdate::new(locale);
    let mut out = 0u64;
    let status = parse_floating_point(locale_update.locale_t(), string, core::ptr::null_mut(), zero, t, &mut out);

    let code = match status {
        Status::Overflow => OVERFLOW_CODE,
        Status::Underflow => UNDERFLOW_CODE,
        _ => 0,
    };
    (out, code)
}

#[no_mangle]
pub unsafe extern "C" fn _atoflt_l(result: *mut CrtFloat, string: *const u8, locale: *mut CrtLocalePointers) -> c_int {
    let (bits, status) = common_atodbl_l::<u8>(!result.is_null(), 0, &FLOAT, string, locale);
    if !result.is_null() {
        (*result).f = core::mem::transmute::<u32, f32>(bits as u32);
    }
    status
}

#[no_mangle]
pub unsafe extern "C" fn _atoflt(result: *mut CrtFloat, string: *const u8) -> c_int {
    _atoflt_l(result, string, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _atodbl_l(result: *mut CrtDouble, string: *mut u8, locale: *mut CrtLocalePointers) -> c_int {
    let (bits, status) = common_atodbl_l::<u8>(!result.is_null(), 0, &DOUBLE, string as *const u8, locale);
    if !result.is_null() {
        (*result).x = core::mem::transmute::<u64, f64>(bits);
    }
    status
}

#[no_mangle]
pub unsafe extern "C" fn _atodbl(result: *mut CrtDouble, string: *mut u8) -> c_int {
    _atodbl_l(result, string, core::ptr::null_mut())
}

/// `common_atof_l<Character>`. Discards the parse status exactly as the C does
/// (the double-only path never reports `_OVERFLOW`/`_UNDERFLOW`, and never
/// touches errno on a successful validate).
unsafe fn common_atof_l<C: crate::strtox::Char>(string: *const C, zero: C, locale: *mut CrtLocalePointers) -> u64 {
    if validate(!string.is_null(), EINVAL, 0.0f64).is_err() {
        return 0;
    }
    let mut locale_update = LocaleUpdate::new(locale);
    let mut out = 0u64;
    parse_floating_point(locale_update.locale_t(), string, core::ptr::null_mut(), zero, &DOUBLE, &mut out);
    out
}

#[no_mangle]
pub unsafe extern "C" fn _atof_l(string: *const u8, locale: *mut CrtLocalePointers) -> f64 {
    core::mem::transmute::<u64, f64>(common_atof_l::<u8>(string, 0, locale))
}

#[no_mangle]
pub unsafe extern "C" fn atof(string: *const u8) -> f64 {
    _atof_l(string, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn _wtof_l(string: *const u16, locale: *mut CrtLocalePointers) -> f64 {
    core::mem::transmute::<u64, f64>(common_atof_l::<u16>(string, 0, locale))
}

#[no_mangle]
pub unsafe extern "C" fn _wtof(string: *const u16) -> f64 {
    _wtof_l(string, core::ptr::null_mut())
}
