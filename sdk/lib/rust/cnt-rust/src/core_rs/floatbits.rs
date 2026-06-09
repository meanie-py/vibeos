//! The ONE audited `unsafe` carve-out in `core_rs` (which is otherwise
//! `#[deny(unsafe_code)]`, the Kani-able single source of truth).
//!
//! On the i386 no-SSE kernel build (CR4.OSFXSR unset in Phase 0) rustc
//! HARD-ERRORS any fn whose Rust-ABI signature returns `f64`/`f32` by
//! value (it would use xmm0). `f{64,32}::from_bits` is exactly such a
//! fn, so a bit→float reinterpret in `floatcvt` can't go through it.
//! `core::mem::transmute` is a compiler intrinsic — no emitted function,
//! no ABI check — so it is the one no-SSE-safe way to do the
//! reinterpret. These shims are `extern "C"` (x87 ST(0) return, never
//! xmm) and are pure bitcasts: bit-identical to `from_bits`, which
//! `cnt-difftest` pins. This mirrors the identical carve-out in
//! `cabi/math.rs` (which is in `cabi`, where `unsafe` is already
//! allowed) — `core_rs` needs its own because it can't depend on
//! `cabi`, and *calling* an `extern "C"` shim from another crate would
//! itself need `unsafe` (defeating the purpose).
#![allow(unsafe_code)]

/// Bits → `f64`, no-SSE-safe (see module docs). Pure bitcast.
#[allow(unnecessary_transmutes)]
#[inline]
pub(crate) extern "C" fn fb64(b: u64) -> f64 {
    unsafe { core::mem::transmute(b) }
}

/// Bits → `f32`, no-SSE twin of [`fb64`].
#[allow(unnecessary_transmutes)]
#[inline]
pub(crate) extern "C" fn fb32(b: u32) -> f32 {
    unsafe { core::mem::transmute(b) }
}

/// `|x|` without SSE: clear the sign bit via [`fb64`]. Bit-identical to
/// `f64::abs` for ALL inputs (incl. ±0/±inf/NaN — masking only the sign
/// bit). `extern "C"` (x87 ST(0) return) so its own f64 return is also
/// no-SSE-safe, just like [`fb64`].
#[inline]
pub(crate) extern "C" fn fabs64(x: f64) -> f64 {
    fb64(x.to_bits() & 0x7FFF_FFFF_FFFF_FFFF)
}
