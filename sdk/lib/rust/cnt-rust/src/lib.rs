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

// Core is the single source of truth and stays unsafe-free; the
// forbid is scoped to the module so the generated `cabi` shims (which
// must be `unsafe extern "C"`) are unaffected.
#[forbid(unsafe_code)]
pub mod core_rs;

// The C-ABI shim layer is generated and unsafe.
#[cfg(feature = "cabi")]
pub mod cabi;
