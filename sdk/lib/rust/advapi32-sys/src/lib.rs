#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused_imports)]
#![allow(clippy::all)]

//! advapi32-sys — Win32 SDK / RPC-client / winsafer / aclapi / evntrace / EFS
//! type substrate for the advapi32.dll C-to-Rust translation (advapi32-rust).
//!
//! bindgen-generated (generate_bindings.sh) from the module's advapi32.h in
//! usermode win32+NDK context, allowlisted to the advapi32-specific surface
//! (TRUSTEE/EXPLICIT_ACCESS, SAFER_*, EVENT_TRACE_*, RPC-client types,
//! ENCRYPTION_CERTIFICATE_*, SC_/SERVICE_, HKEY/REG_*, …) absent from
//! ntdll-sys. Base types it re-bindgens (HANDLE/ULONG/SID/…) are NOT re-exported
//! by advapi32-rust's prelude (curated `pub use`), so they don't collide with
//! ntdll_sys::nt. Per-arch. Regenerate: bash advapi32-sys/generate_bindings.sh

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
