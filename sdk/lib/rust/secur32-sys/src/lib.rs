#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]
#![allow(clippy::all)]

//! secur32-sys — SSPI / security-provider / LSA-LPC / NTLM-MSV / wincred type
//! substrate for the secur32.dll C-to-Rust translation (secur32-rust).
//!
//! bindgen-generated (generate_bindings.sh) from the SSPI headers in usermode
//! win32+NDK context, allowlisted to the secur32-specific surface (SecBuffer/
//! SecBufferDesc/SecPkgInfo, CredHandle/CtxtHandle/SecHandle, SecurityFunction-
//! Table*, SecPkgContext_*, MSV1_0_*/KERB_*, CREDENTIAL_*, LSA-LPC port messages,
//! …) absent from ntdll-sys. Base types it re-bindgens (HANDLE/ULONG/LUID/…) are
//! NOT re-exported by secur32-rust's prelude (curated `pub use`), so they don't
//! collide with ntdll_sys::nt. Per-arch. Regenerate: bash generate_bindings.sh

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
