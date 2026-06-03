//! winlogon-sys — FFI substrate for the winlogon.exe C→Rust leaf-file
//! translation (security/environment/shutdown/setup/screensaver).
//!
//! Auto-generated via `bindgen` for all 3 architectures from a wrapper
//! that pulls the Win32 GUI/registry/security API surface + the GINA
//! interface (`winwlx.h`) + the NDK user-mode native API, and that
//! copies winlogon's OWN private structs (WLSESSION, GINAINSTANCE,
//! GINAFUNCTIONS, LOGON_STATE, NOTIFICATION_TYPE) verbatim from
//! `base/system/winlogon/winlogon.h`.
//!
//! WLSESSION is the crux: the still-C winlogon core owns the `WLSession`
//! global, so every flipped leaf reads it through this crate's layout.
//! The `size_check_*.rs` compile-time asserts (verify_sizes.py) fail the
//! build if any struct ever drifts from the C layout.
//!
//! User-mode analog of `ntdll-sys`/`win32k-sys`. NOT a standalone static
//! library — an `rlib` declaration dependency. See
//! `docs/engineering/smss-csrss-translation-audit.md` (winlogon track)
//! and memory `project_winlogon_track`.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
#![allow(clashing_extern_declarations)]

mod bindgen_common;

#[cfg(target_arch = "aarch64")]
mod bindgen_arm64;

#[cfg(target_arch = "x86_64")]
mod bindgen_amd64;

#[cfg(target_arch = "x86")]
mod bindgen_i386;

// Compile-time struct-size assertions (verify_sizes.py) — fail the
// build if a bindgen/copied struct ever drifts from the C layout.
#[cfg(target_arch = "aarch64")]
mod size_check_arm64;

#[cfg(target_arch = "x86_64")]
mod size_check_amd64;

#[cfg(target_arch = "x86")]
mod size_check_i386;

/// The winlogon FFI surface — Win32 + GINA + NDK + winlogon-private
/// structs, with correct per-arch calling conventions (i386 stdcall
/// decoration too). Named `nt` for parity with the sibling sys crates
/// (ntdll-sys/win32k-sys) and the verify_sizes.py generator. Flipped
/// leaves `use winlogon_sys::nt::*`.
pub mod nt {
    pub use super::bindgen_common::*;

    #[cfg(target_arch = "aarch64")]
    pub use super::bindgen_arm64::*;

    #[cfg(target_arch = "x86_64")]
    pub use super::bindgen_amd64::*;

    #[cfg(target_arch = "x86")]
    pub use super::bindgen_i386::*;
}
