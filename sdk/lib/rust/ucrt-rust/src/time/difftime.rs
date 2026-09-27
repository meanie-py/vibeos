//! time/difftime.cpp: `_difftime32`, `_difftime64`.
//!
//! ## Details that are silent if wrong
//! * **A negative `a` or `b` is EINVAL and returns `0.0`, not a negative
//!   difference** -- `_VALIDATE_RETURN_NOEXC` never invokes the handler (no
//!   Watson call), it only sets errno and returns the fallback.
//! * The subtraction is done in the (32- or 64-bit) `TimeType` BEFORE the
//!   widening cast to `double` -- faithful to a 32-bit wraparound if the
//!   caller passes deliberately-invalid non-negative values that overflow
//!   `__time32_t`, though that is not reachable through any validated path.
//! * **`(b - a) as f64`, never `.into()`**: i386 is a no-SSE target (see
//!   `strtod.rs`'s own note on this), and `<i32 as Into<f64>>::into` is a
//!   Rust-internal function that returns `f64` by the SysV/SSE ABI
//!   convention -- it fails to even INSTANTIATE there ("this function
//!   definition uses SIMD vector type `f64`..."), whereas the primitive `as`
//!   cast is a plain instruction the backend lowers without an SSE register.
//!   Caught by `cargo rustc --emit=obj` for i686-pc-windows-gnu specifically;
//!   `cargo check` alone does not monomorphise far enough to see it.

use core::ffi::c_int;

const EINVAL: c_int = 22;

extern "C" {
    fn _errno() -> *mut c_int;
}

#[no_mangle]
pub unsafe extern "C" fn _difftime32(b: i32, a: i32) -> f64 {
    if !(a >= 0 && b >= 0) {
        *_errno() = EINVAL;
        return 0.0;
    }
    (b - a) as f64
}

#[no_mangle]
pub unsafe extern "C" fn _difftime64(b: i64, a: i64) -> f64 {
    if !(a >= 0 && b >= 0) {
        *_errno() = EINVAL;
        return 0.0;
    }
    (b - a) as f64
}
