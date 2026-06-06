//! cnt-rust: Rust kernel/native C-runtime (libcntpr bucket-B).
//!
//! `core_rs` is the single source of truth (idiomatic, slice-based,
//! `no_std`, no `unsafe`). `cabi` is generated — thin `#[no_mangle]`
//! shims that unsafely reconstruct slices from the C ptr+len ABI and
//! collapse the core return into the exact legacy contract.
//!
//! Semantics track ReactOS libcntpr, not textbook libc. See
//! `cnt-manifest.toml` for the per-symbol contract + the preserved
//! misfeatures, and `cnt-difftest` for the differential harness that
//! pins each symbol against the real compiled reference.
#![cfg_attr(not(test), no_std)]

// Core is the single source of truth and stays unsafe-free. `deny`
// (not `forbid`) so the ONE audited exception — `core_rs::floatbits`,
// whose fb64/fb32 `transmute` shims let floatcvt build an `f64` from
// bits on the no-SSE i386 kernel (f64::from_bits has a Rust-ABI f64
// return that needs SSE; transmute is an intrinsic that doesn't) — can
// locally `#[allow(unsafe_code)]`. Every other core_rs module stays
// unsafe-free (the deny still bites them). The `cabi` shims (which must
// be `unsafe extern "C"`) are outside this scope and unaffected.
#[deny(unsafe_code)]
pub mod core_rs;

// The C-ABI shim layer is generated and unsafe.
#[cfg(feature = "cabi")]
pub mod cabi;
