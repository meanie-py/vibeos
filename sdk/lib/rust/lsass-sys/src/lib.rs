#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

//! lsass-sys — focused LSA LPC-message DDK FFI substrate for lsalib-rust.
//!
//! bindgen-generated (generate_bindings.sh) from subsys/lsass/lsass.h in NDK
//! context, allowlisted to the LSA_/LSASS_ surface (LSA_API_MSG, the per-request
//! message structs, LSA_API_NUMBER, LSA_CONNECTION_INFO). These live in the LSASS
//! subsystem header, NOT ntoskrnl.h, so they are absent from ntkrnl-sys. Per-arch.
//!
//! Regenerate with: bash sdk/lib/rust/lsass-sys/generate_bindings.sh

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
