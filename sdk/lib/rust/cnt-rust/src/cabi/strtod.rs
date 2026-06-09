//! cabi shims for `strtod`/`atof` — the C ptr+sret ABI over the pure
//! `core_rs::strtod` parser. `extern "C"` `f64` return is the x87 ST(0)
//! (no-SSE-safe on i386); the decimal arithmetic is all in core_rs.
//!
//! Stitched into `cabi` like `streamout.rs`/`math.rs` (not a gen_cnt
//! family — bespoke pointer/sret handling). See core_rs/strtod.rs for
//! the documented i386 f80→f64 deviation.

use super::cstr_slice;
use crate::core_rs::strtod::strtod_bytes;

/// `double strtod(const char *s, char **sret)` — faithful to
/// `strtod.c`. NULL `s` returns 0.0 and leaves `*sret` untouched (the
/// C's `if (s == NULL) return r;` early-out). Otherwise `*sret` (when
/// non-NULL) is set to the first unconsumed byte.
#[no_mangle]
pub unsafe extern "C" fn strtod(s: *const u8, sret: *mut *mut u8) -> f64 {
    if s.is_null() {
        return 0.0;
    }
    let (val, consumed) = strtod_bytes(cstr_slice(s));
    if !sret.is_null() {
        *sret = (s as *mut u8).add(consumed);
    }
    val
}

/// `double atof(const char *ascii)` — the genuine `strtod(ascii, 0)`.
#[no_mangle]
pub unsafe extern "C" fn atof(s: *const u8) -> f64 {
    if s.is_null() {
        return 0.0;
    }
    strtod_bytes(cstr_slice(s)).0
}
