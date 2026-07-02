#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

//! samsrv-sys — SAMPR_* RPC marshalled types + Samr* client-stub FFI substrate
//! for the SAM bridge (lsasrv-rust authpackage + future samsrv-rust).
//!
//! bindgen-generated (generate_bindings.sh) from the widl-generated sam_c.h
//! (build-<arch>/dll/win32/samlib/sam_c.h) in its native rpc.h/windows.h
//! context, allowlisted to the SAMPR_/Samr/SamIFree_ surface. Kept separate from
//! lsasrv-sys to avoid the windows.h/ole2.h vs NDK/ntlsa header-world collision.
//! Per-arch (pointer widths + RPC string layout differ).
//!
//! Regenerate with: bash sdk/lib/rust/samsrv-sys/generate_bindings.sh

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
