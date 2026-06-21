#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

//! lsasrv-sys — LSAPR_* RPC marshalled types + DSROLER/policy info FFI substrate
//! for lsasrv-rust.
//!
//! bindgen-generated (generate_bindings.sh) from the lsasrv module's generated
//! lsa_s.h/dssetup_s.h + ntlsa/ntsam in usermode win32+NDK context, allowlisted
//! to the LSAPR_/DSROLE/POLICY_/LSA_ surface the Lsar* server methods + the
//! dssetup RPC take. These usermode RPC marshalled types are absent from
//! ntkrnl-sys. Per-arch (pointer widths + RPC string layout differ).
//!
//! Regenerate with: bash sdk/lib/rust/lsasrv-sys/generate_bindings.sh

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;
#[cfg(target_arch = "aarch64")]
pub use bindgen_arm64::*;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;
#[cfg(target_arch = "x86_64")]
pub use bindgen_amd64::*;

#[cfg(target_arch = "x86")]
mod bindgen_i386;
#[cfg(target_arch = "x86")]
pub use bindgen_i386::*;
