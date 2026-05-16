//! rtl-strings: Rust translation of ReactOS `sdk/lib/rtl/unicode.c`
//! — the NT Rtl* counted-string surface (UNICODE_STRING / STRING /
//! ANSI_STRING / OEM).
//!
//! `core_rs` is the single source of truth (idiomatic, slice-based,
//! `no_std`, no `unsafe`). `cabi` is the thin `#[no_mangle] extern
//! "system"` NTAPI layer that reconstructs the counted-string structs
//! from the raw ABI and collapses the core result into the exact
//! legacy `Rtl*` contract (incl. the faithful misfeatures).
//!
//! Semantics track ReactOS rtl, not textbook — see `rtl-difftest` for
//! the differential harness that pins each symbol against the real
//! compiled `unicode.c`.
#![cfg_attr(not(test), no_std)]

#[forbid(unsafe_code)]
pub mod core_rs;

#[cfg(feature = "cabi")]
pub mod cabi;
